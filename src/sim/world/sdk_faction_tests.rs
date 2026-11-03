//! Real component admission and process-owned Side/Country chronology, followed
//! by existing construction, crew and complete saved-world resource owners.
use super::*;
use crate::assets::asset_manager::{
    AssetManager, MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::mod_sdk::Program;
use crate::rules::audio_sources::AudioDefinitions;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::ProcessedRulesLayers;
use crate::rules::object_type::ObjectCategory;
use crate::rules::process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::retail_sources::RetailRulesSources;
use crate::rules::ruleset::RuleSet;
use crate::sim::house_state::HouseState;
use crate::sim::mission::MissionType;
use crate::sim::snapshot::GameSnapshot;
use serde_json::Value;
use std::sync::Arc;

fn component(variable: &str) -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}

struct Install {
    assets: AssetManager,
    // Drop retained archive handles before removing the fixture directory.
    _directory: TestDirectory,
}

impl Install {
    fn new(label: &str, root: &str, wasm: Option<&[u8]>) -> Self {
        let directory = audio_test_install(label);
        for (name, bytes) in [
            ("rulesmd.ini", root.as_bytes()),
            ("artmd.ini", b"".as_slice()),
            ("soundmd.ini", b"".as_slice()),
            ("evamd.ini", b"".as_slice()),
        ] {
            std::fs::write(directory.path().join(name), bytes).unwrap();
        }
        if let Some(wasm) = wasm {
            std::fs::write(directory.path().join("gameplay.wasm"), wasm).unwrap();
            std::fs::write(
                directory.path().join("vera-mod.json"),
                serde_json::to_vec(&serde_json::json!({
                    "api_version": vera_mod_api::API_VERSION,
                    "program": "gameplay.wasm",
                    "sha256": crate::util::sha256::sha256_hex(wasm),
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let assets =
            crate::mod_sdk::open_active_assets(directory.path(), MediaArchiveMode::STOCK_DIGITAL)
                .unwrap();
        Self {
            assets,
            _directory: directory,
        }
    }

    fn sources(&self) -> (RetailRulesSources, AudioDefinitions) {
        let index = self.assets.load_audio_index().unwrap();
        let audio = AudioDefinitions::select_admitting(
            &self.assets,
            index.as_ref().map(|loaded| &loaded.index),
        );
        (RetailRulesSources::select(&self.assets).unwrap(), audio)
    }

    fn owner(&self) -> NativeRulesProcessOwner {
        let (sources, audio) = self.sources();
        sources
            .into_owner(Arc::clone(audio.sounds()), Arc::clone(audio.eva()))
            .unwrap()
    }
}

fn load(
    install: &Install,
    owner: &mut NativeRulesProcessOwner,
    prefix: NativeScenarioRulesPrefix<'_>,
    map: &str,
) -> RuleSet {
    let (mut rules, ..) = owner
        .load_scenario(prefix, &IniFile::from_str(map))
        .unwrap()
        .into_parts();
    rules
        .bind_external_mod_for_process(&install.assets, owner.saved_scenario_rules().unwrap())
        .unwrap();
    owner.retain_sdk_configuration_program(rules.external_mod.clone());
    rules
}

fn veteran(rules: &RuleSet, country: &str, category: ObjectCategory, name: &str) -> bool {
    rules.country_starts_veteran(country, rules.object_in_category(category, name).unwrap())
}

fn prepare_session(owner: &mut NativeRulesProcessOwner) {
    let selected =
        owner.read_multiplayer_country(&IniFile::from_str("[MultiPlayer]\nSide=Americans\n"), -1);
    assert_eq!(owner.country_name(selected), Some("Americans"));
    owner.refresh_multiplayer_country_bodies().unwrap();
}

#[test]
#[ignore = "requires VERA_SDK_FACTION_WASM and previous .15 in VERA_SDK_FACTION_PREVIOUS_WASM"]
fn sdk_faction_country_prebody_selects_original_or_find_only_lists_before_allocation() {
    let wasm = component("VERA_SDK_FACTION_WASM");
    let previous = component("VERA_SDK_FACTION_PREVIOUS_WASM");
    let root = format!(
        "[Countries]\n0=Americans\n[Americans]\nSide=GDI\n\
         VeteranInfantry=LAZY_I,{}TAIL_I\nVeteranUnits=LAZY_U,{}TAIL_U\nVeteranAircraft=LAZY_A,{}TAIL_A\n\
         VeteranBuildings=LATE_B\n[InfantryTypes]\n0=I\n1=TAIL_I\n\
         [VehicleTypes]\n0=U\n1=TAIL_U\n[AircraftTypes]\n0=A\n1=TAIL_A\n\
         [I]\nStrength=100\n[TAIL_I]\nStrength=100\n[U]\nStrength=100\n[TAIL_U]\nStrength=100\n\
         [A]\nStrength=100\n[TAIL_A]\nStrength=100\n",
        "I,".repeat(64),
        "U,".repeat(64),
        "A,".repeat(64),
    );
    let late = "[BuildingTypes]\n0=LATE_B\n[LATE_B]\nStrength=100\n";
    // These are the four supported installation choices, not hand-authored
    // policies: no module, old module, current default and current explicit off.
    for (label, bytes, root, find_only) in [
        ("faction-stock-reader", None, root.clone(), false),
        (
            "faction-old-reader",
            Some(previous.as_slice()),
            root.clone(),
            false,
        ),
        (
            "faction-current-reader",
            Some(wasm.as_slice()),
            root.clone(),
            true,
        ),
        (
            "faction-off-reader",
            Some(wasm.as_slice()),
            format!("{root}\n[VeraFeatures]\nAresFaction=no\n"),
            false,
        ),
    ] {
        let install = Install::new(label, &root, bytes);
        let mut owner = install.owner();
        prepare_session(&mut owner);
        let rules = load(
            &install,
            &mut owner,
            NativeScenarioRulesPrefix::NonCampaign(None),
            late,
        );
        for (category, lazy, tail) in [
            (ObjectCategory::Infantry, "LAZY_I", "TAIL_I"),
            (ObjectCategory::Vehicle, "LAZY_U", "TAIL_U"),
            (ObjectCategory::Aircraft, "LAZY_A", "TAIL_A"),
        ] {
            assert_eq!(
                rules.object_in_category(category, lazy).is_none(),
                find_only,
                "{label}: {lazy}"
            );
            assert_eq!(
                veteran(&rules, "Americans", category, tail),
                find_only,
                "{label}: 128-byte retail truncation versus 2048-byte Find-only"
            );
        }
        assert!(
            !veteran(&rules, "Americans", ObjectCategory::Building, "LATE_B"),
            "a later map allocation cannot repair the earlier missing building pointer"
        );
    }
}

const SPATIAL_ROOT: &str = "\
[Clear]\nBuildable=yes\n[General]\nAlliedCrew=E1\nSovietCrew=E2\nThirdCrew=INIT\nTechnician=TECH\nEngineer=ENG\n\
AlliedSurvivorDivisor=100\nSovietSurvivorDivisor=200\nThirdSurvivorDivisor=300\n\
[Countries]\n0=PARENT\n1=Americans\n2=EARLY\n3=LATE_PARENT\n4=ORPHAN\n5=Russians\n6=YuriCountry\n7=FourthCountry\n\
[Sides]\nGDI=PARENT,Americans,EARLY,LATE_PARENT,ORPHAN\nNod=Russians\nThirdSide=YuriCountry\nFOURTH=FourthCountry\n\
[PARENT]\nSide=GDI\nVeteranBuildings=VET\n[Americans]\nSide=GDI\nParentCountry=PARENT\n\
[EARLY]\nSide=GDI\nParentCountry=LATE_PARENT\n[LATE_PARENT]\nSide=GDI\nVeteranBuildings=VET\n\
[ORPHAN]\nSide=GDI\nParentCountry=MISSING_PARENT\n[Russians]\nSide=Nod\n[YuriCountry]\nSide=ThirdSide\n[FourthCountry]\nSide=FOURTH\n\
[Nod]\nCrew=<none>\n[InfantryTypes]\n0=E1\n1=E2\n2=INIT\n3=TECH\n4=ENG\n\
[E1]\nStrength=100\nSpeed=4\nLocomotor={4A582744-9839-11d1-B709-00A024DDAFD1}\n\
[E2]\nStrength=100\n[INIT]\nStrength=100\n[TECH]\nStrength=100\n[ENG]\nStrength=100\nEngineer=yes\n\
[BuildingTypes]\n0=VET\n1=OTHER\n[VET]\nStrength=500\nCost=1000\nCrewed=yes\n[OTHER]\nStrength=100\n";

fn world(rules: &RuleSet) -> Simulation {
    let mut sim = crate::sim::arena_fixture::rival_arena(rules, 41);
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

#[test]
#[ignore = "requires the real .16 faction component in VERA_SDK_FACTION_WASM"]
fn sdk_faction_side_fallback_parent_attempts_and_real_building_crew_use_shared_owners() {
    let wasm = component("VERA_SDK_FACTION_WASM");
    let install = Install::new("faction-parent-crew", SPATIAL_ROOT, Some(&wasm));
    let (sources, audio) = install.sources();
    let (frontend, _, mut owner) = sources
        .into_startup(Arc::clone(audio.sounds()), Arc::clone(audio.eva()))
        .unwrap()
        .into_parts();
    let frontend = frontend.unwrap();
    assert_eq!(frontend.side_crew(3), Some(Some("E1")));
    assert!(veteran(
        &frontend,
        "Americans",
        ObjectCategory::Building,
        "VET"
    ));
    let cold = config(owner.sdk_configuration_program().unwrap());
    assert!(
        cold.is_null()
            || cold["factions"]["sides"]
                .as_object()
                .is_some_and(|sides| sides.is_empty()),
        "the frontend's completed Side sweep must not replace the live cold checkpoint"
    );
    let rules = load(
        &install,
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "[Countries]\n8=MISSING_PARENT\n[MISSING_PARENT]\nSide=GDI\nVeteranBuildings=VET\n\
         [ORPHAN]\nParentCountry=MISSING_PARENT\n[PARENT]\nVeteranBuildings=OTHER\n",
    );
    assert!(veteran(
        &rules,
        "Americans",
        ObjectCategory::Building,
        "VET"
    ));
    assert!(
        !veteran(&rules, "Americans", ObjectCategory::Building, "OTHER"),
        "a later parent change does not recopy"
    );
    assert!(veteran(&rules, "PARENT", ObjectCategory::Building, "OTHER"));
    assert!(
        !veteran(&rules, "EARLY", ObjectCategory::Building, "VET"),
        "a later Country body cannot repair its earlier constructor-only parent copy"
    );
    assert!(
        !veteran(&rules, "ORPHAN", ObjectCategory::Building, "VET"),
        "a failed first parent lookup is consumed before the map registers it"
    );
    assert_eq!(rules.side_crew(0), Some(Some("E1")));
    assert_eq!(rules.side_crew(1), Some(None));
    assert_eq!(rules.side_crew(2), Some(Some("INIT")));
    assert_eq!(
        rules.side_crew(3),
        Some(Some("E1")),
        "the fourth allocated Side uses Allied fallback"
    );
    let mut sim = world(&rules);
    let building = sim
        .spawn_object("VET", "Americans", 10, 10, 0, &rules)
        .unwrap();
    assert_eq!(sim.entities().get(building).unwrap().veterancy.rank(), 100);
    let fourth = sim.intern("FourthHouse");
    let country = sim.intern("FourthCountry");
    sim.houses
        .register(HouseState::new(fourth, 3, Some(country), true, 0, 10));
    let source = sim
        .spawn_object("VET", "FourthHouse", 16, 16, 0, &rules)
        .unwrap();
    let before = sim.entities().keys_sorted();
    assert!(sim.spawn_sale_crew(&rules, None, source, 1, &[(18, 16)], FrameEffects::empty()));
    let crew: Vec<_> = sim
        .entities()
        .keys_sorted()
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect();
    assert_eq!(crew.len(), 1);
    let actor = sim.entities().get(crew[0]).unwrap();
    assert_eq!(
        sim.interner.resolve(actor.type_ref()),
        "E1",
        "the original crew producer consumes fourth-Side fallback, rather than stock Technician"
    );
    assert_eq!(actor.owner(), fourth);
    assert!(!actor.lifecycle.in_limbo());
    sim.debug_assert_logic_membership_consistent();
}

fn config(program: &Program) -> Value {
    let saved = serde_json::to_value(program).unwrap();
    let bytes: Vec<u8> = serde_json::from_value(saved["config"].clone()).unwrap();
    if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    }
}

fn non_faction_config(program: &Program) -> Value {
    let mut value = config(program);
    value.as_object_mut().unwrap().remove("factions");
    value["features"]
        .as_object_mut()
        .unwrap()
        .remove("ares_faction");
    value
}

const LIFETIME_ROOT: &str = "\
[Clear]\nBuildable=yes\n[Countries]\n0=PARENT\n1=Americans\n[Sides]\nGDI=Americans,PARENT\n\
[PARENT]\nSide=GDI\nVeteranBuildings=ROOT_B\n[Americans]\nSide=GDI\nParentCountry=PARENT\nVeteranInfantry=ROOT_I\n\
[InfantryTypes]\n0=ROOT_I\n1=MAP_I\n[ROOT_I]\nStrength=100\n[MAP_I]\nStrength=100\n\
[BuildingTypes]\n0=ROOT_B\n1=MAP_B\n[ROOT_B]\nStrength=100\n[MAP_B]\nStrength=100\n\
[VehicleTypes]\n0=TANK\n[TANK]\nStrength=100\nAmmo=6\nReloadAmount=2\nCloakable.Allowed=no\n\
[VeraFeatures]\nShield=no\nAresReload=yes\n";

#[test]
#[ignore = "requires VERA_SDK_FACTION_WASM and previous .15 in VERA_SDK_FACTION_PREVIOUS_WASM"]
fn sdk_faction_country_only_boundaries_and_exact_saved_adoption_keep_one_program_history() {
    let wasm = component("VERA_SDK_FACTION_WASM");
    let install = Install::new("faction-process-lifetime", LIFETIME_ROOT, Some(&wasm));
    let mut owner = install.owner();
    let digest = crate::util::sha256::sha256_hex(&wasm);
    assert_eq!(owner.sdk_configuration_program().unwrap().sha256(), digest);
    prepare_session(&mut owner);
    // PrepareSession publishes its completed Country suffix immediately; no
    // second visit or full ReadTypeData Side sweep is needed for this checkpoint.
    let early = config(owner.sdk_configuration_program().unwrap());
    assert!(early["factions"]["sides"].as_object().unwrap().is_empty());
    assert!(
        !early["factions"]["countries"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    let rules = load(
        &install,
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "[Americans]\nVeteranBuildings=MAP_B\nVeteranInfantry=MAP_I\n[TANK]\nReloadAmount=9\nCloakable.Allowed=yes\n\
         [VeraFeatures]\nShield=yes\nAresReload=no\n[CombatDamage]\nEMPAIRecoverMission=Hunt\n",
    );
    assert!(veteran(
        &rules,
        "Americans",
        ObjectCategory::Building,
        "MAP_B"
    ));
    assert!(veteran(
        &rules,
        "Americans",
        ObjectCategory::Infantry,
        "MAP_I"
    ));
    assert!(
        rules
            .object("TANK")
            .unwrap()
            .type_resource_policy()
            .unwrap()
            .reload
            .is_none()
    );
    let frozen: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(owner.saved_scenario_rules().unwrap()).unwrap(),
    )
    .unwrap();
    let before = owner.sdk_configuration_program().unwrap().clone();
    owner.refresh_multiplayer_country_bodies().unwrap();
    owner.refresh_multiplayer_country_bodies().unwrap();
    let after = owner.sdk_configuration_program().unwrap();
    assert_eq!(after.sha256(), digest);
    assert_eq!(
        non_faction_config(after),
        non_faction_config(&before),
        "Country-only dispatch must not replay root Shield, Reload, cloak or any other domain"
    );
    assert_eq!(
        config(after)["factions"]["sides"],
        config(&before)["factions"]["sides"]
    );
    let mut refreshed = owner.project_saved_scenario_rules(&frozen).unwrap();
    owner.project_multiplayer_countries(&mut refreshed);
    assert!(veteran(
        &refreshed,
        "Americans",
        ObjectCategory::Infantry,
        "ROOT_I"
    ));
    assert!(!veteran(
        &refreshed,
        "Americans",
        ObjectCategory::Infantry,
        "MAP_I"
    ));

    let mut sim = world(&rules);
    let building = sim
        .spawn_object("MAP_B", "Americans", 12, 12, 0, &rules)
        .unwrap();
    assert_eq!(sim.entities().get(building).unwrap().veterancy.rank(), 100);
    let terrain = sim.resolved_terrain.as_ref().unwrap().clone();
    let mut restored = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-FACTION", 0))
        .unwrap()
        .sim;
    let changed_root = LIFETIME_ROOT.replace("VeteranBuildings=ROOT_B", "VeteranBuildings=<none>");
    let current = Install::new("faction-current-input", &changed_root, Some(&wasm));
    let mut current_owner = current.owner();
    let current_rules = load(
        &current,
        &mut current_owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "",
    );
    assert!(
        !veteran(
            &current_rules,
            "Americans",
            ObjectCategory::Building,
            "MAP_B"
        ),
        "a fresh configuration really differs from the saved map selection"
    );
    let mut rebound = owner.project_saved_scenario_rules(&frozen).unwrap();
    crate::mod_sdk::adopt_saved_program(&mut rebound, &restored, &current.assets).unwrap();
    assert_eq!(
        crate::util::save_codec::serialize(rebound.external_mod.as_deref().unwrap()).unwrap(),
        crate::util::save_codec::serialize(sim.external_mod.as_ref().unwrap()).unwrap()
    );
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
        restored.entities().get(building).unwrap().veterancy.rank(),
        100
    );
    let next = restored
        .spawn_object("MAP_B", "Americans", 16, 12, 0, &rebound)
        .unwrap();
    assert_eq!(restored.entities().get(next).unwrap().veterancy.rank(), 100);
    let previous = component("VERA_SDK_FACTION_PREVIOUS_WASM");
    let wrong = Install::new(
        "faction-different-component",
        &changed_root,
        Some(&previous),
    );
    assert!(crate::mod_sdk::adopt_saved_program(&mut rebound, &restored, &wrong.assets).is_err());

    let generated = load(
        &install,
        &mut owner,
        NativeScenarioRulesPrefix::RandomMap(None),
        "",
    );
    assert_eq!(generated.external_mod.as_ref().unwrap().sha256(), digest);
    assert!(veteran(
        &generated,
        "Americans",
        ObjectCategory::Building,
        "ROOT_B"
    ));
    assert!(
        !veteran(&generated, "Americans", ObjectCategory::Building, "MAP_B"),
        "TypeReset clears the old one-shot parent attempt and copied list"
    );
    assert_eq!(
        generated.ares_emp_ai_recover_mission(),
        Some(MissionType::Hunt),
        "TypeReset keeps the same component's process globals"
    );
    let cleared = load(
        &install,
        &mut owner,
        NativeScenarioRulesPrefix::NonCampaign(None),
        "",
    );
    assert_eq!(cleared.external_mod.as_ref().unwrap().sha256(), digest);
    assert_eq!(
        cleared.ares_emp_ai_recover_mission(),
        None,
        "the real ScenarioClear clears the guest global before the same TypeReset/root path"
    );
    restored.debug_assert_logic_membership_consistent();
}
