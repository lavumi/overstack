use crate::engine::runtime::{ActiveRun, TraitOwner, TriggerContext, TRAIT_CHAIN_DEPTH_MAX};
use crate::event::Event;
use crate::log::push_event;
use crate::skill::{DamageKind, EffectSpec, StatType, StatusType};
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
            EffectSpec::DealDamage {
                damage_kind,
                multiplier,
                flat,
            } => {
                let src_idx = context
                    .src_idx
                    .or_else(|| self.owner_unit_idx(context.owner));
                let dst_idx = context.dst_idx.or_else(|| {
                    self.resolve_effect_target(crate::skill::EffectTarget::Opponent, context)
                });
                if let (Some(src_idx), Some(dst_idx)) = (src_idx, dst_idx) {
                    self.apply_scaled_damage(
                        src_idx,
                        dst_idx,
                        damage_kind,
                        multiplier,
                        flat,
                        depth,
                        events,
                    );
                    self.push_trait_effect_event(
                        trait_name,
                        format!("DealDamage x{multiplier:.2} +{flat}"),
                        events,
                    );
                }
            }
            EffectSpec::ApplyStatus {
                status_type,
                base_chance,
                duration,
                stacks,
                power,
            } => {
                let src_idx = context
                    .src_idx
                    .or_else(|| self.owner_unit_idx(context.owner));
                let dst_idx = context.dst_idx.or_else(|| {
                    self.resolve_effect_target(crate::skill::EffectTarget::Opponent, context)
                });
                if let (Some(src_idx), Some(dst_idx)) = (src_idx, dst_idx) {
                    self.apply_status(
                        src_idx,
                        dst_idx,
                        status_type,
                        base_chance,
                        duration,
                        stacks,
                        power,
                        depth,
                        events,
                    );
                    self.push_trait_effect_event(
                        trait_name,
                        format!("ApplyStatus {}", status_type.as_str()),
                        events,
                    );
                }
            }
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
            EffectSpec::SelfBuff {
                stat,
                amount,
                duration,
            } => {
                if let Some(src_idx) = self.owner_unit_idx(context.owner) {
                    let status_type = match stat {
                        StatType::Attack => StatusType::Might,
                        StatType::Speed => StatusType::Haste,
                    };
                    self.apply_status(
                        src_idx,
                        src_idx,
                        status_type,
                        1.0,
                        duration,
                        amount.max(1.0) as u32,
                        amount,
                        depth,
                        events,
                    );
                    self.push_trait_effect_event(
                        trait_name,
                        format!("SelfBuff {}", status_type.as_str()),
                        events,
                    );
                }
            }
            EffectSpec::AddProcBonus { amount } => {
                if let Some(owner_idx) = self.owner_unit_idx(context.owner) {
                    self.add_proc_bonus(owner_idx, amount);
                }
                self.push_trait_effect_event(
                    trait_name,
                    format!("AddProcBonus +{amount:.2}"),
                    events,
                );
            }
            EffectSpec::AddResBonus { amount } => {
                if let Some(owner_idx) = self.owner_unit_idx(context.owner) {
                    self.add_res_bonus(owner_idx, amount);
                }
                self.push_trait_effect_event(
                    trait_name,
                    format!("AddResBonus +{amount:.2}"),
                    events,
                );
            }
            EffectSpec::ModifyStatusPower { status_type, mul } => {
                if let Some(owner_idx) = self.owner_unit_idx(context.owner) {
                    self.update_status_power_mul(owner_idx, status_type, mul);
                }
                self.push_trait_effect_event(
                    trait_name,
                    format!("ModifyStatusPower {} x{mul:.2}", status_type.as_str()),
                    events,
                );
            }
            EffectSpec::AddStatusStacks {
                target,
                status_type,
                stacks,
            } => {
                if let Some(target_idx) = self.resolve_effect_target(target, context) {
                    let src_idx = context.src_idx.unwrap_or(target_idx);
                    self.apply_status(
                        src_idx,
                        target_idx,
                        status_type,
                        1.0,
                        1.0,
                        stacks.max(1),
                        1.0,
                        depth,
                        events,
                    );
                    self.push_trait_effect_event(
                        trait_name,
                        format!(
                            "AddStatusStacks {} +{}",
                            status_type.as_str(),
                            stacks.max(1)
                        ),
                        events,
                    );
                }
            }
            EffectSpec::DealPureDamage { target, amount } => {
                if let Some(dst_idx) = self.resolve_effect_target(target, context) {
                    let src_idx = context
                        .src_idx
                        .or_else(|| self.owner_unit_idx(context.owner))
                        .unwrap_or(dst_idx);
                    self.apply_pure_damage(src_idx, dst_idx, amount.max(0.01), depth, events);
                    self.push_trait_effect_event(
                        trait_name,
                        format!("DealPureDamage {:.2}", amount.max(0.01)),
                        events,
                    );
                }
            }
        }
    }

    pub(crate) fn process_trait_triggers(
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
        self.process_trait_triggers(context, 0, events);
    }
}
