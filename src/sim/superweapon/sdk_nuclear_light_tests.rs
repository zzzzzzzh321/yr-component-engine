//! Actual .40 component, physical Rules reads and ordinary charged Bullet AI.
use super::*;
use crate::sim::light_sources::{LightingEvent, PaletteUpdate};
use crate::sim::radar::RadarEventType;

fn light_mode() -> String {
    let mode = MODE
        .replace("5=Atomic\n", "5=Atomic\n6=Silent\n7=Zero\n8=Maker\n")
        .replace(
            "SuperWeapon2=none\nSuperWeapons=,\n",
            "SuperWeapon2=Silent\nSuperWeapons=Zero,Maker\n",
        )
        .replace("1=FixtureWH\n", "1=FixtureWH\n2=SilentWH\n3=ZeroWH\n")
        .replace(
            "[Atomic]\n",
            "[Atomic]\nNuke.SiloLaunch=no\nLight.Ambient=100\nLight.Red=180\nLight.Green=0\nLight.Blue=-1\nSW.CreateRadarEvent=no\n",
        )
        .replace("[FixtureWH]\n", "[FixtureWH]\nNukeFlash.Duration=1000\n")
        .replace("[NukeMaker]\n", "[NukeMaker]\nNukeFlash.Duration=60\n")
        .replace("[NukePayload]\nDamage=11\nRange=30\nSpeed=10\n", "[NukePayload]\nDamage=11\nRange=30\nSpeed=100\n");
    format!(
        "{mode}\n[Silent]\nType=MultiMissile\nNuke.Payload=FixturePayload\nNuke.SiloLaunch=no\nSW.Warhead=SilentWH\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\nLight.Enabled=no\nSW.CreateRadarEvent=no\n\
         [Zero]\nType=MultiMissile\nNuke.Payload=FixturePayload\nNuke.SiloLaunch=no\nSW.Warhead=ZeroWH\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\nLight.Red=0\nLight.Green=230\nLight.Blue=70\n\
         [Maker]\nType=MultiMissile\nNuke.Payload=FixturePayload\nNuke.SiloLaunch=no\nSW.Warhead=NukeMaker\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\nLight.Enabled=no\n\
         [SilentWH]\nNukeFlash.Duration=30\n[ZeroWH]\nNukeFlash.Duration=0\n"
    )
}

fn ordinal(scene: &Scene, name: &str) -> u32 {
    scene
        .rules
        .super_weapon(name)
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap()
        .source_ordinal
}

fn launch(scene: &mut Scene, name: &str) -> u64 {
    let source = ordinal(scene, name);
    assert!(scene.fire(name, (20, 20)));
    scene
        .sim
        .projectiles
        .iter()
        .find_map(|(&id, bullet)| (bullet.launching_super_weapon() == Some(source)).then_some(id))
        .expect("ordinary direct launch retained its actual source")
}

fn await_source(scene: &mut Scene, name: &str) {
    let source = ordinal(scene, name);
    advance_until(scene, 3000, |sim| {
        sim.session.lighting().current_nuke_source() == Some(source)
    });
}

fn impact_radars(scene: &Scene) -> Vec<(u16, u16)> {
    scene
        .sim
        .sound_events
        .iter()
        .filter_map(|event| match event {
            SimSoundEvent::RadarEvent { radar }
                if radar.event_type == RadarEventType::ImpactSilent =>
            {
                Some((radar.rx, radar.ry))
            }
            _ => None,
        })
        .collect()
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.40)"]
fn sdk_nuclear_light_custom_preimpact_delays_damage_and_survives_restore() {
    let current = component("VERA_SDK_NUCLEAR_WASM");
    // The Anim is registered independently, without AnimList or SW.Animation.
    // The final immutable WH binding must extend the real asset closure.
    let root = format!("{BASE}\n[Animations]\n0=PREVIEW\n");
    let art = format!("{ART}\n[PREVIEW]\nRate=900\n");
    // Four self-authored empty SHP frames: physical bounds, not injected AI
    // lifetime or a rendering-equivalence fixture (same format as anim_damage_tests).
    let mut shp = vec![0u8; 8 + 4 * 24];
    shp[2..4].copy_from_slice(&1u16.to_le_bytes());
    shp[4..6].copy_from_slice(&1u16.to_le_bytes());
    shp[6..8].copy_from_slice(&4u16.to_le_bytes());
    let mode = light_mode().replace(
        "NukeFlash.Duration=1000\n",
        "NukeFlash.Duration=80\nPreImpactAnim=PREVIEW\n",
    );
    let map = format!(
        "{}\n[FixtureWH]\nNukeFlash.Duration=invalidh\nPreImpactAnim=UNKNOWN\n",
        MAP.replace(
            "[Atomic]\n",
            "[Atomic]\nLight.Red=invalidh\nSW.CreateRadarEvent=invalid\n"
        )
    );
    let mut scene = Scene::with_files(
        "sdk-nuclear-preimpact",
        Some(&current),
        &root,
        Some(&mode),
        &map,
        None,
        &[("artmd.ini", art.as_bytes()), ("PREVIEW.SHP", &shp)],
    );
    let source = ordinal(&scene, "Atomic");
    assert_eq!(
        scene.rules.super_weapon_light(source).unwrap().red,
        Some(180)
    );
    assert!(!scene.rules.super_weapon_create_radar_event(source));
    assert_eq!(
        scene
            .rules
            .art()
            .anim_runtime_config("PREVIEW")
            .unwrap()
            .raw_shp_frame_count,
        Some(4),
        "the real gameplay-resource closure bound this otherwise orphan image"
    );
    scene.launcher_at(12, 12);
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &scene.rules)
        .unwrap();
    let health = scene.sim.entities().get(target).unwrap().health.current();
    let bullet = launch(&mut scene, "Atomic");
    scene.sim.sound_events.clear_for_test();
    advance_until(&mut scene, 3000, |sim| {
        sim.projectiles
            .get(bullet)
            .is_some_and(|bullet| bullet.awaiting_anim())
    });
    let anim = scene
        .sim
        .projectiles
        .get(bullet)
        .unwrap()
        .awaited_anim()
        .expect("custom PreImpact constructed");
    assert_eq!(
        scene
            .sim
            .interner
            .resolve(scene.sim.anim(anim).unwrap().type_id()),
        "PREVIEW"
    );
    assert_eq!(
        scene.sim.entities().get(target).unwrap().health.current(),
        health,
        "467EB6 waits before ordinary area damage"
    );
    // Installed impact10036960 accepts this actual WH, not only literal NUKE.
    let lighting = *scene.sim.session.lighting();
    assert_eq!(lighting.current_nuke_source(), Some(source));
    assert_eq!(
        lighting.nuke_flash_for_test(),
        (1, scene.sim.session.binary_frame() as i32 - 1, 80)
    );
    assert_eq!(
        scene.sim.palette_constructor_rgb(&scene.rules),
        Some([1800, 0, 1250])
    );
    assert!(
        impact_radars(&scene).is_empty(),
        "exact SW.CreateRadarEvent=no retained after invalid map input"
    );
    scene.roundtrip();
    scene
        .sim
        .reinitialize_lighting_after_user_load(&scene.rules);
    assert_eq!(*scene.sim.session.lighting(), lighting);
    assert_eq!(
        scene.sim.palette_constructor_rgb(&scene.rules),
        Some([1800, 0, 1250])
    );
    assert_eq!(
        scene.sim.projectiles.get(bullet).unwrap().awaited_anim(),
        Some(anim)
    );
    advance_until(&mut scene, 120, |sim| sim.projectiles.get(bullet).is_none());
    assert!(scene.sim.entities().get(target).unwrap().health.current() < health);
    assert!(
        scene.sim.anim(anim).is_none(),
        "ordinary Anim retirement released the Bullet"
    );
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.40)"]
fn sdk_nuclear_light_disabled_and_zero_replace_source_without_restarting_flash() {
    let current = component("VERA_SDK_NUCLEAR_WASM");
    let mode = light_mode();
    let map = MAP.replace("Speed=10\n", "Speed=100\n");
    let mut scene = Scene::with_mode(
        "sdk-nuclear-overlap",
        Some(&current),
        BASE,
        Some(&mode),
        &map,
    );
    scene.launcher_at(12, 12);
    launch(&mut scene, "Atomic");
    await_source(&mut scene, "Atomic");
    let initial = *scene.sim.session.lighting();
    assert!(initial.nuke_flash_fading_in());
    assert_eq!(
        scene.sim.palette_constructor_rgb(&scene.rules),
        Some([1800, 0, 1250])
    );
    scene.sim.take_lighting_events();
    let disabled = launch(&mut scene, "Silent");
    await_source(&mut scene, "Silent");
    assert!(scene.sim.projectiles.get(disabled).is_none());
    assert_eq!(
        scene.sim.session.lighting().nuke_flash_for_test(),
        initial.nuke_flash_for_test()
    );
    assert_eq!(
        scene.sim.session.lighting().transition_timer(),
        initial.transition_timer()
    );
    assert_eq!(scene.sim.palette_constructor_rgb(&scene.rules), None);
    assert!(
        scene
            .sim
            .take_lighting_events()
            .iter()
            .all(|event| !matches!(event, LightingEvent::Global { .. })),
        "10036998 source assignment alone never globally recolors retained Converts"
    );

    let zero = launch(&mut scene, "Zero");
    await_source(&mut scene, "Zero");
    assert!(scene.sim.projectiles.get(zero).is_none());
    assert_eq!(
        scene.sim.session.lighting().nuke_flash_for_test(),
        initial.nuke_flash_for_test()
    );
    assert_eq!(
        scene.sim.palette_constructor_rgb(&scene.rules),
        Some([0, 2300, 700])
    );
    assert!(
        scene
            .sim
            .take_lighting_events()
            .iter()
            .all(|event| !matches!(event, LightingEvent::Global { .. }))
    );
    assert_eq!(
        impact_radars(&scene),
        [(20, 20)],
        "CreateRadarEvent defaults true for Zero; Atomic and Silent explicitly suppress it"
    );
    let retained = *scene.sim.session.lighting();
    scene.roundtrip();
    assert_eq!(*scene.sim.session.lighting(), retained);

    // NewSW globals1006DE80 saves CurrentNuke even with no running flash
    // and no surviving Bullet. A fresh zero-duration strike exercises that case.
    let mut quiet = Scene::with_mode(
        "sdk-nuclear-no-flash",
        Some(&current),
        BASE,
        Some(&mode),
        &map,
    );
    quiet.launcher_at(12, 12);
    let bullet = launch(&mut quiet, "Zero");
    await_source(&mut quiet, "Zero");
    assert!(quiet.sim.projectiles.get(bullet).is_none());
    assert!(!quiet.sim.session.lighting().nuke_flash_running());
    let source = ordinal(&quiet, "Zero");
    let hash = quiet.sim.state_hash();
    let alternate = ordinal(&quiet, "Atomic");
    quiet
        .sim
        .session
        .lighting_mut()
        .remember_nuke_source(alternate);
    assert_ne!(
        quiet.sim.state_hash(),
        hash,
        "saved source also participates in lockstep identity"
    );
    quiet
        .sim
        .session
        .lighting_mut()
        .remember_nuke_source(source);
    quiet.roundtrip();
    quiet
        .sim
        .reinitialize_lighting_after_user_load(&quiet.rules);
    assert_eq!(
        quiet.sim.session.lighting().current_nuke_source(),
        Some(source)
    );
    assert!(!quiet.sim.session.lighting().nuke_flash_running());
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.40) and VERA_SDK_NUCLEAR_LIGHT_PREVIOUS_WASM (.39)"]
fn sdk_nuclear_light_nukemaker_keeps_prior_source_but_passes_original_association() {
    let current = component("VERA_SDK_NUCLEAR_WASM");
    let mode = light_mode();
    let map = MAP.replace("Speed=10\n", "Speed=100\n");
    let mut scene = Scene::with_mode(
        "sdk-nuclear-maker-light",
        Some(&current),
        BASE,
        Some(&mode),
        &map,
    );
    scene.launcher_at(12, 12);
    launch(&mut scene, "Atomic");
    await_source(&mut scene, "Atomic");
    let prior = ordinal(&scene, "Atomic");
    let flash = scene.sim.session.lighting().nuke_flash_for_test();
    scene.sim.take_lighting_events();
    let maker = launch(&mut scene, "Maker");
    scene.sim.sound_events.clear_for_test();
    advance_until(&mut scene, 3000, |sim| sim.projectiles.get(maker).is_none());
    //1003698C skips source/Enabled/radar for NukeMaker; WH Duration still
    // starts the timer.10036A3E passes Maker, not retained CurrentNuke.
    let lighting = scene.sim.session.lighting();
    assert_eq!(lighting.current_nuke_source(), Some(prior));
    assert_eq!(
        lighting.nuke_flash_for_test(),
        (1, scene.sim.session.binary_frame() as i32 - 1, 60)
    );
    assert_ne!(lighting.nuke_flash_for_test(), flash);
    assert_eq!(
        scene.sim.palette_constructor_rgb(&scene.rules),
        Some([1800, 0, 1250])
    );
    assert!(impact_radars(&scene).is_empty());
    assert!(
        scene
            .sim
            .take_lighting_events()
            .iter()
            .all(|event| !matches!(event, LightingEvent::Global { .. })),
        "disabled explicit Maker declines ChangeLighting; using prior enabled CurrentNuke would wrongly publish Global"
    );

    let previous = component("VERA_SDK_NUCLEAR_LIGHT_PREVIOUS_WASM");
    let mut old = Scene::with_mode(
        "sdk-nuclear-light-historical",
        Some(&previous),
        BASE,
        Some(&mode),
        MAP,
    );
    assert!(!old.rules.superweapon_lighting_enabled());
    assert!(!old.rules.warhead_impact_enabled());
    old.roundtrip();
    assert!(!old.rules.superweapon_lighting_enabled());
    assert!(!old.rules.warhead_impact_enabled());
    assert_eq!(
        old.sim.lighting_palette(&old.rules),
        PaletteUpdate::RestoreNormal
    );
}
