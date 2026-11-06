//! Registered Scenario ammo overrides with physical retail weapons/art/assets.
//! Clear arena, human roster, visibility and pre-Logic semantic command delivery
//! are supplied boundaries; these are not original whole-frame or input goldens.

use super::*;
use crate::rules::ini_parser::IniFile;
use crate::rules::overlay_types::OverlayTypeRegistry;
use crate::sim::combat::TargetKind;
use crate::sim::command::Command;
use crate::sim::runtime::{SimResources, SimRuntime};
use crate::sim::snapshot::GameSnapshot;
use crate::sim::timer::CdTimer;

struct Scene(SimRuntime);

impl Scene {
    fn new(patch: &str) -> Option<Self> {
        Self::with_component(patch, None)
    }

    fn with_component(patch: &str, component: Option<&[u8]>) -> Option<Self> {
        let mut retail = crate::rules::retail_ini_fixture::retail_battle_rules_for_map_with_patch(
            "Hills.mmx",
            &IniFile::from_str(patch),
        )?;
        if let Some(bytes) = component {
            let digest = crate::util::sha256::sha256_hex(bytes);
            let names = retail
                .rules
                .all_objects()
                .map(|object| object.id.clone())
                .collect::<Vec<_>>();
            let program = crate::mod_sdk::Program::load_with_registries(
                bytes.to_vec(),
                &digest,
                retail.rules.sdk_configuration_passes(),
                &names,
                &[],
                &retail.rules.weapon_names(),
                retail.rules.sdk_configuration_registries(),
            )
            .unwrap();
            retail.rules.external_mod = Some(std::sync::Arc::new(program));
            retail.rules.bind_resource_policies().unwrap();
        }
        let (_, mut assets) = crate::rules::retail_ini_fixture::retail_assets()?;
        let theater = crate::map::theater::load_theater(&mut assets, "TEMPERATE").unwrap();
        let roots = crate::rules::effect_asset_catalog::tolerant_anim_class_roots(&retail.rules);
        retail
            .rules
            .bind_anim_class_assets(&roots, &assets, theater.extension, "TEMPERATE");
        retail
            .rules
            .bind_effect_assets(&assets, theater.extension, "TEMPERATE");
        let registry =
            OverlayTypeRegistry::from_ini(&retail.processed_rules, Some(&retail.fixed_art));
        let rules = retail.rules;
        let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 0xa220);
        // Match the production authored-scenario admission in runtime.rs:
        // rules own immutable policies; the world retains saved guest identity.
        sim.external_mod = rules.external_mod.as_deref().cloned();
        // The supplied clear map still owns its empty overlay cells and saved
        // wall-neighbor plane; full map-authority restore requires both.
        sim.install_overlay_grid_for_load(crate::sim::overlay_grid::OverlayGrid::new(32, 32));
        sim.intern_rule_type_ids(&rules);
        sim.resolve_type_handles(&rules);
        for name in ["Americans", "Russians"] {
            let owner = sim.interner.get(name).unwrap();
            let house = sim.houses.get_mut(&owner).unwrap();
            house.initialize_scenario_name(name);
            house.set_is_human_for_test(true);
            house.set_current_iq_for_test(0);
        }
        let owner = sim.interner.get("Americans").unwrap();
        sim.session.set_current_house_for_test(Some(owner));
        sim.session.set_game_mode(
            crate::sim::scenario_session::ScenarioGameMode::from_legacy_nonzero(true),
        );
        sim.session.game_options_mut_for_test().short_game = false;
        sim.fog = crate::sim::vision::FogState::new(32, 32);
        for name in ["Americans", "Russians"] {
            sim.fog
                .reveal_all_for_owner(sim.interner.get(name).unwrap());
        }
        let terrain_template = sim.resolved_terrain().cloned();
        let mut scene = Self(SimRuntime {
            simulation: sim,
            resources: SimResources {
                rules,
                overlay_registry: registry,
                terrain_template,
            },
        });
        scene.spawn("GAPOWR", "Americans", (4, 24));
        scene.spawn("NAPOWR", "Russians", (28, 28));
        // Assess real reactor output before constructing any Powered firer.
        scene.frame();
        Some(scene)
    }

    fn spawn(&mut self, kind: &str, owner: &str, cell: (u16, u16)) -> u64 {
        self.0
            .simulation
            .spawn_object_with_overlay_registry(
                kind,
                owner,
                cell.0,
                cell.1,
                0,
                &self.0.resources.rules,
                &self.0.resources.overlay_registry,
            )
            .expect("retail constructor and ordinary Unlimbo")
    }

    fn actor(&self, id: u64) -> &crate::sim::game_entity::GameEntity {
        self.0.simulation.entities().get(id).expect("live actor")
    }

    fn count(&self, id: u64) -> i32 {
        self.actor(id)
            .ammo()
            .expect("constructed Techno ammo")
            .current()
    }

    fn reload(&self, id: u64) -> CdTimer {
        // Observe the actual serialized clock without depending on a new API:
        // the same test source compiles against the pre-fix production tree.
        let entity = serde_json::to_value(self.actor(id)).unwrap();
        serde_json::from_value(entity["ammo_reload_timer"].clone())
            .expect("authoritative saved reload clock")
    }

    fn now(&self) -> i32 {
        self.0.simulation.session.binary_frame() as i32
    }

    fn command(&mut self, owner: &str, command: Command) {
        // The existing receiver is the supplied delivery boundary. No authored
        // target, mission, facing, position, ammunition or timer state follows.
        assert!(self.0.apply_command_for_test(owner, &command));
    }

    fn frame(&mut self) -> SimFrameOutput {
        let commands = self.0.simulation.take_due_commands();
        let output = self
            .0
            .advance_frame(&commands, 33, TickLane::Ordinary, FrameEffects::default())
            .unwrap();
        assert!(output.tick.frame_committed);
        output
    }

    fn shot(&mut self, source: u64, target: u64) -> i32 {
        for _ in 0..180 {
            let now = self.now();
            let output = self.frame();
            if let Some(shot) = output
                .fire_events
                .iter()
                .find(|shot| shot.attacker_id == source)
            {
                assert_eq!(shot.target, TargetKind::Entity(target));
                return now;
            }
        }
        panic!("ordinary target/facing/FireAt never produced a shot");
    }

    fn restored(&self, bytes: &[u8]) -> Simulation {
        let live = &self.0.simulation;
        let resources = &self.0.resources;
        let mut restored = GameSnapshot::load_validated(
            bytes,
            0, // supplied arena, not a saved physical-map hash claim
            resources.rules.simulation_config_hash(),
            live.session.map_name(),
        )
        .unwrap()
        .sim;
        // The real load owner reseeds Scenario RNG to zero. Preserve the live
        // process cursors through their owner; never overwrite Scenario RNG.
        assert_eq!(
            restored.scenario_rng.logical_state(),
            crate::sim::rng::SimRng::new(0).logical_state()
        );
        restored.retain_process_state_for_load(SnapshotProcessState::Live(live));
        restored.bind_shared_cell_dummy(crate::map::resolved_terrain::SharedCellDummy::fresh());
        restored.reconstruct_cellclass_dummy_for_map_resize();
        restored.restore_after_snapshot_load().unwrap();
        restored.rebuild_caches_after_load(
            resources.terrain_template.clone().unwrap(),
            live.terrain_speed_config.clone(),
            &resources.rules,
        );
        restored
            .restore_map_authority_after_snapshot_load(
                &resources.rules,
                &resources.overlay_registry,
                None,
            )
            .unwrap();
        restored.resolve_type_handles(&resources.rules);
        restored.rebuild_lighting_sources_after_load(&resources.rules);
        restored
    }
}

fn fired(output: &SimFrameOutput, source: u64) -> bool {
    output
        .fire_events
        .iter()
        .any(|shot| shot.attacker_id == source)
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with full-index NoAmmoWeapon"]
fn sdk_resources_high_weapon_slots_use_native_reading_elite_fallback_and_priority() {
    use crate::sim::combat::combat_weapon::{self, TargetFacts};
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let patch = "[VeraFeatures]\nAresNoAmmoWeapon=yes\n\
        [HTNK]\nAmmo=6\nInitialAmmo=0\nTurretCount=1\nWeaponCount=18\nIsGattling=yes\nNoAmmoWeapon=2\n\
        Weapon1=LowP\nWeapon2=LowS\nWeapon3=High\nEliteWeapon3=EliteHigh\nWeapon18=Last\n\
        [MTNK]\nAmmo=6\nInitialAmmo=0\nTurretCount=1\nWeaponCount=3\nIsGattling=no\nNoAmmoWeapon=2\n\
        Weapon1=LowP\nWeapon2=LowS\nWeapon3=High\n\
        [LowP]\nDamage=1\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\n\
        [LowS]\nDamage=2\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\n\
        [High]\nDamage=3\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\n\
        [EliteHigh]\nDamage=4\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\n\
        [Last]\nDamage=5\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\n";
    let Some(mut scene) = Scene::with_component(patch, Some(&wasm)) else {
        return;
    };
    let source = scene.spawn("HTNK", "Americans", (16, 20));
    let gunner = scene.spawn("MTNK", "Americans", (20, 20));
    let rules = &scene.0.resources.rules;
    let object = rules.object("HTNK").unwrap();
    let mut facts = combat_weapon::attacker_facts(scene.actor(source), object);
    let target = TargetFacts::Cell {
        land_type: 0,
        tile_in_water_set: false,
        bridge_flag: false,
    };
    // Ares10047AE0 preserves index2 after the gunner/OpenTransport arms;
    // GetWeapon70E140/7177E0 uses this same index for its elite fallback.
    assert_eq!(
        combat_weapon::what_weapon_should_i_use(rules, object, &facts, Some(&target), |index| {
            combat_weapon::get_weapon(
                scene.actor(source),
                object,
                index,
                scene.0.simulation.entities(),
                rules,
                scene.0.simulation.interner(),
            )
        }),
        2
    );
    assert_eq!(
        combat_weapon::weapon_for_index(object, 0, 2)
            .unwrap()
            .weapon_id,
        Some("High")
    );
    assert_eq!(
        combat_weapon::weapon_for_index(object, 200, 2)
            .unwrap()
            .weapon_id,
        Some("EliteHigh")
    );
    assert_eq!(
        combat_weapon::weapon_for_index(object, 200, 17)
            .unwrap()
            .weapon_id,
        Some("Last")
    );
    assert!(
        combat_weapon::weapon_for_index(object, 0, 3).is_none(),
        "empty slot stays empty"
    );
    assert!(
        combat_weapon::weapon_for_index(object, 0, 18).is_none(),
        "retain bounded lookup"
    );
    facts.open_transport_weapon = Some(1);
    assert_eq!(
        combat_weapon::what_weapon_should_i_use(rules, object, &facts, Some(&target), |index| {
            combat_weapon::get_weapon(
                scene.actor(source),
                object,
                index,
                scene.0.simulation.entities(),
                rules,
                scene.0.simulation.interner(),
            )
        }),
        1
    );
    let gunner_type = rules.object("MTNK").unwrap();
    let gunner_facts = combat_weapon::attacker_facts(scene.actor(gunner), gunner_type);
    assert_eq!(
        combat_weapon::what_weapon_should_i_use(
            rules,
            gunner_type,
            &gunner_facts,
            Some(&target),
            |index| combat_weapon::get_weapon(
                scene.actor(gunner),
                gunner_type,
                index,
                scene.0.simulation.entities(),
                rules,
                scene.0.simulation.interner()
            )
        ),
        0,
        "native non-gattling gunner returns before NoAmmoWeapon"
    );
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with full-index NoAmmoWeapon"]
fn sdk_resources_delayed_building_keeps_full_index_after_reload_and_snapshot_restore() {
    use crate::sim::world::DelayedFire;
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let patch = "[VeraFeatures]\nAresAmmo=yes\nAresReload=yes\nAresNoAmmoWeapon=yes\n\
        [TESLA]\nAmmo=6\nInitialAmmo=4\nReload=24\nTurretCount=1\nWeaponCount=3\nIsGattling=yes\n\
        NoAmmoWeapon=2\nNoAmmoAmount=5\nWeapon1=LowP\nWeapon2=LowS\nWeapon3=High\n\
        CanPassiveAquire=no\nTurret=yes\nROT=32\n\
        [LowP]\nDamage=1\nROF=90\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n\
        [LowS]\nDamage=2\nROF=90\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n\
        [High]\nDamage=3\nROF=90\nRange=9\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n";
    let Some(mut scene) = Scene::with_component(patch, Some(&wasm)) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let source = scene.spawn("TESLA", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source,
            target_id: target,
        },
    );
    let mut pending = None;
    for _ in 0..60 {
        let output = scene.frame();
        assert!(
            !fired(&output, source),
            "the real NATSLA delayed arm must precede FireAt"
        );
        pending = scene.actor(source).pending_building_fire.pending();
        if pending.is_some() {
            break;
        }
    }
    let pending = pending.expect("real building mission arms its animated shot");
    assert_eq!(pending.fire(), DelayedFire::Weapon(2));
    assert!(scene.count(source) <= 5);
    let saved = GameSnapshot::save_validated(
        &scene.0.simulation,
        0,
        scene.0.resources.rules.simulation_config_hash(),
        "full-index delayed shot",
        0,
    );
    let restored = scene.restored(&saved);
    scene.0.resources.rules.external_mod =
        Some(std::sync::Arc::new(restored.external_mod.clone().unwrap()));
    scene.0.resources.rules.bind_resource_policies().unwrap();
    scene.0.replace_simulation(restored);
    assert_eq!(
        scene.actor(source).pending_building_fire.pending(),
        Some(pending)
    );
    let mut launched = false;
    for _ in 0..60 {
        let output = scene.frame();
        if let Some(event) = output
            .fire_events
            .iter()
            .find(|event| event.attacker_id == source)
        {
            assert_eq!(
                scene.count(source),
                6,
                "the normal reload owner changed the selection condition while charging"
            );
            assert_eq!(
                scene.0.simulation.interner.resolve(event.weapon_id),
                "High",
                "ProcessDelayedFire450468/450480 reuses the saved full index, never reselects or folds to primary"
            );
            assert_eq!(event.target, TargetKind::Entity(target));
            assert!(
                scene
                    .actor(source)
                    .pending_building_fire
                    .pending()
                    .is_none()
            );
            launched = true;
            break;
        }
    }
    assert!(launched, "the restored delayed shot must launch");
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_reload_only_infantry_and_building_start_the_empty_clock_after_fire() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    for kind in ["E1", "GAPILL"] {
        let patch = format!(
            "[VeraFeatures]\nAresReload=yes\nAresAmmo=no\nAresNoAmmoWeapon=no\nAresManualControl=no\n[{kind}]\nAmmo=1\nInitialAmmo=1\nReload=8\nEmptyReload=24\nReloadAmount=1\nEmptyReloadAmount=1\nPrimary=ReloadProbe\nElitePrimary=ReloadProbe\nCanPassiveAquire=no\n\
             [ReloadProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\n"
        );
        let Some(mut scene) = Scene::with_component(&patch, Some(&wasm)) else {
            return;
        };
        let target = scene.spawn("GACNST", "Russians", (16, 16));
        let source = scene.spawn(kind, "Americans", (16, 20));
        scene.command(
            "Americans",
            Command::Attack {
                attacker_id: source,
                target_id: target,
            },
        );
        let first = scene.shot(source, target);
        assert_eq!(
            scene.count(source),
            0,
            "{kind}: no immediate refill after a real shot"
        );
        assert_eq!(
            (
                scene.reload(source).start_frame(),
                scene.reload(source).duration()
            ),
            (first, 24)
        );
        while scene.now() < first + 24 {
            assert!(!fired(&scene.frame(), source));
            assert_eq!(
                scene.count(source),
                0,
                "{kind}: configured empty interval must elapse"
            );
        }
        let output = scene.frame();
        assert!(
            scene.count(source) == 1 || fired(&output, source),
            "{kind}: reloaded at the deadline"
        );
    }
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_free_shots_do_not_restart_an_active_type_reload_clock() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let Some(mut scene) = Scene::with_component(
        "[VeraFeatures]\nAresReload=yes\nAresAmmo=yes\n[E1]\nAmmo=2\nInitialAmmo=1\nReload=1000\nReloadAmount=0\nPrimary=FreeProbe\nElitePrimary=FreeProbe\nCanPassiveAquire=no\n\
         [FreeProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n",
        Some(&wasm),
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let source = scene.spawn("E1", "Americans", (16, 20));
    scene.frame();
    let clock = scene.reload(source);
    assert_eq!(clock.duration(), 1000);
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source,
            target_id: target,
        },
    );
    scene.shot(source, target);
    assert_eq!(scene.count(source), 1);
    assert_eq!(
        (
            scene.reload(source).start_frame(),
            scene.reload(source).duration()
        ),
        (clock.start_frame(), clock.duration())
    );
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_reload_empty_then_partial_amounts_survive_snapshot() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let Some(mut scene) = Scene::with_component(
        "[VeraFeatures]\nAresReload=yes\n[LTNK]\nAmmo=6\nInitialAmmo=0\nReload=8\nEmptyReload=12\nReloadAmount=2\nEmptyReloadAmount=3\nCanPassiveAquire=no\n",
        Some(&wasm),
    ) else {
        return;
    };
    let source = scene.spawn("LTNK", "Americans", (16, 20));
    let first = scene.now();
    scene.frame();
    assert_eq!(
        scene.count(source),
        3,
        "exactly zero chooses the empty amount"
    );
    assert_eq!(
        scene.reload(source).duration(),
        8,
        "next interval uses the newly partial count"
    );
    let saved = GameSnapshot::save_validated(
        &scene.0.simulation,
        0,
        scene.0.resources.rules.simulation_config_hash(),
        "type reload",
        0,
    );
    let restored = scene.restored(&saved);
    scene.0.replace_simulation(restored);
    while scene.now() < first + 8 {
        scene.frame();
        assert_eq!(scene.count(source), 3);
    }
    scene.frame();
    assert_eq!(scene.count(source), 5, "partial reload adds two");
    while scene.now() < first + 16 {
        scene.frame();
    }
    scene.frame();
    assert_eq!(scene.count(source), 6, "normal amount caps at capacity");
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_negative_normal_reload_and_zero_empty_reload() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let Some(mut scene) = Scene::with_component(
        "[VeraFeatures]\nAresReload=yes\n[LTNK]\nAmmo=6\nInitialAmmo=3\nReload=4\nReloadAmount=-2\nEmptyReloadAmount=0\nCanPassiveAquire=no\n",
        Some(&wasm),
    ) else {
        return;
    };
    let source = scene.spawn("LTNK", "Americans", (16, 20));
    let first = scene.now();
    scene.frame();
    assert_eq!(scene.count(source), 1);
    while scene.now() <= first + 4 {
        scene.frame();
    }
    assert_eq!(scene.count(source), 0, "negative result clamps to zero");
    for _ in 0..12 {
        scene.frame();
        assert_eq!(
            scene.count(source),
            0,
            "zero empty amount never restores a round"
        );
    }
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_threshold_switches_to_free_secondary_through_real_fire() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let Some(mut scene) = Scene::with_component(
        "[VeraFeatures]\nAresAmmo=yes\nAresNoAmmoWeapon=yes\n[LTNK]\nAmmo=4\nInitialAmmo=4\nReload=1000\nNoAmmoWeapon=1\nNoAmmoAmount=2\nPrimary=AmmoProbe\nElitePrimary=AmmoProbe\nSecondary=FreeProbe\nEliteSecondary=FreeProbe\nCanPassiveAquire=no\n\
         [AmmoProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=2\n\
         [FreeProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n",
        Some(&wasm),
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let source = scene.spawn("LTNK", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source,
            target_id: target,
        },
    );
    scene.shot(source, target);
    assert_eq!(
        scene.count(source),
        2,
        "primary consumes two above the threshold"
    );
    scene.shot(source, target);
    assert_eq!(
        scene.count(source),
        2,
        "equality forces the zero-cost secondary"
    );
}

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with type_resource"]
fn sdk_resources_forced_secondary_still_checks_its_ammo_cost() {
    let wasm = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let Some(mut scene) = Scene::with_component(
        "[VeraFeatures]\nAresAmmo=yes\nAresNoAmmoWeapon=yes\n[LTNK]\nAmmo=1\nInitialAmmo=1\nNoAmmoWeapon=1\nNoAmmoAmount=1\nPrimary=FreeProbe\nElitePrimary=FreeProbe\nSecondary=CostlyProbe\nEliteSecondary=CostlyProbe\nCanPassiveAquire=no\n\
         [FreeProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=0\n\
         [CostlyProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=2\n",
        Some(&wasm),
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let source = scene.spawn("LTNK", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: source,
            target_id: target,
        },
    );
    for _ in 0..60 {
        assert!(
            !fired(&scene.frame(), source),
            "must not fall back to the free primary"
        );
    }
    assert_eq!(scene.count(source), 1);
}

#[test]
fn finite_infantry_reloads_and_resumes_fire_only_with_ares() {
    let Some(mut scene) =
        Scene::new("[E1]\nAmmo=1\nInitialAmmo=1\nReload=8\nEmptyReload=12\nCanPassiveAquire=no\n")
    else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let infantry = scene.spawn("E1", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: infantry,
            target_id: target,
        },
    );
    let first = scene.shot(infantry, target);
    assert_eq!(scene.count(infantry), 0);
    #[cfg(feature = "ares")]
    assert_eq!(
        (
            scene.reload(infantry).start_frame(),
            scene.reload(infantry).duration()
        ),
        (first, 12)
    );
    #[cfg(not(feature = "ares"))]
    let _ = first;
    let mut fired_again = false;
    for _ in 0..100 {
        if fired(&scene.frame(), infantry) {
            fired_again = true;
            break;
        }
    }
    assert_eq!(
        fired_again,
        cfg!(feature = "ares"),
        "Ares adds the Infantry reload visit; stock does not"
    );
}

/// Infantry5200E1..520105 refills only on a positive-fear turn that reaches
/// zero. Stock E1's unlimited count bypasses it; the native Scenario reader
/// admits this finite-magazine map override without an extension.
#[cfg(not(feature = "ares"))]
#[test]
fn finite_infantry_refills_after_damage_fear_expires_and_can_fire_again() {
    use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags};

    let Some(mut scene) = Scene::new("[E1]\nAmmo=1\nInitialAmmo=1\nCanPassiveAquire=no\n") else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let infantry = scene.spawn("E1", "Americans", (16, 20));
    let attack = Command::Attack {
        attacker_id: infantry,
        target_id: target,
    };
    scene.command("Americans", attack.clone());
    scene.shot(infantry, target);
    scene.command(
        "Americans",
        Command::Stop {
            entity_id: infantry,
        },
    );
    for _ in 0..3 {
        scene.frame();
        assert_eq!(scene.count(infantry), 0, "initial zero fear never refills");
    }
    let hit = EntityDamageEvent::direct_receiver(
        infantry,
        1,
        0,
        target,
        None,
        scene.0.simulation.interner.intern("SA"),
        ReceiverCallFlags {
            ignore_defenses: false,
            arg6: false,
        },
    );
    scene.0.simulation.commit_direct_damage_receiver(
        &scene.0.resources.rules,
        Some(&scene.0.resources.overlay_registry),
        hit,
        FrameEffects::default(),
    );
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 100);
    let reload = scene.reload(infantry);
    for _ in 0..99 {
        scene.frame();
        assert_eq!(scene.count(infantry), 0, "positive fear retains empty ammo");
    }
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 1);
    scene.frame();
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 0);
    assert_eq!(
        scene.count(infantry),
        1,
        "fear expiry copies the type's Ammo"
    );
    assert_eq!(
        scene.reload(infantry),
        reload,
        "this arm writes no reload clock"
    );
    scene.command("Americans", attack);
    scene.shot(infantry, target);
    assert_eq!(
        scene.count(infantry),
        0,
        "the next actual FireAt spends the refill"
    );
}

/// InfantryClass::FireAt51DF8C..51DFD6: a Fraidycat's last returned Bullet
/// starts panic and queues Guard. The physical E1 weapons/art stay unchanged;
/// these registered Scenario overrides activate the stock finite-ammo branch.
#[cfg(not(feature = "ares"))]
#[test]
fn fraidycat_last_round_queues_guard_panics_then_refills() {
    let Some(mut scene) =
        Scene::new("[E1]\nAmmo=2\nInitialAmmo=2\nFraidycat=yes\nCanPassiveAquire=no\n")
    else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let infantry = scene.spawn("E1", "Americans", (16, 20));
    let attack = Command::Attack {
        attacker_id: infantry,
        target_id: target,
    };
    scene.command("Americans", attack.clone());
    scene.shot(infantry, target);
    assert_eq!(scene.count(infantry), 1);
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 0);
    scene.shot(infantry, target);
    assert_eq!(scene.count(infantry), 0);
    let actor = scene.actor(infantry);
    assert_eq!(actor.infantry.unwrap().fear_level(), 300);
    assert_eq!(actor.mission.current().raw(), 1, "Guard is only queued");
    assert_eq!(actor.mission.queued().raw(), 5);
    let position = crate::sim::movement::ground_pose::position_world_xy(&actor.position);
    let reload = scene.reload(infantry);
    // A real Stop clears the old attack before the later same-tick refill can
    // feed another shot; it does not cancel the class's existing panic owner.
    scene.command(
        "Americans",
        Command::Stop {
            entity_id: infantry,
        },
    );
    let mut escaped = false;
    for _ in 0..299 {
        scene.frame();
        escaped |=
            crate::sim::movement::ground_pose::position_world_xy(&scene.actor(infantry).position)
                != position;
        assert_eq!(scene.count(infantry), 0);
    }
    assert!(escaped, "the live Fraidycat panic requests movement");
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 1);
    scene.frame();
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 0);
    assert_eq!(scene.count(infantry), 2);
    assert_eq!(scene.reload(infantry), reload, "no reload timer is armed");
    scene.command("Americans", attack);
    scene.shot(infantry, target);
    assert_eq!(scene.count(infantry), 1);
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 0);
}

/// FireAt's panic is independent of ReceiveDamage/ForcePanic's Fearless gate.
#[cfg(not(feature = "ares"))]
#[test]
fn fraidycat_last_round_does_not_use_fearless_gate() {
    let Some(mut scene) = Scene::new(
        "[E1]\nAmmo=1\nInitialAmmo=1\nFraidycat=yes\nFearless=yes\nCanPassiveAquire=no\n",
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let infantry = scene.spawn("E1", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: infantry,
            target_id: target,
        },
    );
    scene.shot(infantry, target);
    assert_eq!(scene.count(infantry), 0);
    assert_eq!(scene.actor(infantry).infantry.unwrap().fear_level(), 300);
    assert_eq!(scene.actor(infantry).mission.queued().raw(), 5);
}

#[test]
#[ignore = "requires the reference component with INI-selected ammo in VERA_SDK_AMMO_WASM"]
fn sdk_ammo_controls_infantry_admission_debit_and_reload() {
    let wasm =
        std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").expect("set VERA_SDK_AMMO_WASM"))
            .unwrap();
    let weapon =
        "[AmmoProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=";
    let Some(mut free) = Scene::with_component(
        &format!(
            "[VeraFeatures]\nAresAmmo=yes\n[E1]\nAmmo=0\nInitialAmmo=0\nPrimary=AmmoProbe\nElitePrimary=AmmoProbe\nCanPassiveAquire=no\n{weapon}0\n"
        ),
        Some(&wasm),
    ) else {
        return;
    };
    let target = free.spawn("GACNST", "Russians", (16, 16));
    let infantry = free.spawn("E1", "Americans", (16, 20));
    free.command(
        "Americans",
        Command::Attack {
            attacker_id: infantry,
            target_id: target,
        },
    );
    free.shot(infantry, target);
    assert_eq!(
        free.count(infantry),
        0,
        "zero-cost weapons fire with an empty magazine"
    );
    assert_eq!(free.reload(infantry).duration(), 0);

    let Some(mut pair) = Scene::with_component(
        &format!(
            "[VeraFeatures]\nAresAmmo=yes\n[E1]\nAmmo=3\nInitialAmmo=3\nReload=12\nPrimary=AmmoProbe\nElitePrimary=AmmoProbe\nCanPassiveAquire=no\n{weapon}2\n"
        ),
        Some(&wasm),
    ) else {
        return;
    };
    let target = pair.spawn("GACNST", "Russians", (16, 16));
    let infantry = pair.spawn("E1", "Americans", (16, 20));
    pair.command(
        "Americans",
        Command::Attack {
            attacker_id: infantry,
            target_id: target,
        },
    );
    let first = pair.shot(infantry, target);
    assert_eq!(pair.count(infantry), 1, "one shot consumes two rounds");
    assert_eq!(
        (
            pair.reload(infantry).start_frame(),
            pair.reload(infantry).duration()
        ),
        (first, 12)
    );
    pair.shot(infantry, target);
}

#[test]
#[ignore = "requires the reference component with INI-selected ammo in VERA_SDK_AMMO_WASM"]
fn sdk_ammo_vehicle_and_building_keep_partial_magazines_until_reload() {
    let wasm =
        std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").expect("set VERA_SDK_AMMO_WASM"))
            .unwrap();
    for kind in ["LTNK", "GAPILL"] {
        let patch = format!(
            "[VeraFeatures]\nAresAmmo=yes\n[{kind}]\nAmmo=3\nInitialAmmo=3\nReload=1000\nPrimary=AmmoProbe\nElitePrimary=AmmoProbe\nCanPassiveAquire=no\n\
             [AmmoProbe]\nDamage=1\nROF=15\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\nAmmo=2\n"
        );
        let Some(mut scene) = Scene::with_component(&patch, Some(&wasm)) else {
            return;
        };
        let target = scene.spawn("GACNST", "Russians", (16, 16));
        let source = scene.spawn(kind, "Americans", (16, 20));
        scene.command(
            "Americans",
            Command::Attack {
                attacker_id: source,
                target_id: target,
            },
        );
        let shot = scene.shot(source, target);
        assert_eq!(
            scene.count(source),
            1,
            "{kind}: one shot consumes two rounds"
        );
        let reload = scene.reload(source);
        assert!(reload.duration() > 0);
        let bytes = GameSnapshot::save_validated(
            &scene.0.simulation,
            0,
            scene.0.resources.rules.simulation_config_hash(),
            "external resource reload",
            0,
        );
        let restored = scene.restored(&bytes);
        scene.0.replace_simulation(restored);
        for _ in 0..100 {
            assert!(
                !fired(&scene.frame(), source),
                "{kind}: one round cannot fund another shot"
            );
            assert_eq!(scene.count(source), 1);
        }
        assert_eq!(scene.reload(source).start_frame(), shot);
    }
}

#[test]
#[cfg(not(feature = "ares"))]
#[ignore = "requires new VERA_SDK_AMMO_WASM and old VERA_SDK_LEGACY_WASM"]
fn sdk_ammo_building_weapon_rearm_preserves_legacy_and_survives_restore() {
    let new = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let old = std::fs::read(std::env::var_os("VERA_SDK_LEGACY_WASM").unwrap()).unwrap();
    let patch = "[VeraFeatures]\nAresAmmo=yes\n[GAPILL]\nAmmo=5\nInitialAmmo=5\nReload=1000\nPrimary=AmmoProbe\nElitePrimary=AmmoProbe\nCanPassiveAquire=no\n\
                 [AmmoProbe]\nDamage=1\nROF=45\nRange=6\nProjectile=InvisibleLow\nWarhead=SA\n";
    for (label, component) in [
        ("stock", None),
        ("old guest", Some(old.as_slice())),
        ("new guest", Some(new.as_slice())),
    ] {
        // The preceding guest required a prototype opt-in; the new component
        // must install this untagged-weapon cadence fix without that section.
        let patch = if label == "new guest" {
            patch
                .strip_prefix("[VeraFeatures]\nAresAmmo=yes\n")
                .unwrap()
        } else {
            patch
        };
        let Some(mut scene) = Scene::with_component(patch, component) else {
            return;
        };
        let target = scene.spawn("GACNST", "Russians", (16, 16));
        let source = scene.spawn("GAPILL", "Americans", (16, 20));
        scene.command(
            "Americans",
            Command::Attack {
                attacker_id: source,
                target_id: target,
            },
        );
        scene.shot(source, target);
        assert_eq!(scene.count(source), 4, "{label}");
        let rearm = scene.actor(source).rearm_timer.timer();
        if label != "new guest" {
            assert_eq!(
                rearm.duration(),
                1,
                "{label}: keep original magazine cadence"
            );
            continue;
        }
        assert!(
            rearm.duration() >= 45,
            "new policy must reach normal GetROF"
        );
        let saved = GameSnapshot::save_validated(
            &scene.0.simulation,
            0,
            scene.0.resources.rules.simulation_config_hash(),
            "component cadence",
            0,
        );
        let restored = scene.restored(&saved);
        scene.0.resources.rules.external_mod = Some(std::sync::Arc::new(
            restored.external_mod.clone().expect("saved component"),
        ));
        scene.0.resources.rules.bind_resource_policies().unwrap();
        scene.0.replace_simulation(restored);
        assert_eq!(scene.actor(source).rearm_timer.timer(), rearm);
        while scene.now() < rearm.start_frame() + rearm.duration() {
            assert!(
                !fired(&scene.frame(), source),
                "no extra shot before rearm expires"
            );
        }
        scene.shot(source, target);
        assert_eq!(scene.count(source), 3);
    }
}

// Stock GetROF6FCFA9 keeps its magazine shortcut without a component policy.
// Installed Ares10047A60 continues at6FCFC1; the test above covers that opt-in.
#[cfg(not(feature = "ares"))]
#[test]
fn finite_building_fires_a_rapid_pair_refills_zero_and_keeps_last_round_rearm() {
    let Some(mut scene) = Scene::new("[GAPILL]\nAmmo=2\nInitialAmmo=2\nCanPassiveAquire=no\n")
    else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (16, 16));
    let gun = scene.spawn("GAPILL", "Americans", (16, 20));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: gun,
            target_id: target,
        },
    );
    let first = scene.shot(gun, target);
    assert_eq!(
        scene.count(gun),
        1,
        "the actual first ordinary shot debits two to one"
    );
    assert_eq!(scene.actor(gun).rearm_timer.duration(), 1);
    let second = scene.now();
    let output = scene.frame();
    assert!(fired(&output, gun), "ROF1 admits the next ordinary visit");
    assert_eq!(second, first + 1);
    assert_eq!(
        scene.count(gun),
        2,
        "zero refills in the same outer Building visit"
    );
    let rearm = scene.actor(gun).rearm_timer.timer();
    assert_eq!(rearm.start_frame(), second);
    assert!(rearm.duration() > 1);
    while scene.now() < rearm.start_frame() + rearm.duration() {
        assert!(
            !fired(&scene.frame(), gun),
            "refill must preserve the long cooldown"
        );
    }
    assert_eq!(scene.actor(gun).rearm_timer.timer(), rearm);
    let third = scene.shot(gun, target);
    assert!(third >= rearm.start_frame() + rearm.duration());
    assert_eq!(scene.count(gun), 1);
}

#[test]
fn finite_unit_empty_reload_expires_after_fire_and_survives_actual_snapshot_restore() {
    let Some(mut scene) = Scene::new(
        "[LTNK]\nAmmo=1\nInitialAmmo=1\nReload=18\nEmptyReload=36\nCanPassiveAquire=no\n",
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (12, 12));
    let tank = scene.spawn("LTNK", "Americans", (12, 16));
    assert_eq!(
        scene.count(tank),
        1,
        "full constructor count avoids prefire reload"
    );
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: tank,
            target_id: target,
        },
    );
    let shot = scene.shot(tank, target);
    assert_eq!(
        scene.count(tank),
        0,
        "the real returned Bullet empties the Unit"
    );
    let reload = scene.reload(tank);
    let rearm = scene.actor(tank).rearm_timer.timer();
    assert_eq!((reload.start_frame(), reload.duration()), (shot, 36));
    assert!((60..=62).contains(&rearm.duration()));
    for _ in 0..7 {
        assert!(!fired(&scene.frame(), tank));
        assert_eq!(scene.count(tank), 0);
        assert_eq!(scene.reload(tank), reload);
    }
    let bytes = GameSnapshot::save_validated(
        &scene.0.simulation,
        0,
        scene.0.resources.rules.simulation_config_hash(),
        "pending empty reload",
        0,
    );
    // Resume through the actual load owner, including its Scenario RNG reseed.
    let loaded = scene.restored(&bytes);
    scene.0.replace_simulation(loaded);
    assert_eq!(scene.reload(tank), reload);
    assert_eq!(scene.actor(tank).rearm_timer.timer(), rearm);
    let mut saw_refill = false;
    let mut saw_next_shot = false;
    for _ in 0..80 {
        let now = scene.now();
        let output = scene.frame();
        let shot_again = fired(&output, tank);
        if now < shot + 36 {
            assert_eq!(scene.count(tank), 0);
        }
        if now == shot + 36 {
            assert!(!shot_again);
            assert_eq!(scene.count(tank), 1);
            assert!(rearm.remaining(now) > 0, "reload never clears weapon rearm");
            assert_eq!(scene.actor(tank).rearm_timer.timer(), rearm);
            saw_refill = true;
        }
        if shot_again {
            assert!(now >= rearm.start_frame() + rearm.duration());
            assert_eq!(scene.count(tank), 0);
            assert_eq!(scene.reload(tank).start_frame(), now);
            assert_eq!(scene.reload(tank).duration(), 36);
            saw_next_shot = true;
        }
        if saw_next_shot {
            break;
        }
    }
    assert!(saw_refill && saw_next_shot);
}

#[test]
fn damage_reduces_readiness_before_object_immunity_and_restarts_reload() {
    use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags};

    let Some(mut scene) = Scene::new(
        "[LTNK]\nAmmo=4\nInitialAmmo=4\nStrength=200\nImmune=yes\nTypeImmune=yes\nReload=12\n\
         EmptyReload=25\nDamageReducesReadiness=yes\nReadinessReductionMultiplier=1\n",
    ) else {
        return;
    };
    let tank = scene.spawn("LTNK", "Americans", (12, 12));
    let now = scene.now();
    let hit = |scene: &mut Scene, damage, source| {
        let event = EntityDamageEvent::direct_receiver(
            tank,
            damage,
            0,
            source,
            None,
            scene.0.simulation.interner.intern("AP"),
            ReceiverCallFlags {
                ignore_defenses: false,
                arg6: false,
            },
        );
        scene.0.simulation.commit_direct_damage_receiver(
            &scene.0.resources.rules,
            Some(&scene.0.resources.overlay_registry),
            event,
            FrameEffects::default(),
        );
    };
    hit(&mut scene, 100, tank);
    assert_eq!(scene.count(tank), 4, "TypeImmune returns before readiness");
    // Native701AEF uses defended damage, before Object's Immune/Verses
    // gates. Half the type's Strength consumes half its four-round magazine.
    hit(&mut scene, 100, 0);
    assert_eq!(scene.actor(tank).health.current(), 200);
    assert_eq!(scene.count(tank), 2);
    assert_eq!(scene.reload(tank), CdTimer::started(now, 12));
    hit(&mut scene, 100, 0);
    assert_eq!(scene.count(tank), 0);
    assert_eq!(scene.reload(tank), CdTimer::started(now, 25));
    hit(&mut scene, -100, tank);
    assert_eq!(scene.count(tank), 2, "healing restores readiness too");
    assert_eq!(scene.reload(tank), CdTimer::started(now, 12));
    while scene.now() <= now + 12 {
        scene.frame();
    }
    assert_eq!(
        scene.count(tank),
        3,
        "the shared Unit reload adds one round"
    );
}

#[test]
fn moving_finite_unit_extends_its_quotient_reload_then_adds_only_one_round_at_a_time() {
    let Some(mut scene) = Scene::new(
        "[LTNK]\nAmmo=6\nInitialAmmo=6\nReload=60\nReloadIncrement=3\nPipWrap=2\n\
         MobileFire=no\nCanPassiveAquire=no\n",
    ) else {
        return;
    };
    let target = scene.spawn("GACNST", "Russians", (12, 12));
    let tank = scene.spawn("LTNK", "Americans", (12, 16));
    scene.command(
        "Americans",
        Command::Attack {
            attacker_id: tank,
            target_id: target,
        },
    );
    scene.shot(tank, target);
    assert_eq!(
        scene.count(tank),
        5,
        "actual first shot from a full magazine"
    );
    scene.shot(tank, target);
    assert_eq!(
        scene.count(tank),
        4,
        "second shot precedes the positive reload expiry"
    );
    assert_eq!(scene.reload(tank).duration(), 72, "60 + 3 * (4 / 2)^2");
    scene.command(
        "Americans",
        Command::Move {
            entity_id: tank,
            target_rx: 12,
            target_ry: 28,
        },
    );
    let mut moving_visits = 0;
    for _ in 0..80 {
        let before = scene.reload(tank);
        let now = scene.now();
        assert!(!fired(&scene.frame(), tank));
        assert_eq!(scene.count(tank), 4);
        if crate::sim::movement::motion_query::is_moving(scene.actor(tank)) == Some(true) {
            let after = scene.reload(tank);
            assert_eq!(after.start_frame(), now);
            assert_eq!(after.duration(), before.remaining(now) + 1);
            moving_visits += 1;
            if moving_visits == 12 {
                break;
            }
        }
    }
    assert_eq!(
        moving_visits, 12,
        "actual installed Drive +10 must report movement"
    );
    scene.command("Americans", Command::Stop { entity_id: tank });
    for _ in 0..80 {
        scene.frame();
        if crate::sim::movement::motion_query::is_moving(scene.actor(tank)) == Some(false) {
            break;
        }
    }
    assert_eq!(
        crate::sim::movement::motion_query::is_moving(scene.actor(tank)),
        Some(false)
    );
    let stopped = scene.reload(tank);
    assert!(stopped.remaining(scene.now()) > 0);
    while scene.now() < stopped.start_frame() + stopped.duration() {
        scene.frame();
        assert_eq!(scene.count(tank), 4);
        assert_eq!(scene.reload(tank), stopped);
    }
    scene.frame();
    assert_eq!(
        scene.count(tank),
        5,
        "expiry adds one, rather than filling the magazine"
    );
    let next = scene.reload(tank);
    assert_eq!(
        next.duration(),
        72,
        "the next real count5 retains quotient2"
    );
    while scene.now() < next.start_frame() + next.duration() {
        scene.frame();
        assert_eq!(scene.count(tank), 5);
    }
    scene.frame();
    assert_eq!(scene.count(tank), 6);
    assert_eq!(
        scene.reload(tank),
        next,
        "a full magazine leaves the old timer"
    );
}
