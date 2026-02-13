use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::battle::create_battle;
use crate::event::Event;
use crate::log::push_event;
use crate::model::{BattleState, NodeType, RunState};
use crate::skill::{player_skill_names, StatusType};
use crate::trait_spec::{
    active_trait_names, selectable_trait_ids, selectable_trait_names, trait_by_id, TraitId, TriggerType,
};

mod manager;

pub(crate) const TRAIT_CHAIN_DEPTH_MAX: u8 = 4;
pub(crate) const STATUS_TICK_THRESHOLD: f32 = 100.0;
pub(crate) const STATUS_TICK_RATE: f32 = 100.0;

pub(crate) fn hp2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

#[derive(Clone)]
pub(crate) struct ActiveStatus {
    pub(crate) status_type: StatusType,
    pub(crate) stacks: u32,
    pub(crate) duration: f32,
    pub(crate) power: f32,
    pub(crate) tick_meter: f32,
}

pub(crate) struct UnitRuntime {
    pub(crate) statuses: Vec<ActiveStatus>,
    pub(crate) proc_bonus: f32,
    pub(crate) res_bonus: f32,
    pub(crate) status_power_mult: HashMap<StatusType, f32>,
}

pub(crate) struct ActiveBattle {
    pub(crate) state: BattleState,
    pub(crate) runtime: Vec<UnitRuntime>,
}

impl ActiveBattle {
    pub(crate) fn new(state: BattleState) -> Self {
        let runtime = (0..state.units.len())
            .map(|_| UnitRuntime {
                statuses: Vec::new(),
                proc_bonus: 0.0,
                res_bonus: 0.0,
                status_power_mult: HashMap::new(),
            })
            .collect();
        Self { state, runtime }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ActionKind {
    BasicAttack,
    SkillSlot(u32),
}

#[derive(Clone, Copy)]
pub(crate) struct TriggerContext {
    pub(crate) trigger_type: TriggerType,
    pub(crate) src_idx: Option<usize>,
    pub(crate) dst_idx: Option<usize>,
    pub(crate) applied_status: Option<StatusType>,
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
pub struct StatusSnapshot {
    pub status_type: String,
    pub stacks: u32,
    pub duration: f32,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct UnitSnapshot {
    pub hp: f32,
    pub max_hp: f32,
    pub action_gauge: f32,
    pub statuses: Vec<StatusSnapshot>,
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

pub(crate) struct ActiveRun {
    pub(crate) seed: u64,
    pub(crate) max_nodes: u32,
    pub(crate) run: RunState,
    pub(crate) planned_nodes: [NodeType; 6],
    pub(crate) node_index: u32,
    pub(crate) battle_index: u32,
    pub(crate) current_battle: Option<ActiveBattle>,
    pub(crate) waiting_for_input: bool,
    pub(crate) ended: bool,
    pub(crate) result: &'static str,
    pub(crate) elapsed_time: f32,
    pub(crate) active_traits: Vec<TraitId>,
}

impl ActiveRun {
    pub(crate) fn new(seed: u64, max_nodes: u32) -> Self {
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
            waiting_for_input: false,
            ended: false,
            result: "none",
            elapsed_time: 0.0,
            active_traits: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new(self.seed, self.max_nodes);
    }

    pub(crate) fn active_trait_names(&self) -> Vec<String> {
        active_trait_names(&self.active_traits)
    }

    pub(crate) fn set_single_active_trait(&mut self, trait_id: &str) -> bool {
        let Some(spec) = trait_by_id(trait_id) else {
            return false;
        };
        self.active_traits.clear();
        self.active_traits.push(spec.id);
        true
    }

    pub(crate) fn current_node_type(&self) -> Option<NodeType> {
        if self.node_index == 0 {
            return None;
        }
        self.planned_nodes.get((self.node_index - 1) as usize).copied()
    }

    pub(crate) fn ensure_battle_started(&mut self, events: &mut Vec<String>) {
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
        let (battle_state, enemy_name) = match node_type {
            NodeType::Boss => (
                create_battle(
                    self.run.player_hp,
                    self.run.player_max_hp,
                    self.run.player_atk,
                    self.run.player_speed,
                    1,
                    220.0,
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
                    84.0,
                    11,
                    28.0,
                ),
                "Rogue Drone",
            ),
        };

        self.current_battle = Some(ActiveBattle::new(battle_state));

        push_event(
            events,
            Event::BattleStart {
                battle_index: self.battle_index,
                enemy_name,
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnBattleStart,
            src_idx: None,
            dst_idx: None,
            applied_status: None,
        };
        self.process_trait_triggers(context, 0, events);
    }
}

#[wasm_bindgen]
pub fn create_run(seed: u32, max_nodes: u32) -> u32 {
    manager::create_run(seed, max_nodes)
}

#[wasm_bindgen]
pub fn destroy_run(handle: u32) {
    manager::destroy_run(handle);
}

#[wasm_bindgen]
pub fn reset_run(handle: u32) -> bool {
    manager::reset_run(handle)
}

#[wasm_bindgen]
pub fn step(handle: u32, dt: f32, player_action: Option<ActionInput>) -> StepResult {
    manager::with_run_mut(handle, |run| {
        let action = player_action.map(|a| a.to_kind());
        run.step_once(dt, action)
    })
    .unwrap_or_else(|| StepResult {
        events: Vec::new(),
        need_input: false,
        ended: true,
        error: format!("invalid_handle:{handle}"),
    })
}

#[wasm_bindgen]
pub fn step_with_action(handle: u32, dt: f32, action_kind: &str, action_arg: i32) -> StepResult {
    manager::with_run_mut(handle, |run| {
        let action = match action_kind {
            "none" | "" => None,
            "basic" => Some(ActionKind::BasicAttack),
            "skill" => Some(ActionKind::SkillSlot(action_arg.clamp(0, 3) as u32)),
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
    .unwrap_or_else(|| StepResult {
        events: Vec::new(),
        need_input: false,
        ended: true,
        error: format!("invalid_handle:{handle}"),
    })
}

#[wasm_bindgen]
pub fn get_snapshot(handle: u32) -> Snapshot {
    manager::with_run(handle, |run| run.snapshot()).unwrap_or_else(|| Snapshot {
        run_state: "ended".to_string(),
        run_result: "invalid_handle".to_string(),
        node_index: 0,
        battle_index: 0,
        elapsed_time: 0.0,
        player: UnitSnapshot {
            hp: 0.0,
            max_hp: 0.0,
            action_gauge: 0.0,
            statuses: Vec::new(),
        },
        enemy: UnitSnapshot {
            hp: 0.0,
            max_hp: 0.0,
            action_gauge: 0.0,
            statuses: Vec::new(),
        },
    })
}

#[wasm_bindgen]
pub fn get_player_skills(handle: u32) -> Vec<String> {
    manager::with_run(handle, |_| player_skill_names()).unwrap_or_default()
}

#[wasm_bindgen]
pub fn get_active_traits(handle: u32) -> Vec<String> {
    manager::with_run(handle, |run| run.active_trait_names()).unwrap_or_default()
}

#[wasm_bindgen]
pub fn get_selectable_trait_names() -> Vec<String> {
    selectable_trait_names()
}

#[wasm_bindgen]
pub fn get_selectable_trait_ids() -> Vec<String> {
    selectable_trait_ids()
}

#[wasm_bindgen]
pub fn set_active_trait(handle: u32, trait_id: &str) -> bool {
    manager::with_run_mut(handle, |run| run.set_single_active_trait(trait_id)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{ActionKind, ActiveRun, TRAIT_CHAIN_DEPTH_MAX};

    #[test]
    fn ember_lash_applies_burn_sometimes_with_fixed_seed() {
        let mut run = ActiveRun::new(20260213, 1);
        let mut burn_applied = 0_u32;

        for _ in 0..50 {
            let result = run.step_once(0.15, None);
            for line in &result.events {
                if line.contains("\"kind\":\"StatusApplied\"")
                    && line.contains("\"status\":\"Burn\"")
                {
                    burn_applied += 1;
                }
            }

            if run.ended {
                break;
            }

            if result.need_input {
                let input_result = run.step_once(0.0, Some(ActionKind::SkillSlot(0)));
                for line in &input_result.events {
                    if line.contains("\"kind\":\"StatusApplied\"")
                        && line.contains("\"status\":\"Burn\"")
                    {
                        burn_applied += 1;
                    }
                }
            }

            if run.ended {
                break;
            }
        }

        assert!(burn_applied > 0, "expected Burn to be applied at least once");
    }

    #[test]
    fn trait_triggered_event_emitted_with_fixed_seed() {
        let mut run = ActiveRun::new(424242, 1);
        assert!(run.set_single_active_trait("overcharge"));
        let mut triggered_count = 0_u32;

        for _ in 0..50 {
            let result = run.step_once(0.15, None);
            for line in &result.events {
                if line.contains("\"kind\":\"TraitTriggered\"") {
                    triggered_count += 1;
                }
            }

            if result.need_input {
                let input_result = run.step_once(0.0, Some(ActionKind::SkillSlot(2)));
                for line in &input_result.events {
                    if line.contains("\"kind\":\"TraitTriggered\"") {
                        triggered_count += 1;
                    }
                }
            }

            if run.ended {
                break;
            }
        }

        assert!(triggered_count > 0, "expected at least one trait trigger event");
    }

    #[test]
    fn trait_chain_depth_guard_keeps_event_count_bounded() {
        let mut run = ActiveRun::new(777, 1);
        assert!(run.set_single_active_trait("overcharge"));
        let mut max_events = 0_usize;

        for _ in 0..40 {
            let result = run.step_once(0.15, None);
            max_events = max_events.max(result.events.len());
            if result.need_input {
                let input_result = run.step_once(0.0, Some(ActionKind::SkillSlot(2)));
                max_events = max_events.max(input_result.events.len());
            }
            if run.ended {
                break;
            }
        }

        assert!(
            max_events < 300,
            "expected event count per step to stay bounded, got {max_events}, depth cap {}",
            TRAIT_CHAIN_DEPTH_MAX
        );
    }
}
