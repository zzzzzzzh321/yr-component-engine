//! World observations for the ra2-mono target-preflight port. No mutable
//! gameplay state is duplicated here: occupancy, power, alliances and fog
//! stay with their existing VERA owners.

use crate::map::entities::EntityCategory;
use crate::map::resolved_terrain::NativeCellQuery;
use crate::rules::ruleset::RuleSet;
use crate::rules::superweapon_targeting::TargetReject;
use crate::rules::superweapon_targeting::{
    HouseMask, LaunchSite, TargetContent, TargetMask, TargetPreflightInput, evaluate_target,
};
use crate::rules::superweapon_type::SuperWeaponType;
use crate::sim::intern::InternedId;
use crate::sim::movement::locomotor::MovementLayer;
use crate::sim::world::Simulation;

/// One read-only TargetingData observation (Ares1006D1E0). The owning
/// command/AI attempt drops it; entity positions, power and alliances remain
/// authoritative. AI prepares it once, rather than rescanning for every cell.
pub(super) struct TargetingData {
    sites: Vec<LaunchSite>,
    needs_designator: bool,
    designators: Vec<RangedSite>,
    inhibitors: Vec<RangedSite>,
}

impl TargetingData {
    fn check_influences(&self, cell: (u16, u16)) -> Result<(), TargetReject> {
        // NewSWType::CanFireAt1006CE5F..1006CF31: after the type and
        // launch-site checks, require a designator and exclude inhibitors.
        if self.needs_designator && !self.designators.iter().any(|site| site.contains(cell)) {
            return Err(TargetReject::NoEligibleDesignator);
        }
        if self.inhibitors.iter().any(|site| site.contains(cell)) {
            return Err(TargetReject::InhibitorInRange);
        }
        Ok(())
    }
}

struct RangedSite {
    center: (i16, i16),
    range_squared: i32,
}

impl RangedSite {
    fn contains(&self, cell: (u16, u16)) -> bool {
        //1006CE80..1006CF16 squares the signed-word differences in x87;
        // their sum is an exact integer below2^34. i64 preserves it without
        // FP. The separately IMUL'd range-square1006D440 wraps as an i32.
        let dx = i64::from(cell.0 as i16) - i64::from(self.center.0);
        let dy = i64::from(cell.1 as i16) - i64::from(self.center.1);
        dx * dx + dy * dy <= i64::from(self.range_squared)
    }
}

pub(super) fn prepare(
    sim: &Simulation,
    rules: &RuleSet,
    owner: InternedId,
    sw: &SuperWeaponType,
) -> TargetingData {
    let mut data = TargetingData {
        sites: launch_sites(sim, rules, owner, sw)
            .into_iter()
            .map(|(_, site)| site)
            .collect(),
        needs_designator: false,
        designators: Vec::new(),
        inhibitors: Vec::new(),
    };
    #[cfg(not(feature = "ares"))]
    return data;
    #[cfg(feature = "ares")]
    {
        if rules.external_mod.is_some() || rules.super_weapon_target_policy(&sw.id).is_none() {
            return data;
        }
        let Some(influence) = rules
            .super_weapon_metadata(&sw.id)
            .map(|metadata| metadata.influence())
        else {
            return data;
        };
        data.needs_designator = influence.any_designator || !influence.designators.is_empty();
        if !data.needs_designator && !influence.any_inhibitor && influence.inhibitors.is_empty() {
            return data;
        }
        // Predicates1006D880/1006D900: exact owner for designation, hostile
        // owner for inhibition, alive/nonzero HP/nonlimbo/not deactivated for
        // both, plus Building IsPowerOnline4555D0 for inhibitors only.
        // Query XY is pure; coalescing the two scans draws no RNG and writes no
        // timer, detach, fog or cell state.
        for entity in sim.substrate.entities.values() {
            if !entity.is_object_alive()
                || entity.health.current() == 0
                || entity.lifecycle.in_limbo()
                || entity.deactivated.is_offline()
            {
                continue;
            }
            let Some(ty) = sim.interner.object_type(entity.type_ref(), rules) else {
                continue;
            };
            let matches = |types: &[crate::rules::superweapon_type::TechnoTypeReference]| {
                types
                    .iter()
                    .any(|reference| reference.category == ty.category && reference.id == ty.id)
            };
            let designator = entity.owner() == owner
                && (influence.any_designator || matches(&influence.designators));
            let inhibitor = entity.owner() != owner
                && !sim
                    .house_alliances
                    .is_allied(&sim.interner, owner, entity.owner())
                && (influence.any_inhibitor || matches(&influence.inhibitors))
                && (entity.category() != EntityCategory::Structure
                    || sim.building_operational_state(entity.stable_id(), rules) == Some(true));
            if !designator && !inhibitor {
                continue;
            }
            let [x, y] = crate::sim::movement::ground_pose::object_center_xy(entity);
            let center = ((x / 256) as i16, (y / 256) as i16);
            if designator {
                let range = ty.designator_range().unwrap_or(ty.sight);
                if range > 0 {
                    data.designators.push(RangedSite {
                        center,
                        range_squared: range.wrapping_mul(range),
                    });
                }
            }
            if inhibitor {
                let range = ty.inhibitor_range().unwrap_or(ty.sight);
                if range > 0 {
                    data.inhibitors.push(RangedSite {
                        center,
                        range_squared: range.wrapping_mul(range),
                    });
                }
            }
        }
        data
    }
}

/// Shared optional-SW target/placement bound, using the terrain owner.
pub(super) fn on_map(sim: &Simulation, cell: (u16, u16)) -> bool {
    sim.resolved_terrain
        .as_ref()
        .and_then(|terrain| terrain.cell(cell.0, cell.1))
        .is_some()
        && crate::sim::cell_rect::cell_is_in_playfield_height_aware_in_query(
            (i32::from(cell.0), i32::from(cell.1)),
            sim.playfield_bounds(),
            sim.resolved_terrain.as_ref(),
            None,
        )
}

impl Simulation {
    /// Shared extension target gate, suitable for cursor preview and the final
    /// command transaction. Stock types retain their existing dispatch gates.
    /// A preview uses an isolated native-cell query, so even a projected fog
    /// lookup outside the grid cannot mutate the authoritative Dummy cell.
    pub fn super_weapon_target_check(
        &self,
        rules: &RuleSet,
        owner: InternedId,
        sw_type: InternedId,
        cell: (u16, u16),
        automatic: bool,
    ) -> Result<(), TargetReject> {
        let sw = rules
            .super_weapon(self.interner.resolve(sw_type))
            .ok_or(TargetReject::InstanceNotFound)?;
        check(self, rules, owner, sw, cell, automatic)
    }
}

pub(super) fn check(
    sim: &Simulation,
    rules: &RuleSet,
    owner: InternedId,
    sw: &SuperWeaponType,
    cell: (u16, u16),
    automatic: bool,
) -> Result<(), TargetReject> {
    if !rules.superweapon_semantics().custom_payloads {
        return if matches!(
            sw.kind,
            crate::rules::superweapon_type::SuperWeaponKind::GenericWarhead
                | crate::rules::superweapon_type::SuperWeaponKind::UnitDelivery
                | crate::rules::superweapon_type::SuperWeaponKind::HunterSeeker
        ) {
            Err(TargetReject::InvalidPayload)
        } else {
            Ok(())
        };
    }
    // The shared prepared query also admits component-authored targeting.
    if rules.external_mod.is_none()
        && !automatic
        && sw.kind == crate::rules::superweapon_type::SuperWeaponKind::HunterSeeker
    {
        return Ok(());
    }
    let data = prepare(sim, rules, owner, sw);
    check_with_data(sim, rules, owner, sw, cell, automatic, &data)
}

/// AI prepares the launcher observation once per SW, not once per candidate.
/// This is a borrowed query input, never a persistent cache or new authority.
pub(super) fn check_with_data(
    sim: &Simulation,
    rules: &RuleSet,
    owner: InternedId,
    sw: &SuperWeaponType,
    cell: (u16, u16),
    automatic: bool,
    data: &TargetingData,
) -> Result<(), TargetReject> {
    let component = sw
        .generic_warhead
        .as_ref()
        .map(|settings| &settings.targeting);
    #[cfg(feature = "ares")]
    let native = rules
        .external_mod
        .is_none()
        .then(|| rules.super_weapon_target_policy(&sw.id))
        .flatten();
    #[cfg(feature = "ares")]
    let native = native.map(|policy| {
        (
            if automatic {
                policy.automatic()
            } else {
                policy.manual()
            },
            policy.fire_into_shroud(),
            policy.range(),
        )
    });
    #[cfg(not(feature = "ares"))]
    let native = None;
    let (requirements, fire_into_shroud, range) = if let Some(policy) = native {
        policy
    } else if let Some(policy) = component {
        (
            if automatic {
                policy.automatic
            } else {
                policy.manual
            },
            policy.fire_into_shroud,
            policy.range(),
        )
    } else {
        return Ok(());
    };
    if sw.generic_warhead.is_some() && !super::generic_warhead::can_launch(sim, rules, sw, cell) {
        return Err(
            if sim
                .resolved_terrain
                .as_ref()
                .is_some_and(|terrain| terrain.cell(cell.0, cell.1).is_some())
            {
                TargetReject::InvalidPayload
            } else {
                TargetReject::TargetOutOfBounds
            },
        );
    }
    let terrain = sim
        .resolved_terrain
        .as_ref()
        .ok_or(TargetReject::TargetOutOfBounds)?;
    let target = terrain
        .cell(cell.0, cell.1)
        .ok_or(TargetReject::TargetOutOfBounds)?;
    let layer =
        if target.bridge_facts.raw_flags & crate::map::bridge_facts::BRIDGE_FLAG_STRUCTURAL != 0 {
            MovementLayer::Bridge
        } else {
            MovementLayer::Ground
        };
    let contents: Vec<_> = sim
        .substrate
        .occupancy
        .get(cell.0, cell.1)
        .into_iter()
        .flat_map(|occupancy| occupancy.iter_layer(layer))
        .filter_map(|occupant| sim.substrate.entities.get(occupant.entity_id))
        .filter(|entity| entity.is_object_alive() && !entity.lifecycle.in_limbo())
        .map(|entity| TargetContent {
            kind: match entity.category() {
                EntityCategory::Infantry => TargetMask::INFANTRY,
                EntityCategory::Structure => TargetMask::BUILDING,
                EntityCategory::Unit | EntityCategory::Aircraft => TargetMask::UNIT,
            },
            relation: sim.ares_house_relation(owner, entity.owner()),
        })
        .collect();
    // Ares tests `SW.FireIntoShroud=` only in the player's cursor,
    // `SWTypeExt::ExtData::WhatAction` (Ares.dll `0x10033EB0`), on the
    // client's own map; the AI's `CanFireAt` path has no shroud test.
    let shrouded = (!automatic && !fire_into_shroud).then(|| {
        crate::sim::vision::cell_is_shrouded(
            &sim.fog,
            &NativeCellQuery::isolated(terrain),
            owner,
            cell,
        )
    });
    evaluate_target(&TargetPreflightInput {
        target_cell: Some((i32::from(cell.0), i32::from(cell.1))),
        requires_target_cell: true,
        requirements: &requirements,
        contents: &contents,
        target_in_bounds: true,
        terrain_is_water: Some(
            target.land_type == crate::rules::terrain_rules::LandType::Water.as_index(),
        ),
        fire_into_shroud,
        shrouded,
        range,
        requires_launch_site: false,
        launch_sites: &data.sites,
    })?;
    data.check_influences(cell)
}

/// ra2-mono ResolveSuperweaponLaunchSites adaptation: VERA's retained House
/// building order and operational-state owner replace OOP/ECS bridge lookups.
pub(super) fn launch_sites(
    sim: &Simulation,
    rules: &RuleSet,
    owner: InternedId,
    sw: &SuperWeaponType,
) -> Vec<(u64, LaunchSite)> {
    sim.houses
        .get(&owner)
        .into_iter()
        .flat_map(|house| house.base_projection().buildings())
        .filter_map(|&id| {
            let entity = sim.substrate.entities.get(id)?;
            if !super::provided_supers(sim, rules, entity)
                .any(|name| name.eq_ignore_ascii_case(&sw.id))
            {
                return None;
            }
            let coord = crate::sim::movement::ground_pose::object_get_coords(
                entity,
                sim.resolved_terrain.as_ref(),
            );
            Some((
                id,
                LaunchSite {
                    center: (coord.x / 256, coord.y / 256),
                    eligible: entity.owner() == owner
                        && entity.is_object_alive()
                        && !entity.lifecycle.in_limbo()
                        && sim.building_operational_state(id, rules) == Some(true),
                    range_minimum_override: None,
                    range_maximum_override: None,
                },
            ))
        })
        .collect()
}

impl Simulation {
    /// Ares's house relation (10021440): the same house is the owner, then
    /// the owner's directed IsAlliedWith(other) makes an ally; any other
    /// house, including an absent one, is an enemy.
    pub(crate) fn ares_house_relation(&self, owner: InternedId, other: InternedId) -> HouseMask {
        if owner == other {
            HouseMask::OWNER
        } else if self.house_alliances.is_allied(&self.interner, owner, other) {
            HouseMask::ALLIES
        } else {
            HouseMask::ENEMIES
        }
    }
}
