# Overstack Data Schema

This document defines the JSON schema for skills, traits, and enemies.
All game content must be defined in JSON and compiled into runtime specs.

============================================================
1. General Rules
============================================================

- All IDs must be unique and stable.
- IDs must not change after release.
- Use snake_case for IDs.
- Strings for enums are case-sensitive.
- Probability values are 0.0 ~ 1.0.
- Duration values are in seconds (float).
- Stacks must be >= 0.

============================================================
2. skills.json
============================================================

Root Structure:

{
  "skills": [ SkillDef, SkillDef, ... ]
}

------------------------------------------------------------
SkillDef
------------------------------------------------------------

{
  "id": "sk_ember_lash",
  "name": "Ember Lash",
  "description": "Burns the enemy.",
  "effects": [ EffectDef, EffectDef ],
  "tags": ["fire", "dot"]
}

Fields:

id                     string   required
name                   string   required
description            string   required
effects                array    required
tags                   string[] optional

============================================================
3. EffectDef
============================================================

Example ApplyStatus:

{
  "type": "ApplyStatus",
  "status": "Burn",
  "chance": 0.35,
  "duration": 4.0,
  "stacks": 1,
  "power": 1.0
}

Supported Effect Types:

DealDamage
{
  "type": "DealDamage",
  "multiplier": 1.0,
  "flat": 0
}

ApplyStatus
{
  "type": "ApplyStatus",
  "status": "Burn",
  "chance": 0.35,
  "duration": 4.0,
  "stacks": 1,
  "power": 1.0
}

ConditionalDamageAmp
{
  "type": "ConditionalDamageAmp",
  "condition": ConditionDef,
  "multiplier": 0.2
}

RemoveStatus
{
  "type": "RemoveStatus",
  "target": "Dst",
  "status": "Burn"
}

============================================================
4. ConditionDef
============================================================

Example:

{
  "type": "TargetHasStatus",
  "status": "Burn"
}

Supported Condition Types:

Always
TargetHasStatus
TargetStatusCountAtLeast
RandomRollBelow
TargetHPBelow
SrcIsPlayer
DstIsEnemy

============================================================
5. traits.json
============================================================

Root Structure:

{
  "traits": [ TraitDef, TraitDef ]
}

------------------------------------------------------------
TraitDef
------------------------------------------------------------

{
  "id": "tr_cinder_scholar",
  "name": "Cinder Scholar",
  "description": "Burn effects are stronger.",
  "cost": 4,
  "triggers": [ TriggerRule ]
}

Fields:

id                     string   required
name                   string   required
description            string   required
cost                   uint     required, must be >= 1
triggers               array    required

------------------------------------------------------------
TriggerRule
------------------------------------------------------------

{
  "on": "StatusApplied",
  "condition": ConditionDef,
  "effects": [ EffectDef ]
}

Supported Trigger Types:

OnBattleStart
OnTurnStart
OnActionUsed
OnDamageDealt
OnStatusApplied
OnStatusTick
OnBattleEnd

============================================================
6. enemies.json
============================================================

Root Structure:

{
  "enemies": [ EnemyDef ]
}

------------------------------------------------------------
EnemyDef
------------------------------------------------------------

{
  "id": "en_wraith",
  "name": "Wraith",
  "max_hp": 120,
  "atk": 18,
  "def": 5,
  "spd": 1.2,
  "skills": ["sk_shadow_strike"]
}

============================================================
7. Validation Rules
============================================================

- All referenced IDs must exist.
- Unknown effect/condition/trigger types must fail validation.
- Duplicate IDs must fail validation.
- Trait cost must be present and >= 1.
- Probability must be 0.0 ~ 1.0.
- Duration must be >= 0.
- Stacks must be >= 0.

============================================================
8. Adding New Content
============================================================

1. Add JSON entry in correct file.
2. Ensure ID is unique.
3. Run cargo wasm build and wasm-bindgen output generation.
4. Verify no validation errors.
5. Test in UI.

============================================================
9. Future Extensions
============================================================

- Cooldown system
- Resource cost (mana)
- Multi-hit support
- Area effects
- Buff duration modifiers
- Meta progression modifiers
