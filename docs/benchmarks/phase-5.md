# Phase 5 — what the visual work costs

**Machine:** Apple M4 Pro, macOS 27.2. Browser figures come from Chromium
(the Claude desktop browser pane) with WebGL 2 via ANGLE/Metal. Native
figures come from release builds.

**Build:** `scripts/build-web.sh`, Rust 1.99.1, Bevy 0.20.0. The build is
served locally, and compressed sizes are computed offline
(`scripts/size-report.mjs`).

## Download

| | Phase 4 | Phase 5 | Change |
|---|---:|---:|---:|
| `cue_web_bg.wasm`, Brotli 11 | 4 251.9 KiB | 4 266.2 KiB | +14.3 KiB |
| `cue_web_bg.wasm`, gzip -9 (as GitHub Pages serves it) | 6 934.1 KiB | 6 957.7 KiB | +23.6 KiB |
| Control layer (`package/*.js`), Brotli 11 | 27.0 KiB | 28.3 KiB | +1.3 KiB |

That buys the following, with no asset files:

- numbered balls;
- a cloth grain;
- diamonds and the foot spot;
- the cue model and its swing;
- the camera insets;
- three quality profiles;
- three game sounds.

## Start-up

| | Time |
|---|---:|
| Click to first frame, compiled module cached | 1.16 s |
| Click to first frame, new build (module compiled) | 2.45 s |
| Generating the 16 ball textures with mips, native release | 66 ms |
| Generating the cloth grain, native release | 1.2 ms |

The cached start-up is in line with Phase 4 (0.7–1.3 s). The new-build
figure is dominated by compiling a new 30 MB module, as before. The textures
are generated on the CPU at start-up; in WebAssembly this takes a few times
the native time.

## GPU memory

| Resource | Size |
|---|---:|
| Ball textures, 16 × 512 × 256 RGBA with mips | ≈ 11.2 MB |
| Cloth grain, 256 × 256 RGBA with mips | ≈ 0.35 MB |
| Shadow map, High / Medium / Low | 16 / 4 / 0 MB |

## Not yet measured

- **Frame rate per profile on the device matrix** (spec §21.5): this is the
  exit criterion of Phase 5. It needs real phones and browsers, and someone
  watching the screen.
- **Whether Auto settles where it should on a mid-range phone:** it should
  stay at High or drop at most to Medium.
