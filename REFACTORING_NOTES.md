# Refactoring Notes (Current)

## Context
Core simulation logic has grown significantly and currently combines multiple responsibilities in a few large files, especially `core/src/step_api.rs`.

## Open Bug (Not Fixed Yet)

1. Trait effect visibility confusion (Cinder Scholar)
- Symptom observed in logs: Burn `StatusTick` appears as repeated `amount=1`, making it unclear whether trait bonus is applied.
- Repro context:
  - Active trait: `Cinder Scholar`
  - Typical log sequence:
    - `StatusApplied` (Burn)
    - repeated `StatusTick` on enemy with constant integer-looking damage
- Expected for debugging:
  - Log should show HP transitions with decimal precision so trait multiplier effects can be inspected.

## Current Pain Points

1. `core/src/step_api.rs` has mixed concerns
- WASM API exports
- Run handle management
- Battle loop and gauge simulation
- Status lifecycle (apply/tick/expire)
- Skill effect execution
- Trait trigger/effect execution
- Snapshot assembly
- Integration tests

2. Data and execution are not fully separated
- `skill.rs` / `trait_spec.rs` hold data specs, but interpreters still live in `step_api.rs`.

3. Numeric policy is duplicated
- HP rounding/display rules are applied in multiple places.

4. Frontend state complexity is growing
- `site/main.js` manages multiple modes (`trait_select`, `running`, `need_input`, `ended`) plus rendering.

## Refactoring Direction

1. Split engine modules (no behavior change first)
- `core/src/engine/run_manager.rs`
- `core/src/engine/combat.rs`
- `core/src/engine/status.rs`
- `core/src/engine/skill_exec.rs`
- `core/src/engine/trait_exec.rs`
- `core/src/engine/snapshot.rs`
- Keep `step_api.rs` as thin wasm binding layer.

2. Shared execution context types
- Unify around reusable context structs:
  - `ActionContext`
  - `TriggerContext`
  - `StatusContext`

3. Centralize numeric policy
- Single utility module for:
  - hp internal precision
  - event/display conversion policy

4. Increase modular tests
- Split tests by domain:
  - skill application
  - trait trigger correctness
  - trait chain-depth safety
  - status tick/expiry

5. Frontend split (minimal)
- Separate:
  - state machine
  - log renderer
  - HUD renderer
  - wasm adapter calls

## Priority Order
1. Engine file split with no logic changes
2. Skill/Trait execution extraction
3. Numeric policy consolidation
4. Test segmentation
5. Frontend modular split
