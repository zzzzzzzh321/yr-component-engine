//! Local static map photographs, composed from the engine's TMP/SHP/INI owners.
//! This presentation-only overview does not run a match or claim tactical-frame
//! parity. Building attachments are frozen at their initial normal frame;
//! mobile/voxel objects and live effects are omitted.
use super::{
    draw_geometry::{
        bridge_body_y_offset, building_art_anchor, compute_bridge_body_shp_frame,
        is_high_bridge_body_identity, stored_frame_offset,
    },
    map_file::MapFile,
    terrain::iso_to_screen,
    theater::{self, TileImage, TileKey},
};
use crate::{
    assets::{asset_manager::AssetManager, pal_file::Palette, shp_file::ShpFile},
    rules::{
        art_data::{
            AnimDrawPalette, ArtRegistry, EffectImageKind, anim_draw_palette,
            building_body_shp_candidates, effect_shp_candidates, object_shp_candidates,
        },
        ini_parser::IniFile,
        native_processing::{
            NativeRulesRegistryState, RulesLayerKind, RulesLayerStack, RulesProcessContext,
        },
        object_image::ImageTheater,
        overlay_types::OverlayTypeRegistry,
    },
    util::{lepton::absolute_leptons_to_screen, pixel_conversion::PixelConversionBounds},
};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use image::{ImageEncoder, RgbImage, codecs::png::PngEncoder, imageops::FilterType};
use serde_json::{Value, json};
use std::collections::HashMap;

const LONG_EDGE: u32 = 1536;
struct Canvas {
    pixels: RgbImage,
    depth: Vec<f32>,
    scale: f32,
    origin: [f32; 2],
}
impl Canvas {
    fn draw(&mut self, tile: &TileImage, x: f32, y: f32, row: f32, tint: [f32; 3]) {
        let left = ((x - self.origin[0]) * self.scale).floor() as i32;
        let top = ((y - self.origin[1]) * self.scale).floor() as i32;
        let right = ((x + tile.width as f32 - self.origin[0]) * self.scale).ceil() as i32;
        let bottom = ((y + tile.height as f32 - self.origin[1]) * self.scale).ceil() as i32;
        for py in top.max(0)..bottom.min(self.pixels.height() as i32) {
            let sy = ((py as f32 + 0.5) / self.scale + self.origin[1] - y).floor() as i32;
            if sy < 0 || sy >= tile.height as i32 {
                continue;
            }
            for px in left.max(0)..right.min(self.pixels.width() as i32) {
                let sx = ((px as f32 + 0.5) / self.scale + self.origin[0] - x).floor() as i32;
                if sx < 0 || sx >= tile.width as i32 {
                    continue;
                }
                let src = (sy as u32 * tile.width + sx as u32) as usize;
                if tile.rgba[src * 4 + 3] == 0 {
                    continue;
                }
                // Terrain uses the existing TMP depth plane (TMP_TileBlitter
                // 0x00547CF0; render/terrain_instances.rs owns its game caller).
                // Static SHPs use their cell foot depth, not the tactical SHP Z pipeline.
                let z = -row + tile.depth.get(src).copied().unwrap_or(0) as f32;
                let dst = py as usize * self.pixels.width() as usize + px as usize;
                if z > self.depth[dst] {
                    continue;
                }
                self.depth[dst] = z;
                self.pixels.put_pixel(
                    px as u32,
                    py as u32,
                    image::Rgb(std::array::from_fn(|c| {
                        (tile.rgba[src * 4 + c] as f32 * tint[c]).clamp(0., 255.) as u8
                    })),
                );
            }
        }
    }
}
fn shp<'a>(
    cache: &'a mut HashMap<String, ShpFile>,
    assets: &AssetManager,
    names: &[String],
) -> Option<(&'a ShpFile, String)> {
    let name = names
        .iter()
        .find(|name| cache.contains_key(*name) || assets.get_ref(name).is_some())?
        .clone();
    if !cache.contains_key(&name) {
        let bytes = assets.get_ref(&name)?;
        cache.insert(name.clone(), ShpFile::from_bytes(bytes).ok()?);
    }
    Some((&cache[&name], name))
}
fn draw_shp(
    canvas: &mut Canvas,
    image: &ShpFile,
    frame: usize,
    palette: &Palette,
    center: [f32; 2],
    row: f32,
    tint: [f32; 3],
) {
    let Some(bounds) = image.frames.get(frame) else {
        return;
    };
    let Ok(rgba) = image.frame_to_rgba(frame, palette) else {
        return;
    };
    let tile = TileImage {
        rgba,
        depth: vec![],
        width: bounds.frame_width as u32,
        height: bounds.frame_height as u32,
        offset_x: 0,
        offset_y: 0,
    };
    let (offset_x, offset_y) = stored_frame_offset(
        image.width,
        image.height,
        bounds.frame_x,
        bounds.frame_y,
        0.,
    );
    canvas.draw(
        &tile,
        center[0] + offset_x,
        center[1] + offset_y,
        row + 30.,
        tint,
    );
}

struct BuildingPiece {
    names: Vec<String>,
    frame: usize,
    center: [f32; 2],
    z_adjust: i32,
    palette: PreviewPalette,
    tint: [f32; 3],
}
enum PreviewPalette {
    Unit,
    Iso,
    Anim,
}

/// A static building is one assembly, not just frame zero of its body SHP.
/// Slot allocation451890/451988 uses the unshifted render coordinate and
/// slot X/Y converted by Tactical6D2360. DrawIt422CA0 adds AnimType.Start and
/// YDrawOffset. See sim/building_art.rs and presentation/instances/shp.rs.
/// This overview freezes normal Active/Idle slots; it does not create combat,
/// production, superweapon or upgrade state to make every declared slot live.
fn building_pieces(
    art: &ArtRegistry,
    type_id: &str,
    rules_image: &str,
    extension: &str,
    theater: &str,
    cell: (u16, u16, u8),
    anchor: [f32; 2],
    tint: [f32; 3],
) -> Vec<BuildingPiece> {
    let image_id = art.resolve_effective_image_id(type_id, rules_image);
    let metadata = art.resolve_metadata_entry(type_id, rules_image);
    let mut body_center = anchor;
    if let Some(entry) = metadata {
        body_center[0] += entry.x_draw_offset as f32;
        body_center[1] += entry.y_draw_offset as f32;
    }
    let mut pieces = vec![BuildingPiece {
        names: building_body_shp_candidates(Some(art), rules_image, &image_id, extension, theater),
        frame: 0,
        center: body_center,
        z_adjust: 0,
        palette: if metadata.is_some_and(|entry| entry.terrain_palette) {
            PreviewPalette::Iso
        } else {
            PreviewPalette::Unit
        },
        tint,
    }];
    let Some(entry) = metadata else { return pieces };
    if let Some(bib) = &entry.bib_shape {
        let bib_image = art.resolve_effective_image_id(bib, bib);
        let (x, y) = art.draw_offsets(bib);
        // Building DrawBody43D9C9 draws the bib in the same building group.
        pieces.insert(
            0,
            BuildingPiece {
                names: object_shp_candidates(Some(art), &bib_image, extension, theater),
                frame: 0,
                center: [anchor[0] + x as f32, anchor[1] + y as f32],
                z_adjust: 0,
                palette: if entry.terrain_palette {
                    PreviewPalette::Iso
                } else {
                    PreviewPalette::Unit
                },
                tint,
            },
        );
    }
    for slot in &entry.building_anims {
        if !matches!(slot.native_slot, 3..=6 | 18) || slot.anim_type.is_empty() {
            continue;
        }
        let Some(config) = art.anim_runtime_config(&slot.anim_type) else {
            continue;
        };
        let Ok(frame) = usize::try_from(config.start) else {
            continue;
        };
        let (dx, dy) = PixelConversionBounds::default().offset_to_leptons(slot.x, slot.y);
        let (sx, sy) = absolute_leptons_to_screen(
            i32::from(cell.0) * 256 + dx,
            i32::from(cell.1) * 256 + dy,
            i32::from(cell.2 as i8) * crate::util::lepton::GROUND_LEVEL_HEIGHT_LEPTONS,
        );
        let anim_image = art.resolve_anim_image_id(&slot.anim_type);
        pieces.push(BuildingPiece {
            names: effect_shp_candidates(
                EffectImageKind::Animation,
                Some(art),
                &slot.anim_type,
                &anim_image,
                extension,
                theater,
            ),
            frame,
            center: [sx, sy + config.y_draw_offset as f32],
            z_adjust: slot.z_adjust,
            palette: if config.should_use_cell_drawer
                || anim_draw_palette(config) == AnimDrawPalette::Unit
            {
                PreviewPalette::Unit
            } else {
                PreviewPalette::Anim
            },
            tint: if config.use_normal_light {
                [1.; 3]
            } else {
                tint
            },
        });
    }
    // Static painter order within the group. Positive ZAdjust is behind the
    // body (e.g. CAAIRP_A=5); negative values put flags in front. Keep all
    // pieces at the host's foot depth, without a flag shifting other buildings.
    pieces.sort_by(|a, b| b.z_adjust.cmp(&a.z_adjust));
    pieces
}

pub(super) fn render(assets: &mut AssetManager, map: &MapFile) -> Result<Value> {
    let td = theater::load_theater(assets, &map.header.theater)
        .context("theater graphics unavailable")?;
    let h = &map.header;
    ensure!(
        h.local_width > 0 && h.local_height > 0 && h.local_width <= 1024 && h.local_height <= 1024,
        "invalid preview rectangle"
    );
    let world = [h.local_width as f32 * 60., h.local_height as f32 * 30.];
    let scale = (LONG_EDGE as f32 * 2. / world[0].max(world[1])).min(1.);
    let width = (world[0] * scale).round().max(1.) as u32;
    let height = (world[1] * scale).round().max(1.) as u32;
    ensure!(
        u64::from(width) * u64::from(height) <= 10_000_000,
        "preview allocation too large"
    );
    let mut canvas = Canvas {
        pixels: RgbImage::from_pixel(width, height, image::Rgb([34, 35, 53])),
        depth: vec![f32::INFINITY; (width * height) as usize],
        scale,
        origin: [
            (2. * h.local_left as f32 - h.width as f32) * 30.,
            (h.width as f32 + 2. * h.local_top as f32) * 15. + 30.,
        ],
    };
    // Reuse MapClass materialization: serialized no-tile cells (including
    // positive 0xFFFF) draw the theater's ClearTile rather than a hole.
    let resolved = super::resolved_terrain::ResolvedTerrainGrid::build(
        map,
        Some(&td),
        Some(assets),
        None,
        None,
        false,
        0,
    );
    let cells = super::terrain::build_terrain_grid_from_resolved(&resolved, None).cells;
    let needed = theater::collect_used_tiles(
        &cells
            .iter()
            .map(|c| (i32::from(c.tile_id), c.sub_tile))
            .collect::<Vec<_>>(),
    );
    let images = theater::load_tile_images(assets, &td.lookup, &td.iso_palette, &needed);
    ensure!(!images.is_empty(), "no terrain graphics resolved");
    let lights = super::lighting::build_cell_light_grid_from_heights_and_units_with_detail(
        cells.iter().map(|c| ((c.rx, c.ry), c.z)),
        super::lighting::parse_lighting_profiles(&map.ini)
            .normal
            .into(),
        2,
    );
    let levels: HashMap<_, _> = cells.iter().map(|c| ((c.rx, c.ry), c.z)).collect();
    let center = |rx, ry| {
        let (x, y) = iso_to_screen(rx, ry, *levels.get(&(rx, ry)).unwrap_or(&0));
        [x + 30., y + 15.]
    };
    let mut drawn = 0usize;
    for cell in &cells {
        if let Some(tile) = images.get(&TileKey {
            tile_id: cell.tile_id,
            sub_tile: cell.sub_tile,
            variant: 0,
        }) {
            let (x, y) = iso_to_screen(cell.rx, cell.ry, cell.z);
            canvas.draw(
                tile,
                x + tile.offset_x as f32,
                y + tile.offset_y as f32,
                (cell.rx as f32 + cell.ry as f32) * 15.,
                lights.terrain_tile_tint_at((cell.rx, cell.ry)),
            );
            drawn += 1;
        }
    }
    ensure!(drawn > 0, "terrain preview is empty");
    // The static overview composes objects over the completed terrain layer.
    // TMP pixel depth and SHP cell-foot depth are different conventions; sharing
    // their buffers incorrectly hides ore and clips the tops of buildings.
    canvas.depth.fill(f32::INFINITY);
    let root = IniFile::from_bytes(
        assets
            .get_ref("rulesmd.ini")
            .context("RULESMD.INI unavailable")?,
    )?;
    let art_ini = IniFile::from_bytes(
        assets
            .get_ref("artmd.ini")
            .context("ARTMD.INI unavailable")?,
    )?;
    let mut layers = RulesLayerStack::new(root);
    if let Some(bytes) = assets.get_ref("langrule.ini") {
        layers.push(RulesLayerKind::LangRule, IniFile::from_bytes(bytes)?)
    }
    if let Some(bytes) = assets.get_ref("mpbattlemd.ini") {
        layers.push(RulesLayerKind::GameMode, IniFile::from_bytes(bytes)?)
    }
    layers.push(RulesLayerKind::Scenario, map.ini.clone());
    let audio_definitions = crate::rules::audio_sources::AudioDefinitions::select(assets);
    let mut registry = NativeRulesRegistryState::default();
    registry.bind_fixed_sound_source(std::sync::Arc::clone(audio_definitions.sounds()));
    let processed = layers
        .process_with_fixed_art_and_registry_state_recovering(
            &art_ini,
            registry,
            RulesProcessContext::ScenarioReset {
                theater: ImageTheater::from_map(&map.ini),
                skip_root_color_add_and_mode: false,
            },
            audio_definitions.eva(),
            audio_definitions.sounds(),
        )
        .map_err(|failure| anyhow::anyhow!("preview rules: {}", failure.into_parts().0))?;
    let overlays = OverlayTypeRegistry::from_processed_rules(&processed);
    let art = ArtRegistry::from_ini(&art_ini);
    let anim_palette = assets
        .get_ref("anim.pal")
        .and_then(|bytes| Palette::from_bytes(bytes).ok());
    let mut cache = HashMap::new();
    for overlay in &map.overlays {
        let Some(name) = overlays.name(overlay.overlay_id) else {
            continue;
        };
        let names = overlays.shp_candidates(name, td.extension, &h.theater);
        let Some((image, _)) = shp(&mut cache, assets, &names) else {
            continue;
        };
        let flags = overlays.flags(overlay.overlay_id);
        let mut point = center(overlay.rx, overlay.ry);
        point[1] += flags.map_or(0., |f| f.y_draw_offset());
        let high_bridge = is_high_bridge_body_identity(overlay.overlay_id, name);
        let frame = if high_bridge {
            let raw_flags = resolved
                .cell(overlay.rx, overlay.ry)
                .map_or(0, |c| c.bridge_facts.raw_flags);
            if raw_flags & super::bridge_facts::BRIDGE_FLAG_ANCHOR_SELF == 0 {
                continue;
            }
            point[1] += bridge_body_y_offset(overlay.frame);
            compute_bridge_body_shp_frame(overlay.frame, overlay.rx, overlay.ry)
        } else {
            overlay.frame
        };
        let pal = if flags.is_some_and(|f| f.tiberium) {
            &td.tiberium_palette
        } else {
            &td.iso_palette
        };
        draw_shp(
            &mut canvas,
            image,
            frame as usize,
            pal,
            point,
            (overlay.rx as f32 + overlay.ry as f32) * 15.,
            lights.terrain_tile_tint_at((overlay.rx, overlay.ry)),
        );
    }
    let terrain_types: HashMap<_, _> = processed
        .terrain_type_projections()
        .map(|t| (t.name.clone(), t))
        .collect();
    for object in &map.terrain_objects {
        let Some(typ) = terrain_types.get(&object.name) else {
            continue;
        };
        let names = [typ.shape_filename(td.extension)];
        let Some((image, _)) = shp(&mut cache, assets, &names) else {
            continue;
        };
        draw_shp(
            &mut canvas,
            image,
            0,
            &td.iso_palette,
            center(object.rx, object.ry),
            (object.rx as f32 + object.ry as f32) * 15.,
            lights.terrain_tile_tint_at((object.rx, object.ry)),
        );
    }
    let references: HashMap<_, _> = processed
        .techno_reference_states()
        .filter(|(category, _, _, _)| {
            *category == crate::rules::object_type::ObjectCategory::Building
        })
        .map(|(_, name, _, image)| (name, image))
        .collect();
    for object in &map.entities {
        if object.category != super::entities::EntityCategory::Structure {
            continue;
        }
        let Some(image_state) = references.get(object.type_id.as_str()) else {
            continue;
        };
        let rules_image = image_state.image();
        let point = center(object.cell_x, object.cell_y);
        let (x, y) = building_art_anchor(point[0], point[1]);
        for piece in building_pieces(
            &art,
            &object.type_id,
            rules_image,
            td.extension,
            &h.theater,
            (
                object.cell_x,
                object.cell_y,
                *levels.get(&(object.cell_x, object.cell_y)).unwrap_or(&0),
            ),
            [x, y],
            lights.tint_or_default((object.cell_x, object.cell_y)),
        ) {
            let Some((image, _)) = shp(&mut cache, assets, &piece.names) else {
                continue;
            };
            let palette = match piece.palette {
                PreviewPalette::Unit => &td.unit_palette,
                PreviewPalette::Iso => &td.iso_palette,
                PreviewPalette::Anim => match &anim_palette {
                    Some(palette) => palette,
                    None => continue,
                },
            };
            draw_shp(
                &mut canvas,
                image,
                piece.frame,
                palette,
                piece.center,
                (object.cell_x as f32 + object.cell_y as f32) * 15.,
                piece.tint,
            );
        }
    }
    let ratio = (LONG_EDGE as f32 / width.max(height) as f32).min(1.);
    let output = image::imageops::resize(
        &canvas.pixels,
        (width as f32 * ratio).round().max(1.) as u32,
        (height as f32 * ratio).round().max(1.) as u32,
        FilterType::Lanczos3,
    );
    let mut png = Vec::new();
    PngEncoder::new(&mut png).write_image(
        output.as_raw(),
        output.width(),
        output.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    let positions:Vec<_>=map.waypoints.values().filter(|p|p.index<20).map(|p|{
        let point=center(p.rx,p.ry);
        json!({"index":p.index,"x":(point[0]-canvas.origin[0])/world[0],"y":(point[1]-canvas.origin[1])/world[1]})
    }).collect();
    Ok(
        json!({"preview":format!("data:image/png;base64,{}",STANDARD.encode(png)),"previewGeometry":{"width":output.width(),"height":output.height(),"positions":positions,"source":"rendered"}}),
    )
}
