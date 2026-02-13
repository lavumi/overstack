use crate::event::Event;
use crate::log::{push_event, set_log_tick};
use crate::model::Team;
use crate::skill::{player_skill_for_slot, skill_by_id, EffectSpec, SkillSpec, StatType, StatusType, BASIC_ATTACK};
use crate::step_api::{ActionKind, ActiveRun, StepResult, TriggerContext};
use crate::trait_spec::TriggerType;

impl ActiveRun {
    fn choose_skill_for_action(&self, action: ActionKind) -> &'static SkillSpec {
        match action {
            ActionKind::BasicAttack => &BASIC_ATTACK,
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
            src_idx: Some(actor_idx),
            dst_idx: Some(target_idx),
            applied_status: None,
        };
        self.process_trait_triggers(context_action, 0, events);

        let mut damage_amp = 1.0_f32;

        for effect in skill.effects {
            match *effect {
                EffectSpec::DealDamage { multiplier, flat } => {
                    let atk = self
                        .state_ref()
                        .map(|s| s.units[actor_idx].atk as f32)
                        .unwrap_or(1.0);
                    let base = atk * skill.base_damage_multiplier * multiplier * damage_amp;
                    let bonus = skill.flat_bonus_damage.unwrap_or(0.0) + flat;
                    let amount = (base + bonus).max(0.01);
                    self.apply_damage(actor_idx, target_idx, amount, 0, events);
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
                            src_idx: Some(actor_idx),
                            dst_idx: Some(target_idx),
                            applied_status: None,
                        },
                    ) {
                        self.apply_damage(actor_idx, dst_idx, amount.max(0.01), 0, events);
                    }
                }
            }
        }
    }

    fn execute_turn(
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
            skill_by_id("basic_attack").unwrap_or(&BASIC_ATTACK)
        };

        self.execute_skill(actor_idx, target_idx, skill, events);
        self.check_and_emit_battle_end(events)
    }

    pub(crate) fn step_once(&mut self, dt: f32, action: Option<ActionKind>) -> StepResult {
        let mut events = Vec::new();
        set_log_tick(self.sim_tick());

        if self.ended {
            return StepResult {
                events,
                need_input: false,
                ended: true,
                error: String::new(),
            };
        }

        if self.node_index == 0 && self.current_battle.is_none() {
            push_event(&mut events, Event::RunStart { seed: self.seed });
        }

        self.ensure_battle_started(&mut events);
        if self.ended {
            return StepResult {
                events,
                need_input: false,
                ended: true,
                error: String::new(),
            };
        }

        let mut queued_action = action;
        if self.waiting_for_input && queued_action.is_none() {
            return StepResult {
                events,
                need_input: true,
                ended: false,
                error: String::new(),
            };
        }

        let mut remaining = dt.max(0.0);
        let mut need_input = false;

        while remaining > 0.0 || (self.waiting_for_input && queued_action.is_some()) {
            let current_tick = self.advance_sim_tick();
            set_log_tick(current_tick);
            let step_dt = if remaining > 0.0 { remaining.min(0.1) } else { 0.0 };
            remaining = (remaining - step_dt).max(0.0);
            self.elapsed_time += step_dt;

            if let Some(outcome) = self.tick_statuses(step_dt, &mut events) {
                self.finalize_battle(outcome, &mut events);
                break;
            }

            if step_dt > 0.0 {
                let unit_count = self.state_ref().map(|s| s.units.len()).unwrap_or(0);
                for unit_idx in 0..unit_count {
                    let speed_mult = self.gauge_speed_multiplier(unit_idx);
                    if let Some(state) = self.state_mut() {
                        if state.units[unit_idx].is_alive() {
                            state.units[unit_idx].action_gauge +=
                                state.units[unit_idx].speed * step_dt * speed_mult;
                        }
                    }
                }
            }

            loop {
                let Some((actor_idx, actor_team)) = ({
                    if let Some(state) = self.state_ref() {
                        let mut ready_indices: Vec<usize> = state
                            .units
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, u)| {
                                if u.is_alive() && u.action_gauge >= 100.0 {
                                    Some(idx)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if ready_indices.is_empty() {
                            None
                        } else {
                            ready_indices.sort_by(|&a, &b| {
                                state.units[b]
                                    .action_gauge
                                    .partial_cmp(&state.units[a].action_gauge)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                            let idx = ready_indices[0];
                            Some((idx, state.units[idx].team))
                        }
                    } else {
                        None
                    }
                }) else {
                    break;
                };

                if actor_team == Team::Player {
                    if queued_action.is_none() {
                        need_input = true;
                        self.waiting_for_input = true;
                        break;
                    }

                    let action_kind = queued_action.take().unwrap_or(ActionKind::BasicAttack);
                    self.waiting_for_input = false;

                    if let Some(outcome) = self.execute_turn(actor_idx, action_kind, &mut events) {
                        self.finalize_battle(outcome, &mut events);
                        break;
                    }
                } else if let Some(outcome) =
                    self.execute_turn(actor_idx, ActionKind::BasicAttack, &mut events)
                {
                    self.finalize_battle(outcome, &mut events);
                    break;
                }

                if self.ended || self.current_battle.is_none() {
                    break;
                }
            }

            if self.ended || self.current_battle.is_none() || need_input {
                break;
            }
        }

        if self.current_battle.is_none() && !self.ended {
            self.ensure_battle_started(&mut events);
        }

        StepResult {
            events,
            need_input,
            ended: self.ended,
            error: String::new(),
        }
    }
}
