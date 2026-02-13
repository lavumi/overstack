use crate::skill::{Condition, EffectSpec, EffectTarget, StatusType};

pub type TraitId = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct TraitSpec {
    pub id: TraitId,
    pub name: &'static str,
    pub description: &'static str,
    pub triggers: &'static [TriggerRule],
}

const CINDER_COND_ALL: [Condition; 3] = [
    Condition::SrcIsPlayer,
    Condition::DstIsEnemy,
    Condition::AppliedStatusIs(StatusType::Burn),
];
const CINDER_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::ModifyStatusPower {
    status_type: StatusType::Burn,
    mul: 1.25,
}];
const CINDER_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnStatusApplied,
    condition: Condition::All(&CINDER_COND_ALL),
    effects: &CINDER_RULE_EFFECTS,
}];

const FROZEN_COND_ALL: [Condition; 2] = [
    Condition::SrcIsPlayer,
    Condition::AppliedStatusIs(StatusType::Freeze),
];
const FROZEN_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::AddStatusStacks {
    target: EffectTarget::Dst,
    status_type: StatusType::Break,
    stacks: 1,
}];
const FROZEN_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnStatusApplied,
    condition: Condition::All(&FROZEN_COND_ALL),
    effects: &FROZEN_RULE_EFFECTS,
}];

const OVERCHARGE_COND_ALL: [Condition; 2] = [
    Condition::SrcIsPlayer,
    Condition::AppliedStatusIs(StatusType::Shock),
];
const OVERCHARGE_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::DealPureDamage {
    target: EffectTarget::Dst,
    amount: 3.0,
}];
const OVERCHARGE_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnStatusApplied,
    condition: Condition::All(&OVERCHARGE_COND_ALL),
    effects: &OVERCHARGE_RULE_EFFECTS,
}];

const HEMORRHAGE_COND_ALL: [Condition; 3] = [
    Condition::SrcIsPlayer,
    Condition::DstIsEnemy,
    Condition::TargetHasStatus(StatusType::Bleed),
];
const HEMORRHAGE_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::DealDamage {
    multiplier: 0.15,
    flat: 0.0,
}];
const HEMORRHAGE_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnDamageDealt,
    condition: Condition::All(&HEMORRHAGE_COND_ALL),
    effects: &HEMORRHAGE_RULE_EFFECTS,
}];

const RUTHLESS_COND_ALL: [Condition; 3] = [
    Condition::SrcIsPlayer,
    Condition::DstIsEnemy,
    Condition::TargetStatusCountAtLeast(2),
];
const RUTHLESS_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::DealDamage {
    multiplier: 0.20,
    flat: 0.0,
}];
const RUTHLESS_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnDamageDealt,
    condition: Condition::All(&RUTHLESS_COND_ALL),
    effects: &RUTHLESS_RULE_EFFECTS,
}];

const SHATTERPOINT_COND_ALL: [Condition; 2] = [
    Condition::SrcIsPlayer,
    Condition::AppliedStatusIs(StatusType::Break),
];
const SHATTERPOINT_RULE_EFFECTS: [EffectSpec; 1] = [EffectSpec::ConditionalApplyStatus {
    condition: Condition::TargetHasStatus(StatusType::Freeze),
    status_type: StatusType::Stun,
    base_chance: 0.50,
    duration: 1.5,
    stacks: 1,
    power: 1.0,
}];
const SHATTERPOINT_RULES: [TriggerRule; 1] = [TriggerRule {
    trigger: TriggerType::OnStatusApplied,
    condition: Condition::All(&SHATTERPOINT_COND_ALL),
    effects: &SHATTERPOINT_RULE_EFFECTS,
}];

pub const CINDER_SCHOLAR: TraitSpec = TraitSpec {
    id: "cinder_scholar",
    name: "Cinder Scholar",
    description: "Burn applied by player enhances Burn power.",
    triggers: &CINDER_RULES,
};

pub const FROZEN_MOMENTUM: TraitSpec = TraitSpec {
    id: "frozen_momentum",
    name: "Frozen Momentum",
    description: "Freeze application adds Break stacks.",
    triggers: &FROZEN_RULES,
};

pub const OVERCHARGE: TraitSpec = TraitSpec {
    id: "overcharge",
    name: "Overcharge",
    description: "Shock application deals pure bonus damage.",
    triggers: &OVERCHARGE_RULES,
};

pub const HEMORRHAGE: TraitSpec = TraitSpec {
    id: "hemorrhage",
    name: "Hemorrhage",
    description: "Damage against Bleed targets gains bonus hit.",
    triggers: &HEMORRHAGE_RULES,
};

pub const RUTHLESS: TraitSpec = TraitSpec {
    id: "ruthless",
    name: "Ruthless",
    description: "Targets with many statuses take extra damage.",
    triggers: &RUTHLESS_RULES,
};

pub const SHATTERPOINT: TraitSpec = TraitSpec {
    id: "shatterpoint",
    name: "Shatterpoint",
    description: "Break on Frozen targets can apply Stun.",
    triggers: &SHATTERPOINT_RULES,
};

#[allow(dead_code)]
pub const DEFAULT_ACTIVE_TRAITS: [TraitId; 6] = [
    CINDER_SCHOLAR.id,
    FROZEN_MOMENTUM.id,
    OVERCHARGE.id,
    HEMORRHAGE.id,
    RUTHLESS.id,
    SHATTERPOINT.id,
];

pub const SELECTABLE_TRAITS: [TraitId; 5] = [
    CINDER_SCHOLAR.id,
    FROZEN_MOMENTUM.id,
    OVERCHARGE.id,
    RUTHLESS.id,
    SHATTERPOINT.id,
];

pub fn trait_by_id(id: &str) -> Option<&'static TraitSpec> {
    match id {
        "cinder_scholar" => Some(&CINDER_SCHOLAR),
        "frozen_momentum" => Some(&FROZEN_MOMENTUM),
        "overcharge" => Some(&OVERCHARGE),
        "hemorrhage" => Some(&HEMORRHAGE),
        "ruthless" => Some(&RUTHLESS),
        "shatterpoint" => Some(&SHATTERPOINT),
        _ => None,
    }
}

pub fn active_trait_names(ids: &[TraitId]) -> Vec<String> {
    ids.iter()
        .filter_map(|id| trait_by_id(*id))
        .map(|t| t.name.to_string())
        .collect()
}

pub fn selectable_trait_names() -> Vec<String> {
    active_trait_names(&SELECTABLE_TRAITS)
}

pub fn selectable_trait_ids() -> Vec<String> {
    SELECTABLE_TRAITS.iter().map(|id| (*id).to_string()).collect()
}
