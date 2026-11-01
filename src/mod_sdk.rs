//! Optional external gameplay. Persistent bytes belong to the world/entity;
//! executors are disposable caches and are never saved or checksummed.
use crate::rules::ini_parser::IniFile;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};
use vera_mod_api::{self as api, Context, Plan, Request};
use vera_mod_runtime::{ComponentRuntime, HostServices, LoadedModule, RuntimeLimits};

pub use api::{Meter, Projection, Tint};
#[path = "mod_sdk/acceptance.rs"]
pub mod acceptance;

#[derive(Serialize, Deserialize)]
pub struct Program {
    bytes: Vec<u8>,
    digest: String,
    manifest: api::ModuleManifest,
    config: Vec<u8>,
    #[serde(skip)]
    compiled: OnceLock<Result<Arc<Executor>, String>>,
}

struct Executor {
    runtime: ComponentRuntime,
    module: LoadedModule,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("digest", &self.digest)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            digest: self.digest.clone(),
            manifest: self.manifest.clone(),
            config: self.config.clone(),
            compiled: self.compiled.clone(),
        }
    }
}

impl Program {
    pub fn load(bytes: Vec<u8>, expected: &str, passes: &[IniFile]) -> Result<Self, String> {
        Self::load_with_types(bytes, expected, passes, &[], &[], &[])
    }

    pub fn load_with_types(
        bytes: Vec<u8>,
        expected: &str,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
    ) -> Result<Self, String> {
        let runtime = ComponentRuntime::new(RuntimeLimits::default()).map_err(|e| e.to_string())?;
        let module = runtime.load(&bytes, expected).map_err(|e| e.to_string())?;
        let manifest = module.manifest().clone();
        if [
            api::HookKind::Configure,
            api::HookKind::Attach,
            api::HookKind::Damage,
            api::HookKind::Tick,
        ]
        .iter()
        .any(|kind| !manifest.hooks.contains(kind))
        {
            return Err("engine SDK v1 requires configure, attach, damage and tick hooks".into());
        }
        let mut program = Self {
            bytes,
            digest: expected.into(),
            manifest,
            config: Vec::new(),
            compiled: OnceLock::new(),
        };
        let executor = Arc::new(Executor { runtime, module });
        for (index, ini) in passes.iter().enumerate() {
            let final_pass = index + 1 == passes.len();
            let request = Request {
                api_version: api::API_VERSION,
                context: Context::Configure(api::ConfigureContext {
                    pass: index as u64,
                    source: format!("rules-pass-{index}"),
                    final_pass,
                    object_types: if final_pass {
                        object_types.to_vec()
                    } else {
                        Vec::new()
                    },
                    warheads: if final_pass {
                        warheads.to_vec()
                    } else {
                        Vec::new()
                    },
                }),
                config: program.config.clone(),
                state: Vec::new(),
            };
            let plan = executor
                .runtime
                .evaluate(
                    &executor.module,
                    &request,
                    Arc::new(NativeHost {
                        ini: Some(Arc::new(ini.clone())),
                        weapons: if final_pass {
                            weapons.to_vec()
                        } else {
                            Vec::new()
                        },
                    }),
                )
                .map_err(|e| e.to_string())?;
            if !plan.state.is_empty() || plan.damage.is_some() || !plan.operations.is_empty() {
                return Err("configuration returned simulation writes".into());
            }
            if let Some(config) = plan.config {
                program.config = config;
            }
        }
        program
            .compiled
            .set(Ok(executor))
            .map_err(|_| "component cache already set".to_string())?;
        Ok(program)
    }

    pub fn sha256(&self) -> &str {
        &self.digest
    }
    pub fn manifest(&self) -> &api::ModuleManifest {
        &self.manifest
    }

    pub(crate) fn weapon_resource_policy(
        &self,
        name: &str,
    ) -> Result<Option<api::WeaponResourcePolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::WeaponResource) {
            return Ok(None);
        }
        let plan = self.evaluate(
            Context::WeaponResource(api::WeaponResourceContext {
                weapon_name: name.to_owned(),
            }),
            &[],
        )?;
        Ok(plan.weapon_resource)
    }

    fn executor(&self) -> Result<&Arc<Executor>, String> {
        self.compiled
            .get_or_init(|| {
                let runtime =
                    ComponentRuntime::new(RuntimeLimits::default()).map_err(|e| e.to_string())?;
                let module = runtime
                    .load(&self.bytes, &self.digest)
                    .map_err(|e| e.to_string())?;
                if module.manifest() != &self.manifest {
                    return Err("saved module manifest mismatch".into());
                }
                Ok(Arc::new(Executor { runtime, module }))
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(crate) fn evaluate(&self, context: Context, state: &[u8]) -> Result<Plan, String> {
        let executor = self.executor()?;
        let request = Request {
            api_version: api::API_VERSION,
            context,
            config: self.config.clone(),
            state: state.to_vec(),
        };
        let plan = executor
            .runtime
            .evaluate(
                &executor.module,
                &request,
                Arc::new(NativeHost {
                    ini: None,
                    weapons: Vec::new(),
                }),
            )
            .map_err(|e| e.to_string())?;
        if plan.config.is_some() || !plan.operations.is_empty() {
            return Err("unsupported operation or configuration write during simulation".into());
        }
        Ok(plan)
    }

    pub(crate) fn validate_restore(&self) -> Result<(), String> {
        if self.bytes.len() > 16 * 1024 * 1024 || self.config.len() > api::MAX_CONFIG_BYTES {
            return Err("saved component exceeds SDK limits".into());
        }
        self.executor().map(|_| ())
    }

    pub(crate) fn fold(&self, hasher: &mut impl Hasher) {
        self.digest.hash(hasher);
        self.manifest.id.hash(hasher);
        self.manifest.version.hash(hasher);
        self.manifest.state_version.hash(hasher);
        self.manifest.config_version.hash(hasher);
        self.config.hash(hasher);
    }
}

/// Resource-owned declaration; paths remain logical asset names.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    api_version: u32,
    program: String,
    sha256: String,
}

fn asset_program(
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<Option<(String, Vec<u8>)>, String> {
    let Some(declaration) = assets.get_ref("vera-mod.json") else {
        return Ok(None);
    };
    if declaration.len() > 16 * 1024 {
        return Err("mod declaration too large".into());
    }
    let declaration: Declaration =
        serde_json::from_slice(&declaration).map_err(|e| format!("mod declaration: {e}"))?;
    if declaration.api_version != api::API_VERSION
        || !crate::assets::community::valid_package_name(&declaration.program)
    {
        return Err("invalid mod declaration".into());
    }
    let bytes = assets
        .get_ref(&declaration.program)
        .ok_or_else(|| format!("missing gameplay component {}", declaration.program))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("gameplay component too large".into());
    }
    if crate::util::sha256::sha256_hex(&bytes) != declaration.sha256 {
        return Err("component content digest mismatch".into());
    }
    Ok(Some((declaration.sha256, bytes.to_vec())))
}

pub(crate) fn load_from_assets(
    assets: &crate::assets::asset_manager::AssetManager,
    passes: &[IniFile],
    object_types: &[String],
    warheads: &[String],
    weapons: &[String],
) -> Result<Option<Arc<Program>>, String> {
    let Some((digest, bytes)) = asset_program(assets)? else {
        return Ok(None);
    };
    Program::load_with_types(bytes, &digest, passes, object_types, warheads, weapons)
        .map(|p| Some(Arc::new(p)))
}

/// Restore the exact saved configuration after projecting retained Rules.
/// The current work must contain the same portable component; Configure is
/// never replayed against a save and updates require a fresh match.
pub fn adopt_saved_program(
    rules: &mut crate::rules::ruleset::RuleSet,
    sim: &crate::sim::world::Simulation,
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<(), String> {
    match (&sim.external_mod, asset_program(assets)?) {
        (None, None) => {
            rules.external_mod = None;
        }
        (Some(saved), Some((digest, _))) if saved.sha256() == digest => {
            saved.validate_restore()?;
            rules.external_mod = Some(Arc::new(saved.clone()));
        }
        _ => return Err("saved/current work disagree about external gameplay identity".into()),
    }
    rules.bind_weapon_resource_policies()
}

struct NativeHost {
    ini: Option<Arc<IniFile>>,
    weapons: Vec<String>,
}

impl HostServices for NativeHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        let section = self.ini.as_ref()?.section(section)?;
        section
            .is_present(key)
            .then(|| section.read_string(key, "", 4096))
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        let ini = self
            .ini
            .as_ref()
            .ok_or("configuration is unavailable during simulation")?;
        let section = ini.section_or_empty(section);
        let value: serde_json::Value =
            serde_json::from_slice(current).map_err(|e| e.to_string())?;
        let bad = || format!("invalid config reader/default: {reader}");
        let result = match reader {
            "int" => serde_json::json!(
                section.read_int(
                    key,
                    value
                        .as_i64()
                        .and_then(|x| i32::try_from(x).ok())
                        .ok_or_else(bad)?
                )
            ),
            "ares-int" => serde_json::json!(
                section.read_ares_int(
                    key,
                    value
                        .as_i64()
                        .and_then(|x| i32::try_from(x).ok())
                        .ok_or_else(bad)?,
                )
            ),
            "bool" => serde_json::json!(section.read_bool(key, value.as_bool().ok_or_else(bad)?)),
            "double-bits" => serde_json::json!(
                section
                    .read_double(key, f64::from_bits(value.as_u64().ok_or_else(bad)?))
                    .to_bits()
            ),
            "nullable-int" => serde_json::json!(section.read_phobos_nullable_int(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "nullable-bool" => serde_json::json!(section.read_phobos_nullable_bool(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "nullable-double-bits" => serde_json::json!(
                section
                    .read_phobos_nullable_double(
                        key,
                        serde_json::from_value::<Option<u64>>(value)
                            .map_err(|e| e.to_string())?
                            .map(f64::from_bits)
                    )
                    .map(f64::to_bits)
            ),
            "int-list" => section
                .read_int_list(key)
                .map_or(value, |x| serde_json::json!(x)),
            "color-rgb" => {
                let old: [u8; 3] = serde_json::from_value(value).map_err(|e| e.to_string())?;
                serde_json::json!(section.read_color_rgb(key, old, None))
            }
            _ => {
                let (kind, size) = reader.split_once(':').ok_or_else(bad)?;
                let size: usize = size.parse().map_err(|_| bad())?;
                if !(1..=4096).contains(&size) {
                    return Err(bad());
                }
                match kind {
                    "string" => serde_json::json!(section.read_string(
                        key,
                        value.as_str().ok_or_else(bad)?,
                        size
                    )),
                    "name" => section
                        .read_name(key, size)
                        .map_or(value, |x| serde_json::json!(x)),
                    "nullable-name" => serde_json::json!(
                        section.read_phobos_nullable_with(
                            key,
                            serde_json::from_value::<Option<String>>(value)
                                .map_err(|e| e.to_string())?,
                            |raw| { Some(section.read_string(key, raw, size)) }
                        )
                    ),
                    "list" => {
                        if section.is_present(key) {
                            let raw = section.read_string(key, "", size);
                            serde_json::json!(
                                crate::rules::ini_value::strtok(&raw, &[',', ' ', '\t'])
                                    .map(str::to_owned)
                                    .collect::<Vec<_>>()
                            )
                        } else {
                            value
                        }
                    }
                    _ => return Err(bad()),
                }
            }
        };
        serde_json::to_vec(&result).map_err(|e| e.to_string())
    }

    fn registry(&self, name: &str) -> Vec<String> {
        if name == "$weapons" {
            return self.weapons.clone();
        }
        self.ini
            .as_ref()
            .and_then(|ini| ini.section(name))
            .map(|s| s.keys().map(str::to_owned).collect())
            .unwrap_or_default()
    }

    fn damage_kernel(&self, input: &api::DamageKernelInput) -> Result<i32, String> {
        Ok(crate::sim::combat::damage::kernel::apply_warhead_damage(
            input.damage,
            f64::from_bits(input.cell_spread_bits),
            f64::from_bits(input.percent_at_max_bits),
            crate::util::native_x87::NativeF64Bits::from_bits(input.multiplier_bits),
            input.distance_leptons,
            input.no_damage,
            input.max_damage,
        ))
    }
}

pub(crate) fn weapon_user(category: crate::map::entities::EntityCategory) -> u8 {
    use crate::map::entities::EntityCategory;
    match category {
        EntityCategory::Unit => api::WEAPON_USER_UNIT,
        EntityCategory::Infantry => api::WEAPON_USER_INFANTRY,
        EntityCategory::Structure => api::WEAPON_USER_BUILDING,
        EntityCategory::Aircraft => api::WEAPON_USER_AIRCRAFT,
    }
}

fn entity_context(
    entity: &crate::sim::game_entity::GameEntity,
    rules: &crate::rules::ruleset::RuleSet,
    interner: &crate::sim::intern::StringInterner,
) -> api::EntityContext {
    let object = interner.object_type(entity.type_ref(), rules);
    api::EntityContext {
        id: entity.stable_id(),
        type_name: interner.resolve(entity.type_ref()).into(),
        owner: interner.resolve(entity.owner()).into(),
        armor: object.map(|o| o.armor.clone()).unwrap_or_default(),
        health: entity.health.current(),
        max_health: object.map_or(0, |o| o.strength),
        alive: entity.lifecycle.object_alive(),
        in_limbo: entity.lifecycle.in_limbo(),
        immobilized: entity.chrono_warp_latch() || entity.ai_frozen(),
        temporal: entity.temporal.is_warped(),
        being_warped_out: entity.is_warped_out(),
        powered: !entity.deactivated.is_offline(),
        chrono_warp_latch: entity.chrono_warp_latch(),
        ai_frozen: entity.ai_frozen(),
        deactivated: entity.deactivated.is_offline(),
        immune: object.is_some_and(|o| o.immune),
        type_immune: object.is_some_and(|o| o.type_immune),
        invulnerable: false,
    }
}

fn live_entity_context(
    sim: &crate::sim::world::Simulation,
    id: u64,
    rules: &crate::rules::ruleset::RuleSet,
) -> Option<api::EntityContext> {
    let entity = sim.substrate.entities.get(id)?;
    let mut context = entity_context(entity, rules, &sim.interner);
    context.powered = crate::sim::power_system::is_building_powered(
        sim.power_states.as_map(),
        rules,
        entity,
        &sim.interner,
    );
    context.invulnerable = entity.invulnerability.as_ref().is_some_and(|state| {
        crate::sim::superweapon::invulnerability::is_invulnerable(
            Some(state),
            sim.session.binary_frame(),
        )
    });
    Some(context)
}

fn commit_entity(entity: &mut crate::sim::game_entity::GameEntity, plan: Plan) {
    entity.mod_component = plan.state;
    if let Some(projection) = plan.projection {
        entity.mod_presentation = projection;
    }
}

/// Constructor extension, before store admission. No module is a zero-work path.
pub(crate) fn attach_entity(
    entity: &mut crate::sim::game_entity::GameEntity,
    rules: &crate::rules::ruleset::RuleSet,
    interner: &crate::sim::intern::StringInterner,
    frame: i32,
) -> Result<(), String> {
    let Some(program) = rules.external_mod.as_deref() else {
        return Ok(());
    };
    let context = api::AttachContext {
        frame,
        entity: entity_context(entity, rules, interner),
    };
    let plan = program.evaluate(Context::Attach(context), &[])?;
    commit_entity(entity, plan);
    Ok(())
}

/// The engine supplies ordinary combat facts. Extension-specific tags and rules
/// are interpreted exclusively by the external component.
fn damage_context(
    sim: &crate::sim::world::Simulation,
    rules: &crate::rules::ruleset::RuleSet,
    event: &crate::sim::combat::EntityDamageEvent,
    input: i32,
    divisors: &crate::sim::combat::damage::DefenceDivisors,
) -> Option<api::DamageContext> {
    let entity = live_entity_context(sim, event.target_id, rules)?;
    let target = sim.substrate.entities.get(event.target_id)?;
    let warhead = rules.warhead(sim.interner.resolve(event.warhead_ref))?;
    let attacker = sim.substrate.entities.get(event.attacker_id);
    let source = event.source_house.or_else(|| attacker.map(|a| a.owner()));
    let source_is_owner = source.is_some_and(|s| s == target.owner());
    let source_is_ally = source.is_some_and(|s| {
        crate::map::houses::is_allied_with(
            sim.house_alliances(),
            sim.interner.resolve(s),
            sim.interner.resolve(target.owner()),
        )
    });
    let neutral = sim
        .houses
        .get(&target.owner())
        .is_some_and(|h| h.multiplay_passive());
    let mut names = crate::rules::armor::RETAIL_ARMOR_NAMES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    for object in rules.all_objects() {
        if !names.iter().any(|n| n.eq_ignore_ascii_case(&object.armor)) {
            names.push(object.armor.clone());
        }
    }
    let verses = names
        .into_iter()
        .map(|armor| {
            let id = rules
                .all_objects()
                .find(|o| o.armor.eq_ignore_ascii_case(&armor))
                .map_or_else(
                    || crate::rules::armor::armor_index(&armor),
                    |o| o.armor_id(),
                );
            api::ArmorVerses {
                armor,
                multiplier_bits: warhead.interaction(id).multiplier().bits(),
            }
        })
        .collect();
    let armor_multiplier = f64::from(f32::from_bits(divisors.house_type_armor.bits()))
        * f64::from_bits(divisors.unit_armor.bits())
        * divisors
            .rank_armor
            .map_or(1.0, |v| f64::from_bits(v.bits()));
    Some(api::DamageContext {
        frame: sim.session.binary_frame() as i32,
        entity,
        attacker: attacker.map(|a| a.stable_id()),
        source_owner: source.map(|s| sim.interner.resolve(s).into()),
        source_is_ally,
        source_is_owner,
        target_is_neutral: neutral,
        attacker_same_type_owner: attacker
            .is_some_and(|a| a.type_ref() == target.type_ref() && a.owner() == target.owner()),
        warhead_name: sim.interner.resolve(event.warhead_ref).into(),
        warhead_temporal: warhead.temporal,
        warhead_psychedelic: warhead.psychedelic,
        incoming: input,
        distance_leptons: event.distance_leptons,
        ignore_defenses: event.receiver_flags.ignore_defenses,
        no_damage: sim.session.no_damage(),
        max_damage: rules.combat_damage.max_damage,
        armor_multiplier_bits: armor_multiplier.to_bits(),
        verses,
        cell_spread_bits: warhead.cell_spread_f64.to_bits(),
        percent_at_max_bits: warhead.percent_at_max_f64.to_bits(),
    })
}

pub(crate) struct DamageReceipt {
    pub(crate) remaining: i32,
}

pub(crate) fn pre_receive_damage(
    sim: &mut crate::sim::world::Simulation,
    rules: &crate::rules::ruleset::RuleSet,
    event: &crate::sim::combat::EntityDamageEvent,
    input: i32,
    divisors: &crate::sim::combat::damage::DefenceDivisors,
) -> Option<DamageReceipt> {
    let program = sim.external_mod.as_ref()?;
    let entity = sim.substrate.entities.get(event.target_id)?;
    if entity.mod_component.is_empty() {
        return None;
    }
    let context = damage_context(sim, rules, event, input, divisors)?;
    let plan = match program.evaluate(Context::Damage(context), &entity.mod_component) {
        Ok(plan) => plan,
        Err(error) => {
            sim.external_mod_fault = Some(format!("entity {} damage: {error}", event.target_id));
            return None;
        }
    };
    let damage = plan.damage;
    if let Some(entity) = sim.substrate.entities.get_mut(event.target_id) {
        commit_entity(entity, plan);
    }
    damage.map(|remaining| DamageReceipt { remaining })
}

pub(crate) fn preview_receive_damage(
    sim: &crate::sim::world::Simulation,
    rules: &crate::rules::ruleset::RuleSet,
    event: &crate::sim::combat::EntityDamageEvent,
    input: i32,
) -> Result<Option<i32>, String> {
    let Some(program) = sim.external_mod.as_ref() else {
        return Ok(None);
    };
    let Some(entity) = sim.substrate.entities.get(event.target_id) else {
        return Ok(None);
    };
    if entity.mod_component.is_empty() {
        return Ok(None);
    }
    let object = sim.interner.object_type(entity.type_ref(), rules);
    let divisors =
        crate::sim::combat::receiver_divisors(entity, object, rules, &sim.interner, &sim.houses);
    let Some(context) = damage_context(sim, rules, event, input, &divisors) else {
        return Ok(None);
    };
    Ok(program
        .evaluate(Context::Damage(context), &entity.mod_component)?
        .damage)
}

pub(crate) fn tick_entity(
    sim: &mut crate::sim::world::Simulation,
    id: u64,
    rules: &crate::rules::ruleset::RuleSet,
) {
    let Some(program) = sim.external_mod.as_ref() else {
        return;
    };
    let Some(entity) = sim.substrate.entities.get(id) else {
        return;
    };
    if entity.mod_component.is_empty() {
        return;
    }
    let Some(context) = live_entity_context(sim, id, rules) else {
        return;
    };
    let request = api::TickContext {
        frame: sim.session.binary_frame() as i32,
        entity: context,
    };
    let plan = match program.evaluate(Context::Tick(request), &entity.mod_component) {
        Ok(plan) => plan,
        Err(error) => {
            sim.external_mod_fault = Some(format!("entity {id} tick: {error}"));
            return;
        }
    };
    if let Some(entity) = sim.substrate.entities.get_mut(id) {
        commit_entity(entity, plan);
    }
}

#[cfg(test)]
#[path = "mod_sdk/tests.rs"]
mod tests;
