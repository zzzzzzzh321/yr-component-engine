//! Prism forwarding against `tools/spatial_oracle/building_prism.json`
//! (retail gamemd.exe under Unicorn; fixture, hooks and row schema in the
//! oracle's docstring): Mission_Attack's PrismType arm, ProcessDelayedFire's
//! shot and support beam, the support bonus and the damage it scales, the
//! multi-tower cadence of BuildingClass::Update's mission pieces, and
//! ReadGeneral's Prism block through the production reader.
//!
//! Not compared: the rearm timer's middle dword (`+0x2F0`, which the support
//! beam fills from an uninitialised stack temporary and VERA's [`CdTimer`]
//! does not hold), the laser (not drawn; module doc) and the raw Scenario
//! draws of a Guard dispatch (see [`prism_cadence_matches_the_original`]).
//!
//! Native states VERA cannot hold, by row: a delayed-fire mode 0 with a
//! countdown (`delayed_fire_countdown`) is held as a delayed shot, since the
//! walk reads only the countdown; a mode 0 with a countdown or a mode other
//! than 1 or 2 at expiry (`mode_0`, `mode_3`) is skipped: every native writer
//! pairs the mode with its countdown and writes only 1 or 2.
//! `fire_at_returns_null` is FireAt's refusal, which VERA meets in its own
//! emission path, where [`ObjectSubstrate::take_support_bonus`](crate::sim::world::ObjectSubstrate::take_support_bonus) runs only for a
//! launched bullet.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::*;
use crate::map::resolved_terrain::test_flat_ground_grid;
use crate::rules::ini_parser::IniFile;
#[cfg(not(feature = "ares"))]
use crate::rules::native_processing::{RulesLayerKind, RulesLayerStack};
#[cfg(not(feature = "ares"))]
use crate::rules::ruleset::PrismSupportRules;
use crate::sim::combat::TargetKind;
use crate::sim::combat::receiver_fixture::with_building_fire_stub;
use crate::sim::mission::MissionDispatchTimer;
use crate::sim::mission::state::MissionTestFixture;
use crate::sim::projectile::ProjectileCoord;
use crate::sim::timer::{CdTimer, PAUSED_START_FRAME};
use crate::util::fixed_math::SimFixed;

/// The oracle fixture's frame (`frame0`).
const FRAME: i32 = 200;
/// The first building's Location, leptons.
const ORIGIN: [i32; 3] = [3200, 3200, 0];
/// The target's distance east of the first building, in and out of the
/// shot's 2048-lepton range.
const IN_RANGE: i32 = 768;
const OUT_OF_RANGE: i32 = 4096;
/// The oracle's Prism tower `DelayedFireDelay=`.
const DELAY: i32 = 28;
/// The oracle's FireAt rearm (the row's `rof`, retail PrismShot's 45).
const ROF: i32 = 45;

fn golden() -> &'static Value {
    crate::test_fixture::json("tools/spatial_oracle/building_prism.json")
}

fn rows<'a>(golden: &'a Value, set: &str) -> &'a [Value] {
    golden[set].as_array().unwrap()
}

/// The oracle's tower type `TWR` (PrismType, Strength 600, `Powered=` like
/// the retail tower), its copy `OTHER`, and `SHED`, the target: Weapon[0]
/// `PrismShot` of 2048 leptons, Weapon[1] `PrismSupport` of `support_range`
/// (EliteWeapon[1] of `elite_range` when given), the SpecialAnim `GAPRIS_A`
/// and its Damaged `GAPRIS_AD`, `DelayedFireDelay=28` and MissionControl as
/// building_guard_attack's cadence (AARate .016).
fn rule_sources(support_range: i32, elite_range: Option<i32>) -> (String, &'static str) {
    let cells = |leptons: i32| f64::from(leptons) / 256.0;
    let elite = elite_range.map_or(String::new(), |range| {
        format!(
            "[PrismSupportElite]\nDamage=200\nROF=45\nRange={}\nProjectile=Beam\nWarhead=Prism\n",
            cells(range)
        )
    });
    let elite_key = if elite_range.is_some() {
        "EliteSecondary=PrismSupportElite\n"
    } else {
        ""
    };
    let tower = format!(
        "Image=TWRART\nStrength=600\nArmor=steel\nFoundation=1x1\nPowered=yes\nPower=-75\n\
         Primary=PrismShot\nSecondary=PrismSupport\n{elite_key}"
    );
    let rules_ini = format!(
        "[Clear]\nBuildable=yes\n[General]\nPrismType=TWR\nPrismSupportModifier=150%\nPrismSupportMax=8\n\
         PrismSupportDelay=45\n\
         [Guard]\nRate=.030\nAARate=.016\n\
         [Attack]\nRate=.016\nAARate=.016\n\
         [BuildingTypes]\n0=TWR\n1=OTHER\n2=SHED\n\
         [Animations]\n0=GAPRIS_A\n1=GAPRIS_AD\n\
         [TWR]\n{tower}\
         [OTHER]\n{tower}\
         [SHED]\nStrength=1000\nArmor=wood\nFoundation=1x1\n\
         [PrismShot]\nDamage=120\nROF=45\nRange=8\nProjectile=Beam\nWarhead=Prism\n\
         [PrismSupport]\nDamage=200\nROF=45\nRange={}\nProjectile=Beam\nWarhead=Prism\n\
         {elite}\
         [Beam]\nAG=yes\nInviso=yes\n\
         [Prism]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
        cells(support_range)
    );
    let art_ini = "[TWRART]\nSpecialAnim=GAPRIS_A\nSpecialAnimDamaged=GAPRIS_AD\n\
         IsAnimDelayedFire=yes\nDelayedFireDelay=28\n\
         [GAPRIS_A]\nStart=0\nEnd=9\nRate=300\n\
         [GAPRIS_AD]\nImage=GAPRIS_A\nStart=10\nEnd=19\nRate=300\n";
    (rules_ini, art_ini)
}

fn rules(support_range: i32, elite_range: Option<i32>) -> RuleSet {
    let (rules_ini, art_ini) = rule_sources(support_range, elite_range);
    let rules_ini = IniFile::from_str(&rules_ini);
    let art_ini = IniFile::from_str(art_ini);
    let mut rules = RuleSet::from_ini_with_fixed_art_for_test(&rules_ini, &art_ini).unwrap();
    install_prism_fixture_art(&mut rules, &art_ini);
    rules
}

fn install_prism_fixture_art(rules: &mut RuleSet, art_ini: &IniFile) {
    let mut art = crate::rules::art_data::ArtRegistry::from_ini(art_ini);
    for name in ["GAPRIS_A", "GAPRIS_AD"] {
        art.bind_anim_frame_count_for_test(name, 20);
    }
    rules.install_art_data(art);
}

fn mission_of(name: &str) -> MissionId {
    match name {
        "none" => MissionId::NONE,
        "attack" => MissionId::from_known(MissionType::Attack),
        "guard" => MissionId::from_known(MissionType::Guard),
        "selling" => MissionId::from_known(MissionType::Selling),
        other => panic!("unexpected mission {other}"),
    }
}

/// A delayed fire's mode (`+0x704`) and countdown (`+0x714`).
fn delayed_fire(entity: &crate::sim::game_entity::GameEntity) -> (i64, i32) {
    match entity.pending_building_fire.pending() {
        None => (0, 0),
        Some(pending) => match pending.fire() {
            DelayedFire::Weapon(_) => (1, pending.remaining_ticks()),
            DelayedFire::SupportBeam { .. } => (2, pending.remaining_ticks()),
        },
    }
}

/// The oracle's building fixture: the first building at [`ORIGIN`], the
/// row's towers at their offsets, `SHED` (the target) [`IN_RANGE`] east of it
/// and the owner House's building list, at the oracle's frame. Every building
/// guards with its dispatch due, `+0x6DD` set and its rearm run out.
struct Fixture {
    sim: Simulation,
    rules: RuleSet,
    owner: crate::sim::intern::InternedId,
    buildings: BTreeMap<String, u64>,
    target: u64,
    /// Buildings holding a native mode VERA represents otherwise.
    mode_unheld: BTreeSet<u64>,
}

impl Fixture {
    fn new(input: &Value, first: &str) -> Self {
        let rules = rules(
            input["support_range"]
                .as_i64()
                .map_or(2048, |range| range as i32),
            input["elite_support_range"]
                .as_i64()
                .map(|range| range as i32),
        );
        Self::with_sim(input, first, rules, Simulation::new())
    }

    fn with_sim(input: &Value, first: &str, rules: RuleSet, mut sim: Simulation) -> Self {
        sim.install_resolved_terrain_for_new_map(test_flat_ground_grid(48));
        crate::sim::arena_fixture::supply_native_map_keyed(&mut sim, "prism flat ground 48");
        let owner = sim.interner.intern("Americans");
        sim.houses
            .register(crate::sim::house_state::HouseState::new(
                owner, 0, None, false, 0, 10,
            ));
        // The target's House: building spawns mark their owner's reservation.
        let target_owner = sim.interner.intern("Russians");
        sim.houses
            .register(crate::sim::house_state::HouseState::new(
                target_owner,
                1,
                None,
                false,
                0,
                10,
            ));
        let mut buildings = BTreeMap::new();
        let mut spawned = Vec::new();
        let towers = input["towers"].as_array().cloned().unwrap_or_default();
        let named =
            std::iter::once((first.to_string(), "TWR", [0; 3])).chain(towers.iter().map(|tower| {
                let offset = tower["offset"].as_array().unwrap();
                let kind = if tower["type"] == "other" {
                    "OTHER"
                } else {
                    "TWR"
                };
                (
                    tower["name"].as_str().unwrap().to_string(),
                    kind,
                    [0, 1, 2].map(|axis| offset[axis].as_i64().unwrap() as i32),
                )
            }));
        for (index, (name, kind, offset)) in named.enumerate() {
            let rx = 2 + 2 * index as u16;
            let id = sim
                .spawn_object(kind, "Americans", rx, 40, 0, &rules)
                .unwrap_or_else(|| panic!("{name} spawns"));
            place(&mut sim, id, offset);
            buildings.insert(name, id);
            spawned.push(id);
        }
        // Unlimbo's House+68 appends, in spawn order.
        assert_eq!(sim.houses[&owner].base_projection().buildings(), spawned);
        let target = sim
            .spawn_object("SHED", "Russians", 15, 12, 0, &rules)
            .unwrap();
        // No power state: every tower is operational until `unpower`.
        sim.power_states.map_mut_for_test().clear();
        sim.session.set_binary_frame_for_test(FRAME as u32);
        for &id in &spawned {
            let entity = sim.substrate.entities.get_mut(id).unwrap();
            assert!(
                entity.building_art.anim_slots().iter().all(Option::is_none),
                "the oracle's anim slots start empty"
            );
            entity.mission.apply_test_fixture(MissionTestFixture {
                current: mission_of("guard"),
                suspended: MissionId::NONE,
                queued: MissionId::NONE,
                movement_bypass_latch: 0,
                handler_state: 0,
                mission_start_frame: FRAME as u32,
                ai_counter: 0,
                dispatch_timer: MissionDispatchTimer::from_raw(FRAME, 0),
            });
            entity.mission_leaf.set_building_ready_latch(1);
            entity
                .rearm_timer
                .set_for_test(CdTimer::started(FRAME - 100, 0));
        }
        let mut fixture = Self {
            sim,
            rules,
            owner,
            buildings,
            target,
            mode_unheld: BTreeSet::new(),
        };
        fixture.move_target(IN_RANGE);
        let vector = input["vector"].as_array().or(input["order"].as_array());
        if let Some(vector) = vector {
            let ids = vector
                .iter()
                .map(|name| name.as_str().map_or(u64::MAX, |name| fixture.id(name)))
                .collect();
            fixture
                .sim
                .houses
                .get_mut(&owner)
                .unwrap()
                .base_projection_mut()
                .replace_buildings_for_test(ids);
        }
        fixture
    }

    fn id(&self, name: &str) -> u64 {
        self.buildings[name]
    }

    fn name(&self, id: u64) -> &str {
        self.buildings
            .iter()
            .find_map(|(name, &building)| (building == id).then_some(name.as_str()))
            .unwrap()
    }

    /// The row's per-building input (`alive`, `health`, `mission`, `queued`,
    /// `target`, `rearm`, `drained`, `count`, `mode`, `countdown`, `payload`,
    /// `veterancy`) over the fixture's defaults.
    fn apply(&mut self, id: u64, state: &Value) {
        let target = self.target;
        let entity = self.sim.substrate.entities.get_mut(id).unwrap();
        if state["alive"] == false {
            entity.lifecycle.set_object_alive_for_test(false);
        }
        if let Some(health) = state["health"].as_u64() {
            entity.health.set_for_test(health as i32);
        }
        if state["mission"].is_string() || state["queued"].is_string() {
            entity.mission.apply_test_fixture(MissionTestFixture {
                current: mission_of(state["mission"].as_str().unwrap_or("guard")),
                suspended: MissionId::NONE,
                queued: mission_of(state["queued"].as_str().unwrap_or("none")),
                movement_bypass_latch: 0,
                handler_state: 0,
                mission_start_frame: FRAME as u32,
                ai_counter: 0,
                dispatch_timer: MissionDispatchTimer::from_raw(FRAME, 0),
            });
        }
        if let Some(aimed) = state["target"].as_bool() {
            entity
                .attack_target
                .set_for_test(aimed.then(|| TargetKind::Entity(target)));
        }
        if let Some(rearm) = state["rearm"].as_array() {
            let start = rearm[0]
                .as_i64()
                .map_or(PAUSED_START_FRAME, |relative| FRAME + relative as i32);
            entity
                .rearm_timer
                .set_for_test(CdTimer::started(start, rearm[1].as_i64().unwrap() as i32));
        }
        if state["drained"] == true {
            entity.drain.set_draining_me_for_test(Some(u64::MAX));
        }
        if let Some(count) = state["count"].as_i64() {
            entity.prism_support_count.set_for_test(count as i32);
        }
        if let Some(veterancy) = state["veterancy"].as_f64() {
            entity
                .veterancy
                .set_rank_for_test((veterancy * 100.0) as u16);
        }
        let countdown = state["countdown"].as_i64().unwrap_or(0) as i32;
        let payload = |axis: usize| state["payload"][axis].as_i64().unwrap_or(0) as i32;
        entity
            .pending_building_fire
            .set_for_test(match state["mode"].as_i64().unwrap_or(0) {
                0 if countdown == 0 => None,
                0 => {
                    self.mode_unheld.insert(id);
                    Some(PendingBuildingFire::new(countdown, DelayedFire::Weapon(0)))
                }
                1 => Some(PendingBuildingFire::new(
                    countdown,
                    DelayedFire::Weapon(payload(0)),
                )),
                2 => Some(PendingBuildingFire::new(
                    countdown,
                    DelayedFire::SupportBeam {
                        to: ProjectileCoord::new(payload(0), payload(1), payload(2)),
                    },
                )),
                other => panic!("mode {other} has no VERA state"),
            });
    }

    /// The target's Location `east` leptons east of [`ORIGIN`].
    fn move_target(&mut self, east: i32) {
        let target = self.target;
        place(&mut self.sim, target, [east, 0, 0]);
    }

    /// A House power outage: every `Powered=` tower of it stops being
    /// operational. The oracle answers Is_Operational per building; in the
    /// replayed rows only the building it names asks (Mission_Guard, the
    /// support beam and the Prism walk do not).
    fn unpower(&mut self) {
        let mut power_state = crate::sim::power_system::PowerState::default();
        power_state.set_total_output_for_test(0);
        power_state.set_total_drain_for_test(75);
        self.sim
            .power_states
            .map_mut_for_test()
            .insert(self.owner, power_state);
    }

    /// `state` after the call: count, delayed-fire mode and countdown, rearm
    /// [start, delay], mission (when the row records it), target and the
    /// turret counter (`+0x148`, when the row records it: the OK arm's tail
    /// advances it after the Prism arm, `0x0044B713`).
    fn assert_state(&self, id: u64, native: &Value, at: &str) {
        let entity = self.sim.substrate.entities.get(id).unwrap();
        let name = self.name(id);
        let (mode, countdown) = delayed_fire(entity);
        assert_eq!(
            i64::from(entity.prism_support_count.count()),
            native["count"].as_i64().unwrap(),
            "{at} {name} count"
        );
        if !self.mode_unheld.contains(&id) {
            assert_eq!(mode, native["mode"].as_i64().unwrap(), "{at} {name} mode");
        }
        assert_eq!(
            i64::from(countdown),
            native["countdown"].as_i64().unwrap(),
            "{at} {name} countdown"
        );
        let rearm = &native["rearm"];
        assert_eq!(
            [
                entity.rearm_timer.start_frame(),
                entity.rearm_timer.duration()
            ],
            [0, 1].map(|index| rearm[index].as_i64().unwrap() as i32),
            "{at} {name} rearm"
        );
        if let Some(mission) = native["mission"].as_str() {
            assert_eq!(
                entity.mission.current(),
                mission_of(mission),
                "{at} {name} mission"
            );
        }
        assert_eq!(
            entity.attack_target.target(),
            native["target"]
                .as_str()
                .map(|_| TargetKind::Entity(self.target)),
            "{at} {name} target"
        );
        if let Some(counter) = native["turret_counter"].as_i64() {
            assert_eq!(
                i64::from(entity.turret_anim_frame.frame()),
                counter,
                "{at} {name} +0x148"
            );
        }
    }

    fn scenario_state(&self) -> crate::sim::rng::SimRngLogicalState {
        self.sim.scenario_rng.logical_state()
    }
}

/// A building's Location: [`ORIGIN`] plus `offset`, leptons.
fn place(sim: &mut Simulation, id: u64, offset: [i32; 3]) {
    let [x, y, z] = [0, 1, 2].map(|axis| ORIGIN[axis] + offset[axis]);
    let position = &mut sim.substrate.entities.get_mut(id).unwrap().position;
    position.set_rx_for_test((x >> 8) as u16);
    position.set_ry_for_test((y >> 8) as u16);
    position.set_sub_x_for_test(SimFixed::from_num(x & 0xFF));
    position.set_sub_y_for_test(SimFixed::from_num(y & 0xFF));
    position.set_exact_z_leptons_for_test(Some(z));
}

fn location(sim: &Simulation, id: u64) -> [i32; 3] {
    let coord = crate::sim::movement::ground_pose::position_world_coord(
        &sim.substrate.entities.get(id).unwrap().position,
    );
    [coord.x, coord.y, coord.z]
}

fn slot_anim(sim: &Simulation, id: u64, slot: usize) -> Option<String> {
    sim.substrate
        .entities
        .get(id)
        .unwrap()
        .building_art
        .anim_slots()[slot]
        .map(|anim| {
            sim.interner
                .resolve(sim.anim(anim).unwrap().type_id())
                .to_string()
        })
}

/// Every `recruit` row: Mission_Attack's arm for a GetFireError answering OK
/// (the oracle answers SelectWeapon 0 and GetFireError 0, and `+0x6DD` is
/// already set). Its return; the recruit, or the master's own arm; the
/// recruit's stored point, natively the master's GetFLH call (weapon 0, base
/// {0, 0, 0}), which the oracle answers with a sentinel: the replay checks
/// the call and expects [`fire_coord::fire_coordinate`]'s building FLH, whose
/// values rest on its own evidence; every building's count, delayed fire,
/// rearm, mission and target; the one building whose Active slot empties and
/// whose SpecialAnim (or its Damaged variant) plays in slot 10; the distance
/// and weapon-1 range of every tower the walk measures (Sqrt_Approx and ftol,
/// elite range when elite); and no Scenario draw.
#[test]
fn prism_recruitment_matches_the_original() {
    let golden = golden();
    let recruit = rows(&golden, "recruit");
    assert_eq!(recruit.len(), 44);
    for row in recruit {
        let input = &row["input"];
        let name = input["name"].as_str().unwrap();
        let mut fixture = Fixture::new(input, "master");
        let master = fixture.id("master");
        let mut master_input = serde_json::json!({"mission": "attack", "target": true});
        if let Some(fields) = input["master"].as_object() {
            for (key, value) in fields {
                master_input[key] = value.clone();
            }
        }
        fixture.apply(master, &master_input);
        for tower in input["towers"].as_array().cloned().unwrap_or_default() {
            let id = fixture.id(tower["name"].as_str().unwrap());
            fixture.apply(id, &tower);
        }
        if let Some(max) = input["rules"]["max"].as_i64() {
            fixture.rules.general.prism_support.max = max as i32;
        }
        let draws_before = fixture.scenario_state();
        let pending_before: BTreeMap<u64, Option<PendingBuildingFire>> = fixture
            .buildings
            .values()
            .map(|&id| {
                let entity = fixture.sim.substrate.entities.get(id).unwrap();
                (id, entity.pending_building_fire.pending())
            })
            .collect();
        let target = TargetKind::Entity(fixture.target);
        let returns = attack_arm(
            &mut fixture.sim,
            master,
            &fixture.rules,
            target,
            0,
            FireError::Ok,
            ObjectAiCtx::default(),
            &mut false,
        );
        assert_eq!(
            i64::from(returns),
            row["returns"].as_i64().unwrap(),
            "{name}"
        );
        assert_eq!(
            fixture.scenario_state(),
            draws_before,
            "{name}: no Scenario draw"
        );

        let armed_now = |id: u64| {
            let entity = fixture.sim.substrate.entities.get(id).unwrap();
            let pending = entity.pending_building_fire.pending();
            pending.filter(|_| pending != pending_before[&id])
        };
        let recruited: Vec<&str> = fixture
            .buildings
            .values()
            .filter(|&&id| {
                id != master
                    && armed_now(id).is_some_and(|pending| {
                        pending.remaining_ticks == DELAY
                            && matches!(pending.fire, DelayedFire::SupportBeam { .. })
                    })
            })
            .map(|&id| fixture.name(id))
            .collect();
        assert_eq!(
            recruited,
            row["recruited"].as_str().into_iter().collect::<Vec<_>>(),
            "{name} recruited"
        );
        assert_eq!(
            armed_now(master) == Some(PendingBuildingFire::new(DELAY, DelayedFire::Weapon(0))),
            row["armed"].as_bool().unwrap(),
            "{name} armed"
        );
        if let Some(supporter) = recruited.first().map(|supporter| fixture.id(supporter)) {
            // `+0x708..+0x710` of a recruit: `vt+0xB0(out, 0, {0, 0, 0})`.
            assert!(
                row["calls"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(["flh", "master", 0, [0, 0, 0]])),
                "{name}"
            );
            let master_entity = fixture.sim.substrate.entities.get(master).unwrap();
            let flh = fire_coord::fire_coordinate(
                &fixture.sim,
                &fixture.rules,
                &fire_coord::FireSource::of_entity(master_entity),
                fixture.rules.object("TWR").unwrap(),
                0,
                0,
                Default::default(),
            )
            .coord;
            let supporter = fixture.sim.substrate.entities.get(supporter).unwrap();
            assert_eq!(
                supporter.pending_building_fire.pending().unwrap().fire,
                DelayedFire::SupportBeam { to: flh },
                "{name} beam point"
            );
        }

        let at = format!("recruit {name}");
        fixture.assert_state(master, &row["master"], &at);
        for (tower, native) in row["towers"].as_object().unwrap() {
            fixture.assert_state(fixture.id(tower), native, &at);
        }

        let plays: Vec<(&str, &str)> = row["calls"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|call| call[0] == "play_anim")
            .map(|call| {
                assert_eq!(call[4], 10, "{name}: the SpecialAnim slot");
                (call[1].as_str().unwrap(), call[2].as_str().unwrap())
            })
            .collect();
        assert_eq!(plays.len(), 1, "{name}: one armed building");
        for (building, &id) in &fixture.buildings {
            let played = plays
                .iter()
                .find_map(|(armed, anim)| (armed == building).then(|| anim.to_string()));
            assert_eq!(
                slot_anim(&fixture.sim, id, 3),
                None,
                "{name} {building} Active"
            );
            assert_eq!(
                slot_anim(&fixture.sim, id, 10),
                played,
                "{name} {building} SpecialAnim"
            );
        }

        let obj = fixture.rules.object("TWR").unwrap();
        for measured in row["distances"].as_array().unwrap() {
            let tower = fixture.id(measured[0].as_str().unwrap());
            let distance = crate::util::native_x87::distance_3d_leptons(
                location(&fixture.sim, master),
                location(&fixture.sim, tower),
            );
            assert_eq!(
                i64::from(distance),
                measured[1].as_i64().unwrap(),
                "{name} distance"
            );
            let range = combat_weapon::weapon_range(
                fixture.sim.substrate.entities.get(master).unwrap(),
                obj,
                1,
                &fixture.sim.substrate.entities,
                &fixture.rules,
                &fixture.sim.interner,
            );
            assert_eq!(
                i64::from(range),
                measured[2].as_i64().unwrap(),
                "{name} range"
            );
        }
    }
}

/// Every `bonus` row VERA can hold (module doc): ProcessDelayedFire's
/// countdown and, at its end, a request for the combat phase's FireAt only
/// with a target and GetFireError OK, whose bullet takes
/// [`ObjectSubstrate::take_support_bonus`](crate::sim::world::ObjectSubstrate::take_support_bonus) (the row's `bullet_multiplier`) and
/// clears the count; otherwise the count stays. The row's GetFireError answer
/// is made real: RANGE by the target's distance, REARM by the rearm timer,
/// CANT by a power outage.
#[test]
fn prism_support_bonus_matches_the_original() {
    let golden = golden();
    let bonus = rows(&golden, "bonus");
    assert_eq!(bonus.len(), 60);
    let mut compared = 0;
    for row in bonus {
        let input = &row["input"];
        let name = input["name"].as_str().unwrap();
        if matches!(name, "mode_0" | "mode_3" | "fire_at_returns_null") {
            continue;
        }
        let mut fixture = Fixture::new(&serde_json::json!({}), "master");
        let master = fixture.id("master");
        fixture.apply(master, &input["master"]);
        if let Some(modifier) = input["rules"]["modifier"].as_i64() {
            fixture.rules.general.prism_support.modifier = modifier as i32;
        }
        match input["fire_error"][0].as_i64() {
            None => {}
            Some(8) => fixture.move_target(OUT_OF_RANGE),
            Some(3) => {
                let entity = fixture.sim.substrate.entities.get_mut(master).unwrap();
                entity
                    .rearm_timer
                    .set_for_test(CdTimer::started(FRAME, ROF));
            }
            Some(6) => fixture.unpower(),
            Some(other) => panic!("{name}: GetFireError {other}"),
        }
        let draws_before = fixture.scenario_state();
        let (_, calls) = with_building_fire_stub(None, || {
            process_delayed_fire(
                &mut fixture.sim,
                master,
                &fixture.rules,
                ObjectAiCtx::default(),
                &mut false,
            )
        });
        assert_eq!(
            fixture.scenario_state(),
            draws_before,
            "{name}: no Scenario draw"
        );
        let requested = calls
            .first()
            .map(|(_, shot, multiplier)| (*shot, *multiplier));
        let fire_at = row["calls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|call| call[0] == "fire_at");
        match (requested, fire_at) {
            (None, None) => {}
            (Some((BuildingShot::Delayed { weapon, target }, multiplier)), Some(fire_at)) => {
                assert_eq!(weapon, fire_at[2].as_i64().unwrap() as i32, "{name} weapon");
                assert_eq!(
                    Some(target),
                    fixture
                        .sim
                        .substrate
                        .entities
                        .get(master)
                        .and_then(|entity| entity.attack_target.target()),
                    "{name} target"
                );
                assert_eq!(
                    Some(i64::from(multiplier)),
                    row["bullet_multiplier"].as_i64(),
                    "{name} multiplier"
                );
            }
            (requested, fire_at) => panic!("{name}: {requested:?}, native {fire_at:?}"),
        }
        let entity = fixture.sim.substrate.entities.get(master).unwrap();
        let native = &row["master"];
        assert_eq!(
            i64::from(entity.prism_support_count.count()),
            native["count"].as_i64().unwrap(),
            "{name} count"
        );
        let (mode, countdown) = delayed_fire(entity);
        assert_eq!(mode, native["mode"].as_i64().unwrap(), "{name} mode");
        assert_eq!(
            i64::from(countdown),
            native["countdown"].as_i64().unwrap(),
            "{name} countdown"
        );
        compared += 1;
    }
    assert_eq!(compared, 57);
}

/// Every `damage` row: the damage DetonateAtCoord hands Apply_area_damage,
/// `(multiplier * damage) >> 8` with the 32-bit product wrapping.
#[test]
fn prism_bonus_damage_matches_the_original() {
    let golden = golden();
    let damage = rows(&golden, "damage");
    assert_eq!(damage.len(), 66);
    let mut interner = crate::sim::intern::StringInterner::new();
    for row in damage {
        let input = &row["input"];
        let payload = ProjectilePayload::new(
            input["damage"].as_i64().unwrap() as i32,
            interner.intern("PRISM"),
            None,
            false,
        )
        .with_damage_multiplier(input["multiplier"].as_i64().unwrap() as i32);
        assert_eq!(
            i64::from(payload.area_damage()),
            row["damage"].as_i64().unwrap(),
            "{}",
            input["name"]
        );
    }
}

/// Every `beam` row: a support beam's end clears the count and starts the
/// downtime, {Frame, `PrismSupportDelay=`}, whether or not the laser was
/// made, with no check of drain, sale or power, no request and no Scenario
/// draw; a countdown still running only counts down.
#[test]
fn prism_support_beam_matches_the_original() {
    let golden = golden();
    let beam = rows(&golden, "beam");
    assert_eq!(beam.len(), 7);
    for row in beam {
        let input = &row["input"];
        let name = input["name"].as_str().unwrap();
        let mut fixture = Fixture::new(&serde_json::json!({}), "supporter");
        let supporter = fixture.id("supporter");
        fixture.apply(supporter, &input["supporter"]);
        if let Some(delay) = input["rules"]["delay"].as_i64() {
            fixture.rules.general.prism_support.delay = delay as i32;
        }
        if input["powered"]["supporter"] == false {
            fixture.unpower();
        }
        let draws_before = fixture.scenario_state();
        process_delayed_fire(
            &mut fixture.sim,
            supporter,
            &fixture.rules,
            ObjectAiCtx::default(),
            &mut false,
        );
        assert!(fixture.sim.fire_events.is_empty(), "{name}");
        assert_eq!(
            fixture.scenario_state(),
            draws_before,
            "{name}: no Scenario draw"
        );
        fixture.assert_state(supporter, &row["supporter"], &format!("beam {name}"));
    }
}

/// Every building's delayed fire, by id.
fn pending_fires(fixture: &Fixture) -> BTreeMap<u64, Option<PendingBuildingFire>> {
    fixture
        .buildings
        .values()
        .map(|&id| {
            let entity = fixture.sim.substrate.entities.get(id).unwrap();
            (id, entity.pending_building_fire.pending())
        })
        .collect()
}

/// One cadence event, applied before the frame's building turns.
fn apply_event(fixture: &mut Fixture, action: &str, building: u64) {
    let target = fixture.target;
    match action {
        // TarCom written as the passive scan or a detach writes it.
        "acquire" | "lose" => {
            let entity = fixture.sim.substrate.entities.get_mut(building).unwrap();
            entity
                .attack_target
                .set_for_test((action == "acquire").then(|| TargetKind::Entity(target)));
        }
        // The IDLE event's BuildingClass::SetTarget(0).
        "stop" => {
            fixture
                .sim
                .assign_target_represented(
                    building,
                    None,
                    &fixture.rules,
                    None,
                    crate::sim::world::FrameEffects::default(),
                )
                .unwrap();
        }
        "out_of_range" => fixture.move_target(OUT_OF_RANGE),
        "in_range" => fixture.move_target(IN_RANGE),
        "unpower" => fixture.unpower(),
        other => panic!("cadence event {other}"),
    }
}

/// Every `cadence` row through BuildingClass::Update's mission pieces, per
/// frame and per building in the row's Logic order as the oracle runs them:
/// the two ready checks around MissionClass::AI, then ProcessDelayedFire.
/// The oracle replaces FireAt with a bullet return and exact ROF45; this
/// caller-only replay installs the same substitute at the synchronous live
/// receiver boundary. Later tower visits read the actual rearm immediately,
/// including `k_shooter_rearms_before_a_later_walk`. Each returned bullet's
/// support multiplier and the entire per-tower state are compared before
/// moving to the next native object. Full live-fire tests run separately.
/// Guard dispatch's `RandomRanged(0, 2)` is pinned by
/// `building_guard_attack.json`'s replay; here VERA's delay is checked to be
/// AARate's 14 plus 0..=2 and replaced by the native one, since each
/// supporter's Guard cadence decides when a tower with its own target turns
/// master.
#[test]
fn prism_cadence_matches_the_original() {
    let golden = golden();
    let cadence = rows(&golden, "cadence");
    assert_eq!(cadence.len(), 17);
    let (mut recruits, mut arms, mut beams, mut shots) = (0, 0, 0, 0);
    for row in cadence {
        let input = &row["input"];
        let name = input["name"].as_str().unwrap();
        let mut fixture = Fixture::new(input, "master");
        let master = fixture.id("master");
        fixture.apply(master, &input["master"]);
        for tower in input["towers"].as_array().cloned().unwrap_or_default() {
            let id = fixture.id(tower["name"].as_str().unwrap());
            fixture.apply(id, &tower);
        }
        let order: Vec<u64> = match input["order"].as_array() {
            Some(order) => order
                .iter()
                .map(|building| fixture.id(building.as_str().unwrap()))
                .collect(),
            None => fixture.sim.houses[&fixture.owner]
                .base_projection()
                .buildings()
                .to_vec(),
        };
        let events = input["events"].as_array().unwrap();
        for frame in row["frames"].as_array().unwrap() {
            let k = frame["frame"].as_i64().unwrap();
            let now = FRAME + k as i32;
            fixture.sim.session.set_binary_frame_for_test(now as u32);
            for event in events.iter().filter(|event| event[0] == k) {
                let building = fixture.id(event[2].as_str().unwrap());
                apply_event(&mut fixture, event[1].as_str().unwrap(), building);
            }
            for &id in &order {
                let building = fixture.name(id).to_string();
                let at = format!("{name} frame {k} {building}");
                let native = &frame["towers"][&building];
                let native_events = native["events"].as_array().unwrap();
                let native_event = |kind: &str| native_events.iter().find(|event| event[0] == kind);

                ready_commence(&fixture.sim.session, &mut fixture.sim.substrate, id, true);
                let entity = fixture.sim.substrate.entities.get(id).unwrap();
                let timer_before = entity.mission.dispatch_timer();
                let mission_before = entity.mission.current();
                let pending_before = pending_fires(&fixture);
                dispatch(
                    &mut fixture.sim,
                    id,
                    &fixture.rules,
                    ObjectAiCtx::default(),
                    &mut false,
                );
                let entity = fixture.sim.substrate.entities.get_mut(id).unwrap();
                let timer = entity.mission.dispatch_timer();
                let native_handler = native_event("guard").or(native_event("attack"));
                assert_eq!(
                    timer != timer_before,
                    native_handler.is_some(),
                    "{at} MissionClass::AI"
                );
                if let Some(handler) = native_handler {
                    let returned = handler[1].as_i64().unwrap() as i32;
                    assert_eq!(timer.start_frame(), now, "{at} dispatch start");
                    if handler[0] == "guard" && returned != 1 {
                        assert_eq!(mission_before, mission_of("guard"), "{at}");
                        assert!((14..=16).contains(&timer.delay()), "{at} Guard delay");
                        entity.mission.write_dispatch_epilogue(now, returned);
                    } else {
                        assert_eq!(timer.delay(), returned, "{at} return");
                    }
                }
                let pending_after = pending_fires(&fixture);
                let armed: Vec<(u64, PendingBuildingFire)> = pending_after
                    .iter()
                    .filter_map(|(&other, &after)| {
                        let after = after?;
                        (pending_before[&other] != Some(after)).then_some((other, after))
                    })
                    .collect();
                assert!(armed.len() <= 1, "{at}: one arm per handler");
                let recruit = armed.iter().find_map(|&(other, fire)| {
                    let beam = fire.remaining_ticks == DELAY
                        && matches!(fire.fire, DelayedFire::SupportBeam { .. });
                    (other != id && beam).then(|| fixture.name(other).to_string())
                });
                assert_eq!(
                    recruit.as_deref(),
                    native_event("recruit").and_then(|event| event[1].as_str()),
                    "{at} recruit"
                );
                let arm =
                    armed.contains(&(id, PendingBuildingFire::new(DELAY, DelayedFire::Weapon(0))));
                assert_eq!(arm, native_event("arm").is_some(), "{at} arm");
                recruits += usize::from(recruit.is_some());
                arms += usize::from(arm);

                ready_commence(&fixture.sim.session, &mut fixture.sim.substrate, id, false);
                let beaming = pending_after[&id].is_some_and(|pending| {
                    pending.remaining_ticks() <= 1
                        && matches!(pending.fire(), DelayedFire::SupportBeam { .. })
                });
                let (_, calls) = with_building_fire_stub(Some(ROF), || {
                    process_delayed_fire(
                        &mut fixture.sim,
                        id,
                        &fixture.rules,
                        ObjectAiCtx::default(),
                        &mut false,
                    )
                });
                assert_eq!(beaming, native_event("beam").is_some(), "{at} beam");
                beams += usize::from(beaming);
                match (calls.as_slice(), native_event("shot")) {
                    ([], None) => {}
                    (
                        [(called_id, BuildingShot::Delayed { weapon: 0, .. }, multiplier)],
                        Some(shot),
                    ) => {
                        assert_eq!(*called_id, id, "{at} firer");
                        assert_eq!(
                            i64::from(*multiplier),
                            shot[3].as_i64().unwrap(),
                            "{at} multiplier"
                        );
                        shots += 1;
                    }
                    (calls, native) => panic!("{at}: calls {calls:?}, native {native:?}"),
                }
                fixture.assert_state(id, native, &at);
            }
        }
    }
    // Every recruit, arm, beam and shot of the rows.
    assert_eq!((recruits, arms, beams, shots), (40, 22, 38, 17));
}

/// Every `reader` row through the production reader: each pass is a Rules
/// layer processed by `RulesLayerStack` and projected by
/// `RuleSet::from_processed_rules`. A pass with a `[General]` section
/// re-reads the modifier as `ftol(ReadDouble(key, current) x 100)`, so one
/// without the key multiplies it by 100 again; a pass without the section
/// reads nothing; `none` and `<none>` clear PrismType. A row's `initial`
/// modifier is an earlier pass naming it. The oracle's keyless `[General]`
/// is a section the INI loader never inserts (a physical section with no
/// entries is dropped, `rules::ini_parser`), so the replay gives it one key
/// ReadGeneral does not read.
#[test]
#[cfg(not(feature = "ares"))]
fn prism_general_reader_matches_the_original() {
    let golden = golden();
    let reader = rows(&golden, "reader");
    assert_eq!(reader.len(), 22);
    let rules_of = |native: &Value| PrismSupportRules {
        modifier: native["modifier"].as_i64().unwrap() as i32,
        max: native["max"].as_i64().unwrap() as i32,
        delay: native["delay"].as_i64().unwrap() as i32,
        duration: native["duration"].as_i64().unwrap() as i32,
    };
    for row in reader {
        let input = &row["input"];
        let name = input["name"].as_str().unwrap();
        assert_eq!(
            PrismSupportRules::default(),
            rules_of(&row["defaults"]),
            "{name} constructor"
        );
        let mut passes: Vec<Option<String>> = Vec::new();
        if let Some(initial) = input["initial"]["modifier"].as_i64() {
            passes.push(Some(format!("PrismSupportModifier={initial}%\n")));
        }
        let first_row_pass = passes.len();
        for pass in input["passes"].as_array().unwrap() {
            passes.push(pass.as_object().map(|keys| {
                if keys.is_empty() {
                    return "FixtureOnly=1\n".to_string();
                }
                keys.iter()
                    .map(|(key, value)| format!("{key}={}\n", value.as_str().unwrap()))
                    .collect()
            }));
        }
        let layer = |index: usize| {
            let general = passes[index]
                .as_ref()
                .map_or(String::new(), |keys| format!("[General]\n{keys}"));
            let types = if index == 0 {
                "[BuildingTypes]\n0=GAPOWR\n1=ATESLA\n"
            } else {
                ""
            };
            IniFile::from_str(&format!("{general}{types}"))
        };
        for (index, native) in row["passes"].as_array().unwrap().iter().enumerate() {
            let last = first_row_pass + index;
            let mut stack = RulesLayerStack::new(layer(0));
            for pass in 1..=last {
                let kind = if pass == 1 {
                    RulesLayerKind::GameMode
                } else {
                    RulesLayerKind::Scenario
                };
                stack.push(kind, layer(pass));
            }
            let rules = RuleSet::from_processed_rules(&stack.process().unwrap()).unwrap();
            let at = format!("{name} pass {index}");
            assert_eq!(rules.general.prism_support, rules_of(native), "{at}");
            assert_eq!(
                rules.general.prism_type.as_deref(),
                native["prism_type"].as_str(),
                "{at} PrismType"
            );
        }
    }
}

/// An admitted pure startup declaration reaches the first General read and
/// remains the same input across isolated previews, reset and saved adoption.
/// Both profiles run this case: explicit false must override a legacy host.
#[test]
#[ignore = "requires VERA_SDK_PRISM_DEFAULT_WASM / VERA_SDK_PRISM_DEFAULT_PREVIOUS_WASM"]
fn ares_prism_modifier_retains_percent_across_map_layers_and_scales_one_shot() {
    use crate::assets::asset_manager::{MediaArchiveMode, test_support::audio_test_install};
    use crate::rules::{
        audio_sources::AudioDefinitions, process_owner::NativeScenarioRulesPrefix,
        retail_sources::RetailRulesSources,
    };
    use crate::sim::runtime::{ScenarioGameplayProgram, bind_scenario_gameplay_resources};
    use crate::util::save_codec;
    use vera_mod_api::{Context, HookKind, ModuleManifest, Plan, Request};

    let component = std::fs::read(
        std::env::var_os("VERA_SDK_PRISM_DEFAULT_WASM").expect("set VERA_SDK_PRISM_DEFAULT_WASM"),
    )
    .unwrap();
    let previous = std::fs::read(
        std::env::var_os("VERA_SDK_PRISM_DEFAULT_PREVIOUS_WASM")
            .expect("set VERA_SDK_PRISM_DEFAULT_PREVIOUS_WASM"),
    )
    .unwrap();
    let request = Request {
        api_version: vera_mod_api::API_VERSION,
        context: Context::PrismSupportDefaults,
        config: vec![],
        state: vec![],
    };
    let unchanged = Plan::unchanged(&request);
    let mut selected = unchanged.clone();
    selected.prism_support_defaults = Some(false);
    let off = vera_mod_runtime::test_support::component_fixture_by_kind(
        &ModuleManifest {
            api_version: vera_mod_api::API_VERSION,
            id: "test.prism-stock".into(),
            version: "1".into(),
            config_version: 1,
            state_version: 1,
            hooks: vec![
                HookKind::Configure,
                HookKind::Attach,
                HookKind::Damage,
                HookKind::Tick,
                HookKind::PrismSupportDefaults,
            ],
        },
        &serde_json::to_vec(&unchanged).unwrap(),
        &[(
            "prism_support_defaults",
            &serde_json::to_vec(&selected).unwrap(),
        )],
    );
    let golden = golden();
    let reader_modifier = |name: &str, pass: usize| -> i32 {
        rows(golden, "reader")
            .iter()
            .find(|row| row["input"]["name"] == name)
            .unwrap()["passes"][pass]["modifier"]
            .as_i64()
            .unwrap() as i32
    };
    let ctor = rows(golden, "reader")[0]["defaults"]["modifier"]
        .as_i64()
        .unwrap() as i32;
    let explicit = IniFile::from_str("[General]\nPrismSupportModifier=150%\n");
    let keyless = IniFile::from_str("[General]\nFixtureOnly=1\n");
    let no_general = IniFile::from_str("[Fixture]\nName=No General section\n");
    let mut controls = Vec::new();
    for (label, module, declaration) in [
        ("prism-current", Some(component.as_slice()), Some(true)),
        ("prism-no-module", None, None),
        ("prism-previous", Some(previous.as_slice()), None),
        ("prism-explicit-stock", Some(off.as_slice()), Some(false)),
    ] {
        eprintln!("Prism default reader control: {label}");
        let directory = audio_test_install(label);
        let (root, art) = rule_sources(2048, None);
        // Keep the original tower, projectile, mission and ART fixture. Only
        // the Prism modifier key is absent from this physical root General.
        let root = root.replace("PrismSupportModifier=150%\n", "");
        for (name, bytes) in [
            ("rulesmd.ini", root.as_bytes()),
            ("artmd.ini", art.as_bytes()),
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
        assert_eq!(
            assets.startup_resources().prism_support_defaults,
            declaration
        );
        let audio = AudioDefinitions::select(&assets);
        let mut owner = RetailRulesSources::select(&assets)
            .unwrap()
            .into_owner(audio.sounds().clone(), audio.eva().clone())
            .unwrap();
        let normalized = declaration.unwrap_or(crate::rules::ARES_ENABLED);
        let isolated = owner.startup_compatibility_projection().unwrap();
        assert_eq!(isolated.prism_support_defaults(), declaration);
        let isolated_rules = RuleSet::from_processed_rules(&isolated).unwrap();
        assert_eq!(
            isolated_rules.general.prism_support.modifier,
            if normalized {
                ctor
            } else {
                reader_modifier("general_without_keys_current_100", 0)
            },
            "the selector already owns an isolated cold root's missing-key default"
        );

        // Seed a reached native field with an explicit value. The next real
        // RMG path performs keyless E_multi General, then keyless reset-root
        // General; a map without General performs no third multiplication.
        let seeded = owner
            .load_scenario(NativeScenarioRulesPrefix::RandomMap(None), &explicit)
            .unwrap();
        assert_eq!(
            seeded.into_parts().0.general.prism_support.modifier,
            reader_modifier("modifier_150pct", 0)
        );
        let loaded = owner
            .load_scenario(NativeScenarioRulesPrefix::RandomMap(None), &no_general)
            .unwrap();
        assert_eq!(
            owner
                .saved_scenario_rules()
                .unwrap()
                .prism_support_defaults(),
            declaration
        );
        let actual = loaded.into_parts().0.general.prism_support.modifier;
        assert_eq!(
            actual,
            if normalized {
                reader_modifier("modifier_150pct", 0)
            } else {
                reader_modifier("retail_then_general_without_keys_twice", 2)
            },
            "RMG prepass and reset each use the same missing-key default; absent General does not read"
        );
        controls.push(actual);
        if declaration != Some(true) {
            continue;
        }

        // Preserve the existing Ares 125% retention regression, now through
        // physical source admission and the actual root/mode/map owners.
        owner
            .load_scenario(
                NativeScenarioRulesPrefix::RandomMap(None),
                &IniFile::from_str("[General]\nPrismSupportModifier=125%\n"),
            )
            .unwrap();
        let (mut rules, _, art, _) = owner
            .load_scenario(
                NativeScenarioRulesPrefix::RandomMap(Some(&keyless)),
                &keyless,
            )
            .unwrap()
            .into_parts();
        assert_eq!(rules.general.prism_support.modifier, 125);
        let preview = owner
            .random_map_preview_projection(Some(&keyless), "TEMPERATE")
            .unwrap();
        assert_eq!(preview.prism_support_defaults(), Some(true));
        assert_eq!(
            RuleSet::from_processed_rules(&preview)
                .unwrap()
                .general
                .prism_support
                .modifier,
            ctor,
            "isolated RMG starts from constructor state, with the same frozen selector"
        );
        assert_eq!(
            owner.saved_scenario_rules().unwrap().prism_support(),
            rules.general.prism_support,
            "preview cannot replace the continuing native registry"
        );
        install_prism_fixture_art(&mut rules, &art);
        let mut simulation = Simulation::new();
        bind_scenario_gameplay_resources(
            &mut rules,
            &assets,
            ScenarioGameplayProgram::Fresh {
                simulation: &mut simulation,
                rules_owner: &mut owner,
            },
            "tem",
            "TEMPERATE",
        )
        .unwrap();
        let saved =
            save_codec::serialize(&(owner.saved_scenario_rules().unwrap(), &simulation)).unwrap();
        let (processed, restored): (
            crate::rules::native_processing::ProcessedRulesLayers,
            Simulation,
        ) = save_codec::deserialize(&saved).unwrap();
        assert_eq!(
            processed.prism_support_defaults(),
            Some(true),
            "native envelope serializes the selector"
        );
        let mut restored_rules = owner.project_saved_scenario_rules(&processed).unwrap();
        assert_eq!(
            restored_rules.general.prism_support,
            rules.general.prism_support
        );
        install_prism_fixture_art(&mut restored_rules, &art);
        bind_scenario_gameplay_resources(
            &mut restored_rules,
            &assets,
            ScenarioGameplayProgram::Saved(&restored),
            "tem",
            "TEMPERATE",
        )
        .unwrap();
        owner.install_saved_scenario_rules(processed, Some(&restored_rules));
        let after_reset = owner
            .load_scenario(
                NativeScenarioRulesPrefix::RandomMap(Some(&keyless)),
                &keyless,
            )
            .unwrap();
        assert_eq!(
            after_reset.into_parts().0.general.prism_support.modifier,
            125,
            "saved adoption and another real Type reset retain the same default units"
        );
        assert_eq!(
            owner
                .saved_scenario_rules()
                .unwrap()
                .prism_support_defaults(),
            Some(true)
        );

        // The original live tower fixture recruits its support count and arms
        // its delayed shot through the production Prism arm. No FireAt stub,
        // synthetic support count or numeric multiplier supplies this result.
        let input = &rows(golden, "cadence")
            .iter()
            .find(|row| row["input"]["name"] == "b_supporters_after_1")
            .unwrap()["input"];
        let mut fixture = Fixture::with_sim(input, "master", restored_rules, restored);
        let master = fixture.id("master");
        let supporter = fixture.id("s1");
        fixture.apply(
            master,
            &serde_json::json!({"mission":"attack", "target":true}),
        );
        let obj = fixture.rules.object("TWR").unwrap().clone();
        prism_arm(&mut fixture.sim, master, &fixture.rules, &obj);
        assert_eq!(
            fixture
                .sim
                .substrate
                .entities
                .get(master)
                .unwrap()
                .prism_support_count
                .count(),
            1
        );
        assert!(matches!(
            fixture
                .sim
                .substrate
                .entities
                .get(supporter)
                .unwrap()
                .pending_building_fire
                .pending()
                .unwrap()
                .fire(),
            DelayedFire::SupportBeam { .. }
        ));
        prism_arm(&mut fixture.sim, master, &fixture.rules, &obj);
        assert_eq!(
            delayed_fire(fixture.sim.substrate.entities.get(master).unwrap()),
            (1, DELAY)
        );
        for step in 0..DELAY {
            fixture
                .sim
                .session
                .set_binary_frame_for_test((FRAME + step) as u32);
            for id in [master, supporter] {
                process_delayed_fire(
                    &mut fixture.sim,
                    id,
                    &fixture.rules,
                    ObjectAiCtx::default(),
                    &mut false,
                );
            }
        }
        let bullets: Vec<_> = fixture
            .sim
            .projectiles()
            .iter()
            .map(|(_, bullet)| bullet.payload().damage_multiplier())
            .collect();
        assert_eq!(
            bullets,
            [576],
            "the existing 125% one-support result reaches a real bullet"
        );
        assert_eq!(
            fixture
                .sim
                .substrate
                .entities
                .get(master)
                .unwrap()
                .prism_support_count
                .count(),
            0
        );
        assert_eq!(
            fixture
                .sim
                .substrate
                .take_support_bonus(master, &fixture.rules),
            256,
            "the support bonus is consumed once"
        );
        process_delayed_fire(
            &mut fixture.sim,
            master,
            &fixture.rules,
            ObjectAiCtx::default(),
            &mut false,
        );
        assert_eq!(
            fixture.sim.projectiles().len(),
            1,
            "an expired delayed fire cannot launch twice"
        );

        // Explicit values still use the unique original ReadDouble/x87 kernel.
        // Every expected value comes from the pre-existing native reader rows.
        for (value, row) in [
            ("150%", "modifier_150pct"),
            ("1.5", "modifier_1.5"),
            ("150", "modifier_150"),
            ("35%", "modifier_35pct"),
        ] {
            let map = IniFile::from_str(&format!("[General]\nPrismSupportModifier={value}\n"));
            let result = owner
                .load_scenario(NativeScenarioRulesPrefix::RandomMap(None), &map)
                .unwrap();
            assert_eq!(
                result.into_parts().0.general.prism_support.modifier,
                reader_modifier(row, 0),
                "explicit {value}"
            );
        }
    }
    assert_eq!(
        controls[1], controls[2],
        "no hook preserves the compiled profile"
    );
    if crate::rules::ARES_ENABLED {
        assert_eq!(controls[0], controls[1]);
        assert_ne!(
            controls[0], controls[3],
            "explicit false overrides the legacy default"
        );
    } else {
        assert_eq!(controls[1], controls[3]);
        assert_ne!(
            controls[0], controls[1],
            "the ordinary engine uses the actual component declaration"
        );
    }
}
