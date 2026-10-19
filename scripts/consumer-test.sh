#!/usr/bin/env bash
# Installs the built npm package the way an outside project would, from its
# tarball, into a fresh copy of web/example-vite, then builds it with Vite and
# checks the output (spec §26.2: test the package in a clean consumer).
#
# Usage: scripts/consumer-test.sh   (after scripts/build-web.sh)
set -euo pipefail
cd "$(dirname "$0")/.."

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

(cd web/dist/package && npm pack --silent --pack-destination "$work" >/dev/null)
tarball=$(ls "$work"/*.tgz)
cp web/example-vite/{index.html,main.js,vite.config.js} "$work/"
cat >"$work/package.json" <<JSON
{ "name": "cue-consumer", "private": true, "type": "module" }
JSON

cd "$work"
vite=$(node -p 'require(process.argv[1]).devDependencies.vite' "$OLDPWD/web/example-vite/package.json")
npm install --no-audit --no-fund --loglevel=error "vite@$vite" "$tarball" >/dev/null
npx vite build --logLevel warn

# The engine, its glue and the wrapper must all be in the build.
wasm=$(ls dist/assets/*.wasm)
test "$(wc -c <"$wasm")" -gt 10000000 || { echo "WASM asset too small: $wasm" >&2; exit 1; }
grep -lq 'createCue' dist/assets/*.js || { echo "wrapper missing from the build" >&2; exit 1; }
# The element module registers itself: a bundler must not drop it as unused.
grep -lq 'cue-game' dist/assets/*.js || { echo "<cue-game> element missing from the build" >&2; exit 1; }
echo "consumer test: $(basename "$tarball") builds with Vite $vite ($(du -h "$wasm" | cut -f1) WASM asset)"
