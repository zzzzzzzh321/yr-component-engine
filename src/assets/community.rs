//! Optional immutable resource snapshot supplied over an inherited launch pipe.
//! This is a distribution adapter, not an anti-debugging or attestation boundary.
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

static SNAPSHOT: OnceLock<Snapshot> = OnceLock::new();
pub const MAX_RESOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;

pub struct Snapshot {
    files: BTreeMap<String, Arc<[u8]>>,
    source: String,
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
        })
    }
    pub fn get(&self, name: &str) -> Option<&Arc<[u8]>> {
        self.files.get(&name.to_ascii_lowercase())
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
