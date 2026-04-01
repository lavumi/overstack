use std::collections::HashMap;
use std::sync::OnceLock;

use super::compile::compile_defs;
use super::defs::parse_embedded_defs;
use super::errors::ErrorReport;
use super::specs::{EnemyId, EnemySpec, SkillId, SkillSpec, TraitId, TraitSpec};
use super::validate::validate_defs;

pub type SkillRegistry = HashMap<SkillId, SkillSpec>;
pub type TraitRegistry = HashMap<TraitId, TraitSpec>;
pub type EnemyRegistry = HashMap<EnemyId, EnemySpec>;

pub struct GameData {
    pub skills: SkillRegistry,
    pub traits: TraitRegistry,
    pub enemies: EnemyRegistry,
    pub player_loadout: [SkillId; 4],
    pub selectable_skills: Vec<SkillId>,
    pub selectable_traits: Vec<TraitId>,
    pub enemy_trait_pool: Vec<TraitId>,
}

static GAME_DATA: OnceLock<Result<GameData, ErrorReport>> = OnceLock::new();
static ERROR_REPORTED: OnceLock<()> = OnceLock::new();

pub fn load_embedded_game_data() -> Result<&'static GameData, ErrorReport> {
    let loaded = GAME_DATA.get_or_init(|| {
        let defs = parse_embedded_defs()?;
        validate_defs(&defs)?;
        compile_defs(&defs)
    });

    match loaded {
        Ok(data) => Ok(data),
        Err(report) => {
            report_error_once(report);
            Err(report.clone())
        }
    }
}

fn report_error_once(report: &ErrorReport) {
    if ERROR_REPORTED.get().is_some() {
        return;
    }

    let _ = ERROR_REPORTED.set(());
    let message = format!("[game_data] load_failed: {}", report.join_messages());

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn error(message: &str);
        }
        error(&message);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::load_embedded_game_data;

    #[test]
    fn embedded_data_loads_successfully() {
        assert!(load_embedded_game_data().is_ok());
    }
}
