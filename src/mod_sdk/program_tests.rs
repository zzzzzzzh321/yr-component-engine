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
    assert!(AdmittedComponent::load(bytes, &"0".repeat(64)).is_err());
}
