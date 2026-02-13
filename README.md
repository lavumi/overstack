# Overstack: Rust WASM Core + Static Site

Rust + `wasm-pack` 기반 WASM 코어와 브라우저 정적 페이지 기본 구조입니다.

## 구조

```text
.
├── core
│   ├── Cargo.toml
│   └── src
│       ├── battle.rs
│       ├── event.rs
│       ├── lib.rs
│       ├── log.rs
│       ├── model.rs
│       ├── rng.rs
│       └── run.rs
└── site
    ├── index.html
    └── main.js
```

- `core`: WebAssembly로 빌드되는 Rust 라이브러리
- `site`: 브라우저에서 WASM을 불러 실행하는 정적 페이지

## Exported API

- `run_sim(seed, steps) -> u32`: 최소 샘플 시뮬레이션
- `run_run(seed, max_nodes) -> Vec<String>`: 한 판(run) 실행 이벤트 배열 반환 (각 원소는 이벤트 JSON 문자열)

`run_run`은 기본적으로 아래 순서로 진행됩니다.

1. 일반 전투 노드 5개
2. 보스 전투 노드 1개

각 전투는 게이지(`action_gauge`)가 100 이상인 유닛이 행동하며,
플레이어/적 모두 기본 공격만 자동으로 수행합니다.
전투 승리 시 임시 규칙으로 플레이어 최대 HP의 20%를 회복합니다.

## 1) WASM 빌드

사전 준비:

1. Rust 설치
2. `wasm-pack` 설치

```bash
cargo install wasm-pack
```

빌드 (`--target web`):

```bash
cd core
wasm-pack build --target web --out-dir ../site/pkg
```

위 명령으로 `site/pkg`에 JS/WASM 번들이 생성됩니다.

## 2) 정적 페이지 실행

브라우저에서 ES module + WASM 로딩을 위해 로컬 서버를 사용합니다.

```bash
cd site
python3 -m http.server
```

그 후 [http://localhost:8000](http://localhost:8000) 접속.

브라우저 화면의 로그 뷰어에서 구조화된 이벤트 기반 로그를 확인할 수 있습니다.

- `RunStart`
- `NodeStart`
- `BattleStart`
- `TurnReady`
- `ActionUsed`
- `DamageDealt`
- `StatusApplied`
- `StatusTick`
- `StatusExpired`
- `BattleEnd`
- `RunEnd`

## 한 번에 실행 (빌드 + 서버 실행)

아래 스크립트는 위 1~2단계를 한 번에 수행합니다.

```bash
./run_wasm_site.sh
```

- 기본 동작: 백그라운드 실행 (`site_server.pid`, `site_server.log` 생성)
- 포그라운드 실행: `./run_wasm_site.sh --fg`

## run_run 호출 예시 (Event JSON)

```js
import init, { run_run } from "./pkg/core.js";

await init();
const events = run_run(42, 6);
for (const eventJson of events) {
  const event = JSON.parse(eventJson);
  console.log(event.kind, event);
}
```
