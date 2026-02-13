// wasm-pack output is loaded via relative path for GitHub Pages root compatibility.
import init, {
  create_run,
  destroy_run,
  get_snapshot,
  reset_run,
  run_run,
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
let waitingForInput = false;
let logLines = [];

function stopLoop() {
  if (loopTimer !== null) {
    clearInterval(loopTimer);
    loopTimer = null;
  }
}

function setInputWaitingState(waiting) {
  waitingForInput = waiting;

  actionBasicBtn.disabled = !waiting;
  for (const button of actionSkillButtons) {
    button.disabled = !waiting;
  }

  inputPrompt.textContent = waiting ? "Choose action" : "";
}

function resetStatus() {
  statusNode.textContent = "-";
  statusBattle.textContent = "-";
  statusPlayerHp.textContent = "-";
  statusEnemyHp.textContent = "-";
  statusResult.textContent = "진행 중";
}

function resetAll() {
  logLines = [];
  logEl.textContent = "";
  resetStatus();
  setInputWaitingState(false);
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
      return `[DamageDealt] ${event.src} -> ${event.dst} dmg=${event.amount} dst_hp=${event.dst_hp_after}`;
    case "StatusApplied":
      return `[StatusApplied] ${event.src} -> ${event.dst} ${event.status} stacks=${event.stacks} duration=${event.duration}`;
    case "StatusTick":
      return `[StatusTick] ${event.dst} ${event.status} amount=${event.amount} hp=${event.dst_hp_after}`;
    case "StatusExpired":
      return `[StatusExpired] ${event.dst} ${event.status}`;
    case "BattleEnd":
      return `[BattleEnd] result=${event.result} player_hp=${event.player_hp_after}`;
    case "RunEnd":
      return `[RunEnd] result=${event.result} final_node=${event.final_node_index}`;
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
    logLines.push(formatEventLine(event));
  }

  if (logLines.length > MAX_LOG_LINES) {
    logLines = logLines.slice(logLines.length - MAX_LOG_LINES);
  }

  logEl.textContent = `${logLines.join("\n")}${logLines.length > 0 ? "\n" : ""}`;
  logEl.scrollTop = logEl.scrollHeight;
}

function updateHudFromSnapshot(snapshot) {
  statusNode.textContent = String(snapshot.node_index);
  statusBattle.textContent = String(snapshot.battle_index);
  statusPlayerHp.textContent = `${snapshot.player.hp}/${snapshot.player.max_hp} | ${snapshot.player.action_gauge.toFixed(1)}`;
  statusEnemyHp.textContent = `${snapshot.enemy.hp}/${snapshot.enemy.max_hp} | ${snapshot.enemy.action_gauge.toFixed(1)}`;

  if (snapshot.run_state === "ended") {
    statusResult.textContent = snapshot.run_result === "win" ? "승리" : "패배";
    setInputWaitingState(false);
  }
}

function processStepResult(result) {
  appendEventLines(result.events);

  if (result.error) {
    statusResult.textContent = `오류: ${result.error}`;
    setInputWaitingState(false);
    stopLoop();
    return;
  }

  if (result.need_input) {
    statusResult.textContent = "입력 대기";
    setInputWaitingState(true);
    stopLoop();
    return;
  }

  if (result.ended) {
    setInputWaitingState(false);
    stopLoop();
    return;
  }

  statusResult.textContent = "진행 중";
}

function tickRun() {
  if (currentHandle === null || waitingForInput) {
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
  updateHudFromSnapshot(get_snapshot(currentHandle));
  startLoop();
}

function submitAction(actionKind, actionArg) {
  if (currentHandle === null || !waitingForInput) {
    return;
  }

  setInputWaitingState(false);

  const result = step_with_action(currentHandle, 0.0, actionKind, actionArg);
  processStepResult(result);
  updateHudFromSnapshot(get_snapshot(currentHandle));

  if (!waitingForInput) {
    startLoop();
  }
}

actionBasicBtn.addEventListener("click", () => {
  submitAction("basic", -1);
});

actionSkillButtons.forEach((button, idx) => {
  button.addEventListener("click", () => {
    submitAction("skill", idx);
  });
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
