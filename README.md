# CUE

**A billiards engine written in Rust, running in the browser through WebAssembly.**

CUE has two parts that ship together:

- **A pool game** that runs inside a web page. There is no sign-up and no
  backend. It has a 3D table, purpose-built physics, a free-play mode and
  local two-player eight-ball.
- **A library** that other sites can embed with one call and a container
  element. Embedding it does not require knowing Rust or Bevy.

> **Status: Phase 1, vertical slice.** You can aim and shoot a full rack on a
> pocketless table, natively or in the browser. There are no pockets, rules or
> audio yet. This README describes only what exists; see the
> [roadmap](#roadmap) for what comes next.

## What works today

- **Event-driven physics.** Sliding, rolling and spin decay are closed-form.
  Collisions are found analytically, so nothing tunnels. A shot is
  **bit-for-bit identical at any frame rate**, which the test suite asserts on a
  full break played at 7, 30, 60 and 144 FPS. See the
  [physics model](docs/physics-model.md).
- **Verified textbook behaviour:** 5/7 rolling speed after a stun shot, natural
  roll at a tip height of 2R/5, the 90° rule, follow and draw, and the rolling
  stopping distance.
- **Browser build:** ≈ 4.1 MiB Brotli, 120 FPS on the reference desktop,
  first frame under a second after clicking Play. Pausing stops the engine
  completely (0 frames). See the [measurements](docs/benchmarks/phase-1-web.md).
- **Embedding API:** `createCue({ container })` with `on`, `pause`, `resume`,
  `rerack` and `destroy`. The vanilla example uses no framework.

## Why it is built this way

| Choice | Reason |
|--------|--------|
| Rust everywhere that matters | Simulation, rules and game logic are the real implementation, not a wrapper. |
| Purpose-built physics, not a generic engine | Sliding-to-rolling, cloth resistance, spin and cushion rebound have closed-form models; a generic solver only approximates them. See [ADR-0002](docs/adr/0002-custom-billiards-solver.md). |
| Engine-free core | Rules and physics compile and test natively in milliseconds, with no GPU or browser. |
| No backend | Static files only. CUE collects no personal data. |

## Architecture

```text
host page ──▶ cue-web ──▶ cue-game ──▶ cue-physics ──▶ cue-core
              (wasm-bindgen) (Bevy)     (solver)        (domain)
```

| Crate | Responsibility | Depends on Bevy? |
|-------|----------------|:----------------:|
| [`cue-core`](crates/cue-core) | Ball identity, table geometry, racks, deterministic RNG; later rulesets and shot reports | no |
| [`cue-physics`](crates/cue-physics) | Event-driven solver: motion regimes, collision detection and response, strikes, versioned profiles | no |
| [`cue-game`](crates/cue-game) | Bevy app: scene, procedural ball textures, aiming, camera, HUD | yes |
| [`cue-web`](crates/cue-web) | WASM boundary: mount, pause/resume, destroy, events | yes |
| [`web/package`](web/package) | `createCue()` ES module wrapper, the only API host pages use | — |

Units are SI. The origin is the centre of the playing surface, `+X` points
towards the foot rail and `+Y` points up.

## Development

You need [rustup](https://rustup.rs). The exact toolchain and the WASM target
are pinned in [`rust-toolchain.toml`](rust-toolchain.toml) and install
automatically on first use.

```bash
cargo test --workspace                              # all tests
cargo test -p cue-physics --release -- --ignored    # 1,000-shot stress test
cargo run -p cue-game --example sandbox             # play natively
cargo fmt --all -- --check                          # formatting
cargo clippy --workspace --all-targets -- -D warnings
```

### Browser build

You also need the `wasm-bindgen` CLI at the exact version in `Cargo.lock`
(the build script checks it), and Node.js for the size report.

```bash
cargo install wasm-bindgen-cli --version 0.2.129 --locked
scripts/build-web.sh                                # → web/dist
python3 scripts/serve.py                            # http://127.0.0.1:8765
```

**Controls:** move the pointer to aim, then press, drag back and release to
shoot (a release with almost no pull cancels). Keyboard: ←/→ aim (Shift for
fine), ↑/↓ power, Space shoot, W/A/S/D spin, C centre the tip, R re-rack, Esc
cancel.

## Roadmap

| Phase | Goal | Exit criterion |
|-------|------|----------------|
| 0 ✓ | Skeleton, CI, first ADRs | `cargo check` and `cargo test` pass on a clean install |
| **1** | Vertical slice in the browser | A simple table with a controllable shot in WASM, plus real size and FPS measurements |
| 2 | Billiards engine | Critical physics cases pass; the solver produces a `ShotReport` |
| 3 | Free Play | A first-time visitor can shoot, understand the result, reset and leave |
| 4 | Local eight-ball | Every rule transition in scope is covered by tests |
| 5 | Visuals, audio, polish | Approved on the device matrix with no physics regressions |
| 6 | Open-source 1.0 | An outside developer can embed CUE using only the public docs |

The full product and engineering specification (in Spanish) is in
[`docs/specification.es.md`](docs/specification.es.md). Architecture decisions
are in [`docs/adr`](docs/adr).

## License

The code is dual-licensed under [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE), at your option. Art and audio assets will carry
their own attribution in `assets/ATTRIBUTION.md`.
