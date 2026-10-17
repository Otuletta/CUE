# ADR-0001 — Rust and Bevy

- **Status:** accepted
- **Date:** 2026-10-17

## Context

CUE is a billiards engine with a playable reference experience that must run
inside a web page, with no backend, and double as a showcase of systems
engineering. The simulation, rules and game logic need to be the real
implementation, not a thin layer over a JavaScript engine.

## Decision

- **Rust** (stable, pinned in `rust-toolchain.toml`) for all simulation, rules
  and game logic.
- **Bevy 0.20** for ECS, scheduling, scene, camera, rendering, input and audio.
- A strict crate split so that only `cue-game` and `cue-web` depend on Bevy:

  | Crate         | Depends on                        |
  |---------------|-----------------------------------|
  | `cue-core`    | `glam`, `thiserror`               |
  | `cue-physics` | `cue-core`                        |
  | `cue-game`    | `cue-core`, `cue-physics`, `bevy` |
  | `cue-web`     | `cue-game`, `wasm-bindgen`        |

## Consequences

- Rules and physics are testable natively in milliseconds, without a GPU.
- `glam` is pinned to the line Bevy uses, so physics state (`f64`) converts
  to render transforms (`f32`) without a second math library.
- Bevy releases are frequent and break APIs. Upgrades happen on their own
  branch, with the physics and rules test suites as the safety net.
- Bevy's WASM output is large. This is the main risk to the portfolio
  integration and is measured in Phase 1 (see ADR-0003).

## Review conditions

Revisit if the Phase 1 vertical slice shows that Bevy on WASM cannot meet an
acceptable time-to-first-shot on the reference mobile device, even after
feature trimming.
