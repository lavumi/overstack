use crate::combat_math::{compute_damage, crit_chance, BREAK_MAX_STACKS};
use crate::engine::numeric::{
    round_hp, status_duration_display_secs, STATUS_TICK_RATE, STATUS_TICK_THRESHOLD,
};
use crate::engine::runtime::{ActiveRun, ActiveStatus, TraitOwner, TriggerContext};
use crate::event::Event;
use crate::log::push_event;
use crate::model::Team;
use crate::skill::{DamageKind, StatusType};
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
        chance += self
            .runtime_ref(src_idx)
            .map(|r| r.proc_bonus)
            .unwrap_or(0.0);
        chance -= self
            .runtime_ref(dst_idx)
            .map(|r| r.res_bonus)
            .unwrap_or(0.0);

        if !self.roll_success(chance) {
            return;
        }

        let power_mul = self.status_power_mul_for(src_idx, status_type);

        let adjusted_power = power * power_mul;

        if let Some(row) = self.statuses_mut(dst_idx) {
            let added_stacks = stacks.max(1);
            if let Some(existing) = row.iter_mut().find(|s| s.status_type == status_type) {
                let merged = existing.stacks.saturating_add(added_stacks);
                existing.stacks = if status_type == StatusType::Break {
                    merged.min(BREAK_MAX_STACKS)
                } else {
                    merged
                };
                existing.duration = existing.duration.max(duration);
                existing.power = existing.power.max(adjusted_power);
            } else {
                let initial_stacks = if status_type == StatusType::Break {
                    added_stacks.min(BREAK_MAX_STACKS)
                } else {
                    added_stacks
                };
                row.push(ActiveStatus {
                    status_type,
                    stacks: initial_stacks,
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
                duration: status_duration_display_secs(duration),
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnStatusApplied,
            owner: TraitOwner::Player,
            src_idx: Some(src_idx),
            dst_idx: Some(dst_idx),
            applied_status: Some(status_type),
        };
        self.emit_post_status_trait_triggers(context, trait_depth + 1, events);

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

    pub(crate) fn remove_status(
        &mut self,
        dst_idx: usize,
        status_type: StatusType,
        events: &mut Vec<String>,
    ) -> bool {
        let mut removed = false;
        if let Some(row) = self.statuses_mut(dst_idx) {
            let before = row.len();
            row.retain(|s| s.status_type != status_type);
            removed = row.len() != before;
        }

        if removed {
            let dst = self.actor_label_for_idx(dst_idx);
            push_event(
                events,
                Event::StatusExpired {
                    dst,
                    status: status_type.as_str(),
                },
            );
        }

        removed
    }

    pub(crate) fn apply_scaled_damage(
        &mut self,
        src_idx: usize,
        dst_idx: usize,
        damage_kind: DamageKind,
        multiplier: f32,
        flat: f32,
        trait_depth: u8,
        events: &mut Vec<String>,
    ) {
        let (src, dst) = {
            let Some(state) = self.state_ref() else {
                return;
            };
            let mut dst = state.units[dst_idx].clone();
            if damage_kind == DamageKind::Physical {
                dst.def = self.effective_def(dst_idx);
            }
            (state.units[src_idx].clone(), dst)
        };
        let crit = self.roll_success(crit_chance(src.crit_rate));
        let breakdown = compute_damage(&src, &dst, damage_kind, multiplier, flat, crit);

        let dst_hp_after = if let Some(state) = self.state_mut() {
            let unit = &mut state.units[dst_idx];
            unit.hp = round_hp((unit.hp - breakdown.amount.max(0.01)).max(0.0));
            unit.hp
        } else {
            return;
        };

        self.emit_damage_event(
            src_idx,
            dst_idx,
            damage_kind,
            breakdown.raw,
            breakdown.defense_used,
            breakdown.mitigation,
            breakdown.crit,
            breakdown.amount,
            dst_hp_after,
            trait_depth,
            events,
        );
    }

    pub(crate) fn apply_pure_damage(
        &mut self,
        src_idx: usize,
        dst_idx: usize,
        amount: f32,
        trait_depth: u8,
        events: &mut Vec<String>,
    ) {
        let dealt = amount.max(0.01);
        let dst_hp_after = if let Some(state) = self.state_mut() {
            let unit = &mut state.units[dst_idx];
            unit.hp = round_hp((unit.hp - dealt).max(0.0));
            unit.hp
        } else {
            return;
        };

        self.emit_damage_event(
            src_idx,
            dst_idx,
            DamageKind::Physical,
            dealt,
            0,
            1.0,
            false,
            dealt,
            dst_hp_after,
            trait_depth,
            events,
        );
    }

    fn emit_damage_event(
        &mut self,
        src_idx: usize,
        dst_idx: usize,
        damage_kind: DamageKind,
        raw: f32,
        defense_used: i32,
        mitigation: f32,
        crit: bool,
        amount: f32,
        dst_hp_after: f32,
        trait_depth: u8,
        events: &mut Vec<String>,
    ) {
        let src_label = self.actor_label_for_idx(src_idx);
        let dst_label = self.actor_label_for_idx(dst_idx);

        push_event(
            events,
            Event::DamageDealt {
                src: src_label,
                dst: dst_label,
                damage_kind: damage_kind.as_str(),
                raw,
                defense_used,
                mitigation,
                crit,
                amount: amount.max(0.01),
                dst_hp_after,
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnDamageDealt,
            owner: TraitOwner::Player,
            src_idx: Some(src_idx),
            dst_idx: Some(dst_idx),
            applied_status: None,
        };
        self.emit_post_damage_trait_triggers(context, trait_depth + 1, events);
    }

    pub(crate) fn check_and_emit_battle_end(
        &mut self,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
        let state = self.state_ref()?;
        let enemy_alive = state
            .units
            .iter()
            .any(|u| u.team == Team::Enemy && u.is_alive());
        let player_alive = state
            .units
            .iter()
            .any(|u| u.team == Team::Player && u.is_alive());

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

    pub(crate) fn tick_statuses(
        &mut self,
        dt: f32,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
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
                    state.units[unit_idx].hp =
                        round_hp((state.units[unit_idx].hp - amount).max(0.0));
                }
            }

            let dst = self.actor_label_for_idx(unit_idx);
            let dst_hp_after = self
                .state_ref()
                .map(|s| s.units[unit_idx].hp)
                .unwrap_or(0.0);

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
                owner: TraitOwner::Player,
                src_idx: None,
                dst_idx: Some(unit_idx),
                applied_status: Some(status_type),
            };
            self.emit_status_tick_trait_triggers(context, 0, events);
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
            let recover = round_hp(self.run.player_max_hp * 0.20);
            self.run.player_hp =
                round_hp((self.run.player_hp + recover).min(self.run.player_max_hp));
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
