use std::collections::HashSet;

use crate::rng::SimpleRng;

pub use crate::data::specs::{TraitId, TraitSpec, TriggerType};

pub const TRAIT_WEIGHT_BASE: f32 = 100.0;
pub const TRAIT_WEIGHT_P: f32 = 1.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraitTriggerPhase {
    PreActionCalc,
    PostDamage,
    PostStatus,
    StatusTick,
    BattleBoundary,
}

impl TriggerType {
    pub fn phase(self) -> TraitTriggerPhase {
        match self {
            TriggerType::OnActionUsed => TraitTriggerPhase::PreActionCalc,
            TriggerType::OnDamageDealt => TraitTriggerPhase::PostDamage,
            TriggerType::OnStatusApplied => TraitTriggerPhase::PostStatus,
            TriggerType::OnStatusTick => TraitTriggerPhase::StatusTick,
            TriggerType::OnBattleStart | TriggerType::OnBattleEnd | TriggerType::OnTurnStart => {
                TraitTriggerPhase::BattleBoundary
            }
        }
    }
}

pub fn trait_weight(cost: u32) -> f32 {
    let safe_cost = cost.max(1) as f32;
    TRAIT_WEIGHT_BASE / safe_cost.powf(TRAIT_WEIGHT_P)
}

pub fn trait_by_id(id: &str) -> Option<&'static TraitSpec> {
    crate::data::load_embedded_game_data()
        .ok()
        .and_then(|d| d.traits.get(id))
}

pub fn active_trait_names(ids: &[TraitId]) -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return ids
            .iter()
            .filter_map(|id| data.traits.get(*id))
            .map(|t| t.name.to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_trait_names() -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_traits
            .iter()
            .filter_map(|id| data.traits.get(*id))
            .map(|t| t.name.to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_trait_ids() -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_traits
            .iter()
            .map(|id| (*id).to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_trait_costs() -> Vec<u32> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_traits
            .iter()
            .filter_map(|id| data.traits.get(*id))
            .map(|t| t.cost)
            .collect();
    }
    Vec::new()
}

pub fn calc_traits_cost(selected_trait_ids: &[TraitId]) -> u32 {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return selected_trait_ids
            .iter()
            .filter_map(|id| data.traits.get(*id))
            .map(|trait_spec| trait_spec.cost)
            .sum();
    }
    0
}

pub fn sample_trait_choices(
    rng: &mut SimpleRng,
    owned_traits: &[TraitId],
    n: usize,
) -> Vec<TraitId> {
    let Ok(data) = crate::data::load_embedded_game_data() else {
        return Vec::new();
    };

    let owned: HashSet<TraitId> = owned_traits.iter().copied().collect();
    let mut candidates = data
        .traits
        .values()
        .filter(|trait_spec| trait_spec.pool.contains(&"player") && !owned.contains(&trait_spec.id))
        .collect::<Vec<_>>();
    let mut picks = Vec::new();

    while picks.len() < n && !candidates.is_empty() {
        let total_weight: f32 = candidates
            .iter()
            .map(|trait_spec| trait_weight(trait_spec.cost))
            .sum();
        if total_weight <= 0.0 {
            break;
        }

        let roll = (rng.next_u32() as f32 / u32::MAX as f32) * total_weight;
        let mut acc = 0.0;
        let mut selected_idx = candidates.len() - 1;
        for (idx, trait_spec) in candidates.iter().enumerate() {
            acc += trait_weight(trait_spec.cost);
            if roll <= acc {
                selected_idx = idx;
                break;
            }
        }

        picks.push(candidates.remove(selected_idx).id);
    }

    picks
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        calc_traits_cost, sample_trait_choices, trait_weight, TraitTriggerPhase, TriggerType,
    };
    use crate::rng::SimpleRng;

    #[test]
    fn lower_cost_trait_has_higher_weight() {
        assert!(trait_weight(3) > trait_weight(8));
    }

    #[test]
    fn sampled_traits_are_unique_and_exclude_owned() {
        let mut rng = SimpleRng::new(42);
        let owned = ["cinder_scholar"];
        let sampled = sample_trait_choices(&mut rng, &owned, 3);
        let unique: HashSet<_> = sampled.iter().copied().collect();

        assert_eq!(sampled.len(), unique.len());
        assert!(!sampled.contains(&"cinder_scholar"));
    }

    #[test]
    fn cost_sum_uses_trait_specs() {
        let total = calc_traits_cost(&["cinder_scholar", "overcharge"]);
        assert!(total > 0);
    }

    #[test]
    fn trigger_types_map_to_expected_phases() {
        assert_eq!(
            TriggerType::OnActionUsed.phase(),
            TraitTriggerPhase::PreActionCalc
        );
        assert_eq!(
            TriggerType::OnDamageDealt.phase(),
            TraitTriggerPhase::PostDamage
        );
        assert_eq!(
            TriggerType::OnStatusApplied.phase(),
            TraitTriggerPhase::PostStatus
        );
    }
}
