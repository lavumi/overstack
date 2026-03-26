use std::collections::HashMap;

use super::defs::{ConditionDef, EffectDef, EmbeddedDefs};
use super::errors::ErrorReport;
use super::registry::{EnemyRegistry, GameData, SkillRegistry, TraitRegistry};
use super::specs::{
    Condition, DamageKind, EffectSpec, EffectTarget, EnemySpec, SkillId, SkillSpec, StatType,
    StatusType, TraitId, TraitSpec, TriggerRule, TriggerType,
};

pub fn compile_defs(defs: &EmbeddedDefs) -> Result<GameData, ErrorReport> {
    let mut report = ErrorReport::default();

    let skills = compile_skills(defs, &mut report);
    let traits = compile_traits(defs, &mut report);
    let enemies = compile_enemies(defs, &mut report);
    let player_loadout = compile_player_loadout(defs, &skills, &mut report);
    let selectable_traits = compile_selectable_traits(defs, &traits, &mut report);
    let enemy_trait_pool = compile_enemy_trait_pool(defs, &traits, &mut report);

    if report.is_empty() {
        Ok(GameData {
            skills,
            traits,
            enemies,
            player_loadout,
            selectable_traits,
            enemy_trait_pool,
        })
    } else {
        Err(report)
    }
}

fn compile_skills(defs: &EmbeddedDefs, report: &mut ErrorReport) -> SkillRegistry {
    let mut out: SkillRegistry = HashMap::new();

    for (i, def) in defs.skills.skills.iter().enumerate() {
        let effects = def
            .effects
            .iter()
            .enumerate()
            .filter_map(|(j, effect)| {
                compile_effect(effect, report, format!("skills.skills[{i}].effects[{j}]"))
            })
            .collect::<Vec<_>>();

        let id = leak_str(def.id.clone());
        let spec = SkillSpec {
            id,
            name: leak_str(def.name.clone()),
            description: leak_str(def.description.clone()),
            damage_kind: DamageKind::Physical,
            effects: leak_effects(effects),
            tags: leak_strings(def.tags.clone().unwrap_or_default()),
        };

        out.insert(id, spec);
    }

    out
}

fn compile_traits(defs: &EmbeddedDefs, report: &mut ErrorReport) -> TraitRegistry {
    let mut out: TraitRegistry = HashMap::new();

    for (i, def) in defs.traits.traits.iter().enumerate() {
        let mut rules = Vec::new();

        for (j, rule) in def.triggers.iter().enumerate() {
            let path = format!("traits.traits[{i}].triggers[{j}]");
            let trigger = match parse_trigger_type(&rule.on) {
                Some(v) => v,
                None => {
                    report.push(
                        format!("{path}.on"),
                        format!("unknown trigger '{}'", rule.on),
                    );
                    continue;
                }
            };

            let condition = compile_condition(&rule.condition, report, format!("{path}.condition"));
            let effects = rule
                .effects
                .iter()
                .enumerate()
                .filter_map(|(k, effect)| {
                    compile_effect(effect, report, format!("{path}.effects[{k}]"))
                })
                .collect::<Vec<_>>();

            rules.push(TriggerRule {
                trigger,
                condition,
                effects: leak_effects(effects),
            });
        }

        let id = leak_str(def.id.clone());
        let spec = TraitSpec {
            id,
            name: leak_str(def.name.clone()),
            description: leak_str(def.description.clone()),
            cost: def.cost.unwrap_or(1),
            pool: leak_strings(def.pool.clone().unwrap_or_default()),
            triggers: leak_trigger_rules(rules),
        };

        out.insert(id, spec);
    }

    out
}

fn compile_enemies(defs: &EmbeddedDefs, _report: &mut ErrorReport) -> EnemyRegistry {
    let mut out: EnemyRegistry = HashMap::new();

    for def in &defs.enemies.enemies {
        let id = leak_str(def.id.clone());
        let spec = EnemySpec {
            id,
            name: leak_str(def.name.clone()),
            max_hp: def.max_hp,
            atk: def.atk,
            matk: def.matk.unwrap_or(def.atk),
            def: def.def.unwrap_or(0),
            mdef: def.mdef.unwrap_or(0),
            crit_rate: def.crit_rate.unwrap_or(15.0).max(0.0),
            crit_mult: def.crit_mult.unwrap_or(1.5).max(1.0),
            speed: def.speed,
            skills: leak_skill_ids(def.skills.clone()),
        };
        out.insert(id, spec);
    }

    out
}

fn compile_player_loadout(
    defs: &EmbeddedDefs,
    skills: &SkillRegistry,
    report: &mut ErrorReport,
) -> [SkillId; 4] {
    let mut out = ["basic_attack"; 4];
    for (i, id) in defs.skills.player_loadout.iter().take(4).enumerate() {
        if !skills.contains_key(id.as_str()) {
            report.push(
                format!("skills.player_loadout[{i}]"),
                format!("unknown skill '{}'", id),
            );
            continue;
        }
        out[i] = leak_str(id.clone());
    }
    out
}

fn compile_selectable_traits(
    defs: &EmbeddedDefs,
    traits: &TraitRegistry,
    report: &mut ErrorReport,
) -> Vec<TraitId> {
    let mut out = Vec::new();

    for (i, id) in defs.traits.selectable_traits.iter().enumerate() {
        if !traits.contains_key(id.as_str()) {
            report.push(
                format!("traits.selectable_traits[{i}]"),
                format!("unknown trait '{}'", id),
            );
            continue;
        }
        out.push(leak_str(id.clone()));
    }

    out
}

fn compile_enemy_trait_pool(
    defs: &EmbeddedDefs,
    traits: &TraitRegistry,
    report: &mut ErrorReport,
) -> Vec<TraitId> {
    let mut out = Vec::new();
    for (i, trait_def) in defs.traits.traits.iter().enumerate() {
        let has_enemy_pool = trait_def
            .pool
            .as_ref()
            .map(|items| items.iter().any(|p| p == "enemy"))
            .unwrap_or(false);
        if !has_enemy_pool {
            continue;
        }
        if !traits.contains_key(trait_def.id.as_str()) {
            report.push(
                format!("traits.traits[{i}].id"),
                format!("unknown trait '{}'", trait_def.id),
            );
            continue;
        }
        out.push(leak_str(trait_def.id.clone()));
    }
    out
}

fn compile_effect(def: &EffectDef, report: &mut ErrorReport, path: String) -> Option<EffectSpec> {
    match def.effect_type.as_str() {
        "DealDamage" => Some(EffectSpec::DealDamage {
            damage_kind: parse_damage_kind(
                def.damage_kind.as_deref().unwrap_or("Physical"),
                report,
                format!("{path}.damage_kind"),
            )
            .unwrap_or(DamageKind::Physical),
            multiplier: def.multiplier.unwrap_or(1.0),
            flat: def.flat.unwrap_or(0.0),
        }),
        "ApplyStatus" => Some(EffectSpec::ApplyStatus {
            status_type: parse_status(
                def.status.as_deref().unwrap_or(""),
                report,
                format!("{path}.status"),
            )?,
            base_chance: def.chance.unwrap_or(0.0),
            duration: def.duration.unwrap_or(0.0),
            stacks: def.stacks.unwrap_or(0),
            power: def.power.unwrap_or(0.0),
        }),
        "ConditionalDamageAmp" => Some(EffectSpec::ConditionalDamageAmp {
            condition: compile_condition_opt(
                def.condition.as_ref(),
                report,
                format!("{path}.condition"),
            )?,
            amp: def.multiplier.unwrap_or(1.0),
        }),
        "ConditionalApplyStatus" => Some(EffectSpec::ConditionalApplyStatus {
            condition: compile_condition_opt(
                def.condition.as_ref(),
                report,
                format!("{path}.condition"),
            )?,
            status_type: parse_status(
                def.status.as_deref().unwrap_or(""),
                report,
                format!("{path}.status"),
            )?,
            base_chance: def.chance.unwrap_or(0.0),
            duration: def.duration.unwrap_or(0.0),
            stacks: def.stacks.unwrap_or(0),
            power: def.power.unwrap_or(0.0),
        }),
        "SelfBuff" => {
            let stat = match def.stat.as_deref().unwrap_or("") {
                "Attack" => StatType::Attack,
                "Speed" => StatType::Speed,
                unknown => {
                    report.push(
                        format!("{path}.stat"),
                        format!("unknown stat '{}'", unknown),
                    );
                    return None;
                }
            };
            Some(EffectSpec::SelfBuff {
                stat,
                amount: def.amount.unwrap_or(0.0),
                duration: def.duration.unwrap_or(0.0),
            })
        }
        "AddProcBonus" => Some(EffectSpec::AddProcBonus {
            amount: def.amount.unwrap_or(0.0),
        }),
        "AddResBonus" => Some(EffectSpec::AddResBonus {
            amount: def.amount.unwrap_or(0.0),
        }),
        "ModifyStatusPower" => Some(EffectSpec::ModifyStatusPower {
            status_type: parse_status(
                def.status.as_deref().unwrap_or(""),
                report,
                format!("{path}.status"),
            )?,
            mul: def.multiplier.unwrap_or(1.0),
        }),
        "AddStatusStacks" => Some(EffectSpec::AddStatusStacks {
            target: parse_target(
                def.target.as_deref().unwrap_or("Dst"),
                report,
                format!("{path}.target"),
            )?,
            status_type: parse_status(
                def.status.as_deref().unwrap_or(""),
                report,
                format!("{path}.status"),
            )?,
            stacks: def.stacks.unwrap_or(0),
        }),
        "RemoveStatus" => Some(EffectSpec::RemoveStatus {
            target: parse_target(
                def.target.as_deref().unwrap_or("Dst"),
                report,
                format!("{path}.target"),
            )?,
            status_type: parse_status(
                def.status.as_deref().unwrap_or(""),
                report,
                format!("{path}.status"),
            )?,
        }),
        "DealPureDamage" => Some(EffectSpec::DealPureDamage {
            target: parse_target(
                def.target.as_deref().unwrap_or("Dst"),
                report,
                format!("{path}.target"),
            )?,
            amount: def.amount.unwrap_or(0.0),
        }),
        unknown => {
            report.push(
                format!("{path}.type"),
                format!("unknown effect '{}'", unknown),
            );
            None
        }
    }
}

fn compile_condition_opt(
    def: Option<&ConditionDef>,
    report: &mut ErrorReport,
    path: String,
) -> Option<Condition> {
    def.map(|v| compile_condition(v, report, path))
}

fn compile_condition(def: &ConditionDef, report: &mut ErrorReport, path: String) -> Condition {
    match def.condition_type.as_str() {
        "Always" => Condition::Always,
        "SrcIsPlayer" => Condition::SrcIsPlayer,
        "DstIsEnemy" => Condition::DstIsEnemy,
        "OwnerIsPlayer" => Condition::OwnerIsPlayer,
        "OwnerIsEnemy" => Condition::OwnerIsEnemy,
        "SrcIsOwner" => Condition::SrcIsOwner,
        "DstIsOwner" => Condition::DstIsOwner,
        "AppliedStatusIs" => parse_status(
            def.status.as_deref().unwrap_or(""),
            report,
            format!("{path}.status"),
        )
        .map(Condition::AppliedStatusIs)
        .unwrap_or(Condition::Always),
        "RandomRollBelow" => Condition::RandomRollBelow(def.p.unwrap_or(0.0)),
        "TargetHPBelow" => Condition::TargetHPBelow(def.ratio.unwrap_or(0.0)),
        "TargetHasStatus" => parse_status(
            def.status.as_deref().unwrap_or(""),
            report,
            format!("{path}.status"),
        )
        .map(Condition::TargetHasStatus)
        .unwrap_or(Condition::Always),
        "TargetStatusCountAtLeast" => Condition::TargetStatusCountAtLeast(def.n.unwrap_or(0)),
        "All" => {
            let values = def
                .all
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            compile_condition(item, report, format!("{path}.all[{i}]"))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Condition::All(leak_conditions(values))
        }
        unknown => {
            report.push(
                format!("{path}.type"),
                format!("unknown condition '{}'", unknown),
            );
            Condition::Always
        }
    }
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

fn parse_status(raw: &str, report: &mut ErrorReport, path: String) -> Option<StatusType> {
    match raw {
        "Burn" => Some(StatusType::Burn),
        "Freeze" => Some(StatusType::Freeze),
        "Shock" => Some(StatusType::Shock),
        "Break" => Some(StatusType::Break),
        "Bleed" => Some(StatusType::Bleed),
        "Stun" => Some(StatusType::Stun),
        "Might" => Some(StatusType::Might),
        "Haste" => Some(StatusType::Haste),
        _ => {
            report.push(path, format!("unknown status '{}'", raw));
            None
        }
    }
}

fn parse_target(raw: &str, report: &mut ErrorReport, path: String) -> Option<EffectTarget> {
    match raw {
        "Owner" => Some(EffectTarget::Owner),
        "Opponent" => Some(EffectTarget::Opponent),
        "Src" => Some(EffectTarget::Src),
        "Dst" => Some(EffectTarget::Dst),
        "Player" => Some(EffectTarget::Player),
        "Enemy" => Some(EffectTarget::Enemy),
        _ => {
            report.push(path, format!("unknown target '{}'", raw));
            None
        }
    }
}

fn parse_damage_kind(raw: &str, report: &mut ErrorReport, path: String) -> Option<DamageKind> {
    match raw {
        "Physical" => Some(DamageKind::Physical),
        "Magical" => Some(DamageKind::Magical),
        _ => {
            report.push(path, format!("unknown damage_kind '{}'", raw));
            None
        }
    }
}

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn leak_strings(values: Vec<String>) -> &'static [&'static str] {
    let leaked = values.into_iter().map(leak_str).collect::<Vec<_>>();
    Box::leak(leaked.into_boxed_slice())
}

fn leak_skill_ids(values: Vec<String>) -> &'static [SkillId] {
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
