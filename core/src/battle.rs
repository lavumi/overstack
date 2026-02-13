use crate::log::log_line;
use crate::model::{BattleOutcome, BattleState, Team, Unit};
use crate::rng::SimpleRng;

/// Creates a normal battle with one player unit and a small enemy pack.
pub fn create_battle(
    player_hp: i32,
    player_max_hp: i32,
    player_atk: i32,
    player_speed: f32,
    enemy_count: u32,
    enemy_hp: i32,
    enemy_atk: i32,
    enemy_speed: f32,
) -> BattleState {
    let mut units = Vec::new();
    units.push(Unit {
        id: 0,
        team: Team::Player,
        hp: player_hp,
        max_hp: player_max_hp,
        atk: player_atk,
        speed: player_speed,
        action_gauge: 0.0,
    });

    for idx in 0..enemy_count {
        units.push(Unit {
            id: idx + 1,
            team: Team::Enemy,
            hp: enemy_hp,
            max_hp: enemy_hp,
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
pub fn run_battle(state: &mut BattleState, rng: &mut SimpleRng, label: &str) -> BattleOutcome {
    log_line(&format!("[battle:start] {label}"));

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
            let damage = state.units[actor_idx].atk.max(1);
            let actor_id = state.units[actor_idx].id;
            let target_id = state.units[target_idx].id;

            state.units[target_idx].hp -= damage;
            if state.units[target_idx].hp < 0 {
                state.units[target_idx].hp = 0;
            }

            log_line(&format!(
                "[battle:act] tick={} actor={:?}#{} -> target={:?}#{} dmg={} target_hp={}",
                state.tick,
                actor_team,
                actor_id,
                target_team,
                target_id,
                damage,
                state.units[target_idx].hp
            ));

            if !has_alive(&state.units, Team::Enemy) {
                log_line(&format!("[battle:end] {label} => VICTORY"));
                return BattleOutcome::Victory;
            }

            if !has_alive(&state.units, Team::Player) {
                log_line(&format!("[battle:end] {label} => DEFEAT"));
                return BattleOutcome::Defeat;
            }
        }
    }

    log_line(&format!(
        "[battle:end] {label} => DEFEAT (tick limit reached)"
    ));
    BattleOutcome::Defeat
}

pub fn player_hp_after_battle(state: &BattleState) -> i32 {
    state
        .units
        .iter()
        .find(|u| u.team == Team::Player)
        .map(|u| u.hp)
        .unwrap_or(0)
}

fn has_alive(units: &[Unit], team: Team) -> bool {
    units.iter().any(|u| u.team == team && u.is_alive())
}
