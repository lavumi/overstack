use crate::event::Event;
use crate::log::push_event;
use crate::model::Team;
use crate::skill::StatusType;
use crate::step_api::{hp2, ActiveRun, TriggerContext, STATUS_TICK_RATE, STATUS_TICK_THRESHOLD};
use crate::trait_spec::TriggerType;

impl ActiveRun {
    pub(crate) fn apply_status(
        &mut self,
        src_idx: usize,
        dst_idx: usize,
        status_type: StatusType,
        base_chance: f32,
        duration: f32,
        stacks: u32,
        power: f32,
        trait_depth: u8,
        events: &mut Vec<String>,
    ) {
        let src_label = self.actor_label_for_idx(src_idx);
        let dst_label = self.actor_label_for_idx(dst_idx);

        let mut chance = base_chance;
        chance += self.runtime_ref(src_idx).map(|r| r.proc_bonus).unwrap_or(0.0);
        chance -= self.runtime_ref(dst_idx).map(|r| r.res_bonus).unwrap_or(0.0);

        if !self.roll_success(chance) {
            return;
        }

        let power_mul = self.status_power_mul_for(src_idx, status_type);

        let adjusted_power = power * power_mul;

        if let Some(row) = self.statuses_mut(dst_idx) {
            if let Some(existing) = row.iter_mut().find(|s| s.status_type == status_type) {
                existing.stacks = existing.stacks.saturating_add(stacks.max(1));
                existing.duration = existing.duration.max(duration);
                existing.power = existing.power.max(adjusted_power);
            } else {
                row.push(crate::step_api::ActiveStatus {
                    status_type,
                    stacks: stacks.max(1),
                    duration: duration.max(0.1),
                    power: adjusted_power,
                    tick_meter: 0.0,
                });
            }
        }

        push_event(
            events,
            Event::StatusApplied {
                src: src_label,
                dst: dst_label,
                status: status_type.as_str(),
                stacks: stacks.max(1),
                duration: duration.max(0.0).round() as u32,
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnStatusApplied,
            src_idx: Some(src_idx),
            dst_idx: Some(dst_idx),
            applied_status: Some(status_type),
        };
        self.process_trait_triggers(context, trait_depth + 1, events);

        let post_mul = self.status_power_mul_for(src_idx, status_type);
        let post_power = power * post_mul;
        if post_power > adjusted_power {
            if let Some(row) = self.statuses_mut(dst_idx) {
                if let Some(existing) = row.iter_mut().find(|s| s.status_type == status_type) {
                    existing.power = existing.power.max(post_power);
                }
            }
        }
    }

    pub(crate) fn apply_damage(
        &mut self,
        src_idx: usize,
        dst_idx: usize,
        amount: f32,
        trait_depth: u8,
        events: &mut Vec<String>,
    ) {
        let src_label = self.actor_label_for_idx(src_idx);
        let dst_label = self.actor_label_for_idx(dst_idx);

        let mut dst_hp_after = 0.0;
        if let Some(state) = self.state_mut() {
            let unit = &mut state.units[dst_idx];
            let dealt = amount.max(0.01);
            unit.hp = hp2((unit.hp - dealt).max(0.0));
            dst_hp_after = unit.hp;
        }

        push_event(
            events,
            Event::DamageDealt {
                src: src_label,
                dst: dst_label,
                amount: amount.max(0.01),
                dst_hp_after,
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnDamageDealt,
            src_idx: Some(src_idx),
            dst_idx: Some(dst_idx),
            applied_status: None,
        };
        self.process_trait_triggers(context, trait_depth + 1, events);
    }

    pub(crate) fn check_and_emit_battle_end(
        &mut self,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
        let state = self.state_ref()?;
        let enemy_alive = state.units.iter().any(|u| u.team == Team::Enemy && u.is_alive());
        let player_alive = state.units.iter().any(|u| u.team == Team::Player && u.is_alive());

        if !enemy_alive {
            let player_hp_after = state
                .units
                .iter()
                .find(|u| u.team == Team::Player)
                .map(|u| u.hp)
                .unwrap_or(0.0);
            push_event(
                events,
                Event::BattleEnd {
                    result: "win",
                    player_hp_after,
                },
            );
            self.emit_battle_end_triggers("win", events);
            return Some("win");
        }

        if !player_alive {
            push_event(
                events,
                Event::BattleEnd {
                    result: "lose",
                    player_hp_after: 0.0,
                },
            );
            self.emit_battle_end_triggers("lose", events);
            return Some("lose");
        }

        None
    }

    pub(crate) fn tick_statuses(&mut self, dt: f32, events: &mut Vec<String>) -> Option<&'static str> {
        if dt <= 0.0 {
            return None;
        }

        let mut pending_ticks: Vec<(usize, StatusType, f32)> = Vec::new();
        let mut pending_expire: Vec<(usize, StatusType)> = Vec::new();

        let unit_count = self.unit_count();
        for unit_idx in 0..unit_count {
            if let Some(row) = self.statuses_mut(unit_idx) {
                for status in row.iter_mut() {
                    status.duration -= dt;
                    status.tick_meter += dt * STATUS_TICK_RATE;

                    let tick_amount = match status.status_type {
                        StatusType::Burn | StatusType::Shock | StatusType::Bleed => {
                            (status.power * status.stacks as f32).max(0.01)
                        }
                        _ => 0.0,
                    };

                    while tick_amount > 0.0 && status.tick_meter >= STATUS_TICK_THRESHOLD {
                        pending_ticks.push((unit_idx, status.status_type, tick_amount));
                        status.tick_meter -= STATUS_TICK_THRESHOLD;
                    }

                    if status.duration <= 0.0 {
                        pending_expire.push((unit_idx, status.status_type));
                    }
                }
            }
        }

        for (unit_idx, status_type, amount) in pending_ticks {
            if let Some(state) = self.state_mut() {
                if state.units[unit_idx].is_alive() {
                    state.units[unit_idx].hp = hp2((state.units[unit_idx].hp - amount).max(0.0));
                }
            }

            let dst = self.actor_label_for_idx(unit_idx);
            let dst_hp_after = self.state_ref().map(|s| s.units[unit_idx].hp).unwrap_or(0.0);

            push_event(
                events,
                Event::StatusTick {
                    dst,
                    status: status_type.as_str(),
                    amount,
                    dst_hp_after,
                },
            );

            let context = TriggerContext {
                trigger_type: TriggerType::OnStatusTick,
                src_idx: None,
                dst_idx: Some(unit_idx),
                applied_status: Some(status_type),
            };
            self.process_trait_triggers(context, 0, events);
        }

        for (unit_idx, status_type) in pending_expire.iter().copied() {
            if let Some(row) = self.statuses_mut(unit_idx) {
                row.retain(|s| !(s.status_type == status_type && s.duration <= 0.0));
            }
        }

        for (unit_idx, status_type) in pending_expire {
            let dst = self.actor_label_for_idx(unit_idx);
            push_event(
                events,
                Event::StatusExpired {
                    dst,
                    status: status_type.as_str(),
                },
            );
        }

        self.check_and_emit_battle_end(events)
    }

    pub(crate) fn gauge_speed_multiplier(&self, unit_idx: usize) -> f32 {
        let mut mult = 1.0;
        if self.has_status(unit_idx, StatusType::Freeze) {
            mult *= 0.5;
        }
        if self.has_status(unit_idx, StatusType::Haste) {
            mult *= 1.25;
        }
        if self.has_status(unit_idx, StatusType::Stun) {
            mult = 0.0;
        }
        mult
    }

    pub(crate) fn finalize_battle(&mut self, outcome: &'static str, events: &mut Vec<String>) {
        if outcome == "win" {
            let player_hp = self
                .state_ref()
                .and_then(|state| state.units.iter().find(|u| u.team == Team::Player))
                .map(|u| u.hp)
                .unwrap_or(self.run.player_hp);

            self.run.player_hp = player_hp;
            let recover = hp2(self.run.player_max_hp * 0.20);
            self.run.player_hp = hp2((self.run.player_hp + recover).min(self.run.player_max_hp));
            self.current_battle = None;
            self.waiting_for_input = false;

            if self.node_index >= self.max_nodes {
                self.ended = true;
                self.result = "win";
                push_event(
                    events,
                    Event::RunEnd {
                        result: self.result,
                        final_node_index: self.node_index,
                    },
                );
            }
        } else {
            self.run.player_hp = 0.0;
            self.current_battle = None;
            self.waiting_for_input = false;
            self.ended = true;
            self.result = "lose";
            push_event(
                events,
                Event::RunEnd {
                    result: self.result,
                    final_node_index: self.node_index,
                },
            );
        }
    }
}
