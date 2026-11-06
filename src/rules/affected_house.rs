//! Relationship flags share a single parser with external gameplay components.

pub use vera_mod_api::relationship::AffectedHouse;

/// Valueable<AffectedHouse>: absent or rejected text retains the current mask.
#[cfg(feature = "phobos")]
pub(crate) fn read_affected_house(
    section: &crate::rules::ini_parser::IniSection,
    key: &str,
    current: AffectedHouse,
) -> AffectedHouse {
    section.read_string_with(key, 2048, current, |current, raw| {
        AffectedHouse::parse(raw).unwrap_or(current)
    })
}
