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
    if value["operation"] == "mo" {
        return probe_mo(
            &assets,
            &archive,
            value["mapName"].as_str().context("map name")?,
        );
    }
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
        use std::io::{Read, Seek, SeekFrom};
        let mut reader = community::snapshot()
            .unwrap()
            .open("large.unknown")?
            .context("streaming resource")?;
        reader.seek(SeekFrom::End(-37))?;
        let mut tail = [0; 37];
        reader.read_exact(&mut tail)?;
        ensure!(
            tail.as_slice()
                == archive.read_range(
                    "large.unknown",
                    archive.entry("large.unknown").unwrap().bytes - 37,
                    37
                )?,
            "stream seek mismatch"
        );
        let bytes = assets.get_ref("声音/test.wav").context("wave")?;
        let wave = vera20k_core::assets::wav_file::WavFile::parse(bytes).context("wave parse")?;
        ensure!(
            wave.channels == 1 && wave.sample_rate == 22050,
            "wave metadata mismatch"
        );
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

fn probe_mo(
    assets: &AssetManager,
    archive: &std::sync::Arc<vera_package::Archive>,
    map_name: &str,
) -> anyhow::Result<()> {
    use vera20k_core::{
        assets::wav_file::{WavReadCursor, WavStreamHeader},
        rules::ini_parser::IniFile,
    };
    let roots = [
        "rulesmd.ini",
        "artmd.ini",
        "aimd.ini",
        "soundmd.ini",
        "evamd.ini",
        "thememd.ini",
    ];
    for name in roots {
        let resolved = assets
            .resolve_ref_bounded(name, 8 * 1024 * 1024)?
            .context("MO root missing")?;
        ensure!(
            resolved.source_archive.starts_with("community:"),
            "retail root won over MO"
        );
    }
    let theme = IniFile::from_bytes(assets.get_ref("thememd.ini").context("theme")?)?;
    let before = archive.decoded_bytes();
    let mut headers = 0;
    let mut selected = None;
    let themes = theme.section_or_empty("Themes");
    for key in themes.keys() {
        let Some(name) = themes.read_name(key, 256) else {
            continue;
        };
        let sound = theme.section_or_empty(name).read_string("Sound", name, 256);
        let filename = format!("{sound}.wav");
        let Some(mut reader) = assets.open_resource(&filename) else {
            continue;
        };
        let Some(header) = WavStreamHeader::read(&mut *reader) else {
            continue;
        };
        headers += 1;
        if selected.is_none() && header.format.sample_rate > 0 && header.format.channels > 0 {
            selected = Some((filename, header));
        }
    }
    ensure!(headers > 20, "MO music catalog absent");
    let catalog_decoded = archive.decoded_bytes() - before;
    ensure!(
        catalog_decoded < 400 * 1024 * 1024,
        "music catalog eagerly decoded archive bodies"
    );
    ensure!(
        assets.package_resident_bytes() < 256 * 1024 * 1024,
        "MIX payloads retained eagerly"
    );
    let (name, header) = selected.context("playable music")?;
    let rate = header.format.sample_rate;
    let channels = header.format.channels;
    let mut cursor = WavReadCursor::new(header).context("music decoder")?;
    let mut reader = assets.open_resource(&name).context("music stream")?;
    let started = std::time::Instant::now();
    let mut max_read = std::time::Duration::ZERO;
    let mut samples = 0usize;
    let mut part = Vec::new();
    while samples < (rate as usize) * (channels as usize) * 90 {
        part.clear();
        let tick = std::time::Instant::now();
        let decoded = cursor.read_next(&mut *reader, &mut part)?;
        max_read = max_read.max(tick.elapsed());
        if !decoded {
            break;
        }
        samples += part.len();
    }
    let seconds = samples as f64 / (rate as f64 * channels as f64);
    ensure!(seconds >= 30., "music stream ended too early");
    ensure!(
        started.elapsed().as_secs_f64() < seconds,
        "music decoding slower than playback"
    );
    let map = vera20k_core::map::source::load_map_by_name_or_path_with_assets(
        &std::path::PathBuf::new(),
        map_name,
        assets,
    )?;
    println!(
        "{}",
        serde_json::json!({"files":archive.metadata().resource_count,"musicHeaders":headers,"catalogDecodedBytes":catalog_decoded,"musicSample":name,"decodedAudioSeconds":seconds,"decodeWallSeconds":started.elapsed().as_secs_f64(),"maxReadMilliseconds":max_read.as_secs_f64()*1000.,"cachedBlockBytes":archive.cached_bytes(),"retainedMixEntryBytes":assets.package_resident_bytes(),"totalDecodedBytes":archive.decoded_bytes(),"blocksDecoded":archive.blocks_decoded(),"mapTheater":map.map.header.theater})
    );
    Ok(())
}
