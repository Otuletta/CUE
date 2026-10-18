//! Graphics quality profiles (spec §21.4). They change how the table looks,
//! never how it plays: anti-aliasing and shadows here, and on the web a
//! lower resolution at [`Tier::Low`], which the page wrapper arranges
//! (ADR-0013).
//!
//! `Auto` starts high and steps down, never up, when the frame rate stays
//! low. It uses no telemetry, only the frame times it measures itself:
//!
//! - It waits out a warm-up after starting and after every change, when
//!   shaders compile and frames are slow for reasons that do not last.
//! - It judges each window by its median frame, so an occasional hitch (a
//!   garbage collection, the first shot's sounds) does not count.
//! - Frames longer than a quarter of a second are ignored: they come from a
//!   hidden tab or a debugger, not from the scene being too heavy.

use bevy::{light::DirectionalLightShadowMap, prelude::*};

use crate::{camera::TableCamera, settings::GameCommand, sim::Paused};

/// A window whose median frame is slower than this steps `Auto` down.
const SLOW_FRAME: f32 = 1.0 / 45.0;
/// Length of one measuring window, seconds of play.
const WINDOW: f32 = 2.0;
/// Time not measured after starting or changing tier, seconds.
const WARM_UP: f32 = 3.0;
/// Frames longer than this are not measured, seconds.
const IGNORED_FRAME: f32 = 0.25;

/// What the player chose.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Quality {
    /// Start high and step down if the frame rate stays low.
    #[default]
    Auto,
    /// Anti-aliasing and fine shadows.
    High,
    /// Anti-aliasing and coarser shadows.
    Medium,
    /// No anti-aliasing and no shadows; on the web, a lower resolution too.
    Low,
}

impl Quality {
    /// Stable name used by the public API and saved settings.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    /// Parses a name from [`Quality::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [Self::Auto, Self::High, Self::Medium, Self::Low]
            .into_iter()
            .find(|q| q.name() == name)
    }
}

/// The level actually rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// The lightest.
    Low,
    /// Balanced.
    Medium,
    /// The richest.
    High,
}

impl Tier {
    /// Stable name used in the state snapshot.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    const fn lower(self) -> Option<Self> {
        match self {
            Self::High => Some(Self::Medium),
            Self::Medium => Some(Self::Low),
            Self::Low => None,
        }
    }

    const fn msaa(self) -> Msaa {
        match self {
            Self::High | Self::Medium => Msaa::Sample4,
            Self::Low => Msaa::Off,
        }
    }

    /// Shadow map size, or `None` without shadows.
    const fn shadow_map(self) -> Option<usize> {
        match self {
            Self::High => Some(2048),
            Self::Medium => Some(1024),
            Self::Low => None,
        }
    }
}

/// The quality setting and what it currently renders.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Graphics {
    /// What the player chose.
    pub setting: Quality,
    /// What is rendered: the setting itself, or where `Auto` has settled.
    pub tier: Tier,
    /// Frame times in the current window.
    samples: Vec<f32>,
    /// Warm-up left before measuring, seconds.
    warm_up: f32,
}

impl Default for Graphics {
    fn default() -> Self {
        Self::new(Quality::Auto)
    }
}

impl Graphics {
    /// Graphics for a setting; `Auto` starts at the top.
    pub fn new(setting: Quality) -> Self {
        let tier = match setting {
            Quality::Auto | Quality::High => Tier::High,
            Quality::Medium => Tier::Medium,
            Quality::Low => Tier::Low,
        };
        Self {
            setting,
            tier,
            samples: Vec::new(),
            warm_up: WARM_UP,
        }
    }

    /// Feeds one frame's duration. Returns whether `Auto` stepped down.
    fn measure(&mut self, dt: f32) -> bool {
        if self.setting != Quality::Auto || dt <= 0.0 || dt > IGNORED_FRAME {
            return false;
        }
        if self.warm_up > 0.0 {
            self.warm_up -= dt;
            return false;
        }
        self.samples.push(dt);
        if self.samples.iter().sum::<f32>() < WINDOW {
            return false;
        }
        self.samples.sort_by(f32::total_cmp);
        let median = self.samples[self.samples.len() / 2];
        self.samples.clear();
        match self.tier.lower().filter(|_| median > SLOW_FRAME) {
            Some(lower) => {
                self.tier = lower;
                // The new pipelines compile now: wait again before judging.
                self.warm_up = WARM_UP;
                true
            }
            None => false,
        }
    }
}

/// Applies [`GameCommand::SetQuality`].
pub(crate) fn apply_commands(
    mut commands: MessageReader<GameCommand>,
    mut graphics: ResMut<Graphics>,
) {
    for command in commands.read() {
        if let GameCommand::SetQuality(q) = command
            && graphics.setting != *q
        {
            *graphics = Graphics::new(*q);
        }
    }
}

/// Measures frame times for `Auto`.
pub(crate) fn watch_frame_rate(
    time: Res<Time>,
    paused: Res<Paused>,
    mut graphics: ResMut<Graphics>,
) {
    if paused.0 || graphics.setting != Quality::Auto {
        return;
    }
    // Measuring is not a change; only a new tier triggers `apply`.
    if graphics
        .bypass_change_detection()
        .measure(time.delta_secs())
    {
        info!(
            "graphics: frame rate low, stepping down to {}",
            graphics.tier.name()
        );
        graphics.set_changed();
    }
}

/// Applies the tier to the camera, the light and the shadow map.
pub(crate) fn apply(
    graphics: Res<Graphics>,
    mut commands: Commands,
    cameras: Query<Entity, With<TableCamera>>,
    mut lights: Query<&mut DirectionalLight>,
    mut shadow_map: ResMut<DirectionalLightShadowMap>,
) {
    let tier = graphics.tier;
    for camera in &cameras {
        commands.entity(camera).insert(tier.msaa());
    }
    for mut light in &mut lights {
        light.shadow_maps_enabled = tier.shadow_map().is_some();
    }
    if let Some(size) = tier.shadow_map()
        && shadow_map.size != size
    {
        shadow_map.size = size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(graphics: &mut Graphics, dt: f32, seconds: f32) -> usize {
        let mut steps = 0;
        for _ in 0..(seconds / dt) as usize {
            steps += usize::from(graphics.measure(dt));
        }
        steps
    }

    #[test]
    fn auto_steps_down_one_tier_per_slow_window_and_never_up() {
        let mut smooth = Graphics::new(Quality::Auto);
        assert_eq!(run(&mut smooth, 1.0 / 60.0, 10.0), 0, "60 FPS is fine");
        assert_eq!(smooth.tier, Tier::High);

        let mut g = Graphics::new(Quality::Auto);
        assert_eq!(run(&mut g, 1.0 / 30.0, WARM_UP), 0, "warming up");
        assert_eq!(run(&mut g, 1.0 / 30.0, 2.5), 1, "one slow window");
        assert_eq!(g.tier, Tier::Medium);
        run(&mut g, 1.0 / 30.0, 10.0);
        assert_eq!(g.tier, Tier::Low, "and no lower");
        run(&mut g, 1.0 / 120.0, 10.0);
        assert_eq!(g.tier, Tier::Low, "never back up on its own");
    }

    #[test]
    fn occasional_hitches_do_not_count() {
        let mut g = Graphics::new(Quality::Auto);
        let mut steps = 0;
        for i in 0..1200 {
            // 60 FPS with a 0.2 s hitch every second.
            let dt = if i % 60 == 0 { 0.2 } else { 1.0 / 60.0 };
            steps += usize::from(g.measure(dt));
        }
        assert_eq!(steps, 0);
        assert_eq!(g.tier, Tier::High);
    }

    #[test]
    fn hidden_tab_frames_are_ignored() {
        let mut g = Graphics::new(Quality::Auto);
        assert_eq!(run(&mut g, 1.0, 30.0), 0);
        assert_eq!(g.tier, Tier::High);
    }

    #[test]
    fn a_fixed_setting_is_never_changed() {
        let mut g = Graphics::new(Quality::High);
        assert_eq!(run(&mut g, 1.0 / 20.0, 10.0), 0);
        assert_eq!(g.tier, Tier::High);
    }

    #[test]
    fn names_round_trip() {
        for q in [Quality::Auto, Quality::High, Quality::Medium, Quality::Low] {
            assert_eq!(Quality::parse(q.name()), Some(q));
        }
        assert_eq!(Quality::parse("ultra"), None);
    }
}
