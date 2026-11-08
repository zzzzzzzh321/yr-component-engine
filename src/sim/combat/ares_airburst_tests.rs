//! Ares AR268 through the ordinary represented-Bullet detonation receiver.
//! The clear arena/forced parent Location are supplied boundaries. The small
//! retarget controls below are native inputs/outputs, not whole-frame goldens.

use super::*;
use crate::rules::ini_parser::IniFile;
use crate::sim::projectile::ProjectileCoord;
use crate::sim::rng::SimRng;
use crate::sim::snapshot::GameSnapshot;

fn component_bytes(variable: &str) -> Vec<u8> {
    std::fs::read(std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable}"))).unwrap()
}

fn bind_component(mut rules: RuleSet, component: &[u8]) -> RuleSet {
    let names = rules
        .all_objects()
        .map(|object| object.id.clone())
        .collect::<Vec<_>>();
    let program = crate::mod_sdk::Program::load_with_registries(
        component.to_vec(),
        &crate::util::sha256::sha256_hex(component),
        rules.sdk_configuration_passes(),
        &names,
        &[],
        &rules.weapon_names(),
        rules.sdk_configuration_registries(),
    )
    .unwrap();
    rules.external_mod = Some(std::sync::Arc::new(program));
    rules.bind_resource_policies().unwrap();
    rules
}

fn patch(keys: &str) -> IniFile {
    let mut ini = IniFile::from_str(super::tests::rules_ini());
    // Edit the selected physical bodies. Appending duplicate section headers
    // in one file does not mean that native reads take the last occurrence.
    ini.merge_source_sections(
        &IniFile::from_str("[CHILD_GUN]\nProjectileRange=5\n[CHILD_SHOT]\nRanged=yes\n"),
        true,
    );
    ini.merge_source_sections(&IniFile::from_str(keys), true);
    ini
}

fn child_ids(world: &Simulation, parent: u64) -> Vec<u64> {
    let mut ids = world
        .projectiles
        .iter()
        .filter_map(|(id, _)| (*id != parent).then_some(*id))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

#[test]
#[ignore = "requires retail INIs and actual .31/.30 VERA_SDK_AIRBURST[_PREVIOUS]_WASM"]
fn ares_airburst_retail_reader_receiver_and_frozen_snapshot() {
    let current = component_bytes("VERA_SDK_AIRBURST_WASM");
    let previous = component_bytes("VERA_SDK_AIRBURST_PREVIOUS_WASM");
    let patch = patch("");
    // Use the physical production INI layering/reader before asking the real
    // guest to bind resources. No hand-built Plan stands in for Configure.
    let retail = crate::rules::retail_ini_fixture::retail_battle_rules_for_map_with_patch(
        "Hills.mmx",
        &patch,
    )
    .expect("supply retail INIs for the production reader");
    let legacy = bind_component(retail.rules.clone(), &previous);
    assert_eq!(
        legacy.airburst_tail_policy(),
        vera_mod_api::AirburstTailPolicy::UnsupportedReplacement,
        ".30 has no admitted replacement fan"
    );
    let (mut control, legacy, _, request) = super::tests::scene_with_rules(600, legacy);
    let before = control.scenario_rng.logical_state();
    super::tests::detonate(&mut control, &legacy, request);
    assert!(child_ids(&control, request.projectile_id).is_empty());
    assert_eq!(control.scenario_rng.logical_state(), before);

    let rules = bind_component(retail.rules, &current);
    assert_eq!(
        rules.airburst_tail_policy(),
        vera_mod_api::AirburstTailPolicy::AresReplacement
    );
    let policy = rules.projectile("PARENT_SHOT").unwrap().airburst_policy();
    assert_eq!(policy.unwrap().spread_bits, 1.5_f64.to_bits());
    let (mut world, rules, source, request) = super::tests::scene_with_rules(600, rules);
    crate::sim::arena_fixture::prepare_empty_map_authority(&mut world, &rules);
    super::tests::detonate(&mut world, &rules, request);
    let ids = child_ids(&world, request.projectile_id);
    let expected = [
        (9, 19),
        (10, 19),
        (11, 19),
        (9, 20),
        (10, 20),
        (11, 20),
        (9, 21),
        (10, 21),
        (11, 21),
    ];
    assert_eq!(
        ids.len(),
        expected.len(),
        "Spread overrides author Cluster=2"
    );
    let weapon = rules.weapon("CHILD_GUN").unwrap();
    let fuel = weapon.firing_policy().unwrap().projectile_range.unwrap();
    assert_eq!(fuel, 5 * 256);
    for (&id, (rx, ry)) in ids.iter().zip(expected) {
        let child = world.projectiles.get(id).unwrap();
        assert_eq!(child.target(), ProjectileTarget::Cell { rx, ry });
        assert_eq!(child.launch_origin(), ProjectileCoord::new(2688, 5248, 600));
        assert_ne!(child.launch_origin(), request.impact);
        assert_eq!(child.source_id(), source);
        assert_eq!(child.payload().base_damage, weapon.damage);
        assert_eq!(child.payload().damage_multiplier(), 256);
        assert!(child.payload().bright());
        assert!(child.payload().installed_weapon().is_none());
        assert_eq!(
            child.velocity().native()[2].bits(),
            f64::from(-weapon.speed).to_bits()
        );
        assert_eq!(child.fuel_remaining(), Some(fuel));
        assert!(!child.on_bridge());
        assert!(child.in_logic_vector());
    }
    assert!(world.projectiles.get(request.projectile_id).is_some());

    let hash = rules.simulation_config_hash();
    let saved = GameSnapshot::save_validated(&world, 0, hash, "Ares Airburst children", 0);
    let mut restored = GameSnapshot::load_validated(&saved, 0, hash, world.session.map_name())
        .unwrap()
        .sim;
    assert_eq!(child_ids(&restored, request.projectile_id), ids);
    let mut rebound = rules.clone();
    // Saved guest/config identity is rebound without Configure or rereading INI.
    rebound.external_mod = Some(std::sync::Arc::new(restored.external_mod.clone().unwrap()));
    rebound.bind_resource_policies().unwrap();
    assert_eq!(rebound.simulation_config_hash(), hash);
    assert_eq!(
        rebound.projectile("PARENT_SHOT").unwrap().airburst_policy(),
        policy
    );
    for &id in &ids {
        assert_eq!(
            restored.projectiles.get(id).unwrap().fuel_remaining(),
            Some(fuel)
        );
        assert_eq!(
            restored.projectiles.get(id).unwrap().target(),
            world.projectiles.get(id).unwrap().target()
        );
    }
    // This checks the owning saved counter/hash, not a claim that deserialize
    // has already performed the app's full cold-map/cache restoration.
    let before = restored.state_hash();
    restored.projectiles.set_weapon_fuel(ids[0], fuel - 1);
    assert_ne!(restored.state_hash(), before);
    restored.projectiles.set_weapon_fuel(ids[0], fuel);
    assert_eq!(restored.state_hash(), before);
}

fn placed(world: &mut Simulation, rules: &RuleSet, owner: &str, x: u16, y: u16) -> u64 {
    let id = world.spawn_object("VICTIM", owner, x, y, 0, rules).unwrap();
    world.add_entity_occupancy(id);
    world
        .substrate
        .entities
        .get_mut(id)
        .unwrap()
        .set_in_playfield(true);
    id
}

#[test]
#[ignore = "requires actual .31 VERA_SDK_AIRBURST_WASM"]
fn ares_splits_receiver_filters_ctor_order_single_tail_and_shared_dummy() {
    let current = component_bytes("VERA_SDK_AIRBURST_WASM");
    let ini = patch(
        "[PARENT_SHOT]\nAirburst=no\nSplits=yes\nAA=yes\nCluster=3\n\
         RetargetSelf=no\nAroundTarget=no\n\
         [CHILD_SHOT]\nAA=no\n[CHILD_WH]\nAffectsAllies=yes\nAffectsEnemies=no\n",
    );
    let rules = bind_component(RuleSet::from_ini(&ini).unwrap(), &current);
    let delivery = super::super::projectile_delivery::classify_projectile_delivery(
        rules.weapon("PARENT_GUN").unwrap(),
        &rules,
    );
    assert!(!rules.projectile("PARENT_SHOT").unwrap().airburst);
    assert!(
        delivery.collision.explodes_airburst(),
        "Splits-only arms the Explodes gate"
    );
    let (mut world, rules, source, mut request) = super::tests::scene_with_rules(600, rules);
    world.add_entity_occupancy(source);
    world
        .substrate
        .entities
        .get_mut(source)
        .unwrap()
        .set_in_playfield(true);
    // Deliberately not spatial order.
    let a = placed(&mut world, &rules, "Americans", 12, 20);
    let b = placed(&mut world, &rules, "Americans", 10, 21);
    let c = placed(&mut world, &rules, "Americans", 9, 20);
    let enemy = placed(&mut world, &rules, "Russians", 10, 19);
    let unmarked = world
        .spawn_object("VICTIM", "Americans", 11, 20, 0, &rules)
        .unwrap();
    world.remove_entity_occupancy(unmarked);
    world
        .substrate
        .entities
        .get_mut(unmarked)
        .unwrap()
        .set_in_playfield(true);
    assert!(
        !world
            .substrate
            .entities
            .get(unmarked)
            .unwrap()
            .lifecycle
            .cell_marked()
    );
    let dead = placed(&mut world, &rules, "Americans", 11, 21);
    world
        .substrate
        .entities
        .get_mut(dead)
        .unwrap()
        .health
        .set_for_test(0);
    let far = placed(&mut world, &rules, "Americans", 22, 20);
    let high = placed(&mut world, &rules, "Americans", 10, 22);
    Simulation::set_object_height_unmarked(
        &mut world.substrate,
        &world.resolved_terrain,
        high,
        600,
    );
    assert!(crate::sim::movement::air_movement::is_high_flying(
        world.substrate.entities.get(high).unwrap(),
        world.resolved_terrain.as_ref(),
        Some((&rules, &world.interner)),
    ));
    request.target = ProjectileTarget::None;
    world
        .projectiles
        .get_mut(request.projectile_id)
        .unwrap()
        .set_target_for_test(request.target);
    world.scenario_rng = SimRng::new(1);
    super::tests::detonate(&mut world, &rules, request);
    let ids = child_ids(&world, request.projectile_id);
    assert_eq!(
        ids.len(),
        3,
        "the outer Cluster loop must not repeat the Splits tail"
    );
    let targets = ids
        .iter()
        .map(|id| world.projectiles.get(*id).unwrap().target())
        .collect::<Vec<_>>();
    assert_eq!(
        targets[0],
        ProjectileTarget::Entity(b),
        "native seed1 picks ctor slot1"
    );
    for id in [a, b, c] {
        assert!(targets.contains(&ProjectileTarget::Entity(id)));
    }
    for id in [source, enemy, unmarked, dead, far, high] {
        assert!(!targets.contains(&ProjectileTarget::Entity(id)));
    }
    // The surviving source lies inside the radius and shares the allowed
    // House: excluding it therefore exercises RetargetSelf, not distance/WH.
    assert!(
        world
            .substrate
            .entities
            .get(source)
            .unwrap()
            .health
            .current()
            > 0
    );

    let (mut edge, rules, _, mut edge_request) = super::tests::scene_with_rules(600, rules);
    let location = ProjectileCoord::new(-10240, -10240, 600);
    edge_request.target = ProjectileTarget::None;
    edge_request.impact = location;
    let parent = edge
        .projectiles
        .get_mut(edge_request.projectile_id)
        .unwrap();
    parent.set_position_for_test(location);
    parent.set_target_for_test(ProjectileTarget::None);
    super::tests::detonate(&mut edge, &rules, edge_request);
    let children = child_ids(&edge, edge_request.projectile_id);
    assert_eq!(children.len(), 3);
    assert!(
        children
            .iter()
            .all(|id| edge.projectiles.get(*id).unwrap().target() == ProjectileTarget::DummyCell)
    );
    let launch_target = edge.projectiles.get(children[0]).unwrap().launch_target();
    // A later canonical miss stamps the same CellClass singleton retained by
    // every filled slot. Targets must not freeze each fill's coordinates.
    assert_eq!(
        crate::map::resolved_terrain::NativeCellQuery::canonical(
            edge.resolved_terrain.as_ref().unwrap()
        )
        .lookup((-123, -124)),
        crate::map::cell_index::NativeCellIdentity::Dummy,
    );
    let current_coord =
        crate::sim::projectile::dummy_cell_ground_coord(&edge.effective_shared_cell_dummy());
    for id in children {
        let target = edge.projectiles.get(id).unwrap().target();
        assert_eq!(
            super::super::projectile_delivery::target_get_coords(&edge, target),
            Some(current_coord)
        );
    }
    assert_eq!(
        edge.projectiles
            .get(child_ids(&edge, edge_request.projectile_id)[0])
            .unwrap()
            .launch_target(),
        launch_target
    );
}

#[test]
fn ares_split_retarget_native_controls_and_original_target_retention() {
    // Native Ares DLL bounded controls, prereq-controls.json: non-null seed0
    // consumes probability+index; null seed1 skips the probability call.
    let candidates = [
        ProjectileTarget::Entity(1),
        ProjectileTarget::Entity(2),
        ProjectileTarget::Entity(3),
    ];
    for (seed, original, source, expected, remaining, raw) in [
        (
            0,
            ProjectileTarget::Entity(99),
            1,
            candidates[2],
            vec![candidates[0], candidates[1]],
            vec![1229352179, 1781224790],
        ),
        (
            1,
            ProjectileTarget::None,
            1,
            candidates[1],
            vec![candidates[0], candidates[2]],
            vec![2025287381],
        ),
        // Additional original controls select the source itself, so the
        // owner probability draw occurs. Both observed values are < 0.5:
        // retain that selected slot without a second index draw.
        (
            0,
            ProjectileTarget::Entity(99),
            3,
            candidates[2],
            vec![candidates[0], candidates[1]],
            vec![1229352179, 1781224790, 2424954917],
        ),
        (
            1,
            ProjectileTarget::None,
            2,
            candidates[1],
            vec![candidates[0], candidates[2]],
            vec![2025287381, 660436142],
        ),
    ] {
        let mut rng = SimRng::new(seed);
        let mut targets = candidates.to_vec();
        assert_eq!(
            select_split_target(&mut rng, original, source, 0.0, &mut targets),
            expected
        );
        assert_eq!(targets, remaining, "retarget consumes by swap-last");
        let mut native_stream = SimRng::new(seed);
        for draw in raw {
            assert_eq!(native_stream.next_u32(), draw);
        }
        assert_eq!(rng.logical_state(), native_stream.logical_state());
    }
    let mut rng = SimRng::new(0);
    let original = ProjectileTarget::Entity(99);
    let mut targets = candidates.to_vec();
    assert_eq!(
        select_split_target(&mut rng, original, 1, 1.0, &mut targets),
        original
    );
    assert_eq!(
        targets, candidates,
        "retaining the original target does not consume a candidate"
    );
    let mut one_probability = SimRng::new(0);
    one_probability.random_double();
    assert_eq!(rng.logical_state(), one_probability.logical_state());
}
