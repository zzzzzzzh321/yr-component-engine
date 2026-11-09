//! The local command producer shared by tactical and radar clicks. Actual
//! component admission, no ready-state injection or alternative target logic.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;
use crate::assets::asset_manager::{MediaArchiveMode, test_support::audio_test_install};
use crate::rules::{ini_parser::IniFile, process_owner::NativeScenarioRulesPrefix};
use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};

#[test]
#[ignore = "requires VERA_SDK_COMMON_TARGETING_WASM (.37)"]
fn common_superweapon_local_click_checks_target_before_emitting_command() {
    let module = std::fs::read(
        std::env::var_os("VERA_SDK_COMMON_TARGETING_WASM").expect("set actual .37 component"),
    )
    .unwrap();
    // One ordinary targeted click and its UseAI counterpart: the latter's
    // placeholder must remain a selector request, even outside manual range.
    for use_ai in [false, true] {
        let dir = audio_test_install(if use_ai {
            "input-common-ai"
        } else {
            "input-common"
        });
        std::fs::write(dir.path().join("gameplay.wasm"), &module).unwrap();
        std::fs::write(
            dir.path().join("vera-mod.json"),
            serde_json::to_vec(&serde_json::json!({
                "api_version": 1, "program": "gameplay.wasm",
                "sha256": crate::util::sha256::sha256_hex(&module)
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(dir.path().join("rulesmd.ini"), format!(
            "[Clear]\nBuildable=yes\n[Countries]\n0=Americans\n\
             [Americans]\nName=USAlias\nParentCountry=Americans\n[Sides]\nGDI=Americans\n\
             [BuildingTypes]\n0=PROVIDER\n[PROVIDER]\nStrength=200\nArmor=wood\nSuperWeapon=Strike\n\
             [Warheads]\n0=Payload\n[Payload]\nCellSpread=0\n\
             [SuperWeaponTypes]\n0=Strike\n[Strike]\nType=GenericWarhead\nIsPowered=no\n\
             SW.Warhead=Payload\nSW.Damage=1\nSW.RangeMaximum=5\nSW.UseAITargeting={}\n",
            if use_ai { "yes" } else { "no" }
        )).unwrap();
        std::fs::write(dir.path().join("artmd.ini"), "[PROVIDER]\nFoundation=1x1\n").unwrap();
        let assets =
            crate::mod_sdk::open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
                .unwrap();
        let audio = crate::rules::audio_sources::AudioDefinitions::select(&assets);
        let mut owner = crate::rules::retail_sources::RetailRulesSources::select(&assets)
            .unwrap()
            .into_owner(audio.sounds().clone(), audio.eva().clone())
            .unwrap();
        let (mut rules, ..) = owner
            .load_scenario(
                NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::from_str(""),
            )
            .unwrap()
            .into_parts();
        let mut sim = crate::sim::world::Simulation::with_seed(17);
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
        crate::sim::arena_fixture::flat_ground(&mut sim, &rules);
        let house = sim.intern("Americans");
        sim.houses_mut_for_test()
            .register(crate::sim::house_state::HouseState::new(
                house,
                0,
                Some(house),
                true,
                2000,
                10,
            ));
        sim.session_mut_for_test()
            .set_current_house_for_test(Some(house));
        sim.session_mut_for_test()
            .game_options_mut_for_test()
            .super_weapons = true;
        sim.spawn_object_at_height("PROVIDER", "Americans", 12, 12, 0, 0, &rules)
            .unwrap();
        let strike = sim.interner().get("Strike").unwrap();
        let before = sim.state_hash();
        let refused = super_weapon_click_command(&sim, &rules, house, strike, (28, 28));
        assert_eq!(refused.is_some(), use_ai);
        assert_eq!(
            sim.state_hash(),
            before,
            "local admission must not mutate simulation"
        );
        assert!(
            sim.network_command_queues()
                .local_records()
                .unwrap()
                .is_empty()
        );
        let cell = if use_ai { (0, 0) } else { (14, 14) };
        let command = super_weapon_click_command(&sim, &rules, house, strike, cell).unwrap();
        assert_eq!(
            command,
            Command::LaunchSuperWeapon {
                sw_type_id: strike,
                target_rx: cell.0,
                target_ry: cell.1
            }
        );
        assert!(schedule_rules_command_in_sim(&mut sim, &rules, "Americans", command).is_some());
        assert_eq!(
            sim.network_command_queues().local_records().unwrap().len(),
            1
        );
    }
}
