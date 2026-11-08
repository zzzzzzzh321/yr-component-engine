//! AR265 ordinary FireAt with the real retained retail reader and component.
//! The arena/command delivery are supplied boundaries, not native frame goldens.

use super::*;
use crate::sim::projectile::{ProjectileCoord, ProjectileVelocity};

fn scatter_patch(flak: bool) -> String {
    format!(
        "[LTNK]\nPrimary=ScatterGun\nElitePrimary=ScatterGun\n\
         Ammo=1\nInitialAmmo=1\nReload=1000\nCanPassiveAquire=no\n\
         [HTNK]\nPrimary=none\nSecondary=none\nElitePrimary=none\nEliteSecondary=none\n\
         Strength=1000\nCanPassiveAquire=no\n\
         [ScatterGun]\nDamage=40\nROF=40\nRange=8\nProjectile=ScatterShot\n\
         Speed=20\nWarhead=ScatterWH\n\
         [ScatterShot]\nImage=120MM\nArcing=yes\nInaccurate=yes\n\
         Inviso=no\nRanged=no\nFlakScatter={}\nBallisticScatter.Min=0\nBallisticScatter.Max=0\n\
         [ScatterWH]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
         CellSpread=2\nAnimList=TWLT070\n",
        if flak { "yes" } else { "no" },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Flight {
    origin: ProjectileCoord,
    aim: ProjectileCoord,
    position: ProjectileCoord,
    velocity: ProjectileVelocity,
}

impl Flight {
    fn read(scene: &Scene, id: u64) -> Self {
        let bullet = scene.0.simulation.projectiles().get(id).unwrap();
        Self {
            origin: bullet.launch_origin(),
            aim: bullet.launch_target(),
            position: bullet.position(),
            velocity: bullet.velocity(),
        }
    }
}

fn scatter_words(draws: &[serde_json::Value]) -> Vec<u32> {
    draws
        .iter()
        .filter(|draw| {
            draw["callers"]
                .as_str()
                .unwrap()
                .contains("fireat_launch_scatter")
        })
        .map(|draw| draw["value"].as_u64().unwrap() as u32)
        .collect()
}

fn restore_and_finish(scene: &mut Scene, id: u64, source: u64, target: u64) {
    let flight = Flight::read(scene, id);
    let payload = scene.0.simulation.projectiles().get(id).unwrap().payload();
    let rules_hash = scene.0.resources.rules.simulation_config_hash();
    let policy = scene
        .0
        .resources
        .rules
        .projectile("ScatterShot")
        .unwrap()
        .ballistic_scatter_policy();
    let saved = GameSnapshot::save_validated(
        &scene.0.simulation,
        0,
        rules_hash,
        "in-flight projectile scatter",
        0,
    );
    let restored = scene.restored(&saved);
    // Restore binds the frozen guest configuration. Neither the INI reader nor
    // the launch-scatter owner runs again for a Bullet already in flight.
    scene.0.resources.rules.external_mod = Some(std::sync::Arc::new(
        restored
            .external_mod
            .clone()
            .expect("saved scatter component"),
    ));
    scene.0.resources.rules.bind_resource_policies().unwrap();
    assert_eq!(scene.0.resources.rules.simulation_config_hash(), rules_hash);
    assert_eq!(
        scene
            .0
            .resources
            .rules
            .projectile("ScatterShot")
            .unwrap()
            .ballistic_scatter_policy(),
        policy,
    );
    scene.0.replace_simulation(restored);
    assert_eq!(
        Flight::read(scene, id),
        flight,
        "saved payload and motion are authoritative"
    );
    assert_eq!(
        scene.0.simulation.projectiles().get(id).unwrap().payload(),
        payload
    );
    let (output, draws) = crate::sim::rng::trace_draws(|| scene.frame());
    assert!(
        output.fire_events.is_empty(),
        "the one round was already spent"
    );
    // The normal frame keeps timer/House/other random activity. Observe only
    // this launch owner, whose trace was calibrated on the real firing call.
    assert!(
        scatter_words(&draws).is_empty(),
        "resumed flight must not draw launch scatter again"
    );
    assert_ne!(Flight::read(scene, id).position, flight.position);

    let health = scene.actor(target).health.current();
    let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
        for _ in 0..180 {
            let output = scene.frame();
            assert!(
                !output
                    .fire_events
                    .iter()
                    .any(|event| event.attacker_id == source)
            );
            if scene.0.simulation.projectiles().get(id).is_none() {
                return;
            }
        }
        panic!("the restored ordinary ballistic Bullet never retired");
    });
    assert_eq!(
        visits.len(),
        1,
        "one real Bullet reaches the shared explosion receiver"
    );
    assert_eq!(visits[0]["warhead"], "ScatterWH");
    assert_eq!(visits[0]["source_id"], source);
    assert!(visits[0]["damage"].as_i64().unwrap() > 0);
    assert!(scene.actor(target).health.current() < health);
    assert_eq!(scene.count(source), 0);
    assert!(
        !scene
            .0
            .simulation
            .projectiles()
            .iter()
            .any(|(_, bullet)| bullet.source_id() == source)
    );
}

#[test]
#[cfg(not(feature = "ares"))]
#[ignore = "requires retail assets, VERA_SDK_SCATTER_WASM and VERA_SDK_SCATTER_PREVIOUS_WASM"]
fn sdk_projectile_scatter_fireat_plain_flak_and_saved_flight() {
    let current = std::fs::read(std::env::var_os("VERA_SDK_SCATTER_WASM").unwrap()).unwrap();
    let previous =
        std::fs::read(std::env::var_os("VERA_SDK_SCATTER_PREVIOUS_WASM").unwrap()).unwrap();
    let mut observations = Vec::new();
    for (label, component, overridden) in [
        ("stock", None, false),
        ("previous .31", Some(previous.as_slice()), false),
        ("scatter .32", Some(current.as_slice()), true),
    ] {
        let mut pair = Vec::new();
        for flak in [false, true] {
            let mut scene = Scene::with_component(&scatter_patch(flak), component)
                .expect("supply retail INIs/assets for the real FireAt fixture");
            assert!(scene.0.resources.rules.combat_damage.ballistic_scatter > 0);
            let policy = scene
                .0
                .resources
                .rules
                .projectile("ScatterShot")
                .unwrap()
                .ballistic_scatter_policy();
            assert_eq!(
                policy.is_some(),
                overridden,
                "{label}: component owns the optional domain"
            );
            if overridden {
                assert_eq!(
                    policy.unwrap(),
                    vera_mod_api::BallisticScatterPolicy {
                        min: Some(0),
                        max: Some(0)
                    }
                );
            }
            // Keep both endpoints inside the arena's real map diamond. A
            // stationary unarmed human vehicle avoids building-foundation aim.
            for cell in [(6, 22), (6, 16)] {
                assert!(scene.0.simulation.map_cell_in_bounds(cell));
            }
            let target = scene.spawn("HTNK", "Russians", (6, 16));
            let source = scene.spawn("LTNK", "Americans", (6, 22));
            scene.command(
                "Americans",
                Command::Attack {
                    attacker_id: source,
                    target_id: target,
                },
            );
            let ((_, event), draws) =
                crate::sim::rng::trace_draws(|| scene.shot_event(source, target));
            assert_eq!(
                scene.0.simulation.interner.resolve(event.weapon_id),
                "ScatterGun"
            );
            assert_eq!(
                scene.count(source),
                0,
                "one ordinary FireAt consumes the round"
            );
            let (&id, _) = scene
                .0
                .simulation
                .projectiles()
                .iter()
                .find(|(_, bullet)| bullet.source_id() == source)
                .expect("the real ballistic Bullet survives its first Logic visit");
            let raw = scatter_words(&draws);
            assert!(
                !raw.is_empty(),
                "the real FireAt enters the observed scatter owner"
            );
            pair.push((
                Flight::read(&scene, id),
                scene.0.simulation.scenario_rng.logical_state(),
                raw,
            ));
            if overridden && !flak {
                restore_and_finish(&mut scene, id, source, target);
            }
        }
        observations.push(pair);
    }
    // No scatter math is repeated as an oracle. The same actual FireAt input
    // must retain stock behavior for .31, while equal zero bounds remove only
    // the magnitude in both new hook branches (10019930 / 10019980). Their
    // angle call remains on the same Scenario stream.
    assert_eq!(
        observations[0], observations[1],
        "old component ignores the new INI keys"
    );
    assert_eq!(
        observations[2][0], observations[2][1],
        "zero plain/flak scatter has the same flight and angle stream"
    );
    for index in 0..2 {
        assert_ne!(
            observations[2][index].0.velocity, observations[0][index].0.velocity,
            "custom zero bounds change the real flight relative to global scatter"
        );
    }
}
