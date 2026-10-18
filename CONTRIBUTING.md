# Contributing

Thank you for helping. Small, focused pull requests with tests are the
easiest to review and merge.

## Setting up

[docs/getting-started.md](docs/getting-started.md) lists the tools, the
build, and every check CI runs. Run them before opening a pull request.

## What a good pull request has

- **A clear reason.** For a bug: how to reproduce it. For a feature: which
  part of the [specification](docs/specification.es.md) or which ADR it
  serves.
- **Tests.** The project is test-first where it can be:
  - **Physics:** a scenario in `crates/cue-physics/tests` that states the
    behaviour in words a player would recognise. If a change alters shot
    outcomes, bump the physics profile id (`baseline-1`), explain the change
    in the pull request and add a changelog line.
  - **Rules:** a case in `crates/cue-core/tests/eight_ball.rs`, named after
    the WPA rule (`rule_4_3_d_…`), and an update to
    [docs/rules-eight-ball.md](docs/rules-eight-ball.md) if the conformance
    table changes.
  - **Wrapper and controls:** `web/package/test`. Every interface string
    must exist in English and Spanish; the tests check it.
  - **Page lifecycle:** `web/e2e`, in headless Chrome against the built
    bundle. A change to loading, pausing, events or teardown needs a case
    there.
- **No asset files** unless agreed first. CUE generates its textures, meshes
  and sounds. An exception goes in [docs/assets.md](docs/assets.md) with its
  source and licence before merging.
- **Accessibility kept:** new controls must work with a keyboard and a
  screen reader ([docs/accessibility.md](docs/accessibility.md)).
- **Documentation where behaviour changes:** [docs/api.md](docs/api.md) for
  the public API, and a new ADR for a decision with trade-offs
  ([docs/adr](docs/adr)).

## Style

- Code, comments and documentation are in English. The interface is in
  English and Spanish.
- `cargo fmt` and `cargo clippy -D warnings` must be clean. Comments explain
  why, not what.
- Commit messages: a short imperative summary line, then the reasoning.

## Labels

Issues use `bug`, `physics`, `rules`, `rendering`, `accessibility`,
`documentation`, `good first issue` and `breaking change`.

## Licence

By contributing you agree that your work is dual-licensed under
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE), like the rest of CUE.
