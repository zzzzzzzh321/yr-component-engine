//! Native Airburst fan and production receiver integration. Native goldens
//! cover the fan/Construct/Fire/RNG; target damage below is a Rust regression
//! through the shared damage owner, not a claim of whole native combat parity.

use super::*;
use crate::sim::projectile::{ProjectileCoord, ProjectileDetonationReason};
use crate::sim::rng::SimRng;
#[cfg(not(feature = "ares"))]
use crate::sim::world::{FrameEffects, UninitContext};
use serde_json::Value;

fn rules_ini() -> &'static str {
    // Secondary registers the child BEFORE the stock Weapon sweep. The
    // Airburst factory in the later Bullet sweep never recursively reads it.
    "[Clear]\nBuildable=yes\n[VehicleTypes]\n0=SOURCE\n1=VICTIM\n\
         [SOURCE]\nStrength=100\nPrimary=PARENT_GUN\nSecondary=CHILD_GUN\nSpeed=4\n\
         [VICTIM]\nStrength=1000\nArmor=none\nSpeed=4\n\
         [PARENT_GUN]\nProjectile=PARENT_SHOT\nWarhead=QUIET\nDamage=0\nRange=10\n\
         [CHILD_GUN]\nProjectile=CHILD_SHOT\nWarhead=CHILD_WH\nDamage=37\nSpeed=73\nBright=yes\nRange=4\n\
         [PARENT_SHOT]\nAirburst=yes\nAirburstWeapon=CHILD_GUN\nCluster=2\n\
         [CHILD_SHOT]\nFirersPalette=yes\n\
         [Warheads]\n0=QUIET\n1=CHILD_WH\n\
         [QUIET]\nVerses=0%,0%,0%,0%,0%,0%,0%,0%,0%,0%,0%\n\
         [CHILD_WH]\nCellSpread=0\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n"
}

fn rules() -> RuleSet {
    let ini = crate::rules::ini_parser::IniFile::from_str(rules_ini());
    let mut rules = RuleSet::from_ini(&ini).unwrap();
    // The native fixture supplies a post-Process Weapon field73, independent
    // of the already-tested ROT0 ballistic Speed postpass.
    rules.set_weapon_speed_for_test("CHILD_GUN", 73);
    rules
}

fn scene(z: i32) -> (Simulation, RuleSet, u64, ProjectileDetonation) {
    let rules = rules();
    let mut world = crate::sim::arena_fixture::rival_arena(&rules, 31);
    world.intern_rule_type_ids(&rules);
    world.resolve_type_handles(&rules);
    world.session.set_binary_frame_for_test(100);
    let source = world
        .spawn_object("SOURCE", "Americans", 6, 20, 0, &rules)
        .unwrap();
    let wh = world.interner.intern("QUIET");
    let kind = world.interner.intern("PARENT_SHOT");
    let payload = ProjectilePayload::new(0, wh, Some(kind), true).with_damage_multiplier(768);
    let target = ProjectileTarget::Cell { rx: 10, ry: 20 };
    let parent = world.construct_projectile(source, target, payload, &rules);
    world
        .projectiles
        .get_mut(parent)
        .unwrap()
        .set_position_for_test(ProjectileCoord::new(2688, 5248, z));
    let request = ProjectileDetonation {
        projectile_id: parent,
        source_id: source,
        target,
        impact: ProjectileCoord::new(2699, 5259, z + 10),
        payload,
        reason: ProjectileDetonationReason::ReachedTarget,
    };
    (world, rules, source, request)
}

fn detonate(world: &mut Simulation, rules: &RuleSet, request: ProjectileDetonation) {
    let mut run = crate::sim::combat::world_receiver::ReceiverRun::default();
    crate::sim::combat::world_receiver::commit_projectiles(
        world,
        &mut run,
        &[request],
        rules,
        None,
    );
    run.finish();
}

#[test]
#[cfg(not(feature = "ares"))]
fn stock_airburst_tail_matches_native_children_rng_and_cleanup() {
    let rows: Value = serde_json::from_str(crate::test_fixture::text(
        "tools/projectile_oracle/airburst.json",
    ))
    .unwrap();
    let native = &rows[0];
    let (mut world, rules, source, request) = scene(600);
    world.scenario_rng =
        SimRng::from_native_state_hex_for_test(native["rng_before"]["scenario"].as_str().unwrap());
    let main = world.main_rng.native_state_hex();
    let mapgen = world.mapgen_rng.native_state_hex();
    detonate(&mut world, &rules, request);
    let mut children: Vec<_> = world
        .projectiles
        .iter()
        .filter(|(id, _)| **id != request.projectile_id)
        .collect();
    children.sort_by_key(|(id, _)| **id);
    assert_eq!(children.len(), native["children"].as_array().unwrap().len());
    for ((_, child), expected) in children.iter().zip(native["children"].as_array().unwrap()) {
        let target = &expected["target"];
        assert_eq!(
            child.target(),
            ProjectileTarget::Cell {
                rx: target[0].as_u64().unwrap() as u16,
                ry: target[1].as_u64().unwrap() as u16,
            }
        );
        let origin = child.launch_origin();
        assert_eq!([origin.x, origin.y, origin.z], [2688, 5248, 600]);
        let bits: Vec<_> = child
            .velocity()
            .native()
            .iter()
            .map(|value| format!("{:016x}", value.bits()))
            .collect();
        assert_eq!(
            serde_json::json!(bits),
            expected["after_fire"]["velocity"]["bits"]
        );
        assert_eq!(child.source_id(), source);
        assert_eq!(child.payload().base_damage, 37);
        assert_eq!(child.payload().damage_multiplier(), 256);
        assert!(!child.payload().bright());
        assert!(child.payload().installed_weapon().is_none());
        assert_eq!(
            child.firer_house(),
            Some(world.substrate.entities.get(source).unwrap().owner()),
        );
        assert_eq!(child.arm_timer().start_frame(), 100);
        assert_eq!(child.arm_timer().duration(), 0);
        assert!(child.in_logic_vector());
    }
    assert_eq!(
        world.scenario_rng.native_state_hex(),
        native["rng_after"]["scenario"]
    );
    assert_eq!(world.main_rng.native_state_hex(), main);
    assert_eq!(world.mapgen_rng.native_state_hex(), mapgen);
    let ids: Vec<_> = children.iter().map(|(id, _)| **id).collect();
    assert!(
        world.retire_non_entity_object(request.projectile_id, UninitContext::with_rules(&rules))
    );
    // Parent removal must leave the children alive. Each owns its source,
    // cell target, Logic slot and expiry registration through existing owners.
    assert!(ids.iter().all(|id| world.projectiles.get(*id).is_some()));
    world.uninit_with_rules(source, &rules);
    assert!(
        ids.iter()
            .all(|id| world.projectiles.get(*id).unwrap().source_id() == 0)
    );
    for id in ids {
        world.retire_non_entity_object(id, UninitContext::with_rules(&rules));
    }
    world.flush_pending_delete();
    assert!(world.projectiles.is_empty());
}

#[test]
#[cfg(not(feature = "ares"))]
fn stock_airburst_children_reach_shared_damage_and_expire() {
    let (mut world, rules, _source, request) = scene(0);
    let victim = world
        .spawn_object("VICTIM", "Russians", 10, 20, 0, &rules)
        .unwrap();
    let before = world
        .substrate
        .entities
        .get(victim)
        .unwrap()
        .health
        .current();
    detonate(&mut world, &rules, request);
    let first = world
        .projectiles
        .iter()
        .filter(|(id, _)| **id != request.projectile_id)
        .map(|(id, _)| *id)
        .min()
        .unwrap();
    world.retire_non_entity_object(request.projectile_id, UninitContext::with_rules(&rules));
    let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
        for frame in 101..106 {
            world.session.set_binary_frame_for_test(frame);
            world.visit_combat_tail(first, &rules, None, FrameEffects::default());
            world.flush_pending_delete();
        }
    });
    assert!(
        visits
            .iter()
            .any(|visit| visit["warhead"] == "CHILD_WH" && visit["damage"] == 37),
        "actual area entries: {:?}; remaining Bullets: {}",
        visits
            .iter()
            .map(|visit| (
                &visit["frame"],
                &visit["coord"],
                &visit["warhead"],
                &visit["damage"]
            ))
            .collect::<Vec<_>>(),
        world.projectiles.iter().count(),
    );
    assert!(
        world
            .substrate
            .entities
            .get(victim)
            .unwrap()
            .health
            .current()
            < before
    );
    assert!(world.projectiles.is_empty());
}

#[test]
#[cfg(not(feature = "ares"))]
fn stock_airburst_reads_live_source_and_requires_a_represented_parent() {
    let (mut world, rules, source, request) = scene(600);
    // Keep the old packet's source intentionally stale. The authoritative
    // Bullet pointer expiry must determine every newly constructed child.
    world.notify_pointer_expired(source, UninitContext::with_rules(&rules));
    detonate(&mut world, &rules, request);
    let children: Vec<_> = world
        .projectiles
        .iter()
        .filter(|(id, _)| **id != request.projectile_id)
        .collect();
    assert_eq!(children.len(), 9);
    assert!(
        children
            .iter()
            .all(|(_, child)| child.source_id() == 0 && child.firer_house().is_none())
    );
    let before = world.projectiles.iter().count();
    let rng = world.scenario_rng.native_state_hex();
    emit(
        &mut world,
        &rules,
        &ProjectileDetonation {
            projectile_id: u64::MAX,
            ..request
        },
    );
    assert_eq!(world.projectiles.iter().count(), before);
    assert_eq!(world.scenario_rng.native_state_hex(), rng);
}

/// The same real parent receiver runs before either tail selection. Existing
/// stock tests retain velocity/cleanup parity; this connected component case
/// uses the normal parsed Weapon speed, with no fixture-only Speed override.
#[test]
#[ignore = "requires VERA_SDK_COMBAT_TAIL_WASM / VERA_SDK_COMBAT_TAIL_PREVIOUS_WASM"]
fn component_airburst_refusal_keeps_parent_receivers_and_saved_selection() {
    use crate::assets::asset_manager::{MediaArchiveMode, test_support::audio_test_install};
    use crate::rules::{
        art_data::ArtRegistry, audio_sources::AudioDefinitions, ini_parser::IniFile,
        native_processing::ProcessedRulesLayers, process_owner::NativeScenarioRulesPrefix,
        retail_sources::RetailRulesSources,
    };
    use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};
    use crate::util::save_codec;
    use vera_mod_api::{
        AirburstTailPolicy, Context, HookKind, MatchRulesContext, ModuleManifest, Plan, Request,
    };
    let component = std::fs::read(
        std::env::var_os("VERA_SDK_COMBAT_TAIL_WASM").expect("set VERA_SDK_COMBAT_TAIL_WASM"),
    )
    .unwrap();
    let previous = std::fs::read(
        std::env::var_os("VERA_SDK_COMBAT_TAIL_PREVIOUS_WASM")
            .expect("set VERA_SDK_COMBAT_TAIL_PREVIOUS_WASM"),
    )
    .unwrap();
    let request = Request {
        api_version: vera_mod_api::API_VERSION,
        context: Context::MatchRules(MatchRulesContext {}),
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut selected = unchanged.clone();
    selected.match_rules = Some(
        serde_json::from_value(serde_json::json!({
            "survival": "retail", "airburst_tail": "retail_fixed_nine",
        }))
        .unwrap(),
    );
    let retail = vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: vera_mod_api::API_VERSION,
            id: "test.airburst-retail".into(),
            version: "1".into(),
            config_version: 1,
            state_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::MatchRules,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[("match_rules", &serde_json::to_vec(&selected).unwrap())],
    );
    let native: Value = serde_json::from_str(crate::test_fixture::text(
        "tools/projectile_oracle/airburst.json",
    ))
    .unwrap();
    let native = &native[0];
    for (label, module, refused) in [
        ("airburst-current", Some(component.as_slice()), true),
        ("airburst-no-module", None, crate::rules::ARES_ENABLED),
        (
            "airburst-previous",
            Some(previous.as_slice()),
            crate::rules::ARES_ENABLED,
        ),
        ("airburst-explicit-retail", Some(retail.as_slice()), false),
    ] {
        eprintln!("Airburst tail component control: {label}");
        let directory = audio_test_install(label);
        // The fan oracle uses zero-radius damage and an offset impact point.
        // This positive-parent regression needs a receiver inside the blast;
        // give only its physical INI a radius. Keep the stock oracle unchanged.
        let parent_rules = rules_ini()
            .replace("Warhead=QUIET\nDamage=0", "Warhead=CHILD_WH\nDamage=37")
            .replace("[CHILD_WH]\nCellSpread=0\n", "[CHILD_WH]\nCellSpread=0.5\n");
        for (name, bytes) in [
            ("rulesmd.ini", parent_rules.as_bytes()),
            ("artmd.ini", b"".as_slice()),
            ("soundmd.ini", b"".as_slice()),
            ("evamd.ini", b"".as_slice()),
        ] {
            std::fs::write(directory.path().join(name), bytes).unwrap();
        }
        if let Some(module) = module {
            std::fs::write(directory.path().join("gameplay.wasm"), module).unwrap();
            std::fs::write(directory.path().join("vera-mod.json"), serde_json::to_vec(&serde_json::json!({
                "api_version": vera_mod_api::API_VERSION, "program": "gameplay.wasm", "sha256": crate::util::sha256::sha256_hex(module),
            })).unwrap()).unwrap();
        }
        let assets =
            crate::mod_sdk::open_active_assets(directory.path(), MediaArchiveMode::STOCK_DIGITAL)
                .unwrap();
        let audio = AudioDefinitions::select(&assets);
        let mut owner = RetailRulesSources::select(&assets)
            .unwrap()
            .into_owner(audio.sounds().clone(), audio.eva().clone())
            .unwrap();
        let (mut rules, _, art, _) = owner
            .load_scenario(
                NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::empty(),
            )
            .unwrap()
            .into_parts();
        rules.install_art_data(ArtRegistry::from_ini(&art));
        let mut world = crate::sim::arena_fixture::rival_arena(&rules, 31);
        bind_scenario_gameplay_resources(
            &mut rules,
            &assets,
            ScenarioGameplayProgram::Fresh {
                simulation: &mut world,
                rules_owner: &mut owner,
            },
            "tem",
            "TEMPERATE",
        )
        .unwrap();
        assert_eq!(
            rules.airburst_tail_policy(),
            if refused {
                AirburstTailPolicy::UnsupportedReplacement
            } else {
                AirburstTailPolicy::RetailFixedNine
            }
        );
        // The shared arena helper installs the same map owners used by the
        // existing EMP saved-receiver test before any saved-world continuation.
        crate::sim::arena_fixture::prepare_empty_map_authority(&mut world, &rules);
        world.intern_rule_type_ids(&rules);
        world.resolve_type_handles(&rules);
        world.session.set_binary_frame_for_test(100);
        let source = world
            .spawn_object("SOURCE", "Americans", 6, 20, 0, &rules)
            .unwrap();
        let victim = world
            .spawn_object("VICTIM", "Russians", 10, 20, 0, &rules)
            .unwrap();
        let before_launch = world.entities().get(victim).unwrap().health.current();
        // FireAt owns native identity, source links, Display/Logic membership
        // and Bullet admission. Keep the original non-Inviso parent type; only
        // its already-live Location is moved to the existing fan fixture point.
        world.commit_fire_visit(
            crate::sim::combat::world_fire::FireVisit::Direct {
                id: source,
                target: crate::sim::combat::TargetKind::Cell(10, 20),
                weapon_index: 0,
            },
            &rules,
            None,
            crate::sim::world::FrameEffects::empty(),
        );
        assert_eq!(
            world.projectiles.len(),
            1,
            "the real FireAt admits one live parent before save"
        );
        assert_eq!(
            world.entities().get(victim).unwrap().health.current(),
            before_launch,
            "the parent is admitted before its one later detonation"
        );
        let (&parent, _) = world.projectiles.iter().next().unwrap();
        let parent_bullet = world.projectiles.get_mut(parent).unwrap();
        assert!(
            parent_bullet.in_logic_vector(),
            "formal admission registered the parent"
        );
        parent_bullet.set_position_for_test(ProjectileCoord::new(2688, 5248, 0));
        let request = ProjectileDetonation {
            projectile_id: parent,
            source_id: parent_bullet.source_id(),
            target: parent_bullet.target(),
            impact: ProjectileCoord::new(2699, 5259, 10),
            payload: parent_bullet.payload(),
            reason: ProjectileDetonationReason::ReachedTarget,
        };
        assert_eq!(request.payload.base_damage, 37);
        assert_eq!(world.interner.resolve(request.payload.warhead), "CHILD_WH");
        if label == "airburst-current" {
            let hash = rules.simulation_config_hash();
            let terrain = world.resolved_terrain().unwrap().clone();
            let saved =
                save_codec::serialize(&(owner.saved_scenario_rules().unwrap(), &world)).unwrap();
            let (processed, mut restored): (ProcessedRulesLayers, Simulation) =
                save_codec::deserialize(&saved).unwrap();
            rules = owner.project_saved_scenario_rules(&processed).unwrap();
            rules.install_art_data(ArtRegistry::from_ini(&art));
            bind_scenario_gameplay_resources(
                &mut rules,
                &assets,
                ScenarioGameplayProgram::Saved(&restored),
                "tem",
                "TEMPERATE",
            )
            .unwrap();
            assert_eq!(
                rules.simulation_config_hash(),
                hash,
                "Saved binds the same frozen policy"
            );
            assert_eq!(
                rules.airburst_tail_policy(),
                AirburstTailPolicy::UnsupportedReplacement
            );
            // Raw deserialization intentionally omits map-derived caches.
            // Follow the same saved-load transaction as PreparedLoad and the
            // existing EMP receiver regression before collecting live victims.
            restored.retain_process_state_for_load(crate::sim::world::SnapshotProcessState::Live(
                &world,
            ));
            restored.bind_shared_cell_dummy(crate::map::resolved_terrain::SharedCellDummy::fresh());
            restored.reconstruct_cellclass_dummy_for_map_resize();
            restored.restore_after_snapshot_load().unwrap();
            restored.rebuild_caches_after_load(
                terrain,
                world.terrain_speed_config().clone(),
                &rules,
            );
            restored
                .restore_map_authority_after_snapshot_load(&rules, rules.overlay_registry(), None)
                .unwrap();
            restored.resolve_type_handles(&rules);
            owner.install_saved_scenario_rules(processed, Some(&rules));
            world = restored;
        }
        world.scenario_rng = SimRng::from_native_state_hex_for_test(
            native["rng_before"]["scenario"].as_str().unwrap(),
        );
        let scenario = world.scenario_rng.native_state_hex();
        let main = world.main_rng.native_state_hex();
        let mapgen = world.mapgen_rng.native_state_hex();
        let before = world
            .substrate
            .entities
            .get(victim)
            .unwrap()
            .health
            .current();
        let (_, visits) = crate::sim::combat::receiver_fixture::trace_area_visits(|| {
            detonate(&mut world, &rules, request)
        });
        assert_eq!(visits.len(), 1, "the actual parent entered AreaDamage once");
        assert_eq!(visits[0]["warhead"], "CHILD_WH");
        assert_eq!(visits[0]["source_id"], source);
        assert!(visits[0]["damage"].as_i64().unwrap() > 0);
        assert!(
            world
                .substrate
                .entities
                .get(victim)
                .unwrap()
                .health
                .current()
                < before,
            "refusing only the fan must preserve parent damage"
        );
        assert!(
            world.projectiles.get(request.projectile_id).is_some(),
            "the represented parent survives the tail"
        );
        let mut children: Vec<_> = world
            .projectiles
            .iter()
            .filter(|(id, _)| **id != request.projectile_id)
            .collect();
        children.sort_by_key(|(id, _)| **id);
        assert_eq!(
            children.len(),
            if refused {
                0
            } else {
                native["children"].as_array().unwrap().len()
            }
        );
        if refused {
            assert_eq!(
                world.scenario_rng.native_state_hex(),
                scenario,
                "refusal creates no child and draws no fan RNG"
            );
        } else {
            for ((_, child), expected) in
                children.iter().zip(native["children"].as_array().unwrap())
            {
                let target = &expected["target"];
                assert_eq!(
                    child.target(),
                    ProjectileTarget::Cell {
                        rx: target[0].as_u64().unwrap() as u16,
                        ry: target[1].as_u64().unwrap() as u16
                    }
                );
                assert_eq!(child.source_id(), source);
                assert_eq!(child.payload().base_damage, 37);
                assert_eq!(child.payload().damage_multiplier(), 256);
                assert!(!child.payload().bright());
                assert!(child.payload().installed_weapon().is_none());
            }
            assert_eq!(
                world.scenario_rng.native_state_hex(),
                native["rng_after"]["scenario"],
                "explicit Retail keeps the existing nine-child RNG draws even on a legacy host"
            );
        }
        assert_eq!(world.main_rng.native_state_hex(), main);
        assert_eq!(world.mapgen_rng.native_state_hex(), mapgen);
    }
}
