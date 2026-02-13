mod battle;
mod log;
mod model;
mod rng;
mod run;

use wasm_bindgen::prelude::*;

/// Runs a tiny deterministic simulation and returns the final state.
/// `seed` is the initial value and `steps` controls iteration count.
#[wasm_bindgen]
pub fn run_sim(seed: u32, steps: u32) -> u32 {
    let mut state = seed;

    // Simple linear-congruential update to keep the example minimal.
    for _ in 0..steps {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    }

    state
}

/// Runs one game run skeleton and returns number of cleared nodes.
#[wasm_bindgen]
pub fn run_run(seed: u32, max_nodes: u32) -> u32 {
    run::run_run_internal(seed as u64, max_nodes)
}
