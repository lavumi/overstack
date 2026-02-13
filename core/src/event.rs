/// Structured run event emitted from Rust and consumed by JS UI.
pub enum Event {
    RunStart {
        seed: u64,
    },
    NodeStart {
        node_index: u32,
        node_type: &'static str,
    },
    BattleStart {
        battle_index: u32,
        enemy_name: &'static str,
    },
    TurnReady {
        actor: &'static str,
    },
    ActionUsed {
        actor: &'static str,
        action_name: &'static str,
    },
    DamageDealt {
        src: &'static str,
        dst: &'static str,
        amount: f32,
        dst_hp_after: f32,
    },
    StatusApplied {
        src: &'static str,
        dst: &'static str,
        status: &'static str,
        stacks: u32,
        duration: u32,
    },
    StatusTick {
        dst: &'static str,
        status: &'static str,
        amount: f32,
        dst_hp_after: f32,
    },
    StatusExpired {
        dst: &'static str,
        status: &'static str,
    },
    BattleEnd {
        result: &'static str,
        player_hp_after: f32,
    },
    RunEnd {
        result: &'static str,
        final_node_index: u32,
    },
    TraitTriggered {
        trait_name: &'static str,
        trigger_type: &'static str,
    },
    TraitEffectApplied {
        trait_name: &'static str,
        effect_summary: String,
    },
}

impl Event {
    pub fn to_json_line(&self) -> String {
        match self {
            Event::RunStart { seed } => {
                format!(r#"{{"kind":"RunStart","seed":{seed}}}"#)
            }
            Event::NodeStart {
                node_index,
                node_type,
            } => {
                format!(
                    r#"{{"kind":"NodeStart","node_index":{},"node_type":"{}"}}"#,
                    node_index,
                    escape_json(node_type)
                )
            }
            Event::BattleStart {
                battle_index,
                enemy_name,
            } => {
                format!(
                    r#"{{"kind":"BattleStart","battle_index":{},"enemy_name":"{}"}}"#,
                    battle_index,
                    escape_json(enemy_name)
                )
            }
            Event::TurnReady { actor } => {
                format!(
                    r#"{{"kind":"TurnReady","actor":"{}"}}"#,
                    escape_json(actor)
                )
            }
            Event::ActionUsed { actor, action_name } => {
                format!(
                    r#"{{"kind":"ActionUsed","actor":"{}","action_name":"{}"}}"#,
                    escape_json(actor),
                    escape_json(action_name)
                )
            }
            Event::DamageDealt {
                src,
                dst,
                amount,
                dst_hp_after,
            } => {
                format!(
                    r#"{{"kind":"DamageDealt","src":"{}","dst":"{}","amount":{},"dst_hp_after":{}}}"#,
                    escape_json(src),
                    escape_json(dst),
                    json_f32(*amount),
                    json_f32(*dst_hp_after)
                )
            }
            Event::StatusApplied {
                src,
                dst,
                status,
                stacks,
                duration,
            } => {
                format!(
                    r#"{{"kind":"StatusApplied","src":"{}","dst":"{}","status":"{}","stacks":{},"duration":{}}}"#,
                    escape_json(src),
                    escape_json(dst),
                    escape_json(status),
                    stacks,
                    duration
                )
            }
            Event::StatusTick {
                dst,
                status,
                amount,
                dst_hp_after,
            } => {
                format!(
                    r#"{{"kind":"StatusTick","dst":"{}","status":"{}","amount":{},"dst_hp_after":{}}}"#,
                    escape_json(dst),
                    escape_json(status),
                    json_f32(*amount),
                    json_f32(*dst_hp_after)
                )
            }
            Event::StatusExpired { dst, status } => {
                format!(
                    r#"{{"kind":"StatusExpired","dst":"{}","status":"{}"}}"#,
                    escape_json(dst),
                    escape_json(status)
                )
            }
            Event::BattleEnd {
                result,
                player_hp_after,
            } => {
                format!(
                    r#"{{"kind":"BattleEnd","result":"{}","player_hp_after":{}}}"#,
                    escape_json(result),
                    json_f32(*player_hp_after)
                )
            }
            Event::RunEnd {
                result,
                final_node_index,
            } => {
                format!(
                    r#"{{"kind":"RunEnd","result":"{}","final_node_index":{}}}"#,
                    escape_json(result),
                    final_node_index
                )
            }
            Event::TraitTriggered {
                trait_name,
                trigger_type,
            } => {
                format!(
                    r#"{{"kind":"TraitTriggered","trait_name":"{}","trigger_type":"{}"}}"#,
                    escape_json(trait_name),
                    escape_json(trigger_type)
                )
            }
            Event::TraitEffectApplied {
                trait_name,
                effect_summary,
            } => {
                format!(
                    r#"{{"kind":"TraitEffectApplied","trait_name":"{}","effect_summary":"{}"}}"#,
                    escape_json(trait_name),
                    escape_json(effect_summary)
                )
            }
        }
    }
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn json_f32(v: f32) -> String {
    format!("{:.2}", v)
}
