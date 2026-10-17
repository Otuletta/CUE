# ADR-0008 — Controls and the accessible control layer

- **Status:** accepted; the power mechanic stays provisional until the Phase 3
  usability test (resolves ADR-010 and ADR-011 in spec §31 partially)
- **Date:** 2026-10-18

## Context

Spec §10 asks for natural pointer and touch controls that never fire by
accident, a separate way to shoot, a 2D spin selector, keyboard alternatives
for everything, and controls outside the canvas for accessibility (§22). The
host may be any site, with any CSS.

## Decision

**In the canvas,** one gesture works for mouse and touch: aim at a point,
drag *back* for power, release to shoot. A release with almost no pull
cancels. While the cue ball is in hand, pressing *on* it drags it instead.

**Outside the canvas,** the wrapper renders an HTML control layer in a
**shadow root** over the canvas:

- a toolbar (help, undo, new rack, sound, settings, full screen, pause,
  optional exit);
- a dock with a 2D spin selector, a native power slider and a Shoot button;
- a status line and a live region for shot results;
- dialogs for help, settings, confirmation and pause.

Hosts can turn the layer off (`ui: false`) and build their own on the same
public API.

**Commands, not reach-ins.** The layer talks to the engine through the public
API (`setPower`, `shoot`, `setTip`, `undo`…), which `cue-web` turns into
`GameCommand` messages. It reads a per-frame state snapshot instead of
querying the ECS.

**Keyboard shortcuts are split by owner.** In-canvas keys (arrows, Space,
WASD, IJKL, C, U, R, Esc) belong to the game. Layer keys (H, P, M, F, N, O)
never collide with them, so one key press never triggers both.

## Consequences

- The game is fully playable without a pointer, and the shot can always be
  completed with a separate button, as §10.2 requires.
- The shadow root isolates the layer from host CSS in both directions. The
  canvas stays in the light DOM because Bevy finds it by CSS selector.
- The drag-back gesture is the one the author validated on a phone in
  Phase 1. A dedicated mobile power bar is already available as the slider.
  Whether it should replace drag-back on touch is decided by the usability
  test (`docs/usability-test.md`).

## Review conditions

Revisit after the Phase 3 usability test, or if more than one in five new
players fires a shot by accident.
