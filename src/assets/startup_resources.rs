//! Plain startup resource choices. Asset owners consume these values without
//! depending on gameplay modules, component runtimes or configuration state.

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupComponentIdentity {
    pub api_version: u32,
    pub program: String,
    pub sha256: String,
}

#[derive(Clone, Default)]
pub enum StartupComponentSource {
    #[default]
    Unprobed,
    Absent,
    Deferred {
        identity: StartupComponentIdentity,
    },
    Admitted {
        identity: StartupComponentIdentity,
        bytes: Arc<[u8]>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPair {
    pub index: String,
    pub bag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioCatalogPolicy {
    Retail,
    OrderedPairs { pairs: Vec<AudioPair> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsfLanguageGate {
    Ignore,
    MatchBaseOrNeutral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsfCandidate {
    pub name: String,
    pub language: CsfLanguageGate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsfCompositionPolicy {
    BaseOnly,
    OrderedRecords { candidates: Vec<CsfCandidate> },
}

pub type StringSupplementProvider =
    Arc<dyn Fn(u32) -> Result<Option<CsfCompositionPolicy>, String> + Send + Sync>;

#[derive(Clone, Default)]
pub struct StartupResources {
    pub component: StartupComponentSource,
    pub hash_include: Option<bool>,
    /// Fixed speech-catalog layout, selected before its one reader runs.
    pub eva_registered_columns: Option<bool>,
    pub audio: Option<AudioCatalogPolicy>,
    pub strings: Option<StringSupplementProvider>,
}
