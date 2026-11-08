//! Scoped SuperWeapon source facts at the original Type/Building readers.
//! The existing native registry remains the only allocation and lookup owner.
use super::*;
use serde::{Serialize, de::DeserializeOwned};
use vera_mod_api::{
    NullableResourceName, ProductionObjectKind, ProductionTypeRef, SuperReadValue,
    SuperWeaponReadPoint, SuperWeaponReadSite, SuperWeaponRegistryView,
};

#[derive(Debug, Clone)]
pub(super) struct SuperReadRecord {
    site: SuperWeaponReadSite,
    superweapons: usize,
    buildings: usize,
    technos: usize,
    animations: usize,
    /// HouseType5117D0 reads each then-current Name before its stored ID.
    /// Later Country bodies may replace that alias, so IDs alone are insufficient.
    countries: Vec<(String, Option<String>)>,
    /// Only a reached Type body has a retained physical scalar source. This
    /// is a snapshot of the same member.body, not a second extension registry.
    effective: Option<IniSection>,
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| error.to_string())
}

impl RulesPassProcessor {
    fn record_sdk_superweapon_read(
        &mut self,
        site: SuperWeaponReadSite,
        effective: Option<IniSection>,
    ) -> u32 {
        let read = u32::try_from(self.sdk_superweapon_reads.len())
            .expect("SuperWeapon read count exceeds u32");
        self.sdk_superweapon_reads.push(SuperReadRecord {
            site,
            superweapons: self.family_len(RulesTypeFamily::SuperWeapon),
            buildings: self.family_len(RulesTypeFamily::Building),
            technos: self.techno_type_order.len(),
            animations: self.family_len(RulesTypeFamily::Animation),
            countries: self
                .families
                .get(&RulesTypeFamily::Country)
                .into_iter()
                .flatten()
                .map(|member| {
                    (
                        member.native_stored_id.clone(),
                        member.body.read_name("Name", 0x31).map(str::to_owned),
                    )
                })
                .collect(),
            effective,
        });
        read
    }

    pub(super) fn record_sdk_superweapon_type_read(
        &mut self,
        index: usize,
        body_read: bool,
        pass: &IniFile,
    ) -> u32 {
        // SWTypeExt::LoadFromINIFile10033250 reaches first initialization even
        // on10033284's absent-section branch. Only the reached body may read.
        let member = &self.families[&RulesTypeFamily::SuperWeapon][index];
        let section = body_read.then(|| {
            pass.section(&member.native_stored_id)
                .expect("reached SuperWeapon Rules body")
                .name
                .clone()
        });
        let effective = body_read.then(|| member.body.clone());
        self.record_sdk_superweapon_read(
            SuperWeaponReadSite::Type {
                ordinal: u32::try_from(index).expect("SuperWeapon ordinal exceeds u32"),
                section,
            },
            effective,
        )
    }

    pub(super) fn record_sdk_superweapon_building_read(&mut self, index: usize, section: &str) {
        self.record_sdk_superweapon_read(
            SuperWeaponReadSite::Building {
                object: self.sdk_production_identity(RulesTypeFamily::Building, index),
                section: section.to_owned(),
            },
            None,
        );
    }

    pub(super) fn sdk_superweapon_view(&self) -> SuperWeaponRegistryView {
        SuperWeaponRegistryView {
            superweapons: self
                .families
                .get(&RulesTypeFamily::SuperWeapon)
                .into_iter()
                .flatten()
                .map(|member| member.native_stored_id.clone())
                .collect(),
            reads: self
                .sdk_superweapon_reads
                .iter()
                .map(|point| SuperWeaponReadPoint {
                    site: point.site.clone(),
                })
                .collect(),
        }
    }
}

impl SdkRegistryPass {
    pub(crate) fn superweapon_eva_source(&self) -> Option<&EvaRegistry> {
        self.faction_eva.as_deref()
    }
    pub(crate) fn superweapon_view(&self) -> &SuperWeaponRegistryView {
        &self.superweapon
    }

    /// Charge the same retained sections exposed to scalar reads. These bytes
    /// are derived from this pass's sidecar, never a separately cached source.
    pub(crate) fn superweapon_effective_input_bytes(&self) -> Result<usize, String> {
        self.superweapon_reads
            .iter()
            .filter_map(|point| point.effective.as_ref())
            .try_fold(0usize, |size, section| {
                let bytes = encode(section)?;
                Ok(size.saturating_add(bytes.len()))
            })
    }

    fn superweapon_point(&self, read: u32, owner: &str) -> Result<&SuperReadRecord, String> {
        let point = self
            .superweapon_reads
            .get(read as usize)
            .ok_or("invalid SuperWeapon read ordinal")?;
        if self
            .superweapon
            .reads
            .get(read as usize)
            .map(|entry| &entry.site)
            != Some(&point.site)
        {
            return Err("SuperWeapon read differs from its physical source".into());
        }
        if point.superweapons > self.superweapon.superweapons.len()
            || point.buildings > self.production.buildings.len()
            || point.technos > self.production.technos.len()
            || point.animations > self.animations.len()
            || point.countries.len() > self.production.countries.len()
            || point
                .countries
                .iter()
                .zip(&self.production.countries)
                .any(|((id, _), name)| id != name)
        {
            return Err("invalid SuperWeapon read catalog prefix".into());
        }
        let (canonical, section) = match &point.site {
            SuperWeaponReadSite::Type { ordinal, section } => (
                self.superweapon
                    .superweapons
                    .get(*ordinal as usize)
                    .filter(|_| (*ordinal as usize) < point.superweapons),
                section.as_deref(),
            ),
            SuperWeaponReadSite::Building { object, section } => {
                if object.category != ProductionObjectKind::Building
                    || object.ordinal as usize >= point.buildings
                {
                    return Err("invalid SuperWeapon Building read identity".into());
                }
                let canonical = self.production.technos[..point.technos]
                    .iter()
                    .find(|entry| entry.identity == *object)
                    .map(|entry| &entry.name)
                    .filter(|name| {
                        self.production.buildings.get(object.ordinal as usize) == Some(*name)
                    });
                (canonical, Some(section.as_str()))
            }
        };
        // Identity and physical section are separate facts. The original INI
        // lookup selected this recorded body; do not substitute the stored ID
        // for it or require a later string lookup to reproduce that selection.
        if canonical.is_none() || !section.is_some_and(|name| name.eq_ignore_ascii_case(owner)) {
            return Err("SuperWeapon reader requires its actual reached section and owner".into());
        }
        Ok(point)
    }

    pub(crate) fn superweapon_effective_section(
        &self,
        read: u32,
        owner: &str,
    ) -> Result<&IniSection, String> {
        let point = self.superweapon_point(read, owner)?;
        if !matches!(point.site, SuperWeaponReadSite::Type { .. }) {
            return Err("retained SuperWeapon scalar source requires a Type read".into());
        }
        point
            .effective
            .as_ref()
            .ok_or_else(|| "missing reached SuperWeapon body".into())
    }

    pub(crate) fn read_superweapon_value(
        &self,
        reader: &str,
        owner: &str,
        section: &IniSection,
        key: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, String> {
        if section.name.is_empty() || !section.name.eq_ignore_ascii_case(owner) {
            return Err("SuperWeapon physical section differs from its requested owner".into());
        }
        let read = if matches!(reader, "super-animation-reference" | "super-eva-reference") {
            decode::<SuperReadValue<NullableResourceName>>(input)?.read
        } else {
            decode::<SuperReadValue<()>>(input)?.read
        };
        let point = self.superweapon_point(read, owner)?;
        let building = matches!(point.site, SuperWeaponReadSite::Building { .. });
        if building != (reader == "super-provider-list") {
            return Err("SuperWeapon reader is not available at this physical read kind".into());
        }
        match reader {
            "super-nuclear-psi-default" => {
                if !key.is_empty() {
                    return Err("Nuclear Psi default accepts no authored key".into());
                }
                // Nuclear Initialize1007337A ->Anim Find1000D3E0 uses the
                // literal PSIWARN against the then-current registry, without
                // allocation. A later Anim constructor cannot repair it.
                let animation = ares_type_index_of_name(
                    "PSIWARN",
                    self.animations[..point.animations]
                        .iter()
                        .map(String::as_str),
                )
                .map(|index| self.animations[index].clone());
                encode(&animation.map_or(NullableResourceName::Clear, NullableResourceName::Named))
            }
            "super-country-mask" => encode(&read_country_mask(section, key, |name| {
                house_type_index_of_name(
                    name,
                    point
                        .countries
                        .iter()
                        .map(|(id, alias)| (id.as_str(), alias.as_deref())),
                )
            })),
            "super-building-list" => encode(
                &read_registered_index_list(section, key, 2048, |name| {
                    ares_type_index_of_name(
                        name,
                        self.production.buildings[..point.buildings]
                            .iter()
                            .map(String::as_str),
                    )
                })
                .map(|indices| {
                    indices
                        .into_iter()
                        .map(|index| index as u32)
                        .collect::<Vec<_>>()
                }),
            ),
            "super-provider-list" => encode(
                &read_registered_index_list(section, key, 2048, |name| {
                    super_weapon_index_of_name(
                        name,
                        self.superweapon.superweapons[..point.superweapons]
                            .iter()
                            .map(String::as_str),
                    )
                })
                .map(|indices| {
                    indices
                        .into_iter()
                        .map(|index| index as u32)
                        .collect::<Vec<_>>()
                }),
            ),
            "super-delivery-list" => {
                // Existing UnitDelivery uses each family's raw find_existing,
                // then admits exactly one category. Keep first ID per class,
                // repeated tokens and cross-class ambiguity at this read point.
                let types = section.read_list(key, 2048).map(|tokens| {
                    tokens
                        .into_iter()
                        .filter_map(|name| {
                            let mut matches = [
                                ProductionObjectKind::Infantry,
                                ProductionObjectKind::Unit,
                                ProductionObjectKind::Aircraft,
                                ProductionObjectKind::Building,
                            ]
                            .into_iter()
                            .filter_map(|category| {
                                let mut entries = self.production.technos[..point.technos]
                                    .iter()
                                    .filter(|entry| entry.identity.category == category);
                                let index = super_weapon_index_of_name(
                                    name,
                                    entries.clone().map(|entry| entry.name.as_str()),
                                )?;
                                entries.nth(index).map(|entry| entry.identity)
                            });
                            let first = matches.next()?;
                            matches.next().is_none().then_some(first)
                        })
                        .collect::<Vec<ProductionTypeRef>>()
                });
                encode(&types)
            }
            "super-animation-reference" => {
                let input: SuperReadValue<NullableResourceName> = decode(input)?;
                let value = match read_ares_registered_reference(
                    section,
                    key,
                    self.animations[..point.animations]
                        .iter()
                        .map(String::as_str),
                ) {
                    None => input.current,
                    Some(None) => NullableResourceName::Clear,
                    Some(Some(name)) => NullableResourceName::Named(name),
                };
                encode(&value)
            }
            "super-eva-reference" => {
                let input: SuperReadValue<NullableResourceName> = decode(input)?;
                let eva = self
                    .faction_eva
                    .as_ref()
                    .ok_or("missing fixed EVAMD source")?;
                let current = match input.current {
                    NullableResourceName::Named(name) => {
                        eva.entry(&name).map(|entry| entry.name.clone())
                    }
                    NullableResourceName::Clear | NullableResourceName::Inherit => None,
                };
                // ValueableIdx<Vox> uses its fixed catalog before none; the
                // constructor default is supplied by the declaration owner.
                encode(
                    &eva.read_ares_reference(section, key, current)
                        .map_or(NullableResourceName::Clear, NullableResourceName::Named),
                )
            }
            _ => Err(format!("unknown scoped SuperWeapon reader: {reader}")),
        }
    }
}
