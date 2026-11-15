use super::*;

fn read<T: serde::Serialize, O: serde::de::DeserializeOwned>(
    registry: &SdkRegistryPass,
    pass: &IniFile,
    read: u32,
    owner: &str,
    reader: &str,
    key: &str,
    current: T,
) -> O {
    let input = serde_json::to_vec(&SuperReadValue { read, current }).unwrap();
    let bytes = registry
        .read_warhead_impact_value(reader, owner, pass, key, &input)
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn absent_nuke_attempt_precedes_later_anim_allocation_and_keeps_body_scopes_separate() {
    let pass = IniFile::from_str(
        "[Warheads]\n0=NUKE\n1=HOLDER\n[HOLDER]\nAnimList=NUKEBALL\n\
         [SpecialWeapons]\nNukeWarhead=LATE\n",
    );
    let processed = RulesLayerStack::new(pass.clone()).process().unwrap();
    let registry = &processed.sdk_registries()[0];
    let view = registry.warhead_impact_view();
    assert_eq!(view.warheads, ["NUKE", "HOLDER", "LATE"]);
    assert_eq!(processed.warhead_impact_type_names(), view.warheads);
    assert_eq!(
        view.reads[0],
        WarheadImpactReadPoint {
            ordinal: 0,
            body_read: false
        }
    );
    let initial: Option<u32> = read(
        registry,
        &pass,
        0,
        "NUKE",
        "warhead-impact-nukeball-default",
        "",
        (),
    );
    assert_eq!(
        initial, None,
        "later WH AnimList is outside the first prefix"
    );
    let later = view
        .reads
        .iter()
        .enumerate()
        .find(|(read, point)| *read > 0 && point.ordinal == 0)
        .unwrap()
        .0 as u32;
    let after: Option<u32> = read(
        registry,
        &pass,
        later,
        "NUKE",
        "warhead-impact-nukeball-default",
        "",
        (),
    );
    assert_eq!(after, Some(0));
    assert_eq!(
        registry.names("$warheads"),
        ["HOLDER"],
        "old EMP/AE scope stays present-only"
    );
    assert!(
        registry
            .names("$armor-warheads")
            .contains(&"NUKE".to_owned())
    );
    let input = serde_json::to_vec(&SuperReadValue {
        read: 0,
        current: Some(30),
    })
    .unwrap();
    assert!(
        registry
            .read_warhead_impact_value(
                "warhead-impact-duration",
                "NUKE",
                &pass,
                "NukeFlash.Duration",
                &input,
            )
            .is_err()
    );
}

#[test]
fn preimpact_uses_native_slots_physical_anim_prefix_and_registered_sentinel_priority() {
    let pass = IniFile::from_str(
        "[Warheads]\n0=ABCDEFGHIJKLMNOPQRSTUVWXY\n1=MIDDLE\n2=ABCDEFGHIJKLMNOPQRSTUVWXY\n\
         [ABCDEFGHIJKLMNOPQRSTUVWX]\nPreImpactAnim=CHOSEN\nNukeFlash.Duration=$-7\n\
         [MIDDLE]\nAnimList=CHOSEN\nPreImpactAnim=CHOSEN\n",
    );
    let processed = RulesLayerStack::new(pass.clone()).process().unwrap();
    let registry = &processed.sdk_registries()[0];
    let view = registry.warhead_impact_view();
    assert_eq!(
        view.warheads,
        [
            "ABCDEFGHIJKLMNOPQRSTUVWX",
            "MIDDLE",
            "ABCDEFGHIJKLMNOPQRSTUVWX"
        ]
    );
    assert_eq!(
        view.reads
            .iter()
            .map(|point| point.ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let owner = &view.warheads[0];
    let early: Option<u32> = read(
        registry,
        &pass,
        0,
        owner,
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        None::<u32>,
    );
    let later: Option<u32> = read(
        registry,
        &pass,
        2,
        owner,
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        None::<u32>,
    );
    assert_eq!(early, None);
    assert_eq!(later, Some(0));
    let own_list: Option<u32> = read(
        registry,
        &pass,
        1,
        "MIDDLE",
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        None::<u32>,
    );
    assert_eq!(
        own_list,
        Some(0),
        "original AnimList allocation precedes this suffix"
    );
    let duration: Option<i32> = read(
        registry,
        &pass,
        0,
        owner,
        "warhead-impact-duration",
        "NukeFlash.Duration",
        Some(30),
    );
    assert_eq!(
        duration,
        Some(-7),
        "shared Ares integer syntax, without clamping"
    );

    let mut processor = RulesPassProcessor::default();
    processor.find_or_construct(RulesTypeFamily::Animation, "none");
    processor.find_or_construct(RulesTypeFamily::Warhead, "WH");
    let sentinel = IniFile::from_str("[WH]\nPreImpactAnim=NoNe\nNukeFlash.Duration=invalid\n");
    processor.process_warhead_family(&sentinel);
    let registry = processor.sdk_registry_pass(true);
    let animation: Option<u32> = read(
        &registry,
        &sentinel,
        0,
        "WH",
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        None::<u32>,
    );
    let duration: Option<i32> = read(
        &registry,
        &sentinel,
        0,
        "WH",
        "warhead-impact-duration",
        "NukeFlash.Duration",
        Some(-7),
    );
    assert_eq!(
        animation,
        Some(0),
        "registered sentinel wins before none clearing"
    );
    assert_eq!(duration, Some(-7));
    let unknown = IniFile::from_str("[WH]\nPreImpactAnim=UNKNOWN\n");
    processor.process_warhead_family(&unknown);
    let retained: Option<u32> = read(
        &processor.sdk_registry_pass(true),
        &unknown,
        1,
        "WH",
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        animation,
    );
    assert_eq!(
        retained, animation,
        "unknown Find-only input retains without allocation"
    );
    assert_eq!(processor.family_len(RulesTypeFamily::Animation), 1);
    let clear = IniFile::from_str("[WH]\nPreImpactAnim=<none>\n");
    processor.process_warhead_family(&clear);
    let cleared: Option<u32> = read(
        &processor.sdk_registry_pass(true),
        &clear,
        2,
        "WH",
        "warhead-impact-pre-impact",
        "PreImpactAnim",
        retained,
    );
    assert_eq!(
        cleared, None,
        "an unregistered sentinel clears after Find fails"
    );
}
