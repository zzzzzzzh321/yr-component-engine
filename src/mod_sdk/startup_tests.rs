//! Real components at the original startup archive frontier, then the existing
//! INI, map, sound/PCM, string-table and exact saved-configuration owners.
use super::*;
use crate::assets::asset_manager::{
    AssetManager, MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::assets::audio_bag::{decode_bag_audio, test_support::build_idx};
use crate::assets::csf_file::{
    test_support::{build_test_csf, supplemental_fixture},
    type_ui_name,
};
use crate::assets::startup_resources::StartupComponentSource;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerStack};
use crate::rules::retail_sources::select_ini;
use crate::rules::ruleset::RuleSet;
use crate::rules::sound_ini::SoundRegistry;
use crate::sim::snapshot::GameSnapshot;
use crate::sim::world::Simulation;
use crate::util::sha256::sha256_hex;
use serde_json::Value;

fn component(variable: &str) -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}

fn write(dir: &TestDirectory, name: &str, bytes: &[u8]) {
    std::fs::write(dir.path().join(name), bytes).unwrap();
}

fn declaration(dir: &TestDirectory, wasm: &[u8]) {
    write(
        dir,
        "vera-mod.json",
        &serde_json::to_vec(&serde_json::json!({
            "api_version": vera_mod_api::API_VERSION,
            "program": "gameplay.wasm",
            "sha256": sha256_hex(wasm),
        }))
        .unwrap(),
    );
}

fn early_component(dir: &TestDirectory, wasm: &[u8]) {
    declaration(dir, wasm);
    write(dir, "gameplay.wasm", wasm);
}

fn archive(dir: &TestDirectory, name: &str, files: &[(String, Vec<u8>)]) {
    let entries: Vec<_> = files
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    write(
        dir,
        name,
        &crate::map::source::test_support::make_new_format_mix_bytes(&entries),
    );
}

fn pair(files: &mut Vec<(String, Vec<u8>)>, stem: &str, sample: &str, value: i16, rate: u32) {
    files.push((format!("{stem}.idx"), build_idx(&[(sample, 2, 4, rate, 4)])));
    let mut bag = vec![0xaa, 0xbb];
    bag.extend(value.to_le_bytes());
    bag.extend((-value).to_le_bytes());
    files.push((format!("{stem}.bag"), bag));
}

fn open(dir: &TestDirectory) -> AssetManager {
    open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap()
}

fn configured(assets: &AssetManager) -> (ProcessedRulesLayers, RuleSet) {
    let selected = select_ini(assets, "rulesmd.ini").unwrap();
    let processed = RulesLayerStack::new(selected.ini).process().unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    rules
        .bind_external_mod_for_process(assets, &processed)
        .unwrap();
    (processed, rules)
}

fn long_list_owner(assets: &AssetManager) -> crate::rules::process_owner::NativeRulesProcessOwner {
    let audio = crate::rules::audio_sources::AudioDefinitions::select(assets);
    crate::rules::retail_sources::RetailRulesSources::select(assets)
        .unwrap()
        .into_owner(audio.sounds().clone(), audio.eva().clone())
        .unwrap()
}

fn reload(rules: &RuleSet) -> i32 {
    rules
        .object("TANK")
        .unwrap()
        .type_resource_policy()
        .unwrap()
        .reload
        .unwrap()
        .normal_amount
}

#[test]
#[ignore = "requires VERA_SDK_LONG_LIST_WASM and .18 VERA_SDK_LONG_LIST_PREVIOUS_WASM"]
fn sdk_long_list_cold_mode_map_and_explicit_retail_preserve_native_reader_boundaries() {
    use crate::rules::prerequisite::{Prerequisite, PrerequisiteGroup};
    use crate::rules::process_owner::NativeScenarioRulesPrefix;
    use vera_mod_api::{Context, HookKind, LongListReadersPolicy, ModuleManifest, Plan, Request};
    let wasm = component("VERA_SDK_LONG_LIST_WASM");
    let previous = component("VERA_SDK_LONG_LIST_PREVIOUS_WASM");
    let request = Request {
        api_version: 1,
        context: Context::LongListReaders,
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut selected = unchanged.clone();
    selected.long_list_readers = Some(LongListReadersPolicy::Retail128);
    let retail = vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: 1,
            id: "long-list-retail".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::LongListReaders,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[("long_list_readers", &serde_json::to_vec(&selected).unwrap())],
    );
    let tail = format!("{},TAIL", "X".repeat(126));
    let plain = format!("{},PLAIN", "X".repeat(126));
    let root = format!(
        "[VeraFeatures]\nAresProduction=no\n[General]\nPrerequisitePower={tail}\n\
        [BuildingTypes]\n0=TAIL\n1=PLAIN\n[VehicleTypes]\n0=TANK\n\
        [TAIL]\nStrength=100\n[PLAIN]\nStrength=100\n\
        [TANK]\nStrength=100\nPrerequisite={tail}\nPrerequisiteOverride={tail}\n"
    );
    let mode = IniFile::from_str(&format!(
        "[General]\nPrerequisitePower={plain}\n[TANK]\nPrerequisite={plain}\n"
    ));
    let map = IniFile::from_str(&format!(
        "[General]\nPrerequisitePower={tail}\n[TANK]\nPrerequisiteOverride={tail}\n"
    ));
    for (label, module, extended) in [
        ("long-new", Some(wasm.as_slice()), true),
        ("long-none", None, false),
        ("long-previous", Some(previous.as_slice()), false),
        ("long-retail", Some(retail.as_slice()), false),
    ] {
        let dir = audio_test_install(label);
        if let Some(module) = module {
            early_component(&dir, module);
        }
        write(&dir, "rulesmd.ini", root.as_bytes());
        write(&dir, "artmd.ini", b"");
        let assets = open(&dir);
        let mut owner = long_list_owner(&assets);
        let cold =
            RuleSet::from_processed_rules(&owner.startup_compatibility_projection().unwrap())
                .unwrap();
        let expected = if extended {
            vec![Prerequisite::Building(0)]
        } else {
            vec![]
        };
        assert_eq!(cold.object("TANK").unwrap().prerequisite, expected);
        assert_eq!(cold.object("TANK").unwrap().prerequisite_override, expected);
        let (rules, ..) = owner
            .load_scenario(NativeScenarioRulesPrefix::NonCampaign(Some(&mode)), &map)
            .unwrap()
            .into_parts();
        assert_eq!(
            rules.object("TANK").unwrap().prerequisite,
            if extended {
                vec![Prerequisite::Building(1)]
            } else {
                vec![]
            }
        );
        assert_eq!(
            rules.object("TANK").unwrap().prerequisite_override,
            expected
        );
        assert!(
            rules.prerequisite_list(PrerequisiteGroup::Power).is_empty(),
            "late General reads stay128 even after TAIL and PLAIN are registered"
        );
    }
    let late = audio_test_install("long-only-late");
    declaration(&late, &retail);
    archive(&late, "conqmd.mix", &[("gameplay.wasm".into(), retail)]);
    let error = open_active_assets(late.path(), MediaArchiveMode::STOCK_DIGITAL)
        .err()
        .unwrap();
    assert!(error.to_string().contains("before resource selection"));
}

#[test]
#[ignore = "requires VERA_SDK_LONG_LIST_WASM and .18 VERA_SDK_LONG_LIST_PREVIOUS_WASM"]
fn sdk_long_list_allies_and_saved_native_envelope_keep_the_admitted_selection() {
    use crate::rules::prerequisite::Prerequisite;
    use crate::rules::process_owner::NativeScenarioRulesPrefix;
    use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};
    let wasm = component("VERA_SDK_LONG_LIST_WASM");
    let dir = audio_test_install("long-saved-allies");
    early_component(&dir, &wasm);
    let root = format!(
        "[VeraFeatures]\nAresProduction=no\n[Countries]\n0=Americans\n\
        [Americans]\nParentCountry=Americans\n[Sides]\nGDI=Americans\n\
        [BuildingTypes]\n0=TAIL\n[VehicleTypes]\n0=TANK\n[TAIL]\nStrength=100\n\
        [TANK]\nStrength=100\nPrerequisite={},TAIL\n",
        "X".repeat(126)
    );
    write(&dir, "rulesmd.ini", root.as_bytes());
    write(&dir, "artmd.ini", b"");
    let assets = open(&dir);
    let mut owner = long_list_owner(&assets);
    let map = IniFile::from_str(&format!(
        "[Houses]\n0=A\n1=B\n[A]\nCountry=Americans\nAllies={},B\n[B]\nCountry=Americans\n",
        "X".repeat(126)
    ));
    let (mut rules, ..) = owner
        .load_scenario(NativeScenarioRulesPrefix::NonCampaign(None), &map)
        .unwrap()
        .into_parts();
    let mut sim = Simulation::with_seed(53);
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
    let roster = crate::map::houses::parse_house_roster(&map, &[], Some(&rules));
    crate::sim::scenario_bootstrap::initialize_map_roster_houses(
        &mut sim,
        &roster,
        &rules,
        Some(&map),
    );
    sim.install_house_alliances(
        crate::sim::house_diplomacy::HouseAllianceAdmission::ScenarioInitialization {
            roster: &roster,
        },
        &rules,
    );
    assert!(sim.house_alliances()["A"].contains("B"));
    assert!(!sim.house_alliances()["B"].contains("A"));
    let saved = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "LONG-LIST", 0))
        .unwrap()
        .sim;
    let retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(owner.saved_scenario_rules().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(retained.long_list_readers(), Some(true));
    write(
        &dir,
        "rulesmd.ini",
        b"[VeraFeatures]\nAresProduction=no\n[VehicleTypes]\n0=TANK\n[TANK]\nStrength=100\n",
    );
    let current = open(&dir);
    let current_owner = long_list_owner(&current);
    let mut rebound = current_owner
        .project_saved_scenario_rules(&retained)
        .unwrap();
    adopt_saved_program(&mut rebound, &saved, &current).unwrap();
    assert_eq!(rebound.long_list_read_capacity(), 2048);
    assert_eq!(
        rebound.object("TANK").unwrap().prerequisite,
        [Prerequisite::Building(0)]
    );
    assert_eq!(saved.house_alliances(), sim.house_alliances());
    assert_eq!(
        rebound.simulation_config_hash(),
        rules.simulation_config_hash()
    );
    assert_eq!(
        crate::util::save_codec::serialize(rebound.external_mod.as_deref().unwrap()).unwrap(),
        crate::util::save_codec::serialize(saved.external_mod.as_ref().unwrap()).unwrap(),
    );
    early_component(&dir, &component("VERA_SDK_LONG_LIST_PREVIOUS_WASM"));
    assert!(adopt_saved_program(&mut rebound, &saved, &open(&dir)).is_err());
}

#[test]
#[ignore = "requires VERA_SDK_STARTUP_WASM and the previous component in VERA_SDK_PREVIOUS_WASM"]
fn sdk_startup_sources_audio_consumers_and_saved_adoption_share_one_component_identity() {
    let wasm = component("VERA_SDK_STARTUP_WASM");
    let dir = audio_test_install("sdk-startup-resources");
    early_component(&dir, &wasm);
    let root = b"[#include]\n0=child.ini\n1=CHILD.INI\n2=absent.ini\n[VehicleTypes]\n0=TANK\n[TANK]\nStrength=100\nAmmo=6\n";
    write(&dir, "rulesmd.ini", root);
    write(&dir, "child.ini", b"[TANK]\nReloadAmount=5\n");
    let map = b"[#include]\n0=map-extra.ini\n[Basic]\nName=Physical Map\n";
    write(&dir, "Arena.map", map);
    write(&dir, "map-extra.ini", b"[TANK]\nStrength=250\n");
    write(&dir, "soundmd.ini", b"[SoundList]\n0=VoiceCue\n1=BuildCue\n2=LateCue\n[VoiceCue]\nSounds=$VoIcE\n[BuildCue]\nSounds=#place\n[LateCue]\nSounds=late\n");
    let mut early = Vec::new();
    pair(&mut early, "audio", "voice", 1000, 22050);
    pair(&mut early, "ares", "voice", 2000, 11025);
    pair(&mut early, "audio02", "place", 3000, 22050);
    pair(&mut early, "audio99", "voice", 4000, 44100);
    archive(&dir, "expandmd20.mix", &early);
    let mut late = Vec::new();
    pair(&mut late, "audio98", "late", 9000, 8000);
    archive(&dir, "conqmd.mix", &late);
    let assets = open(&dir);
    assert_eq!(assets.hash_include_override(), Some(true));
    let selected = select_ini(&assets, "rulesmd.ini").unwrap();
    assert_eq!(selected.source.source_sha256, sha256_hex(root));
    assert_eq!(selected.source.payload_len, root.len());
    assert_eq!(
        selected.physical_ini.content_hash(),
        IniFile::from_bytes(root).unwrap().content_hash()
    );
    assert_eq!(
        selected.physical_ini.multiplayer_digest(),
        IniFile::from_bytes(root).unwrap().multiplayer_digest()
    );
    assert_eq!(
        selected.included_sources.len(),
        1,
        "hash include is once per physical root; missing children are skipped"
    );
    assert_eq!(
        selected
            .ini
            .section_or_empty("TANK")
            .read_int("ReloadAmount", 0),
        5
    );
    let decoded = crate::map::source::load_map_ini_by_name_or_path_with_assets(
        dir.path(),
        "Arena.map",
        &assets,
    )
    .unwrap();
    assert_eq!(
        decoded.source.physical_identity(),
        Some((map.len(), sha256_hex(map).as_str()))
    );
    assert_eq!(
        decoded.ini.section_or_empty("TANK").read_int("Strength", 0),
        250
    );

    assert!(
        assets.resolve_ref("audio98.idx").is_some(),
        "the later MIX is eventually mounted"
    );
    let audio = assets.load_audio_index().unwrap().unwrap();
    assert_eq!(audio.additional_sources.len(), 3);
    assert_eq!(
        audio.additional_sources.last().unwrap().idx.entry_id,
        crate::assets::mix_hash::mix_hash("audio99.idx")
    );
    let sounds = SoundRegistry::from_ini_admitting(
        &select_ini(&assets, "soundmd.ini").unwrap().ini,
        Some(&audio.index),
    );
    assert!(sounds.get("VoiceCue").unwrap().samples_resolved);
    assert!(sounds.get("BuildCue").unwrap().samples_resolved);
    assert!(
        !sounds.get("LateCue").unwrap().samples_resolved,
        "late MIX samples cannot repair the early catalog"
    );
    let (entry, bytes) = audio
        .index
        .get(&sounds.get("VoiceCue").unwrap().sounds[0])
        .unwrap();
    assert_eq!(
        (entry.offset, entry.sample_rate),
        (2, 44100),
        "the complete later entry and its BAG win together"
    );
    assert_eq!(
        decode_bag_audio(entry, bytes).unwrap().samples_i16,
        [4000, -4000]
    );
    let (entry, bytes) = audio
        .index
        .get(&sounds.get("BuildCue").unwrap().sounds[0])
        .unwrap();
    assert_eq!(
        decode_bag_audio(entry, bytes).unwrap().samples_i16,
        [3000, -3000]
    );
    assert!(audio.index.get("late").is_none());

    let (processed, rules) = configured(&assets);
    assert_eq!(
        processed.sdk_passes().len(),
        2,
        "includes do not invent additional Rules Process passes"
    );
    assert_eq!(reload(&rules), 5);
    let mut sim = Simulation::with_seed(19);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    let saved = GameSnapshot::load(&GameSnapshot::save(&sim, 0, 0, "SDK-STARTUP", 0))
        .unwrap()
        .sim;
    let retained: ProcessedRulesLayers = crate::util::save_codec::deserialize(
        &crate::util::save_codec::serialize(&processed).unwrap(),
    )
    .unwrap();
    write(&dir, "child.ini", b"[TANK]\nReloadAmount=9\n");
    let current = open(&dir);
    assert_eq!(
        reload(&configured(&current).1),
        9,
        "a fresh configuration really sees the changed input"
    );
    let mut rebound = RuleSet::from_processed_rules(&retained).unwrap();
    adopt_saved_program(&mut rebound, &saved, &current).unwrap();
    assert_eq!(
        reload(&rebound),
        5,
        "saved adoption must not run Configure against current INI"
    );
    assert_eq!(
        crate::util::save_codec::serialize(rebound.external_mod.as_deref().unwrap()).unwrap(),
        crate::util::save_codec::serialize(saved.external_mod.as_ref().unwrap()).unwrap()
    );
    let previous = component("VERA_SDK_PREVIOUS_WASM");
    early_component(&dir, &previous);
    let changed = open(&dir);
    assert!(
        adopt_saved_program(&mut rebound, &saved, &changed).is_err(),
        "saved configuration cannot adopt another component digest"
    );
}

fn original_pairs(value: &Value) -> Vec<(&str, &str)> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| (entry[0].as_str().unwrap(), entry[1].as_str().unwrap()))
        .collect()
}

#[test]
#[ignore = "requires the real startup component in VERA_SDK_STARTUP_WASM"]
fn sdk_startup_csf_language_name_lookup_duplicate_order_and_bad_suffix_use_native_histories() {
    let wasm = component("VERA_SDK_STARTUP_WASM");
    let ordered = crate::test_fixture::json("tools/rules_oracle/ares_csf_ordered.json");
    let row = &ordered["controls"][0];
    let dir = audio_test_install("sdk-startup-csf-order");
    early_component(&dir, &wasm);
    let base = original_pairs(&row["base"]);
    write(
        &dir,
        "ra2md.csf",
        &supplemental_fixture(&base, 0, 3, base.len() as u32),
    );
    // Same-value replacements leave the executed duplicate-order history intact,
    // while the receipts prove generic language-ignore and base-derived US selection.
    write(
        &dir,
        "ares.csf",
        &supplemental_fixture(&[("A", "base")], 9, 3, 1),
    );
    write(&dir, "ares_us.csf", &build_test_csf("A", "base"));
    write(
        &dir,
        "ares_ch.csf",
        &build_test_csf("A", "wrong-host-locale"),
    );
    let extra = original_pairs(&row["supplemental"]);
    write(
        &dir,
        "stringtable00.csf",
        &supplemental_fixture(&extra, 0, 3, extra.len() as u32),
    );
    let assets = open(&dir);
    let (table, admitted) = assets.load_active_csf_with_sources().unwrap();
    assert_eq!(admitted, ["ares.csf", "ares_us.csf", "stringtable00.csf"]);
    assert_eq!(table.language, 0);
    assert_eq!(
        serde_json::json!(table.entries().collect::<Vec<_>>()),
        row["result"]["labels"]
    );
    for (key, value) in row["result"]["lookups"].as_object().unwrap() {
        assert_eq!(table.get(key), value.as_str());
    }
    assert_eq!(
        type_ui_name(Some("KEEP"), Some(&table)),
        row["result"]["lookups"]["KEEP"].as_str().unwrap()
    );

    // A second existing native history is needed for malformed-body partial
    // commits: it has a different base and must not be mixed with duplicate ties.
    let partial = crate::test_fixture::json("tools/rules_oracle/ares_csf.json");
    let expected = &partial["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "partial_body_keeps_prior_writes")
        .unwrap()["result"];
    let dir = audio_test_install("sdk-startup-csf-partial");
    early_component(&dir, &wasm);
    write(
        &dir,
        "ra2md.csf",
        &supplemental_fixture(&[("A", "base"), ("KEEP", "keep")], 0, 3, 2),
    );
    let mut bytes = build_test_csf("NEW", "committed");
    let broken = build_test_csf("A", "clear_old");
    bytes.extend_from_slice(&broken[24..24 + 12 + 1]);
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    write(&dir, "stringtable00.csf", &bytes);
    let assets = open(&dir);
    let (table, admitted) = assets.load_active_csf_with_sources().unwrap();
    assert_eq!(serde_json::json!(admitted), expected["accepted"]);
    assert_eq!(serde_json::json!(table.language), expected["base_language"]);
    for (key, value) in expected["entries"].as_object().unwrap() {
        assert_eq!(table.get(key), value.as_str());
    }
    assert_eq!(type_ui_name(Some("NEW"), Some(&table)), "committed");
    assert_eq!(type_ui_name(Some("A"), Some(&table)), "MISSING:'A'");
}

#[test]
#[ignore = "requires VERA_SDK_STARTUP_WASM and the previous component in VERA_SDK_PREVIOUS_WASM"]
fn sdk_startup_deferred_old_component_binds_but_new_startup_and_invalid_identity_are_rejected() {
    let previous = component("VERA_SDK_PREVIOUS_WASM");
    let wasm = component("VERA_SDK_STARTUP_WASM");
    let dir = audio_test_install("sdk-startup-deferred");
    declaration(&dir, &previous);
    archive(
        &dir,
        "conqmd.mix",
        &[("gameplay.wasm".into(), previous.clone())],
    );
    write(
        &dir,
        "rulesmd.ini",
        b"[VehicleTypes]\n0=TANK\n[TANK]\nStrength=100\nAmmo=6\n",
    );
    let assets = open(&dir);
    assert!(matches!(
        &assets.startup_resources().component,
        StartupComponentSource::Deferred { .. }
    ));
    assert_eq!(assets.hash_include_override(), None);
    assert_eq!(
        configured(&assets)
            .1
            .external_mod
            .as_ref()
            .unwrap()
            .sha256(),
        sha256_hex(&previous)
    );
    drop(assets);

    // An identity-only component still needs the early frontier, even though
    // it has no StartupResources hook and no gameplay configuration.
    let request = vera_mod_api::Request {
        api_version: 1,
        context: vera_mod_api::Context::SpawnIdentity,
        config: vec![],
        state: vec![],
    };
    let mut plan = vera_mod_api::Plan::unchanged(&request);
    plan.spawn_identity = Some(vera_mod_api::SpawnIdentityPolicy {
        extended_countries: true,
        campaign_side_byte: false,
    });
    let identity_only = vera_mod_runtime::test_support::component_fixture_by_kind(
        &vera_mod_api::ModuleManifest {
            api_version: 1,
            id: "spawn-identity-only".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                vera_mod_api::HookKind::Configure,
                vera_mod_api::HookKind::Attach,
                vera_mod_api::HookKind::Damage,
                vera_mod_api::HookKind::Tick,
                vera_mod_api::HookKind::SpawnIdentity,
            ],
        },
        &serde_json::to_vec(&vera_mod_api::Plan::unchanged(&request)).unwrap(),
        &[("spawn_identity", &serde_json::to_vec(&plan).unwrap())],
    );
    let identity_dir = audio_test_install("sdk-spawn-identity-only");
    early_component(&identity_dir, &identity_only);
    let identity_assets = open(&identity_dir);
    assert_eq!(
        identity_assets.startup_resources().spawn_identity,
        Some(crate::assets::startup_resources::SpawnIdentityPolicy {
            extended_countries: true,
            campaign_side_byte: false,
        })
    );
    let late_identity = audio_test_install("sdk-spawn-identity-only-late");
    declaration(&late_identity, &identity_only);
    archive(
        &late_identity,
        "conqmd.mix",
        &[("gameplay.wasm".into(), identity_only)],
    );
    assert!(
        open_active_assets(late_identity.path(), MediaArchiveMode::STOCK_DIGITAL)
            .err()
            .unwrap()
            .to_string()
            .contains("before resource selection")
    );

    declaration(&dir, &wasm);
    archive(
        &dir,
        "conqmd.mix",
        &[("gameplay.wasm".into(), wasm.clone())],
    );
    let error = open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
        .err()
        .expect("late startup capability cannot silently use stock resources");
    assert!(error.to_string().contains("before resource selection"));
    write(&dir, "gameplay.wasm", &wasm);
    write(
        &dir,
        "vera-mod.json",
        b"{\"api_version\":0,\"program\":\"gameplay.wasm\",\"sha256\":\"bad\"}",
    );
    let error = open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
        .err()
        .expect("invalid declaration must fail at early admission");
    assert!(error.to_string().contains("invalid mod declaration"));
    let wrong: Value = serde_json::json!({
        "api_version": vera_mod_api::API_VERSION, "program": "gameplay.wasm", "sha256": sha256_hex(&previous),
    });
    assert_ne!(
        wrong["sha256"],
        sha256_hex(&wasm),
        "previous component must have a distinct digest"
    );
    write(&dir, "vera-mod.json", &serde_json::to_vec(&wrong).unwrap());
    let error = open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
        .err()
        .expect("visible mismatched bytes are not a deferred payload");
    assert!(error.to_string().contains("content digest mismatch"));
}
