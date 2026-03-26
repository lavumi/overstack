use wasm_bindgen::prelude::*;

use crate::engine::runtime::ActionKind;
use crate::model::PlayerInitStats;
use crate::skill::player_skill_names;
use crate::trait_spec::{selectable_trait_ids, selectable_trait_names};

mod manager;

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
    pub atk: i32,
    pub matk: i32,
    pub base_def: i32,
    pub effective_def: i32,
    pub mdef: i32,
    pub crit_rate: f32,
    pub statuses: Vec<StatusSnapshot>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Snapshot {
    pub run_state: String,
    pub run_result: String,
    pub node_index: u32,
    pub battle_index: u32,
    pub elapsed_time: f32,
    pub enemy_next_intent: String,
    pub player_traits: Vec<String>,
    pub enemy_traits: Vec<String>,
    pub player: UnitSnapshot,
    pub enemy: UnitSnapshot,
}

#[wasm_bindgen]
pub fn create_run(seed: u32, max_nodes: u32) -> u32 {
    manager::create_run(seed, max_nodes)
}

#[wasm_bindgen]
pub fn create_run_with_stats(
    seed: u32,
    max_nodes: u32,
    max_hp: f32,
    atk: i32,
    matk: i32,
    def: i32,
    mdef: i32,
    speed: f32,
    crit_rate: f32,
    crit_mult: f32,
) -> u32 {
    let stats = PlayerInitStats {
        max_hp,
        atk,
        matk,
        def,
        mdef,
        speed,
        crit_rate,
        crit_mult,
    };
    manager::create_run_with_stats(seed, max_nodes, stats)
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
        enemy_next_intent: "-".to_string(),
        player_traits: Vec::new(),
        enemy_traits: Vec::new(),
        player: UnitSnapshot {
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
    use crate::engine::runtime::{
        ActionKind, ActiveRun, TraitOwner, TriggerContext, TRAIT_CHAIN_DEPTH_MAX,
    };
    use crate::skill::{EffectSpec, StatusType};
    use crate::trait_spec::TriggerType;

    #[test]
    fn ember_lash_applies_burn_sometimes_with_fixed_seed() {
        let mut run = ActiveRun::new(1234, 1);
        let mut burn_applied = 0_u32;

        for _ in 0..80 {
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

        assert!(
            burn_applied > 0,
            "expected Burn to be applied at least once"
        );
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

        assert!(
            triggered_count > 0,
            "expected at least one trait trigger event"
        );
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

    #[test]
    fn enemy_trait_trigger_event_is_emitted() {
        let mut run = ActiveRun::new(1234, 1);
        let result = run.step_once(0.01, None);
        let has_enemy_trigger = result.events.iter().any(|line| {
            line.contains("\"kind\":\"TraitTriggered\"")
                && line.contains("\"owner\":\"enemy\"")
                && line.contains("Iron Shell")
        });
        assert!(
            has_enemy_trigger,
            "expected enemy trait trigger event from battle start"
        );
    }

    #[test]
    fn remove_status_effect_clears_existing_status() {
        let mut run = ActiveRun::new(1234, 1);
        let mut events = Vec::new();
        run.ensure_battle_started(&mut events);

        run.apply_status(0, 0, StatusType::Burn, 1.0, 4.0, 1, 1.0, 0, &mut events);
        assert!(run.has_status(0, StatusType::Burn));

        let removed = run.execute_primitive_effect(
            EffectSpec::RemoveStatus {
                target: crate::skill::EffectTarget::Dst,
                status_type: StatusType::Burn,
            },
            0,
            0,
            TriggerContext {
                trigger_type: TriggerType::OnActionUsed,
                owner: TraitOwner::Player,
                src_idx: Some(0),
                dst_idx: Some(0),
                applied_status: None,
            },
            0,
            &mut events,
        );

        assert_eq!(removed.as_deref(), Some("RemoveStatus Burn"));
        assert!(!run.has_status(0, StatusType::Burn));
        assert!(events.iter().any(|line| {
            line.contains("\"kind\":\"StatusExpired\"") && line.contains("\"status\":\"Burn\"")
        }));
    }
}
