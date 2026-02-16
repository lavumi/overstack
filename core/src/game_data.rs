use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::skill::{Condition, EffectSpec, EffectTarget, SkillId, SkillSpec, StatType, StatusType};
use crate::trait_spec::{TraitId, TraitSpec, TriggerRule, TriggerType};

pub type EnemyId = &'static str;
pub type SkillRegistry = HashMap<SkillId, &'static SkillSpec>;
pub type TraitRegistry = HashMap<TraitId, &'static TraitSpec>;
pub type EnemyRegistry = HashMap<EnemyId, &'static EnemySpec>;

#[derive(Clone, Copy, Debug)]
pub struct EnemySpec {
    pub id: EnemyId,
    pub name: &'static str,
    pub max_hp: f32,
    pub atk: i32,
    pub speed: f32,
}

pub struct GameData {
    pub skills: SkillRegistry,
    pub traits: TraitRegistry,
    pub enemies: EnemyRegistry,
    pub player_loadout: [SkillId; 4],
    pub selectable_traits: Vec<TraitId>,
}

static GAME_DATA: OnceLock<Result<GameData, Vec<String>>> = OnceLock::new();

pub fn load_embedded_game_data() -> Result<&'static GameData, Vec<String>> {
    let result = GAME_DATA.get_or_init(build_game_data);
    match result {
        Ok(data) => Ok(data),
        Err(errors) => Err(errors.clone()),
    }
}

pub fn enemy_by_id(id: &str) -> Option<&'static EnemySpec> {
    load_embedded_game_data()
        .ok()
        .and_then(|d| d.enemies.get(id).copied())
}

fn build_game_data() -> Result<GameData, Vec<String>> {
    let mut errors = Vec::new();

    let skills_def: SkillsFileDef = parse_json(
        "skills.json",
        include_str!("../data/skills.json"),
        &mut errors,
    );
    let traits_def: TraitsFileDef = parse_json(
        "traits.json",
        include_str!("../data/traits.json"),
        &mut errors,
    );
    let enemies_def: EnemiesFileDef = parse_json(
        "enemies.json",
        include_str!("../data/enemies.json"),
        &mut errors,
    );

    if !errors.is_empty() {
        return Err(errors);
    }

    let skills = compile_skills(&skills_def, &mut errors);
    let traits = compile_traits(&traits_def, &mut errors);
    let enemies = compile_enemies(&enemies_def, &mut errors);

    let player_loadout = compile_loadout(&skills_def.player_loadout, &skills, &mut errors);
    let selectable_traits = compile_selectable_traits(&traits_def.selectable_traits, &traits, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(GameData {
        skills,
        traits,
        enemies,
        player_loadout,
        selectable_traits,
    })
}

fn parse_json<T: for<'de> Deserialize<'de> + Default>(
    name: &str,
    raw: &str,
    errors: &mut Vec<String>,
) -> T {
    match serde_json::from_str::<T>(raw) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("failed_to_parse_{name}: {e}"));
            T::default()
        }
    }
}

fn compile_skills(def: &SkillsFileDef, errors: &mut Vec<String>) -> SkillRegistry {
    let mut ids = HashSet::new();
    let mut registry = HashMap::new();

    for skill in &def.skills {
        if !ids.insert(skill.id.clone()) {
            errors.push(format!("duplicate_skill_id:{}", skill.id));
            continue;
        }

        if skill.base_damage_multiplier < 0.0 {
            errors.push(format!("invalid_base_damage_multiplier:{}", skill.id));
        }

        let effects = skill
            .effects
            .iter()
            .filter_map(|e| compile_effect_def(e, errors, &format!("skill:{}", skill.id)))
            .collect::<Vec<_>>();

        let leaked_id = leak_str(skill.id.clone());
        let leaked_name = leak_str(skill.name.clone());
        let leaked_description = leak_str(skill.description.clone());
        let leaked_tags = leak_str_vec(skill.tags.clone().unwrap_or_default());
        let leaked_effects = leak_effects(effects);

        let spec = Box::new(SkillSpec {
            id: leaked_id,
            name: leaked_name,
            description: leaked_description,
            base_damage_multiplier: skill.base_damage_multiplier,
            flat_bonus_damage: skill.flat_bonus_damage,
            effects: leaked_effects,
            tags: leaked_tags,
        });

        let spec_ref: &'static SkillSpec = Box::leak(spec);
        registry.insert(leaked_id, spec_ref);
    }

    registry
}

fn compile_traits(def: &TraitsFileDef, errors: &mut Vec<String>) -> TraitRegistry {
    let mut ids = HashSet::new();
    let mut registry = HashMap::new();

    for t in &def.traits {
        if !ids.insert(t.id.clone()) {
            errors.push(format!("duplicate_trait_id:{}", t.id));
            continue;
        }

        let mut rules = Vec::new();
        for (idx, rule) in t.triggers.iter().enumerate() {
            let Some(trigger) = parse_trigger_type(&rule.on) else {
                errors.push(format!("unknown_trigger:{} at trait:{} rule:{}", rule.on, t.id, idx));
                continue;
            };

            let condition = compile_condition_def(&rule.condition, errors, &format!("trait:{} rule:{}", t.id, idx));
            let effects = rule
                .effects
                .iter()
                .filter_map(|e| compile_effect_def(e, errors, &format!("trait:{} rule:{}", t.id, idx)))
                .collect::<Vec<_>>();

            let leaked_effects = leak_effects(effects);
            rules.push(TriggerRule {
                trigger,
                condition,
                effects: leaked_effects,
            });
        }

        let leaked_id = leak_str(t.id.clone());
        let leaked_name = leak_str(t.name.clone());
        let leaked_desc = leak_str(t.description.clone());
        let leaked_rules = leak_trigger_rules(rules);

        let spec = Box::new(TraitSpec {
            id: leaked_id,
            name: leaked_name,
            description: leaked_desc,
            triggers: leaked_rules,
        });

        let spec_ref: &'static TraitSpec = Box::leak(spec);
        registry.insert(leaked_id, spec_ref);
    }

    registry
}

fn compile_enemies(def: &EnemiesFileDef, errors: &mut Vec<String>) -> EnemyRegistry {
    let mut ids = HashSet::new();
    let mut registry = HashMap::new();

    for enemy in &def.enemies {
        if !ids.insert(enemy.id.clone()) {
            errors.push(format!("duplicate_enemy_id:{}", enemy.id));
            continue;
        }

        if enemy.max_hp <= 0.0 {
            errors.push(format!("invalid_enemy_max_hp:{}", enemy.id));
        }
        if enemy.speed <= 0.0 {
            errors.push(format!("invalid_enemy_speed:{}", enemy.id));
        }

        let leaked_id = leak_str(enemy.id.clone());
        let leaked_name = leak_str(enemy.name.clone());

        let spec = Box::new(EnemySpec {
            id: leaked_id,
            name: leaked_name,
            max_hp: enemy.max_hp,
            atk: enemy.atk,
            speed: enemy.speed,
        });

        let spec_ref: &'static EnemySpec = Box::leak(spec);
        registry.insert(leaked_id, spec_ref);
    }

    registry
}

fn compile_loadout(
    loadout: &[String],
    skills: &SkillRegistry,
    errors: &mut Vec<String>,
) -> [SkillId; 4] {
    let mut out = ["basic_attack"; 4];

    if loadout.len() < 4 {
        errors.push("player_loadout_requires_4_skill_ids".to_string());
        return out;
    }

    for (i, id) in loadout.iter().take(4).enumerate() {
        if !skills.contains_key(id.as_str()) {
            errors.push(format!("player_loadout_unknown_skill:{}", id));
            continue;
        }
        out[i] = leak_str(id.clone());
    }

    out
}

fn compile_selectable_traits(
    selectable: &[String],
    traits: &TraitRegistry,
    errors: &mut Vec<String>,
) -> Vec<TraitId> {
    let mut out = Vec::new();

    for id in selectable {
        if !traits.contains_key(id.as_str()) {
            errors.push(format!("selectable_trait_unknown:{}", id));
            continue;
        }
        out.push(leak_str(id.clone()));
    }

    out
}

fn compile_effect_def(def: &EffectDef, errors: &mut Vec<String>, where_: &str) -> Option<EffectSpec> {
    match def.effect_type.as_str() {
        "DealDamage" => Some(EffectSpec::DealDamage {
            multiplier: def.multiplier.unwrap_or(1.0),
            flat: def.flat.unwrap_or(0.0),
        }),
        "ApplyStatus" => {
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)?;
            let chance = def.chance.unwrap_or(0.0);
            let duration = def.duration.unwrap_or(0.0);
            let stacks = def.stacks.unwrap_or(1);
            let power = def.power.unwrap_or(1.0);

            validate_probability(chance, errors, where_);
            validate_duration(duration, errors, where_);

            Some(EffectSpec::ApplyStatus {
                status_type: status,
                base_chance: chance,
                duration,
                stacks,
                power,
            })
        }
        "ConditionalDamageAmp" => {
            let condition = compile_condition_def_opt(def.condition.as_ref(), errors, where_)?;
            let amp = def.multiplier.unwrap_or(1.0);
            Some(EffectSpec::ConditionalDamageAmp { condition, amp })
        }
        "ConditionalApplyStatus" => {
            let condition = compile_condition_def_opt(def.condition.as_ref(), errors, where_)?;
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)?;
            let chance = def.chance.unwrap_or(0.0);
            let duration = def.duration.unwrap_or(0.0);
            let stacks = def.stacks.unwrap_or(1);
            let power = def.power.unwrap_or(1.0);

            validate_probability(chance, errors, where_);
            validate_duration(duration, errors, where_);

            Some(EffectSpec::ConditionalApplyStatus {
                condition,
                status_type: status,
                base_chance: chance,
                duration,
                stacks,
                power,
            })
        }
        "SelfBuff" => {
            let stat = match def.stat.as_deref().unwrap_or("") {
                "Attack" => StatType::Attack,
                "Speed" => StatType::Speed,
                unknown => {
                    errors.push(format!("unknown_stat:{} at {}", unknown, where_));
                    return None;
                }
            };
            let amount = def.amount.unwrap_or(0.0);
            let duration = def.duration.unwrap_or(0.0);
            validate_duration(duration, errors, where_);
            Some(EffectSpec::SelfBuff {
                stat,
                amount,
                duration,
            })
        }
        "AddProcBonus" => Some(EffectSpec::AddProcBonus {
            amount: def.amount.unwrap_or(0.0),
        }),
        "AddResBonus" => Some(EffectSpec::AddResBonus {
            amount: def.amount.unwrap_or(0.0),
        }),
        "ModifyStatusPower" => {
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)?;
            Some(EffectSpec::ModifyStatusPower {
                status_type: status,
                mul: def.multiplier.unwrap_or(1.0),
            })
        }
        "AddStatusStacks" => {
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)?;
            let target = parse_target(def.target.as_deref().unwrap_or("Dst"), errors, where_)?;
            Some(EffectSpec::AddStatusStacks {
                target,
                status_type: status,
                stacks: def.stacks.unwrap_or(1),
            })
        }
        "DealPureDamage" => {
            let target = parse_target(def.target.as_deref().unwrap_or("Dst"), errors, where_)?;
            Some(EffectSpec::DealPureDamage {
                target,
                amount: def.amount.unwrap_or(0.0),
            })
        }
        unknown => {
            errors.push(format!("unknown_effect_type:{} at {}", unknown, where_));
            None
        }
    }
}

fn compile_condition_def_opt(
    def: Option<&ConditionDef>,
    errors: &mut Vec<String>,
    where_: &str,
) -> Option<Condition> {
    def.map(|v| compile_condition_def(v, errors, where_))
}

fn compile_condition_def(def: &ConditionDef, errors: &mut Vec<String>, where_: &str) -> Condition {
    match def.condition_type.as_str() {
        "Always" => Condition::Always,
        "SrcIsPlayer" => Condition::SrcIsPlayer,
        "DstIsEnemy" => Condition::DstIsEnemy,
        "AppliedStatusIs" => {
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)
                .unwrap_or(StatusType::Burn);
            Condition::AppliedStatusIs(status)
        }
        "RandomRollBelow" => {
            let p = def.p.unwrap_or(0.0);
            validate_probability(p, errors, where_);
            Condition::RandomRollBelow(p)
        }
        "TargetHPBelow" => {
            let ratio = def.ratio.unwrap_or(0.0);
            validate_probability(ratio, errors, where_);
            Condition::TargetHPBelow(ratio)
        }
        "TargetHasStatus" => {
            let status = parse_status(def.status.as_deref().unwrap_or(""), errors, where_)
                .unwrap_or(StatusType::Burn);
            Condition::TargetHasStatus(status)
        }
        "TargetStatusCountAtLeast" => Condition::TargetStatusCountAtLeast(def.n.unwrap_or(0)),
        "All" => {
            let children = def
                .all
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .map(|item| compile_condition_def(item, errors, where_))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Condition::All(leak_conditions(children))
        }
        unknown => {
            errors.push(format!("unknown_condition:{} at {}", unknown, where_));
            Condition::Always
        }
    }
}

fn parse_status(raw: &str, errors: &mut Vec<String>, where_: &str) -> Option<StatusType> {
    let status = match raw {
        "Burn" => StatusType::Burn,
        "Freeze" => StatusType::Freeze,
        "Shock" => StatusType::Shock,
        "Break" => StatusType::Break,
        "Bleed" => StatusType::Bleed,
        "Stun" => StatusType::Stun,
        "Might" => StatusType::Might,
        "Haste" => StatusType::Haste,
        _ => {
            errors.push(format!("unknown_status:{} at {}", raw, where_));
            return None;
        }
    };
    Some(status)
}

fn parse_trigger_type(raw: &str) -> Option<TriggerType> {
    match raw {
        "OnBattleStart" | "BattleStart" => Some(TriggerType::OnBattleStart),
        "OnTurnStart" | "TurnStart" => Some(TriggerType::OnTurnStart),
        "OnActionUsed" | "ActionUsed" => Some(TriggerType::OnActionUsed),
        "OnDamageDealt" | "DamageDealt" => Some(TriggerType::OnDamageDealt),
        "OnStatusApplied" | "StatusApplied" => Some(TriggerType::OnStatusApplied),
        "OnStatusTick" | "StatusTick" => Some(TriggerType::OnStatusTick),
        "OnBattleEnd" | "BattleEnd" => Some(TriggerType::OnBattleEnd),
        _ => None,
    }
}

fn parse_target(raw: &str, errors: &mut Vec<String>, where_: &str) -> Option<EffectTarget> {
    let target = match raw {
        "Src" => EffectTarget::Src,
        "Dst" => EffectTarget::Dst,
        "Player" => EffectTarget::Player,
        "Enemy" => EffectTarget::Enemy,
        _ => {
            errors.push(format!("unknown_target:{} at {}", raw, where_));
            return None;
        }
    };
    Some(target)
}

fn validate_probability(v: f32, errors: &mut Vec<String>, where_: &str) {
    if !(0.0..=1.0).contains(&v) {
        errors.push(format!("probability_out_of_range:{} at {}", v, where_));
    }
}

fn validate_duration(v: f32, errors: &mut Vec<String>, where_: &str) {
    if v < 0.0 {
        errors.push(format!("duration_negative:{} at {}", v, where_));
    }
}

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn leak_str_vec(values: Vec<String>) -> &'static [&'static str] {
    let leaked = values.into_iter().map(leak_str).collect::<Vec<_>>();
    Box::leak(leaked.into_boxed_slice())
}

fn leak_effects(values: Vec<EffectSpec>) -> &'static [EffectSpec] {
    Box::leak(values.into_boxed_slice())
}

fn leak_conditions(values: Vec<Condition>) -> &'static [Condition] {
    Box::leak(values.into_boxed_slice())
}

fn leak_trigger_rules(values: Vec<TriggerRule>) -> &'static [TriggerRule] {
    Box::leak(values.into_boxed_slice())
}

#[derive(Default, Deserialize)]
struct SkillsFileDef {
    #[serde(default)]
    skills: Vec<SkillDef>,
    #[serde(default)]
    player_loadout: Vec<String>,
}

#[derive(Default, Deserialize)]
struct SkillDef {
    id: String,
    name: String,
    description: String,
    base_damage_multiplier: f32,
    flat_bonus_damage: Option<f32>,
    #[serde(default)]
    effects: Vec<EffectDef>,
    tags: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
struct TraitsFileDef {
    #[serde(default)]
    traits: Vec<TraitDef>,
    #[serde(default)]
    selectable_traits: Vec<String>,
}

#[derive(Default, Deserialize)]
struct TraitDef {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    triggers: Vec<TriggerRuleDef>,
}

#[derive(Default, Deserialize)]
struct TriggerRuleDef {
    on: String,
    condition: ConditionDef,
    #[serde(default)]
    effects: Vec<EffectDef>,
}

#[derive(Default, Deserialize)]
struct EnemiesFileDef {
    #[serde(default)]
    enemies: Vec<EnemyDef>,
}

#[derive(Default, Deserialize)]
struct EnemyDef {
    id: String,
    name: String,
    max_hp: f32,
    atk: i32,
    #[serde(alias = "spd")]
    speed: f32,
}

#[derive(Default, Deserialize)]
struct EffectDef {
    #[serde(rename = "type")]
    effect_type: String,
    multiplier: Option<f32>,
    flat: Option<f32>,
    status: Option<String>,
    chance: Option<f32>,
    duration: Option<f32>,
    stacks: Option<u32>,
    power: Option<f32>,
    condition: Option<ConditionDef>,
    stat: Option<String>,
    amount: Option<f32>,
    target: Option<String>,
}

#[derive(Default, Deserialize)]
struct ConditionDef {
    #[serde(rename = "type")]
    condition_type: String,
    status: Option<String>,
    n: Option<u32>,
    p: Option<f32>,
    ratio: Option<f32>,
    all: Option<Vec<ConditionDef>>,
}
