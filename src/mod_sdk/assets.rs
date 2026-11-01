//! Work asset admission and exact saved-program rebinding. Configuration is
//! never replayed against current assets when restoring a saved match.
use super::Program;
use crate::rules::ini_parser::IniFile;
use serde::Deserialize;
use std::sync::Arc;
use vera_mod_api as api;

/// Resource-owned declaration; paths remain logical asset names.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    api_version: u32,
    program: String,
    sha256: String,
}

fn asset_program(
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<Option<(String, Vec<u8>)>, String> {
    let Some(declaration) = assets.get_ref("vera-mod.json") else {
        return Ok(None);
    };
    if declaration.len() > 16 * 1024 {
        return Err("mod declaration too large".into());
    }
    let declaration: Declaration =
        serde_json::from_slice(&declaration).map_err(|e| format!("mod declaration: {e}"))?;
    if declaration.api_version != api::API_VERSION
        || !crate::assets::community::valid_package_name(&declaration.program)
    {
        return Err("invalid mod declaration".into());
    }
    let bytes = assets
        .get_ref(&declaration.program)
        .ok_or_else(|| format!("missing gameplay component {}", declaration.program))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("gameplay component too large".into());
    }
    if crate::util::sha256::sha256_hex(&bytes) != declaration.sha256 {
        return Err("component content digest mismatch".into());
    }
    Ok(Some((declaration.sha256, bytes.to_vec())))
}

pub(crate) fn load_from_assets(
    assets: &crate::assets::asset_manager::AssetManager,
    passes: &[IniFile],
    object_types: &[String],
    warheads: &[String],
    weapons: &[String],
) -> Result<Option<Arc<Program>>, String> {
    let Some((digest, bytes)) = asset_program(assets)? else {
        return Ok(None);
    };
    Program::load_with_types(bytes, &digest, passes, object_types, warheads, weapons)
        .map(|p| Some(Arc::new(p)))
}

/// Restore the exact saved configuration after projecting retained Rules.
/// The current work must contain the same portable component; Configure is
/// never replayed against a save and updates require a fresh match.
pub fn adopt_saved_program(
    rules: &mut crate::rules::ruleset::RuleSet,
    sim: &crate::sim::world::Simulation,
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<(), String> {
    match (&sim.external_mod, asset_program(assets)?) {
        (None, None) => {
            rules.external_mod = None;
        }
        (Some(saved), Some((digest, _))) if saved.sha256() == digest => {
            saved.validate_restore()?;
            rules.external_mod = Some(Arc::new(saved.clone()));
        }
        _ => return Err("saved/current work disagree about external gameplay identity".into()),
    }
    rules.bind_weapon_resource_policies()
}
