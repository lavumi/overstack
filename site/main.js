// wasm-pack build 결과물(site/pkg/core.js)을 로드합니다.
import init, { run_run, run_sim } from "./pkg/core.js";

async function boot() {
  // WASM 모듈 초기화.
  await init();

  console.log("sim started");

  // 기존 최소 함수도 유지.
  const simValue = run_sim(1234, 10);
  console.log("sim result:", simValue);

  // 한 판 실행 엔트리.
  const cleared = run_run(42, 6);
  console.log("run finished, cleared nodes:", cleared);
}

boot().catch((err) => {
  console.error("failed to initialize wasm:", err);
});
