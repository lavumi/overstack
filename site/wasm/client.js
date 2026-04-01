// wasm-bindgen output is loaded via relative path for GitHub Pages root compatibility.
import init, {
  create_run_with_stats,
  destroy_run,
  get_player_skills,
  get_selectable_skill_costs,
  get_selectable_skill_ids,
  get_selectable_skill_names,
  get_selectable_trait_costs,
  get_selectable_trait_ids,
  get_selectable_trait_names,
  get_snapshot,
  reset_run,
  run_run,
  sample_starting_skill_ids,
  sample_starting_trait_ids,
  set_player_skills,
  set_active_traits,
  step_with_action,
} from "../pkg/core.js";

export function createWasmClient() {
  async function boot() {
    await init();
  }

  function runSmoke(seed = 1234, nodes = 1) {
    return run_run(seed, nodes);
  }

  function getSelectableTraits() {
    return {
      ids: get_selectable_trait_ids(),
      names: get_selectable_trait_names(),
      costs: get_selectable_trait_costs(),
    };
  }

  function getSelectableSkills() {
    return {
      ids: get_selectable_skill_ids(),
      names: get_selectable_skill_names(),
      costs: get_selectable_skill_costs(),
    };
  }

  function sampleStartingTraitIds(seed, count) {
    return sample_starting_trait_ids(seed, count);
  }

  function sampleStartingSkillIds(seed, count) {
    return sample_starting_skill_ids(seed, count);
  }

  function createRunWithStats(seed, maxNodes, stats) {
    return create_run_with_stats(
      seed,
      maxNodes,
      stats.max_hp,
      stats.atk,
      stats.matk,
      stats.def,
      stats.mdef,
      stats.speed,
      stats.crit_rate,
      stats.crit_mult,
    );
  }

  function destroyRun(handle) {
    destroy_run(handle);
  }

  function resetRun(handle) {
    reset_run(handle);
  }

  function setActiveTraits(handle, traitIds) {
    return set_active_traits(handle, traitIds.join(","));
  }

  function setPlayerSkills(handle, skillIds) {
    return set_player_skills(handle, skillIds.join(","));
  }

  function getPlayerSkills(handle) {
    return get_player_skills(handle);
  }

  function getSnapshot(handle) {
    return get_snapshot(handle);
  }

  function step(handle, dt, actionKind, actionArg) {
    return step_with_action(handle, dt, actionKind, actionArg);
  }

  return {
    boot,
    runSmoke,
    getSelectableSkills,
    getSelectableTraits,
    sampleStartingSkillIds,
    sampleStartingTraitIds,
    createRunWithStats,
    destroyRun,
    resetRun,
    setActiveTraits,
    setPlayerSkills,
    getPlayerSkills,
    getSnapshot,
    step,
  };
}
