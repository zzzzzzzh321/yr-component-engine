//! Actual-component production declarations, existing live owners and process
//! continuation. These tests deliberately use the ordinary non-Ares engine.
use super::*;
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::rules::process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::ruleset::RuleSet;
use std::sync::Arc;

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(
        std::env::var_os("VERA_SDK_PRODUCTION_WASM").expect("set VERA_SDK_PRODUCTION_WASM"),
    )
    .unwrap()
}

fn program(
    processed: &ProcessedRulesLayers,
    rules: &RuleSet,
    wasm: &[u8],
    seed_override: Option<&Program>,
) -> Result<Program, String> {
    let state = processed.native_registry_state();
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
        seed_override.or_else(|| state.sdk_configuration_seed().map(Arc::as_ref)),
        state.sdk_configuration_steps(),
    )
}

fn bind(processed: &ProcessedRulesLayers, mut rules: RuleSet, wasm: &[u8]) -> RuleSet {
    rules.external_mod = Some(Arc::new(program(processed, &rules, wasm, None).unwrap()));
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
#[ignore = "requires the packaged Ares component in VERA_SDK_PRODUCTION_WASM"]
fn sdk_production_read_points_join_factory_plans_admission_and_saved_rebinding() {
    use crate::sim::production::{CanBuild, can_build};
    use crate::sim::snapshot::GameSnapshot;
    let wasm = component();
    let mut layers = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(
        "[Countries]\n0=SOURCE\n1=Americans\n2=Russians\n\
         [SOURCE]\nParentCountry=SOURCE\n[Americans]\nParentCountry=SOURCE\n[Russians]\nParentCountry=Russians\n\
         [BuildingTypes]\n0=POWER\n1=YARD\n2=TARGET\n3=PLAIN\n\
         [VehicleTypes]\n0=EARLY\n[EARLY]\nStrength=100\nPrerequisite=late\n\
         [GenericPrerequisites]\nfacility=POWER\n\
         [POWER]\nStrength=100\nOwner=SOURCE,Americans,Russians\n\
         [YARD]\nStrength=100\nFactory=BuildingType\nOwner=SOURCE,Americans,Russians\n\
         Factory.ExplicitOnly=yes\nFactoryOwners.Permanent=yes\n\
         [TARGET]\nStrength=100\nTechLevel=1\nOwner=SOURCE,Americans,Russians\n\
         Prerequisite=facility\nBuiltAt=YARD\nFactoryOwners=SOURCE\nBuildLimit=1\n\
         [PLAIN]\nStrength=100\nTechLevel=1\nOwner=SOURCE,Americans,Russians\nFactoryOwners=SOURCE\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[GenericPrerequisites]\nlate=YARD\n"),
    );
    let processed = layers.process().unwrap();
    let rules = bind(
        &processed,
        RuleSet::from_processed_rules(&processed).unwrap(),
        &wasm,
    );
    assert!(
        rules.object("EARLY").unwrap().prerequisites().is_empty(),
        "later group registration cannot repair an earlier unknown reference"
    );
    let mut sim = Simulation::with_seed(2);
    crate::sim::arena_fixture::flat_ground(&mut sim, &rules);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(&rules);
    for (index, (name, human)) in [("Americans", true), ("Russians", false)]
        .into_iter()
        .enumerate()
    {
        let id = sim.interner.intern(name);
        sim.houses
            .register(crate::sim::house_state::HouseState::new(
                id,
                index as i32,
                Some(id),
                human,
                0,
                10,
            ));
    }
    sim.resolve_type_handles(&rules);
    let human = sim.interner.get("Americans").unwrap();
    let receiver = sim.interner.get("Russians").unwrap();
    let source = rules.country_index("SOURCE").unwrap();
    let yard = sim
        .spawn_object("YARD", "Americans", 10, 10, 0, &rules)
        .unwrap();
    assert_eq!(
        sim.entities().get(yard).unwrap().original_factory_country(),
        Some(source)
    );
    // The actual constructor and common GrandOpening producer own the value;
    // the component never manufactures a second House plan collection.
    sim.grand_opening(yard, false, false, &rules, None, FrameEffects::default());
    assert!(
        sim.houses[&human]
            .permanent_factory_plans()
            .contains(&source)
    );
    assert_eq!(
        can_build(
            &sim,
            &rules,
            human,
            rules.object("TARGET").unwrap(),
            false,
            true
        ),
        CanBuild::No
    );
    sim.spawn_object("POWER", "Americans", 14, 10, 0, &rules)
        .unwrap();
    assert_eq!(
        can_build(
            &sim,
            &rules,
            human,
            rules.object("TARGET").unwrap(),
            false,
            true
        ),
        CanBuild::Yes
    );
    assert_eq!(
        can_build(
            &sim,
            &rules,
            human,
            rules.object("PLAIN").unwrap(),
            false,
            true
        ),
        CanBuild::No,
        "plans do not bypass the explicit source-factory requirement"
    );
    sim.change_owner_with_rules(yard, receiver, &rules, None, FrameEffects::default());
    assert_eq!(
        sim.entities().get(yard).unwrap().original_factory_country(),
        Some(source)
    );
    assert!(
        sim.houses[&receiver]
            .permanent_factory_plans()
            .contains(&source)
    );
    assert_eq!(
        can_build(
            &sim,
            &rules,
            human,
            rules.object("TARGET").unwrap(),
            false,
            true
        ),
        CanBuild::No,
        "retained plans cannot replace a live factory"
    );
    assert_eq!(
        can_build(
            &sim,
            &rules,
            receiver,
            rules.object("TARGET").unwrap(),
            false,
            true
        ),
        CanBuild::Yes
    );

    let saved = GameSnapshot::save(&sim, 0, 0, "SDK-PRODUCTION", 0);
    let mut restored = GameSnapshot::load(&saved).unwrap().sim;
    restored.restore_after_snapshot_load().unwrap();
    let retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(&processed).unwrap(),
    )
    .unwrap();
    let mut rebound = RuleSet::from_processed_rules(&retained).unwrap();
    rebound.external_mod = restored.external_mod.as_ref().map(|p| Arc::new(p.clone()));
    rebound.bind_resource_policies().unwrap();
    restored.resolve_type_handles(&rebound);
    restored
        .validate_factory_country_references(&rebound)
        .unwrap();
    assert!(
        restored.houses[&receiver]
            .permanent_factory_plans()
            .contains(&source)
    );
    assert_eq!(
        can_build(
            &restored,
            &rebound,
            receiver,
            rebound.object("TARGET").unwrap(),
            false,
            true
        ),
        CanBuild::Yes
    );
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_PRODUCTION_WASM"]
fn sdk_production_process_seed_survives_generated_reset_and_clears_before_campaign_prefix() {
    let wasm = component();
    let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
        IniFile::empty(),
        None,
        IniFile::empty(),
        Arc::new(crate::rules::sound_ini::SoundRegistry::default()),
        Arc::new(crate::rules::sound_ini::EvaRegistry::default()),
    )
    .unwrap();
    let flags = |rules: &RuleSet| {
        std::array::from_fn::<_, 3, _>(|index| rules.ares_ai_bypasses_build_limit(index))
    };
    let first = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "[GlobalControls]\nAllowBypassBuildLimit=yes,no,yes\n",
        &wasm,
    );
    assert_eq!(flags(&first), [true, false, true]);
    let generated = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::RandomMap(None),
        "",
        &wasm,
    );
    assert_eq!(
        flags(&generated),
        [true, false, true],
        "TypeReset retains opaque process-global state"
    );

    // Saved registries deliberately omit continuation inputs. The formal
    // saved-load commit must attach its already-validated exact Program before
    // the next generated scene, without replaying Configure during restore.
    let mut saving = Simulation::new();
    saving.external_mod = generated.external_mod.as_deref().cloned();
    let loaded = crate::sim::snapshot::GameSnapshot::load(
        &crate::sim::snapshot::GameSnapshot::save(&saving, 0, 0, "SDK-PROCESS", 0),
    )
    .unwrap()
    .sim;
    let mut retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(owner.saved_scenario_rules().unwrap()).unwrap(),
    )
    .unwrap();
    loaded
        .external_mod
        .as_ref()
        .unwrap()
        .validate_restore()
        .unwrap();
    // The formal restore owner retains the process-selected audio catalogs;
    // a raw RuleSet projection cannot authorize their frozen declarations.
    retained.retain_sdk_configuration_program(
        loaded
            .external_mod
            .as_ref()
            .map(|program| Arc::new(program.clone())),
    );
    let mut saved_rules = owner.project_saved_scenario_rules(&retained).unwrap();
    saved_rules.external_mod = loaded
        .external_mod
        .as_ref()
        .map(|program| Arc::new(program.clone()));
    saved_rules.bind_resource_policies().unwrap();
    owner.install_saved_scenario_rules(retained, Some(&saved_rules));
    assert!(
        owner
            .saved_scenario_rules()
            .unwrap()
            .sdk_configuration_steps()
            .is_empty()
    );
    let generated = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::RandomMap(None),
        "",
        &wasm,
    );
    assert_eq!(
        flags(&generated),
        [true, false, true],
        "saved Program seeds the subsequent generated scene"
    );

    let mut changed =
        serde_json::to_value(generated.external_mod.as_ref().unwrap().as_ref()).unwrap();
    changed["manifest"]["version"] = serde_json::json!("different-component-schema");
    let changed: Program = serde_json::from_value(changed).unwrap();
    let error = program(
        owner.saved_scenario_rules().unwrap(),
        &generated,
        &wasm,
        Some(&changed),
    )
    .unwrap_err();
    assert!(error.contains("identity changed"), "{error}");

    let early = IniFile::from_str("[GlobalControls]\nAllowBypassBuildLimit=yes,yes\n");
    let campaign = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::Campaign(Some(&early)),
        "",
        &wasm,
    );
    assert_eq!(
        flags(&campaign),
        [false, true, true],
        "SceneClear precedes the early named Process; TypeReset retains its partial write"
    );
    let ordinary = load_process(
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "",
        &wasm,
    );
    assert_eq!(flags(&ordinary), [false; 3]);
}
