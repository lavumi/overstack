// wasm-pack output is loaded via relative path for GitHub Pages root compatibility.
import init, {
  create_run,
  destroy_run,
  get_player_skills,
  get_selectable_trait_ids,
  get_selectable_trait_names,
  get_snapshot,
  reset_run,
  run_run,
  set_active_trait,
  step_with_action,
} from "./pkg/core.js";

const seedInput = document.getElementById("seedInput");
const speedSelect = document.getElementById("speedSelect");
const startBtn = document.getElementById("startBtn");
const resetBtn = document.getElementById("resetBtn");
const bootStatus = document.getElementById("bootStatus");
const logEl = document.getElementById("log");
const arenaStatus = document.getElementById("arenaStatus");

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

const inputPrompt = document.getElementById("inputPrompt");
const actionBasicBtn = document.getElementById("actionBasic");
const actionSkillButtons = [
  document.getElementById("actionSkill1"),
  document.getElementById("actionSkill2"),
  document.getElementById("actionSkill3"),
  document.getElementById("actionSkill4"),
];

const STEP_DT_BASE = 0.15;
const LOOP_MS = 120;
const MAX_NODES = 6;
const MAX_LOG_LINES = 2000;
const CRIT_C = 100;

let currentHandle = null;
let loopTimer = null;
let logLines = [];
let uiMode = "idle"; // idle | trait_select | running | need_input | ended
let selectableTraitIds = [];
let lastEnemyName = "Enemy";

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
  arenaStatus.textContent = text;
}

function setCombatLabels(skillNames) {
  actionBasicBtn.textContent = "Basic Attack";
  for (let i = 0; i < actionSkillButtons.length; i += 1) {
    actionSkillButtons[i].textContent = skillNames[i] || `Skill ${i + 1}`;
  }
}

function setTraitLabels(traitNames) {
  const labels = [traitNames[0], traitNames[1], traitNames[2], traitNames[3], traitNames[4]];
  actionBasicBtn.textContent = labels[0] || "Trait 1";
  for (let i = 0; i < actionSkillButtons.length; i += 1) {
    actionSkillButtons[i].textContent = labels[i + 1] || `Trait ${i + 2}`;
  }
}

function clampPct(v) {
  return Math.max(0, Math.min(100, v));
}

function setBar(fillEl, current, max) {
  const pct = max > 0 ? clampPct((current / max) * 100) : 0;
  fillEl.style.width = `${pct.toFixed(1)}%`;
}

function fmtStatuses(statuses) {
  if (!statuses || statuses.length === 0) {
    return ["None"];
  }
  return statuses.map((s) => `${s.status_type} x${s.stacks} (${Number(s.duration).toFixed(1)}s)`);
}

function renderStatusList(listEl, statuses) {
  const lines = fmtStatuses(statuses);
  listEl.innerHTML = "";
  const frag = document.createDocumentFragment();
  for (const line of lines) {
    const li = document.createElement("li");
    li.textContent = line;
    frag.appendChild(li);
  }
  listEl.appendChild(frag);
}

function resetStatus() {
  statusNode.textContent = "-";
  statusBattle.textContent = "-";
  playerTraits.textContent = "-";
  enemyTraits.textContent = "-";
  playerName.textContent = "Player";
  enemyName.textContent = "Enemy";
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
  setBar(playerHpFill, 0, 1);
  setBar(enemyHpFill, 0, 1);
  setBar(playerGaugeFill, 0, 100);
  setBar(enemyGaugeFill, 0, 100);
  setArenaStatus("Ready");
  lastEnemyName = "Enemy";
}

function resetAll() {
  logLines = [];
  logEl.textContent = "";
  resetStatus();
  setCombatLabels([]);
  setActionButtonsEnabled(false);
  setInputPrompt("");
  selectableTraitIds = [];
  uiMode = "idle";
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

function parseEvent(line) {
  try {
    return JSON.parse(line);
  } catch (error) {
    return { kind: "InvalidJSON", raw: line, error: String(error) };
  }
}

function updateArenaStatusByEvent(event) {
  if (event.kind === "BattleStart") {
    lastEnemyName = event.enemy_name || "Enemy";
    enemyName.textContent = lastEnemyName;
    setArenaStatus(`Encounter: ${lastEnemyName}`);
    return;
  }
  if (event.kind === "TurnReady") {
    setArenaStatus(event.actor === "player" ? "Player Turn" : "Enemy Turn");
    return;
  }
  if (event.kind === "ActionUsed") {
    const actor = event.actor === "player" ? "Player" : "Enemy";
    setArenaStatus(`${actor} uses ${event.action_name}`);
    return;
  }
  if (event.kind === "BattleEnd") {
    setArenaStatus(event.result === "win" ? "Battle Won" : "Battle Lost");
    return;
  }
  if (event.kind === "RunEnd") {
    setArenaStatus(event.result === "win" ? "Run Complete" : "Run Failed");
  }
}

function appendEventLines(events) {
  if (events.length === 0) {
    return;
  }

  const fragLines = [];
  for (const line of events) {
    const event = parseEvent(line);
    updateArenaStatusByEvent(event);
    const tickLabel = Number.isFinite(Number(event.tick))
      ? `t=${String(Math.trunc(Number(event.tick))).padStart(4, "0")}`
      : "t=----";
    fragLines.push(`[${tickLabel}] ${formatEventLine(event)}`);
  }

  logLines.push(...fragLines);
  if (logLines.length > MAX_LOG_LINES) {
    logLines = logLines.slice(logLines.length - MAX_LOG_LINES);
  }

  logEl.textContent = `${logLines.join("\n")}${logLines.length > 0 ? "\n" : ""}`;
  logEl.scrollTop = logEl.scrollHeight;
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
  playerTraits.textContent = (snapshot.player_traits || []).join(", ") || "-";
  enemyTraits.textContent = (snapshot.enemy_traits || []).join(", ") || "-";

  renderStatusList(playerStatuses, snapshot.player.statuses);
  renderStatusList(enemyStatuses, snapshot.enemy.statuses);

  if (snapshot.run_state === "ended") {
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
  }
}

function processStepResult(result) {
  appendEventLines(result.events);

  if (result.error) {
    setArenaStatus(`Error: ${result.error}`);
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

function startRun() {
  stopLoop();

  if (currentHandle !== null) {
    destroy_run(currentHandle);
    currentHandle = null;
  }

  resetAll();

  const seed = Number.parseInt(seedInput.value, 10);
  const safeSeed = Number.isNaN(seed) ? 1234 : seed;
  currentHandle = create_run(safeSeed, MAX_NODES);

  const traitNames = get_selectable_trait_names();
  selectableTraitIds = get_selectable_trait_ids();
  setTraitLabels(traitNames);
  setActionButtonsEnabled(true);
  setInputPrompt("Choose one trait");
  setArenaStatus("Choose a starting trait");
  uiMode = "trait_select";

  updateHudFromSnapshot(get_snapshot(currentHandle));
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

function chooseTraitByButtonIndex(index) {
  if (currentHandle === null || uiMode !== "trait_select") {
    return;
  }

  const traitId = selectableTraitIds[index];
  if (!traitId) {
    return;
  }

  const ok = set_active_trait(currentHandle, traitId);
  if (!ok) {
    setArenaStatus("Trait select failed");
    return;
  }

  const skills = get_player_skills(currentHandle);
  setCombatLabels(skills);
  setActionButtonsEnabled(false);
  setInputPrompt("");
  setArenaStatus("Simulation running");
  updateHudFromSnapshot(get_snapshot(currentHandle));

  uiMode = "running";
  startLoop();
}

function onActionButton(index) {
  if (uiMode === "trait_select") {
    chooseTraitByButtonIndex(index);
    return;
  }

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
  startRun();
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
