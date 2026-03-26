use crate::event::Event;
use crate::log::{push_event, set_log_tick};
use crate::model::Team;
use crate::step_api::{ActionKind, ActiveRun, StepResult};

impl ActiveRun {
    pub(crate) fn step_once(&mut self, dt: f32, action: Option<ActionKind>) -> StepResult {
        let mut events = Vec::new();
        set_log_tick(self.sim_tick());

        if let Some(errors) = &self.data_error {
            return StepResult {
                events,
                need_input: false,
                ended: true,
                error: format!("data_load_failed: {}", errors.join_messages()),
            };
        }

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
            let step_dt = if remaining > 0.0 {
                remaining.min(0.1)
            } else {
                0.0
            };
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
