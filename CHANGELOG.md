# Changelog

All notable changes to CUE. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/) for the public API (spec §26.3).

A change that alters shot outcomes is called out on its own line and bumps
the physics profile id, even when no signature changes.

## [1.0.0-beta.2] - 2026-10-18

Hardening before 1.0: the events now match the specification's table
(§16.4), loads can be cancelled, failures after start are reported, dense
screens render fewer pixels, and the page lifecycle is tested in a real
browser on every pull request. No change to shot outcomes: the physics
profile is still `baseline-1`.

### Changed (breaking for hosts)

- The event **`rack-over` is now `game-over`**, as in the specification.
  The refusal code `rack-over` keeps its name.
- **`shot-start`** carries `{ shotId, shooter }`: the id its `shot-end`
  repeats, and the seat that shot (`null` in free play).
- **`turn-changed`** adds `previous`, the outgoing player.
- **`paused` and `resumed`** carry `{ reason }`: `user` for CUE's own pause
  control, else what the host passed to `pause(reason)` or
  `resume(reason)`, `host` by default.
- **`error`** carries `{ code, message }`, with a plain description that is
  safe to show.

### Added

- **`signal`:** an `AbortSignal` that cancels a load. `createCue` rejects
  with `ABORTED` (an `AbortError`), the download stops, nothing is left
  behind, and the page can call `createCue` again at once, even within the
  same task (React's StrictMode).
- **Failures after start:** an engine panic (`ENGINE_FAILED`) or a lost
  WebGL context (`CONTEXT_LOST`) is reported by an `error` event. The
  built-in UI shows a message with Reload. Methods then do nothing instead
  of throwing, so host code keeps working.
- **`render.pixelRatioCap`** (default 2): at most that many pixels per CSS
  pixel, so a 3× phone renders about half as many pixels. The `low` tier
  renders at most 1.5.
- **Browser tests** (`web/e2e`), in CI and in the release workflow. They
  run in headless Chrome under the strict CSP and cover:
  - mounting, a shot, pausing, moving and destroying;
  - cancelling, a failed download, a lost context;
  - the pixel ratio cap on a real 3× screen.
- **Download budgets:** the build fails if the engine passes 4.6 MiB or the
  control layer 35 KiB, Brotli 11.
- **ADR-0013** records the camera, first mode, persistence, languages and
  resolution. The ADR index maps every open decision in spec §31 to the
  record that answers it.

### Fixed

- The first frame rendered at Bevy's default 1280 × 720 before the canvas
  took its real size. On a 3× phone that was a 3840 × 2160 render with 4×
  MSAA. The engine now starts at the canvas's size.
- An invalid `mode` threw only after CUE had claimed the page's instance.
  Every later `createCue` then failed with `INSTANCE_ALREADY_MOUNTED`.
- `createCue` after a `destroy()` downloaded the whole engine again just to
  reject with `RELOAD_REQUIRED`. It now rejects at once.
- A mount that came before the engine's last frame (in a hidden tab)
  reported `INSTANCE_ALREADY_MOUNTED` instead of `RELOAD_REQUIRED`.
- With `ui: false`, a failed load left an empty root in the host's
  container.
- Leaving the immersive layout restored styles through the `style`
  attribute, which a strict `style-src` can block. It now uses CSSOM.
- The single-page-app example in the docs failed under React's StrictMode
  with `INSTANCE_ALREADY_MOUNTED`. It now keeps the instance as a promise.

## [1.0.0-beta.1] - 2026-10-18

The first public build: everything planned for 1.0 is in, and testing on
real devices is under way.

### Added

- **Physics** (physics profile `baseline-1`):
  - an event-driven solver with closed-form sliding, rolling and spinning,
    so outcomes are bit-identical at any frame rate;
  - Han-style cushions, speed-dependent ball–ball friction (throw) and
    squirt;
  - pockets with WPA facing angles;
  - a head-string sensor and frozen-ball rail contacts (WPA 2.7).
- **Free play:**
  - ball in hand and one-level undo, as an exact snapshot even mid-shot;
  - automatic re-racking;
  - three aiming-guide levels.
- **Eight-ball** for two players on one device:
  - a pure rules engine for WPA *Rules of Play* (effective 2025-09-15),
    section 4, with 38 rule tests;
  - `eight-ball-wpa` and `eight-ball-casual` profiles;
  - calls proposed from the aim, decisions after the break, matches with
    alternating breaks;
  - [rule-by-rule conformance notes](docs/rules-eight-ball.md).
- **Visuals,** generated in code:
  - numbered balls;
  - an eight-section cue whose swing lands the strike at contact;
  - diamonds, foot spot and cloth grain;
  - contact shadows fitted to the table;
  - three themes, including high contrast.
- **Graphics quality** `auto`, `high`, `medium` and `low`, with automatic
  stepping down.
- **Procedural sound:** impacts, plus quiet turn, foul and win cues.
- **Accessible controls** in English and Spanish:
  - a full keyboard path and real HTML controls in a shadow root;
  - live announcements;
  - reduced motion respected.
- **The web package `@tuletta/cue`:**
  - `createCue`, with lifecycle methods (`start`, `pause`, `resume`,
    `attach`, `resize`, `destroy`), table and match methods, and typed
    events;
  - a per-frame state snapshot and TypeScript declarations;
  - works under a strict CSP and in Vite with no plugin.
- **Documentation:** API, web integration, browser support, architecture,
  physics model, controls, accessibility, assets, releasing, and 12 ADRs.
- **CI:** formatting, lints, tests, a 1,000-shot stress test, licence
  checks, the browser bundle, wrapper tests and a clean-consumer Vite
  build. A tag-driven release workflow.

### Known limits

- One instance per page; `destroy()` is final until reload.
- WebGL 2 only. Resolution scaling is not one of the quality profiles.
- Browser support beyond Chromium on macOS is
  [not yet verified](docs/browser-support.md).
