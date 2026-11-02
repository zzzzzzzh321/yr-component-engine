//! External input-policy consumers. Native action rungs are cited at their
//! production owners; these checks cover visible cursor/order behavior and
//! keep the simulation admission owners independent of manual restrictions.

use super::*;
use crate::app::input::cursor::capability_cursor_for_hover;
use crate::app::types::CursorFeedbackKind;
use crate::mod_sdk::InputPolicy;
use crate::rules::ruleset::RuleSet;
use crate::sim::passenger::{PassengerCargo, PassengerRole};
use crate::sim::world::Simulation;

fn rules() -> RuleSet {
    RuleSet::from_ini(&crate::sim::arena_fixture::clear_ground_ini(
        "[InfantryTypes]\n0=SEAL\n1=SPY\n2=MEDIC\n3=GI\n\
         [VehicleTypes]\n0=TRANSPORT\n[AircraftTypes]\n0=PLANE\n\
         [BuildingTypes]\n0=PLANT\n1=GRINDER\n\
         [SEAL]\nStrength=100\nC4=yes\nPrimary=RIFLE\nDeployer=yes\nDeployFire=no\n\
         [SPY]\nStrength=100\nAgent=yes\nDeployer=yes\nDeployFire=no\n\
         [MEDIC]\nStrength=100\nPrimary=HEAL\nDeployer=yes\nDeployFire=yes\n\
         [GI]\nStrength=100\nSize=1\nPrimary=RIFLE\n\
         [TRANSPORT]\nStrength=200\nPassengers=3\nSizeLimit=2\n\
         [PLANE]\nStrength=200\nPassengers=3\nSizeLimit=2\n\
         [PLANT]\nStrength=500\nSpyable=yes\nCapturable=yes\nCanC4=yes\n\
         [GRINDER]\nStrength=500\nGrinding=yes\nPassengers=2\nSizeLimit=2\n\
         [RIFLE]\nDamage=10\nRange=8\nProjectile=Invisible\nWarhead=AP\n\
         [HEAL]\nDamage=-5\nRange=8\nProjectile=Invisible\nWarhead=AP\n\
         [Invisible]\nInviso=yes\n\
         [AP]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
    ))
    .unwrap()
}

fn arena(rules: &RuleSet) -> Simulation {
    let mut sim = crate::sim::arena_fixture::rival_arena(rules, 0x51E748);
    sim.intern_rule_type_ids(rules);
    sim.resolve_type_handles(rules);
    let player = sim.interner().get("Americans").unwrap();
    sim.session_mut_for_test()
        .set_current_house_for_test(Some(player));
    sim.session_mut_for_test()
        .set_game_mode(crate::sim::scenario_session::ScenarioGameMode::from_legacy_nonzero(true));
    sim.fog_mut_for_test().reveal_all_for_owner(player);
    sim
}

fn hover(id: u64, kind: HoverTargetKind) -> HoverTargetKindWithId {
    HoverTargetKindWithId {
        stable_id: id,
        kind,
    }
}

fn cursor(
    sim: &Simulation,
    rules: &RuleSet,
    actor: u64,
    target: &HoverTargetKindWithId,
) -> CursorFeedbackKind {
    capability_cursor_for_hover(
        sim,
        &[actor],
        Some(actor),
        target,
        Some(rules),
        None,
        OrderModifier::Normal,
        false,
        false,
    )
}

#[test]
fn deployed_c4_uses_early_attack_while_spy_keeps_the_later_deployment_gate() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let seal = sim
        .spawn_object("SEAL", "Americans", 5, 5, 0, &rules)
        .unwrap();
    let spy = sim
        .spawn_object("SPY", "Americans", 6, 5, 0, &rules)
        .unwrap();
    let plant = sim
        .spawn_object("PLANT", "Russians", 10, 5, 0, &rules)
        .unwrap();
    for id in [seal, spy] {
        sim.entities_mut_for_test()
            .get_mut(id)
            .unwrap()
            .mission_leaf
            .set_infantry_doing_verified(28)
            .unwrap();
    }
    let target = hover(plant, HoverTargetKind::EnemyStructure);
    // Infantry51E9F1 C4 is terminal before51EB81 deployment; infiltration
    // at51EE62 remains after that deployed-state rejection.
    assert_eq!(
        attack_action(&sim, &rules, seal, &TargetKind::Entity(plant), false, None),
        AttackAction::NoMove
    );
    assert_eq!(
        cursor(&sim, &rules, seal, &target),
        CursorFeedbackKind::Demolish
    );
    assert_eq!(
        infantry_c4_orders(&sim, &rules, &[seal], &target),
        vec![Command::PlantC4 {
            attacker_id: seal,
            target_building_id: plant
        }]
    );
    assert!(!infantry_infiltration_action(&sim, &rules, spy, plant));
    rules.bind_input_policy_for_test(
        "SEAL",
        InputPolicy {
            deny_manual_attack: true,
            ..Default::default()
        },
    );
    assert!(infantry_c4_orders(&sim, &rules, &[seal], &target).is_empty());
    assert_ne!(
        cursor(&sim, &rules, seal, &target),
        CursorFeedbackKind::Demolish
    );
}

#[test]
fn no_manual_fire_reaches_c4_and_spy_consumers_without_changing_simulation_admission() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let seal = sim
        .spawn_object("SEAL", "Americans", 5, 5, 0, &rules)
        .unwrap();
    let spy = sim
        .spawn_object("SPY", "Americans", 6, 5, 0, &rules)
        .unwrap();
    let target = sim
        .spawn_object("PLANT", "Russians", 10, 5, 0, &rules)
        .unwrap();
    let target = hover(target, HoverTargetKind::EnemyStructure);
    assert_eq!(
        infantry_c4_action(&sim, &rules, seal, &target, false),
        Some(InfantryC4Action::Sabotage)
    );
    assert!(infantry_infiltration_action(
        &sim,
        &rules,
        spy,
        target.stable_id
    ));
    assert_eq!(
        cursor(&sim, &rules, seal, &target),
        CursorFeedbackKind::Demolish
    );
    assert_eq!(
        cursor(&sim, &rules, spy, &target),
        CursorFeedbackKind::Enter
    );
    let policy = InputPolicy {
        deny_manual_attack: true,
        ..InputPolicy::default()
    };
    rules.bind_input_policy_for_test("SEAL", policy);
    rules.bind_input_policy_for_test("SPY", policy);
    let before = (sim.state_hash(), sim.rng_state());
    assert_eq!(infantry_c4_action(&sim, &rules, seal, &target, false), None);
    assert!(!infantry_infiltration_action(
        &sim,
        &rules,
        spy,
        target.stable_id
    ));
    let mut selected = vec![spy];
    assert_eq!(
        infiltrator_capture_orders(&sim, &rules, &mut selected, &target),
        None
    );
    assert_eq!(selected, [spy]);
    assert_ne!(
        cursor(&sim, &rules, seal, &target),
        CursorFeedbackKind::Demolish
    );
    assert_ne!(
        cursor(&sim, &rules, spy, &target),
        CursorFeedbackKind::Enter
    );
    assert!(
        sim.infiltrator_building_action(spy, target.stable_id, &rules),
        "scripted Capture keeps the simulation owner"
    );
    assert_eq!(
        (sim.state_hash(), sim.rng_state()),
        before,
        "UI queries do not retask or draw RNG"
    );
}

#[test]
fn no_manual_enter_hides_the_passenger_row_but_not_real_transport_admission() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let gi = sim
        .spawn_object("GI", "Americans", 5, 5, 0, &rules)
        .unwrap();
    let carrier = sim
        .spawn_object("TRANSPORT", "Americans", 10, 5, 0, &rules)
        .unwrap();
    let target = hover(carrier, HoverTargetKind::FriendlyUnit);
    assert_eq!(
        entry_action(
            &sim,
            &rules,
            gi,
            carrier,
            OrderModifier::Normal,
            false,
            None
        ),
        Some(crate::sim::world::EntryClickAction::Enter)
    );
    assert_eq!(cursor(&sim, &rules, gi, &target), CursorFeedbackKind::Enter);
    rules.bind_input_policy_for_test(
        "TRANSPORT",
        InputPolicy {
            hide_transport_entry: true,
            ..InputPolicy::default()
        },
    );
    let before = (sim.state_hash(), sim.rng_state());
    assert_eq!(
        entry_action(
            &sim,
            &rules,
            gi,
            carrier,
            OrderModifier::Normal,
            false,
            None
        ),
        None
    );
    assert_eq!(
        entry_action(
            &sim,
            &rules,
            gi,
            carrier,
            OrderModifier::ForceMove,
            true,
            None
        ),
        None
    );
    assert_ne!(cursor(&sim, &rules, gi, &target), CursorFeedbackKind::Enter);
    assert!(
        crate::sim::passenger::can_enter_transport(&sim, &rules, gi, carrier),
        "AI/scripts still use real transport admission"
    );
    assert_eq!((sim.state_hash(), sim.rng_state()), before);
    let grinder = sim
        .spawn_object("GRINDER", "Americans", 15, 5, 0, &rules)
        .unwrap();
    rules.bind_input_policy_for_test(
        "GRINDER",
        InputPolicy {
            hide_transport_entry: true,
            ..InputPolicy::default()
        },
    );
    assert_eq!(
        entry_action(
            &sim,
            &rules,
            gi,
            grinder,
            OrderModifier::Normal,
            false,
            None
        ),
        Some(crate::sim::world::EntryClickAction::Grind),
        "skipping the passenger row preserves the later grinder row"
    );
}

#[test]
fn no_manual_unload_hides_empty_and_loaded_self_actions_and_leaves_cargo_intact() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let unit = sim
        .spawn_object("TRANSPORT", "Americans", 10, 5, 0, &rules)
        .unwrap();
    let aircraft = sim
        .spawn_object_limbo_at_height("PLANE", "Americans", 15, 5, 0, 0, &rules)
        .unwrap();
    let owner = sim.session().current_house().unwrap();
    let target = hover(unit, HoverTargetKind::FriendlyUnit);
    assert!(self_answers_no_deploy(
        &sim,
        &rules,
        None,
        owner,
        &[unit],
        &target,
        false
    ));
    let policy = InputPolicy {
        deny_manual_unload: true,
        ..InputPolicy::default()
    };
    rules.bind_input_policy_for_test("TRANSPORT", policy);
    rules.bind_input_policy_for_test("PLANE", policy);
    assert!(!self_answers_no_deploy(
        &sim,
        &rules,
        None,
        owner,
        &[unit],
        &target,
        false
    ));
    for id in [unit, aircraft] {
        let mut cargo = PassengerCargo::new(3, 0.0);
        cargo.board_forced_chain(&[(999, 1.0)]);
        sim.entities_mut_for_test()
            .get_mut(id)
            .unwrap()
            .passenger_role = PassengerRole::transport(cargo);
        let target = hover(id, HoverTargetKind::FriendlyUnit);
        assert_eq!(
            self_click_command(&sim, &rules, None, owner, &[id], Some(&target), false),
            None
        );
        assert!(!self_answers_no_deploy(
            &sim,
            &rules,
            None,
            owner,
            &[id],
            &target,
            false
        ));
        assert!(!matches!(
            cursor(&sim, &rules, id, &target),
            CursorFeedbackKind::Deploy | CursorFeedbackKind::NoDeploy
        ));
        assert_eq!(
            super::super::transport_orders::unload_command(
                sim.entities().get(id).unwrap(),
                rules.object(if id == unit { "TRANSPORT" } else { "PLANE" }),
                Some(owner)
            ),
            Some(Command::UnloadPassengers { transport_id: id }),
            "restriction does not rewrite cargo/unload admission"
        );
    }
}

#[test]
fn no_self_guard_area_restores_the_deploy_successor_for_a_healing_infantryman() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let medic = sim
        .spawn_object("MEDIC", "Americans", 10, 5, 0, &rules)
        .unwrap();
    let owner = sim.session().current_house().unwrap();
    let target = hover(medic, HoverTargetKind::FriendlyUnit);
    let query = |rules: &RuleSet| {
        self_click_command(&sim, rules, None, owner, &[medic], Some(&target), false)
    };
    let guard = Some(Command::Guard {
        entity_id: medic,
        target: Some(TargetKind::Entity(medic)),
    });
    assert_eq!(query(&rules), guard);
    assert_eq!(
        cursor(&sim, &rules, medic, &target),
        CursorFeedbackKind::GuardArea
    );
    rules.bind_input_policy_for_test(
        "MEDIC",
        InputPolicy {
            deny_self_guard_area: true,
            ..InputPolicy::default()
        },
    );
    assert_eq!(
        query(&rules),
        Some(Command::ToggleInfantryDeploy { entity_id: medic })
    );
    assert_eq!(
        cursor(&sim, &rules, medic, &target),
        CursorFeedbackKind::Deploy
    );
    rules.bind_input_policy_for_test("MEDIC", InputPolicy::default());
    assert_eq!(
        query(&rules),
        guard,
        "explicit false restores the original self-guard rung"
    );
}

#[test]
fn mixed_medics_use_one_cursor_owner_then_each_receivers_complete_class_action() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let target = sim
        .spawn_object("MEDIC", "Americans", 10, 5, 0, &rules)
        .unwrap();
    let healer = sim
        .spawn_object("MEDIC", "Americans", 8, 5, 0, &rules)
        .unwrap();
    let spy = sim
        .spawn_object("SPY", "Americans", 7, 5, 0, &rules)
        .unwrap();
    let group = [target, healer, spy];
    for id in group {
        sim.entities_mut_for_test()
            .get_mut(id)
            .unwrap()
            .selection
            .set_selected_for_test(true);
    }
    sim.entities_mut_for_test().get_mut(target).unwrap().health =
        crate::sim::combat::Health::new(50);
    let hover = hover(target, HoverTargetKind::FriendlyUnit);
    let overlays = crate::rules::overlay_types::OverlayTypeRegistry::empty();
    // NoManualFire rejects Techno's attack rung; Infantry's later healer
    // action still supplies Guard26 for self and Heal27 for the other medic.
    rules.bind_input_policy_for_test(
        "MEDIC",
        InputPolicy {
            deny_manual_attack: true,
            ..Default::default()
        },
    );
    assert_eq!(cursor_owner(&sim, &rules, &group, target), Some(target));
    let feedback = capability_cursor_for_hover(
        &sim,
        &group,
        Some(target),
        &hover,
        Some(&rules),
        Some(&overlays),
        OrderModifier::Normal,
        false,
        false,
    );
    assert_eq!(feedback, CursorFeedbackKind::GuardArea);
    let dispatch = |rules: &RuleSet| {
        infantry_class_branch_orders(
            &sim,
            rules,
            &group,
            &group,
            &hover,
            OrderModifier::Normal,
            crate::app::types::OrderMode::Move,
            false,
            &overlays,
        )
        .unwrap()
    };
    let (release, orders) = dispatch(&rules);
    assert_eq!(release, LeftRelease::Dispatch);
    assert_eq!(
        orders.commands,
        [
            Command::Guard {
                entity_id: target,
                target: Some(TargetKind::Entity(target))
            },
            Command::Attack {
                attacker_id: healer,
                target_id: target
            },
        ]
    );
    assert!(
        !orders.receivers.contains(&spy),
        "unrelated receiver keeps its existing class path"
    );
    rules.bind_input_policy_for_test(
        "MEDIC",
        InputPolicy {
            deny_manual_attack: true,
            deny_self_guard_area: true,
            ..Default::default()
        },
    );
    let (release, orders) = dispatch(&rules);
    assert_eq!(release, LeftRelease::Dispatch);
    assert_eq!(
        orders.commands,
        [Command::Attack {
            attacker_id: healer,
            target_id: target
        }]
    );
    assert!(
        !orders.receivers.contains(&target),
        "self continues after the skipped guard rung"
    );

    let rifle = sim
        .spawn_object("GI", "Americans", 6, 5, 0, &rules)
        .unwrap();
    sim.entities_mut_for_test()
        .get_mut(target)
        .unwrap()
        .selection
        .set_selected_for_test(false);
    sim.entities_mut_for_test()
        .get_mut(rifle)
        .unwrap()
        .selection
        .set_selected_for_test(true);
    let group = [healer, rifle];
    assert_eq!(cursor_owner(&sim, &rules, &group, target), Some(rifle));
    let (release, orders) = infantry_class_branch_orders(
        &sim,
        &rules,
        &group,
        &group,
        &hover,
        OrderModifier::Normal,
        crate::app::types::OrderMode::Move,
        false,
        &overlays,
    )
    .unwrap();
    assert_eq!(
        release,
        LeftRelease::Select,
        "the rifle owner's Select decides the whole group release"
    );
    assert!(
        orders.commands.is_empty(),
        "a lower-ranked medic cannot force group dispatch"
    );
}

#[test]
fn infantry_heal_overrides_attack_rejection_but_preserves_the_transport_and_enemy_successors() {
    let mut rules = rules();
    let mut sim = arena(&rules);
    let medic = sim
        .spawn_object("MEDIC", "Americans", 5, 5, 0, &rules)
        .unwrap();
    let patient = sim
        .spawn_object("GI", "Americans", 8, 5, 0, &rules)
        .unwrap();
    let carrier = sim
        .spawn_object("TRANSPORT", "Americans", 10, 5, 0, &rules)
        .unwrap();
    let enemy = sim
        .spawn_object("GI", "Russians", 10, 7, 0, &rules)
        .unwrap();
    sim.entities_mut_for_test().get_mut(patient).unwrap().health =
        crate::sim::combat::Health::new(99);
    rules.bind_input_policy_for_test(
        "MEDIC",
        InputPolicy {
            deny_manual_attack: true,
            ..Default::default()
        },
    );
    let overlays = crate::rules::overlay_types::OverlayTypeRegistry::empty();
    let patient_hover = hover(patient, HoverTargetKind::FriendlyUnit);
    assert_eq!(
        cursor(&sim, &rules, medic, &patient_hover),
        CursorFeedbackKind::InfantryHeal
    );
    assert_eq!(
        crate::app::input::cursor::cursor_id_for_feedback(CursorFeedbackKind::InfantryHeal),
        Some(crate::render::cursor_atlas::CursorId::NoTogglePower)
    );
    assert_eq!(
        object_attack_click_command(
            &sim,
            &rules,
            &overlays,
            sim.session().current_house().unwrap(),
            crate::app::types::OrderMode::Move,
            OrderModifier::GuardArea,
            true,
            medic,
            patient
        ),
        Some(Command::Attack {
            attacker_id: medic,
            target_id: patient
        })
    );
    let carrier_hover = hover(carrier, HoverTargetKind::FriendlyUnit);
    assert_eq!(
        cursor(&sim, &rules, medic, &carrier_hover),
        CursorFeedbackKind::Enter,
        "a Foot with nonzero Passengers continues to the transport entry row"
    );
    assert!(
        infantry_class_branch_orders(
            &sim,
            &rules,
            &[medic],
            &[medic],
            &carrier_hover,
            OrderModifier::Normal,
            crate::app::types::OrderMode::Move,
            false,
            &overlays,
        )
        .is_none(),
        "the healer prefix leaves the ordinary entry command owner reachable"
    );
    assert_eq!(
        entry_orders(
            &sim,
            &rules,
            &[medic],
            &mut vec![medic],
            &carrier_hover,
            OrderModifier::Normal,
            false,
            Some(&overlays),
        ),
        Some(vec![Command::MegaMission {
            entity_id: medic,
            mission: crate::sim::mission::MissionType::Enter,
            target: None,
            destination: Some(TargetKind::Entity(carrier)),
        }])
    );
    assert_eq!(
        capability_cursor_for_hover(
            &sim,
            &[medic],
            Some(medic),
            &carrier_hover,
            Some(&rules),
            Some(&overlays),
            OrderModifier::ForceMove,
            true,
            false
        ),
        CursorFeedbackKind::Enter,
        "Alt's Move survives the healer prefix and reaches entry"
    );
    assert_eq!(
        cursor(
            &sim,
            &rules,
            medic,
            &hover(enemy, HoverTargetKind::EnemyUnit)
        ),
        CursorFeedbackKind::Default,
        "enemy healer action59 keeps the default arrow"
    );
    assert_eq!(
        object_attack_click_command(
            &sim,
            &rules,
            &overlays,
            sim.session().current_house().unwrap(),
            crate::app::types::OrderMode::Move,
            OrderModifier::ForceFire,
            false,
            medic,
            enemy
        ),
        None
    );
}
