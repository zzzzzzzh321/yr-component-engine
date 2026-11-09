//! Actual components, physical source passes, charged commands and the shared
//! selector. No charge injection or alternate targeting/flight implementation.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;

fn root() -> String {
    format!(
        "{}\n{}",
        BASE.replace("4=Warp\n", "4=Warp\n5=Atomic\n")
            .replace(
                "SuperWeapon=Strike\nSuperWeapon2=Delivery\nSuperWeapons=Sphere\n",
                "SuperWeapon=Atomic\nSuperWeapon2=none\nSuperWeapons=,\nNukeSilo=yes\n"
            )
            .replace(
                "[TARGET]\nStrength=200\nArmor=wood\n",
                "[TARGET]\nStrength=200\nArmor=wood\nSuperWeapon=Atomic\nNukeSilo=no\n"
            ),
        "[WeaponTypes]\n0=RangePayload\n1=RangeCarrier\n\
         [Atomic]\nType=MultiMissile\nAction=Nuke\nWeaponType=RangeCarrier\n\
         Nuke.Payload=RangePayload\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\n\
         SW.RangeMaximum=50\nSW.AITargeting.Constraints=none\n\
         [RangeCarrier]\nDamage=0\nRange=30\nSpeed=100\nProjectile=RangeRise\nWarhead=Payload\n\
         [RangePayload]\nDamage=50\nRange=30\nSpeed=100\nProjectile=RangeFall\nWarhead=Payload\n\
         [RangeRise]\nArm=2\nVertical=yes\nAcceleration=1\nDetonationAltitude=20000\n\
         [RangeFall]\nArm=2\nVertical=yes\nAcceleration=1\nDetonationAltitude=30000\n"
    )
}

#[test]
#[ignore = "requires VERA_SDK_COMMON_TARGETING_WASM (.37) and previous (.36)"]
fn sdk_common_targeting_retains_physical_range_and_selects_silo_before_restore() {
    let module = component("VERA_SDK_COMMON_TARGETING_WASM");
    let previous = component("VERA_SDK_COMMON_TARGETING_PREVIOUS_WASM");
    let mode = "[Atomic]\nType=UnitDelivery\nSW.RangeMinimum=2\nSW.RangeMaximum=500%\n\
                SW.AITargeting.Preference=none\n";
    let map = "[Atomic]\nType=ForceShield\nSW.RangeMaximum=invalid\n\
               SW.AITargeting.Preference=unknown\n";
    let old = Scene::with_mode(
        "common-range-old",
        Some(&previous),
        &root(),
        Some(mode),
        map,
    );
    assert!(old.rules.super_weapon_target_policy("Atomic").is_none());
    let mut scene = Scene::with_mode("common-range", Some(&module), &root(), Some(mode), map);
    let policy = scene.rules.super_weapon_target_policy("Atomic").unwrap();
    assert_eq!(policy.range(), (2.0, 5.0));
    assert_eq!(
        policy.preference(),
        crate::rules::superweapon_targeting::AiTargetPreference::None
    );
    assert_eq!(
        policy.mode(),
        crate::rules::superweapon_targeting::AiTargetingMode::Nuke
    );
    let ordinal = scene
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap()
        .source_ordinal;
    let carrier = scene
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .weapon_type
        .unwrap();
    // A near ordinary provider must not satisfy Nuclear's prepared range gate.
    scene
        .sim
        .spawn_object_at_height("TARGET", "Americans", 26, 26, 0, 0, &scene.rules)
        .unwrap();
    let first = scene.launcher_at(12, 12);
    let selected = scene.launcher_at(18, 18);
    assert!(scene.sim.map_cell_in_bounds((12, 12)) && scene.sim.map_cell_in_bounds((18, 18)));
    let (owner, sw) = (scene.id("Americans"), scene.id("Atomic"));
    let before = scene.sim.state_hash();
    assert_eq!(
        scene
            .sim
            .super_weapon_target_check(&scene.rules, owner, sw, (20, 20), false),
        Ok(())
    );
    assert_eq!(
        scene
            .sim
            .super_weapon_target_check(&scene.rules, owner, sw, (28, 28), false),
        Err(crate::rules::superweapon_targeting::TargetReject::NoEligibleLaunchSite)
    );
    assert_eq!(
        scene.sim.state_hash(),
        before,
        "preview must not mutate fog, charge or RNG"
    );
    assert!(scene.fire("Atomic", (20, 20)));
    assert_eq!(
        scene
            .sim
            .entities()
            .get(selected)
            .unwrap()
            .mission_leaf
            .as_building()
            .unwrap()
            .firing_super_weapon(),
        ordinal as i32
    );
    assert_eq!(
        scene
            .sim
            .entities()
            .get(first)
            .unwrap()
            .mission_leaf
            .as_building()
            .unwrap()
            .firing_super_weapon(),
        -1
    );
    nuclear::advance_until(&mut scene, 120, |sim| {
        sim.projectiles
            .iter()
            .any(|(_, bullet)| bullet.payload().installed_weapon() == Some(carrier))
    });
    let (bullet, source) = scene
        .sim
        .projectiles
        .iter()
        .find_map(|(&id, bullet)| {
            (bullet.payload().installed_weapon() == Some(carrier))
                .then_some((id, bullet.source_id()))
        })
        .unwrap();
    assert_eq!(source, selected);
    scene.roundtrip();
    assert_eq!(
        scene
            .rules
            .super_weapon_target_policy("Atomic")
            .unwrap()
            .range(),
        (2.0, 5.0)
    );
    assert_eq!(
        scene.sim.projectiles.get(bullet).unwrap().source_id(),
        selected
    );
    assert_eq!(
        scene
            .sim
            .projectiles
            .get(bullet)
            .unwrap()
            .launching_super_weapon(),
        Some(ordinal)
    );
}

#[test]
#[ignore = "requires VERA_SDK_COMMON_TARGETING_WASM (.37)"]
fn sdk_common_targeting_selected_ai_bypasses_preview_and_uses_direct_fallback() {
    let module = component("VERA_SDK_COMMON_TARGETING_WASM");
    let map = "[Atomic]\nSW.UseAITargeting=yes\nSW.RangeMaximum=1\n";
    let mut scene = Scene::new("common-ai-preferred", Some(&module), &root(), map);
    let silo = scene.launcher_at(12, 12);
    let (owner, sw) = (scene.id("Americans"), scene.id("Atomic"));
    // Task135's existing House owner supplies the selector-entry preference;
    // this does not claim an end-to-end trigger/Team-AI scheduling test.
    scene
        .sim
        .houses
        .get_mut(&owner)
        .unwrap()
        .set_preferred_super_weapon_target((20, 20));
    assert!(
        scene
            .sim
            .super_weapon_target_check(&scene.rules, owner, sw, (20, 20), false)
            .is_err()
    );
    let payload = scene
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap()
        .payload
        .unwrap();
    // SPECIAL_PLACE10038E90 selects the preferred cell instead of the event's
    // placeholder. SelectedAutomatic must not repeat the ordinary range gate;
    // Nuclear100730DA then finds no in-range silo and10073145 drops directly.
    assert!(scene.fire("Atomic", (0, 0)));
    let direct = scene
        .sim
        .projectiles
        .iter()
        .find_map(|(&id, bullet)| {
            (bullet.payload().installed_weapon() == Some(payload)).then_some(id)
        })
        .unwrap();
    assert_eq!(scene.sim.projectiles.get(direct).unwrap().source_id(), 0);
    assert_eq!(
        scene
            .sim
            .entities()
            .get(silo)
            .unwrap()
            .mission_leaf
            .as_building()
            .unwrap()
            .firing_super_weapon(),
        -1
    );
    assert_eq!(scene.sim.super_weapons[&owner][&sw].ares_launch_count, 1);
    assert!(!scene.sim.super_weapons[&owner][&sw].is_ready());
    let position = scene.sim.projectiles.get(direct).unwrap().position();
    scene.roundtrip();
    assert_eq!(
        scene.sim.projectiles.get(direct).unwrap().position(),
        position
    );

    let mut automatic = Scene::new(
        "common-ai-human",
        Some(&module),
        &root(),
        &format!("{map}SW.AutoFire=yes\nSW.ManualFire=no\n"),
    );
    automatic.launcher_at(12, 12);
    let (owner, sw) = (automatic.id("Americans"), automatic.id("Atomic"));
    automatic
        .sim
        .houses
        .get_mut(&owner)
        .unwrap()
        .set_preferred_super_weapon_target((20, 20));
    let view = superweapon_views_for_owner(&automatic.sim, &automatic.rules, &owner)
        .into_iter()
        .find(|view| view.type_id == sw)
        .unwrap();
    assert_eq!(
        view.ares_refusal,
        Some(AresCameoRefusal::ManualFireDisabled)
    );
    // The actual House AI entry also admits a human AutoFire-only type.
    ai_fire::try_fire(
        &mut automatic.sim,
        &automatic.rules,
        owner,
        None,
        FrameEffects::default(),
    );
    assert_eq!(
        automatic.sim.super_weapons[&owner][&sw].ares_launch_count,
        1
    );
    assert!(!automatic.sim.super_weapons[&owner][&sw].is_ready());
    automatic.roundtrip();

    let mut event = Scene::new(
        "common-explicit-event",
        Some(&module),
        &root(),
        "[Atomic]\nSW.RangeMaximum=1\n",
    );
    event.launcher_at(12, 12);
    let (owner, sw) = (event.id("Americans"), event.id("Atomic"));
    assert!(
        event
            .sim
            .super_weapon_target_check(&event.rules, owner, sw, (28, 28), false)
            .is_err()
    );
    // Actual SPECIAL_PLACE without UseAI does not repeat the cursor query.
    // The accepted event still reaches Activate's direct-fallback owner.
    assert!(event.fire("Atomic", (28, 28)));
    let payload = event
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap()
        .payload
        .unwrap();
    assert!(
        event
            .sim
            .projectiles
            .iter()
            .any(
                |(_, bullet)| bullet.payload().installed_weapon() == Some(payload)
                    && bullet.source_id() == 0
            )
    );
    assert_eq!(event.sim.super_weapons[&owner][&sw].ares_launch_count, 1);
    event.roundtrip();
}
