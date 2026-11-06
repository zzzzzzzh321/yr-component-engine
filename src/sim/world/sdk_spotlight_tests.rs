//! Actual-component Type spotlight policies through the original independent
//! light constructor, AI, drawing descriptor, expiry and saved-world owners.
use super::*;
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::rules::ruleset::RuleSet;
use crate::sim::snapshot::GameSnapshot;
use std::sync::Arc;

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os("VERA_SDK_SPOTLIGHT_WASM").expect("set VERA_SDK_SPOTLIGHT_WASM"))
        .unwrap()
}

const SCENE: &str = "\
[General]\nSpotlightMovementRadius=2000\nSpotlightLocationRadius=1000\n\
SpotlightSpeed=.015\nSpotlightAcceleration=.0025\nSpotlightAngle=.5\nCloakingStages=2\n\
[BuildingTypes]\n0=SPACE\n1=STOCK\n[VehicleTypes]\n0=BODY\n1=TURRET\n\
[SPACE]\nStrength=100\nHasSpotlight=  yes\nSpotlight.StartHeight=650\n\
[STOCK]\nStrength=100\nHasSpotlight=yes\nSpotlight.StartHeight=900\nSpotlight.DisableRed=yes\n\
[BODY]\nStrength=120\nSpeed=6\nROT=6\nMovementZone=Normal\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\nHasSpotlight= yes\n\
Spotlight.AttachedTo=body\nSpotlight.Distance=800\nSpotlight.StartHeight=650\nSpotlight.DisableGreen=yes\n\
[TURRET]\nStrength=120\nSpeed=6\nROT=6\nTurret=yes\nMovementZone=Normal\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\nHasSpotlight=yes\n\
Cloakable=yes\nCloakingSpeed=1\nSpotlight.AttachedTo=turret\n\
Spotlight.StartHeight=600\nSpotlight.DisableBlue=yes\n";

fn layers() -> RulesLayerStack {
    let mut layers = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(SCENE));
    layers.push(RulesLayerKind::Scenario, IniFile::from_str(
        "[SPACE]\nHasSpotlight=invalid\nSpotlight.StartHeight=invalid\nSpotlight.DisableGreen=yes\n\
         [BODY]\nSpotlight.DisableRed=yes\n[TURRET]\nSpotlight.StartHeight=700\nSpotlight.AttachedTo= barrel\n",
    ));
    layers
}

fn admit(layers: &RulesLayerStack, wasm: Option<&[u8]>) -> (ProcessedRulesLayers, RuleSet) {
    let processed = layers.process().unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    if let Some(wasm) = wasm {
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
    }
    rules.bind_resource_policies().unwrap();
    (processed, rules)
}

fn world(rules: &RuleSet) -> Simulation {
    let mut sim = crate::sim::arena_fixture::rival_arena(rules, 37);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(rules);
    sim.resolve_type_handles(rules);
    sim.session
        .set_current_house_for_test(sim.interner.get("Americans"));
    // A complete empty map authority supports the formal saved-resource seam,
    // beyond the movement-only arena's terrain/navigation input.
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

fn spawn(sim: &mut Simulation, rules: &RuleSet, name: &str, x: u16) -> u64 {
    let id = sim
        .spawn_object(name, "Americans", x, 10, 0, rules)
        .unwrap();
    if sim.entities().get(id).unwrap().category() == EntityCategory::Structure {
        sim.visit_building_operational(id, rules, None, FrameEffects::empty());
    }
    id
}

fn child(sim: &Simulation, parent: u64) -> u64 {
    sim.entities()
        .get(parent)
        .unwrap()
        .building_light
        .light()
        .unwrap()
}

fn visit(sim: &mut Simulation, id: u64, rules: &RuleSet) {
    assert!(sim.object_ai_visit_one(id, Some(rules), ObjectAiCtx::default()));
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_SPOTLIGHT_WASM"]
fn sdk_spotlight_physical_layers_and_explicit_off_preserve_stock_building_lights() {
    let wasm = component();
    let mut source = layers();
    let (physical, bare) = admit(&source, None);
    assert_eq!(
        physical.sdk_passes()[0]
            .section("SPACE")
            .unwrap()
            .get_for_test("HasSpotlight"),
        Some("yes"),
        "the physical INI parser trims values before either bool reader sees them"
    );
    assert!(bare.object("SPACE").unwrap().has_spotlight());
    assert!(bare.object("STOCK").unwrap().has_spotlight());

    let (_, enabled) = admit(&source, Some(&wasm));
    let mut sim = world(&enabled);
    let space = spawn(&mut sim, &enabled, "SPACE", 10);
    let light = child(&sim, space);
    visit(&mut sim, light, &enabled);
    let draw = sim
        .building_spotlight(light, sim.session().current_house(), &enabled)
        .unwrap();
    let z = crate::sim::movement::ground_pose::object_get_coords(
        sim.entities().get(space).unwrap(),
        sim.resolved_terrain.as_ref(),
    )
    .z;
    assert_eq!(draw.beam_origin[2], z + 650);
    assert_eq!(draw.glow_flags, 4);
    let body = spawn(&mut sim, &enabled, "BODY", 14);
    let body_light = child(&sim, body);
    visit(&mut sim, body_light, &enabled);
    assert!(
        sim.building_spotlight(body_light, sim.session().current_house(), &enabled)
            .is_some()
    );

    source.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresSpotlight=no\n"),
    );
    let (_, disabled) = admit(&source, Some(&wasm));
    let mut off = world(&disabled);
    let stock = spawn(&mut off, &disabled, "STOCK", 10);
    let stock_light = child(&off, stock);
    visit(&mut off, stock_light, &disabled);
    let draw = off
        .building_spotlight(stock_light, off.session().current_house(), &disabled)
        .unwrap();
    let z = crate::sim::movement::ground_pose::object_get_coords(
        off.entities().get(stock).unwrap(),
        off.resolved_terrain.as_ref(),
    )
    .z;
    assert_eq!(draw.beam_origin[2], z + 430);
    assert_eq!(
        draw.glow_flags, 0,
        "off uses original BuildingLight presentation"
    );
    let space = spawn(&mut off, &disabled, "SPACE", 14);
    let body = spawn(&mut off, &disabled, "BODY", 18);
    let space_light = child(&off, space);
    visit(&mut off, space_light, &disabled);
    let draw = off
        .building_spotlight(space_light, off.session().current_house(), &disabled)
        .unwrap();
    let z = crate::sim::movement::ground_pose::object_get_coords(
        off.entities().get(space).unwrap(),
        off.resolved_terrain.as_ref(),
    )
    .z;
    assert_eq!(draw.beam_origin[2], z + 430);
    assert_eq!(
        draw.glow_flags, 0,
        "off retains the original Building light after physical whitespace normalization"
    );
    assert!(
        off.entities()
            .get(body)
            .unwrap()
            .building_light
            .light()
            .is_none(),
        "stock Foot does not construct a BuildingLight child"
    );
    off.debug_assert_logic_membership_consistent();
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_SPOTLIGHT_WASM"]
fn sdk_spotlight_foot_facing_detach_activation_and_saved_child_share_original_owners() {
    let wasm = component();
    let (processed, rules) = admit(&layers(), Some(&wasm));
    let mut sim = world(&rules);
    let body = spawn(&mut sim, &rules, "BODY", 10);
    let turret = spawn(&mut sim, &rules, "TURRET", 14);
    let body_light = child(&sim, body);
    let turret_light = child(&sim, turret);

    // Unit737BA0 only establishes state2 outside the playfield. This normally
    // placed Unit starts visible; two real Techno AI visits drive the original
    // cloak transition at the authored speed/stage count, retaining its light.
    assert!(sim.entities().get(turret).unwrap().in_playfield());
    assert_eq!(
        sim.entities()
            .get(turret)
            .unwrap()
            .cloak
            .as_ref()
            .unwrap()
            .state(),
        0
    );
    visit(&mut sim, turret, &rules);
    assert_eq!(
        sim.entities()
            .get(turret)
            .unwrap()
            .cloak
            .as_ref()
            .unwrap()
            .state(),
        1
    );
    sim.session
        .set_binary_frame_for_test(sim.session.binary_frame() + 1);
    visit(&mut sim, turret, &rules);
    assert!(
        sim.entities()
            .get(turret)
            .unwrap()
            .cloak
            .as_ref()
            .unwrap()
            .is_fully_cloaked()
    );
    assert!(
        sim.building_spotlight(turret_light, sim.session().current_house(), &rules)
            .is_none()
    );
    sim.detach_all_pointer_expired(turret, &rules, None, FrameEffects::empty());
    assert_eq!(child(&sim, turret), turret_light);
    assert!(sim.substrate.spotlights.contains_key(turret_light));
    crate::sim::combat::world_fire::start_uncloaking_to_fire(&mut sim, &rules, turret);
    sim.session
        .set_binary_frame_for_test(sim.session.binary_frame() + 1);
    visit(&mut sim, turret, &rules);
    visit(&mut sim, turret_light, &rules);
    assert!(
        sim.building_spotlight(turret_light, sim.session().current_house(), &rules)
            .is_some(),
        "the same child regains a drawable parent after the real uncloak visit"
    );

    let frame = sim.session.binary_frame();
    sim.substrate
        .entities
        .get_mut(body)
        .unwrap()
        .body_facing
        .snap(0, frame);
    let actor = sim.substrate.entities.get_mut(turret).unwrap();
    actor.body_facing.snap(0, frame);
    actor.barrel_facing.as_mut().unwrap().snap(0, frame);
    visit(&mut sim, body_light, &rules);
    visit(&mut sim, turret_light, &rules);
    let body_pose = sim
        .building_spotlight(body_light, sim.session().current_house(), &rules)
        .unwrap();
    let secondary_pose = sim
        .building_spotlight(turret_light, sim.session().current_house(), &rules)
        .unwrap();
    assert_eq!(body_pose.glow_flags, 6);
    assert_eq!(secondary_pose.glow_flags, 8);
    let actor_z = crate::sim::movement::ground_pose::position_world_coord(
        &sim.entities().get(turret).unwrap().position,
    )
    .z;
    assert_eq!(secondary_pose.beam_origin[2], actor_z + 700);
    sim.substrate
        .entities
        .get_mut(body)
        .unwrap()
        .body_facing
        .snap(0x4000, frame);
    sim.substrate
        .entities
        .get_mut(turret)
        .unwrap()
        .body_facing
        .snap(0x4000, frame);
    visit(&mut sim, body_light, &rules);
    visit(&mut sim, turret_light, &rules);
    assert_ne!(
        sim.building_spotlight(body_light, sim.session().current_house(), &rules)
            .unwrap()
            .coord,
        body_pose.coord
    );
    assert_eq!(
        sim.building_spotlight(turret_light, sim.session().current_house(), &rules)
            .unwrap()
            .coord,
        secondary_pose.coord,
        "the installed Barrel selector follows Secondary, independently of Body"
    );
    sim.substrate
        .entities
        .get_mut(turret)
        .unwrap()
        .barrel_facing
        .as_mut()
        .unwrap()
        .snap(0x4000, frame);
    visit(&mut sim, turret_light, &rules);
    assert_ne!(
        sim.building_spotlight(turret_light, sim.session().current_house(), &rules)
            .unwrap()
            .coord,
        secondary_pose.coord
    );

    sim.techno_deactivate(turret, &rules, FrameEffects::empty());
    assert!(
        sim.entities()
            .get(turret)
            .unwrap()
            .building_light
            .light()
            .is_none()
    );
    assert!(!sim.substrate.spotlights.contains_key(turret_light));
    sim.techno_reactivate(turret, &rules, FrameEffects::empty());
    let recreated = child(&sim, turret);
    assert_ne!(recreated, turret_light);
    visit(&mut sim, recreated, &rules);
    let retained_light = sim.substrate.spotlights.get(recreated).unwrap().clone();
    let terrain = sim.resolved_terrain.as_ref().unwrap().clone();
    let mut restored = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-SPOTLIGHT", 0))
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
        crate::util::save_codec::serialize(sim.external_mod.as_ref().unwrap()).unwrap(),
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
    rebound.bind_resource_policies().unwrap();
    restored.retain_process_state_for_load(SnapshotProcessState::Live(&sim));
    restored.bind_shared_cell_dummy(crate::map::resolved_terrain::SharedCellDummy::fresh());
    restored.reconstruct_cellclass_dummy_for_map_resize();
    restored.restore_after_snapshot_load().unwrap();
    restored.rebuild_caches_after_load(terrain, sim.terrain_speed_config.clone(), &rebound);
    restored
        .restore_map_authority_after_snapshot_load(
            &rebound,
            &crate::rules::overlay_types::OverlayTypeRegistry::empty(),
            None,
        )
        .unwrap();
    restored.resolve_type_handles(&rebound);
    assert_eq!(child(&restored, turret), recreated);
    assert_eq!(
        restored.substrate.spotlights.get(recreated),
        Some(&retained_light)
    );
    assert!(
        restored
            .building_spotlight(recreated, restored.session().current_house(), &rebound)
            .is_some()
    );
    restored.techno_limbo_with_rules(turret, &rebound, None);
    assert!(!restored.substrate.spotlights.contains_key(recreated));
    restored.uninit_with_rules(body, &rebound);
    restored.flush_pending_delete();
    assert!(!restored.substrate.spotlights.contains_key(body_light));
    assert_eq!(restored.substrate.spotlights.iter().count(), 0);
    restored.debug_assert_logic_membership_consistent();
}
