//! Window-free acceptance through the production retail loader and simulation.
//! Each process admits one optional component and one memory-only test map.
//! No profile, window, audio device or network service is opened.

use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

use crate::{
    assets::{
        asset_manager::{AssetManager, MediaArchiveMode},
        community,
    },
    headless_scenario::{self, HeadlessScenario},
    sim::{
        combat::{EntityDamageEvent, ReceiverCallFlags},
        world::FrameEffects,
    },
};

const AMMO_UNIT_INI: &str = "Ammo=5\r\nInitialAmmo=5\r\nReload=1000\r\nPrimary=SDKAmmoGun\r\nElitePrimary=SDKAmmoGun\r\nCanPassiveAquire=no\r\n";
const AMMO_PROBE_INI: &str = "\r\n[SDKAmmoGun]\r\nDamage=1\r\nROF=15\r\nRange=6\r\nSpeed=100\r\nProjectile=InvisibleLow\r\nWarhead=SDKProbeWH\r\nAmmo=2\r\n";

const PROBE_MAP: &str = "sdk-probe.map";
// The default-installation probe also retains a real, originally unlisted WH
// through the same-pass Super reference reader and the saved resource envelope.
const SUPER_PROBE_INI: &str = "\r\n[SuperWeaponTypes]\r\nSDKProbe=SDKSuperStrike\r\n[SDKSuperStrike]\r\nType=GenericWarhead\r\nSW.Warhead=SDKSuperPayload\r\nSW.Damage=1\r\nMoney.Amount=-333\r\n[SDKSuperPayload]\r\nCellSpread=0.5\r\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\r\n";
// An authored demonstration layer, never a host implementation of shield rules.
// A 100% receiver for every retail armor isolates the component from unrelated
// weapon/armor tuning. Only the authored feature selection differs by mode.
const PROBE_INI: &str = "\r\n[ShieldTypes]\r\nSDKProbe=SDKShield\r\n[SDKShield]\r\nStrength=100\r\nArmor=none\r\nSelfHealing=0.25\r\nSelfHealing.Rate=0.1\r\nRespawn=0.5\r\nRespawn.Rate=0.1\r\nPips=16,16,16\r\n[MTNK]\r\nShieldType=SDKShield\r\n[Warheads]\r\nSDKProbe=SDKProbeWH\r\n[SDKProbeWH]\r\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\r\nCellSpread=0\r\nPercentAtMax=1\r\n[VeraOverload]\r\nHeatLimit=60\r\nDuration=75\r\nCooling=1\r\n";

/// Early main dispatch. Named probe arguments are deliberately separate from
/// the native substring switch parser, so paths cannot turn into game switches.
pub fn try_run<I>(args: I) -> Option<Result<i32>>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let flag = args.next()?;
    if flag == OsStr::new("--mod-sdk-save-probe") {
        return Some((|| {
            let retail = PathBuf::from(args.next().context("missing retail directory")?);
            let save = PathBuf::from(args.next().context("missing previous snapshot")?);
            let output = PathBuf::from(args.next().context("missing report path")?);
            let module = args.next().map(PathBuf::from);
            ensure!(
                args.next().is_none(),
                "usage: --mod-sdk-save-probe RETAIL SAVE OUTPUT [CURRENT_MODULE]"
            );
            validate_output(&retail, &output)?;
            write_result(
                &output,
                probe_saved_snapshot(&save, &retail, module.as_deref()),
            )
        })());
    }
    if flag == OsStr::new("--game-data-checkfast") {
        return Some((|| {
            let retail = PathBuf::from(args.next().context("missing retail directory")?);
            let map = args
                .next()
                .context("missing retail map name")?
                .into_string()
                .map_err(|_| anyhow::anyhow!("map name must be Unicode"))?;
            let output = PathBuf::from(args.next().context("missing report path")?);
            ensure!(
                args.next().is_none(),
                "usage: --game-data-checkfast RETAIL MAP OUTPUT"
            );
            validate_output(&retail, &output)?;
            let result = (|| {
                let mut scenario = headless_scenario::load(&retail, &map, 0x5344_4b31)
                    .map_err(anyhow::Error::msg)?;
                advance(&mut scenario)?;
                Ok(
                    json!({"ok":true,"kind":"game_data_checkfast","map":map,"source":scenario.source,"frame":scenario.sim().session().binary_frame(),"entities":scenario.sim().entities().len(),"rulesHash":format!("{:016x}",scenario.runtime.resources.rules.simulation_config_hash())}),
                )
            })();
            write_result(&output, result)
        })());
    }
    if flag != OsStr::new("--mod-sdk-probe") {
        return None;
    }
    Some((|| {
        let mut fields = BTreeMap::new();
        while let Some(key) = args.next() {
            let key = key
                .into_string()
                .map_err(|_| anyhow::anyhow!("invalid probe option"))?;
            ensure!(
                ["--retail-dir", "--map", "--mode", "--module", "--output"].contains(&key.as_str()),
                "unknown probe option {key}"
            );
            let value = args
                .next()
                .with_context(|| format!("missing value for {key}"))?;
            ensure!(
                fields.insert(key, value).is_none(),
                "duplicate probe option"
            );
        }
        let mut take = |key: &str| fields.remove(key).with_context(|| format!("missing {key}"));
        let retail = PathBuf::from(take("--retail-dir")?);
        let map = take("--map")?
            .into_string()
            .map_err(|_| anyhow::anyhow!("map name must be Unicode"))?;
        let mode = take("--mode")?
            .into_string()
            .map_err(|_| anyhow::anyhow!("invalid probe mode"))?;
        let output = PathBuf::from(take("--output")?);
        let module = fields.remove("--module").map(PathBuf::from);
        ensure!(
            [
                "none",
                "disabled",
                "compat",
                "overload",
                "ammo",
                "ammo-only",
                "all",
                "ares"
            ]
            .contains(&mode.as_str()),
            "mode must be none, disabled, compat, overload, ammo, ammo-only, all or ares"
        );
        ensure!(
            (mode == "none") == module.is_none(),
            "none takes no component; all other modes require --module"
        );
        validate_output(&retail, &output)?;
        write_result(
            &output,
            run_probe(&retail, &map, &mode, module.as_deref(), &output),
        )
    })())
}

fn validate_output(retail: &Path, output: &Path) -> Result<()> {
    let retail = retail
        .canonicalize()
        .context("resolve read-only retail directory")?;
    let cwd = std::env::current_dir()?.canonicalize()?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .context("report parent must already exist")?;
    ensure!(
        !cwd.starts_with(&retail) && !parent.starts_with(&retail),
        "run the probe in an isolated work directory, with output outside retail data"
    );
    ensure!(!output.exists(), "probe report already exists");
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create {}", path.display()))?;
    file.write_all(bytes)?;
    Ok(())
}

fn write_result(output: &Path, result: Result<Value>) -> Result<i32> {
    let (code, receipt) = match result {
        Ok(receipt) => (0, receipt),
        Err(error) => (1, json!({"ok":false,"error":format!("{error:#}")})),
    };
    write_new(output, &serde_json::to_vec_pretty(&receipt)?)?;
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(code)
}

/// Use the shipping build's schema: lib tests retain TriggerRuntime's fixture
/// difficulty field and cannot decode a shipping save as the same layout.
/// The caller supplies an actual preceding-release file, not a current save
/// relabelled with an older header. No saved component is replaced/configured.
fn probe_saved_snapshot(path: &Path, retail: &Path, module_path: Option<&Path>) -> Result<Value> {
    use crate::sim::snapshot::GameSnapshot;
    let bytes = std::fs::read(path).context("read previous full snapshot")?;
    let metadata = GameSnapshot::read_metadata(&bytes)?;
    let mut loaded = GameSnapshot::load_with_resources(&bytes)?;
    ensure!(
        metadata.is_compatible(),
        "browser rejected admitted snapshot"
    );
    let digest = loaded.sim.external_mod_digest();
    let module = loaded
        .sim
        .external_mod
        .as_ref()
        .map(|p| p.manifest().clone());
    let rebound = if let Some(module_path) = module_path {
        let mut files = Vec::new();
        push_probe_component(&mut files, module_path)?;
        community::install(community::Snapshot::new(
            "mod-sdk-save-probe".into(),
            files,
        )?)?;
        let assets = super::open_active_assets(retail, MediaArchiveMode::STOCK_DIGITAL)?;
        let mut resources = loaded
            .take_scenario_resources()
            .context("saved resources missing")?;
        resources
            .admit_configuration_program(&loaded.sim)
            .map_err(anyhow::Error::msg)?;
        let mut rules =
            crate::rules::ruleset::RuleSet::from_processed_rules(resources.processed_rules())
                .map_err(anyhow::Error::msg)?;
        super::adopt_saved_program(&mut rules, &loaded.sim, &assets).map_err(anyhow::Error::msg)?;
        Some(json!({
            "weapon":rules.weapon("SDKAmmoGun").and_then(|w| w.resource_policy(vera_mod_api::WEAPON_USER_UNIT)),
            "object":rules.object("MTNK").and_then(|t| t.type_resource_policy()),
        }))
    } else {
        None
    };
    loaded.sim.restore_after_snapshot_load()?;
    ensure!(
        loaded.sim.external_mod_digest() == digest,
        "restored program changed"
    );
    Ok(json!({
        "ok":true, "kind":"mod_sdk_saved_snapshot", "layout":metadata.layout,
        "browserAdmission":true, "resourceAdmission":true, "stateRestore":true,
        "frozenProgramPreserved":true, "module":module, "reboundPolicies":rebound,
        "scope":"saved resources and structural restore; no resident asset rebind or GUI"
    }))
}

fn push_probe_component(files: &mut Vec<(String, Vec<u8>)>, module_path: &Path) -> Result<()> {
    ensure!(
        std::fs::metadata(module_path)?.len() <= 16 * 1024 * 1024,
        "component exceeds admission limit"
    );
    let component = std::fs::read(module_path)?;
    let digest = crate::util::sha256::sha256_hex(&component);
    files.push((
        "vera-mod.json".into(),
        serde_json::to_vec(&json!({
            "api_version":vera_mod_api::API_VERSION,"program":"probe-gameplay.wasm","sha256":digest
        }))?,
    ));
    files.push(("probe-gameplay.wasm".into(), component));
    Ok(())
}

fn advance(scenario: &mut HeadlessScenario) -> Result<()> {
    scenario
        .runtime
        .advance_frame_for_tooling(&[], headless_scenario::SIM_TICK_MS)
        .map_err(|e| anyhow::anyhow!("advance production frame: {e}"))
}

fn spawn_target(scenario: &mut HeadlessScenario) -> Result<u64> {
    let owner = scenario
        .sim()
        .houses()
        .iter()
        .find(|(_, house)| house.is_human())
        .map(|(owner, _)| scenario.sim().interner().resolve(*owner).to_owned())
        .context("retail launch has no human house")?;
    let mut starts = scenario
        .map
        .waypoints
        .values()
        .map(|w| (w.index, w.rx, w.ry))
        .collect::<Vec<_>>();
    starts.sort_unstable();
    for (_, x, y) in starts {
        for radius in 1..=8i32 {
            for (dx, dy) in [
                (radius, 0),
                (0, radius),
                (-radius, 0),
                (0, -radius),
                (radius, radius),
                (-radius, -radius),
            ] {
                let (Ok(rx), Ok(ry)) = (
                    u16::try_from(i32::from(x) + dx),
                    u16::try_from(i32::from(y) + dy),
                ) else {
                    continue;
                };
                let runtime = &mut scenario.runtime;
                if let Some(id) = runtime.simulation.spawn_object_with_overlay_registry(
                    "MTNK",
                    &owner,
                    rx,
                    ry,
                    0,
                    &runtime.resources.rules,
                    &runtime.resources.overlay_registry,
                ) {
                    return Ok(id);
                }
            }
        }
    }
    bail!("retail map has no free MTNK placement near its waypoints")
}

fn sample(scenario: &HeadlessScenario, target: u64, stage: &str) -> Result<Value> {
    let entity = scenario
        .sim()
        .entities()
        .get(target)
        .context("probe target disappeared")?;
    Ok(
        json!({"stage":stage,"frame":scenario.sim().session().binary_frame(),"health":entity.health.current(),"projection":entity.mod_presentation(),"componentBytes":entity.mod_component().len(),"componentSha256":crate::util::sha256::sha256_hex(entity.mod_component()),"stateHash":format!("{:016x}",scenario.sim().state_hash())}),
    )
}

fn run_probe(
    retail: &Path,
    selected_map: &str,
    mode: &str,
    module_path: Option<&Path>,
    output: &Path,
) -> Result<Value> {
    let has_shield = matches!(mode, "compat" | "overload" | "ammo" | "all");
    let has_overload = matches!(mode, "overload" | "all");
    let has_ammo = matches!(mode, "ammo" | "ammo-only" | "all" | "ares");
    let has_manual = matches!(mode, "all" | "ares");
    let full_ares = matches!(mode, "all" | "ares");
    let tests_ammo = has_ammo || mode == "disabled";
    let assets = AssetManager::new(retail, MediaArchiveMode::STOCK_DIGITAL)?;
    ensure!(
        !assets.contains("vera-mod.json"),
        "acceptance requires a retail base without an installed gameplay component"
    );
    // Source selection and bounded read are the same ones used for map upload.
    let selected =
        crate::map::source::load_map_by_name_or_path_with_assets(retail, selected_map, &assets)?;
    let (length, digest) = selected
        .source
        .physical_identity()
        .context("probe needs an authored retail map")?;
    let mut bytes = crate::map::source::prepare_transfer_map_source(&selected.source, &assets)?
        .read(digest, length)?;
    // A packed .mmx wrapper must be selected as its plain inner .map instead:
    // appending INI to a MIX wrapper would silently test the wrong source.
    crate::map::map_file::MapFile::from_bytes(&bytes)
        .context("choose a plain authored .map entry, not a wrapped .mmx")?;
    ensure!(
        !selected
            .map
            .ini
            .section_or_empty("MTNK")
            .is_present("ShieldType"),
        "test map already overrides MTNK ShieldType"
    );
    ensure!(
        selected.map.ini.section("SDKProbeWH").is_none()
            && selected.map.ini.section("SDKShield").is_none(),
        "test map reserves SDK probe identities"
    );
    if full_ares {
        ensure!(
            selected.map.ini.section("SuperWeaponTypes").is_none()
                && selected.map.ini.section("SDKSuperStrike").is_none()
                && selected.map.ini.section("SDKSuperPayload").is_none(),
            "test map reserves the SDK SuperWeapon probe sections"
        );
    }
    let original_source = selected.source;
    drop(selected.map);
    // Keep one physical MTNK section: duplicate INI headers do not merge.
    let probe_ini = if tests_ammo {
        PROBE_INI.replacen(
            "ShieldType=SDKShield\r\n",
            &format!("ShieldType=SDKShield\r\n{}", AMMO_UNIT_INI),
            1,
        )
    } else {
        PROBE_INI.to_owned()
    };
    let probe_ini = if has_manual {
        probe_ini.replacen("ShieldType=SDKShield\r\n", "ShieldType=SDKShield\r\nNoManualFire=yes\r\nNoManualUnload=yes\r\nNoManualEnter=yes\r\nNoSelfGuardArea=yes\r\n", 1)
    } else {
        probe_ini
    };
    let probe_ini = if full_ares {
        probe_ini.replacen("ShieldType=SDKShield\r\n", "ShieldType=SDKShield\r\nReloadAmount=2\r\nEmptyReloadAmount=3\r\nNoAmmoWeapon=1\r\nNoAmmoAmount=0\r\nSecondary=SDKAmmoGun\r\nEliteSecondary=SDKAmmoGun\r\n", 1)
    } else {
        probe_ini
    };
    bytes.extend_from_slice(probe_ini.as_bytes());
    if tests_ammo {
        bytes.extend_from_slice(AMMO_PROBE_INI.as_bytes());
    }
    if full_ares {
        bytes.extend_from_slice(SUPER_PROBE_INI.as_bytes());
    }
    // The default-installation case writes NO VeraFeatures section. Other
    // modes explicitly select every module to retain isolated comparisons.
    if mode != "ares" {
        bytes.extend_from_slice(
        format!(
            "\r\n[VeraFeatures]\r\nShield={}\r\nOverload={}\r\nAresAmmo={}\r\nAresManualControl={}\r\nAresReload={}\r\nAresNoAmmoWeapon={}\r\nAresKeepAlive={}\r\nAresAttachEffect={}\r\nAresArmor={}\r\nAresProduction={}\r\nAresEMP={}\r\nAresSpotlight={}\r\nAresTypeConversion={}\r\nAresFoundation={}\r\nAresFaction={}\r\n",
            if has_shield { "yes" } else { "no" },
            if has_overload { "yes" } else { "no" },
            if has_ammo { "yes" } else { "no" },
            if has_manual { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
            if full_ares { "yes" } else { "no" },
        )
        .as_bytes(),
    );
    }
    let mut files = vec![(PROBE_MAP.to_string(), bytes)];
    if let Some(module_path) = module_path {
        push_probe_component(&mut files, module_path)?;
    }
    community::install(community::Snapshot::new("mod-sdk-probe".into(), files)?)?;
    drop(assets);
    let assets = super::open_active_assets(retail, MediaArchiveMode::STOCK_DIGITAL)?;
    let audio = crate::rules::audio_sources::AudioDefinitions::select(&assets);
    let (_, _, mut owner) = crate::rules::retail_sources::load_startup_rules(
        &assets,
        std::sync::Arc::clone(audio.sounds()),
        std::sync::Arc::clone(audio.eva()),
    )
    .context("read native startup rules")?
    .into_parts();
    let launch = crate::sim::scenario_bootstrap::MatchLaunchDescriptor::from_resolved(
        headless_scenario::battle_session(PROBE_MAP, &[]),
    )
    .map_err(anyhow::Error::msg)?;
    let mut scenario = headless_scenario::load_with_launch_and_ini(
        retail,
        PROBE_MAP,
        0x5344_4b31,
        launch,
        &Default::default(),
        crate::sim::scenario_session::ScenarioGameMode::Skirmish,
        None,
        Some(&mut owner),
    )
    .map_err(anyhow::Error::msg)?;
    let identity = scenario
        .sim()
        .external_mod
        .as_ref()
        .map(|p| json!({"sha256":p.sha256(),"manifest":p.manifest()}));
    let super_resources = if full_ares
        && scenario
            .runtime
            .resources
            .rules
            .superweapon_semantics()
            .custom_payloads
    {
        let rules = &scenario.runtime.resources.rules;
        let sw = rules
            .super_weapon("SDKSuperStrike")
            .context("configured SuperWeapon missing")?;
        ensure!(
            sw.generic_warhead
                .as_ref()
                .and_then(|payload| payload.warhead.as_deref())
                == Some("SDKSuperPayload")
                && sw.money_amount == -333
                && rules
                    .warhead("SDKSuperPayload")
                    .is_some_and(|wh| wh.cell_spread_f64 == 0.5),
            "SuperWeapon did not retain its original unlisted Warhead allocation"
        );
        Some(json!({"unlistedWarheadBound":true,"moneyAmount":sw.money_amount}))
    } else {
        None
    };
    match mode {
        "none" => ensure!(identity.is_none(), "no-module mode admitted a module"),
        _ => {
            let legacy_id = match mode {
                "ammo" => Some("vera.compat-shield-ammo"),
                "compat" => Some("vera.compat-shield"),
                "overload" => Some("vera.overload-shield"),
                _ => None,
            };
            ensure!(
                scenario.sim().external_mod.as_ref().is_some_and(|p| {
                    p.manifest().id == "vera.reference-gameplay"
                        || legacy_id == Some(p.manifest().id.as_str())
                }),
                "component identity does not match requested mode"
            );
        }
    }
    let expected_input = has_manual.then_some(vera_mod_api::InputPolicy {
        deny_manual_attack: true,
        deny_manual_unload: true,
        hide_transport_entry: true,
        deny_self_guard_area: false,
    });
    ensure!(
        scenario
            .runtime
            .resources
            .rules
            .object("MTNK")
            .and_then(|t| t.input_policy())
            == expected_input,
        "manual input projection did not bind through production retail Rules"
    );
    let default_policies = if full_ares {
        use vera_mod_api::{DepletedWeaponPolicy, ReloadPolicy, WEAPON_USER_BUILDING};
        let rules = &scenario.runtime.resources.rules;
        let policy = rules
            .object("MTNK")
            .and_then(|t| t.type_resource_policy())
            .context("Ares object policies missing")?;
        ensure!(
            policy.reload
                == Some(ReloadPolicy {
                    normal_amount: 2,
                    empty_amount: 3
                })
                && policy.depleted_weapon
                    == Some(DepletedWeaponPolicy {
                        weapon_index: 1,
                        threshold: 0
                    }),
            "original reload/NoAmmoWeapon tags did not bind"
        );
        let ordinary = rules
            .weapon("120mm")
            .and_then(|w| w.resource_policy(WEAPON_USER_BUILDING))
            .context("ordinary weapon policy missing")?;
        ensure!(
            ordinary.cost == 1 && ordinary.use_weapon_rof,
            "installed Ares must also own untagged weapon cost and building cadence"
        );
        let defence = rules
            .object("TESLA")
            .and_then(|t| t.type_resource_policy())
            .context("ordinary defence policies missing")?;
        ensure!(
            defence.reload
                == Some(ReloadPolicy {
                    normal_amount: 1,
                    empty_amount: 1
                }),
            "installed Ares omitted ordinary building reload"
        );
        Some(
            json!({"featureOptInWritten":mode != "ares", "object":policy,
            "ordinaryWeapon":ordinary, "ordinaryDefence":defence}),
        )
    } else {
        None
    };
    let target = spawn_target(&mut scenario)?;
    let initial = scenario
        .sim()
        .entities()
        .get(target)
        .context("constructed target absent")?
        .health
        .current();
    ensure!(initial > 180, "retail MTNK health too low for probe");
    let mut samples = vec![sample(&scenario, target, "attached")?];
    for shot in 0..3 {
        let runtime = &mut scenario.runtime;
        ensure!(
            runtime.resources.rules.warhead("SDKProbeWH").is_some(),
            "authored probe warhead missing from Rules"
        );
        let warhead = runtime.simulation.interner.intern("SDKProbeWH");
        let event = EntityDamageEvent::direct_receiver(
            target,
            60,
            0,
            0,
            None,
            warhead,
            ReceiverCallFlags {
                ignore_defenses: false,
                arg6: false,
            },
        );
        runtime.simulation.commit_direct_damage_receiver(
            &runtime.resources.rules,
            Some(&runtime.resources.overlay_registry),
            event,
            FrameEffects::default(),
        );
        ensure!(
            runtime.simulation.external_mod_fault.is_none(),
            "component damage fault: {:?}",
            runtime.simulation.external_mod_fault
        );
        samples.push(sample(&scenario, target, &format!("damage-{}", shot + 1))?);
        let entity = scenario
            .sim()
            .entities()
            .get(target)
            .context("probe target died")?;
        let expected_loss = if !has_shield {
            [60, 120, 180]
        } else if has_overload {
            [0, 60, 120]
        } else {
            [0, 20, 80]
        }[shot];
        ensure!(
            entity.health.current() == initial - expected_loss,
            "{mode} damage {}: health {}, expected {}",
            shot + 1,
            entity.health.current(),
            initial - expected_loss
        );
        if !has_shield {
            ensure!(
                entity.mod_presentation().meters.is_empty() && entity.mod_component().is_empty(),
                "shield-disabled entity contains shield state"
            );
        } else {
            let meter = entity
                .mod_presentation()
                .meters
                .first()
                .context("component omitted its meter")?;
            let expected = if !has_overload && shot > 0 { 0 } else { 40 };
            ensure!(
                meter.current == expected && meter.maximum == 100,
                "unexpected component meter after damage {}",
                shot + 1
            );
        }
        advance(&mut scenario)?;
    }
    let ammunition = if tests_ammo {
        Some(probe_ammunition(&mut scenario, has_ammo)?)
    } else {
        None
    };
    let crc = scenario
        .runtime
        .simulation
        .compute_retail_multiplayer_checksum()?;
    let snapshot = snapshot_roundtrip(&mut scenario, &owner, &assets, target, output)?;
    ensure!(
        scenario
            .runtime
            .resources
            .rules
            .object("MTNK")
            .and_then(|t| t.input_policy())
            == expected_input,
        "restored frozen manual input projection differs"
    );
    Ok(
        json!({"ok":true,"kind":"mod_sdk_probe","mode":mode,"apiVersion":vera_mod_api::API_VERSION,"originalSource":original_source,"scenarioSource":scenario.source,"module":identity,"target":target,"initialHealth":initial,"samples":samples,"ammunition":ammunition,"defaultPolicies":default_policies,"superweaponResources":super_resources,"nativeChecksum":{"value":crc.value,"diagnosticRngSample":crc.diagnostic_rng_sample},"snapshot":snapshot,"final":sample(&scenario,target,"restored-frame")?,"scope":"production retail load/constructor/ReceiveDamage/frame; resource-admitted snapshot continuation, no GUI or transport"}),
    )
}

fn probe_ammunition(scenario: &mut HeadlessScenario, enabled: bool) -> Result<Value> {
    use crate::sim::command::Command;
    let actor = spawn_target(scenario)?;
    let entity = scenario
        .sim()
        .entities()
        .get(actor)
        .context("ammo actor absent")?;
    let owner = scenario.sim().interner().resolve(entity.owner()).to_owned();
    let command = Command::ForceAttackCell {
        attacker_id: actor,
        target_rx: entity
            .position
            .rx()
            .checked_add(1)
            .context("ammo target coordinate")?,
        target_ry: entity
            .position
            .ry()
            .checked_add(1)
            .context("ammo target coordinate")?,
    };
    ensure!(
        entity.ammo().is_some_and(|ammo| ammo.current() == 5),
        "ammo constructor did not start at five"
    );
    let runtime = &mut scenario.runtime;
    ensure!(
        runtime.simulation.apply_command_with_overlays(
            &owner,
            &command,
            &runtime.resources.rules,
            Some(&runtime.resources.overlay_registry),
            FrameEffects::default()
        ),
        "force-fire command rejected"
    );
    let expected: &[i32] = if enabled { &[3, 1] } else { &[4, 3, 2, 1, 0] };
    let mut shots = Vec::new();
    let mut last_shot = None;
    for visit in 0..300 {
        let output = scenario.runtime.advance_frame(
            &[],
            headless_scenario::SIM_TICK_MS,
            crate::sim::world::TickLane::Ordinary,
            FrameEffects::default(),
        )?;
        if output
            .fire_events
            .iter()
            .any(|event| event.attacker_id == actor)
        {
            let count = scenario
                .sim()
                .entities()
                .get(actor)
                .and_then(|entity| entity.ammo())
                .context("ammo actor disappeared")?
                .current();
            shots.push(count);
            last_shot = Some(visit);
            ensure!(
                shots.len() <= expected.len(),
                "insufficient ammunition incorrectly funded another shot"
            );
        }
        if shots.len() == expected.len() && last_shot.is_some_and(|last| visit > last + 90) {
            break;
        }
    }
    ensure!(
        shots == expected,
        "expected real shot counts {expected:?}, got {shots:?}"
    );
    Ok(
        json!({"actor":actor,"initial":5,"afterShots":shots,"aresAmmoEnabled":enabled,"blockedBelowCost":true,"source":"production command/CanFire/FireAt/Logic"}),
    )
}

fn snapshot_roundtrip(
    scenario: &mut HeadlessScenario,
    owner: &crate::rules::process_owner::NativeRulesProcessOwner,
    assets: &AssetManager,
    target: u64,
    output: &Path,
) -> Result<Value> {
    use crate::sim::snapshot::{
        GameSnapshot,
        saved_local_view::SavedLocalView,
        scenario_resources::{SavedMapMetadata, SavedScenarioResourcesRef},
    };
    let rules = &scenario.runtime.resources.rules;
    let sim = &scenario.runtime.simulation;
    let before = sim
        .entities()
        .get(target)
        .context("snapshot target absent")?
        .mod_component()
        .to_vec();
    let module_before = sim.external_mod_digest();
    let weapon_policy_before = rules
        .weapon("SDKAmmoGun")
        .and_then(|weapon| weapon.resource_policy(vera_mod_api::WEAPON_USER_UNIT));
    let type_policy_before = rules.object("MTNK").and_then(|t| t.type_resource_policy());
    let live = sim.resolved_terrain().context("live terrain missing")?;
    let terrain = live.capture_saved_template()?;
    let map_hash = scenario.map.ini.content_hash();
    let roster = crate::map::houses::parse_house_roster(
        &scenario.map.ini,
        rules.color_schemes.as_slice(),
        Some(rules),
    );
    let map = SavedMapMetadata::capture(
        map_hash,
        &scenario.source,
        &scenario.map.basic,
        &scenario.map.ranking,
        &Default::default(),
        &scenario.map.terrain_objects,
        &roster,
        &Default::default(),
        &crate::map::lighting::parse_lighting(&scenario.map.ini),
        0,
        Some("Player"),
        [-1, -1],
        false,
    );
    let mut resources = SavedScenarioResourcesRef::capture(
        &scenario.runtime.resources,
        owner
            .saved_scenario_rules()
            .context("native process did not retain Rules passes")?,
        owner.fixed_art().content_hash(),
        terrain,
        map,
    );
    resources.capture_live_cell_land(live)?;
    let view = SavedLocalView::new([[0, 0]; 4], Vec::new(), false, None, Default::default())?;
    let bytes = GameSnapshot::save_validated_with_presentation(
        sim,
        map_hash,
        rules.simulation_config_hash(),
        "SDK acceptance",
        0,
        &[],
        &[],
        None,
        None,
        Some(&view),
        Some(&resources),
    );
    let save_path = output.with_extension("vsave");
    write_new(&save_path, &bytes)?;
    let mut loaded = GameSnapshot::load_with_resources(&bytes)?;
    ensure!(
        loaded.rules_hash == rules.simulation_config_hash(),
        "snapshot Rules digest changed"
    );
    ensure!(
        loaded
            .sim
            .entities()
            .get(target)
            .context("loaded target absent")?
            .mod_component()
            == before,
        "snapshot lost opaque entity state"
    );
    ensure!(
        loaded.sim.external_mod_digest() == module_before,
        "snapshot changed module/config/component digest"
    );
    // This is the same saved-program admission used by App. In particular it
    // restores Rules' constructor binding without re-running Configure.
    // Simulate a freshly projected RuleSet, whose module binding starts empty.
    // Otherwise an implementation that only validates and forgets to rebind
    // would pass by retaining the outgoing world's constructor hook.
    scenario.runtime.resources.rules.external_mod = None;
    scenario
        .runtime
        .resources
        .rules
        .bind_resource_policies()
        .map_err(anyhow::Error::msg)?;
    super::adopt_saved_program(&mut scenario.runtime.resources.rules, &loaded.sim, assets)
        .map_err(anyhow::Error::msg)?;
    ensure!(
        scenario
            .runtime
            .resources
            .rules
            .weapon("SDKAmmoGun")
            .and_then(|weapon| weapon.resource_policy(vera_mod_api::WEAPON_USER_UNIT))
            == weapon_policy_before,
        "saved weapon policy was not rebound"
    );
    ensure!(
        scenario
            .runtime
            .resources
            .rules
            .object("MTNK")
            .and_then(|t| t.type_resource_policy())
            == type_policy_before,
        "saved type policies were not rebound"
    );
    let mut resources = loaded
        .take_scenario_resources()
        .context("saved resource owner absent")?;
    resources
        .admit_configuration_program(&loaded.sim)
        .map_err(anyhow::Error::msg)?;
    let saved_land = resources.terrain().user_load_land_values()?;
    let dummy = crate::map::resolved_terrain::SharedCellDummy::fresh();
    let terrain = resources.terrain().clone().restore_values(dummy.clone())?;
    let mut restored = loaded.sim;
    restored.retain_process_state_for_load(crate::sim::world::SnapshotProcessState::Live(
        &scenario.runtime.simulation,
    ));
    restored.bind_shared_cell_dummy(dummy);
    restored.reconstruct_cellclass_dummy_for_map_resize();
    restored.restore_after_snapshot_load()?;
    let rules = &scenario.runtime.resources.rules;
    restored.rebuild_caches_after_load(
        terrain,
        scenario.runtime.simulation.terrain_speed_config().clone(),
        rules,
    );
    restored.restore_map_authority_after_user_load(
        rules,
        &scenario.runtime.resources.overlay_registry,
        None,
        saved_land,
    )?;
    restored.intern_rule_type_ids(rules);
    restored.resolve_type_handles(rules);
    restored.rebuild_lighting_sources_after_load(rules);
    scenario.runtime.simulation = restored;
    advance(scenario)?;
    // A post-load constructor verifies that the Rules owner also retained the
    // saved program; continuing only old entities would miss this regression.
    let new_target = spawn_target(scenario)?;
    let new_entity = scenario
        .sim()
        .entities()
        .get(new_target)
        .context("post-load constructor absent")?;
    // Loading a module does not imply this object subscribes: disabled and
    // ammo-only works legitimately keep no per-object component state.
    ensure!(
        new_entity.mod_component().is_empty() == before.is_empty(),
        "post-load constructor changed its component subscription"
    );
    Ok(
        json!({"resourceAdmission":true,"opaqueStatePreserved":true,"simulationResume":true,"postLoadAttach":true,"weaponResourcesRebound":true,"file":save_path,"bytes":bytes.len(),"sha256":crate::util::sha256::sha256_hex(&bytes),"guiStateRestored":false,"residentAssetRebindExercised":false}),
    )
}
