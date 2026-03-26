use crate::engine::runtime::{ActiveRun, TraitOwner, TriggerContext, TRAIT_CHAIN_DEPTH_MAX};
use crate::event::Event;
use crate::log::push_event;
use crate::skill::{DamageKind, EffectSpec};
use crate::trait_spec::{trait_by_id, TriggerType};

impl ActiveRun {
    fn owner_label(owner: TraitOwner) -> &'static str {
        match owner {
            TraitOwner::Player => "player",
            TraitOwner::Enemy => "enemy",
        }
    }

    fn owner_unit_idx(&self, owner: TraitOwner) -> Option<usize> {
        let ctx = TriggerContext {
            trigger_type: TriggerType::OnBattleStart,
            owner,
            src_idx: None,
            dst_idx: None,
            applied_status: None,
        };
        self.resolve_effect_target(crate::skill::EffectTarget::Owner, ctx)
    }

    fn push_trait_effect_event(
        &self,
        trait_name: &'static str,
        summary: String,
        events: &mut Vec<String>,
    ) {
        push_event(
            events,
            Event::TraitEffectApplied {
                trait_name,
                effect_summary: summary,
            },
        );
    }

    fn process_trait_effect(
        &mut self,
        trait_name: &'static str,
        effect: EffectSpec,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        if depth >= TRAIT_CHAIN_DEPTH_MAX {
            return;
        }

        match effect {
            EffectSpec::ConditionalDamageAmp { condition, amp } => {
                if self.evaluate_condition(condition, context) {
                    let next = EffectSpec::DealDamage {
                        damage_kind: DamageKind::Physical,
                        multiplier: amp,
                        flat: 0.0,
                    };
                    self.process_trait_effect(trait_name, next, context, depth + 1, events);
                }
            }
            EffectSpec::ConditionalApplyStatus {
                condition,
                status_type,
                base_chance,
                duration,
                stacks,
                power,
            } => {
                if self.evaluate_condition(condition, context) {
                    let next = EffectSpec::ApplyStatus {
                        status_type,
                        base_chance,
                        duration,
                        stacks,
                        power,
                    };
                    self.process_trait_effect(trait_name, next, context, depth + 1, events);
                }
            }
            other => {
                let src_idx = context
                    .src_idx
                    .or_else(|| self.owner_unit_idx(context.owner));
                let dst_idx = context.dst_idx.or_else(|| {
                    self.resolve_effect_target(crate::skill::EffectTarget::Opponent, context)
                });
                if let (Some(src_idx), Some(dst_idx)) = (src_idx, dst_idx) {
                    if let Some(summary) = self
                        .execute_primitive_effect(other, src_idx, dst_idx, context, depth, events)
                    {
                        self.push_trait_effect_event(trait_name, summary, events);
                    }
                }
            }
        }
    }

    fn dispatch_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        if depth >= TRAIT_CHAIN_DEPTH_MAX {
            return;
        }

        let owners = [TraitOwner::Player, TraitOwner::Enemy];
        for owner in owners {
            let trait_ids = match owner {
                TraitOwner::Player => self.player_traits.clone(),
                TraitOwner::Enemy => self.enemy_traits.clone(),
            };
            for trait_id in trait_ids {
                let Some(spec) = trait_by_id(trait_id) else {
                    continue;
                };

                let owner_context = TriggerContext { owner, ..context };

                for rule in spec.triggers {
                    if !self.trigger_matches(rule.trigger, owner_context.trigger_type) {
                        continue;
                    }
                    if !self.evaluate_condition(rule.condition, owner_context) {
                        continue;
                    }

                    push_event(
                        events,
                        Event::TraitTriggered {
                            owner: Self::owner_label(owner),
                            trait_name: spec.name,
                            trigger_type: rule.trigger.as_str(),
                        },
                    );

                    for effect in rule.effects {
                        self.process_trait_effect(
                            spec.name,
                            *effect,
                            owner_context,
                            depth + 1,
                            events,
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn emit_pre_action_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        debug_assert_eq!(context.trigger_type, TriggerType::OnActionUsed);
        self.dispatch_trait_triggers(context, depth, events);
    }

    pub(crate) fn emit_post_damage_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        debug_assert_eq!(context.trigger_type, TriggerType::OnDamageDealt);
        self.dispatch_trait_triggers(context, depth, events);
    }

    pub(crate) fn emit_post_status_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        debug_assert_eq!(context.trigger_type, TriggerType::OnStatusApplied);
        self.dispatch_trait_triggers(context, depth, events);
    }

    pub(crate) fn emit_status_tick_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        debug_assert_eq!(context.trigger_type, TriggerType::OnStatusTick);
        self.dispatch_trait_triggers(context, depth, events);
    }

    pub(crate) fn emit_battle_start_trait_triggers(
        &mut self,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) {
        debug_assert_eq!(context.trigger_type, TriggerType::OnBattleStart);
        self.dispatch_trait_triggers(context, depth, events);
    }

    pub(crate) fn emit_battle_end_triggers(
        &mut self,
        result: &'static str,
        events: &mut Vec<String>,
    ) {
        let _ = result;
        let context = TriggerContext {
            trigger_type: TriggerType::OnBattleEnd,
            owner: TraitOwner::Player,
            src_idx: None,
            dst_idx: None,
            applied_status: None,
        };
        self.dispatch_trait_triggers(context, 0, events);
    }
}
