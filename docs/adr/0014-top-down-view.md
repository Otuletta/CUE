# ADR-0014 — The table from straight above

- **Status:** accepted
- **Date:** 2026-10-19

Supersedes the camera decision of
[ADR-0013](0013-camera-entry-persistence-languages.md) (spec §31 ADR-005).
The rest of ADR-0013 stands.

(This file's number is not the specification's ID ADR-014, challenge URLs,
which is still open.)

## Context

ADR-0013 chose a camera tilted 0.2 rad from straight down, so balls read as
spheres and cushions as raised edges. It also listed the alternative,
straight overhead, and named the usability round as the moment to revisit.

The project's owner reviewed the beta before that round and asked for a
change of direction: more elegant, closer to the look players know from
mobile pool games, but different enough to stand on its own, so that it
belongs in a portfolio. Asked to choose, they picked a **pure top-down
view**, a **modern luxury** look and **calibrated realism**.

From straight above, three things change:

- The perspective tilt made the far half of the table smaller and the near
  pockets larger. A top-down view needs no tilt to frame the table, so it
  can be orthographic: every part of the cloth at one scale.
- Balls are seen from one side only, and small: about 30 CSS pixels across
  on a laptop. Their numbers decide whether the table reads at a glance.
- The tilt used to show the rails' and pockets' depth. From above, depth has
  to come from light: chamfers, bevels and highlights.

## Decision

**One fixed orthographic camera, straight down.**

- **Orthographic:** angles on screen are angles on the table, and the cloth
  is drawn at one scale everywhere. Light, shadows and gloss are still
  rendered in 3D.
- **Orientation:** landscape screens see the table lengthwise with the far
  rail at the top; portrait screens see it rotated so it still fills the
  view. Unchanged from ADR-0013.
- **Clear of the controls:** the host's bars stay off the playing surface,
  as before. The off-axis projection is gone: an orthographic camera slides
  along the table to the same effect, and the view direction cannot change.
- **Picking:** pointer rays come from the projection
  (`viewport_to_world`), so aiming and ball-in-hand work unchanged.

**The table is designed to be seen from above.**

- The frame is one continuous piece of wood: chamfered rails, pocket
  surrounds cut round each hole, and a bevel round the outer edge. The
  pocket holes are deep, with brass lips.
- **Balls:** a white number disc spanning 60 % of the ball, with bold digits
  drawn undistorted when the disc faces up. The lacquer is glossy and the
  ink satin, so the lamp's reflections pass over a number without washing
  it out. Each ball carries a highlight on its shoulder from a low room
  light. That light is a distant point light rather than a directional one:
  it must cast no shadows, or it would draw long streaks across the cloth.
- **The cue:** a maple shaft, a brass joint and trim rings, and an ebony
  forearm with four inlaid maple points edged in brass veneer.
- **The guide:** line widths are in CSS pixels, scaled by the screen's
  pixel ratio, so the guide is equally fine on a phone and a monitor.

> **Amended (2026-10-19), after the owner's first look:** "no queda para
> nada bien", for the table and the balls. Three changes followed:
>
> - **A 7-foot table** (physics profile `baseline-4`): on a 9-foot table seen
>   from above the balls were about 20 CSS pixels across on a laptop. On the
>   bar-box size most eight-ball is played on, they are 28 % larger
>   (`docs/physics-model.md`, "Table size").
> - **Depth from light and form:**
>   - the cushions' tops slope up to the rail and catch the lamp;
>   - the lamp is stronger and the fill lights weaker, so the cloth has a
>     pool of light;
>   - the wood is a warmer walnut.
> - **Pockets:** a leather cap with a brass bead on the rail round each
>   hole, swept in one piece, replaces the brass ring that floated over the
>   cushions.

## Consequences

**Positive**

- The table reads like a designed object at any size, and the aiming
  geometry on screen is exact.
- Ball numbers are legible on a phone. The rack is now as easy to read as
  in the mobile games the owner compared it with.
- Nothing in the physics, rules or API depends on the view.

**Costs and risks**

- No camera move shows depth. Balls can look flat if the lighting is
  wrong; the room light and the gloss carry that weight now.
- A number disc faces up only part of the time while balls roll, as on a
  real table.
- Still no orbit camera (ADR-0013's cost stands).

## Evidence

- Framing: the framing tests in `cue-game/src/camera.rs`, unchanged, and
  the browser tests that click through ball-in-hand and aiming.
- Balls: the tests in `cue-game/src/ball_texture.rs` (digits fit their discs,
  read upright from outside, satin ink and glossy lacquer).
- Off-screen renders: `CUE_VIEW=table|rack|cue cargo run -p cue-game
  --example still --release` renders each view from above.

## Review conditions

- The usability round (spec §27) finds that players cannot judge depth or
  spin from above.
- A target device cannot hold the frame rate with the room light.
