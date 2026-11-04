//! Actual early component admission and the app's pending external-launch
//! transition, without a window or audio device. Community frame decoding is
//! exercised without installing its one-shot process-global Snapshot.

use super::{App, process_assets::ProcessAssets};
use crate::app::frontend::cncnet_spawn::{PendingSpawnRequest, SpawnGame};
use crate::app::match_bootstrap::{MatchSeedClock, MatchSeedSource};
use crate::assets::asset_manager::{
    MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::rules::{
    art_data::ArtRegistry, ini_parser::IniFile, process_owner::NativeScenarioRulesPrefix,
};
use crate::sim::{
    runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources},
    world::Simulation,
};
use base64::{Engine, engine::general_purpose::STANDARD};

const MAP: &[u8] = b"[Map]\nTheater=TEMPERATE\nSize=0,0,2,1\nLocalSize=0,0,2,1\n[IsoMapPack5]\n1=DwALABwBAAIA/////wAAABEAAA==\n";
const SPAWN: &str = "[Settings]\nName=Commander\nScenario=spawnmap.ini\nPlayerCount=1\nSide=10\nColor=0\nAIPlayers=1\nSeed=137\nGameSpeed=1\nSpecialHouseIsAlly=No\n[HouseHandicaps]\nMulti2=1\n[HouseCountries]\nMulti2=10\n[HouseColors]\nMulti2=1\n";

fn country_rules() -> String {
    // Same retained Country/Side/BaseUnit chain as the existing CnCNet
    // regression. C11 is physically declared even for a no-hook component.
    let mut ini = String::from("[Clear]\nBuildable=yes\n[Countries]\n");
    for index in 0..12 {
        ini.push_str(&format!("{index}=C{index:02}\n"));
    }
    ini.push_str("12=Neutral\n13=Special\n[Sides]\nGDI=C00,C01,C02\nNod=C03,C04,C05\nThirdSide=C06,C07,C08\nFourthSide=C09,C10,C11\nCivilian=Neutral\nMutant=Special\n[General]\nBaseUnit=BASEMCV,FOURMCV\n[VehicleTypes]\n0=BASEMCV\n1=FOURMCV\n[BASEMCV]\nOwner=C00,C01,C02,C03,C04,C05,C06,C07,C08\nStrength=100\n[FOURMCV]\nOwner=C09,C10,C11\nStrength=100\n");
    for index in 0..12 {
        let side = ["GDI", "Nod", "ThirdSide", "FourthSide"][index / 3];
        ini.push_str(&format!("[C{index:02}]\nSide={side}\nMultiplay=yes\n"));
    }
    ini
}

fn fixture(label: &str, module: Option<&[u8]>) -> (TestDirectory, ProcessAssets) {
    let directory = audio_test_install(label);
    std::fs::create_dir(directory.path().join("client")).unwrap();
    for (name, bytes) in [
        ("rulesmd.ini", country_rules().into_bytes()),
        ("artmd.ini", vec![]), ("soundmd.ini", vec![]), ("evamd.ini", vec![]),
        ("battlemd.ini", b"[Battles]\n0=FIRST\n1=SECOND\n[FIRST]\nCD=2\nScenario=FIRST.MAP\n[SECOND]\nCD=1\nScenario=SECOND.MAP\n".to_vec()),
        ("client/spawn.ini", SPAWN.as_bytes().to_vec()), ("client/spawnmap.ini", MAP.to_vec()),
        ("client/other.Map", MAP.to_vec()),
    ] { std::fs::write(directory.path().join(name), bytes).unwrap(); }
    if let Some(module) = module {
        std::fs::write(directory.path().join("gameplay.wasm"), module).unwrap();
        std::fs::write(
            directory.path().join("vera-mod.json"),
            serde_json::to_vec(&serde_json::json!({
                "api_version": vera_mod_api::API_VERSION, "program": "gameplay.wasm",
                "sha256": crate::util::sha256::sha256_hex(module),
            }))
            .unwrap(),
        )
        .unwrap();
    }
    // The one admitted AssetManager goes into the real process owner. No
    // probe manager or second component read supplies a launch decision.
    let assets =
        crate::mod_sdk::open_active_assets(directory.path(), MediaArchiveMode::STOCK_DIGITAL)
            .unwrap();
    let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
    process.initialize_sources_if_needed(&assets).unwrap();
    process.return_from_loading(assets);
    (directory, process)
}

fn read_client(directory: &TestDirectory, text: &str) -> PendingSpawnRequest {
    let path = directory.path().join("client/spawn.ini");
    std::fs::write(&path, text).unwrap();
    let pending = PendingSpawnRequest::read(&path).unwrap();
    // The Ready transition must use retained bytes, not reread a replaced INI.
    std::fs::write(&path, b"not a launch request").unwrap();
    pending
}

fn community_frame(spawn: &str, protocol: u32, map: &str) -> Vec<u8> {
    let body = serde_json::to_vec(&serde_json::json!({
        "protocol": protocol, "release": "spawn-sdk-test", "spawn": spawn,
        "resources": [{"name": map, "data": STANDARD.encode(MAP)}],
    }))
    .unwrap();
    let mut frame = (body.len() as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(&body);
    frame
}

fn scenario_rules(process: &mut ProcessAssets) -> crate::rules::ruleset::RuleSet {
    let assets = process.lease_for_loading().unwrap();
    let owner = process.native_rules_mut_with_tile_cache().0.unwrap();
    let (mut rules, _, art, _) = owner
        .load_scenario(
            NativeScenarioRulesPrefix::NonCampaign(None),
            &IniFile::empty(),
        )
        .unwrap()
        .into_parts();
    rules.install_art_data(ArtRegistry::from_ini(&art));
    let mut simulation = Simulation::with_seed(137);
    bind_scenario_gameplay_resources(
        &mut rules,
        &assets,
        ScenarioGameplayProgram::Fresh {
            simulation: &mut simulation,
            rules_owner: owner,
        },
        "tem",
        "TEMPERATE",
    )
    .unwrap();
    process.return_from_loading(assets);
    rules
}

#[derive(Default)]
struct CountingClock(usize);
impl MatchSeedClock for CountingClock {
    fn low_u32(&mut self) -> u32 {
        self.0 += 1;
        137
    }
    fn source(&self) -> MatchSeedSource {
        MatchSeedSource::Controlled
    }
    fn seed_authority_certifying(&self) -> bool {
        false
    }
}

#[test]
#[ignore = "requires VERA_SDK_SPAWN_IDENTITY_WASM / VERA_SDK_SPAWN_IDENTITY_PREVIOUS_WASM"]
fn component_spawn_identity_approves_pending_client_and_community_launches() {
    use vera_mod_api::{Context, HookKind, ModuleManifest, Plan, Request, SpawnIdentityPolicy};
    use crate::sim::scenario_bootstrap::{
        MatchLaunchDescriptor, normalized_launch_slots, populate_launch_houses,
        launch_mcv_type_for_country,
    };
    use crate::sim::house_state::house_state_for_owner;
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    let component = std::fs::read(
        std::env::var_os("VERA_SDK_SPAWN_IDENTITY_WASM").expect("set VERA_SDK_SPAWN_IDENTITY_WASM"),
    )
    .unwrap();
    let previous = std::fs::read(
        std::env::var_os("VERA_SDK_SPAWN_IDENTITY_PREVIOUS_WASM")
            .expect("set VERA_SDK_SPAWN_IDENTITY_PREVIOUS_WASM"),
    )
    .unwrap();
    let request = Request {
        api_version: vera_mod_api::API_VERSION,
        context: Context::SpawnIdentity,
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut selected = unchanged.clone();
    selected.spawn_identity = Some(SpawnIdentityPolicy {
        extended_countries: false,
        campaign_side_byte: false,
    });
    let off = vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: vera_mod_api::API_VERSION,
            id: "test.spawn-stock".into(),
            version: "1".into(),
            config_version: 1,
            state_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::SpawnIdentity,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[("spawn_identity", &serde_json::to_vec(&selected).unwrap())],
    );
    for (label, module, current) in [
        ("spawn-current", Some(component.as_slice()), true),
        ("spawn-no-module", None, false),
        ("spawn-previous", Some(previous.as_slice()), false),
        ("spawn-explicit-stock", Some(off.as_slice()), false),
    ] {
        eprintln!("spawn identity app control: {label}");
        let (directory, mut process) = fixture(label, module);
        let pending = read_client(&directory, SPAWN);
        assert!(matches!(pending.game(), SpawnGame::Skirmish(_)));
        let client = App::resolve_pending_spawn(pending, &process);
        assert_eq!(
            client.is_ok(),
            current,
            "Country10 local and AI admission: {label}"
        );
        let frame = community_frame(SPAWN, 1, "spawnmap.ini");
        let (pending, snapshot, network) = super::community::decode(frame.as_slice()).unwrap();
        assert!(snapshot.contains("spawnmap.ini"));
        assert!(network.is_none());
        let community = App::resolve_pending_spawn(pending, &process);
        assert_eq!(
            community.is_ok(),
            current,
            "same frozen policy for community: {label}"
        );
        // A source registry containing C11 is never an admission capability.
        let source11 =
            SPAWN
                .replacen("Side=10", "Side=11", 1)
                .replacen("Multi2=10", "Multi2=11", 1);
        assert_eq!(
            App::resolve_pending_spawn(read_client(&directory, &source11), &process).is_ok(),
            current
        );
        let stock_text = SPAWN
            .replace("Side=10", "Side=0")
            .replace("Multi2=10", "Multi2=1");
        assert!(
            App::resolve_pending_spawn(read_client(&directory, &stock_text), &process).is_ok(),
            "positive stock admission"
        );
        if current {
            let SpawnGame::Skirmish(session) = client.unwrap().game else {
                panic!("client Battle");
            };
            assert_eq!(
                session
                    .selected_map_file
                    .as_deref()
                    .map(std::path::Path::new),
                Some(directory.path().join("client/spawnmap.ini").as_path())
            );
            let selected = session.selected_map_file.as_deref().unwrap();
            let loaded = crate::map::source::load_map_by_name_or_path_with_assets(
                directory.path(),
                selected,
                process.manager().unwrap(),
            )
            .unwrap();
            assert!(
                matches!(loaded.source, crate::map::source::LoadedMapSource::Loose { path, .. }
                if path == directory.path().join("client/spawnmap.ini"))
            );
            let SpawnGame::Skirmish(community_session) = community.unwrap().game else {
                panic!("community Battle");
            };
            assert_eq!(community_session.local_slot().country.raw_index(), 10);
            assert_eq!(community_session.opponents[0].country.raw_index(), 10);
            std::fs::write(
                directory.path().join("client/spawn.ini"),
                SPAWN.replace("spawnmap.ini", "missing.ini"),
            )
            .unwrap();
            assert!(
                PendingSpawnRequest::read(&directory.path().join("client/spawn.ini")).is_err(),
                "missing client map has no retail fallback"
            );
            assert_eq!(session.local_slot().country.raw_index(), 10);
            assert_eq!(session.opponents[0].country.raw_index(), 10);
            crate::app::frontend::skirmish_session::prepare_external_skirmish_countries(
                &mut process,
                &session,
            )
            .unwrap();
            let rules = scenario_rules(&mut process);
            assert_eq!(
                rules.country_name(crate::rules::ruleset::CountryIdx(11)),
                Some("C11")
            );
            let launch = MatchLaunchDescriptor::from_resolved(*session).unwrap();
            launch.validate_countries(&rules).unwrap();
            let slots = normalized_launch_slots(launch.session());
            let mut simulation = Simulation::new();
            let rng = simulation.rng_state();
            populate_launch_houses(&mut simulation, &slots, &rules);
            for slot in &slots {
                let house = house_state_for_owner(
                    simulation.houses(),
                    &slot.owner_name,
                    simulation.interner(),
                )
                .unwrap();
                assert_eq!(house.side_index(), 3);
                assert_eq!(simulation.interner().resolve(house.house_type_id()), "C10");
            }
            assert_eq!(
                launch_mcv_type_for_country(launch.session().local_slot().country, &rules),
                Some("FOURMCV")
            );
            assert_eq!(
                launch_mcv_type_for_country(launch.session().opponents[0].country, &rules),
                Some("FOURMCV")
            );
            assert_eq!(
                simulation.rng_state(),
                rng,
                "resolved identities add no draw"
            );
            for text in [
                SPAWN.replacen("Side=10", "Side=65535", 1),
                SPAWN.replacen("Multi2=10", "Multi2=65535", 1),
            ] {
                let request =
                    App::resolve_pending_spawn(read_client(&directory, &text), &process).unwrap();
                let SpawnGame::Skirmish(session) = request.game else {
                    panic!("Battle");
                };
                let launch = MatchLaunchDescriptor::from_resolved(*session).unwrap();
                assert!(
                    launch.validate_countries(&rules).is_err(),
                    "u16 representation does not allocate Country65535"
                );
            }
        }
        for side in [1, 10, 255] {
            let text = format!(
                "[Settings]\nIsSinglePlayer=Yes\nScenario=other.Map\nSide={side}\nCampaignID=1\nGameSpeed=2\nDifficultyModeHuman=2\nDifficultyModeComputer=0\n"
            );
            let result = App::resolve_pending_spawn(read_client(&directory, &text), &process);
            assert_eq!(
                result.is_ok(),
                current || side <= 1,
                "campaign metadata byte: {label} Side{side}"
            );
            if let Ok(request) = result {
                let SpawnGame::Campaign(campaign) = request.game else {
                    panic!("campaign");
                };
                assert_eq!(campaign.side, side);
                let mut clock = CountingClock::default();
                let startup = campaign
                    .prepare_startup(process.campaigns().unwrap(), request.game_speed, &mut clock)
                    .unwrap();
                assert_eq!(
                    startup.campaign_index(),
                    0,
                    "Side/CampaignID/Scenario never select a record"
                );
                assert_eq!(startup.campaign().id(), "FIRST");
                assert_eq!(startup.campaign().cd(), 2);
                assert_eq!(startup.scenario(), "OTHER.MAP");
                assert_eq!(
                    std::path::Path::new(startup.selected_map_file()),
                    directory.path().join("client/other.Map")
                );
                assert_eq!(clock.0, 1);
            }
        }
        let assets = process.lease_for_loading().unwrap();
        let pending = read_client(&directory, SPAWN);
        assert_eq!(
            App::resolve_pending_spawn(pending, &process).is_ok(),
            current,
            "the frozen recipe survives the original manager's lease"
        );
        process.return_from_loading(assets);
    }
    for value in ["-1", "65536"] {
        assert!(
            PendingSpawnRequest::parse(&SPAWN.replacen("Side=10", &format!("Side={value}"), 1))
                .is_err()
        );
        assert!(
            PendingSpawnRequest::parse(&SPAWN.replacen("Multi2=10", &format!("Multi2={value}"), 1))
                .is_err()
        );
    }
    for side in [-1, 256] {
        assert!(
            PendingSpawnRequest::parse(&format!(
                "[Settings]\nIsSinglePlayer=Yes\nScenario=other.Map\nSide={side}\n"
            ))
            .is_err(),
            "campaign byte representation"
        );
    }
    // Existing frame/map/package admission remains before the pending Ready
    // transition. No shared Snapshot is installed by this lib test.
    assert!(
        super::community::decode(community_frame(SPAWN, 1, "different.map").as_slice()).is_err()
    );
    assert!(
        super::community::decode(community_frame(SPAWN, 2, "spawnmap.ini").as_slice()).is_err(),
        "signed protocol still requires its package"
    );
}
