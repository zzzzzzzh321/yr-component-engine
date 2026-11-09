//! Real .28 components through cold startup, scenario Rules, original Super
//! owners and saved Program adoption. Synthetic scenes are behavioral
//! regressions, not newly calculated native execution goldens.
use super::*;
use crate::assets::asset_manager::{
    AssetManager, MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::ProcessedRulesLayers;
use crate::rules::process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::superweapon_type::{GenericWarheadAiTargeting, SuperWeaponKind};
use crate::sim::command::Command;
use crate::sim::house_state::HouseState;
use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};
use crate::sim::snapshot::GameSnapshot;

const ART: &str =
    "[LAUNCHER]\nFoundation=1x1\n[TARGET]\nFoundation=1x1\n[DELIVERYBASE]\nFoundation=1x1\n";
const BASE: &str = "[Clear]\nBuildable=yes\n[VeraFeatures]\nAresProduction=no\n\
[Countries]\n0=Americans\n1=Russians\n2=Neutral\n\
[Americans]\nName=USAlias\nParentCountry=Americans\n[Russians]\nName=RUAlias\nParentCountry=Russians\n\
[Sides]\nGDI=Americans\nNod=Russians\nCivilian=Neutral\n\
[InfantryTypes]\n[VehicleTypes]\n[AircraftTypes]\n[BuildingTypes]\n0=LAUNCHER\n1=TARGET\n2=DELIVERYBASE\n\
[Warheads]\n0=Payload\n\
[LAUNCHER]\nStrength=200\nArmor=wood\nSuperWeapon=Strike\nSuperWeapon2=Delivery\nSuperWeapons=Sphere\n\
[TARGET]\nStrength=200\nArmor=wood\n[DELIVERYBASE]\nStrength=1000\nPower=200\nBaseNormal=yes\nAdjacent=1\n\
[SuperWeaponTypes]\n0=Reveal\n1=Strike\n2=Delivery\n3=Sphere\n4=Warp\n\
[Reveal]\nType=PsychicReveal\nIsPowered=no\n\
[Strike]\nType=GenericWarhead\nIsPowered=no\nRechargeTime=1\nSW.Warhead=Payload\nSW.Damage=75\n\
SW.RequiresTarget=building\nSW.RequiresHouse=enemies\nSW.FireIntoShroud=yes\nSW.InitialReady=yes\nSW.VirtualCharge=yes\nMoney.Amount=-100\n\
[Payload]\nCellSpread=1.5\nPercentAtMax=1\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
[Delivery]\nType=UnitDelivery\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\n\
Deliver.Types=DELIVERYBASE\nDeliver.Owner=Neutral\nDeliver.BaseNormal=no\nSW.Deferment=4\nMoney.Amount=-50\n\
[Sphere]\nType=ChronoSphere\nIsPowered=no\nPreClick=yes\nRechargeTime=1\nMoney.Amount=-500\n\
[Warp]\nType=ChronoWarp\nIsPowered=no\nPostClick=yes\nPreDependent=ChronoSphere\nMoney.Amount=-9000\n";

fn component(variable: &str) -> Vec<u8> {
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}
fn install(dir: &TestDirectory, module: &[u8]) {
    std::fs::write(dir.path().join("gameplay.wasm"), module).unwrap();
    std::fs::write(
        dir.path().join("vera-mod.json"),
        serde_json::to_vec(&serde_json::json!({
            "api_version": vera_mod_api::API_VERSION, "program": "gameplay.wasm",
            "sha256": crate::util::sha256::sha256_hex(module)
        }))
        .unwrap(),
    )
    .unwrap();
}
fn open(dir: &TestDirectory) -> AssetManager {
    crate::mod_sdk::open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap()
}
fn process(assets: &AssetManager) -> NativeRulesProcessOwner {
    let audio = crate::rules::audio_sources::AudioDefinitions::select(assets);
    crate::rules::retail_sources::RetailRulesSources::select(assets)
        .unwrap()
        .into_owner(audio.sounds().clone(), audio.eva().clone())
        .unwrap()
}
struct Scene {
    dir: TestDirectory,
    assets: AssetManager,
    owner: NativeRulesProcessOwner,
    rules: RuleSet,
    sim: Simulation,
}
impl Scene {
    fn new(label: &str, module: Option<&[u8]>, root: &str, map: &str) -> Self {
        Self::with_mode(label, module, root, None, map)
    }
    fn with_mode(
        label: &str,
        module: Option<&[u8]>,
        root: &str,
        mode: Option<&str>,
        map: &str,
    ) -> Self {
        Self::with_eva(label, module, root, mode, map, None)
    }
    fn with_eva(
        label: &str,
        module: Option<&[u8]>,
        root: &str,
        mode: Option<&str>,
        map: &str,
        eva: Option<&str>,
    ) -> Self {
        let dir = audio_test_install(label);
        if let Some(eva) = eva {
            std::fs::write(dir.path().join("evamd.ini"), eva).unwrap();
        }
        std::fs::write(dir.path().join("rulesmd.ini"), root).unwrap();
        std::fs::write(dir.path().join("artmd.ini"), ART).unwrap();
        if let Some(module) = module {
            install(&dir, module);
        }
        let assets = open(&dir);
        let mut owner = process(&assets);
        let mode = mode.map(IniFile::from_str);
        let (mut rules, ..) = owner
            .load_scenario(
                NativeScenarioRulesPrefix::NonCampaign(mode.as_ref()),
                &IniFile::from_str(map),
            )
            .unwrap()
            .into_parts();
        let mut sim = Simulation::with_seed(17);
        bind_scenario_gameplay_resources(
            &mut rules,
            &assets,
            ScenarioGameplayProgram::Fresh {
                simulation: &mut sim,
                rules_owner: &mut owner,
            },
            crate::map::theater::theater_extension("TEMPERATE").unwrap(),
            "TEMPERATE",
        )
        .unwrap();
        sim.intern_rule_type_ids(&rules);
        sim.resolve_type_handles(&rules);
        for (name, human) in [("Americans", true), ("Russians", false), ("Neutral", false)] {
            let id = sim.interner.intern(name);
            sim.houses
                .register(HouseState::new(id, 0, Some(id), human, 2000, 10));
            construct_house_supers(&mut sim, &rules, id, false);
        }
        sim.session.set_game_mode(
            crate::sim::scenario_session::ScenarioGameMode::from_legacy_nonzero(true),
        );
        sim.session.game_options_mut_for_test().super_weapons = true;
        sim.session.set_binary_frame_for_test(100);
        crate::sim::arena_fixture::flat_ground(&mut sim, &rules);
        crate::sim::arena_fixture::prepare_empty_map_authority(&mut sim, &rules);
        Self {
            dir,
            assets,
            owner,
            rules,
            sim,
        }
    }
    fn id(&self, name: &str) -> InternedId {
        self.sim.interner.get(name).unwrap()
    }
    fn launcher(&mut self) -> u64 {
        self.launcher_at(6, 6)
    }
    fn launcher_at(&mut self, rx: u16, ry: u16) -> u64 {
        let id = self
            .sim
            .spawn_object_at_height("LAUNCHER", "Americans", rx, ry, 0, 0, &self.rules)
            .unwrap();
        let owner = self.id("Americans");
        grant_provided_supers(&mut self.sim, &self.rules, owner);
        id
    }
    fn fire(&mut self, name: &str, cell: (u16, u16)) -> bool {
        let command = Command::LaunchSuperWeapon {
            sw_type_id: self.id(name),
            target_rx: cell.0,
            target_ry: cell.1,
        };
        self.sim.apply_command_with_overlays(
            "Americans",
            &command,
            &self.rules,
            None,
            FrameEffects::default(),
        )
    }
    fn roundtrip(&mut self) {
        self.roundtrip_with_reference_seed_check(true);
    }
    fn roundtrip_with_reference_seed_check(&mut self, requires_reference_seed: bool) {
        let exact =
            crate::util::save_codec::serialize(self.sim.external_mod.as_ref().unwrap()).unwrap();
        let mut retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
            &crate::util::save_codec::serialize(self.owner.saved_scenario_rules().unwrap())
                .unwrap(),
        )
        .unwrap();
        let bytes = GameSnapshot::save(
            &self.sim,
            0,
            self.rules.simulation_config_hash(),
            "SDK-SUPER",
            0,
        );
        let mut restored = GameSnapshot::load(&bytes).unwrap().sim;
        // GameSnapshot's lightweight test envelope deliberately omits physical
        // map resources. Supply the same terrain, then use both restore owners.
        restored.resolved_terrain = self.sim.resolved_terrain.clone();
        restored.restore_after_snapshot_load().unwrap();
        assert_eq!(
            self.owner.project_saved_scenario_rules(&retained).is_err(),
            requires_reference_seed,
            "only retained custom references require a transient SDK seed for projection"
        );
        // This lightweight envelope has no SavedScenarioResources wrapper.
        // Follow its admit_configuration_program owner: validate the exact
        // saved Program, then install that seed before projecting saved Rules.
        // Do not configure the current on-disk component or current INI here.
        let program = restored.external_mod.as_ref().unwrap();
        program.validate_restore().unwrap();
        retained.retain_sdk_configuration_program(Some(std::sync::Arc::new(program.clone())));
        let mut rebound = self.owner.project_saved_scenario_rules(&retained).unwrap();
        bind_scenario_gameplay_resources(
            &mut rebound,
            &self.assets,
            ScenarioGameplayProgram::Saved(&restored),
            crate::map::theater::theater_extension("TEMPERATE").unwrap(),
            "TEMPERATE",
        )
        .unwrap();
        restored.resolve_type_handles(&rebound);
        restored
            .restore_map_authority_after_snapshot_load(
                &rebound,
                &crate::rules::overlay_types::OverlayTypeRegistry::empty(),
                None,
            )
            .unwrap();
        assert_eq!(
            exact,
            crate::util::save_codec::serialize(restored.external_mod.as_ref().unwrap()).unwrap()
        );
        assert_eq!(
            exact,
            crate::util::save_codec::serialize(rebound.external_mod.as_deref().unwrap()).unwrap()
        );
        assert_eq!(
            self.rules.simulation_config_hash(),
            rebound.simulation_config_hash()
        );
        self.rules = rebound;
        self.sim = restored;
    }
}

fn stock_component() -> Vec<u8> {
    use vera_mod_api::{Context, HookKind, ModuleManifest, Plan, Request, SuperWeaponResourceContext};
    let request = Request {
        api_version: 1,
        context: Context::SuperWeaponResource(SuperWeaponResourceContext {}),
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut stock = unchanged.clone();
    stock.superweapon_resource = Some(Default::default());
    let mut read = unchanged.clone();
    read.super_reference_read = Some(vera_mod_api::SuperReferenceRead::Retain);
    vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: 1,
            id: "super-stock".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::ConfigureBoundary,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::ConfigureSuperRead,
                HookKind::SuperWeaponResource,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[
            ("configure_super_read", &serde_json::to_vec(&read).unwrap()),
            ("superweapon_resource", &serde_json::to_vec(&stock).unwrap()),
        ],
    )
}

#[test]
#[ignore = "requires VERA20K_ARES_COMPONENT (.28) and VERA20K_ARES_PREVIOUS_COMPONENT (.27)"]
fn sdk_super_read_chronology_provider_grant_and_stock_controls() {
    let current = component("VERA20K_ARES_COMPONENT");
    let previous = component("VERA20K_ARES_PREVIOUS_COMPONENT");
    let stock = stock_component();
    // A physical INI keeps duplicate section bodies; it does not merge them.
    // Extend each original body once and use the separate map for later reads.
    let root = BASE
        .replace("4=Warp\n", "4=Warp\n5=Late\n")
        .replace("[Reveal]\n", "[Reveal]\nSW.RequiredHouses=USAlias\n")
        .replace("[Strike]\n", "[Strike]\nSW.ForbiddenHouses=RUAlias\n")
        .replace(
            "SuperWeapon=Strike\nSuperWeapon2=Delivery\nSuperWeapons=Sphere\n",
            "SuperWeapon=Reveal\nSuperWeapon2=Late\nSuperWeapons=strike,Missing,Delivery\n",
        );
    // The physical parser omits an empty value. A comma is a present list
    // with no tokens, so this really replaces the earlier forbidden mask.
    // Ares common SW.Warhead100336F0/10008B00 is FindOnly. Declare the
    // intended WH before its Super read; that read never allocates a WH.
    let map = "[Warheads]\n0=LatePayload\n[Late]\nType=GenericWarhead\nSW.Warhead=LatePayload\n[LatePayload]\nCellSpread=2\n[Strike]\nSW.ForbiddenHouses=,\n";
    for (label, module, enabled) in [
        ("super-current", Some(current.as_slice()), true),
        ("super-none", None, crate::rules::ARES_ENABLED),
        (
            "super-previous",
            Some(previous.as_slice()),
            crate::rules::ARES_ENABLED,
        ),
        ("super-stock", Some(stock.as_slice()), false),
    ] {
        let mut scene = Scene::new(label, module, &root, map);
        assert_eq!(scene.rules.superweapon_semantics().custom_payloads, enabled);
        let strike = scene.rules.super_weapon("Strike").unwrap();
        if enabled {
            assert_eq!(strike.kind, SuperWeaponKind::GenericWarhead);
            assert_eq!(
                strike.generic_warhead.as_ref().unwrap().ai_targeting,
                GenericWarheadAiTargeting::Offensive
            );
            assert_eq!(
                scene
                    .rules
                    .super_weapon("Late")
                    .unwrap()
                    .generic_warhead
                    .as_ref()
                    .unwrap()
                    .ai_targeting,
                GenericWarheadAiTargeting::None
            );
            assert_eq!(
                scene.rules.warhead("LatePayload").unwrap().cell_spread_f64,
                2.0
            );
            let policy = scene
                .rules
                .super_weapon_metadata("Reveal")
                .unwrap()
                .availability();
            assert!(
                policy.allows_country(0),
                "Country Name alias resolves to the original index"
            );
            assert!(
                !policy.allows_country(1),
                "required mask remains independent of the cleared forbidden mask"
            );
            assert!(
                scene
                    .rules
                    .super_weapon_metadata("Strike")
                    .unwrap()
                    .availability()
                    .allows_country(1),
                "empty forbidden list replaces the earlier alias mask"
            );
            assert_eq!(
                scene.rules.additional_super_weapon_indices(0),
                if label == "super-current" {
                    Some(&[1, 2][..])
                } else {
                    // None retains the legacy object's provider vector; the
                    // common runtime iterator below must give the same order.
                    None
                }
            );
        } else {
            assert_eq!(strike.kind, SuperWeaponKind::NoEffect);
            assert!(scene.rules.warhead("LatePayload").is_some());
        }
        let building = scene.launcher();
        let names: Vec<_> = provided_supers(
            &scene.sim,
            &scene.rules,
            scene.sim.entities().get(building).unwrap(),
        )
        .collect();
        assert_eq!(
            names,
            if enabled {
                vec!["Reveal", "Late", "Strike", "Delivery"]
            } else {
                vec!["Reveal", "Late"]
            }
        );
        let (owner, strike) = (scene.id("Americans"), scene.id("Strike"));
        assert_eq!(
            scene.sim.super_weapons[&owner][&strike].is_active(),
            enabled
        );
        if enabled {
            assert!(scene.sim.super_weapons[&owner][&strike].is_ready());
        }
        #[cfg(feature = "phobos")]
        {
            // Real map reads clear both stock slots. Keep the extra Strike
            // uncharged so a wrong fallback to Delivery cannot look identical.
            let display_map = format!(
                "{}[LAUNCHER]\nSuperWeapon=none\nSuperWeapon2=none\n",
                map.replace(
                    "SW.ForbiddenHouses=,\n",
                    "SW.ForbiddenHouses=,\nSW.InitialReady=no\n"
                )
            );
            let mut display = Scene::new(&format!("{label}-display"), module, &root, &display_map);
            let building = display.launcher();
            let entity = display.sim.entities().get(building).unwrap();
            let object = display
                .sim
                .object_type(entity.type_ref(), &display.rules)
                .unwrap();
            let retained =
                &display.sim.super_weapons[&display.id("Americans")][&display.id("Strike")];
            let expected = enabled.then_some((
                retained.charge_remaining(display.sim.session.binary_frame() as i32),
                retained.charge_duration(),
            ));
            let value = |index| {
                crate::sim::digital_display::values_for_display(
                    &display.sim,
                    entity,
                    object,
                    &display.rules,
                    crate::rules::digital_display::DigitalInfoType::SuperWeapon,
                    index,
                    None,
                )
            };
            assert_eq!(value(1), None, "{label}: empty primary retains slot1");
            assert_eq!(value(2), None, "{label}: empty secondary retains slot2");
            assert_eq!(
                value(3),
                expected,
                "{label}: first additional definition is slot3"
            );
            assert_eq!(value(0), expected, "{label}: zero chooses first set slot");
            assert_eq!(
                value(i32::MAX),
                expected,
                "{label}: out-of-range chooses first set slot"
            );
        }
    }
}

#[path = "sdk_nuclear_tests.rs"]
mod nuclear;

#[path = "sdk_common_targeting_tests.rs"]
mod common_targeting;

#[path = "sdk_influence_tests.rs"]
mod influence;

#[path = "sdk_eva_tests.rs"]
mod eva;

#[cfg(not(feature = "ares"))]
#[test]
#[ignore = "requires VERA20K_ARES_COMPONENT (.28)"]
fn sdk_super_generic_transaction_history_and_exact_saved_program() {
    let current = component("VERA20K_ARES_COMPONENT");
    let mut scene = Scene::new("super-generic", Some(&current), BASE, "");
    let launcher = scene.launcher();
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &scene.rules)
        .unwrap();
    let (owner, strike) = (scene.id("Americans"), scene.id("Strike"));
    let credits = scene.sim.houses[&owner].economy().credits();
    let hash = scene.sim.state_hash();
    assert!(
        scene
            .sim
            .super_weapon_target_check(&scene.rules, owner, strike, (6, 6), false)
            .is_err(),
        "the cursor rejects an owner target before queueing an event"
    );
    assert_eq!(scene.sim.state_hash(), hash);
    let previous = component("VERA_SDK_COMMON_TARGETING_PREVIOUS_WASM");
    let mut old = Scene::new("super-generic-old-preflight", Some(&previous), BASE, "");
    old.launcher();
    let old_hash = old.sim.state_hash();
    assert!(
        !old.fire("Strike", (6, 6)),
        "historical Generic ABI retains its extra final-event preflight"
    );
    assert_eq!(old.sim.state_hash(), old_hash);
    assert_eq!(
        scene.sim.entities().get(launcher).unwrap().health.current(),
        200
    );
    assert!(scene.fire("Strike", (20, 20)));
    assert!(scene.sim.entities().get(target).unwrap().health.current() < 200);
    assert_eq!(scene.sim.houses[&owner].economy().credits(), credits - 100);
    assert_eq!(
        scene.sim.super_weapons[&owner][&strike].ares_launch_count,
        1
    );
    scene
        .sim
        .super_weapons
        .instance_mut(&owner, &strike)
        .unwrap()
        .deactivate();
    // Changed current text must not replace the exact saved Program's config.
    std::fs::write(
        scene.dir.path().join("rulesmd.ini"),
        BASE.replace("Money.Amount=-100", "Money.Amount=-900"),
    )
    .unwrap();
    scene.assets = open(&scene.dir);
    scene.roundtrip();
    scene.sim.session.set_binary_frame_for_test(500);
    grant_provided_supers(&mut scene.sim, &scene.rules, owner);
    let instance = &scene.sim.super_weapons[&owner][&strike];
    assert!(!instance.is_ready(), "initial-ready is first-success only");
    assert_eq!(
        (instance.charge_start_tick(), instance.charge_duration()),
        (100, 900)
    );
    assert_eq!(
        scene.rules.super_weapon("Strike").unwrap().money_amount,
        -100
    );
    scene
        .sim
        .super_weapons
        .instance_mut(&owner, &strike)
        .unwrap()
        .deactivate();
    scene.sim.session.set_binary_frame_for_test(1000);
    grant_provided_supers(&mut scene.sim, &scene.rules, owner);
    assert!(
        scene.sim.super_weapons[&owner][&strike].is_ready(),
        "virtual recharge uses the retained original deadline"
    );

    // Generic vtable100A8ECC inherits common CanFireAt1006CDA0: a null WH
    // fails Activate10071689, after ClickFire, rather than cursor selection.
    let mut refused = Scene::new(
        "super-generic-null-wh",
        Some(&current),
        BASE,
        "[Strike]\nSW.Warhead=none\n",
    );
    refused.launcher();
    let target = refused
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &refused.rules)
        .unwrap();
    let (owner, strike) = (refused.id("Americans"), refused.id("Strike"));
    let credits = refused.sim.houses[&owner].economy().credits();
    assert_eq!(
        refused
            .sim
            .super_weapon_target_check(&refused.rules, owner, strike, (20, 20), false),
        Ok(())
    );
    assert!(refused.fire("Strike", (20, 20)));
    assert!(!refused.sim.super_weapons[&owner][&strike].is_ready());
    assert_eq!(
        refused.sim.super_weapons[&owner][&strike].ares_launch_count,
        0
    );
    assert_eq!(refused.sim.houses[&owner].economy().credits(), credits);
    assert_eq!(
        refused.sim.entities().get(target).unwrap().health.current(),
        200
    );
    // This scene has no non-null custom WH/Weapon reference to reject during
    // bare Rules projection; the exact saved Program still needs admission.
    refused.roundtrip_with_reference_seed_check(false);
}

#[cfg(not(feature = "ares"))]
#[test]
#[ignore = "requires VERA20K_ARES_COMPONENT (.28)"]
fn sdk_super_delivery_restore_and_two_stage_source_settlement() {
    let current = component("VERA20K_ARES_COMPONENT");
    let mut scene = Scene::new("super-delivery-pair", Some(&current), BASE, "");
    scene.launcher();
    let owner = scene.id("Americans");
    let payload_count = |sim: &Simulation| {
        sim.entities()
            .values()
            .filter(|entity| {
                sim.interner
                    .resolve(entity.type_ref())
                    .eq_ignore_ascii_case("DELIVERYBASE")
            })
            .count()
    };
    assert!(scene.fire("Delivery", (12, 10)));
    assert_eq!(payload_count(&scene.sim), 0);
    assert_eq!(scene.sim.houses[&owner].economy().credits(), 1950);
    scene.roundtrip();
    scene.sim.session.set_binary_frame_for_test(103);
    tick_active_superweapon_effects(&mut scene.sim, &scene.rules, None, FrameEffects::default());
    assert_eq!(payload_count(&scene.sim), 0);
    scene.sim.session.set_binary_frame_for_test(104);
    tick_active_superweapon_effects(&mut scene.sim, &scene.rules, None, FrameEffects::default());
    assert_eq!(payload_count(&scene.sim), 1);
    let delivered = scene
        .sim
        .entities()
        .values()
        .find(|entity| {
            scene
                .sim
                .interner
                .resolve(entity.type_ref())
                .eq_ignore_ascii_case("DELIVERYBASE")
        })
        .unwrap();
    let delivered_id = delivered.stable_id();
    assert_eq!(delivered.owner(), scene.id("Neutral"));
    assert!(!delivered.unit_delivery_provides_base_space());
    scene.roundtrip();
    assert!(
        !scene
            .sim
            .entities()
            .get(delivered_id)
            .unwrap()
            .unit_delivery_provides_base_space()
    );
    let sphere = scene.id("Sphere");
    // Original charging owner, not an additional test readiness implementation.
    scene
        .sim
        .super_weapons
        .instance_mut(&owner, &sphere)
        .unwrap()
        .set_charged(scene.rules.super_weapon("Sphere").unwrap(), 104);
    assert!(scene.fire("Sphere", (10, 10)));
    let old = scene.sim.paid_super_weapon_selection(owner).unwrap();
    assert_eq!(scene.sim.houses[&owner].economy().credits(), 1950);
    assert!(scene.sim.apply_command_with_overlays(
        "Americans",
        &Command::CancelPaidSuperWeapon { selection: old },
        &scene.rules,
        None,
        FrameEffects::default()
    ));
    assert!(scene.fire("Sphere", (11, 11)));
    let ticket = scene.sim.paid_super_weapon_selection(owner).unwrap();
    assert!(ticket.sequence > old.sequence);
    let hash = scene.sim.state_hash();
    assert!(!scene.sim.apply_command_with_overlays(
        "Americans",
        &Command::CancelPaidSuperWeapon { selection: old },
        &scene.rules,
        None,
        FrameEffects::default()
    ));
    assert_eq!(scene.sim.state_hash(), hash);
    scene.roundtrip();
    scene.sim.session.set_binary_frame_for_test(105);
    tick_house_superweapon_instances(&mut scene.sim, &scene.rules, owner);
    assert!(scene.sim.apply_command_with_overlays(
        "Americans",
        &Command::CompletePaidSuperWeapon {
            selection: ticket,
            target_rx: 15,
            target_ry: 15
        },
        &scene.rules,
        None,
        FrameEffects::default()
    ));
    assert_eq!(
        scene.sim.houses[&owner].economy().credits(),
        1450,
        "source -500 settles once; destination -9000 does not"
    );
    assert!(scene.sim.paid_super_weapon_selection(owner).is_none());
}
