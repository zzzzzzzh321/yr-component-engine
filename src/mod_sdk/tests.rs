use super::*;

fn entity() -> api::EntityContext {
    api::EntityContext {
        id: 1,
        type_name: "TANK".into(),
        owner: "Owner".into(),
        armor: "heavy".into(),
        health: 300,
        max_health: 300,
        alive: true,
        in_limbo: false,
        immobilized: false,
        temporal: false,
        being_warped_out: false,
        powered: true,
        chrono_warp_latch: false,
        ai_frozen: false,
        deactivated: false,
        immune: false,
        type_immune: false,
        invulnerable: false,
    }
}

#[test]
fn sdk_native_nullable_readers_preserve_clear_and_failed_conversion_policy() {
    let host = NativeHost {
        weapons: Vec::new(),
        ini: Some(Arc::new(IniFile::from_str(
            "[S]\nNumber=$12\nBad=garbage\nClear=<default>\nRate=0.125\nName=none\n",
        ))),
    };
    let read = |key, reader, current| {
        serde_json::from_slice::<serde_json::Value>(
            &host.config_read("S", key, reader, current).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(read("Number", "nullable-int", b"null"), 12);
    assert_eq!(read("Bad", "nullable-int", b"12"), 12);
    assert_eq!(
        read("Clear", "nullable-int", b"12"),
        serde_json::Value::Null
    );
    assert_eq!(
        read("Name", "nullable-name:128", br#""old""#),
        serde_json::Value::Null
    );
    assert_eq!(
        read("Rate", "nullable-double-bits", b"null"),
        0.125_f64.to_bits()
    );
}

#[test]
#[ignore = "requires the built ares-ammo component in VERA_SDK_AMMO_WASM"]
fn sdk_resource_policy_replays_native_layers_and_rebinds_frozen_saved_config() {
    use crate::rules::native_processing::{RulesLayerKind, RulesLayerStack};
    use crate::rules::ruleset::RuleSet;
    let wasm =
        std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").expect("set VERA_SDK_AMMO_WASM"))
            .unwrap();
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nPrimary=PAIR\nSecondary=FREE\n\
         [PAIR]\nAmmo=$12\n[FREE]\nAmmo=0\n[pair]\nAmmo=99\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[PAIR]\nAmmo=invalid\n"),
    );
    let processed = layers.process().unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    let digest = crate::util::sha256::sha256_hex(&wasm);
    let program = Program::load_with_types(
        wasm,
        &digest,
        processed.sdk_passes(),
        &["TANK".into()],
        &[],
        &rules.weapon_names(),
    )
    .unwrap();
    // Only the plugin interprets Ammo; the engine has not bound a policy yet.
    assert!(
        rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_UNIT)
            .is_none()
    );
    let saved = crate::util::save_codec::serialize(&program).unwrap();
    rules.external_mod = Some(Arc::new(program));
    rules.bind_weapon_resource_policies().unwrap();
    assert_eq!(
        rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_UNIT)
            .unwrap()
            .cost,
        12
    );
    assert_eq!(
        rules
            .weapon("FREE")
            .unwrap()
            .resource_policy(api::WEAPON_USER_INFANTRY)
            .unwrap()
            .cost,
        0
    );
    assert!(
        rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_AIRCRAFT)
            .is_none()
    );
    // New Rules contain a different current INI, but load must rebuild the
    // read-only projection from the exact saved component configuration.
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[PAIR]\nAmmo=99\n"),
    );
    let mut loaded_rules = RuleSet::from_rules_layers(&layers).unwrap();
    loaded_rules.external_mod = Some(Arc::new(
        crate::util::save_codec::deserialize(&saved).unwrap(),
    ));
    loaded_rules.bind_weapon_resource_policies().unwrap();
    assert_eq!(
        loaded_rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_UNIT)
            .unwrap()
            .cost,
        12
    );
}

/// The actual built component + production native readers, not a second config
/// parser. Build the SDK artifact and set VERA_SDK_COMPAT_WASM before running.
#[test]
#[ignore = "requires the built reference component in VERA_SDK_COMPAT_WASM"]
fn sdk_component_layers_and_preview_use_production_host_without_mutation() {
    let bytes =
        std::fs::read(std::env::var_os("VERA_SDK_COMPAT_WASM").expect("set VERA_SDK_COMPAT_WASM"))
            .unwrap();
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let root = IniFile::from_str(
        "[ShieldTypes]\n0=TEST\n[TEST]\nStrength=80\nPassPercent=0.5\nInitialStrength=$12\n",
    );
    let map = IniFile::from_str("[TEST]\nStrength=40\nInitialStrength=bad\n");
    let final_ini = IniFile::from_str("[TANK]\nShieldType=TEST\n");
    let program = Program::load_with_types(
        bytes,
        &digest,
        &[root, map, final_ini],
        &["TANK".into()],
        &["WH".into()],
        &[],
    )
    .unwrap();
    let attached = program
        .evaluate(
            Context::Attach(api::AttachContext {
                frame: 0,
                entity: entity(),
            }),
            &[],
        )
        .unwrap();
    let meter = &attached.projection.as_ref().unwrap().meters[0];
    assert_eq!((meter.current, meter.maximum), (12, 40));
    let context = Context::Damage(api::DamageContext {
        frame: 1,
        entity: entity(),
        attacker: None,
        source_owner: None,
        source_is_ally: false,
        source_is_owner: false,
        target_is_neutral: false,
        attacker_same_type_owner: false,
        warhead_name: "WH".into(),
        warhead_temporal: false,
        warhead_psychedelic: false,
        incoming: 4,
        distance_leptons: 0,
        ignore_defenses: false,
        no_damage: false,
        max_damage: 999,
        armor_multiplier_bits: 1_f64.to_bits(),
        verses: vec![api::ArmorVerses {
            armor: "none".into(),
            multiplier_bits: 1_f64.to_bits(),
        }],
        cell_spread_bits: 0_f64.to_bits(),
        percent_at_max_bits: 1_f64.to_bits(),
    });
    let preview = program.evaluate(context.clone(), &attached.state).unwrap();
    let committed = program.evaluate(context, &attached.state).unwrap();
    assert_eq!(preview, committed);
    assert_eq!(
        committed.damage,
        Some(2),
        "unmentioned PassPercent survives the second pass"
    );
    assert_eq!(committed.projection.as_ref().unwrap().meters[0].current, 8);
    assert_eq!(attached.projection.as_ref().unwrap().meters[0].current, 12);
    // Executable bytes/config survive the production save codec. Restore
    // validates the raw component again instead of trusting a cached executor.
    let saved = crate::util::save_codec::serialize(&program).unwrap();
    let restored: Program = crate::util::save_codec::deserialize(&saved).unwrap();
    restored.validate_restore().unwrap();
    assert_eq!(restored.sha256(), program.sha256());
    let mut poisoned: Program = crate::util::save_codec::deserialize(&saved).unwrap();
    poisoned.bytes[0] ^= 1;
    assert!(poisoned.validate_restore().is_err());
}

#[test]
#[ignore = "requires the built reference component in VERA_SDK_COMPAT_WASM"]
fn sdk_invalid_component_state_stops_real_receiver_and_frame_without_fallback() {
    let ini = crate::sim::arena_fixture::clear_ground_ini(
        "[Countries]\n0=Americans\n1=Russians\n[VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nArmor=heavy\nSpeed=0\nShieldType=TEST\n[ShieldTypes]\n0=TEST\n[TEST]\nStrength=100\n[Warheads]\n0=WH\n[WH]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
    );
    let bytes =
        std::fs::read(std::env::var_os("VERA_SDK_COMPAT_WASM").expect("set VERA_SDK_COMPAT_WASM"))
            .unwrap();
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let program = Program::load_with_types(
        bytes,
        &digest,
        &[ini.clone(), ini.clone()],
        &["TANK".into()],
        &["WH".into()],
        &[],
    )
    .unwrap();
    let mut rules = crate::rules::ruleset::RuleSet::from_ini(&ini).unwrap();
    rules.external_mod = Some(Arc::new(program.clone()));
    let mut sim = crate::sim::arena_fixture::rival_arena(&rules, 23);
    sim.external_mod = Some(program);
    let id = sim
        .spawn_object_at_height("TANK", "Americans", 10, 10, 0, 0, &rules)
        .unwrap();
    assert!(
        !sim.substrate
            .entities
            .get(id)
            .unwrap()
            .mod_component()
            .is_empty()
    );
    sim.substrate.entities.get_mut(id).unwrap().mod_component = vec![0xff];
    let warhead = sim.interner.intern("WH");
    let event = crate::sim::combat::EntityDamageEvent::direct_receiver(
        id,
        60,
        0,
        0,
        None,
        warhead,
        crate::sim::combat::ReceiverCallFlags {
            ignore_defenses: false,
            arg6: false,
        },
    );
    sim.commit_direct_damage_receiver(&rules, None, event, Default::default());
    assert_eq!(
        sim.substrate.entities.get(id).unwrap().health.current(),
        300,
        "guest failure must not fall through to hull damage"
    );
    assert!(sim.external_mod_fault.is_some());
    let frame = sim.session.binary_frame();
    assert!(
        sim.advance_master_frame(
            &[],
            &rules,
            None,
            33,
            crate::sim::world::TickLane::Ordinary,
            false,
            Default::default()
        )
        .is_err()
    );
    assert_eq!(sim.session.binary_frame(), frame);
    assert!(matches!(
        sim.restore_after_snapshot_load(),
        Err(crate::sim::snapshot::SnapshotRestoreError::InvalidExternalMod(_))
    ));
}
