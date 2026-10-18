# ADR-0011 — Visual direction, procedural assets and graphics quality

- **Status:** accepted
- **Date:** 2026-10-18

## Context

Phase 5 turns a working engine into something people want to look at (spec
§7.4, §12, §13). The spec asks for:

- legible numbers on the balls;
- a cue with a distinguishable tip and a short stroke;
- discreet cloth texture and soft lighting;
- quality profiles that never touch the physics (§21.4);
- feedback sounds for fouls, turns and wins;
- an inventory of every asset's origin and licence (§12.3).

It also left two problems open. On a phone the overlay's bars covered part of
the cloth: the rack in portrait, and the cue ball's area under the dock. And
on WebGL2 the balls cast almost no shadow.

## Decision

### Everything is generated in code

Nothing is downloaded or licensed separately: no images, no fonts, no
sounds. See [`docs/assets.md`](../assets.md).

- **Ball numbers** come from a small stroke font designed for CUE: lines and
  circular arcs, rendered as distance fields into the white discs. Each ball
  has two discs. A disc reads upright with the pole as "up", so on a stripe
  the band runs under the number. Tests check that every digit is distinct,
  that numbers are not mirrored on the sphere, and that two-digit numbers fit
  inside their disc.
- **Textures** are 512 × 256 for balls and 256 × 256 for the cloth grain.
  Each carries a full mip chain, averaged in linear light, so small balls do
  not shimmer and keep their brightness.
- **Racked balls** start with their number facing up, each with a small fixed
  twist, so a fresh rack can be read and looks set by hand. After that their
  orientation follows the simulated spin.
- **The table** gains diamond sights at the WPA positions, a marked foot
  spot, and a faint, tileable cloth grain.
- **The cue** is eight tapered sections, from a chalked tip and a ferrule to a
  rubber bumper, at real dimensions (1.47 m).
- **Sounds:** besides the impact sounds, three short tonal cues mark a turn,
  a foul and a won rack (a falling minor third and a rising major chord).
  They are quiet, and each has a visible counterpart.

### The stroke delays the strike, never changes it

Pressing Shoot fixes the strike: direction, power, spin and, in eight-ball,
the call. The stick then swings forward with an accelerating ease. The
impulse is applied when the tip meets the ball, after a swing timed to the
strike's speed, between 50 and 300 ms. A short follow-through comes after.
Input that could change the table is ignored during the swing.

With `prefers-reduced-motion` the strike is immediate (spec §7.3). The wrapper
passes the preference to the engine.

### The camera keeps the playing surface clear of the controls

The overlay measures its top bar and dock and reports them to the engine as
view insets. The camera then:

- fits the **playing surface** inside the free band between them;
- lets the cushions and rails run under the bars;
- keeps the whole table on the canvas.

When the bars are taller than the rails, the table shrinks a little and moves
to the middle of the free band. The move uses an **off-axis projection**
(Bevy's sub camera view), so the viewing direction, and with it aiming, is
unchanged. Pointer rays use the same matrix.

> **Amended (2026-10-18), lighting and materials:** the table is now lit
> by a pool-table lamp. Three shades hang over the playing surface as spot
> lights without shadows, which would cost a shadow map each on WebGL2.
> They brighten the middle of the cloth, leave the room dim, and put the
> shades' highlights on every ball. The key light below stays, softer, for
> the contact shadows. The rails are varnished wood with a procedural
> grain, the pockets have leather lips and dark drops, and a table body
> and a floor stand under it, all generated in code like the rest.

### Shadows fitted to the table

On WebGL2, Bevy has a single shadow cascade, which by default spreads over
150 m. CUE limits it to 7 m, about what the camera sees, so the same
2048-texel map gives each ball a crisp contact shadow at no extra cost.

### Quality profiles change only anti-aliasing and shadows

| Profile | MSAA | Shadows |
|---|---|---|
| High | 4× | 2048-texel map |
| Medium | 4× | 1024-texel map |
| Low | off | off |
| Auto | starts High, steps down | — |

> **Amended by [ADR-0013](0013-camera-entry-persistence-languages.md)
> (2026-10-18):** every profile renders at most `render.pixelRatioCap`
> pixels per CSS pixel (default 2), and Low at most 1.5. Resolution is the
> only addition; the physics is still untouched.

**Auto** works locally, with no telemetry and no permissions:

- It waits through a 3-second warm-up after starting and after every change,
  while shaders compile.
- It judges each 2-second window by its **median** frame, so a single hitch
  does not count.
- It ignores frames longer than 250 ms, which come from a hidden tab or a
  debugger.
- It steps down one level when the median is slower than 45 FPS, and never
  steps up on its own.

The setting is saved with the other preferences (`cue:v1:settings`).

## Consequences

**Positive**

- Nothing to license, attribute or download. The bundle grew by about 20 KiB
  Brotli for all of the above.
- On a phone the HUD no longer covers balls, and aiming is unaffected.
- Visual changes can be checked without a screen. `examples/still.rs`
  renders the table, a rack close-up or the cue to a PNG through an
  off-screen target, and works on a locked machine or in CI.
- Graphics and physics stay separate: no profile can change a simulation
  result.

**Costs**

- Ball textures take about 11 MB of GPU memory with their mips, against
  about 3 MB before.
- The procedural font is CUE's own. It is legible, but plainer than a
  traditional ball font.
- The strike now lands up to 300 ms after Shoot. Players feel this as the
  stroke; with reduced motion there is no delay.
- Resolution scaling is not one of the profiles. On the web, winit sizes the
  canvas from `devicePixelRatio`, and Bevy's scale-factor override does not
  reduce the render resolution there. Rendering below the device ratio would
  need canvas sizing in the wrapper, which is left for later.

## Evidence

- `cargo test -p cue-game` covers:
  - the font, the mip chain and the resting orientation (`ball_texture`);
  - the grain's tiling (`textures`);
  - framing with and without insets (`camera`);
  - the swing: a strike only at contact, swing timing, reduced motion and
    refusals (`shot`);
  - Auto's warm-up, median, hidden-tab and never-up rules (`quality`).
- `cargo test -p cue-audio`: the new cues decay, stay in tune between
  variants, and rise (win) or fall (foul).
- Off-screen renders of every theme with `examples/still.rs`.

## Review conditions

- Device testing on the compatibility matrix (spec §21.5), which is the exit
  criterion of Phase 5.
- If Auto steps down on devices that play smoothly, the threshold or the
  warm-up needs to change.
- If texture memory matters on low-end phones, Low could use smaller ball
  textures.
