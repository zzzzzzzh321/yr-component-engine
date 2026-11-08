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
            .evaluate(&first.executor.module, &request, Arc::new(StartupHost),)
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
