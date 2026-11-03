//! Canonical pure binding of frozen faction declarations to a read-only catalog.
//! Catalog implementations borrow an existing authority; this module never reads
//! INI, allocates registry identities, advances configuration, or caches results.
use crate::mod_sdk::Program;
use crate::rules::ares_country_paradrop::AresCountryParadrop;
use crate::rules::ares_country_presentation::AresCountryPresentation;
use crate::rules::ares_side_extension::AresSideExtension;
use vera_mod_api::{
    FactionCountryLoading, FactionCountryPolicy, FactionNullableRef, FactionResourceContext,
    FactionResourcePolicy, FactionSidePolicy, ProductionObjectKind, ProductionTypeRef,
};

/// Category-local ordinals and their exact stored IDs, including constructor-only
/// entries and duplicate truncated IDs. An adapter must not resolve by name or
/// consult another receipt's larger/final catalog to repair a missing identity.
pub(crate) trait FactionCatalog {
    fn side_id(&self, ordinal: u32) -> Option<&str>;
    fn country_id(&self, ordinal: u32) -> Option<&str>;
    fn type_id(&self, reference: ProductionTypeRef) -> Option<&str>;
    fn animation_id(&self, ordinal: u32) -> Option<&str>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FrozenCountry {
    pub veteran_buildings: Vec<u32>,
    pub paradrop: AresCountryParadrop,
    pub loading: FactionCountryLoading,
    pub presentation: AresCountryPresentation,
}

/// None is the legacy/profile path; Some(None) explicitly selects stock.
/// Only declared policies are resolved, using this same catalog throughout.
pub(crate) fn query_side(
    program: &Program,
    catalog: &impl FactionCatalog,
    ordinal: u32,
) -> Result<Option<Option<AresSideExtension>>, String> {
    let stored_id = catalog
        .side_id(ordinal)
        .ok_or_else(|| format!("Side ordinal is outside its registry: {ordinal}"))?;
    let context = FactionResourceContext::Side {
        ordinal,
        stored_id: stored_id.to_owned(),
    };
    let policy = program.faction_resource_policy(&context)?;
    if let Some(policy) = &policy {
        policy.validate_for(&context)?;
    }
    match policy {
        None => Ok(None),
        Some(FactionResourcePolicy::Side { policy }) => {
            Ok(Some(policy.map(|p| bind_side(catalog, p)).transpose()?))
        }
        _ => Err("Side query returned a Country policy".into()),
    }
}

pub(crate) fn query_country(
    program: &Program,
    catalog: &impl FactionCatalog,
    ordinal: u32,
) -> Result<Option<Option<FrozenCountry>>, String> {
    let stored_id = catalog
        .country_id(ordinal)
        .ok_or_else(|| format!("Country ordinal is outside its registry: {ordinal}"))?;
    let context = FactionResourceContext::Country {
        ordinal,
        stored_id: stored_id.to_owned(),
    };
    let policy = program.faction_resource_policy(&context)?;
    if let Some(policy) = &policy {
        policy.validate_for(&context)?;
    }
    match policy {
        None => Ok(None),
        Some(FactionResourcePolicy::Country { policy }) => {
            Ok(Some(policy.map(|p| bind_country(catalog, p)).transpose()?))
        }
        _ => Err("Country query returned a Side policy".into()),
    }
}

// Match the ProductionTypeRef category-local resolver used by production and
// conversion, including its signed native-index bound.
fn faction_type_name(
    catalog: &impl FactionCatalog,
    reference: ProductionTypeRef,
) -> Result<String, String> {
    catalog
        .type_id(reference)
        .filter(|_| reference.ordinal <= i32::MAX as u32)
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "faction reference is outside its {:?} registry: {}",
                reference.category, reference.ordinal
            )
        })
}

fn faction_animation_name(catalog: &impl FactionCatalog, ordinal: u32) -> Result<String, String> {
    catalog
        .animation_id(ordinal)
        .map(str::to_owned)
        .ok_or_else(|| format!("faction animation is outside its registry: {ordinal}"))
}

fn faction_nullable_type(
    catalog: &impl FactionCatalog,
    reference: FactionNullableRef<ProductionTypeRef>,
    category: ProductionObjectKind,
) -> Result<Option<Option<String>>, String> {
    match reference {
        FactionNullableRef::Unset => Ok(None),
        FactionNullableRef::Null => Ok(Some(None)),
        FactionNullableRef::Resolved(value) if value.category == category => {
            faction_type_name(catalog, value).map(|name| Some(Some(name)))
        }
        _ => Err("faction reference has the wrong type family".into()),
    }
}

fn faction_drop_types(
    catalog: &impl FactionCatalog,
    types: Vec<ProductionTypeRef>,
) -> Result<Vec<String>, String> {
    types
        .into_iter()
        .map(|reference| {
            if !matches!(
                reference.category,
                ProductionObjectKind::Infantry | ProductionObjectKind::Unit
            ) {
                return Err("faction paradrop reference must be Infantry or Unit".into());
            }
            faction_type_name(catalog, reference)
        })
        .collect()
}

pub(crate) fn bind_side(
    catalog: &impl FactionCatalog,
    p: FactionSidePolicy,
) -> Result<AresSideExtension, String> {
    use ProductionObjectKind::{Aircraft, Building, Infantry, Unit};
    let text = p.presentation;
    let parachute_anim = match p.parachute_anim {
        FactionNullableRef::Unset => None,
        FactionNullableRef::Null => Some(None),
        FactionNullableRef::Resolved(index) => Some(Some(faction_animation_name(catalog, index)?)),
    };
    Ok(AresSideExtension {
        default_disguise: faction_nullable_type(catalog, p.default_disguise, Infantry)?,
        crew: faction_nullable_type(catalog, p.crew, Infantry)?,
        engineer: faction_nullable_type(catalog, p.engineer, Infantry)?,
        technician: faction_nullable_type(catalog, p.technician, Infantry)?,
        survivor_divisor: p.survivor_divisor,
        base_defenses: p
            .base_defenses
            .map(|indices| {
                indices
                    .into_iter()
                    .map(|ordinal| {
                        faction_type_name(
                            catalog,
                            ProductionTypeRef {
                                category: Building,
                                ordinal,
                            },
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?,
        base_defense_counts: p.base_defense_counts,
        paradrop_types: p
            .paradrop_types
            .map(|types| faction_drop_types(catalog, types))
            .transpose()?,
        paradrop_num: p.paradrop_num,
        paradrop_aircraft: faction_nullable_type(catalog, p.paradrop_aircraft, Aircraft)?,
        parachute_anim,
        tooltip_color: p.tooltip_color,
        message_text_color_index: p.message_text_color_index,
        sidebar_mix_file_index: p.sidebar_mix_file_index,
        sidebar_yuri_file_names: p.sidebar_yuri_file_names,
        eva_voice_index: p.eva_voice_index,
        hunter_seeker: faction_nullable_type(catalog, p.hunter_seeker, Unit)?,
        campaign_score_background: text.campaign_score_background,
        campaign_score_transition: text.campaign_score_transition,
        campaign_score_animation: text.campaign_score_animation,
        campaign_score_palette: text.campaign_score_palette,
        multiplayer_score_background: text.multiplayer_score_background,
        multiplayer_score_bars: text.multiplayer_score_bars,
        multiplayer_score_palette: text.multiplayer_score_palette,
        campaign_score_under_par_theme: text.campaign_score_under_par_theme,
        campaign_score_over_par_theme: text.campaign_score_over_par_theme,
        multiplayer_score_win_theme: text.multiplayer_score_win_theme,
        multiplayer_score_lose_theme: text.multiplayer_score_lose_theme,
        graphical_text_image: text.graphical_text_image,
        graphical_text_palette: text.graphical_text_palette,
        dialog_background_image: text.dialog_background_image,
        dialog_background_palette: text.dialog_background_palette,
    })
}

pub(crate) fn bind_country(
    catalog: &impl FactionCatalog,
    p: FactionCountryPolicy,
) -> Result<FrozenCountry, String> {
    for &ordinal in &p.veteran_buildings {
        faction_type_name(
            catalog,
            ProductionTypeRef {
                category: ProductionObjectKind::Building,
                ordinal,
            },
        )?;
    }
    if p.paradrop
        .aircraft
        .as_ref()
        .is_some_and(|r| r.category != ProductionObjectKind::Aircraft)
    {
        return Err("Country paradrop plane has the wrong type family".into());
    }
    let text = p.presentation;
    Ok(FrozenCountry {
        veteran_buildings: p.veteran_buildings,
        paradrop: AresCountryParadrop {
            parachute_anim: p
                .paradrop
                .parachute_anim
                .map(|i| faction_animation_name(catalog, i))
                .transpose()?,
            aircraft: p
                .paradrop
                .aircraft
                .map(|r| faction_type_name(catalog, r))
                .transpose()?,
            types: faction_drop_types(catalog, p.paradrop.types)?,
            counts: p.paradrop.counts,
        },
        loading: p.loading,
        presentation: AresCountryPresentation {
            flag: text.flag,
            status: text.status,
            taunt: text.taunt,
            observer_background: text.observer_background,
            observer_flag: text.observer_flag,
            observer_flag_alt_palette: text.observer_flag_alt_palette,
            list_index: text.list_index,
            random_weight: text.random_weight,
            ai_random_weight: text.ai_random_weight,
        },
    })
}
