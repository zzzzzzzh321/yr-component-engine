//! Saved component identity and frozen configuration. The compiled executor is a
//! disposable cache; each evaluation still gets a fresh guest instance.
use super::host::{IniNamesHost, NativeHost, StartupHost};
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

/// Immutable module admission shared by early resource recipes and later Rules
/// configuration. The runtime and its LoadedModule never cross engine tokens.
pub(super) struct AdmittedComponent {
    bytes: Vec<u8>,
    digest: String,
    manifest: api::ModuleManifest,
    executor: Arc<Executor>,
}

impl AdmittedComponent {
    pub(super) fn load(bytes: Vec<u8>, expected: &str) -> Result<Self, String> {
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
        if (manifest
            .hooks
            .contains(&api::HookKind::ConfigureCountryRead)
            || manifest
                .hooks
                .contains(&api::HookKind::LoadingThemeResource)
            || manifest.hooks.contains(&api::HookKind::CameoResource))
            && !manifest.hooks.contains(&api::HookKind::ConfigureBoundary)
        {
            return Err(
                "configuration history requires configure_boundary lifecycle support".into(),
            );
        }
        Ok(Self {
            bytes,
            digest: expected.into(),
            manifest,
            executor: Arc::new(Executor { runtime, module }),
        })
    }

    pub(super) fn digest(&self) -> &str {
        &self.digest
    }

    pub(super) fn has_startup_resources(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::StartupResources)
    }

    pub(super) fn has_loading_theme_resource(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::LoadingThemeResource)
    }

    pub(super) fn requires_early_admission(&self) -> bool {
        self.has_startup_resources()
            || self.manifest.hooks.contains(&api::HookKind::EvaCatalog)
            || self.manifest.hooks.contains(&api::HookKind::LongListReaders)
    }

    pub(super) fn startup_resource_policy(
        &self,
        context: api::StartupResourceContext,
    ) -> Result<Option<api::StartupResourcePolicy>, String> {
        if !self.has_startup_resources() {
            return Ok(None);
        }
        Ok(self
            .evaluate_startup(Context::StartupResources(context))?
            .startup_resources)
    }

    pub(super) fn eva_catalog_policy(&self) -> Result<Option<api::EvaCatalogPolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::EvaCatalog) {
            return Ok(None);
        }
        Ok(self.evaluate_startup(Context::EvaCatalog)?.eva_catalog)
    }

    pub(super) fn long_list_readers_policy(&self) -> Result<Option<api::LongListReadersPolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::LongListReaders) {
            return Ok(None);
        }
        Ok(self
            .evaluate_startup(Context::LongListReaders)?
            .long_list_readers)
    }

    fn evaluate_startup(&self, context: Context) -> Result<Plan, String> {
        let request = Request {
            api_version: api::API_VERSION,
            context,
            config: Vec::new(),
            state: Vec::new(),
        };
        self.executor
            .runtime
            .evaluate(&self.executor.module, &request, Arc::new(StartupHost))
            .map_err(|e| e.to_string())
    }

    /// Start the same immutable gameplay configuration before cold Rules
    /// processing, without inventing a final Configure or another seed owner.
    pub(super) fn into_configuration_seed(self) -> Program {
        Program {
            bytes: self.bytes,
            digest: self.digest,
            manifest: self.manifest,
            config: Vec::new(),
            compiled: OnceLock::from(Ok(self.executor)),
        }
    }

    pub(super) fn configure(
        self,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
        seed: Option<&Program>,
        steps: &[crate::rules::native_processing::SdkConfigurationStep],
    ) -> Result<Program, String> {
        let retained = seed.filter(|_| {
            self.manifest
                .hooks
                .contains(&api::HookKind::ConfigureBoundary)
        });
        if let Some(seed) = retained
            && (seed.digest != self.digest || seed.manifest != self.manifest)
        {
            return Err("retained process component identity changed without ScenarioClear".into());
        }
        let mut program = self.into_configuration_seed();
        if let Some(seed) = retained {
            program.config = seed.config.clone();
        }
        program.configure_history(
            passes,
            object_types,
            warheads,
            weapons,
            registries,
            steps,
            retained.is_some(),
        )?;
        Ok(program)
    }
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
    fn configure_step(
        &mut self,
        executor: &Executor,
        index: usize,
        step: &crate::rules::native_processing::SdkConfigurationStep,
    ) -> Result<(), String> {
        use crate::rules::native_processing::SdkConfigurationStep;
        match step {
            SdkConfigurationStep::RulesPass { ini, registry } => {
                self.configure_pass(executor, index, ini, false, &[], &[], &[], Some(registry))
            }
            SdkConfigurationStep::CountryPass { ini, registry } => {
                if !self
                    .manifest
                    .hooks
                    .contains(&api::HookKind::ConfigureFactionPass)
                {
                    return Ok(());
                }
                let host = NativeHost::configuration(ini.clone(), Vec::new())
                    .with_registries(registry.clone())
                    .enable_faction_inputs();
                self.configure(
                    executor,
                    Context::ConfigureFactionPass,
                    host,
                    &format!("Country-only configuration {index}"),
                )
            }
            SdkConfigurationStep::Boundary(boundary) => self.configure(
                executor,
                Context::ConfigureBoundary(api::ConfigureBoundaryContext {
                    boundary: *boundary,
                }),
                NativeHost::simulation(),
                &format!("configuration boundary {index} ({boundary:?})"),
            ),
        }
    }

    fn configure_steps(
        &mut self,
        executor: &Executor,
        steps: &[crate::rules::native_processing::SdkConfigurationStep],
    ) -> Result<(), String> {
        for (index, step) in steps.iter().enumerate() {
            self.configure_step(executor, index, step)?;
        }
        Ok(())
    }

    pub(crate) fn has_country_read(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::ConfigureCountryRead)
    }

    pub(crate) fn has_loading_theme_resource(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::LoadingThemeResource)
    }

    pub(crate) fn has_early_configuration(&self) -> bool {
        self.has_country_read() || self.has_loading_theme_resource() || self.has_cameo_resource()
    }

    /// Query the selected physical source using the accepted configuration.
    /// This cannot advance history or expose other INI/configuration services.
    pub(crate) fn loading_theme_policy(
        &self,
        context: api::LoadingThemeResourceContext,
        source: &IniFile,
    ) -> Result<Option<api::LoadingThemePolicy>, String> {
        if !self.has_loading_theme_resource() {
            return Ok(None);
        }
        context.validate()?;
        let host = IniNamesHost::new(source.clone(), context.sections(), 128);
        Ok(self
            .evaluate_with_host(Context::LoadingThemeResource(context), &[], Arc::new(host))?
            .loading_theme)
    }

    /// An isolated projection constructs fresh registries from the same
    /// selected module. It must not inherit or advance the live config history.
    pub(crate) fn fresh_configuration_seed(&self) -> Self {
        let mut program = self.clone();
        program.config.clear();
        program
    }

    pub(crate) fn has_faction_resource(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::FactionResource)
    }

    pub(crate) fn faction_resource_policy(
        &self,
        resource: &api::FactionResourceContext,
    ) -> Result<Option<api::FactionResourcePolicy>, String> {
        if !self.has_faction_resource() {
            return Ok(None);
        }
        Ok(self
            .evaluate(Context::FactionResource(resource.clone()), &[])?
            .faction_resource)
    }

    /// The registry owner commits each successful returned checkpoint and
    /// removes that one pending step. A later failure keeps prior checkpoints.
    pub(crate) fn advance_configuration_step(
        &self,
        index: usize,
        step: &crate::rules::native_processing::SdkConfigurationStep,
    ) -> Result<Self, String> {
        if !self.has_early_configuration() {
            return Err("component has no early configuration capability".into());
        }
        let executor = self.executor()?;
        let mut candidate = self.clone();
        candidate.configure_step(executor, index, step)?;
        Ok(candidate)
    }

    /// Only the current physical INI is exposed. The Country body has not
    /// executed, so no completed pass or suffix view is fabricated here.
    pub(crate) fn configure_country_read(
        &self,
        pass: &IniFile,
    ) -> Result<(Self, Option<api::CountryReadPolicy>), String> {
        if !self.has_country_read() {
            return Ok((self.clone(), None));
        }
        let executor = self.executor()?;
        let plan = self.configuration_plan(
            executor,
            Context::ConfigureCountryRead,
            NativeHost::configuration(pass.clone(), Vec::new()),
            "Country pre-body configuration",
        )?;
        let mut candidate = self.clone();
        if let Some(config) = plan.config {
            candidate.config = config;
        }
        Ok((candidate, plan.country_read))
    }

    /// Finish an isolated receipt with its own already-admitted identity and
    /// chronology. This never updates the live registry's checkpoint.
    pub(crate) fn finish_configuration(
        &self,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
        steps: &[crate::rules::native_processing::SdkConfigurationStep],
    ) -> Result<Self, String> {
        let mut program = self.clone();
        program.configure_history(
            passes,
            object_types,
            warheads,
            weapons,
            registries,
            steps,
            true,
        )?;
        Ok(program)
    }

    /// One configuration owner serves fresh admission, early continuation and
    /// isolated source projections. Saved restoration never enters this path.
    fn configure_history(
        &mut self,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
        steps: &[crate::rules::native_processing::SdkConfigurationStep],
        resumed: bool,
    ) -> Result<(), String> {
        if (self.manifest.hooks.contains(&api::HookKind::Effects)
            || self.manifest.hooks.contains(&api::HookKind::ArmorResource)
            || self
                .manifest
                .hooks
                .contains(&api::HookKind::ProductionResource)
            || self.manifest.hooks.contains(&api::HookKind::EmpResource)
            || self
                .manifest
                .hooks
                .contains(&api::HookKind::FactionResource)
            || self
                .manifest
                .hooks
                .contains(&api::HookKind::FoundationResource))
            && (passes.is_empty() || registries.len() + 1 != passes.len())
        {
            return Err(
                "resource configuration requires physical Rules read-point registries".into(),
            );
        }
        let executor = Arc::clone(self.executor()?);
        if self
            .manifest
            .hooks
            .contains(&api::HookKind::ConfigureBoundary)
            && (resumed || !steps.is_empty())
        {
            self.configure_steps(&executor, steps)?;
            let final_ini = passes
                .last()
                .ok_or("no final Rules projection after process history")?;
            self.configure_pass(
                &executor,
                steps.len(),
                final_ini,
                true,
                object_types,
                warheads,
                weapons,
                None,
            )?;
        } else {
            for (index, ini) in passes.iter().enumerate() {
                self.configure_pass(
                    &executor,
                    index,
                    ini,
                    index + 1 == passes.len(),
                    object_types,
                    warheads,
                    weapons,
                    registries.get(index),
                )?;
            }
        }
        Ok(())
    }

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
        Self::load_with_registries(
            bytes,
            expected,
            passes,
            object_types,
            warheads,
            weapons,
            &[],
        )
    }

    pub(crate) fn load_with_registries(
        bytes: Vec<u8>,
        expected: &str,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
    ) -> Result<Self, String> {
        Self::load_with_process_context(
            bytes,
            expected,
            passes,
            object_types,
            warheads,
            weapons,
            registries,
            None,
            &[],
        )
    }

    /// Configure an admitted module against the one retained Rules timeline.
    pub(crate) fn load_with_process_context(
        bytes: Vec<u8>,
        expected: &str,
        passes: &[IniFile],
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registries: &[crate::rules::native_processing::SdkRegistryPass],
        seed: Option<&Program>,
        steps: &[crate::rules::native_processing::SdkConfigurationStep],
    ) -> Result<Self, String> {
        AdmittedComponent::load(bytes, expected)?.configure(
            passes,
            object_types,
            warheads,
            weapons,
            registries,
            seed,
            steps,
        )
    }

    fn configure_pass(
        &mut self,
        executor: &Executor,
        index: usize,
        ini: &IniFile,
        final_pass: bool,
        object_types: &[String],
        warheads: &[String],
        weapons: &[String],
        registry: Option<&crate::rules::native_processing::SdkRegistryPass>,
    ) -> Result<(), String> {
        let context = Context::Configure(api::ConfigureContext {
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
        });
        let mut host = NativeHost::configuration(
            ini.clone(),
            if final_pass {
                weapons.to_vec()
            } else {
                Vec::new()
            },
        );
        if let Some(registry) = registry {
            host = host.with_registries(registry.clone());
        }
        if self.has_foundation_resource() {
            host = host.enable_fixed_art_inputs();
        }
        if self.has_cameo_resource() {
            host = host.enable_cameo_inputs();
        }
        if self.has_faction_resource() {
            host = host.enable_faction_inputs();
        }
        self.configure(
            executor,
            context,
            host,
            &format!("configuration pass {index} (final={final_pass})"),
        )
    }

    fn configure(
        &mut self,
        executor: &Executor,
        context: Context,
        host: NativeHost,
        label: &str,
    ) -> Result<(), String> {
        let plan = self.configuration_plan(executor, context, host, label)?;
        if let Some(config) = plan.config {
            self.config = config;
        }
        Ok(())
    }

    fn configuration_plan(
        &self,
        executor: &Executor,
        context: Context,
        host: NativeHost,
        label: &str,
    ) -> Result<Plan, String> {
        let request = Request {
            api_version: api::API_VERSION,
            context,
            config: self.config.clone(),
            state: Vec::new(),
        };
        let plan = executor
            .runtime
            .evaluate(&executor.module, &request, Arc::new(host))
            .map_err(|e| format!("{label} (frozen bytes={}): {e}", self.config.len()))?;
        if !plan.state.is_empty() || plan.damage.is_some() || !plan.operations.is_empty() {
            return Err("configuration returned simulation writes".into());
        }
        Ok(plan)
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

    pub(crate) fn has_foundation_resource(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::FoundationResource)
    }

    pub(crate) fn has_cameo_resource(&self) -> bool {
        self.manifest.hooks.contains(&api::HookKind::CameoResource)
    }

    pub(crate) fn cameo_resource_policy(&self) -> Result<Option<api::CameoResourcePolicy>, String> {
        if !self.has_cameo_resource() {
            return Ok(None);
        }
        Ok(self
            .evaluate(Context::CameoResource(api::CameoResourceContext {}), &[])?
            .cameo_resource)
    }

    pub(crate) fn foundation_resource_policy(
        &self,
        resource: &api::FoundationResourceContext,
    ) -> Result<Option<api::FoundationResourcePolicy>, String> {
        if !self.has_foundation_resource() {
            return Ok(None);
        }
        let plan = self.evaluate(Context::FoundationResource(resource.clone()), &[])?;
        Ok(plan.foundation_resource)
    }

    pub(crate) fn has_emp_resource(&self) -> bool {
        self.manifest.hooks.contains(&api::HookKind::EmpResource)
    }

    pub(crate) fn emp_resource_policy(
        &self,
        resource: &api::EmpResourceContext,
    ) -> Result<Option<api::EmpResourcePolicy>, String> {
        if !self.has_emp_resource() {
            return Ok(None);
        }
        let plan = self.evaluate(Context::EmpResource(resource.clone()), &[])?;
        Ok(plan.emp_resource)
    }

    pub(crate) fn has_production_resource(&self) -> bool {
        self.manifest
            .hooks
            .contains(&api::HookKind::ProductionResource)
    }

    pub(crate) fn production_resource_policy(
        &self,
        resource: &api::ProductionResourceContext,
    ) -> Result<Option<api::ProductionResourcePolicy>, String> {
        if !self.has_production_resource() {
            return Ok(None);
        }
        let plan = self.evaluate(Context::ProductionResource(resource.clone()), &[])?;
        Ok(plan.production_resource)
    }

    pub(crate) fn has_armor_resource(&self) -> bool {
        self.manifest.hooks.contains(&api::HookKind::ArmorResource)
    }

    pub(crate) fn armor_resource_policy(
        &self,
        resource: &api::ArmorResourceContext,
    ) -> Result<Option<api::ArmorResourcePolicy>, String> {
        if !self.has_armor_resource() {
            return Ok(None);
        }
        let plan = self.evaluate(Context::ArmorResource(resource.clone()), &[])?;
        Ok(plan.armor_resource)
    }

    pub(crate) fn has_effect_runtime(&self) -> bool {
        self.manifest.hooks.contains(&api::HookKind::Effects)
    }

    pub(crate) fn effect_resource_policy(
        &self,
        resource: &api::EffectResourceContext,
    ) -> Result<Option<api::EffectDescriptor>, String> {
        if !self.has_effect_runtime() {
            return Ok(None);
        }
        let plan = self.evaluate(Context::EffectResource(resource.clone()), &[])?;
        Ok(plan.effect_resource)
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
        if plan
            .type_resource
            .is_some_and(|policy| policy.keep_alive.is_some())
            && !self.manifest.hooks.contains(&api::HookKind::MatchRules)
        {
            return Err("survival type policies require the match_rules capability".into());
        }
        Ok(plan.type_resource)
    }

    pub(crate) fn match_rules_policy(&self) -> Result<Option<api::MatchRulesPolicy>, String> {
        if !self.manifest.hooks.contains(&api::HookKind::MatchRules) {
            return Ok(None);
        }
        Ok(self
            .evaluate(Context::MatchRules(api::MatchRulesContext {}), &[])?
            .match_rules)
    }

    fn executor(&self) -> Result<&Arc<Executor>, String> {
        self.compiled
            .get_or_init(|| {
                let admitted = AdmittedComponent::load(self.bytes.clone(), &self.digest)?;
                if admitted.manifest != self.manifest {
                    return Err("saved module manifest mismatch".into());
                }
                Ok(admitted.executor)
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(crate) fn evaluate(&self, context: Context, state: &[u8]) -> Result<Plan, String> {
        self.evaluate_with_host(context, state, Arc::new(NativeHost::simulation()))
    }

    fn evaluate_with_host(
        &self,
        context: Context,
        state: &[u8],
        host: Arc<dyn vera_mod_runtime::HostServices>,
    ) -> Result<Plan, String> {
        let executor = self.executor()?;
        let request = Request {
            api_version: api::API_VERSION,
            context,
            config: self.config.clone(),
            state: state.to_vec(),
        };
        let plan = executor
            .runtime
            .evaluate(&executor.module, &request, host)
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
