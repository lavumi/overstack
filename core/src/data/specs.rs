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

pub type SkillId = &'static str;
pub type TraitId = &'static str;
pub type EnemyId = &'static str;

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

#[derive(Clone, Copy, Debug)]
pub enum StatType {
    Attack,
    Speed,
}

#[derive(Clone, Copy, Debug)]
pub enum EffectTarget {
    Src,
    Dst,
    Player,
    Enemy,
}

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

#[derive(Clone, Copy, Debug)]
pub struct SkillSpec {
    pub id: SkillId,
    pub name: &'static str,
    pub description: &'static str,
    pub base_damage_multiplier: f32,
    pub flat_bonus_damage: Option<f32>,
    pub effects: &'static [EffectSpec],
    pub tags: &'static [&'static str],
}

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

#[derive(Clone, Debug)]
pub struct EnemySpec {
    pub id: EnemyId,
    pub name: &'static str,
    pub max_hp: f32,
    pub atk: i32,
    pub speed: f32,
    pub skills: &'static [SkillId],
}
