//! Physical Warhead extension visits and their immutable impact settings.
//! Registry ownership and lookup prefixes remain with the host's Rules reader.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarheadImpactRegistryView {
    /// Native Warhead array order. Equal stored IDs remain distinct slots.
    pub warheads: Vec<String>,
    pub reads: Vec<WarheadImpactReadPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarheadImpactReadPoint {
    pub ordinal: u32,
    /// Absent sections still reach first initialization, but read no keys.
    pub body_read: bool,
}

impl WarheadImpactRegistryView {
    pub fn validate(&self) -> Result<(), String> {
        if self.warheads.iter().any(|name| !valid_stored_id(name))
            || self.reads.iter().any(|point| {
                usize::try_from(point.ordinal)
                    .ok()
                    .is_none_or(|ordinal| ordinal >= self.warheads.len())
            })
        {
            return Err("invalid Warhead impact registry".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarheadImpactDeclaration {
    pub ordinal: u32,
    pub stored_id: String,
    pub flash_duration: Option<i32>,
    /// Native Anim array ordinal; no final-name rebinding or allocation.
    pub pre_impact: Option<u32>,
}

impl WarheadImpactDeclaration {
    pub fn validate(&self) -> Result<(), String> {
        if !valid_stored_id(&self.stored_id) {
            return Err("invalid Warhead impact stored ID".into());
        }
        Ok(())
    }
}

fn valid_stored_id(name: &str) -> bool {
    !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
}
