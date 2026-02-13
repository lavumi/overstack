use crate::battle::{create_battle, player_hp_after_battle, run_battle};
use crate::log::log_line;
use crate::model::{BattleOutcome, NodeType, RunState};

/// Runs one full run skeleton: normal battle nodes + final boss node.
pub fn run_run_internal(seed: u64, max_nodes: u32) -> u32 {
    let mut run = RunState::new(seed);

    let planned_nodes: [NodeType; 6] = [
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Boss,
    ];

    let node_limit = max_nodes.min(planned_nodes.len() as u32) as usize;

    log_line(&format!(
        "[run:start] seed={} max_nodes={} executing_nodes={}",
        run.seed, max_nodes, node_limit
    ));

    let mut cleared_nodes = 0_u32;

    for (i, node_type) in planned_nodes.iter().take(node_limit).enumerate() {
        run.stage = (i as u32) + 1;
        let label = format!("node={} type={:?}", run.stage, node_type);

        let mut battle = match node_type {
            NodeType::Battle => create_battle(
                run.player_hp,
                run.player_max_hp,
                run.player_atk,
                run.player_speed,
                2,
                42,
                9,
                28.0,
            ),
            NodeType::Boss => create_battle(
                run.player_hp,
                run.player_max_hp,
                run.player_atk,
                run.player_speed,
                1,
                190,
                14,
                32.0,
            ),
            _ => continue,
        };

        match run_battle(&mut battle, &mut run.rng, &label) {
            BattleOutcome::Victory => {
                cleared_nodes += 1;
                run.player_hp = player_hp_after_battle(&battle);

                // Temporary sustain rule for skeleton pacing.
                let recover = ((run.player_max_hp as f32) * 0.20).round() as i32;
                run.player_hp = (run.player_hp + recover).min(run.player_max_hp);

                log_line(&format!(
                    "[run:node_clear] node={} player_hp={} recovered={}",
                    run.stage, run.player_hp, recover
                ));
            }
            BattleOutcome::Defeat => {
                run.player_hp = 0;
                log_line(&format!(
                    "[run:fail] node={} floor={} stage={}",
                    run.stage, run.floor, run.stage
                ));
                return cleared_nodes;
            }
        }
    }

    run.meta_placeholder = cleared_nodes;
    log_line(&format!(
        "[run:end] cleared_nodes={} floor={} stage={} meta_placeholder={}",
        cleared_nodes, run.floor, run.stage, run.meta_placeholder
    ));
    cleared_nodes
}
