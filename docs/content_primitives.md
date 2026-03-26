# Content Primitives

이 문서는 현재 Overstack가 JSON config만으로 표현할 수 있는 콘텐츠 primitive를 정리한 카탈로그입니다.

목적:

- 새 스킬 / 특성 / 몬스터를 만들 때 먼저 "이미 가능한 조합인가?"를 판단
- 새 코드 추가 전에 기존 primitive 재사용 가능성을 확인
- validator / compiler / runtime 구현 위치를 빠르게 찾기

## 읽는 순서

1. 이 문서에서 가능한 primitive를 확인
2. [schema.md](/Users/lavumi/private/overstack/core/data/schema.md)에서 JSON 필드 형식 확인
3. [skills.json](/Users/lavumi/private/overstack/core/data/skills.json), [traits.json](/Users/lavumi/private/overstack/core/data/traits.json), [enemies.json](/Users/lavumi/private/overstack/core/data/enemies.json)에서 실제 예제 확인

## 선언 / 검증 / 구현 위치

- 선언: [specs.rs](/Users/lavumi/private/overstack/core/src/data/specs.rs)
- 검증: [validate.rs](/Users/lavumi/private/overstack/core/src/data/validate.rs)
- 컴파일: [compile.rs](/Users/lavumi/private/overstack/core/src/data/compile.rs)
- 스킬 실행: [skill_exec.rs](/Users/lavumi/private/overstack/core/src/engine/skill_exec.rs)
- 특성 실행: [trait_system.rs](/Users/lavumi/private/overstack/core/src/engine/trait_system.rs)
- 상태이상 처리: [status_system.rs](/Users/lavumi/private/overstack/core/src/engine/status_system.rs)
- 조건 평가: [combat_state.rs](/Users/lavumi/private/overstack/core/src/engine/combat_state.rs)

## Status

현재 선언된 상태이상:

- `Burn`
- `Freeze`
- `Shock`
- `Break`
- `Bleed`
- `Stun`
- `Might`
- `Haste`

구현 메모:

- DoT tick이 있는 상태: `Burn`, `Shock`, `Bleed`
- 게이지 속도에 직접 영향: `Freeze`, `Haste`, `Stun`
- 방어력 상호작용: `Break`
- 버프 성격: `Might`, `Haste`

핵심 구현 위치:

- [status_system.rs](/Users/lavumi/private/overstack/core/src/engine/status_system.rs)
- [combat_state.rs](/Users/lavumi/private/overstack/core/src/engine/combat_state.rs)

## Condition

현재 사용 가능한 condition:

- `Always`
- `SrcIsPlayer`
- `DstIsEnemy`
- `OwnerIsPlayer`
- `OwnerIsEnemy`
- `SrcIsOwner`
- `DstIsOwner`
- `AppliedStatusIs(status)`
- `RandomRollBelow(p)`
- `TargetHPBelow(ratio)`
- `TargetHasStatus(status)`
- `TargetStatusCountAtLeast(n)`
- `All([...])`

대표 용도:

- 특정 상태가 걸린 적에게만 추가 효과
- 특정 owner에서만 trait 발동
- 랜덤 확률 기반 trait / effect
- 상태이상 개수 기반 조건 분기

제약:

- 현재는 `Any`, `Not`, `TargetHasAllStatuses`, `TargetHasAnyStatus` 같은 조합형 condition은 없음
- 대상 status stack 수를 직접 참조하는 condition은 없음

## Trigger

현재 사용 가능한 trigger:

- `OnBattleStart`
- `OnTurnStart`
- `OnActionUsed`
- `OnDamageDealt`
- `OnStatusApplied`
- `OnStatusTick`
- `OnBattleEnd`

주 사용처:

- trait 트리거

제약:

- 현재 trigger는 trait 중심
- 스킬 자체는 "사용 시 즉시 실행" 구조이며 별도 trigger graph는 없음

관련 문서:

- trait timing model: [trait_timing_model.md](/Users/lavumi/private/overstack/docs/trait_timing_model.md)

## Effect

현재 사용 가능한 effect:

- `DealDamage`
- `ApplyStatus`
- `ConditionalDamageAmp`
- `ConditionalApplyStatus`
- `SelfBuff`
- `AddProcBonus`
- `AddResBonus`
- `ModifyStatusPower`
- `AddStatusStacks`
- `RemoveStatus`
- `DealPureDamage`

### DealDamage

설명:

- 대상에게 일반 피해를 준다
- `damage_kind`, `multiplier`, `flat` 사용

적합한 경우:

- 기본 공격
- 일반 공격형 스킬
- 조건부 추가타의 실제 피해 구간

### ApplyStatus

설명:

- 대상에게 상태이상을 건다
- `status`, `chance`, `duration`, `stacks`, `power` 사용

적합한 경우:

- Burn / Freeze / Shock / Break 부여
- Might / Haste 같은 버프 부여

### ConditionalDamageAmp

설명:

- condition이 참이면 추가 피해 배율을 만든다

적합한 경우:

- "Burn 걸린 적에게 추가 피해"
- "상태이상 2개 이상이면 강화"

### ConditionalApplyStatus

설명:

- condition이 참일 때만 상태이상을 건다

적합한 경우:

- "Freeze 상태면 Stun 시도"
- "특정 상황에서만 추가 디버프"

### SelfBuff

설명:

- 자기 자신에게 버프 성격 상태를 부여한다
- 현재 `Attack`, `Speed`만 지원

적합한 경우:

- self-empower 타입 스킬

제약:

- 현재 buff stat 축이 매우 제한적

### AddProcBonus

설명:

- 상태이상 부여 확률에 추가 보정치를 준다

적합한 경우:

- "다음 디버프 부여 확률 증가"

### AddResBonus

설명:

- 상태이상 저항 보정치를 준다

적합한 경우:

- "적중 저항 증가"

### ModifyStatusPower

설명:

- 특정 status의 power multiplier를 수정한다

적합한 경우:

- "Burn이 더 강해진다"
- "특정 상태이상의 위력을 증폭"

### AddStatusStacks

설명:

- 대상에게 특정 status stack을 추가한다

적합한 경우:

- "Freeze 걸리면 Break +1"
- "기존 상태를 더 두껍게 쌓기"

### RemoveStatus

설명:

- 대상의 특정 상태이상을 제거한다

적합한 경우:

- "Burn 제거"
- "디버프 정리"
- "상태 소비형 스킬의 첫 단계"

### DealPureDamage

설명:

- 방어 무시 성격의 고정 피해를 준다

적합한 경우:

- Shock 연계 추가 피해
- on-trigger flat damage

## Target

현재 effect target:

- `Owner`
- `Opponent`
- `Src`
- `Dst`
- `Player`
- `Enemy`

## 지금 JSON만으로 잘 만들 수 있는 것

- 단일 타격 스킬
- 상태이상 부여 스킬
- 상태이상 조건부 추가 피해 스킬
- 상태이상 조건부 추가 상태이상 스킬
- 자기 강화형 스킬
- 특정 상태 power를 강화하는 trait
- 상태 부여 시 연쇄 반응하는 trait

## 아직 JSON만으로 불편하거나 어려운 것

- 특정 상태이상 제거
- 특정 상태이상 소비 후 보상 획득
- 상태 stack 수에 비례한 동적 수치 계산
- 여러 조건을 더 풍부하게 조합하는 논리식
- 몬스터의 가중치 풀/테마/역할 기반 등장 규칙

## 새 primitive 추가 전 체크

새 effect / condition / trigger를 추가하기 전에 먼저 확인:

1. 기존 primitive 조합으로 표현 가능한가
2. 스킬과 trait 양쪽에서 재사용 가능한가
3. 하나의 콘텐츠가 아니라 여러 콘텐츠에 공통으로 쓰일 수 있는가
4. JSON 작성자가 직관적으로 이해할 수 있는가

이 문서에 없는 내용을 추가하게 되면, 구현 후 반드시 이 문서도 갱신합니다.
