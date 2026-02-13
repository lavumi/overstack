use crate::model::{BattleState, Team};
use crate::skill::{Condition, EffectTarget, StatusType};
use crate::step_api::{ActiveRun, ActiveStatus, TriggerContext, UnitRuntime};
use crate::trait_spec::TriggerType;

impl ActiveRun {
    fn battle_ref(&self) -> Option<&crate::step_api::ActiveBattle> {
        self.current_battle.as_ref()
    }

    fn battle_mut(&mut self) -> Option<&mut crate::step_api::ActiveBattle> {
        self.current_battle.as_mut()
    }

    pub(crate) fn state_ref(&self) -> Option<&BattleState> {
        self.battle_ref().map(|b| &b.state)
    }

    pub(crate) fn state_mut(&mut self) -> Option<&mut BattleState> {
        self.battle_mut().map(|b| &mut b.state)
    }

    pub(crate) fn runtime_ref(&self, unit_idx: usize) -> Option<&UnitRuntime> {
        self.battle_ref().and_then(|b| b.runtime.get(unit_idx))
    }

    pub(crate) fn runtime_mut(&mut self, unit_idx: usize) -> Option<&mut UnitRuntime> {
        self.battle_mut().and_then(|b| b.runtime.get_mut(unit_idx))
    }

    pub(crate) fn statuses_ref(&self, unit_idx: usize) -> Option<&Vec<ActiveStatus>> {
        self.runtime_ref(unit_idx).map(|r| &r.statuses)
    }

    pub(crate) fn statuses_mut(&mut self, unit_idx: usize) -> Option<&mut Vec<ActiveStatus>> {
        self.runtime_mut(unit_idx).map(|r| &mut r.statuses)
    }

    pub(crate) fn unit_count(&self) -> usize {
        self.state_ref().map(|s| s.units.len()).unwrap_or(0)
    }

    pub(crate) fn sim_tick(&self) -> u32 {
        self.state_ref().map(|s| s.tick).unwrap_or(0)
    }

    pub(crate) fn advance_sim_tick(&mut self) -> u32 {
        if let Some(state) = self.state_mut() {
            state.tick = state.tick.saturating_add(1);
            state.tick
        } else {
            0
        }
    }

    pub(crate) fn add_proc_bonus(&mut self, unit_idx: usize, amount: f32) {
        if let Some(runtime) = self.runtime_mut(unit_idx) {
            runtime.proc_bonus += amount;
        }
    }

    pub(crate) fn add_res_bonus(&mut self, unit_idx: usize, amount: f32) {
        if let Some(runtime) = self.runtime_mut(unit_idx) {
            runtime.res_bonus += amount;
        }
    }

    pub(crate) fn status_power_mul_for(&self, unit_idx: usize, status_type: StatusType) -> f32 {
        self.runtime_ref(unit_idx)
            .and_then(|r| r.status_power_mult.get(&status_type).copied())
            .unwrap_or(1.0)
            .max(0.1)
    }

    pub(crate) fn update_status_power_mul(&mut self, unit_idx: usize, status_type: StatusType, mul: f32) {
        if let Some(runtime) = self.runtime_mut(unit_idx) {
            let entry = runtime.status_power_mult.entry(status_type).or_insert(1.0);
            *entry = entry.max(mul.max(0.1));
        }
    }

    pub(crate) fn actor_label_for_idx(&self, idx: usize) -> &'static str {
        let Some(state) = self.state_ref() else {
            return "enemy";
        };
        if state.units[idx].team == Team::Player {
            "player"
        } else {
            "enemy"
        }
    }

    pub(crate) fn roll_success(&mut self, chance: f32) -> bool {
        let clamped = chance.clamp(0.0, 1.0);
        if clamped <= 0.0 {
            return false;
        }
        if clamped >= 1.0 {
            return true;
        }
        let roll = (self.run.rng.next_u32() as f64) / (u32::MAX as f64);
        roll < clamped as f64
    }

    pub(crate) fn has_status(&self, unit_idx: usize, status_type: StatusType) -> bool {
        self.statuses_ref(unit_idx)
            .map(|row| row.iter().any(|s| s.status_type == status_type && s.duration > 0.0))
            .unwrap_or(false)
    }

    pub(crate) fn status_count(&self, unit_idx: usize) -> u32 {
        self.statuses_ref(unit_idx)
            .map(|row| row.iter().filter(|s| s.duration > 0.0).count() as u32)
            .unwrap_or(0)
    }

    pub(crate) fn target_hp_ratio(&self, unit_idx: usize) -> f32 {
        self.state_ref()
            .map(|state| {
                let unit = &state.units[unit_idx];
                if unit.max_hp <= 0.0 {
                    0.0
                } else {
                    unit.hp.max(0.0) / unit.max_hp
                }
            })
            .unwrap_or(1.0)
    }

    pub(crate) fn evaluate_condition(&mut self, condition: Condition, context: TriggerContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::SrcIsPlayer => context
                .src_idx
                .map(|idx| self.actor_label_for_idx(idx) == "player")
                .unwrap_or(false),
            Condition::DstIsEnemy => context
                .dst_idx
                .map(|idx| self.actor_label_for_idx(idx) == "enemy")
                .unwrap_or(false),
            Condition::AppliedStatusIs(status_type) => context.applied_status == Some(status_type),
            Condition::RandomRollBelow(p) => self.roll_success(p),
            Condition::TargetHPBelow(ratio) => context
                .dst_idx
                .map(|idx| self.target_hp_ratio(idx) < ratio)
                .unwrap_or(false),
            Condition::TargetHasStatus(status_type) => context
                .dst_idx
                .map(|idx| self.has_status(idx, status_type))
                .unwrap_or(false),
            Condition::TargetStatusCountAtLeast(n) => context
                .dst_idx
                .map(|idx| self.status_count(idx) >= n)
                .unwrap_or(false),
            Condition::All(items) => items
                .iter()
                .all(|item| self.evaluate_condition(*item, context)),
        }
    }

    pub(crate) fn trigger_matches(&self, expected: TriggerType, actual: TriggerType) -> bool {
        expected == actual
    }

    pub(crate) fn resolve_effect_target(
        &self,
        target: EffectTarget,
        context: TriggerContext,
    ) -> Option<usize> {
        match target {
            EffectTarget::Src => context.src_idx,
            EffectTarget::Dst => context.dst_idx,
            EffectTarget::Player => self
                .state_ref()
                .and_then(|s| s.units.iter().position(|u| u.team == Team::Player)),
            EffectTarget::Enemy => self
                .state_ref()
                .and_then(|s| s.units.iter().position(|u| u.team == Team::Enemy && u.is_alive()))
                .or_else(|| {
                    self.state_ref()
                        .and_then(|s| s.units.iter().position(|u| u.team == Team::Enemy))
                }),
        }
    }

    pub(crate) fn pick_target_index(&mut self, target_team: Team) -> Option<usize> {
        let state = self.state_ref()?;
        let targets: Vec<usize> = state
            .units
            .iter()
            .enumerate()
            .filter_map(|(idx, u)| {
                if u.is_alive() && u.team == target_team {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        if targets.is_empty() {
            None
        } else {
            Some(targets[self.run.rng.range_usize(targets.len())])
        }
    }
}
