//! Canonical-ABI protocol fixture shared with engine integration tests. This
//! returns canned bytes; gameplay behavior belongs to real plugin components.
use vera_mod_api::ModuleManifest;

/// Use raw response bytes so consumers can also exercise malformed output.
/// `body` and `extra` retain the original runtime fuel/import fixture seams.
pub fn component_fixture(
    manifest: &ModuleManifest,
    response: &[u8],
    body: &str,
    extra: &str,
) -> Vec<u8> {
    build_component(manifest, response, &[], body, extra)
}

/// Exact protocol-kind responses, with one common response for lifecycle calls.
/// This tests capability negotiation and rejection without a feature parser.
pub fn component_fixture_by_kind(
    manifest: &ModuleManifest,
    default: &[u8],
    responses: &[(&str, &[u8])],
) -> Vec<u8> {
    build_component(manifest, default, responses, "", "")
}

fn build_component(
    manifest: &ModuleManifest,
    response: &[u8],
    responses: &[(&str, &[u8])],
    body: &str,
    extra: &str,
) -> Vec<u8> {
    let escaped =
        |bytes: &[u8]| -> String { bytes.iter().map(|byte| format!("\\{byte:02x}")).collect() };
    let manifest = serde_json::to_vec(manifest).unwrap();
    assert!(manifest.len() <= 2048 - 256);
    let mut offset = 2048 + response.len();
    let mut data = String::new();
    let mut routes = String::new();
    for (kind, bytes) in responses {
        let condition = kind.bytes().enumerate().fold(
            format!("(i32.eq (local.get 1) (i32.const {}))", kind.len()),
            |condition, (index, byte)| format!(
                "(i32.and {condition} (i32.eq (i32.load8_u offset={index} (local.get 0)) (i32.const {byte})))"
            ),
        );
        data += &format!("(data (i32.const {offset}) \"{}\")", escaped(bytes));
        routes += &format!(
            "(if {condition} (then (i32.store (i32.const 4) (i32.const {offset})) (i32.store (i32.const 8) (i32.const {}))))",
            bytes.len()
        );
        offset += bytes.len();
    }
    assert!(offset <= 8192);
    let text = format!(
        r#"(component
        {extra}
        (core module $m
            (memory (export "memory") 1)
            (global $heap (mut i32) (i32.const 8192))
            (global $counter (mut i32) (i32.const 0))
            (data (i32.const 256) "{}")
            (data (i32.const 2048) "{}")
            {data}
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
                        (i32.store (i32.const 4) (i32.const 2048))
                        (i32.store (i32.const 8) (i32.const {}))
                        {routes}))
                (i32.const 0)))
        (core instance $i (instantiate $m))
        (func (export "evaluate")
            (param "kind" string) (param "input" (list u8))
            (result (result (list u8) (error string)))
            (canon lift (core func $i "evaluate")
                (memory (core memory $i "memory")) (realloc (core func $i "realloc")))))"#,
        escaped(&manifest),
        escaped(response),
        manifest.len(),
        response.len()
    );
    wat::parse_str(text).unwrap()
}
