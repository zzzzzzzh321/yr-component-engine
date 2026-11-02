//! Read-only native INI readers, canonical registries and the shared damage
//! kernel. No feature-specific configuration interpretation belongs here.
use crate::rules::ini_parser::IniFile;
use std::sync::Arc;
use vera_mod_api as api;
use vera_mod_runtime::HostServices;

pub(super) struct NativeHost {
    ini: Option<Arc<IniFile>>,
    weapons: Vec<String>,
}

impl NativeHost {
    pub(super) fn configuration(ini: IniFile, weapons: Vec<String>) -> Self {
        Self {
            ini: Some(Arc::new(ini)),
            weapons,
        }
    }

    pub(super) fn simulation() -> Self {
        Self {
            ini: None,
            weapons: Vec::new(),
        }
    }
}

impl HostServices for NativeHost {
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
        let ini = self
            .ini
            .as_ref()
            .ok_or("configuration is unavailable during simulation")?;
        let section = ini.section_or_empty(section);
        let value: serde_json::Value =
            serde_json::from_slice(current).map_err(|e| e.to_string())?;
        let bad = || format!("invalid config reader/default: {reader}");
        let result = match reader {
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
            "bool" => serde_json::json!(section.read_bool(key, value.as_bool().ok_or_else(bad)?)),
            "ares-bool" => {
                serde_json::json!(section.read_ares_bool(key, value.as_bool().ok_or_else(bad)?))
            }
            "double-bits" => serde_json::json!(
                section
                    .read_double(key, f64::from_bits(value.as_u64().ok_or_else(bad)?))
                    .to_bits()
            ),
            "nullable-int" => serde_json::json!(section.read_phobos_nullable_int(
                key,
                serde_json::from_value(value).map_err(|e| e.to_string())?
            )),
            "nullable-bool" => serde_json::json!(section.read_phobos_nullable_bool(
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
