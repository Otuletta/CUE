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
| [0012](0012-distribution-and-integration.md) | Distribution, integration and releases | accepted |
| [0013](0013-camera-entry-persistence-languages.md) | Camera, first mode, persistence, languages and resolution | accepted |

## Open decisions in the specification

§31 of the [specification](../specification.es.md) lists the decisions it
left open, under its own IDs. This is where each one is answered:

| Spec ID | Decision | Answered in | Outcome |
|---|---|---|---|
| ADR-005 | Camera | [0013](0013-camera-entry-persistence-languages.md) | One camera, tilted slightly (0.2 rad) |
| ADR-006 | Table appearance | [0011](0011-visual-direction-and-graphics-quality.md) | Classic, studio (default) and contrast themes |
| ADR-007 | Physics time step | [0005](0005-event-driven-integration.md) | No time step: event-driven, exact event times |
| ADR-008 | Friction and spin model | [0002](0002-custom-billiards-solver.md), [0007](0007-cushion-and-pocket-model.md) | A purpose-built solver instead of Rapier |
| ADR-009 | First mode | [0013](0013-camera-entry-persistence-languages.md) | Free Play at once; the host may start a match |
| ADR-010 | Power mechanic | [0008](0008-controls-and-accessible-overlay.md) | **Provisional** until the usability round |
| ADR-011 | Call-shot interface | [0010](0010-eight-ball-rules-and-call-shot.md) | A visual call that follows the aim, plus accessible pickers |
| ADR-012 | Web graphics backend | [0003](0003-wasm-and-webgl2.md) | One WebGL 2 build |
| ADR-013 | Local persistence | [0013](0013-camera-entry-persistence-languages.md) | Preferences only |
| ADR-014 | Challenge URLs | — | **Open**: Phase 7, after 1.0 |
| ADR-015 | Distribution namespace | [0012](0012-distribution-and-integration.md) | `@tuletta/cue`: the npm organization `tuletta` is reserved |
| ADR-016 | Asset licence | [0011](0011-visual-direction-and-graphics-quality.md) | No third-party assets: all procedural, under the code's licence |
| ADR-017 | First eight-ball | [0010](0010-eight-ball-rules-and-call-shot.md) | Full WPA rules, plus a casual profile |
| ADR-018 | Several instances per page | [0006](0006-web-lifecycle.md) | Not in v1: one instance per page |
| ADR-019 | Interface languages | [0013](0013-camera-entry-persistence-languages.md) | English and Spanish |

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
