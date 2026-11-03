//! Immutable name projections of the accepted component's faction resources.
//! This module never reads INI, advances guest phases, or changes registries.
use super::RuleSet;
use crate::rules::ares_country_paradrop::AresCountryParadrop;
use crate::rules::ares_country_presentation::AresCountryPresentation;
use crate::rules::ares_side_extension::AresSideExtension;
use vera_mod_api::{
    FactionCountryLoading, FactionCountryPolicy, FactionNullableRef, FactionResourceContext,
    FactionResourcePolicy, FactionSidePolicy, ProductionObjectKind, ProductionTypeRef,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(super) struct FrozenFactionBindings {
    // An absent entry keeps the old component/profile path. Some(None) is an
    // explicit stock choice, which suppresses an installed native extension.
    pub sides: Vec<Option<Option<AresSideExtension>>>,
    pub countries: Vec<Option<Option<FrozenCountry>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct FrozenCountry {
    pub veteran_buildings: Vec<u32>,
    pub paradrop: AresCountryParadrop,
    pub loading: FactionCountryLoading,
    pub presentation: AresCountryPresentation,
}

impl FrozenFactionBindings {
    pub fn has_overrides(&self) -> bool {
        self.sides.iter().any(Option::is_some) || self.countries.iter().any(Option::is_some)
    }
}

impl RuleSet {
    pub(super) fn collect_faction_resource_bindings(
        &self,
    ) -> Result<FrozenFactionBindings, String> {
        let Some(program) = self
            .external_mod
            .as_deref()
            .filter(|p| p.has_faction_resource())
        else {
            return Ok(FrozenFactionBindings::default());
        };
        let mut result = FrozenFactionBindings::default();
        for (ordinal, stored_id) in self.side_ids.iter().enumerate() {
            let context = FactionResourceContext::Side {
                ordinal: ordinal
                    .try_into()
                    .map_err(|_| "Side ordinal exceeds SDK identity")?,
                stored_id: stored_id.clone(),
            };
            let policy = program.faction_resource_policy(&context)?;
            if let Some(policy) = &policy {
                policy.validate_for(&context)?;
            }
            result.sides.push(match policy {
                None => None,
                Some(FactionResourcePolicy::Side { policy }) => {
                    Some(policy.map(|p| self.bind_faction_side(p)).transpose()?)
                }
                _ => return Err("Side query returned a Country policy".into()),
            });
        }
        for (ordinal, stored_id) in self.country_ids.iter().enumerate() {
            let context = FactionResourceContext::Country {
                ordinal: ordinal
                    .try_into()
                    .map_err(|_| "Country ordinal exceeds SDK identity")?,
                stored_id: stored_id.clone(),
            };
            let policy = program.faction_resource_policy(&context)?;
            if let Some(policy) = &policy {
                policy.validate_for(&context)?;
            }
            result.countries.push(match policy {
                None => None,
                Some(FactionResourcePolicy::Country { policy }) => {
                    Some(policy.map(|p| self.bind_faction_country(p)).transpose()?)
                }
                _ => return Err("Country query returned a Side policy".into()),
            });
        }
        Ok(result)
    }

    // Same category-local ProductionTypeRef resolver used by production and
    // conversion. A constructor-only registered entry is a valid identity.
    fn faction_type_name(&self, reference: ProductionTypeRef) -> Result<String, String> {
        self.type_array_ids(crate::rules::prerequisite::production_category(
            reference.category,
        ))
        .get(reference.ordinal as usize)
        .filter(|_| reference.ordinal <= i32::MAX as u32)
        .cloned()
        .ok_or_else(|| {
            format!(
                "faction reference is outside its {:?} registry: {}",
                reference.category, reference.ordinal
            )
        })
    }

    fn faction_nullable_type(
        &self,
        reference: FactionNullableRef<ProductionTypeRef>,
        category: ProductionObjectKind,
    ) -> Result<Option<Option<String>>, String> {
        match reference {
            FactionNullableRef::Unset => Ok(None),
            FactionNullableRef::Null => Ok(Some(None)),
            FactionNullableRef::Resolved(value) if value.category == category => {
                self.faction_type_name(value).map(|name| Some(Some(name)))
            }
            _ => Err("faction reference has the wrong type family".into()),
        }
    }

    fn faction_animation_name(&self, ordinal: u32) -> Result<String, String> {
        self.sdk_animation_names
            .get(ordinal as usize)
            .cloned()
            .ok_or_else(|| format!("faction animation is outside its registry: {ordinal}"))
    }

    fn faction_drop_types(&self, types: Vec<ProductionTypeRef>) -> Result<Vec<String>, String> {
        types
            .into_iter()
            .map(|reference| {
                if !matches!(
                    reference.category,
                    ProductionObjectKind::Infantry | ProductionObjectKind::Unit
                ) {
                    return Err("faction paradrop reference must be Infantry or Unit".into());
                }
                self.faction_type_name(reference)
            })
            .collect()
    }

    fn bind_faction_side(&self, p: FactionSidePolicy) -> Result<AresSideExtension, String> {
        use ProductionObjectKind::{Aircraft, Building, Infantry, Unit};
        let text = p.presentation;
        let parachute_anim = match p.parachute_anim {
            FactionNullableRef::Unset => None,
            FactionNullableRef::Null => Some(None),
            FactionNullableRef::Resolved(index) => Some(Some(self.faction_animation_name(index)?)),
        };
        Ok(AresSideExtension {
            default_disguise: self.faction_nullable_type(p.default_disguise, Infantry)?,
            crew: self.faction_nullable_type(p.crew, Infantry)?,
            engineer: self.faction_nullable_type(p.engineer, Infantry)?,
            technician: self.faction_nullable_type(p.technician, Infantry)?,
            survivor_divisor: p.survivor_divisor,
            base_defenses: p
                .base_defenses
                .map(|indices| {
                    indices
                        .into_iter()
                        .map(|ordinal| {
                            self.faction_type_name(ProductionTypeRef {
                                category: Building,
                                ordinal,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
            base_defense_counts: p.base_defense_counts,
            paradrop_types: p
                .paradrop_types
                .map(|types| self.faction_drop_types(types))
                .transpose()?,
            paradrop_num: p.paradrop_num,
            paradrop_aircraft: self.faction_nullable_type(p.paradrop_aircraft, Aircraft)?,
            parachute_anim,
            tooltip_color: p.tooltip_color,
            message_text_color_index: p.message_text_color_index,
            sidebar_mix_file_index: p.sidebar_mix_file_index,
            sidebar_yuri_file_names: p.sidebar_yuri_file_names,
            eva_voice_index: p.eva_voice_index,
            hunter_seeker: self.faction_nullable_type(p.hunter_seeker, Unit)?,
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

    fn bind_faction_country(&self, p: FactionCountryPolicy) -> Result<FrozenCountry, String> {
        for &ordinal in &p.veteran_buildings {
            self.faction_type_name(ProductionTypeRef {
                category: ProductionObjectKind::Building,
                ordinal,
            })?;
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
                    .map(|i| self.faction_animation_name(i))
                    .transpose()?,
                aircraft: p
                    .paradrop
                    .aircraft
                    .map(|r| self.faction_type_name(r))
                    .transpose()?,
                types: self.faction_drop_types(p.paradrop.types)?,
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

    pub(super) fn external_country_faction(&self, country: &str) -> Option<Option<&FrozenCountry>> {
        self.external_factions
            .countries
            .get(self.country_index(country)?.0 as usize)?
            .as_ref()
            .map(|policy| policy.as_ref())
    }

    /// Select the existing construction/cameo family, independently of this
    /// particular type's membership. An explicit stock declaration suppresses
    /// an installed profile; an absent response preserves that profile.
    pub fn country_extended_veterancy_enabled(&self, country: &str) -> bool {
        self.external_country_faction(country)
            .map(|policy| policy.is_some())
            .unwrap_or(crate::rules::ARES_ENABLED)
    }

    /// Same accepted receipt as gameplay; no loading-time INI read or parent replay.
    pub(crate) fn faction_country_loading(
        &self,
        country: &str,
    ) -> Option<Option<&FactionCountryLoading>> {
        self.external_country_faction(country)
            .map(|policy| policy.map(|p| &p.loading))
    }

    /// One common Side binding for gameplay, score art and theme consumers.
    pub fn side_score_art(&self, side: i32) -> super::SideScoreArt<'_> {
        self.side_extension(side)
            .map_or_else(Default::default, AresSideExtension::score_art)
    }

    pub fn side_score_theme(&self, side: i32, theme: super::ScoreTheme) -> Option<&str> {
        self.side_extension(side)?.score_theme(theme)
    }

    /// Canopy roots shared by bounds admission, the atlas and overlay drawing.
    /// Keep the original General/NPatch roots at their existing call sites.
    pub fn faction_parachute_animations(&self) -> impl Iterator<Item = &str> {
        let null_side = (0..self.side_ids.len()).any(|index| {
            self.side_extension(index as i32)
                .is_some_and(|p| p.parachute_anim == Some(None))
        });
        self.side_parachute_animations()
            .chain(
                self.country_ids
                    .iter()
                    .filter_map(|id| self.country_parachute_anim(id)),
            )
            // GetParachuteAnim1001BBC0 substitutes this literal for an
            // explicitly-null Side canopy even if General uses other art.
            .chain(null_side.then_some("PARACH"))
    }

    pub(super) fn country_presentation(&self, country: &str) -> Option<&AresCountryPresentation> {
        if let Some(policy) = self.external_country_faction(country) {
            return policy.map(|p| &p.presentation);
        }
        #[cfg(feature = "ares")]
        return self.country_rules(country).map(|p| &p.presentation);
        #[cfg(not(feature = "ares"))]
        None
    }

    pub(super) fn country_drop_rules(&self, country: &str) -> Option<&AresCountryParadrop> {
        if let Some(policy) = self.external_country_faction(country) {
            return policy.map(|p| &p.paradrop);
        }
        #[cfg(feature = "ares")]
        return self.country_rules(country).map(|p| &p.paradrop);
        #[cfg(not(feature = "ares"))]
        None
    }

    /// Ares1002F2D0 selector; the AI owner still owns House state and evaluation.
    pub(crate) fn ai_side_token_admits(&self, trigger_side: i32, house_side: i32) -> bool {
        let extended = self
            .match_rules_policy
            .and_then(|p| p.extended_ai_side_tokens)
            .unwrap_or(crate::rules::ARES_ENABLED);
        if extended {
            trigger_side == 0 || trigger_side.checked_sub(1) == Some(house_side)
        } else {
            match trigger_side {
                1 => house_side == 0,
                2 => house_side == 1,
                3 => house_side == 2,
                _ => true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ini_parser::IniFile;

    fn faction_rules() -> RuleSet {
        RuleSet::from_ini(&IniFile::from_str(
            "[Countries]\n0=A\n1=B\n2=C\n3=D\n[Sides]\nGDI=A\nNod=B\nThirdSide=C\nFourth=D\n\
             [General]\nAlliedCrew=E1\nEngineer=ENG\nAlliedSurvivorDivisor=125\n\
             [InfantryTypes]\n0=E1\n1=ENG\n[E1]\nStrength=100\n[ENG]\nStrength=100\n\
             [BuildingTypes]\n0=FACT\n[FACT]\nStrength=500\n\
             [AircraftTypes]\n0=PDPLANE\n[PDPLANE]\nStrength=100\n",
        ))
        .unwrap()
    }

    #[test]
    fn declared_fourth_side_and_country_reach_shared_gameplay_getters() {
        let mut rules = faction_rules();
        let legacy_hash = rules.simulation_config_hash();
        rules.external_factions.sides = vec![None; 4];
        rules.external_factions.countries = vec![None; 4];
        assert_eq!(
            rules.simulation_config_hash(),
            legacy_hash,
            "absent responses keep legacy hash bytes"
        );
        let side = FactionSidePolicy::default();
        let side = rules.bind_faction_side(side).unwrap();
        rules.external_factions.sides = vec![None, None, None, Some(Some(side))];
        // The SideExt fallback is Allied for the fourth Side, not the stock
        // "everything else is Third" branch in each individual consumer.
        assert_eq!(rules.side_crew(3), Some(Some("E1")));
        assert_eq!(rules.side_engineer(3), Some(Some("ENG")));
        assert_eq!(rules.side_survivor_divisor(3), Some(125));
        let mut country = FactionCountryPolicy::default();
        country.veteran_buildings = vec![0];
        country.paradrop.types = vec![ProductionTypeRef {
            category: ProductionObjectKind::Infantry,
            ordinal: 0,
        }];
        // An owned Country type list with missing counts must not fall back.
        let bound = rules.bind_faction_country(country).unwrap();
        rules.external_factions.countries = vec![None, None, None, Some(Some(bound))];
        assert_eq!(
            rules.country_paradrop_lists("D"),
            Some((&["E1".to_string()][..], &[][..]))
        );
        // Both construction and sidebar initial-rank eligibility use this
        // exact query; no separate list or Trainable gate is introduced.
        assert!(rules.country_starts_veteran("D", rules.object("FACT").unwrap()));
        assert!(rules.country_extended_veterancy_enabled("D"));
        assert!(!rules.country_starts_veteran("A", rules.object("FACT").unwrap()));
        rules.external_factions.countries[3] = Some(None);
        rules.external_factions.sides[3] = Some(None);
        assert!(!rules.country_starts_veteran("D", rules.object("FACT").unwrap()));
        assert!(!rules.country_extended_veterancy_enabled("D"));
        assert_eq!(
            rules.side_crew(3),
            None,
            "explicit stock suppresses the installed extension"
        );
        assert_eq!(rules.country_paradrop_lists("D"), None);
        rules.match_rules_policy = Some(vera_mod_api::MatchRulesPolicy {
            survival: vera_mod_api::SurvivalRule::Retail,
            effects: None,
            script_type_conversion: None,
            extended_ai_side_tokens: Some(true),
        });
        assert!(rules.ai_side_token_admits(4, 3));
        assert!(!rules.ai_side_token_admits(4, 0));
        rules
            .match_rules_policy
            .as_mut()
            .unwrap()
            .extended_ai_side_tokens = Some(false);
        assert!(
            rules.ai_side_token_admits(4, 0),
            "explicit retail restores unrestricted unknown tokens"
        );
    }

    #[test]
    fn faction_binding_rejects_wrong_family_and_out_of_catalog_references() {
        let rules = faction_rules();
        let mut side = FactionSidePolicy::default();
        side.crew = FactionNullableRef::Resolved(ProductionTypeRef {
            category: ProductionObjectKind::Building,
            ordinal: 0,
        });
        assert!(rules.bind_faction_side(side).is_err());
        let mut country = FactionCountryPolicy::default();
        country.veteran_buildings = vec![rules.building_ids.len() as u32];
        assert!(rules.bind_faction_country(country).is_err());
        let mut side = FactionSidePolicy::default();
        side.parachute_anim = FactionNullableRef::Resolved(rules.sdk_animation_names.len() as u32);
        assert!(rules.bind_faction_side(side).is_err());
        let mut country = FactionCountryPolicy::default();
        country.paradrop.aircraft = Some(ProductionTypeRef {
            category: ProductionObjectKind::Aircraft,
            ordinal: rules.aircraft_ids.len() as u32,
        });
        assert!(rules.bind_faction_country(country).is_err());
    }
}
