import { createScreenController } from "./ui/screens.js";
import { createBuilderUI } from "./ui/builder.js";
import { createHud } from "./ui/hud.js";
import { createWasmClient } from "./wasm/client.js";

const seedInput = document.getElementById("seedInput");
const speedSelect = document.getElementById("speedSelect");
const resetBtn = document.getElementById("resetBtn");

const STEP_DT_BASE = 0.15;
const LOOP_MS = 120;
const MAX_NODES = 6;
const CRIT_C = 100;
const START_BUILD_BUDGET = 100;
const RANDOM_BUILD_BUDGET = 120;

const DEFAULT_BUILDER_STATS = {
  max_hp: 130,
  atk: 30,
  matk: 25,
  def: 20,
  mdef: 15,
  speed: 35.0,
  crit_rate: 50,
  crit_mult: 1.5,
};

const STAT_RANGES = {
  max_hp: [80, 200],
  atk: [10, 60],
  matk: [10, 60],
  def: [-20, 80],
  mdef: [0, 80],
  speed: [20, 50],
  crit_rate: [0, 300],
  crit_mult: [1.25, 2.5],
};

let currentHandle = null;
let loopTimer = null;
let uiMode = "idle"; // idle | builder | running | need_input | ended
let wasmReady = false;
const wasm = createWasmClient();

const screens = createScreenController({
  landingView: document.getElementById("landingView"),
  builderView: document.getElementById("builderOverlay"),
  gameShell: document.getElementById("gameShell"),
  startBtn: document.getElementById("startBtn"),
  bootStatus: document.getElementById("bootStatus"),
});

const hud = createHud(
  {
    arenaStatus: document.getElementById("arenaStatus"),
    arenaHint: document.getElementById("arenaHint"),
    statusNode: document.getElementById("statusNode"),
    statusBattle: document.getElementById("statusBattle"),
    playerTraits: document.getElementById("playerTraits"),
    enemyTraits: document.getElementById("enemyTraits"),
    playerName: document.getElementById("playerName"),
    playerHpText: document.getElementById("playerHpText"),
    playerHpFill: document.getElementById("playerHpFill"),
    playerGaugeText: document.getElementById("playerGaugeText"),
    playerGaugeFill: document.getElementById("playerGaugeFill"),
    playerAtkMatk: document.getElementById("playerAtkMatk"),
    playerDef: document.getElementById("playerDef"),
    playerMdef: document.getElementById("playerMdef"),
    playerCrit: document.getElementById("playerCrit"),
    playerStatuses: document.getElementById("playerStatuses"),
    enemyName: document.getElementById("enemyName"),
    enemyHpText: document.getElementById("enemyHpText"),
    enemyHpFill: document.getElementById("enemyHpFill"),
    enemyGaugeText: document.getElementById("enemyGaugeText"),
    enemyGaugeFill: document.getElementById("enemyGaugeFill"),
    enemyAtkMatk: document.getElementById("enemyAtkMatk"),
    enemyDef: document.getElementById("enemyDef"),
    enemyMdef: document.getElementById("enemyMdef"),
    enemyIntent: document.getElementById("enemyIntent"),
    enemyStatuses: document.getElementById("enemyStatuses"),
    stagePlayer: document.getElementById("stagePlayer"),
    stageEnemy: document.getElementById("stageEnemy"),
    stagePlayerName: document.getElementById("stagePlayerName"),
    stageEnemyName: document.getElementById("stageEnemyName"),
    stagePlayerVitals: document.getElementById("stagePlayerVitals"),
    stageEnemyVitals: document.getElementById("stageEnemyVitals"),
    stagePlayerState: document.getElementById("stagePlayerState"),
    stageEnemyState: document.getElementById("stageEnemyState"),
    inputPrompt: document.getElementById("inputPrompt"),
    actionBasicBtn: document.getElementById("actionBasic"),
    actionSkillButtons: [
      document.getElementById("actionSkill1"),
      document.getElementById("actionSkill2"),
      document.getElementById("actionSkill3"),
      document.getElementById("actionSkill4"),
    ],
  },
  { critConstant: CRIT_C },
);

const builder = createBuilderUI(
  {
    builderModeRandom: document.getElementById("builderModeRandom"),
    builderModeManual: document.getElementById("builderModeManual"),
    builderBudgetText: document.getElementById("builderBudgetText"),
    builderStatsCostText: document.getElementById("builderStatsCostText"),
    builderTraitCostText: document.getElementById("builderTraitCostText"),
    builderRemainingText: document.getElementById("builderRemainingText"),
    builderTraitHint: document.getElementById("builderTraitHint"),
    builderTraitChoices: document.getElementById("builderTraitChoices"),
    builderError: document.getElementById("builderError"),
    builderRandomBtn: document.getElementById("builderRandomBtn"),
    builderRerollBtn: document.getElementById("builderRerollBtn"),
    builderConfirmBtn: document.getElementById("builderConfirmBtn"),
    builderCancelBtn: document.getElementById("builderCancelBtn"),
    statInputs: {
      max_hp: document.getElementById("statMaxHp"),
      atk: document.getElementById("statAtk"),
      matk: document.getElementById("statMatk"),
      def: document.getElementById("statDef"),
      mdef: document.getElementById("statMdef"),
      speed: document.getElementById("statSpeed"),
      crit_rate: document.getElementById("statCritRate"),
      crit_mult: document.getElementById("statCritMult"),
    },
  },
  {
    startBuildBudget: START_BUILD_BUDGET,
    randomBuildBudget: RANDOM_BUILD_BUDGET,
    defaultStats: DEFAULT_BUILDER_STATS,
    statRanges: STAT_RANGES,
    randomSeed32,
    sampleTraitIds: wasm.sampleStartingTraitIds,
  },
);

function randomSeed32() {
  return Math.floor(Math.random() * 0x100000000) >>> 0;
}

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

function resetUiState() {
  hud.reset();
}

function resetAll() {
  resetUiState();
  builder.reset();
  screens.showLanding();
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
    hud.setEnemyName(event.enemy_name || "Enemy");
    hud.setArenaStatus(`Encounter: ${event.enemy_name || "Enemy"}`);
    hud.setArenaHint("New hostile pattern detected.");
    return;
  }
  if (event.kind === "TurnReady") {
    hud.setArenaStatus(event.actor === "player" ? "Player Turn" : "Enemy Turn");
    hud.setArenaHint(event.actor === "player" ? "Action window open." : "Brace for incoming action.");
    return;
  }
  if (event.kind === "ActionUsed") {
    const actor = event.actor === "player" ? "Player" : "Enemy";
    hud.setArenaStatus(`${actor} uses ${event.action_name}`);
    hud.setArenaHint(`${actor} committed ${event.action_name}.`);
    return;
  }
  if (event.kind === "BattleEnd") {
    hud.setArenaStatus(event.result === "win" ? "Battle Won" : "Battle Lost");
    hud.setArenaHint(event.result === "win" ? "Arena secured. Prepare the next node." : "Recovery required.");
    return;
  }
  if (event.kind === "RunEnd") {
    hud.setArenaStatus(event.result === "win" ? "Run Complete" : "Run Failed");
    hud.setArenaHint(event.result === "win" ? "Simulation complete." : "Run terminated.");
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

function processStepResult(result) {
  appendEventLines(result.events);

  if (result.error) {
    hud.setArenaStatus(`Error: ${result.error}`);
    hud.setArenaHint("See console for the last emitted events.");
    uiMode = "ended";
    hud.setActionButtonsEnabled(false);
    hud.setInputPrompt("");
    stopLoop();
    return;
  }

  if (result.need_input) {
    uiMode = "need_input";
    hud.setActionButtonsEnabled(true);
    hud.setInputPrompt("Choose action");
    hud.setArenaStatus("Input Required: Choose action");
    hud.setArenaHint("Select a command from the action dock.");
    stopLoop();
    return;
  }

  if (result.ended) {
    uiMode = "ended";
    hud.setActionButtonsEnabled(false);
    hud.setInputPrompt("");
    stopLoop();
    return;
  }

  uiMode = "running";
}

function tickRun() {
  if (currentHandle === null || uiMode !== "running") {
    return;
  }

  const result = wasm.step(currentHandle, currentStepDt(), "none", -1);
  processStepResult(result);

  const snapshot = wasm.getSnapshot(currentHandle);
  hud.updateFromSnapshot(snapshot);

  if (snapshot.run_state === "ended") {
    stopLoop();
  }
}

function startLoop() {
  if (loopTimer === null) {
    loopTimer = setInterval(tickRun, LOOP_MS);
  }
}

function openBuilder() {
  screens.showBuilder();
  builder.open(wasm.getSelectableTraits());
  uiMode = "builder";
}

function startRunWithBuild({ stats, traitIds }) {
  stopLoop();

  if (currentHandle !== null) {
    wasm.destroyRun(currentHandle);
    currentHandle = null;
  }

  resetUiState();
  screens.showGame();

  const seed = Number.parseInt(seedInput.value, 10);
  const safeSeed = Number.isNaN(seed) ? 1234 : seed;
  currentHandle = wasm.createRunWithStats(safeSeed, MAX_NODES, stats);

  if (traitIds[0]) {
    const ok = wasm.setActiveTraits(currentHandle, traitIds);
    if (!ok) {
      hud.setArenaStatus("Trait apply failed");
      screens.showBuilder();
      uiMode = "builder";
      return;
    }
  }

  const skills = wasm.getPlayerSkills(currentHandle);
  hud.setCombatLabels(skills);
  hud.setActionButtonsEnabled(false);
  hud.setInputPrompt("");
  hud.setArenaStatus("Simulation running");
  uiMode = "running";

  hud.updateFromSnapshot(wasm.getSnapshot(currentHandle));
  startLoop();
}

function submitCombatAction(actionKind, actionArg) {
  if (currentHandle === null || uiMode !== "need_input") {
    return;
  }

  hud.setActionButtonsEnabled(false);
  hud.setInputPrompt("");

  const result = wasm.step(currentHandle, 0.0, actionKind, actionArg);
  processStepResult(result);
  hud.updateFromSnapshot(wasm.getSnapshot(currentHandle));

  if (uiMode === "running") {
    startLoop();
  }
}

function onActionButton(index) {
  if (uiMode !== "need_input") {
    return;
  }
  if (index === 0) {
    submitCombatAction("basic", -1);
  } else {
    submitCombatAction("skill", index - 1);
  }
}

hud.onAction(onActionButton);

screens.onStart(() => {
  if (!wasmReady) {
    return;
  }
  openBuilder();
});

builder.onConfirm(startRunWithBuild);
builder.onCancel(() => {
  screens.showLanding();
  hud.setArenaStatus("Ready");
  uiMode = "idle";
});

resetBtn.addEventListener("click", () => {
  stopLoop();
  if (currentHandle !== null) {
    wasm.resetRun(currentHandle);
    wasm.destroyRun(currentHandle);
    currentHandle = null;
  }
  resetAll();
});

async function boot() {
  await wasm.boot();
  wasmReady = true;
  screens.setBootStatus("WASM ready");
  screens.setStartEnabled(true);
  console.log("sim started");

  const smoke = wasm.runSmoke(1234, 1);
  console.log("run_run smoke event count:", smoke.length);

  resetAll();
}

boot().catch((err) => {
  wasmReady = false;
  screens.setBootStatus("WASM init failed");
  screens.setStartEnabled(false);
  console.error("failed to initialize wasm:", err);
});
