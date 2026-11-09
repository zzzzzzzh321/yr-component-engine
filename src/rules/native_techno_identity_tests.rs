//! Actual Process allocations, their independent retained bodies and aliases.

use super::*;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::util::save_codec;

#[test]
fn native_techno_slots_keep_separate_bodies_weapons_and_saved_identity() {
    const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[InfantryTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nCost=40\nTechLevel=7\nImage=FIRST\nPrimary=Gun\n\
         [Gun]\nDamage=5\n",
    ));
    // Full25 misses the stored24 Find alias and constructs a second Type.
    // A late member reads only this pass, retaining its own constructor
    // defaults after the invalid ints (ReadInt714570 and Abstract41088D).
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[InfantryTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
             [ABCDEFGHIJKLMNOPQRSTUVWX]\nCost=invalidh\nTechLevel=invalidh\n",
        ),
    );
    let processed = layers.process().unwrap();
    let rules = RuleSet::from_processed_rules(&processed).unwrap();
    let first = rules.type_array_at(ObjectCategory::Infantry, 0).unwrap();
    let second = rules.type_array_at(ObjectCategory::Infantry, 1).unwrap();
    assert_eq!(
        rules.type_array_ids(ObjectCategory::Infantry),
        [STORED, STORED]
    );
    assert_eq!(
        (first.native_type_index(), second.native_type_index()),
        (0, 1)
    );
    assert_eq!((first.cost, second.cost), (40, 0));
    assert_eq!((first.tech_level, second.tech_level), (7, 255));
    assert_eq!(
        (first.image.as_str(), second.image.as_str()),
        ("FIRST", STORED)
    );
    assert_eq!(first.weapon_list[0].as_deref(), Some("Gun"));
    assert_eq!(second.weapon_list[0], None);
    assert_eq!(
        rules.resolved_weapon_handle("Gun", first.resolved_weapons().slot(0, false)),
        rules.weapon_handle("Gun")
    );
    assert_eq!(
        rules.resolved_weapon_handle("Gun", second.resolved_weapons().slot(0, false)),
        None
    );
    assert!(std::ptr::eq(rules.object(STORED).unwrap(), first));
    assert!(std::ptr::eq(
        rules.native_techno_type_exact(STORED).unwrap(),
        second
    ));
    assert!(rules.type_array_at(ObjectCategory::Infantry, 2).is_none());
    assert!(rules.type_array_at(ObjectCategory::Infantry, -1).is_none());

    let saved: ProcessedRulesLayers =
        save_codec::deserialize(&save_codec::serialize(&processed).unwrap()).unwrap();
    let restored = RuleSet::from_processed_rules(&saved).unwrap();
    assert_eq!(
        restored.simulation_config_hash(),
        rules.simulation_config_hash()
    );
    assert_eq!(
        restored
            .type_array_at(ObjectCategory::Infantry, 0)
            .unwrap()
            .cost,
        40
    );
    assert_eq!(
        restored
            .native_techno_type_exact(STORED)
            .unwrap()
            .native_type_index(),
        1
    );

    // A state domain previously collected by stored name must hash both
    // slots, including a change to the first rather than the last alias.
    let mut changed = rules.clone();
    let first_handle = changed.native_type_arrays[ObjectCategory::Infantry as usize][0];
    changed.object_list[first_handle.0 as usize].radial_fire_segments += 1;
    assert_ne!(
        changed.simulation_config_hash(),
        rules.simulation_config_hash()
    );
}

#[test]
fn native_techno_constructor_only_slots_and_audio_keep_their_physical_source() {
    const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[VehicleTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nImage=FIRST\nVoiceSelect=Earlier\n\
         [InfantryTypes]\n0=CLASH\n[BuildingTypes]\n0=CLASH\n[CLASH]\nCost=5\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[VehicleTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n1=UNREAD\n\
             [ABCDEFGHIJKLMNOPQRSTUVWX]\nCost=1\n",
        ),
    );
    let processed = layers
        .process_with_fixed_art(&IniFile::from_str("[FIRST]\nFiringFrames=3\n"))
        .unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    let sounds = crate::rules::sound_ini::SoundRegistry::from_ini(&IniFile::from_str(
        "[SoundList]\n0=Earlier\n[Earlier]\nSounds=earlier\n",
    ));
    rules.bind_type_sound_references(processed.ini(), &sounds);
    let first = rules.type_array_at(ObjectCategory::Vehicle, 0).unwrap();
    let second = rules.type_array_at(ObjectCategory::Vehicle, 1).unwrap();
    assert_eq!((first.id.as_str(), second.id.as_str()), (STORED, STORED));
    assert_eq!(first.voice_select, ["Earlier"]);
    assert!(second.voice_select.is_empty());
    assert_eq!(
        rules
            .unit_shp_read_state_for_type(first)
            .unwrap()
            .firing_frames(),
        3
    );
    assert_eq!(
        rules
            .unit_shp_read_state_for_type(second)
            .unwrap()
            .firing_frames(),
        0
    );
    let unread = rules.type_array_at(ObjectCategory::Vehicle, 2).unwrap();
    assert_eq!(
        (
            unread.id.as_str(),
            unread.native_type_index(),
            unread.tech_level
        ),
        ("UNREAD", 2, 255)
    );
    assert_eq!(unread.cost, 0);
    assert_eq!(unread.image, "UNREAD");
    assert_eq!(
        rules.unit_shp_read_state_for_type(unread),
        Some(&crate::rules::shp_vehicle_sequence::UnitShpReadState::default())
    );
    assert_eq!(
        rules.object("CLASH").unwrap().category,
        ObjectCategory::Building
    );
    assert_eq!(
        rules
            .object_in_category(ObjectCategory::Infantry, "CLASH")
            .unwrap()
            .native_type_index(),
        0
    );
    assert_eq!(
        rules.native_techno_type_exact("CLASH").unwrap().category,
        ObjectCategory::Infantry
    );
    assert!(
        rules
            .native_techno_type_order
            .iter()
            .all(|(_, _, handle)| handle.is_some())
    );
}
