//! Actual-component armor admission and its shared gameplay consumers.
use super::*;
use crate::mod_sdk::Program;
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{ProcessedRulesLayers, RulesLayerKind, RulesLayerStack};
use crate::rules::ruleset::RuleSet;
use std::sync::Arc;

fn component() -> Vec<u8> {
    assert!(!crate::rules::ARES_ENABLED, "exercise the ordinary engine");
    std::fs::read(std::env::var_os("VERA_SDK_ARMOR_WASM").expect("set VERA_SDK_ARMOR_WASM"))
        .unwrap()
}

fn admit(layers: &RulesLayerStack, component: Option<&[u8]>) -> RuleSet {
    let processed = layers.process().unwrap();
    admit_processed(&processed, component)
}

fn admit_processed(processed: &ProcessedRulesLayers, component: Option<&[u8]>) -> RuleSet {
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    if let Some(bytes) = component {
        let objects = rules
            .all_objects()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>();
        let warheads = rules
            .warheads_iter()
            .map(|w| w.id.clone())
            .collect::<Vec<_>>();
        rules.external_mod = Some(Arc::new(
            Program::load_with_registries(
                bytes.to_vec(),
                &crate::util::sha256::sha256_hex(bytes),
                processed.sdk_passes(),
                &objects,
                &warheads,
                &rules.weapon_names(),
                processed.sdk_registries(),
            )
            .unwrap(),
        ));
    }
    rules.bind_resource_policies().unwrap();
    rules
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_ARMOR_WASM"]
fn sdk_armor_retains_read_points_inheritance_and_absent_warhead_suffixes() {
    let wasm = component();
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[ArmorTypes]\npaper=steel\ncopy=paper\n\
         [VehicleTypes]\n0=EARLY\n1=LATE\n2=COPY\n\
         [TerrainTypes]\n0=TREE\n\
         [EARLY]\nArmor=future\n[LATE]\nArmor=paper\n[COPY]\nArmor=copy\n\
         [TREE]\nArmor=paper\n[Warheads]\n0=TestWH\n\
         [TestWH]\nVerses=100%,1 %,100%\nVersus.wood=50%\nVersus.steel=50%\nVersus.paper=0%\nVersus.paper.ForceFire=yes\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[ArmorTypes]\nfuture=wood\npaper=100%\n\
             [EARLY]\nStrength=100\n[LATE]\nArmor=future\n",
        ),
    );
    let rules = admit(&layers, Some(&wasm));
    let early = rules.object("EARLY").unwrap().armor_id();
    let late = rules.object("LATE").unwrap().armor_id();
    let copied = rules.object("COPY").unwrap().armor_id();
    let terrain = rules
        .terrain_object_type_case_insensitive("TREE")
        .unwrap()
        .armor_id();
    assert_eq!(early, crate::rules::armor::ArmorId::NONE);
    assert_eq!(rules.armor_name(late), Some("future"));
    let wh = rules.warhead("TestWH").unwrap();
    assert_eq!(
        wh.interaction(crate::rules::armor::armor_index("plate"))
            .multiplier()
            .bits(),
        1.0f64.to_bits(),
        "internal whitespace stays inside its comma-delimited token"
    );
    // Exact half isolates inheritance from native percent-parser rounding.
    assert_eq!(wh.interaction(late).multiplier().bits(), 0.5f64.to_bits());
    for armor in [copied, terrain] {
        let record = wh.interaction(armor);
        assert_eq!(record.multiplier().bits(), 0.0f64.to_bits());
        assert!(record.can_fire());
        assert!(!record.can_retaliate());
        assert!(!record.can_passively_acquire());
    }
    // Installing the module cannot retrospectively reinterpret the raw
    // identical Armor=future text of the early object.
    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 1);
    sim.intern_rule_type_ids(&rules);
    let first = sim
        .spawn_object("EARLY", "Russians", 8, 8, 0, &rules)
        .unwrap();
    let second = sim
        .spawn_object("LATE", "Russians", 10, 8, 0, &rules)
        .unwrap();
    sim.resolve_type_handles(&rules);
    assert_eq!(
        crate::mod_sdk::live_entity_context(&sim, first, &rules)
            .unwrap()
            .armor,
        "none"
    );
    assert_eq!(
        crate::mod_sdk::live_entity_context(&sim, second, &rules)
            .unwrap()
            .armor,
        "future"
    );
    let bare = admit(&layers, None);
    assert_eq!(bare.object("LATE").unwrap().armor_id(), early);
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresArmor=no\n"),
    );
    assert_eq!(
        admit(&layers, Some(&wasm))
            .object("LATE")
            .unwrap()
            .armor_id(),
        early
    );
}

#[test]
#[ignore = "requires the packaged Ares component in VERA_SDK_ARMOR_WASM"]
fn sdk_armor_wide_ids_reach_weapon_selection_damage_shields_and_saved_rebinding() {
    use crate::sim::combat::{TargetKind, combat_weapon, world_fire::FireVisit};
    use crate::sim::snapshot::GameSnapshot;
    let wasm = component();
    // One boundary index above255; this is not a per-index behavior sweep.
    let mut source = String::from("[ArmorTypes]\n");
    for index in 0..260 {
        source.push_str(&format!("custom{index}=steel\n"));
    }
    source.push_str(
        "[VehicleTypes]\n0=EMITTER\n1=TARGET\n2=PROTECTED\n\
         [EMITTER]\nStrength=1000\nArmor=heavy\nSpeed=6\nPrimary=BADGUN\nSecondary=GUN\n\
         Locomotor={4A582741-9839-11d1-B709-00A024DDAFD1}\nMovementZone=Normal\n\
         [TARGET]\nStrength=1000\nArmor=custom259\n\
         [PROTECTED]\nStrength=1000\nArmor=custom259\nShieldType=SDKArmorShield\n\
         [BADGUN]\nDamage=100\nRange=9.5\nROF=1\nSpeed=100\nProjectile=INVIS\nWarhead=NO_WIDE\n\
         [GUN]\nDamage=100\nRange=9.5\nROF=1\nSpeed=100\nProjectile=INVIS\nWarhead=TestWH\n\
         [INVIS]\nInviso=yes\nAG=yes\n\
         [Warheads]\n0=TestWH\n1=NO_WIDE\n\
         [TestWH]\nVersus.custom259=25%\n[NO_WIDE]\nVersus.custom259=0%\n\
         [ShieldTypes]\n0=SDKArmorShield\n[SDKArmorShield]\nStrength=100\nInheritArmorFromTechno=yes\n\
         [VeraFeatures]\nShield=yes\n",
    );
    let processed = RulesLayerStack::new(crate::sim::arena_fixture::clear_ground_ini(&source))
        .process()
        .unwrap();
    let rules = admit_processed(&processed, Some(&wasm));
    assert!(rules.object("TARGET").unwrap().armor_id().index() > usize::from(u8::MAX));
    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 2);
    sim.external_mod = rules.external_mod.as_deref().cloned();
    sim.intern_rule_type_ids(&rules);
    let emitter = sim
        .spawn_object("EMITTER", "Americans", 12, 10, 0, &rules)
        .unwrap();
    let target = sim
        .spawn_object("TARGET", "Russians", 10, 10, 0, &rules)
        .unwrap();
    let protected = sim
        .spawn_object("PROTECTED", "Russians", 10, 12, 0, &rules)
        .unwrap();
    sim.resolve_type_handles(&rules);
    let actor = sim.entities().get(emitter).unwrap();
    let object = rules.object("EMITTER").unwrap();
    let selected = combat_weapon::select_weapon_against(
        &rules,
        actor,
        object,
        &combat_weapon::attacker_facts(actor, object),
        Some(&TargetKind::Entity(target)),
        &sim,
        sim.interner(),
        sim.resolved_terrain(),
        Some(sim.house_alliances()),
    )
    .unwrap()
    .0;
    assert_eq!(
        selected, 1,
        "the real selector reads the custom armor record"
    );
    let fire = |sim: &mut Simulation, rules: &RuleSet, victim| {
        let existing = sim
            .projectiles
            .iter()
            .map(|(&id, _)| id)
            .collect::<std::collections::BTreeSet<_>>();
        sim.commit_fire_visit(
            FireVisit::Direct {
                id: emitter,
                target: TargetKind::Entity(victim),
                weapon_index: selected,
            },
            rules,
            None,
            FrameEffects::empty(),
        );
        let bullet = sim
            .projectiles
            .iter()
            .find(|(id, _)| !existing.contains(id))
            .map(|(&id, _)| id)
            .unwrap();
        sim.object_ai_visit_one(bullet, Some(rules), ObjectAiCtx::default());
        assert!(sim.external_mod_fault.is_none());
    };
    fire(&mut sim, &rules, target);
    assert_eq!(sim.entities().get(target).unwrap().health.current(), 975);
    fire(&mut sim, &rules, protected);
    assert_eq!(
        sim.entities().get(protected).unwrap().health.current(),
        1000,
        "shield receives the registered custom multiplier"
    );
    let saved = GameSnapshot::save(&sim, 0, 0, "SDK-ARMOR", 0);
    let mut restored = GameSnapshot::load(&saved).unwrap().sim;
    restored.restore_after_snapshot_load().unwrap();
    let retained_bytes = crate::util::save_codec::serialize(&processed).unwrap();
    let retained: ProcessedRulesLayers =
        crate::util::save_codec::deserialize(&retained_bytes).unwrap();
    // Rebuild from saved retained Rules, without its disposable Configure
    // metadata or any already-bound external caches from the live Rules.
    let mut rebound = RuleSet::from_processed_rules(&retained).unwrap();
    rebound.external_mod = restored.external_mod.as_ref().map(|p| Arc::new(p.clone()));
    rebound.bind_resource_policies().unwrap();
    assert_eq!(
        rebound.object("TARGET").unwrap().armor_id(),
        rules.object("TARGET").unwrap().armor_id()
    );
    fire(&mut restored, &rebound, target);
    assert_eq!(
        restored.entities().get(target).unwrap().health.current(),
        950
    );
}
