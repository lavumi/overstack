// wasm-bindgen output is loaded via relative path for GitHub Pages root compatibility.
import init, {
  create_run_with_stats,
  destroy_run,
  get_player_skills,
  get_selectable_trait_costs,
  get_selectable_trait_ids,
  get_selectable_trait_names,
  get_snapshot,
  reset_run,
  run_run,
  sample_starting_trait_ids,
  set_active_traits,
  step_with_action,
} from "./pkg/core.js";

const seedInput = document.getElementById("seedInput");
const speedSelect = document.getElementById("speedSelect");
const startBtn = document.getElementById("startBtn");
const resetBtn = document.getElementById("resetBtn");
const bootStatus = document.getElementById("bootStatus");
const arenaStatus = document.getElementById("arenaStatus");
const arenaHint = document.getElementById("arenaHint");

const statusNode = document.getElementById("statusNode");
const statusBattle = document.getElementById("statusBattle");
const playerTraits = document.getElementById("playerTraits");
const enemyTraits = document.getElementById("enemyTraits");

const playerName = document.getElementById("playerName");
const playerHpText = document.getElementById("playerHpText");
const playerHpFill = document.getElementById("playerHpFill");
const playerGaugeText = document.getElementById("playerGaugeText");
const playerGaugeFill = document.getElementById("playerGaugeFill");
const playerAtkMatk = document.getElementById("playerAtkMatk");
const playerDef = document.getElementById("playerDef");
const playerMdef = document.getElementById("playerMdef");
const playerCrit = document.getElementById("playerCrit");
const playerStatuses = document.getElementById("playerStatuses");

const enemyName = document.getElementById("enemyName");
const enemyHpText = document.getElementById("enemyHpText");
const enemyHpFill = document.getElementById("enemyHpFill");
const enemyGaugeText = document.getElementById("enemyGaugeText");
const enemyGaugeFill = document.getElementById("enemyGaugeFill");
const enemyAtkMatk = document.getElementById("enemyAtkMatk");
const enemyDef = document.getElementById("enemyDef");
const enemyMdef = document.getElementById("enemyMdef");
const enemyIntent = document.getElementById("enemyIntent");
const enemyStatuses = document.getElementById("enemyStatuses");
const stagePlayer = document.getElementById("stagePlayer");
const stageEnemy = document.getElementById("stageEnemy");
const stagePlayerName = document.getElementById("stagePlayerName");
const stageEnemyName = document.getElementById("stageEnemyName");
const stagePlayerVitals = document.getElementById("stagePlayerVitals");
const stageEnemyVitals = document.getElementById("stageEnemyVitals");
const stagePlayerState = document.getElementById("stagePlayerState");
const stageEnemyState = document.getElementById("stageEnemyState");

const inputPrompt = document.getElementById("inputPrompt");
const actionBasicBtn = document.getElementById("actionBasic");
const actionSkillButtons = [
  document.getElementById("actionSkill1"),
  document.getElementById("actionSkill2"),
  document.getElementById("actionSkill3"),
  document.getElementById("actionSkill4"),
];

const builderOverlay = document.getElementById("builderOverlay");
const builderModeRandom = document.getElementById("builderModeRandom");
const builderModeManual = document.getElementById("builderModeManual");
const builderBudgetText = document.getElementById("builderBudgetText");
const builderStatsCostText = document.getElementById("builderStatsCostText");
const builderTraitCostText = document.getElementById("builderTraitCostText");
const builderRemainingText = document.getElementById("builderRemainingText");
const builderTraitHint = document.getElementById("builderTraitHint");
const builderTraitChoices = document.getElementById("builderTraitChoices");
const builderError = document.getElementById("builderError");
const builderRandomBtn = document.getElementById("builderRandomBtn");
const builderRerollBtn = document.getElementById("builderRerollBtn");
const builderConfirmBtn = document.getElementById("builderConfirmBtn");
const builderCancelBtn = document.getElementById("builderCancelBtn");

const statInputs = {
  max_hp: document.getElementById("statMaxHp"),
  atk: document.getElementById("statAtk"),
  matk: document.getElementById("statMatk"),
  def: document.getElementById("statDef"),
  mdef: document.getElementById("statMdef"),
  speed: document.getElementById("statSpeed"),
  crit_rate: document.getElementById("statCritRate"),
  crit_mult: document.getElementById("statCritMult"),
};

const STEP_DT_BASE = 0.15;
const LOOP_MS = 120;
const MAX_NODES = 6;
const CRIT_C = 100;
const START_BUILD_BUDGET = 100;

const DEFAULT_BUILDER_STATS = {
  max_hp: 130,
  atk: 30,
  matk: 25,
  def: 20,
  mdef: 15,
  speed: 1.0,
  crit_rate: 50,
  crit_mult: 1.5,
};

const STAT_RANGES = {
  max_hp: [80, 200],
  atk: [10, 60],
  matk: [10, 60],
  def: [-20, 80],
  mdef: [0, 80],
  speed: [0.6, 2.0],
  crit_rate: [0, 300],
  crit_mult: [1.25, 2.5],
};

let currentHandle = null;
let loopTimer = null;
let uiMode = "idle"; // idle | builder | running | need_input | ended
let selectableTraitIds = [];
let selectableTraitNames = [];
let selectableTraitCosts = [];
let selectedBuilderTraitIds = [];
let lastEnemyName = "Enemy";
let builderMode = "random";

function stopLoop() {
  if (loopTimer !== null) {
    clearInterval(loopTimer);
    loopTimer = null;
  }
}

function currentStepDt() {
  const speed = Number.parseFloat(speedSelect.value);
  const safe = Number.isFinite(speed) && speed > 0 ? speed : 1;
  return STEP_DT_BASE * safe;
}

function setActionButtonsEnabled(enabled) {
  actionBasicBtn.disabled = !enabled;
  for (const button of actionSkillButtons) {
    button.disabled = !enabled;
  }
}

function setInputPrompt(text) {
  inputPrompt.textContent = text;
}

function setArenaStatus(text) {
  arenaStatus.firstElementChild.textContent = text;
}

function setCombatLabels(skillNames) {
  actionBasicBtn.textContent = "Basic Attack";
  for (let i = 0; i < actionSkillButtons.length; i += 1) {
    actionSkillButtons[i].textContent = skillNames[i] || `Skill ${i + 1}`;
  }
}

function clampPct(v) {
  return Math.max(0, Math.min(100, v));
}

function setBar(fillEl, current, max) {
  const pct = max > 0 ? clampPct((current / max) * 100) : 0;
  fillEl.style.width = `${pct.toFixed(1)}%`;
}

function statusKey(statusType) {
  return String(statusType || "")
    .trim()
    .toLowerCase();
}

function renderPillList(container, items, renderItem, emptyLabel, extraClass = "") {
  container.innerHTML = "";
  const frag = document.createDocumentFragment();

  if (!items || items.length === 0) {
    const empty = document.createElement("span");
    empty.className = `${extraClass} empty-pill`.trim();
    empty.textContent = emptyLabel;
    frag.appendChild(empty);
    container.appendChild(frag);
    return;
  }

  for (const item of items) {
    frag.appendChild(renderItem(item));
  }

  container.appendChild(frag);
}

function renderStatusList(container, statuses) {
  renderPillList(
    container,
    statuses,
    (status) => {
      const chip = document.createElement("span");
      chip.className = "status-pill";
      chip.dataset.status = statusKey(status.status_type);
      chip.textContent = `${status.status_type} x${status.stacks} · ${Number(status.duration).toFixed(1)}s`;
      return chip;
    },
    "No active statuses",
    "status-pill",
  );
}

function renderTraitList(container, traits) {
  renderPillList(
    container,
    traits,
    (traitName) => {
      const chip = document.createElement("span");
      chip.className = "trait-pill";
      chip.textContent = traitName;
      return chip;
    },
    "No active traits",
    "trait-pill",
  );
}

function randomSeed32() {
  return Math.floor(Math.random() * 0x100000000) >>> 0;
}

function selectedBuilderTraitName() {
  return selectedBuilderTraitIds
    .map((selectedId) => {
      const idx = selectableTraitIds.indexOf(selectedId);
      return idx >= 0 ? selectableTraitNames[idx] || selectedId : selectedId;
    })
    .join(", ");
}

function renderBuilderTraits() {
  builderTraitChoices.innerHTML = "";
  const frag = document.createDocumentFragment();
  const selectedSet = new Set(selectedBuilderTraitIds);

  for (let i = 0; i < selectableTraitIds.length; i += 1) {
    const id = selectableTraitIds[i];
    const name = selectableTraitNames[i] || id;
    const cost = Number(selectableTraitCosts[i] || 0);
    const button = document.createElement("button");
    button.type = "button";
    button.className = "builder-trait-option";
    if (selectedSet.has(id)) {
      button.classList.add("is-selected");
    }
    button.disabled = builderMode !== "manual";
    button.innerHTML = `
      <span class="builder-trait-name">${name}</span>
      <span class="builder-trait-cost">Cost ${cost}</span>
      <span class="builder-trait-id">${id}</span>
    `;
    button.addEventListener("click", () => {
      if (builderMode !== "manual") return;
      if (selectedSet.has(id)) {
        selectedBuilderTraitIds = selectedBuilderTraitIds.filter((traitId) => traitId !== id);
      } else {
        const candidate = [...selectedBuilderTraitIds, id];
        const stats = readBuilderStats();
        const candidateCost = candidate.reduce((sum, traitId) => {
          const traitIdx = selectableTraitIds.indexOf(traitId);
          return sum + Number(selectableTraitCosts[traitIdx] || 0);
        }, 0);
        const totalCost = calcTotalCost(stats) + candidateCost;
        if (totalCost > START_BUILD_BUDGET) {
          builderError.textContent = `Trait budget exceeded: ${totalCost.toFixed(1)} / ${START_BUILD_BUDGET.toFixed(1)}`;
          return;
        }
        selectedBuilderTraitIds = candidate;
      }
      builderError.textContent = "";
      renderBuilderTraits();
      refreshBudgetText();
    });
    frag.appendChild(button);
  }

  builderTraitChoices.appendChild(frag);
}

function sampleBuilderTraits() {
  const stats = readBuilderStats();
  const statsCost = calcTotalCost(stats);
  let remaining = START_BUILD_BUDGET - statsCost;
  if (remaining <= 0) {
    selectedBuilderTraitIds = [];
    renderBuilderTraits();
    refreshBudgetText(stats);
    return;
  }

  const sampled = sample_starting_trait_ids(randomSeed32(), selectableTraitIds.length);
  const picked = [];
  for (const traitId of sampled) {
    const idx = selectableTraitIds.indexOf(traitId);
    const cost = idx >= 0 ? Number(selectableTraitCosts[idx] || 0) : 0;
    if (cost <= remaining) {
      picked.push(traitId);
      remaining -= cost;
    }
  }

  selectedBuilderTraitIds = picked;
  renderBuilderTraits();
  refreshBudgetText(stats);
}

function unitStageState(unit) {
  const hp = Number(unit.hp);
  const maxHp = Number(unit.max_hp);
  const gauge = Number(unit.action_gauge);
  if (hp <= 0) return "down";
  if (gauge >= 100) return "ready";
  if (maxHp > 0 && hp / maxHp <= 0.35) return "critical";
  return "idle";
}

function unitStateSummary(unit, traitCount) {
  const statuses = unit.statuses || [];
  if (Number(unit.hp) <= 0) {
    return "Unit collapsed";
  }
  if (Number(unit.action_gauge) >= 100) {
    return "Action gauge primed";
  }
  if (statuses.length > 0) {
    return `${statuses.length} status effect${statuses.length > 1 ? "s" : ""} active`;
  }
  if (traitCount > 0) {
    return `${traitCount} trait${traitCount > 1 ? "s" : ""} influencing combat`;
  }
  return "Stable combat posture";
}

function resetStatus() {
  statusNode.textContent = "-";
  statusBattle.textContent = "-";
  playerName.textContent = "Player";
  enemyName.textContent = "Enemy";
  stagePlayerName.textContent = "Player";
  stageEnemyName.textContent = "Enemy";
  stagePlayerVitals.textContent = "HP - / Gauge -";
  stageEnemyVitals.textContent = "HP - / Gauge -";
  stagePlayerState.textContent = "Awaiting deployment";
  stageEnemyState.textContent = "No encounter loaded";
  stagePlayer.dataset.state = "idle";
  stageEnemy.dataset.state = "idle";
  playerHpText.textContent = "-";
  enemyHpText.textContent = "-";
  playerGaugeText.textContent = "-";
  enemyGaugeText.textContent = "-";
  playerAtkMatk.textContent = "-";
  enemyAtkMatk.textContent = "-";
  playerDef.textContent = "-";
  enemyDef.textContent = "-";
  playerMdef.textContent = "-";
  enemyMdef.textContent = "-";
  playerCrit.textContent = "-";
  enemyIntent.textContent = "-";
  renderStatusList(playerStatuses, []);
  renderStatusList(enemyStatuses, []);
  renderTraitList(playerTraits, []);
  renderTraitList(enemyTraits, []);
  setBar(playerHpFill, 0, 1);
  setBar(enemyHpFill, 0, 1);
  setBar(playerGaugeFill, 0, 100);
  setBar(enemyGaugeFill, 0, 100);
  setArenaStatus("Ready");
  arenaHint.textContent = "Prepare your opening line.";
  lastEnemyName = "Enemy";
}

function closeBuilder() {
  builderOverlay.classList.add("hidden");
  if (uiMode === "builder") {
    uiMode = "idle";
  }
}

function resetAll() {
  resetStatus();
  setCombatLabels([]);
  setActionButtonsEnabled(false);
  setInputPrompt("");
  selectableTraitIds = [];
  selectableTraitNames = [];
  selectableTraitCosts = [];
  selectedBuilderTraitIds = [];
  closeBuilder();
  uiMode = "idle";
}

function parseEvent(line) {
  try {
    return JSON.parse(line);
  } catch (error) {
    return { kind: "InvalidJSON", raw: line, error: String(error) };
  }
}

function formatEventLine(event) {
  switch (event.kind) {
    case "RunStart":
      return `[RunStart] seed=${event.seed}`;
    case "NodeStart":
      return `[NodeStart] node=${event.node_index} type=${event.node_type}`;
    case "BattleStart":
      return `[BattleStart] battle=${event.battle_index} enemy=${event.enemy_name}`;
    case "TurnReady":
      return `[TurnReady] actor=${event.actor}`;
    case "ActionUsed":
      return `[ActionUsed] actor=${event.actor} action=${event.action_name}`;
    case "DamageDealt":
      return `[DamageDealt] ${event.src} -> ${event.dst} kind=${event.damage_kind} raw=${Number(event.raw).toFixed(2)} def=${event.defense_used} mit=${Number(event.mitigation).toFixed(2)} crit=${event.crit} dmg=${Number(event.amount).toFixed(2)} dst_hp=${Number(event.dst_hp_after).toFixed(2)}`;
    case "StatusApplied":
      return `[StatusApplied] ${event.src} -> ${event.dst} ${event.status} stacks=${event.stacks} duration=${event.duration}`;
    case "StatusTick":
      return `[StatusTick] ${event.dst} ${event.status} amount=${Number(event.amount).toFixed(2)} hp=${Number(event.dst_hp_after).toFixed(2)}`;
    case "StatusExpired":
      return `[StatusExpired] ${event.dst} ${event.status}`;
    case "BattleEnd":
      return `[BattleEnd] result=${event.result} player_hp=${Number(event.player_hp_after).toFixed(2)}`;
    case "RunEnd":
      return `[RunEnd] result=${event.result} final_node=${event.final_node_index}`;
    case "TraitTriggered":
      return `[${event.owner === "enemy" ? "E" : "P"}] [TraitTriggered] ${event.trait_name} via ${event.trigger_type}`;
    case "TraitEffectApplied":
      return `[TraitEffectApplied] ${event.trait_name}: ${event.effect_summary}`;
    default:
      return `[UnknownEvent] ${JSON.stringify(event)}`;
  }
}

function updateArenaStatusByEvent(event) {
  if (event.kind === "BattleStart") {
    lastEnemyName = event.enemy_name || "Enemy";
    enemyName.textContent = lastEnemyName;
    stageEnemyName.textContent = lastEnemyName;
    setArenaStatus(`Encounter: ${lastEnemyName}`);
    arenaHint.textContent = "New hostile pattern detected.";
    return;
  }
  if (event.kind === "TurnReady") {
    setArenaStatus(event.actor === "player" ? "Player Turn" : "Enemy Turn");
    arenaHint.textContent =
      event.actor === "player" ? "Action window open." : "Brace for incoming action.";
    return;
  }
  if (event.kind === "ActionUsed") {
    const actor = event.actor === "player" ? "Player" : "Enemy";
    setArenaStatus(`${actor} uses ${event.action_name}`);
    arenaHint.textContent = `${actor} committed ${event.action_name}.`;
    return;
  }
  if (event.kind === "BattleEnd") {
    setArenaStatus(event.result === "win" ? "Battle Won" : "Battle Lost");
    arenaHint.textContent =
      event.result === "win" ? "Arena secured. Prepare the next node." : "Recovery required.";
    return;
  }
  if (event.kind === "RunEnd") {
    setArenaStatus(event.result === "win" ? "Run Complete" : "Run Failed");
    arenaHint.textContent =
      event.result === "win" ? "Simulation complete." : "Run terminated.";
  }
}

function appendEventLines(events) {
  if (events.length === 0) {
    return;
  }

  for (const line of events) {
    const event = parseEvent(line);
    updateArenaStatusByEvent(event);
    const tickLabel = Number.isFinite(Number(event.tick))
      ? `t=${String(Math.trunc(Number(event.tick))).padStart(4, "0")}`
      : "t=----";
    console.log(`[${tickLabel}] ${formatEventLine(event)}`);
  }
}

function critChancePercent(critRate) {
  const rate = Number(critRate);
  if (!Number.isFinite(rate) || rate <= 0) {
    return 0;
  }
  return (rate / (rate + CRIT_C)) * 100;
}

function updateHudFromSnapshot(snapshot) {
  statusNode.textContent = String(snapshot.node_index);
  statusBattle.textContent = String(snapshot.battle_index);

  playerName.textContent = "Player";
  enemyName.textContent = lastEnemyName || "Enemy";
  stagePlayerName.textContent = "Player";
  stageEnemyName.textContent = lastEnemyName || "Enemy";

  const php = Number(snapshot.player.hp);
  const pmax = Number(snapshot.player.max_hp);
  const ehp = Number(snapshot.enemy.hp);
  const emax = Number(snapshot.enemy.max_hp);
  const pg = Number(snapshot.player.action_gauge);
  const eg = Number(snapshot.enemy.action_gauge);

  playerHpText.textContent = `${php.toFixed(0)} / ${pmax.toFixed(0)}`;
  enemyHpText.textContent = `${ehp.toFixed(0)} / ${emax.toFixed(0)}`;
  playerGaugeText.textContent = pg.toFixed(1);
  enemyGaugeText.textContent = eg.toFixed(1);
  stagePlayerVitals.textContent = `${php.toFixed(0)} HP · Gauge ${pg.toFixed(0)}`;
  stageEnemyVitals.textContent = `${ehp.toFixed(0)} HP · Gauge ${eg.toFixed(0)}`;
  stagePlayerState.textContent = unitStateSummary(snapshot.player, (snapshot.player_traits || []).length);
  stageEnemyState.textContent = unitStateSummary(snapshot.enemy, (snapshot.enemy_traits || []).length);
  stagePlayer.dataset.state = unitStageState(snapshot.player);
  stageEnemy.dataset.state = unitStageState(snapshot.enemy);

  setBar(playerHpFill, php, pmax);
  setBar(enemyHpFill, ehp, emax);
  setBar(playerGaugeFill, pg, 100);
  setBar(enemyGaugeFill, eg, 100);

  playerAtkMatk.textContent = `${snapshot.player.atk} / ${snapshot.player.matk}`;
  enemyAtkMatk.textContent = `${snapshot.enemy.atk} / ${snapshot.enemy.matk}`;
  playerDef.textContent = `${snapshot.player.base_def} -> ${snapshot.player.effective_def}`;
  enemyDef.textContent = `${snapshot.enemy.base_def} -> ${snapshot.enemy.effective_def}`;
  playerMdef.textContent = String(snapshot.player.mdef);
  enemyMdef.textContent = String(snapshot.enemy.mdef);
  playerCrit.textContent = `${critChancePercent(snapshot.player.crit_rate).toFixed(1)}%`;
  enemyIntent.textContent = snapshot.enemy_next_intent || "-";
  renderTraitList(playerTraits, snapshot.player_traits || []);
  renderTraitList(enemyTraits, snapshot.enemy_traits || []);
  renderStatusList(playerStatuses, snapshot.player.statuses);
  renderStatusList(enemyStatuses, snapshot.enemy.statuses);

  if (snapshot.run_state === "ended") {
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
    arenaHint.textContent = "Simulation ended.";
  }
}

function processStepResult(result) {
  appendEventLines(result.events);

  if (result.error) {
    setArenaStatus(`Error: ${result.error}`);
    arenaHint.textContent = "See console for the last emitted events.";
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
    stopLoop();
    return;
  }

  if (result.need_input) {
    uiMode = "need_input";
    setActionButtonsEnabled(true);
    setInputPrompt("Choose action");
    setArenaStatus("Input Required: Choose action");
    arenaHint.textContent = "Select a command from the action dock.";
    stopLoop();
    return;
  }

  if (result.ended) {
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
    stopLoop();
    return;
  }

  uiMode = "running";
}

function tickRun() {
  if (currentHandle === null || uiMode !== "running") {
    return;
  }

  const result = step_with_action(currentHandle, currentStepDt(), "none", -1);
  processStepResult(result);

  const snapshot = get_snapshot(currentHandle);
  updateHudFromSnapshot(snapshot);

  if (snapshot.run_state === "ended") {
    stopLoop();
  }
}

function startLoop() {
  if (loopTimer === null) {
    loopTimer = setInterval(tickRun, LOOP_MS);
  }
}

function toNumber(inputEl, fallback = 0) {
  const v = Number.parseFloat(inputEl.value);
  return Number.isFinite(v) ? v : fallback;
}

function readBuilderStats() {
  return {
    max_hp: toNumber(statInputs.max_hp, DEFAULT_BUILDER_STATS.max_hp),
    atk: Math.round(toNumber(statInputs.atk, DEFAULT_BUILDER_STATS.atk)),
    matk: Math.round(toNumber(statInputs.matk, DEFAULT_BUILDER_STATS.matk)),
    def: Math.round(toNumber(statInputs.def, DEFAULT_BUILDER_STATS.def)),
    mdef: Math.round(toNumber(statInputs.mdef, DEFAULT_BUILDER_STATS.mdef)),
    speed: toNumber(statInputs.speed, DEFAULT_BUILDER_STATS.speed),
    crit_rate: toNumber(statInputs.crit_rate, DEFAULT_BUILDER_STATS.crit_rate),
    crit_mult: toNumber(statInputs.crit_mult, DEFAULT_BUILDER_STATS.crit_mult),
  };
}

function writeBuilderStats(stats) {
  statInputs.max_hp.value = String(stats.max_hp);
  statInputs.atk.value = String(stats.atk);
  statInputs.matk.value = String(stats.matk);
  statInputs.def.value = String(stats.def);
  statInputs.mdef.value = String(stats.mdef);
  statInputs.speed.value = String(stats.speed.toFixed(2));
  statInputs.crit_rate.value = String(stats.crit_rate.toFixed(1));
  statInputs.crit_mult.value = String(stats.crit_mult.toFixed(2));
  refreshBudgetText(stats);
}

function setBuilderMode(mode) {
  builderMode = mode;
  const editable = mode === "manual";
  Object.values(statInputs).forEach((el) => {
    el.readOnly = !editable;
  });
}

function calcTotalCost(stats) {
  const hp_cost = Math.max(0, (stats.max_hp - 100) / 5);
  const atk_cost = stats.atk * 1.0;
  const matk_cost = stats.matk * 1.0;
  const def_cost = Math.max(0, stats.def) * 0.8;
  const mdef_cost = stats.mdef * 0.8;
  const speed_cost = Math.max(0, stats.speed - 1.0) * 60;
  const crit_rate_cost = stats.crit_rate / 10;
  const crit_mult_cost = Math.max(0, stats.crit_mult - 1.5) * 40;
  return hp_cost + atk_cost + matk_cost + def_cost + mdef_cost + speed_cost + crit_rate_cost + crit_mult_cost;
}

function calcSelectedBuilderTraitCost() {
  if (!selectedBuilderTraitIds.length || !selectableTraitIds.length) {
    return 0;
  }

  let total = 0;
  for (const traitId of selectedBuilderTraitIds) {
    const idx = selectableTraitIds.indexOf(traitId);
    if (idx >= 0) {
      total += Number(selectableTraitCosts[idx] || 0);
    }
  }
  return total;
}

function calcBuildCost(stats = readBuilderStats()) {
  const statsCost = calcTotalCost(stats);
  const traitsCost = calcSelectedBuilderTraitCost();
  const totalCost = statsCost + traitsCost;
  const remainingBudget = START_BUILD_BUDGET - totalCost;
  return {
    statsCost,
    traitsCost,
    totalCost,
    remainingBudget,
  };
}

function refreshBudgetText(stats = readBuilderStats()) {
  const summary = calcBuildCost(stats);
  builderStatsCostText.textContent = summary.statsCost.toFixed(1);
  builderTraitCostText.textContent = summary.traitsCost.toFixed(0);
  builderBudgetText.textContent = `${summary.totalCost.toFixed(1)} / ${START_BUILD_BUDGET.toFixed(1)}`;
  builderRemainingText.textContent = summary.remainingBudget.toFixed(1);
  const traitName = selectedBuilderTraitName();
  builderTraitHint.textContent = traitName
    ? `Selected traits: ${traitName}`
    : builderMode === "manual"
      ? "Select any starting traits that fit the remaining budget."
      : "Random mode rolls weighted traits that fit the remaining budget.";
}

function generateRandomStats() {
  const stats = { ...DEFAULT_BUILDER_STATS };
  let guard = 0;
  while (calcTotalCost(stats) < START_BUILD_BUDGET && guard < 2000) {
    guard += 1;
    const roll = Math.random();
    const candidate = { ...stats };

    if (roll < 0.2) candidate.max_hp += 5;
    else if (roll < 0.4) candidate.atk += 1;
    else if (roll < 0.6) candidate.matk += 1;
    else if (roll < 0.72) candidate.def += 1;
    else if (roll < 0.84) candidate.mdef += 1;
    else if (roll < 0.92) candidate.crit_rate += 5;
    else if (roll < 0.97) candidate.speed += 0.05;
    else candidate.crit_mult += 0.05;

    if (candidate.max_hp > STAT_RANGES.max_hp[1]) continue;
    if (candidate.atk > STAT_RANGES.atk[1]) continue;
    if (candidate.matk > STAT_RANGES.matk[1]) continue;
    if (candidate.def > STAT_RANGES.def[1]) continue;
    if (candidate.mdef > STAT_RANGES.mdef[1]) continue;
    if (candidate.speed > STAT_RANGES.speed[1]) continue;
    if (candidate.crit_rate > STAT_RANGES.crit_rate[1]) continue;
    if (candidate.crit_mult > STAT_RANGES.crit_mult[1]) continue;

    if (calcTotalCost(candidate) <= START_BUILD_BUDGET) {
      Object.assign(stats, candidate);
    }
  }

  stats.speed = Number(stats.speed.toFixed(2));
  stats.crit_rate = Number(stats.crit_rate.toFixed(1));
  stats.crit_mult = Number(stats.crit_mult.toFixed(2));
  return stats;
}

function validateStatsForConfirm(stats) {
  const errors = [];
  for (const [key, [min, max]] of Object.entries(STAT_RANGES)) {
    const value = stats[key];
    if (!Number.isFinite(value) || value < min || value > max) {
      errors.push(`${key} must be in ${min}..${max}`);
    }
  }

  const summary = calcBuildCost(stats);
  if (summary.totalCost > START_BUILD_BUDGET) {
    errors.push(`Budget exceeded: ${summary.totalCost.toFixed(1)} / ${START_BUILD_BUDGET.toFixed(1)}`);
  }

  return { ok: errors.length === 0, errors, ...summary };
}

function openBuilder() {
  builderOverlay.classList.remove("hidden");
  builderError.textContent = "";
  selectableTraitNames = get_selectable_trait_names();
  selectableTraitIds = get_selectable_trait_ids();
  selectableTraitCosts = get_selectable_trait_costs();
  builderModeRandom.checked = true;
  builderModeManual.checked = false;
  setBuilderMode("random");
  writeBuilderStats(generateRandomStats());
  sampleBuilderTraits();
  uiMode = "builder";
}

function startRunWithStats(stats) {
  stopLoop();

  if (currentHandle !== null) {
    destroy_run(currentHandle);
    currentHandle = null;
  }

  resetAll();

  const seed = Number.parseInt(seedInput.value, 10);
  const safeSeed = Number.isNaN(seed) ? 1234 : seed;
  currentHandle = create_run_with_stats(
    safeSeed,
    MAX_NODES,
    stats.max_hp,
    stats.atk,
    stats.matk,
    stats.def,
    stats.mdef,
    stats.speed,
    stats.crit_rate,
    stats.crit_mult,
  );
  if (selectedBuilderTraitIds[0]) {
    const ok = set_active_traits(currentHandle, selectedBuilderTraitIds.join(","));
    if (!ok) {
      setArenaStatus("Trait apply failed");
      return;
    }
  }

  const skills = get_player_skills(currentHandle);
  setCombatLabels(skills);
  setActionButtonsEnabled(false);
  setInputPrompt("");
  setArenaStatus("Simulation running");
  uiMode = "running";

  updateHudFromSnapshot(get_snapshot(currentHandle));
  startLoop();
}

function submitCombatAction(actionKind, actionArg) {
  if (currentHandle === null || uiMode !== "need_input") {
    return;
  }

  setActionButtonsEnabled(false);
  setInputPrompt("");

  const result = step_with_action(currentHandle, 0.0, actionKind, actionArg);
  processStepResult(result);
  updateHudFromSnapshot(get_snapshot(currentHandle));

  if (uiMode === "running") {
    startLoop();
  }
}

function onActionButton(index) {
  if (uiMode === "need_input") {
    if (index === 0) {
      submitCombatAction("basic", -1);
    } else {
      submitCombatAction("skill", index - 1);
    }
  }
}

actionBasicBtn.addEventListener("click", () => onActionButton(0));
actionSkillButtons.forEach((button, idx) => {
  button.addEventListener("click", () => onActionButton(idx + 1));
});

startBtn.addEventListener("click", () => {
  if (bootStatus.textContent !== "WASM ready") {
    return;
  }
  openBuilder();
});

resetBtn.addEventListener("click", () => {
  stopLoop();
  if (currentHandle !== null) {
    reset_run(currentHandle);
    destroy_run(currentHandle);
    currentHandle = null;
  }
  resetAll();
});

builderModeRandom.addEventListener("change", () => {
  if (!builderModeRandom.checked) return;
  setBuilderMode("random");
  writeBuilderStats(generateRandomStats());
  sampleBuilderTraits();
  builderError.textContent = "";
});

builderModeManual.addEventListener("change", () => {
  if (!builderModeManual.checked) return;
  setBuilderMode("manual");
  renderBuilderTraits();
  builderError.textContent = "";
  refreshBudgetText();
});

builderRandomBtn.addEventListener("click", () => {
  writeBuilderStats(generateRandomStats());
  sampleBuilderTraits();
  builderError.textContent = "";
});

builderRerollBtn.addEventListener("click", () => {
  if (builderMode === "random") {
    writeBuilderStats(generateRandomStats());
    sampleBuilderTraits();
  } else {
    renderBuilderTraits();
    refreshBudgetText();
  }
  builderError.textContent = "";
});

Object.values(statInputs).forEach((el) => {
  el.addEventListener("input", () => {
    renderBuilderTraits();
    refreshBudgetText();
  });
});

builderConfirmBtn.addEventListener("click", () => {
  const stats = readBuilderStats();
  const checked = validateStatsForConfirm(stats);
  builderStatsCostText.textContent = checked.statsCost.toFixed(1);
  builderTraitCostText.textContent = checked.traitsCost.toFixed(0);
  builderBudgetText.textContent = `${checked.totalCost.toFixed(1)} / ${START_BUILD_BUDGET.toFixed(1)}`;
  builderRemainingText.textContent = checked.remainingBudget.toFixed(1);

  if (!checked.ok) {
    builderError.textContent = checked.errors.join(" | ");
    return;
  }

  builderError.textContent = "";
  closeBuilder();
  startRunWithStats(stats);
});

builderCancelBtn.addEventListener("click", () => {
  closeBuilder();
  setArenaStatus("Ready");
});

async function boot() {
  await init();
  bootStatus.textContent = "WASM ready";
  console.log("sim started");

  const smoke = run_run(1234, 1);
  console.log("run_run smoke event count:", smoke.length);

  resetAll();
}

boot().catch((err) => {
  bootStatus.textContent = "WASM init failed";
  console.error("failed to initialize wasm:", err);
});
