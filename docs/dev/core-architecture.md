# Core Architecture

Rust 코어는 크게 네 층으로 나뉩니다.

## 1. Data

위치:

- `core/src/data/*`

역할:

- JSON 로드
- 검증
- 런타임 spec 컴파일
- registry 캐싱

주요 파일:

- `defs.rs`
- `validate.rs`
- `compile.rs`
- `registry.rs`
- `specs.rs`

## 2. Rules / Policy

역할:

- 스킬 조회
- trait 조회
- 비용과 weight 계산
- 전투 수학 정책

중요 포인트:

- trait `cost`와 weight 계산은 여기서 다뤄집니다.
- speed는 플레이어/적 공통 절대값 스탯으로 해석됩니다.

주요 파일:

- `core/src/skill.rs`
- `core/src/trait_spec.rs`
- `core/src/combat_math.rs`
- `core/src/engine/numeric.rs`

## 3. Engine

위치:

- `core/src/engine/*`

역할:

- 실제 전투 실행
- run flow
- turn flow
- 상태이상 처리
- trait 반응 처리
- snapshot 생성

주요 파일:

- `run_flow.rs`
- `turn_system.rs`
- `skill_exec.rs`
- `status_system.rs`
- `trait_system.rs`
- `snapshot.rs`

실행 규칙 메모:

- run 시작과 battle start는 `run_flow.rs`
- 게이지 진행은 `turn_system.rs`
- 스킬 실행은 `skill_exec.rs`
- 상태이상 적용/만료/속도 배율은 `status_system.rs`
- trait 반응은 `trait_system.rs`

현재 액션 파이프라인은 `calc -> damage -> status apply` 모델에 맞춰 정리되어 있습니다.

## 4. WASM / Step API

역할:

- 브라우저에서 호출할 공개 API 제공
- run handle 관리

주요 파일:

- `core/src/step_api.rs`
- `core/src/step_api/manager.rs`

## 현재 방향

현재 구조 방향은:

- `step_api`를 얇게 유지
- 실제 규칙 실행은 `engine/*`에 모음
- 데이터 기반 확장성을 유지

추가 메모:

- `step_api`는 브라우저와 연결되는 얇은 경계를 목표로 함
- 프런트는 `site/wasm/client.js`를 통해 이 API를 사용

즉, 프런트는 상태 표시와 입력 전달에 집중하고, 실제 게임 규칙은 Rust 코어에 두는 것이 기본 원칙입니다.
