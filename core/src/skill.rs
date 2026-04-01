use std::collections::HashSet;

use crate::rng::SimpleRng;

pub use crate::data::specs::{
    Condition, DamageKind, EffectSpec, EffectTarget, SkillId, SkillSpec, StatType, StatusType,
};

pub const SKILL_WEIGHT_BASE: f32 = 100.0;
pub const SKILL_WEIGHT_P: f32 = 1.3;

pub fn skill_weight(cost: u32) -> f32 {
    let safe_cost = cost.max(1) as f32;
    SKILL_WEIGHT_BASE / safe_cost.powf(SKILL_WEIGHT_P)
}

pub fn skill_by_id(id: &str) -> Option<&'static SkillSpec> {
    crate::data::load_embedded_game_data()
        .ok()
        .and_then(|d| d.skills.get(id))
}

pub fn player_skill_for_slot(slot: u32) -> &'static SkillSpec {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        let idx = (slot as usize).min(data.player_loadout.len().saturating_sub(1));
        let id = data.player_loadout[idx];
        if let Some(spec) = data.skills.get(id) {
            return spec;
        }
    }

    skill_by_id("basic_attack").unwrap_or_else(|| {
        static EMPTY_EFFECTS: [EffectSpec; 0] = [];
        static EMPTY_TAGS: [&str; 0] = [];
        static FALLBACK: SkillSpec = SkillSpec {
            id: "basic_attack",
            name: "Basic Attack",
            description: "Fallback basic attack",
            cost: 0,
            damage_kind: DamageKind::Physical,
            effects: &EMPTY_EFFECTS,
            tags: &EMPTY_TAGS,
        };
        &FALLBACK
    })
}

pub fn player_skill_names() -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .player_loadout
            .iter()
            .filter_map(|id| data.skills.get(id))
            .map(|spec| spec.name.to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_skill_ids() -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_skills
            .iter()
            .map(|id| (*id).to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_skill_names() -> Vec<String> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_skills
            .iter()
            .filter_map(|id| data.skills.get(*id))
            .map(|skill| skill.name.to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_skill_costs() -> Vec<u32> {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return data
            .selectable_skills
            .iter()
            .filter_map(|id| data.skills.get(*id))
            .map(|skill| skill.cost)
            .collect();
    }
    Vec::new()
}

pub fn calc_skills_cost(selected_skill_ids: &[SkillId]) -> u32 {
    if let Ok(data) = crate::data::load_embedded_game_data() {
        return selected_skill_ids
            .iter()
            .filter_map(|id| data.skills.get(*id))
            .map(|skill| skill.cost)
            .sum();
    }
    0
}

pub fn sample_skill_choices(
    rng: &mut SimpleRng,
    owned_skills: &[SkillId],
    n: usize,
) -> Vec<SkillId> {
    let Ok(data) = crate::data::load_embedded_game_data() else {
        return Vec::new();
    };

    let owned: HashSet<SkillId> = owned_skills.iter().copied().collect();
    let mut candidates = data
        .selectable_skills
        .iter()
        .filter_map(|id| data.skills.get(*id))
        .filter(|skill| !owned.contains(&skill.id))
        .collect::<Vec<_>>();
    let mut picks = Vec::new();

    while picks.len() < n && !candidates.is_empty() {
        let total_weight: f32 = candidates.iter().map(|skill| skill_weight(skill.cost)).sum();
        if total_weight <= 0.0 {
            break;
        }

        let roll = (rng.next_u32() as f32 / u32::MAX as f32) * total_weight;
        let mut acc = 0.0;
        let mut selected_idx = candidates.len() - 1;
        for (idx, skill) in candidates.iter().enumerate() {
            acc += skill_weight(skill.cost);
            if roll <= acc {
                selected_idx = idx;
                break;
            }
        }

        picks.push(candidates.remove(selected_idx).id);
    }

    picks
}
