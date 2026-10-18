# Architecture

CUE is a Rust workspace of five crates and a small JavaScript package. Each
layer depends only on the ones below it, and the two at the bottom know
nothing about engines, GPUs or browsers (spec §15.3).

```text
host page
   │  createCue({ container })                       web/package (ES module)
   ▼
cue-web     WASM boundary: commands in, events out, a state snapshot per frame
   │
cue-game    Bevy app: scene, input, camera, free-play and eight-ball sessions
   │                     │
cue-physics  solver      cue-audio  modal synthesis (no files)
   │
cue-core    domain: balls, pockets, rails, table, racks, ShotReport, rules
```

| Crate | Knows about | Never touches |
|---|---|---|
| `cue-core` | WPA geometry, balls, the `ShotReport`, the eight-ball rules | Bevy, physics, the browser |
| `cue-physics` | Motion regimes, contacts, pockets, the head-string sensor, frozen balls | Bevy, the rules, the browser |
| `cue-audio` | Rendering sounds to samples | Playback, the browser |
| `cue-game` | Bevy: rendering, input, camera, quality, sessions | WASM bindings, the DOM |
| `cue-web` | `wasm-bindgen`, Web Audio, the command queue, events | Game rules (it only reports them) |

## One shot, end to end

1. **Input** (`cue-game/shot.rs`): pointer, touch, keyboard or a host command
   charges the aim. Shooting fixes a `Strike`: direction, power and tip. In
   eight-ball it also fixes the call and the table layout.
2. **The swing:** the cue swings in, and the `Strike` reaches the solver at
   contact. With reduced motion it reaches it at once.
3. **Physics** (`cue-physics/world.rs`): the solver finds the next event in
   closed form and jumps to it. Events are contacts, pocketings,
   regime changes and the head-string sensor. The renderer samples any
   instant in between. The outcome does not depend on the frame rate.
4. **Report:** when every ball stops, the world hands over an immutable
   `ShotReport` with first contacts, rails (marking frozen balls), pocketings
   and head-string crossings.
5. **Rules** (`cue-core/rules`): the eight-ball state machine turns the call,
   the report and the earlier layout into a `Ruling`. It has no engine
   anywhere in it.
6. **Session** (`cue-game/eight.rs`): carries the ruling out on the table. It
   sets ball in hand and its region, spots the 8, re-racks and keeps score.
7. **Bridge** (`cue-web`): publishes the new state, then emits `ruling`,
   `foul`, `turn-changed` and the rest, and plays the matching cue.
8. **Overlay** (`web/package`): updates the scoreboard, announces the ruling,
   and shows a decision or result panel when needed.

## Principles

- **The simulation is the single source of truth.** Systems ask it ("is the
  table at rest?") instead of keeping parallel flags that could drift.
- **Determinism.** The same strike on the same table gives bit-identical
  results at any frame rate; the tests assert it.
- **Facts are not rules.** The physics reports what happened, frozen balls
  included. Only `cue-core` judges it.
- **Presentation is replaceable.** Themes, quality profiles, the camera and
  the overlay never change a result.
- **Nothing to license.** Every texture, mesh and sound is generated in code
  ([assets.md](assets.md)).

Decisions and their trade-offs are recorded in [`docs/adr`](adr).
