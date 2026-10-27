//! One process owns one immutable launch snapshot, isolated from other tests.
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use vera20k_core::{
    assets::{
        asset_manager::AssetManager,
        community::{self, Snapshot},
    },
    map::source::{self, LoadedMapSource},
};

#[test]
fn snapshot_wins_all_resource_reads_and_map_loading_over_stale_disk() {
    let parent = std::env::current_dir()
        .unwrap()
        .join("../work/engine-memory-tests");
    fs::create_dir_all(&parent).unwrap();
    let directory = parent.join(format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("community-demo.map"),
        b"stale and invalid disk map",
    )
    .unwrap();
    fs::write(directory.join("community-unit.shp"), b"stale disk sprite").unwrap();
    let map = include_bytes!("maps/match.map").to_vec();
    community::install(
        Snapshot::new(
            "fixture@1".into(),
            vec![
                ("community-demo.map".into(), map.clone()),
                ("community-unit.shp".into(), vec![1, 2, 3]),
            ],
        )
        .unwrap(),
    )
    .unwrap();
    let assets = AssetManager::from_ini_probe_root(&directory).unwrap();
    assert_eq!(assets.get("COMMUNITY-UNIT.SHP").unwrap(), [1, 2, 3]);
    assert_eq!(assets.get_ref("community-unit.shp").unwrap(), [1, 2, 3]);
    assert_eq!(
        &*assets.get_shared("community-unit.shp").unwrap(),
        &[1, 2, 3]
    );
    assert_eq!(
        assets.get_archive_ref("community-unit.shp").unwrap(),
        [1, 2, 3]
    );
    assert_eq!(
        assets.read_install_file("community-unit.shp").unwrap(),
        [1, 2, 3]
    );
    assert_eq!(
        assets.get_with_source("community-unit.shp").unwrap().1,
        "community:fixture@1"
    );
    assert_eq!(
        assets
            .load_file_from_mix("community-unit.shp")
            .unwrap()
            .bytes
            .as_ref(),
        &[1, 2, 3]
    );
    assert!(assets.resolve_ref_bounded("community-unit.shp", 2).is_err());
    let loaded =
        source::load_map_by_name_or_path_with_assets(&directory, "community-demo.map", &assets)
            .unwrap();
    assert_eq!(loaded.map.header.theater, "TEMPERATE");
    assert!(
        matches!(loaded.source, LoadedMapSource::Mix { source_archive, payload_len, .. } if source_archive == "community:fixture@1" && payload_len == map.len())
    );
    assert!(
        community::install(
            Snapshot::new("other".into(), vec![("map.map".into(), vec![])]).unwrap()
        )
        .is_err()
    );
    fs::remove_file(directory.join("community-demo.map")).unwrap();
    fs::remove_file(directory.join("community-unit.shp")).unwrap();
    fs::remove_dir(directory).unwrap();
}
