//! Actual components, original physical Rules visits and the saved Program.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;

#[test]
#[ignore = "requires VERA_SDK_EVA_WASM (.39) and VERA_SDK_EVA_PREVIOUS_WASM (.38)"]
fn sdk_eva_fixed_catalog_physical_retention_initialization_and_saved_cohorts() {
    let current = component("VERA_SDK_EVA_WASM");
    let previous = component("VERA_SDK_EVA_PREVIOUS_WASM");
    let root = BASE.replace("4=Warp\n", "4=Warp\n5=Atomic\n6=Late\n");
    let root = format!(
        "{root}\n[Atomic]\nType=MultiMissile\nEVA.Ready=CustomReady\nEVA.Activated=missing\nEVA.Impatient=CustomWait\n"
    );
    let mode = "[Atomic]\nEVA.Ready=unknown\nEVA.Detected=none\nEVA.SelectTarget=<none>\n[Late]\nType=MultiMissile\n";
    let map = "[Atomic]\nType=GenericWarhead\nEVA.Impatient=\nEVA.Activated=unknown\nEVA.SelectTarget=\n[DialogList]\n999=unknown\n";
    let eva = "[DialogList]\n0=EVA_NuclearMissileReady\n1=EVA_NuclearMissileLaunched\n2=EVA_NuclearSiloDetected\n3=EVA_InsufficientFunds\n4=EVA_SelectTarget\n5=CustomReady\n6=CustomWait\n7=CustomTarget\n8=none\n";
    let mut scene = Scene::with_eva(
        "sdk-eva-retention",
        Some(&current),
        &root,
        Some(mode),
        map,
        Some(eva),
    );
    let inspect = |rules: &RuleSet| {
        let atomic = rules.super_weapon("Atomic").unwrap();
        let refs = rules.super_weapon_eva(atomic).unwrap();
        assert_eq!(refs.ready(), Some("CustomReady"));
        assert_eq!(
            refs.activated(),
            Some("EVA_NuclearMissileLaunched"),
            "unknown overrides retain the first Nuclear initializer across Type changes"
        );
        assert_eq!(
            refs.detected(),
            Some("none"),
            "registered names precede the sentinel"
        );
        assert_eq!(refs.impatient(), Some("CustomWait"));
        assert_eq!(
            refs.select_target(),
            None,
            "an empty map value retains the mode's explicit silence"
        );
        assert_eq!(
            atomic.insufficient_funds_eva.as_deref(),
            Some("EVA_InsufficientFunds")
        );
        let late = rules.super_weapon("Late").unwrap();
        let refs = rules.super_weapon_eva(late).unwrap();
        assert_eq!(
            refs.ready(),
            None,
            "the absent first body consumes Initialize"
        );
        assert_eq!(
            refs.select_target(),
            Some("EVA_SelectTarget"),
            "constructor defaults do not require a body"
        );
        assert!(late.nuclear.is_none());
    };
    inspect(&scene.rules);
    scene.roundtrip();
    inspect(&scene.rules);
    let mut old = Scene::with_eva(
        "sdk-eva-historical",
        Some(&previous),
        &root,
        Some(mode),
        map,
        Some(eva),
    );
    assert!(
        old.rules
            .super_weapon_eva(old.rules.super_weapon("Atomic").unwrap())
            .is_none()
    );
    old.roundtrip();
    assert!(
        old.rules
            .super_weapon_eva(old.rules.super_weapon("Atomic").unwrap())
            .is_none()
    );
}

#[test]
#[ignore = "requires VERA_SDK_EVA_WASM (.39)"]
fn sdk_eva_additional_provider_detection_repeats_on_real_capture_opening() {
    let module = component("VERA_SDK_EVA_WASM");
    let root = BASE
        .replace(
            "SuperWeapon=Strike\nSuperWeapon2=Delivery\nSuperWeapons=Sphere\n",
            "SuperWeapon=Reveal\nSuperWeapon2=none\nSuperWeapons=Strike,Delivery\n",
        )
        .replace(
            "[Reveal]\nType=PsychicReveal\n",
            "[Reveal]\nType=PsychicReveal\nSW.AllowPlayer=no\nSW.AllowAI=no\n",
        );
    let mut scene = Scene::new("sdk-eva-capture", Some(&module), &root, "");
    let building = scene.launcher();
    let owner = scene.id("Americans");
    let enemy = scene.id("Russians");
    let strike = scene.id("Strike");
    assert!(scene.sim.sound_events.iter().any(|event| matches!(event, SimSoundEvent::SuperWeaponDetected { owner: who, sw_type } if *who == owner && *sw_type == strike)));
    scene.sim.take_sound_events_for_test();
    scene
        .sim
        .change_owner_with_rules(building, enemy, &scene.rules, None, FrameEffects::default());
    let detected: Vec<_> = scene
        .sim
        .sound_events
        .iter()
        .filter_map(|event| match event {
            SimSoundEvent::SuperWeaponDetected { owner, sw_type } => Some((*owner, *sw_type)),
            _ => None,
        })
        .collect();
    assert_eq!(detected, [(enemy, strike)]);
}
