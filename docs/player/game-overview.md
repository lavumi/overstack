# Game Overview

Overstack는 상태이상과 trait 조합을 중심으로 하는 게이지 기반 전투 게임입니다.

한 판의 기본 흐름:

1. 랜딩 화면에서 시작한다.
2. 캐릭터 빌더에서 시작 trait와 스탯을 정한다.
3. 전투가 시작되면 게이지가 차는 순서대로 행동한다.
4. 상태이상, trait, 속도 차이가 전투 리듬을 만든다.
5. 모든 노드를 넘기면 승리, 중간에 쓰러지면 패배한다.

## 핵심 축

### 상태이상

현재 전투의 중심 상태이상:

- Burn
- Freeze
- Shock
- Break
- Bleed
- Stun
- Might
- Haste

상태이상과 trait 반응이 빌드의 대부분을 결정합니다.

### 게이지 전투

전투는 고정 교대 턴이 아니라 게이지 기반입니다.

- 각 유닛은 `speed`에 따라 `action_gauge`를 채웁니다.
- `action_gauge >= 100`이 되면 행동할 수 있습니다.
- 따라서 속도와 상태이상이 턴 순서를 크게 바꿉니다.

## 현재 기본 스킬

플레이어 기본 로드아웃:

1. `Ember Lash`
2. `Frost Bite`
3. `Arc Jolt`
4. `Ruin Strike`

이 스킬들은 각각 Burn, Freeze, Shock, Break 축을 담당합니다.

## 이어서 읽을 문서

- 빌드 규칙: [Building Rules](building-rules.md)
- 전투 규칙: [Combat Rules](combat-rules.md)
- 상태이상 상세: [Status Reference](status-reference.md)
- trait 상세: [Trait Reference](trait-reference.md)
