//! Pure module admission at the original resource-selection frontier. Assets
//! retain only format policies, immutable bytes and a stateless CSF supplier;
//! gameplay configuration still belongs exclusively to Program.
use super::assets::{read_declaration, read_program_bytes};
use super::program::AdmittedComponent;
use crate::assets::asset_manager::{AssetManager, MediaArchiveMode};
use crate::assets::error::AssetError;
use crate::assets::startup_resources as native;
use std::{path::Path, sync::Arc};
use vera_mod_api as api;

fn asset_error(detail: String) -> AssetError {
    AssetError::ParseError {
        format: "startup component".into(),
        detail,
    }
}

fn admit_name(name: &str) -> Result<(), String> {
    if crate::assets::community::valid_package_name(name) {
        Ok(())
    } else {
        Err(format!("invalid logical startup resource name: {name}"))
    }
}

fn audio_policy(policy: api::StartupAudioCatalog) -> Result<native::AudioCatalogPolicy, String> {
    Ok(match policy {
        api::StartupAudioCatalog::Retail => native::AudioCatalogPolicy::Retail,
        api::StartupAudioCatalog::OrderedPairs { pairs } => {
            let pairs = pairs
                .into_iter()
                .map(|pair| {
                    admit_name(&pair.index)?;
                    admit_name(&pair.bag)?;
                    Ok(native::AudioPair {
                        index: pair.index,
                        bag: pair.bag,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            native::AudioCatalogPolicy::OrderedPairs { pairs }
        }
    })
}

fn string_policy(
    policy: api::StartupCsfComposition,
) -> Result<native::CsfCompositionPolicy, String> {
    Ok(match policy {
        api::StartupCsfComposition::BaseOnly => native::CsfCompositionPolicy::BaseOnly,
        api::StartupCsfComposition::OrderedRecords { candidates } => {
            let candidates = candidates
                .into_iter()
                .map(|candidate| {
                    admit_name(&candidate.name)?;
                    Ok(native::CsfCandidate {
                        name: candidate.name,
                        language: match candidate.language {
                            api::CsfLanguageGate::Ignore => native::CsfLanguageGate::Ignore,
                            api::CsfLanguageGate::MatchBaseOrNeutral => {
                                native::CsfLanguageGate::MatchBaseOrNeutral
                            }
                        },
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            native::CsfCompositionPolicy::OrderedRecords { candidates }
        }
    })
}

fn early_resources(assets: &AssetManager) -> Result<native::StartupResources, String> {
    let Some(identity) = read_declaration(assets)? else {
        return Ok(native::StartupResources {
            component: native::StartupComponentSource::Absent,
            ..Default::default()
        });
    };
    let Some(bytes) = read_program_bytes(assets, &identity)? else {
        // A declaration can precede a module in a later registered MIX. Its
        // capability is unknown here; preserve the old late gameplay path.
        return Ok(native::StartupResources {
            component: native::StartupComponentSource::Deferred { identity },
            ..Default::default()
        });
    };
    let admitted = Arc::new(AdmittedComponent::load(bytes.clone(), &identity.sha256)?);
    let mut resources = native::StartupResources {
        component: native::StartupComponentSource::Admitted {
            identity,
            bytes: Arc::from(bytes),
        },
        ..Default::default()
    };
    if !admitted.has_startup_resources() {
        return Ok(resources);
    }
    resources.hash_include =
        match admitted.startup_resource_policy(api::StartupResourceContext::SourceComposition)? {
            None => None,
            Some(api::StartupResourcePolicy::SourceComposition { include }) => {
                Some(include == api::IncludeSemantics::HashIncludeOnce)
            }
            _ => return Err("wrong source-composition recipe".into()),
        };
    resources.audio = match admitted
        .startup_resource_policy(api::StartupResourceContext::AudioCatalog)?
    {
        None => None,
        Some(api::StartupResourcePolicy::AudioCatalog { catalog }) => Some(audio_policy(catalog)?),
        _ => return Err("wrong audio-catalog recipe".into()),
    };
    resources.strings = Some(Arc::new(move |base_language| {
        match admitted.startup_resource_policy(api::StartupResourceContext::StringSupplements {
            base_language,
        })? {
            None => Ok(None),
            Some(api::StartupResourcePolicy::StringSupplements { composition }) => {
                string_policy(composition).map(Some)
            }
            _ => Err("wrong string-supplement recipe".into()),
        }
    }));
    Ok(resources)
}

/// Active app/headless construction shares exactly the native audio prefix.
/// Explicit raw archive/INI probes continue using AssetManager's raw constructors.
pub fn open_active_assets(
    path: &Path,
    media: MediaArchiveMode,
) -> Result<AssetManager, AssetError> {
    let assets = AssetManager::new_with_startup_admission(path, media, |assets| {
        early_resources(assets).map_err(asset_error)
    })?;
    validate_visible_late_component(&assets).map_err(asset_error)?;
    Ok(assets)
}

/// Retain the original process recipes and admitted bytes after a lost lease.
pub fn reopen_active_assets(
    path: &Path,
    media: MediaArchiveMode,
    frozen: &native::StartupResources,
) -> Result<AssetManager, AssetError> {
    let assets = AssetManager::new_with_startup_admission(path, media, |_| Ok(frozen.clone()))?;
    validate_visible_late_component(&assets).map_err(asset_error)?;
    Ok(assets)
}

fn validate_visible_late_component(assets: &AssetManager) -> Result<(), String> {
    // Reject a known late startup module before CSF/splash consumption when it
    // is already visible. Still-missing legacy payloads retain their old binder.
    if let Some((identity, bytes, early)) = resolve_program_bytes(assets, true)?
        && !early
    {
        let admitted = AdmittedComponent::load(bytes, &identity.sha256)?;
        require_startup_frontier(&admitted, false)?;
    }
    Ok(())
}

pub(super) fn require_startup_frontier(
    admitted: &AdmittedComponent,
    early: bool,
) -> Result<(), String> {
    if !early && admitted.has_startup_resources() {
        Err("startup component was not available before resource selection".into())
    } else {
        Ok(())
    }
}

/// Source ownership, not the final VFS winner, fixes an early admitted module.
/// Deferred/absent modules may still resolve at the original gameplay binder;
/// their manifest must then prove that no startup frontier was required.
pub(super) fn resolve_program_bytes(
    assets: &AssetManager,
    allow_missing_payload: bool,
) -> Result<Option<(native::StartupComponentIdentity, Vec<u8>, bool)>, String> {
    use native::StartupComponentSource;
    let current = read_declaration(assets)?;
    let expected = match &assets.startup_resources().component {
        StartupComponentSource::Deferred { identity }
        | StartupComponentSource::Admitted { identity, .. } => Some(identity),
        StartupComponentSource::Unprobed | StartupComponentSource::Absent => None,
    };
    if expected.is_some_and(|identity| current.as_ref() != Some(identity)) {
        return Err("component declaration changed after startup admission".into());
    }
    if let StartupComponentSource::Admitted { identity, bytes } =
        &assets.startup_resources().component
    {
        return Ok(Some((identity.clone(), bytes.to_vec(), true)));
    }
    let Some(identity) = current else {
        return Ok(None);
    };
    let Some(bytes) = read_program_bytes(assets, &identity)? else {
        return if allow_missing_payload {
            Ok(None)
        } else {
            Err(format!("missing gameplay component {}", identity.program))
        };
    };
    Ok(Some((identity, bytes, false)))
}
