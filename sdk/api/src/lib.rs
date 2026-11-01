//! Messages shared by the engine host and external gameplay components.
//! All persistent component data is returned to the host as bounded bytes.

pub mod native_string;
pub mod relationship;
pub mod timer;

use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;
pub const MAX_STATE_BYTES: usize = 16 * 1024;
pub const MAX_CONFIG_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_OPERATIONS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    pub api_version: u32,
    pub id: String,
    pub version: String,
    pub state_version: u32,
    pub config_version: u32,
    pub hooks: Vec<HookKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    Configure,
    Attach,
    Damage,
    Tick,
}

impl HookKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Configure => "configure",
            Self::Attach => "attach",
            Self::Damage => "damage",
            Self::Tick => "tick",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityContext {
    pub id: u64,
    pub type_name: String,
    pub owner: String,
    pub armor: String,
    pub health: i32,
    pub max_health: i32,
    pub alive: bool,
    pub in_limbo: bool,
    pub immobilized: bool,
    pub temporal: bool,
    pub being_warped_out: bool,
    pub powered: bool,
    pub chrono_warp_latch: bool,
    pub ai_frozen: bool,
    pub deactivated: bool,
    pub immune: bool,
    pub type_immune: bool,
    pub invulnerable: bool,
}

pub type EntityView = EntityContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigureContext {
    pub pass: u64,
    pub source: String,
    pub final_pass: bool,
    #[serde(default)]
    pub object_types: Vec<String>,
    #[serde(default)]
    pub warheads: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachContext {
    pub frame: i32,
    pub entity: EntityContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorVerses {
    pub armor: String,
    pub multiplier_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageContext {
    pub frame: i32,
    pub entity: EntityContext,
    pub attacker: Option<u64>,
    pub source_owner: Option<String>,
    pub source_is_ally: bool,
    pub source_is_owner: bool,
    pub target_is_neutral: bool,
    pub attacker_same_type_owner: bool,
    pub warhead_name: String,
    pub warhead_temporal: bool,
    pub warhead_psychedelic: bool,
    pub incoming: i32,
    pub distance_leptons: i32,
    pub ignore_defenses: bool,
    pub no_damage: bool,
    pub max_damage: i32,
    /// Native binary64 representation, not a JSON floating-point value.
    pub armor_multiplier_bits: u64,
    pub verses: Vec<ArmorVerses>,
    pub cell_spread_bits: u64,
    pub percent_at_max_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TickContext {
    pub frame: i32,
    pub entity: EntityContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Context {
    Configure(ConfigureContext),
    Attach(AttachContext),
    Damage(DamageContext),
    Tick(TickContext),
}

impl Context {
    pub const fn kind(&self) -> HookKind {
        match self {
            Self::Configure(_) => HookKind::Configure,
            Self::Attach(_) => HookKind::Attach,
            Self::Damage(_) => HookKind::Damage,
            Self::Tick(_) => HookKind::Tick,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub api_version: u32,
    pub context: Context,
    pub config: Vec<u8>,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operation {
    AdjustHealth { entity: u64, delta: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub api_version: u32,
    pub config: Option<Vec<u8>>,
    pub state: Vec<u8>,
    pub damage: Option<i32>,
    pub operations: Vec<Operation>,
    pub projection: Option<Projection>,
}

impl Plan {
    pub fn unchanged(request: &Request) -> Self {
        Self {
            api_version: API_VERSION,
            config: None,
            state: request.state.clone(),
            damage: None,
            operations: Vec::new(),
            projection: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub meters: Vec<Meter>,
    pub tint: Option<Tint>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Meter {
    pub id: String,
    pub current: i32,
    pub maximum: i32,
    pub visible: bool,
    pub active: bool,
    pub color_rgb: [u8; 3],
    pub pip_frame: u32,
    pub building_pip_frame: u32,
    pub building_empty_frame: u32,
    pub bracket_delta: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tint {
    pub color_rgb: [u8; 3],
    pub intensity: i32,
    /// Generic relationship visibility: owner=1, allies=2, enemies=4, neutral=8.
    pub visible_to_houses: u8,
}

impl ModuleManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported Mod API version".into());
        }
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        {
            return Err("invalid module identity".into());
        }
        if self.version.is_empty()
            || self.version.len() > 64
            || self.version.chars().any(char::is_control)
            || self.state_version == 0
            || self.config_version == 0
        {
            return Err("invalid module schema version".into());
        }
        let hooks: std::collections::BTreeSet<_> = self.hooks.iter().collect();
        if hooks.is_empty() || hooks.len() != self.hooks.len() {
            return Err("invalid module hook list".into());
        }
        Ok(())
    }
}

impl Request {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported request API version".into());
        }
        if self.state.len() > MAX_STATE_BYTES || self.config.len() > MAX_CONFIG_BYTES {
            return Err("component input state/config exceeds its limit".into());
        }
        Ok(())
    }
    pub fn entity_id(&self) -> Option<u64> {
        match &self.context {
            Context::Configure(_) => None,
            Context::Attach(c) => Some(c.entity.id),
            Context::Damage(c) => Some(c.entity.id),
            Context::Tick(c) => Some(c.entity.id),
        }
    }
}

impl Plan {
    /// Structural capability checks precede every caller's atomic commit.
    /// Semantic legality, lifecycle routing and final health clamping stay host-owned.
    pub fn validate_for(&self, request: &Request) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported plan API version".into());
        }
        if self.state.len() > MAX_STATE_BYTES
            || self
                .config
                .as_ref()
                .is_some_and(|c| c.len() > MAX_CONFIG_BYTES)
        {
            return Err("component output state/config exceeds its limit".into());
        }
        if self.config.is_some() && request.context.kind() != HookKind::Configure {
            return Err("only configure may replace module configuration".into());
        }
        if self.damage.is_some() && request.context.kind() != HookKind::Damage {
            return Err("only damage may replace receiver damage".into());
        }
        if self.operations.len() > MAX_OPERATIONS {
            return Err("component operation count exceeds its limit".into());
        }
        for op in &self.operations {
            match op {
                Operation::AdjustHealth { entity, .. } if Some(*entity) == request.entity_id() => {}
                _ => return Err("component operation targets another entity".into()),
            }
        }
        if let Some(projection) = &self.projection {
            if projection.meters.len() > 4 {
                return Err("component meter count exceeds its limit".into());
            }
            let mut ids = std::collections::BTreeSet::new();
            for meter in &projection.meters {
                if meter.id.is_empty()
                    || meter.id.len() > 64
                    || meter.id.chars().any(char::is_control)
                    || !ids.insert(&meter.id)
                    || meter.maximum < 0
                    || meter.visible && meter.maximum == 0
                {
                    return Err("invalid component meter".into());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageKernelInput {
    pub damage: i32,
    pub cell_spread_bits: u64,
    pub percent_at_max_bits: u64,
    pub multiplier_bits: u64,
    pub distance_leptons: i32,
    pub no_damage: bool,
    pub max_damage: i32,
}
