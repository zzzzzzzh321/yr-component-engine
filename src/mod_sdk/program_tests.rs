use super::*;

#[test]
fn sdk_hook_binary_ordinals_preserve_accepted_component_saves() {
    // Program persists ModuleManifest through the existing bincode1 layout.
    // These are the accepted API1 ordinals before the optional firing hook.
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::TypeResource).unwrap(),
        5u32.to_le_bytes()
    );
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::SuperWeaponResource).unwrap(),
        27u32.to_le_bytes()
    );
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::WeaponFiringResource).unwrap(),
        28u32.to_le_bytes()
    );
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::ProjectileResource).unwrap(),
        29u32.to_le_bytes()
    );
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::WeaponInstanceResource).unwrap(),
        30u32.to_le_bytes()
    );
    assert_eq!(
        crate::util::save_codec::serialize(&api::HookKind::MasterTypeLists).unwrap(),
        31u32.to_le_bytes()
    );
}

#[test]
fn repeated_admission_shares_compilation_without_trusting_a_cached_identity() {
    let request = Request {
        api_version: api::API_VERSION,
        context: Context::Configure(api::ConfigureContext {
            pass: 0,
            source: "rules".into(),
            final_pass: false,
            object_types: vec![],
            warheads: vec![],
        }),
        config: vec![],
        state: vec![],
    };
    let bytes = vera_mod_runtime::test_support::component_fixture(
        &api::ModuleManifest {
            api_version: api::API_VERSION,
            id: "admission-cache".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                api::HookKind::Configure,
                api::HookKind::Attach,
                api::HookKind::Damage,
                api::HookKind::Tick,
            ],
        },
        &serde_json::to_vec(&Plan::unchanged(&request)).unwrap(),
        "",
        "",
    );
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let first = AdmittedComponent::load(bytes.clone(), &digest).unwrap();
    let second = AdmittedComponent::load(bytes.clone(), &digest).unwrap();
    assert!(!first.requires_early_admission());
    assert!(Arc::ptr_eq(
        &first.executor.runtime,
        &second.executor.runtime
    ));
    assert!(
        second
            .executor
            .runtime
            .evaluate(
                &first.executor.module,
                &request,
                Arc::new(StartupHost::default()),
            )
            .is_ok(),
        "re-admissions must use the same runtime's compiled module cache"
    );
    let legacy = second.into_configuration_seed();
    assert!(!legacy.has_master_type_lists());
    assert_eq!(legacy.master_type_lists_policy().unwrap(), None);
    assert!(AdmittedComponent::load(bytes, &"0".repeat(64)).is_err());
}

#[test]
fn master_type_lists_uses_early_admission_and_ignores_retained_configuration() {
    let request = Request {
        api_version: api::API_VERSION,
        context: Context::MasterTypeLists,
        config: vec![],
        state: vec![],
    };
    let policy = api::MasterTypeListsPolicy { weapon_types: true };
    let mut plan = Plan::unchanged(&request);
    plan.master_type_lists = Some(policy);
    let bytes = vera_mod_runtime::test_support::component_fixture(
        &api::ModuleManifest {
            api_version: api::API_VERSION,
            id: "master-type-lists".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                api::HookKind::Configure,
                api::HookKind::Attach,
                api::HookKind::Damage,
                api::HookKind::Tick,
                api::HookKind::MasterTypeLists,
            ],
        },
        &serde_json::to_vec(&plan).unwrap(),
        "",
        "",
    );
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let admitted = AdmittedComponent::load(bytes, &digest).unwrap();
    assert!(admitted.requires_early_admission());
    assert!(super::super::startup::require_startup_frontier(&admitted, false).is_err());
    let mut program = admitted.into_configuration_seed();
    program.config.push(1);
    assert!(program.has_master_type_lists());
    // Request validation rejects this config for a pure query if the getter
    // accidentally routes through the ordinary configured evaluator.
    assert_eq!(program.master_type_lists_policy().unwrap(), Some(policy));
    assert_eq!(program.master_type_lists_policy().unwrap(), Some(policy));
    assert_eq!(program.config, [1]);
}

#[test]
fn effects_send_only_exact_resources_in_the_current_event() {
    use api::{EffectControl, EffectEvent, EffectResourceKind, EntityContext};
    let key = |kind, name: &str| api::EffectResourceContext {
        kind,
        name: name.into(),
    };
    let own = key(EffectResourceKind::Unit, "SAME");
    let hit = key(EffectResourceKind::Warhead, "SAME");
    let active = key(EffectResourceKind::Warhead, "ACTIVE");
    let unrelated = key(EffectResourceKind::Building, "SAME");
    let records = [
        (&own, "own"),
        (&hit, "hit"),
        (&active, "active"),
        (&unrelated, "unused"),
    ]
    .into_iter()
    .map(|(resource, value)| (resource.clone(), String::from(value)))
    .collect::<BTreeMap<_, _>>();
    let entry = api::EffectInstanceView {
        resource: active.clone(),
        remaining: 3,
        source_owner: None,
        has_animation: false,
    };
    let context = api::EffectContext {
        frame: 0,
        entity: EntityContext {
            id: 1,
            type_name: "SAME".into(),
            owner: "Multi1".into(),
            armor: "heavy".into(),
            health: 100,
            max_health: 100,
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
        },
        user: api::WEAPON_USER_UNIT,
        inside_transport: false,
        cloaked_or_cloaking: false,
        entries: vec![entry.clone(), entry],
        control: EffectControl::Disabled(0),
        animations_hidden: false,
        event: EffectEvent::Apply {
            resource: hit.clone(),
            source_owner: None,
            source_is_owner: false,
            source_is_ally: false,
            affects_allies: true,
            verses_bits: 1.0_f64.to_bits(),
        },
        configuration: None,
    };

    let mut projected = Context::Effects(context.clone());
    assert!(scope_effect_configuration(&mut projected, &records));
    let Context::Effects(projected) = projected else {
        unreachable!()
    };
    let scoped = projected.configuration.unwrap();
    assert_eq!(
        scoped
            .iter()
            .map(|record| (&record.resource, record.data.as_str()))
            .collect::<BTreeMap<_, _>>(),
        BTreeMap::from([(&own, "own"), (&hit, "hit"), (&active, "active")])
    );

    let request = Request {
        api_version: api::API_VERSION,
        context: Context::Effects(context.clone()),
        config: vec![],
        state: vec![],
    };
    let mut plan = Plan::unchanged(&request);
    plan.effects = Some(api::EffectPlan::unchanged(&context));
    let bytes = vera_mod_runtime::test_support::component_fixture(
        &api::ModuleManifest {
            api_version: api::API_VERSION,
            id: "scoped-effect".into(),
            version: "1".into(),
            state_version: 1,
            config_version: 1,
            hooks: vec![
                api::HookKind::Configure,
                api::HookKind::Attach,
                api::HookKind::Damage,
                api::HookKind::Tick,
                api::HookKind::EffectResource,
                api::HookKind::Effects,
                api::HookKind::EntityConfiguration,
            ],
        },
        &serde_json::to_vec(&plan).unwrap(),
        "(if (i32.gt_u (local.get 3) (i32.const 4096)) (then unreachable))",
        "",
    );
    let digest = crate::util::sha256::sha256_hex(&bytes);
    let program = AdmittedComponent::load(bytes, &digest)
        .unwrap()
        .into_configuration_seed();
    assert!(
        program
            .entity_config
            .set(Ok(Arc::new(EntityConfiguration {
                bytes: vec![b'x'; 50_000],
                attach_types: None,
                effect_visit_exceptions: None,
                effect_configuration: Some(records),
            })))
            .is_ok()
    );
    assert_eq!(program.evaluate(request.context, &[]).unwrap(), plan);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn independent_resource_shards_retain_overflow_fallback_and_the_single_query_tail() {
    let query = Context::EffectResource(api::EffectResourceContext {
        kind: api::EffectResourceKind::Warhead,
        name: "WH".into(),
    });
    let mut plan = Plan::unchanged(&api::resource_batch::query_request(query.clone()));
    let descriptor = api::EffectDescriptor {
        firepower_multiplier_bits: 0.5f64.to_bits(),
        armor_multiplier_bits: 1.0f64.to_bits(),
        rof_multiplier_bits: 1.0f64.to_bits(),
        speed_multiplier_bits: 1.0f64.to_bits(),
        cloakable: false,
        animation: None,
    };
    plan.effect_resource = Some(descriptor.clone());
    let manifest = api::ModuleManifest {
        api_version: api::API_VERSION,
        id: "resource-overflow".into(),
        version: "1".into(),
        state_version: 1,
        config_version: 1,
        hooks: vec![
            api::HookKind::Configure,
            api::HookKind::Attach,
            api::HookKind::Damage,
            api::HookKind::Tick,
            api::HookKind::EffectResource,
            api::HookKind::Effects,
            api::HookKind::ResourceBatch,
        ],
    };
    let module = vera_mod_runtime::test_support::component_fixture_by_kind(
        &manifest,
        &serde_json::to_vec(&plan).unwrap(),
        &[(api::HookKind::ResourceBatch.as_str(), b"null")],
    );
    let digest = crate::util::sha256::sha256_hex(&module);
    let program = AdmittedComponent::load(module, &digest)
        .unwrap()
        .into_configuration_seed();
    let workers = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let count = api::resource_batch::MAX_QUERIES + 1;
    let queries = vec![query; count];

    let policies = workers
        .install(|| program.collect_resources(&queries, |plan| plan.effect_resource))
        .unwrap();
    assert_eq!(policies, vec![Some(descriptor); count]);
}
