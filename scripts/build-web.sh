#!/usr/bin/env bash
# Builds the browser bundle into web/dist:
#
#   web/dist/
#   ├── index.html, main.js, styles.css   vanilla example host
#   └── package/
#       ├── index.js                      createCue() wrapper
#       └── pkg/cue_web.js, cue_web_bg.wasm
#
# Usage: scripts/build-web.sh            (web-release profile)
#        PROFILE=release scripts/build-web.sh
#        WASM_OPT=1 scripts/build-web.sh (also run wasm-opt -Oz)
#
# wasm-opt is opt-in: it shrinks the raw binary but makes the Brotli/gzip
# download larger (docs/benchmarks/phase-1-web.md).
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${PROFILE:-web-release}"
OUT=web/dist
TARGET=wasm32-unknown-unknown

# The CLI must match the wasm-bindgen crate exactly, or the glue breaks.
locked=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/version = |"/, ""); print; exit }' Cargo.lock)
installed=$(wasm-bindgen --version | awk '{ print $2 }')
if [[ "$locked" != "$installed" ]]; then
  echo "wasm-bindgen CLI $installed does not match Cargo.lock ($locked)." >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version $locked --locked" >&2
  exit 1
fi

# Anonymise source paths embedded in the binary (log and panic locations), so
# shipped errors never reveal a developer's file system (spec §23.4). This
# replaces the target rustflags in .cargo/config.toml, so repeat those here.
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\" \
--remap-path-prefix=$HOME/.cargo/registry/src=/deps --remap-path-prefix=$HOME/.rustup=/rustup \
--remap-path-prefix=$PWD=/cue"

cargo build -p cue-web --target "$TARGET" --profile "$PROFILE" --locked

rm -rf "$OUT"
mkdir -p "$OUT/package/pkg"
wasm-bindgen --target web --no-typescript --out-dir "$OUT/package/pkg" \
  "target/$TARGET/$PROFILE/cue_web.wasm"

wasm="$OUT/package/pkg/cue_web_bg.wasm"
if [[ "${WASM_OPT:-0}" == 1 ]]; then
  before=$(wc -c <"$wasm")
  wasm-opt -Oz --all-features "$wasm" -o "$wasm.opt" && mv "$wasm.opt" "$wasm"
  echo "wasm-opt: $before -> $(wc -c <"$wasm") bytes"
fi

cp web/package/src/index.js "$OUT/package/index.js"
cp web/example-vanilla/{index.html,main.js,styles.css} "$OUT/"

node scripts/size-report.mjs "$OUT"
