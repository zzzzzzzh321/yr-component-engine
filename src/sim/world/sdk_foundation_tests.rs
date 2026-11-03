//! Actual-component immutable ART references through the original Foundation,
//! construction, placement, Mark/Unmark and saved-map authority owners.
use super::*;
use crate::mod_sdk::Program;
use crate::rules::art_data::ArtRegistry;
use crate::rules::foundation::Foundation;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::rules::ruleset::RuleSet;
use crate::sim::movement::locomotor::MovementLayer;
use crate::sim::snapshot::GameSnapshot;
use serde_json::Value;
use std::sync::Arc;
use vera_mod_api::{
    FixedIniSource, FoundationResourceContext, FoundationResourcePolicy, ProductionObjectKind,
    ProductionTypeRef,
};

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(
        std::env::var_os("VERA_SDK_FOUNDATION_WASM").expect("set VERA_SDK_FOUNDATION_WASM"),
    )
    .unwrap()
}

fn configure(
    source: &RulesLayerStack,
    art: &IniFile,
    wasm: &[u8],
) -> (ProcessedRulesLayers, RuleSet) {
    let processed = source.process_with_fixed_art(art).unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    let state = processed.native_registry_state();
    rules.external_mod = Some(Arc::new(
        Program::load_with_process_context(
            wasm.to_vec(),
            &crate::util::sha256::sha256_hex(wasm),
            processed.sdk_passes(),
            &rules
                .all_objects()
                .map(|object| object.id.clone())
                .collect::<Vec<_>>(),
            &rules
                .warheads_iter()
                .map(|warhead| warhead.id.clone())
                .collect::<Vec<_>>(),
            &rules.weapon_names(),
            processed.sdk_registries(),
            state.sdk_configuration_seed().map(Arc::as_ref),
            state.sdk_configuration_steps(),
        )
        .unwrap(),
    ));
    (processed, rules)
}

fn selected(rules: &RuleSet) -> FoundationResourcePolicy {
    rules
        .external_mod
        .as_ref()
        .unwrap()
        .foundation_resource_policy(&FoundationResourceContext {
            building: ProductionTypeRef {
                category: ProductionObjectKind::Building,
                ordinal: 0,
            },
        })
        .unwrap()
        .expect("the Foundation-aware component selects every Building")
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_FOUNDATION_WASM"]
fn sdk_foundation_first_art_reference_survives_off_image_changes_and_maximum_outline() {
    let wasm = component();
    let art = IniFile::from_str(
        "[FIRST]\nImage=VISUAL\nFoundation=Custom\nFoundation.X=2\nFoundation.Y=3\n\
         Foundation.0=0,0\nFoundation.1=1,1\n\
         FoundationOutline.Length=1048576\nFoundationOutline.0=-1,-1\n\
         FoundationOutline.10=7,8\nFoundationOutline.1048575=-2,9\n\
         [SECOND]\nFoundation=Custom\nFoundation.X=1\nFoundation.Y=1\nFoundation.0=0,0\n\
         [VISUAL]\nFoundation=4x4\n[MAX]\nFoundation=2x1\n",
    );
    let mut source = RulesLayerStack::new(IniFile::from_str(
        "[BuildingTypes]\n0=MAX\n[MAX]\nImage=FIRST\nFoundation=4x4\n\
         [VeraFeatures]\nAresFoundation=no\n",
    ));
    source.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[MAX]\nImage=SECOND\n"),
    );
    let (_, mut off) = configure(&source, &art, &wasm);
    off.bind_resource_policies().unwrap();
    assert_eq!(
        selected(&off),
        FoundationResourcePolicy::Stock {
            selector: Foundation::from("2x1").selector(),
        }
    );
    assert_eq!(off.object("MAX").unwrap().foundation.dimensions(), (2, 1));

    source.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresFoundation=yes\n"),
    );
    let (_, mut rules) = configure(&source, &art, &wasm);
    rules.bind_resource_policies().unwrap();
    let FoundationResourcePolicy::ArtSection { reference } = selected(&rules) else {
        panic!("on must recover the first Custom reference retained while off");
    };
    assert_eq!(reference.source, FixedIniSource::FixedArt);
    assert_eq!(reference.source_hash, art.content_hash());
    assert_eq!(
        reference.section, "FIRST",
        "neither the later Rules Image nor ART visual Image chooses geometry"
    );
    let geometry = &rules.object("MAX").unwrap().foundation;
    assert_eq!(geometry.dimensions(), (2, 3));
    assert_eq!(geometry.cell_offsets(), [(0, 0), (1, 1)]);
    let backing = geometry.custom_outline().unwrap();
    assert_eq!(backing.len(), 1_048_577);
    assert_eq!(backing[1], (0x7fff, 0x7fff));
    assert_eq!(
        crate::sim::docking::building_dock::foundation_exit_list(geometry),
        [(-1, -1)]
    );
    assert_eq!(
        crate::sim::docking::building_dock::foundation_exit_pair(geometry, 10),
        Some((7, 8))
    );
    assert_eq!(
        backing[1_048_575],
        (-2, 9),
        "an earlier sentinel must not discard backing slots"
    );
    assert_eq!(backing.last(), Some(&(0x7fff, 0x7fff)));
    let saved_program = serde_json::to_value(rules.external_mod.as_deref().unwrap()).unwrap();
    assert!(
        saved_program["config"].as_array().unwrap().len() < 64 * 1024,
        "million-slot geometry stays with Foundation, outside the frozen guest configuration"
    );

    // The ordinary native projection never eagerly parses an unselected
    // Custom section. Turning the same declaration on must still reject it.
    let broken =
        IniFile::from_str("[BROKEN]\nFoundation=Custom\nFoundation.X=65536\nFoundation.Y=1\n");
    let mut source = RulesLayerStack::new(IniFile::from_str(
        "[BuildingTypes]\n0=BAD\n[BAD]\nImage=BROKEN\n[VeraFeatures]\nAresFoundation=no\n",
    ));
    let (_, mut disabled) = configure(&source, &broken, &wasm);
    disabled.bind_resource_policies().unwrap();
    assert_eq!(
        disabled.object("BAD").unwrap().foundation,
        Foundation::Stock(0)
    );
    source.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresFoundation=yes\n"),
    );
    let (_, mut enabled) = configure(&source, &broken, &wasm);
    assert!(
        enabled
            .bind_resource_policies()
            .unwrap_err()
            .contains("invalid custom foundation")
    );
    assert_eq!(
        enabled.object("BAD").unwrap().foundation,
        Foundation::Stock(0),
        "failed resource admission publishes no partial geometry"
    );
}

fn sparse_scene() -> (RulesLayerStack, IniFile, Value) {
    // Reuse the executed reader's sparse source and expected cells. This test
    // supplies a real component, rather than a second geometry implementation.
    let packet: Value = serde_json::from_str(crate::test_fixture::text(
        "tools/rules_oracle/ares_foundation.json",
    ))
    .unwrap();
    let row = packet["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "sparse_holes")
        .unwrap()
        .clone();
    let mut text = String::from("[SPARSE_ART]\nHeight=3\n");
    for (key, value) in row["authored"].as_object().unwrap() {
        text.push_str(&format!("{key}={}\n", value.as_str().unwrap()));
    }
    text.push_str("[PROBE_ART]\nFoundation=1x1\n");
    let source = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(
        "[Countries]\n0=Americans\n1=Russians\n[Americans]\nSide=GDI\n\
         [BuildingTypes]\n0=SPARSE\n1=PROBE\n\
         [SPARSE]\nImage=SPARSE_ART\nOwner=Americans\nStrength=500\nTechLevel=1\nFoundation=4x4\n\
         [PROBE]\nImage=PROBE_ART\nOwner=Americans\nStrength=100\nTechLevel=1\n",
    ));
    (source, IniFile::from_str(&text), row)
}

fn world(rules: &RuleSet) -> Simulation {
    let mut sim = crate::sim::arena_fixture::rival_arena(rules, 0x568411);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(rules);
    sim.resolve_type_handles(rules);
    let terrain = sim.resolved_terrain.as_ref().unwrap();
    let overlay = crate::sim::overlay_grid::OverlayGrid::new(terrain.width(), terrain.height());
    let smudges = crate::sim::smudge_grid::SmudgeGrid::from_map_entries(
        &[],
        &rules.smudge_types,
        terrain,
        &overlay,
        terrain.width(),
        terrain.height(),
    );
    let (width, height) = sim.map_size_diamond().unwrap();
    let overlays = crate::rules::overlay_types::OverlayTypeRegistry::empty();
    crate::sim::runtime::initialize_native_tiberium_queues(
        &mut sim,
        &Default::default(),
        &Default::default(),
        rules,
        &overlays,
        Some(&overlay),
        (
            u16::try_from(width).unwrap(),
            u16::try_from(height).unwrap(),
        ),
    )
    .unwrap();
    sim.install_overlay_grid_for_load(overlay);
    sim.install_smudge_grid(smudges);
    sim.restore_map_authority_after_snapshot_load(rules, &overlays, None)
        .unwrap();
    sim
}

fn assert_marked_cells(sim: &Simulation, id: u64, occupied: &[(u16, u16)], marked: bool) {
    for &at in occupied.iter().chain(std::iter::once(&(21, 20))) {
        let expected = marked && occupied.contains(&at);
        assert_eq!(
            sim.cell_objects(at, MovementLayer::Ground)
                .any(|member| matches!(member,
            crate::sim::occupancy::CellObjectMember::Entity(found) if found == id)),
            expected
        );
        assert_eq!(
            sim.path_grid_snapshot().unwrap().is_walkable(at.0, at.1),
            !expected
        );
    }
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_FOUNDATION_WASM"]
fn sdk_foundation_sparse_mark_placement_and_exact_saved_reference_use_the_original_owners() {
    let wasm = component();
    let (source, art, row) = sparse_scene();
    let (processed, mut rules) = configure(&source, &art, &wasm);
    rules.bind_resource_policies().unwrap();
    rules.install_art_data(ArtRegistry::from_ini(&art));
    let policy = selected(&rules);
    let geometry = &rules.object("SPARSE").unwrap().foundation;
    assert_eq!(
        serde_json::json!(geometry.dimensions()),
        serde_json::json!([row["installed"]["width"], row["installed"]["height"]])
    );
    let occupied: Vec<_> = row["installed"]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .take_while(|pair| pair[0] != 32767 || pair[1] != 32767)
        .map(|pair| {
            (
                (20 + pair[0].as_i64().unwrap()) as u16,
                (20 + pair[1].as_i64().unwrap()) as u16,
            )
        })
        .collect();
    let mut sim = world(&rules);
    let id = sim
        .spawn_object("SPARSE", "Americans", 20, 20, 0, &rules)
        .unwrap();
    let entity = sim.entities().get(id).unwrap();
    assert!(entity.lifecycle.cell_marked());
    assert_eq!(entity.type_facts.foundation(), geometry);
    assert!(
        !entity.footprint_contains_cell((21, 20)),
        "the declared rectangle contains an unoccupied hole"
    );
    assert!(sim.rebuild_dynamic_navigation(&rules));
    assert_marked_cells(&sim, id, &occupied, true);
    let probe = rules.object("PROBE").unwrap();
    let owner = sim.interner.get("Americans");
    assert!(
        crate::sim::build_site::can_place_building_at(
            &sim,
            &rules,
            None,
            probe,
            (21, 20),
            owner,
            None,
            None,
        ),
        "placement can use the sparse hole"
    );
    assert!(
        !crate::sim::build_site::can_place_building_at(
            &sim,
            &rules,
            None,
            probe,
            (20, 20),
            owner,
            None,
            None,
        ),
        "the original Mark cell list blocks an occupied cell"
    );

    let terrain = sim.resolved_terrain.as_ref().unwrap().clone();
    let mut restored = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-FOUNDATION", 0))
        .unwrap()
        .sim;
    restored
        .external_mod
        .as_ref()
        .unwrap()
        .validate_restore()
        .unwrap();
    assert_eq!(
        crate::util::save_codec::serialize(restored.external_mod.as_ref().unwrap()).unwrap(),
        crate::util::save_codec::serialize(sim.external_mod.as_ref().unwrap()).unwrap()
    );
    let retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(&processed).unwrap(),
    )
    .unwrap();
    let mut rebound = RuleSet::from_processed_rules(&retained).unwrap();
    rebound.external_mod = restored
        .external_mod
        .as_ref()
        .map(|program| Arc::new(program.clone()));
    rebound.bind_fixed_art_source(Arc::new(art.clone()));
    rebound.bind_resource_policies().unwrap();
    rebound.install_art_data(ArtRegistry::from_ini(&art));
    assert_eq!(selected(&rebound), policy);
    assert_eq!(
        rebound.simulation_config_hash(),
        rules.simulation_config_hash()
    );
    restored.retain_process_state_for_load(SnapshotProcessState::Live(&sim));
    restored.bind_shared_cell_dummy(crate::map::resolved_terrain::SharedCellDummy::fresh());
    restored.reconstruct_cellclass_dummy_for_map_resize();
    restored.restore_after_snapshot_load().unwrap();
    restored.rebuild_caches_after_load(terrain, sim.terrain_speed_config().clone(), &rebound);
    restored
        .restore_map_authority_after_snapshot_load(
            &rebound,
            &crate::rules::overlay_types::OverlayTypeRegistry::empty(),
            None,
        )
        .unwrap();
    restored.resolve_type_handles(&rebound);
    assert_eq!(
        restored.entities().get(id).unwrap().type_facts.foundation(),
        geometry
    );
    assert_marked_cells(&restored, id, &occupied, true);

    let mut wrong = RuleSet::from_processed_rules(&retained).unwrap();
    wrong.external_mod = rebound.external_mod.clone();
    let mut changed_art = art.clone();
    changed_art.merge(&IniFile::from_str(
        "[SPARSE_ART]\nFoundationOutline.0=9,9\n",
    ));
    wrong.bind_fixed_art_source(Arc::new(changed_art));
    let before = wrong.object("SPARSE").unwrap().foundation.clone();
    assert!(
        wrong
            .bind_resource_policies()
            .unwrap_err()
            .contains("foundation source differs from fixed ART")
    );
    assert_eq!(wrong.object("SPARSE").unwrap().foundation, before);

    restored.techno_limbo_with_rules(id, &rebound, None);
    let entity = restored.entities().get(id).unwrap();
    assert!(entity.lifecycle.in_limbo());
    assert!(!entity.lifecycle.cell_marked());
    assert!(restored.rebuild_dynamic_navigation(&rebound));
    assert_marked_cells(&restored, id, &occupied, false);
    assert_eq!(restored.substrate.hidden_occupation.entry_count(), 0);
    assert!(crate::sim::build_site::can_place_building_at(
        &restored,
        &rebound,
        None,
        rebound.object("PROBE").unwrap(),
        (20, 20),
        owner,
        None,
        None,
    ));
    restored.debug_assert_logic_membership_consistent();
}
