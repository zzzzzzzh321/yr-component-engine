//! Community launch protocol v1: little-endian u32 length + UTF-8 JSON via stdin.
//! The launcher keeps package encryption and accounts outside the GPL engine.
use super::{
    App,
    frontend::{
        cncnet_spawn::{SpawnGame, SpawnRequest},
        startup_options::RetailStartupOptions,
    },
};
use crate::assets::community::{self, Snapshot};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use std::io::Read;

const MAX_FRAME: usize = 48 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    protocol: u32,
    release: String,
    spawn: String,
    resources: Vec<Resource>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resource {
    name: String,
    data: String,
}

fn decode(mut reader: impl Read) -> Result<(SpawnRequest, Snapshot)> {
    let mut header = [0; 4];
    reader
        .read_exact(&mut header)
        .context("community launch pipe header")?;
    let size = u32::from_le_bytes(header) as usize;
    ensure!(
        (1..=MAX_FRAME).contains(&size),
        "invalid community launch frame size"
    );
    let mut bytes = vec![0; size];
    reader
        .read_exact(&mut bytes)
        .context("incomplete community launch frame")?;
    let launch: Launch =
        serde_json::from_slice(&bytes).context("invalid community launch frame")?;
    ensure!(
        launch.protocol == 1,
        "unsupported community launch protocol"
    );
    ensure!(
        launch.spawn.len() <= 64 * 1024,
        "community launch settings too large"
    );
    let request = SpawnRequest::parse(&launch.spawn)?;
    let snapshot = Snapshot::new(
        launch.release,
        launch
            .resources
            .into_iter()
            .map(|r| {
                ensure!(
                    r.data.len() <= 24 * 1024 * 1024,
                    "encoded resource too large"
                );
                Ok((
                    r.name,
                    STANDARD
                        .decode(r.data)
                        .context("invalid resource encoding")?,
                ))
            })
            .collect::<Result<Vec<_>>>()?,
    )?;
    match &request.game {
        SpawnGame::Skirmish(s) => {
            let name = s
                .selected_map_file
                .as_deref()
                .context("missing community map")?;
            ensure!(
                community::valid_name(name) && snapshot.get(name).is_some(),
                "community map absent from snapshot"
            );
        }
        SpawnGame::Campaign(_) => {
            anyhow::bail!("community protocol v1 supports offline skirmish only")
        }
    }
    Ok((request, snapshot))
}
impl App {
    pub fn new_community(options: RetailStartupOptions, reader: impl Read) -> Result<Self> {
        ensure!(
            std::env::var_os("RA2_QUICKPLAY").is_none(),
            "community launch conflicts with RA2_QUICKPLAY"
        );
        let (request, snapshot) = decode(reader)?;
        community::install(snapshot)?;
        log::info!("Community snapshot installed from inherited pipe; entering external skirmish");
        let mut app = Self::new(options);
        app.spawn_request = Some(request);
        Ok(app)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_and_truncated_frames_before_game_creation() {
        assert!(decode((MAX_FRAME as u32 + 1).to_le_bytes().as_slice()).is_err());
        assert!(decode([5, 0, 0, 0, b'{'].as_slice()).is_err());
    }
}
