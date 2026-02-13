// wasm-pack output is loaded via relative path for GitHub Pages root compatibility.
import init, { run_run } from "./pkg/core.js";

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

function resetStatus() {
  statusNode.textContent = "-";
  statusBattle.textContent = "-";
  statusPlayerHp.textContent = "-";
  statusEnemyHp.textContent = "-";
  statusResult.textContent = "진행 중";
}

function resetAll() {
  logEl.textContent = "";
  resetStatus();
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

function applyEventToHud(event) {
  switch (event.kind) {
    case "NodeStart":
      statusNode.textContent = String(event.node_index);
      break;
    case "BattleStart":
      statusBattle.textContent = String(event.battle_index);
      break;
    case "DamageDealt":
      if (event.dst === "player") {
        statusPlayerHp.textContent = String(event.dst_hp_after);
      } else if (event.dst === "enemy") {
        statusEnemyHp.textContent = String(event.dst_hp_after);
      }
      break;
    case "StatusTick":
      if (event.dst === "player") {
        statusPlayerHp.textContent = String(event.dst_hp_after);
      } else if (event.dst === "enemy") {
        statusEnemyHp.textContent = String(event.dst_hp_after);
      }
      break;
    case "BattleEnd":
      statusPlayerHp.textContent = String(event.player_hp_after);
      statusResult.textContent = event.result === "win" ? "승리" : "패배";
      break;
    case "RunEnd":
      statusResult.textContent = event.result === "win" ? "승리" : "패배";
      break;
    default:
      break;
  }
}

function parseEvent(line) {
  try {
    return JSON.parse(line);
  } catch (error) {
    return { kind: "InvalidJSON", raw: line, error: String(error) };
  }
}

startBtn.addEventListener("click", () => {
  resetAll();
  const seed = Number.parseInt(seedInput.value, 10);
  const safeSeed = Number.isNaN(seed) ? 1234 : seed;

  const eventLines = run_run(safeSeed, 6);
  const displayLines = [];

  for (const line of eventLines) {
    const event = parseEvent(line);
    applyEventToHud(event);
    displayLines.push(formatEventLine(event));
  }

  if (displayLines.length > 0) {
    logEl.textContent = `${displayLines.join("\n")}\n`;
    logEl.scrollTop = logEl.scrollHeight;
  }
});

resetBtn.addEventListener("click", () => {
  resetAll();
});

async function boot() {
  await init();
  bootStatus.textContent = "WASM ready";
  console.log("sim started");
  resetAll();
}

boot().catch((err) => {
  bootStatus.textContent = "WASM init failed";
  console.error("failed to initialize wasm:", err);
});
