//! Collected real-component regressions. Run with the normal core build and
//! VERA_SDK_EFFECT_WASM pointing at the packaged component, not core Wasm.
use super::*;
use crate::rules::{
    art_data::ArtRegistry, ini_parser::IniFile, native_processing::RulesLayerStack,
    ruleset::TypeHandle,
};
use crate::sim::{
    combat::{TargetKind, world_fire::FireVisit},
    world::{FrameEffects, ObjectAiCtx},
};
use crate::util::native_x87::NativeF64Bits;
use std::sync::Arc;

fn scene(component: Option<&[u8]>, visual: bool) -> (Simulation, RuleSet, u64, u64) {
    let ini = crate::sim::arena_fixture::clear_ground_ini(&format!(
        "[VehicleTypes]\n0=EMITTER\n1=VICTIM\n[Warheads]\n0=AE_TEST\n[Animations]\n0=VISUAL\n\
         [EMITTER]\nStrength=400\nArmor=heavy\nSpeed=6\nPrimary=PULSE\nLocomotor={{4A582741-9839-11d1-B709-00A024DDAFD1}}\nMovementZone=Normal\n\
         [VICTIM]\nStrength=1000\nArmor=heavy\nSpeed=4\nLocomotor={{4A582741-9839-11d1-B709-00A024DDAFD1}}\nMovementZone=Normal\n\
         [PULSE]\nDamage=0\nROF=1\nRange=9.5\nSpeed=100\nProjectile=INVIS\nWarhead=AE_TEST\n\
         [INVIS]\nInviso=yes\nAG=yes\nAA=no\n\
         [AE_TEST]\nCellSpread=.7\nVerses=3%,3%,3%,3%,3%,3%,3%,3%,3%,3%,3%\nAffectsAllies=yes\nAttachEffect.Duration=5\n\
         AttachEffect.FirepowerMultiplier=2\nAttachEffect.ArmorMultiplier=2\nAttachEffect.SpeedMultiplier=2\nAttachEffect.ROFMultiplier=0.5\n\
         AttachEffect.Animation={}\nAttachEffect.AnimResetOnReapply=yes\nAttachEffect.DiscardOnEntry={}\n",
        if visual { "VISUAL" } else { "none" },
        if visual { "yes" } else { "no" },
    ));
    let art = IniFile::from_str("[VISUAL]\nRate=1\nEnd=4\n");
    let processed = RulesLayerStack::new(ini)
        .process_with_fixed_art(&art)
        .unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    let mut registry = ArtRegistry::from_ini(&art);
    registry.bind_anim_frame_count_for_test("VISUAL", 4);
    rules.install_art_data(registry);
    if let Some(wasm) = component {
        let objects = (0..rules.object_count())
            .map(|i| rules.object_by_handle(TypeHandle(i as u32)).id.clone())
            .collect::<Vec<_>>();
        let warheads = rules
            .warheads_iter()
            .map(|wh| wh.id.clone())
            .collect::<Vec<_>>();
        let program = crate::mod_sdk::Program::load_with_registries(
            wasm.to_vec(),
            &crate::util::sha256::sha256_hex(wasm),
            processed.sdk_passes(),
            &objects,
            &warheads,
            &rules.weapon_names(),
            processed.sdk_registries(),
        )
        .unwrap();
        rules.external_mod = Some(Arc::new(program));
        rules.bind_resource_policies().unwrap();
    }
    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 17);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(&rules);
    let target = sim
        .spawn_object("VICTIM", "Russians", 10, 10, 0, &rules)
        .unwrap();
    let source = sim
        .spawn_object("EMITTER", "Americans", 12, 10, 0, &rules)
        .unwrap();
    sim.resolve_type_handles(&rules);
    (sim, rules, target, source)
}

fn component() -> Vec<u8> {
    assert!(
        !crate::rules::ARES_ENABLED,
        "exercise the ordinary engine, without builtin Ares"
    );
    std::fs::read(std::env::var_os("VERA_SDK_EFFECT_WASM").expect("set VERA_SDK_EFFECT_WASM"))
        .unwrap()
}

fn fire(sim: &mut Simulation, rules: &RuleSet, source: u64, target: u64) {
    let existing = sim
        .projectiles
        .iter()
        .map(|(&id, _)| id)
        .collect::<std::collections::BTreeSet<_>>();
    sim.commit_fire_visit(
        FireVisit::Direct {
            id: source,
            target: TargetKind::Entity(target),
            weapon_index: 0,
        },
        rules,
        None,
        FrameEffects::empty(),
    );
    let bullets = sim
        .projectiles
        .iter()
        .filter(|(id, _)| !existing.contains(id))
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(bullets.len(), 1);
    for id in bullets {
        sim.object_ai_visit_one(id, Some(rules), ObjectAiCtx::default());
    }
    assert!(
        sim.external_mod_fault.is_none(),
        "{:?}",
        sim.external_mod_fault
    );
}

fn visit(sim: &mut Simulation, rules: &RuleSet, target: u64) {
    sim.session
        .set_binary_frame_for_test(sim.session.binary_frame() + 1);
    sim.object_ai_visit_one(target, Some(rules), ObjectAiCtx::default());
    assert!(
        sim.external_mod_fault.is_none(),
        "{:?}",
        sim.external_mod_fault
    );
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_EFFECT_WASM"]
fn zero_damage_bullet_updates_the_only_effect_store_and_resumes_after_save() {
    let wasm = component();
    let (mut base, base_rules, base_target, base_source) = scene(None, false);
    fire(&mut base, &base_rules, base_source, base_target);
    assert!(
        base.entities()
            .get(base_target)
            .unwrap()
            .attach_effects
            .entries
            .is_empty()
    );
    let (mut sim, rules, target, source) = scene(Some(&wasm), false);
    let baseline = sim
        .entities()
        .get(target)
        .unwrap()
        .foot_speed
        .effective_multiplier();
    fire(&mut sim, &rules, source, target);
    let actor = sim.entities().get(target).unwrap();
    assert_eq!(
        actor.health.current(),
        1000,
        "zero damage still reaches the pre-effect slot"
    );
    assert_eq!(actor.attach_effects.entries.len(), 1);
    assert!(
        actor.mod_component.is_empty(),
        "there is no second guest instance vector"
    );
    // Numeric folding itself retains the existing native-kernel regressions;
    // this assertion checks the authored factors reach its actual scalar owners.
    assert_eq!(actor.crate_multipliers.firepower().bits(), 2.0f64.to_bits());
    assert_eq!(actor.crate_multipliers.armor().bits(), 2.0f64.to_bits());
    assert_eq!(
        actor.foot_speed.effective_multiplier().bits(),
        2.0f64.to_bits()
    );
    assert_eq!(actor.attached_rof_multiplier().bits(), 0.5f64.to_bits());
    visit(&mut sim, &rules, target);
    let held = sim.entities().get(target).unwrap().attach_effects.clone();
    assert_eq!(held.entries[0].remaining, 4);
    let saved = crate::sim::snapshot::GameSnapshot::save(&sim, 0, 0, "SDK-EFFECT", 0);
    let mut restored = crate::sim::snapshot::GameSnapshot::load(&saved)
        .unwrap()
        .sim;
    restored.restore_after_snapshot_load().unwrap();
    let mut restored_rules = rules.clone();
    restored_rules.external_mod = restored.external_mod.as_ref().map(|p| Arc::new(p.clone()));
    restored_rules.bind_resource_policies().unwrap();
    restored
        .validate_attached_stats_after_restore(&restored_rules)
        .unwrap();
    assert_eq!(
        restored.entities().get(target).unwrap().attach_effects,
        held
    );
    for _ in 0..4 {
        visit(&mut restored, &restored_rules, target);
    }
    let actor = restored.entities().get(target).unwrap();
    assert!(actor.attach_effects.entries.is_empty());
    assert_eq!(actor.foot_speed.effective_multiplier(), baseline);
    assert_eq!(actor.crate_multipliers.firepower(), NativeF64Bits::ONE);
    assert_eq!(actor.crate_multipliers.armor(), NativeF64Bits::ONE);
    assert_eq!(actor.attached_rof_multiplier(), NativeF64Bits::ONE);
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_EFFECT_WASM"]
fn animation_expiry_reapply_and_limbo_use_the_existing_reference_owners() {
    let wasm = component();
    let (mut sim, rules, target, source) = scene(Some(&wasm), true);
    fire(&mut sim, &rules, source, target);
    let anim = sim.entities().get(target).unwrap().attach_effects.entries[0]
        .animation
        .unwrap();
    assert_eq!(
        sim.anim(anim).unwrap().owner_house(),
        Some(sim.entities().get(source).unwrap().owner())
    );
    sim.destroy_anim_with_context(anim, &rules);
    assert!(
        sim.entities().get(target).unwrap().attach_effects.entries[0]
            .animation
            .is_none()
    );
    visit(&mut sim, &rules, target);
    assert!(
        sim.entities().get(target).unwrap().attach_effects.entries[0]
            .animation
            .is_none(),
        "pointer expiry alone never recreates an animation"
    );
    sim.session.set_binary_frame_for_test(10);
    fire(&mut sim, &rules, source, target);
    let replacement = sim.entities().get(target).unwrap().attach_effects.entries[0]
        .animation
        .unwrap();
    assert_ne!(replacement, anim);
    assert_eq!(
        sim.entities().get(target).unwrap().attach_effects.entries[0].remaining,
        5
    );
    sim.techno_limbo_with_rules(target, &rules, None);
    assert!(
        sim.entities()
            .get(target)
            .unwrap()
            .attach_effects
            .entries
            .is_empty()
    );
    assert!(sim.pending_delete().contains_object(&replacement));
    assert_eq!(
        sim.entities()
            .get(target)
            .unwrap()
            .foot_speed
            .effective_multiplier(),
        NativeF64Bits::ONE
    );
    sim.uninit_with_rules(target, &rules);
    sim.process_pending_delete_with(&rules, None, FrameEffects::empty(), 1, false);
    assert!(sim.entities().get(target).is_none());
}
