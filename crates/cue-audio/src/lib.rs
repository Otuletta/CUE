//! # cue-audio
//!
//! Procedural synthesis of CUE's impact sounds. There are no audio files to
//! license, store or download: every clip is computed at start-up from a
//! small recipe, in a few milliseconds.
//!
//! An impact on a hard, ringing body is well described by **modal
//! synthesis**: a sum of exponentially decaying sinusoids, one per vibration
//! mode, plus a short burst of filtered noise for the contact itself. Phenolic
//! balls ring high and briefly (a sharp click), cushions low and dull (a
//! thump), and a pocket drop is a thud followed by a short rattle.
//!
//! Each sound comes in several variants with slightly detuned modes, so
//! repeated contacts do not sound mechanical (spec §13.2). Clips are
//! deterministic: the same sound, variant and sample rate always render the
//! same samples.

#![warn(missing_docs)]

use std::f32::consts::TAU;

/// The impact sounds CUE plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sound {
    /// Two balls colliding.
    BallClick,
    /// A ball hitting a cushion.
    Cushion,
    /// A ball dropping into a pocket.
    Pocket,
    /// The cue tip striking the cue ball.
    CueTip,
}

impl Sound {
    /// Every sound, in a stable order.
    pub const ALL: [Self; 4] = [Self::BallClick, Self::Cushion, Self::Pocket, Self::CueTip];

    fn recipe(self) -> Recipe {
        match self {
            Self::BallClick => Recipe {
                length: 0.12,
                hits: &[(0.0, 1.0)],
                modes: &[
                    Mode {
                        freq: 3150.0,
                        decay: 0.012,
                        gain: 1.0,
                    },
                    Mode {
                        freq: 5230.0,
                        decay: 0.008,
                        gain: 0.6,
                    },
                    Mode {
                        freq: 7900.0,
                        decay: 0.005,
                        gain: 0.35,
                    },
                    Mode {
                        freq: 10800.0,
                        decay: 0.003,
                        gain: 0.2,
                    },
                ],
                noise: Noise {
                    gain: 0.5,
                    decay: 0.0015,
                    cutoff: 9000.0,
                },
            },
            Self::Cushion => Recipe {
                length: 0.22,
                hits: &[(0.0, 1.0)],
                modes: &[
                    Mode {
                        freq: 160.0,
                        decay: 0.045,
                        gain: 1.0,
                    },
                    Mode {
                        freq: 390.0,
                        decay: 0.03,
                        gain: 0.5,
                    },
                    Mode {
                        freq: 820.0,
                        decay: 0.015,
                        gain: 0.25,
                    },
                ],
                noise: Noise {
                    gain: 0.35,
                    decay: 0.004,
                    cutoff: 2500.0,
                },
            },
            Self::Pocket => Recipe {
                length: 0.45,
                // A thud, then the ball rattling down the liner.
                hits: &[(0.0, 1.0), (0.045, 0.32), (0.09, 0.2), (0.15, 0.1)],
                modes: &[
                    Mode {
                        freq: 95.0,
                        decay: 0.08,
                        gain: 1.0,
                    },
                    Mode {
                        freq: 230.0,
                        decay: 0.05,
                        gain: 0.5,
                    },
                    Mode {
                        freq: 2200.0,
                        decay: 0.01,
                        gain: 0.25,
                    },
                ],
                noise: Noise {
                    gain: 0.3,
                    decay: 0.006,
                    cutoff: 1800.0,
                },
            },
            Self::CueTip => Recipe {
                length: 0.08,
                hits: &[(0.0, 1.0)],
                modes: &[
                    Mode {
                        freq: 1400.0,
                        decay: 0.006,
                        gain: 1.0,
                    },
                    Mode {
                        freq: 2900.0,
                        decay: 0.004,
                        gain: 0.4,
                    },
                ],
                noise: Noise {
                    gain: 0.6,
                    decay: 0.002,
                    cutoff: 4000.0,
                },
            },
        }
    }

    fn index(self) -> u64 {
        match self {
            Self::BallClick => 0,
            Self::Cushion => 1,
            Self::Pocket => 2,
            Self::CueTip => 3,
        }
    }
}

/// One vibration mode: frequency (Hz), exponential decay time (s, to 1/e)
/// and relative amplitude.
#[derive(Clone, Copy, Debug)]
struct Mode {
    freq: f32,
    decay: f32,
    gain: f32,
}

/// The contact transient: white noise through a one-pole low-pass filter,
/// decaying exponentially.
#[derive(Clone, Copy, Debug)]
struct Noise {
    gain: f32,
    decay: f32,
    cutoff: f32,
}

#[derive(Clone, Copy, Debug)]
struct Recipe {
    /// Clip length, seconds.
    length: f32,
    /// `(offset, gain)` of each excitation; most sounds have one.
    hits: &'static [(f32, f32)],
    modes: &'static [Mode],
    noise: Noise,
}

/// Peak amplitude of every rendered clip.
pub const PEAK: f32 = 0.9;

/// Length of the fade-in that avoids a click at the clip start, seconds.
const FADE_IN: f32 = 0.0002;

/// Renders one variant of a sound as mono samples in `[-PEAK, PEAK]`.
pub fn render(sound: Sound, variant: u32, sample_rate: f32) -> Vec<f32> {
    let recipe = sound.recipe();
    let mut rng = XorShift::new(sound.index() << 32 | u64::from(variant));
    let len = (recipe.length * sample_rate).round() as usize;
    let mut out = vec![0.0f32; len];
    let dt = 1.0 / sample_rate;

    // Detune each mode by up to ±3 % and vary its decay by ±10 %.
    let modes: Vec<(Mode, f32)> = recipe
        .modes
        .iter()
        .map(|m| {
            let mode = Mode {
                freq: m.freq * (1.0 + 0.06 * (rng.unit() - 0.5)),
                decay: m.decay * (1.0 + 0.2 * (rng.unit() - 0.5)),
                gain: m.gain,
            };
            (mode, rng.unit() * TAU)
        })
        .collect();
    // One-pole low-pass coefficient for the noise burst.
    let alpha = 1.0 - (-TAU * recipe.noise.cutoff * dt).exp();

    for &(offset, hit_gain) in recipe.hits {
        let start = (offset * sample_rate).round() as usize;
        let mut filtered = 0.0f32;
        for (k, sample) in out.iter_mut().enumerate().skip(start) {
            let t = (k - start) as f32 * dt;
            let tonal: f32 = modes
                .iter()
                .map(|(m, phase)| m.gain * (-t / m.decay).exp() * (TAU * m.freq * t + phase).sin())
                .sum();
            filtered += alpha * (rng.signed() - filtered);
            let noise = recipe.noise.gain * (-t / recipe.noise.decay).exp() * filtered;
            let fade = (t / FADE_IN).min(1.0);
            *sample += hit_gain * fade * (tonal + noise);
        }
    }

    let peak = out.iter().fold(0.0f32, |p, s| p.max(s.abs()));
    if peak > 0.0 {
        let scale = PEAK / peak;
        out.iter_mut().for_each(|s| *s *= scale);
    }
    out
}

/// A tiny, deterministic pseudo-random generator for noise and detuning.
/// Not suitable for anything but sound.
#[derive(Debug, Clone)]
struct XorShift(u64);

impl XorShift {
    fn new(seed: u64) -> Self {
        // Mix the seed so that small, similar seeds give unrelated streams.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self((z ^ (z >> 31)) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Uniform in `[0, 1)`.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `[-1, 1)`.
    fn signed(&mut self) -> f32 {
        2.0 * self.unit() - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = 48_000.0;

    fn rms(samples: &[f32]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
    }

    /// Fraction of adjacent samples that change sign: a cheap proxy for how
    /// bright a sound is.
    fn zero_crossings(samples: &[f32]) -> f32 {
        let crossings = samples
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count();
        crossings as f32 / samples.len() as f32
    }

    #[test]
    fn every_clip_is_finite_normalised_and_decays() {
        for sound in Sound::ALL {
            for variant in 0..4 {
                let clip = render(sound, variant, RATE);
                assert!(!clip.is_empty());
                assert!(clip.iter().all(|s| s.is_finite()), "{sound:?}");
                let peak = clip.iter().fold(0.0f32, |p, s| p.max(s.abs()));
                assert!((peak - PEAK).abs() < 1e-5, "{sound:?} peak {peak}");
                let fifth = clip.len() / 5;
                let (head, tail) = (rms(&clip[..fifth]), rms(&clip[clip.len() - fifth..]));
                assert!(
                    tail < 0.05 * head,
                    "{sound:?} does not decay: {tail} vs {head}"
                );
            }
        }
    }

    #[test]
    fn clips_start_without_a_click() {
        for sound in Sound::ALL {
            let clip = render(sound, 0, RATE);
            assert!(clip[0].abs() < 1e-3, "{sound:?} starts at {}", clip[0]);
        }
    }

    #[test]
    fn rendering_is_deterministic_and_variants_differ() {
        assert_eq!(
            render(Sound::BallClick, 2, RATE),
            render(Sound::BallClick, 2, RATE)
        );
        assert_ne!(
            render(Sound::BallClick, 2, RATE),
            render(Sound::BallClick, 3, RATE)
        );
    }

    #[test]
    fn duration_does_not_depend_on_sample_rate() {
        for sound in Sound::ALL {
            let a = render(sound, 0, 44_100.0).len() as f32 / 44_100.0;
            let b = render(sound, 0, 48_000.0).len() as f32 / 48_000.0;
            assert!((a - b).abs() < 1e-3);
        }
    }

    #[test]
    fn balls_click_brighter_than_cushions_thump() {
        let click = zero_crossings(&render(Sound::BallClick, 0, RATE));
        let thump = zero_crossings(&render(Sound::Cushion, 0, RATE));
        assert!(click > 5.0 * thump, "click {click} vs thump {thump}");
    }
}
