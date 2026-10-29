//! Optional immutable resource snapshot supplied over an inherited launch pipe.
//! This is a distribution adapter, not an anti-debugging or attestation boundary.
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

static SNAPSHOT: OnceLock<Snapshot> = OnceLock::new();
pub fn valid_package_name(name: &str) -> bool {
    vera_package::normalize(name).is_ok()
}
pub fn open_package(
    path: &std::path::Path,
    public_key: &[u8; 32],
    key: [u8; 32],
    sha256: &str,
) -> Result<Arc<vera_package::Archive>> {
    vera_package::Archive::open(path, public_key, key, sha256)
}
pub const MAX_RESOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;

pub struct Snapshot {
    files: BTreeMap<String, Arc<[u8]>>,
    source: String,
    packages: Vec<Arc<vera_package::Archive>>,
    package_files: BTreeMap<String, (usize, OnceLock<Arc<[u8]>>)>,
    music_enabled: bool,
}
impl Snapshot {
    pub fn new(source: String, files: Vec<(String, Vec<u8>)>) -> Result<Self> {
        ensure!(
            !source.is_empty() && source.len() <= 128,
            "invalid community release identity"
        );
        ensure!(
            !files.is_empty() && files.len() <= 256,
            "invalid community resource count"
        );
        let mut result = BTreeMap::new();
        let mut total = 0usize;
        for (name, bytes) in files {
            ensure!(valid_name(&name), "invalid community resource name");
            ensure!(
                bytes.len() <= MAX_RESOURCE_BYTES,
                "community resource too large"
            );
            total = total
                .checked_add(bytes.len())
                .ok_or_else(|| anyhow::anyhow!("resource size overflow"))?;
            ensure!(total <= MAX_TOTAL_BYTES, "community release too large");
            ensure!(
                result
                    .insert(name.to_ascii_lowercase(), Arc::from(bytes))
                    .is_none(),
                "duplicate community resource"
            );
        }
        Ok(Self {
            files: result,
            source: format!("community:{source}"),
            packages: Vec::new(),
            package_files: BTreeMap::new(),
            music_enabled: true,
        })
    }
    /// Package content stays encrypted on disk. Legacy whole-file consumers
    /// retain only files they request; seekable consumers can use `open`.
    pub fn from_package(
        source: String,
        package: Arc<vera_package::Archive>,
        overlays: Vec<(String, Vec<u8>)>,
    ) -> Result<Self> {
        Self::from_packages(source, vec![package], overlays)
    }
    /// Independently verified optional archives may add resources, but cannot
    /// replace a resource from the main package or a launch overlay.
    pub fn from_packages(
        source: String,
        packages: Vec<Arc<vera_package::Archive>>,
        overlays: Vec<(String, Vec<u8>)>,
    ) -> Result<Self> {
        ensure!(!packages.is_empty() && packages.len() <= 4, "invalid package count");
        let package = &packages[0];
        ensure!(
            format!(
                "{}@{}",
                package.metadata().id,
                package.metadata().release_id
            ) == source,
            "package release mismatch"
        );
        let mut snapshot = if overlays.is_empty() {
            Self {
                files: BTreeMap::new(),
                source: format!("community:{source}"),
                packages: Vec::new(),
                package_files: BTreeMap::new(),
                music_enabled: true,
            }
        } else {
            Self::new(source, overlays)?
        };
        for (owner, package) in packages.iter().enumerate() {
        for entry in package.entries() {
            let name = vera_package::normalize(&entry.name)?;
            ensure!(
                !snapshot.files.contains_key(&name) && !snapshot.package_files.contains_key(&name),
                "package overlay name conflict"
            );
            snapshot.package_files.insert(name, (owner, OnceLock::new()));
            ensure!(snapshot.package_files.len() <= 65_536, "too many package resources");
        }
        }
        snapshot.packages = packages;
        Ok(snapshot)
    }
    pub fn set_music_enabled(&mut self, enabled: bool) { self.music_enabled = enabled; }
    pub fn music_enabled(&self) -> bool { self.music_enabled }
    pub fn contains(&self, name: &str) -> bool {
        vera_package::normalize(name)
            .is_ok_and(|n| self.files.contains_key(&n) || self.package_files.contains_key(&n))
    }
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.files
            .keys()
            .chain(self.package_files.keys())
            .map(String::as_str)
    }
    pub fn get(&self, name: &str) -> Option<&Arc<[u8]>> {
        self.get_bounded(name, 256 * 1024 * 1024)
            .expect("published resource is unreadable; refusing retail fallback")
    }
    pub fn get_bounded(&self, name: &str, maximum: usize) -> Result<Option<&Arc<[u8]>>> {
        let Ok(normalized) = vera_package::normalize(name) else {
            return Ok(None);
        };
        if let Some(bytes) = self.files.get(&normalized) {
            ensure!(bytes.len() <= maximum, "resource exceeds read budget");
            return Ok(Some(bytes));
        }
        let Some((owner, slot)) = self.package_files.get(&normalized) else {
            return Ok(None);
        };
        let package = &self.packages[*owner];
        ensure!(
            package.entry(&normalized).unwrap().bytes <= maximum as u64,
            "resource exceeds read budget"
        );
        if slot.get().is_none() {
            let data = package.read_file(&normalized, maximum)?;
            let _ = slot.set(Arc::from(data));
        }
        Ok(slot.get())
    }
    pub fn open(&self, name: &str) -> Result<Option<vera_package::ArchiveReader>> {
        let Some((owner, _)) = self.package_files.get(&vera_package::normalize(name)?) else {
            return Ok(None);
        };
        let package = &self.packages[*owner];
        Ok(Some(package.reader(name)?))
    }
    pub fn file_view(&self, name: &str) -> Result<Option<vera_package::ResourceView>> {
        let Some((owner, _)) = self.package_files.get(&vera_package::normalize(name)?) else {
            return Ok(None);
        };
        let package = &self.packages[*owner];
        Ok(Some(package.view(name)?))
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
pub fn install(snapshot: Snapshot) -> Result<()> {
    SNAPSHOT
        .set(snapshot)
        .map_err(|_| anyhow::anyhow!("community snapshot already installed"))
}
pub fn snapshot() -> Option<&'static Snapshot> {
    SNAPSHOT.get()
}
pub fn get(name: &str) -> Option<&'static Arc<[u8]>> {
    snapshot()?.get(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_host_paths_duplicates_and_oversize() {
        for name in ["../map.map", "a/b", "C:\\a", ".hidden", ""] {
            assert!(!valid_name(name));
        }
        assert!(
            Snapshot::new(
                "r1".into(),
                vec![("A.SHP".into(), vec![]), ("a.shp".into(), vec![])]
            )
            .is_err()
        );
        assert!(
            Snapshot::new(
                "r1".into(),
                vec![("a.shp".into(), vec![0; MAX_RESOURCE_BYTES + 1])]
            )
            .is_err()
        );
    }
    #[test]
    fn shares_case_insensitive_bytes_without_disk() {
        let s = Snapshot::new("r1".into(), vec![("UNIT.SHP".into(), vec![1, 2, 3])]).unwrap();
        assert_eq!(s.get("unit.shp").unwrap().as_ref(), &[1, 2, 3]);
        assert_eq!(s.source(), "community:r1");
    }
}
