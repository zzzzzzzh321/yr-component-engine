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
}
