//! One atomic projection of immutable Super resources into the existing types.
//! Allocation, physical reader history and all live Super state stay with their
//! original owners; this binder does not read new INI or allocate references.
use super::{RuleSet, WeaponHandle};
use crate::rules::ini_parser::IniSection;
use crate::rules::native_processing::ProcessedRulesLayers;
use crate::rules::superweapon_targeting::{
    HouseMask, SuperWeaponTargeting, TargetConstraints, TargetMask, TargetRequirements,
};
use crate::rules::superweapon_type::{
    AresSuperWeaponAvailability, AresSuperWeaponCharge, AresSuperWeaponEva,
    AresSuperWeaponMetadata, AresSuperWeaponPresentation, DeliveryOwner, DeliveryType,
    GenericWarheadAiTargeting, GenericWarheadSettings, NuclearSettings,
    SuperWeaponBuildingRequirements, SuperWeaponKind, SuperWeaponType, UnitDeliveryRules,
};
use crate::util::native_x87::NativeF64Bits;
use std::collections::{BTreeMap, HashMap};
use vera_mod_api::{
    NullableResourceName, ProductionTypeRef, SuperWeaponDeliveryOwner, SuperWeaponPayloadPolicy,
    SuperWeaponResourcePolicy, SuperWeaponSemantics,
};

/// Derived immutable binding inputs from the same ordered Process registry.
/// The profile copy preserves resolved native references and the old component
/// fallback. The body preserves the stock reader's retained enum history.
#[derive(Debug, Clone)]
pub(super) struct SdkSuperWeaponSource {
    stored_id: String,
    projected_id: String,
    body: Option<IniSection>,
    profile: Option<SuperWeaponType>,
    warhead: Option<String>,
    /// Outer Some is authority from the original Nuclear physical action;
    /// inner None is its retained null payload, not an absent declaration.
    nuclear_payload: Option<Option<WeaponHandle>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct FrozenSuperWeaponBindings {
    policy: SuperWeaponResourcePolicy,
    pub(super) metadata: BTreeMap<String, AresSuperWeaponMetadata>,
    providers: Vec<Vec<u32>>,
    /// The stock Type reader may retain an earlier valid kind after a later
    /// extension name. Hash its effective dispatch, not just the final text.
    dispatch: BTreeMap<String, (SuperWeaponKind, Option<String>, bool, bool, i32)>,
}

pub(super) struct SuperWeaponResourceBindings {
    frozen: Option<FrozenSuperWeaponBindings>,
    types: Option<HashMap<String, SuperWeaponType>>,
    ranges: Vec<Option<(Option<i32>, Option<i32>)>>,
}

impl SuperWeaponResourceBindings {
    pub(super) fn has_nuclear(&self) -> bool {
        self.types
            .as_ref()
            .is_some_and(|types| types.values().any(|sw| sw.nuclear.is_some()))
    }
}

impl RuleSet {
    pub(super) fn capture_superweapon_resource_sources(
        &mut self,
        processed: &ProcessedRulesLayers,
    ) {
        self.sdk_superweapon_sources = processed
            .super_weapon_resource_reference_states()
            .map(|(name, (warhead, nuclear_payload))| SdkSuperWeaponSource {
                stored_id: name.to_owned(),
                projected_id: self
                    .super_weapon_order
                    .iter()
                    .find(|id| id.eq_ignore_ascii_case(name))
                    .cloned()
                    .unwrap_or_else(|| name.to_owned()),
                body: processed.ini().section(name).cloned(),
                profile: self.super_weapon(name).cloned(),
                // This is the original common SW.Warhead slot after its
                // physical FindOnly read. Never resolve a declaration's name.
                warhead: warhead.map(str::to_owned),
                nuclear_payload: nuclear_payload.map(|payload| {
                    payload.map(|index| {
                        WeaponHandle(
                            u32::try_from(index).expect("Weapon array exceeds stable identity"),
                        )
                    })
                }),
            })
            .collect();
    }

    /// The original audio binders run before resource admission. Keep their
    /// canonical profile inputs for a later explicit stock/profile rebind.
    pub(super) fn capture_superweapon_profile_audio(&mut self) {
        let profile_active = self.external_superweapons.is_none();
        for source in &mut self.sdk_superweapon_sources {
            let Some(profile) = source.profile.as_mut() else {
                continue;
            };
            let Some(current) = self.super_weapons.get(&source.projected_id) else {
                continue;
            };
            profile.special_sound = current.special_sound.clone();
            profile.start_sound = current.start_sound.clone();
            if profile_active {
                profile.presentation.sound = current.presentation.sound.clone();
                profile.insufficient_funds_eva = current.insufficient_funds_eva.clone();
                if let Some(delivery) = profile.unit_delivery.as_mut() {
                    delivery.activation_sound = current
                        .unit_delivery
                        .as_ref()
                        .and_then(|current| current.activation_sound.clone());
                }
            }
        }
    }

    /// One installed semantic selection, shared by app and simulation. Neither
    /// hook presence nor individual nonempty lists establish this selection.
    pub fn superweapon_semantics(&self) -> SuperWeaponSemantics {
        self.external_superweapons.as_ref().map_or_else(
            crate::rules::superweapon_type::profile_semantics,
            |catalog| catalog.policy.semantics,
        )
    }

    /// Admitted external EVA references. An absent domain keeps historical
    /// producers; legacy profile readers and their three consumers stay separate.
    pub fn super_weapon_eva(&self, sw: &SuperWeaponType) -> Option<&AresSuperWeaponEva> {
        self.external_superweapons.as_ref()?;
        self.super_weapon_metadata(&sw.id)?.eva()
    }

    /// Detection100349D0 uses GetSuperWeapon1000D610 for the new cohort.
    /// Old external components preserve their original primary-only producer.
    pub fn superweapon_eva_providers_enabled(&self) -> bool {
        self.external_superweapons
            .as_ref()
            .map_or(cfg!(feature = "ares"), |catalog| {
                catalog
                    .policy
                    .definitions
                    .iter()
                    .any(|definition| definition.eva.is_some())
            })
    }

    /// Historical external configurations never inherit the compiled profile's
    /// influence rules. Domain presence is admitted with the immutable batch.
    pub(crate) fn superweapon_influence_enabled(&self) -> bool {
        if self.external_mod.is_none() {
            cfg!(feature = "ares")
        } else {
            self.external_superweapons
                .as_ref()
                .is_some_and(|catalog| catalog.policy.techno_influence.is_some())
        }
    }

    /// Some(empty) suppresses the old profile vector for this Building. None
    /// leaves the existing object-owned legacy vector and its original order.
    pub(crate) fn additional_super_weapon_indices(&self, building: usize) -> Option<&[u32]> {
        self.external_superweapons
            .as_ref()?
            .providers
            .get(building)
            .map(Vec::as_slice)
    }

    /// Installed Nuclear10073113 stores the actual SW definition ordinal.
    /// Saved Bullet associations may resolve only that admitted unique source.
    pub(crate) fn nuclear_super_weapon(&self, ordinal: u32) -> Option<&SuperWeaponType> {
        self.external_superweapons.as_ref()?;
        let source = self.sdk_superweapon_sources.get(ordinal as usize)?;
        let super_weapon = self.super_weapons.get(&source.projected_id)?;
        (super_weapon.nuclear.as_ref()?.source_ordinal == ordinal
            && self
                .sdk_superweapon_sources
                .iter()
                .filter(|other| {
                    other
                        .projected_id
                        .eq_ignore_ascii_case(&source.projected_id)
                })
                .count()
                == 1)
            .then_some(super_weapon)
    }

    pub(super) fn collect_superweapon_resource_bindings(
        &self,
    ) -> Result<SuperWeaponResourceBindings, String> {
        let policy = self
            .external_mod
            .as_deref()
            .filter(|program| program.has_superweapon_resource())
            .map(|program| program.superweapon_resource_policy())
            .transpose()?
            .flatten();
        let Some(mut policy) = policy else {
            let types = self.external_superweapons.as_ref().map(|_| {
                self.sdk_superweapon_sources
                    .iter()
                    .filter_map(|source| {
                        source
                            .profile
                            .clone()
                            .map(|profile| (source.projected_id.clone(), profile))
                    })
                    .collect()
            });
            return Ok(SuperWeaponResourceBindings {
                frozen: None,
                types,
                ranges: vec![None; self.object_list.len()],
            });
        };
        policy.validate()?;
        let exact_type = |identity: ProductionTypeRef| {
            i32::try_from(identity.ordinal)
                .ok()
                .and_then(|index| {
                    self.type_array_at(
                        crate::rules::prerequisite::production_category(identity.category),
                        index,
                    )
                })
                .ok_or_else(|| {
                    "SuperWeapon Techno reference is outside its native array".to_owned()
                })
        };
        // TechnoTypeExt nullable10008CC0: the external domain has an explicit
        // unset base. Validate everything before publishing any ObjectType.
        let mut ranges = vec![None; self.object_list.len()];
        if let Some(declarations) = &mut policy.techno_influence {
            declarations.sort_by_key(|entry| entry.object);
            let mut declared = BTreeMap::new();
            for entry in declarations {
                exact_type(entry.object)?;
                declared.insert(
                    entry.object,
                    (entry.designator_range, entry.inhibitor_range),
                );
            }
            for (object, destination) in self.object_list.iter().zip(&mut ranges) {
                let identity = ProductionTypeRef {
                    category: crate::rules::prerequisite::production_kind(object.category),
                    ordinal: u32::try_from(object.native_type_index())
                        .map_err(|_| "invalid native Techno ordinal")?,
                };
                *destination = Some(declared.get(&identity).copied().unwrap_or((None, None)));
            }
        }
        // This existing native construction-order projection is reconstructed
        // from retained members on fresh and saved loads. Readpoint logs are
        // skipped by save and must never be used as the final type directory.
        let native_type_name = |category, ordinal: u32| {
            self.native_techno_type_order
                .iter()
                .filter(|(kind, _, _)| *kind == category)
                .nth(ordinal as usize)
                .map(|(_, name, _)| name)
        };
        let sounds = self.fixed_sound_source.as_deref();
        let eva = self.fixed_eva_source.as_deref();
        if let Some(expected) = policy.sound_source_hash
            && sounds.is_none_or(|source| source.source_hash() != expected)
        {
            return Err(
                "SuperWeapon sounds differ from the process-selected SOUNDMD catalog".into(),
            );
        }
        if let Some(expected) = policy.eva_source_hash
            && eva.is_none_or(|source| source.source_hash() != expected)
        {
            return Err("SuperWeapon EVA differs from the process-selected EVAMD catalog".into());
        }
        let resolve_sound = |name: &mut NullableResourceName| -> Result<(), String> {
            if let NullableResourceName::Named(value) = name {
                *value = sounds
                    .and_then(|sounds| sounds.get(value))
                    .ok_or_else(|| format!("unregistered SuperWeapon sound: {value}"))?
                    .id
                    .clone();
            }
            Ok(())
        };
        let resolve_eva = |name: &mut NullableResourceName| -> Result<(), String> {
            if let NullableResourceName::Named(value) = name {
                *value = eva
                    .and_then(|eva| eva.entry(value))
                    .ok_or_else(|| format!("unregistered SuperWeapon EVA: {value}"))?
                    .name
                    .clone();
            }
            Ok(())
        };
        let building_name = |ordinal: u32| -> Result<String, String> {
            native_type_name(crate::rules::object_type::ObjectCategory::Building, ordinal)
                .cloned()
                .ok_or_else(|| "SuperWeapon Building reference is outside its native array".into())
        };
        // Native array order is the publication order, even if a component
        // serializes its independent declarations in another order.
        policy
            .definitions
            .sort_by_key(|definition| definition.ordinal);
        policy.providers.sort_by_key(|provider| provider.object);
        let mut definitions = BTreeMap::new();
        let mut metadata_by_ordinal = BTreeMap::new();
        for definition in &mut policy.definitions {
            let source = self
                .sdk_superweapon_sources
                .get(definition.ordinal as usize)
                .filter(|source| source.stored_id == definition.stored_id)
                .ok_or("SuperWeapon declaration does not match its native ordinal and stored ID")?;
            resolve_sound(&mut definition.presentation.sound)?;
            resolve_eva(&mut definition.insufficient_funds_eva)?;
            if let Some(eva) = &mut definition.eva {
                for reference in [
                    &mut eva.ready,
                    &mut eva.activated,
                    &mut eva.detected,
                    &mut eva.impatient,
                    &mut eva.select_target,
                ] {
                    resolve_eva(reference)?;
                }
            }
            if let NullableResourceName::Named(name) = &mut definition.presentation.animation {
                *name = self
                    .sdk_animation_names
                    .iter()
                    .find(|id| id.eq_ignore_ascii_case(name))
                    .ok_or_else(|| format!("unregistered SuperWeapon animation: {name}"))?
                    .clone();
            }
            let buildings = SuperWeaponBuildingRequirements {
                auxiliary: definition
                    .availability
                    .auxiliary
                    .iter()
                    .map(|&index| building_name(index))
                    .collect::<Result<_, _>>()?,
                negative: definition
                    .availability
                    .negative
                    .iter()
                    .map(|&index| building_name(index))
                    .collect::<Result<_, _>>()?,
            };
            let mut candidate = stock_type(source);
            candidate.building_requirements = Some(buildings.clone());
            candidate.target_constraints =
                TargetConstraints::from_bits(u32::from(definition.constraints))
                    .ok_or("invalid SuperWeapon constraint bits")?;
            candidate.use_ai_targeting = definition.use_ai_targeting;
            candidate.money_amount = definition.money_amount;
            candidate.insufficient_funds_eva =
                nullable_name(&definition.insufficient_funds_eva).flatten();
            candidate.presentation = AresSuperWeaponPresentation {
                animation: nullable_name(&definition.presentation.animation),
                height: definition.presentation.height,
                visibility: HouseMask::from_bits(u32::from(definition.presentation.visibility))
                    .ok_or("invalid SuperWeapon presentation house bits")?,
                sound: nullable_name(&definition.presentation.sound),
                ..AresSuperWeaponPresentation::default()
            };
            // Validate the entire batch, including references in a currently
            // disabled mechanism, before publishing any effective field.
            match &mut definition.payload {
                SuperWeaponPayloadPolicy::Retail => {}
                SuperWeaponPayloadPolicy::Nuclear {
                    damage,
                    silo_launch,
                    take_off,
                    psi_warning,
                    activation_sound,
                } => {
                    let payload = source.nuclear_payload.ok_or(
                        "Nuclear declaration requires its original physical reference reader",
                    )?;
                    if self
                        .sdk_superweapon_sources
                        .iter()
                        .filter(|other| {
                            other
                                .projected_id
                                .eq_ignore_ascii_case(&source.projected_id)
                        })
                        .count()
                        != 1
                    {
                        return Err(
                            "Nuclear declaration requires a unique native SuperWeapon projection"
                                .into(),
                        );
                    }
                    for animation in [&mut *take_off, &mut *psi_warning] {
                        if let NullableResourceName::Named(name) = animation {
                            *name = self
                                .sdk_animation_names
                                .iter()
                                .find(|id| id.eq_ignore_ascii_case(name))
                                .ok_or_else(|| format!("unregistered Nuclear animation: {name}"))?
                                .clone();
                        }
                    }
                    resolve_sound(activation_sound)?;
                    // Nuclear100732F0/10073320 use the retained payload's
                    // Damage/WH unless the common nullable override applies.
                    // Native kind/action history remains with stock_type.
                    if policy.semantics.custom_payloads {
                        candidate.nuclear = Some(NuclearSettings {
                            source_ordinal: definition.ordinal,
                            payload,
                            warhead: source.warhead.clone(),
                            damage: *damage,
                            silo_launch: *silo_launch,
                            take_off: nullable_name(take_off),
                            psi_warning: nullable_name(psi_warning).flatten(),
                            activation_sound: nullable_name(activation_sound),
                        });
                    }
                }
                SuperWeaponPayloadPolicy::GenericWarhead { damage, targeting } => {
                    let requirements = |target, house| -> Result<TargetRequirements, String> {
                        Ok(TargetRequirements {
                            target: TargetMask::from_bits(u32::from(target))
                                .ok_or("invalid SuperWeapon target bits")?,
                            house: HouseMask::from_bits(u32::from(house))
                                .ok_or("invalid SuperWeapon house bits")?,
                        })
                    };
                    let settings = GenericWarheadSettings {
                        warhead: source.warhead.clone(),
                        damage: *damage,
                        ai_targeting: if targeting.offensive_ai {
                            GenericWarheadAiTargeting::Offensive
                        } else {
                            GenericWarheadAiTargeting::None
                        },
                        targeting: SuperWeaponTargeting {
                            manual: requirements(targeting.manual_target, targeting.manual_house)?,
                            automatic: requirements(
                                targeting.automatic_target,
                                targeting.automatic_house,
                            )?,
                            fire_into_shroud: targeting.fire_into_shroud,
                            range_minimum: NativeF64Bits::from_bits(targeting.range_minimum),
                            range_maximum: NativeF64Bits::from_bits(targeting.range_maximum),
                        },
                    };
                    if policy.semantics.custom_payloads {
                        candidate.kind = SuperWeaponKind::GenericWarhead;
                        candidate.generic_warhead = Some(settings);
                        candidate.action = Some("Attack".into());
                        candidate.pre_click = false;
                        candidate.post_click = false;
                        candidate.pre_dependent = -1;
                    }
                }
                SuperWeaponPayloadPolicy::UnitDelivery {
                    types,
                    deferment,
                    owner,
                    base_normal,
                    action,
                    activation_sound,
                } => {
                    resolve_sound(activation_sound)?;
                    let types = types
                        .iter()
                        .map(|identity| {
                            let category = crate::rules::prerequisite::production_category(
                                identity.category,
                            );
                            let id = native_type_name(category, identity.ordinal)
                                .ok_or("SuperWeapon delivery reference is outside its native Type array")?
                                .clone();
                            Ok(DeliveryType { category, id })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    if policy.semantics.custom_payloads {
                        candidate.kind = SuperWeaponKind::UnitDelivery;
                        candidate.unit_delivery = Some(UnitDeliveryRules {
                            types,
                            deferment: *deferment,
                            owner: match owner {
                                SuperWeaponDeliveryOwner::Invoker => DeliveryOwner::Invoker,
                                SuperWeaponDeliveryOwner::Neutral => DeliveryOwner::Neutral,
                                SuperWeaponDeliveryOwner::Special => DeliveryOwner::Special,
                                SuperWeaponDeliveryOwner::Civilian => DeliveryOwner::Civilian,
                            },
                            base_normal: *base_normal,
                            activation_sound: nullable_name(activation_sound).flatten(),
                        });
                        candidate.action = Some(action.clone());
                        candidate.pre_click = false;
                        candidate.post_click = false;
                        candidate.pre_dependent = -1;
                    }
                }
            }
            let availability = AresSuperWeaponAvailability::from_config(
                definition.availability.allow_player,
                definition.availability.allow_ai,
                definition.availability.required_houses,
                definition.availability.forbidden_houses,
                definition.availability.shots,
            );
            let charge = AresSuperWeaponCharge {
                initial_ready: definition.charge.initial_ready,
                virtual_charge: definition.charge.virtual_charge,
            };
            let mut influence =
                crate::rules::superweapon_influence::SuperWeaponInfluence::default();
            if let Some(declaration) = &definition.influence {
                for &identity in declaration
                    .designators
                    .iter()
                    .chain(&declaration.inhibitors)
                {
                    exact_type(identity)?;
                }
                influence.designators = declaration.designators.clone();
                influence.any_designator = declaration.any_designator;
                influence.inhibitors = declaration.inhibitors.clone();
                influence.any_inhibitor = declaration.any_inhibitor;
            }
            metadata_by_ordinal.insert(
                definition.ordinal,
                AresSuperWeaponMetadata::project(
                    &availability,
                    candidate.aux_building.as_deref(),
                    Some(&buildings),
                    &charge,
                    &influence,
                    definition.common_targeting.as_ref()
                        .filter(|_| policy.semantics.custom_payloads)
                        .map(|common| {
                            crate::rules::superweapon_targeting::AresSuperWeaponTargetingState::from_external_config(
                                candidate.target_constraints,
                                candidate.use_ai_targeting,
                                common,
                            )
                        }),
                    definition.eva.as_ref().map(AresSuperWeaponEva::from_external),
                ),
            );
            definitions.insert(definition.ordinal, candidate);
        }
        let mut provider_declarations = BTreeMap::new();
        for provider in &policy.providers {
            exact_type(provider.object)?;
            let mut indices = Vec::new();
            for &index in &provider.additional {
                let source = self
                    .sdk_superweapon_sources
                    .get(index as usize)
                    .ok_or("SuperWeapon provider reference is outside the native Super array")?;
                let projected = self
                    .super_weapon_index(&source.stored_id)
                    .ok_or("registered SuperWeapon has no runtime projection")?;
                indices.push(
                    u32::try_from(projected)
                        .map_err(|_| "SuperWeapon projected ordinal exceeds u32")?,
                );
            }
            provider_declarations.insert(provider.object.ordinal, indices);
        }
        let mut providers = vec![Vec::new(); self.building_ids.len()];
        for (ordinal, indices) in provider_declarations {
            // BuildingTypeExt::GetSuperWeapon1000D610 reads the actual Type's
            // own vector+178/+17C, reached via Building+520 at10034A6A.
            // Re-Find by stored ID would merge distinct native allocations.
            *providers
                .get_mut(ordinal as usize)
                .ok_or("invalid native Building ordinal")? = indices;
        }
        let mut types = HashMap::new();
        let mut metadata = BTreeMap::new();
        for (ordinal, source) in self.sdk_superweapon_sources.iter().enumerate() {
            if let Some(value) = metadata_by_ordinal.remove(&(ordinal as u32)) {
                metadata.insert(source.stored_id.to_ascii_uppercase(), value);
            } else {
                metadata.remove(&source.stored_id.to_ascii_uppercase());
            }
            let candidate = definitions
                .remove(&(ordinal as u32))
                .or_else(|| source.profile.as_ref().map(|_| stock_type(source)));
            if let Some(candidate) = candidate {
                // Retain the existing name-keyed projection's last-ordinal
                // overwrite for duplicate/truncated IDs. This is not a new
                // runtime identity model; admission above still checks every ID.
                types.insert(source.projected_id.clone(), candidate);
            }
        }
        let dispatch = types
            .iter()
            .map(|(id, sw)| {
                (
                    id.to_ascii_uppercase(),
                    (
                        sw.kind,
                        sw.action.clone(),
                        sw.pre_click,
                        sw.post_click,
                        sw.pre_dependent,
                    ),
                )
            })
            .collect();
        Ok(SuperWeaponResourceBindings {
            frozen: Some(FrozenSuperWeaponBindings {
                policy,
                metadata,
                providers,
                dispatch,
            }),
            types: Some(types),
            ranges,
        })
    }

    pub(super) fn publish_superweapon_resource_bindings(
        &mut self,
        bindings: SuperWeaponResourceBindings,
    ) {
        if let Some(types) = bindings.types {
            self.super_weapons = types;
        }
        for (object, ranges) in self.object_list.iter_mut().zip(bindings.ranges) {
            object.bind_external_superweapon_ranges(ranges);
        }
        self.external_superweapons = bindings.frozen;
    }
}

fn nullable_name(value: &NullableResourceName) -> Option<Option<String>> {
    match value {
        NullableResourceName::Inherit => None,
        NullableResourceName::Clear => Some(None),
        NullableResourceName::Named(name) => Some(Some(name.clone())),
    }
}

fn stock_type(source: &SdkSuperWeaponSource) -> SuperWeaponType {
    // SuperWeaponType::ReadINI6CEA20 uses the original ordered typed-reader
    // body. A later unsupported Type keeps its earlier stock kind; replacing
    // a final custom kind with NoEffect would lose that history.
    let mut stock = SuperWeaponType::from_ini_section_with_profile(
        &source.projected_id,
        source.body.as_ref().unwrap_or(IniSection::empty()),
        false,
    );
    if let Some(profile) = &source.profile {
        stock.weapon_type = profile.weapon_type;
        stock.aux_building = profile.aux_building.clone();
        stock.special_sound = profile.special_sound.clone();
        stock.start_sound = profile.start_sound.clone();
    }
    stock
}
