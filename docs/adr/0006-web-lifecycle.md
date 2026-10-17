# ADR-0006 — Web lifecycle: one instance per page, pause instead of remount

- **Status:** accepted for 1.0 (relates to open decision ADR-018 in spec §31: multi-instance)
- **Date:** 2026-10-17

## Context

Spec §18.4 asks that leaving and re-entering `/cue` in a single-page app
behave like a clean start, with no hidden render loop left behind. Spec §17.6
already allows one instance per page if the engine cannot do better.

Bevy runs on winit. winit creates **one event loop per process**, and in a
browser the process is the page. After `AppExit` the loop stops for good. A
second `EventLoop` cannot be created without reloading the page.

## Decision

1. **One live instance per page.** A second `mount` while one is live fails
   with `INSTANCE_ALREADY_MOUNTED`.
2. **`destroy()` is final for that page load.** It stops the loop, releases
   the surface, removes the canvas and emits `destroyed`. A later `mount`
   fails with `RELOAD_REQUIRED`. Every failure is an explicit error; nothing
   fails silently.
3. **`pause()` is the SPA navigation primitive.** It freezes simulation and
   input and switches winit to reactive mode with an unbounded wait, so the
   engine runs **zero frames** while paused. `resume()` wakes it at once
   through winit's event-loop proxy. Hosts that route away from `/cue` and
   back should pause, hide the container and resume.
4. **Hosts that need true teardown and remount** (for example to free memory)
   can load CUE in an `<iframe>`. Removing the frame frees everything. This is
   documented as the supported isolation strategy, not hidden.

## Evidence (Phase 1, Chromium, vanilla host)

- `mount → shot → pause → resume → rerack → destroy` completed with no
  console errors. After `destroy` no canvas remains in the DOM and `destroy()`
  called twice returns the same promise.
- A remount attempt after `destroy` returns `RELOAD_REQUIRED`.
- While paused the engine frame counter (`stats().frames`) does not advance.

## Consequences

- The portfolio integration (Phase 6) uses pause/resume across routes, or an
  iframe if the portfolio's router unmounts the container.
- Multi-instance (spec ADR-018) remains out of scope until winit or Bevy can
  run several apps per page.
