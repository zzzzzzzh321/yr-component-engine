//! Local, read-only map inspection process. GPL engine code stays in this process.
use std::io::{Read, Write};
fn main() {
    let result = (|| -> anyhow::Result<serde_json::Value> {
        let mut input = Vec::new();
        std::io::stdin().take(65537).read_to_end(&mut input)?;
        anyhow::ensure!(input.len() <= 65536, "map request too large");
        vera20k_core::map::community_library::run(serde_json::from_slice(&input)?)
    })();
    let (response, code) = match result {
        Ok(value) => (value, 0),
        Err(error) => (serde_json::json!({"error":format!("{error:#}")}), 1),
    };
    let _ = std::io::stdout().write_all(serde_json::to_string(&response).unwrap().as_bytes());
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use vera20k_core::{map::community_library::inspect_map_bytes, util::sha256::sha256_hex};
    #[test]
    fn map_identity_and_sparse_starts_come_from_payload() {
        let bytes=b"[Map]\nSize=0,0,80,80\nLocalSize=0,0,80,80\nTheater=SNOW\n[Basic]\nName=Actual\n[IsoMapPack5]\n1=x\n[Waypoints]\n0=10020\n3=30040\n";
        let m = inspect_map_bytes(bytes, "test.yrm", None, "local").unwrap();
        assert_eq!(m["name"], "Actual");
        assert_eq!(m["starts"][1]["index"], 3);
        assert_eq!(m["maxPlayers"], 2);
        assert_eq!(m["sourceSha256"], sha256_hex(bytes));
        assert!(inspect_map_bytes(b"[Map]\nSize=0,0,-1,4", "bad.map", None, "local").is_err());
    }
    #[test]
    fn high_bridge_draw_geometry_matches_original_controls() {
        use vera20k_core::map::draw_geometry::{
            bridge_body_y_offset, compute_bridge_body_shp_frame,
        };
        let packet: serde_json::Value = serde_json::from_str(include_str!(
            "../../tools/spatial_oracle/bridge_body_render.json"
        ))
        .unwrap();
        for row in packet["controls"].as_array().unwrap() {
            let input = &row["input"];
            if input["flags"].as_u64().unwrap() & 0x80 == 0 {
                continue;
            }
            let state = input["state"].as_u64().unwrap() as u8;
            let rx = input["coords"][0].as_u64().unwrap() as u16;
            let ry = input["coords"][1].as_u64().unwrap() as u16;
            let draw = &row["draws"][0];
            assert_eq!(
                u64::from(compute_bridge_body_shp_frame(state, rx, ry)),
                draw["frame"].as_u64().unwrap(),
                "{}",
                row["name"]
            );
            let level = input["level"].as_i64().unwrap();
            let original_offset =
                draw["point"][1].as_i64().unwrap() - input["cell_point"][1].as_i64().unwrap() - 15
                    + level * 15;
            assert_eq!(
                bridge_body_y_offset(state),
                original_offset as f32,
                "{}",
                row["name"]
            );
        }
    }
    #[test]
    fn building_art_anchor_matches_original_render_coordinate() {
        use vera20k_core::{
            map::draw_geometry::building_art_anchor, util::lepton::absolute_leptons_to_screen,
        };
        let packet: serde_json::Value = serde_json::from_str(include_str!(
            "../../tools/spatial_oracle/building_pixel_coordinates.json"
        ))
        .unwrap();
        // Unchanged original Building receiver, no admitted per-art pixel offset:
        // 0x004518E3 reaches its render-coordinate virtual at 0x00459EF0.
        let row = &packet["constructor_rows"][0];
        let project = |v: &serde_json::Value| {
            absolute_leptons_to_screen(
                v[0].as_i64().unwrap() as i32,
                v[1].as_i64().unwrap() as i32,
                v[2].as_i64().unwrap() as i32,
            )
        };
        let location = project(&row["raw_building"]);
        assert_eq!(
            building_art_anchor(location.0, location.1),
            project(&row["world"])
        );
    }
    #[test]
    #[ignore = "requires a local, licensed YR install in RA2_DIR"]
    fn static_preview_decodes_real_assets_at_high_resolution() {
        use base64::Engine;
        let root = std::env::var("RA2_DIR").expect("RA2_DIR");
        let scan = vera20k_core::map::community_library::run(
            serde_json::from_value(serde_json::json!({"operation":"scan","retailDir":root}))
                .unwrap(),
        )
        .unwrap();
        let record = scan["maps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["info"]["fileName"] == "XMP02T2.MAP")
            .expect("Isle of War");
        let result=vera20k_core::map::community_library::run(serde_json::from_value(serde_json::json!({"operation":"render","retailDir":root,"locator":record["locator"],"expectedHash":record["info"]["sourceSha256"]})).unwrap()).unwrap();
        let png = base64::engine::general_purpose::STANDARD
            .decode(
                result["preview"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("data:image/png;base64,")
                    .unwrap(),
            )
            .unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgb8();
        assert_eq!(decoded.width(), 1536);
        assert_eq!(result["previewGeometry"]["height"], decoded.height());
        let distinct: std::collections::HashSet<_> = decoded.pixels().map(|p| p.0).collect();
        assert!(
            distinct.len() > 512,
            "A rendered overview must retain artwork detail"
        );
    }

    #[test]
    #[ignore = "requires the stock Bay of Pigs map in a licensed RA2_DIR"]
    fn bay_of_pigs_preview_includes_oil_pump_attachments() {
        use base64::Engine;
        let root = std::env::var("RA2_DIR").expect("RA2_DIR");
        let result = vera20k_core::map::community_library::run(
            serde_json::from_value(serde_json::json!({
                "operation":"render", "retailDir":root,
                "locator":{"kind":"archive","name":"XBayOPigs.MAP"},
                "expectedHash":"15b761afdc3403cac20f32ad65b60cc158b385900a73f46f78cadf021160dd15"
            }))
            .unwrap(),
        )
        .unwrap();
        let png = base64::engine::general_purpose::STANDARD
            .decode(
                result["preview"]
                    .as_str()
                    .unwrap()
                    .split_once(',')
                    .unwrap()
                    .1,
            )
            .unwrap();
        let image = image::load_from_memory(&png).unwrap().to_rgb8();
        // Two separate oil pumps beside the northern warehouse, sampled in
        // the final 1536px overview. The stock body-only preview contains just
        // 24/20 dark neutral pixels here; the missing CAOILD_A pump frames
        // contribute the tall dark machinery. A broad coverage threshold
        // avoids tying this regression to PNG encoding or exact RGB values.
        for (left, top, right, bottom) in [(906, 263, 922, 285), (887, 274, 903, 294)] {
            let dark_machinery = (top..bottom)
                .flat_map(|y| (left..right).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    let p = image.get_pixel(x, y).0;
                    let spread = p.iter().max().unwrap() - p.iter().min().unwrap();
                    spread < 30 && p.into_iter().map(u16::from).sum::<u16>() < 270
                })
                .count();
            assert!(
                dark_machinery >= 55,
                "Oil-pump attachment missing at {left},{top}: {dark_machinery}"
            );
        }
    }
}
