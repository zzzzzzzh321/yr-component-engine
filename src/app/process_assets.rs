//! Process-wide asset ownership (F11): one `AssetManager` for the process,
//! leased to the loading pipeline and always returned.
//!
//! Retail keeps one process-global MIX list plus the sticky
//! `LoadFileFromMIX` CRC cache and the active theater archive group. All of
//! that state lives on the manager, so dropping or reconstructing it
//! mid-process silently discards process-sticky semantics. The slot makes the
//! lifecycle explicit:
//!
//! - `Available` — the manager is home; shell, audio, render, and capture
//!   borrow it in place.
//! - `Loading` — leased into the loading job; every loading outcome
//!   (success, failure, cancellation) must route the manager back through
//!   `return_from_loading`.
//! - Absent — startup could not open the retail archives; lookups no-op.
//!
//! A double return keeps the resident manager and logs instead of silently
//! replacing process-sticky state.

use crate::assets::asset_manager::{AssetManager, MediaArchiveMode};

pub(crate) struct ProcessAssets {
    /// Selected at launch, retained even if initial archive loading fails.
    media_archive_mode: MediaArchiveMode,
    /// Launch request and its first resolved names survive a lost asset lease.
    ini_config: crate::util::ini_roots::IniRootConfig,
    selected_ini_roots: Option<crate::util::ini_roots::IniRoots>,
    /// Immutable resource recipes survive a lost MIX-manager lease. They are
    /// derived from one admitted module, not another gameplay configuration.
    startup_resources: Option<crate::assets::startup_resources::StartupResources>,
    /// Personal rules are frozen at startup, like the selected game sources.
    personal_rules: crate::rules::personal_rules::PersonalRules,
    manager: Option<AssetManager>,
    leased: bool,
    /// Frozen launch policy, including deferred recovery after assetless startup.
    load_audio_index: bool,
    /// None means sources have not been selected. A selected empty catalog is
    /// still final for this process; later maps cannot replace its definitions.
    audio_catalog: Option<ProcessAudioCatalog>,
    /// One process-resident native Rules registry authority. Unlike the MIX
    /// manager it never leaves this owner: scenario transitions mutate it
    /// synchronously in place, so later failures cannot drop or roll it back.
    native_rules: Option<crate::rules::process_owner::NativeRulesProcessOwner>,
    /// CampaignClass's retained array. Init_Game52C605 creates it before
    /// Load_Game_Rules selects Movies; ClearScene685609 rereads its sources.
    campaigns: Option<crate::rules::campaigns::CampaignRegistry>,
    /// MapClass's one process-global fallback CellClass (`0x00ABDC50`). Map
    /// reloads bind their resolved grid to this same live identity.
    pub(crate) shared_cell_dummy: crate::map::resolved_terrain::SharedCellDummy,
    /// CSF string table — localized display names for units, buildings, UI
    /// text. Loaded once at startup from the retail archives; process-wide.
    pub(crate) csf: Option<crate::assets::csf_file::CsfFile>,
    /// Process-persistent terrain-load cache. Scenario teardown, failed
    /// loads, reseeds, and save transitions must not clear it.
    pub(crate) tile_variant_selector_cache:
        crate::map::tile_variant_selector::TileVariantSelectorCache,
    /// The network games' Rules/AI/Art INI hashes, computed once from the
    /// process sources; `None` without them.
    native_ini_hashes: Option<crate::rules::native_ini_hash::NativeIniHashes>,
}

/// Immutable process sources; output players and match queues have separate
/// lifetimes. Rules binding shares this exact SOUNDMD allocation.
pub(crate) struct ProcessAudioCatalog {
    definitions: crate::rules::audio_sources::AudioDefinitions,
    index: Option<crate::assets::asset_manager::LoadedAudioIndex>,
}

impl ProcessAudioCatalog {
    pub(crate) fn sounds(&self) -> &crate::rules::sound_ini::SoundRegistry {
        self.definitions.sounds()
    }

    pub(crate) fn eva(&self) -> &crate::rules::sound_ini::EvaRegistry {
        self.definitions.eva()
    }

    pub(crate) fn index(&self) -> Option<&crate::assets::audio_bag::AudioIndex> {
        self.index.as_ref().map(|loaded| &loaded.index)
    }
}

impl ProcessAssets {
    pub(crate) fn new(media_archive_mode: MediaArchiveMode, load_audio_index: bool) -> Self {
        Self {
            media_archive_mode,
            ini_config: Default::default(),
            selected_ini_roots: None,
            startup_resources: None,
            personal_rules: Default::default(),
            manager: None,
            leased: false,
            load_audio_index,
            audio_catalog: None,
            native_rules: None,
            campaigns: None,
            shared_cell_dummy: Default::default(),
            csf: None,
            tile_variant_selector_cache: Default::default(),
            native_ini_hashes: None,
        }
    }

    pub(crate) fn new_with_ini_config(
        media: MediaArchiveMode,
        audio: bool,
        config: crate::util::ini_roots::IniRootConfig,
    ) -> Self {
        let mut process = Self::new(media, audio);
        process.ini_config = config;
        process
    }

    pub(crate) fn with_personal_rules(
        mut self,
        rules: crate::rules::personal_rules::PersonalRules,
    ) -> Self {
        self.personal_rules = rules;
        self
    }

    /// Recovery uses the process's already resolved choice, never a new Auto
    /// decision against an archive set that may have changed since startup.
    pub(crate) fn reconstruct_manager(
        &self,
        root: &std::path::Path,
    ) -> anyhow::Result<AssetManager> {
        let assets = if let Some(frozen) = &self.startup_resources {
            crate::mod_sdk::reopen_active_assets(root, self.media_archive_mode, frozen)?
        } else {
            crate::mod_sdk::open_active_assets(root, self.media_archive_mode)?
        };
        Ok(if let Some(roots) = &self.selected_ini_roots {
            assets.with_selected_ini_roots(roots.clone())
        } else {
            assets.with_ini_roots(&self.ini_config)?
        })
    }

    /// Init_Game52C763/52C796 and52C843/52C8A0 select fixed SOUNDMD/EVAMD
    /// before the ordinary shell/scenario loop. See tools/audio_catalog_owner.md.
    /// Recovery uses this same boundary before request preparation. A failed
    /// Rules load retains the audio selection; a live Rules registry is never
    /// replaced or rolled back. VERA retains its tolerant missing-audio policy.
    /// Returns shell compatibility projections only when Rules is first created.
    pub(crate) fn initialize_sources_if_needed(
        &mut self,
        assets: &AssetManager,
    ) -> Result<
        (
            Option<crate::rules::ruleset::RuleSet>,
            Option<crate::rules::ini_parser::IniFile>,
        ),
        String,
    > {
        self.startup_resources
            .get_or_insert_with(|| assets.startup_resources().clone());
        if self
            .selected_ini_roots
            .as_ref()
            .is_some_and(|roots| roots != assets.ini_roots())
        {
            return Err("process INI roots changed after source selection".into());
        }
        self.selected_ini_roots
            .get_or_insert_with(|| assets.ini_roots().clone());
        if self.campaigns.is_none() {
            self.reload_campaigns(assets);
        }
        if self.audio_catalog.is_none() {
            let index = if self.load_audio_index {
                assets
                    .load_audio_index()
                    .map_err(|error| log::warn!("Process audio index unavailable: {error}"))
                    .ok()
                    .flatten()
            } else {
                None
            };
            let definitions = crate::rules::audio_sources::AudioDefinitions::select_admitting(
                assets,
                index.as_ref().map(|loaded| &loaded.index),
            );
            let source_receipt = serde_json::json!({
                "soundmd": definitions.sound_source(),
                "evamd": definitions.eva_source(),
                "index_enabled": self.load_audio_index,
                "index": index.as_ref().map(|loaded| &loaded.sources),
                "sound_count": definitions.sounds().len(),
                "eva_count": definitions.eva().len(),
            });
            let source_receipt = {
                let mut receipt = source_receipt;
                receipt["additional_indexes"] =
                    serde_json::json!(index.as_ref().map(|loaded| &loaded.additional_sources));
                receipt
            };
            log::info!("Process audio catalog selected: {source_receipt}");
            self.audio_catalog = Some(ProcessAudioCatalog { definitions, index });
        }
        if self.native_rules.is_some() {
            return Ok((None, None));
        }
        let sounds =
            std::sync::Arc::clone(self.audio_catalog.as_ref().unwrap().definitions.sounds());
        let eva = std::sync::Arc::clone(self.audio_catalog.as_ref().unwrap().definitions.eva());
        let startup = crate::rules::retail_sources::RetailRulesSources::select(assets)?
            .with_personal_rules(self.personal_rules.clone())
            .into_startup(sounds, eva)?;
        let (rules, projection, owner) = startup.into_parts();
        self.native_rules = Some(owner);
        Ok((rules, projection))
    }

    pub(crate) fn audio_catalog(&self) -> Option<&ProcessAudioCatalog> {
        self.audio_catalog.as_ref()
    }

    /// Init_Game52C605/ClearScene685609 use fresh local INI objects while keeping the
    /// campaign array. This selected retail route reads BATTLEMD.INI through
    /// the same archive/loose-file resolver as the other process sources.
    pub(crate) fn reload_campaigns(&mut self, assets: &AssetManager) {
        self.apply_campaign_source(crate::rules::retail_sources::select_ini(
            assets,
            "BATTLEMD.INI",
        ));
    }

    /// PrepareSession52DF25 retries Load_Campaigns when the retained array is empty.
    pub(crate) fn refresh_empty_campaigns_if_available(&mut self) {
        if self
            .campaigns
            .as_ref()
            .is_some_and(|campaigns| !campaigns.is_empty())
        {
            return;
        }
        self.refresh_campaigns_if_available();
    }

    /// ClearScene685609 re-reads the process-retained Campaign registry on
    /// saved loads too. It never infers campaign identity from a map name.
    pub(crate) fn refresh_campaigns_if_available(&mut self) {
        let selected = self
            .manager
            .as_ref()
            .map(|assets| crate::rules::retail_sources::select_ini(assets, "BATTLEMD.INI"));
        if let Some(selected) = selected {
            self.apply_campaign_source(selected);
        }
    }

    fn apply_campaign_source(
        &mut self,
        selected: Result<crate::rules::retail_sources::SelectedIni, String>,
    ) {
        let registry = self.campaigns.get_or_insert_with(Default::default);
        let selected = match selected {
            Ok(selected) => selected,
            Err(error) => {
                log::warn!("Campaign catalog unavailable: {error}");
                return;
            }
        };
        let empty_movies = crate::rules::movies::MovieRegistry::default();
        let movies = self
            .native_rules
            .as_ref()
            .map_or(&empty_movies, |owner| owner.movies());
        registry.apply_ini(&selected.ini, movies);
        log::info!(
            "Campaign catalog read: {}",
            serde_json::json!(selected.source)
        );
    }

    pub(crate) fn campaigns(&self) -> Option<&crate::rules::campaigns::CampaignRegistry> {
        self.campaigns.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn media_archive_mode(&self) -> MediaArchiveMode {
        self.media_archive_mode
    }

    pub(crate) fn has_native_rules(&self) -> bool {
        self.native_rules.is_some()
    }

    pub(crate) fn native_rules(
        &self,
    ) -> Option<&crate::rules::process_owner::NativeRulesProcessOwner> {
        self.native_rules.as_ref()
    }

    pub(crate) fn native_ini_hashes(
        &self,
    ) -> Option<crate::rules::native_ini_hash::NativeIniHashes> {
        self.native_ini_hashes
    }

    /// The VERA protocol compares the exact process-selected Rules/LANGRULE/ART
    /// plus the selected mode and AIMD snapshots used by scenario loading.
    /// The legacy advertisement CRC remains a separate native UI value.
    pub(crate) fn multiplayer_rules_digest(&self, mode: &str) -> Result<String, String> {
        use crate::rules::retail_sources::{select_ini, select_mode_ini};
        let rules = self
            .native_rules
            .as_ref()
            .ok_or("retail rules not loaded")?;
        let assets = self.manager.as_ref().ok_or("retail sources are loading")?;
        let ai = select_ini(assets, assets.ini_roots().ai())?;
        let selected_mode = select_mode_ini(assets, Some(mode))?;
        let mut digests = rules.multiplayer_source_digests().to_vec();
        digests.push(Some(ai.ini.multiplayer_digest()));
        digests.push(
            selected_mode
                .as_ref()
                .map(crate::rules::ini_parser::IniFile::multiplayer_digest),
        );
        let mut bytes = b"VERA20k rules agreement v1\0".to_vec();
        for digest in digests {
            bytes.push(u8::from(digest.is_some()));
            if let Some(digest) = digest {
                bytes.extend_from_slice(digest.as_bytes());
            }
        }
        Ok(crate::util::sha256::sha256_hex(&bytes))
    }

    /// A seed regenerates its map after Start, so its rules agreement also
    /// covers the inputs selected by the ordinary generation preparation.
    pub(crate) fn multiplayer_generated_rules_digest(
        &mut self,
        mode: &str,
        options: &crate::map::rmg::RmgOptions,
    ) -> Result<String, String> {
        let rules = self.multiplayer_rules_digest(mode)?;
        let assets = self.manager.as_mut().ok_or("retail sources are loading")?;
        let mode_rules = crate::rules::retail_sources::select_mode_ini(assets, Some(mode))?;
        let prepared = crate::app::shell_random_map::prepare_random_map_generation(
            assets,
            options,
            self.native_rules.as_ref(),
            mode_rules.as_ref(),
        )
        .ok_or("generated map theater is unavailable")?;
        let mut bytes = b"VERA20k generated rules agreement v1\0".to_vec();
        bytes.extend_from_slice(rules.as_bytes());
        bytes.extend_from_slice(prepared.multiplayer_digest().as_bytes());
        Ok(crate::util::sha256::sha256_hex(&bytes))
    }

    /// Hash the process sources once (`crate::rules::native_ini_hash`):
    /// the selected rules root and ART, AIMD, each mode's INI by id
    /// `1..=count` and CoopCampMD.
    pub(crate) fn install_native_ini_hashes(
        &mut self,
        assets: &AssetManager,
        modes: &[crate::skirmish_modes::SkirmishGameMode],
    ) {
        use crate::rules::ini_parser::IniFile;
        use crate::rules::retail_sources::{select_ini, select_mode_ini};
        let Some(rules) = self.native_rules.as_ref() else {
            return;
        };
        let ai = match select_ini(assets, assets.ini_roots().ai()) {
            Ok(selected) => selected.ini,
            Err(error) => {
                log::warn!("No INI hashes without the AI root: {error}");
                return;
            }
        };
        // 0x00679D90 walks the ids 1..=count (0x005D5ED0) and finds each
        // mode by id (0x005D5F30); 0x005D67B0 gives an empty INI for a
        // missing file. Retail ids are 1..9 without a gap.
        let mode_inis: Vec<IniFile> = (1..=modes.len() as i32)
            .filter_map(|id| crate::skirmish_modes::mode_by_id(modes, id))
            .map(|mode| {
                select_mode_ini(assets, Some(&mode.override_file))
                    .ok()
                    .flatten()
                    .unwrap_or_else(IniFile::empty)
            })
            .collect();
        let cooperative = select_ini(
            assets,
            crate::app::frontend::skirmish_session::cooperative::COOPERATIVE_CAMPAIGN_INI,
        )
        .ok()
        .map(|selected| selected.ini);
        let mut hashes = crate::rules::native_ini_hash::NativeIniHashes::compute(
            rules.selected_rules_root(),
            &mode_inis,
            cooperative.as_ref(),
            &ai,
            rules.fixed_art(),
        );
        // VERA extension: peers must agree on both personal rules and whether
        // they may replace map-authored rules. An inactive file adds nothing.
        if let Some(crc) = rules.personal_rules_compatibility_crc() {
            hashes.rules = hashes.rules.wrapping_add(crc);
        }
        self.native_ini_hashes = Some(hashes);
    }

    pub(crate) fn read_multiplayer_country(
        &mut self,
        settings: &crate::rules::ini_parser::IniFile,
        current: i32,
    ) -> i32 {
        self.native_rules.as_mut().map_or(current, |rules| {
            rules.read_multiplayer_country(settings, current)
        })
    }

    /// Direct-launch adapters subscribe explicitly to the pre-body Country
    /// initialization frontier. A disabled faction module still selects the
    /// retail reader there; the presence of a Side policy is not a subscription.
    pub(crate) fn requires_external_country_initialization(&self) -> bool {
        cfg!(feature = "ares")
            || self.native_rules.as_ref().is_some_and(|owner| {
                owner.uses_component_country_reader() || owner.uses_component_loading_theme()
            })
    }

    pub(crate) fn refresh_multiplayer_country_bodies(
        &mut self,
    ) -> Result<(), crate::rules::error::RulesError> {
        match self.native_rules.as_mut() {
            Some(rules) => rules.refresh_multiplayer_country_bodies(),
            None => Ok(()),
        }
    }

    pub(crate) fn multiplayer_countries(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.native_rules
            .iter()
            .flat_map(|rules| rules.multiplayer_countries())
    }

    pub(crate) fn country_menu_rows(
        &self,
    ) -> Result<
        Option<Vec<crate::rules::process_owner::LiveCountryMenuRow>>,
        crate::rules::error::RulesError,
    > {
        self.native_rules
            .as_ref()
            .map_or(Ok(None), |owner| owner.country_menu_rows())
    }

    pub(crate) fn defers_loading_theme(&self) -> bool {
        cfg!(feature = "ares")
            || self
                .native_rules
                .as_ref()
                .is_some_and(|owner| owner.uses_component_loading_theme())
    }

    pub(crate) fn select_loading_theme(
        &self,
        country: Option<crate::rules::ruleset::CountryIdx>,
        campaign: Option<&crate::rules::ini_parser::IniFile>,
    ) -> Result<crate::rules::process_owner::LoadingThemeSelection, crate::rules::error::RulesError>
    {
        crate::rules::process_owner::select_loading_theme(
            self.native_rules.as_ref(),
            country,
            campaign,
        )
    }

    /// Freeze the currently accepted Side presentation before its scenario
    /// retires. Consumers retain this value, never a second live rules owner.
    pub(crate) fn side_presentation(
        &self,
        side: i32,
    ) -> Result<Option<crate::rules::AresSideExtension>, crate::rules::error::RulesError> {
        self.native_rules
            .as_ref()
            .map(|rules| rules.side_presentation(side))
            .unwrap_or(Ok(None))
    }

    /// `Game_GetFlagSurface @ 0x1001DCE0` country insignia by HouseType index,
    /// for lobby/diplomacy rows that resolve before a match Rules projection
    /// exists.
    pub(crate) fn country_flag_surface(
        &self,
        index: u16,
    ) -> Result<Option<String>, crate::rules::error::RulesError> {
        self.native_rules
            .as_ref()
            .map_or(Ok(None), |owner| owner.country_flag_surface(index))
    }

    pub(crate) fn project_multiplayer_countries(
        &self,
        projection: &mut crate::rules::ruleset::RuleSet,
    ) {
        if let Some(rules) = self.native_rules.as_ref() {
            rules.project_multiplayer_countries(projection);
        }
    }

    /// Split the process-resident authorities used by a synchronous
    /// scenario load. The leased MIX manager lives in `LoadingJob`; the native
    /// Rules registry remains here so every `?` after its destructive reset
    /// preserves the actual live state.
    pub(crate) fn native_rules_mut_with_tile_cache(
        &mut self,
    ) -> (
        Option<&mut crate::rules::process_owner::NativeRulesProcessOwner>,
        &mut crate::map::tile_variant_selector::TileVariantSelectorCache,
        Option<&ProcessAudioCatalog>,
    ) {
        (
            self.native_rules.as_mut(),
            &mut self.tile_variant_selector_cache,
            self.audio_catalog.as_ref(),
        )
    }

    /// External launch admission uses the original frozen component recipe,
    /// including while a manager lease has been lost or is being recovered.
    pub(crate) fn spawn_identity_policy(
        &self,
    ) -> Option<crate::assets::startup_resources::SpawnIdentityPolicy> {
        self.startup_resources
            .as_ref()
            .and_then(|resources| resources.spawn_identity)
    }

    /// Shared borrow while the manager is home (`Available`).
    pub(crate) fn manager(&self) -> Option<&AssetManager> {
        self.manager.as_ref()
    }

    /// Field-level split borrow for a scenario rescan: the scan registers loose
    /// YRO archives, and its PKT titles resolve through the process CSF. Both
    /// fields live on this owner, so the pair is borrowed together.
    #[cfg(feature = "verax")]
    pub(crate) fn scenario_rescan_sources(
        &mut self,
    ) -> (
        Option<&mut AssetManager>,
        Option<&crate::assets::csf_file::CsfFile>,
    ) {
        (self.manager.as_mut(), self.csf.as_ref())
    }

    /// Field-level split borrow: the resident manager and the terrain-variant
    /// cache live on the same owner, and the RMG preview path needs both at
    /// once.
    pub(crate) fn manager_and_rules_with_tile_cache(
        &mut self,
    ) -> (
        Option<&mut AssetManager>,
        Option<&crate::rules::process_owner::NativeRulesProcessOwner>,
        &mut crate::map::tile_variant_selector::TileVariantSelectorCache,
    ) {
        (
            self.manager.as_mut(),
            self.native_rules.as_ref(),
            &mut self.tile_variant_selector_cache,
        )
    }

    pub(crate) fn is_available(&self) -> bool {
        self.manager.is_some()
    }

    /// True while a lease is outstanding — distinguishes `Loading` (a manager
    /// existed and went out) from Absent (startup never constructed one), so
    /// the reconstruction path only warns about a real loss.
    pub(crate) fn is_leased(&self) -> bool {
        self.leased
    }

    /// `Available -> Loading`. Returns `None` when the slot is absent or the
    /// manager is already leased out.
    pub(crate) fn lease_for_loading(&mut self) -> Option<AssetManager> {
        let leased = self.manager.take();
        if leased.is_some() {
            self.leased = true;
        }
        leased
    }

    /// `Loading -> Available`. Every loading outcome must come back through
    /// here so the sticky CRC cache and active theater identity survive the
    /// process. A double return keeps the resident manager and logs.
    pub(crate) fn return_from_loading(&mut self, manager: AssetManager) {
        self.leased = false;
        if self.manager.is_some() {
            log::error!(
                "ProcessAssets: manager returned while one is already resident; \
                 keeping the resident manager (process-sticky state preserved)"
            );
            return;
        }
        self.startup_resources
            .get_or_insert_with(|| manager.startup_resources().clone());
        self.manager = Some(manager);
    }

    /// A lease ended without a manager to return (the loading job lost it or
    /// reconstructed elsewhere). Clears the lease so the slot can lease again;
    /// the loss itself is the caller's anomaly to log.
    pub(crate) fn note_lease_ended_without_return(&mut self) {
        self.leased = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{MediaArchiveMode, ProcessAssets};
    use crate::assets::asset_manager::AssetManager;

    #[test]
    #[ignore = "requires VERA_SDK_STARTUP_WASM and VERA_SDK_PREVIOUS_WASM"]
    fn startup_component_app_sources_and_lost_lease_recovery_keep_the_same_recipes() {
        use crate::assets::{
            asset_manager::test_support::audio_test_install,
            audio_bag::{decode_bag_audio, test_support::build_idx},
            csf_file::test_support::build_test_csf,
            startup_resources::StartupComponentSource,
        };
        use std::sync::Arc;
        assert!(!crate::rules::ARES_ENABLED);
        let wasm = std::fs::read(std::env::var_os("VERA_SDK_STARTUP_WASM").unwrap()).unwrap();
        let dir = audio_test_install("sdk-startup-app");
        let write =
            |name: &str, bytes: &[u8]| std::fs::write(dir.path().join(name), bytes).unwrap();
        let install_component = |bytes: &[u8]| {
            write("gameplay.wasm", bytes);
            write(
                "vera-mod.json",
                &serde_json::to_vec(&serde_json::json!({
                    "api_version": 1, "program": "gameplay.wasm",
                    "sha256": crate::util::sha256::sha256_hex(bytes),
                }))
                .unwrap(),
            );
        };
        install_component(&wasm);
        for (name, text) in [
            ("rulesmd.ini", "[General]\nBuildSpeed=.7\n"),
            ("artmd.ini", "[Animations]\n"),
            ("aimd.ini", "[AI]\n"),
            (
                "soundmd.ini",
                "[SoundList]\n0=StartupCue\n[StartupCue]\nSounds=modtone\n",
            ),
            ("evamd.ini", "[DialogList]\n"),
        ] {
            write(name, text.as_bytes());
        }
        write("ra2md.csf", &build_test_csf("NAME", "original"));
        write("ares.csf", &build_test_csf("NAME", "extension"));
        let pcm: Vec<u8> = [1000i16, -1000]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect();
        write("audio.idx", &build_idx(&[("baseonly", 0, 4, 22050, 4)]));
        write("audio.bag", &pcm);
        write("ares.idx", &build_idx(&[("modtone", 0, 4, 22050, 4)]));
        write("ares.bag", &pcm);
        let assets =
            crate::mod_sdk::open_active_assets(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
                .unwrap();
        let frozen = assets.startup_resources().clone();
        assert_eq!(frozen.long_list_readers, Some(true));
        let identity = crate::assets::startup_resources::SpawnIdentityPolicy {
            extended_countries: true,
            campaign_side_byte: true,
        };
        assert_eq!(frozen.spawn_identity, Some(identity));
        assert_eq!(frozen.prism_support_defaults, Some(true));
        assert_eq!(frozen.sound_sample_admission, Some(true));
        let StartupComponentSource::Admitted {
            bytes: original, ..
        } = &frozen.component
        else {
            panic!("component must be admitted at the real startup frontier");
        };
        let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, true);
        // These are the same CSF and process-source entry points as AppState
        // initialization, including actual SoundRegistry sample admission.
        process.csf = Some(crate::app::loading::init::load_csf(&assets).unwrap());
        process.initialize_sources_if_needed(&assets).unwrap();
        assert_eq!(process.spawn_identity_policy(), Some(identity));
        assert_eq!(process.csf.as_ref().unwrap().get("NAME"), Some("extension"));
        let audio = process.audio_catalog.as_ref().unwrap();
        let sounds = Arc::clone(audio.definitions.sounds());
        let cue = audio.sounds().get("StartupCue").unwrap();
        assert!(cue.samples_resolved);
        let (entry, data) = audio.index().unwrap().get(&cue.sounds[0]).unwrap();
        assert_eq!(
            decode_bag_audio(entry, data).unwrap().samples_i16,
            [1000, -1000]
        );
        assert_eq!(audio.index.as_ref().unwrap().additional_sources.len(), 1);
        process.return_from_loading(assets);
        drop(process.lease_for_loading().unwrap());
        process.note_lease_ended_without_return();
        assert_eq!(process.spawn_identity_policy(), Some(identity));
        let recovered = process.reconstruct_manager(dir.path()).unwrap();
        assert_eq!(recovered.startup_resources().long_list_readers, Some(true));
        assert_eq!(recovered.startup_resources().spawn_identity, Some(identity));
        assert_eq!(
            recovered.startup_resources().prism_support_defaults,
            Some(true)
        );
        assert_eq!(
            recovered.startup_resources().sound_sample_admission,
            Some(true)
        );
        let StartupComponentSource::Admitted { bytes, .. } =
            &recovered.startup_resources().component
        else {
            panic!("recovery must retain the admitted source");
        };
        assert!(Arc::ptr_eq(original, bytes));
        assert!(Arc::ptr_eq(
            frozen.strings.as_ref().unwrap(),
            recovered.startup_resources().strings.as_ref().unwrap(),
        ));
        process.initialize_sources_if_needed(&recovered).unwrap();
        assert!(Arc::ptr_eq(
            &sounds,
            process.audio_catalog.as_ref().unwrap().definitions.sounds(),
        ));
        assert_eq!(process.csf.as_ref().unwrap().get("NAME"), Some("extension"));
        let previous = std::fs::read(std::env::var_os("VERA_SDK_PREVIOUS_WASM").unwrap()).unwrap();
        install_component(&previous);
        assert!(
            process.reconstruct_manager(dir.path()).is_err(),
            "recovery cannot change the selected component identity"
        );
    }

    #[test]
    fn generated_rules_agreement_covers_rmg_settings_and_theater_inputs() {
        let dir = crate::assets::asset_manager::test_support::audio_test_install("rmg-agreement");
        for (name, bytes) in [
            ("rulesmd.ini", "[General]\nBuildSpeed=.7\n"),
            ("artmd.ini", "[UNIT]\nVoxel=yes\n"),
            ("aimd.ini", "[AI]\nValue=1\n"),
            ("rmgmd.ini", "[General]\nMaxTrees=500\n"),
            (
                "temperatmd.ini",
                "[General]\n[TileSet0000]\nFileName=clear\nTilesInSet=1\nMorphable=yes\n",
            ),
        ] {
            std::fs::write(dir.path().join(name), bytes).unwrap();
        }
        for name in ["unittem.pal", "isotem.pal", "temperat.pal"] {
            std::fs::write(dir.path().join(name), [0_u8; 768]).unwrap();
        }
        let digest = || {
            let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
            let assets = AssetManager::new(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap();
            process.initialize_sources_if_needed(&assets).unwrap();
            process.return_from_loading(assets);
            process
                .multiplayer_generated_rules_digest("", &crate::map::rmg::RmgOptions::default())
                .unwrap()
        };
        let original = digest();
        std::fs::write(dir.path().join("rmgmd.ini"), "[General]\nMaxTrees=501\n").unwrap();
        assert_ne!(original, digest(), "generator tuning must agree");
        std::fs::write(dir.path().join("rmgmd.ini"), "[General]\nMaxTrees=500\n").unwrap();
        std::fs::write(
            dir.path().join("temperatmd.ini"),
            "[General]\n[TileSet0000]\nFileName=clear\nTilesInSet=1\nMorphable=no\n",
        )
        .unwrap();
        assert_ne!(original, digest(), "resolved theater inputs must agree");
    }

    #[test]
    fn multiplayer_agreement_covers_frozen_langrule_and_the_selected_mode() {
        let dir =
            crate::assets::asset_manager::test_support::audio_test_install("multiplayer-agreement");
        for (name, bytes) in [
            ("rulesmd.ini", "[General]\nBuildSpeed=.7\n"),
            ("artmd.ini", "[UNIT]\nVoxel=yes\n"),
            ("aimd.ini", "[AI]\nValue=1\n"),
            (
                "langrule.ini",
                crate::rules::retail_ini_fixture::DIGESTED_LANGRULE_CONTROL,
            ),
            ("mode.ini", "[General]\nBuildSpeed=.6\n"),
        ] {
            std::fs::write(dir.path().join(name), bytes).unwrap();
        }
        let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        let assets = AssetManager::new(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap();
        process.initialize_sources_if_needed(&assets).unwrap();
        process.return_from_loading(assets);
        let original = process.multiplayer_rules_digest("mode.ini").unwrap();
        assert_ne!(original, process.multiplayer_rules_digest("").unwrap());
        std::fs::remove_file(dir.path().join("langrule.ini")).unwrap();
        assert_eq!(
            original,
            process.multiplayer_rules_digest("mode.ini").unwrap(),
            "the live process retains its source snapshot"
        );
        let mut fresh = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        let assets = AssetManager::new(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap();
        fresh.initialize_sources_if_needed(&assets).unwrap();
        fresh.return_from_loading(assets);
        assert_ne!(
            original,
            fresh.multiplayer_rules_digest("mode.ini").unwrap()
        );
    }

    #[test]
    fn personal_rules_and_map_priority_participate_in_network_compatibility() {
        use crate::map::source::test_support::TestDirectory;
        use crate::rules::personal_rules::PersonalRules;

        let dir = TestDirectory::new("personal-rules-network");
        dir.write("rulesmd.ini", b"[General]\nBuildSpeed=.7\n");
        dir.write("artmd.ini", b"[Test]\nVoxel=no\n");
        dir.write("aimd.ini", b"[Test]\nValue=1\n");
        let assets = AssetManager::from_loose_root_for_test(dir.path());
        let hashes = |bytes: &[u8]| {
            let mut process = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false)
                .with_personal_rules(PersonalRules::from_bytes(bytes).unwrap());
            process.initialize_sources_if_needed(&assets).unwrap();
            process.install_native_ini_hashes(&assets, &[]);
            process.native_ini_hashes().unwrap()
        };
        let stock = hashes(b"");
        assert_eq!(stock, hashes(b";[General]\n;BuildSpeed=.5\n"));
        let personal = hashes(b"[General]\nBuildSpeed=.5\n");
        assert_ne!(stock.rules, personal.rules);
        let priority = hashes(b"[General]\nBuildSpeed=.5\n[VERA]\nOverrideMapRules=yes\n");
        assert_ne!(personal.rules, priority.rules);
        assert_eq!((stock.ai, stock.art), (priority.ai, priority.art));
    }

    #[test]
    fn ini_roots_process_recovery_keeps_auto_choice_and_refuses_a_foreign_triplet() {
        use crate::util::ini_roots::{IniProfile, IniRootConfig};
        let dir =
            crate::assets::asset_manager::test_support::audio_test_install("ini-roots-process");
        std::fs::write(
            dir.path().join("rulesmo.ini"),
            b"[General]\nBuildSpeed=.7\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("artmo.ini"), b"[TEST]\nVoxel=yes\n").unwrap();
        std::fs::write(dir.path().join("aimo.ini"), b"[Marker]\nValue=MO\n").unwrap();
        let config = IniRootConfig {
            profile: IniProfile::Auto,
            ..Default::default()
        };
        let manager = AssetManager::new(dir.path(), MediaArchiveMode::STOCK_DIGITAL)
            .unwrap()
            .with_ini_roots(&config)
            .unwrap();
        let mut process = ProcessAssets::new_with_ini_config(
            MediaArchiveMode::STOCK_DIGITAL,
            false,
            config.clone(),
        );
        process.initialize_sources_if_needed(&manager).unwrap();
        process.return_from_loading(manager);
        drop(process.lease_for_loading().unwrap());
        process.note_lease_ended_without_return();
        for name in ["rulesmd.ini", "artmd.ini", "aimd.ini"] {
            std::fs::write(dir.path().join(name), b"[Late]\nValue=YR\n").unwrap();
        }
        let fresh = AssetManager::new(dir.path(), MediaArchiveMode::STOCK_DIGITAL).unwrap();
        assert!(process.initialize_sources_if_needed(&fresh).is_err());
        assert!(
            fresh.with_ini_roots(&config).is_err(),
            "new Auto choice would now be ambiguous"
        );
        let recovered = process.reconstruct_manager(dir.path()).unwrap();
        assert_eq!(recovered.ini_roots().profile(), IniProfile::Mo);
        process.initialize_sources_if_needed(&recovered).unwrap();
        assert_eq!(
            process
                .native_rules()
                .unwrap()
                .fixed_art()
                .section_or_empty("TEST")
                .read_bool("Voxel", false),
            true
        );
    }

    /// The production install over the retail sources against the native
    /// run of `0x00679D90`/`0x00679ED0`/`0x00679EC0`
    /// (tools/storage_oracle/ini_crc).
    #[test]
    fn retail_ini_hashes_match_the_native_composite() {
        let Some((_, assets)) = crate::rules::retail_ini_fixture::retail_assets() else {
            return;
        };
        let mut process = ProcessAssets::new_with_ini_config(
            MediaArchiveMode::STOCK_DIGITAL,
            false,
            Default::default(),
        );
        process.initialize_sources_if_needed(&assets).unwrap();
        let modes = crate::skirmish_modes::skirmish_modes_from_assets(&assets).unwrap();
        process.install_native_ini_hashes(&assets, &modes);
        let corpus: serde_json::Value = serde_json::from_str(crate::test_fixture::text(
            "tools/storage_oracle/ini_crc.json",
        ))
        .unwrap();
        let composite = &corpus["composite"];
        let value = |key: &str| composite[key]["unsigned"].as_u64().unwrap() as u32;
        let hashes = process.native_ini_hashes().expect("retail INI hashes");
        assert_eq!(
            (hashes.rules, hashes.ai, hashes.art),
            (value("rules"), value("ai"), value("art"))
        );
    }

    fn test_manager(label: &str) -> AssetManager {
        let dir = std::env::temp_dir().join(format!(
            "vera20k-process-assets-{}-{label}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("test asset dir");
        AssetManager::from_loose_root_for_test(&dir)
    }

    /// F11: the slot's lease/return transitions cover every loading outcome —
    /// success and failure/cancel both return the same manager, a double
    /// return keeps the resident one, and an absent slot leases nothing.
    #[test]
    fn asset_manager_lease_returns_on_success_failure_and_cancel() {
        // Absent slot: nothing to lease, nothing available.
        let mut absent = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        assert!(!absent.is_available());
        assert!(absent.lease_for_loading().is_none());

        // Success-shaped cycle: lease out, manager comes home.
        let mut assets = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        assets.return_from_loading(test_manager("cycle"));
        assert!(assets.is_available());
        let leased = assets
            .lease_for_loading()
            .expect("available manager leases");
        assert!(
            !assets.is_available(),
            "leased slot has no resident manager"
        );
        assert!(
            assets.lease_for_loading().is_none(),
            "a leased slot cannot lease again"
        );
        assets.return_from_loading(leased);
        assert!(assets.is_available(), "success returns the manager");

        // Failure/cancel-shaped cycle: identical return path.
        let leased = assets.lease_for_loading().expect("re-lease after return");
        assets.return_from_loading(leased);
        assert!(assets.is_available(), "failure/cancel returns the manager");

        // Double return: the resident manager is kept, not replaced.
        assets.return_from_loading(test_manager("stray"));
        assert!(assets.is_available());

        // A lost lease clears the lease flag so the slot can lease again.
        let _ = assets.lease_for_loading().expect("lease");
        assets.note_lease_ended_without_return();
        assert!(!assets.is_available());
        assert!(
            assets.lease_for_loading().is_none(),
            "manager is truly gone"
        );
    }

    #[test]
    fn failed_rules_recovery_retains_selected_empty_audio_and_same_sound_owner() {
        use crate::map::source::test_support::TestDirectory;
        use crate::rules::ini_parser::IniFile;
        let dir = TestDirectory::new("audio-failed-rules-recovery");
        // Missing Rules is fallible, but missing optional audio is selected-empty,
        // not an invitation to reselect it when the next manager is acquired.
        let mut owner = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        let manager = AssetManager::from_loose_root_for_test(dir.path());
        assert!(owner.initialize_sources_if_needed(&manager).is_err());
        assert!(owner.native_rules().is_none());
        let sounds = std::sync::Arc::clone(owner.audio_catalog().unwrap().definitions.sounds());
        assert!(sounds.is_empty());
        dir.write("rulesmd.ini", b"[General]\nBuildSpeed=.7\n");
        dir.write("artmd.ini", b"[Test]\n");
        dir.write("soundmd.ini", b"[SoundList]\n0=Late\n[Late]\nSounds=late\n");
        let manager = AssetManager::from_loose_root_for_test(dir.path());
        owner.initialize_sources_if_needed(&manager).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &sounds,
            owner.native_rules().unwrap().fixed_sounds()
        ));
        assert!(
            owner
                .audio_catalog()
                .unwrap()
                .sounds()
                .get("Late")
                .is_none()
        );
        for _ in 0..2 {
            owner
                .native_rules
                .as_mut()
                .unwrap()
                .load_scenario(
                    crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                    &IniFile::empty(),
                )
                .unwrap();
            owner.initialize_sources_if_needed(&manager).unwrap();
            assert!(std::sync::Arc::ptr_eq(
                &sounds,
                owner.native_rules().unwrap().fixed_sounds()
            ));
        }
    }

    #[test]
    fn gsi_04_01_process_owner_binds_grid_clones_and_map_reloads() {
        let assets = ProcessAssets::new(MediaArchiveMode::STOCK_DIGITAL, false);
        let process_dummy = assets.shared_cell_dummy.clone();
        let mut first =
            crate::map::resolved_terrain::ResolvedTerrainGrid::from_cells(0, 0, Vec::new());
        first.bind_shared_cell_dummy(process_dummy.clone());
        let first_clone = first.clone();
        first.stamp_dummy_cell_requested_coord(7, 9);
        assert!(
            first
                .shared_cell_dummy()
                .same_identity(&first_clone.shared_cell_dummy())
        );
        assert_eq!(first_clone.dummy_cell_requested_coord(), (7, 9));

        process_dummy.set_level_slope(-7, 11);
        process_dummy.reconstruct_for_map_resize();
        let mut reloaded =
            crate::map::resolved_terrain::ResolvedTerrainGrid::from_cells(0, 0, Vec::new());
        reloaded.bind_shared_cell_dummy(assets.shared_cell_dummy.clone());
        assert!(
            reloaded
                .shared_cell_dummy()
                .same_identity(&first.shared_cell_dummy())
        );
        assert_eq!(reloaded.dummy_cell_requested_coord(), (0, 0));
        assert_eq!(reloaded.dummy_cell_level_slope(), (0, 0));

        let headless_process = crate::map::resolved_terrain::SharedCellDummy::fresh();
        assert!(!headless_process.same_identity(&assets.shared_cell_dummy));
        assert_eq!(headless_process.snapshot().coord, (0, 0));
    }
}
