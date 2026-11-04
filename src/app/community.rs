//! Community launch protocol v1: little-endian u32 length + UTF-8 JSON via stdin.
//! The launcher keeps package encryption and accounts outside the GPL engine.
use super::{
    App,
    frontend::{
        cncnet_spawn::{PendingSpawnRequest, SpawnGame},
        startup_options::RetailStartupOptions,
    },
};
use crate::assets::community::{self, Snapshot};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use std::io::Read;
use zeroize::Zeroize;

const MAX_FRAME: usize = 48 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    protocol: u32,
    release: String,
    spawn: String,
    resources: Vec<Resource>,
    #[serde(default)]
    package: Option<PackageLaunch>,
    #[serde(default)]
    packages: Vec<PackageLaunch>,
    #[serde(default, rename = "musicEnabled")]
    music_enabled: Option<bool>,
    #[serde(default)]
    network: Option<super::shell_wol::ExternalNetworkLaunch>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PackageLaunch {
    path: std::path::PathBuf,
    sha256: String,
    public_key: String,
    key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resource {
    name: String,
    data: String,
}

pub(super) fn decode(
    mut reader: impl Read,
) -> Result<(
    PendingSpawnRequest,
    Snapshot,
    Option<super::shell_wol::ExternalNetworkLaunch>,
)> {
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
    let mut launch: Launch =
        serde_json::from_slice(&bytes).context("invalid community launch frame")?;
    ensure!(
        launch.protocol == 1 || launch.protocol == 2,
        "unsupported community launch protocol"
    );
    ensure!(
        launch.spawn.len() <= 64 * 1024,
        "community launch settings too large"
    );
    let request = PendingSpawnRequest::parse(&launch.spawn)?;
    if let Some(network) = launch.network.as_ref() {
        network.validate()?;
        ensure!(
            matches!(request.game(), SpawnGame::Skirmish(_)),
            "network launch requires skirmish settings"
        );
    }
    let resources = launch
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
        .collect::<Result<Vec<_>>>()?;
    ensure!(launch.packages.len() <= 3, "too many optional packages");
    let mut snapshot = if launch.protocol == 2 {
        let mut package = launch
            .package
            .take()
            .context("community protocol v2 requires package")?;
        let key: [u8; 32] = STANDARD
            .decode(&package.key)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid package key"))?;
        package.key.zeroize();
        let public_key: [u8; 32] = STANDARD
            .decode(&package.public_key)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid package public key"))?;
        let archive = vera20k_core::assets::community::open_package(
            &package.path,
            &public_key,
            key,
            &package.sha256,
        )?;
        let mut archives = vec![archive];
        for mut extra in launch.packages {
            let key: [u8; 32] = STANDARD
                .decode(&extra.key)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid package key"))?;
            extra.key.zeroize();
            ensure!(
                extra.public_key == package.public_key,
                "optional package signer mismatch"
            );
            archives.push(community::open_package(
                &extra.path,
                &public_key,
                key,
                &extra.sha256,
            )?);
        }
        Snapshot::from_packages(launch.release, archives, resources)?
    } else {
        ensure!(
            launch.package.is_none()
                && launch.packages.is_empty()
                && launch.music_enabled.is_none(),
            "legacy protocol cannot include package"
        );
        Snapshot::new(launch.release, resources)?
    };
    snapshot.set_music_enabled(launch.music_enabled.unwrap_or(true));
    match request.game() {
        SpawnGame::Skirmish(s) => {
            let name = s
                .selected_map_file
                .as_deref()
                .context("missing community map")?;
            ensure!(
                (if launch.protocol == 2 {
                    vera20k_core::assets::community::valid_package_name(name)
                } else {
                    community::valid_name(name)
                }) && snapshot.contains(name),
                "community map absent from snapshot"
            );
        }
        SpawnGame::Campaign(_) => {
            anyhow::bail!("community protocol v1 supports offline skirmish only")
        }
    }
    Ok((request, snapshot, launch.network))
}
impl App {
    pub fn new_community(options: RetailStartupOptions, reader: impl Read) -> Result<Self> {
        ensure!(
            std::env::var_os("RA2_QUICKPLAY").is_none(),
            "community launch conflicts with RA2_QUICKPLAY"
        );
        let (request, snapshot, network) = decode(reader)?;
        community::install(snapshot)?;
        log::info!("Community snapshot installed from inherited pipe; entering external game");
        let mut app = Self::new(options);
        app.pending_spawn = Some(request);
        app.community_network = network;
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
