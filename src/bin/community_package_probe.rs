//! Local integration diagnostic. Stdin carries launch secrets; stdout contains
//! resource hashes and metadata only. Never extracts published resources.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{io::Read, path::PathBuf};
use vera20k_core::assets::{
    asset_manager::{AssetManager, MediaArchiveMode},
    community::{self, Snapshot},
};
fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().take(32768).read_to_string(&mut input)?;
    let value: serde_json::Value = serde_json::from_str(&input)?;
    let retail = PathBuf::from(value["retailDir"].as_str().context("retail directory")?);
    if value["operation"] == "prepare" {
        let assets = AssetManager::new(&retail, MediaArchiveMode::STOCK_DIGITAL)?;
        let output = PathBuf::from(value["output"].as_str().context("fixture directory")?);
        std::fs::create_dir_all(&output)?;
        for name in ["rulesmd.ini", "artmd.ini", "aimd.ini"] {
            std::fs::write(
                output.join(name),
                assets.get(name).context("missing base INI")?,
            )?;
        }
        println!("{{\"prepared\":true}}");
        return Ok(());
    }
    let archive = community::open_package(
        &PathBuf::from(value["path"].as_str().context("package path")?),
        &STANDARD
            .decode(value["publicKey"].as_str().context("public key")?)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("public key length"))?,
        STANDARD
            .decode(value["key"].as_str().context("key")?)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("key length"))?,
        value["sha256"].as_str().context("digest")?,
    )?;
    let source = format!(
        "{}@{}",
        archive.metadata().id,
        archive.metadata().release_id
    );
    community::install(Snapshot::from_package(source, archive.clone(), vec![])?)?;
    let assets = AssetManager::new(&retail, MediaArchiveMode::STOCK_DIGITAL)?;
    let mut checks = vec![];
    for item in value["checks"].as_array().context("checks")? {
        let name = item["name"].as_str().context("resource name")?;
        let data = assets
            .resolve_ref_bounded(name, 64 * 1024 * 1024)?
            .context("resource absent")?;
        let bytes = data.bytes.len();
        ensure!(
            bytes as u64 == item["bytes"].as_u64().unwrap(),
            "resource size mismatch"
        );
        checks.push(serde_json::json!({"name":name,"bytes":bytes,"source":data.source_archive,"prefix":data.bytes.iter().take(8).copied().collect::<Vec<_>>()}));
    }
    let name = value["mapName"].as_str().context("map name")?;
    {
        use std::io::{Read,Seek,SeekFrom};
        let mut reader=community::snapshot().unwrap().open("large.unknown")?.context("streaming resource")?;
        reader.seek(SeekFrom::End(-37))?;let mut tail=[0;37];reader.read_exact(&mut tail)?;
        ensure!(tail.as_slice()==archive.read_range("large.unknown",archive.entry("large.unknown").unwrap().bytes-37,37)?,"stream seek mismatch");
        let bytes=assets.get_ref("声音/test.wav").context("wave")?;
        let wave=vera20k_core::assets::wav_file::WavFile::parse(bytes).context("wave parse")?;
        ensure!(wave.channels==1&&wave.sample_rate==22050,"wave metadata mismatch");
    }
    if let Some(name) = value["nestedMix"].as_str() {
        let outer = vera20k_core::assets::mix_archive::MixArchive::from_bytes(
            assets.get(name).context("outer mix")?,
        )?;
        let nested = outer
            .nested_archive_by_id(vera20k_core::assets::mix_hash::mix_hash(
                "fixture-nested.mix",
            ))?
            .context("nested mix")?;
        ensure!(
            nested.get_by_name("nested-marker.dat") == Some(b"nested payload".as_slice()),
            "nested payload mismatch"
        );
    }
    let map =
        vera20k_core::map::source::load_map_by_name_or_path_with_assets(&retail, name, &assets)?;
    println!(
        "{}",
        serde_json::json!({"resources":checks,"theater":map.map.header.theater,"cacheBytes":archive.cached_bytes()})
    );
    Ok(())
}
