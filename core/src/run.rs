use crate::battle::{create_battle, player_hp_after_battle, run_battle};
use crate::event::Event;
use crate::log::push_event;
use crate::model::{BattleOutcome, NodeType, RunState};

/// Runs one full run skeleton: normal battle nodes + final boss node.
pub fn run_run_internal(seed: u64, max_nodes: u32) -> Vec<String> {
    let mut run = RunState::new(seed);
    let mut logs = Vec::new();

    let planned_nodes: [NodeType; 6] = [
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Battle,
        NodeType::Boss,
    ];

    let node_limit = max_nodes.min(planned_nodes.len() as u32) as usize;

    push_event(
        &mut logs,
        Event::RunStart { seed: run.seed },
    );

    let mut final_node_index = 0_u32;

    for (i, node_type) in planned_nodes.iter().take(node_limit).enumerate() {
        run.stage = (i as u32) + 1;
        final_node_index = run.stage;
        let node_type_label = match node_type {
            NodeType::Battle => "Battle",
            NodeType::Boss => "Boss",
            NodeType::Event => "Event",
            NodeType::Shop => "Shop",
            NodeType::Rest => "Rest",
        };
        push_event(
            &mut logs,
            Event::NodeStart {
                node_index: run.stage,
                node_type: node_type_label,
            },
        );

        let mut battle = match node_type {
            NodeType::Battle => create_battle(
                run.player_hp, run.player_max_hp, run.player_atk, run.player_speed, 1, 84, 11, 28.0,
            ),
            NodeType::Boss => create_battle(
                run.player_hp,
                run.player_max_hp,
                run.player_atk,
                run.player_speed,
                1,
                220,
                14,
                32.0,
            ),
            _ => continue,
        };
        let enemy_name = match node_type {
            NodeType::Battle => "Rogue Drone",
            NodeType::Boss => "Overstack Core",
            _ => "Unknown",
        };

        match run_battle(&mut battle, &mut run.rng, run.stage, enemy_name, &mut logs) {
            BattleOutcome::Victory => {
                run.player_hp = player_hp_after_battle(&battle);

                // Temporary sustain rule for skeleton pacing.
                let recover = ((run.player_max_hp as f32) * 0.20).round() as i32;
                run.player_hp = (run.player_hp + recover).min(run.player_max_hp);
            }
            BattleOutcome::Defeat => {
                run.player_hp = 0;
                push_event(
                    &mut logs,
                    Event::RunEnd {
                        result: "lose",
                        final_node_index,
                    },
                );
                return logs;
            }
        }
    }

    run.meta_placeholder = final_node_index;
    push_event(
        &mut logs,
        Event::RunEnd {
            result: "win",
            final_node_index,
        },
    );
    logs
}
