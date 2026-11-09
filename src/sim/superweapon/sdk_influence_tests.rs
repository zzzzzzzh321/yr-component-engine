//! Actual .38/.37 components, the physical Rules readers, paid typed production
//! and the ordinary common-target owner. No synthetic actor Type or power facts.
#![cfg(not(any(feature = "ares", feature = "phobos")))]
use super::*;
use crate::rules::object_type::ObjectCategory;
use crate::rules::superweapon_targeting::TargetReject;
use crate::sim::command::{CommandEnvelope, ProductionEvent};
use crate::sim::production::{ProductionCategory, ProductionSelection};
use crate::sim::world::{TickLane, UninitContext};
use vera_mod_api::{ProductionObjectKind, ProductionTypeRef, SuperWeaponResourcePolicy};

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

fn root() -> String {
    // Native readers select a physical section. Extend the original registry
    // and SW bodies rather than appending duplicate section headers.
    let base = BASE
        .replace(
            "[InfantryTypes]\n",
            "[InfantryTypes]\n0=ABCDEFGHIJKLMNOPQRSTUVWX_FIRST\n1=ABCDEFGHIJKLMNOPQRSTUVWX_SECOND\n2=SCOUT\n3=ZERO\n4=NEGATIVE\n",
        )
        .replace(
            "2=DELIVERYBASE\n",
            "2=DELIVERYBASE\n3=BARRACKS\n4=BLOCKER\n5=PLANT\n6=PROVIDERABCDEFGHIJKLMNOP_FIRST\n",
        )
        .replace(
            "SW.RequiresTarget=building\nSW.RequiresHouse=enemies\n",
            &format!("SW.RequiresTarget=none\nSW.RequiresHouse=none\nSW.AITargeting.Constraints=none\nSW.Designators={STORED}\n"),
        )
        .replace(
            "[Reveal]\nType=PsychicReveal\nIsPowered=no\n",
            "[Reveal]\nType=PsychicReveal\nIsPowered=no\nSW.Designators=ZERO,NEGATIVE\nSW.FireIntoShroud=yes\n",
        );
    format!(
        "{base}\n\
         [General]\nBuildSpeed=.1\nParanoid=no\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nLocomotor={{4A582744-9839-11d1-B709-00A024DDAFD1}}\nStrength=125\nCost=200\nSpeed=4\nSight=13\nTechLevel=1\nOwner=Americans\n\
         [SCOUT]\nLocomotor={{4A582744-9839-11d1-B709-00A024DDAFD1}}\nStrength=125\nSpeed=4\nSight=13\n\
         [ZERO]\nLocomotor={{4A582744-9839-11d1-B709-00A024DDAFD1}}\nStrength=125\nSight=13\nDesignatorRange=0\nInhibitorRange=0\n\
         [NEGATIVE]\nLocomotor={{4A582744-9839-11d1-B709-00A024DDAFD1}}\nStrength=125\nSight=13\nDesignatorRange=-1\nInhibitorRange=-1\n\
         [BARRACKS]\nImage=LAUNCHER\nStrength=1000\nFactory=InfantryType\nOwner=Americans,Russians\n\
         [BLOCKER]\nImage=LAUNCHER\nStrength=200\nPower=-100\nPowered=yes\nSight=13\nDesignatorRange=0\nInhibitorRange=3\n\
         [PLANT]\nImage=LAUNCHER\nStrength=1000\nPower=200\n\
         [PROVIDERABCDEFGHIJKLMNOP]\nSuperWeapons=Strike\n"
    )
}

fn resource(scene: &Scene) -> SuperWeaponResourcePolicy {
    scene
        .sim
        .external_mod
        .as_ref()
        .unwrap()
        .superweapon_resource_policy()
        .unwrap()
        .unwrap()
}

fn infantry(ordinal: u32) -> ProductionTypeRef {
    ProductionTypeRef {
        category: ProductionObjectKind::Infantry,
        ordinal,
    }
}

fn preview(scene: &Scene, name: &str, cell: (u16, u16)) -> Result<(), TargetReject> {
    scene.sim.super_weapon_target_check(
        &scene.rules,
        scene.id("Americans"),
        scene.id(name),
        cell,
        false,
    )
}

// The app's same retained strip identities and ordinary bound-resource frame.
// This is only invocation glue; Factory/Place/Exit and power remain production owners.
fn frame(scene: &mut Scene, commands: &[CommandEnvelope]) {
    let visits = scene
        .sim
        .production
        .factories()
        .strip_visits_for_test(scene.id("Americans"));
    scene
        .sim
        .advance_app_frame(
            commands,
            &scene.rules,
            Some(scene.rules.overlay_registry()),
            67,
            TickLane::Ordinary,
            FrameEffects::default().with_production_strip(&visits, &|_| {}),
        )
        .unwrap();
}

fn paid_infantry(scene: &mut Scene, slot: i32) -> u64 {
    let owner = scene.id("Americans");
    let object = scene
        .rules
        .type_array_at(ObjectCategory::Infantry, slot)
        .unwrap();
    let selection = ProductionSelection::for_type(&scene.rules, object).unwrap();
    let record = scene
        .sim
        .encode_selected_production_record(owner, ProductionEvent::Produce, selection)
        .unwrap();
    let command = CommandEnvelope::synchronized(record, scene.sim.session.tick() + 1);
    frame(scene, &[command]);
    let product = scene
        .sim
        .production
        .factories()
        .view(owner, ProductionCategory::Infantry)
        .unwrap()
        .object
        .as_ref()
        .unwrap()
        .entity_id
        .unwrap();
    for _ in 0..2048 {
        frame(scene, &[]);
        if !scene
            .sim
            .entities()
            .get(product)
            .unwrap()
            .lifecycle
            .in_limbo()
        {
            return product;
        }
    }
    let factory = scene
        .sim
        .production
        .factories()
        .view(owner, ProductionCategory::Infantry)
        .map(|factory| {
            (
                factory.progress,
                factory.on_hold,
                factory.suspended,
                factory.is_running(),
                factory.object.and_then(|object| object.entity_id),
            )
        });
    let producer =
        crate::sim::production::object_find_factory(&scene.sim, &scene.rules, product, false, true);
    let dock = producer.and_then(|producer| {
        crate::sim::docking::building_dock::building_dock_cell(
            &scene.sim,
            producer,
            product,
            (0, 0),
            &scene.rules,
            None,
        )
    });
    panic!(
        "paid Infantry slot {slot} did not reach ordinary Place/Exit: factory(progress,hold,suspended,running,object)={factory:?}, producer={producer:?}, dock={dock:?}, credits={}, won={}, lost={}, outcome_wait={:?}",
        scene.sim.houses[&owner].economy().credits(),
        scene.sim.houses[&owner].has_won(),
        scene.sim.houses[&owner].has_lost(),
        scene.sim.pending_outcome_wait(),
    );
}

#[test]
#[ignore = "requires VERA_SDK_INFLUENCE_WASM (.38) and VERA_SDK_INFLUENCE_PREVIOUS_WASM (.37)"]
fn sdk_influence_physical_prefix_retention_and_previous_cohort() {
    let current = component("VERA_SDK_INFLUENCE_WASM");
    let previous = component("VERA_SDK_INFLUENCE_PREVIOUS_WASM");
    let root = root()
        .replace(
            &format!("SW.Designators={STORED}\n"),
            &format!("SW.Designators={STORED},LATE,{STORED}\n"),
        )
        .replace(
            "SW.Designators=ZERO,NEGATIVE\n",
            "SW.Designators=SCOUT\nSW.Inhibitors=SCOUT\nSW.AnyDesignator=yes\n",
        )
        .replace(
            "[SCOUT]\n",
            "[SCOUT]\nDesignatorRange=13\nInhibitorRange=0\n",
        );
    let mode = "[Strike]\nSW.Designators= \t \nSW.Inhibitors=LATE\n\
                [Reveal]\nSW.AnyDesignator=invalid\n\
                [SCOUT]\nDesignatorRange=invalid\nInhibitorRange=invalid\n";
    let map = "[InfantryTypes]\n5=LATE\n[LATE]\nStrength=100\n\
               [BuildingTypes]\n7=PROVIDERABCDEFGHIJKLMNOP_SECOND\n\
               [Strike]\nSW.Designators=\nSW.Inhibitors=LATE\n\
               [Reveal]\nSW.Designators=none\nSW.Inhibitors=,,,\n\
               [SCOUT]\nDesignatorRange=\nInhibitorRange=\n";
    // ReadINI1003396E/Find10007EA0 may only see the actual construction prefix.
    // A later allocation must not repair the root's unknown LATE token.
    let prefix = Scene::with_mode("influence-prefix", Some(&current), &root, Some(mode), "");
    let policy = resource(&prefix);
    let strike = policy
        .definitions
        .iter()
        .find(|sw| sw.stored_id == "Strike")
        .unwrap();
    assert_eq!(
        strike.influence.as_ref().unwrap().designators,
        [infantry(0), infantry(0)]
    );
    assert!(strike.influence.as_ref().unwrap().inhibitors.is_empty());

    let scene = Scene::with_mode("influence-physical", Some(&current), &root, Some(mode), map);
    let policy = resource(&scene);
    let strike = policy
        .definitions
        .iter()
        .find(|sw| sw.stored_id == "Strike")
        .unwrap();
    let influence = strike.influence.as_ref().unwrap();
    assert_eq!(influence.designators, [infantry(0), infantry(0)]);
    assert_eq!(influence.inhibitors, [infantry(5)]);
    // GetSuperWeapon1000D610 follows this actual Building Type, including a
    // constructor-only later alias. Publishing its empty additional vector
    // must not overwrite the first slot through another stored-name Find.
    let first = scene
        .rules
        .type_array_at(ObjectCategory::Building, 6)
        .unwrap();
    let second = scene
        .rules
        .type_array_at(ObjectCategory::Building, 7)
        .unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(
        building_super_weapon_slots(&scene.rules, Some(first)).collect::<Vec<_>>(),
        [None, None, Some("Strike")]
    );
    assert_eq!(
        building_super_weapon_slots(&scene.rules, Some(second)).collect::<Vec<_>>(),
        [None, None]
    );
    let reveal = policy
        .definitions
        .iter()
        .find(|sw| sw.stored_id == "Reveal")
        .unwrap();
    let influence = reveal.influence.as_ref().unwrap();
    assert!(influence.designators.is_empty() && influence.inhibitors.is_empty());
    assert!(
        influence.any_designator,
        "invalid later bool retains the physical root"
    );
    let scout = policy
        .techno_influence
        .as_ref()
        .unwrap()
        .iter()
        .find(|ty| ty.object == infantry(2))
        .unwrap();
    assert_eq!(
        (scout.designator_range, scout.inhibitor_range),
        (Some(13), Some(0))
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 20)),
        Err(TargetReject::NoEligibleDesignator)
    );

    let mut old = Scene::with_mode("influence-old", Some(&previous), &root, Some(mode), map);
    assert!(resource(&old).techno_influence.is_none());
    assert!(
        resource(&old)
            .definitions
            .iter()
            .all(|sw| sw.influence.is_none())
    );
    assert_eq!(preview(&old, "Strike", (20, 20)), Ok(()));
    old.roundtrip();
    assert!(resource(&old).techno_influence.is_none());
    assert_eq!(preview(&old, "Strike", (20, 20)), Ok(()));
}

#[test]
#[ignore = "requires VERA_SDK_INFLUENCE_WASM (.38)"]
fn sdk_influence_paid_aliases_raw_sight_and_frozen_restore() {
    let current = component("VERA_SDK_INFLUENCE_WASM");
    let mut scene = Scene::new("influence-paid-alias", Some(&current), &root(), "");
    let owner = scene.id("Americans");
    scene.sim.session.set_current_house_for_test(Some(owner));
    scene.launcher_at(12, 12);
    scene
        .sim
        .spawn_object_at_height("BARRACKS", "Americans", 16, 16, 0, 0, &scene.rules)
        .unwrap();
    // Match the paid-production fixture's live opposing base. Empty rival
    // Houses would otherwise be defeated during this ordinary-frame loop.
    scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 26, 26, 0, 0, &scene.rules)
        .unwrap();
    assert!(scene.sim.rebuild_dynamic_navigation(&scene.rules));
    assert_eq!(
        scene
            .rules
            .type_array_at(ObjectCategory::Infantry, 0)
            .unwrap()
            .id,
        STORED
    );
    assert_eq!(
        scene
            .rules
            .type_array_at(ObjectCategory::Infantry, 1)
            .unwrap()
            .id,
        STORED
    );
    let credits = scene.sim.houses[&owner].economy().credits();
    let second = paid_infantry(&mut scene, 1);
    let actor = scene.sim.entities().get(second).unwrap();
    assert!(std::ptr::eq(
        scene.sim.entity_object_type(actor, &scene.rules).unwrap(),
        scene
            .rules
            .type_array_at(ObjectCategory::Infantry, 1)
            .unwrap()
    ));
    let [x, y] = crate::sim::movement::ground_pose::object_center_xy(actor);
    assert_eq!(
        preview(&scene, "Strike", ((x / 256) as u16, (y / 256) as u16)),
        Err(TargetReject::NoEligibleDesignator),
        "the second stored24 alias must not match the first Type pointer"
    );

    let first = paid_infantry(&mut scene, 0);
    let actor = scene.sim.entities().get(first).unwrap();
    let ty = scene.sim.entity_object_type(actor, &scene.rules).unwrap();
    assert!(std::ptr::eq(
        ty,
        scene
            .rules
            .type_array_at(ObjectCategory::Infantry, 0)
            .unwrap()
    ));
    assert_eq!(ty.sight, 13);
    assert_eq!(ty.designator_range(), None);
    assert_eq!(actor.sight.range(), 13);
    assert_eq!(scene.sim.houses[&owner].economy().credits(), credits - 400);
    let [x, y] = crate::sim::movement::ground_pose::object_center_xy(actor);
    let (cx, cy) = ((x / 256) as u16, (y / 256) as u16);
    let [sx, sy] = crate::sim::movement::ground_pose::object_center_xy(
        scene.sim.entities().get(second).unwrap(),
    );
    let direction = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .filter(|&(dx, dy)| {
            (0..32).contains(&(i32::from(cx) + 14 * dx))
                && (0..32).contains(&(i32::from(cy) + 14 * dy))
        })
        .max_by_key(|&(dx, dy)| {
            let x = i32::from(cx) + 13 * dx - sx / 256;
            let y = i32::from(cy) + 13 * dy - sy / 256;
            x * x + y * y
        })
        .unwrap();
    let at = |range| {
        (
            (i32::from(cx) + range * direction.0) as u16,
            (i32::from(cy) + range * direction.1) as u16,
        )
    };
    let (visible_edge, edge, outside) = (at(10), at(13), at(14));
    // Prepare1006D414 uses the actual Type's raw Sight; CanFireAt1006CE80
    // includes the squared-distance boundary, independently of vision's cap10.
    let hash = scene.sim.state_hash();
    assert!(
        scene
            .sim
            .fog
            .is_cell_visible(owner, visible_edge.0, visible_edge.1)
    );
    assert!(
        !scene.sim.fog.is_cell_visible(owner, edge.0, edge.1),
        "the independent vision walk remains capped at10"
    );
    assert_eq!(preview(&scene, "Strike", edge), Ok(()));
    assert_eq!(
        preview(&scene, "Strike", outside),
        Err(TargetReject::NoEligibleDesignator)
    );
    assert_eq!(scene.sim.state_hash(), hash, "preview is read-only");

    for (name, cell) in [("ZERO", (20, 20)), ("NEGATIVE", (22, 20))] {
        scene
            .sim
            .spawn_object_at_height(name, "Americans", cell.0, cell.1, 0, 0, &scene.rules)
            .unwrap();
        assert_eq!(
            preview(&scene, "Reveal", cell),
            Err(TargetReject::NoEligibleDesignator),
            "explicit signed nonpositive range disables its source"
        );
    }
    // Restore the retained physical source and exact saved Program, despite a
    // changed current file that would disable both alias slots on a fresh read.
    std::fs::write(
        scene.dir.path().join("rulesmd.ini"),
        root().replace(
            &format!("[{STORED}]\n"),
            &format!("[{STORED}]\nDesignatorRange=0\n"),
        ),
    )
    .unwrap();
    scene.assets = open(&scene.dir);
    scene.roundtrip();
    assert_eq!(preview(&scene, "Strike", edge), Ok(()));
    let actor = scene.sim.entities().get(first).unwrap();
    assert!(std::ptr::eq(
        scene.sim.entity_object_type(actor, &scene.rules).unwrap(),
        scene
            .rules
            .type_array_at(ObjectCategory::Infantry, 0)
            .unwrap()
    ));
    scene
        .sim
        .uninit_with_context(first, UninitContext::with_rules(&scene.rules));
    frame(&mut scene, &[]);
    assert!(scene.sim.entities().contains(second));
    assert_eq!(
        preview(&scene, "Strike", edge),
        Err(TargetReject::NoEligibleDesignator),
        "the surviving same-class alias must remain a different Type after restore"
    );
}

#[test]
#[ignore = "requires VERA_SDK_INFLUENCE_WASM (.38)"]
fn sdk_influence_live_building_power_relations_lifecycle_and_event_boundary() {
    let current = component("VERA_SDK_INFLUENCE_WASM");
    let map = "[Strike]\nSW.AnyDesignator=yes\nSW.AnyInhibitor=yes\n\
               SW.RequiresTarget=buildings\nSW.RequiresHouse=none\nSW.AIRequiresTarget=buildings\nSW.AIRequiresHouse=enemies\nSW.AITargeting=Offensive\n";
    let mut scene = Scene::new("influence-live", Some(&current), &root(), map);
    let (owner, enemy, sw) = (
        scene.id("Americans"),
        scene.id("Russians"),
        scene.id("Strike"),
    );
    scene.launcher_at(12, 12);
    let scout = scene
        .sim
        .spawn_object_at_height("SCOUT", "Americans", 16, 16, 0, 0, &scene.rules)
        .unwrap();
    let target = scene
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 16, 0, 0, &scene.rules)
        .unwrap();
    let blocker = scene
        .sim
        .spawn_object_at_height("BLOCKER", "Russians", 23, 16, 0, 0, &scene.rules)
        .unwrap();
    scene
        .sim
        .spawn_object_at_height("PLANT", "Russians", 26, 26, 0, 0, &scene.rules)
        .unwrap();
    // Keep a third live House so the real CanAlly501540 owner permits a live
    // alliance. This is a scene prior, not a direct ally-mask mutation.
    scene
        .sim
        .spawn_object_at_height("TARGET", "Neutral", 6, 6, 0, 0, &scene.rules)
        .unwrap();
    for (name, cell) in [("ZERO", (20, 18)), ("NEGATIVE", (22, 18))] {
        scene
            .sim
            .spawn_object_at_height(name, "Russians", cell.0, cell.1, 0, 0, &scene.rules)
            .unwrap();
    }
    frame(&mut scene, &[]);
    assert_eq!(
        scene.sim.building_operational_state(blocker, &scene.rules),
        Some(true)
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange),
        "hostile Building covers its inclusive three-cell edge"
    );
    assert!(!crate::sim::superweapon::ai_fire::fire_extension_automatic(
        &mut scene.sim,
        &scene.rules,
        owner,
        sw,
        None,
        FrameEffects::default(),
        false
    ));
    assert_eq!(scene.sim.super_weapons[&owner][&sw].ares_launch_count, 0);

    // IsInhibitor1006D974 calls Building virtual350 =4555D0. Its independent
    // online latch must be read, rather than only the owner's power ratio.
    assert!(
        scene
            .sim
            .go_building_offline(blocker, &scene.rules, FrameEffects::default())
    );
    frame(&mut scene, &[]);
    assert_eq!(
        scene.sim.building_operational_state(blocker, &scene.rules),
        Some(false)
    );
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    assert!(
        scene
            .sim
            .go_building_online(blocker, &scene.rules, FrameEffects::default())
    );
    frame(&mut scene, &[]);
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange)
    );

    scene.sim.make_ally(
        enemy,
        owner,
        &scene.rules,
        None,
        FrameEffects::default(),
        false,
    );
    assert!(
        scene
            .sim
            .house_alliances
            .is_allied(&scene.sim.interner, enemy, owner)
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange),
        "the firing House's directed mask decides hostility"
    );
    scene.sim.make_ally(
        owner,
        enemy,
        &scene.rules,
        None,
        FrameEffects::default(),
        false,
    );
    assert!(
        scene
            .sim
            .house_alliances
            .is_allied(&scene.sim.interner, owner, enemy)
    );
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    scene.sim.make_enemy(
        owner,
        enemy,
        &scene.rules,
        None,
        FrameEffects::default(),
        false,
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange)
    );

    scene
        .sim
        .change_owner_with_rules(blocker, owner, &scene.rules, None, FrameEffects::default());
    frame(&mut scene, &[]);
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    scene
        .sim
        .change_owner_with_rules(blocker, enemy, &scene.rules, None, FrameEffects::default());
    frame(&mut scene, &[]);
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange)
    );
    scene
        .sim
        .techno_deactivate(blocker, &scene.rules, FrameEffects::default());
    // IsInhibitor1006D900 observes the current Deactivated latch. Query
    // before another object head: the existing EMP operability owner
    // (TechnoUpdate1004B528..575) Reactivates an operational, zero-EMP
    // Building on that next head, even after this direct Deactivate call.
    assert!(
        scene
            .sim
            .entities()
            .get(blocker)
            .unwrap()
            .deactivated
            .is_offline()
    );
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    scene
        .sim
        .techno_reactivate(blocker, &scene.rules, FrameEffects::default());
    assert!(
        !scene
            .sim
            .entities()
            .get(blocker)
            .unwrap()
            .deactivated
            .is_offline()
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::InhibitorInRange)
    );
    scene
        .sim
        .techno_limbo_with_context(blocker, UninitContext::with_rules(&scene.rules));
    frame(&mut scene, &[]);
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    scene
        .sim
        .techno_deactivate(scout, &scene.rules, FrameEffects::default());
    // IsDesignator1006D880 likewise reads the actual current latch; a later
    // owner transfer is a separate lifecycle boundary below.
    assert!(
        scene
            .sim
            .entities()
            .get(scout)
            .unwrap()
            .deactivated
            .is_offline()
    );
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::NoEligibleDesignator)
    );
    scene
        .sim
        .techno_reactivate(scout, &scene.rules, FrameEffects::default());
    assert_eq!(preview(&scene, "Strike", (20, 16)), Ok(()));
    scene
        .sim
        .change_owner_with_rules(scout, enemy, &scene.rules, None, FrameEffects::default());
    frame(&mut scene, &[]);
    assert_eq!(
        preview(&scene, "Strike", (20, 16)),
        Err(TargetReject::NoEligibleDesignator)
    );
    scene
        .sim
        .change_owner_with_rules(scout, owner, &scene.rules, None, FrameEffects::default());
    frame(&mut scene, &[]);
    assert!(crate::sim::superweapon::ai_fire::fire_extension_automatic(
        &mut scene.sim,
        &scene.rules,
        owner,
        sw,
        None,
        FrameEffects::default(),
        false
    ));
    assert!(scene.sim.entities().get(target).unwrap().health.current() < 200);
    assert_eq!(scene.sim.super_weapons[&owner][&sw].ares_launch_count, 1);

    // Common Activate10071689 does not rerun CanFireAt. Preserve .37's event
    // cohort: a cursor refusal is not a newly invented universal preflight.
    let mut event = Scene::new("influence-event", Some(&current), &root(), "");
    event.launcher_at(12, 12);
    let target = event
        .sim
        .spawn_object_at_height("TARGET", "Russians", 20, 20, 0, 0, &event.rules)
        .unwrap();
    assert_eq!(
        preview(&event, "Strike", (20, 20)),
        Err(TargetReject::NoEligibleDesignator)
    );
    assert!(event.fire("Strike", (20, 20)));
    assert!(event.sim.entities().get(target).unwrap().health.current() < 200);
}
