use crate::model::Team;
use crate::step_api::{ActiveRun, Snapshot, StatusSnapshot, UnitSnapshot};

impl ActiveRun {
    fn enemy_next_intent_text(&self) -> String {
        let Some(state) = self.state_ref() else {
            return "-".to_string();
        };

        let enemy_ready = state
            .units
            .iter()
            .any(|u| u.team == Team::Enemy && u.is_alive() && u.action_gauge >= 100.0);
        if enemy_ready {
            "Basic Attack (ready)".to_string()
        } else if state.units.iter().any(|u| u.team == Team::Enemy && u.is_alive()) {
            "Basic Attack".to_string()
        } else {
            "-".to_string()
        }
    }

    fn to_status_snapshots(&self, unit_idx: usize) -> Vec<StatusSnapshot> {
        self.statuses_ref(unit_idx)
            .map(|row| {
                row.iter()
                    .filter(|s| s.duration > 0.0)
                    .map(|s| StatusSnapshot {
                        status_type: s.status_type.as_str().to_string(),
                        stacks: s.stacks,
                        duration: s.duration.max(0.0),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        if let Some(battle) = &self.current_battle {
            let player_idx = battle
                .state
                .units
                .iter()
                .enumerate()
                .find(|(_, u)| u.team == Team::Player)
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            let enemy_idx = battle
                .state
                .units
                .iter()
                .enumerate()
                .find(|(_, u)| u.team == Team::Enemy)
                .map(|(idx, _)| idx)
                .unwrap_or(0);

            let player_unit = &battle.state.units[player_idx];
            let enemy_unit = &battle.state.units[enemy_idx];

            Snapshot {
                run_state: if self.ended {
                    "ended".to_string()
                } else {
                    "running".to_string()
                },
                run_result: self.result.to_string(),
                node_index: self.node_index,
                battle_index: self.battle_index,
                elapsed_time: self.elapsed_time,
                enemy_next_intent: self.enemy_next_intent_text(),
                player_traits: self.active_trait_names(),
                enemy_traits: self.enemy_trait_names(),
                player: UnitSnapshot {
                    hp: player_unit.hp,
                    max_hp: player_unit.max_hp,
                    action_gauge: player_unit.action_gauge,
                    atk: player_unit.atk,
                    matk: player_unit.matk,
                    base_def: player_unit.def,
                    effective_def: self.effective_def(player_idx),
                    mdef: player_unit.mdef,
                    crit_rate: player_unit.crit_rate,
                    statuses: self.to_status_snapshots(player_idx),
                },
                enemy: UnitSnapshot {
                    hp: enemy_unit.hp,
                    max_hp: enemy_unit.max_hp,
                    action_gauge: enemy_unit.action_gauge,
                    atk: enemy_unit.atk,
                    matk: enemy_unit.matk,
                    base_def: enemy_unit.def,
                    effective_def: self.effective_def(enemy_idx),
                    mdef: enemy_unit.mdef,
                    crit_rate: enemy_unit.crit_rate,
                    statuses: self.to_status_snapshots(enemy_idx),
                },
            }
        } else {
            Snapshot {
                run_state: if self.ended {
                    "ended".to_string()
                } else {
                    "running".to_string()
                },
                run_result: self.result.to_string(),
                node_index: self.node_index,
                battle_index: self.battle_index,
                elapsed_time: self.elapsed_time,
                enemy_next_intent: "-".to_string(),
                player_traits: self.active_trait_names(),
                enemy_traits: self.enemy_trait_names(),
                player: UnitSnapshot {
                    hp: self.run.player_hp,
                    max_hp: self.run.player_max_hp,
                    action_gauge: 0.0,
                    atk: self.run.player_atk,
                    matk: self.run.player_matk,
                    base_def: self.run.player_def,
                    effective_def: self.run.player_def,
                    mdef: self.run.player_mdef,
                    crit_rate: self.run.player_crit_rate,
                    statuses: Vec::new(),
                },
                enemy: UnitSnapshot {
                    hp: 0.0,
                    max_hp: 0.0,
                    action_gauge: 0.0,
                    atk: 0,
                    matk: 0,
                    base_def: 0,
                    effective_def: 0,
                    mdef: 0,
                    crit_rate: 0.0,
                    statuses: Vec::new(),
                },
            }
        }
    }
}
