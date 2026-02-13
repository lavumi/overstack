use crate::model::Team;
use crate::step_api::{ActiveRun, Snapshot, StatusSnapshot, UnitSnapshot};

impl ActiveRun {
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
                player: UnitSnapshot {
                    hp: player_unit.hp,
                    max_hp: player_unit.max_hp,
                    action_gauge: player_unit.action_gauge,
                    statuses: self.to_status_snapshots(player_idx),
                },
                enemy: UnitSnapshot {
                    hp: enemy_unit.hp,
                    max_hp: enemy_unit.max_hp,
                    action_gauge: enemy_unit.action_gauge,
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
                player: UnitSnapshot {
                    hp: self.run.player_hp,
                    max_hp: self.run.player_max_hp,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
                enemy: UnitSnapshot {
                    hp: 0.0,
                    max_hp: 0.0,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
            }
        }
    }
}
