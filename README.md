# Overstack

Rust + `wasm-pack` 기반의 전투 시뮬레이션 코어와, 이를 브라우저에서 실행하는 정적 사이트 프로젝트입니다.

핵심 방향은 다음 두 가지입니다.

1. 게임 규칙과 데이터는 Rust/WASM 코어에 둔다.
2. UI는 상태 표시와 입력 전달에 집중한다.

데이터 파일(`core/data/*.json`)은 런타임 fetch 없이 `include_str!`로 WASM 빌드 시점에 임베딩됩니다.

## 현재 구조

```text
.
├── core
│   ├── Cargo.toml
│   ├── data
│   │   ├── enemies.json
│   │   ├── schema.md
│   │   ├── skills.json
│   │   └── traits.json
│   └── src
│       ├── battle.rs
│       ├── combat_math.rs
│       ├── data
│       │   ├── compile.rs
│       │   ├── defs.rs
│       │   ├── errors.rs
│       │   ├── mod.rs
│       │   ├── registry.rs
│       │   ├── specs.rs
│       │   └── validate.rs
│       ├── engine
│       │   ├── combat_state.rs
│       │   ├── mod.rs
│       │   ├── numeric.rs
│       │   ├── run_flow.rs
│       │   ├── skill_exec.rs
│       │   ├── snapshot.rs
│       │   ├── status_system.rs
│       │   ├── trait_system.rs
│       │   └── turn_system.rs
│       ├── event.rs
│       ├── lib.rs
│       ├── log.rs
│       ├── model.rs
│       ├── rng.rs
│       ├── run.rs
│       ├── skill.rs
│       ├── step_api
│       │   └── manager.rs
│       ├── step_api.rs
│       └── trait_spec.rs
├── run_wasm_site.sh
└── site
    ├── index.html
    └── main.js
```

- `core`: WebAssembly로 빌드되는 게임 코어
- `site`: 코어를 불러와 실행하는 브라우저 UI

## 코어 아키텍처

### 1. 데이터 계층

`core/src/data`는 JSON 게임 데이터를 로드하고 런타임 registry로 컴파일합니다.

- `defs.rs`: serde 역직렬화용 JSON Def 구조체
- `validate.rs`: 다중 에러 수집 validator
- `compile.rs`: Def -> enum 기반 `Spec` 컴파일
- `registry.rs`: `OnceLock` 기반 1회 로드/캐시
- `errors.rs`: `DataError`, `ErrorReport`
- `specs.rs`: 런타임에서 쓰는 `SkillSpec`, `TraitSpec`, `EnemySpec`

검증 실패 시 에러는 path 포함으로 누적되며, step API에서는 `data_load_failed: ...` 형태로 노출됩니다.

### 2. 규칙/정책 계층

- `skill.rs`: 스킬 조회와 플레이어 기본 로드아웃 접근
- `trait_spec.rs`: trait 조회, selectable trait 목록, cost 합산, cost 기반 weight 계산, reward용 비복원 샘플링
- `combat_math.rs`: 피해 계산, 방어력 보정, 치명타 계산
- `engine/numeric.rs`: HP 반올림, status tick 상수, 표시용 duration 변환 같은 숫자 정책

### 3. 실행 계층

`core/src/engine`은 실제 step 기반 전투 실행을 담당합니다.

- `run_flow.rs`: `ActiveRun` 생성/리셋, trait 적용, battle start
- `combat_state.rs`: battle/runtime 상태 접근 유틸
- `skill_exec.rs`: 스킬 선택과 effect 실행
- `status_system.rs`: 상태이상 적용, tick, 만료, 전투 종료 처리
- `trait_system.rs`: trait trigger/effect 처리
- `turn_system.rs`: step 루프, 게이지 진행, 입력 대기, actor turn 진행
- `snapshot.rs`: HUD용 snapshot 조립

현재 리팩터링 방향은 `step_api.rs`를 점점 얇게 만들고, 실제 규칙 실행을 `engine/*`로 모으는 것입니다.

### 4. API/WASM 계층

- `lib.rs`: 공개 엔트리 (`run_sim`, `run_run`)
- `step_api.rs`: WASM export 타입과 step API 진입점
- `step_api/manager.rs`: run handle 저장/조회

## 현재 게임 시스템 요약

### 전투

- 전투는 게이지 기반으로 진행됩니다.
- `action_gauge >= 100`인 유닛이 행동합니다.
- 플레이어는 기본 공격 또는 슬롯 스킬을 사용할 수 있습니다.
- 적은 현재 기본 공격만 사용합니다.
- 승리 시 임시 규칙으로 플레이어 최대 HP의 20%를 회복합니다.

### 스킬

플레이어 기본 슬롯 매핑:

1. slot0: `Ember Lash`
2. slot1: `Frost Bite`
3. slot2: `Arc Jolt`
4. slot3: `Ruin Strike`

스킬 효과는 데이터 기반이며, 대표적으로 다음을 지원합니다.

- 직접 피해
- 상태이상 부여
- 조건부 추가 피해
- 조건부 상태이상
- 자기 버프
- proc/res 보정
- 상태이상 power 수정

### Trait

Trait은 이제 `rarity_weight` 없이 `cost`만 가집니다.

- `TraitSpec.cost`에 저장됩니다.
- 보상 등장 weight는 `cost`에서 자동 계산합니다.
- 공식:
  `trait_weight(cost) = TRAIT_WEIGHT_BASE / cost^TRAIT_WEIGHT_P`
- 현재 기본 상수:
  `TRAIT_WEIGHT_BASE = 100.0`
  `TRAIT_WEIGHT_P = 1.3`

`cost`는 다음 두 용도를 공통으로 지원하도록 설계되어 있습니다.

1. 시작 캐릭터 생성 시 선택 trait 총 비용 계산
2. 추후 보상 시스템의 등장 확률 계산

현재 준비된 trait 유틸:

- 시작 선택용 selectable trait 조회
- 선택 trait 비용 합산
- owned trait 제외
- 중복 없는 trait 샘플링

### 상태이상

현재 주요 상태이상:

- `Burn`
- `Freeze`
- `Shock`
- `Break`
- `Bleed`
- `Stun`
- `Might`
- `Haste`

`status_system.rs`가 apply/tick/expire 전 과정을 맡습니다.

## Step API

현재 UI가 주로 사용하는 흐름은 아래와 같습니다.

1. `create_run(seed, max_nodes)` 또는 `create_run_with_stats(...)`
2. 시작 trait 목록은 `get_selectable_trait_names()` / `get_selectable_trait_ids()`로 조회
3. 시작 trait 1개를 `set_active_trait(handle, trait_id)`로 적용
4. 루프에서 `step_with_action(handle, dt, "none", -1)` 호출
5. `StepResult.need_input === true`면 `"basic"` 또는 `"skill"` 입력 전달
6. 매 루프마다 `get_snapshot(handle)`로 HUD 갱신
7. 필요 시 `reset_run(handle)` / `destroy_run(handle)`

### Exported API

- `run_sim(seed, steps) -> u32`
- `run_run(seed, max_nodes) -> Vec<String>`
- `create_run(seed, max_nodes) -> u32`
- `create_run_with_stats(seed, max_nodes, ...) -> u32`
- `step(handle, dt, player_action?) -> StepResult`
- `step_with_action(handle, dt, action_kind, action_arg) -> StepResult`
- `get_snapshot(handle) -> Snapshot`
- `get_player_skills(handle) -> Vec<String>`
- `get_active_traits(handle) -> Vec<String>`
- `get_selectable_trait_names() -> Vec<String>`
- `get_selectable_trait_ids() -> Vec<String>`
- `set_active_trait(handle, trait_id) -> bool`
- `reset_run(handle) -> bool`
- `destroy_run(handle)`

## 이벤트 로그

브라우저 로그 뷰어에서 아래 구조화 이벤트를 확인할 수 있습니다.

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
- `TraitTriggered`
- `TraitEffectApplied`

## 캐릭터 생성기

현재 정적 사이트에는 pre-run builder가 들어가 있습니다.

- random/manual 스탯 설정
- 스탯 budget 검사
- `create_run_with_stats(...)`로 초기 플레이어 스탯 전달

trait 선택 UI는 아직 시작 시 1개 선택 방식이지만, 엔진 쪽에는 trait 비용 계산과 reward 샘플링용 유틸이 준비되어 있습니다.

## 빌드

사전 준비:

1. Rust 설치
2. `wasm-pack` 설치

```bash
cargo install wasm-pack
```

빌드:

```bash
cd core
wasm-pack build --target web --out-dir ../site/pkg
```

## 실행

정적 서버 실행:

```bash
cd site
python3 -m http.server
```

브라우저에서 [http://localhost:8000](http://localhost:8000) 접속.

한 번에 빌드 + 서버 실행:

```bash
./run_wasm_site.sh
```

- 기본: 백그라운드 실행
- 포그라운드: `./run_wasm_site.sh --fg`

## 데이터 수정 방법

1. `core/data/skills.json`, `core/data/traits.json`, `core/data/enemies.json` 수정
2. 필드 규칙은 `core/data/schema.md` 참고
3. `cargo test` 또는 `wasm-pack build --target web --out-dir ../site/pkg`
4. `site`에서 동작과 로그 확인

Trait 데이터 작성 시 주의:

- `cost`는 필수
- `cost >= 1`
- selectable trait / enemy trait pool은 `pool`과 registry를 통해 결정

## 테스트

```bash
cd core
cargo test
```

현재 테스트는 대략 다음 영역을 커버합니다.

- 데이터 검증
- trait cost/weight/sampling
- 전투 피해 계산
- step API 기반 trait trigger
- chain-depth guard
- enemy trait 부여
