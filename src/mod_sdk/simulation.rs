//! Convert engine facts into SDK contexts and commit validated plans at the
//! existing constructor, damage and AI phases. Gameplay state stays with its
//! engine owner; the guest owns only its opaque component state.
use vera_mod_api::{self as api, Context, Plan};

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
