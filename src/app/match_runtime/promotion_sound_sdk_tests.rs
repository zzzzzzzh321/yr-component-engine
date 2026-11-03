//! Real component → fixed SOUNDMD/native Rules → promotion AI → app queue.
//! No audio device runs: this verifies admitted cue/EVA delivery and persistence,
//! not PCM output or device timing. The production dispatcher remains the owner.

use super::dispatch_sim_sound_events;
use crate::assets::asset_manager::{
    AssetManager, MediaArchiveMode,
    test_support::{TestDirectory, audio_test_install},
};
use crate::audio::events::{GameSoundEvent, SoundEventQueue};
use crate::rules::{
    art_data::ArtRegistry,
    audio_sources::AudioDefinitions,
    ini_parser::IniFile,
    native_processing::ProcessedRulesLayers,
    process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix},
    retail_sources::RetailRulesSources,
};
use crate::sim::{
    arena_fixture::rival_arena,
    runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources},
    world::{SimSoundEvent, Simulation},
};
use crate::util::save_codec;

const SOUNDMD: &str =
    "[SoundList]\n0=GlobalVeteran\n1=GlobalElite\n2=HeroVeteran\n3=HeroElite\n4=none\n";
const RULESMD: &str = "[Clear]\nBuildable=yes\n\
    [General]\nVeteranCombat=1.0\n[Countries]\n0=Americans\n1=Russians\n\
    [AudioVisual]\nUpgradeVeteranSound=GlobalVeteran\nUpgradeEliteSound=GlobalElite\n\
    [InfantryTypes]\n0=HERO\n1=SILENT\n2=DEFAULT\n3=LITERAL\n\
    [HERO]\nStrength=100\nSpeed=4\nPromote.VeteranSound=HeroVeteran\nPromote.EliteSound=HeroElite\n\
    [SILENT]\nStrength=100\nSpeed=4\nPromote.VeteranSound=HeroVeteran\n\
    [DEFAULT]\nStrength=100\nSpeed=4\n\
    [LITERAL]\nStrength=100\nSpeed=4\nPromote.VeteranSound=none\nPromote.EliteSound=none\n";

struct Fixture {
    _directory: TestDirectory,
    assets: AssetManager,
    owner: NativeRulesProcessOwner,
    rules: crate::rules::ruleset::RuleSet,
    art: IniFile,
}

fn fixture(label: &str, module: Option<&[u8]>, sounds: &str) -> Fixture {
    let directory = audio_test_install(label);
    // Physical sources, including the module, precede the startup frontier.
    for (name, contents) in [
        ("rulesmd.ini", RULESMD),
        ("artmd.ini", ""),
        ("soundmd.ini", sounds),
        (
            "evamd.ini",
            "[DialogList]\n0=EVA_UnitPromoted\n[EVA_UnitPromoted]\nAllied=promoted\n",
        ),
    ] {
        std::fs::write(directory.path().join(name), contents).unwrap();
    }
    if let Some(module) = module {
        std::fs::write(directory.path().join("gameplay.wasm"), module).unwrap();
        std::fs::write(
            directory.path().join("vera-mod.json"),
            serde_json::to_vec(&serde_json::json!({
                "api_version": vera_mod_api::API_VERSION,
                "program": "gameplay.wasm",
                "sha256": crate::util::sha256::sha256_hex(module),
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let assets =
        crate::mod_sdk::open_active_assets(directory.path(), MediaArchiveMode::STOCK_DIGITAL)
            .unwrap();
    let audio = AudioDefinitions::select(&assets);
    let mut owner = RetailRulesSources::select(&assets)
        .unwrap()
        .into_owner(audio.sounds().clone(), audio.eva().clone())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(owner.fixed_sounds(), audio.sounds()));
    // Ares NullableIdx<Voc>1000E9D0: unknown and empty keep HERO's earlier
    // valid declaration; <none> clears only SILENT's veteran cue. Bare none
    // remains the registered cue used by LITERAL, not a silence token.
    let (mut rules, _, art, _) = owner.load_scenario(
        NativeScenarioRulesPrefix::NonCampaign(None),
        &IniFile::from_str("[HERO]\nPromote.VeteranSound=Unknown\nPromote.EliteSound=\n[SILENT]\nPromote.VeteranSound=<none>\n"),
    ).unwrap().into_parts();
    rules.install_art_data(ArtRegistry::from_ini(&art));
    Fixture {
        _directory: directory,
        assets,
        owner,
        rules,
        art,
    }
}

/// Collect only actual promotion facts from the existing authoritative stage.
/// Other AI sound categories have their own dispatcher regressions.
fn sample_promotions(
    sim: &mut Simulation,
    rules: &crate::rules::ruleset::RuleSet,
) -> Vec<SimSoundEvent> {
    sim.object_ai_stage(Some(rules));
    sim.take_sound_events_for_test()
        .into_iter()
        .filter(|event| matches!(event, SimSoundEvent::UnitPromoted { .. }))
        .collect()
}

fn dispatch_promotions(
    events: Vec<SimSoundEvent>,
    sim: &Simulation,
    rules: &crate::rules::ruleset::RuleSet,
) -> Vec<GameSoundEvent> {
    let hash = sim.state_hash();
    let mut output = SoundEventQueue::new();
    dispatch_sim_sound_events(
        events,
        sim,
        rules,
        Some("Americans"),
        &mut |_| panic!("promotion has no radar gate"),
        &mut output,
    );
    assert_eq!(
        hash,
        sim.state_hash(),
        "presentation cannot change the simulation"
    );
    output.drain()
}

#[test]
#[ignore = "requires VERA_SDK_PROMOTION_SOUND_WASM / VERA_SDK_PROMOTION_SOUND_PREVIOUS_WASM"]
fn component_promotion_cues_reach_dispatcher_and_restore_against_fixed_soundmd() {
    use vera_mod_api::{
        Context, HookKind, ModuleManifest, Plan, Request, TypeSoundResourceContext,
        TypeSoundResourcePolicy,
    };
    let legacy = crate::rules::ARES_ENABLED;
    let component = std::fs::read(
        std::env::var_os("VERA_SDK_PROMOTION_SOUND_WASM")
            .expect("set VERA_SDK_PROMOTION_SOUND_WASM"),
    )
    .unwrap();
    let previous = std::fs::read(
        std::env::var_os("VERA_SDK_PROMOTION_SOUND_PREVIOUS_WASM")
            .expect("set VERA_SDK_PROMOTION_SOUND_PREVIOUS_WASM"),
    )
    .unwrap();
    let request = Request {
        api_version: vera_mod_api::API_VERSION,
        context: Context::TypeSoundResource(TypeSoundResourceContext {}),
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut selected = unchanged.clone();
    selected.type_sound_resource = Some(TypeSoundResourcePolicy::default());
    let off = vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: vera_mod_api::API_VERSION,
            id: "test.promotion-stock".into(),
            version: "1".into(),
            config_version: 1,
            state_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::ConfigureBoundary,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::TypeSoundResource,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[(
            "type_sound_resource",
            &serde_json::to_vec(&selected).unwrap(),
        )],
    );
    let mut controls = Vec::new();
    for (label, module, current, overrides) in [
        (
            "promotion-component",
            Some(component.as_slice()),
            true,
            true,
        ),
        ("promotion-stock", None, false, legacy),
        (
            "promotion-explicit-stock",
            Some(off.as_slice()),
            false,
            false,
        ),
        (
            "promotion-previous",
            Some(previous.as_slice()),
            false,
            legacy,
        ),
    ] {
        eprintln!("promotion sound dispatcher control: {label}");
        let mut scene = fixture(label, module, SOUNDMD);
        let mut sim = rival_arena(&scene.rules, 31);
        bind_scenario_gameplay_resources(
            &mut scene.rules,
            &scene.assets,
            ScenarioGameplayProgram::Fresh {
                simulation: &mut sim,
                rules_owner: &mut scene.owner,
            },
            "tem",
            "TEMPERATE",
        )
        .unwrap();
        let mut ids = Vec::new();
        for (name, x) in [
            ("HERO", 10),
            ("SILENT", 12),
            ("DEFAULT", 14),
            ("LITERAL", 16),
        ] {
            ids.push(
                sim.spawn_object(name, "Americans", x, 10, 0, &scene.rules)
                    .unwrap(),
            );
        }
        let remote = sim
            .spawn_object("HERO", "Russians", 20, 10, 0, &scene.rules)
            .unwrap();
        ids.push(remote);
        // A newly constructed, already-elite object must also sample silently.
        // Set only the public veterancy accumulator; never write its rank cache.
        let initially_elite = sim
            .spawn_object("DEFAULT", "Americans", 22, 10, 0, &scene.rules)
            .unwrap();
        sim.entities_mut_for_test()
            .get_mut(initially_elite)
            .unwrap()
            .veterancy
            .set_elite();
        assert!(
            sample_promotions(&mut sim, &scene.rules).is_empty(),
            "first sampling is silent even for an elite"
        );
        let mut heard_ranks = Vec::new();
        for elite in [false, true] {
            for id in &ids {
                let actor = sim.entities_mut_for_test().get_mut(*id).unwrap();
                if elite {
                    actor.veterancy.set_elite();
                } else {
                    actor.veterancy.set_veteran();
                }
            }
            let events = sample_promotions(&mut sim, &scene.rules);
            assert_eq!(
                events.len(),
                5,
                "one actual promotion for each crossing, {label} elite={elite}"
            );
            let remote_events: Vec<_> = events.iter().filter(|event|
                matches!(event, SimSoundEvent::UnitPromoted { owner, .. } if sim.interner().resolve(*owner) == "Russians")
            ).cloned().collect();
            assert_eq!(remote_events.len(), 1, "the remote producer really ran");
            assert!(
                dispatch_promotions(remote_events, &sim, &scene.rules).is_empty(),
                "a remote promotion has neither local SFX nor EVA"
            );
            let played = dispatch_promotions(events, &sim, &scene.rules);
            let global = if elite {
                "GlobalElite"
            } else {
                "GlobalVeteran"
            };
            let expected = if overrides {
                [
                    Some(if elite { "HeroElite" } else { "HeroVeteran" }),
                    elite.then_some(global),
                    Some(global),
                    Some("none"),
                ]
            } else {
                [Some(global); 4]
            };
            let mut cursor = 0;
            for ((cue, x), id) in expected.into_iter().zip([10, 12, 14, 16]).zip(&ids) {
                if let Some(cue) = cue {
                    assert!(
                        matches!(&played[cursor], GameSoundEvent::UnitPromoted { sound_id, source: Some(source) }
                        if sound_id == cue && source.cell() == (x, 10)),
                        "ordered positional SFX for {id}: {played:?}"
                    );
                    cursor += 1;
                }
                assert!(
                    matches!(&played[cursor], GameSoundEvent::Eva { event, type_override: None }
                    if event == "EVA_UnitPromoted"),
                    "SFX precedes EVA, and silence retains EVA: {played:?}"
                );
                cursor += 1;
            }
            assert_eq!(cursor, played.len(), "only the local four promotions play");
            heard_ranks.push(
                played
                    .iter()
                    .map(|event| event.sound_id().to_owned())
                    .collect::<Vec<_>>(),
            );
            assert!(
                sample_promotions(&mut sim, &scene.rules).is_empty(),
                "same rank never repeats"
            );
            if current && !elite {
                // Restore both authoritative owners, using the same saved
                // Program/config rather than replaying Configure against INI.
                let bytes =
                    save_codec::serialize(&(scene.owner.saved_scenario_rules().unwrap(), &sim))
                        .unwrap();
                let (processed, restored): (ProcessedRulesLayers, Simulation) =
                    save_codec::deserialize(&bytes).unwrap();
                let mut restored_rules = scene
                    .owner
                    .project_saved_scenario_rules(&processed)
                    .unwrap();
                restored_rules.install_art_data(ArtRegistry::from_ini(&scene.art));
                bind_scenario_gameplay_resources(
                    &mut restored_rules,
                    &scene.assets,
                    ScenarioGameplayProgram::Saved(&restored),
                    "tem",
                    "TEMPERATE",
                )
                .unwrap();
                // Same cue names and ordinals, changed fixed source: a saved
                // Named cue must not silently rebind to this different catalog.
                let changed_soundmd = format!("{SOUNDMD}[HeroElite]\nSounds=other-sample\n");
                let other = fixture(
                    "promotion-changed-soundmd",
                    Some(component.as_slice()),
                    &changed_soundmd,
                );
                let first = scene.owner.fixed_sounds();
                let second = other.owner.fixed_sounds();
                assert_ne!(first.source_hash(), second.source_hash());
                for index in 0..5 {
                    assert_eq!(
                        first.entry_by_index(index).unwrap().id,
                        second.entry_by_index(index).unwrap().id,
                        "names and ordinals alone do not establish source identity"
                    );
                }
                let mut rejected = other
                    .owner
                    .project_saved_scenario_rules(&processed)
                    .unwrap();
                rejected.install_art_data(ArtRegistry::from_ini(&other.art));
                let error = bind_scenario_gameplay_resources(
                    &mut rejected,
                    &other.assets,
                    ScenarioGameplayProgram::Saved(&restored),
                    "tem",
                    "TEMPERATE",
                )
                .unwrap_err();
                assert_eq!(
                    error,
                    "promotion sounds differ from the process-selected sound catalog"
                );
                scene
                    .owner
                    .install_saved_scenario_rules(processed, Some(&restored_rules));
                scene.rules = restored_rules;
                sim = restored;
                assert!(
                    sample_promotions(&mut sim, &scene.rules).is_empty(),
                    "saved veteran cache prevents replay before elite"
                );
            }
        }
        controls.push(heard_ranks);
    }
    let [current, stock, off, previous] = controls.as_slice() else {
        unreachable!("four controls")
    };
    assert_eq!(
        stock, previous,
        "the previous component retains the original host profile"
    );
    if legacy {
        assert_eq!(current, stock, "the component retains legacy cue results");
        assert_ne!(
            current, off,
            "an explicit empty catalog overrides legacy cues"
        );
    } else {
        assert_eq!(stock, off, "an explicit empty catalog uses global cues");
        assert_ne!(
            current, stock,
            "the component changes ordinary promotion cues"
        );
    }
}
