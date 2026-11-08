//! Actual Primary pointers through physical mode/map reads and ordinary FireAt.
//! Scene supplies the arena and command delivery, not native whole-frame goldens.

use super::*;
use crate::sim::projectile::ProjectileTarget;
use crate::sim::snapshot::SnapshotRestoreError;

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

fn physical_passes() -> (&'static str, &'static str) {
    // FindOrAllocate772FA0 compares the complete25-byte input with stored24.
    // Mode Techno7129D5 creates W0; the later map Techno read creates W1.
    // WeaponReadINI772080 visits both; Ares10055872/88/9E invalid inputs
    // retain each actual receiver's own Range/ApplyDamage/Ammo history.
    // AmbientDamage uses native ReadInt instead: malformed decimal is zero;
    // invalidh reaches its failed hexadecimal scan and retains the current value.
    (
        "[LTNK]\nPrimary=ABCDEFGHIJKLMNOPQRSTUVWXY\nElitePrimary=none\n\
         Ammo=10\nInitialAmmo=10\nReload=10000\nCanPassiveAquire=no\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=100\nROF=10000\nRange=8\n\
         Speed=20\nProjectile=InstanceShot\nWarhead=InstanceWH\n\
         IsSonic=yes\nAmbientDamage=15\nOmniFire=yes\nAmmo=2\nApplyDamage=yes\nProjectileRange=4\n\
         [InstanceShot]\nImage=120MM\nArcing=yes\nInaccurate=no\n\
         Inviso=no\nROT=0\nRanged=yes\n\
         [InstanceWH]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
         CellSpread=1\nAnimList=TWLT070\n",
        "[MTNK]\nPrimary=ABCDEFGHIJKLMNOPQRSTUVWXY\nElitePrimary=none\n\
         Ammo=10\nInitialAmmo=10\nReload=10000\nCanPassiveAquire=no\n\
         [HTNK]\nPrimary=none\nSecondary=none\nElitePrimary=none\nEliteSecondary=none\n\
         Strength=2000\nSpeed=0\nCanPassiveAquire=no\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=100\nROF=10000\nRange=8\n\
         Speed=20\nProjectile=InstanceShot\nWarhead=InstanceWH\n\
         IsSonic=yes\nAmbientDamage=invalidh\nOmniFire=yes\nAmmo=invalid\nApplyDamage=invalid\nProjectileRange=invalid\n",
    )
}

/// Supply one recorded cell at the existing Wave receiver boundary, as the
/// lifecycle Wave controls do. Use the Wave actually created by FireAt and
/// real registered targets; do not reproduce Wave or ReceiveDamage arithmetic.
fn one_wave_cell(scene: &mut Scene, source: u64, target: u64, cell: (u16, u16)) -> i32 {
    let wave_id = scene
        .0
        .simulation
        .active_wave_id(source)
        .expect("real Sonic FireAt Wave");
    let before = scene.actor(target).health.current();
    let coord = crate::sim::movement::ground_pose::object_get_coords(
        scene.actor(target),
        scene.0.simulation.resolved_terrain.as_ref(),
    );
    scene.0.simulation.clear_lifecycle_test_events_for_test();
    scene.0.simulation.commit_logic_wave_damage_request(
        &scene.0.resources.rules,
        Some(&scene.0.resources.overlay_registry),
        &crate::sim::wave::WaveDamageRequest {
            wave_id,
            firer_id: source,
            recorded_cells: vec![crate::sim::wave::WaveRecordedCell::real(cell.0, cell.1)],
            wave_z: coord.z,
        },
        FrameEffects::default(),
    );
    assert!(scene.0.simulation.lifecycle_test_events_for_test().iter().any(|event| matches!(
        event, crate::sim::world::lifecycle::LifecycleTestEvent::WaveDamageReceiverSelected {
            wave_id: selected_wave, target_id, ..
        } if *selected_wave == wave_id && *target_id == target
    )));
    before - scene.actor(target).health.current()
}

#[test]
#[cfg(not(feature = "ares"))]
#[ignore = "requires retail assets and VERA_SDK_WEAPON_INSTANCE_WASM"]
fn sdk_weapon_instance_primary_flight_restore_and_receiver_keep_actual_slots() {
    let component =
        std::fs::read(std::env::var_os("VERA_SDK_WEAPON_INSTANCE_WASM").unwrap()).unwrap();
    let (mode, map) = physical_passes();
    let mut scene = Scene::with_component_passes(mode, map, Some(&component)).unwrap();
    let rules = &scene.0.resources.rules;
    let handle = |name| {
        crate::sim::combat::combat_weapon::weapon_for_index(rules.object(name).unwrap(), 0, 0)
            .unwrap()
            .handle(rules)
            .unwrap()
    };
    let (first, second) = (handle("LTNK"), handle("MTNK"));
    assert_ne!(
        first, second,
        "two real Techno reads retain distinct Weapon pointers"
    );
    assert_eq!(
        rules.weapon_handle(STORED),
        Some(first),
        "short-name Find returns W0"
    );
    for (slot, cost, applies, budget) in [(first, 2, true, 1024), (second, 1, false, 100_000)] {
        let weapon = rules.weapon_by_handle(slot).unwrap();
        assert_eq!(weapon.id, STORED);
        assert_eq!(weapon.ambient_damage, if slot == first { 15 } else { 0 });
        assert_eq!(
            weapon
                .resource_policy(vera_mod_api::WEAPON_USER_UNIT)
                .unwrap()
                .cost,
            cost
        );
        let firing = weapon.firing_policy().unwrap();
        assert_eq!(
            (firing.apply_damage, firing.projectile_range),
            (applies, Some(budget))
        );
    }
    let target_a = scene.spawn("HTNK", "Russians", (6, 19));
    let target_b = scene.spawn("HTNK", "Russians", (12, 19));
    let source_a = scene.spawn("LTNK", "Americans", (6, 22));
    let source_b = scene.spawn("MTNK", "Americans", (12, 22));
    // The passive-scan consumer must keep the same actual W1 as FireAt;
    // equal names and core Damage cannot prove the selected instance.
    let scan_weapon = crate::sim::world::techno_ai::target_scan::weapon_at_index(
        &scene.0.simulation,
        &scene.0.resources.rules,
        source_b,
        0,
    )
    .unwrap();
    assert!(std::ptr::eq(
        scan_weapon,
        scene.0.resources.rules.weapon_by_handle(second).unwrap()
    ));
    assert!(!scan_weapon.firing_policy().unwrap().apply_damage);
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source_b,
            target_id: target_b,
        },
    );
    scene.shot_event(source_b, target_b);
    assert_eq!(scene.count(source_b), 9, "W1 debits its own default Ammo=1");
    let (&bullet_id, bullet) = scene
        .0
        .simulation
        .projectiles()
        .iter()
        .find(|(_, bullet)| bullet.source_id() == source_b)
        .unwrap();
    let payload = bullet.payload();
    assert_eq!(payload.installed_weapon(), Some(second));
    assert_eq!(
        payload.base_damage, 0,
        "W1 does not borrow W0's ApplyDamage=yes"
    );
    let position = bullet.position();
    let fuel = bullet.fuel_remaining();
    assert!(fuel.is_some_and(|value| value > 99_000));
    let second_wave_damage = one_wave_cell(&mut scene, source_b, target_b, (12, 19));
    let rules_hash = scene.0.resources.rules.simulation_config_hash();
    let saved =
        GameSnapshot::save_validated(&scene.0.simulation, 0, rules_hash, "Weapon W1 in flight", 0);

    // Corrupt the serialized identity, then use the real rules-backed load
    // admission; an arbitrary u32 is never trusted as a valid Weapon slot.
    let mut invalid =
        GameSnapshot::load_validated(&saved, 0, rules_hash, scene.0.simulation.session.map_name())
            .unwrap()
            .sim;
    let invalid_slot = serde_json::from_value(serde_json::json!(u32::MAX)).unwrap();
    invalid
        .projectiles
        .get_mut(bullet_id)
        .unwrap()
        .set_payload_for_test(payload.with_installed_weapon(Some(invalid_slot)));
    let bad_bytes = GameSnapshot::save_validated(&invalid, 0, rules_hash, "invalid Weapon slot", 0);
    let mut rejected = GameSnapshot::load_validated(
        &bad_bytes,
        0,
        rules_hash,
        scene.0.simulation.session.map_name(),
    )
    .unwrap()
    .sim;
    assert!(matches!(rejected.restore_map_authority_after_snapshot_load(
        &scene.0.resources.rules, &scene.0.resources.overlay_registry, None,
    ), Err(SnapshotRestoreError::InvalidProjectileWeapon { projectile_id, weapon_ordinal: u32::MAX }) if projectile_id == bullet_id));

    let restored = scene.restored(&saved);
    scene.0.resources.rules.external_mod =
        Some(std::sync::Arc::new(restored.external_mod.clone().unwrap()));
    scene.0.resources.rules.bind_resource_policies().unwrap();
    assert_eq!(scene.0.resources.rules.simulation_config_hash(), rules_hash);
    scene.0.replace_simulation(restored);
    let bullet = scene.0.simulation.projectiles().get(bullet_id).unwrap();
    assert_eq!(
        (bullet.payload(), bullet.position(), bullet.fuel_remaining()),
        (payload, position, fuel)
    );
    let baseline = scene.0.simulation.state_hash();
    scene
        .0
        .simulation
        .projectiles
        .get_mut(bullet_id)
        .unwrap()
        .set_payload_for_test(payload.with_installed_weapon(Some(first)));
    assert_ne!(
        scene.0.simulation.state_hash(),
        baseline,
        "same stored name, different actual Weapon identity"
    );
    scene
        .0
        .simulation
        .projectiles
        .get_mut(bullet_id)
        .unwrap()
        .set_payload_for_test(payload);

    let health = scene.actor(target_a).health.current();
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source_a,
            target_id: target_a,
        },
    );
    let mut first_fired = false;
    let mut first_wave_damage = None;
    let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
        for _ in 0..180 {
            let output = scene.frame();
            if output
                .fire_events
                .iter()
                .any(|shot| shot.attacker_id == source_a)
            {
                assert!(!first_fired);
                first_fired = true;
                assert_eq!(scene.count(source_a), 8, "W0 retains its Ammo=2 history");
                let (_, bullet) = scene
                    .0
                    .simulation
                    .projectiles()
                    .iter()
                    .find(|(_, bullet)| bullet.source_id() == source_a)
                    .unwrap();
                assert_eq!(bullet.payload().installed_weapon(), Some(first));
                assert_eq!(bullet.payload().base_damage, 100);
                assert_eq!(bullet.target(), ProjectileTarget::Entity(target_a));
                assert!(
                    bullet
                        .fuel_remaining()
                        .is_some_and(|fuel| (0..=1024).contains(&fuel))
                );
                first_wave_damage = Some(one_wave_cell(&mut scene, source_a, target_a, (6, 19)));
            }
            if first_fired
                && scene
                    .0
                    .simulation
                    .projectiles()
                    .iter()
                    .all(|(_, bullet)| ![source_a, source_b].contains(&bullet.source_id()))
            {
                return;
            }
        }
        panic!("ordinary instance-backed Bullets never completed their receiver and retirement");
    });
    assert!(first_fired);
    let areas: Vec<_> = visits
        .iter()
        .filter(|visit| visit["warhead"] == "InstanceWH")
        .collect();
    assert_eq!(areas.len(), 2);
    assert!(
        areas
            .iter()
            .any(|visit| visit["source_id"] == source_a && visit["damage"] == 100)
    );
    assert!(
        areas
            .iter()
            .any(|visit| visit["source_id"] == source_b && visit["damage"] == 0)
    );
    assert!(scene.actor(target_a).health.current() < health);
    // ApplyDamage selects only the Bullet payload. Sonic's independent Wave
    // may damage B even with AmbientDamage=0 (direct ReceiveDamage minimum).
    // Compare isolated real Wave receiver commits instead of total target HP
    // or a second implementation of the shared damage calculation.
    assert!(
        first_wave_damage.unwrap() > second_wave_damage,
        "W1's Wave must use its own zero AmbientDamage, not W0's retained15"
    );
}
