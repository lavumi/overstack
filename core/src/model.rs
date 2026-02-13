/// High-level map node categories for a run.
/// For now, only `Battle` and `Boss` are executed by the skeleton loop.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Battle,
    Event,
    Shop,
    Rest,
    Boss,
}

/// Simple two-side team marker used in battle targeting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

/// Runtime unit data used by the gauge-based timeline.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: u32,
    pub team: Team,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub speed: f32,
    pub action_gauge: f32,
}

impl Unit {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

/// Per-battle runtime state.
pub struct BattleState {
    pub units: Vec<Unit>,
    pub delta_time: f32,
    pub tick: u32,
}

/// Full run state placeholder. Keeps RNG and run progression fields.
#[allow(dead_code)]
pub struct RunState {
    pub seed: u64,
    pub rng: crate::rng::SimpleRng,
    pub floor: u32,
    pub stage: u32,
    pub meta_placeholder: u32,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_atk: i32,
    pub player_speed: f32,
}

impl RunState {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: crate::rng::SimpleRng::new(seed),
            floor: 1,
            stage: 0,
            meta_placeholder: 0,
            player_hp: 140,
            player_max_hp: 140,
            player_atk: 17,
            player_speed: 35.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleOutcome {
    Victory,
    Defeat,
}
