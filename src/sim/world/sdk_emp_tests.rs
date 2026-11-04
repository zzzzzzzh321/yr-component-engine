//! Existing EMP status owners reached through the real external component in
//! an ordinary engine. Synthetic arenas declare their own terrain/art inputs;
//! the retained native controls continue to cover the numeric/status kernel.
use super::*;
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerStack};
use crate::rules::process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::ruleset::RuleSet;
use crate::sim::combat::{TargetKind, fire_error::FireError, world_fire::FireVisit};
use crate::sim::mission::{MissionId, MissionType};
use crate::sim::snapshot::GameSnapshot;
use std::sync::Arc;

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os("VERA_SDK_EMP_WASM").expect("set VERA_SDK_EMP_WASM")).unwrap()
}

fn bind(processed: &ProcessedRulesLayers, mut rules: RuleSet, wasm: &[u8]) -> RuleSet {
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
    rules
}

fn load_process(
    owner: &mut NativeRulesProcessOwner,
    prefix: NativeScenarioRulesPrefix<'_>,
    map: &str,
    wasm: &[u8],
) -> RuleSet {
    let rules = owner
        .load_scenario(prefix, &IniFile::from_str(map))
        .unwrap()
        .into_parts()
        .0;
    let rules = bind(owner.saved_scenario_rules().unwrap(), rules, wasm);
    owner.retain_sdk_configuration_program(rules.external_mod.clone());
    rules
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_EMP_WASM"]
fn sdk_emp_physical_references_and_process_globals_follow_the_real_rules_owner() {
    let wasm = component();
    let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
        IniFile::from_str(
            "[BuildingTypes]\n0=EARLY\n[VehicleTypes]\n0=LATE\n1=CLEAR\n\
             [General]\nEMPulseSparkles=GENERAL\n\
             [EARLY]\nEMP.Sparkles=LATE_ANIM\n\
             [LATE]\nDeployingAnim=LATE_ANIM\nEMP.Sparkles=LATE_ANIM\n\
             [CLEAR]\nEMP.Sparkles=<none>\n",
        ),
        None,
        IniFile::empty(),
        Arc::new(crate::rules::sound_ini::SoundRegistry::default()),
        Arc::new(crate::rules::sound_ini::EvaRegistry::default()),
    )
    .unwrap();
    let first = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "[CombatDamage]\nEMPAIRecoverMission=Hunt\n",
        &wasm,
    );
    assert!(first.emp_enabled());
    assert_eq!(first.ares_emp_ai_recover_mission(), Some(MissionType::Hunt));
    assert_eq!(
        first
            .object("EARLY")
            .unwrap()
            .emp_rules()
            .unwrap()
            .sparkles(),
        None,
        "Vehicle allocation must not repair the preceding Building lookup"
    );
    assert_eq!(
        first
            .object("CLEAR")
            .unwrap()
            .emp_rules()
            .unwrap()
            .sparkles(),
        Some(None)
    );
    assert_eq!(
        first
            .object("LATE")
            .unwrap()
            .emp_rules()
            .unwrap()
            .sparkles(),
        Some(Some("LATE_ANIM"))
    );
    let generated = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::RandomMap(None),
        "",
        &wasm,
    );
    assert_eq!(
        generated.ares_emp_ai_recover_mission(),
        Some(MissionType::Hunt),
        "TypeReset retains the component's process-global mission"
    );
    let disabled = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "[VeraFeatures]\nAresEMP=no\n",
        &wasm,
    );
    assert!(!disabled.emp_enabled());
    assert_eq!(
        disabled
            .object("LATE")
            .unwrap()
            .emp_rules()
            .unwrap()
            .sparkles(),
        Some(Some("LATE_ANIM")),
        "off retains complete frozen declarations; the Rules selector owns activation"
    );
    let (_, off_rules, mut off_sim, target, source) = scene_from_ini(
        &wasm,
        "TANK",
        "Americans",
        &format!("{SCENE}\n[VeraFeatures]\nAresEMP=no\n"),
    );
    pulse(&mut off_sim, &off_rules, source, target);
    assert!(off_sim.object_ai_visit_one(target, Some(&off_rules), ObjectAiCtx::default()));
    let victim = off_sim.entities().get(target).unwrap();
    assert_eq!(victim.emp_remaining(), 0);
    assert!(!victim.deactivated.is_offline());
    assert!(victim.emp_sparkle().is_none());
    assert!(off_sim.can_deploy_slash_unload(target, &off_rules));
    let cleared = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "",
        &wasm,
    );
    assert!(cleared.emp_enabled());
    assert_eq!(
        cleared.ares_emp_ai_recover_mission(),
        None,
        "ScenarioClear rebuilds globals"
    );
}

const SCENE: &str = "\
[VehicleTypes]\n0=EMITTER\n1=TANK\n[BuildingTypes]\n0=TOWER\n\
[Warheads]\n0=EMPWH\n1=HURT\n[Animations]\n0=SPARK\n1=TOWER_B\n\
[General]\nEMPulseSparkles=SPARK\nPrismType=TOWER\n\
[EMITTER]\nStrength=400\nArmor=heavy\nSpeed=6\nPrimary=PULSE\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\nMovementZone=Normal\n\
[TANK]\nStrength=400\nArmor=heavy\nSpeed=6\nPrimary=BEAM\nDeployFire=yes\n\
Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\nMovementZone=Normal\n\
[TOWER]\nImage=TOWERART\nStrength=1000\nArmor=steel\nPowered=yes\nPower=-75\n\
Primary=BEAM\nCanPassiveAquire=no\nHasStupidGuardMode=no\nImmuneToEMP=no\n\
[PULSE]\nDamage=0\nROF=1\nRange=9.5\nSpeed=100\nProjectile=INVIS\nWarhead=EMPWH\n\
[BEAM]\nDamage=20\nROF=1\nRange=10\nSpeed=100\nProjectile=INVIS\nWarhead=HURT\n\
[INVIS]\nInviso=yes\nAG=yes\nAA=no\n\
[EMPWH]\nCellSpread=.7\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
EMP.Duration=2\nEMP.Cap=2\nAffectsAllies=yes\n\
[HURT]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n";

fn art() -> IniFile {
    IniFile::from_str(
        "[TOWERART]\nFoundation=1x1\nDelayedFireDelay=1\nActiveAnim=TOWER_B\nActiveAnimPowered=yes\n\
         [TOWER_B]\nLoopCount=-1\nRate=1\nEnd=100\n[SPARK]\nLoopCount=-1\nRate=1\nEnd=4\n",
    )
}

fn install_art(rules: &mut RuleSet) {
    let mut registry = crate::rules::art_data::ArtRegistry::from_ini(&art());
    // Explicit synthetic physical-frame boundary, not extracted game art.
    registry.bind_anim_frame_count_for_test("SPARK", 4);
    registry.bind_anim_frame_count_for_test("TOWER_B", 100);
    rules.install_art_data(registry);
}

fn scene(
    wasm: &[u8],
    target_type: &str,
    target_house: &str,
) -> (ProcessedRulesLayers, RuleSet, Simulation, u64, u64) {
    scene_from_ini(wasm, target_type, target_house, SCENE)
}

fn scene_from_ini(
    wasm: &[u8],
    target_type: &str,
    target_house: &str,
    source_ini: &str,
) -> (ProcessedRulesLayers, RuleSet, Simulation, u64, u64) {
    let processed = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(source_ini))
        .process_with_fixed_art(&art())
        .unwrap();
    let mut rules = bind(
        &processed,
        RuleSet::from_processed_rules(&processed).unwrap(),
        wasm,
    );
    install_art(&mut rules);
    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 17);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(&rules);
    sim.session
        .set_current_house_for_test(sim.interner.get("Americans"));
    let target = sim
        .spawn_object(target_type, target_house, 10, 10, 0, &rules)
        .unwrap();
    let source_house = if target_house == "Americans" {
        "Russians"
    } else {
        "Americans"
    };
    let source = sim
        .spawn_object("EMITTER", source_house, 12, 10, 0, &rules)
        .unwrap();
    sim.resolve_type_handles(&rules);
    sim.substrate
        .entities
        .get_mut(target)
        .unwrap()
        .mission
        .set_current_for_test(MissionId::from_known(MissionType::Guard), 0);
    (processed, rules, sim, target, source)
}

fn pulse(sim: &mut Simulation, rules: &RuleSet, source: u64, target: u64) {
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
        .filter(|(_, bullet)| bullet.in_logic_vector())
        .map(|(&id, _)| id)
        .collect::<Vec<_>>();
    assert_eq!(bullets.len(), 1);
    assert!(sim.object_ai_visit_one(bullets[0], Some(rules), ObjectAiCtx::default()));
}

fn install_tag(sim: &mut Simulation, rules: &RuleSet, target: u64, source: u64) {
    let source_house = sim
        .base_reservation_house_index(sim.entities().get(source).unwrap().owner())
        .unwrap();
    let map = crate::map::map_file::MapFile::from_bytes(
        format!(
            "[Map]\nTheater=TEMPERATE\nSize=0,0,2,1\nLocalSize=0,0,2,1\n\
         [IsoMapPack5]\n1=DwALABwBAAIA/////wAAABEAAA==\n[Basic]\nNewINIFormat=4\n\
         [Triggers]\nE62=Neutral,E63,E62,0,1,1,1,0\nE63=Neutral,<none>,E63,0,1,1,1,0\n\
         [Events]\nE62=1,62,0,0\nE63=1,63,0,{source_house}\n\
         [Actions]\nE62=1,56,0,0,0,0,0,0,A\nE63=1,56,0,1,0,0,0,0,A\n\
         [Tags]\nEMP=2,EMP,E62\n"
        )
        .as_bytes(),
    )
    .unwrap();
    sim.initialize_trigger_definitions(&map, &Default::default(), None, None, rules);
    sim.attach_object_trigger_tag(target, Some("EMP"));
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_EMP_WASM"]
fn sdk_emp_parent_bullet_tags_and_actual_visits_survive_frozen_program_restore() {
    let wasm = component();
    let (processed, rules, mut sim, target, source) = scene(&wasm, "TANK", "Americans");
    install_tag(&mut sim, &rules, target, source);
    crate::sim::arena_fixture::prepare_empty_map_authority(&mut sim, &rules);
    assert!(sim.can_deploy_slash_unload(target, &rules));
    let health = sim.entities().get(target).unwrap().health.current();
    pulse(&mut sim, &rules, source, target);
    // Bullet46920B / deliver10062320 runs even for a zero-damage parent.
    assert_eq!(sim.entities().get(target).unwrap().health.current(), health);
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 2);
    assert_eq!(
        sim.tag_dispatch_trace_for_test()
            .iter()
            .map(|(_, kind)| *kind)
            .collect::<Vec<_>>(),
        [63, 62]
    );
    assert!(sim.trigger_local_is_set_for_test(0) && sim.trigger_local_is_set_for_test(1));
    assert!(!sim.techno_player_controllable(target, &rules));
    assert!(!sim.can_deploy_slash_unload(target, &rules));
    sim.techno_reactivate(target, &rules, FrameEffects::empty());
    assert!(
        sim.entities().get(target).unwrap().deactivated.is_offline(),
        "shared Activate1004CCB0 veto"
    );
    let sparkle = sim.entities().get(target).unwrap().emp_sparkle().unwrap();
    sim.clear_tag_dispatch_trace_for_test();

    let terrain = sim.resolved_terrain.as_ref().unwrap().clone();
    let mut restored = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-EMP", 0))
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
        "restore keeps the exact component and frozen configuration"
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
    install_art(&mut rebound);
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
    assert_eq!(
        restored.entities().get(target).unwrap().emp_sparkle(),
        Some(sparkle)
    );
    assert!(restored.object_ai_visit_one(target, Some(&rebound), ObjectAiCtx::default()));
    assert_eq!(restored.entities().get(target).unwrap().emp_remaining(), 1);
    assert!(restored.object_ai_visit_one(target, Some(&rebound), ObjectAiCtx::default()));
    assert_eq!(restored.entities().get(target).unwrap().emp_remaining(), 0);
    assert!(
        !restored
            .entities()
            .get(target)
            .unwrap()
            .deactivated
            .is_offline()
    );
    assert!(restored.techno_player_controllable(target, &rebound));
    assert!(restored.can_deploy_slash_unload(target, &rebound));
    assert!(
        restored.tag_dispatch_trace_for_test().is_empty(),
        "natural expiry emits no removal Tag events"
    );
    assert_eq!(restored.anim(sparkle).unwrap().runtime().loop_remaining, 0);
}

fn tower_fire_error(sim: &Simulation, rules: &RuleSet, tower: u64, target: u64) -> FireError {
    crate::sim::combat::fire_error_world::FireSubject {
        world: sim,
        rules,
        overlay_registry: None,
        fog: None,
        firer: sim.entities().get(tower).unwrap(),
        obj: rules.object("TOWER").unwrap(),
        target: Some(TargetKind::Entity(target)),
        weapon_index: 0,
    }
    .fire_error(false)
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_EMP_WASM"]
fn sdk_emp_building_expiry_retains_low_power_latch_until_the_real_head_recovers() {
    let wasm = component();
    let (_, rules, mut sim, target, source) = scene(&wasm, "TOWER", "Russians");
    let owner = sim.entities().get(target).unwrap().owner();
    let power = sim
        .power_states
        .map_mut_for_test()
        .entry(owner)
        .or_default();
    power.set_total_output_for_test(100);
    power.set_total_drain_for_test(75);
    let slot = sim
        .set_building_anim_slot(target, 3, false, false, 0, &rules)
        .unwrap();
    sim.visit_building_operational(target, &rules, None, FrameEffects::empty());
    assert_eq!(
        tower_fire_error(&sim, &rules, target, source),
        FireError::Ok
    );
    pulse(&mut sim, &rules, source, target);
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 2);
    assert_eq!(
        tower_fire_error(&sim, &rules, target, source),
        FireError::Cant
    );
    assert!(sim.anim(slot).unwrap().runtime().paused);
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(0);
    // Techno6FAF0D visits, not a global frame decrement or direct counter write.
    sim.object_ai_visit_one(target, Some(&rules), ObjectAiCtx::default());
    sim.object_ai_visit_one(target, Some(&rules), ObjectAiCtx::default());
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 0);
    assert!(sim.entities().get(target).unwrap().deactivated.is_offline());
    assert!(sim.entities().get(target).unwrap().emp_sparkle().is_none());
    assert!(sim.anim(slot).unwrap().runtime().paused);
    sim.object_ai_visit_one(target, Some(&rules), ObjectAiCtx::default());
    assert!(sim.entities().get(target).unwrap().deactivated.is_offline());
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(100);
    // The operational edge and1004B528 head use the same retained status owner.
    sim.object_ai_visit_one(target, Some(&rules), ObjectAiCtx::default());
    assert!(!sim.entities().get(target).unwrap().deactivated.is_offline());
    assert!(!sim.anim(slot).unwrap().runtime().paused);
    assert_eq!(
        tower_fire_error(&sim, &rules, target, source),
        FireError::Ok
    );
    assert!(
        !sim.entities()
            .get(target)
            .unwrap()
            .building_art
            .stuff_enabled(),
        "delayed low-power recovery must not invent a second EnableStuff"
    );
}
