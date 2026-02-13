use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::battle::{create_battle, player_hp_after_battle};
use crate::event::Event;
use crate::log::push_event;
use crate::model::{BattleState, NodeType, RunState, Team};

#[derive(Clone, Copy)]
enum ActionKind {
    BasicAttack,
    SkillSlot(u32),
}

#[wasm_bindgen]
pub struct ActionInput {
    kind: u8,
    index: u32,
}

#[wasm_bindgen]
impl ActionInput {
    pub fn basic_attack() -> ActionInput {
        ActionInput { kind: 0, index: 0 }
    }

    pub fn skill_slot(index: u32) -> ActionInput {
        ActionInput { kind: 1, index }
    }
}

impl ActionInput {
    fn to_kind(&self) -> ActionKind {
        match self.kind {
            1 => ActionKind::SkillSlot(self.index.min(3)),
            _ => ActionKind::BasicAttack,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct StepResult {
    pub events: Vec<String>,
    pub need_input: bool,
    pub ended: bool,
    pub error: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct UnitSnapshot {
    pub hp: i32,
    pub max_hp: i32,
    pub action_gauge: f32,
    pub statuses: Vec<String>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Snapshot {
    pub run_state: String,
    pub run_result: String,
    pub node_index: u32,
    pub battle_index: u32,
    pub elapsed_time: f32,
    pub player: UnitSnapshot,
    pub enemy: UnitSnapshot,
}

struct ActiveRun {
    seed: u64,
    max_nodes: u32,
    run: RunState,
    planned_nodes: [NodeType; 6],
    node_index: u32,
    battle_index: u32,
    current_battle: Option<BattleState>,
    current_enemy_name: &'static str,
    waiting_for_input: bool,
    ended: bool,
    result: &'static str,
    elapsed_time: f32,
}

impl ActiveRun {
    fn new(seed: u64, max_nodes: u32) -> Self {
        Self {
            seed,
            max_nodes: max_nodes.min(6),
            run: RunState::new(seed),
            planned_nodes: [
                NodeType::Battle,
                NodeType::Battle,
                NodeType::Battle,
                NodeType::Battle,
                NodeType::Battle,
                NodeType::Boss,
            ],
            node_index: 0,
            battle_index: 0,
            current_battle: None,
            current_enemy_name: "",
            waiting_for_input: false,
            ended: false,
            result: "none",
            elapsed_time: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new(self.seed, self.max_nodes);
    }

    fn current_node_type(&self) -> Option<NodeType> {
        if self.node_index == 0 {
            return None;
        }
        self.planned_nodes.get((self.node_index - 1) as usize).copied()
    }

    fn ensure_battle_started(&mut self, events: &mut Vec<String>) {
        if self.current_battle.is_some() || self.ended {
            return;
        }

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
            return;
        }

        self.node_index += 1;
        let node_type = self.current_node_type().unwrap_or(NodeType::Battle);
        let node_type_label = match node_type {
            NodeType::Boss => "Boss",
            _ => "Battle",
        };

        push_event(
            events,
            Event::NodeStart {
                node_index: self.node_index,
                node_type: node_type_label,
            },
        );

        self.battle_index += 1;
        let (battle, enemy_name) = match node_type {
            NodeType::Boss => (
                create_battle(
                    self.run.player_hp,
                    self.run.player_max_hp,
                    self.run.player_atk,
                    self.run.player_speed,
                    1,
                    220,
                    14,
                    32.0,
                ),
                "Overstack Core",
            ),
            _ => (
                create_battle(
                    self.run.player_hp,
                    self.run.player_max_hp,
                    self.run.player_atk,
                    self.run.player_speed,
                    1,
                    84,
                    11,
                    28.0,
                ),
                "Rogue Drone",
            ),
        };

        self.current_enemy_name = enemy_name;
        self.current_battle = Some(battle);

        push_event(
            events,
            Event::BattleStart {
                battle_index: self.battle_index,
                enemy_name,
            },
        );
    }

    fn execute_attack(
        &mut self,
        actor_idx: usize,
        action: ActionKind,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
        let battle = match &mut self.current_battle {
            Some(b) => b,
            None => return None,
        };

        if !battle.units[actor_idx].is_alive() || battle.units[actor_idx].action_gauge < 100.0 {
            return None;
        }

        battle.units[actor_idx].action_gauge -= 100.0;

        let actor_team = battle.units[actor_idx].team;
        let actor = match actor_team {
            Team::Player => "player",
            Team::Enemy => "enemy",
        };
        let target_team = if actor_team == Team::Player {
            Team::Enemy
        } else {
            Team::Player
        };

        let targets: Vec<usize> = battle
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
            return None;
        }

        let target_idx = targets[self.run.rng.range_usize(targets.len())];
        let target = if target_team == Team::Player {
            "player"
        } else {
            "enemy"
        };

        let (action_name, bonus) = match action {
            ActionKind::BasicAttack => ("basic_attack", 0),
            ActionKind::SkillSlot(index) => {
                let name = match index {
                    0 => "skill_slot_0",
                    1 => "skill_slot_1",
                    2 => "skill_slot_2",
                    _ => "skill_slot_3",
                };
                (name, 5 + (index as i32 * 2))
            }
        };

        push_event(events, Event::TurnReady { actor });
        push_event(
            events,
            Event::ActionUsed {
                actor,
                action_name,
            },
        );

        let damage = (battle.units[actor_idx].atk + bonus).max(1);
        battle.units[target_idx].hp = (battle.units[target_idx].hp - damage).max(0);

        push_event(
            events,
            Event::DamageDealt {
                src: actor,
                dst: target,
                amount: damage,
                dst_hp_after: battle.units[target_idx].hp,
            },
        );

        push_event(
            events,
            Event::StatusApplied {
                src: actor,
                dst: target,
                status: "burn",
                stacks: 1,
                duration: 1,
            },
        );
        push_event(
            events,
            Event::StatusTick {
                dst: target,
                status: "burn",
                amount: 0,
                dst_hp_after: battle.units[target_idx].hp,
            },
        );
        push_event(
            events,
            Event::StatusExpired {
                dst: target,
                status: "burn",
            },
        );

        let enemy_alive = battle
            .units
            .iter()
            .any(|u| u.team == Team::Enemy && u.is_alive());
        let player_alive = battle
            .units
            .iter()
            .any(|u| u.team == Team::Player && u.is_alive());

        if !enemy_alive {
            let hp_after = player_hp_after_battle(battle);
            push_event(
                events,
                Event::BattleEnd {
                    result: "win",
                    player_hp_after: hp_after,
                },
            );
            return Some("win");
        }

        if !player_alive {
            push_event(
                events,
                Event::BattleEnd {
                    result: "lose",
                    player_hp_after: 0,
                },
            );
            return Some("lose");
        }

        None
    }

    fn finalize_battle(&mut self, outcome: &'static str, events: &mut Vec<String>) {
        if outcome == "win" {
            if let Some(battle) = &self.current_battle {
                self.run.player_hp = player_hp_after_battle(battle);
            }
            let recover = ((self.run.player_max_hp as f32) * 0.20).round() as i32;
            self.run.player_hp = (self.run.player_hp + recover).min(self.run.player_max_hp);
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
            self.run.player_hp = 0;
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

    fn step_once(&mut self, dt: f32, action: Option<ActionKind>) -> StepResult {
        let mut events = Vec::new();

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
            let step_dt = if remaining > 0.0 { remaining.min(0.1) } else { 0.0 };
            remaining = (remaining - step_dt).max(0.0);
            self.elapsed_time += step_dt;

            if step_dt > 0.0 {
                if let Some(battle) = self.current_battle.as_mut() {
                    for unit in &mut battle.units {
                        if unit.is_alive() {
                            unit.action_gauge += unit.speed * step_dt;
                        }
                    }
                }
            }

            loop {
                let Some((actor_idx, actor_team)) = ({
                    if let Some(battle) = self.current_battle.as_ref() {
                        let mut ready_indices: Vec<usize> = battle
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
                                battle.units[b]
                                    .action_gauge
                                    .partial_cmp(&battle.units[a].action_gauge)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });

                            let idx = ready_indices[0];
                            Some((idx, battle.units[idx].team))
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
                    if let Some(outcome) = self.execute_attack(actor_idx, action_kind, &mut events) {
                        self.finalize_battle(outcome, &mut events);
                        break;
                    }
                } else if let Some(outcome) = self.execute_attack(actor_idx, ActionKind::BasicAttack, &mut events) {
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

    fn snapshot(&self) -> Snapshot {
        let (player, enemy) = if let Some(battle) = &self.current_battle {
            let player_unit = battle
                .units
                .iter()
                .find(|u| u.team == Team::Player)
                .map(|u| UnitSnapshot {
                    hp: u.hp,
                    max_hp: u.max_hp,
                    action_gauge: u.action_gauge,
                    statuses: Vec::new(),
                })
                .unwrap_or(UnitSnapshot {
                    hp: self.run.player_hp,
                    max_hp: self.run.player_max_hp,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                });

            let enemy_hp = battle
                .units
                .iter()
                .filter(|u| u.team == Team::Enemy && u.is_alive())
                .map(|u| u.hp)
                .sum();
            let enemy_max_hp = battle
                .units
                .iter()
                .filter(|u| u.team == Team::Enemy)
                .map(|u| u.max_hp)
                .sum();
            let enemy_gauge = battle
                .units
                .iter()
                .filter(|u| u.team == Team::Enemy && u.is_alive())
                .map(|u| u.action_gauge)
                .fold(0.0_f32, f32::max);

            (
                player_unit,
                UnitSnapshot {
                    hp: enemy_hp,
                    max_hp: enemy_max_hp,
                    action_gauge: enemy_gauge,
                    statuses: Vec::new(),
                },
            )
        } else {
            (
                UnitSnapshot {
                    hp: self.run.player_hp,
                    max_hp: self.run.player_max_hp,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
                UnitSnapshot {
                    hp: 0,
                    max_hp: 0,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
            )
        };

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
            player,
            enemy,
        }
    }
}

#[derive(Default)]
struct RunManager {
    next_handle: u32,
    runs: HashMap<u32, ActiveRun>,
}

impl RunManager {
    fn create_run(&mut self, seed: u32, max_nodes: u32) -> u32 {
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        let handle = self.next_handle;
        self.runs
            .insert(handle, ActiveRun::new(seed as u64, max_nodes));
        handle
    }

    fn destroy_run(&mut self, handle: u32) {
        self.runs.remove(&handle);
    }

    fn reset_run(&mut self, handle: u32) -> bool {
        if let Some(run) = self.runs.get_mut(&handle) {
            run.reset();
            true
        } else {
            false
        }
    }
}

thread_local! {
    static MANAGER: RefCell<RunManager> = RefCell::new(RunManager::default());
}

#[wasm_bindgen]
pub fn create_run(seed: u32, max_nodes: u32) -> u32 {
    MANAGER.with(|manager| manager.borrow_mut().create_run(seed, max_nodes))
}

#[wasm_bindgen]
pub fn destroy_run(handle: u32) {
    MANAGER.with(|manager| manager.borrow_mut().destroy_run(handle));
}

#[wasm_bindgen]
pub fn reset_run(handle: u32) -> bool {
    MANAGER.with(|manager| manager.borrow_mut().reset_run(handle))
}

#[wasm_bindgen]
pub fn step(handle: u32, dt: f32, player_action: Option<ActionInput>) -> StepResult {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        let Some(run) = manager.runs.get_mut(&handle) else {
            return StepResult {
                events: Vec::new(),
                need_input: false,
                ended: true,
                error: format!("invalid_handle:{handle}"),
            };
        };

        let action = player_action.map(|a| a.to_kind());
        run.step_once(dt, action)
    })
}

#[wasm_bindgen]
pub fn step_with_action(handle: u32, dt: f32, action_kind: &str, action_arg: i32) -> StepResult {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        let Some(run) = manager.runs.get_mut(&handle) else {
            return StepResult {
                events: Vec::new(),
                need_input: false,
                ended: true,
                error: format!("invalid_handle:{handle}"),
            };
        };

        let action = match action_kind {
            "none" | "" => None,
            "basic" => Some(ActionKind::BasicAttack),
            "skill" => {
                let clamped = action_arg.clamp(0, 3) as u32;
                Some(ActionKind::SkillSlot(clamped))
            }
            _ => {
                return StepResult {
                    events: Vec::new(),
                    need_input: false,
                    ended: run.ended,
                    error: format!("invalid_action:{action_kind}"),
                };
            }
        };

        run.step_once(dt, action)
    })
}

#[wasm_bindgen]
pub fn get_snapshot(handle: u32) -> Snapshot {
    MANAGER.with(|manager| {
        let manager = manager.borrow();
        if let Some(run) = manager.runs.get(&handle) {
            run.snapshot()
        } else {
            Snapshot {
                run_state: "ended".to_string(),
                run_result: "invalid_handle".to_string(),
                node_index: 0,
                battle_index: 0,
                elapsed_time: 0.0,
                player: UnitSnapshot {
                    hp: 0,
                    max_hp: 0,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
                enemy: UnitSnapshot {
                    hp: 0,
                    max_hp: 0,
                    action_gauge: 0.0,
                    statuses: Vec::new(),
                },
            }
        }
    })
}
