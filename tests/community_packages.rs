//! Optional archives stay independently readable and cannot override content.
use std::{fs,path::PathBuf,io::Read};
use base64::{Engine,engine::general_purpose::STANDARD};
use vera20k_core::assets::community::{self,Snapshot};
fn archives()->Vec<std::sync::Arc<vera_package::Archive>>{
    let root=PathBuf::from(std::env::var_os("VERA_LAYER_FIXTURE").expect("private fixture directory"));
    let value:serde_json::Value=serde_json::from_slice(&fs::read(root.join("fixture.json")).unwrap()).unwrap();
    value["archives"].as_array().unwrap().iter().map(|v|community::open_package(
        &root.join(v["file"].as_str().unwrap()),
        &STANDARD.decode(value["publicKey"].as_str().unwrap()).unwrap().try_into().unwrap(),
        STANDARD.decode(v["key"].as_str().unwrap()).unwrap().try_into().unwrap(),
        v["sha256"].as_str().unwrap(),
    ).unwrap()).collect()
}
#[test]
fn main_overlay_and_optional_archive_keep_their_own_bounded_readers(){
    let mut archives=archives();archives.pop();
    let mut snapshot=Snapshot::from_packages("main@1".into(),archives,vec![("selected.map".into(),b"chosen map".to_vec())]).unwrap();
    assert_eq!(snapshot.get("main.bin").unwrap().as_ref(),b"main");
    assert_eq!(snapshot.get("selected.map").unwrap().as_ref(),b"chosen map");
    let mut reader=snapshot.file_view("music.bin").unwrap().unwrap().reader();let mut text=String::new();reader.read_to_string(&mut text).unwrap();assert_eq!(text,"optional music");
    assert!(snapshot.open("missing").unwrap().is_none());assert!(snapshot.music_enabled());snapshot.set_music_enabled(false);assert!(!snapshot.music_enabled());
    assert_eq!(snapshot.get("music.bin").unwrap().as_ref(),b"optional music");
}
#[test]
fn optional_resources_cannot_replace_the_main_or_selected_map(){
    assert!(Snapshot::from_packages("main@1".into(),archives(),vec![]).is_err());
    let mut archives=archives();archives.pop();
    assert!(Snapshot::from_packages("main@1".into(),archives,vec![("music.bin".into(),b"replace".to_vec())]).is_err());
}
