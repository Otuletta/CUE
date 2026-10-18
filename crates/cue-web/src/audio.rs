//! Plays `cue-audio` clips through the Web Audio API.
//!
//! Browsers only allow audio after a user gesture (spec §13.3), so nothing is
//! created until the host calls `unlockAudio()` from inside one. At that
//! point the clips are synthesised at the context's sample rate, once.

use std::cell::{Cell, RefCell};

use cue_audio::Sound;
use cue_physics::SimEventKind;
use wasm_bindgen::JsValue;
use web_sys::{AudioBuffer, AudioContext, AudioContextState, GainNode};

/// Variants rendered per sound; contacts pick one at random.
const VARIANTS: u32 = 4;
/// Most contacts voiced per frame. A break produces dozens of contacts in a
/// few milliseconds; past a handful they blur into one sound anyway.
pub(crate) const MAX_PER_FRAME: usize = 6;

struct Engine {
    ctx: AudioContext,
    master: GainNode,
    clips: Vec<(Sound, Vec<AudioBuffer>)>,
    counter: Cell<u32>,
}

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
    static ENABLED: Cell<bool> = const { Cell::new(true) };
    static VOLUME: Cell<f32> = const { Cell::new(0.7) };
}

impl Engine {
    fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let rate = ctx.sample_rate();
        let master = ctx.create_gain()?;
        master.gain().set_value(effective_volume());
        master.connect_with_audio_node(&ctx.destination())?;
        let clips = Sound::ALL
            .iter()
            .map(|&sound| {
                let buffers = (0..VARIANTS)
                    .map(|v| {
                        let samples = cue_audio::render(sound, v, rate);
                        let buffer = ctx.create_buffer(1, samples.len() as u32, rate)?;
                        buffer.copy_to_channel(&samples, 0)?;
                        Ok(buffer)
                    })
                    .collect::<Result<Vec<_>, JsValue>>()?;
                Ok((sound, buffers))
            })
            .collect::<Result<Vec<_>, JsValue>>()?;
        Ok(Self {
            ctx,
            master,
            clips,
            counter: Cell::new(0),
        })
    }

    /// Plays `sound` `delay` seconds from now.
    fn play(&self, sound: Sound, gain: f32, rate: f32, delay: f64) -> Result<(), JsValue> {
        let Some((_, buffers)) = self.clips.iter().find(|(s, _)| *s == sound) else {
            return Ok(());
        };
        // Cycle through the variants; with four of them no listener can tell.
        let n = self.counter.get().wrapping_add(1);
        self.counter.set(n);
        let buffer = &buffers[n as usize % buffers.len()];
        let source = self.ctx.create_buffer_source()?;
        source.set_buffer(Some(buffer));
        source.playback_rate().set_value(rate);
        let voice = self.ctx.create_gain()?;
        voice.gain().set_value(gain);
        source.connect_with_audio_node(&voice)?;
        voice.connect_with_audio_node(&self.master)?;
        source.start_with_when(self.ctx.current_time() + delay)?;
        Ok(())
    }
}

fn effective_volume() -> f32 {
    if ENABLED.get() { VOLUME.get() } else { 0.0 }
}

/// Creates (or resumes) the audio context. Must be called from inside a
/// user gesture, or the browser will keep it suspended.
pub(crate) fn unlock() -> Result<(), JsValue> {
    ENGINE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(Engine::new()?);
        }
        if let Some(engine) = slot.as_ref()
            && engine.ctx.state() == AudioContextState::Suspended
        {
            let _ = engine.ctx.resume()?;
        }
        Ok(())
    })
}

/// Sets whether sound plays and how loud, `0..=1`.
pub(crate) fn configure(enabled: bool, volume: f32) {
    ENABLED.set(enabled);
    VOLUME.set(volume.clamp(0.0, 1.0));
    ENGINE.with(|slot| {
        if let Some(engine) = slot.borrow().as_ref() {
            engine.master.gain().set_value(effective_volume());
        }
    });
}

/// Suspends or resumes the audio context with the game.
pub(crate) fn set_running(running: bool) {
    ENGINE.with(|slot| {
        if let Some(engine) = slot.borrow().as_ref() {
            let _ = if running {
                engine.ctx.resume()
            } else {
                engine.ctx.suspend()
            };
        }
    });
}

/// Voices one simulation event, if it makes a sound, as the `nth` sound of
/// its frame. Returns whether it did.
pub(crate) fn play_event(kind: &SimEventKind, nth: usize) -> bool {
    let Some((sound, gain, rate)) = voice(kind) else {
        return false;
    };
    let gain = gain * crowd(nth);
    // A pocketed ball rolls on into the pocket before it drops (the drawing
    // in `cue-game`); its clunk comes with the drop.
    let delay = if sound == Sound::Pocket {
        POCKET_DELAY
    } else {
        0.0
    };
    if !ENABLED.get() {
        return false;
    }
    ENGINE.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|engine| engine.play(sound, gain, rate, delay).is_ok())
    })
}

/// Plays one of the game cues (foul, win), quietly: they mark the game,
/// they must not drown the table.
pub(crate) fn play_cue(sound: Sound) {
    const CUE_GAIN: f32 = 0.18;
    if !ENABLED.get() {
        return;
    }
    ENGINE.with(|slot| {
        if let Some(engine) = slot.borrow().as_ref() {
            let _ = engine.play(sound, CUE_GAIN, 1.0, 0.0);
        }
    });
}

/// Which clip an event plays, how loud and at what playback rate. Harder
/// contacts are louder and very slightly higher; whispers are skipped.
///
/// The mix is a quiet room, not an arcade: only a hard break reaches full
/// level, ordinary contacts sit well below it, and cushions, pockets and the
/// tip stay in the background.
pub(crate) fn voice(kind: &SimEventKind) -> Option<(Sound, f32, f32)> {
    let scaled = |speed: f64, full: f64, floor: f64| -> Option<(f32, f32)> {
        (speed >= floor).then(|| {
            let level = (speed / full).clamp(0.0, 1.0) as f32;
            (level.powf(0.9), 0.97 + 0.06 * level)
        })
    };
    match *kind {
        SimEventKind::BallBall { normal_speed, .. } => {
            scaled(normal_speed, 6.0, 0.03).map(|(g, r)| (Sound::BallClick, 0.7 * g, r))
        }
        SimEventKind::Cushion { normal_speed, .. } => {
            scaled(normal_speed, 4.0, 0.05).map(|(g, r)| (Sound::Cushion, 0.45 * g, r))
        }
        SimEventKind::Struck { speed } => {
            scaled(speed, 12.0, 0.0).map(|(g, r)| (Sound::CueTip, 0.15 + 0.4 * g, r))
        }
        SimEventKind::Pocketed { .. } => Some((Sound::Pocket, 0.4, 1.0)),
        SimEventKind::Transition { .. } | SimEventKind::CueCrossedHeadString => None,
    }
}

/// Seconds from a ball entering a pocket to the sound of its drop.
const POCKET_DELAY: f64 = 0.2;

/// How much quieter the `n`-th sound in one frame plays (from 0): a break's
/// burst of clicks blends instead of piling up.
pub(crate) fn crowd(n: usize) -> f32 {
    1.0 / (1.0 + 0.45 * n as f32)
}

#[cfg(test)]
mod tests {
    use cue_core::{BallId, PocketId};

    use super::*;

    #[test]
    fn harder_contacts_are_louder_and_whispers_are_silent() {
        let hit = |speed| SimEventKind::BallBall {
            a: BallId::CUE,
            b: BallId::EIGHT,
            normal_speed: speed,
        };
        let soft = voice(&hit(0.3)).unwrap();
        let hard = voice(&hit(3.0)).unwrap();
        assert!(hard.1 > soft.1);
        assert!(hard.2 > soft.2);
        assert!(voice(&hit(0.001)).is_none());
        assert!(hard.1 <= 1.0);
        // An ordinary contact sits well below full level.
        assert!(voice(&hit(1.0)).unwrap().1 < 0.2);
        // A burst in one frame gets quieter, never louder.
        assert!((crowd(0) - 1.0).abs() < 1e-6 && crowd(3) < crowd(1) && crowd(5) > 0.0);
    }

    #[test]
    fn every_audible_event_maps_to_its_clip() {
        let pocket = SimEventKind::Pocketed {
            ball: BallId::EIGHT,
            pocket: PocketId::SideFar,
        };
        assert_eq!(voice(&pocket).map(|v| v.0), Some(Sound::Pocket));
        assert_eq!(
            voice(&SimEventKind::Struck { speed: 4.0 }).map(|v| v.0),
            Some(Sound::CueTip)
        );
    }
}
