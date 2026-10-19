# Changelog

All notable changes to CUE. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[SemVer](https://semver.org/) for the public API (spec §26.3).

A change that alters shot outcomes is called out on its own line and bumps
the physics profile id, even when no signature changes.

## [Unreleased]

### Changed: a 7-foot table, and a table that reads from above

- **The table is a 7-foot bar box** (78 × 39 in), the size most eight-ball
  is played on. The balls, pockets and cushions are unchanged, so the balls
  are 28 % larger on screen and their numbers read at a glance.
- **Depth from light:** the cushions' tops slope up to the rail and catch
  the lamp, the lamp is stronger and the fill lights weaker, so the cloth
  has a pool of light, and the wood is a warmer walnut.
- **Pockets:** each has a leather cap with a brass bead on the rail, in one
  piece, instead of a brass ring that floated over the cushions.

### Changed: shot outcomes (physics profile `baseline-4`)

- **The 7-foot table** changes every shot's geometry. Shots still settle in
  4–6 seconds; the break opens the rack wider (6.8 balls to a rail, 0.70
  pocketed, 20/20 legal).
- **Ordinary shots top out at 6 m/s** (was 7), the same five-cushion power
  shot on the smaller table. Half power is 2 m/s.

### Changed: the table from straight above

- **The camera looks straight down,** with an orthographic projection:
  every part of the cloth at one scale, and angles on screen are angles on
  the table (ADR-0014, superseding ADR-0013's slight tilt).
- **Balls read at a glance:** larger number discs (60 % of the ball), bold
  digits drawn undistorted from above, and clear spacing in 10–15. The
  lacquer is glossy and the ink satin, so reflections never wash a number
  out, and a low room light gives every ball a highlight on its shoulder.
- **One wood frame:** chamfered rails, pocket surrounds cut round each
  hole and a bevel round the outer edge, so the frame reads as one piece
  from above. The floor round the table is a plain dark backdrop.
- **A finer cue:** a brass joint and trim rings, and an ebony forearm with
  four inlaid maple points edged in brass.
- **The guide keeps its width on every screen:** line widths are CSS
  pixels, scaled by the pixel ratio, instead of device pixels.

### Changed: shot outcomes (physics profile `baseline-3`)

- **Shots settle in 4–6 seconds** instead of 7–9: the cloth's rolling
  resistance is 0.018 (was 0.01), a medium-slow cloth, so the long, slow
  roll at the end of a shot is shorter (`docs/physics-model.md`, "Pace").
- **The break is 12.5 m/s** (was 12), which keeps it as strong on the
  slower cloth: about 5.4 balls to a rail and a ball pocketed most of the
  time. Other shots still top out at 7 m/s.

### Added: the `<cue-game>` element

- **`@tuletta/cue/element` defines `<cue-game>`,** CUE as one tag
  (ADR-0015). It sizes itself, downloads the engine only as it nears the
  viewport, pauses while out of view, in a hidden tab or removed from the
  page, and hands its game to the next `<cue-game>` across route changes.
  Attributes set the mode, the match and the preferences, several of them
  live; every event is dispatched as `cue-<name>`, and `.game` is the full
  API. It works from a CDN with one `<script type="module">`.
- **`createCue({ onEvent })`:** one callback for every event, from the
  start of the load.
- The Vite example and the consumer test now use the element.

### Changed: the look, the sound and the cue

- **A premium control layer,** in the style of a dark club room:
  - smoked glass with hairline edges, one champagne accent and serif
    titles;
  - every tool in one pill, and the cue controls in another, with a gold
    Shoot;
  - segmented controls with theme swatches, a sound switch, gold sliders
    and a ticked power gauge at the side;
  - player cards with an initial that turns gold on their turn;
  - close buttons on dialogs, which fade in over a softly blurred table.

  On narrow stages, choices stay on one row.
- **A finer table:** a brass line inlaid along the rails, brass pocket lips,
  and lacquered balls.
- **A calmer sound mix:**
  - ordinary contacts sit well below a hard break;
  - a burst of contacts in one frame blends instead of piling up;
  - cushions, pockets and the tip stay in the background;
  - a change of turn no longer plays a chime, and the foul and win cues
    are quieter;
  - the pocket's clunk comes as the ball drops.
- **A subtler cue:** at full power it draws back 15 cm (was 22). It slides
  back in from 12 cm (was 30) and follows through 3 cm (was 5).
- **Pocketed balls roll in, then drop:** a ball carries on into the pocket
  the way it was going, spinning, tips over the edge and falls out of
  sight. Before, it sank on the spot in a third of a second.

### Changed: shot outcomes (physics profile `baseline-2`)

- **Breaks open the rack.** Full power is now 12 m/s, a strong real
  break; it was 8. Racked balls now sit about half a millimetre apart,
  varied by the rack's seed, instead of frozen. A square full-power break
  now drives about 6 balls to a rail and pockets one most of the time,
  where before it drove about 4 and rarely pocketed anything
  (`docs/physics-model.md`, "The break").
- **Full power is a break only on the break.** The break's full power is
  12 m/s (slider curve `p²`). Every other shot tops out at a hard 7 m/s
  (curve `p^1.6`): half power is 2.3 m/s, a quarter 0.76 m/s. At 12 m/s
  any full-power shot ran into eight cushions.

### Added

- **Animations:**
  - a pocketed ball drops into its pocket instead of vanishing;
  - the cue slides back in when the balls stop;
  - the ball-in-hand ring breathes;
  - banners for the break, each change of turn, fouls (with the reason) and
    wins, and a burst of confetti for the win;
  - each player's group shows as small balls in the scoreboard, which pop
    and dim as they are pocketed.

  Reduced motion turns them off.
- **A fuller aiming guide:** the basic guide also shows a short stretch of
  the object ball's path and the cue ball's tangent, or the bounce off the
  first cushion. The assisted guide runs each path to its next contact.
- **Pull to shoot:** on a wide stage the power slider stands at the side of
  the table; pull it down and let go to shoot. `Enter` on the slider shoots.
- **Fine aiming:**
  - with the mouse, hold `Shift` and the aim turns a tenth as much as the
    pointer moves;
  - on any device, a ridged fine-aim strip in the dock that you drag
    sideways, or turn with the arrow keys;
  - `turnAim(radians)` for hosts' own controls, and `aimAngle` in the
    state.

  The aim now follows the mouse only when it moves, so a fine adjustment
  stays put.
- **Haptics:** a short pulse on the strike and on each pocketed ball, on
  touch devices that can vibrate, while sound is on.
- **Dialogs keep their way out in view:** on a small stage a long card
  scrolls under its buttons instead of hiding them.
- **Accessibility checks in the browser tests:**
  - an axe audit of every dialog and state, in English and Spanish;
  - focus trapping in dialogs, `Esc` and focus return;
  - the shortcuts, and the announcement of each shot's result.
- **A match played through the controls, in the browser tests:**
  - setup;
  - a break with the power slider and the Shoot button;
  - decisions, turns and the call picker;
  - a stalemate;
  - free play's Undo and New rack.

### Changed

- The control layer's download budget is 45 KiB (was 35): 40 for the
  features above, then 45 for the `<cue-game>` element, which only hosts
  that use it download (`docs/benchmarks/phase-1-web.md`).

### Fixed

- The Shoot button could ignore a shot right after the power slider moved.
  It checked the power against the last frame's state, still the old value,
  and showed the "set the power" hint instead of shooting.

- Shift+Tab could leave a dialog for the canvas behind it. This happened
  when a radio group's checked option was not its first, or when focus
  started on a dialog's heading.

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
