//! Read-only local catalog using the same MIX, CSF, INI and preview owners as gameplay.
//! The closed launcher communicates by JSON over stdio, never links this GPL module.
use crate::{
    assets::{
        asset_manager::{AssetManager, MediaArchiveMode},
        csf_file::CsfFile,
        mix_archive::MixArchive,
    },
    rules::ini_parser::IniFile,
    util::sha256::sha256_hex,
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_MAP: usize = 8 * 1024 * 1024;
const MAX_MAPS: usize = 1500;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    operation: String,
    retail_dir: PathBuf,
    #[serde(default)]
    map_dirs: Vec<PathBuf>,
    locator: Option<Locator>,
    expected_hash: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Locator {
    File { path: PathBuf },
    Archive { name: String },
}

fn allowed_file(path: &Path, roots: &[PathBuf]) -> Result<PathBuf> {
    let path = fs::canonicalize(path)?;
    ensure!(
        path.is_file() && roots.iter().any(|r| path.starts_with(r)),
        "map outside configured library"
    );
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    ensure!(
        ["map", "mpr", "yrm", "mmx", "yro"].contains(&ext.as_str()),
        "unsupported map file"
    );
    Ok(path)
}
fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    ensure!(
        file.metadata()?.len() <= MAX_MAP as u64,
        "map file too large"
    );
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_MAP as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= MAX_MAP, "map file grew beyond limit");
    Ok(bytes)
}
fn unwrap_map(bytes: Vec<u8>) -> Result<Vec<u8>> {
    if !bytes.starts_with(&[0, 0]) {
        return Ok(bytes);
    }
    let archive = MixArchive::from_bytes(bytes)?;
    ensure!(
        archive.entries().len() <= 64,
        "map wrapper has too many entries"
    );
    let mut found = None;
    for entry in archive.entries() {
        if let Some(data) = archive.get_by_id(entry.id) {
            ensure!(data.len() <= MAX_MAP, "wrapped map too large");
            if IniFile::from_bytes(data).ok().is_some_and(|ini| {
                ini.section("Map").is_some() && ini.section("IsoMapPack5").is_some()
            }) {
                ensure!(found.is_none(), "ambiguous map wrapper");
                found = Some(data.to_vec());
            }
        }
    }
    found.context("map wrapper contains no playable map")
}
fn read_source(locator: &Locator, roots: &[PathBuf], assets: &AssetManager) -> Result<Vec<u8>> {
    let bytes = match locator {
        Locator::File { path } => read_bounded(&allowed_file(path, roots)?)?,
        Locator::Archive { name } => {
            ensure!(
                !name.contains(['/', '\\', ':']) && !name.starts_with('.') && name.len() <= 128,
                "invalid archived map name"
            );
            let bytes = assets
                .get_ref(name)
                .context("archived map no longer exists")?;
            ensure!(bytes.len() <= MAX_MAP, "archived map too large");
            bytes.to_vec()
        }
    };
    unwrap_map(bytes)
}
fn walk(root: &Path, depth: u8, files: &mut Vec<PathBuf>) {
    if depth > 5 || files.len() >= MAX_MAPS {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut paths: Vec<_> = entries.filter_map(Result::ok).collect();
    paths.sort_by_key(|e| e.file_name());
    for e in paths {
        let Ok(kind) = e.file_type() else { continue };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            walk(&e.path(), depth + 1, files)
        } else if kind.is_file() {
            let p = e.path();
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ["map", "mpr", "yrm", "mmx", "yro"].contains(&ext.as_str()) && files.len() < MAX_MAPS
            {
                files.push(p)
            }
        }
    }
}
pub fn inspect_map_bytes(
    bytes: &[u8],
    filename: &str,
    title: Option<&str>,
    source: &str,
) -> Result<Value> {
    let ini = IniFile::from_bytes(bytes)?;
    let map = ini.section("Map").context("missing [Map]")?;
    ensure!(ini.section("IsoMapPack5").is_some(), "missing terrain pack");
    let size = map.read_rect("Size", [0; 4]);
    ensure!(
        size[2] > 0 && size[3] > 0 && size[2] + size[3] <= 512,
        "invalid map dimensions"
    );
    let local = map.read_rect("LocalSize", [0, 0, size[2], size[3]]);
    ensure!(
        local.iter().all(|v| *v >= 0 && *v <= 1024) && local[2] > 0 && local[3] > 0,
        "invalid visible map rectangle"
    );
    let basic = ini.section_or_empty("Basic");
    let capacity = crate::map::waypoints::skirmish_player_capacity(&ini).clamp(2, 20);
    let start_limit = if basic.read_int("MaxPlayers", 0) > 8 {
        20
    } else {
        8
    };
    let starts: Vec<_> = crate::map::waypoints::parse_waypoints(&ini)
        .into_values()
        .filter(|p| p.index < start_limit && p.rx > 0 && p.ry > 0)
        .map(|p| json!({"index":p.index,"x":p.rx,"y":p.ry}))
        .collect();
    ensure!(starts.len() >= 2, "map has fewer than two starting points");
    let mut starts = starts;
    starts.sort_by_key(|p| p["index"].as_u64().unwrap());
    let hash = sha256_hex(bytes);
    let name = title
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| basic.read_string("Name", filename, 256));
    let bounds = crate::map::scenario_menu::preview_source_bounds_from_verified_source(&ini);
    let positions: Vec<Value> = starts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (x, y) = if let Some(b) = bounds.as_ref().filter(|b| i < b.start_points.len()) {
                let q = b.start_points[i];
                (
                    (q.x - b.origin_x) as f64 / b.width as f64,
                    (q.y - b.origin_y) as f64 / b.height as f64,
                )
            } else {
                let rx = p["x"].as_f64().unwrap();
                let ry = p["y"].as_f64().unwrap();
                (
                    (rx - ry + size[2] as f64 - 2. * local[0] as f64)
                        / (2. * local[2].max(1) as f64),
                    (rx + ry - size[2] as f64 - 2. * local[1] as f64)
                        / (2. * local[3].max(1) as f64),
                )
            };
            json!({"index":p["index"],"x":x.clamp(0.03,0.97),"y":y.clamp(0.03,0.97)})
        })
        .collect();
    let maximum = starts.len().min(capacity as usize);
    let preview = crate::map::preview::parse_preview_section(&ini);
    Ok(
        json!({"origin":"local","name":name,"logicalName":format!("local-{}.map",&hash[..24]),"width":size[2],"height":size[3],"localSize":local,"theater":map.read_string("Theater","TEMPERATE",64).to_uppercase(),"maxPlayers":maximum,"minPlayers":basic.read_int("MinPlayers",2).clamp(2,maximum as i32),"starts":starts,"previewPositions":positions,"previewSize":preview.size.map(|(w,h)|[w,h]),"sourceSha256":hash,"description":basic.read_string("Description","",512),"fileName":filename,"author":basic.read_string("Author","",256),"sourceLabel":source}),
    )
}
fn preview_bmp(ini: &IniFile) -> Option<String> {
    let preview = crate::map::preview::decode_preview_image_from_ini(ini).ok()??;
    let scale = (512. / preview.width.max(preview.height) as f64).min(1.);
    let w = (preview.width as f64 * scale).round().max(1.) as u32;
    let h = (preview.height as f64 * scale).round().max(1.) as u32;
    let stride = (w * 3 + 3) & !3;
    let size = 54 + stride * h;
    let mut bmp = vec![0u8; size as usize];
    bmp[..2].copy_from_slice(b"BM");
    for (offset, value) in [
        (2, size),
        (10, 54),
        (14, 40),
        (18, w),
        (22, h),
        (34, stride * h),
    ] {
        bmp[offset..offset + 4].copy_from_slice(&value.to_le_bytes())
    }
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
    for y in 0..h {
        for x in 0..w {
            let sx = x * preview.width / w;
            let sy = y * preview.height / h;
            let p = ((sy * preview.width + sx) * 4) as usize;
            let q = (54 + (h - 1 - y) * stride + x * 3) as usize;
            bmp[q..q + 3].copy_from_slice(&[
                preview.rgba[p + 2],
                preview.rgba[p + 1],
                preview.rgba[p],
            ])
        }
    }
    Some(format!("data:image/bmp;base64,{}", STANDARD.encode(bmp)))
}
pub fn run(request: Request) -> Result<Value> {
    ensure!(request.map_dirs.len() <= 8, "too many library directories");
    let retail = fs::canonicalize(&request.retail_dir)?;
    let mut roots = vec![retail.clone()];
    for p in &request.map_dirs {
        if let Ok(p) = fs::canonicalize(p) {
            ensure!(p.is_dir(), "map library is not a directory");
            if !roots.contains(&p) {
                roots.push(p)
            }
        }
    }
    let mut assets = AssetManager::new(&retail, MediaArchiveMode::STOCK_DIGITAL)?;
    let csf = assets
        .get_ref("ra2md.csf")
        .and_then(|b| CsfFile::from_bytes(b).ok());
    let records = crate::map::scenario_sources::list_skirmish_scenario_records_with_assets(
        &retail,
        &mut assets,
        csf.as_ref(),
    )?;
    if request.operation == "read" {
        let locator = request.locator.context("missing map source")?;
        let bytes = read_source(&locator, &roots, &assets)?;
        ensure!(
            Some(sha256_hex(&bytes)) == request.expected_hash,
            "map changed since library scan; refresh and select it again"
        );
        // A real selection must pass the gameplay terrain parser, not just its header.
        crate::map::map_file::load_from_bytes(bytes.clone())?;
        let ini = IniFile::from_bytes(&bytes)?;
        return Ok(
            json!({"data":STANDARD.encode(&bytes),"sourceSha256":sha256_hex(&bytes),"preview":preview_bmp(&ini)}),
        );
    }
    ensure!(request.operation == "scan", "unknown map operation");
    let mut maps = Vec::new();
    let mut seen = BTreeSet::new();
    let mut skipped = 0;
    let mut add =
        |locator: Locator, title: Option<&str>, filename: &str, source: &str| match read_source(
            &locator, &roots, &assets,
        )
        .and_then(|b| inspect_map_bytes(&b, filename, title, source))
        {
            Ok(info) => {
                let hash = info["sourceSha256"].as_str().unwrap().to_owned();
                if maps.len() < MAX_MAPS && seen.insert(hash) {
                    maps.push(json!({"info":info,"locator":locator}))
                }
            }
            Err(_) => skipped += 1,
        };
    for r in records {
        add(
            Locator::Archive {
                name: r.file_name.clone(),
            },
            Some(&r.display_name),
            &r.file_name,
            "原版地图库",
        )
    }
    let mut paths = Vec::new();
    // Retail root loose files plus explicit custom directories; do not traverse an entire install tree.
    if let Ok(entries) = fs::read_dir(&retail) {
        for e in entries.filter_map(Result::ok) {
            if e.file_type().is_ok_and(|t| t.is_file()) {
                paths.push(e.path())
            }
        }
    }
    for root in roots.iter().skip(1) {
        walk(root, 0, &mut paths)
    }
    for p in paths {
        if allowed_file(&p, &roots).is_ok() {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            add(Locator::File { path: p }, None, &name, "本地文件")
        }
    }
    maps.sort_by(|a, b| a["info"]["name"].as_str().cmp(&b["info"]["name"].as_str()));
    Ok(json!({"maps":maps,"skipped":skipped,"truncated":maps.len()==MAX_MAPS}))
}
