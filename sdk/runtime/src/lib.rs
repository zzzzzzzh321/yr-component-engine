//! Stateless Component Model execution. No WASI, filesystem, clock, random,
//! networking or mutable Simulation access is linked into a component.
//! Compiled code is cached; every invocation receives a new Store and instance.

use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use vera_mod_api::{
    API_VERSION, DamageKernelInput, MAX_MESSAGE_BYTES, ModuleManifest, Plan, Request,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};

#[cfg(test)]
mod tests;

mod bindings {
    wasmtime::component::bindgen!({
        path: "../wit",
        world: "gameplay",
        imports: { default: trappable },
    });
}

/// A read-only, frozen host view for one evaluation. Engine adapters must use
/// the existing typed INI readers and damage kernel, not duplicate their math.
pub trait HostServices: Send + Sync {
    /// Additional frozen input exposed out of band during configuration, for
    /// example a physical registry snapshot. It shares the existing capped
    /// input allowance and never increases gameplay-call budgets.
    fn configuration_input_bytes(&self) -> usize {
        0
    }
    fn config(&self, section: &str, key: &str) -> Option<String>;
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String>;
    fn registry(&self, section: &str) -> Vec<String>;
    fn damage_kernel(&self, input: &DamageKernelInput) -> Result<i32, String>;
}

#[derive(Debug, Clone)]
pub struct RuntimeLimits {
    /// Base budget for manifest, immutable resource and gameplay queries.
    pub fuel: u64,
    /// A physical Rules pass initializes a whole catalog once, rather than
    /// updating one actor. It has a separate, still bounded startup budget.
    pub configure_fuel: u64,
    /// Additional shared budget per input byte, including frozen host metadata
    /// for configuration. JSON data costs guest instructions to decode too.
    /// Zero retains a fixed per-call budget; the extra allowance is capped.
    pub fuel_per_input_byte: u64,
    pub max_input_fuel: u64,
    pub memory_bytes: usize,
    pub component_bytes: usize,
    pub host_calls: u32,
    pub cache_entries: usize,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            fuel: 5_000_000,
            configure_fuel: 50_000_000,
            fuel_per_input_byte: 256,
            max_input_fuel: 50_000_000,
            memory_bytes: 32 * 1024 * 1024,
            component_bytes: 16 * 1024 * 1024,
            host_calls: 65_536,
            cache_entries: 8,
        }
    }
}

impl RuntimeLimits {
    fn fuel_for(&self, kind: &str, input_bytes: usize) -> u64 {
        let base = if matches!(kind, "configure" | "configure_boundary") {
            self.configure_fuel
        } else {
            self.fuel
        };
        let input_fuel = self
            .fuel_per_input_byte
            .saturating_mul(input_bytes as u64)
            .min(self.max_input_fuel);
        base.saturating_add(input_fuel)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("invalid gameplay component: {0}")]
    Invalid(String),
    #[error("gameplay component execution failed: {0}")]
    Execution(String),
    #[error("gameplay component rejected evaluation: {0}")]
    Rejected(String),
}

#[derive(Clone)]
pub struct LoadedModule {
    component: Component,
    manifest: ModuleManifest,
    sha256: String,
    engine_token: Arc<()>,
}

impl LoadedModule {
    pub fn manifest(&self) -> &ModuleManifest {
        &self.manifest
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

pub struct ComponentRuntime {
    engine: Engine,
    linker: Linker<HostState>,
    limits: RuntimeLimits,
    components: Mutex<BTreeMap<String, Component>>,
    engine_token: Arc<()>,
}

struct HostState {
    services: Arc<dyn HostServices>,
    limits: StoreLimits,
    calls_left: u32,
}

impl HostState {
    fn charge(&mut self) -> wasmtime::Result<()> {
        self.calls_left = self
            .calls_left
            .checked_sub(1)
            .ok_or_else(|| wasmtime::Error::msg("component host-call budget exhausted"))?;
        Ok(())
    }
}

fn host_name(name: &str) -> wasmtime::Result<()> {
    if name.len() > 256 || name.chars().any(char::is_control) {
        return Err(wasmtime::Error::msg(
            "component host name exceeds its limit",
        ));
    }
    Ok(())
}

impl bindings::vera::mod_sdk::host::Host for HostState {
    fn config(&mut self, section: String, key: String) -> wasmtime::Result<Option<String>> {
        self.charge()?;
        host_name(&section)?;
        host_name(&key)?;
        let result = self.services.config(&section, &key);
        if result.as_ref().is_some_and(|v| v.len() > 65_536) {
            return Err(wasmtime::Error::msg(
                "component config value exceeds its limit",
            ));
        }
        Ok(result)
    }
    fn config_read(
        &mut self,
        section: String,
        key: String,
        reader: String,
        current: Vec<u8>,
    ) -> wasmtime::Result<Result<Vec<u8>, String>> {
        self.charge()?;
        host_name(&section)?;
        host_name(&key)?;
        host_name(&reader)?;
        if current.len() > 65_536 {
            return Err(wasmtime::Error::msg(
                "component reader input exceeds its limit",
            ));
        }
        let result = self.services.config_read(&section, &key, &reader, &current);
        // A physical registry snapshot is a bounded bulk message, not one
        // scalar INI value. Keep scalar readers at their original64KiB cap.
        let output_limit = if matches!(reader.as_str(), "rules-registry-view" | "fixed-art-view") {
            MAX_MESSAGE_BYTES
        } else {
            65_536
        };
        if result.as_ref().is_ok_and(|v| v.len() > output_limit)
            || result.as_ref().is_err_and(|e| e.len() > 4096)
        {
            return Err(wasmtime::Error::msg(
                "component reader output exceeds its limit",
            ));
        }
        Ok(result)
    }
    fn registry(&mut self, section: String) -> wasmtime::Result<Vec<String>> {
        self.charge()?;
        host_name(&section)?;
        let values = self.services.registry(&section);
        if values.len() > 16_384 || values.iter().any(|name| name.len() > 256) {
            return Err(wasmtime::Error::msg("component registry exceeds its limit"));
        }
        Ok(values)
    }
    fn damage_kernel(&mut self, input: Vec<u8>) -> wasmtime::Result<Result<Vec<u8>, String>> {
        self.charge()?;
        if input.len() > 4096 {
            return Err(wasmtime::Error::msg(
                "component kernel input exceeds its limit",
            ));
        }
        let input: DamageKernelInput = match serde_json::from_slice(&input) {
            Ok(v) => v,
            Err(_) => return Ok(Err("invalid damage-kernel input".into())),
        };
        let result = self.services.damage_kernel(&input).and_then(|value| {
            serde_json::to_vec(&value).map_err(|_| "cannot encode kernel result".into())
        });
        if result.as_ref().is_err_and(|error| error.len() > 4096) {
            return Err(wasmtime::Error::msg(
                "component kernel error exceeds its limit",
            ));
        }
        Ok(result)
    }
}

struct ManifestHost;
impl HostServices for ManifestHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        None
    }
    fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
        Err("manifest has no configuration".into())
    }
    fn registry(&self, _: &str) -> Vec<String> {
        Vec::new()
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        Err("manifest has no gameplay services".into())
    }
}

impl ComponentRuntime {
    pub fn new(limits: RuntimeLimits) -> Result<Self, RuntimeError> {
        if limits.fuel == 0
            || limits.configure_fuel == 0
            || limits.memory_bytes < 65_536
            || limits.component_bytes == 0
            || limits.host_calls == 0
            || limits.cache_entries == 0
        {
            return Err(RuntimeError::Invalid(
                "zero or insufficient runtime limits".into(),
            ));
        }
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .cranelift_nan_canonicalization(true);
        // The optional Wasmtime `threads` feature is not compiled into this crate.
        config
            .wasm_multi_memory(false)
            .wasm_memory64(false)
            .wasm_relaxed_simd(false);
        config.max_wasm_stack(512 * 1024);
        let engine = Engine::new(&config).map_err(|e| RuntimeError::Invalid(e.to_string()))?;
        let mut linker = Linker::new(&engine);
        bindings::Gameplay::add_to_linker::<_, wasmtime::component::HasSelf<_>>(
            &mut linker,
            |state: &mut HostState| state,
        )
        .map_err(|e| RuntimeError::Invalid(e.to_string()))?;
        Ok(Self {
            engine,
            linker,
            limits,
            components: Mutex::new(BTreeMap::new()),
            engine_token: Arc::new(()),
        })
    }

    /// Verify raw bytes before compiling. Never deserialize supplied native
    /// Wasmtime artifacts: only validated portable component bytes are accepted.
    pub fn load(&self, bytes: &[u8], expected_sha256: &str) -> Result<LoadedModule, RuntimeError> {
        if bytes.len() > self.limits.component_bytes || !bytes.starts_with(b"\0asm") {
            return Err(RuntimeError::Invalid(
                "invalid component size or binary header".into(),
            ));
        }
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        if sha256 != expected_sha256 {
            return Err(RuntimeError::Invalid(
                "component content digest mismatch".into(),
            ));
        }
        let mut cache = self
            .components
            .lock()
            .map_err(|_| RuntimeError::Invalid("component cache unavailable".into()))?;
        let component = if let Some(c) = cache.get(&sha256) {
            c.clone()
        } else {
            let c = Component::new(&self.engine, bytes)
                .map_err(|e| RuntimeError::Invalid(e.to_string()))?;
            if cache.len() >= self.limits.cache_entries {
                cache.pop_first();
            }
            cache.insert(sha256.clone(), c.clone());
            c
        };
        drop(cache);
        let output = self.call(&component, "manifest", b"{}", Arc::new(ManifestHost))?;
        let manifest: ModuleManifest = serde_json::from_slice(&output)
            .map_err(|_| RuntimeError::Invalid("invalid module manifest".into()))?;
        manifest.validate().map_err(RuntimeError::Invalid)?;
        Ok(LoadedModule {
            component,
            manifest,
            sha256,
            engine_token: Arc::clone(&self.engine_token),
        })
    }

    pub fn evaluate(
        &self,
        module: &LoadedModule,
        request: &Request,
        host: Arc<dyn HostServices>,
    ) -> Result<Plan, RuntimeError> {
        if !Arc::ptr_eq(&self.engine_token, &module.engine_token) {
            return Err(RuntimeError::Invalid(
                "module belongs to a different runtime".into(),
            ));
        }
        request.validate().map_err(RuntimeError::Invalid)?;
        let kind = request.context.kind();
        if module.manifest.api_version != API_VERSION || !module.manifest.hooks.contains(&kind) {
            return Err(RuntimeError::Invalid(
                "module did not declare this hook".into(),
            ));
        }
        let input = serde_json::to_vec(request)
            .map_err(|_| RuntimeError::Invalid("cannot encode component request".into()))?;
        let output = self.call(&module.component, kind.as_str(), &input, host)?;
        let plan: Plan = serde_json::from_slice(&output)
            .map_err(|_| RuntimeError::Invalid("invalid component change plan".into()))?;
        plan.validate_for(request).map_err(RuntimeError::Invalid)?;
        Ok(plan)
    }

    fn call(
        &self,
        component: &Component,
        kind: &str,
        input: &[u8],
        host: Arc<dyn HostServices>,
    ) -> Result<Vec<u8>, RuntimeError> {
        if input.len() > MAX_MESSAGE_BYTES {
            return Err(RuntimeError::Invalid(
                "component request exceeds its wire limit".into(),
            ));
        }
        let limits = StoreLimitsBuilder::new()
            .memory_size(self.limits.memory_bytes)
            .memories(1)
            .instances(8)
            .tables(4)
            .table_elements(65_536)
            .trap_on_grow_failure(true)
            .build();
        let configuration_bytes = if matches!(kind, "configure" | "configure_boundary") {
            host.configuration_input_bytes()
        } else {
            0
        };
        let mut store = Store::new(
            &self.engine,
            HostState {
                services: host,
                limits,
                calls_left: self.limits.host_calls,
            },
        );
        store.limiter(|state| &mut state.limits);
        let fuel = self
            .limits
            .fuel_for(kind, input.len().saturating_add(configuration_bytes));
        store
            .set_fuel(fuel)
            .map_err(|e| RuntimeError::Execution(format!("{e:#}")))?;
        let instance = bindings::Gameplay::instantiate(&mut store, component, &self.linker)
            .map_err(|e| RuntimeError::Execution(format!("{e:#}")))?;
        let output = instance
            .call_evaluate(&mut store, kind, input)
            .map_err(|e| {
                RuntimeError::Execution(format!(
                    "{kind} failed (input bytes {}, configuration metadata bytes {configuration_bytes}, fuel used {}/{fuel}, host calls {}): {e:#}",
                    input.len(),
                    fuel.saturating_sub(store.get_fuel().unwrap_or(0)),
                    self.limits
                        .host_calls
                        .saturating_sub(store.data().calls_left),
                ))
            })?
            .map_err(|e| RuntimeError::Rejected(e.chars().take(4096).collect()))?;
        if output.len() > MAX_MESSAGE_BYTES {
            return Err(RuntimeError::Invalid(
                "component result exceeds its wire limit".into(),
            ));
        }
        Ok(output)
    }
}
