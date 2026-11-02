//! Saved component identity and frozen configuration. The compiled executor is a
//! disposable cache; each evaluation still gets a fresh guest instance.
use super::host::NativeHost;
use crate::rules::ini_parser::IniFile;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};
use vera_mod_api::{self as api, Context, Plan, Request};
use vera_mod_runtime::{ComponentRuntime, LoadedModule, RuntimeLimits};

#[derive(Serialize, Deserialize)]
pub struct Program {
    bytes: Vec<u8>,
    digest: String,
    manifest: api::ModuleManifest,
    config: Vec<u8>,
    #[serde(skip)]
    compiled: OnceLock<Result<Arc<Executor>, String>>,
}

struct Executor {
    runtime: ComponentRuntime,
    module: LoadedModule,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("digest", &self.digest)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            digest: self.digest.clone(),
            manifest: self.manifest.clone(),
            config: self.config.clone(),
            compiled: self.compiled.clone(),
        }
    }
}

impl Program {
    pub fn load(bytes: Vec<u8>, expected: &str, passes: &[IniFile]) -> Result<Self, String> {
        Self::load_with_types(bytes, expected, passes, &[], &[], &[])
    }

    pub fn load_with_types(
        bytes: Vec<u8>,
        expected: &str,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
    ) -> Result<Self, String> {
        let runtime = ComponentRuntime::new(RuntimeLimits::default()).map_err(|e| e.to_string())?;
        let module = runtime.load(&bytes, expected).map_err(|e| e.to_string())?;
        let manifest = module.manifest().clone();
        if [
            api::HookKind::Configure,
            api::HookKind::Attach,
            api::HookKind::Damage,
            api::HookKind::Tick,
        ]
        .iter()
        .any(|kind| !manifest.hooks.contains(kind))
        {
            return Err("engine SDK v1 requires configure, attach, damage and tick hooks".into());
        }
        let mut program = Self {
            bytes,
            digest: expected.into(),
            manifest,
            config: Vec::new(),
            compiled: OnceLock::new(),
        };
        let executor = Arc::new(Executor { runtime, module });
        for (index, ini) in passes.iter().enumerate() {
            let final_pass = index + 1 == passes.len();
            let request = Request {
                api_version: api::API_VERSION,
                context: Context::Configure(api::ConfigureContext {
                    pass: index as u64,
                    source: format!("rules-pass-{index}"),
                    final_pass,
                    object_types: if final_pass {
                        object_types.to_vec()
                    } else {
                        Vec::new()
                    },
                    warheads: if final_pass {
                        warheads.to_vec()
                    } else {
                        Vec::new()
                    },
                }),
                config: program.config.clone(),
                state: Vec::new(),
            };
            let plan = executor
                .runtime
                .evaluate(
                    &executor.module,
                    &request,
                    Arc::new(NativeHost::configuration(
                        ini.clone(),
                        if final_pass {
                            weapons.to_vec()
                        } else {
                            Vec::new()
                        },
                    )),
                )
                .map_err(|e| e.to_string())?;
            if !plan.state.is_empty() || plan.damage.is_some() || !plan.operations.is_empty() {
                return Err("configuration returned simulation writes".into());
            }
            if let Some(config) = plan.config {
                program.config = config;
            }
        }
        program
            .compiled
            .set(Ok(executor))
            .map_err(|_| "component cache already set".to_string())?;
        Ok(program)
    }

    pub fn sha256(&self) -> &str {
        &self.digest
    }
    pub fn manifest(&self) -> &api::ModuleManifest {
        &self.manifest
    }

    pub(crate) fn weapon_resource_policy(
        &self,
        name: &str,
    ) -> Result<Option<api::WeaponResourcePolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::WeaponResource) {
            return Ok(None);
        }
        let plan = self.evaluate(
            Context::WeaponResource(api::WeaponResourceContext {
                weapon_name: name.to_owned(),
            }),
            &[],
        )?;
        Ok(plan.weapon_resource)
    }

    pub(crate) fn type_resource_policy(
        &self,
        name: &str,
        user: u8,
    ) -> Result<Option<api::TypeResourcePolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::TypeResource) {
            return Ok(None);
        }
        let plan = self.evaluate(
            Context::TypeResource(api::TypeResourceContext {
                type_name: name.to_owned(),
                user,
            }),
            &[],
        )?;
        Ok(plan.type_resource)
    }

    fn executor(&self) -> Result<&Arc<Executor>, String> {
        self.compiled
            .get_or_init(|| {
                let runtime =
                    ComponentRuntime::new(RuntimeLimits::default()).map_err(|e| e.to_string())?;
                let module = runtime
                    .load(&self.bytes, &self.digest)
                    .map_err(|e| e.to_string())?;
                if module.manifest() != &self.manifest {
                    return Err("saved module manifest mismatch".into());
                }
                Ok(Arc::new(Executor { runtime, module }))
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(crate) fn evaluate(&self, context: Context, state: &[u8]) -> Result<Plan, String> {
        let executor = self.executor()?;
        let request = Request {
            api_version: api::API_VERSION,
            context,
            config: self.config.clone(),
            state: state.to_vec(),
        };
        let plan = executor
            .runtime
            .evaluate(
                &executor.module,
                &request,
                Arc::new(NativeHost::simulation()),
            )
            .map_err(|e| e.to_string())?;
        if plan.config.is_some() || !plan.operations.is_empty() {
            return Err("unsupported operation or configuration write during simulation".into());
        }
        Ok(plan)
    }

    pub(crate) fn validate_restore(&self) -> Result<(), String> {
        if self.bytes.len() > 16 * 1024 * 1024 || self.config.len() > api::MAX_CONFIG_BYTES {
            return Err("saved component exceeds SDK limits".into());
        }
        self.executor().map(|_| ())
    }

    pub(crate) fn fold(&self, hasher: &mut impl Hasher) {
        self.digest.hash(hasher);
        self.manifest.id.hash(hasher);
        self.manifest.version.hash(hasher);
        self.manifest.state_version.hash(hasher);
        self.manifest.config_version.hash(hasher);
        self.config.hash(hasher);
    }
}
