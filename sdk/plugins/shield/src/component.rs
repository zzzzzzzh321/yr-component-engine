//! Static composition of this work's features, not a dynamic plugin manager.
//! Dispatch order is explicit: configure ammo then shield, and route each event
//! to its owner. Only this layer serializes the shared frozen envelope.
#[cfg(feature = "ares-ammo")]
use crate::features::ammo;
use crate::{
    features::shield,
    support::{ConfigView, Host},
};
use serde::{Deserialize, Serialize};
use vera_mod_api::{API_VERSION, Context, HookKind, ModuleManifest, Plan, Request};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    // Keep v1/v2's flat JSON keys and field order for existing frozen config.
    #[serde(flatten)]
    pub(crate) shield: shield::Configuration,
    #[cfg(feature = "ares-ammo")]
    ammunition: ammo::Ammunition,
}

pub fn manifest() -> ModuleManifest {
    let mut hooks = vec![
        HookKind::Configure,
        HookKind::Attach,
        HookKind::Damage,
        HookKind::Tick,
    ];
    if cfg!(feature = "ares-ammo") {
        hooks.push(HookKind::WeaponResource);
    }
    ModuleManifest {
        api_version: API_VERSION,
        id: if cfg!(all(feature = "overload", feature = "ares-ammo")) {
            "vera.overload-shield-ammo"
        } else if cfg!(feature = "ares-ammo") {
            "vera.compat-shield-ammo"
        } else if cfg!(feature = "overload") {
            "vera.overload-shield"
        } else {
            "vera.compat-shield"
        }
        .into(),
        version: if cfg!(feature = "ares-ammo") {
            "0.2.0"
        } else {
            "0.1.0"
        }
        .into(),
        state_version: 1,
        config_version: if cfg!(feature = "ares-ammo") { 2 } else { 1 },
        hooks,
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
    match &request.context {
        Context::Configure(context) => {
            let view = ConfigView::new(host);
            #[cfg(feature = "ares-ammo")]
            if context.final_pass {
                config.ammunition.configure(host, &view);
            }
            config.shield.configure(context, &view);
            view.finish()?;
            let mut plan = Plan::unchanged(request);
            plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
            Ok(plan)
        }
        Context::WeaponResource(context) => {
            let mut plan = Plan::unchanged(request);
            #[cfg(feature = "ares-ammo")]
            {
                plan.weapon_resource = Some(config.ammunition.policy(&context.weapon_name));
            }
            #[cfg(not(feature = "ares-ammo"))]
            {
                let _ = context;
                let _ = &mut plan;
            }
            Ok(plan)
        }
        _ => shield::evaluate(request, &config.shield, host),
    }
}

#[cfg(test)]
pub(crate) fn freeze_shield(shield: shield::Configuration) -> Vec<u8> {
    serde_json::to_vec(&Configuration {
        shield,
        ..Default::default()
    })
    .unwrap()
}
