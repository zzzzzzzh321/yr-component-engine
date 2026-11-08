//! Formal FireAt followed by a supplied parent position/fuel-expiry boundary.
//! Real Shrapnel rings, random fill, child Fire, flight and receivers follow.

use super::*;
use crate::sim::projectile::{ProjectileCoord, ProjectileTarget};

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

fn physical_passes(short_alias: bool, flak: bool) -> (&'static str, String) {
    // Weapon772FA0 compares the full25 input with stored24. The Techno
    // read creates C0 before the Weapon sweep, BulletReadINI46C305 creates
    // C1 afterwards. Failed map reads keep each receiver's own defaults.
    (
        "[LTNK]\nPrimary=ABCDEFGHIJKLMNOPQRSTUVWXY\nElitePrimary=none\n\
         [MTNK]\nPrimary=ShrapnelParent\nElitePrimary=none\n\
         Ammo=1\nInitialAmmo=1\nReload=10000\nCanPassiveAquire=no\n\
         [ShrapnelParent]\nDamage=0\nROF=10000\nRange=8\nSpeed=20\n\
         Projectile=ShrapnelParentShot\nWarhead=ShrapnelParentWH\nOmniFire=yes\n\
         [ShrapnelParentShot]\nShrapnelWeapon=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
         ShrapnelCount=2\nROT=1\nRanged=yes\nInviso=no\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=30\nBright=yes\nRadLevel=240\n\
         Range=2\nSpeed=20\nProjectile=ShrapnelChildShot\nWarhead=ShrapnelChildWH\nProjectileRange=2\n\
         [ShrapnelChildShot]\nImage=120MM\nROT=0\nRanged=yes\nInviso=no\nFlakScatter=no\nArm=0\n\
         [ShrapnelParentWH]\nCellSpread=0\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
         [ShrapnelChildWH]\nCellSpread=3\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
        format!(
            "[HTNK]\nPrimary=none\nSecondary=none\nElitePrimary=none\nEliteSecondary=none\n\
             Strength=10000\nSpeed=0\nCanPassiveAquire=no\n\
             [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=invalidh\nBright=invalid\nRadLevel=invalidh\n\
             Range=2\nSpeed=20\nProjectile=ShrapnelChildShot\nWarhead=ShrapnelChildWH\nProjectileRange=invalid\n{}{}{}",
            if short_alias || flak {
                "[ShrapnelParentShot]\n"
            } else {
                ""
            },
            if short_alias {
                "ShrapnelWeapon=ABCDEFGHIJKLMNOPQRSTUVWX\n"
            } else {
                ""
            },
            if flak {
                "ShrapnelCount=1\n\
                 [ShrapnelChildShot]\nInviso=yes\nFlakScatter=yes\nInaccurate=yes\n"
            } else {
                ""
            }
        ),
    )
}

fn fire_parent(scene: &mut Scene) -> (u64, u64, u64) {
    let target = scene.spawn("HTNK", "Russians", (12, 19));
    // First sweep offset (1,-1) selects this enemy.
    let neighbor = scene.spawn("HTNK", "Russians", (13, 18));
    let source = scene.spawn("MTNK", "Americans", (12, 22));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source,
            target_id: target,
        },
    );
    let (_, shot) = scene.shot_event(source, target);
    assert_eq!(
        scene.0.simulation.interner.resolve(shot.weapon_id),
        "ShrapnelParent"
    );
    assert_eq!(scene.count(source), 0);
    let parent = *scene
        .0
        .simulation
        .projectiles()
        .iter()
        .find(|(_, bullet)| bullet.source_id() == source)
        .expect("ordinary FireAt constructs a live parent")
        .0;
    // Supply the represented parent's detonation entry, as in Airburst's
    // live fixture. The real next Logic visit owns spawning and retirement.
    scene
        .0
        .simulation
        .projectiles
        .get_mut(parent)
        .unwrap()
        .set_position_for_test(ProjectileCoord::new(12 * 256 + 128, 19 * 256 + 128, 600));
    scene.0.simulation.projectiles.set_weapon_fuel(parent, 0);
    (source, parent, neighbor)
}

#[test]
#[cfg(not(any(feature = "ares", feature = "phobos")))]
#[ignore = "requires retail assets, VERA_SDK_SHRAPNEL_INSTANCE_WASM and VERA_SDK_SHRAPNEL_PREVIOUS_WASM"]
fn sdk_shrapnel_actual_weapon_survives_both_branches_flight_restore_and_radiation() {
    let current =
        std::fs::read(std::env::var_os("VERA_SDK_SHRAPNEL_INSTANCE_WASM").unwrap()).unwrap();
    let previous =
        std::fs::read(std::env::var_os("VERA_SDK_SHRAPNEL_PREVIOUS_WASM").unwrap()).unwrap();
    let mut current_hashes = Vec::new();
    for (component, short_alias, selector) in [
        (current.as_slice(), false, Some(true)),
        (current.as_slice(), true, Some(true)),
        (previous.as_slice(), true, None),
    ] {
        let (mode, map) = physical_passes(short_alias, false);
        let mut scene = Scene::with_component_passes(mode, &map, Some(component)).unwrap();
        let rules = &scene.0.resources.rules;
        assert_eq!(
            rules
                .match_rules_policy()
                .unwrap()
                .shrapnel_weapon_assignment,
            selector
        );
        let first = rules.weapon_handle(STORED).unwrap();
        let selected = rules
            .projectile("ShrapnelParentShot")
            .unwrap()
            .shrapnel_weapon
            .unwrap();
        assert_eq!(selected == first, short_alias);
        let instances = rules
            .weapons_iter()
            .filter(|weapon| weapon.id == STORED)
            .collect::<Vec<_>>();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            (
                instances[0].damage,
                instances[0].bright,
                instances[0].rad_level
            ),
            (30, true, 240)
        );
        assert_eq!(
            (
                instances[1].damage,
                instances[1].bright,
                instances[1].rad_level
            ),
            (0, false, 0)
        );
        assert!(std::ptr::eq(
            rules.weapon_by_handle(selected).unwrap(),
            instances[usize::from(!short_alias)]
        ));
        let hash = rules.simulation_config_hash();
        if selector.is_some() {
            current_hashes.push(hash);
        }
        let (source, parent, neighbor) = fire_parent(&mut scene);
        let health = scene.actor(neighbor).health.current();
        let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
            scene.frame();
            assert!(scene.0.simulation.projectiles().get(parent).is_none());
            let children = scene
                .0
                .simulation
                .projectiles()
                .iter()
                .filter(|(_, bullet)| bullet.source_id() == source)
                .collect::<Vec<_>>();
            assert_eq!(children.len(), 2);
            assert!(
                children
                    .iter()
                    .any(|(_, child)| child.target() == ProjectileTarget::Entity(neighbor))
            );
            assert!(
                children
                    .iter()
                    .any(|(_, child)| matches!(child.target(), ProjectileTarget::Cell { .. }))
            );
            let flights = children
                .iter()
                .map(|(id, child)| {
                    let id = **id;
                    assert_eq!(
                        child.payload().installed_weapon(),
                        selector.map(|_| selected)
                    );
                    assert_eq!(
                        child.payload().base_damage,
                        if short_alias { 30 } else { 0 }
                    );
                    assert_eq!(child.payload().bright(), short_alias);
                    // 1001ABF0 installs +130, without the weapon-aware factory's
                    // Ext+C8 copy. Bare Construct1001A920 retains INT_MAX fuel.
                    assert!(
                        child
                            .fuel_remaining()
                            .is_some_and(|left| left > i32::MAX - 4096)
                    );
                    assert!(child.position().z > 0);
                    (
                        id,
                        child.payload(),
                        child.position(),
                        child.velocity(),
                        child.fuel_remaining(),
                    )
                })
                .collect::<Vec<_>>();
            let saved = GameSnapshot::save_validated(
                &scene.0.simulation,
                0,
                hash,
                "Shrapnel actual Weapon",
                0,
            );
            let restored = scene.restored(&saved);
            scene.0.resources.rules.external_mod =
                Some(std::sync::Arc::new(restored.external_mod.clone().unwrap()));
            scene.0.resources.rules.bind_resource_policies().unwrap();
            assert_eq!(scene.0.resources.rules.simulation_config_hash(), hash);
            assert_eq!(
                scene
                    .0
                    .resources
                    .rules
                    .match_rules_policy()
                    .unwrap()
                    .shrapnel_weapon_assignment,
                selector
            );
            scene.0.replace_simulation(restored);
            for &(id, payload, position, velocity, fuel) in &flights {
                let child = scene.0.simulation.projectiles().get(id).unwrap();
                assert_eq!(
                    (
                        child.payload(),
                        child.position(),
                        child.velocity(),
                        child.fuel_remaining()
                    ),
                    (payload, position, velocity, fuel)
                );
            }
            for _ in 0..120 {
                scene.frame();
                if flights
                    .iter()
                    .all(|(id, ..)| scene.0.simulation.projectiles().get(*id).is_none())
                {
                    return;
                }
            }
            panic!("restored Shrapnel children never completed their ordinary receivers");
        });
        let impacts = visits
            .iter()
            .filter(|visit| visit["warhead"] == "ShrapnelChildWH")
            .collect::<Vec<_>>();
        assert_eq!(
            impacts.len(),
            2,
            "one receiver for each real child, including after restore"
        );
        assert!(impacts.iter().all(|visit| visit["source_id"] == source
            && visit["damage"] == if short_alias { 30 } else { 0 }));
        let irradiates = short_alias && selector == Some(true);
        assert_eq!(
            scene.0.simulation.radiation.sites().next().is_some(),
            irradiates
        );
        if irradiates {
            assert!(
                scene
                    .0
                    .simulation
                    .radiation
                    .sites()
                    .all(|site| site.level() >= 240)
            );
            assert!(scene.actor(neighbor).health.current() < health);
        }
    }
    assert_ne!(
        current_hashes[0], current_hashes[1],
        "actual Shrapnel pointer contributes to rules identity"
    );
}

#[test]
#[cfg(not(any(feature = "ares", feature = "phobos")))]
#[ignore = "requires retail assets and VERA_SDK_SHRAPNEL_INSTANCE_WASM"]
fn sdk_shrapnel_inviso_flak_fire_uses_installed_weapon_and_shared_rng_owner() {
    let component =
        std::fs::read(std::env::var_os("VERA_SDK_SHRAPNEL_INSTANCE_WASM").unwrap()).unwrap();
    let (mode, map) = physical_passes(true, true);
    let mut scene = Scene::with_component_passes(mode, &map, Some(&component)).unwrap();
    assert_eq!(
        scene
            .0
            .resources
            .rules
            .projectile("ShrapnelParentShot")
            .unwrap()
            .shrapnel_count,
        1
    );
    let child_type = scene
        .0
        .resources
        .rules
        .projectile("ShrapnelChildShot")
        .unwrap();
    assert!(child_type.inviso && child_type.flak_scatter && child_type.inaccurate);
    assert_eq!(
        scene
            .0
            .resources
            .rules
            .match_rules_policy()
            .unwrap()
            .shrapnel_weapon_assignment,
        Some(true)
    );
    let (source, parent, neighbor) = fire_parent(&mut scene);
    let target = crate::sim::movement::ground_pose::object_get_coords(
        scene.actor(neighbor),
        scene.0.simulation.resolved_terrain.as_ref(),
    );
    let target = ProjectileCoord::new(target.x, target.y, target.z);
    let selected = scene
        .0
        .resources
        .rules
        .projectile("ShrapnelParentShot")
        .unwrap()
        .shrapnel_weapon
        .unwrap();
    assert_eq!(
        Some(selected),
        scene.0.resources.rules.weapon_handle(STORED)
    );
    let range = scene
        .0
        .resources
        .rules
        .weapon_by_handle(selected)
        .unwrap()
        .range_leptons;
    let mut rng = scene.0.simulation.scenario_rng.clone();
    let ((_, draws), visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
        crate::sim::rng::trace_draws(|| {
            // The live Logic cursor can leave an appended child for the next
            // frame. Observe its real receiver without prescribing that slot.
            for _ in 0..2 {
                scene.frame();
                if !scene
                    .0
                    .simulation
                    .projectiles()
                    .iter()
                    .any(|(_, bullet)| bullet.source_id() == source)
                {
                    break;
                }
            }
        })
    });
    // The elevated parent's impact ladder preserves its live Location, also
    // used by SpawnShrapnel. Observe that real post-motion origin through the
    // parent receiver; this does not assume its supplied entry stayed still.
    let parent_visit = visits
        .iter()
        .find(|visit| visit["warhead"] == "ShrapnelParentWH")
        .unwrap();
    let axis = |index: usize| parent_visit["coord"][index].as_i64().unwrap() as i32;
    let origin = ProjectileCoord::new(axis(0), axis(1), axis(2));
    let impacts = visits
        .iter()
        .filter(|visit| visit["warhead"] == "ShrapnelChildWH")
        .collect::<Vec<_>>();
    assert_eq!(
        impacts.len(),
        1,
        "the single real child reaches one ordinary receiver"
    );
    let flak = draws
        .iter()
        .filter(|draw| {
            draw["callers"]
                .as_str()
                .unwrap()
                .contains("bullet_fire_flak_placement")
        })
        .collect::<Vec<_>>();
    assert!(
        !flak.is_empty(),
        "child Fire must reach the existing flak owner"
    );
    assert!(flak.iter().all(|draw| draw["logic_object"] == parent));
    // Other frame activity can draw Main as well. Seek only this Scenario
    // cursor; the bounded requests can consume rejected raw words too.
    for _ in 0..250 {
        let state = rng.logical_state();
        if serde_json::json!([state.index_a, state.index_b]) == flak[0]["before_indices"] {
            break;
        }
        rng.next_u32();
    }
    let state = rng.logical_state();
    assert_eq!(
        serde_json::json!([state.index_a, state.index_b]),
        flak[0]["before_indices"]
    );
    // Reuse the existing native-calibrated numeric/RNG owner. This checks
    // caller wiring; launch.rs keeps its original instruction goldens.
    let (expected, replayed) = crate::sim::rng::trace_draws(|| {
        crate::sim::projectile::launch::bullet_fire_flak_placement(
            origin,
            target,
            scene.0.resources.rules.combat_damage.ballistic_scatter,
            range,
            &mut rng,
        )
    });
    assert_eq!(
        replayed
            .iter()
            .map(|draw| &draw["value"])
            .collect::<Vec<_>>(),
        flak.iter().map(|draw| &draw["value"]).collect::<Vec<_>>(),
        "child Fire keeps the shared magnitude/angle requests and rejection order"
    );
    assert_ne!(
        expected, target,
        "nonzero flak scatter changes the actual placement"
    );
    assert!(scene.0.simulation.projectiles().get(parent).is_none());
    assert!(
        !scene
            .0
            .simulation
            .projectiles()
            .iter()
            .any(|(_, child)| child.source_id() == source)
    );
    assert_eq!(
        impacts[0]["coord"],
        serde_json::json!([expected.x, expected.y, expected.z])
    );
    assert_eq!(impacts[0]["damage"], 30);
    assert_eq!(impacts[0]["source_id"], source);
    assert!(
        scene
            .0
            .simulation
            .radiation
            .sites()
            .any(|site| site.level() == 240)
    );
}
