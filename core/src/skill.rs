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

pub fn skill_by_id(id: &str) -> Option<&'static SkillSpec> {
    crate::game_data::load_embedded_game_data()
        .ok()
        .and_then(|d| d.skills.get(id).copied())
}

pub fn player_skill_for_slot(slot: u32) -> &'static SkillSpec {
    if let Ok(data) = crate::game_data::load_embedded_game_data() {
        let idx = (slot as usize).min(data.player_loadout.len().saturating_sub(1));
        let id = data.player_loadout[idx];
        if let Some(spec) = data.skills.get(id) {
            return spec;
        }
    }

    skill_by_id("basic_attack").unwrap_or_else(|| {
        // This fallback is only reached when embedded data loading failed.
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
    if let Ok(data) = crate::game_data::load_embedded_game_data() {
        return data
            .player_loadout
            .iter()
            .filter_map(|id| data.skills.get(id))
            .map(|spec| spec.name.to_string())
            .collect();
    }
    Vec::new()
}
