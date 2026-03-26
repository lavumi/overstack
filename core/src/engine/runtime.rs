use std::collections::HashMap;

use crate::data::{ErrorReport, GameData};
use crate::model::{BattleState, NodeType, RunState};
use crate::skill::StatusType;
use crate::trait_spec::{TraitId, TriggerType};

pub(crate) const TRAIT_CHAIN_DEPTH_MAX: u8 = 4;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TraitOwner {
    Player,
    Enemy,
}

#[derive(Clone, Copy)]
pub(crate) struct TriggerContext {
    pub(crate) trigger_type: TriggerType,
    pub(crate) owner: TraitOwner,
    pub(crate) src_idx: Option<usize>,
    pub(crate) dst_idx: Option<usize>,
    pub(crate) applied_status: Option<StatusType>,
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
    pub(crate) player_traits: Vec<TraitId>,
    pub(crate) enemy_traits: Vec<TraitId>,
    pub(crate) game_data: Option<&'static GameData>,
    pub(crate) data_error: Option<ErrorReport>,
}
