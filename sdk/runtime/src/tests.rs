use super::*;
use vera_mod_api::{Context, EntityContext, HookKind, TickContext};

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

fn escaped(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("\\{byte:02x}")).collect()
}

/// A real canonical-ABI component. Its mutable global is intentionally absent
/// from the returned plan, so evaluating it twice detects accidental reuse.
fn fixture(api: u32, body: &str, extra: &str) -> Vec<u8> {
    let manifest = serde_json::to_vec(&ModuleManifest {
        api_version: api,
        id: "test.component".into(),
        version: "1".into(),
        state_version: 1,
        config_version: 1,
        hooks: vec![HookKind::Tick, HookKind::Configure],
    })
    .unwrap();
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
    let text = format!(
        r#"(component
        {extra}
        (core module $m
            (memory (export "memory") 1)
            (global $heap (mut i32) (i32.const 8192))
            (global $counter (mut i32) (i32.const 0))
            (data (i32.const 256) "{}")
            (data (i32.const 2048) "{}")
            (func (export "realloc") (param i32 i32 i32 i32) (result i32)
                (local $old i32)
                global.get $heap local.tee $old local.get 3 i32.add global.set $heap local.get $old)
            (func (export "evaluate") (param i32 i32 i32 i32) (result i32)
                (i32.store (i32.const 0) (i32.const 0))
                (if (i32.eq (local.get 1) (i32.const 8))
                    (then
                        (i32.store (i32.const 4) (i32.const 256))
                        (i32.store (i32.const 8) (i32.const {})))
                    (else
                        {body}
                        (global.set $counter (i32.add (global.get $counter) (i32.const 1)))
                        (i32.store8 (i32.const {counter}) (i32.add (global.get $counter) (i32.const 48)))
                        (i32.store (i32.const 4) (i32.const 2048))
                        (i32.store (i32.const 8) (i32.const {}))))
                (i32.const 0)))
        (core instance $i (instantiate $m))
        (func (export "evaluate")
            (param "kind" string) (param "input" (list u8))
            (result (result (list u8) (error string)))
            (canon lift (core func $i "evaluate")
                (memory (core memory $i "memory")) (realloc (core func $i "realloc")))))"#,
        escaped(&manifest),
        escaped(&plan),
        manifest.len(),
        plan.len()
    );
    wat::parse_str(text).unwrap()
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
