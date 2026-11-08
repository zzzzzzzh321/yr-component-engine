//! Real FireAt followed by a supplied parent detonation-entry boundary.
//! Children use the ordinary fan, live flight, save and AreaDamage owners.

use super::*;
use crate::sim::projectile::ProjectileCoord;

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

fn physical_passes(short_alias: bool) -> (&'static str, String) {
    // Weapon772FA0 compares full25 with stored24. LTNK creates C0 before
    // the Weapon sweep; BulletReadINI46C2C6 creates C1 afterwards. The map
    // gives the late instance valid Projectile/WH while failed reads retain
    // each object's own Damage/Bright/Ares10055872 range history.
    (
        "[LTNK]\nPrimary=ABCDEFGHIJKLMNOPQRSTUVWXY\nElitePrimary=none\n\
         [MTNK]\nPrimary=IdentityParentGun\nElitePrimary=none\n\
         Ammo=1\nInitialAmmo=1\nReload=10000\nCanPassiveAquire=no\n\
         [IdentityParentGun]\nDamage=0\nROF=10000\nRange=8\nSpeed=20\n\
         Projectile=IdentityParentShot\nWarhead=IdentityParentWH\nOmniFire=yes\n\
         [IdentityParentShot]\nAirburst=yes\nAirburstSpread=0\nCluster=1\n\
         AirburstWeapon=ABCDEFGHIJKLMNOPQRSTUVWXY\nROT=1\nRanged=yes\nInviso=no\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=30\nBright=yes\nRange=8\nSpeed=20\n\
         Projectile=IdentityChildShot\nWarhead=IdentityChildWH\nProjectileRange=2\n\
         [IdentityChildShot]\nROT=1\nRanged=yes\nInviso=no\nFlakScatter=no\nArm=0\n\
         [IdentityParentWH]\nCellSpread=0\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
         [IdentityChildWH]\nCellSpread=3\n\
         Verses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
        format!(
            "[HTNK]\nPrimary=none\nSecondary=none\nElitePrimary=none\nEliteSecondary=none\n\
             Strength=10000\nSpeed=0\nCanPassiveAquire=no\n\
             [ABCDEFGHIJKLMNOPQRSTUVWX]\nDamage=invalidh\nBright=invalid\nRange=8\nSpeed=20\n\
             Projectile=IdentityChildShot\nWarhead=IdentityChildWH\nProjectileRange=invalid\n{}",
            if short_alias {
                "[IdentityParentShot]\nAirburstWeapon=ABCDEFGHIJKLMNOPQRSTUVWX\n"
            } else {
                ""
            }
        ),
    )
}

#[test]
#[cfg(not(any(feature = "ares", feature = "phobos")))]
#[ignore = "requires retail assets and VERA_SDK_AIRBURST_INSTANCE_WASM"]
fn sdk_airburst_child_keeps_actual_weapon_through_factory_flight_and_restore() {
    let component =
        std::fs::read(std::env::var_os("VERA_SDK_AIRBURST_INSTANCE_WASM").unwrap()).unwrap();
    let mut hashes = Vec::new();
    for (short_alias, damage, bright, budget) in [(false, 0, false, 100_000), (true, 30, true, 512)]
    {
        let (mode, map) = physical_passes(short_alias);
        let mut scene = Scene::with_component_passes(mode, &map, Some(&component)).unwrap();
        let rules = &scene.0.resources.rules;
        let instances = rules
            .weapons_iter()
            .filter(|weapon| weapon.id == STORED)
            .collect::<Vec<_>>();
        assert_eq!(instances.len(), 2);
        assert_eq!((instances[0].damage, instances[0].bright), (30, true));
        assert_eq!((instances[1].damage, instances[1].bright), (0, false));
        assert_eq!(
            instances
                .iter()
                .map(|weapon| weapon.firing_policy().unwrap().projectile_range.unwrap())
                .collect::<Vec<_>>(),
            [512, 100_000]
        );
        let hash = rules.simulation_config_hash();
        hashes.push(hash);
        let target = scene.spawn("HTNK", "Russians", (12, 19));
        let source = scene.spawn("MTNK", "Americans", (12, 22));
        let health = scene.actor(target).health.current();
        scene.command(
            "Americans",
            Command::Attack {
                attacker_id: source,
                target_id: target,
            },
        );
        scene.shot_event(source, target);
        let (&parent_id, _) = scene
            .0
            .simulation
            .projectiles()
            .iter()
            .find(|(_, bullet)| bullet.source_id() == source)
            .expect("ordinary FireAt constructs the real parent");
        // Supply the already represented parent's entry state at the existing
        // fuel-expiry seam. The next real frame owns detonation and retirement;
        // no manually constructed child or synthetic detonation packet follows.
        scene
            .0
            .simulation
            .projectiles
            .get_mut(parent_id)
            .unwrap()
            .set_position_for_test(ProjectileCoord::new(12 * 256 + 128, 19 * 256 + 128, 600));
        scene.0.simulation.projectiles.set_weapon_fuel(parent_id, 0);
        let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
            scene.frame();
            assert!(scene.0.simulation.projectiles().get(parent_id).is_none());
            let children = scene
                .0
                .simulation
                .projectiles()
                .iter()
                .filter(|(_, bullet)| bullet.source_id() == source)
                .collect::<Vec<_>>();
            assert_eq!(children.len(), 1, "Spread=0 emits the center child");
            let (&child_id, child) = children[0];
            let payload = child.payload();
            let position = child.position();
            let velocity = child.velocity();
            let fuel = child.fuel_remaining();
            assert_eq!(payload.base_damage, damage);
            assert_eq!(payload.bright(), bright);
            // Ares10019210 reads the actual child Weapon but never calls
            // SetWeaponType46B260: Bullet ctor466401's +130 stays null.
            assert!(payload.installed_weapon().is_none());
            assert_eq!(
                scene
                    .0
                    .simulation
                    .interner
                    .resolve(payload.projectile_type().unwrap()),
                "IdentityChildShot"
            );
            assert!(position.z > 0, "child is still in flight before save");
            assert!(fuel.is_some_and(|left| left > budget - 256 && left <= budget));
            let saved = GameSnapshot::save_validated(
                &scene.0.simulation,
                0,
                hash,
                "Airburst actual child Weapon",
                0,
            );
            let restored = scene.restored(&saved);
            scene.0.resources.rules.external_mod =
                Some(std::sync::Arc::new(restored.external_mod.clone().unwrap()));
            scene.0.resources.rules.bind_resource_policies().unwrap();
            assert_eq!(scene.0.resources.rules.simulation_config_hash(), hash);
            scene.0.replace_simulation(restored);
            let child = scene.0.simulation.projectiles().get(child_id).unwrap();
            assert_eq!(
                (
                    child.payload(),
                    child.position(),
                    child.velocity(),
                    child.fuel_remaining()
                ),
                (payload, position, velocity, fuel)
            );
            scene.frame();
            assert!(
                scene
                    .0
                    .simulation
                    .projectiles()
                    .get(child_id)
                    .and_then(|child| child.fuel_remaining().zip(fuel))
                    .is_some_and(|(left, before)| left < before),
                "restored null-installed-Weapon child still spends its saved travel budget"
            );
            for _ in 0..120 {
                scene.frame();
                if scene.0.simulation.projectiles().get(child_id).is_none() {
                    return;
                }
            }
            panic!("child never completed its real receiver and retirement");
        });
        let child_visits = visits
            .iter()
            .filter(|visit| visit["warhead"] == "IdentityChildWH")
            .collect::<Vec<_>>();
        assert_eq!(
            child_visits.len(),
            1,
            "one child impact, including after restore"
        );
        assert_eq!(child_visits[0]["damage"], damage);
        assert_eq!(child_visits[0]["source_id"], source);
        if short_alias {
            assert!(scene.actor(target).health.current() < health);
        }
    }
    assert_ne!(
        hashes[0], hashes[1],
        "actual child pointer changes rules identity"
    );
}
