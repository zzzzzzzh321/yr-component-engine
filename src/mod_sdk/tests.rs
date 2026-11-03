use super::{Program, host::NativeHost};
use crate::rules::ini_parser::IniFile;
use crate::rules::native_processing::{RulesLayerKind, RulesLayerStack};
use std::sync::Arc;
use vera_mod_api::{self as api, Context};
use vera_mod_runtime::HostServices;

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
    let host = NativeHost::configuration(
        IniFile::from_str(
            "[S]\nNumber=$12\nNegative=-255\nBad=garbage\nClear=<default>\nCppClear=<none>\nRate=0.125\nPrecise=0.10000000000000001\nName=none\nYes=  yes\nEmpty=\nMission=Hunt\nThreshold=inair\n",
        ),
        Vec::new(),
    );
    let read = |key, reader, current| {
        serde_json::from_slice::<serde_json::Value>(
            &host.config_read("S", key, reader, current).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(read("Number", "nullable-int", b"null"), 12);
    assert_eq!(read("Bad", "nullable-int", b"12"), 12);
    assert_eq!(read("Number", "ares-nullable-int", b"null"), 12);
    assert_eq!(read("Negative", "ares-nullable-int", b"null"), -255);
    assert_eq!(read("Clear", "ares-nullable-int", b"12"), 12);
    assert_eq!(read("Name", "ares-nullable-int", b"12"), 12);
    assert_eq!(
        read("Empty", "ares-nullable-int", b"null"),
        serde_json::Value::Null
    );
    assert_eq!(read("Yes", "ares-bool", b"false"), true);
    assert_eq!(read("Bad", "ares-bool", b"true"), true);
    assert_eq!(read("Empty", "ares-bool", b"true"), true);
    assert_eq!(read("Name", "ares-nullable-bool", b"true"), false);
    assert_eq!(read("CppClear", "ares-nullable-bool", b"true"), true);
    assert_eq!(
        read("Empty", "ares-nullable-bool", b"null"),
        serde_json::Value::Null
    );
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
    // The SDK must dispatch Ares Valueable<double> to its existing binary64
    // reader, not stock ReadDouble's intermediate binary32 conversion.
    let ini = IniFile::from_str("[S]\nPrecise=0.10000000000000001\n");
    let section = ini.section("S").unwrap();
    let expected =
        section.read_ares_double_bits("Precise", crate::util::native_x87::NativeF64Bits::ONE);
    assert_eq!(read("Precise", "ares-double-bits", b"0"), expected.bits());
    assert_ne!(
        expected.bits(),
        section.read_double("Precise", 1.0).to_bits()
    );
    assert_eq!(read("Bad", "ares-double-bits", b"17"), 17);
    assert_eq!(read("Threshold", "ares-emp-threshold", b"6"), -1);
    assert_eq!(read("Bad", "ares-emp-threshold", b"6"), 0);
    assert_eq!(read("Mission", "ares-nullable-mission", b"null"), 15);
    assert_eq!(read("Name", "ares-nullable-mission", b"15"), 15);
    assert_eq!(read("Number", "ares-nullable-mission", b"15"), 15);
}

#[test]
fn sdk_registered_references_use_each_types_reached_reader_not_the_final_catalog() {
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[BuildingTypes]\n0=EARLY\n[VehicleTypes]\n0=LATE\n[Warheads]\n0=EFFECT\n\
         [EARLY]\nAttachEffect.Animation=LATE_ANIM\n\
         [LATE]\nDeployingAnim=LATE_ANIM\n\
         [EFFECT]\nAttachEffect.Animation=LATE_ANIM\nClear=<none>\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[EARLY]\nAttachEffect.Animation=LATE_ANIM\n\
             [EFFECT]\nAttachEffect.Animation=UNKNOWN\n",
        ),
    );
    let processed = layers.process().unwrap();
    let host = |index: usize| {
        NativeHost::configuration(processed.sdk_passes()[index].clone(), Vec::new())
            .with_registries(processed.sdk_registries()[index].clone())
    };
    let read = |host: &NativeHost, owner, key, scope, old: Option<&str>| {
        let value = host
            .config_read(
                owner,
                key,
                &format!("ares-ref:$animations@{scope}"),
                &serde_json::to_vec(&old).unwrap(),
            )
            .unwrap();
        serde_json::from_slice::<Option<String>>(&value).unwrap()
    };
    let root = host(0);
    assert_eq!(
        root.configuration_input_bytes(),
        root.config_read("", "", "rules-registry-view", b"null")
            .unwrap()
            .len(),
        "fuel accounts for the exact frozen bulk payload supplied to the guest"
    );
    let nullable_read = |owner, key, scope| {
        let value = root
            .config_read(
                owner,
                key,
                &format!("ares-nullable-ref:$animations@{scope}"),
                &serde_json::to_vec(&api::NullableResourceName::Inherit).unwrap(),
            )
            .unwrap();
        serde_json::from_slice::<api::NullableResourceName>(&value).unwrap()
    };
    assert_eq!(
        nullable_read("EARLY", "AttachEffect.Animation", "$buildings"),
        api::NullableResourceName::Inherit
    );
    assert_eq!(
        nullable_read("EFFECT", "Clear", "$warheads"),
        api::NullableResourceName::Clear
    );
    assert_eq!(
        nullable_read("EFFECT", "AttachEffect.Animation", "$warheads"),
        api::NullableResourceName::Named("LATE_ANIM".into())
    );
    assert!(
        root.registry("$animations")
            .iter()
            .any(|id| id == "LATE_ANIM")
    );
    // Building's common extension precedes Vehicle's DeployingAnim allocation;
    // the later Warhead suffix can see that allocation in the SAME pass.
    assert_eq!(
        read(&root, "EARLY", "AttachEffect.Animation", "$buildings", None),
        None
    );
    let animation = read(&root, "EFFECT", "AttachEffect.Animation", "$warheads", None);
    assert_eq!(animation.as_deref(), Some("LATE_ANIM"));
    assert_eq!(
        read(&root, "EFFECT", "Clear", "$warheads", Some("LATE_ANIM")),
        None
    );
    let map = host(1);
    assert_eq!(
        read(&map, "EARLY", "AttachEffect.Animation", "$buildings", None),
        animation
    );
    assert_eq!(
        read(
            &map,
            "EFFECT",
            "AttachEffect.Animation",
            "$warheads",
            animation.as_deref()
        ),
        animation
    );
    assert!(
        map.registry("$vehicles").is_empty(),
        "missing bodies do not run a suffix"
    );
}

#[test]
fn sdk_registered_techno_references_keep_common_identity_and_physical_prefix() {
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[BuildingTypes]\n0=EARLY\n1=SHARED\n[VehicleTypes]\n0=SOURCE\n1=SHARED\n\
         [EARLY]\nConvert.Script=LATE\nFreeUnit=LATE\n\
         [SOURCE]\nConvert.Script=SHARED\nClear=<none>\n[SHARED]\nStrength=100\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[EARLY]\nConvert.Script=LATE\n[SOURCE]\nConvert.Script=UNKNOWN\n"),
    );
    let processed = layers.process().unwrap();
    let host = |index: usize| {
        NativeHost::configuration(processed.sdk_passes()[index].clone(), Vec::new())
            .with_registries(processed.sdk_registries()[index].clone())
    };
    let read = |host: &NativeHost, owner, key, scope, current: Option<api::ProductionTypeRef>| {
        let value = host
            .config_read(
                owner,
                key,
                &format!("ares-ref:$technos@{scope}"),
                &serde_json::to_vec(&current).unwrap(),
            )
            .unwrap();
        serde_json::from_slice::<Option<api::ProductionTypeRef>>(&value).unwrap()
    };
    let root = host(0);
    let first = Some(api::ProductionTypeRef {
        category: api::ProductionObjectKind::Building,
        ordinal: 1,
    });
    assert_eq!(
        read(&root, "SOURCE", "Convert.Script", "$vehicles", None),
        first,
        "Find returns the earlier Building identity, not the later same-name Unit"
    );
    assert_eq!(
        read(&root, "EARLY", "Convert.Script", "$buildings", None),
        None,
        "Building's common suffix precedes its own FreeUnit allocation"
    );
    assert_eq!(read(&root, "SOURCE", "Clear", "$vehicles", first), None);
    let late = processed.sdk_registries()[0]
        .production_view()
        .technos
        .iter()
        .find(|entry| entry.name == "LATE")
        .unwrap()
        .identity;
    let map = host(1);
    assert_eq!(
        read(&map, "EARLY", "Convert.Script", "$buildings", None),
        Some(late)
    );
    assert_eq!(
        read(&map, "SOURCE", "Convert.Script", "$vehicles", first),
        first
    );
    assert!(
        root.config_read(
            "SOURCE",
            "Convert.Script",
            "ares-ref:$technos@$infantry",
            b"null"
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires the reference component with INI-selected ammo in VERA_SDK_AMMO_WASM"]
fn sdk_resource_policy_replays_native_layers_and_rebinds_frozen_saved_config() {
    use crate::rules::ruleset::RuleSet;
    let wasm =
        std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").expect("set VERA_SDK_AMMO_WASM"))
            .unwrap();
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nPrimary=PAIR\nSecondary=FREE\nReloadAmount=2\nNoAmmoWeapon=1\nNoAmmoAmount=1\nNoManualFire=yes\nNoManualUnload=yes\nNoManualEnter=yes\nNoSelfGuardArea=yes\n\
         [PAIR]\nAmmo=$12\n[FREE]\nAmmo=0\n[pair]\nAmmo=99\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[PAIR]\nAmmo=invalid\n[TANK]\nReloadAmount=bad\nEmptyReloadAmount=3\nNoAmmoAmount=invalid\nNoManualFire=invalid\nNoManualUnload=\nNoManualEnter=no\n"),
    );
    let processed = layers.process().unwrap();
    let mut rules = RuleSet::from_processed_rules(&processed).unwrap();
    let digest = crate::util::sha256::sha256_hex(&wasm);
    let program = Program::load_with_registries(
        wasm,
        &digest,
        processed.sdk_passes(),
        &["TANK".into()],
        &[],
        &rules.weapon_names(),
        processed.sdk_registries(),
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
    rules.bind_resource_policies().unwrap();
    assert!(
        rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_BUILDING)
            .unwrap()
            .use_weapon_rof
    );
    let expected_type_policy = rules
        .object("TANK")
        .unwrap()
        .type_resource_policy()
        .unwrap();
    assert_eq!(
        expected_type_policy.input,
        Some(api::InputPolicy {
            deny_manual_attack: true,
            deny_manual_unload: true,
            hide_transport_entry: false,
            deny_self_guard_area: false,
        })
    );
    assert!(rules.object("TANK").unwrap().no_manual_fire());
    assert_eq!(
        expected_type_policy.reload,
        Some(api::ReloadPolicy {
            normal_amount: 2,
            empty_amount: 3
        })
    );
    assert_eq!(
        expected_type_policy.depleted_weapon,
        Some(api::DepletedWeaponPolicy {
            weapon_index: 1,
            threshold: 1
        })
    );
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
        IniFile::from_str("[PAIR]\nAmmo=99\n[VeraFeatures]\nAresReload=no\nAresNoAmmoWeapon=no\nAresManualControl=no\n[TANK]\nReloadAmount=99\nNoAmmoWeapon=0\nNoManualFire=no\n"),
    );
    let mut loaded_rules = RuleSet::from_rules_layers(&layers).unwrap();
    loaded_rules.external_mod = Some(Arc::new(
        crate::util::save_codec::deserialize(&saved).unwrap(),
    ));
    loaded_rules.bind_resource_policies().unwrap();
    assert!(
        loaded_rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_BUILDING)
            .unwrap()
            .use_weapon_rof,
        "saved guest rebuilds the cadence policy"
    );
    assert_eq!(
        loaded_rules.object("TANK").unwrap().type_resource_policy(),
        Some(expected_type_policy),
        "saved config owns type policies, not current INI"
    );
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

#[test]
#[ignore = "requires VERA_SDK_AMMO_WASM with default Ares installation"]
fn sdk_ares_opt_out_survives_missing_invalid_layers_and_only_explicit_reenable() {
    use crate::rules::ruleset::RuleSet;
    let bytes = std::fs::read(std::env::var_os("VERA_SDK_AMMO_WASM").unwrap()).unwrap();
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let mut layers = RulesLayerStack::new(IniFile::from_str(
        "[VeraFeatures]\nAresAmmo=no\nAresReload=no\nAresNoAmmoWeapon=no\nAresManualControl=no\n\
         [VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nPrimary=PAIR\nSecondary=PAIR\n\
         ReloadAmount=3\nNoAmmoWeapon=1\nNoManualFire=yes\n[PAIR]\nAmmo=2\n",
    ));
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str(
            "[VeraFeatures]\nAresAmmo=invalid\nAresReload=\nAresNoAmmoWeapon=invalid\n",
        ),
    );
    let load = |layers: &RulesLayerStack| {
        let processed = layers.process().unwrap();
        let rules = RuleSet::from_processed_rules(&processed).unwrap();
        Program::load_with_registries(
            bytes.clone(),
            &digest,
            processed.sdk_passes(),
            &["TANK".into()],
            &[],
            &rules.weapon_names(),
            processed.sdk_registries(),
        )
        .unwrap()
    };
    let disabled = load(&layers);
    let query = |program: &Program| {
        program
            .type_resource_policy("TANK", api::WEAPON_USER_UNIT)
            .unwrap()
    };
    assert!(disabled.weapon_resource_policy("PAIR").unwrap().is_none());
    let disabled_policy = query(&disabled).unwrap();
    assert!(
        disabled_policy.reload.is_none()
            && disabled_policy.depleted_weapon.is_none()
            && disabled_policy.input.is_none()
    );
    assert_eq!(disabled_policy.cloak_allowed, Some(true));
    let saved = crate::util::save_codec::serialize(&disabled).unwrap();
    layers.push(
        RulesLayerKind::Scenario,
        IniFile::from_str("[VeraFeatures]\nAresNoAmmoWeapon=yes\n"),
    );
    let enabled = load(&layers);
    let policy = query(&enabled).unwrap();
    assert_eq!(
        policy.depleted_weapon,
        Some(api::DepletedWeaponPolicy {
            weapon_index: 1,
            threshold: 0
        })
    );
    assert!(policy.reload.is_none() && policy.input.is_none());
    assert_eq!(policy.cloak_allowed, Some(true));
    assert!(enabled.weapon_resource_policy("PAIR").unwrap().is_none());
    let restored: Program = crate::util::save_codec::deserialize(&saved).unwrap();
    restored.validate_restore().unwrap();
    let restored_policy = query(&restored).unwrap();
    assert!(
        restored_policy.reload.is_none()
            && restored_policy.depleted_weapon.is_none()
            && restored_policy.input.is_none(),
        "current INI cannot reenable a frozen disabled module"
    );
    assert_eq!(restored_policy.cloak_allowed, Some(true));
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
        "[VeraFeatures]\nShield=yes\n[ShieldTypes]\n0=TEST\n[TEST]\nStrength=80\nPassPercent=0.5\nInitialStrength=$12\n[TANK]\nShieldType=TEST\n",
    );
    let map = IniFile::from_str("[TEST]\nStrength=40\nInitialStrength=bad\n");
    let mut layers = RulesLayerStack::new(root);
    layers.push(RulesLayerKind::Scenario, map);
    let processed = layers.process().unwrap();
    let program = Program::load_with_registries(
        bytes,
        &digest,
        processed.sdk_passes(),
        &["TANK".into()],
        &["WH".into()],
        &[],
        processed.sdk_registries(),
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
    let context = shield_damage_context(1, 4);
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
    let mut poisoned = serde_json::to_value(&restored).unwrap();
    let byte = poisoned["bytes"][0].as_u64().unwrap();
    poisoned["bytes"][0] = serde_json::json!(byte ^ 1);
    let poisoned: Program = serde_json::from_value(poisoned).unwrap();
    assert!(poisoned.validate_restore().is_err());
}

#[test]
#[ignore = "requires the built reference component in VERA_SDK_COMPAT_WASM"]
fn sdk_invalid_component_state_stops_real_receiver_and_frame_without_fallback() {
    let ini = crate::sim::arena_fixture::clear_ground_ini(
        "[VeraFeatures]\nShield=yes\n[Countries]\n0=Americans\n1=Russians\n[VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nArmor=heavy\nSpeed=0\nShieldType=TEST\n[ShieldTypes]\n0=TEST\n[TEST]\nStrength=100\n[Warheads]\n0=WH\n[WH]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
    );
    let bytes =
        std::fs::read(std::env::var_os("VERA_SDK_COMPAT_WASM").expect("set VERA_SDK_COMPAT_WASM"))
            .unwrap();
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let processed = RulesLayerStack::new(ini).process().unwrap();
    let program = Program::load_with_registries(
        bytes,
        &digest,
        processed.sdk_passes(),
        &["TANK".into()],
        &["WH".into()],
        &[],
        processed.sdk_registries(),
    )
    .unwrap();
    let mut rules = crate::rules::ruleset::RuleSet::from_processed_rules(&processed).unwrap();
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

fn shield_damage_context(frame: i32, incoming: i32) -> Context {
    Context::Damage(api::DamageContext {
        frame,
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
        incoming,
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
    })
}

/// Compatibility is byte-exact, not merely equivalent JSON. Run this against
/// retained release artifacts; the old guest supplies the expected output.
#[test]
#[ignore = "requires VERA_SDK_BASELINE_WASM and VERA_SDK_CANDIDATE_WASM"]
fn sdk_refactor_preserves_frozen_configuration_and_entity_state_bytes() {
    let ini = IniFile::from_str(
        "[ShieldTypes]\n0=TEST\n[TEST]\nStrength=100\nSelfHealing=0.25\nSelfHealing.Rate=0.1\n\
         Respawn=0.5\nRespawn.Rate=0.1\n[TANK]\nShieldType=TEST\n[PAIR]\nAmmo=2\n\
         [VeraOverload]\nHeatLimit=30\nDuration=75\nCooling=1\n",
    );
    let processed = RulesLayerStack::new(ini).process().unwrap();
    let load = |key| {
        let bytes = std::fs::read(std::env::var_os(key).expect(key)).unwrap();
        let digest = crate::util::sha256::sha256_hex(&bytes);
        Program::load_with_registries(
            bytes,
            &digest,
            processed.sdk_passes(),
            &["TANK".into()],
            &["WH".into()],
            &["PAIR".into()],
            processed.sdk_registries(),
        )
        .unwrap()
    };
    let baseline = load("VERA_SDK_BASELINE_WASM");
    let candidate = load("VERA_SDK_CANDIDATE_WASM");
    assert_eq!(baseline.manifest(), candidate.manifest());
    let old_saved = serde_json::to_value(&baseline).unwrap();
    let new_saved = serde_json::to_value(&candidate).unwrap();
    assert_eq!(
        old_saved["config"], new_saved["config"],
        "frozen config bytes enter the checksum"
    );
    assert_eq!(
        baseline.weapon_resource_policy("PAIR").unwrap(),
        candidate.weapon_resource_policy("PAIR").unwrap()
    );
    let mut state = Vec::new();
    for context in [
        Context::Attach(api::AttachContext {
            frame: 0,
            entity: entity(),
        }),
        shield_damage_context(1, 35),
        Context::Tick(api::TickContext {
            frame: 30,
            entity: entity(),
        }),
        shield_damage_context(31, 70),
        Context::Tick(api::TickContext {
            frame: 200,
            entity: entity(),
        }),
    ] {
        let expected = baseline.evaluate(context.clone(), &state).unwrap();
        let actual = candidate.evaluate(context, &state).unwrap();
        assert_eq!(
            expected, actual,
            "candidate must accept retained opaque state unchanged"
        );
        state = expected.state;
    }
}

/// The same portable component selects all behavior from the real Rules INI
/// reader. Saved selection overrides the current work's changed INI on restore.
#[test]
#[ignore = "requires VERA_SDK_REFERENCE_WASM"]
fn sdk_ini_selection_is_layered_frozen_and_restored_with_the_program() {
    use crate::rules::ruleset::RuleSet;
    let bytes = std::fs::read(
        std::env::var_os("VERA_SDK_REFERENCE_WASM").expect("set VERA_SDK_REFERENCE_WASM"),
    )
    .unwrap();
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let load = |patch: &str| {
        let mut layers = RulesLayerStack::new(IniFile::from_str(
            "[VeraFeatures]\nOverload=yes\n\
             [VehicleTypes]\n0=TANK\n[TANK]\nStrength=300\nPrimary=PAIR\nShieldType=TEST\n\
             [PAIR]\nAmmo=2\n[ShieldTypes]\n0=TEST\n[TEST]\nStrength=100\n",
        ));
        layers.push(RulesLayerKind::Scenario, IniFile::from_str(patch));
        let processed = layers.process().unwrap();
        let rules = RuleSet::from_processed_rules(&processed).unwrap();
        Program::load_with_registries(
            bytes.clone(),
            &digest,
            processed.sdk_passes(),
            &["TANK".into()],
            &[],
            &rules.weapon_names(),
            processed.sdk_registries(),
        )
        .map(|program| (program, rules))
    };
    // Do not reject the early layer: its missing dependency is enabled later.
    let (enabled, _) = load("[VeraFeatures]\nShield=yes\nAresAmmo=invalid\n").unwrap();
    let saved = crate::util::save_codec::serialize(&enabled).unwrap();
    let (disabled, mut current_rules) =
        load("[VeraFeatures]\nShield=no\nOverload=no\nAresAmmo=no\nAresReload=no\nAresNoAmmoWeapon=no\nAresManualControl=no\n").unwrap();
    assert_eq!(
        enabled.sha256(),
        disabled.sha256(),
        "one component serves both works"
    );
    let attach = Context::Attach(api::AttachContext {
        frame: 0,
        entity: entity(),
    });
    assert!(
        disabled
            .evaluate(attach.clone(), &[])
            .unwrap()
            .state
            .is_empty()
    );
    assert_eq!(disabled.weapon_resource_policy("PAIR").unwrap(), None);
    assert!(
        load("")
            .unwrap_err()
            .contains("Overload=yes requires Shield=yes")
    );

    let restored: Program = crate::util::save_codec::deserialize(&saved).unwrap();
    restored.validate_restore().unwrap();
    current_rules.external_mod = Some(Arc::new(restored));
    current_rules.bind_resource_policies().unwrap();
    assert_eq!(
        current_rules
            .weapon("PAIR")
            .unwrap()
            .resource_policy(api::WEAPON_USER_UNIT)
            .unwrap()
            .cost,
        2
    );
    let restored = current_rules.external_mod.as_ref().unwrap();
    let attached = restored.evaluate(attach, &[]).unwrap();
    assert_eq!(attached.projection.unwrap().meters[0].current, 100);
    let first = restored
        .evaluate(shield_damage_context(10, 60), &attached.state)
        .unwrap();
    assert_eq!(first.damage, Some(0));
    assert!(!first.projection.unwrap().meters[0].active);
    assert_eq!(
        restored
            .evaluate(shield_damage_context(11, 10), &first.state)
            .unwrap()
            .damage,
        Some(10)
    );
    use std::hash::Hasher;
    let checksum = |program: &Program| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        program.fold(&mut hasher);
        hasher.finish()
    };
    assert_ne!(
        checksum(restored),
        checksum(&disabled),
        "frozen selection enters the program checksum"
    );
}
