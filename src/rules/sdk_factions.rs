//! Immutable name projections of the accepted component's faction resources.
//! This module never reads INI, advances guest phases, or changes registries.
use super::RuleSet;
use crate::rules::ares_country_paradrop::AresCountryParadrop;
use crate::rules::ares_country_presentation::AresCountryPresentation;
use crate::rules::ares_side_extension::AresSideExtension;
use crate::rules::faction_resource_bindings::{self, FactionCatalog};
pub(super) use crate::rules::faction_resource_bindings::FrozenCountry;
use vera_mod_api::{FactionCountryLoading, ProductionTypeRef};
#[cfg(test)]
use vera_mod_api::{FactionCountryPolicy, FactionNullableRef, FactionSidePolicy, ProductionObjectKind};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(super) struct FrozenFactionBindings {
    // An absent entry keeps the old component/profile path. Some(None) is an
    // explicit stock choice, which suppresses an installed native extension.
    pub sides: Vec<Option<Option<AresSideExtension>>>,
    pub countries: Vec<Option<Option<FrozenCountry>>>,
}

impl FrozenFactionBindings {
    pub fn has_overrides(&self) -> bool {
        self.sides.iter().any(Option::is_some) || self.countries.iter().any(Option::is_some)
    }
}

impl FactionCatalog for RuleSet {
    fn side_id(&self, ordinal: u32) -> Option<&str> {
        self.side_ids.get(ordinal as usize).map(String::as_str)
    }

    fn country_id(&self, ordinal: u32) -> Option<&str> {
        self.country_ids.get(ordinal as usize).map(String::as_str)
    }

    fn type_id(&self, reference: ProductionTypeRef) -> Option<&str> {
        self.type_array_ids(crate::rules::prerequisite::production_category(
            reference.category,
        ))
        .get(reference.ordinal as usize)
        .map(String::as_str)
    }

    fn animation_id(&self, ordinal: u32) -> Option<&str> {
        self.sdk_animation_names
            .get(ordinal as usize)
            .map(String::as_str)
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
        for ordinal in 0..self.side_ids.len() {
            let ordinal = ordinal
                .try_into()
                .map_err(|_| "Side ordinal exceeds SDK identity")?;
            result.sides.push(faction_resource_bindings::query_side(
                program, self, ordinal,
            )?);
        }
        for ordinal in 0..self.country_ids.len() {
            let ordinal = ordinal
                .try_into()
                .map_err(|_| "Country ordinal exceeds SDK identity")?;
            result
                .countries
                .push(faction_resource_bindings::query_country(
                    program, self, ordinal,
                )?);
        }
        Ok(result)
    }

    // Preserve the existing regression entry points while the implementation
    // lives in the same binder used by production queries.
    #[cfg(test)]
    fn bind_faction_side(&self, policy: FactionSidePolicy) -> Result<AresSideExtension, String> {
        faction_resource_bindings::bind_side(self, policy)
    }

    #[cfg(test)]
    fn bind_faction_country(&self, policy: FactionCountryPolicy) -> Result<FrozenCountry, String> {
        faction_resource_bindings::bind_country(self, policy)
    }

    pub(super) fn external_country_faction(&self, country: &str) -> Option<Option<&FrozenCountry>> {
        self.external_country_faction_by_index(self.country_index(country)?)
    }

    pub(super) fn external_country_faction_by_index(
        &self,
        country: super::CountryIdx,
    ) -> Option<Option<&FrozenCountry>> {
        self.external_factions
            .countries
            .get(usize::from(country.0))?
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

    /// Global UI dispatch installed by InitSideMixFiles534FA0, EVA1002FB00
    /// and MessageColor1002FA80. It cannot be inferred from a missing Side
    /// declaration: older components retain their original build profile.
    pub fn registered_side_ui_enabled(&self) -> bool {
        self.match_rules_policy
            .and_then(|policy| policy.registered_side_ui)
            .unwrap_or(crate::rules::ARES_ENABLED)
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
            registered_side_ui: None,
            building_presentation: None,
            anim_damage: None,
            warhead_house_filter: None,
            airburst_tail: None,
            projectile_fuel: None,
            shrapnel_weapon_assignment: None,
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
