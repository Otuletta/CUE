# Getting started

From a fresh clone to playing natively, building the browser bundle and
running every test. These commands are the ones CI runs.

## Requirements

| Tool | Why | Install |
|---|---|---|
| [rustup](https://rustup.rs) | The pinned toolchain (`rust-toolchain.toml`) and the `wasm32-unknown-unknown` target install on first use. | `curl https://sh.rustup.rs -sSf \| sh` |
| `wasm-bindgen` CLI 0.2.129 | Must match `Cargo.lock` exactly; the build script checks it. | `cargo install wasm-bindgen-cli --version 0.2.129 --locked` |
| Node.js 22+ | Size reports, the manifest, wrapper tests, the Vite consumer test. | [nodejs.org](https://nodejs.org) |
| Google Chrome | The browser tests. Any Chromium build works through `CHROME_PATH`. | [google.com/chrome](https://www.google.com/chrome/) |
| Python 3 | The local static server. | Usually present. |
| Linux only | Bevy's audio, input and window libraries. | `sudo apt-get install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev` |

## Play natively

```sh
cargo run -p cue-game --example sandbox
```

Press `` ` `` for the developer overlay. The full controls are in
[controls.md](controls.md).

## Build and serve the browser version

```sh
scripts/build-web.sh          # → web/dist (the demo) and web/dist/package (npm)
python3 scripts/serve.py      # → http://127.0.0.1:8765
```

The build ends with a size report and fails if a download budget is
exceeded: 4.6 MiB for the engine and 35 KiB for the control layer, Brotli
11. `CUE_SIZE_BUDGET=off` reports without failing, for experiments.

## Test

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                            # physics, rules, game
cargo test -p cue-physics --release -- --ignored  # 1,000-shot stress test
node --test 'web/package/test/*.test.mjs'         # wrapper: strings, settings
scripts/consumer-test.sh                          # the npm tarball in a Vite app
(cd web/e2e && npm ci && npm test)                # the page lifecycle in Chrome
cargo deny check                                  # licences and advisories
```

The browser tests drive the bundle in `web/dist`, so build it first. They
start Chrome headless under the strict Content Security Policy from
[web-integration.md](web-integration.md#content-security-policy) and fail on
any violation or uncaught error. To use another Chromium build:

```sh
CHROME_PATH="/path/to/chrome" npm test --prefix web/e2e
```

## Look at the table without a window

```sh
CUE_STILL=table.png CUE_VIEW=table cargo run -p cue-game --example still --release
```

`CUE_VIEW` is `table`, `rack` or `cue`; `CUE_THEME` and `CUE_SIZE=1600x900`
also apply. This renders off-screen, so it works on a locked machine or a
CI runner.

## Where things are

See [architecture.md](architecture.md) for how the crates fit together, and
[api.md](api.md) for the embedding API.
