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
    pub hp: f32,
    pub max_hp: f32,
    pub atk: i32,
    pub matk: i32,
    pub def: i32,
    pub mdef: i32,
    pub crit_rate: f32,
    pub crit_mult: f32,
    pub speed: f32,
    pub action_gauge: f32,
}

impl Unit {
    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
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
    pub player_hp: f32,
    pub player_max_hp: f32,
    pub player_atk: i32,
    pub player_matk: i32,
    pub player_def: i32,
    pub player_mdef: i32,
    pub player_crit_rate: f32,
    pub player_crit_mult: f32,
    pub player_speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerInitStats {
    pub max_hp: f32,
    pub atk: i32,
    pub matk: i32,
    pub def: i32,
    pub mdef: i32,
    pub speed: f32,
    pub crit_rate: f32,
    pub crit_mult: f32,
}

impl PlayerInitStats {
    pub fn default_run() -> Self {
        Self {
            max_hp: 140.0,
            atk: 17,
            matk: 17,
            def: 10,
            mdef: 10,
            speed: 35.0,
            crit_rate: 15.0,
            crit_mult: 1.5,
        }
    }
}

impl RunState {
    pub fn new(seed: u64) -> Self {
        let stats = PlayerInitStats::default_run();
        Self {
            seed,
            rng: crate::rng::SimpleRng::new(seed),
            floor: 1,
            stage: 0,
            meta_placeholder: 0,
            player_hp: stats.max_hp,
            player_max_hp: stats.max_hp,
            player_atk: stats.atk,
            player_matk: stats.matk,
            player_def: stats.def,
            player_mdef: stats.mdef,
            player_crit_rate: stats.crit_rate,
            player_crit_mult: stats.crit_mult,
            player_speed: stats.speed,
        }
    }

    pub fn apply_player_stats(&mut self, stats: PlayerInitStats) {
        self.player_max_hp = stats.max_hp;
        self.player_hp = stats.max_hp;
        self.player_atk = stats.atk;
        self.player_matk = stats.matk;
        self.player_def = stats.def;
        self.player_mdef = stats.mdef;
        self.player_speed = stats.speed;
        self.player_crit_rate = stats.crit_rate;
        self.player_crit_mult = stats.crit_mult;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleOutcome {
    Victory,
    Defeat,
}
