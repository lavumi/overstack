use crate::engine::runtime::{ActionKind, ActiveRun, TraitOwner, TriggerContext};
use crate::event::Event;
use crate::log::push_event;
use crate::model::Team;
use crate::skill::{player_skill_for_slot, skill_by_id, DamageKind, EffectSpec, SkillSpec};
use crate::trait_spec::TriggerType;

struct ActionSnapshot {
    actor_idx: usize,
    target_idx: usize,
    skill: &'static SkillSpec,
    context: TriggerContext,
}

struct DamageStep {
    damage_kind: DamageKind,
    multiplier: f32,
    flat: f32,
}

struct ResolvedActionLayers {
    damage_steps: Vec<DamageStep>,
    status_steps: Vec<EffectSpec>,
}

impl ActiveRun {
    fn choose_skill_for_action(&self, action: ActionKind) -> &'static SkillSpec {
        match action {
            ActionKind::BasicAttack => {
                skill_by_id("basic_attack").unwrap_or_else(|| player_skill_for_slot(0))
            }
            ActionKind::SkillSlot(slot) => player_skill_for_slot(slot),
        }
    }

    fn execute_skill(
        &mut self,
        actor_idx: usize,
        target_idx: usize,
        skill: &'static SkillSpec,
        events: &mut Vec<String>,
    ) {
        let actor = self.actor_label_for_idx(actor_idx);
        let owner = if actor == "player" {
            TraitOwner::Player
        } else {
            TraitOwner::Enemy
        };

        push_event(events, Event::TurnReady { actor });
        push_event(
            events,
            Event::ActionUsed {
                actor,
                action_name: skill.name,
            },
        );

        let snapshot = ActionSnapshot {
            actor_idx,
            target_idx,
            skill,
            context: TriggerContext {
                trigger_type: TriggerType::OnActionUsed,
                owner,
                src_idx: Some(actor_idx),
                dst_idx: Some(target_idx),
                applied_status: None,
            },
        };
        self.emit_pre_action_trait_triggers(snapshot.context, 0, events);

        let layers = self.resolve_action_layers(&snapshot);
        self.execute_action_layers(&snapshot, layers, events);
    }

    fn resolve_action_layers(&mut self, snapshot: &ActionSnapshot) -> ResolvedActionLayers {
        let mut damage_amp = 1.0_f32;
        let mut damage_steps = Vec::new();
        let mut status_steps = Vec::new();

        for effect in snapshot.skill.effects {
            match *effect {
                EffectSpec::ConditionalDamageAmp { condition, amp } => {
                    if self.evaluate_condition(condition, snapshot.context) {
                        damage_amp *= amp.max(0.1);
                    }
                }
                EffectSpec::ConditionalApplyStatus {
                    condition,
                    status_type,
                    base_chance,
                    duration,
                    stacks,
                    power,
                } => {
                    if self.evaluate_condition(condition, snapshot.context) {
                        status_steps.push(EffectSpec::ApplyStatus {
                            status_type,
                            base_chance,
                            duration,
                            stacks,
                            power,
                        });
                    }
                }
                EffectSpec::DealDamage {
                    damage_kind,
                    multiplier,
                    flat,
                } => {
                    let final_multiplier =
                        snapshot.skill.base_damage_multiplier * multiplier * damage_amp;
                    let final_flat = snapshot.skill.flat_bonus_damage.unwrap_or(0.0) + flat;
                    let final_kind = match snapshot.skill.id {
                        "basic_attack" => DamageKind::Physical,
                        _ => damage_kind,
                    };
                    damage_steps.push(DamageStep {
                        damage_kind: final_kind,
                        multiplier: final_multiplier,
                        flat: final_flat,
                    });
                }
                other => status_steps.push(other),
            }
        }

        ResolvedActionLayers {
            damage_steps,
            status_steps,
        }
    }

    fn execute_action_layers(
        &mut self,
        snapshot: &ActionSnapshot,
        layers: ResolvedActionLayers,
        events: &mut Vec<String>,
    ) {
        for step in layers.damage_steps {
            self.apply_scaled_damage(
                snapshot.actor_idx,
                snapshot.target_idx,
                step.damage_kind,
                step.multiplier,
                step.flat,
                0,
                events,
            );
        }

        for effect in layers.status_steps {
            let _ = self.execute_primitive_effect(
                effect,
                snapshot.actor_idx,
                snapshot.target_idx,
                snapshot.context,
                0,
                events,
            );
        }
    }

    pub(crate) fn execute_turn(
        &mut self,
        actor_idx: usize,
        action: ActionKind,
        events: &mut Vec<String>,
    ) -> Option<&'static str> {
        let Some(state) = self.state_ref() else {
            return None;
        };

        if !state.units[actor_idx].is_alive() || state.units[actor_idx].action_gauge < 100.0 {
            return None;
        }

        let actor_team = state.units[actor_idx].team;
        let target_team = if actor_team == Team::Player {
            Team::Enemy
        } else {
            Team::Player
        };

        let Some(target_idx) = self.pick_target_index(target_team) else {
            return None;
        };

        if let Some(state) = self.state_mut() {
            state.units[actor_idx].action_gauge -= 100.0;
        }

        let skill = if actor_team == Team::Player {
            self.choose_skill_for_action(action)
        } else {
            skill_by_id("basic_attack").unwrap_or_else(|| player_skill_for_slot(0))
        };

        self.execute_skill(actor_idx, target_idx, skill, events);
        self.check_and_emit_battle_end(events)
    }
}
