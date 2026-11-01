//! Features are selected at compile time and composed by component.rs.
//! Each feature owns its configuration/decisions; shared host imports live in support.
#[cfg(feature = "ares-ammo")]
pub(crate) mod ammo;
pub(crate) mod shield;
