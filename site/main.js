// wasm-pack output is loaded via relative path for GitHub Pages root compatibility.
import init, {
  create_run,
  destroy_run,
  get_active_traits,
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
const startBtn = document.getElementById("startBtn");
const resetBtn = document.getElementById("resetBtn");
const bootStatus = document.getElementById("bootStatus");
const logEl = document.getElementById("log");
const statusNode = document.getElementById("statusNode");
const statusBattle = document.getElementById("statusBattle");
const statusPlayerHp = document.getElementById("statusPlayerHp");
const statusEnemyHp = document.getElementById("statusEnemyHp");
const statusResult = document.getElementById("statusResult");
const statusTraits = document.getElementById("statusTraits");
const inputPrompt = document.getElementById("inputPrompt");

const actionBasicBtn = document.getElementById("actionBasic");
const actionSkillButtons = [
  document.getElementById("actionSkill1"),
  document.getElementById("actionSkill2"),
  document.getElementById("actionSkill3"),
  document.getElementById("actionSkill4"),
];

const STEP_DT = 0.15;
const LOOP_MS = 120;
const MAX_NODES = 6;
const MAX_LOG_LINES = 30;

let currentHandle = null;
let loopTimer = null;
let logLines = [];
let uiMode = "idle"; // idle | trait_select | running | need_input | ended
let selectableTraitIds = [];

function stopLoop() {
  if (loopTimer !== null) {
    clearInterval(loopTimer);
    loopTimer = null;
  }
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

function resetStatus() {
  statusNode.textContent = "-";
  statusBattle.textContent = "-";
  statusPlayerHp.textContent = "-";
  statusEnemyHp.textContent = "-";
  statusResult.textContent = "진행 중";
  statusTraits.textContent = "-";
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
      return `[DamageDealt] ${event.src} -> ${event.dst} dmg=${Number(event.amount).toFixed(2)} dst_hp=${Number(event.dst_hp_after).toFixed(2)}`;
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
      return `[TraitTriggered] ${event.trait_name} via ${event.trigger_type}`;
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

function appendEventLines(events) {
  if (events.length === 0) {
    return;
  }

  for (const line of events) {
    const event = parseEvent(line);
    const tickLabel = Number.isFinite(Number(event.tick))
      ? `t=${String(Math.trunc(Number(event.tick))).padStart(4, "0")}`
      : "t=----";
    logLines.push(`[${tickLabel}] ${formatEventLine(event)}`);
  }

  if (logLines.length > MAX_LOG_LINES) {
    logLines = logLines.slice(logLines.length - MAX_LOG_LINES);
  }

  logEl.textContent = `${logLines.join("\n")}${logLines.length > 0 ? "\n" : ""}`;
  logEl.scrollTop = logEl.scrollHeight;
}

function updateHudFromSnapshot(snapshot) {
  const playerHpInt = Math.round(snapshot.player.hp);
  const playerMaxHpInt = Math.round(snapshot.player.max_hp);
  const enemyHpInt = Math.round(snapshot.enemy.hp);
  const enemyMaxHpInt = Math.round(snapshot.enemy.max_hp);

  statusNode.textContent = String(snapshot.node_index);
  statusBattle.textContent = String(snapshot.battle_index);
  statusPlayerHp.textContent = `${playerHpInt}/${playerMaxHpInt} | ${snapshot.player.action_gauge.toFixed(1)}`;
  statusEnemyHp.textContent = `${enemyHpInt}/${enemyMaxHpInt} | ${snapshot.enemy.action_gauge.toFixed(1)}`;

  if (snapshot.run_state === "ended") {
    statusResult.textContent = snapshot.run_result === "win" ? "승리" : "패배";
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
  }
}

function processStepResult(result) {
  appendEventLines(result.events);

  if (result.error) {
    statusResult.textContent = `오류: ${result.error}`;
    uiMode = "ended";
    setActionButtonsEnabled(false);
    setInputPrompt("");
    stopLoop();
    return;
  }

  if (result.need_input) {
    statusResult.textContent = "입력 대기";
    uiMode = "need_input";
    setActionButtonsEnabled(true);
    setInputPrompt("Choose action");
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

  statusResult.textContent = "진행 중";
  uiMode = "running";
}

function tickRun() {
  if (currentHandle === null || uiMode !== "running") {
    return;
  }

  const result = step_with_action(currentHandle, STEP_DT, "none", -1);
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
  statusResult.textContent = "특성 선택";
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
    statusResult.textContent = "특성 선택 실패";
    return;
  }

  const activeTraits = get_active_traits(currentHandle);
  statusTraits.textContent = activeTraits.length > 0 ? activeTraits.join(", ") : "-";

  const skills = get_player_skills(currentHandle);
  setCombatLabels(skills);
  setActionButtonsEnabled(false);
  setInputPrompt("");

  statusResult.textContent = "진행 중";
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

  // Keep run_run alive for regression/debug use.
  const smoke = run_run(1234, 1);
  console.log("run_run smoke event count:", smoke.length);

  resetAll();
}

boot().catch((err) => {
  bootStatus.textContent = "WASM init failed";
  console.error("failed to initialize wasm:", err);
});
