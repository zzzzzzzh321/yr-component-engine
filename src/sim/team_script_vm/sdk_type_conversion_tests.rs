//! Actual-component configuration through Script68's original Team cursor,
//! same-object converter and saved-world owners, in the ordinary engine.
use super::*;
use crate::map::entities::EntityCategory;
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::locomotor_type::LocomotorKind;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::rules::ruleset::RuleSet;
use crate::sim::snapshot::GameSnapshot;
use crate::sim::world::{FrameEffects, ObjectAiCtx, Simulation, SnapshotProcessState};
use crate::util::native_x87::NativeF64Bits;
use std::sync::Arc;

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(
        std::env::var_os("VERA_SDK_CONVERSION_WASM").expect("set VERA_SDK_CONVERSION_WASM"),
    )
    .unwrap()
}

const SCENE: &str = "\
[Countries]\n0=Americans\n1=Russians\n\
[BuildingTypes]\n0=EARLY\n1=AFTER\n2=DUP\n3=REFUSED\n\
[VehicleTypes]\n0=OLD\n1=NEW\n2=BAD\n3=DUP\n4=CLEAR\n5=UNSET\n6=SAME\n\
[InfantryTypes]\n0=FOOT\n1=FOOT2\n[Tiberiums]\n0=CONVERT_ORE\n\
[CONVERT_ORE]\nValue=25\nImage=1\n\
[EARLY]\nStrength=100\nConvert.Script=LATE\nFreeUnit=LATE\n\
[AFTER]\nStrength=100\nConvert.Script=LATE\n\
[DUP]\nStrength=100\nSpeed=5\nMovementZone=Normal\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[REFUSED]\nStrength=100\nConvert.Script=EARLY\n\
[LATE]\nStrength=100\nSpeed=5\nMovementZone=Normal\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[OLD]\nStrength=100\nSpeed=5\nSpeedType=Track\nMovementZone=Normal\n\
Ammo=6\nInitialAmmo=5\nConvert.Script=NEW\nHarvester=yes\nStorage=3\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[NEW]\nStrength=200\nSpeed=6\nSpeedType=Hover\nMovementZone=Normal\n\
Ammo=3\nInitialAmmo=0\nROT=8\nTurret=yes\nTurretROT=-1\nStorage=2\n\
Locomotor={4A582742-9839-11d1-B709-00A024DDAFD1}\n\
[BAD]\nStrength=100\nSpeed=5\nMovementZone=Normal\nConvert.Script=DUP\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[CLEAR]\nStrength=100\nSpeed=5\nConvert.Script=NEW\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[UNSET]\nStrength=100\nConvert.Script=MISSING\n\
[SAME]\nStrength=100\nSpeed=5\nAmmo=2\nInitialAmmo=7\nConvert.Script=SAME\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\n\
[FOOT]\nStrength=100\nSpeed=4\nConvert.Script=FOOT2\n\
AttachEffect.Duration=30\nAttachEffect.InitialDelay=0\nAttachEffect.FirepowerMultiplier=2\n\
Locomotor={4A582744-9839-11d1-B709-00A024DDAFD1}\n\
[FOOT2]\nStrength=120\nSpeed=4\nROT=6\nHasSpotlight=yes\n\
AttachEffect.Duration=30\nAttachEffect.InitialDelay=0\nAttachEffect.FirepowerMultiplier=3\n\
Spotlight.StartHeight=650\nSpotlight.DisableGreen=yes\n\
Locomotor={4A582744-9839-11d1-B709-00A024DDAFD1}\n";

fn layers() -> RulesLayerStack {
    let mut layers = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(SCENE));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[OLD]\nConvert.Script=MISSING\n[NEW]\nTurretROT=bad\n\
             [CLEAR]\nConvert.Script=<none>\n[UNSET]\nConvert.Script=MISSING\n",
        ),
    );
    layers
}

fn admit(layers: &RulesLayerStack, wasm: &[u8]) -> (ProcessedRulesLayers, RuleSet) {
    let processed = layers.process().unwrap();
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
    rules.bind_resource_policies().unwrap();
    (processed, rules)
}

fn world(rules: &RuleSet) -> Simulation {
    let mut sim = crate::sim::arena_fixture::rival_arena(rules, 19);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(rules);
    sim.resolve_type_handles(rules);
    sim.session
        .set_current_house_for_test(sim.interner.get("Americans"));
    // The saved-resource seam needs full map authority, beyond the arena's
    // movement inputs. Install empty overlays/smudges and the original queues.
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
    sim.spawn_object(name, "Americans", x, 10, 0, rules)
        .unwrap()
}

fn team(sim: &mut Simulation, members: Vec<u64>) -> u64 {
    let owner = sim.interner.get("Americans").unwrap();
    let script = sim.intern("SDK-CONVERSION");
    sim.team_script_vm.register_script(TeamScriptDefinition {
        id: script,
        source: TeamAiDefinitionSource::FixedAimd,
        actions: vec![
            TeamScriptAction {
                action_id: 68,
                argument: 99,
            },
            TeamScriptAction {
                action_id: 68,
                argument: -1,
            },
            TeamScriptAction {
                action_id: 2,
                argument: 0,
            },
        ],
    });
    sim.team_script_vm
        .create_team(owner, script, members, sim.session.binary_frame() as i32)
}

fn step(sim: &mut Simulation, rules: &RuleSet, team: u64) {
    sim.team_step_script(team, rules, None, FrameEffects::empty());
}

fn visit(sim: &mut Simulation, rules: &RuleSet, actor: u64) {
    assert!(sim.object_ai_visit_one(actor, Some(rules), ObjectAiCtx::default()));
    assert!(
        sim.external_mod_fault.is_none(),
        "{:?}",
        sim.external_mod_fault
    );
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_CONVERSION_WASM"]
fn sdk_script68_physical_prefix_layers_and_explicit_off_reach_the_live_dispatcher() {
    let wasm = component();
    let mut source = layers();
    let (_, rules) = admit(&source, &wasm);
    assert!(rules.script_type_conversion_enabled());
    assert!(
        rules.object("EARLY").unwrap().script_conversion().is_none(),
        "FreeUnit allocates LATE only after EARLY's common Techno suffix"
    );
    let after = rules.object("AFTER").unwrap().script_conversion().unwrap();
    assert_eq!(
        (after.category, after.id.as_str()),
        (ObjectCategory::Vehicle, "LATE")
    );
    let old = rules.object("OLD").unwrap().script_conversion().unwrap();
    assert_eq!(
        (old.category, old.id.as_str()),
        (ObjectCategory::Vehicle, "NEW"),
        "the later unknown token retains the first physical pass binding"
    );
    assert!(
        rules
            .object_in_category(ObjectCategory::Vehicle, "DUP")
            .is_some()
    );
    let bad = rules.object("BAD").unwrap().script_conversion().unwrap();
    assert_eq!(
        (bad.category, bad.id.as_str()),
        (ObjectCategory::Building, "DUP"),
        "Find preserves the first common-array match, even from another class"
    );
    assert!(rules.object("CLEAR").unwrap().script_conversion().is_none());
    assert!(rules.object("UNSET").unwrap().script_conversion().is_none());
    assert!(
        rules.object("MISSING").is_none(),
        "Find never allocates a missing target"
    );
    assert_eq!(rules.object("NEW").unwrap().turret_rot_override(), Some(-1));

    source.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresTypeConversion=no\n"),
    );
    let (_, off) = admit(&source, &wasm);
    assert!(!off.script_type_conversion_enabled());
    let mut sim = world(&off);
    let actor = spawn(&mut sim, &off, "OLD", 10);
    let idle = team(&mut sim, vec![actor]);
    step(&mut sim, &off, idle);
    step(&mut sim, &off, idle);
    assert_eq!(
        sim.interner
            .resolve(sim.entities().get(actor).unwrap().type_ref()),
        "OLD"
    );
    let state = sim.team_script_vm.team(idle).unwrap();
    assert_eq!(state.cursor(), 0);
    assert!(!state.advance_pending());
    assert!(
        state.refusal.is_none(),
        "stock range rejection does not invent a refusal"
    );
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_CONVERSION_WASM"]
fn sdk_script68_mixed_team_preserves_instances_and_resumes_exact_program_after_save() {
    let wasm = component();
    let (processed, rules) = admit(&layers(), &wasm);
    let mut sim = world(&rules);
    let members: Vec<_> = [
        ("OLD", 10),
        ("FOOT", 12),
        ("BAD", 14),
        ("SAME", 16),
        ("CLEAR", 18),
    ]
    .into_iter()
    .map(|(name, x)| spawn(&mut sim, &rules, name, x))
    .collect();
    let [unit, foot, bad, same, _] = members[..] else {
        unreachable!()
    };
    let refused = spawn(&mut sim, &rules, "REFUSED", 20);
    let target = rules
        .object("REFUSED")
        .unwrap()
        .script_conversion()
        .unwrap();
    assert!(
        !sim.convert_techno_type(refused, target, &rules, FrameEffects::empty()),
        "a valid Building declaration is rejected by the converter at runtime"
    );
    let cargo = vec![
        crate::sim::miner::CargoBale::of_type(
            crate::rules::tiberium_type::TiberiumTypeId(0),
            crate::sim::miner::ResourceType::Ore,
            &rules,
        )
        .unwrap(),
    ];
    let actor = sim.substrate.entities.get_mut(unit).unwrap();
    actor.health.set_for_test(50);
    actor.estimated_health.reset(7);
    actor
        .miner
        .as_mut()
        .unwrap()
        .set_cargo_for_test(cargo.clone());
    let original = (
        actor.owner(),
        actor.original_factory_country(),
        actor.native_unique_id(),
    );
    sim.substrate
        .entities
        .get_mut(foot)
        .unwrap()
        .locomotor
        .as_mut()
        .unwrap()
        .power_off();
    visit(&mut sim, &rules, foot);
    assert_eq!(
        sim.entities()
            .get(foot)
            .unwrap()
            .crate_multipliers
            .firepower(),
        NativeF64Bits::from_bits(2.0_f64.to_bits())
    );
    let group = team(&mut sim, members.clone());
    let empty = team(&mut sim, Vec::new());
    let count = sim.entities().len();
    sim.take_lifecycle_outputs_for_test();
    step(&mut sim, &rules, group);
    step(&mut sim, &rules, empty);
    assert!(
        sim.team_script_vm.team(empty).unwrap().advance_pending(),
        "empty Script68 still completes"
    );
    let changed: Vec<_> = sim
        .take_lifecycle_outputs_for_test()
        .into_iter()
        .filter_map(|output| match output {
            crate::sim::world::LifecycleOutput::ObjectTypeChanged { stable_id } => Some(stable_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        changed,
        members[..2],
        "same-type and rejected entries emit no type-change output"
    );
    assert_eq!(
        sim.entities().len(),
        count,
        "conversion does not replace actors"
    );
    let state = sim.team_script_vm.team(group).unwrap();
    assert_eq!(state.members().collect::<Vec<_>>(), members);
    assert_eq!(state.cursor(), 0);
    assert!(
        state.advance_pending(),
        "the following action waits for the next Team visit"
    );
    let actor = sim.entities().get(unit).unwrap();
    assert_eq!(sim.interner.resolve(actor.type_ref()), "NEW");
    assert_eq!(
        (
            actor.owner(),
            actor.original_factory_country(),
            actor.native_unique_id()
        ),
        original
    );
    assert_eq!(
        (actor.health.current(), actor.estimated_health.get()),
        (100, 100)
    );
    assert_eq!(
        (actor.ammo().unwrap().current(), actor.ammo().unwrap().max()),
        (3, 3)
    );
    assert_eq!(
        actor.locomotor.as_ref().unwrap().active_kind(),
        LocomotorKind::Hover
    );
    assert_eq!(actor.body_facing.rot_per_frame(), 8 << 8);
    assert_eq!(
        actor.barrel_facing.as_ref().unwrap().rot_per_frame(),
        0xff00,
        "the existing Facing owner keeps the authored negative ROT's low byte"
    );
    assert!(!actor.is_harvester());
    assert_eq!(actor.miner.as_ref().unwrap().capacity_bales(), 2);
    assert_eq!(actor.miner.as_ref().unwrap().cargo(), cargo.as_slice());
    assert_eq!(
        sim.interner
            .resolve(sim.entities().get(bad).unwrap().type_ref()),
        "BAD"
    );
    assert_eq!(
        sim.entities().get(same).unwrap().ammo().unwrap().current(),
        2,
        "same-type conversion still runs the capacity owner instead of becoming a no-op"
    );
    let actor = sim.entities().get(foot).unwrap();
    assert_eq!(sim.interner.resolve(actor.type_ref()), "FOOT2");
    assert!(
        !actor.locomotor.as_ref().unwrap().is_powered(),
        "equal CLSIDs retain the controller"
    );
    assert_eq!(actor.body_facing.rot_per_frame(), 6 << 8);
    assert!(actor.barrel_facing.is_none());
    assert_eq!(
        actor.crate_multipliers.firepower(),
        NativeF64Bits::ONE,
        "conversion removes the old self effect"
    );
    let light = actor.building_light.light().unwrap();
    visit(&mut sim, &rules, foot);
    visit(&mut sim, &rules, light);
    assert_eq!(
        sim.entities()
            .get(foot)
            .unwrap()
            .crate_multipliers
            .firepower(),
        NativeF64Bits::from_bits(3.0_f64.to_bits())
    );
    let draw = sim
        .building_spotlight(light, sim.session().current_house(), &rules)
        .unwrap();
    assert_eq!(draw.glow_flags, 4);
    let z = crate::sim::movement::ground_pose::position_world_coord(
        &sim.entities().get(foot).unwrap().position,
    )
    .z;
    assert_eq!(draw.beam_origin[2], z + 650);
    let owner = sim.interner.get("Americans").unwrap();
    let tracking = sim.houses.get(&owner).unwrap().tracking();
    assert_eq!(
        tracking.owned_count(EntityCategory::Unit, sim.interner.get("OLD").unwrap()),
        0
    );
    assert_eq!(
        tracking.owned_count(EntityCategory::Unit, sim.interner.get("NEW").unwrap()),
        1
    );

    let terrain = sim.resolved_terrain.as_ref().unwrap().clone();
    let mut restored = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-CONVERSION", 0))
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
    rebound.bind_resource_policies().unwrap();
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
        restored
            .entities()
            .get(unit)
            .unwrap()
            .miner
            .as_ref()
            .unwrap()
            .cargo(),
        cargo.as_slice()
    );
    assert_eq!(
        restored
            .entities()
            .get(foot)
            .unwrap()
            .building_light
            .light(),
        Some(light)
    );
    assert!(
        restored
            .building_spotlight(light, restored.session().current_house(), &rebound)
            .is_some()
    );
    step(&mut restored, &rebound, group);
    assert_eq!(restored.team_script_vm.team(group).unwrap().cursor(), 1);
    assert!(
        restored
            .team_script_vm
            .team(group)
            .unwrap()
            .advance_pending(),
        "a visit with no changing target still completes Script68"
    );
    step(&mut restored, &rebound, group);
    let state = restored.team_script_vm.team(group).unwrap();
    assert_eq!(state.cursor(), 2);
    assert!(!state.advance_pending());
    assert_eq!(state.members().collect::<Vec<_>>(), members);
    assert_eq!(
        restored
            .team_script_vm
            .team_for_member(unit)
            .map(|(id, _)| id),
        Some(group)
    );
    assert_eq!(
        restored
            .interner
            .resolve(restored.entities().get(unit).unwrap().type_ref()),
        "NEW"
    );
    restored.debug_assert_logic_membership_consistent();
}
