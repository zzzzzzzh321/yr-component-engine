//! Collected unit-category regressions through the actual external component.
//! House counters, lifecycle and the defeat pass remain their production owners.
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{RulesLayerKind, RulesLayerStack};
use crate::rules::ruleset::{RuleSet, TypeHandle};
use crate::sim::house_state::HouseState;
use crate::sim::intern::InternedId;
use crate::sim::snapshot::GameSnapshot;
use crate::sim::world::Simulation;
use std::sync::Arc;

const BASE: &str = "[Countries]\n0=Country0\n1=Country1\n[Country0]\n[Country1]\n\
    [Sides]\nSide0=Country0\nSide1=Country1\n[General]\nBaseUnit=MCV1,MCV2,MCV3,MCV4\n\
    [VehicleTypes]\n0=BEACON\n1=OFF\n2=MCV1\n3=MCV2\n4=MCV3\n5=MCV4\n\
    [BEACON]\nStrength=100\nInsignificant=yes\nKeepAlive=yes\n\
    [OFF]\nStrength=100\nInsignificant=yes\nKeepAlive=no\n\
    [MCV1]\nStrength=100\n[MCV2]\nStrength=100\n[MCV3]\nStrength=100\n\
    [MCV4]\nStrength=100\nKeepAlive=no\n\
    [BuildingTypes]\n0=DECOR\n1=BASE\n[DECOR]\nStrength=100\nKeepAlive=no\n[BASE]\nStrength=100\n\
    [CombatDamage]\nC4Warhead=Super\n[Warheads]\n0=Super\n\
    [Super]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n";

pub(crate) fn component() -> Vec<u8> {
    std::fs::read(
        std::env::var_os("VERA_SDK_SURVIVAL_WASM")
            .expect("set VERA_SDK_SURVIVAL_WASM to the category candidate"),
    )
    .unwrap()
}

pub(crate) fn rules(patch: &str, component: Option<&[u8]>) -> RuleSet {
    let mut layers = RulesLayerStack::new(IniFile::from_str(BASE));
    layers.push(RulesLayerKind::Scenario, IniFile::from_str(patch));
    let processed = layers.process().unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    if let Some(bytes) = component {
        let names = (0..rules.object_count())
            .map(|index| rules.object_by_handle(TypeHandle(index as u32)).id.clone())
            .collect::<Vec<_>>();
        let digest = crate::util::sha256::sha256_hex(bytes);
        rules.external_mod = Some(Arc::new(
            Program::load_with_registries(
                bytes.to_vec(),
                &digest,
                processed.sdk_passes(),
                &names,
                &["Super".into()],
                &rules.weapon_names(),
                processed.sdk_registries(),
            )
            .unwrap(),
        ));
    }
    rules.bind_resource_policies().unwrap();
    rules
}

pub(crate) fn world(rules: &RuleSet) -> (Simulation, InternedId, InternedId) {
    let mut sim = Simulation::with_seed(3);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.session
        .set_game_mode(crate::sim::scenario_session::ScenarioGameMode::from_legacy_nonzero(true));
    sim.session.set_binary_frame_for_test(100);
    sim.session.game_options_mut_for_test().short_game = true;
    let first = sim.interner.intern("First");
    let second = sim.interner.intern("Second");
    for (index, owner) in [first, second].into_iter().enumerate() {
        let country = sim.interner.intern(&format!("Country{index}"));
        sim.houses.register(HouseState::new(
            owner,
            index as i32,
            Some(country),
            true,
            0,
            10,
        ));
    }
    (sim, first, second)
}

fn contribution_keeps_alive(sim: &Simulation, owner: InternedId, rules: &RuleSet) -> bool {
    sim.houses[&owner]
        .tracking()
        .game_alive(rules.survival_rule(), true, [], None, |_| false)
}

#[test]
#[cfg(not(feature = "ares"))]
#[ignore = "requires VERA_SDK_SURVIVAL_WASM and VERA_SDK_PREVIOUS_WASM"]
fn sdk_survival_full_short_gate_reaches_defeat_and_preserves_old_guest_fallback() {
    let current = component();
    let previous = std::fs::read(std::env::var_os("VERA_SDK_PREVIOUS_WASM").unwrap()).unwrap();
    for (label, bytes, tracked) in [
        ("current", Some(current.as_slice()), true),
        ("old guest", Some(previous.as_slice()), false),
        ("no guest", None, false),
    ] {
        let rules = rules("", bytes);
        let (mut sim, first, _) = world(&rules);
        sim.spawn_object_limbo_at_height("MCV1", "Second", 20, 20, 0, 0, &rules)
            .unwrap();
        sim.spawn_object_at_height("DECOR", "First", 10, 10, 0, 0, &rules)
            .unwrap();
        let beacon = sim
            .spawn_object_limbo_at_height("BEACON", "First", 12, 12, 0, 0, &rules)
            .unwrap();
        sim.check_defeat(Some(&rules), None);
        assert!(
            !sim.houses[&first].is_defeated(),
            "{label}: contribution or retail building keeps alive"
        );
        sim.uninit_with_rules(beacon, &rules);
        sim.flush_pending_delete();
        sim.check_defeat(Some(&rules), None);
        assert_eq!(
            sim.houses[&first].is_defeated(),
            tracked,
            "{label}: an excluded building must not keep the tracked rule alive"
        );

        let (mut sim, first, _) = world(&rules);
        sim.spawn_object_limbo_at_height("MCV1", "Second", 20, 20, 0, 0, &rules)
            .unwrap();
        sim.spawn_object_limbo_at_height("MCV4", "First", 10, 10, 0, 0, &rules)
            .unwrap();
        sim.check_defeat(Some(&rules), None);
        assert_eq!(
            !sim.houses[&first].is_defeated(),
            tracked,
            "{label}: fourth BaseUnit slot survives independently of KeepAlive=no"
        );
    }
}

#[test]
#[ignore = "requires VERA_SDK_SURVIVAL_WASM"]
fn sdk_survival_limbo_capture_and_full_snapshot_rebind_frozen_contributions() {
    let bytes = component();
    let rules = rules("", Some(&bytes));
    let (mut sim, first, second) = world(&rules);
    let beacon = sim
        .spawn_object_limbo_at_height("BEACON", "First", 10, 10, 0, 0, &rules)
        .unwrap();
    assert!(contribution_keeps_alive(&sim, first, &rules));
    sim.change_owner(beacon, second);
    assert!(!contribution_keeps_alive(&sim, first, &rules));
    assert!(contribution_keeps_alive(&sim, second, &rules));
    #[cfg(feature = "ares")]
    {
        use crate::rules::object_type::ObjectCategory;
        use crate::rules::superweapon_type::DeliveryType;
        use crate::sim::world::FrameEffects;
        // The supported Script68 conversion owner removes old tracking,
        // refreshes type facts, then adds new tracking (Ares10043703..755).
        assert!(sim.convert_techno_type(
            beacon,
            &DeliveryType {
                category: ObjectCategory::Vehicle,
                id: "OFF".into(),
            },
            &rules,
            FrameEffects::default()
        ));
        assert!(!contribution_keeps_alive(&sim, second, &rules));
        sim.change_owner(beacon, first);
        assert!(!contribution_keeps_alive(&sim, first, &rules));
        assert!(sim.convert_techno_type(
            beacon,
            &DeliveryType {
                category: ObjectCategory::Vehicle,
                id: "BEACON".into(),
            },
            &rules,
            FrameEffects::default()
        ));
        assert!(contribution_keeps_alive(&sim, first, &rules));
        sim.change_owner(beacon, second);
    }
    sim.scenario_rng = crate::sim::rng::SimRng::new(0);
    let before = sim.state_hash();
    let saved = GameSnapshot::save(&sim, 0, 0, "external keep-alive", 0);
    let mut loaded = GameSnapshot::load(&saved).unwrap().sim;
    loaded.restore_after_snapshot_load().unwrap();
    assert_eq!(loaded.state_hash(), before);

    let mut current_rules = self::rules(
        "[VeraFeatures]\nAresKeepAlive=no\n[BEACON]\nKeepAlive=no\n",
        Some(&bytes),
    );
    current_rules.external_mod = loaded
        .external_mod
        .as_ref()
        .map(|program| Arc::new(program.clone()));
    current_rules.bind_resource_policies().unwrap();
    assert_eq!(
        current_rules.survival_rule(),
        vera_mod_api::SurvivalRule::TrackedPresence
    );
    assert!(contribution_keeps_alive(&loaded, second, &current_rules));
    let new_beacon = loaded
        .spawn_object_limbo_at_height("BEACON", "First", 11, 11, 0, 0, &current_rules)
        .unwrap();
    assert!(
        contribution_keeps_alive(&loaded, first, &current_rules),
        "new births use saved overrides too"
    );
    loaded.uninit_with_rules(beacon, &current_rules);
    loaded.flush_pending_delete();
    assert!(!contribution_keeps_alive(&loaded, second, &current_rules));
    loaded.uninit_with_rules(new_beacon, &current_rules);
    loaded.flush_pending_delete();
    assert!(!contribution_keeps_alive(&loaded, first, &current_rules));
}

#[test]
#[ignore = "requires VERA_SDK_SURVIVAL_WASM"]
fn sdk_survival_normal_game_uses_owned_foot_presence_even_with_explicit_no() {
    let bytes = component();
    let rules = rules("[BEACON]\nInsignificant=no\nKeepAlive=no\n", Some(&bytes));
    let (mut sim, first, _) = world(&rules);
    sim.session.game_options_mut_for_test().short_game = false;
    sim.spawn_object_at_height("MCV1", "Second", 20, 20, 0, 0, &rules)
        .unwrap();
    let beacon = sim
        .spawn_object_at_height("BEACON", "First", 10, 10, 0, 0, &rules)
        .unwrap();
    assert!(!contribution_keeps_alive(&sim, first, &rules));
    sim.check_defeat(Some(&rules), None);
    assert!(!sim.houses[&first].is_defeated());
    sim.uninit_with_rules(beacon, &rules);
    sim.flush_pending_delete();
    sim.check_defeat(Some(&rules), None);
    assert!(sim.houses[&first].is_defeated());
}
