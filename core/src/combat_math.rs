use crate::data::specs::DamageKind;
use crate::model::Unit;

pub const DEFENSE_K: f32 = 100.0;
pub const CRIT_C: f32 = 100.0;
pub const BREAK_DEF_DOWN_PER_STACK: i32 = 10;
pub const BREAK_MAX_STACKS: u32 = 5;

#[derive(Clone, Copy, Debug)]
pub struct DamageBreakdown {
    pub amount: f32,
    pub raw: f32,
    pub defense_used: i32,
    pub mitigation: f32,
    pub crit: bool,
}

pub fn defense_for_kind(unit: &Unit, kind: DamageKind) -> i32 {
    match kind {
        DamageKind::Physical => unit.def,
        DamageKind::Magical => unit.mdef,
    }
}

pub fn attack_for_kind(unit: &Unit, kind: DamageKind) -> f32 {
    match kind {
        DamageKind::Physical => unit.atk as f32,
        DamageKind::Magical => unit.matk as f32,
    }
}

pub fn crit_chance(crit_rate: f32) -> f32 {
    let r = crit_rate.max(0.0);
    r / (r + CRIT_C)
}

pub fn mitigation_from_defense(defense_stat: i32) -> (i32, f32) {
    let min_def = -(DEFENSE_K as i32 - 1);
    let used = defense_stat.max(min_def);
    let mitigation = DEFENSE_K / (DEFENSE_K + used as f32);
    (used, mitigation)
}

pub fn effective_physical_defense(base_def: i32, break_stacks: u32) -> i32 {
    let capped_stacks = break_stacks.min(BREAK_MAX_STACKS) as i32;
    let lowered = base_def - (capped_stacks * BREAK_DEF_DOWN_PER_STACK);
    let min_def = -(DEFENSE_K as i32 - 1);
    lowered.max(min_def)
}

pub fn compute_damage(
    attacker: &Unit,
    defender: &Unit,
    kind: DamageKind,
    multiplier: f32,
    flat: f32,
    crit_roll_success: bool,
) -> DamageBreakdown {
    let attack_stat = attack_for_kind(attacker, kind);
    let raw = attack_stat * multiplier + flat;
    let (defense_used, mitigation) = mitigation_from_defense(defense_for_kind(defender, kind));
    let mut amount = (raw * mitigation).max(1.0);

    let crit = crit_roll_success;
    if crit {
        amount *= attacker.crit_mult.max(1.0);
    }

    DamageBreakdown {
        amount,
        raw,
        defense_used,
        mitigation,
        crit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Team, Unit};

    fn unit_with_def(def: i32) -> Unit {
        Unit {
            id: 1,
            team: Team::Enemy,
            hp: 100.0,
            max_hp: 100.0,
            atk: 10,
            matk: 10,
            def,
            mdef: def,
            crit_rate: 0.0,
            crit_mult: 1.5,
            speed: 10.0,
            action_gauge: 0.0,
        }
    }

    fn attacker() -> Unit {
        Unit {
            id: 0,
            team: Team::Player,
            hp: 100.0,
            max_hp: 100.0,
            atk: 20,
            matk: 20,
            def: 0,
            mdef: 0,
            crit_rate: 0.0,
            crit_mult: 1.5,
            speed: 10.0,
            action_gauge: 0.0,
        }
    }

    #[test]
    fn negative_defense_increases_damage() {
        let atk = attacker();
        let d0 = unit_with_def(0);
        let dneg = unit_with_def(-50);

        let normal = compute_damage(&atk, &d0, DamageKind::Physical, 1.0, 0.0, false);
        let vuln = compute_damage(&atk, &dneg, DamageKind::Physical, 1.0, 0.0, false);

        assert!(vuln.amount > normal.amount);
    }

    #[test]
    fn defense_floor_prevents_div_by_zero() {
        let (_used, m) = mitigation_from_defense(-9999);
        assert!(m.is_finite());
        assert!(m > 0.0);
    }

    #[test]
    fn break_lowers_physical_defense_with_stack_cap() {
        assert_eq!(effective_physical_defense(20, 0), 20);
        assert_eq!(effective_physical_defense(20, 3), -10);
        assert_eq!(effective_physical_defense(20, 5), -30);
        assert_eq!(effective_physical_defense(20, 9), -30);
    }

    #[test]
    fn break_stacks_increase_physical_damage() {
        let atk = attacker();
        let d0 = unit_with_def(20);
        let mut d3 = unit_with_def(20);
        let mut d5 = unit_with_def(20);
        d3.def = effective_physical_defense(d3.def, 3);
        d5.def = effective_physical_defense(d5.def, 5);

        let hit0 = compute_damage(&atk, &d0, DamageKind::Physical, 1.0, 0.0, false);
        let hit3 = compute_damage(&atk, &d3, DamageKind::Physical, 1.0, 0.0, false);
        let hit5 = compute_damage(&atk, &d5, DamageKind::Physical, 1.0, 0.0, false);

        assert!(hit3.mitigation > hit0.mitigation);
        assert!(hit5.mitigation > hit3.mitigation);
        assert!(hit3.amount > hit0.amount);
        assert!(hit5.amount > hit3.amount);
    }
}
