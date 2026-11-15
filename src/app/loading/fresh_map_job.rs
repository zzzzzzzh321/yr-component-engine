//! Main-thread continuations for the existing fresh-map funnel. Each slice
//! borrows the same process authorities anew; suspended slices retain only
//! their owned scenario construction data, never references into AppState.

use super::*;
use std::task::Poll;

type RulesOwner = crate::rules::process_owner::NativeRulesProcessOwner;
type RulesPrefix = crate::rules::process_owner::NativeNonCampaignPrefixPreparation;
pub(crate) type RunRules = Box<
    dyn FnOnce(
            &mut RulesOwner,
            &mut AssetManager,
            &mut Option<RulesPrefix>,
            bool,
        ) -> Result<PreparedScenarioRules>
        + Send,
>;
type ResumeRules = Box<
    dyn for<'a> FnOnce(FreshMapLoadServices<'a>, PreparedScenarioRules) -> Result<FreshMapLoadStep>,
>;

pub(crate) struct ScenarioRulesTask {
    result: crate::app::loading::cpu_job::CpuLoadJob<Option<ScenarioRulesOutcome>>,
    pub(crate) retired: bool,
}

pub(crate) struct ScenarioRulesOutcome {
    pub(crate) assets: AssetManager,
    pub(crate) owner: RulesOwner,
    pub(crate) rules_prefix: Option<RulesPrefix>,
    rules: Result<PreparedScenarioRules>,
}

impl ScenarioRulesTask {
    pub(crate) fn start(
        owner: &mut Option<RulesOwner>,
        assets: &mut Option<AssetManager>,
        rules_prefix: &mut Option<RulesPrefix>,
        the_team_enabled: bool,
        work: RunRules,
    ) -> Result<Self> {
        // Spawn before moving the process authorities: the OS can reject the
        // thread creation, dropping its closure, while both remain at home.
        let (send, receive) =
            std::sync::mpsc::channel::<(RulesOwner, AssetManager, Option<RulesPrefix>)>();
        let result =
            crate::app::loading::cpu_job::CpuLoadJob::start("scenario-rules", move || {
                let (mut owner, mut assets, mut rules_prefix) = receive.recv().ok()?;
                let rules = work(&mut owner, &mut assets, &mut rules_prefix, the_team_enabled);
                Some(ScenarioRulesOutcome {
                    assets,
                    owner,
                    rules_prefix,
                    rules,
                })
            })?;
        let owner = owner
            .take()
            .expect("scenario load owns Rules before starting work");
        let assets = assets
            .take()
            .expect("scenario load has an asset lease before starting work");
        // The worker owns the only receiver and starts by waiting for this send.
        assert!(send.send((owner, assets, rules_prefix.take())).is_ok());
        Ok(Self {
            result,
            retired: false,
        })
    }

    pub(crate) fn poll(&mut self) -> Result<Poll<ScenarioRulesOutcome>> {
        match self.result.poll()? {
            Poll::Pending => Ok(Poll::Pending),
            Poll::Ready(Some(result)) => Ok(Poll::Ready(result)),
            Poll::Ready(None) => anyhow::bail!("scenario Rules worker received no authorities"),
        }
    }
}

pub(crate) struct FreshMapLoadServices<'a> {
    pub(crate) gpu: &'a GpuContext,
    pub(crate) batch: &'a BatchRenderer,
    pub(crate) asset_manager: &'a mut AssetManager,
    pub(crate) native_rules_owner: &'a mut crate::rules::process_owner::NativeRulesProcessOwner,
    pub(crate) the_team_enabled: bool,
    pub(crate) prepared_noncampaign_prefix:
        &'a mut Option<crate::rules::process_owner::NativeNonCampaignPrefixPreparation>,
    pub(crate) audio: Option<&'a crate::app::process_assets::ProcessAudioCatalog>,
    pub(crate) themes: Option<&'a crate::rules::theme_ini::ThemeCatalog>,
    pub(crate) shared_cell_dummy: crate::map::resolved_terrain::SharedCellDummy,
    pub(crate) tile_variant_selector_cache:
        &'a mut crate::map::tile_variant_selector::TileVariantSelectorCache,
    pub(crate) progress: &'a mut dyn crate::app::loading::pump::LoadingProgressSink,
    /// `[VERA] HouseColorStrength=`: how strongly this battle's `[Colors]`
    /// team-color bands are boosted. Presentation only, read once while the
    /// world's colours are built, so the atlas, the voxel ramp texture, the
    /// radar dots and the target lines all share one table.
    pub(crate) house_color_strength: u8,
}

pub(super) struct FreshMapLoadContinuation {
    label: &'static str,
    log_phase_begin: bool,
    action: FreshMapLoadAction,
}

enum FreshMapLoadAction {
    Main(Box<dyn for<'a> FnOnce(FreshMapLoadServices<'a>) -> Result<FreshMapLoadStep>>),
    Rules(RunRules, ResumeRules),
    AwaitingRules(ResumeRules),
}

pub(super) enum FreshMapLoadStep {
    Continue(FreshMapLoadContinuation),
    Complete(Box<MapLoadResult>),
}

impl FreshMapLoadContinuation {
    pub(super) fn rules(
        work: impl FnOnce(
            &mut RulesOwner,
            &mut AssetManager,
            &mut Option<RulesPrefix>,
            bool,
        ) -> Result<PreparedScenarioRules>
        + Send
        + 'static,
        resume: impl for<'a> FnOnce(
            FreshMapLoadServices<'a>,
            PreparedScenarioRules,
        ) -> Result<FreshMapLoadStep>
        + 'static,
    ) -> Self {
        Self {
            label: "rules",
            log_phase_begin: true,
            action: FreshMapLoadAction::Rules(Box::new(work), Box::new(resume)),
        }
    }

    pub(super) fn gameplay_resources(
        work: crate::sim::runtime::FreshScenarioGameplayWork,
        resume: impl for<'a> FnOnce(
            FreshMapLoadServices<'a>,
            crate::rules::ruleset::RuleSet,
            Result<crate::sim::runtime::ConfiguredFreshScenarioGameplay, String>,
        ) -> Result<FreshMapLoadStep>
        + 'static,
    ) -> Self {
        match crate::app::loading::cpu_job::CpuLoadJob::start("scenario-gameplay", move || {
            work.configure()
        }) {
            Ok(job) => {
                let mut continuation = Self::await_gameplay_resources(job, resume);
                continuation.log_phase_begin = true;
                continuation
            }
            Err(error) => Self::new("gameplay-resources", move |_| Err(error)),
        }
    }

    fn await_gameplay_resources(
        mut job: crate::app::loading::cpu_job::CpuLoadJob<(
            crate::rules::ruleset::RuleSet,
            Result<crate::sim::runtime::ConfiguredFreshScenarioGameplay, String>,
        )>,
        resume: impl for<'a> FnOnce(
            FreshMapLoadServices<'a>,
            crate::rules::ruleset::RuleSet,
            Result<crate::sim::runtime::ConfiguredFreshScenarioGameplay, String>,
        ) -> Result<FreshMapLoadStep>
        + 'static,
    ) -> Self {
        let mut continuation =
            Self::new("gameplay-resources", move |services| match job.poll()? {
                std::task::Poll::Pending => Ok(FreshMapLoadStep::Continue(
                    Self::await_gameplay_resources(job, resume),
                )),
                std::task::Poll::Ready((rules, configured)) => resume(services, rules, configured),
            });
        continuation.log_phase_begin = false;
        continuation
    }

    pub(super) fn new(
        label: &'static str,
        run: impl for<'a> FnOnce(FreshMapLoadServices<'a>) -> Result<FreshMapLoadStep> + 'static,
    ) -> Self {
        Self {
            label,
            log_phase_begin: true,
            action: FreshMapLoadAction::Main(Box::new(run)),
        }
    }
}

pub(crate) struct FreshMapLoadJob {
    /// Immutable observations for loading art/failure routing. The consuming
    /// startup and source remain in the single continuation; these copies
    /// neither construct a world nor admit another runtime.
    startup: LoadingStartup,
    map_source: LoadedMapSource,
    theater_cache_mismatch: bool,
    next: Option<FreshMapLoadContinuation>,
}

impl FreshMapLoadJob {
    pub(crate) fn new(
        initial: MapLoadInitial,
        startup: LoadingStartup,
        context: FreshScenarioLoadContextDescriptor,
        theater_cache_mismatch: bool,
        runtime_color_scheme_count: usize,
    ) -> Self {
        let map_source = initial.map_source().clone();
        Self {
            startup: startup.clone(),
            map_source,
            theater_cache_mismatch,
            next: Some(FreshMapLoadContinuation::new("theater", move |services| {
                super::load_map_first_step(
                    services,
                    initial,
                    startup,
                    context,
                    theater_cache_mismatch,
                    runtime_color_scheme_count,
                )
            })),
        }
    }

    pub(crate) fn startup(&self) -> &LoadingStartup {
        &self.startup
    }
    pub(crate) fn map_source(&self) -> &LoadedMapSource {
        &self.map_source
    }
    pub(crate) fn theater_cache_mismatch(&self) -> bool {
        self.theater_cache_mismatch
    }

    pub(crate) fn awaiting_rules(&self) -> bool {
        self.next
            .as_ref()
            .is_some_and(|next| matches!(next.action, FreshMapLoadAction::AwaitingRules(_)))
    }

    /// Offline desktop loading can yield with Rules/AI preparation in flight.
    /// The process retains the worker so retiring this job cannot discard the
    /// process registries or the original asset lease.
    pub(crate) fn poll_offline_rules(
        &mut self,
        process: &mut crate::app::process_assets::ProcessAssets,
        assets: &mut Option<AssetManager>,
        rules_prefix: &mut Option<RulesPrefix>,
        the_team_enabled: bool,
    ) -> Result<Poll<()>> {
        if cfg!(all(
            feature = "wasm",
            target_arch = "wasm32",
            target_os = "unknown"
        )) || !matches!(
            self.startup,
            LoadingStartup::Campaign(_)
                | LoadingStartup::Accepted(_)
                | LoadingStartup::UnverifiedLegacy { .. }
        ) || matches!(self.map_source, LoadedMapSource::Generated { .. })
        {
            return Ok(Poll::Ready(()));
        }
        let Some(next) = self.next.take() else {
            return Ok(Poll::Ready(()));
        };
        match next.action {
            FreshMapLoadAction::Rules(work, resume) => {
                log::debug!(target: "vera20k::loading_attempt", "phase_begin rules");
                process.start_scenario_rules(assets, rules_prefix, the_team_enabled, work)?;
                self.next = Some(FreshMapLoadContinuation {
                    action: FreshMapLoadAction::AwaitingRules(resume),
                    ..next
                });
                Ok(Poll::Pending)
            }
            FreshMapLoadAction::AwaitingRules(resume) => {
                let Poll::Ready(outcome) = process.poll_scenario_rules()? else {
                    self.next = Some(FreshMapLoadContinuation {
                        action: FreshMapLoadAction::AwaitingRules(resume),
                        ..next
                    });
                    return Ok(Poll::Pending);
                };
                let ScenarioRulesOutcome {
                    assets: returned,
                    owner,
                    rules_prefix: returned_prefix,
                    rules,
                } = outcome;
                process.restore_scenario_rules_owner(owner);
                *assets = Some(returned);
                *rules_prefix = returned_prefix;
                let prepared = rules?;
                self.next = Some(FreshMapLoadContinuation::new(
                    "rules-ready",
                    move |services| resume(services, prepared),
                ));
                Ok(Poll::Ready(()))
            }
            _ => {
                self.next = Some(next);
                Ok(Poll::Ready(()))
            }
        }
    }

    /// A slice ends only after a complete native operation; in particular,
    /// Rules load_scenario restores its registry before any event-loop yield.
    /// Callers cannot resume a failed or completed transaction a second time.
    pub(crate) fn step(
        &mut self,
        services: FreshMapLoadServices<'_>,
    ) -> Result<Option<MapLoadResult>> {
        let continuation = self
            .next
            .take()
            .ok_or_else(|| anyhow::anyhow!("fresh-map continuation already consumed"))?;
        if continuation.log_phase_begin {
            log::debug!(target: "vera20k::loading_attempt", "phase_begin {}", continuation.label);
        }
        let result = match continuation.action {
            FreshMapLoadAction::Main(run) => run(services)?,
            FreshMapLoadAction::Rules(work, resume) => {
                let rules = work(
                    services.native_rules_owner,
                    services.asset_manager,
                    services.prepared_noncampaign_prefix,
                    services.the_team_enabled,
                )?;
                resume(services, rules)?
            }
            FreshMapLoadAction::AwaitingRules(_) => {
                anyhow::bail!("scenario Rules continuation resumed before its authorities returned")
            }
        };
        match result {
            FreshMapLoadStep::Continue(next) => {
                self.next = Some(next);
                Ok(None)
            }
            FreshMapLoadStep::Complete(result) => Ok(Some(*result)),
        }
    }
}
