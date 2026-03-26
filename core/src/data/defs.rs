use serde::Deserialize;

use super::errors::ErrorReport;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SkillsFileDef {
    #[serde(default)]
    pub skills: Vec<SkillDef>,
    #[serde(default)]
    pub player_loadout: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TraitsFileDef {
    #[serde(default)]
    pub traits: Vec<TraitDef>,
    #[serde(default)]
    pub selectable_traits: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EnemiesFileDef {
    #[serde(default)]
    pub enemies: Vec<EnemyDef>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_damage_multiplier: f32,
    pub flat_bonus_damage: Option<f32>,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: Option<u32>,
    pub pool: Option<Vec<String>>,
    #[serde(default)]
    pub triggers: Vec<TriggerRuleDef>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub max_hp: f32,
    pub atk: i32,
    pub matk: Option<i32>,
    pub def: Option<i32>,
    pub mdef: Option<i32>,
    pub crit_rate: Option<f32>,
    pub crit_mult: Option<f32>,
    #[serde(alias = "spd")]
    pub speed: f32,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TriggerRuleDef {
    pub on: String,
    pub condition: ConditionDef,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EffectDef {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub damage_kind: Option<String>,
    pub multiplier: Option<f32>,
    pub flat: Option<f32>,
    pub status: Option<String>,
    pub chance: Option<f32>,
    pub duration: Option<f32>,
    pub stacks: Option<u32>,
    pub power: Option<f32>,
    pub condition: Option<ConditionDef>,
    pub stat: Option<String>,
    pub amount: Option<f32>,
    pub target: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConditionDef {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: Option<String>,
    pub n: Option<u32>,
    pub p: Option<f32>,
    pub ratio: Option<f32>,
    pub all: Option<Vec<ConditionDef>>,
}

#[derive(Clone, Debug, Default)]
pub struct EmbeddedDefs {
    pub skills: SkillsFileDef,
    pub traits: TraitsFileDef,
    pub enemies: EnemiesFileDef,
}

pub fn parse_embedded_defs() -> Result<EmbeddedDefs, ErrorReport> {
    let mut report = ErrorReport::default();

    let skills = parse_json_file::<SkillsFileDef>(
        "skills",
        include_str!("../../data/skills.json"),
        &mut report,
    );
    let traits = parse_json_file::<TraitsFileDef>(
        "traits",
        include_str!("../../data/traits.json"),
        &mut report,
    );
    let enemies = parse_json_file::<EnemiesFileDef>(
        "enemies",
        include_str!("../../data/enemies.json"),
        &mut report,
    );

    if report.is_empty() {
        Ok(EmbeddedDefs {
            skills,
            traits,
            enemies,
        })
    } else {
        Err(report)
    }
}

pub fn parse_json_file<T: for<'de> Deserialize<'de> + Default>(
    root: &str,
    raw: &str,
    report: &mut ErrorReport,
) -> T {
    match serde_json::from_str::<T>(raw) {
        Ok(v) => v,
        Err(e) => {
            report.push(root, format!("json_parse_error: {e}"));
            T::default()
        }
    }
}
