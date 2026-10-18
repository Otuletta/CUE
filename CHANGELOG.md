# Changelog

All notable changes to CUE. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/) for the public API (spec §26.3).

A change that alters shot outcomes is called out on its own line and bumps
the physics profile id, even when no signature changes.

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
