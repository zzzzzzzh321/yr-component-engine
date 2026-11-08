//! Read-only native INI readers, canonical registries and the shared damage
//! kernel. No feature-specific configuration interpretation belongs here.
use crate::rules::ini_parser::IniFile;
use std::sync::{Arc, OnceLock};
use vera_mod_api as api;
use vera_mod_runtime::HostServices;

/// Early recipes have no source, configuration, numeric or simulation services.
/// Their only input is the closed stage context supplied in the request.
pub(super) struct StartupHost;

impl HostServices for StartupHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        None
    }
    fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
        Err("configuration is unavailable to startup recipes".into())
    }
    fn registry(&self, _: &str) -> Vec<String> {
        Vec::new()
    }
    fn damage_kernel(&self, _: &api::DamageKernelInput) -> Result<i32, String> {
        Err("numeric services are unavailable to startup recipes".into())
    }
}

/// A physical-source query may read only bounded native names in the supplied
/// sections. All parsing remains in NativeHost's canonical INI reader.
pub(super) struct IniNamesHost {
    host: NativeHost,
    sections: Vec<String>,
    reader: String,
}

impl IniNamesHost {
    pub(super) fn new(ini: IniFile, sections: Vec<String>, capacity: usize) -> Self {
        Self {
            host: NativeHost::configuration(ini, Vec::new()),
            sections,
            reader: format!("name:{capacity}"),
        }
    }
}

impl HostServices for IniNamesHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        None
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader != self.reader
            || !self
                .sections
                .iter()
                .any(|id| id.eq_ignore_ascii_case(section))
        {
            return Err("INI name query exceeds its selected source scope".into());
        }
        self.host.config_read(section, key, reader, input)
    }

    fn registry(&self, _: &str) -> Vec<String> {
        Vec::new()
    }

    fn damage_kernel(&self, _: &api::DamageKernelInput) -> Result<i32, String> {
        Err("numeric services are unavailable to INI name queries".into())
    }
}

pub(super) struct NativeHost {
    ini: Option<Arc<IniFile>>,
    weapons: Vec<String>,
    registries: Option<Arc<crate::rules::native_processing::SdkRegistryPass>>,
    /// Derived bytes of the immutable per-call registry. The same payload is
    /// counted for bounded decode work and returned by the bulk reader.
    registry_message: OnceLock<Result<Vec<u8>, String>>,
    fixed_art_inputs: bool,
    fixed_art_message: OnceLock<Result<Vec<u8>, String>>,
    cameo_inputs: bool,
    cameo_message: OnceLock<Result<Vec<u8>, String>>,
    type_sound_inputs: bool,
    faction_inputs: bool,
    faction_message: OnceLock<Result<Vec<u8>, String>>,
    superweapon_inputs: bool,
    superweapon_message: OnceLock<Result<Vec<u8>, String>>,
    weapon_inputs: bool,
    projectile_inputs: bool,
}

impl NativeHost {
    pub(super) fn configuration(ini: IniFile, weapons: Vec<String>) -> Self {
        Self {
            ini: Some(Arc::new(ini)),
            weapons,
            registries: None,
            registry_message: OnceLock::new(),
            fixed_art_inputs: false,
            fixed_art_message: OnceLock::new(),
            cameo_inputs: false,
            cameo_message: OnceLock::new(),
            type_sound_inputs: false,
            faction_inputs: false,
            faction_message: OnceLock::new(),
            superweapon_inputs: false,
            superweapon_message: OnceLock::new(),
            weapon_inputs: false,
            projectile_inputs: false,
        }
    }

    pub(super) fn with_registries(
        mut self,
        registries: crate::rules::native_processing::SdkRegistryPass,
    ) -> Self {
        self.registries = Some(Arc::new(registries));
        self.registry_message = OnceLock::new();
        self.fixed_art_message = OnceLock::new();
        self.cameo_message = OnceLock::new();
        self.faction_message = OnceLock::new();
        self.superweapon_message = OnceLock::new();
        self
    }

    pub(super) fn enable_fixed_art_inputs(mut self) -> Self {
        self.fixed_art_inputs = true;
        self
    }

    pub(super) fn enable_cameo_inputs(mut self) -> Self {
        self.cameo_inputs = true;
        self
    }

    pub(super) fn enable_type_sound_inputs(mut self) -> Self {
        self.type_sound_inputs = true;
        self
    }

    pub(super) fn enable_faction_inputs(mut self) -> Self {
        self.faction_inputs = true;
        self
    }

    pub(super) fn enable_superweapon_inputs(mut self) -> Self {
        self.superweapon_inputs = true;
        self
    }

    pub(super) fn enable_weapon_inputs(mut self) -> Self {
        self.weapon_inputs = true;
        self
    }

    pub(super) fn enable_projectile_inputs(mut self) -> Self {
        self.projectile_inputs = true;
        self
    }

    pub(super) fn simulation() -> Self {
        Self {
            ini: None,
            weapons: Vec::new(),
            registries: None,
            registry_message: OnceLock::new(),
            fixed_art_inputs: false,
            fixed_art_message: OnceLock::new(),
            cameo_inputs: false,
            cameo_message: OnceLock::new(),
            type_sound_inputs: false,
            faction_inputs: false,
            faction_message: OnceLock::new(),
            superweapon_inputs: false,
            superweapon_message: OnceLock::new(),
            weapon_inputs: false,
            projectile_inputs: false,
        }
    }

    fn fixed_sounds(&self) -> Result<&crate::rules::sound_ini::SoundRegistry, String> {
        if !self.type_sound_inputs || self.ini.is_none() {
            return Err("fixed sound configuration is unavailable".into());
        }
        self.registries
            .as_ref()
            .and_then(|registry| registry.fixed_sound_source())
            .map(std::sync::Arc::as_ref)
            .ok_or_else(|| "missing process-selected sound catalog".into())
    }

    fn registry_view_bytes(&self) -> Result<&[u8], String> {
        self.registry_message
            .get_or_init(|| {
                let registry = self
                    .registries
                    .as_ref()
                    .ok_or("registry view requires physical Rules read-point data")?;
                serde_json::to_vec(registry.production_view()).map_err(|e| e.to_string())
            })
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }

    fn fixed_art_view_bytes(&self) -> Result<&[u8], String> {
        if !self.fixed_art_inputs || self.ini.is_none() {
            return Err("fixed ART configuration is unavailable".into());
        }
        self.fixed_art_message
            .get_or_init(|| {
                let registry = self
                    .registries
                    .as_ref()
                    .ok_or("missing fixed ART read points")?;
                registry
                    .fixed_art_source()
                    .ok_or("missing fixed ART source")?;
                serde_json::to_vec(registry.foundation_view()).map_err(|e| e.to_string())
            })
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }

    fn fixed_art_for(&self, reference: &api::ImmutableIniSectionRef) -> Result<&IniFile, String> {
        reference.validate()?;
        if !(self.fixed_art_inputs || self.cameo_inputs) || self.ini.is_none() {
            return Err("fixed ART configuration is unavailable".into());
        }
        let registry = self
            .registries
            .as_ref()
            .ok_or("missing fixed ART read points")?;
        if reference.source != api::FixedIniSource::FixedArt
            || reference.source_hash != registry.fixed_art_hash()
        {
            return Err("immutable INI reference differs from fixed ART".into());
        }
        registry
            .fixed_art_source()
            .ok_or_else(|| "missing fixed ART source".into())
    }

    fn faction_view_bytes(&self) -> Result<&[u8], String> {
        if !self.faction_inputs || self.ini.is_none() {
            return Err("faction configuration is unavailable".into());
        }
        self.faction_message
            .get_or_init(|| {
                let registry = self
                    .registries
                    .as_ref()
                    .ok_or("missing faction read points")?;
                serde_json::to_vec(registry.faction_view()).map_err(|e| e.to_string())
            })
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }

    fn superweapon_view_bytes(&self) -> Result<&[u8], String> {
        if !self.superweapon_inputs || self.ini.is_none() {
            return Err("SuperWeapon configuration is unavailable".into());
        }
        self.superweapon_message
            .get_or_init(|| {
                let registry = self
                    .registries
                    .as_ref()
                    .ok_or("missing SuperWeapon read points")?;
                registry.superweapon_effective_input_bytes()?;
                serde_json::to_vec(registry.superweapon_view()).map_err(|e| e.to_string())
            })
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }

    fn superweapon_source_bytes(&self) -> usize {
        if !self.superweapon_inputs || self.ini.is_none() {
            return 0;
        }
        // An unencodable source is rejected by the view/read services. Until
        // then charge the capped maximum, rather than silently treating it as
        // an empty configuration and granting a smaller execution budget.
        self.registries.as_ref().map_or(usize::MAX, |registry| {
            registry
                .superweapon_effective_input_bytes()
                .unwrap_or(usize::MAX)
        })
    }

    fn retained_resource_source_bytes(&self) -> usize {
        if self.ini.is_none() {
            return 0;
        }
        [
            (self.weapon_inputs, "$weapon-config"),
            (self.projectile_inputs, "$projectile-config"),
        ]
        .into_iter()
        .filter(|(enabled, _)| *enabled)
        .fold(0usize, |size, (_, family)| {
            size.saturating_add(self.registries.as_ref().map_or(0, |registry| {
                registry
                    .resource_effective_input_bytes(family)
                    .unwrap_or(usize::MAX)
            }))
        })
    }

    fn cameo_view_bytes(&self) -> Result<&[u8], String> {
        if !self.cameo_inputs || self.ini.is_none() {
            return Err("cameo configuration is unavailable".into());
        }
        self.cameo_message
            .get_or_init(|| {
                let registry = self
                    .registries
                    .as_ref()
                    .ok_or("missing cameo read points")?;
                serde_json::to_vec(registry.cameo_view()).map_err(|e| e.to_string())
            })
            .as_ref()
            .map(Vec::as_slice)
            .map_err(Clone::clone)
    }
}

impl HostServices for NativeHost {
    fn configuration_input_bytes(&self) -> usize {
        self.registry_view_bytes()
            .map_or(0, <[u8]>::len)
            .saturating_add(self.fixed_art_view_bytes().map_or(0, <[u8]>::len))
            .saturating_add(self.cameo_view_bytes().map_or(0, <[u8]>::len))
            .saturating_add(self.superweapon_view_bytes().map_or(0, <[u8]>::len))
            .saturating_add(self.superweapon_source_bytes())
            .saturating_add(self.retained_resource_source_bytes())
    }

    fn config(&self, section: &str, key: &str) -> Option<String> {
        let section = self.ini.as_ref()?.section(section)?;
        section
            .is_present(key)
            .then(|| section.read_string(key, "", 4096))
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "sound-source-hash" {
            return serde_json::to_vec(&self.fixed_sounds()?.source_hash())
                .map_err(|e| e.to_string());
        }
        if matches!(reader, "eva-source-hash" | "eva-find") {
            if !self.superweapon_inputs || self.ini.is_none() {
                return Err("fixed EVA configuration is unavailable".into());
            }
            let eva = self
                .registries
                .as_ref()
                .and_then(|registry| registry.superweapon_eva_source())
                .ok_or("missing process-selected EVA catalog")?;
            if reader == "eva-source-hash" {
                return serde_json::to_vec(&eva.source_hash()).map_err(|e| e.to_string());
            }
            let name: String = serde_json::from_slice(current).map_err(|e| e.to_string())?;
            if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
                return Err("invalid fixed EVA lookup name".into());
            }
            return serde_json::to_vec(&eva.entry(&name).map(|entry| &entry.name))
                .map_err(|e| e.to_string());
        }
        if reader == "sound-reference" {
            let sounds = self.fixed_sounds()?;
            let current: api::NullableResourceName =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            let ini = self
                .ini
                .as_ref()
                .expect("fixed_sounds checks configuration");
            // NullableIdx<Voc>1000E9D0 remains the unique native reader;
            // unknown/missing retains the previous three-state declaration.
            let value = match sounds.read_ares_reference(ini.section_or_empty(section), key) {
                None => current,
                Some(None) => api::NullableResourceName::Clear,
                Some(Some(name)) => api::NullableResourceName::Named(name),
            };
            return serde_json::to_vec(&value).map_err(|e| e.to_string());
        }
        if reader == "rules-registry-view" {
            return self.registry_view_bytes().map(<[u8]>::to_vec);
        }
        if reader == "fixed-art-view" {
            return self.fixed_art_view_bytes().map(<[u8]>::to_vec);
        }
        if reader == "cameo-registry-view" {
            return self.cameo_view_bytes().map(<[u8]>::to_vec);
        }
        if reader == "faction-registry-view" {
            return self.faction_view_bytes().map(<[u8]>::to_vec);
        }
        if reader == "superweapon-registry-view" {
            return self.superweapon_view_bytes().map(<[u8]>::to_vec);
        }
        if reader.starts_with("super-") && !reader.starts_with("super-effective:") {
            if !self.superweapon_inputs {
                return Err("component has no SuperWeapon configuration capability".into());
            }
            let ini = self
                .ini
                .as_ref()
                .ok_or("SuperWeapon input is unavailable during simulation")?;
            return self
                .registries
                .as_ref()
                .ok_or("missing SuperWeapon read points")?
                .read_superweapon_value(
                    reader,
                    section,
                    ini.section_or_empty(section),
                    key,
                    current,
                );
        }
        if reader.starts_with("faction-") {
            if !self.faction_inputs {
                return Err("component has no faction configuration capability".into());
            }
            let ini = self
                .ini
                .as_ref()
                .ok_or("faction input is unavailable during simulation")?;
            return self
                .registries
                .as_ref()
                .ok_or("missing faction read points")?
                .read_faction_value(reader, section, ini.section_or_empty(section), key, current);
        }
        if let Some(capacity) = reader.strip_prefix("source-string:") {
            let capacity = capacity.parse::<usize>().map_err(|e| e.to_string())?;
            if !(1..=4096).contains(&capacity) {
                return Err("invalid fixed-source string capacity".into());
            }
            let input: api::ImmutableIniValue<String> =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            let ini = self.fixed_art_for(&input.reference)?;
            let value = ini.section_or_empty(&input.reference.section).read_string(
                key,
                &input.current,
                capacity,
            );
            return serde_json::to_vec(&value).map_err(|e| e.to_string());
        }
        if reader == "source-pcx-filename" {
            if !self.cameo_inputs {
                return Err("component has no cameo source capability".into());
            }
            let input: api::ImmutableIniValue<Option<String>> =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            let ini = self.fixed_art_for(&input.reference)?;
            let value = ini
                .section_or_empty(&input.reference.section)
                .read_ares_pcx_filename(key);
            return serde_json::to_vec(&value).map_err(|e| e.to_string());
        }
        if reader == "foundation-stock" {
            if !self.fixed_art_inputs {
                return Err("component has no foundation source capability".into());
            }
            let input: api::ImmutableIniValue<u8> =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            if key != "Foundation" || input.current >= api::STOCK_FOUNDATION_COUNT {
                return Err("invalid stock foundation reader input".into());
            }
            let ini = self.fixed_art_for(&input.reference)?;
            let value = crate::rules::foundation::read_building_foundation(
                input.current,
                &input.reference.section,
                section,
                |name, current| {
                    crate::rules::foundation::read_foundation(ini.section_or_empty(name), current)
                },
            );
            return serde_json::to_vec(&value).map_err(|e| e.to_string());
        }
        if reader == "verse-token" {
            let token: String = serde_json::from_slice(current).map_err(|e| e.to_string())?;
            return serde_json::to_vec(&crate::rules::ares_armor::parse_verse_token(&token))
                .map_err(|e| e.to_string());
        }
        let ini = self
            .ini
            .as_ref()
            .ok_or("configuration is unavailable during simulation")?;
        let owner = section;
        let (section, reader, value) = if let Some(reader) = reader.strip_prefix("super-effective:")
        {
            if !self.superweapon_inputs {
                return Err("component has no SuperWeapon configuration capability".into());
            }
            // Only scalar native readers may use this retained physical source.
            // Registry, fixed-source and recursive adapters cannot be smuggled
            // through it; they have their own source authority above.
            if !matches!(
                reader,
                "presence"
                    | "int"
                    | "ares-int"
                    | "bool"
                    | "ares-bool"
                    | "double-bits"
                    | "ares-double-bits"
            ) && !reader.starts_with("name:")
                && !reader.starts_with("string:")
            {
                return Err("unsupported retained SuperWeapon scalar reader".into());
            }
            let input: api::SuperReadValue<serde_json::Value> =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            let section = self
                .registries
                .as_ref()
                .ok_or("missing SuperWeapon read points")?
                .superweapon_effective_section(input.read, owner)?;
            (section, reader, input.current)
        } else if let Some((family, enabled, reader)) = reader
            .strip_prefix("weapon-effective:")
            .map(|reader| ("$weapon-config", self.weapon_inputs, reader))
            .or_else(|| {
                reader
                    .strip_prefix("projectile-effective:")
                    .map(|reader| ("$projectile-config", self.projectile_inputs, reader))
            })
        {
            if !enabled {
                return Err("component has no retained resource configuration capability".into());
            }
            // Select this native family's actual body, then delegate scalar
            // conversion to the single canonical reader below.
            if !matches!(
                reader,
                "presence"
                    | "int"
                    | "ares-int"
                    | "bool"
                    | "ares-bool"
                    | "ares-nullable-bool"
                    | "nullable-int"
                    | "ares-nullable-int"
                    | "double-bits"
                    | "ares-double-bits"
                    | "ares-leptons"
                    | "ares-nullable-leptons"
            ) && !reader.starts_with("name:")
                && !reader.starts_with("string:")
            {
                return Err("unsupported retained resource scalar reader".into());
            }
            let section = self
                .registries
                .as_ref()
                .ok_or("missing retained resource source")?
                .resource_effective_section(family, owner)?;
            (
                section,
                reader,
                serde_json::from_slice(current).map_err(|e| e.to_string())?,
            )
        } else {
            (
                ini.section_or_empty(section),
                reader,
                serde_json::from_slice(current).map_err(|e| e.to_string())?,
            )
        };
        let bad = || format!("invalid config reader/default: {reader}");
        let result = match reader {
            "presence" => serde_json::json!(section.is_present(key)),
            "comma-mask:2048" => {
                let tokens: Vec<(String, u32)> =
                    serde_json::from_value(value).map_err(|e| e.to_string())?;
                if tokens.len() > 256
                    || tokens.iter().any(|(name, _)| {
                        name.is_empty() || name.len() > 128 || name.chars().any(char::is_control)
                    })
                {
                    return Err("invalid comma-mask token table".into());
                }
                serde_json::json!(section.read_comma_mask(key, &tokens))
            }
            "pcx-filename" => serde_json::json!(section.read_ares_pcx_filename(key)),
            "bool-prefix-reverse3" => serde_json::json!(section.read_bool_prefix_reverse3(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?,
            )),
            "int" => serde_json::json!(
                section.read_int(
                    key,
                    value
                        .as_i64()
                        .and_then(|x| i32::try_from(x).ok())
                        .ok_or_else(bad)?
                )
            ),
            "ares-int" => serde_json::json!(
                section.read_ares_int(
                    key,
                    value
                        .as_i64()
                        .and_then(|x| i32::try_from(x).ok())
                        .ok_or_else(bad)?,
                )
            ),
            "ares-emp-threshold" => serde_json::json!(section.read_ares_emp_threshold(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?,
            )),
            "ares-nullable-mission" => {
                use crate::rules::mission_data::MissionType;
                let current: Option<u16> =
                    serde_json::from_value(value).map_err(|e| e.to_string())?;
                let current = current
                    .map(|id| {
                        u8::try_from(id)
                            .ok()
                            .and_then(MissionType::from_id)
                            .ok_or_else(bad)
                    })
                    .transpose()?;
                serde_json::json!(
                    section
                        .read_ares_nullable_mission(key, current)
                        .map(|mission| mission as u16)
                )
            }
            "bool" => serde_json::json!(section.read_bool(key, value.as_bool().ok_or_else(bad)?)),
            "ares-bool" => {
                serde_json::json!(section.read_ares_bool(key, value.as_bool().ok_or_else(bad)?))
            }
            "double-bits" => serde_json::json!(
                section
                    .read_double(key, f64::from_bits(value.as_u64().ok_or_else(bad)?))
                    .to_bits()
            ),
            "ares-leptons" => serde_json::json!(section.read_ares_leptons(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "ares-nullable-leptons" => serde_json::json!(section.read_ares_nullable_leptons(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "ares-double-bits" => serde_json::json!(
                section
                    .read_ares_double_bits(
                        key,
                        crate::util::native_x87::NativeF64Bits::from_bits(
                            value.as_u64().ok_or_else(bad)?,
                        ),
                    )
                    .bits()
            ),
            "nullable-int" => serde_json::json!(section.read_phobos_nullable_int(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "ares-nullable-int" => serde_json::json!(section.read_ares_nullable_int(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "nullable-bool" => serde_json::json!(section.read_phobos_nullable_bool(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "ares-nullable-bool" => serde_json::json!(section.read_ares_nullable_bool(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "nullable-double-bits" => serde_json::json!(
                section
                    .read_phobos_nullable_double(
                        key,
                        serde_json::from_value::<Option<u64>>(value)
                            .map_err(|e| e.to_string())?
                            .map(f64::from_bits)
                    )
                    .map(f64::to_bits)
            ),
            "int-list" => section
                .read_int_list(key)
                .map_or(value, |x| serde_json::json!(x)),
            "color-rgb" => {
                let old: [u8; 3] = serde_json::from_value(value).map_err(|e| e.to_string())?;
                serde_json::json!(section.read_color_rgb(key, old, None))
            }
            _ if reader.starts_with("ares-ref:") || reader.starts_with("ares-nullable-ref:") => {
                let (nullable, spec) = if let Some(spec) = reader.strip_prefix("ares-nullable-ref:")
                {
                    (true, spec)
                } else {
                    (false, &reader["ares-ref:".len()..])
                };
                let (target, scope) = spec.split_once('@').ok_or_else(bad)?;
                if target != "$animations" && !(target == "$technos" && !nullable) {
                    return Err(bad());
                }
                let registries = self
                    .registries
                    .as_ref()
                    .ok_or("registered references require physical Rules read-point data")?;
                if target == "$technos" {
                    serde_json::json!(registries.read_techno_reference(
                        owner,
                        &section,
                        key,
                        scope,
                        serde_json::from_value(value).map_err(|e| e.to_string())?,
                    )?)
                } else if nullable {
                    serde_json::json!(registries.read_nullable_animation_reference(
                        owner,
                        &section,
                        key,
                        scope,
                        serde_json::from_value(value).map_err(|e| e.to_string())?,
                    )?)
                } else {
                    serde_json::json!(registries.read_animation_reference(
                        owner,
                        &section,
                        key,
                        scope,
                        serde_json::from_value(value).map_err(|e| e.to_string())?,
                    )?)
                }
            }
            _ => {
                let (kind, size) = reader.split_once(':').ok_or_else(bad)?;
                let size: usize = size.parse().map_err(|_| bad())?;
                if !(1..=4096).contains(&size) {
                    return Err(bad());
                }
                match kind {
                    "string" => serde_json::json!(section.read_string(
                        key,
                        value.as_str().ok_or_else(bad)?,
                        size
                    )),
                    "name" => section
                        .read_name(key, size)
                        .map_or(value, |x| serde_json::json!(x)),
                    "nullable-name" => serde_json::json!(
                        section.read_phobos_nullable_with(
                            key,
                            serde_json::from_value::<Option<String>>(value)
                                .map_err(|e| e.to_string())?,
                            |raw| { Some(section.read_string(key, raw, size)) }
                        )
                    ),
                    "list" => {
                        if section.is_present(key) {
                            let raw = section.read_string(key, "", size);
                            serde_json::json!(
                                crate::rules::ini_value::strtok(&raw, &[',', ' ', '\t'])
                                    .map(str::to_owned)
                                    .collect::<Vec<_>>()
                            )
                        } else {
                            value
                        }
                    }
                    "comma-list" => section
                        .read_list(key, size)
                        .map_or(value, |items| serde_json::json!(items)),
                    _ => return Err(bad()),
                }
            }
        };
        serde_json::to_vec(&result).map_err(|e| e.to_string())
    }

    fn registry(&self, name: &str) -> Vec<String> {
        if matches!(name, "$weapon-config" | "$projectile-config") {
            let enabled = if name == "$weapon-config" {
                self.weapon_inputs
            } else {
                self.projectile_inputs
            };
            return if enabled && self.ini.is_some() {
                self.registries
                    .as_ref()
                    .map_or_else(Vec::new, |r| r.names(name))
            } else {
                Vec::new()
            };
        }
        if name == "$sounds" {
            return if self.fixed_sounds().is_ok() {
                vec!["FixedSounds".into()]
            } else {
                Vec::new()
            };
        }
        if name == "$cameos" {
            return if self.cameo_inputs && self.ini.is_some() && self.registries.is_some() {
                vec!["Cameos".into()]
            } else {
                Vec::new()
            };
        }
        if name == "$fixed-art" {
            return if self.fixed_art_inputs
                && self.ini.is_some()
                && self
                    .registries
                    .as_ref()
                    .is_some_and(|r| r.fixed_art_source().is_some())
            {
                vec!["FixedArt".into()]
            } else {
                Vec::new()
            };
        }
        if name == "$weapons" {
            return self.weapons.clone();
        }
        if name.starts_with('$') {
            return self
                .registries
                .as_ref()
                .map_or_else(Vec::new, |r| r.names(name));
        }
        self.ini
            .as_ref()
            .and_then(|ini| ini.section(name))
            .map(|s| s.keys().map(str::to_owned).collect())
            .unwrap_or_default()
    }

    fn damage_kernel(&self, input: &api::DamageKernelInput) -> Result<i32, String> {
        Ok(crate::sim::combat::damage::kernel::apply_warhead_damage(
            input.damage,
            f64::from_bits(input.cell_spread_bits),
            f64::from_bits(input.percent_at_max_bits),
            crate::util::native_x87::NativeF64Bits::from_bits(input.multiplier_bits),
            input.distance_leptons,
            input.no_damage,
            input.max_damage,
        ))
    }
}
