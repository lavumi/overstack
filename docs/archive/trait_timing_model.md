# Trait Timing Model

이 문서는 trait를 `modifier trait`와 `reactive trait` 관점으로 나누고, 각 trigger가 액션 파이프라인의 어느 시점에서 발동하는지 정리합니다.

핵심 목표:

- 스킬 실행 순서와 trait 반응 시점을 일관되게 유지
- "이번 액션에서 새로 생긴 상태가 이번 액션 계산에 되먹임되지 않게" 규칙을 명확히 함
- 새 trait를 추가할 때 calc 계층인지 reactive 계층인지 먼저 판단 가능하게 함

## 기본 모델

현재 액션 실행의 기본 모델은 아래와 같습니다.

1. `calc layer`
2. `damage layer`
3. `status apply layer`

즉:

- 먼저 액션 시작 시점의 상태를 읽는다
- 그 상태를 바탕으로 damage modifier를 계산한다
- 실제 피해를 적용한다
- 마지막에 상태이상/버프/후속 상태 효과를 적용한다

중요한 규칙:

- 이번 액션에서 새로 적용된 상태는 이번 액션의 calc layer에 영향을 주지 않는다
- 상태 적용 이후에만 reactive trait가 반응한다

## Trait 분류

### Modifier Trait

설명:

- 액션 시작 시점 snapshot만으로 판단 가능한 trait
- 주로 calc layer에 영향을 준다

예시:

- 특정 상태가 걸린 적에게 피해 증가
- 특정 조건이면 상태 위력 증가
- 기본 피해 배율 보정

현재 코드에서는 주로 `OnActionUsed` 성격으로 보는 것이 자연스럽다.

### Reactive Trait

설명:

- 실제 이벤트가 발생한 뒤 반응하는 trait
- damage 이후, status 적용 이후, tick 시점 등에 반응한다

예시:

- Shock를 실제로 적용한 뒤 bonus pure damage
- Freeze 적용 후 Break stack 추가
- Status tick 시 추가 반응
- Battle start/end 시 효과 발생

## Trigger -> Phase 매핑

현재 코드 기준 매핑:

- `OnActionUsed` -> `PreActionCalc`
- `OnDamageDealt` -> `PostDamage`
- `OnStatusApplied` -> `PostStatus`
- `OnStatusTick` -> `StatusTick`
- `OnBattleStart` -> `BattleBoundary`
- `OnBattleEnd` -> `BattleBoundary`
- `OnTurnStart` -> `BattleBoundary`

코드 위치:

- phase 정의: `core/src/trait_spec.rs`
- dispatch/emit 함수: `core/src/engine/trait_system.rs`

## 현재 trait 분류

### Modifier 쪽에 가까운 trait

- `Cinder Scholar`
  - trigger: `OnStatusApplied`
  - 현재 구현은 reactive지만, 설계 관점에서는 "Burn power modifier" 성격이 강함
  - 장기적으로는 상태 적용 계산 계층으로 일부 이동 가능성 있음

### Reactive trait

- `Frozen Momentum`
  - `Freeze`가 실제로 적용된 뒤 `Break` stack 추가
- `Overcharge`
  - `Shock`가 실제로 적용된 뒤 bonus pure damage
- `Ruthless`
  - 피해가 실제로 들어간 뒤 추가 피해 반응
- `Shatterpoint`
  - `Break`가 실제로 적용된 뒤 조건부 `Stun`
- `Iron Shell`
  - 전투 시작 시 저항 증가

즉 현재 trait 대부분은 reactive trait 쪽에 가깝습니다.

## 코드 호출 지점

현재 trait 호출 이름도 시점을 드러내도록 정리되어 있습니다.

- pre-action: `core/src/engine/skill_exec.rs`
  - `emit_pre_action_trait_triggers(...)`
- post-damage: `core/src/engine/status_system.rs`
  - `emit_post_damage_trait_triggers(...)`
- post-status: `core/src/engine/status_system.rs`
  - `emit_post_status_trait_triggers(...)`
- status tick: `core/src/engine/status_system.rs`
  - `emit_status_tick_trait_triggers(...)`
- battle start: `core/src/engine/run_flow.rs`
  - `emit_battle_start_trait_triggers(...)`
- battle end: `core/src/engine/trait_system.rs`
  - `emit_battle_end_triggers(...)`

## 새 trait를 추가할 때의 질문

새 trait를 만들 때 먼저 아래를 묻습니다.

1. 이 trait는 액션 시작 시점 snapshot만으로 판정 가능한가
2. 아니면 실제 피해/상태 적용 결과를 본 뒤에만 반응 가능한가
3. calc layer modifier로 두는 것이 맞는가
4. reactive trigger로 두는 것이 맞는가

정리:

- snapshot만으로 충분하면 modifier trait
- 실제 이벤트 결과가 필요하면 reactive trait

## 앞으로의 개선 방향

장기적으로는:

- modifier trait는 calc layer에서 더 많이 흡수
- reactive trait만 이벤트 반응 시스템에 남김

이 방향으로 가면:

- damage 계산은 더 예측 가능해지고
- trait 표현력은 유지하면서도
- JSON 기반 콘텐츠 확장이 더 쉬워진다
