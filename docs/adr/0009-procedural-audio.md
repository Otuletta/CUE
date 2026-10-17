# ADR-0009 — Procedural audio

- **Status:** accepted
- **Date:** 2026-10-18

## Context

Spec §13 requires sounds for the cue strike, ball–ball and ball–cushion
contacts and pocketing. They must vary with impact strength, avoid monotony,
start only after a user gesture, and never be the only feedback. Spec §12.3
requires every asset to have a licence compatible with open-source
redistribution. Bevy's audio stack was dropped from the WASM build in Phase 1
for size.

## Decision

Synthesise the sounds at runtime in Rust (`crates/cue-audio`). Each sound is
**modal synthesis**: a sum of exponentially decaying sinusoids, one per
vibration mode, plus a short burst of low-pass-filtered noise for the
contact.

- **Ball click:** high (3–11 kHz) and very short.
- **Cushion thump:** low (160–820 Hz) and dull.
- **Pocket:** a thud followed by a short rattle.
- **Cue tip:** a soft, woody tick.

Four detuned variants of each break up repetition. Impact speed scales the
gain and, very slightly, the playback rate. At most six contacts are voiced
per frame.

Playback goes through the Web Audio API from `cue-web`, via `web-sys`. The
context is created inside the first `pointerdown` or `keydown` on CUE's root,
as autoplay policies require. It is suspended while the game is paused.

## Consequences

**Positive**

- No audio files: nothing to license, attribute, compress or download.
- The DSP code is deterministic and tested natively: finite output, peak
  normalisation, decay, no start click, and a brightness ordering check
  (clicks brighter than thumps).
- Synthesis at the context's own sample rate means no resampling.

**Costs**

- The sounds are recognisable rather than recorded-realistic. Recorded
  samples with a clear licence could replace a recipe later without changing
  the playback code.
- The native sandbox is silent. Audio is a web-target feature for now.
