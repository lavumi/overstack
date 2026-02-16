use crate::skill::{Condition, EffectSpec};

pub type TraitId = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerType {
    OnBattleStart,
    OnTurnStart,
    OnActionUsed,
    OnDamageDealt,
    OnStatusApplied,
    OnStatusTick,
    OnBattleEnd,
}

impl TriggerType {
    pub fn as_str(self) -> &'static str {
        match self {
            TriggerType::OnBattleStart => "OnBattleStart",
            TriggerType::OnTurnStart => "OnTurnStart",
            TriggerType::OnActionUsed => "OnActionUsed",
            TriggerType::OnDamageDealt => "OnDamageDealt",
            TriggerType::OnStatusApplied => "OnStatusApplied",
            TriggerType::OnStatusTick => "OnStatusTick",
            TriggerType::OnBattleEnd => "OnBattleEnd",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TriggerRule {
    pub trigger: TriggerType,
    pub condition: Condition,
    pub effects: &'static [EffectSpec],
}

#[derive(Clone, Copy, Debug)]
pub struct TraitSpec {
    pub id: TraitId,
    pub name: &'static str,
    pub description: &'static str,
    pub triggers: &'static [TriggerRule],
}

pub fn trait_by_id(id: &str) -> Option<&'static TraitSpec> {
    crate::game_data::load_embedded_game_data()
        .ok()
        .and_then(|d| d.traits.get(id).copied())
}

pub fn active_trait_names(ids: &[TraitId]) -> Vec<String> {
    if let Ok(data) = crate::game_data::load_embedded_game_data() {
        return ids
            .iter()
            .filter_map(|id| data.traits.get(*id))
            .map(|t| t.name.to_string())
            .collect();
    }
    Vec::new()
}

pub fn selectable_trait_names() -> Vec<String> {
    if let Ok(data) = crate::game_data::load_embedded_game_data() {
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
    if let Ok(data) = crate::game_data::load_embedded_game_data() {
        return data
            .selectable_traits
            .iter()
            .map(|id| (*id).to_string())
            .collect();
    }
    Vec::new()
}
