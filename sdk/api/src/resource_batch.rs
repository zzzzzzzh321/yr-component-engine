//! Bounded batches of independent, immutable resource queries. A batch shares
//! one frozen configuration and returns one ordinary plan per query in order.

use crate::{API_VERSION, Context, HookKind, MAX_CONFIG_BYTES, Request};
use serde::{Deserialize, Serialize};

pub const MAX_QUERIES: usize = 128;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBatch<'a> {
    pub api_version: u32,
    pub config: std::borrow::Cow<'a, [u8]>,
    pub queries: std::borrow::Cow<'a, [Context]>,
}

impl ResourceBatch<'_> {
    pub fn validate(&self, hooks: &[HookKind]) -> Result<(), String> {
        if self.api_version != API_VERSION
            || self.config.len() > MAX_CONFIG_BYTES
            || self.queries.is_empty()
            || self.queries.len() > MAX_QUERIES
        {
            return Err("invalid resource batch size or API version".into());
        }
        for context in self.queries.iter() {
            let kind = context.kind();
            if !matches!(
                kind,
                HookKind::FoundationResource
                    | HookKind::ArmorResource
                    | HookKind::EmpResource
                    | HookKind::ProductionResource
                    | HookKind::EffectResource
                    | HookKind::WeaponInstanceResource
                    | HookKind::ProjectileResource
                    | HookKind::TypeResource
            ) || !hooks.contains(&kind)
            {
                return Err(format!(
                    "undeclared or non-batchable resource query: {}",
                    kind.as_str()
                ));
            }
            query_request(context.clone()).validate()?;
        }
        Ok(())
    }
}

/// Resource plan validation needs the original context and no entity state.
pub fn query_request(context: Context) -> Request {
    Request {
        api_version: API_VERSION,
        context,
        config: Vec::new(),
        state: Vec::new(),
    }
}
