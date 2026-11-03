//! Physical faction read scopes and adapters over the existing INI kernels.
//! No extension phases, inheritance, registry allocation or live House state.
use super::*;
use serde::{Serialize, de::DeserializeOwned};
use vera_mod_api::{
    CountryReadPolicy, FactionLookup, FactionNullableRef, FactionReadPoint, FactionReadSite,
    FactionReadValue, FactionRegistryPrefix, FactionRegistryView, ProductionObjectKind,
    ProductionTypeRef,
};

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| error.to_string())
}
fn nullable<T>(value: FactionNullableRef<T>) -> Option<Option<T>> {
    match value {
        FactionNullableRef::Unset => None,
        FactionNullableRef::Null => Some(None),
        FactionNullableRef::Resolved(value) => Some(Some(value)),
    }
}
fn wire_nullable<T>(value: Option<Option<T>>) -> FactionNullableRef<T> {
    match value {
        None => FactionNullableRef::Unset,
        Some(None) => FactionNullableRef::Null,
        Some(Some(value)) => FactionNullableRef::Resolved(value),
    }
}

impl SdkRegistryPass {
    pub(crate) fn faction_view(&self) -> &FactionRegistryView {
        &self.faction
    }

    fn faction_point(
        &self,
        ordinal: u32,
        section: &str,
        literal: bool,
    ) -> Result<&FactionReadPoint, String> {
        let point = self
            .faction
            .reads
            .get(ordinal as usize)
            .ok_or("invalid faction read ordinal")?;
        let (owner, prefix) = match point.site {
            FactionReadSite::SideSweep { side } => (
                self.faction
                    .sides
                    .get(side as usize)
                    .filter(|_| side < point.prefix.sides),
                point.prefix,
            ),
            FactionReadSite::CountrySuffix { country, .. } => (
                self.faction
                    .countries
                    .get(country as usize)
                    .filter(|_| country < point.prefix.countries),
                point.prefix,
            ),
        };
        let owner = owner.ok_or("invalid faction read identity")?;
        if !literal && !owner.eq_ignore_ascii_case(section) {
            return Err("faction section differs from its actual read owner".into());
        }
        if prefix.sides as usize > self.faction.sides.len()
            || prefix.countries as usize > self.faction.countries.len()
            || prefix.animations as usize > self.animations.len()
            || prefix.colors as usize > self.faction_colors.len()
        {
            return Err("invalid faction read catalog prefix".into());
        }
        for (kind, count) in [
            (ProductionObjectKind::Infantry, prefix.infantry),
            (ProductionObjectKind::Unit, prefix.units),
            (ProductionObjectKind::Aircraft, prefix.aircraft),
            (ProductionObjectKind::Building, prefix.buildings),
        ] {
            if count as usize
                > self
                    .production
                    .technos
                    .iter()
                    .filter(|entry| entry.identity.category == kind)
                    .count()
            {
                return Err("invalid faction Type read prefix".into());
            }
        }
        Ok(point)
    }

    fn faction_type(
        &self,
        point: &FactionReadPoint,
        kind: ProductionObjectKind,
        name: &str,
    ) -> Option<ProductionTypeRef> {
        let count = match kind {
            ProductionObjectKind::Infantry => point.prefix.infantry,
            ProductionObjectKind::Unit => point.prefix.units,
            ProductionObjectKind::Aircraft => point.prefix.aircraft,
            ProductionObjectKind::Building => point.prefix.buildings,
        };
        // Side1002D910/1002D970 family lookup is before the outer none test.
        self.production
            .technos
            .iter()
            .find(|entry| {
                entry.identity.category == kind
                    && entry.identity.ordinal < count
                    && entry.name.eq_ignore_ascii_case(name)
            })
            .map(|entry| entry.identity)
    }

    fn faction_paradroppable(
        &self,
        point: &FactionReadPoint,
        name: &str,
    ) -> Option<ProductionTypeRef> {
        self.faction_type(point, ProductionObjectKind::Infantry, name)
            .or_else(|| self.faction_type(point, ProductionObjectKind::Unit, name))
    }
    fn faction_animation(&self, point: &FactionReadPoint, name: &str) -> Option<u32> {
        self.animations[..point.prefix.animations as usize]
            .iter()
            .position(|id| id.eq_ignore_ascii_case(name))
            .map(|index| index as u32)
    }
    fn faction_color(&self, point: &FactionReadPoint, name: &str) -> Option<i32> {
        // ColorScheme1001BB30 kind1: each retained Colors row contributes two slots.
        ares_color_scheme_index(
            name,
            self.faction_colors[..point.prefix.colors as usize]
                .iter()
                .map(String::as_str),
        )
    }

    pub(crate) fn read_faction_value(
        &self,
        reader: &str,
        owner: &str,
        section: &IniSection,
        key: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, String> {
        if !section.name.is_empty() && !section.name.eq_ignore_ascii_case(owner) {
            return Err("faction physical section differs from its requested owner".into());
        }
        if let Some(kind) = reader.strip_prefix("faction-find:") {
            if !owner.is_empty() || !key.is_empty() {
                return Err("literal faction lookup requires empty section/key".into());
            }
            let seed: FactionLookup = decode(input)?;
            let point = self.faction_point(seed.read, "", true)?;
            return match kind {
                "country" => encode(
                    &ares_country_index_of_name(
                        &seed.name,
                        self.faction.countries[..point.prefix.countries as usize]
                            .iter()
                            .map(String::as_str),
                    )
                    .map(|index| u32::from(index.0)),
                ),
                "color" => encode(&self.faction_color(point, &seed.name)),
                "aircraft" => encode(
                    &self
                        .faction_type(point, ProductionObjectKind::Aircraft, &seed.name)
                        .map(|identity| identity.ordinal),
                ),
                _ => Err(format!("unknown faction literal registry: {kind}")),
            };
        }
        macro_rules! seed {
            ($value:ty) => {{
                let seed: FactionReadValue<$value> = decode(input)?;
                let point = self.faction_point(seed.read, owner, false)?;
                (seed.current, point)
            }};
        }
        if let Some(family) = reader.strip_prefix("faction-nullable-ref:") {
            if family == "animation" {
                let (current, point) = seed!(FactionNullableRef<u32>);
                return encode(&wire_nullable(read_ares_type_ref(
                    section,
                    key,
                    &nullable(current),
                    |name| self.faction_animation(point, name),
                )));
            }
            let kind = match family {
                "infantry" => ProductionObjectKind::Infantry,
                "unit" => ProductionObjectKind::Unit,
                "aircraft" => ProductionObjectKind::Aircraft,
                _ => return Err(format!("unknown faction nullable registry: {family}")),
            };
            let (current, point) = seed!(FactionNullableRef<ProductionTypeRef>);
            return encode(&wire_nullable(read_ares_type_ref(
                section,
                key,
                &nullable(current),
                |name| self.faction_type(point, kind, name),
            )));
        }
        match reader {
            "faction-valueable-ref:aircraft" => {
                let (current, point) = seed!(Option<ProductionTypeRef>);
                encode(&read_valueable_type(section, key, &current, |name| {
                    self.faction_type(point, ProductionObjectKind::Aircraft, name)
                }))
            }
            "faction-valueable-ref:animation" => {
                let (current, point) = seed!(Option<u32>);
                encode(&read_valueable_type(section, key, &current, |name| {
                    self.faction_animation(point, name)
                }))
            }
            "faction-nullable-vector:buildings" => {
                let (current, point) = seed!(Option<Vec<u32>>);
                encode(&read_nullable_type_vector(
                    section,
                    key,
                    current.as_ref(),
                    |name| {
                        self.faction_type(point, ProductionObjectKind::Building, name)
                            .map(|id| id.ordinal)
                    },
                ))
            }
            "faction-valueable-vector:buildings" => {
                let (current, point) = seed!(Vec<u32>);
                encode(&read_valueable_type_vector(
                    section,
                    key,
                    &current,
                    |name| {
                        self.faction_type(point, ProductionObjectKind::Building, name)
                            .map(|id| id.ordinal)
                    },
                ))
            }
            "faction-nullable-vector:paradroppables" => {
                let (current, point) = seed!(Option<Vec<ProductionTypeRef>>);
                encode(&read_nullable_type_vector(
                    section,
                    key,
                    current.as_ref(),
                    |name| self.faction_paradroppable(point, name),
                ))
            }
            "faction-valueable-vector:paradroppables" => {
                let (current, point) = seed!(Vec<ProductionTypeRef>);
                encode(&read_valueable_type_vector(
                    section,
                    key,
                    &current,
                    |name| self.faction_paradroppable(point, name),
                ))
            }
            "faction-nullable-ints" => {
                let (current, _) = seed!(Option<Vec<i32>>);
                encode(&read_nullable_int_vector(section, key, current.as_ref()))
            }
            "faction-valueable-ints" => {
                let (current, _) = seed!(Vec<i32>);
                encode(&read_valueable_int_vector(section, key, &current))
            }
            "faction-color-index" => {
                let (current, point) = seed!(i32);
                encode(&read_ares_index(section, key, current, |name| {
                    self.faction_color(point, name)
                }))
            }
            "faction-eva-index" => {
                let (current, point) = seed!(i32);
                if !matches!(point.site, FactionReadSite::SideSweep { .. }) {
                    return Err("EVA reference is outside an actual Side reader".into());
                }
                let eva = self
                    .faction_eva
                    .as_deref()
                    .ok_or("fixed EVA catalog unavailable at this read")?;
                encode(&read_ares_index(section, key, current, |name| {
                    eva.voice_index_for_tag(name)
                }))
            }
            "faction-pcx" => {
                let (current, _) = seed!(Option<String>);
                encode(&match section.read_ares_pcx_filename(key) {
                    Some(name) => (!is_native_none_type_name(&name)).then_some(name),
                    None => current,
                })
            }
            _ => Err(format!("unknown faction reader: {reader}")),
        }
    }
}

/// One acceptance loop for completed physical reads. Each successful step
/// commits its opaque checkpoint before consuming that step; a later failure
/// keeps that checkpoint and the unconsumed failure/suffix. Legacy modules keep
/// their pending history for the original final-binding path.
pub(super) fn advance_sdk_configuration(
    seed: &mut Option<std::sync::Arc<crate::mod_sdk::Program>>,
    steps: &mut Vec<SdkConfigurationStep>,
) -> Result<(), RulesError> {
    if seed
        .as_ref()
        .is_none_or(|program| !program.has_country_read())
    {
        return Ok(());
    }
    let mut index = 0;
    while let Some(step) = steps.first() {
        let candidate = seed
            .as_ref()
            .expect("checked configuration seed")
            .advance_configuration_step(index, step)
            .map_err(|value| RulesError::InvalidValue {
                section: "vera-mod".into(),
                key: "configure_step".into(),
                expected: "accepted completed component configuration".into(),
                value,
            })?;
        *seed = Some(std::sync::Arc::new(candidate));
        steps.remove(0);
        index += 1;
    }
    Ok(())
}

impl RulesPassProcessor {
    pub(super) fn external_faction_owner(&self) -> bool {
        self.sdk_configuration_seed
            .as_ref()
            .is_some_and(|program| program.has_faction_resource())
    }

    pub(super) fn record_sdk_faction_read(&mut self, site: FactionReadSite) {
        let count =
            |family| u32::try_from(self.family_len(family)).expect("faction registry exceeds u32");
        let prefix = FactionRegistryPrefix {
            sides: count(RulesTypeFamily::Side),
            countries: count(RulesTypeFamily::Country),
            infantry: count(RulesTypeFamily::Infantry),
            units: count(RulesTypeFamily::Vehicle),
            aircraft: count(RulesTypeFamily::Aircraft),
            buildings: count(RulesTypeFamily::Building),
            animations: count(RulesTypeFamily::Animation),
            colors: u32::try_from(self.colors.len()).expect("Colors ordinal"),
        };
        self.sdk_faction_reads
            .push(FactionReadPoint { prefix, site });
    }

    pub(super) fn process_country_only_pass(
        &mut self,
        pass: &IniFile,
        primary_rules: bool,
    ) -> Result<(), RulesError> {
        self.sdk_type_reads.clear();
        self.sdk_production_reads.clear();
        self.sdk_foundation_reads.clear();
        self.sdk_faction_reads.clear();
        let result = self.process_house_family(pass, primary_rules);
        if self.external_faction_owner() {
            self.sdk_configuration_steps
                .push(SdkConfigurationStep::CountryPass {
                    ini: pass.clone(),
                    registry: self.sdk_registry_pass(),
                });
        }
        result
    }

    pub(super) fn configure_country_frontier(
        &mut self,
        pass: &IniFile,
    ) -> Result<CountryReadPolicy, RulesError> {
        let fallback = if crate::rules::ARES_ENABLED {
            CountryReadPolicy::ExistingOnly2048
        } else {
            CountryReadPolicy::RetailAllocate128
        };
        if self
            .sdk_configuration_seed
            .as_ref()
            .is_none_or(|program| !program.has_country_read())
        {
            return Ok(fallback);
        }
        let error = |value| RulesError::InvalidValue {
            section: "vera-mod".into(),
            key: "configure_country_read".into(),
            expected: "accepted pre-body component configuration".into(),
            value,
        };
        advance_sdk_configuration(
            &mut self.sdk_configuration_seed,
            &mut self.sdk_configuration_steps,
        )?;
        let seed = self
            .sdk_configuration_seed
            .as_ref()
            .expect("checked configuration seed");
        let (candidate, policy) = seed.configure_country_read(pass).map_err(error)?;
        self.sdk_configuration_seed = Some(std::sync::Arc::new(candidate));
        Ok(policy.unwrap_or(fallback))
    }
}
