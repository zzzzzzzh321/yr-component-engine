//! Original BulletClass::DetonateAtCoord Airburst tail469EBA..46A290.

use crate::map::cell_index::NativeCellIdentity;
use crate::map::resolved_terrain::NativeCellQuery;
use crate::rules::ruleset::RuleSet;
use crate::sim::projectile::{ProjectileDetonation, ProjectilePayload, ProjectileTarget};
use crate::sim::world::Simulation;

/// The shared tail calls this after damage/debris, using live Bullet fields.
#[cfg_attr(feature = "phobos", allow(dead_code))]
/// Eight native compass neighbors followed by the retained center identity;
/// Cluster controls neither this count nor a second parent detonation.
pub(super) fn emit(world: &mut Simulation, rules: &RuleSet, detonation: &ProjectileDetonation) {
    match rules.airburst_tail_policy() {
        vera_mod_api::AirburstTailPolicy::RetailFixedNine => emit_retail(world, rules, detonation),
        vera_mod_api::AirburstTailPolicy::AresReplacement => emit_ares(world, rules, detonation),
        vera_mod_api::AirburstTailPolicy::UnsupportedReplacement => {}
    }
}

fn emit_retail(world: &mut Simulation, rules: &RuleSet, detonation: &ProjectileDetonation) {
    // Native always has an actual `this` Bullet. Receiver-only packets (for
    // example a bare death weapon) do not invent a fan from an impact point.
    let Some(parent) = world.projectiles.get(detonation.projectile_id) else {
        return;
    };
    let location = parent.position();
    let Some(kind) = parent
        .payload()
        .projectile_type()
        .and_then(|id| rules.projectile(world.interner.resolve(id)))
        .filter(|kind| kind.airburst)
    else {
        return;
    };
    let Some(weapon) = kind
        .airburst_weapon
        .as_deref()
        .and_then(|id| rules.weapon(id))
    else {
        return;
    };
    let Some(child) = weapon
        .projectile
        .as_deref()
        .and_then(|id| rules.projectile(id))
    else {
        return;
    };
    let Some(warhead) = weapon.warhead.as_deref().and_then(|id| rules.warhead(id)) else {
        return;
    };
    let Some(terrain) = world.resolved_terrain.as_ref() else {
        return;
    };
    let center = NativeCellQuery::canonical(terrain).lookup_world(location.x, location.y);
    for direction in 0..9 {
        let target = {
            let cells = NativeCellQuery::canonical(world.resolved_terrain.as_ref().unwrap());
            let cell = cells.neighbor(center, direction);
            match cell {
                NativeCellIdentity::Real(_) => {
                    let (x, y) = cells.coord(cell);
                    ProjectileTarget::Cell {
                        rx: x as u16,
                        ry: y as u16,
                    }
                }
                NativeCellIdentity::Dummy => ProjectileTarget::DummyCell,
            }
        };
        // Source can have expired in the parent's synchronous AreaDamage.
        // current_detonation follows the existing pointer-expiry authority;
        // do not retain a firing-side house or attacker snapshot here.
        let live = world.projectiles.current_detonation(*detonation);
        let native_unique_id = world.next_native_runtime_id();
        let bullet_id = world.substrate.allocate_stable_id();
        let payload = ProjectilePayload::new(
            weapon.damage,
            world.interner.intern(&warhead.id),
            Some(world.interner.intern(&child.id)),
            false,
        );
        // Native constructs before drawing. Reserve its ID in that order,
        // then let the existing admission construct/publish after the draw.
        // No world callback occurs between these points: RNG reads no object
        // state, and publication completes before the next child/Logic visit.
        let velocity = crate::sim::projectile::launch::airburst_launch_velocity(
            weapon.speed,
            &mut world.scenario_rng,
            crate::sim::projectile::launch::AirburstLaunchProfile::Retail,
        );
        if let Some(bullet) = super::FiredBullet::launched(
            world,
            rules,
            native_unique_id,
            live.source_id,
            target,
            payload,
            50,
            location,
            velocity,
        ) {
            super::admit_fired_bullet(world, rules, weapon, bullet_id, bullet);
        }
    }
}

/// Installed Ares DetonateAt_Splits1001A090 replaces469EBA..46A290. The
/// existing caller has already committed damage/animation/debris and retains
/// this parent until the tail returns; neither this function nor its guest
/// duplicates that transaction or retires the parent.
fn emit_ares(world: &mut Simulation, rules: &RuleSet, detonation: &ProjectileDetonation) {
    use crate::sim::projectile::launch::{AirburstLaunchProfile, airburst_launch_velocity};
    use crate::util::native_x87::{NativeF64Bits, X87Chop53 as X, X87Ordering, true_distance_3d};
    let Some(parent) = world.projectiles.get(detonation.projectile_id) else {
        return;
    };
    let location = parent.position();
    let source = parent.source_id();
    let initial_target = parent.target();
    let Some(kind) = parent
        .payload()
        .projectile_type()
        .and_then(|id| rules.projectile(world.interner.resolve(id)))
        .filter(|kind| kind.airburst_arm())
    else {
        return;
    };
    let Some(policy) = kind.airburst_policy() else {
        return;
    };
    let Some(weapon) = kind
        .airburst_weapon
        .as_deref()
        .and_then(|id| rules.weapon(id))
    else {
        return;
    };
    let Some(child) = weapon
        .projectile
        .as_deref()
        .and_then(|id| rules.projectile(id))
    else {
        return;
    };
    let Some(warhead) = weapon.warhead.as_deref().and_then(|id| rules.warhead(id)) else {
        return;
    };
    // CreateBullet10019210 leaves +130 null. Fire4687E5 would dereference
    // that null for Inviso+FlakScatter; do not silently supply our factory's
    // weapon argument as a replacement for the missing native pointer.
    if child.inviso && child.flak_scatter {
        log::error!(
            "Ares Airburst [{}]: Inviso+FlakScatter child requires a native null Weapon dereference",
            kind.id
        );
        return;
    }
    let Some(terrain) = world.resolved_terrain.as_ref() else {
        return;
    };
    let around = if policy.around_target {
        super::projectile_delivery::target_get_coords(world, initial_target).unwrap_or(location)
    } else {
        location
    };
    let center = ((around.x / 256) as i16, (around.y / 256) as i16);
    let mut targets = Vec::new();
    let clusters;
    if !policy.splits {
        let circle = match crate::sim::cell_rect::NativeCircularCells::new(
            center,
            NativeF64Bits::from_bits(policy.spread_bits),
        ) {
            Ok(circle) => circle,
            Err(error) => {
                log::error!("Ares Airburst [{}]: {error}", kind.id);
                return;
            }
        };
        let cells = NativeCellQuery::canonical(terrain);
        for (x, y) in circle {
            //1001A2CD TryGet: misses do not stamp or retain the Dummy. Keep
            // real cell identity, including the map owner's packed aliases.
            if let Some(index) = terrain.native_fixed_cell_index(x, y) {
                let (x, y) = cells.coord(NativeCellIdentity::Real(index));
                targets.push(ProjectileTarget::Cell {
                    rx: x as u16,
                    ry: y as u16,
                });
            }
        }
        clusters = targets.len(); //1001A35B replaces the author's Cluster.
    } else {
        // Techno ctor6F31BD appends; dtor6F463B->63F000 shifts on removal.
        // This is the existing constructor registry, not Logic/spatial order.
        let owner = world
            .substrate
            .entities
            .get(source)
            .map(|source| source.owner());
        for slot in 0..world.substrate.entities.techno_registry_len() {
            let id = world
                .substrate
                .entities
                .techno_registry_at(slot)
                .expect("captured Techno slot");
            let entity = world.substrate.entities.get(id).expect("registered Techno");
            if !entity.in_playfield()
                || !entity.lifecycle.cell_marked()
                || entity.health.current() <= 0
            {
                continue;
            }
            if !policy.retarget_self && id == source {
                continue;
            }
            // CanTargetHouse10051B70 uses the same frozen Ares relationships
            // as EMP/AttachEffect and ReceiveDamage, not Phobos neutral gates.
            let relationships = warhead.emp_rules().expect("admitted Ares relationships");
            if !relationships.can_affect_house(
                owner.is_some(),
                owner == Some(entity.owner()),
                owner.is_some_and(|owner| {
                    world
                        .house_alliances
                        .is_allied(&world.interner, owner, entity.owner())
                }),
                warhead.affects_allies,
            ) {
                continue;
            }
            let coord = crate::sim::movement::ground_pose::object_get_coords(entity, Some(terrain));
            //1001A414..46F compares the actual sqrt, not a signed32 distance
            // (the latter would turn an out-of-domain distance into INT_MIN).
            let distance =
                true_distance_3d([around.x, around.y, around.z], [coord.x, coord.y, coord.z]);
            if X::compare(distance, X::load_i32(1280)) != X87Ordering::Less {
                continue;
            }
            if !child.aa
                && crate::sim::movement::air_movement::is_high_flying(
                    entity,
                    Some(terrain),
                    Some((rules, &world.interner)),
                )
            {
                continue;
            }
            targets.push(ProjectileTarget::Entity(id));
        }
        clusters = kind.cluster.max(0) as usize;
        //1001A500: fill all missing entries before selection, X then Y. A
        // miss retains the one shared Dummy identity, not its current coords.
        let cells = NativeCellQuery::canonical(terrain);
        while targets.len() < clusters {
            let x = center
                .0
                .wrapping_add(world.scenario_rng.next_range_i32_inclusive(-3, 3) as i16);
            let y = center
                .1
                .wrapping_add(world.scenario_rng.next_range_i32_inclusive(-3, 3) as i16);
            let cell = cells.lookup((x, y));
            targets.push(match cell {
                NativeCellIdentity::Dummy => ProjectileTarget::DummyCell,
                NativeCellIdentity::Real(_) => {
                    let (x, y) = cells.coord(cell);
                    ProjectileTarget::Cell {
                        rx: x as u16,
                        ry: y as u16,
                    }
                }
            });
        }
    }
    for index in 0..clusters {
        //1001A5B0 reads the parent's current Target/Owner each iteration.
        let live = world.projectiles.current_detonation(*detonation);
        let target = if policy.splits {
            select_split_target(
                &mut world.scenario_rng,
                live.target,
                live.source_id,
                f64::from_bits(policy.retarget_accuracy_bits),
                &mut targets,
            )
        } else {
            targets[index]
        };
        let payload = ProjectilePayload::new(
            weapon.damage,
            world.interner.intern(&warhead.id),
            Some(world.interner.intern(&child.id)),
            weapon.bright,
        );
        //10019210 constructs with weapon fields/range, WITHOUT SetWeaponType.
        // No Phobos house fallback, damage multiplier or simulated effects.
        let id = world.construct_projectile(live.source_id, target, payload, rules);
        let range = weapon
            .firing_policy()
            .and_then(|p| p.projectile_range)
            .expect("admitted Ares child weapon range");
        world.projectiles.set_weapon_fuel(id, range);
        let native = world.projectiles.get(id).unwrap().native_unique_id();
        let velocity = airburst_launch_velocity(
            weapon.speed,
            &mut world.scenario_rng,
            AirburstLaunchProfile::Ares,
        );
        if let Some(mut bullet) = super::FiredBullet::launched(
            world,
            rules,
            native,
            live.source_id,
            target,
            payload,
            weapon.speed,
            location,
            velocity,
        ) {
            bullet.target_on_bridge = false; //MoveTo1001A742, never FireAt's copy.
            super::admit_fired_bullet(world, rules, weapon, id, bullet);
        } else {
            world.retire_non_entity_object(id, crate::sim::world::UninitContext::new(rules, None));
        }
    }
}

/// Ares1001A5B0..68F. No extra legality filter on the retained original target;
/// only retargets consume a slot, using swap-last (not the global array's shift).
fn select_split_target(
    rng: &mut crate::sim::rng::SimRng,
    original: ProjectileTarget,
    source: u64,
    accuracy: f64,
    targets: &mut Vec<ProjectileTarget>,
) -> ProjectileTarget {
    if original != ProjectileTarget::None && !(rng.random_double() > accuracy) {
        return original;
    }
    let mut slot = rng.next_range_i32_inclusive(0, targets.len() as i32 - 1) as usize;
    if targets[slot] == ProjectileTarget::Entity(source) && rng.random_double() > 0.5 {
        slot = rng.next_range_i32_inclusive(0, targets.len() as i32 - 1) as usize;
    }
    targets.swap_remove(slot)
}

#[cfg(test)]
#[path = "airburst_tests.rs"]
mod tests;

#[cfg(all(test, not(feature = "phobos")))]
#[path = "ares_airburst_tests.rs"]
mod ares_tests;
