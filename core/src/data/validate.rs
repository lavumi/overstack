use std::collections::HashSet;

use super::defs::{ConditionDef, EffectDef, EmbeddedDefs};
use super::errors::ErrorReport;

const EFFECT_TYPES: &[&str] = &[
    "DealDamage",
    "ApplyStatus",
    "ConditionalDamageAmp",
    "ConditionalApplyStatus",
    "SelfBuff",
    "AddProcBonus",
    "AddResBonus",
    "ModifyStatusPower",
    "AddStatusStacks",
    "RemoveStatus",
    "DealPureDamage",
];
const DAMAGE_KINDS: &[&str] = &["Physical", "Magical"];

const CONDITION_TYPES: &[&str] = &[
    "Always",
    "SrcIsPlayer",
    "DstIsEnemy",
    "OwnerIsPlayer",
    "OwnerIsEnemy",
    "SrcIsOwner",
    "DstIsOwner",
    "AppliedStatusIs",
    "RandomRollBelow",
    "TargetHPBelow",
    "TargetHasStatus",
    "TargetStatusCountAtLeast",
    "All",
];

const TRIGGER_TYPES: &[&str] = &[
    "OnBattleStart",
    "OnTurnStart",
    "OnActionUsed",
    "OnDamageDealt",
    "OnStatusApplied",
    "OnStatusTick",
    "OnBattleEnd",
    "BattleStart",
    "TurnStart",
    "ActionUsed",
    "DamageDealt",
    "StatusApplied",
    "StatusTick",
    "BattleEnd",
];

const STATUS_TYPES: &[&str] = &[
    "Burn", "Freeze", "Shock", "Bleed", "Stun", "Break", "Might", "Haste",
];
const EFFECT_TARGETS: &[&str] = &["Owner", "Opponent", "Src", "Dst", "Player", "Enemy"];

pub fn validate_defs(defs: &EmbeddedDefs) -> Result<(), ErrorReport> {
    let mut report = ErrorReport::default();

    validate_duplicate_ids(
        defs.skills.skills.iter().map(|s| s.id.as_str()),
        "skills",
        &mut report,
    );
    validate_duplicate_ids(
        defs.traits.traits.iter().map(|t| t.id.as_str()),
        "traits",
        &mut report,
    );
    validate_duplicate_ids(
        defs.enemies.enemies.iter().map(|e| e.id.as_str()),
        "enemies",
        &mut report,
    );

    let skill_ids: HashSet<&str> = defs.skills.skills.iter().map(|s| s.id.as_str()).collect();

    for (i, enemy) in defs.enemies.enemies.iter().enumerate() {
        if enemy.max_hp <= 0.0 {
            report.push(format!("enemies.enemies[{i}].max_hp"), "must be > 0");
        }
        if enemy.speed <= 0.0 {
            report.push(format!("enemies.enemies[{i}].spd"), "must be > 0");
        }
        if let Some(v) = enemy.crit_rate {
            if v < 0.0 {
                report.push(format!("enemies.enemies[{i}].crit_rate"), "must be >= 0");
            }
        }
        if let Some(v) = enemy.crit_mult {
            if v < 1.0 {
                report.push(format!("enemies.enemies[{i}].crit_mult"), "must be >= 1");
            }
        }

        for (j, skill_id) in enemy.skills.iter().enumerate() {
            if !skill_ids.contains(skill_id.as_str()) {
                report.push(
                    format!("enemies.enemies[{i}].skills[{j}]"),
                    format!("unknown skill id '{skill_id}'"),
                );
            }
        }
    }

    for (i, skill) in defs.skills.skills.iter().enumerate() {
        if skill.id.trim().is_empty() {
            report.push(format!("skills.skills[{i}].id"), "must not be empty");
        }
        if skill.name.trim().is_empty() {
            report.push(format!("skills.skills[{i}].name"), "must not be empty");
        }
        match skill.cost {
            Some(cost) => {
                if skill.id != "basic_attack" && cost < 1 {
                    report.push(format!("skills.skills[{i}].cost"), "must be >= 1 for non-basic skills");
                }
            }
            None => {
                report.push(format!("skills.skills[{i}].cost"), "required");
            }
        }

        for (j, effect) in skill.effects.iter().enumerate() {
            validate_effect(
                effect,
                &format!("skills.skills[{i}].effects[{j}]"),
                &mut report,
            );
        }
    }

    for (i, trait_def) in defs.traits.traits.iter().enumerate() {
        if trait_def.id.trim().is_empty() {
            report.push(format!("traits.traits[{i}].id"), "must not be empty");
        }
        match trait_def.cost {
            Some(0) => {
                report.push(format!("traits.traits[{i}].cost"), "must be >= 1");
            }
            Some(_) => {}
            None => {
                report.push(format!("traits.traits[{i}].cost"), "required");
            }
        }
        if let Some(pool) = &trait_def.pool {
            for (j, p) in pool.iter().enumerate() {
                if p != "enemy" && p != "player" {
                    report.push(
                        format!("traits.traits[{i}].pool[{j}]"),
                        "must be one of: enemy, player",
                    );
                }
            }
        }
        for (j, trigger) in trait_def.triggers.iter().enumerate() {
            if !TRIGGER_TYPES.contains(&trigger.on.as_str()) {
                report.push(
                    format!("traits.traits[{i}].triggers[{j}].on"),
                    format!("unknown trigger '{}'", trigger.on),
                );
            }
            validate_condition(
                &trigger.condition,
                &format!("traits.traits[{i}].triggers[{j}].condition"),
                &mut report,
            );
            for (k, effect) in trigger.effects.iter().enumerate() {
                validate_effect(
                    effect,
                    &format!("traits.traits[{i}].triggers[{j}].effects[{k}]"),
                    &mut report,
                );
            }
        }
    }

    if defs.skills.player_loadout.len() < 4 {
        report.push("skills.player_loadout", "must contain at least 4 entries");
    }

    for (i, id) in defs.skills.player_loadout.iter().enumerate() {
        if !skill_ids.contains(id.as_str()) {
            report.push(
                format!("skills.player_loadout[{i}]"),
                format!("unknown skill id '{id}'"),
            );
        }
    }

    for (i, id) in defs.skills.selectable_skills.iter().enumerate() {
        if !skill_ids.contains(id.as_str()) {
            report.push(
                format!("skills.selectable_skills[{i}]"),
                format!("unknown skill id '{id}'"),
            );
        }
    }

    let trait_ids: HashSet<&str> = defs.traits.traits.iter().map(|t| t.id.as_str()).collect();
    for (i, id) in defs.traits.selectable_traits.iter().enumerate() {
        if !trait_ids.contains(id.as_str()) {
            report.push(
                format!("traits.selectable_traits[{i}]"),
                format!("unknown trait id '{id}'"),
            );
        }
    }

    if report.is_empty() {
        Ok(())
    } else {
        Err(report)
    }
}

fn validate_duplicate_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    namespace: &str,
    report: &mut ErrorReport,
) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id.to_string()) {
            report.push(format!("{namespace}.id"), format!("duplicate id '{id}'"));
        }
    }
}

fn validate_effect(effect: &EffectDef, path: &str, report: &mut ErrorReport) {
    if !EFFECT_TYPES.contains(&effect.effect_type.as_str()) {
        report.push(
            format!("{path}.type"),
            format!("unknown effect '{}'", effect.effect_type),
        );
        return;
    }

    match effect.effect_type.as_str() {
        "DealDamage" => {
            if let Some(kind) = effect.damage_kind.as_deref() {
                if !DAMAGE_KINDS.contains(&kind) {
                    report.push(
                        format!("{path}.damage_kind"),
                        format!("unknown damage_kind '{kind}'"),
                    );
                }
            }
            if effect.multiplier.is_none() {
                report.push(format!("{path}.multiplier"), "required");
            }
            if effect.flat.is_none() {
                report.push(format!("{path}.flat"), "required");
            }
            if let Some(v) = effect.multiplier {
                if !(0.0..=10.0).contains(&v) {
                    report.push(format!("{path}.multiplier"), "must be in range 0..=10");
                }
            }
        }
        "ApplyStatus" => {
            validate_required_status(effect.status.as_deref(), &format!("{path}.status"), report);
            validate_required_prob(effect.chance, &format!("{path}.chance"), report);
            validate_required_non_negative(effect.duration, &format!("{path}.duration"), report);
            if effect.stacks.is_none() {
                report.push(format!("{path}.stacks"), "required");
            }
            if effect.power.is_none() {
                report.push(format!("{path}.power"), "required");
            }
        }
        "ConditionalDamageAmp" => {
            if effect.condition.is_none() {
                report.push(format!("{path}.condition"), "required");
            }
            if effect.multiplier.is_none() {
                report.push(format!("{path}.multiplier"), "required");
            }
        }
        "ConditionalApplyStatus" => {
            if effect.condition.is_none() {
                report.push(format!("{path}.condition"), "required");
            }
            validate_required_status(effect.status.as_deref(), &format!("{path}.status"), report);
            validate_required_prob(effect.chance, &format!("{path}.chance"), report);
            validate_required_non_negative(effect.duration, &format!("{path}.duration"), report);
            if effect.stacks.is_none() {
                report.push(format!("{path}.stacks"), "required");
            }
            if effect.power.is_none() {
                report.push(format!("{path}.power"), "required");
            }
        }
        "SelfBuff" => {
            if effect.stat.is_none() {
                report.push(format!("{path}.stat"), "required");
            }
            if effect.amount.is_none() {
                report.push(format!("{path}.amount"), "required");
            }
            validate_required_non_negative(effect.duration, &format!("{path}.duration"), report);
        }
        "ModifyStatusPower" => {
            validate_required_status(effect.status.as_deref(), &format!("{path}.status"), report);
            if effect.multiplier.is_none() {
                report.push(format!("{path}.multiplier"), "required");
            }
        }
        "AddStatusStacks" => {
            validate_required_status(effect.status.as_deref(), &format!("{path}.status"), report);
            if effect.target.is_none() {
                report.push(format!("{path}.target"), "required");
            } else if let Some(target) = effect.target.as_deref() {
                if !EFFECT_TARGETS.contains(&target) {
                    report.push(
                        format!("{path}.target"),
                        format!("unknown target '{target}'"),
                    );
                }
            }
            if effect.stacks.is_none() {
                report.push(format!("{path}.stacks"), "required");
            }
        }
        "RemoveStatus" => {
            validate_required_status(effect.status.as_deref(), &format!("{path}.status"), report);
            if effect.target.is_none() {
                report.push(format!("{path}.target"), "required");
            } else if let Some(target) = effect.target.as_deref() {
                if !EFFECT_TARGETS.contains(&target) {
                    report.push(
                        format!("{path}.target"),
                        format!("unknown target '{target}'"),
                    );
                }
            }
        }
        "DealPureDamage" => {
            if effect.target.is_none() {
                report.push(format!("{path}.target"), "required");
            } else if let Some(target) = effect.target.as_deref() {
                if !EFFECT_TARGETS.contains(&target) {
                    report.push(
                        format!("{path}.target"),
                        format!("unknown target '{target}'"),
                    );
                }
            }
            if effect.amount.is_none() {
                report.push(format!("{path}.amount"), "required");
            }
        }
        "AddProcBonus" | "AddResBonus" => {
            if effect.amount.is_none() {
                report.push(format!("{path}.amount"), "required");
            }
        }
        _ => {}
    }

    if let Some(cond) = &effect.condition {
        validate_condition(cond, &format!("{path}.condition"), report);
    }
}

fn validate_condition(cond: &ConditionDef, path: &str, report: &mut ErrorReport) {
    if !CONDITION_TYPES.contains(&cond.condition_type.as_str()) {
        report.push(
            format!("{path}.type"),
            format!("unknown condition '{}'", cond.condition_type),
        );
        return;
    }

    match cond.condition_type.as_str() {
        "AppliedStatusIs" | "TargetHasStatus" => {
            validate_required_status(cond.status.as_deref(), &format!("{path}.status"), report);
        }
        "RandomRollBelow" => {
            validate_required_prob(cond.p, &format!("{path}.p"), report);
        }
        "TargetHPBelow" => {
            validate_required_prob(cond.ratio, &format!("{path}.ratio"), report);
        }
        "TargetStatusCountAtLeast" => {
            if cond.n.is_none() {
                report.push(format!("{path}.n"), "required");
            }
        }
        "All" => {
            if let Some(items) = &cond.all {
                for (i, item) in items.iter().enumerate() {
                    validate_condition(item, &format!("{path}.all[{i}]"), report);
                }
            } else {
                report.push(format!("{path}.all"), "required");
            }
        }
        _ => {}
    }
}

fn validate_required_status(status: Option<&str>, path: &str, report: &mut ErrorReport) {
    let Some(status) = status else {
        report.push(path, "required");
        return;
    };
    if !STATUS_TYPES.contains(&status) {
        report.push(path, format!("unknown status '{status}'"));
    }
}

fn validate_required_prob(v: Option<f32>, path: &str, report: &mut ErrorReport) {
    let Some(v) = v else {
        report.push(path, "required");
        return;
    };
    if !(0.0..=1.0).contains(&v) {
        report.push(path, "must be in range 0..=1");
    }
}

fn validate_required_non_negative(v: Option<f32>, path: &str, report: &mut ErrorReport) {
    let Some(v) = v else {
        report.push(path, "required");
        return;
    };
    if v < 0.0 {
        report.push(path, "must be >= 0");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::defs::{
        parse_json_file, EmbeddedDefs, EnemiesFileDef, SkillsFileDef, TraitsFileDef,
    };

    #[test]
    fn invalid_status_typo_reports_error() {
        let mut parse_report = ErrorReport::default();
        let skills = parse_json_file::<SkillsFileDef>(
            "skills",
            r#"{
              "skills": [
                {
                  "id":"s1",
                  "name":"X",
                  "description":"x",
                  "cost":1,
                  "effects":[{"type":"ApplyStatus","status":"Bunr","chance":0.3,"duration":1.0,"stacks":1,"power":1.0}]
                }
              ],
              "player_loadout":["s1","s1","s1","s1"]
            }"#,
            &mut parse_report,
        );
        assert!(parse_report.is_empty());

        let defs = EmbeddedDefs {
            skills,
            traits: TraitsFileDef::default(),
            enemies: EnemiesFileDef::default(),
        };

        let result = validate_defs(&defs);
        assert!(result.is_err());
        let report = result.err().unwrap_or_default();
        assert!(report
            .errors
            .iter()
            .any(|e| e.path.contains("status") && e.message.contains("unknown status")));
    }

    #[test]
    fn missing_trait_cost_reports_path() {
        let mut parse_report = ErrorReport::default();
        let traits = parse_json_file::<TraitsFileDef>(
            "traits",
            r#"{
              "traits": [
                {
                  "id":"t1",
                  "name":"Trait One",
                  "description":"x",
                  "pool":["player"],
                  "triggers":[]
                }
              ]
            }"#,
            &mut parse_report,
        );
        assert!(parse_report.is_empty());

        let defs = EmbeddedDefs {
            skills: SkillsFileDef::default(),
            traits,
            enemies: EnemiesFileDef::default(),
        };

        let report = validate_defs(&defs).expect_err("expected missing cost to fail validation");
        assert!(report
            .errors
            .iter()
            .any(|e| e.path == "traits.traits[0].cost" && e.message == "required"));
    }

    #[test]
    fn remove_status_requires_target_and_status() {
        let mut parse_report = ErrorReport::default();
        let skills = parse_json_file::<SkillsFileDef>(
            "skills",
            r#"{
              "skills": [
                {
                  "id":"s1",
                  "name":"X",
                  "description":"x",
                  "cost":1,
                  "effects":[{"type":"RemoveStatus"}]
                }
              ],
              "player_loadout":["s1","s1","s1","s1"]
            }"#,
            &mut parse_report,
        );
        assert!(parse_report.is_empty());

        let defs = EmbeddedDefs {
            skills,
            traits: TraitsFileDef::default(),
            enemies: EnemiesFileDef::default(),
        };

        let report = validate_defs(&defs).expect_err("expected RemoveStatus validation failure");
        assert!(report
            .errors
            .iter()
            .any(|e| e.path == "skills.skills[0].effects[0].target" && e.message == "required"));
        assert!(report
            .errors
            .iter()
            .any(|e| e.path == "skills.skills[0].effects[0].status" && e.message == "required"));
    }
}
