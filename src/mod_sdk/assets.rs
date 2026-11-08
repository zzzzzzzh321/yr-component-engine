//! Work asset admission and exact saved-program rebinding. Configuration is
//! never replayed against current assets when restoring a saved match.
use super::Program;
use crate::rules::ini_parser::IniFile;
use std::sync::Arc;
use vera_mod_api as api;

use super::program::AdmittedComponent;
use crate::assets::startup_resources::StartupComponentIdentity;

/// One declaration parser for both the early frontier and the original binder.
pub(super) fn read_declaration(
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<Option<StartupComponentIdentity>, String> {
    let Some(bytes) = assets.get_ref("vera-mod.json") else {
        return Ok(None);
    };
    if bytes.len() > 16 * 1024 {
        return Err("mod declaration too large".into());
    }
    let identity: StartupComponentIdentity =
        serde_json::from_slice(&bytes).map_err(|e| format!("mod declaration: {e}"))?;
    if identity.api_version != api::API_VERSION
        || !crate::assets::community::valid_package_name(&identity.program)
    {
        return Err("invalid mod declaration".into());
    }
    Ok(Some(identity))
}

pub(super) fn read_program_bytes(
    assets: &crate::assets::asset_manager::AssetManager,
    identity: &StartupComponentIdentity,
) -> Result<Option<Vec<u8>>, String> {
    let Some(bytes) = assets.get_ref(&identity.program) else {
        return Ok(None);
    };
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("gameplay component too large".into());
    }
    if crate::util::sha256::sha256_hex(&bytes) != identity.sha256 {
        return Err("component content digest mismatch".into());
    }
    Ok(Some(bytes.to_vec()))
}

fn asset_program(
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<Option<AdmittedComponent>, String> {
    let Some((identity, bytes, early)) = super::startup::resolve_program_bytes(assets, false)?
    else {
        return Ok(None);
    };
    let admitted = AdmittedComponent::load(bytes, &identity.sha256)?;
    super::startup::require_startup_frontier(&admitted, early)?;
    Ok(Some(admitted))
}

pub(crate) fn load_from_assets(
    assets: &crate::assets::asset_manager::AssetManager,
    passes: &[IniFile],
    registries: &[crate::rules::native_processing::SdkRegistryPass],
    object_types: &[String],
    warheads: &[String],
    weapons: &[String],
) -> Result<Option<Arc<Program>>, String> {
    load_from_assets_for_process(
        assets,
        passes,
        registries,
        object_types,
        warheads,
        weapons,
        None,
        &[],
    )
}

pub(crate) fn load_from_assets_for_process(
    assets: &crate::assets::asset_manager::AssetManager,
    passes: &[IniFile],
    registries: &[crate::rules::native_processing::SdkRegistryPass],
    object_types: &[String],
    warheads: &[String],
    weapons: &[String],
    seed: Option<&Program>,
    steps: &[crate::rules::native_processing::SdkConfigurationStep],
) -> Result<Option<Arc<Program>>, String> {
    prepare_process_mod_binding(assets, seed, steps)?.configure(
        passes,
        registries,
        object_types,
        warheads,
        weapons,
    )
}

/// Frozen inputs for one isolated configuration candidate, without any live
/// Rules registry, asset-manager borrow or Simulation authority.
pub(crate) struct PreparedProcessModBinding {
    source: Option<(StartupComponentIdentity, Vec<u8>, bool)>,
    seed: Option<Program>,
    steps: Vec<crate::rules::native_processing::SdkConfigurationStep>,
}

pub(crate) fn prepare_process_mod_binding(
    assets: &crate::assets::asset_manager::AssetManager,
    seed: Option<&Program>,
    steps: &[crate::rules::native_processing::SdkConfigurationStep],
) -> Result<PreparedProcessModBinding, String> {
    Ok(PreparedProcessModBinding {
        source: super::startup::resolve_program_bytes(assets, false)?,
        seed: seed.cloned(),
        steps: steps.to_vec(),
    })
}

impl PreparedProcessModBinding {
    pub(crate) fn configure(
        self,
        passes: &[IniFile],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
    ) -> Result<Option<Arc<Program>>, String> {
        let Some((identity, bytes, early)) = self.source else {
            return Ok(None);
        };
        let admitted = AdmittedComponent::load(bytes, &identity.sha256)?;
        super::startup::require_startup_frontier(&admitted, early)?;
        if admitted.has_loading_theme_resource() && self.seed.is_none() {
            return Err(
                "loading theme component was not admitted before cold Rules processing".into(),
            );
        }
        admitted
            .configure(
                passes,
                object_types,
                warheads,
                weapons,
                registries,
                self.seed.as_ref(),
                &self.steps,
            )
            .map(|p| Some(Arc::new(p)))
    }
}

/// The Rules registry retains this same Program before physical Rules readers
/// and Country bodies. Missing legacy payloads keep the original late binder.
pub(crate) fn early_configuration_seed(
    assets: &crate::assets::asset_manager::AssetManager,
) -> Result<Option<Arc<Program>>, String> {
    let Some((identity, bytes, early)) = super::startup::resolve_program_bytes(assets, true)?
    else {
        return Ok(None);
    };
    let admitted = AdmittedComponent::load(bytes, &identity.sha256)?;
    super::startup::require_startup_frontier(&admitted, early)?;
    let program = admitted.into_configuration_seed();
    Ok(program.requires_rules_seed().then(|| Arc::new(program)))
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
        (Some(saved), Some(admitted)) if saved.sha256() == admitted.digest() => {
            saved.validate_restore()?;
            rules.external_mod = Some(Arc::new(saved.clone()));
        }
        _ => return Err("saved/current work disagree about external gameplay identity".into()),
    }
    rules.bind_resource_policies()
}
