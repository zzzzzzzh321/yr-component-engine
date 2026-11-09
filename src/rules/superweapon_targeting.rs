//! Target-preflight port attributed by contributor baixiong443 to their
//! earlier ra2-mono work, submitted under VERA's GPLv3. The donor source was
//! unavailable for independent review; see the documented provenance below.
//!
//! The decision is pure: commands, AI and cursor queries pass observations
//! from their existing world owners. It never spends charge or changes fog/RNG.
//! Source mapping and intentional adaptations: docs/ares-generic-warhead.md.
//! The target and house masks are read with Ares's own readers, not the
//! donor's alias parser.

#[cfg(feature = "ares")]
use crate::rules::ini_parser::IniSection;
use crate::util::native_x87::NativeF64Bits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TargetMask(u8);

impl TargetMask {
    pub const NONE: Self = Self(0);
    pub const LAND: Self = Self(1);
    pub const WATER: Self = Self(2);
    pub const EMPTY: Self = Self(4);
    pub const INFANTRY: Self = Self(8);
    pub const UNIT: Self = Self(16);
    pub const BUILDING: Self = Self(32);
    pub const ALL_CELLS: Self = Self(3);
    pub const ALL_TECHNOS: Self = Self(56);
    pub const ALL_CONTENTS: Self = Self(60);
    pub const ALL: Self = Self(255);

    /// Preserve the full byte mask, including the high bits of native ALL.
    /// Reject a wider SDK value instead of truncating it into another policy.
    pub(crate) fn from_bits(bits: u32) -> Option<Self> {
        u8::try_from(bits).ok().map(Self)
    }

    pub(crate) fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Ares SuperWeaponTarget reader10034430, tokens1009BC40; same
    /// comma-only list and keep-on-failure rules as [`HouseMask::read_ares`].
    #[cfg(feature = "ares")]
    pub(crate) fn read_ares(section: &IniSection, key: &str) -> Option<Self> {
        section.read_comma_mask(key, &TARGET_TOKENS).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct HouseMask(u8);

impl HouseMask {
    pub const NONE: Self = Self(0);
    pub const OWNER: Self = Self(1);
    pub const ALLIES: Self = Self(2);
    pub const ENEMIES: Self = Self(4);
    pub const TEAM: Self = Self(3);
    pub const ALL: Self = Self(7);

    /// Ares AffectedHouse100341D0 admits only its three relationship bits.
    pub(crate) fn from_bits(bits: u32) -> Option<Self> {
        (bits <= u32::from(Self::ALL.0)).then_some(Self(bits as u8))
    }

    pub(crate) fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Ares household-mask reader100341D0, tokens1009BCE0: successful
    /// native comma-only lists replace; absent or unknown keeps the prior.
    #[cfg(feature = "ares")]
    pub(crate) fn read_ares(section: &IniSection, key: &str) -> Option<Self> {
        section.read_comma_mask(key, &HOUSE_TOKENS).map(Self)
    }
}

/// Ares AffectedHouse names1009BCE0..1009BD20, read by100341D0.
#[cfg(feature = "ares")]
const HOUSE_TOKENS: [(&str, u8); 8] = [
    ("owner", 1),
    ("allies", 2),
    ("enemies", 4),
    ("team", 3),
    ("trump", 5),
    ("others", 6),
    ("all", 7),
    ("none", 0),
];

/// Ares SuperWeaponTarget names1009BC40..1009BC80, read by10034430.
#[cfg(feature = "ares")]
const TARGET_TOKENS: [(&str, u8); 8] = [
    ("land", 1),
    ("water", 2),
    ("empty", 4),
    ("infantry", 8),
    ("units", 0x10),
    ("buildings", 0x20),
    ("all", 0xFF),
    ("none", 0),
];

/// Ares targeting-constraint names1009B9A0..1009B9F0, read by10034290 with
/// the same comma-only list and keep-on-failure rules as the masks above.
#[cfg(feature = "ares")]
const CONSTRAINT_TOKENS: [(&str, u16); 10] = [
    ("none", 0),
    ("offensive_cell_clear", OFFENSIVE_CELL_CLEAR),
    ("defensive_cell_clear", DEFENSIVE_CELL_CLEAR),
    ("enemy", ENEMY),
    ("lightningstorm_inactive", LIGHTNING_STORM_INACTIVE),
    ("dominator_inactive", DOMINATOR_INACTIVE),
    ("attacked", ATTACKED),
    ("lowpower", LOW_POWER),
    ("offensive_cell_set", OFFENSIVE_CELL_SET),
    ("defensive_cell_set", DEFENSIVE_CELL_SET),
];

const OFFENSIVE_CELL_CLEAR: u16 = 0x1;
const DEFENSIVE_CELL_CLEAR: u16 = 0x2;
const ENEMY: u16 = 0x4;
const LIGHTNING_STORM_INACTIVE: u16 = 0x8;
const DOMINATOR_INACTIVE: u16 = 0x10;
const ATTACKED: u16 = 0x20;
const LOW_POWER: u16 = 0x40;
const OFFENSIVE_CELL_SET: u16 = 0x80;
const DEFENSIVE_CELL_SET: u16 = 0x100;

/// SWTypeExt's effective AI-targeting constraints: the explicit
/// `SW.AITargeting.Constraints` or the mode row's default.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TargetConstraints(u16);

impl TargetConstraints {
    /// Reader10034290's nine bits include both cell-set constraints (0x80,
    /// 0x100), beyond the lower seven flags used by the default mode rows.
    pub(crate) fn from_bits(bits: u32) -> Option<Self> {
        (bits <= 0x1ff).then_some(Self(bits as u16))
    }

    /// `IsTargetConstraintsEligible`10032110 with its manual argument set, as
    /// Phobos's SWButtonClass and Ares's sidebar hooks (10034E88, 10034F52)
    /// call it: the offensive-cell (+0x54F0), defensive-cell (+0x54F8) and
    /// enemy (+0x5600) tests, then the manual return at100321C4 skips the
    /// storm, dominator, attacked and low-power tests.
    pub fn allows_manual_fire(self, house: HouseTargetingObservations) -> bool {
        let bits = self.0;
        let offensive_set = house.offensive_cell != (0, 0);
        let defensive_set = false;
        !(bits & OFFENSIVE_CELL_CLEAR != 0 && offensive_set
            || bits & OFFENSIVE_CELL_SET != 0 && !offensive_set
            || bits & DEFENSIVE_CELL_CLEAR != 0 && defensive_set
            || bits & DEFENSIVE_CELL_SET != 0 && !defensive_set
            || bits & ENEMY != 0 && !house.has_enemy)
    }

    /// `IsTargetConstraintsEligible`10032110 for an automatic launch: the
    /// manual tests, then the storm flag `0x00A9FAB4` (100321D0), PsyDom
    /// Active `0x0053B400` (100321DE), a building attack (+0x54D8) more than
    /// 75 frames old (100321EC..100321FF) and a House that is not low on
    /// power (+0x53A4/+0x53A8, 10032206..10032216).
    pub fn allows_automatic_fire(
        self,
        house: HouseTargetingObservations,
        world: AutomaticConstraintObservations,
    ) -> bool {
        let bits = self.0;
        self.allows_manual_fire(house)
            && !(bits & LIGHTNING_STORM_INACTIVE != 0 && world.lightning_storm_active
                || bits & DOMINATOR_INACTIVE != 0 && world.dominator_active
                || bits & ATTACKED != 0
                    && world.last_building_attack_frame != 0
                    && world.last_building_attack_frame.wrapping_add(75) < world.frame
                || bits & LOW_POWER != 0
                    && (world.power_drain == 0 || world.power_output >= world.power_drain))
    }
}

/// The world facts [`TargetConstraints::allows_automatic_fire`] reads after
/// the manual return.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutomaticConstraintObservations {
    pub lightning_storm_active: bool,
    pub dominator_active: bool,
    /// `House+0x54D8`, 0 before the first building attack.
    pub last_building_attack_frame: i32,
    pub frame: i32,
    /// `House+0x53A4` and `House+0x53A8`.
    pub power_output: i32,
    pub power_drain: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TargetRequirements {
    pub target: TargetMask,
    pub house: HouseMask,
}

/// An Ares AITargeting mode, in the order of its names at 1009BAF8 (reader
/// 10034350), which is SWTypeExt+0x210's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiTargetingMode {
    None,
    Nuke,
    LightningStorm,
    PsychicDominator,
    ParaDrop,
    GeneticMutator,
    ForceShield,
    NoTarget,
    Offensive,
    Stealth,
    SelfTarget,
    Base,
    MultiMissile,
    HunterSeeker,
    EnemyBase,
    IronCurtain,
    Attack,
    LowPower,
    LowPowerAttack,
    DropPod,
    LightningRandom,
}

/// `SW.AITargeting.Preference`: the names at 1009BAB4 in reader order
/// (100343C0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiTargetPreference {
    None,
    Offensive,
    Defensive,
}

const AI_TARGET_PREFERENCES: [(&str, AiTargetPreference); 3] = [
    ("none", AiTargetPreference::None),
    ("offensive", AiTargetPreference::Offensive),
    ("defensive", AiTargetPreference::Defensive),
];

/// Ares's AITargeting modes: names1009BAF8 in reader order (10034350), each
/// with its row of table1009B8A4 (stride 12): the dword +0, the constraints
/// used while SW.AITargeting.Constraints is unset (10032127..10032135); the
/// preference byte +4, used while SW.AITargeting.Preference is unset; and
/// bytes +5/+6, the AI target and house masks used while
/// SW.AIRequiresTarget/SW.AIRequiresHouse are unset.
const AI_TARGETING_MODES: [(&str, AiTargetingMode, u16, u8, u8, u8); 21] = {
    use AiTargetingMode as M;
    [
        ("none", M::None, 0, 0, 0, 0),
        ("nuke", M::Nuke, 0x04, 1, 0x38, 4),
        ("lightningstorm", M::LightningStorm, 0x0C, 1, 0x38, 4),
        ("psychicdominator", M::PsychicDominator, 0x15, 0, 0x18, 7),
        ("paradrop", M::ParaDrop, 0, 1, 0, 0),
        ("geneticmutator", M::GeneticMutator, 0x01, 0, 0x08, 7),
        ("forceshield", M::ForceShield, 0, 2, 0, 0),
        ("notarget", M::NoTarget, 0, 0, 0, 0),
        ("offensive", M::Offensive, 0x04, 0, 0x38, 4),
        ("stealth", M::Stealth, 0, 0, 0x38, 4),
        ("self", M::SelfTarget, 0, 0, 0, 0),
        ("base", M::Base, 0, 0, 0, 0),
        ("multimissile", M::MultiMissile, 0x04, 1, 0x20, 4),
        ("hunterseeker", M::HunterSeeker, 0x04, 0, 0, 0),
        ("enemybase", M::EnemyBase, 0x04, 0, 0, 0),
        ("ironcurtain", M::IronCurtain, 0, 0, 0, 0),
        ("attack", M::Attack, 0x20, 0, 0, 0),
        ("lowpower", M::LowPower, 0x40, 0, 0, 0),
        ("lowpowerattack", M::LowPowerAttack, 0x60, 0, 0, 0),
        ("droppod", M::DropPod, 0x04, 0, 0, 0),
        ("lightningrandom", M::LightningRandom, 0, 0, 0, 0),
    ]
};

#[cfg(feature = "ares")]
const OFFENSIVE_MODE: u8 = 8;

/// The mode each Ares type handler's Initialize (vtable +0x14) stores at
/// SWTypeExt+0x210. LoadFromINIFile10033250 runs it on its first call with
/// the section, for a new Type or the stock Type's handler (1006CFB0).
/// ChronoSphere's (1006EF10) and ChronoWarp's (100700B0) store nothing and
/// EMPulse's stores 0 (1007146B), so they keep the constructor's 0.
#[cfg(feature = "ares")]
fn handler_initial_mode(type_name: &str) -> u8 {
    const MODES: [(&str, u8); 17] = [
        ("MultiMissile", 1),                // 100733BC
        ("IronCurtain", 15),                // 10074BFE
        ("Protect", 15),                    // 10074BFE
        ("LightningStorm", 2),              // 10072CD5
        ("ParaDrop", 4),                    // 10073F48
        ("AmerParaDrop", 4),                // 10073F48
        ("PsychicDominator", 3),            // 10070A01
        ("SpyPlane", 4),                    // 10075865
        ("GeneticConverter", 5),            // 100720DF
        ("ForceShield", 6),                 // 10074B9A
        ("PsychicReveal", 4),               // 10075336
        ("SonarPulse", 9),                  // 100756E5
        ("UnitDelivery", 4),                // 100759D4
        ("GenericWarhead", OFFENSIVE_MODE), // 10071864
        ("HunterSeeker", 13),               // 100729E1
        ("DropPod", 19),                    // 10071125
        ("Battery", 17),                    // 1006ECA4
    ];
    MODES
        .iter()
        .find_map(|(name, mode)| name.eq_ignore_ascii_case(type_name).then_some(*mode))
        .unwrap_or(0)
}

/// SWTypeExt's common targeting fields, read by every ordered rules pass in
/// LoadFromINI10033370: SW.FireIntoShroud/AutoFire/ManualFire
/// (+0x1E3/+0x1E1/+0x1E2), SW.AffectsHouse (+0x23C), SW.UseAITargeting
/// (+0x20E), the AITargeting mode (+0x210), the nullable
/// `SW.AITargeting.Constraints` (+0x214/+0x218) and `.Preference`
/// (+0x21C/+0x21D), SW.AffectsTarget (+0x240), `SW.RequiresTarget`/
/// `SW.RequiresHouse` (+0x241/+0x23D), the nullable
/// `SW.AIRequiresTarget`/`SW.AIRequiresHouse` (+0x242/+0x23E) and
/// SW.RangeMinimum/Maximum (+0x460/+0x468).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct AresSuperWeaponTargetingState {
    /// LoadFromINIFile10033250 has left state+4 = 0.
    initialized: bool,
    mode: u8,
    constraints: Option<u16>,
    preference: Option<u8>,
    use_ai_targeting: bool,
    auto_fire: bool,
    manual_fire: bool,
    fire_into_shroud: bool,
    affects_house: HouseMask,
    affects_target: TargetMask,
    manual: TargetRequirements,
    ai_target: Option<TargetMask>,
    ai_house: Option<HouseMask>,
    range_minimum: NativeF64Bits,
    range_maximum: NativeF64Bits,
}

impl Default for AresSuperWeaponTargetingState {
    fn default() -> Self {
        // SWTypeExt ctor10031460: the dword 0x01010007 at 10031733 leaves
        // AutoFire 0, ManualFire 1 and FireIntoShroud 1; 10031771 stores
        // mode 0; the dword stores of AffectsHouse 7 and AffectsTarget 0xFF
        // (100317AE/100317B8) zero both manual masks and both AI flags; both
        // ranges take binary64 -1 (1009C338) at 1003180E..10031859.
        let unbounded = NativeF64Bits::from_bits((-1.0_f64).to_bits());
        Self {
            initialized: false,
            mode: 0,
            constraints: None,
            preference: None,
            use_ai_targeting: false,
            auto_fire: false,
            manual_fire: true,
            fire_into_shroud: true,
            affects_house: HouseMask::ALL,
            affects_target: TargetMask::ALL,
            manual: TargetRequirements {
                target: TargetMask::NONE,
                house: HouseMask::NONE,
            },
            ai_target: None,
            ai_house: None,
            range_minimum: unbounded,
            range_maximum: unbounded,
        }
    }
}

impl AresSuperWeaponTargetingState {
    /// Derived adapter for the frozen external catalog. It performs no INI
    /// read and does not enable capabilities absent from the component ABI.
    pub(crate) fn from_external_config(
        constraints: TargetConstraints,
        use_ai_targeting: bool,
        common: &vera_mod_api::SuperWeaponCommonTargetingPolicy,
    ) -> Self {
        // The resource binder validates the closed declaration first. This is
        // its immutable projection into the same selector owner, never a read
        // of merged/current INI or a second gameplay-state store.
        Self {
            initialized: true,
            mode: common.mode,
            constraints: Some(constraints.0),
            preference: Some(common.preference),
            use_ai_targeting,
            auto_fire: common.auto_fire,
            manual_fire: common.manual_fire,
            fire_into_shroud: common.fire_into_shroud,
            manual: TargetRequirements {
                target: TargetMask(common.manual_target),
                house: HouseMask(common.manual_house),
            },
            ai_target: Some(TargetMask(common.automatic_target)),
            ai_house: Some(HouseMask(common.automatic_house)),
            range_minimum: NativeF64Bits::from_bits(common.range_minimum),
            range_maximum: NativeF64Bits::from_bits(common.range_maximum),
            ..Self::default()
        }
    }

    /// LoadFromINIFile10033250 with this type's section. Only its first call
    /// runs the `Type` handler's Initialize (10033337), which stores the
    /// handler's mode ([`handler_initial_mode`]); HunterSeeker's (10072990)
    /// also stores AffectsHouse=enemies (100729EB). Returns whether this call
    /// ran Initialize, whose other writes the caller's owners make.
    /// LoadFromINI10033370 then reads the fields in this order with
    /// Valueable<bool> (1000EBF0), the house (10033CE0) and target (10033D20)
    /// masks, the mode (10033D00), the Nullable constraints, preference and
    /// AI masks (10033C80, 10033CA0, 10033CC0, 10033C60), and
    /// Valueable<double> (1000EB00). The mode (10034350) and preference
    /// (100343C0) readers need the whole value to `_strcmpi` a name; anything
    /// else keeps the prior value. The Nullable wrappers set their flag only
    /// on a successful read.
    #[cfg(feature = "ares")]
    pub(crate) fn read_pass(&mut self, section: &IniSection, type_name: Option<&str>) -> bool {
        let initialize = !self.initialized;
        if initialize {
            self.mode = type_name.map_or(0, handler_initial_mode);
            if type_name.is_some_and(|name| name.eq_ignore_ascii_case("HunterSeeker")) {
                self.affects_house = HouseMask::ENEMIES;
            }
        }
        self.initialized = true;
        // 1003341A..10033457
        self.fire_into_shroud = section.read_ares_bool("SW.FireIntoShroud", self.fire_into_shroud);
        self.auto_fire = section.read_ares_bool("SW.AutoFire", self.auto_fire);
        self.manual_fire = section.read_ares_bool("SW.ManualFire", self.manual_fire);
        // 10033590
        if let Some(mask) = HouseMask::read_ares(section, "SW.AffectsHouse") {
            self.affects_house = mask;
        }
        // 100335BC..1003360F
        self.use_ai_targeting = section.read_ares_bool("SW.UseAITargeting", self.use_ai_targeting);
        if let Some(mode) = section.read_name("SW.AITargeting", 0x800).and_then(|name| {
            AI_TARGETING_MODES
                .iter()
                .position(|(mode, ..)| mode.eq_ignore_ascii_case(name))
        }) {
            self.mode = mode as u8;
        }
        if let Some(bits) =
            section.read_comma_mask("SW.AITargeting.Constraints", &CONSTRAINT_TOKENS)
        {
            self.constraints = Some(bits);
        }
        if let Some(preference) = section
            .read_name("SW.AITargeting.Preference", 0x800)
            .and_then(|name| {
                AI_TARGET_PREFERENCES
                    .iter()
                    .position(|(preference, _)| preference.eq_ignore_ascii_case(name))
            })
        {
            self.preference = Some(preference as u8);
        }
        // 10033614..1003367D
        if let Some(mask) = TargetMask::read_ares(section, "SW.AffectsTarget") {
            self.affects_target = mask;
        }
        if let Some(mask) = TargetMask::read_ares(section, "SW.RequiresTarget") {
            self.manual.target = mask;
        }
        if let Some(mask) = HouseMask::read_ares(section, "SW.RequiresHouse") {
            self.manual.house = mask;
        }
        if let Some(mask) = TargetMask::read_ares(section, "SW.AIRequiresTarget") {
            self.ai_target = Some(mask);
        }
        if let Some(mask) = HouseMask::read_ares(section, "SW.AIRequiresHouse") {
            self.ai_house = Some(mask);
        }
        // 10033942/10033958
        self.range_minimum = section.read_ares_double_bits("SW.RangeMinimum", self.range_minimum);
        self.range_maximum = section.read_ares_double_bits("SW.RangeMaximum", self.range_maximum);
        initialize
    }

    /// A pass without this type's section still ends the first call
    /// (10033284 -> 10033347), so a later pass never runs Initialize.
    #[cfg(feature = "ares")]
    pub(crate) fn skip_pass(&mut self) {
        self.initialized = true;
    }

    pub(crate) fn mode(self) -> AiTargetingMode {
        AI_TARGETING_MODES[self.mode as usize].1
    }

    pub(crate) fn use_ai_targeting(self) -> bool {
        self.use_ai_targeting
    }

    pub(crate) fn auto_fire(self) -> bool {
        self.auto_fire
    }

    pub(crate) fn manual_fire(self) -> bool {
        self.manual_fire
    }

    pub(crate) fn fire_into_shroud(self) -> bool {
        self.fire_into_shroud
    }

    /// `SW.AffectsHouse` (+0x23C).
    pub(crate) fn affects_house(self) -> HouseMask {
        self.affects_house
    }

    /// `SW.AffectsTarget` (+0x240).
    pub(crate) fn affects_target(self) -> TargetMask {
        self.affects_target
    }

    pub(crate) fn manual(self) -> TargetRequirements {
        self.manual
    }

    /// 10032110..10032135: the flagged constraints, else the mode row's.
    pub(crate) fn constraints(self) -> TargetConstraints {
        TargetConstraints(
            self.constraints
                .unwrap_or(AI_TARGETING_MODES[self.mode as usize].2),
        )
    }

    /// The flagged preference, else the mode row's byte +4.
    pub(crate) fn preference(self) -> AiTargetPreference {
        let index = self
            .preference
            .unwrap_or(AI_TARGETING_MODES[self.mode as usize].3);
        AI_TARGET_PREFERENCES[index as usize].1
    }

    /// 10032270..10032322 and100328D0: an unset AI mask falls back to its
    /// mode's row.
    pub(crate) fn automatic(self) -> TargetRequirements {
        let (_, _, _, _, target, house) = AI_TARGETING_MODES[self.mode as usize];
        TargetRequirements {
            target: self.ai_target.unwrap_or(TargetMask(target)),
            house: self.ai_house.unwrap_or(HouseMask(house)),
        }
    }

    pub(crate) fn range(self) -> (f64, f64) {
        (
            f64::from_bits(self.range_minimum.bits()),
            f64::from_bits(self.range_maximum.bits()),
        )
    }
}

/// Immutable configuration. Only explicitly opted-in extension types read it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuperWeaponTargeting {
    pub manual: TargetRequirements,
    pub automatic: TargetRequirements,
    pub fire_into_shroud: bool,
    pub range_minimum: NativeF64Bits,
    pub range_maximum: NativeF64Bits,
}

impl SuperWeaponTargeting {
    /// The masks start as a freshly initialized GenericWarhead's; RuleSet
    /// installs the ordered [`AresSuperWeaponTargetingState`] reads.
    #[cfg(feature = "ares")]
    pub fn from_ini(section: &IniSection) -> Self {
        let range = |key| {
            let value = if section.is_present(key) {
                section.read_double(key, f64::NAN)
            } else {
                -1.0
            };
            NativeF64Bits::from_bits(value.to_bits())
        };
        let mut initial = AresSuperWeaponTargetingState::default();
        initial.read_pass(&IniSection::new(String::new()), Some("GenericWarhead"));
        Self {
            manual: initial.manual(),
            automatic: initial.automatic(),
            fire_into_shroud: section.read_bool("SW.FireIntoShroud", true),
            range_minimum: range("SW.RangeMinimum"),
            range_maximum: range("SW.RangeMaximum"),
        }
    }

    pub fn range(&self) -> (f64, f64) {
        (
            f64::from_bits(self.range_minimum.bits()),
            f64::from_bits(self.range_maximum.bits()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetReject {
    InstanceNotFound,
    InvalidPayload,
    TargetRequired,
    TargetOutOfBounds,
    TargetRequiresLand,
    TargetRequiresWater,
    TargetContentMismatch,
    TargetHouseMismatch,
    TargetShrouded,
    InvalidRangeConfiguration,
    NoEligibleLaunchSite,
    NoEligibleDesignator,
    InhibitorInRange,
}

#[derive(Debug, Clone, Copy)]
pub struct TargetContent {
    pub kind: TargetMask,
    pub relation: HouseMask,
}

#[derive(Debug, Clone, Copy)]
pub struct LaunchSite {
    pub center: (i32, i32),
    pub eligible: bool,
    pub range_minimum_override: Option<f64>,
    pub range_maximum_override: Option<f64>,
}

impl LaunchSite {
    pub fn admits(self, target: (i32, i32), range: (f64, f64)) -> bool {
        let minimum = self.range_minimum_override.unwrap_or(range.0);
        let maximum = self.range_maximum_override.unwrap_or(range.1);
        if !self.eligible {
            return false;
        }
        let dx = i64::from(target.0 as i16) - i64::from(self.center.0 as i16);
        let dy = i64::from(target.1 as i16) - i64::from(self.center.1 as i16);
        // IsLaunchSiteEligible1006DA94 uses signed-word cells. f64 exactly
        // represents their squared distance; no sqrt, FMA or RNG is involved.
        // The existing squared comparison replaces native1006DAC5's sqrt;
        // subcell binary64 boundary rounding remains a numerical residual.
        let distance_squared = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
        (minimum < 0.0 || distance_squared >= minimum * minimum)
            && (maximum < 0.0 || distance_squared <= maximum * maximum)
    }
}

/// The House fields `IsTargetConstraintsEligible`10032110 reads before its
/// manual return. The second defensive cell (+0x54F8/+0x54FA) has no writer
/// in VERA (see `sim::superweapon::ai_fire`'s residuals), so it is always
/// (0, 0) here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HouseTargetingObservations {
    /// `House+0x5600` is not -1.
    pub has_enemy: bool,
    /// `House+0x54F0` scripted offensive cell, `(0, 0)` when unset.
    pub offensive_cell: (u16, u16),
}

pub struct TargetPreflightInput<'a> {
    pub target_cell: Option<(i32, i32)>,
    pub requires_target_cell: bool,
    pub requirements: &'a TargetRequirements,
    pub contents: &'a [TargetContent],
    pub target_in_bounds: bool,
    pub terrain_is_water: Option<bool>,
    pub fire_into_shroud: bool,
    pub shrouded: Option<bool>,
    pub range: (f64, f64),
    pub requires_launch_site: bool,
    pub launch_sites: &'a [LaunchSite],
}

/// Port of Evaluate: same ordered gates and rejection categories. A missing
/// terrain/visibility observation is not invented as water or shroud.
pub fn evaluate_target(input: &TargetPreflightInput<'_>) -> Result<(), TargetReject> {
    let TargetRequirements {
        target: target_mask,
        house: house_mask,
    } = *input.requirements;
    // Prepare1006D1E0 decides NeedsLaunchSite from the global getter before
    // collecting individual sites. A per-site override cannot enable that scan.
    let constrained_range = input.range.0 >= 0.0 || input.range.1 >= 0.0;
    let Some(target) = input.target_cell else {
        return if input.requires_target_cell
            || target_mask != TargetMask::NONE
            || !input.fire_into_shroud
            || constrained_range
        {
            Err(TargetReject::TargetRequired)
        } else {
            Ok(())
        };
    };
    if !input.target_in_bounds {
        return Err(TargetReject::TargetOutOfBounds);
    }
    if target_mask.intersects(TargetMask::ALL_CELLS)
        && let Some(water) = input.terrain_is_water
    {
        if water && !target_mask.intersects(TargetMask::WATER) {
            return Err(TargetReject::TargetRequiresLand);
        }
        if !water && !target_mask.intersects(TargetMask::LAND) {
            return Err(TargetReject::TargetRequiresWater);
        }
    }
    let content_mask = TargetMask(target_mask.0 & TargetMask::ALL_CONTENTS.0);
    let constrains_content = content_mask != TargetMask::NONE;
    if constrains_content || house_mask != HouseMask::NONE {
        let kind_matches =
            |content: &TargetContent| !constrains_content || content_mask.intersects(content.kind);
        let accepted = if input.contents.is_empty() {
            !constrains_content || content_mask.intersects(TargetMask::EMPTY)
        } else {
            input.contents.iter().any(|content| {
                kind_matches(content)
                    && (house_mask == HouseMask::NONE || house_mask.intersects(content.relation))
            })
        };
        if !accepted {
            return Err(
                if !constrains_content || input.contents.iter().any(kind_matches) {
                    TargetReject::TargetHouseMismatch
                } else {
                    TargetReject::TargetContentMismatch
                },
            );
        }
    }
    if !input.fire_into_shroud && input.shrouded == Some(true) {
        return Err(TargetReject::TargetShrouded);
    }
    if input.requires_launch_site && !input.launch_sites.iter().any(|site| site.eligible) {
        return Err(TargetReject::NoEligibleLaunchSite);
    }
    if constrained_range
        && !input
            .launch_sites
            .iter()
            .any(|site| site.admits(target, input.range))
    {
        return Err(TargetReject::NoEligibleLaunchSite);
    }
    Ok(())
}

#[cfg(all(test, feature = "ares"))]
#[path = "superweapon_targeting_tests.rs"]
mod tests;

#[cfg(test)]
mod sdk_admission_tests {
    use super::*;

    #[test]
    fn common_range_matches_native_prepared_gate_and_site_comparisons() {
        // Ares IsLaunchSiteEligible1006DA20 and CanFireAt1006CE19 use
        // inclusive bounds without a finite-value clamp. Prepare1006D260
        // enables the range gate only when either bound compares >= zero.
        let site = LaunchSite {
            center: (0, 0),
            eligible: true,
            range_minimum_override: None,
            range_maximum_override: None,
        };
        assert!(site.admits((3, 4), (f64::NEG_INFINITY, f64::INFINITY)));
        assert!(site.admits((3, 4), (5.0, 5.0)));
        assert!(!site.admits((3, 4), (-1.0, f64::NAN)));
        let requirements = TargetRequirements {
            target: TargetMask::NONE,
            house: HouseMask::NONE,
        };
        let mut input = TargetPreflightInput {
            target_cell: Some((3, 4)),
            requires_target_cell: true,
            requirements: &requirements,
            contents: &[],
            target_in_bounds: true,
            terrain_is_water: Some(false),
            fire_into_shroud: true,
            shrouded: Some(false),
            range: (-1.0, f64::NAN),
            requires_launch_site: false,
            launch_sites: &[],
        };
        // Neither bound activates Prepare's site scan; the raw NaN slot
        // alone must not create a global invalid-range refusal.
        assert_eq!(evaluate_target(&input), Ok(()));
        input.range = (0.0, f64::NAN);
        input.launch_sites = std::slice::from_ref(&site);
        assert_eq!(
            evaluate_target(&input),
            Err(TargetReject::NoEligibleLaunchSite)
        );
        let overridden = LaunchSite {
            range_minimum_override: Some(0.0),
            range_maximum_override: Some(10.0),
            ..site
        };
        input.range = (8.0, 2.0);
        input.launch_sites = std::slice::from_ref(&overridden);
        assert_eq!(evaluate_target(&input), Ok(()));
        let dormant = LaunchSite {
            range_maximum_override: Some(1.0),
            ..site
        };
        input.range = (-1.0, -1.0);
        input.launch_sites = std::slice::from_ref(&dormant);
        assert_eq!(evaluate_target(&input), Ok(()));
    }

    #[test]
    fn admitted_masks_keep_high_constraint_bits_without_truncating_unknown_bits() {
        assert_eq!(TargetMask::from_bits(0xff), Some(TargetMask::ALL));
        assert!(TargetMask::from_bits(0x100).is_none());
        assert_eq!(HouseMask::from_bits(7), Some(HouseMask::ALL));
        assert!(HouseMask::from_bits(8).is_none());
        let offensive = TargetConstraints::from_bits(0x80).unwrap();
        let mut observation = HouseTargetingObservations::default();
        assert!(!offensive.allows_manual_fire(observation));
        observation.offensive_cell = (5, 6);
        assert!(offensive.allows_manual_fire(observation));
        // The existing defensive-cell owner remains constructor-zero. Its
        // explicit set requirement must not be truncated to an empty mask.
        assert!(
            !TargetConstraints::from_bits(0x100)
                .unwrap()
                .allows_manual_fire(observation)
        );
        assert!(TargetConstraints::from_bits(0x1ff).is_some());
        assert!(TargetConstraints::from_bits(0x200).is_none());
    }
}
