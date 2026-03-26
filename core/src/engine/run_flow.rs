use crate::battle::create_battle;
use crate::data::load_embedded_game_data;
use crate::event::Event;
use crate::log::push_event;
use crate::model::{NodeType, PlayerInitStats, RunState};
use crate::rng::SimpleRng;
use crate::step_api::{ActiveBattle, ActiveRun, TraitOwner, TriggerContext};
use crate::trait_spec::{
    active_trait_names, calc_traits_cost as calc_selected_traits_cost, sample_trait_choices,
    trait_by_id, TraitId, TriggerType,
};

impl ActiveRun {
    pub(crate) fn new(seed: u64, max_nodes: u32) -> Self {
        Self::new_with_stats(seed, max_nodes, None)
    }

    pub(crate) fn new_with_stats(
        seed: u64,
        max_nodes: u32,
        player_stats: Option<PlayerInitStats>,
    ) -> Self {
        let (game_data, data_error) = match load_embedded_game_data() {
            Ok(data) => (Some(data), None),
            Err(err) => (None, Some(err)),
        };

        let mut run = RunState::new(seed);
        if let Some(stats) = player_stats {
            run.apply_player_stats(stats);
        }

        Self {
            seed,
            max_nodes: max_nodes.min(6),
            run,
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
            player_traits: Vec::new(),
            enemy_traits: Vec::new(),
            game_data,
            data_error,
        }
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new(self.seed, self.max_nodes);
    }

    pub(crate) fn active_trait_names(&self) -> Vec<String> {
        active_trait_names(&self.player_traits)
    }

    pub(crate) fn enemy_trait_names(&self) -> Vec<String> {
        active_trait_names(&self.enemy_traits)
    }

    pub(crate) fn set_single_active_trait(&mut self, trait_id: &str) -> bool {
        self.apply_traits_to_run(&[trait_id])
    }

    pub(crate) fn apply_traits_to_run(&mut self, selected_trait_ids: &[&str]) -> bool {
        let mut next_traits = Vec::new();

        for trait_id in selected_trait_ids {
            let Some(spec) = trait_by_id(trait_id) else {
                return false;
            };
            if !next_traits.contains(&spec.id) {
                next_traits.push(spec.id);
            }
        }

        self.player_traits = next_traits;
        true
    }

    pub(crate) fn calc_traits_cost(&self, selected_trait_ids: &[&str]) -> Option<u32> {
        let mut resolved = Vec::new();
        for trait_id in selected_trait_ids {
            let spec = trait_by_id(trait_id)?;
            resolved.push(spec.id);
        }
        Some(calc_selected_traits_cost(&resolved))
    }

    fn roll_enemy_traits(&mut self, node_type: NodeType) -> Vec<TraitId> {
        let Some(game_data) = self.game_data else {
            return Vec::new();
        };
        if game_data.enemy_trait_pool.is_empty() {
            return Vec::new();
        }

        let max_pick: usize = match node_type {
            NodeType::Boss => 3,
            _ => 2,
        };
        let min_pick: usize = 1;
        let span = max_pick.saturating_sub(min_pick) + 1;
        let count =
            (min_pick + self.run.rng.range_usize(span)).min(game_data.enemy_trait_pool.len());

        let mut bag = game_data.enemy_trait_pool.clone();
        let mut out = Vec::new();
        for _ in 0..count {
            if bag.is_empty() {
                break;
            }
            let idx = self.run.rng.range_usize(bag.len());
            out.push(bag.remove(idx));
        }
        out
    }

    pub(crate) fn sample_trait_choices(
        &self,
        rng: &mut SimpleRng,
        owned_traits: &[TraitId],
        n: usize,
    ) -> Vec<TraitId> {
        sample_trait_choices(rng, owned_traits, n)
    }

    pub(crate) fn current_node_type(&self) -> Option<NodeType> {
        if self.node_index == 0 {
            return None;
        }
        self.planned_nodes
            .get((self.node_index - 1) as usize)
            .copied()
    }

    fn enemy_battle_config(
        &self,
        node_type: NodeType,
    ) -> (f32, i32, i32, i32, i32, f32, f32, f32, &'static str) {
        match node_type {
            NodeType::Boss => self
                .game_data
                .and_then(|d| d.enemies.get("overstack_core"))
                .map(|e| {
                    (
                        e.max_hp,
                        e.atk,
                        e.matk,
                        e.def,
                        e.mdef,
                        e.crit_rate,
                        e.crit_mult,
                        e.speed,
                        e.name,
                    )
                })
                .unwrap_or((220.0, 14, 14, 8, 8, 15.0, 1.5, 32.0, "Overstack Core")),
            _ => self
                .game_data
                .and_then(|d| d.enemies.get("rogue_drone"))
                .map(|e| {
                    (
                        e.max_hp,
                        e.atk,
                        e.matk,
                        e.def,
                        e.mdef,
                        e.crit_rate,
                        e.crit_mult,
                        e.speed,
                        e.name,
                    )
                })
                .unwrap_or((84.0, 11, 11, 5, 5, 10.0, 1.5, 28.0, "Rogue Drone")),
        }
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
        let (
            enemy_hp,
            enemy_atk,
            enemy_matk,
            enemy_def,
            enemy_mdef,
            enemy_crit_rate,
            enemy_crit_mult,
            enemy_speed,
            enemy_name,
        ) = self.enemy_battle_config(node_type);

        let battle_state = create_battle(
            self.run.player_hp,
            self.run.player_max_hp,
            self.run.player_atk,
            self.run.player_matk,
            self.run.player_def,
            self.run.player_mdef,
            self.run.player_crit_rate,
            self.run.player_crit_mult,
            self.run.player_speed,
            1,
            enemy_hp,
            enemy_atk,
            enemy_matk,
            enemy_def,
            enemy_mdef,
            enemy_crit_rate,
            enemy_crit_mult,
            enemy_speed,
        );

        self.current_battle = Some(ActiveBattle::new(battle_state));
        self.enemy_traits = self.roll_enemy_traits(node_type);

        push_event(
            events,
            Event::BattleStart {
                battle_index: self.battle_index,
                enemy_name,
            },
        );

        let context = TriggerContext {
            trigger_type: TriggerType::OnBattleStart,
            owner: TraitOwner::Player,
            src_idx: None,
            dst_idx: None,
            applied_status: None,
        };
        self.process_trait_triggers(context, 0, events);
    }
}

#[cfg(test)]
mod tests {
    use super::ActiveRun;
    use crate::rng::SimpleRng;

    #[test]
    fn apply_traits_to_run_updates_player_traits() {
        let mut run = ActiveRun::new(7, 1);
        assert!(run.apply_traits_to_run(&["cinder_scholar", "overcharge"]));
        assert_eq!(run.player_traits, vec!["cinder_scholar", "overcharge"]);
    }

    #[test]
    fn calc_traits_cost_sums_selected_traits() {
        let run = ActiveRun::new(7, 1);
        let total = run
            .calc_traits_cost(&["cinder_scholar", "overcharge"])
            .expect("traits should exist");
        assert!(total > 0);
    }

    #[test]
    fn sample_trait_choices_excludes_owned_traits() {
        let run = ActiveRun::new(7, 1);
        let mut rng = SimpleRng::new(77);
        let sampled = run.sample_trait_choices(&mut rng, &["cinder_scholar"], 3);
        assert!(!sampled.contains(&"cinder_scholar"));
    }

    #[test]
    fn enemy_traits_are_assigned_on_battle_start() {
        let mut run = ActiveRun::new(1234, 1);
        let result = run.step_once(0.01, None);
        assert!(!result.events.is_empty());
        assert!(
            !run.enemy_traits.is_empty(),
            "expected enemy traits to be assigned at battle start"
        );
    }
}
