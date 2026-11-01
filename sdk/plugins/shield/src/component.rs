//! Static composition, selected by INI once per match. All code is present in
//! one component; simulation only uses the frozen selection, never live INI.
use crate::{
    features::{ammo, shield},
    support::{ConfigView, Host},
};
use serde::{Deserialize, Serialize};
use vera_mod_api::{API_VERSION, Context, HookKind, ModuleManifest, Plan, Projection, Request};

/// The only authority for feature activation. Physical Rules passes retain
/// previous values through the native bool reader; the final pass freezes them.
#[derive(Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Features {
    shield: bool,
    overload: bool,
    ares_ammo: bool,
}

impl Features {
    fn apply_pass<H: Host>(&mut self, view: &ConfigView<'_, H>) {
        let section = view.section_or_empty("VeraFeatures");
        self.shield = section.read_bool("Shield", self.shield);
        self.overload = section.read_bool("Overload", self.overload);
        self.ares_ammo = section.read_bool("AresAmmo", self.ares_ammo);
    }

    fn validate(&self) -> Result<(), String> {
        if self.overload && !self.shield {
            return Err("[VeraFeatures] Overload=yes requires Shield=yes".into());
        }
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    #[serde(default)]
    features: Features,
    #[serde(flatten)]
    shield: shield::Configuration,
    ammunition: ammo::Ammunition,
}

pub fn manifest() -> ModuleManifest {
    ModuleManifest {
        api_version: API_VERSION,
        id: "vera.reference-gameplay".into(),
        version: "0.3.0".into(),
        state_version: 1,
        config_version: 3,
        hooks: vec![
            HookKind::Configure,
            HookKind::Attach,
            HookKind::Damage,
            HookKind::Tick,
            HookKind::WeaponResource,
        ],
    }
}

fn decode_config(bytes: &[u8]) -> Result<Configuration, String> {
    if bytes.is_empty() {
        Ok(Configuration::default())
    } else {
        serde_json::from_slice(bytes).map_err(|e| format!("component configuration: {e}"))
    }
}

pub(crate) fn evaluate<H: Host>(request: &Request, host: &H) -> Result<Plan, String> {
    request.validate()?;
    let mut config = decode_config(&request.config)?;
    if let Context::Configure(context) = &request.context {
        let view = ConfigView::new(host);
        config.features.apply_pass(&view);
        if context.final_pass {
            config.features.validate()?;
            if config.features.ares_ammo {
                config.ammunition.configure(host, &view);
            }
        }
        // Always accumulate shield definitions: a later layer may enable them.
        config
            .shield
            .configure(context, &view, config.features.overload);
        view.finish()?;
        let mut plan = Plan::unchanged(request);
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    config.features.validate()?;
    match &request.context {
        Context::WeaponResource(context) => {
            let mut plan = Plan::unchanged(request);
            if config.features.ares_ammo {
                plan.weapon_resource = Some(config.ammunition.policy(&context.weapon_name));
            }
            Ok(plan)
        }
        _ if config.features.shield => {
            shield::evaluate(request, &config.shield, host, config.features.overload)
        }
        _ => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.projection = Some(Projection::default());
            Ok(plan)
        }
    }
}

#[cfg(test)]
pub(crate) fn freeze_shield(shield: shield::Configuration, overload: bool) -> Vec<u8> {
    serde_json::to_vec(&Configuration {
        features: Features {
            shield: true,
            overload,
            ..Default::default()
        },
        shield,
        ..Default::default()
    })
    .unwrap()
}
