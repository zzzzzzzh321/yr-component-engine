//! Cold ordered mode/map Rules and the charged vertical silo chain. Native
//! addresses establish identity/ownership; no flight-timing golden is fitted.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;
use crate::rules::ruleset::WeaponHandle;
use crate::sim::snapshot::SnapshotRestoreError;

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";
const MODE: &str = "[SuperWeaponTypes]\n5=Atomic\n\
[Atomic]\nType=MultiMissile\nWeaponType=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
Nuke.Payload=FixturePayload\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\nMoney.Amount=-50\n\
[LAUNCHER]\nSuperWeapon=Atomic\nSuperWeapon2=none\nSuperWeapons=,\nNukeSilo=yes\n\
[Warheads]\n0=NukeMaker\n1=FixtureWH\n\
[ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=30\nRange=30\nSpeed=100\n\
Projectile=Rise\nWarhead=NukeMaker\nProjectileRange=2\n\
[NukePayload]\nDamage=11\nRange=30\nSpeed=10\nProjectile=Fall\nWarhead=FixtureWH\n\
[Rise]\nArm=2\nShadow=no\nAcceleration=1\nVertical=yes\nDetonationAltitude=20000\n\
[Fall]\nArm=2\nShadow=no\nAcceleration=1\nVertical=yes\nDetonationAltitude=30000\n\
[NukeMaker]\nNukeMaker=yes\n\
[FixtureWH]\nCellSpread=1.5\nPercentAtMax=1\n\
Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n";
const MAP: &str = "[WeaponTypes]\n0=FixturePayload\n\
[Atomic]\nWeaponType=ABCDEFGHIJKLMNOPQRSTUVWXY\nNuke.Payload=FixturePayload\n\
[ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=invalidh\nRange=30\nSpeed=100\n\
Projectile=Rise\nWarhead=NukeMaker\nProjectileRange=2\n\
[FixturePayload]\nDamage=75\nRange=30\nSpeed=10\nProjectile=Fall\nWarhead=FixtureWH\nProjectileRange=1\n";

fn bullet_with(sim: &Simulation, handle: WeaponHandle) -> Option<u64> {
    sim.projectiles.iter().find_map(|(&id, bullet)| {
        (bullet.payload().installed_weapon() == Some(handle)).then_some(id)
    })
}

fn advance_until(scene: &mut Scene, frames: u32, done: impl Fn(&Simulation) -> bool) {
    for _ in 0..frames {
        let commands = scene.sim.take_due_commands();
        scene
            .sim
            .advance_tick(&commands, Some(&scene.rules), None, None, 33);
        if done(&scene.sim) {
            return;
        }
    }
    let buildings: Vec<_> = scene
        .sim
        .entities()
        .values()
        .filter(|entity| entity.category() == crate::map::entities::EntityCategory::Structure)
        .map(|entity| {
            (
                entity.stable_id(),
                entity.mission,
                entity.building_body_state(),
                entity.queued_building_body_state(),
                entity.building_ready_latch(),
                entity
                    .mission_leaf
                    .as_building()
                    .map(|leaf| leaf.firing_super_weapon()),
            )
        })
        .collect();
    let bullets: Vec<_> = scene
        .sim
        .projectiles
        .iter()
        .map(|(&id, bullet)| {
            (
                id,
                bullet.payload().installed_weapon(),
                bullet.launching_super_weapon(),
                bullet.position(),
                bullet.fuel_remaining(),
            )
        })
        .collect();
    panic!(
        "Nuclear boundary not reached within {frames} frames: buildings={buildings:?}, bullets={bullets:?}"
    );
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.36) and VERA_SDK_NUCLEAR_PREVIOUS_WASM (.35)"]
fn sdk_nuclear_actual_carrier_payload_restore_and_receiver() {
    let current = component("VERA_SDK_NUCLEAR_WASM");
    let previous = component("VERA_SDK_NUCLEAR_PREVIOUS_WASM");
    // Installed1002B345 declares WeaponTypes before the SW sweep. The mode
    // lookup cannot allocate FixturePayload; its map declaration can bind it.
    let mode = Scene::with_mode("nuclear-mode", Some(&current), BASE, Some(MODE), "");
    let initialized = mode
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap();
    assert_eq!(initialized.payload, mode.rules.weapon_handle("NukePayload"));
    assert!(initialized.payload.is_some());
    assert!(mode.rules.weapon_handle("FixturePayload").is_none());
    let old = Scene::with_mode("nuclear-old", Some(&previous), BASE, Some(MODE), MAP);
    assert!(old.rules.super_weapon("Atomic").unwrap().nuclear.is_none());

    let mut scene = Scene::with_mode("nuclear-live", Some(&current), BASE, Some(MODE), MAP);
    let definition = scene.rules.super_weapon("Atomic").unwrap();
    let carrier = definition.weapon_type.unwrap();
    let nuclear = definition.nuclear.as_ref().unwrap();
    let (ordinal, payload) = (nuclear.source_ordinal, nuclear.payload.unwrap());
    assert_ne!(
        ordinal, 0,
        "Ares10073113 stores the actual SW definition slot"
    );
    assert_eq!(scene.rules.super_weapon_order[ordinal as usize], "Atomic");
    assert!(scene.rules.nuclear_super_weapon(ordinal).is_some());
    assert_eq!(Some(payload), scene.rules.weapon_handle("FixturePayload"));
    let first = scene.rules.weapon_handle(STORED).unwrap();
    assert_ne!(
        carrier, first,
        "map full25 allocates C1; short24 Find selects C0"
    );
    assert_eq!(scene.rules.weapon_by_handle(first).unwrap().damage, 30);
    assert_eq!(scene.rules.weapon_by_handle(carrier).unwrap().damage, 0);
    assert_eq!(scene.rules.weapon_by_handle(carrier).unwrap().id, STORED);

    // OPEN_PLAYFIELD is the display envelope, not MapClass's Size diamond.
    // A carrier born at the shared fixture's old (6,6) is correctly retired
    // by the ordinary Bullet tail before the test can observe its flight.
    let silo = scene.launcher_at(12, 12);
    assert!(scene.sim.map_cell_in_bounds((12, 12)));
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &scene.rules)
        .unwrap();
    let (owner, sw) = (scene.id("Americans"), scene.id("Atomic"));
    assert!(scene.sim.super_weapons[&owner][&sw].is_ready());
    assert!(scene.rules.object("LAUNCHER").unwrap().nuke_silo);
    assert_eq!(
        scene.sim.building_operational_state(silo, &scene.rules),
        Some(true),
        "the finished spawn adapter must supply an operational silo: {:?}",
        scene.sim.entities().get(silo).unwrap().mission
    );
    assert!(
        crate::sim::superweapon::targeting::launch_sites(
            &scene.sim,
            &scene.rules,
            owner,
            scene.rules.super_weapon("Atomic").unwrap()
        )
        .iter()
        .any(|(id, site)| *id == silo && site.eligible),
        "the original launch-site owner must admit this provider"
    );
    let carrier_type = scene.rules.weapon_by_handle(carrier).unwrap();
    assert_eq!(carrier_type.warhead.as_deref(), Some("NukeMaker"));
    assert!(
        scene
            .rules
            .projectile(carrier_type.projectile.as_deref().unwrap())
            .unwrap()
            .vertical
    );
    let health = scene.sim.entities().get(target).unwrap().health.current();
    assert!(
        scene.fire("Atomic", (20, 20)),
        "ordinary charged command accepts a silo"
    );
    assert_eq!(scene.sim.houses[&owner].economy().credits(), 1950);
    assert_eq!(scene.sim.super_weapons[&owner][&sw].ares_launch_count, 1);
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
        ordinal as i32,
        "accepted silo launch must retain its source before ordinary mission visits"
    );
    // The fixture supplies no SHP. Its zero-rate stock BuildingBody control
    // completes through ordinary Building AI; no ready byte is forced here.
    advance_until(&mut scene, 120, |sim| bullet_with(sim, carrier).is_some());
    let rising = bullet_with(&scene.sim, carrier).unwrap();
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
        ordinal as i32
    );
    let bullet = scene.sim.projectiles.get(rising).unwrap();
    assert_eq!(bullet.source_id(), silo);
    assert_eq!(
        bullet.payload().base_damage,
        0,
        "44CAFF installs C1, without a name lookup"
    );
    assert_eq!(bullet.launching_super_weapon(), Some(ordinal));
    assert!(
        bullet.fuel_remaining().unwrap() > i32::MAX / 2,
        "bare CreateBullet retains INT_MAX, not Weapon ProjectileRange=2"
    );
    let (position, fuel) = (bullet.position(), bullet.fuel_remaining());
    scene.roundtrip();
    let restored = scene.sim.projectiles.get(rising).unwrap();
    assert_eq!(
        (restored.position(), restored.fuel_remaining()),
        (position, fuel)
    );
    assert_eq!(restored.payload().installed_weapon(), Some(carrier));
    assert_eq!(restored.launching_super_weapon(), Some(ordinal));

    let bytes = GameSnapshot::save(
        &scene.sim,
        0,
        scene.rules.simulation_config_hash(),
        "NUCLEAR",
        0,
    );
    let mut bad = GameSnapshot::load(&bytes).unwrap().sim;
    bad.resolved_terrain = scene.sim.resolved_terrain.clone();
    bad.restore_after_snapshot_load().unwrap();
    bad.projectiles
        .set_launching_super_weapon(rising, Some(u32::MAX));
    assert!(
        matches!(bad.restore_map_authority_after_snapshot_load(&scene.rules, &crate::rules::overlay_types::OverlayTypeRegistry::empty(), None), Err(SnapshotRestoreError::InvalidProjectileSuperWeapon { projectile_id, super_ordinal: u32::MAX }) if projectile_id == rising)
    );

    // Real Bullet AI reaches NukeMaker46B310, retires its parent and constructs
    // the downward child before Fire; association propagates at10036958.
    advance_until(&mut scene, 600, |sim| sim.projectiles.get(rising).is_none());
    let falling =
        bullet_with(&scene.sim, payload).expect("NukeMaker selects the retained map payload");
    let bullet = scene.sim.projectiles.get(falling).unwrap();
    assert_eq!(bullet.launching_super_weapon(), Some(ordinal));
    assert_eq!(bullet.source_id(), silo);
    assert_eq!(bullet.payload().base_damage, 75);
    assert!(
        bullet.fuel_remaining().unwrap() > i32::MAX / 2,
        "falling Construct also keeps the bare budget"
    );
    advance_until(&mut scene, 3000, |sim| {
        sim.projectiles.get(falling).is_none()
    });
    assert!(
        scene.sim.entities().get(target).unwrap().health.current() < health,
        "the custom payload reaches the ordinary area receiver"
    );
    assert!(
        scene.sim.projectiles.iter().next().is_none(),
        "both stages retire through their ordinary owners"
    );
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.36)"]
fn sdk_nuclear_direct_overrides_restore_receiver_and_refused_payload() {
    let current = component("VERA_SDK_NUCLEAR_WASM");
    let mode = format!(
        "{}[DirectWH]\nCellSpread=1.5\nPercentAtMax=1\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
        MODE.replace(
            "Nuke.Payload=FixturePayload\n",
            "Nuke.Payload=FixturePayload\nNuke.SiloLaunch=no\nSW.Damage=90\nSW.Warhead=DirectWH\n",
        )
        .replace("1=FixtureWH\n", "1=FixtureWH\n2=DirectWH\n")
    );
    let map = MAP.replace(
        "Nuke.Payload=FixturePayload\n",
        "Nuke.Payload=FixturePayload\nSW.Damage=invalidh\nSW.Warhead=UnregisteredDirectWH\n",
    );
    let mut scene = Scene::with_mode("nuclear-direct", Some(&current), BASE, Some(&mode), &map);
    let settings = scene
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap();
    let (ordinal, payload) = (settings.source_ordinal, settings.payload.unwrap());
    assert_eq!(
        (settings.damage, settings.warhead.as_deref()),
        (90, Some("DirectWH"))
    );
    assert!(
        scene.rules.warhead("UnregisteredDirectWH").is_none(),
        "SW.Warhead10008B00 is FindOnly and unknown retains"
    );
    let silo = scene.launcher();
    let mission = scene.sim.entities().get(silo).unwrap().mission.current();
    let (owner, sw) = (scene.id("Americans"), scene.id("Atomic"));
    assert!(scene.sim.super_weapons[&owner][&sw].is_ready());
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &scene.rules)
        .unwrap();
    let health = scene.sim.entities().get(target).unwrap().health.current();
    assert!(scene.fire("Atomic", (20, 20)));
    assert_eq!(scene.sim.houses[&owner].economy().credits(), 1950);
    assert_eq!(scene.sim.super_weapons[&owner][&sw].ares_launch_count, 1);
    let direct =
        bullet_with(&scene.sim, payload).expect("SiloLaunch=no constructs the payload directly");
    let bullet = scene.sim.projectiles.get(direct).unwrap();
    assert_eq!((bullet.source_id(), bullet.firer_house()), (0, None));
    assert_eq!(bullet.launching_super_weapon(), Some(ordinal));
    assert_eq!(bullet.fuel_remaining(), Some(i32::MAX));
    assert_eq!(bullet.payload().base_damage, 90);
    assert_eq!(
        scene.sim.interner.resolve(bullet.payload().warhead),
        "DirectWH"
    );
    assert_eq!(
        bullet.velocity(),
        crate::sim::projectile::ProjectileVelocity::new(0, 0, -100)
    );
    assert_eq!(
        scene.sim.entities().get(silo).unwrap().mission.current(),
        mission
    );
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
    let position = bullet.position();
    scene.roundtrip();
    let bullet = scene.sim.projectiles.get(direct).unwrap();
    assert_eq!(
        (
            bullet.position(),
            bullet.fuel_remaining(),
            bullet.launching_super_weapon()
        ),
        (position, Some(i32::MAX), Some(ordinal))
    );
    advance_until(&mut scene, 3000, |sim| {
        sim.projectiles.get(direct).is_none()
    });
    assert!(scene.sim.entities().get(target).unwrap().health.current() < health);

    //10073145 also enters direct delivery when an ordinary provider is not
    // a NukeSilo. The nonvertical arm10073237 supplies its horizontal vector.
    let fallback_mode = mode
        .replace("Nuke.SiloLaunch=no", "Nuke.SiloLaunch=yes")
        .replace("SW.Damage=90", "SW.Damage=-1")
        .replace("NukeSilo=yes", "NukeSilo=no")
        .replace(
            "[Fall]\nArm=2\nShadow=no\nAcceleration=1\nVertical=yes",
            "[Fall]\nArm=2\nShadow=no\nAcceleration=1\nVertical=no",
        );
    let mut fallback = Scene::with_mode(
        "nuclear-no-silo",
        Some(&current),
        BASE,
        Some(&fallback_mode),
        &map,
    );
    fallback.launcher();
    assert!(fallback.fire("Atomic", (20, 20)));
    let payload = fallback
        .rules
        .super_weapon("Atomic")
        .unwrap()
        .nuclear
        .as_ref()
        .unwrap()
        .payload
        .unwrap();
    let bullet = fallback
        .sim
        .projectiles
        .get(bullet_with(&fallback.sim, payload).unwrap())
        .unwrap();
    assert_eq!(
        bullet.velocity(),
        crate::sim::projectile::ProjectileVelocity::new(0, 100, 0)
    );
    assert_eq!(bullet.source_id(), 0);
    assert_eq!(
        bullet.payload().base_damage,
        75,
        "negative SW.Damage falls back to actual payload damage"
    );

    // Unknown payload names retain; an explicit null BulletType is the
    // malformed payload rejected at10073159 before settlement10032FDF.
    let broken = map.replace(
        "Projectile=Fall\nWarhead=FixtureWH\nProjectileRange=1",
        "Projectile=none\nWarhead=FixtureWH\nProjectileRange=1",
    );
    let mut refused = Scene::with_mode(
        "nuclear-refused",
        Some(&current),
        BASE,
        Some(&mode),
        &broken,
    );
    refused.launcher();
    let (owner, sw) = (refused.id("Americans"), refused.id("Atomic"));
    let credits = refused.sim.houses[&owner].economy().credits();
    // ClickFire accepted the charged command and consumes its charge even
    // when Activate refuses construction. Only successful effects settle
    // money/history (the existing shared ClickFire/Launch owner).
    assert!(refused.fire("Atomic", (20, 20)));
    assert!(!refused.sim.super_weapons[&owner][&sw].is_ready());
    assert!(refused.sim.projectiles.iter().next().is_none());
    assert_eq!(refused.sim.houses[&owner].economy().credits(), credits);
    assert_eq!(refused.sim.super_weapons[&owner][&sw].ares_launch_count, 0);
}

#[test]
#[ignore = "requires VERA_SDK_NUCLEAR_WASM (.36)"]
fn sdk_nuclear_common_warhead_survives_intermediate_custom_type() {
    let module = component("VERA_SDK_NUCLEAR_WASM");
    // Common10033701 reads WH before the selected handler at10033ABD. A
    // physically reached custom-Type body must update this same pointer even
    // when the final map returns to the first mapped Nuclear handler.
    let root = format!(
        "{}\n[WeaponTypes]\n0=TransitionPayload\n\
         [Atomic]\nType=MultiMissile\nAction=Nuke\nNuke.Payload=TransitionPayload\n\
         Nuke.SiloLaunch=no\nIsPowered=no\nRechargeTime=1\nSW.InitialReady=yes\n\
         [TransitionPayload]\nDamage=75\nSpeed=100\nProjectile=TransitionFall\nWarhead=Payload\n\
         [TransitionFall]\nArm=2\nVertical=yes\nAcceleration=1\nDetonationAltitude=30000\n\
         [TransitionWH]\nCellSpread=1.5\nPercentAtMax=1\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
        BASE.replace("4=Warp\n", "4=Warp\n5=Atomic\n")
            .replace(
                "[Warheads]\n0=Payload\n",
                "[Warheads]\n0=Payload\n1=TransitionWH\n"
            )
            .replace("SuperWeapon=Strike\n", "SuperWeapon=Atomic\n")
            // A stale null override falls back to this muted payload WH.
            .replace(
                "Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%",
                "Verses=0%,0%,0%,0%,0%,0%,0%,0%,0%,0%,0%"
            )
    );
    let mut scene = Scene::with_mode(
        "nuclear-common-wh",
        Some(&module),
        &root,
        Some("[Atomic]\nType=UnitDelivery\nSW.Warhead=TransitionWH\nSW.Damage=90\n"),
        "[Atomic]\nType=ForceShield\n",
    );
    let definition = scene.rules.super_weapon("Atomic").unwrap();
    assert_eq!(definition.kind, SuperWeaponKind::ForceShield);
    let settings = definition.nuclear.as_ref().unwrap();
    assert_eq!(settings.warhead.as_deref(), Some("TransitionWH"));
    assert_eq!(settings.damage, 90);
    let payload = settings.payload.unwrap();
    scene.launcher();
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &scene.rules)
        .unwrap();
    let health = scene.sim.entities().get(target).unwrap().health.current();
    assert!(scene.fire("Atomic", (20, 20)));
    let bullet = bullet_with(&scene.sim, payload).unwrap();
    assert_eq!(
        scene
            .sim
            .interner
            .resolve(scene.sim.projectiles.get(bullet).unwrap().payload().warhead),
        "TransitionWH"
    );
    scene.roundtrip();
    advance_until(&mut scene, 600, |sim| sim.projectiles.get(bullet).is_none());
    assert!(scene.sim.entities().get(target).unwrap().health.current() < health);
}
