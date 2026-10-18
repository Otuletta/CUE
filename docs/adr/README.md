# Architecture Decision Records

Each significant decision lives in its own numbered file. Records are never
edited silently: a reversed decision gets a new ADR that supersedes the old
one.

| ADR | Decision | Status |
|-----|----------|--------|
| [0001](0001-rust-and-bevy.md) | Rust and Bevy | accepted |
| [0002](0002-custom-billiards-solver.md) | Purpose-built billiards solver instead of Rapier | accepted |
| [0003](0003-wasm-and-webgl2.md) | WebAssembly with WebGL2 first | accepted (measured) |
| [0004](0004-no-backend.md) | No backend | accepted |
| [0005](0005-event-driven-integration.md) | Event-driven integration (no physics time step) | accepted |
| [0006](0006-web-lifecycle.md) | Web lifecycle: one instance per page, pause instead of remount | accepted |
| [0007](0007-cushion-and-pocket-model.md) | Cushion, pocket and contact models | accepted |
| [0008](0008-controls-and-accessible-overlay.md) | Controls and the accessible control layer | accepted (power mechanic provisional) |
| [0009](0009-procedural-audio.md) | Procedural audio | accepted |
| [0010](0010-eight-ball-rules-and-call-shot.md) | Eight-ball rules engine, profiles and call-shot interface | accepted |
| [0011](0011-visual-direction-and-graphics-quality.md) | Visual direction, procedural assets and graphics quality | accepted |

Open decisions (camera, power mechanic and others) are listed in §31 of the
[specification](../specification.es.md) and get an ADR when they are
resolved.

## Template

```markdown
# ADR-XXXX — Title

- **Status:** proposed | accepted | superseded by ADR-YYYY
- **Date:** YYYY-MM-DD

## Context
## Decision
## Consequences
## Evidence
## Review conditions
```
