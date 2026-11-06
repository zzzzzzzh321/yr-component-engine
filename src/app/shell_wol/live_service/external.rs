//! External desktop room handoff. The existing MatchSession, online worker,
//! map agreement and relay transport retain their native ownership.
use super::*;
use crate::app::frontend::cncnet_spawn::{SpawnGame, SpawnRequest};
use crate::skirmish_launch::{
    LaunchCountry, LaunchHumanRole, LaunchStartPosition, LaunchTeam, SkirmishHumanSlot,
    SkirmishLocalSlot,
};
use crate::ui::multiplayer_setup::{SetupDialog, SetupFlavor};

// No Debug: the inherited pipe contains a short-lived bearer credential.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExternalNetworkLaunch {
    service_url: String,
    session: wire::Session,
    cursor: u64,
    channel: wire::Channel,
    members: Vec<wire::Member>,
    seed: u32,
    map_sha256: String,
    players: Vec<Player>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Player {
    player_id: String,
    name: String,
    row: u8,
    country: i32,
    color: u8,
    start: i32,
    team: u8,
}

fn service_base(base: &str) -> Result<()> {
    ensure!(
        !base
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || "@?#\\".contains(ch)),
        "invalid external service URL"
    );
    if let Some(local) = base.strip_prefix("http://127.0.0.1:") {
        let (port, path) = local
            .split_once('/')
            .ok_or_else(|| anyhow!("external loopback service needs its work path"))?;
        ensure!(
            port.parse::<u16>().is_ok_and(|port| port > 0)
                && path.starts_with("native/")
                && path
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || b"/-_".contains(&ch)),
            "invalid external loopback service URL"
        );
    } else {
        ensure!(
            base.starts_with("https://")
                && base.len() > 8
                && base.len() <= 2048
                && !base[8..].starts_with('/'),
            "external service requires HTTPS"
        );
    }
    Ok(())
}
impl ExternalNetworkLaunch {
    pub(crate) fn validate(&self) -> Result<()> {
        service_base(&self.service_url)?;
        validate_online_identity(&self.session.player_id, &self.session.nickname)?;
        let now = crate::util::time::SystemTime::now()
            .duration_since(crate::util::time::UNIX_EPOCH)?
            .as_secs();
        ensure!(
            self.session.expires_at > now
                && self.session.token.len() == 43
                && base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(&self.session.token)
                    .is_ok_and(|bytes| bytes.len() == 32),
            "invalid external session credential"
        );
        ensure!(
            self.channel.kind == 41
                && !self.channel.name.is_empty()
                && self.channel.name.len() <= 96
                && self
                    .channel
                    .name
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || b"_-".contains(&ch))
                && self.channel.tournament == 0
                && self.channel.game_flags == 0,
            "invalid external room identity"
        );
        ensure!(
            (2..=8).contains(&self.players.len()) && self.members.len() == self.players.len(),
            "external room requires 2-8 admitted humans"
        );
        ensure!(
            crate::map::source::valid_source_sha256(&self.map_sha256),
            "external map digest missing"
        );
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        let mut rows = std::collections::BTreeSet::new();
        let mut colors = std::collections::BTreeSet::new();
        let mut starts = std::collections::BTreeSet::new();
        for player in &self.players {
            validate_online_identity(&player.player_id, &player.name)?;
            ensure!(
                ids.insert(&player.player_id)
                    && names.insert(player.name.to_ascii_lowercase())
                    && rows.insert(player.row)
                    && player.row < 8
                    && player.color < 8
                    && colors.insert(player.color)
                    && (-1..=65535).contains(&player.country)
                    && (-1..=7).contains(&player.start)
                    && player.team <= 4
                    && (player.start < 0 || starts.insert(player.start)),
                "invalid external player assignments"
            );
            ensure!(
                self.members
                    .iter()
                    .filter(|m| m.player_id == player.player_id && m.name == player.name)
                    .count()
                    == 1,
                "external membership differs from assigned players"
            );
        }
        ensure!(
            self.members.iter().filter(|m| m.flags & 1 != 0).count() == 1
                && self
                    .players
                    .iter()
                    .filter(|p| p.player_id == self.session.player_id
                        && p.name == self.session.nickname)
                    .count()
                    == 1,
            "external room host/local identity invalid"
        );
        Ok(())
    }

    fn apply_players(
        &self,
        launch: &mut crate::skirmish_launch::SkirmishLaunchSession,
    ) -> Result<()> {
        // The offline parser represents alliances by the smallest allied raw
        // row, while the external room exposes four display team numbers.
        // Map the complete template graph before replacing dummy human rows;
        // otherwise an AI can accidentally acquire another human's team.
        let template_teams: BTreeMap<u8, LaunchTeam> = launch
            .humans
            .iter()
            .filter_map(|human| human.raw_slot_index.map(|row| (row, human.slot.team)))
            .chain(
                launch
                    .opponents
                    .iter()
                    .map(|ai| (ai.raw_slot_index, ai.team)),
            )
            .collect();
        let mut groups = BTreeMap::new();
        let mut used: std::collections::BTreeSet<u8> = self
            .players
            .iter()
            .filter(|player| player.team > 0)
            .map(|player| player.team - 1)
            .collect();
        for player in &self.players {
            let team = template_teams
                .get(&player.row)
                .ok_or_else(|| anyhow!("external human has no template row"))?;
            if let LaunchTeam::Team(group) = team {
                ensure!(
                    player.team > 0,
                    "external team differs from template alliances"
                );
                let selected = player.team - 1;
                ensure!(
                    groups
                        .insert(*group, selected)
                        .is_none_or(|old| old == selected),
                    "external allied humans selected different teams"
                );
            }
            for other in &self.players {
                if other.row != player.row && player.team > 0 && other.team == player.team {
                    ensure!(
                        matches!((team, template_teams.get(&other.row)), (LaunchTeam::Team(left), Some(LaunchTeam::Team(right))) if left == right),
                        "external human team is absent from template alliances"
                    );
                }
            }
        }
        for team in template_teams.values() {
            if let LaunchTeam::Team(group) = team
                && !groups.contains_key(group)
            {
                let selected = (0..4)
                    .find(|value| !used.contains(value))
                    .ok_or_else(|| anyhow!("external alliances exceed four teams"))?;
                used.insert(selected);
                groups.insert(*group, selected);
            }
        }
        for ai in &mut launch.opponents {
            if let LaunchTeam::Team(group) = ai.team {
                ai.team = LaunchTeam::Team(groups[&group]);
            }
        }
        launch.humans = self
            .players
            .iter()
            .map(|player| SkirmishHumanSlot {
                raw_slot_index: Some(player.row),
                name: player.name.clone(),
                role: LaunchHumanRole::default(),
                slot: SkirmishLocalSlot {
                    country: LaunchCountry::from_country_index(player.country.max(0) as u32),
                    country_random: player.country < 0,
                    color_index: player.color,
                    color_random: false,
                    start_position: if player.start < 0 {
                        LaunchStartPosition::Auto
                    } else {
                        LaunchStartPosition::Position(player.start as u8)
                    },
                    team: if player.team == 0 {
                        LaunchTeam::None
                    } else {
                        LaunchTeam::Team(player.team - 1)
                    },
                },
            })
            .collect();
        launch.local_player_index = self
            .players
            .iter()
            .position(|p| p.player_id == self.session.player_id)
            .ok_or_else(|| anyhow!("external local player absent"))?;
        launch
            .opponents
            .retain(|ai| !self.players.iter().any(|p| p.row == ai.raw_slot_index));
        launch.ai_player_count = launch.opponents.len() as i32;
        ensure!(
            launch.humans.len() + launch.opponents.len() <= 8,
            "external room exceeds eight player palette"
        );
        let mut colors = std::collections::BTreeSet::new();
        for color in launch
            .humans
            .iter()
            .map(|p| p.slot.color_index)
            .chain(launch.opponents.iter().map(|p| p.color_index))
        {
            ensure!(colors.insert(color), "external player colors overlap");
        }
        launch.rebuild_house_roster();
        Ok(())
    }
}

#[cfg(not(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown")))]
pub(crate) fn start_external_network(
    state: &mut AppState,
    request: SpawnRequest,
    input: ExternalNetworkLaunch,
) -> Result<()> {
    input.validate()?;
    ensure!(
        state.frontend.main_menu_shell_error.is_none(),
        "retail assets are unavailable"
    );
    let SpawnGame::Skirmish(mut launch) = request.game else {
        anyhow::bail!("external network requires a skirmish template");
    };
    input.apply_players(&mut launch)?;
    let file = launch
        .selected_map_file
        .as_deref()
        .ok_or_else(|| anyhow!("external map missing"))?
        .to_owned();
    crate::map::source::validate_network_map_name(&file)?;
    let root = &state
        .platform
        .game_config
        .as_ref()
        .ok_or_else(|| anyhow!("retail root missing"))?
        .paths
        .ra2_dir;
    let assets = state
        .process_assets
        .manager()
        .ok_or_else(|| anyhow!("assets missing"))?;
    let loaded = crate::map::source::load_network_map_ini(root, &file, assets)?;
    ensure!(
        loaded
            .source
            .physical_identity()
            .is_some_and(|(_, digest)| digest == input.map_sha256),
        "external map bytes differ from frozen room"
    );
    let record = crate::map::skirmish_scenarios::SkirmishScenarioRecord::concrete_from_ini(
        0,
        crate::map::skirmish_scenarios::SkirmishScenarioSource::Synthetic,
        &file,
        &loaded.ini,
    );
    {
        let mut records = state.frontend.scenario_catalog.records_mut();
        records.retain(|record| !record.file_name.eq_ignore_ascii_case(&file));
        records.push(record);
    }
    let local = input
        .members
        .iter()
        .find(|member| member.player_id == input.session.player_id)
        .ok_or_else(|| anyhow!("external local membership missing"))?;
    let host = local.flags & 1 != 0;
    state.frontend.external_launch = true;
    state.persistence.options_profile.game_speed = request.game_speed;
    crate::app::App::destroy_current_shell_dialog(state);
    state.frontend.shell_first_paint_slide = None;
    // External startup bypasses WOL_Main, which normally prepares these
    // shared atlases before C2/C9 or connection verification can draw.
    ensure!(
        crate::app::App::ensure_skirmish_shell_chrome(state),
        "external network verification art unavailable"
    );
    crate::app::App::ensure_wol_welcome_art(state);
    state.frontend.startup_rng_seed = input.seed;
    state.frontend.wol_account.nickname = input.session.nickname.clone();
    state.frontend.wol_account.password.clear();
    state
        .frontend
        .multiplayer_shell_runtime
        .accept_online_handle(&input.session.nickname)?;
    let mut service = LiveService::with_base(input.service_url)?;
    service.player_id = input.session.player_id.clone();
    service.member_ids = input
        .members
        .iter()
        .map(|m| (m.name.to_ascii_lowercase(), m.player_id.clone()))
        .collect();
    service.external_start_deadline = Some(Instant::now() + Duration::from_secs(180));
    let members = input
        .members
        .into_iter()
        .map(|member| member.into_member(&input.session.nickname))
        .collect();
    let mut visit = super::super::WolVisit::open(false);
    visit.provider = WolProvider::Online;
    visit.mode = Some(super::super::WolMode::CustomMatch);
    visit.authenticated = true;
    visit.connected = true;
    visit.api = true;
    visit.page = super::super::WolPage::Lobby;
    visit.lobby = Some(super::super::lobby::external_room(
        input.channel.into(),
        members,
    ));
    state.frontend.wol = Some(visit);
    state
        .match_state
        .session
        .enter_network(crate::app::match_runtime::session::NetworkReturnContext::Wol);
    crate::app::frontend::skirmish_session::prepare_external_skirmish_countries(
        &mut state.process_assets,
        &launch,
    )?;
    crate::app::shell_network_setup::install_setup(
        state,
        *launch,
        SetupDialog {
            host,
            flavor: SetupFlavor::Wol,
            modern: true,
        },
        [crate::app::match_runtime::session::NetworkVacancy::Closed;
            crate::skirmish_launch::NETWORK_PLAYER_SLOT_COUNT],
        None,
    )?;
    state
        .match_state
        .session
        .set_network_start_seed(input.seed)?;
    service
        .requests
        .send(http::Request::AdoptRoomSession {
            session: input.session,
            cursor: input.cursor,
        })
        .map_err(|_| anyhow!("external session worker unavailable"))?;
    if host {
        service.command("options", json!({"options":options(state)?}));
    }
    state
        .frontend
        .wol
        .as_mut()
        .expect("external visit")
        .live_service = Some(service);
    log::info!("External multiplayer room admitted; verifying map, rules and relay");
    Ok(())
}
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_os = "unknown"))]
pub(crate) fn start_external_network(
    _state: &mut AppState,
    _request: SpawnRequest,
    _input: ExternalNetworkLaunch,
) -> Result<()> {
    anyhow::bail!("external multiplayer is unavailable in the single-player browser build")
}

pub(super) fn poll_start(state: &mut AppState, service: &mut LiveService) -> Result<()> {
    let Some(deadline) = service.external_start_deadline else {
        return Ok(());
    };
    if service.match_id.is_some() {
        service.external_start_deadline = None;
        return Ok(());
    }
    ensure!(
        Instant::now() < deadline,
        "external multiplayer admission timed out"
    );
    if service.accepted_options.is_none()
        || service.pending.is_some()
        || local_map_status(state, service) != 1
    {
        return Ok(());
    }
    let session = &state.match_state.session;
    let local = session
        .network_setup()
        .ok_or_else(|| anyhow!("external setup missing"))?
        .local_player_index;
    if !session.network_player_ready(local)
        && service
            .last_ready_request
            .is_none_or(|(_, revision)| revision != service.revision)
    {
        // Native Accept commits this local observation before sending Ready.
        // The lobby broadcasts readiness only to other members, never its sender.
        state
            .match_state
            .session
            .set_network_player_ready(local, true)?;
        request_setup(state, service, NetworkSetupRequest::Ready(true))?;
    }
    if state.match_state.session.host()
        && service.start_preparation.is_none()
        && state
            .match_state
            .session
            .network_setup()
            .is_some_and(|launch| {
                (0..launch.humans.len()).all(|index| {
                    state.match_state.session.network_player_ready(index)
                        && state.match_state.session.network_map_status(index) == Some(1)
                })
            })
    {
        crate::app::shell_network_setup::request_external_start(state)?;
    }
    Ok(())
}

pub(super) fn observe_event(state: &AppState, service: &LiveService, kind: &str) {
    if !state.frontend.external_launch
        || !crate::app::match_diagnostics::MatchDiagnosticsState::external_network_audit_enabled()
        || !matches!(
            kind,
            "options"
                | "ready"
                | "map_status"
                | "map_available"
                | "game_started"
                | "start_confirmed"
                | "timing_confirmed"
        )
    {
        return;
    }
    let session = &state.match_state.session;
    let count = session
        .network_setup()
        .map_or(0, |launch| launch.humans.len());
    crate::app::match_diagnostics::MatchDiagnosticsState::external_network_audit(json!({
        "event":"handshake", "kind":kind, "revision":service.revision,
        "localSource":session.network_setup().map(|launch|launch.local_player_index),
        "verified":local_map_status(state, service),
        "ready":(0..count).map(|index|session.network_player_ready(index)).collect::<Vec<_>>(),
        "maps":(0..count).map(|index|session.network_map_status(index)).collect::<Vec<_>>()
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn handoff() -> ExternalNetworkLaunch {
        serde_json::from_value(json!({
            "serviceUrl":"http://127.0.0.1:47836/native/work", "cursor":7,"seed":42,"mapSha256":"a".repeat(64),
            "session":{"player_id":"22222222-2222-2222-2222-222222222222","nickname":"Guest","token":base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0;32]),"expires_at":u64::MAX},
            "channel":{"name":"vera-room","kind":41,"max_users":8,"users":2},
            "members":[{"player_id":"11111111-1111-1111-1111-111111111111","name":"Host","flags":1},{"player_id":"22222222-2222-2222-2222-222222222222","name":"Guest","flags":0}],
            "players":[{"player_id":"11111111-1111-1111-1111-111111111111","name":"Host","row":0,"country":0,"color":0,"start":-1,"team":0},{"player_id":"22222222-2222-2222-2222-222222222222","name":"Guest","row":1,"country":-1,"color":1,"start":-1,"team":2}]
        })).unwrap()
    }
    #[test]
    fn authenticated_roster_cannot_substitute_guest_or_host_identity() {
        let mut input = handoff();
        assert!(input.validate().is_ok());
        input.players[1].player_id = input.players[0].player_id.clone();
        assert!(input.validate().is_err());
        let mut input = handoff();
        input.members[1].flags = 1;
        assert!(input.validate().is_err());
    }
    #[test]
    fn human_handoff_replaces_dummy_ai_and_preserves_other_ai_assignments() {
        let fixture: Value = serde_json::from_str(crate::test_fixture::text(
            "tools/map_observation.command-bar.example.json",
        ))
        .unwrap();
        let mut launch: crate::skirmish_launch::SkirmishLaunchSession =
            serde_json::from_value(fixture["launch"].clone()).unwrap();
        // Dummy Guest and the real AI share parser group1 (smallest allied
        // row1); the external room calls their alliance Team4.
        launch.opponents[0].team = LaunchTeam::Team(1);
        let mut extra = launch.opponents[0].clone();
        extra.raw_slot_index = 2;
        extra.color_index = 2;
        launch.opponents.push(extra.clone());
        let mut input = handoff();
        input.players[1].team = 4;
        input.apply_players(&mut launch).unwrap();
        assert_eq!(launch.local_player_index, 1);
        assert_eq!(launch.humans[1].name, "Guest");
        assert!(launch.humans[1].slot.country_random);
        assert_eq!(launch.humans[1].slot.team, LaunchTeam::Team(3));
        extra.team = LaunchTeam::Team(3);
        assert_eq!(launch.opponents, vec![extra]);
        assert_eq!(launch.ai_player_count, 1);
    }
    #[test]
    fn external_services_only_allow_https_or_literal_loopback_work_mounts() {
        assert!(service_base("https://example.org/native/work").is_ok());
        assert!(service_base("http://127.0.0.1:47836/native/work").is_ok());
        assert!(service_base("http://192.168.1.2:47836/native/work").is_err());
        assert!(service_base("http://127.0.0.1.evil:47836/native/work").is_err());
        assert!(service_base("https://a@evil/native/work").is_err());
    }
}
