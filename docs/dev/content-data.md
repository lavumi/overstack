# Content Data

이 문서는 현재 게임 데이터를 어떤 파일에서 관리하는지 정리합니다.

## 주요 데이터 파일

- `core/data/skills.json`
- `core/data/traits.json`
- `core/data/enemies.json`
- `core/data/schema.md`

## Skills

`core/data/skills.json`은 다음을 포함합니다.

- 플레이어 기본 로드아웃
- 시작 빌더에서 고를 수 있는 selectable skill 목록
- 각 스킬의 `id`, `name`, `description`
- 각 스킬의 `cost`
- effect 목록
- tags

현재 예시:

- `basic_attack`
- `ember_lash`
- `frost_bite`
- `arc_jolt`
- `ruin_strike`
- `purge_strike`

구조 메모:

- `basic_attack`은 무료 기본 액션이고, 다른 시작 스킬은 `cost`를 가집니다.
- 현재 시작 빌드는 고른 starting skills만 run별 loadout으로 override합니다.
- 현재 스킬은 `effects` 배열 중심으로 정의됩니다.
- 실제 피해는 `DealDamage` effect가 담당합니다.
- 상태이상 부여, 제거, 조건부 증폭도 같은 배열 안에서 조합됩니다.

## Traits

`core/data/traits.json`은 다음을 포함합니다.

- selectable trait 목록
- 각 trait의 `id`, `name`, `description`
- `cost`
- `pool`
- `triggers`

Trait은 `rarity_weight` 없이 `cost`만 가집니다.

구조 메모:

- 시작 빌드에서는 현재 selectable trait 중 정확히 1개를 사용합니다.
- reward sampling도 장기적으로 같은 `cost` 축을 공유합니다.
- trait 등장 가중치는 `cost`에서 자동 파생됩니다.

## Enemies

`core/data/enemies.json`은 적 정의를 포함합니다.

- 기본 스탯
- `spd`
- crit 관련 값
- skill 목록

주의:

- enemy JSON은 `spd` 키를 사용하지만, 런타임에서는 `speed`로 읽습니다.
- 플레이어와 적 모두 같은 speed 스케일을 사용합니다.
- 현재 적 속도값은 대략 20~30대 절대값 스케일입니다.

## 시작 빌드 데이터와 콘텐츠 데이터의 연결

시작 빌드는 trait, skills, stats를 함께 사용합니다.

- trait는 `core/data/traits.json`
- 스킬은 `core/data/skills.json`
- 적 기준은 `core/data/enemies.json`

즉 플레이어 빌드, trait weight, 적 스탯이 서로 완전히 분리된 시스템이 아니라 같은 전투 규칙 위에서 만납니다.

## 스키마

`core/data/schema.md`는 JSON 필드 레벨의 형식을 설명합니다.

이 문서는 "데이터 파일이 무엇을 담당하는가"를 설명하고, `schema.md`는 "각 필드가 어떤 형식인가"를 설명하는 역할로 분리합니다.
