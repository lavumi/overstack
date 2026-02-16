pub use crate::data::specs::{
    Condition, EffectSpec, EffectTarget, SkillSpec, StatType, StatusType,
};

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
            base_damage_multiplier: 1.0,
            flat_bonus_damage: Some(0.0),
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
