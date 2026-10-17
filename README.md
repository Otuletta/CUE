# CUE

**A billiards engine written in Rust, running in the browser through WebAssembly.**

CUE has two parts that ship together:

- **A pool game** that runs inside a web page. There is no sign-up and no
  backend. It has a 3D table, purpose-built physics, a free-play mode and
  local two-player eight-ball.
- **A library** that other sites can embed with one call and a container
  element. Embedding it does not require knowing Rust or Bevy.

> **Status: Phase 0, project skeleton.** Nothing is playable yet. This README
> describes only what exists; see the [roadmap](#roadmap) for what comes next.

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
| [`cue-physics`](crates/cue-physics) | Ball state, motion regimes, versioned physical profiles; later the solver | no |
| [`cue-game`](crates/cue-game) | Bevy app: scene, input, camera, audio, presentation | yes |
| [`cue-web`](crates/cue-web) | WASM boundary: mount, lifecycle, events | yes |

Units are SI. The origin is the centre of the playing surface, `+X` points
towards the foot rail and `+Y` points up.

## Development

You need [rustup](https://rustup.rs). The exact toolchain and the WASM target
are pinned in [`rust-toolchain.toml`](rust-toolchain.toml) and install
automatically on first use.

```bash
cargo test --workspace                              # all tests
cargo run -p cue-game --example sandbox             # native window with a racked table
cargo fmt --all -- --check                          # formatting
cargo clippy --workspace --all-targets -- -D warnings
```

## Roadmap

| Phase | Goal | Exit criterion |
|-------|------|----------------|
| **0** | Skeleton, CI, first ADRs | `cargo check` and `cargo test` pass on a clean install |
| 1 | Vertical slice in the browser | A simple table with a controllable shot in WASM, plus real size and FPS measurements |
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
