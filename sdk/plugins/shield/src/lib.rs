//! Reference gameplay component. Shield configuration, state, and decisions live here.
//!
//! Moved from VERA20K's Rust `sim/shield.rs` and `rules/phobos_shields.rs`.
//! Native references remain beside their corresponding behavior. No Ares or
//! Phobos C++ source or proprietary assets are incorporated.
mod config;
mod host;
mod logic;

use config::{ShieldGlobals, ShieldTypeCatalog, ShieldWarheadRules};
use host::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{API_VERSION, Context, HookKind, ModuleManifest, Plan, Request};

#[cfg(target_arch = "wasm32")]
mod bindings {
    wit_bindgen::generate!({ path: "../../wit", world: "gameplay" });
}

#[derive(Default, Serialize, Deserialize)]
struct Configuration {
    catalog: ShieldTypeCatalog,
    globals: ShieldGlobals,
    objects: BTreeMap<String, String>,
    warheads: BTreeMap<String, ShieldWarheadRules>,
    overload: OverloadConfig,
}

#[derive(Serialize, Deserialize)]
struct OverloadConfig {
    limit: i32,
    duration: i32,
    cooling: i32,
}

impl Default for OverloadConfig {
    fn default() -> Self {
        Self {
            limit: 60,
            duration: 75,
            cooling: 1,
        }
    }
}

pub fn manifest() -> ModuleManifest {
    ModuleManifest {
        api_version: API_VERSION,
        id: if cfg!(feature = "overload") {
            "vera.overload-shield"
        } else {
            "vera.compat-shield"
        }
        .into(),
        version: "0.1.0".into(),
        state_version: 1,
        config_version: 1,
        hooks: vec![
            HookKind::Configure,
            HookKind::Attach,
            HookKind::Damage,
            HookKind::Tick,
        ],
    }
}

fn decode_config(bytes: &[u8]) -> Result<Configuration, String> {
    if bytes.is_empty() {
        Ok(Configuration::default())
    } else {
        serde_json::from_slice(bytes).map_err(|e| format!("component configuration: {e}"))
    }
}

fn evaluate<H: Host>(request: &Request, host: &H) -> Result<Plan, String> {
    request.validate()?;
    let mut config = decode_config(&request.config)?;
    if let Context::Configure(context) = &request.context {
        let view = ConfigView::new(host);
        if context.final_pass {
            config.globals = ShieldGlobals::from_ini(&view);
            config.objects.clear();
            for name in &context.object_types {
                if let Some(binding) = view.section_or_empty(name).read_name("ShieldType", 0x80)
                    && config.catalog.entry(&binding).is_some()
                {
                    config.objects.insert(name.to_ascii_uppercase(), binding);
                }
            }
            config.warheads.clear();
            for name in &context.warheads {
                let rules = ShieldWarheadRules::from_section(&view.section_or_empty(name));
                // Missing entries resolve to this same default in damage_stage.
                // Keep only semantic overrides in every entity's config message.
                if rules != ShieldWarheadRules::default() {
                    config.warheads.insert(name.to_ascii_uppercase(), rules);
                }
            }
            if cfg!(feature = "overload") {
                let section = view.section_or_empty("VeraOverload");
                config.overload.limit = section.read_int("HeatLimit", config.overload.limit).max(1);
                config.overload.duration = section
                    .read_int("Duration", config.overload.duration)
                    .max(1);
                config.overload.cooling =
                    section.read_int("Cooling", config.overload.cooling).max(0);
            }
        } else {
            config.catalog.apply_pass(&view);
        }
        view.finish()?;
        let mut plan = Plan::unchanged(request);
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    logic::evaluate(request, &config, host)
}

#[cfg(target_arch = "wasm32")]
struct Component;

#[cfg(target_arch = "wasm32")]
impl bindings::Guest for Component {
    fn evaluate(kind: String, input: Vec<u8>) -> Result<Vec<u8>, String> {
        if kind == "manifest" {
            return serde_json::to_vec(&manifest()).map_err(|e| e.to_string());
        }
        let request: Request = serde_json::from_slice(&input).map_err(|e| e.to_string())?;
        if kind != request.context.kind().as_str() {
            return Err("hook/context mismatch".into());
        }
        let plan = evaluate(&request, &host::WasmHost)?;
        serde_json::to_vec(&plan).map_err(|e| e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
bindings::export!(Component with_types_in bindings);

mod float_bits {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(value.to_bits())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
        u64::deserialize(deserializer).map(f64::from_bits)
    }
}

mod optional_float_bits {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error> {
        value.map(f64::to_bits).serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<f64>, D::Error> {
        Option::<u64>::deserialize(deserializer).map(|value| value.map(f64::from_bits))
    }
}

#[cfg(test)]
mod tests;
