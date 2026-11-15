//! WH impact read-point facts in the existing native Rules traversal.
use super::*;
use vera_mod_api::{SuperReadValue, WarheadImpactReadPoint, WarheadImpactRegistryView};

impl RulesPassProcessor {
    pub(super) fn record_sdk_warhead_impact_read(&mut self, index: usize, pass: &IniFile) {
        let owner = self.families[&RulesTypeFamily::Warhead][index]
            .native_stored_id
            .clone();
        self.sdk_type_reads.push(SdkRegistryRead {
            registry: "$warhead-impact",
            warhead_impact: Some(WarheadImpactReadPoint {
                ordinal: u32::try_from(index).expect("native Warhead ordinal exceeds u32"),
                body_read: pass.section(&owner).is_some(),
            }),
            owner,
            animation_count: self.family_len(RulesTypeFamily::Animation),
        });
    }
}

impl SdkRegistryPass {
    pub(crate) fn warhead_impact_view(&self) -> WarheadImpactRegistryView {
        WarheadImpactRegistryView {
            warheads: self.warheads.clone(),
            reads: self
                .reads
                .iter()
                .filter_map(|read| read.warhead_impact)
                .collect(),
        }
    }

    fn warhead_impact_point(&self, read: u32, owner: &str) -> Result<&SdkRegistryRead, String> {
        let point = self
            .reads
            .iter()
            .filter(|point| point.warhead_impact.is_some())
            .nth(read as usize)
            .ok_or("invalid Warhead impact read ordinal")?;
        let identity = point.warhead_impact.expect("filtered Warhead impact point");
        if point.owner != owner
            || self
                .warheads
                .get(identity.ordinal as usize)
                .map(String::as_str)
                != Some(owner)
            || point.animation_count > self.animations.len()
        {
            return Err("Warhead impact reader requires its actual native slot and owner".into());
        }
        Ok(point)
    }

    pub(crate) fn read_warhead_impact_value(
        &self,
        reader: &str,
        owner: &str,
        ini: &IniFile,
        key: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, String> {
        // Only the read ordinal is shared across these typed requests. The
        // existing envelope rejects unknown fields and each branch decodes its
        // canonical current value, without a second scalar or INI parser.
        let input: SuperReadValue<serde_json::Value> =
            serde_json::from_slice(input).map_err(|error| error.to_string())?;
        let point = self.warhead_impact_point(input.read, owner)?;
        let ids = &self.animations[..point.animation_count];
        let find = |name: &str| {
            super_weapon_index_of_name(name, ids.iter().map(String::as_str))
                .map(|index| u32::try_from(index).expect("native Anim ordinal exceeds u32"))
        };
        if reader == "warhead-impact-nukeball-default" {
            if !key.is_empty() || !owner.eq_ignore_ascii_case("NUKE") || !input.current.is_null() {
                return Err("NUKE impact initialization accepts no authored key or default".into());
            }
            // First WH initialization10051D25..54 precedes the section gate.
            // FindIndex10051BE0 sees the Anim prefix after original WH reads;
            // an unsuccessful default is retained, never retried later.
            return serde_json::to_vec(&find("NUKEBALL")).map_err(|error| error.to_string());
        }
        if !point
            .warhead_impact
            .expect("checked Warhead impact point")
            .body_read
        {
            return Err("Warhead impact scalar/reference read requires a reached body".into());
        }
        let section = ini
            .section(owner)
            .ok_or("missing physical Warhead impact section")?;
        match reader {
            "warhead-impact-duration" if key == "NukeFlash.Duration" => {
                let current: Option<i32> =
                    serde_json::from_value(input.current).map_err(|error| error.to_string())?;
                // Common WH LoadFromINI10052053: shared nullable integer,
                // before PreImpactAnim10052066, with no duration clamp.
                serde_json::to_vec(&section.read_ares_nullable_int(key, current))
                    .map_err(|error| error.to_string())
            }
            "warhead-impact-pre-impact" if key == "PreImpactAnim" => {
                let current: Option<u32> =
                    serde_json::from_value(input.current).map_err(|error| error.to_string())?;
                if current.is_some_and(|index| index as usize >= self.animations.len()) {
                    return Err("invalid retained Warhead PreImpact Anim ordinal".into());
                }
                // PreImpact reader100524C0 uses the existing ReadString2048
                // and raw FindIndex10051BE0 BEFORE the none predicate. A real
                // sentinel-named Anim wins; unknown/empty input retains.
                let result = read_ares_registered_index(
                    section,
                    key,
                    AresRegisteredLookupOrder::RegisteredFirst,
                    ids.iter().map(String::as_str),
                )
                .map_or(current, |value| {
                    value
                        .map(|index| u32::try_from(index).expect("native Anim ordinal exceeds u32"))
                });
                serde_json::to_vec(&result).map_err(|error| error.to_string())
            }
            _ => Err("unsupported Warhead impact reader/key".into()),
        }
    }
}

#[cfg(test)]
#[path = "native_warhead_impact_tests.rs"]
mod tests;
