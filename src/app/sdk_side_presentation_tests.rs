//! Real component source/consumer joins. No window, device or second Side state.
#![cfg(not(feature = "ares"))]
use crate::app::process_assets::ProcessAssets;
use crate::assets::asset_manager::{
    MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::map::source::test_support::make_new_format_mix_bytes;
use crate::rules::{ini_parser::IniFile, process_owner::NativeScenarioRulesPrefix, ruleset::RuleSet};
use crate::sim::{
    runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources},
    world::Simulation,
};
use serde_json::Value;

fn component(variable: &str) -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}

fn fixture(
    label: &str,
    root: &str,
    module: Option<&[u8]>,
    files: &[(&str, &[u8])],
) -> (TestDirectory, ProcessAssets) {
    let directory = audio_test_install(label);
    for (name, bytes) in [
        ("rulesmd.ini", root.as_bytes()),
        ("artmd.ini", b"".as_slice()),
        ("soundmd.ini", b"".as_slice()),
        ("evamd.ini", b"".as_slice()),
    ]
    .into_iter()
    .chain(files.iter().copied())
    {
        std::fs::write(directory.path().join(name), bytes).unwrap();
    }
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
    // All loose resources exist before the manager freezes its name catalog.
    let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
    let manager = process.reconstruct_manager(directory.path()).unwrap();
    process.initialize_sources_if_needed(&manager).unwrap();
    process.return_from_loading(manager);
    (directory, process)
}

fn load(process: &mut ProcessAssets, map: &str) -> (RuleSet, Simulation) {
    let assets = process.lease_for_loading().unwrap();
    let owner = process.native_rules_mut_with_tile_cache().0.unwrap();
    let (mut rules, ..) = owner
        .load_scenario(
            NativeScenarioRulesPrefix::NonCampaign(None),
            &IniFile::from_str(map),
        )
        .unwrap()
        .into_parts();
    let mut simulation = Simulation::with_seed(41);
    // The public fresh-scenario binder publishes the completed configuration to
    // the same process authority; app tests never reach its private seed.
    bind_scenario_gameplay_resources(
        &mut rules,
        &assets,
        ScenarioGameplayProgram::Fresh {
            simulation: &mut simulation,
            rules_owner: owner,
        },
        crate::map::theater::theater_extension("TEMPERATE").unwrap(),
        "TEMPERATE",
    )
    .unwrap();
    process.return_from_loading(assets);
    (rules, simulation)
}

const ROOT: &str = "[Countries]\n0=Americans\n1=Russians\n2=YuriCountry\n3=FourthCountry\n\
    [Sides]\nGDI=Americans\nNod=Russians\nThirdSide=YuriCountry\nFourthSynthetic=FourthCountry\n\
    [Americans]\nSide=GDI\n[Russians]\nSide=Nod\n[YuriCountry]\nSide=ThirdSide\n[FourthCountry]\nSide=FourthSynthetic\n\
    [FourthSynthetic]\nSidebar.MixFileIndex=4\nSidebar.YuriFileNames=yes\nEVA.Tag=VoiceB\n\
    CampaignScore.Background=early.shp\nCampaignScore.UnderParTheme=COUNTRY_SCORE\nMultiplayerScore.Bars=early~~.pcx\n";

#[test]
#[ignore = "requires VERA_SDK_SIDE_PRESENTATION_WASM and .17 VERA_SDK_SIDE_PRESENTATION_PREVIOUS_WASM"]
fn sdk_side_sidebar_dispatch_and_saved_candidate_restore_original_vfs_authority() {
    use crate::render::sidebar_chrome::{SidebarChromeRoute, sidebar_route_for_side};
    let wasm = component("VERA_SDK_SIDE_PRESENTATION_WASM");
    let previous = component("VERA_SDK_SIDE_PRESENTATION_PREVIOUS_WASM");
    let base = make_new_format_mix_bytes(&[("SIDEBAR.PAL", &[12; 768])]);
    let fourth = make_new_format_mix_bytes(&[("SIDEBAR.PAL", &[20; 768])]);
    let partial = make_new_format_mix_bytes(&[("partial.bin", b"mounted before missing base")]);
    let native = crate::test_fixture::json("tools/rules_oracle/ares_side_sidebar.json");
    let expected = &native["retained"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "explicit_extra")
        .unwrap()["after"];
    for (label, module, root, registered) in [
        ("side-no-module", None, ROOT.to_owned(), false),
        (
            "side-old17",
            Some(previous.as_slice()),
            ROOT.to_owned(),
            false,
        ),
        ("side-new18", Some(wasm.as_slice()), ROOT.to_owned(), true),
        (
            "side-explicit-stock",
            Some(wasm.as_slice()),
            format!("{ROOT}[VeraFeatures]\nAresFaction=no\n"),
            false,
        ),
    ] {
        let (_directory, mut process) = fixture(
            label,
            &root,
            module,
            &[
                ("sidec01.mix", &base),
                ("sidec04.mix", &fourth),
                ("sidec05md.mix", &partial),
            ],
        );
        let (rules, _) = load(&mut process, "");
        assert_eq!(rules.registered_side_ui_enabled(), registered);
        let route = sidebar_route_for_side(&rules, 3).unwrap();
        if !registered {
            assert_eq!(route, SidebarChromeRoute::Retail);
            continue;
        }
        let SidebarChromeRoute::Registered {
            side,
            mix_index,
            yuri_names,
        } = route
        else {
            panic!("registered route")
        };
        assert_eq!(side, 3);
        assert_eq!(
            mix_index,
            expected["mix_file_index"].as_i64().unwrap() as i32
        );
        assert_eq!(yuri_names, expected["yuri_file_names"].as_bool().unwrap());
        assert!(sidebar_route_for_side(&rules, -1).is_err());
        assert!(sidebar_route_for_side(&rules, 4).is_err());
        let mut assets = process.lease_for_loading().unwrap();
        assert!(assets.activate_sidebar_archives(1).unwrap());
        assert_eq!(
            assets
                .load_file_from_mix("SIDEBAR.PAL")
                .unwrap()
                .bytes
                .as_ref(),
            &[12; 768]
        );
        assert!(assets.activate_sidebar_archives(mix_index).unwrap());
        let retained = assets.load_file_from_mix("SIDEBAR.PAL").unwrap();
        assert_eq!(
            retained.bytes.as_ref(),
            &[20; 768],
            "successful Side switch retires the old palette cache"
        );
        assert_eq!(retained.source_archive.as_ref(), "sidec04.mix");
        let failed: Result<(), &str> = assets.prepare_saved_load(|candidate| {
            assert!(candidate.activate_sidebar_archives(1).unwrap());
            assert_eq!(
                candidate
                    .load_file_from_mix("SIDEBAR.PAL")
                    .unwrap()
                    .bytes
                    .as_ref(),
                &[12; 768]
            );
            assert!(
                !candidate.activate_sidebar_archives(5).unwrap(),
                "required SIDEC05 is absent"
            );
            assert_eq!(
                candidate.get_ref("partial.bin"),
                Some(b"mounted before missing base".as_slice())
            );
            Err("the candidate failed after partial MD registration")
        });
        assert!(failed.is_err());
        assert_eq!(
            assets.resolve_ref("SIDEBAR.PAL").unwrap().source_archive,
            "sidec04.mix"
        );
        let restored = assets.load_file_from_mix("SIDEBAR.PAL").unwrap();
        assert_eq!(restored.bytes, retained.bytes);
        assert_eq!(restored.source_archive, retained.source_archive);
        assert!(assets.get_ref("partial.bin").is_none());
        process.return_from_loading(assets);
    }
}

fn eva_control(kind: &str, field: &str, value: i32) -> &'static Value {
    crate::test_fixture::json("tools/spatial_oracle/ares_eva.json")["read_bind"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == kind && row[field] == value)
        .unwrap()
}

#[test]
#[ignore = "requires the real .18 component in VERA_SDK_SIDE_PRESENTATION_WASM"]
fn sdk_side_scenario_identity_binds_the_original_vox_queue_and_fixed_catalog_at_dequeue() {
    use crate::audio::vox::{VoxQueue, VoxRequest};
    let wasm = component("VERA_SDK_SIDE_PRESENTATION_WASM");
    let eva = b"[EVATypes]\n0=VoiceA\n1=VoiceB\n[DialogList]\n0=EVA_TEST\n\
        [EVA_TEST]\nAllied=STOCK_A\nRussian=STOCK_R\nYuri=STOCK_Y\nVoiceA=AbCdEf123456789XYZTAIL\nVoiceB=EXTRA_B\n";
    let (_directory, mut process) =
        fixture("sdk-side-vox", ROOT, Some(&wasm), &[("evamd.ini", eva)]);
    let (rules, mut sim) = load(&mut process, "");
    let request = crate::app::frontend::cncnet_spawn::SpawnRequest::parse(
        "[Settings]\nScenario=fixture.map\nSide=3\nColor=0\nSpecialHouseIsAlly=No\nAIPlayers=1\nSeed=77\n\
         [HouseHandicaps]\nMulti2=1\n[HouseCountries]\nMulti2=0\n[HouseColors]\nMulti2=1\n[SpawnLocations]\nMulti1=0\nMulti2=1\n").unwrap();
    let crate::app::frontend::cncnet_spawn::SpawnGame::Skirmish(launch) = request.game else {
        panic!("skirmish")
    };
    let descriptor =
        crate::sim::scenario_bootstrap::MatchLaunchDescriptor::from_resolved(*launch).unwrap();
    crate::sim::scenario_bootstrap::initialize_skirmish_launch_houses(
        &mut sim,
        &crate::map::houses::HouseRoster::default(),
        &rules,
        &descriptor,
    );
    assert_eq!(sim.session().scenario_side(), 3);
    let registry = process.audio_catalog().unwrap().eva();
    let entry = registry.entry("EVA_TEST").unwrap();
    let mut queue = VoxQueue::new();
    queue.set_side(sim.session().scenario_side(), &rules);
    let voice_b = eva_control("filename", "voice_index", 4);
    assert_eq!(queue.selected_voice(), 4);
    let line = || VoxRequest {
        event: &entry.name,
        eva_type: entry.eva_type,
        priority: entry.priority,
    };
    assert!(queue.queue_voice(line(), None, None).inserted);
    let extra = queue.take_next(1, false).unwrap();
    assert_eq!(
        queue.sample_for_test(&extra, registry),
        voice_b["filename"].as_str()
    );
    queue.started(extra);
    assert!(queue.reset_all());
    assert_eq!(
        queue.selected_voice(),
        4,
        "ResetAll preserves the selected process voice"
    );
    queue.set_paused(true);
    assert!(queue.queue_voice(line(), None, None).inserted);
    assert!(queue.take_next(1, false).is_none());
    // The node owns the event, not a filename. A later accepted SetSide is
    // observed by the same dequeue/sample owner, without rebuilding EVAMD.
    queue.set_side(2, &rules);
    queue.set_paused(false);
    let node = queue.take_next(1, false).unwrap();
    assert_eq!(
        queue.sample_for_test(&node, registry),
        eva_control("set_side", "side_index", 2)["filename"].as_str()
    );
    queue.started(node.clone());
    assert!(queue.reset_all());
    queue.set_side(4, &rules);
    assert_eq!(
        queue.selected_voice(),
        eva_control("set_side", "side_index", 4)["bound"]
            .as_i64()
            .unwrap() as i32
    );
    queue.set_side(-1, &rules);
    assert_eq!(
        queue.selected_voice(),
        eva_control("set_side", "side_index", -1)["bound"]
            .as_i64()
            .unwrap() as i32
    );
    assert!(queue.sample_for_test(&node, registry).is_none());

    let off = format!("{ROOT}[VeraFeatures]\nAresFaction=no\n");
    let (_off_directory, mut off_process) = fixture(
        "sdk-side-vox-stock",
        &off,
        Some(&wasm),
        &[("evamd.ini", eva)],
    );
    let (off_rules, _) = load(&mut off_process, "");
    queue.set_side(-1, &off_rules);
    assert_eq!(
        queue.selected_voice(),
        0,
        "explicit stock keeps SetSide7534E0's -1 -> Allied branch"
    );
}

#[test]
#[ignore = "requires the real .18 component in VERA_SDK_SIDE_PRESENTATION_WASM"]
fn sdk_side_score_model_keeps_one_live_snapshot_through_type_reset_and_theme_playback() {
    use crate::audio::theme::{MusicOutputState, ThemeGates, ThemeRuntime};
    let wasm = component("VERA_SDK_SIDE_PRESENTATION_WASM");
    let themes = b"[Themes]\n0=LOADING\n1=SIDE_SCORE\n2=COUNTRY_SCORE\n3=MISSION_SCORE\n\
        [LOADING]\nSound=LOADING\n[SIDE_SCORE]\nSound=SIDE_SCORE\n[COUNTRY_SCORE]\nSound=COUNTRY_SCORE\n[MISSION_SCORE]\nSound=MISSION_SCORE\n";
    let (_directory, mut process) = fixture(
        "sdk-side-score",
        ROOT,
        Some(&wasm),
        &[("thememd.ini", themes)],
    );
    let (rules, _) = load(&mut process, "");
    assert!(rules.registered_side_ui_enabled());
    let presentation = process.side_presentation(3).unwrap().unwrap();
    let model = crate::app::match_runtime::sim_tick::build_campaign_score_screen_model_from_sources(
        0,
        1,
        &Default::default(),
        [(true, 7)],
        3,
        Some(presentation),
        str::to_owned,
    );
    assert_eq!(model.theme.as_deref(), Some("COUNTRY_SCORE"));
    let before = model.clone();
    let (_new_rules, _) = load(
        &mut process,
        "[FourthSynthetic]\nCampaignScore.Background=late.shp\nCampaignScore.UnderParTheme=MISSION_SCORE\n",
    );
    assert_eq!(
        process
            .side_presentation(3)
            .unwrap()
            .unwrap()
            .campaign_score_background
            .as_deref(),
        Some("late.shp")
    );
    assert_eq!(
        model, before,
        "the accepted page owns its Side snapshot across the next real TypeReset"
    );
    let frozen = model.side_presentation.as_ref().unwrap();
    assert_eq!(frozen.score_art().campaign_background, Some("early.shp"));
    let mut theme = ThemeRuntime::default();
    theme.play_track(
        model.theme.as_deref().unwrap_or("SCORE"),
        process.manager().unwrap(),
        ThemeGates {
            launcher_audio_available: true,
            startup_suppressed: false,
        },
        MusicOutputState::Unavailable,
    );
    let history =
        crate::test_fixture::json("tools/rules_oracle/ares_loading_themes.json")["histories"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == "skirmishSideCountry")
            .unwrap();
    let expected = history["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["label"] == "country_overrides_side")
        .unwrap()["play_requests"][0]
        .as_i64()
        .unwrap() as i32;
    assert_eq!(theme.slots().retained, expected);
}
