use super::*;
use vera_mod_api::{Context, EntityContext, HookKind, TickContext};

#[test]
fn registry_snapshot_has_a_bounded_bulk_limit_without_widening_scalar_readers() {
    use bindings::vera::mod_sdk::host::Host;
    struct ReaderOutput(usize);
    impl HostServices for ReaderOutput {
        fn config(&self, _: &str, _: &str) -> Option<String> {
            None
        }
        fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
            Ok(vec![0; self.0])
        }
        fn registry(&self, _: &str) -> Vec<String> {
            vec![]
        }
        fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
            Err("unused test service".into())
        }
    }
    let state = |bytes| HostState {
        services: Arc::new(ReaderOutput(bytes)),
        limits: StoreLimitsBuilder::new().build(),
        calls_left: 1,
    };
    assert!(
        state(65_537)
            .config_read("".into(), "".into(), "name:32".into(), vec![])
            .is_err()
    );
    assert_eq!(
        state(65_537)
            .config_read("".into(), "".into(), "rules-registry-view".into(), vec![])
            .unwrap()
            .unwrap()
            .len(),
        65_537
    );
    assert!(
        state(MAX_MESSAGE_BYTES + 1)
            .config_read("".into(), "".into(), "rules-registry-view".into(), vec![])
            .is_err()
    );
    assert_eq!(
        state(65_537)
            .config_read("".into(), "".into(), "fixed-art-view".into(), vec![])
            .unwrap()
            .unwrap()
            .len(),
        65_537
    );
    assert!(
        state(MAX_MESSAGE_BYTES + 1)
            .config_read("".into(), "".into(), "fixed-art-view".into(), vec![])
            .is_err()
    );
    assert_eq!(
        state(65_537)
            .config_read("".into(), "".into(), "faction-registry-view".into(), vec![])
            .unwrap()
            .unwrap()
            .len(),
        65_537
    );
    assert!(
        state(MAX_MESSAGE_BYTES + 1)
            .config_read("".into(), "".into(), "faction-registry-view".into(), vec![])
            .is_err()
    );
    assert_eq!(
        state(65_537)
            .config_read("".into(), "".into(), "cameo-registry-view".into(), vec![])
            .unwrap()
            .unwrap()
            .len(),
        65_537
    );
    assert!(
        state(MAX_MESSAGE_BYTES + 1)
            .config_read("".into(), "".into(), "cameo-registry-view".into(), vec![])
            .is_err()
    );
}

fn entity() -> EntityContext {
    EntityContext {
        id: 7,
        type_name: "UNIT".into(),
        owner: "Owner".into(),
        armor: "none".into(),
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
    }
}

fn request() -> Request {
    Request {
        api_version: API_VERSION,
        context: Context::Tick(TickContext {
            frame: 3,
            entity: entity(),
        }),
        config: Vec::new(),
        state: Vec::new(),
    }
}

/// A real canonical-ABI component. Its mutable global is intentionally absent
/// from the returned plan, so evaluating it twice detects accidental reuse.
fn fixture(api: u32, body: &str, extra: &str) -> Vec<u8> {
    let manifest = ModuleManifest {
        api_version: api,
        id: "test.component".into(),
        version: "1".into(),
        state_version: 1,
        config_version: 1,
        hooks: vec![HookKind::Tick, HookKind::Configure],
    };
    let plan = serde_json::to_vec(&Plan {
        state: vec![1],
        ..Plan::unchanged(&request())
    })
    .unwrap();
    let counter = 2048
        + String::from_utf8(plan.clone())
            .unwrap()
            .find("[1]")
            .unwrap()
        + 1;
    let body = format!(
        "{body}\n\
         (global.set $counter (i32.add (global.get $counter) (i32.const 1)))\n\
         (i32.store8 (i32.const {counter}) (i32.add (global.get $counter) (i32.const 48)))"
    );
    test_support::component_fixture(&manifest, &plan, &body, extra)
}

fn load(runtime: &ComponentRuntime, bytes: &[u8]) -> Result<LoadedModule, RuntimeError> {
    runtime.load(bytes, &format!("{:x}", Sha256::digest(bytes)))
}

#[test]
fn runtime_checks_content_and_exported_api_identity() {
    let runtime = ComponentRuntime::new(Default::default()).unwrap();
    let good = fixture(API_VERSION, "", "");
    assert!(matches!(
        runtime.load(&good, &"0".repeat(64)),
        Err(RuntimeError::Invalid(_))
    ));
    assert!(load(&runtime, &fixture(API_VERSION + 1, "", "")).is_err());
    assert_eq!(
        load(&runtime, &good).unwrap().manifest().id,
        "test.component"
    );
}

#[test]
fn runtime_restarts_component_memory_and_rejects_undeclared_authority() {
    let runtime = ComponentRuntime::new(Default::default()).unwrap();
    let module = load(&runtime, &fixture(API_VERSION, "", "")).unwrap();
    for _ in 0..2 {
        let plan = runtime
            .evaluate(&module, &request(), Arc::new(ManifestHost))
            .unwrap();
        assert_eq!(
            plan.state,
            [1],
            "unreturned mutable globals must not persist"
        );
    }
    let other = ComponentRuntime::new(Default::default()).unwrap();
    assert!(
        other
            .evaluate(&module, &request(), Arc::new(ManifestHost))
            .is_err()
    );
    let wasi = fixture(
        API_VERSION,
        "",
        r#"(import "wasi:random/random@0.2.0" (instance (export "get-random-u64" (func (result u64)))))"#,
    );
    assert!(
        load(&runtime, &wasi).is_err(),
        "no WASI interface is linked"
    );
}

#[test]
fn runtime_bounds_guest_fuel_and_linear_memory_without_poisoning_later_calls() {
    let runtime = ComponentRuntime::new(RuntimeLimits {
        fuel: 10_000,
        memory_bytes: 65_536,
        ..Default::default()
    })
    .unwrap();
    let infinite = load(
        &runtime,
        &fixture(API_VERSION, "(loop $spin (br $spin))", ""),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&infinite, &request(), Arc::new(ManifestHost))
            .is_err()
    );
    let growing = load(
        &runtime,
        &fixture(API_VERSION, "(drop (memory.grow (i32.const 1)))", ""),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&growing, &request(), Arc::new(ManifestHost))
            .is_err()
    );
    let good = load(&runtime, &fixture(API_VERSION, "", "")).unwrap();
    assert_eq!(
        runtime
            .evaluate(&good, &request(), Arc::new(ManifestHost))
            .unwrap()
            .state,
        [1]
    );
}

#[test]
fn runtime_separates_bounded_configuration_work_from_actor_work() {
    let runtime = ComponentRuntime::new(RuntimeLimits {
        fuel: 10_000,
        configure_fuel: 200_000,
        fuel_per_input_byte: 0,
        ..Default::default()
    })
    .unwrap();
    let workload = load(
        &runtime,
        &fixture(
            API_VERSION,
            r#"
        (global.set $counter (i32.const 10000))
        (loop $work
            (global.set $counter (i32.sub (global.get $counter) (i32.const 1)))
            (br_if $work (global.get $counter)))
    "#,
            "",
        ),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&workload, &request(), Arc::new(ManifestHost))
            .is_err()
    );
    let configure = Request {
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: 0,
            source: "rules".into(),
            final_pass: false,
            object_types: Vec::new(),
            warheads: Vec::new(),
        }),
        ..request()
    };
    assert!(
        runtime
            .evaluate(&workload, &configure, Arc::new(ManifestHost))
            .is_ok()
    );
    for kind in ["configure_country_read", "configure_faction_pass"] {
        assert!(
            runtime
                .call(&workload.component, kind, b"{}", Arc::new(ManifestHost))
                .is_ok(),
            "{kind} uses the bounded configuration budget"
        );
    }
    let infinite = load(
        &runtime,
        &fixture(API_VERSION, "(loop $spin (br $spin))", ""),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&infinite, &configure, Arc::new(ManifestHost))
            .is_err()
    );
    assert!(
        ComponentRuntime::new(RuntimeLimits {
            configure_fuel: 0,
            ..Default::default()
        })
        .is_err()
    );
}

#[test]
fn runtime_accounts_for_bounded_wire_input_without_disabling_fuel() {
    let runtime = ComponentRuntime::new(RuntimeLimits {
        fuel: 10_000,
        fuel_per_input_byte: 64,
        max_input_fuel: 200_000,
        ..Default::default()
    })
    .unwrap();
    let workload = load(
        &runtime,
        &fixture(
            API_VERSION,
            r#"
        (global.set $counter (i32.const 10000))
        (loop $work
            (global.set $counter (i32.sub (global.get $counter) (i32.const 1)))
            (br_if $work (global.get $counter)))
    "#,
            "",
        ),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&workload, &request(), Arc::new(ManifestHost))
            .is_err()
    );
    let padded = Request {
        config: vec![255; 1024],
        ..request()
    };
    assert_eq!(runtime.limits.fuel_for("tick", usize::MAX), 210_000);
    assert!(
        runtime
            .evaluate(&workload, &padded, Arc::new(ManifestHost))
            .is_ok()
    );
    let infinite = load(
        &runtime,
        &fixture(API_VERSION, "(loop $spin (br $spin))", ""),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&infinite, &padded, Arc::new(ManifestHost))
            .is_err()
    );
}

#[test]
fn runtime_counts_frozen_configuration_input_within_the_existing_cap() {
    struct MetadataHost;
    impl HostServices for MetadataHost {
        fn configuration_input_bytes(&self) -> usize {
            2048
        }
        fn config(&self, section: &str, key: &str) -> Option<String> {
            ManifestHost.config(section, key)
        }
        fn config_read(
            &self,
            section: &str,
            key: &str,
            reader: &str,
            current: &[u8],
        ) -> Result<Vec<u8>, String> {
            ManifestHost.config_read(section, key, reader, current)
        }
        fn registry(&self, section: &str) -> Vec<String> {
            ManifestHost.registry(section)
        }
        fn damage_kernel(&self, input: &DamageKernelInput) -> Result<i32, String> {
            ManifestHost.damage_kernel(input)
        }
    }
    let runtime = ComponentRuntime::new(RuntimeLimits {
        fuel: 10_000,
        configure_fuel: 10_000,
        fuel_per_input_byte: 64,
        max_input_fuel: 200_000,
        ..Default::default()
    })
    .unwrap();
    let workload = load(
        &runtime,
        &fixture(
            API_VERSION,
            r#"
        (global.set $counter (i32.const 10000))
        (loop $work
            (global.set $counter (i32.sub (global.get $counter) (i32.const 1)))
            (br_if $work (global.get $counter)))
    "#,
            "",
        ),
    )
    .unwrap();
    let configure = Request {
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: 0,
            source: "rules".into(),
            final_pass: false,
            object_types: vec![],
            warheads: vec![],
        }),
        ..request()
    };
    assert!(
        runtime
            .evaluate(&workload, &configure, Arc::new(ManifestHost))
            .is_err()
    );
    assert!(
        runtime
            .evaluate(&workload, &configure, Arc::new(MetadataHost))
            .is_ok()
    );
    for kind in ["configure_country_read", "configure_faction_pass"] {
        assert!(
            runtime
                .call(&workload.component, kind, b"{}", Arc::new(ManifestHost))
                .is_err()
        );
        assert!(
            runtime
                .call(&workload.component, kind, b"{}", Arc::new(MetadataHost))
                .is_ok(),
            "{kind} counts frozen metadata within the existing input cap"
        );
    }
    assert!(
        runtime
            .evaluate(&workload, &request(), Arc::new(MetadataHost))
            .is_err(),
        "configuration metadata cannot grant an actor more fuel"
    );
    let infinite = load(
        &runtime,
        &fixture(API_VERSION, "(loop $spin (br $spin))", ""),
    )
    .unwrap();
    assert!(
        runtime
            .evaluate(&infinite, &configure, Arc::new(MetadataHost))
            .is_err()
    );
    assert_eq!(runtime.limits.fuel_for("configure", usize::MAX), 210_000);
    assert_eq!(
        RuntimeLimits {
            configure_fuel: 10_000,
            fuel_per_input_byte: 0,
            ..Default::default()
        }
        .fuel_for("configure", usize::MAX),
        10_000
    );
}

#[test]
#[ignore = "requires the built reference component in VERA_SDK_COMPAT_WASM"]
fn runtime_admits_the_built_reference_component_without_wasi_services() {
    let bytes =
        std::fs::read(std::env::var_os("VERA_SDK_COMPAT_WASM").expect("set VERA_SDK_COMPAT_WASM"))
            .unwrap();
    let runtime = ComponentRuntime::new(Default::default()).unwrap();
    let module = load(&runtime, &bytes).unwrap();
    assert!(
        ["vera.compat-shield", "vera.reference-gameplay"].contains(&module.manifest().id.as_str())
    );
    let plan = runtime
        .evaluate(&module, &request(), Arc::new(ManifestHost))
        .unwrap();
    assert!(plan.state.is_empty());
    assert!(plan.operations.is_empty());
}
