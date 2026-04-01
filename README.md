# Overstack

Rust + `wasm-bindgen` 기반의 전투 시뮬레이션 코어와, 이를 브라우저에서 실행하는 정적 사이트 프로젝트입니다.

핵심 방향은 다음 두 가지입니다.

1. 게임 규칙과 데이터는 Rust/WASM 코어에 둔다.
2. UI는 상태 표시와 입력 전달에 집중한다.

데이터 파일(`core/data/*.json`)은 런타임 fetch 없이 `include_str!`로 WASM 빌드 시점에 임베딩됩니다.

## 현재 구조

```text
.
├── build.sh
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
└── site
    ├── index.html
    ├── main.js
    ├── styles
    │   ├── landing.css
    │   └── main.css
    ├── ui
    │   ├── builder.js
    │   ├── hud.js
    │   └── screens.js
    └── wasm
        └── client.js
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

- `skill.rs`: 스킬 조회, selectable starting skill 목록, cost/weight 기반 샘플링
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

플레이어는 항상 `Basic Attack`을 사용할 수 있고, 시작 빌더에서 추가 시작 스킬 `0~4개`를 예산 안에 선택합니다.

선택하지 않은 슬롯은 `Empty`로 남아 전투 중 비활성 상태가 됩니다.

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

## Config 중심 확장 방향

이 프로젝트에서 중요한 목표는 콘텐츠 scale-out이 유연하게 작동하는 것입니다.

즉, 가능한 한 많은 콘텐츠를:

- Rust 로직 추가 없이
- 기존 규칙 조합만으로
- JSON config 수정만으로

늘릴 수 있어야 합니다.

완전히 새로운 전투 메커니즘은 코드가 필요할 수 있지만, 우선순위는 "새 스킬/특성/몬스터를 config만으로 많이 찍어낼 수 있는 구조"입니다.

### Config 대상 3축

현재 기준으로 장기적으로 config 확장의 중심은 아래 3개입니다.

1. `Skill`
2. `Trait`
3. `Monster`

### 1. Skill

스킬은 가장 많이 추가될 콘텐츠입니다.

목표:

- 신규 스킬은 가능하면 `core/data/skills.json`만 수정해서 추가
- 기존 effect / condition / status 조합만으로 표현
- 플레이어용 스킬과 몬스터용 스킬을 같은 데이터 모델로 공유

현재 스킬이 주로 표현하는 축:

- 피해
- 상태이상 부여
- 조건부 추가 피해
- 조건부 추가 상태이상
- 자기 버프
- 상태이상/trait와의 상호작용

중요한 설계 원칙:

- 새 스킬을 만들기 위해 새 Rust enum variant를 자주 추가하지 않는다.
- 먼저 "기존 상태이상과 조건 조합으로 표현 가능한가"를 본다.

### 2. Trait

Trait는 가장 형태가 다양해질 가능성이 큰 축입니다.

예:

- 특정 상태이상에 걸리면 오히려 강화됨
- 특정 행동 시 추가 효과 발생
- 플레이어/적/소유자 기준으로 다른 조건 반응

현재 원칙:

- Trait은 `cost`를 가진다.
- 등장 weight는 `cost`에서 자동 파생한다.
- trigger / condition / effect 조합으로 표현한다.

중요한 설계 원칙:

- 새 trait를 만들 때는 새 trait 전용 로직보다 기존 trigger/effect 조합을 우선 사용한다.
- trait는 "규칙 예외"가 아니라 "기존 시스템의 반응형 조합"으로 유지한다.

### 3. Monster

몬스터는 단순 stat blob가 아니라, "스킬과 trait를 가진 적 유닛 정의"로 취급합니다.

장기적으로 몬스터 config가 가져야 할 정보:

- 기본 스탯
- 보유 스킬
- 보유 trait
- 등장 풀
- 등장 weight
- 테마/태그/역할

즉 몬스터도 나중에는 "어떤 액션과 수동 효과를 가진 적인가"를 config로 조립하는 방향이 됩니다.

## Weight / Cost 원칙

모든 것을 같은 방식으로 weight 처리하지는 않습니다.

현재 권장 규칙:

- `Trait`: `cost`를 직접 갖고, 등장 weight는 `cost -> weight`로 자동 계산
- `Monster`: 등장 weight를 직접 가질 가능성이 큼
- `Skill`: 보상/상점/적 로드아웃 시스템이 생기면 직접 weight 또는 별도 풀 규칙을 가질 수 있음

즉 현재는 trait만 특별히 `cost` 중심으로 보고, 나머지는 explicit weight 쪽이 더 현실적입니다.

## 앞으로 config로 넣고 싶은 것

### Skill config

최소 목표:

- 신규 스킬 추가
- 플레이어 로드아웃 교체
- 몬스터 전용 스킬 부여

장기 목표:

- reward/shop 풀에서 weight 기반 등장
- 태그 기반 풀 구성

### Trait config

최소 목표:

- 시작 선택
- reward 선택
- enemy trait pool

장기 목표:

- pool/owner 구분
- 특정 조건에서만 등장
- cost 기반 확률 자동 조절

### Monster config

최소 목표:

- 적 스탯
- 적 스킬
- 적 trait

장기 목표:

- stage/theme/pool 기반 샘플링
- weight 기반 등장 빈도 조절
- role/tag 기반 적군 구성

## 콘텐츠 확장 원칙

새 콘텐츠를 추가할 때의 우선순위는 아래와 같습니다.

1. 기존 status / condition / effect 조합으로 표현 가능한지 먼저 본다.
2. 가능하면 JSON만 수정한다.
3. 정말 필요한 경우에만 새 effect/condition/type을 추가한다.

즉 리팩터링의 궁극적인 목적은 단순히 `step_api.rs`를 얇게 만드는 것이 아니라,
"콘텐츠 추가 비용을 낮추는 구조"를 만드는 것입니다.

관련 문서:

- 플레이 가이드: [docs/player/index.md](/Users/lavumi/private/overstack/docs/player/index.md)
- 개발 가이드: [docs/dev/index.md](/Users/lavumi/private/overstack/docs/dev/index.md)

## Step API

현재 UI가 주로 사용하는 흐름은 아래와 같습니다.

1. 랜딩 화면에서 builder 진입
2. `create_run_with_stats(...)`로 run 생성
3. builder에서 고른 trait를 `set_active_traits(...)`로 적용
4. builder에서 고른 starting skills를 `set_player_skills(...)`로 적용
5. 루프에서 `step_with_action(handle, dt, "none", -1)` 호출
6. `StepResult.need_input === true`면 `"basic"` 또는 `"skill"` 입력 전달
7. 매 루프마다 `get_snapshot(handle)`로 HUD 갱신
8. 필요 시 `reset_run(handle)` / `destroy_run(handle)`

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
- `get_selectable_trait_costs() -> Vec<u32>`
- `get_selectable_skill_names() -> Vec<String>`
- `get_selectable_skill_ids() -> Vec<String>`
- `get_selectable_skill_costs() -> Vec<u32>`
- `sample_starting_trait_ids(seed, n) -> Vec<String>`
- `sample_starting_skill_ids(seed, n) -> Vec<String>`
- `set_active_trait(handle, trait_id) -> bool`
- `set_active_traits(handle, trait_ids_csv) -> bool`
- `set_player_skills(handle, skill_ids_csv) -> bool`
- `reset_run(handle) -> bool`
- `destroy_run(handle)`

## 이벤트 로그

브라우저 이벤트 로그는 별도 DOM 패널이 아니라 DevTools console에서 확인합니다.

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

현재 정적 사이트의 시작 흐름은 아래와 같습니다.

1. 랜딩 화면
2. 전체 화면 builder
3. 전투 HUD

builder에서는 다음을 함께 정합니다.

- 시작 trait 1개
- 시작 스킬 0~4개
- 스탯 세트
- manual/random 예산 확인

현재 규칙:

- manual budget: `100`
- random budget: `120`
- random은 trait-first로 시작 trait 1개와 시작 스킬들을 먼저 고른 뒤 남은 예산으로 stats를 생성

## 빌드

사전 준비:

1. Rust 설치
2. `wasm-bindgen-cli` 설치
3. `wasm32-unknown-unknown` 타깃 설치

```bash
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
```

빌드:

```bash
cd core
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ../site/pkg target/wasm32-unknown-unknown/release/core.wasm
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
./build.sh
```

- 기본: 백그라운드 실행
- 포그라운드: `./build.sh --fg`

## 데이터 수정 방법

1. `core/data/skills.json`, `core/data/traits.json`, `core/data/enemies.json` 수정
2. 필드 규칙은 `core/data/schema.md` 참고
3. `cargo test` 또는 `cargo build --target wasm32-unknown-unknown --release` 후 `wasm-bindgen --target web --out-dir ../site/pkg target/wasm32-unknown-unknown/release/core.wasm`
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
