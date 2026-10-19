# ADR-0013 — Camera, first mode, persistence, languages and resolution

- **Status:** accepted; the camera decision is superseded by
  [ADR-0014](0014-top-down-view.md) (2026-10-19)
- **Date:** 2026-10-18

Records four decisions that spec §31 left open and that the code settled
during Phases 1–6:

- ADR-005, the camera;
- ADR-009, the first mode;
- ADR-013, local persistence;
- ADR-019, the interface languages.

It also records the rendering resolution cap that spec §12.5 asks for
("respetar `devicePixelRatio` con límite configurable"), which amends the
quality profiles of [ADR-0011](0011-visual-direction-and-graphics-quality.md).

## Context

Each of these was decided in code without a record. Each already has tests
or measurements behind it, and the usability round still to come could
reopen some of them. Writing them down now gives that round a baseline to
argue against.

## Decision

### Camera: one fixed camera with a slight tilt (ADR-005)

> **Superseded by [ADR-0014](0014-top-down-view.md) (2026-10-19):** the
> camera now looks straight down, with an orthographic projection.

The alternatives were straight overhead and a slight oblique view. The
camera is **slightly oblique**: tilted 0.2 rad (about 11°) from straight
down.

- **Why the tilt:** balls read as spheres and the cushions as raised edges.
  The cue, its shadow and the ghost ball stay readable from every side.
- **Why no more:** the tilt barely foreshortens the near–far extent, so
  aiming lines keep their apparent angles.
- **What stays fixed:** there is one camera, not an orbit camera. Portrait
  screens see the table rotated so it still fills the view.
- **Clear of the controls:** the host's bars are kept off the playing
  surface by an off-axis projection, which leaves the view direction and
  aiming unchanged (ADR-0011).

### First mode: straight into Free Play (ADR-009)

The alternatives were Free Play at once, or a selector first. `createCue`
**starts in Free Play** unless the host asks for `mode: 'eight-ball-local'`.

- On a first visit the how-to-play dialog opens over the table, then the
  first shot is one gesture away.
- The match setup is one button (or `N`) away, and Exit is there whenever
  the host provides `onExit`.
- A selector is the host's to show. The spec's flow (§7.2) has the portfolio
  present CUE and offer «Jugar»; a second choice inside CUE would only add a
  step before the first shot.

### Persistence: preferences only (ADR-013)

The alternatives were preferences only, or also some game state. CUE keeps
**preferences only**, under two `localStorage` keys:

- `cue:v1:settings`: theme, aiming guide, graphics quality and sound,
  validated on every read;
- `cue:v1:tutorial-seen`: whether the how-to-play dialog has been shown.

No rack, match, score or player name is stored. A reload starts a fresh
table. The reasons:

- Player names never leave memory: the base experience needs no personal
  data (spec §2.5, point 6).
- A match restored after a reload would need versioning against rule and
  physics profiles, for little gain on a portfolio visit.
- `persist: false` turns all storage off, and `localStorage` failures
  (private modes, quotas) are ignored.

### Languages: English and Spanish (ADR-019)

The alternatives were English only, or Spanish and English. The interface
ships in **English and Spanish**, with the same strings in both; a test
fails if either language is missing a string or has a different shape.

- The `locale` option wins. Otherwise the browser's languages are tried in
  order, and English is the fallback.
- Every event carries stable codes (fouls, pockets, choices), never
  translated text, so hosts can translate into other languages.
- Code, documentation and commit messages are in English. The specification
  stays in Spanish.

### Resolution: at most two pixels per CSS pixel

Phones with 3× screens would otherwise render 2.25 times the pixels of a 2×
render, for detail that is hard to see on a table.

- **The cap:** CUE renders at most `render.pixelRatioCap` pixels per CSS
  pixel. The default is 2.
- **The low tier:** it also caps the ratio at 1.5. This adds a resolution
  column to the profiles of ADR-0011: High and Medium are bounded only by
  the cap.
- **How:** past the cap, the canvas gets a smaller CSS box inside its frame,
  so the browser sizes the drawing buffer to the cap, and a CSS transform
  scales the canvas back up. The engine reads pointer positions in the
  canvas's own coordinates (`offsetX`, `offsetY`), so input lands where it
  looks.
- **The first frame:** the engine starts at the canvas's real size instead
  of Bevy's default 1280 × 720. That default made the first frame on a 3×
  phone a 3840 × 2160 render, with 4× MSAA on top.

## Consequences

**Positive**

- A first-time visitor reaches the first shot in one step, on a camera
  that needs no handling.
- Nothing personal is ever stored, and there is nothing to migrate between
  versions.
- On dense screens the cap roughly halves the pixels drawn per frame at the
  default, and more at the low tier.

**Costs and risks**

- No orbit camera: players used to 3D pool games may want to look along the
  shot. It can be added later as an option, without changing aiming.
- A reload loses a match in progress.
- Above the cap, the table is slightly softer than the screen could show.

## Evidence

- Camera and framing: the framing tests in `cue-game/src/camera.rs` (no
  bars, bars over the rails, tall bars, absurd insets) and the off-screen
  renders in `docs/images/`.
- Persistence: `web/package/test/settings.test.mjs`.
- Languages: `web/package/test/i18n.test.mjs`.
- Resolution: `web/e2e`, on a real 3× screen
  (`--force-device-scale-factor=3`). It asserts:
  - a 640 × 360 canvas draws a 1280 × 720 buffer from its first frame on;
  - the low tier draws 960 × 540;
  - a click at the visible centre lands at the canvas's own centre.

## Review conditions

- **Camera and first mode:** revisit if the Phase 3 usability round finds
  that players misread distances or angles, or look for a mode choice
  before they shoot.
- **Persistence:** revisit if a match mode longer than a few racks
  arrives.
- **Resolution:** revisit the default cap once the device round has
  measured frame rates on 3× phones.
- **The power mechanic** (spec ADR-010) is not settled here. It stays
  provisional in ADR-0008 until the usability round.
