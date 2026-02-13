pub type SkillId = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusType {
    Burn,
    Freeze,
    Shock,
    Break,
    Bleed,
    Stun,
    Might,
    Haste,
}

impl StatusType {
    pub fn as_str(self) -> &'static str {
        match self {
            StatusType::Burn => "Burn",
            StatusType::Freeze => "Freeze",
            StatusType::Shock => "Shock",
            StatusType::Break => "Break",
            StatusType::Bleed => "Bleed",
            StatusType::Stun => "Stun",
            StatusType::Might => "Might",
            StatusType::Haste => "Haste",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Condition {
    Always,
    SrcIsPlayer,
    DstIsEnemy,
    AppliedStatusIs(StatusType),
    RandomRollBelow(f32),
    TargetHPBelow(f32),
    TargetHasStatus(StatusType),
    TargetStatusCountAtLeast(u32),
    All(&'static [Condition]),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum StatType {
    Attack,
    Speed,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum EffectTarget {
    Src,
    Dst,
    Player,
    Enemy,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum EffectSpec {
    DealDamage {
        multiplier: f32,
        flat: f32,
    },
    ApplyStatus {
        status_type: StatusType,
        base_chance: f32,
        duration: f32,
        stacks: u32,
        power: f32,
    },
    ConditionalDamageAmp {
        condition: Condition,
        amp: f32,
    },
    ConditionalApplyStatus {
        condition: Condition,
        status_type: StatusType,
        base_chance: f32,
        duration: f32,
        stacks: u32,
        power: f32,
    },
    SelfBuff {
        stat: StatType,
        amount: f32,
        duration: f32,
    },
    AddProcBonus {
        amount: f32,
    },
    AddResBonus {
        amount: f32,
    },
    ModifyStatusPower {
        status_type: StatusType,
        mul: f32,
    },
    AddStatusStacks {
        target: EffectTarget,
        status_type: StatusType,
        stacks: u32,
    },
    DealPureDamage {
        target: EffectTarget,
        amount: f32,
    },
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct SkillSpec {
    pub id: SkillId,
    pub name: &'static str,
    pub base_damage_multiplier: f32,
    pub flat_bonus_damage: Option<f32>,
    pub effects: &'static [EffectSpec],
    pub tags: &'static [&'static str],
}

const BASIC_ATTACK_EFFECTS: [EffectSpec; 1] = [EffectSpec::DealDamage {
    multiplier: 1.0,
    flat: 0.0,
}];

const EMBER_LASH_EFFECTS: [EffectSpec; 2] = [
    EffectSpec::DealDamage {
        multiplier: 1.0,
        flat: 0.0,
    },
    EffectSpec::ApplyStatus {
        status_type: StatusType::Burn,
        base_chance: 0.35,
        duration: 4.0,
        stacks: 1,
        power: 1.0,
    },
];

const FROST_BITE_EFFECTS: [EffectSpec; 2] = [
    EffectSpec::DealDamage {
        multiplier: 0.9,
        flat: 0.0,
    },
    EffectSpec::ApplyStatus {
        status_type: StatusType::Freeze,
        base_chance: 0.30,
        duration: 3.5,
        stacks: 1,
        power: 1.0,
    },
];

const ARC_JOLT_EFFECTS: [EffectSpec; 2] = [
    EffectSpec::DealDamage {
        multiplier: 0.8,
        flat: 0.0,
    },
    EffectSpec::ApplyStatus {
        status_type: StatusType::Shock,
        base_chance: 0.40,
        duration: 4.0,
        stacks: 1,
        power: 1.0,
    },
];

const RUIN_STRIKE_EFFECTS: [EffectSpec; 2] = [
    EffectSpec::DealDamage {
        multiplier: 1.1,
        flat: 0.0,
    },
    EffectSpec::ApplyStatus {
        status_type: StatusType::Break,
        base_chance: 0.35,
        duration: 6.0,
        stacks: 1,
        power: 1.0,
    },
];

pub const BASIC_ATTACK: SkillSpec = SkillSpec {
    id: "basic_attack",
    name: "Basic Attack",
    base_damage_multiplier: 1.0,
    flat_bonus_damage: None,
    effects: &BASIC_ATTACK_EFFECTS,
    tags: &["basic", "physical"],
};

pub const EMBER_LASH: SkillSpec = SkillSpec {
    id: "ember_lash",
    name: "Ember Lash",
    base_damage_multiplier: 1.0,
    flat_bonus_damage: None,
    effects: &EMBER_LASH_EFFECTS,
    tags: &["skill", "fire"],
};

pub const FROST_BITE: SkillSpec = SkillSpec {
    id: "frost_bite",
    name: "Frost Bite",
    base_damage_multiplier: 1.0,
    flat_bonus_damage: None,
    effects: &FROST_BITE_EFFECTS,
    tags: &["skill", "ice"],
};

pub const ARC_JOLT: SkillSpec = SkillSpec {
    id: "arc_jolt",
    name: "Arc Jolt",
    base_damage_multiplier: 1.0,
    flat_bonus_damage: None,
    effects: &ARC_JOLT_EFFECTS,
    tags: &["skill", "lightning"],
};

pub const RUIN_STRIKE: SkillSpec = SkillSpec {
    id: "ruin_strike",
    name: "Ruin Strike",
    base_damage_multiplier: 1.0,
    flat_bonus_damage: None,
    effects: &RUIN_STRIKE_EFFECTS,
    tags: &["skill", "debuff"],
};

pub const PLAYER_SLOT_SKILL_IDS: [SkillId; 4] = [
    EMBER_LASH.id,
    FROST_BITE.id,
    ARC_JOLT.id,
    RUIN_STRIKE.id,
];

pub fn skill_by_id(id: SkillId) -> Option<&'static SkillSpec> {
    match id {
        "basic_attack" => Some(&BASIC_ATTACK),
        "ember_lash" => Some(&EMBER_LASH),
        "frost_bite" => Some(&FROST_BITE),
        "arc_jolt" => Some(&ARC_JOLT),
        "ruin_strike" => Some(&RUIN_STRIKE),
        _ => None,
    }
}

pub fn player_skill_for_slot(slot: u32) -> &'static SkillSpec {
    let idx = (slot as usize).min(PLAYER_SLOT_SKILL_IDS.len() - 1);
    let id = PLAYER_SLOT_SKILL_IDS[idx];
    skill_by_id(id).unwrap_or(&BASIC_ATTACK)
}

pub fn player_skill_names() -> Vec<String> {
    PLAYER_SLOT_SKILL_IDS
        .iter()
        .filter_map(|id| skill_by_id(*id))
        .map(|spec| spec.name.to_string())
        .collect()
}
