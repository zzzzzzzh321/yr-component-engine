//! Read-only native INI readers, canonical registries and the shared damage
//! kernel. No feature-specific configuration interpretation belongs here.
use crate::rules::ini_parser::IniFile;
use std::sync::{Arc, OnceLock};
use vera_mod_api as api;
use vera_mod_runtime::HostServices;

pub(super) struct NativeHost {
    ini: Option<Arc<IniFile>>,
    weapons: Vec<String>,
    registries: Option<Arc<crate::rules::native_processing::SdkRegistryPass>>,
    /// Derived bytes of the immutable per-call registry. The same payload is
    /// counted for bounded decode work and returned by the bulk reader.
    registry_message: OnceLock<Result<Vec<u8>, String>>,
}

impl NativeHost {
    pub(super) fn configuration(ini: IniFile, weapons: Vec<String>) -> Self {
        Self {
            ini: Some(Arc::new(ini)),
            weapons,
            registries: None,
            registry_message: OnceLock::new(),
        }
    }

    pub(super) fn with_registries(
        mut self,
        registries: crate::rules::native_processing::SdkRegistryPass,
    ) -> Self {
        self.registries = Some(Arc::new(registries));
        self.registry_message = OnceLock::new();
        self
    }

    pub(super) fn simulation() -> Self {
        Self {
            ini: None,
            weapons: Vec::new(),
            registries: None,
            registry_message: OnceLock::new(),
        }
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
}

impl HostServices for NativeHost {
    fn configuration_input_bytes(&self) -> usize {
        self.registry_view_bytes().map_or(0, <[u8]>::len)
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
        if reader == "rules-registry-view" {
            return self.registry_view_bytes().map(<[u8]>::to_vec);
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
        let section = ini.section_or_empty(section);
        let value: serde_json::Value =
            serde_json::from_slice(current).map_err(|e| e.to_string())?;
        let bad = || format!("invalid config reader/default: {reader}");
        let result = match reader {
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
