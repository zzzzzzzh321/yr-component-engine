use super::*;

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
