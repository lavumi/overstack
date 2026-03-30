Original prompt: 로그는 consolelog로 옮기는 방향으로 진행하자. 1번 진행해봐.

- 2026-03-26: Frontend HUD pass 1 started.
- Converted the center panel from an in-page raw log viewer into a scene-first battle stage.
- Moved combat event output from DOM log accumulation to `console.log`.
- Reworked status and trait rendering into pill/chip UI for both player and enemy side panels.
- Added central duel cards for player/enemy with state summaries driven by snapshot data.
- Verification: `node --check site/main.js` passed.
- Verification limit: local Playwright run was blocked because the shared client could not import the `playwright` package in this environment.
- TODO: Browser-check the new layout and confirm no runtime errors after the scene-first HUD pass.
- TODO: Consider splitting `site/main.js` into smaller UI modules before adding canvas or richer animations.
