# Browser support

CUE needs **WebAssembly** and **WebGL 2**. Without them it shows a short
explanation and a link instead of a blank box, and the host page keeps
working.

Support is announced only after testing (spec §21.5). This page says what
has been tested, and how. "Untested" means only that: not that it fails.

| Environment | Priority | Status | Evidence |
|---|---|---|---|
| Chromium, macOS (Apple M4 Pro) | High | **Tested** | Development and every measurement in `docs/benchmarks/`, in the Claude desktop browser pane: loading, input, pausing, eight-ball flows, a strict CSP and a Vite host. |
| Chrome or Safari on a phone | High | **Smoke-tested** (Phase 1) | The maintainer played on a phone in Phase 1. Device and browser not recorded. |
| Firefox, desktop | High | Untested | |
| Safari, macOS | High | Untested | |
| Safari, iOS and iPadOS | High | Untested | Expected limit: no Fullscreen API for elements, so CUE uses its immersive layout. |
| Chrome, Android | High | Untested beyond the Phase 1 smoke test | |
| No WebGL 2 or no WebAssembly | Degradation | **Tested** in code | `createCue` rejects with `UNSUPPORTED_BROWSER`, and the UI explains why. |
| WebGPU | Experimental | Not built | CUE ships a WebGL 2 build only (ADR-0003). |

## Known limits

- **Audio** starts only after a gesture on CUE, as browsers require.
- **Fullscreen:** iPhone Safari cannot fullscreen an element, so CUE fills
  the window with a fixed layout instead and restores everything on exit.
- **One instance per page,** and a destroyed instance needs a reload
  ([web-integration.md](web-integration.md#single-page-apps)).
- **Hidden tabs** run very slowly or not at all. CUE ignores those frames
  for automatic quality, and hosts should pause CUE when hidden; the
  reference host does.
- **Memory:** about 11 MB of textures plus the shadow map (up to 16 MB at
  High).

## How to report

Open an issue with the browser, its version, the device, what you expected,
and the `error` event's code or the console output.
