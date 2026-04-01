# Frontend Architecture

현재 프런트는 single-page 구조를 유지하면서 역할별 JS 모듈로 분리되어 있습니다.

## 화면 단계

현재 화면 흐름:

1. landing
2. builder
3. game

즉:

- 랜딩 화면
- 전체 화면 캐릭터 빌더
- 실제 전투 HUD

를 한 페이지 안에서 상태 전환으로 처리합니다.

## 주요 파일

### `site/main.js`

역할:

- 앱 부팅
- WASM 연결
- run loop
- 이벤트 로그 포맷
- 각 UI 모듈 연결

즉, 실제 DOM 세부 조작보다 orchestration과 흐름 제어를 맡습니다.

### `site/ui/screens.js`

역할:

- landing / builder / game 전환

### `site/ui/builder.js`

역할:

- 시작 빌드 상태 관리
- trait-first 랜덤 빌드
- manual/random 전환
- 예산 계산
- confirm / cancel

현재 규칙:

- trait-first
- 시작 trait 정확히 1개
- 시작 스킬 0~4개
- manual budget 100
- random budget 120
- speed는 절대값 스탯
- random은 trait 1개를 먼저 고르고, weighted starting skills를 채운 뒤 남은 예산으로 stats를 만든다

### `site/ui/hud.js`

역할:

- snapshot 렌더링
- 상태이상/trait pills
- arena 상태 문구
- 액션 도크

### `site/wasm/client.js`

역할:

- wasm-bindgen export 래퍼
- `main.js`가 코어 함수 이름을 직접 많이 알지 않게 하는 경계

현재 이 레이어는 다음을 감쌉니다.

- run 생성/파괴
- snapshot 읽기
- step 진행
- selectable trait 조회
- starting trait sampling
- selectable skill 조회
- starting skill sampling
- 선택한 trait/skills를 run에 적용

## 스타일

- 레이아웃과 비주얼은 `site/styles/main.css`
- 랜딩 화면 전용 비주얼은 `site/styles/landing.css`
- 로그는 DOM 패널이 아니라 DevTools console

## 현재 남은 분리 후보

- 이벤트/로그 포맷 전용 모듈
- run loop 상태 머신 helper

## 현재 의도

프런트의 목적은 다음 두 가지입니다.

1. 빌드와 전투 상태를 명확하게 보여준다.
2. 게임 규칙은 가능한 한 Rust/WASM 코어에 둔다.
