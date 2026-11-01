//! Command-line adapter for the offline SDK component packager.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: vera-mod-pack INPUT_CORE_WASM OUTPUT_NEW_DIR")?,
    );
    let output = PathBuf::from(args.next().context("missing OUTPUT_NEW_DIR")?);
    ensure!(
        args.next().is_none(),
        "usage: vera-mod-pack INPUT_CORE_WASM OUTPUT_NEW_DIR"
    );
    let sha256 = vera_mod_pack::package(&input, &output)?;
    println!(
        "{}",
        serde_json::json!({"directory": output, "program": "gameplay.wasm", "sha256": sha256})
    );
    Ok(())
}
