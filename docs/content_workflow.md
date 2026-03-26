# Content Workflow

이 문서는 Overstack에서 새 스킬 / 특성 / 몬스터 / primitive를 추가할 때 따르는 기본 프로세스를 정리한 문서입니다.

핵심 원칙:

- 먼저 JSON로 표현을 시도한다.
- 기존 primitive 재사용을 우선한다.
- 정말 필요할 때만 새 코드 primitive를 추가한다.

## 목표

콘텐츠 추가 비용을 낮춘다.

즉, 이상적인 흐름은:

1. 아이디어 작성
2. JSON으로 표현
3. validator 통과
4. 테스트 통과
5. 게임에서 확인

입니다.

## A. 새 Skill / Trait / Monster 추가 프로세스

### 1. 기획 문장으로 먼저 적기

예:

- "Burn이 걸린 적에게 추가 피해를 주는 스킬"
- "Freeze에 걸릴 때 오히려 빨라지는 trait"
- "Shock와 Break를 둘 다 쓰는 몬스터"

### 2. 기존 primitive로 표현 가능한지 확인

먼저 [content_primitives.md](/Users/lavumi/private/overstack/docs/content_primitives.md)를 본다.

확인 질문:

- 기존 `Effect` 조합으로 가능한가
- 기존 `Condition`으로 분기 가능한가
- 기존 `Status`만으로 표현 가능한가
- trait라면 modifier trait인지 reactive trait인지 먼저 구분했는가

가능하면 코드 변경 없이 JSON만 수정한다.

### 3. config 작성

- 스킬: [skills.json](/Users/lavumi/private/overstack/core/data/skills.json)
- 특성: [traits.json](/Users/lavumi/private/overstack/core/data/traits.json)
- 몬스터: [enemies.json](/Users/lavumi/private/overstack/core/data/enemies.json)

### 4. 검증 / 테스트

```bash
cd /Users/lavumi/private/overstack/core
cargo test
```

필요하면 WASM 빌드도 수행:

```bash
cd /Users/lavumi/private/overstack/core
wasm-pack build --target web --out-dir ../site/pkg
```

### 5. 실제 로그 확인

정적 사이트 실행 후:

- 의도한 이벤트가 발생하는지
- 상태이상 적용/만료가 맞는지
- trait trigger가 기대대로 나는지

확인한다.

## B. 새 primitive 추가 프로세스

이건 새 `Effect`, `Condition`, `Trigger`, `Status` 등을 추가할 때만 사용합니다.

### 1. 새 primitive가 정말 필요한지 먼저 확인

아래 셋 중 하나일 때만 추가를 고려:

- 기존 조합으로 정말 표현 불가
- 앞으로 반복해서 쓸 가능성이 큼
- 스킬 / 특성 / 몬스터 2개 이상에 공통으로 적용 가능

### 2. 추가 전에 예제 JSON부터 적어보기

최소 2개 권장:

- 최소 재현 예제
- 실제 게임에서 쓸 예제

이 단계에서 JSON이 이상하게 복잡하면 primitive 설계가 나쁜 신호일 수 있다.

### 3. 수정 위치

새 primitive 추가 시 보통 아래를 같이 수정:

1. [specs.rs](/Users/lavumi/private/overstack/core/src/data/specs.rs)
2. [validate.rs](/Users/lavumi/private/overstack/core/src/data/validate.rs)
3. [compile.rs](/Users/lavumi/private/overstack/core/src/data/compile.rs)
4. 관련 engine 구현 파일

대표 위치:

- condition 평가: [combat_state.rs](/Users/lavumi/private/overstack/core/src/engine/combat_state.rs)
- effect 실행: [skill_exec.rs](/Users/lavumi/private/overstack/core/src/engine/skill_exec.rs), [trait_system.rs](/Users/lavumi/private/overstack/core/src/engine/trait_system.rs)
- 상태이상 처리: [status_system.rs](/Users/lavumi/private/overstack/core/src/engine/status_system.rs)
- trait 시점 모델: [trait_timing_model.md](/Users/lavumi/private/overstack/docs/trait_timing_model.md)

### 4. 테스트 추가

반드시 둘 다 추가를 권장:

- validator 테스트
- runtime 동작 테스트

즉:

- 잘못된 JSON이 좋은 에러를 내는지
- 올바른 JSON이 실제로 기대한 이벤트를 내는지

둘 다 확인한다.

### 5. 문서 업데이트

primitive를 추가했으면 같이 갱신:

- [content_primitives.md](/Users/lavumi/private/overstack/docs/content_primitives.md)
- 필요 시 [README.md](/Users/lavumi/private/overstack/README.md)
- 필요 시 [schema.md](/Users/lavumi/private/overstack/core/data/schema.md)

## C. 리뷰 체크리스트

새 콘텐츠 또는 새 primitive를 추가할 때 아래를 확인:

- JSON만으로 해결 가능한데 불필요한 코드가 들어가지는 않았는가
- 신규 primitive가 하나의 케이스만 위한 과잉 설계는 아닌가
- validator 에러 메시지가 path 포함으로 충분히 친절한가
- 스킬과 trait 양쪽에서 일관된 의미로 동작하는가
- 나중에 monster 쪽에서도 재사용 가능한가

## D. 현재 추천 우선순위

지금 이 프로젝트에서의 권장 순서:

1. 가능한 한 JSON 콘텐츠를 먼저 늘린다
2. 필요한 공통 primitive를 최소 단위로 추가한다
3. step API는 계속 얇게 유지한다
4. 콘텐츠 scale-out에 직접 도움이 되는 방향으로만 구조를 확장한다

즉, "새 코드를 넣는 것"보다 "새 콘텐츠를 싸게 넣을 수 있게 만드는 것"이 우선입니다.
