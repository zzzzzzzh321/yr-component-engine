//! Translate read-only actor facts and commit component transactions through
//! the existing effect, animation, numeric and cloak owners. No Ares INI tags
//! or effect lifecycle decisions are interpreted by this adapter.
use super::{AttachedEffect, EffectOrigin, SelfEffectState};
use crate::rules::{object_type::ObjectCategory, ruleset::RuleSet, warhead_type::WarheadType};
use crate::sim::{intern::InternedId, projectile::ProjectileCoord, world::Simulation};
use vera_mod_api::{self as api, Context};
#[cfg(test)]
mod tests;

fn control_view(value: SelfEffectState) -> api::EffectControl {
    match value {
        SelfEffectState::Pending(n) => api::EffectControl::Waiting(n),
        SelfEffectState::Applied(n) => api::EffectControl::Active(n),
        SelfEffectState::Disabled(n) => api::EffectControl::Disabled(n),
    }
}

fn control_value(value: api::EffectControl) -> SelfEffectState {
    match value {
        api::EffectControl::Waiting(n) => SelfEffectState::Pending(n),
        api::EffectControl::Active(n) => SelfEffectState::Applied(n),
        api::EffectControl::Disabled(n) => SelfEffectState::Disabled(n),
    }
}

impl Simulation {
    fn component_effect_context(
        &self,
        target: u64,
        event: api::EffectEvent,
        rules: &RuleSet,
    ) -> Result<api::EffectContext, String> {
        let entity = crate::mod_sdk::live_entity_context(self, target, rules)
            .ok_or("effect target no longer exists")?;
        let actor = self
            .substrate
            .entities
            .get(target)
            .ok_or("effect target no longer exists")?;
        let entries = actor
            .attach_effects
            .entries
            .iter()
            .map(|entry| {
                Ok(api::EffectInstanceView {
                    resource: entry
                        .origin
                        .resource(&self.interner)
                        .ok_or("effect resource identity is absent")?,
                    remaining: entry.remaining,
                    source_owner: entry
                        .source_house
                        .map(|id| {
                            self.interner
                                .try_resolve(id)
                                .map(str::to_owned)
                                .ok_or("effect source House identity is absent")
                        })
                        .transpose()?,
                    has_animation: entry.animation.is_some(),
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(api::EffectContext {
            frame: self.session.binary_frame() as i32,
            entity,
            user: crate::mod_sdk::weapon_user(actor.category()),
            inside_transport: actor.passenger_role.inside_transport_id().is_some(),
            cloaked_or_cloaking: matches!(
                actor.cloak.as_ref().map(|cloak| cloak.state()),
                Some(1 | 2)
            ),
            entries,
            control: control_view(actor.attach_effects.self_effect),
            animations_hidden: actor.attach_effects.animations_hidden,
            event,
        })
    }

    fn component_effect_event(&mut self, target: u64, event: api::EffectEvent, rules: &RuleSet) {
        if !rules.external_effects_enabled() || self.external_mod_fault.is_some() {
            return;
        }
        let result = (|| {
            let program = self
                .external_mod
                .as_ref()
                .filter(|p| p.has_effect_runtime())
                .ok_or("Rules and simulation disagree about the effect component")?;
            let context = self.component_effect_context(target, event, rules)?;
            let plan = program.evaluate(Context::Effects(context.clone()), &[])?;
            if let Some(plan) = plan.effects {
                self.commit_component_effects(&context, &plan, rules)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            self.external_mod_fault = Some(format!("entity {target} effects: {error}"));
        }
    }

    fn commit_component_effects(
        &mut self,
        context: &api::EffectContext,
        plan: &api::EffectPlan,
        rules: &RuleSet,
    ) -> Result<(), String> {
        plan.validate(context)?;
        // Semantic admission completes before the first write. Append only
        // consumes the current impact resource or the current actor's self
        // resource; it cannot inject effects or source Houses from elsewhere.
        for action in &plan.actions {
            if let api::EffectAction::Append {
                resource,
                source_owner,
                ..
            } = action
            {
                if rules.effect_descriptor(resource).is_none() {
                    return Err("component referenced an unqualified effect resource".into());
                }
                let allowed = match &context.event {
                    api::EffectEvent::Apply {
                        resource: asked,
                        source_owner: source,
                        ..
                    } => resource == asked && source_owner == source,
                    api::EffectEvent::Visit => {
                        resource.kind.user() == Some(context.user)
                            && resource.name == context.entity.type_name
                            && source_owner.as_deref() == Some(context.entity.owner.as_str())
                    }
                    _ => false,
                };
                if !allowed {
                    return Err("effect append exceeds its event scope".into());
                }
            }
        }
        let target = context.entity.id;
        for action in &plan.actions {
            if self.substrate.entities.get(target).is_none() {
                return Ok(());
            }
            match action {
                api::EffectAction::SetControl { control } => {
                    self.substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .attach_effects
                        .self_effect = control_value(*control);
                }
                api::EffectAction::SetAnimationsHidden { hidden } => {
                    self.substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .attach_effects
                        .animations_hidden = *hidden;
                }
                api::EffectAction::SetRemaining { index, remaining } => {
                    self.substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .attach_effects
                        .entries
                        .get_mut(*index)
                        .ok_or("effect list changed during a callback")?
                        .remaining = *remaining;
                }
                api::EffectAction::Append {
                    resource,
                    remaining,
                    source_owner,
                } => {
                    let id = self.interner.intern(&resource.name);
                    let origin = match resource.kind {
                        api::EffectResourceKind::Warhead => EffectOrigin::Warhead(id),
                        kind => EffectOrigin::TechnoType(
                            match kind {
                                api::EffectResourceKind::Unit => ObjectCategory::Vehicle,
                                api::EffectResourceKind::Infantry => ObjectCategory::Infantry,
                                api::EffectResourceKind::Building => ObjectCategory::Building,
                                api::EffectResourceKind::Aircraft => ObjectCategory::Aircraft,
                                api::EffectResourceKind::Warhead => unreachable!(),
                            },
                            id,
                        ),
                    };
                    let source_house = source_owner.as_ref().map(|name| self.interner.intern(name));
                    self.substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .attach_effects
                        .entries
                        .push(AttachedEffect {
                            origin,
                            animation: None,
                            remaining: *remaining,
                            source_house,
                        });
                }
                api::EffectAction::Remove { index } => {
                    let entries = &mut self
                        .substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .attach_effects
                        .entries;
                    if *index >= entries.len() {
                        return Err("effect list changed during a callback".into());
                    }
                    // Preserve the existing type-change owner's erasure before
                    // releasing its held reference. Explicit Retire before
                    // Remove retains ordinary expiry/limbo ordering.
                    let removed = entries.remove(*index);
                    if let Some(animation) = removed.animation {
                        self.destroy_anim_with_context(animation, rules);
                    }
                }
                api::EffectAction::CreateAnimation { index } => {
                    self.create_attached_effect_anim(target, *index, rules)
                }
                api::EffectAction::RetireAnimation { index } => {
                    self.retire_attached_effect_anim(target, *index, rules)
                }
                api::EffectAction::RecalculateStats => {
                    self.substrate
                        .entities
                        .get_mut(target)
                        .unwrap()
                        .recompute_ares_attached_stats(rules, &self.interner);
                }
                api::EffectAction::Decloak => self.decloak_attached_effect(target, rules),
            }
        }
        if let Some(actor) = self.substrate.entities.get(target)
            && (control_view(actor.attach_effects.self_effect) != plan.control
                || actor.attach_effects.animations_hidden != plan.animations_hidden)
        {
            return Err("effect registers changed during a callback".into());
        }
        Ok(())
    }

    pub(crate) fn apply_component_effects_detonation(
        &mut self,
        impact: ProjectileCoord,
        source_house: Option<InternedId>,
        warhead: &WarheadType,
        rules: &RuleSet,
    ) {
        let resource = api::EffectResourceContext {
            kind: api::EffectResourceKind::Warhead,
            name: warhead.id.clone(),
        };
        if !rules.external_effects_enabled() || rules.effect_descriptor(&resource).is_none() {
            return;
        }
        let targets = self.ares_effect_ground_targets(impact, warhead.cell_spread_f64, rules);
        for target in targets {
            let Some(victim) = self.substrate.entities.get(target) else {
                continue;
            };
            let Some(kind) = self.interner.object_type_in_category(
                victim.type_ref(),
                victim.category().into(),
                rules,
            ) else {
                continue;
            };
            let source_is_ally = source_house.is_some_and(|source| {
                crate::sim::combat::combat_weapon::is_ally_by_object(
                    Some(&self.house_alliances),
                    &self.interner,
                    source,
                    victim.owner(),
                )
            });
            let event = api::EffectEvent::Apply {
                resource: resource.clone(),
                source_owner: source_house.map(|id| self.interner.resolve(id).to_owned()),
                source_is_owner: source_house == Some(victim.owner()),
                source_is_ally,
                affects_allies: warhead.affects_allies,
                verses_bits: warhead.interaction(kind.armor_id()).multiplier().bits(),
            };
            self.component_effect_event(target, event, rules);
            if self.external_mod_fault.is_some() {
                return;
            }
        }
    }

    pub(crate) fn update_component_effects(&mut self, target: u64, rules: &RuleSet) {
        if !rules.external_effects_enabled() {
            return;
        }
        let Some(actor) = self.substrate.entities.get(target) else {
            return;
        };
        if actor.attach_effects.entries.is_empty()
            && (!matches!(
                actor.attach_effects.self_effect,
                SelfEffectState::Pending(_)
            ) || EffectOrigin::for_type(actor)
                .resource(&self.interner)
                .is_none_or(|resource| rules.effect_descriptor(&resource).is_none()))
        {
            return;
        }
        self.component_effect_event(target, api::EffectEvent::Visit, rules);
    }

    pub(crate) fn limbo_component_effects(&mut self, target: u64, rules: &RuleSet) {
        self.component_effect_event(target, api::EffectEvent::EnterLimbo, rules);
    }

    pub(crate) fn reset_component_effects_for_type_change(
        &mut self,
        target: u64,
        old_type: InternedId,
        rules: &RuleSet,
    ) {
        let Some(actor) = self.substrate.entities.get(target) else {
            return;
        };
        let Some(previous) =
            EffectOrigin::TechnoType(actor.category().into(), old_type).resource(&self.interner)
        else {
            return;
        };
        self.component_effect_event(target, api::EffectEvent::TypeChanged { previous }, rules);
    }
}
