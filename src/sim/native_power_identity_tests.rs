//! Actual Type construction/admission into House power and stock Super hold.

use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{RulesLayerKind, RulesLayerStack};
use crate::rules::object_type::ObjectCategory;
use crate::rules::ruleset::RuleSet;
use crate::sim::components::DriveCoord;
use crate::sim::world::{FrameEffects, Simulation};

#[test]
fn same_name_building_slots_supply_actual_house_power_and_super_hold() {
    const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";
    let mut layers = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(
        "[Countries]\n0=Americans\n1=Russians\n\
         [BuildingTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n1=LOAD\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nStrength=1000\nPower=200\nOwner=Americans\n\
         [LOAD]\nStrength=1000\nPower=-100\nPowered=yes\nSuperWeapon=Probe\nOwner=Americans\n\
         [SuperWeaponTypes]\n0=Probe\n[Probe]\nType=PsychicReveal\nRechargeTime=1\nIsPowered=yes\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[BuildingTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
             [ABCDEFGHIJKLMNOPQRSTUVWX]\nStrength=1000\nPower=invalidh\n",
        ),
    );
    let art =
        IniFile::from_str("[ABCDEFGHIJKLMNOPQRSTUVWX]\nFoundation=1x1\n[LOAD]\nFoundation=1x1\n");
    let processed = layers.process_with_fixed_art(&art).unwrap();
    let rules = RuleSet::from_processed_rules(&processed).unwrap();
    let first = rules.type_array_at(ObjectCategory::Building, 0).unwrap();
    let later = rules.type_array_at(ObjectCategory::Building, 2).unwrap();
    assert_eq!((first.id.as_str(), later.id.as_str()), (STORED, STORED));
    assert_eq!((first.strength, later.strength), (1000, 1000));
    assert_eq!((first.power, later.power), (200, 0));

    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 0x44e7_b500);
    sim.intern_rule_type_ids(&rules);
    sim.resolve_type_handles(&rules);
    let owner = sim.interner.get("Americans").unwrap();
    sim.session.set_current_house_for_test(Some(owner));
    sim.session
        .set_game_mode(crate::sim::scenario_session::ScenarioGameMode::from_legacy_nonzero(true));
    crate::sim::superweapon::construct_house_supers(&mut sim, &rules, owner, false);
    let probe = sim.interner.get("Probe").unwrap();
    sim.spawn_object("LOAD", "Americans", 14, 14, 0, &rules)
        .unwrap();

    let place = |sim: &mut Simulation, object, x| {
        let id = sim
            .construct_resolved_object_limbo_at_height(
                object,
                "Americans",
                x,
                14,
                0,
                0,
                &rules,
                FrameEffects::default(),
            )
            .unwrap();
        assert_eq!(
            sim.reveal_finished_object_at_coord(
                id,
                DriveCoord::cell(x, 14, 0),
                false,
                &rules,
                None,
                FrameEffects::default(),
            ),
            Some(id)
        );
        id
    };
    let zero_output = place(&mut sim, later, 18);
    assert!(std::ptr::eq(
        sim.entity_object_type(sim.entities().get(zero_output).unwrap(), &rules)
            .unwrap(),
        later,
    ));
    // Real House508CF4/508D09 assessment must consume the receiver's Type
    // pointer (Building+520), not the first stored-ID alias's Power=200.
    sim.advance_tick(&[], Some(&rules), None, None, 67);
    assert_eq!(sim.power_states[&owner].total_output(), 0);
    assert_eq!(sim.power_states[&owner].total_drain(), 100);
    assert!(sim.power_states[&owner].is_low_power());
    assert!(sim.super_weapons[&owner][&probe].is_active());
    assert!(sim.super_weapons[&owner][&probe].is_suspended());

    place(&mut sim, first, 22);
    sim.advance_tick(&[], Some(&rules), None, None, 67);
    assert_eq!(sim.power_states[&owner].total_output(), 200);
    assert_eq!(sim.power_states[&owner].total_drain(), 100);
    assert!(!sim.power_states[&owner].is_low_power());
    assert!(!sim.super_weapons[&owner][&probe].is_suspended());
}
