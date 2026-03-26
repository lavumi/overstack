use crate::event::Event;
use crate::log::push_event;
use crate::model::Team;
use crate::skill::{
    player_skill_for_slot, skill_by_id, DamageKind, EffectSpec, SkillSpec, StatType, StatusType,
};
use crate::step_api::{ActionKind, ActiveRun, TraitOwner, TriggerContext};
use crate::trait_spec::TriggerType;

impl ActiveRun {
    fn choose_skill_for_action(&self, action: ActionKind) -> &'static SkillSpec {
        match action {
            ActionKind::BasicAttack => {
                skill_by_id("basic_attack").unwrap_or_else(|| player_skill_for_slot(0))
            }
            ActionKind::SkillSlot(slot) => player_skill_for_slot(slot),
        }
    }

    fn execute_skill(
        &mut self,
        actor_idx: usize,
        target_idx: usize,
        skill: &'static SkillSpec,
        events: &mut Vec<String>,
    ) {
        let actor = self.actor_label_for_idx(actor_idx);
        let owner = if actor == "player" {
            TraitOwner::Player
        } else {
            TraitOwner::Enemy
        };

        push_event(events, Event::TurnReady { actor });
        push_event(
            events,
            Event::ActionUsed {
                actor,
                action_name: skill.name,
            },
        );

        let context_action = TriggerContext {
            trigger_type: TriggerType::OnActionUsed,
            owner,
            src_idx: Some(actor_idx),
            dst_idx: Some(target_idx),
            applied_status: None,
        };
        self.process_trait_triggers(context_action, 0, events);

        let mut damage_amp = 1.0_f32;

        for effect in skill.effects {
            match *effect {
                EffectSpec::DealDamage {
                    damage_kind,
                    multiplier,
                    flat,
                } => {
                    let final_multiplier = skill.base_damage_multiplier * multiplier * damage_amp;
                    let final_flat = skill.flat_bonus_damage.unwrap_or(0.0) + flat;
                    let kind = match skill.id {
                        "basic_attack" => DamageKind::Physical,
                        _ => damage_kind,
                    };
                    self.apply_scaled_damage(
                        actor_idx,
                        target_idx,
                        kind,
                        final_multiplier,
                        final_flat,
                        0,
                        events,
                    );
                }
                EffectSpec::ApplyStatus {
                    status_type,
                    base_chance,
                    duration,
                    stacks,
                    power,
                } => {
                    self.apply_status(
                        actor_idx,
                        target_idx,
                        status_type,
                        base_chance,
                        duration,
                        stacks,
                        power,
                        0,
                        events,
                    );
                }
                EffectSpec::ConditionalDamageAmp { condition, amp } => {
                    let context = TriggerContext {
                        trigger_type: TriggerType::OnActionUsed,
                        owner,
                        src_idx: Some(actor_idx),
                        dst_idx: Some(target_idx),
                        applied_status: None,
                    };
                    if self.evaluate_condition(condition, context) {
                        damage_amp *= amp.max(0.1);
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
                    let context = TriggerContext {
                        trigger_type: TriggerType::OnActionUsed,
                        owner,
                        src_idx: Some(actor_idx),
                        dst_idx: Some(target_idx),
                        applied_status: None,
                    };
                    if self.evaluate_condition(condition, context) {
                        self.apply_status(
                            actor_idx,
                            target_idx,
                            status_type,
                            base_chance,
                            duration,
                            stacks,
                            power,
                            0,
                            events,
                        );
                    }
                }
                EffectSpec::SelfBuff {
                    stat,
                    amount,
                    duration,
                } => {
                    let status_type = match stat {
                        StatType::Attack => StatusType::Might,
                        StatType::Speed => StatusType::Haste,
                    };
                    self.apply_status(
                        actor_idx,
                        actor_idx,
                        status_type,
                        1.0,
                        duration,
                        amount.max(1.0) as u32,
                        amount,
                        0,
                        events,
                    );
                }
                EffectSpec::AddProcBonus { amount } => {
                    self.add_proc_bonus(actor_idx, amount);
                }
                EffectSpec::AddResBonus { amount } => {
                    self.add_res_bonus(actor_idx, amount);
                }
                EffectSpec::ModifyStatusPower { status_type, mul } => {
                    self.update_status_power_mul(actor_idx, status_type, mul);
                }
                EffectSpec::AddStatusStacks {
                    target,
                    status_type,
                    stacks,
                } => {
                    if let Some(dst_idx) = self.resolve_effect_target(
                        target,
                        TriggerContext {
                            trigger_type: TriggerType::OnActionUsed,
                            owner,
                            src_idx: Some(actor_idx),
                            dst_idx: Some(target_idx),
                            applied_status: None,
                        },
                    ) {
                        self.apply_status(
                            actor_idx,
                            dst_idx,
                            status_type,
                            1.0,
                            1.0,
                            stacks.max(1),
                            1.0,
                            0,
                            events,
                        );
                    }
                }
                EffectSpec::DealPureDamage { target, amount } => {
                    if let Some(dst_idx) = self.resolve_effect_target(
                        target,
                        TriggerContext {
                            trigger_type: TriggerType::OnActionUsed,
                            owner,
                            src_idx: Some(actor_idx),
                            dst_idx: Some(target_idx),
                            applied_status: None,
                        },
                    ) {
                        self.apply_pure_damage(actor_idx, dst_idx, amount.max(0.01), 0, events);
                    }
                }
            }
        }
    }

    pub(crate) fn execute_turn(
        &mut self,
        actor_idx: usize,
        action: ActionKind,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
        let Some(state) = self.state_ref() else {
            return None;
        };

        if !state.units[actor_idx].is_alive() || state.units[actor_idx].action_gauge < 100.0 {
            return None;
        }

        let actor_team = state.units[actor_idx].team;
        let target_team = if actor_team == Team::Player {
            Team::Enemy
        } else {
            Team::Player
        };

        let Some(target_idx) = self.pick_target_index(target_team) else {
            return None;
        };

        if let Some(state) = self.state_mut() {
            state.units[actor_idx].action_gauge -= 100.0;
        }

        let skill = if actor_team == Team::Player {
            self.choose_skill_for_action(action)
        } else {
            skill_by_id("basic_attack").unwrap_or_else(|| player_skill_for_slot(0))
        };

        self.execute_skill(actor_idx, target_idx, skill, events);
        self.check_and_emit_battle_end(events)
    }
}
