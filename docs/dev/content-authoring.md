# Content Authoring

이 문서는 새 스킬, trait, enemy를 추가할 때의 기본 원칙을 설명합니다.

## 기본 원칙

1. 먼저 JSON만으로 표현 가능한지 본다.
2. 기존 상태이상, condition, effect 조합을 우선 사용한다.
3. 정말 필요할 때만 새 primitive를 추가한다.

## 어디에 추가하나

- 스킬: `core/data/skills.json`
- trait: `core/data/traits.json`
- enemy: `core/data/enemies.json`

## 새 스킬 추가

새 스킬을 추가할 때 먼저 확인할 것:

- 시작 빌더 selectable skill로 열어둘 것인가
- `cost`를 얼마로 둘 것인가
- 피해형인가
- 상태이상 부여형인가
- 조건부 증폭형인가
- 상태 소비/제거형인가

현재 지원되는 대표 effect:

- `DealDamage`
- `ApplyStatus`
- `ConditionalDamageAmp`
- `ConditionalApplyStatus`
- `RemoveStatus`
- `DealPureDamage`

작성 팁:

- `basic_attack`을 제외한 시작 스킬은 `cost >= 1`로 잡습니다.
- 피해가 필요하면 `DealDamage`
- 상태를 걸고 싶으면 `ApplyStatus`
- 특정 상태가 있을 때만 더 세게 때리고 싶으면 `ConditionalDamageAmp`
- 상태를 소비하고 싶으면 `RemoveStatus`

현재 액션 처리 순서는 `calc -> damage -> status apply` 이므로, 새로 적용된 상태가 같은 액션의 damage 계산에 바로 되먹임되지는 않습니다.

## 새 trait 추가

새 trait를 추가할 때는 아래를 먼저 묻습니다.

1. 이 trait는 modifier인가
2. reactive trigger인가
3. 기존 trigger/effect 조합으로 표현 가능한가

현재 trait는 가능한 한 기존 시스템의 반응형 조합으로 유지하는 것이 원칙입니다.

추가 체크:

- 시작 trait로 selectable 해야 하는가
- `cost`는 어느 정도가 적절한가
- player pool인지 enemy pool인지
- 기존 상태이상 축과 실제 시너지가 있는가

## 새 enemy 추가

새 enemy는 최소 다음을 가져야 합니다.

- 이름
- 기본 스탯
- speed
- skill 목록

향후에는 trait까지 더 적극적으로 확장될 수 있습니다.

enemy 작성 시 주의:

- `spd`는 플레이어와 같은 절대 speed 스케일로 넣어야 합니다.
- 너무 높은 speed는 게이지 전투를 크게 왜곡할 수 있습니다.

## 새 primitive가 필요한 경우

아래 조건일 때만 새 primitive를 고려합니다.

- 기존 조합으로 표현 불가
- 앞으로 여러 콘텐츠에 재사용 가능
- 시스템 복잡도를 감당할 가치가 있음

현재 목표는 "새 코드를 늘리는 것"보다 "새 콘텐츠를 값싸게 넣을 수 있는 구조"를 유지하는 것입니다.
