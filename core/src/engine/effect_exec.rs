use crate::engine::runtime::{ActiveRun, TriggerContext};
use crate::skill::{EffectSpec, StatType, StatusType};

impl ActiveRun {
    pub(crate) fn execute_primitive_effect(
        &mut self,
        effect: EffectSpec,
        source_idx: usize,
        fallback_target_idx: usize,
        context: TriggerContext,
        depth: u8,
        events: &mut Vec<String>,
    ) -> Option<String> {
        match effect {
            EffectSpec::DealDamage {
                damage_kind,
                multiplier,
                flat,
            } => {
                self.apply_scaled_damage(
                    source_idx,
                    fallback_target_idx,
                    damage_kind,
                    multiplier,
                    flat,
                    depth,
                    events,
                );
                Some(format!("DealDamage x{multiplier:.2} +{flat}"))
            }
            EffectSpec::ApplyStatus {
                status_type,
                base_chance,
                duration,
                stacks,
                power,
            } => {
                self.apply_status(
                    source_idx,
                    fallback_target_idx,
                    status_type,
                    base_chance,
                    duration,
                    stacks,
                    power,
                    depth,
                    events,
                );
                Some(format!("ApplyStatus {}", status_type.as_str()))
            }
            EffectSpec::SelfBuff {
                stat,
                amount,
                duration,
            } => {
                let status_type = match stat {
                    StatType::Attack => StatusType::Might,
                    StatType::Speed => StatusType::Haste,
                };
                self.apply_status(
                    source_idx,
                    source_idx,
                    status_type,
                    1.0,
                    duration,
                    amount.max(1.0) as u32,
                    amount,
                    depth,
                    events,
                );
                Some(format!("SelfBuff {}", status_type.as_str()))
            }
            EffectSpec::AddProcBonus { amount } => {
                self.add_proc_bonus(source_idx, amount);
                Some(format!("AddProcBonus +{amount:.2}"))
            }
            EffectSpec::AddResBonus { amount } => {
                self.add_res_bonus(source_idx, amount);
                Some(format!("AddResBonus +{amount:.2}"))
            }
            EffectSpec::ModifyStatusPower { status_type, mul } => {
                self.update_status_power_mul(source_idx, status_type, mul);
                Some(format!(
                    "ModifyStatusPower {} x{mul:.2}",
                    status_type.as_str()
                ))
            }
            EffectSpec::AddStatusStacks {
                target,
                status_type,
                stacks,
            } => {
                if let Some(target_idx) = self.resolve_effect_target(target, context) {
                    self.apply_status(
                        source_idx,
                        target_idx,
                        status_type,
                        1.0,
                        1.0,
                        stacks.max(1),
                        1.0,
                        depth,
                        events,
                    );
                    Some(format!(
                        "AddStatusStacks {} +{}",
                        status_type.as_str(),
                        stacks.max(1)
                    ))
                } else {
                    None
                }
            }
            EffectSpec::DealPureDamage { target, amount } => {
                if let Some(dst_idx) = self.resolve_effect_target(target, context) {
                    self.apply_pure_damage(source_idx, dst_idx, amount.max(0.01), depth, events);
                    Some(format!("DealPureDamage {:.2}", amount.max(0.01)))
                } else {
                    None
                }
            }
            EffectSpec::ConditionalDamageAmp { .. } | EffectSpec::ConditionalApplyStatus { .. } => {
                None
            }
        }
    }
}
