pub use crate::data::specs::{TraitId, TraitSpec, TriggerType};

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
