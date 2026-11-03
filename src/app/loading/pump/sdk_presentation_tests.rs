//! Actual component presentation through process sources, menu snapshots and
//! Selected -> Prepared loading. Frame rendering and audio devices are separate.
#![cfg(not(feature = "ares"))]
use super::*;
use crate::app::process_assets::ProcessAssets;
use crate::assets::asset_manager::{
    MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::rules::{
    ini_parser::IniFile,
    ruleset::{CountryIdx, RuleSet},
};
use crate::ui::skirmish_shell::{
    SkirmishComboId, SkirmishComboItem, SkirmishCountryChoice, SkirmishShellState, combo_items,
};
use serde_json::Value;

fn component(variable: &str) -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}

fn theme_corpus() -> &'static Value {
    crate::test_fixture::json("tools/rules_oracle/ares_loading_themes.json")
}

fn expected_theme(history: &str, label: &str) -> i32 {
    let history = theme_corpus()["histories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == history)
        .unwrap();
    history["stages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["label"] == label)
        .unwrap()["play_requests"][0]
        .as_i64()
        .unwrap() as i32
}

fn fixture(
    label: &str,
    root: &str,
    wasm: Option<&[u8]>,
    files: &[(&str, &[u8])],
) -> (TestDirectory, ProcessAssets, RuleSet) {
    let directory = audio_test_install(label);
    for (name, bytes) in [
        ("rulesmd.ini", root.as_bytes()),
        ("artmd.ini", b"[Movies]\n".as_slice()),
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
                "api_version": vera_mod_api::API_VERSION, "program": "gameplay.wasm",
                "sha256": crate::util::sha256::sha256_hex(wasm),
            }))
            .unwrap(),
        )
        .unwrap();
    }
    // Reuse the original ordered Theme catalog from the executed controls.
    let keys = theme_corpus()["histories"][0]["theme_keys"]
        .as_array()
        .unwrap();
    let mut themes = String::from("[Themes]\n");
    for (index, key) in keys.iter().enumerate() {
        themes += &format!("{index}={}\n", key.as_str().unwrap());
    }
    for key in keys {
        let key = key.as_str().unwrap();
        themes += &format!("[{key}]\nSound={key}\nRepeat=yes\n");
    }
    std::fs::write(directory.path().join("thememd.ini"), themes).unwrap();
    // Match country_loading_tests'32x32 interior and authored starts. The
    // formal source parser must see both consecutive slots before the two
    // pre-Fill Gather callbacks, rather than enter deficient-start search.
    let map = b"[Map]\nTheater=TEMPERATE\nSize=0,0,36,44\nLocalSize=2,5,32,32\n[Basic]\nNewINIFormat=4\nLoadingTheme=MISSION_SCORE\n[Waypoints]\n0=24020\n1=46042\n[IsoMapPack5]\n1=DwALABwBAAIA/////wAAABEAAA==\n";
    for name in ["fixture.map", "ALL01UMD.MAP"] {
        std::fs::write(directory.path().join(name), map).unwrap();
    }
    // AssetManager catalogs loose names when opened, before payloads are read.
    for (name, bytes) in files {
        std::fs::write(directory.path().join(name), bytes).unwrap();
    }
    let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
    let manager = process.reconstruct_manager(directory.path()).unwrap();
    let (rules, _) = process.initialize_sources_if_needed(&manager).unwrap();
    process.return_from_loading(manager);
    (directory, process, rules.unwrap())
}

fn skirmish_request(country: u32) -> LoadingRequest {
    let mut launch = tests::test_launch_session(LaunchCountry::from_country_index(country));
    launch.selected_map_file = Some("fixture.map".into());
    LoadingRequest::unverified_legacy_skirmish(launch, tests::unverified_seed(77))
}

fn prepare(
    directory: &TestDirectory,
    process: &mut ProcessAssets,
    audio: &mut crate::app::audio_runtime::AppAudioRuntime,
    request: LoadingRequest,
) -> anyhow::Result<LoadingSession> {
    let mut session = LoadingSession::from_request(request);
    session.job.ra2_dir = Some(directory.path().to_owned());
    let mut slot = None;
    replace_loading_attempt(
        &mut slot,
        &mut Default::default(),
        process,
        audio,
        session,
        100,
    );
    prepare_loading_session(
        process,
        audio,
        &mut Default::default(),
        None,
        slot.take().unwrap(),
        true,
        false,
        Some(directory.path().to_owned()),
        None,
    )
}

#[test]
#[ignore = "requires VERA_SDK_PRESENTATION_WASM and .16 VERA_SDK_PRESENTATION_PREVIOUS_WASM"]
fn sdk_presentation_live_menu_keeps_constructor_parent_and_duplicate_ordinals() {
    let wasm = component("VERA_SDK_PRESENTATION_WASM");
    let previous = component("VERA_SDK_PRESENTATION_PREVIOUS_WASM");
    // Find compares the full input, while410800 stores only24 bytes. The two
    // equal stored IDs visit their parent before and after its first body.
    let root = "[Countries]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n1=PARENT\n2=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
        [Sides]\nGDI=PARENT\n[ABCDEFGHIJKLMNOPQRSTUVWX]\nSide=GDI\nParentCountry=PARENT\nMultiplay=yes\nUIName=Name:Duplicate\n\
        [PARENT]\nSide=GDI\nMultiplay=yes\nUIName=Name:Parent\nFile.Flag=parent.pcx\nMenuText.Status=STT:Parent\nListIndex=3\nRandomSelectionWeight=7\n";
    for (label, module, root, extended) in [
        ("menu-no-module", None, root.to_owned(), false),
        (
            "menu-previous16",
            Some(previous.as_slice()),
            root.to_owned(),
            true,
        ),
        (
            "menu-component17",
            Some(wasm.as_slice()),
            root.to_owned(),
            true,
        ),
        (
            "menu-explicit-stock",
            Some(wasm.as_slice()),
            format!("{root}[VeraFeatures]\nAresFaction=no\n"),
            false,
        ),
    ] {
        let (_directory, mut process, rules) = fixture(label, &root, module, &[]);
        process.read_multiplayer_country(&IniFile::empty(), 0);
        let ctor = process
            .native_rules()
            .unwrap()
            .country_loading(CountryIdx(0))
            .unwrap()
            .unwrap();
        // The constructor has not visited AresFaction's physical reader yet.
        // Even an explicit-off install initially exposes the guest defaults.
        assert_eq!(ctor.extension.is_some(), module.is_some());
        if module.is_some() {
            assert_eq!(
                process
                    .native_rules()
                    .unwrap()
                    .country_flag_surface(0)
                    .unwrap()
                    .as_deref(),
                Some("rani.pcx")
            );
        }
        process.refresh_multiplayer_country_bodies().unwrap();
        let rows = process.country_menu_rows().unwrap();
        let mut shell = SkirmishShellState::default();
        shell.refresh_country_catalog(&rules);
        if let Some(rows) = rows.as_ref() {
            shell.refresh_live_country_catalog(rows);
        }
        let choice =
            |ordinal| SkirmishCountryChoice::Country(LaunchCountry::from_country_index(ordinal));
        if extended {
            let rows = rows.unwrap();
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].id, rows[2].id);
            assert_eq!(rows[0].presentation.as_ref().unwrap().random_weight, 1);
            assert_eq!(rows[2].presentation.as_ref().unwrap().random_weight, 7);
            assert_eq!(
                shell
                    .country_catalog
                    .flag_for_choice(false, LaunchCountry::from_country_index(0)),
                Some("rani.pcx")
            );
            assert_eq!(
                shell
                    .country_catalog
                    .flag_for_choice(false, LaunchCountry::from_country_index(2)),
                Some("parent.pcx")
            );
            assert_eq!(
                shell.country_catalog.status_key_for_choice(choice(2)),
                Some("STT:Parent")
            );
            assert_eq!(
                combo_items(&shell, &[], SkirmishComboId::Side(0)),
                [
                    SkirmishCountryChoice::Random,
                    choice(1),
                    choice(2),
                    choice(0)
                ]
                .map(SkirmishComboItem::Country)
            );
        } else {
            assert!(rows.is_none_or(|rows| rows.iter().all(|row| row.presentation.is_none())));
            assert!(
                process
                    .native_rules()
                    .unwrap()
                    .country_loading(CountryIdx(0))
                    .unwrap()
                    .unwrap()
                    .extension
                    .is_none()
            );
            assert_eq!(
                process
                    .native_rules()
                    .unwrap()
                    .country_flag_surface(0)
                    .unwrap(),
                None
            );
        }
    }
}

const LOADING_ROOT: &str = "[Countries]\n0=PARENT\n1=CHILD\n[Sides]\nSIDE=PARENT,CHILD\n\
    [Colors]\nFirst=17,128,240\n[PARENT]\nSide=SIDE\nFile.LoadScreen=parent.shp\nFile.LoadScreenPAL=parent.pal\nLoadScreenText.Name=Name:Parent\nLoadScreenText.Color=First\n\
    [CHILD]\nSide=SIDE\nParentCountry=PARENT\nFile.LoadScreen=child.shp\nFile.LoadScreenPAL=child.pal\nLoadScreenText.Name=Name:Child\nLoadScreenText.Color=First\nLoadingTheme=COUNTRY_SCORE\n\
    [SIDE]\nLoadingTheme=SIDE_SCORE\n";

#[test]
#[ignore = "requires the real .17 component in VERA_SDK_PRESENTATION_WASM"]
fn sdk_presentation_prepared_loading_freezes_early_art_and_uses_physical_theme_sources() {
    let wasm = component("VERA_SDK_PRESENTATION_WASM");
    for absent in [false, true] {
        let palettes: &[(&str, &[u8])] = if absent {
            &[("child.pal", &[20; 768])]
        } else {
            &[("parent.pal", &[12; 768]), ("child.pal", &[20; 768])]
        };
        let (directory, mut process, _) = fixture(
            "sdk-early-late-loading",
            LOADING_ROOT,
            Some(&wasm),
            palettes,
        );
        let request = skirmish_request(1);
        crate::app::frontend::skirmish_session::prepare_external_skirmish_countries(
            &mut process,
            request.startup().launch_session().unwrap(),
        )
        .unwrap();
        let mut audio = tests::test_audio();
        let session = prepare(&directory, &mut process, &mut audio, request).unwrap();
        assert!(matches!(session.stage, LoadingStage::Prepared(_)));
        let native = session.native.as_ref().unwrap();
        let early = native.early_country_loading.as_ref().unwrap();
        assert_eq!(early.palette_name, "parent.pal");
        assert_eq!(early.palette.is_none(), absent);
        if let Some(palette) = &early.palette {
            assert_eq!(palette.color(1), Color::rgb(48, 48, 48));
        }
        assert_eq!(
            process
                .native_rules()
                .unwrap()
                .country_loading_text_hsv(early.country)
                .unwrap(),
            Some([17, 128, 240])
        );
        assert_eq!(native.text_rgb, early.text_rgb.unwrap());
        let late = native.late_country_loading.as_ref().unwrap();
        assert_eq!(late.files(), ("child.shp", "child.pal"));
        assert_eq!(late.background_file(false).as_deref(), Some("child.shp"));
        let csf = crate::assets::csf_file::test_csf(&[
            ("Name:Parent", "Parent"),
            ("Name:Child", "Child"),
        ]);
        let LoadingStage::Prepared(prepared) = &session.stage else {
            unreachable!()
        };
        let mut composition = build_loading_composition(
            prepared.initial.map_data(),
            prepared.request.startup().launch_session().unwrap(),
            Some(&csf),
            [800, 600],
            ShellScale::RETAIL,
            &[],
        );
        composition.bind_country_loading(late, Some(&csf));
        assert_eq!(composition.text.country_name.as_deref(), Some("Child"));
        assert_eq!(
            audio.theme.slots().retained,
            expected_theme("skirmishSideCountry", "country_overrides_side")
        );
        session.retire(&mut process);
        assert!(process.is_available() && !process.is_leased());
        assert_eq!(
            process
                .native_rules()
                .unwrap()
                .country_loading(CountryIdx(1))
                .unwrap()
                .unwrap()
                .extension
                .unwrap()
                .files()
                .0,
            "child.shp",
            "retirement cancels the prefix ticket without rewinding E_multi"
        );

        let session = prepare(
            &directory,
            &mut process,
            &mut audio,
            LoadingRequest::campaign(tests::prepared_campaign()),
        )
        .unwrap();
        assert_eq!(
            audio.theme.slots().retained,
            expected_theme("campaignBasic", "named")
        );
        // Prepared visits reuse the same Theme request, rather than restarting it.
        audio.play_theme(
            "COUNTRY_SCORE",
            session.job.asset_manager.as_ref().unwrap(),
            600,
        );
        let retained = audio.theme.slots().retained;
        let session = prepare_loading_session(
            &mut process,
            &mut audio,
            &mut Default::default(),
            None,
            session,
            false,
            false,
            Some(directory.path().to_owned()),
            None,
        )
        .unwrap();
        assert_eq!(audio.theme.slots().retained, retained);
        session.retire(&mut process);
        assert!(process.is_available() && !process.is_leased());
    }
}

#[test]
fn sdk_presentation_theme_only_component_and_malformed_output_use_formal_failure_retirement() {
    use vera_mod_api::{
        API_VERSION, Context, HookKind, LoadingThemePolicy, LoadingThemeResourceContext,
        ModuleManifest, Plan, Request,
    };
    assert!(!crate::rules::ARES_ENABLED);
    let request = Request {
        api_version: API_VERSION,
        context: Context::LoadingThemeResource(LoadingThemeResourceContext::Campaign),
        config: vec![],
        state: vec![],
    };
    let unchanged = serde_json::to_vec(&Plan::unchanged(&request)).unwrap();
    let selected = serde_json::to_vec(&Plan {
        loading_theme: Some(LoadingThemePolicy::Names {
            values: vec![Some("MISSION_SCORE".into())],
        }),
        ..Plan::unchanged(&request)
    })
    .unwrap();
    let fallback = serde_json::to_vec(&Plan {
        loading_theme: Some(LoadingThemePolicy::Names { values: vec![None] }),
        ..Plan::unchanged(&request)
    })
    .unwrap();
    let manifest = ModuleManifest {
        api_version: API_VERSION,
        id: "test.theme-only".into(),
        version: "1".into(),
        config_version: 1,
        state_version: 1,
        hooks: vec![
            HookKind::Configure,
            HookKind::Attach,
            HookKind::Damage,
            HookKind::Tick,
            HookKind::ConfigureBoundary,
            HookKind::LoadingThemeResource,
        ],
    };
    assert!(!manifest.hooks.contains(&HookKind::ConfigureCountryRead));
    for (response, expected) in [
        (
            selected.as_slice(),
            Some(expected_theme("campaignBasic", "named")),
        ),
        (
            fallback.as_slice(),
            Some(expected_theme("campaignBasic", "missing")),
        ),
        (b"{".as_slice(), None),
    ] {
        let component = vera_mod_runtime::test_support::component_fixture_by_kind(
            &manifest,
            &unchanged,
            &[("loading_theme_resource", response)],
        );
        let (directory, mut process, _) =
            fixture("sdk-independent-theme", LOADING_ROOT, Some(&component), &[]);
        assert!(process.defers_loading_theme());
        assert!(process.country_menu_rows().unwrap().is_none());
        let mut audio = tests::test_audio();
        let result = prepare(
            &directory,
            &mut process,
            &mut audio,
            LoadingRequest::campaign(tests::prepared_campaign()),
        );
        if expected.is_none() {
            assert!(
                result.is_err(),
                "malformed resource output must not silently choose stock"
            );
            assert!(process.is_available() && !process.is_leased());
            assert!(
                process.has_native_rules(),
                "query failure retains the live process owner"
            );
        } else {
            let session = result.unwrap();
            assert_eq!(audio.theme.slots().retained, expected.unwrap());
            session.retire(&mut process);
            assert!(process.is_available() && !process.is_leased());
        }
    }
}
