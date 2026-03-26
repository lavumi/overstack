use crate::combat_math::{compute_damage, crit_chance};
use crate::data::specs::DamageKind;
use crate::engine::numeric::round_hp;
use crate::event::Event;
use crate::log::push_event;
use crate::model::{BattleOutcome, BattleState, Team, Unit};
use crate::rng::SimpleRng;

/// Creates a normal battle with one player unit and a small enemy pack.
pub fn create_battle(
    player_hp: f32,
    player_max_hp: f32,
    player_atk: i32,
    player_matk: i32,
    player_def: i32,
    player_mdef: i32,
    player_crit_rate: f32,
    player_crit_mult: f32,
    player_speed: f32,
    enemy_count: u32,
    enemy_hp: f32,
    enemy_atk: i32,
    enemy_matk: i32,
    enemy_def: i32,
    enemy_mdef: i32,
    enemy_crit_rate: f32,
    enemy_crit_mult: f32,
    enemy_speed: f32,
) -> BattleState {
    let mut units = Vec::new();
    units.push(Unit {
        id: 0,
        team: Team::Player,
        hp: round_hp(player_hp),
        max_hp: round_hp(player_max_hp),
        atk: player_atk,
        matk: player_matk,
        def: player_def,
        mdef: player_mdef,
        crit_rate: player_crit_rate.max(0.0),
        crit_mult: player_crit_mult.max(1.0),
        speed: player_speed,
        action_gauge: 0.0,
    });

    for idx in 0..enemy_count {
        units.push(Unit {
            id: idx + 1,
            team: Team::Enemy,
            hp: round_hp(enemy_hp),
            max_hp: round_hp(enemy_hp),
            atk: enemy_atk,
            matk: enemy_matk,
            def: enemy_def,
            mdef: enemy_mdef,
            crit_rate: enemy_crit_rate.max(0.0),
            crit_mult: enemy_crit_mult.max(1.0),
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
            let crit_chance = crit_chance(state.units[actor_idx].crit_rate);
            let crit = ((rng.next_u32() as f64) / (u32::MAX as f64)) < crit_chance as f64;
            let breakdown = compute_damage(
                &state.units[actor_idx],
                &state.units[target_idx],
                DamageKind::Physical,
                1.0,
                0.0,
                crit,
            );
            let target = team_to_actor(target_team);

            push_event(logs, Event::TurnReady { actor });
            push_event(
                logs,
                Event::ActionUsed {
                    actor,
                    action_name: "basic_attack",
                },
            );

            state.units[target_idx].hp =
                round_hp((state.units[target_idx].hp - breakdown.amount).max(0.0));

            push_event(
                logs,
                Event::DamageDealt {
                    src: actor,
                    dst: target,
                    damage_kind: DamageKind::Physical.as_str(),
                    raw: breakdown.raw,
                    defense_used: breakdown.defense_used,
                    mitigation: breakdown.mitigation,
                    crit: breakdown.crit,
                    amount: breakdown.amount,
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
