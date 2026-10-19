# Phase 1 — web measurements

First real numbers for the browser build. They are the input to ADR-0003 and
replace every size or speed estimate in the specification.

**Machine:** Apple M4 Pro, macOS 27.2, Chromium (Claude desktop browser pane),
WebGL 2 via ANGLE/Metal, 120 Hz display.
**Build:** `scripts/build-web.sh`, Rust 1.99.1, Bevy 0.20.0,
wasm-bindgen 0.2.129, binaryen 133.
**Served:** locally over HTTP, uncompressed. Transfer times therefore exclude
the network; compressed sizes are computed offline (`scripts/size-report.mjs`).

## Bundle size

Each step builds on the previous one. Sizes are for `cue_web_bg.wasm`.

| Step | Change | Raw | gzip -9 | Brotli 11 |
|---|---|---:|---:|---:|
| 1 | Bevy default features, `opt-level = "s"`, thin LTO, debug info stripped | 91.6 MiB | 13.3 MiB | 7.36 MiB |
| 2 | + all symbols stripped (`strip = true`), fat LTO | 37.6 MiB | 8.85 MiB | 5.59 MiB |
| 3a | step 2 + `wasm-opt -Oz` | 24.6 MiB | — | 5.93 MiB |
| 3b | step 2 + `wasm-opt -Os` | 33.2 MiB | — | 5.95 MiB |
| **4** | **step 2 + only the Bevy features CUE uses** | **28.9 MiB** | **6.71 MiB** | **4.11 MiB** |
| 4 + Oz | step 4 + `wasm-opt -Oz` | 19.2 MiB | 7.05 MiB | 4.39 MiB |

JS glue: 108 KiB raw, 12.8 KiB Brotli. The `createCue` wrapper is 1.6 KiB
Brotli.

### Findings

1. **Most of the first build was metadata.** The `name` section held 54 MiB of
   the 91.6 MiB. Stripping symbols is free and mandatory.
2. **`wasm-opt` makes the download *larger*.** It shrinks the raw binary by a
   third but adds about 7 % after Brotli or gzip, so it is **off**. Revisit if
   raw size starts to matter for compile time or memory on low-end phones.
3. **Feature trimming saves a quarter of the compressed size** and removed
   both RUSTSEC advisories from the dependency tree (zstd decoder,
   unmaintained font parser). Dropped: audio, glTF, scenes, animation,
   picking, sprites, gamepads, PNG and the tonemapping LUTs (the camera uses
   the LUT-free ACES fitted operator).

**Current download: ≈ 4.1 MiB Brotli** for the engine. The specification set
no number before measuring. This one is acceptable for an opt-in "Play"
button, but too large to preload on the portfolio home page, which the spec
already forbids.

### As served by GitHub Pages

The live demo (`https://otuletta.github.io/CUE/`) is served with the correct
`application/wasm` type, so browsers can compile while downloading. Pages
compresses with **gzip only, never Brotli**, so visitors download about
**6.7 MiB** (7,047,907 bytes), not 4.1 MiB. A host that serves Brotli
(Cloudflare, Netlify, or the portfolio's own) would save about 39 %. This
feeds the hosting choice in Phase 6. `Cache-Control: max-age=600` is Pages'
default; versioned file names would allow long-lived caching.

From the public URL, desktop Chromium on a fast connection: first frame
**716 ms** after clicking Play, 120 engine frames per second, empty console.

### Later phases

| Phase | Engine `.wasm` (Brotli) | Added |
|---|---:|---|
| 1 | 4.11 MiB | baseline after trimming |
| 2 | 4.12 MiB | pockets, Han cushions, throw, squirt, shot reports (+11 KiB) |
| 3 | 4.14 MiB | procedural audio, command bridge (+14 KiB); the JS control layer adds ≈ 14 KiB |
| 6 | 4.17 MiB | eight-ball, visuals, lifecycle hardening; the control layer is 32 KiB |

Since `1.0.0-beta.2` the build enforces budgets: **4.6 MiB** for the engine
and **35 KiB** for the control layer, Brotli 11 (`scripts/size-report.mjs`).
Raising one is a decision, to be recorded here.

- **Control layer 35 → 40 KiB (gameplay pass, after `1.0.0-beta.2`).** The
  pass added banners and a win celebration, the players' balls in the
  scoreboard, a fine-aim strip, haptics and fine aiming with `Shift`. That
  took the layer from 32 to 35.3 KiB. It is about 0.1 % of the engine's
  download, so the features were worth more than the bytes.
- **Control layer 40 → 45 KiB (redesign, after `1.0.0-beta.2`).** The
  premium controls took the layer to 38.3 KiB, and the `<cue-game>` element
  (ADR-0015) adds 3.3 KiB: 41.6 KiB in all. Hosts that call `createCue`
  never download the element. The engine is 4.18 MiB with the top-down
  redesign (ADR-0014): its procedural textures are generated at start, so
  they cost no download.

## Runtime

| Metric | Result |
|---|---|
| Time from Play click to first frame (local, warm cache) | 415–854 ms across runs |
| Frame rate, desktop, 16 balls, idle and during a break | 120 FPS (display-locked), session p5 ≥ 111 |
| Engine frames while paused (2 s window) | **0** |
| Engine frames in the first second after resume | 120 |
| Console output across mount → shot → pause → resume → rerack → destroy | **empty**: no errors, no warnings, no local paths |
| Remount after `destroy` | `RELOAD_REQUIRED`, as designed (ADR-0006) |

Browser testing found one real bug, now fixed: the wrapper's `destroyed`
handler called back into the module while it was dispatching that event,
which panicked on a re-entrant `RefCell` borrow and left the canvas behind.
Events are now dispatched with the borrow released.

Physics cost is negligible: about 2.5 µs per event natively, a few hundred
events per break.

## Input

| Path | Status |
|---|---|
| Mouse: hover to aim, drag back and release | Verified in the browser |
| Keyboard: arrows, Space, WASD, R, Esc | Verified natively |
| Touch | **Verified on a real phone** by the project author against the live demo (2026-10-18). Note: Chromium's mobile emulation cannot validate touch, because synthetic `PointerEvent`s report wrong `offsetX/Y` and carry no coalesced events. |
| Portrait layout | Verified at 375 × 812: the camera rotates and the table fills the stage |

## Still to measure

- Systematic mobile matrix (Safari iOS, Chrome Android): FPS and thermal behaviour over long sessions.
- Firefox and Safari desktop.
- Download and compile time over a throttled network with Brotli served.
- Memory after repeated pause/resume cycles.
