//! Ordinary host, actual component and charged provider. Sidebar decisions,
//! successful Nuclear launch and final queue removal use production owners.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;
use crate::app::input::superweapon_cameo;
use crate::assets::asset_manager::{MediaArchiveMode, test_support::audio_test_install};
use crate::rules::{
    art_data::ArtRegistry, audio_sources::AudioDefinitions, ini_parser::IniFile,
    process_owner::NativeScenarioRulesPrefix, retail_sources::RetailRulesSources,
};
use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};

#[test]
#[ignore = "requires VERA_SDK_EVA_WASM (.39)"]
fn sdk_eva_live_charge_sidebar_nuclear_success_and_vox_cleanup() {
    let module =
        std::fs::read(std::env::var_os("VERA_SDK_EVA_WASM").expect("set actual .39 component"))
            .unwrap();
    let dir = audio_test_install("sdk-eva-live");
    let write =
        |name: &str, contents: &[u8]| std::fs::write(dir.path().join(name), contents).unwrap();
    write("gameplay.wasm", &module);
    write("vera-mod.json", &serde_json::to_vec(&serde_json::json!({"api_version":1,"program":"gameplay.wasm","sha256":crate::util::sha256::sha256_hex(&module)})).unwrap());
    write("rulesmd.ini", b"[Clear]\nBuildable=yes\n[Countries]\n0=Americans\n[Americans]\nParentCountry=Americans\n[Sides]\nGDI=Americans\n[BuildingTypes]\n0=PROVIDER\n1=KEEP\n[PROVIDER]\nStrength=1000\nSuperWeapon=Atomic\n[KEEP]\nStrength=1000\n[WeaponTypes]\n0=Carrier\n1=Payload\n[Warheads]\n0=WH\n[WH]\nCellSpread=0\n[Carrier]\nDamage=0\nRange=30\nSpeed=100\nProjectile=Fall\nWarhead=WH\n[Payload]\nDamage=1\nRange=30\nSpeed=100\nProjectile=Fall\nWarhead=WH\n[Fall]\nVertical=yes\nArm=2\n[SuperWeaponTypes]\n0=Atomic\n[Atomic]\nType=MultiMissile\nAction=Nuke\nWeaponType=Carrier\nNuke.Payload=Payload\nNuke.SiloLaunch=no\nIsPowered=no\nRechargeTime=1\nSW.AITargeting.Constraints=none\nEVA.Ready=CustomReady\nEVA.Activated=CustomLaunch\nEVA.Detected=CustomDetected\nEVA.Impatient=CustomWait\nEVA.SelectTarget=CustomTarget\n");
    write(
        "artmd.ini",
        b"[PROVIDER]\nFoundation=1x1\n[KEEP]\nFoundation=1x1\n",
    );
    write("evamd.ini", b"[DialogList]\n0=CustomReady\n1=CustomLaunch\n2=CustomDetected\n3=CustomWait\n4=CustomTarget\n5=EVA_NuclearMissileReady\n[CustomReady]\nType=Queue\n[CustomLaunch]\nType=Queue\n");
    let assets =
        crate::mod_sdk::open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap();
    let audio = AudioDefinitions::select(&assets);
    let mut process = RetailRulesSources::select(&assets)
        .unwrap()
        .into_owner(audio.sounds().clone(), audio.eva().clone())
        .unwrap();
    let (mut rules, _, art, _) = process
        .load_scenario(
            NativeScenarioRulesPrefix::NonCampaign(None),
            &IniFile::empty(),
        )
        .unwrap()
        .into_parts();
    rules.install_art_data(ArtRegistry::from_ini(&art));
    let mut sim = Simulation::with_seed(17);
    bind_scenario_gameplay_resources(
        &mut rules,
        &assets,
        ScenarioGameplayProgram::Fresh {
            simulation: &mut sim,
            rules_owner: &mut process,
        },
        "tem",
        "TEMPERATE",
    )
    .unwrap();
    sim.intern_rule_type_ids(&rules);
    sim.resolve_type_handles(&rules);
    let request = crate::app::frontend::cncnet_spawn::SpawnRequest::parse("[Settings]\nScenario=fixture.map\nSide=0\nColor=0\nSpecialHouseIsAlly=No\nAIPlayers=1\nSeed=17\n[HouseHandicaps]\nMulti2=1\n[HouseCountries]\nMulti2=0\n[HouseColors]\nMulti2=1\n[SpawnLocations]\nMulti1=0\nMulti2=1\n").unwrap();
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
    let local = sim.session().current_house().unwrap();
    let local_name = sim.interner().resolve(local).to_owned();
    sim.install_session_mode(crate::sim::scenario_session::ScenarioGameMode::Skirmish);
    crate::sim::arena_fixture::flat_ground(&mut sim, &rules);
    sim.session_mut_for_test().set_binary_frame_for_test(100);
    sim.spawn_object_at_height("PROVIDER", "Computer1", 20, 20, 0, 0, &rules)
        .unwrap();
    sim.spawn_object_at_height("PROVIDER", &local_name, 8, 8, 0, 0, &rules)
        .unwrap();
    let hear = |sim: &mut Simulation| {
        let mut output = SoundEventQueue::new();
        dispatch_sim_sound_events(
            sim.take_sound_events_for_test(),
            sim,
            &rules,
            Some(&local_name),
            &mut |_| true,
            &mut output,
        );
        output.drain()
    };
    assert!(
        hear(&mut sim).iter().any(
            |cue| matches!(cue, GameSoundEvent::Eva { event, .. } if event == "CustomDetected")
        )
    );
    sim.advance_tick(&[], Some(&rules), None, None, 33);
    let atomic = sim.interner().get("Atomic").unwrap();
    let sidebar = |sim: &Simulation| {
        let view = crate::sim::superweapon::superweapon_views_for_owner(sim, &rules, &local)
            .into_iter()
            .find(|view| view.type_id == atomic)
            .unwrap();
        let mut input = crate::app::input::state::MatchInputState::new(
            crate::app::input::hotkeys::HotkeyBindings::default(),
        );
        let mut gadgets = crate::ui::sidebar::gadget_flash::SidebarGadgetState::default();
        let click = superweapon_cameo::press(
            &mut input,
            &mut gadgets,
            Some(&view),
            rules.super_weapon("Atomic").unwrap().action.as_deref(),
            false,
        );
        (
            click,
            superweapon_cameo::eva(&rules, "Atomic", Some(&view), click).map(str::to_owned),
        )
    };
    assert_eq!(
        sidebar(&sim),
        (
            superweapon_cameo::SuperWeaponClick::Refused,
            Some("CustomWait".into())
        )
    );
    hear(&mut sim);
    sim.session_mut_for_test().set_binary_frame_for_test(1001);
    sim.advance_tick(&[], Some(&rules), None, None, 33);
    let ready = hear(&mut sim);
    assert!(
        ready
            .iter()
            .any(|cue| matches!(cue, GameSoundEvent::Eva { event, .. } if event == "CustomReady"))
    );
    assert_eq!(
        sidebar(&sim),
        (
            superweapon_cameo::SuperWeaponClick::Target,
            Some("CustomTarget".into())
        )
    );
    let mut queue = crate::audio::vox::VoxQueue::new();
    let line = |name| {
        let entry = audio.eva().entry(name).unwrap();
        crate::audio::vox::VoxRequest {
            event: &entry.name,
            eva_type: entry.eva_type,
            priority: entry.priority,
        }
    };
    queue.queue_voice(line("CustomReady"), None, None);
    queue.queue_voice(line("EVA_NuclearMissileReady"), None, None);
    assert!(sim.apply_command(
        &local_name,
        &crate::sim::command::Command::LaunchSuperWeapon {
            sw_type_id: atomic,
            target_rx: 16,
            target_ry: 16
        },
        Some(&rules)
    ));
    let launch = hear(&mut sim);
    let lifecycle: Vec<_> = launch
        .iter()
        .filter_map(|cue| match cue {
            GameSoundEvent::SuperWeaponActivated {
                eva_event: Some(event),
                ..
            } => Some((false, event.as_str())),
            GameSoundEvent::EvaRemove { event } => Some((true, event.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(lifecycle, [(false, "CustomLaunch"), (true, "CustomReady")]);
    for cue in &launch {
        if let GameSoundEvent::EvaRemove { event } = cue {
            queue.remove_from_queues(event);
        }
    }
    assert_eq!(
        queue.queued_count(),
        1,
        "configured cleanup preserves an unrelated stock Ready entry"
    );
    let hash = sim.state_hash();
    let remote = [SimSoundEvent::AresSuperWeaponFinished {
        owner: sim.interner().get("Computer1").unwrap(),
        sw_type: atomic,
    }];
    let mut output = SoundEventQueue::new();
    dispatch_sim_sound_events(
        remote,
        &sim,
        &rules,
        Some(&local_name),
        &mut |_| true,
        &mut output,
    );
    assert!(output.drain().is_empty());
    assert_eq!(sim.state_hash(), hash);
}
