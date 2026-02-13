use crate::event::Event;
use crate::log::push_event;
use crate::model::{BattleOutcome, BattleState, Team, Unit};
use crate::rng::SimpleRng;

fn hp2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

/// Creates a normal battle with one player unit and a small enemy pack.
pub fn create_battle(
    player_hp: f32,
    player_max_hp: f32,
    player_atk: i32,
    player_speed: f32,
    enemy_count: u32,
    enemy_hp: f32,
    enemy_atk: i32,
    enemy_speed: f32,
) -> BattleState {
    let mut units = Vec::new();
    units.push(Unit {
        id: 0,
        team: Team::Player,
        hp: hp2(player_hp),
        max_hp: hp2(player_max_hp),
        atk: player_atk,
        speed: player_speed,
        action_gauge: 0.0,
    });

    for idx in 0..enemy_count {
        units.push(Unit {
            id: idx + 1,
            team: Team::Enemy,
            hp: hp2(enemy_hp),
            max_hp: hp2(enemy_hp),
            atk: enemy_atk,
            speed: enemy_speed,
            action_gauge: 0.0,
        });
    }

    BattleState {
        units,
        delta_time: 1.0,
        tick: 0,
    }
}

/// Runs gauge-based battle ticks until victory/defeat is decided.
pub fn run_battle(
    state: &mut BattleState,
    rng: &mut SimpleRng,
    battle_index: u32,
    enemy_name: &'static str,
    logs: &mut Vec<String>,
) -> BattleOutcome {
    push_event(
        logs,
        Event::BattleStart {
            battle_index,
            enemy_name,
        },
    );

    let hard_tick_limit = 20_000;
    while state.tick < hard_tick_limit {
        state.tick += 1;

        for unit in &mut state.units {
            if unit.is_alive() {
                unit.action_gauge += unit.speed * state.delta_time;
            }
        }

        let mut ready_indices: Vec<usize> = state
            .units
            .iter()
            .enumerate()
            .filter_map(|(i, u)| {
                if u.is_alive() && u.action_gauge >= 100.0 {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        ready_indices.sort_by(|&a, &b| {
            state.units[b]
                .action_gauge
                .partial_cmp(&state.units[a].action_gauge)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for actor_idx in ready_indices {
            if !state.units[actor_idx].is_alive() || state.units[actor_idx].action_gauge < 100.0 {
                continue;
            }

            state.units[actor_idx].action_gauge -= 100.0;

            let actor_team = state.units[actor_idx].team;
            let actor = team_to_actor(actor_team);
            let target_team = if actor_team == Team::Player {
                Team::Enemy
            } else {
                Team::Player
            };

            let target_indices: Vec<usize> = state
                .units
                .iter()
                .enumerate()
                .filter_map(|(i, u)| {
                    if u.is_alive() && u.team == target_team {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            if target_indices.is_empty() {
                continue;
            }

            let target_idx = target_indices[rng.range_usize(target_indices.len())];
            let damage = (state.units[actor_idx].atk as f32).max(0.01);
            let target = team_to_actor(target_team);

            push_event(logs, Event::TurnReady { actor });
            push_event(
                logs,
                Event::ActionUsed {
                    actor,
                    action_name: "basic_attack",
                },
            );

            state.units[target_idx].hp = hp2((state.units[target_idx].hp - damage).max(0.0));

            push_event(
                logs,
                Event::DamageDealt {
                    src: actor,
                    dst: target,
                    amount: damage,
                    dst_hp_after: state.units[target_idx].hp,
                },
            );

            push_event(
                logs,
                Event::StatusApplied {
                    src: actor,
                    dst: target,
                    status: "burn",
                    stacks: 1,
                    duration: 1,
                },
            );
            push_event(
                logs,
                Event::StatusTick {
                    dst: target,
                    status: "burn",
                    amount: 0.0,
                    dst_hp_after: state.units[target_idx].hp,
                },
            );
            push_event(
                logs,
                Event::StatusExpired {
                    dst: target,
                    status: "burn",
                },
            );

            if !has_alive(&state.units, Team::Enemy) {
                push_event(
                    logs,
                    Event::BattleEnd {
                        result: "win",
                        player_hp_after: player_hp_after_battle(state),
                    },
                );
                return BattleOutcome::Victory;
            }

            if !has_alive(&state.units, Team::Player) {
                push_event(
                    logs,
                    Event::BattleEnd {
                        result: "lose",
                        player_hp_after: 0.0,
                    },
                );
                return BattleOutcome::Defeat;
            }
        }
    }

    push_event(
        logs,
        Event::BattleEnd {
            result: "lose",
            player_hp_after: player_hp_after_battle(state),
        },
    );
    BattleOutcome::Defeat
}

pub fn player_hp_after_battle(state: &BattleState) -> f32 {
    state
        .units
        .iter()
        .find(|u| u.team == Team::Player)
        .map(|u| u.hp)
        .unwrap_or(0.0)
}

fn has_alive(units: &[Unit], team: Team) -> bool {
    units.iter().any(|u| u.team == team && u.is_alive())
}

fn team_to_actor(team: Team) -> &'static str {
    match team {
        Team::Player => "player",
        Team::Enemy => "enemy",
    }
}
