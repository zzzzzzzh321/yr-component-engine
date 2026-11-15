use super::*;
use crate::rules::native_processing::RulesLayerStack;

#[test]
fn warhead_impact_host_scopes_and_accounts_for_its_cached_view() {
    let pass = IniFile::from_str("[Warheads]\n0=NUKE\n");
    let processed = RulesLayerStack::new(pass.clone()).process().unwrap();
    let make = || {
        NativeHost::configuration(pass.clone(), vec![])
            .with_registries(processed.sdk_registries()[0].clone())
    };
    assert!(
        make()
            .config_read("", "", "warhead-impact-registry-view", b"null")
            .is_err()
    );
    let base_bytes = make().configuration_input_bytes();
    let host = make().enable_superweapon_inputs();
    let bytes = host
        .config_read("", "", "warhead-impact-registry-view", b"null")
        .unwrap();
    let view: api::WarheadImpactRegistryView = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        view.reads,
        [api::WarheadImpactReadPoint {
            ordinal: 0,
            body_read: false
        }]
    );
    assert_eq!(
        host.configuration_input_bytes(),
        base_bytes
            + host.superweapon_view_bytes().unwrap().len()
            + host.superweapon_source_bytes()
            + bytes.len(),
    );
    let input = serde_json::to_vec(&api::SuperReadValue {
        read: 0,
        current: (),
    })
    .unwrap();
    assert_eq!(
        host.config_read("NUKE", "", "warhead-impact-nukeball-default", &input)
            .unwrap(),
        b"null"
    );
    assert!(
        host.config_read("OTHER", "", "warhead-impact-nukeball-default", &input)
            .is_err()
    );
    assert!(
        host.config_read(
            "NUKE",
            "Arbitrary",
            "warhead-impact-nukeball-default",
            &input
        )
        .is_err()
    );
    assert!(
        NativeHost::simulation()
            .enable_superweapon_inputs()
            .config_read("NUKE", "", "warhead-impact-nukeball-default", &input)
            .is_err()
    );
}
