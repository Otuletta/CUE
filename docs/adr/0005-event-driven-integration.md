# ADR-0005 — Event-driven integration

- **Status:** accepted (resolves open decision ADR-007 in spec §31: physics time step)
- **Date:** 2026-10-17

## Context

The spec proposed a fixed physics step (1/120 s, or 1/60 s with sub-steps)
and required the outcome to be independent of the render frame rate. ADR-0002
left the choice between a fixed step and event-driven integration to the
Phase 1 spike.

## Decision

Event-driven integration. Ball motion between events is closed-form. Events
are roots of polynomials of degree ≤ 4 and are resolved one at a time, in a
deterministic order. The authoritative state changes only at events.
Presentation samples the trajectories at whatever instant it needs.

There is **no physics time step**. The game feeds wall-clock time to
`World::advance_to`, clamped to 0.1 s per frame. That clamp affects only
playback speed, never the outcome.

## Evidence (Phase 1)

- **Frame-rate independence, bit for bit.** A full 16-ball break played at 7,
  30, 60 and 144 FPS, and with irregular frame pacing, produces identical
  event logs and final states (`break_outcome_is_identical_at_any_frame_rate`).
- **No tunnelling by construction.** Contacts are found analytically at any
  speed; a touching-and-closing pair is caught at once. Over 48 randomised
  breaks and 1,000 consecutive random shots, no invariant was violated: no
  interpenetration beyond 1 nm, no ball off the surface, no NaN.
- **Cost.** About 2.5 µs per event natively; a whole break is a few hundred
  events, so per-frame cost is negligible next to rendering.
- **Textbook results reproduced:** 5/7 rolling speed after a stun, the
  `v²/(2μr·g)` stopping distance, natural roll at a tip height of 2R/5, and the
  90° rule.

## Consequences

- Replays and shareable challenges (Phase 7) only need the initial layout,
  the strikes and the profile id.
- Determinism holds within one build and platform. Across native and WASM it
  holds wherever IEEE-754 basic operations agree, which is the case for
  everything the solver uses today (`+ − × ÷ sqrt`). Adding transcendental
  functions must go through `libm` to keep it.
- Pressing contacts are handled with a 0.1 mm/s minimum separation speed
  (see `docs/physics-model.md`). This is a modelling choice to revisit if
  frozen-ball shots look wrong.
