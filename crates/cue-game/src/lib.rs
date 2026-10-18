//! # cue-game
//!
//! The Bevy application. It owns presentation and orchestration: it turns
//! player input into [`Strike`](cue_physics::Strike)s, hands them to
//! `cue-physics`, and draws whatever the simulation says. It never makes a
//! physical or regulatory decision of its own.
//!
//! The simulation is the single source of truth. Systems read it to decide
//! what is allowed ("can the player shoot?") rather than keeping a parallel
//! state machine that could drift out of sync.

mod ball_texture;
mod camera;
mod eight;
mod hud;
mod scene;
mod settings;
mod shot;
mod sim;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub use eight::{EightBall, GameSetup, Mode};
pub use settings::{Assists, DevHud, GameCommand, Theme};
pub use shot::{Aim, MIN_SHOT_POWER};
pub use sim::{Ball, Contact, GameEvent, HandRegion, LastShot, Paused, ShotFinished, Simulation};

/// Everything CUE adds to a Bevy [`App`].
#[derive(Debug, Default)]
pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        app.insert_resource(ClearColor(Color::srgb(0.035, 0.037, 0.045)))
            .insert_resource(GlobalAmbientLight {
                color: Color::WHITE,
                brightness: 250.0,
                affects_lightmapped_meshes: true,
            })
            .insert_resource(Simulation::new(1))
            .init_resource::<Paused>()
            .init_resource::<LastShot>()
            .init_resource::<Theme>()
            .init_resource::<Assists>()
            .init_resource::<DevHud>()
            .init_resource::<Mode>()
            .add_message::<Contact>()
            .add_message::<ShotFinished>()
            .add_message::<GameEvent>()
            .add_message::<GameCommand>()
            .init_resource::<Aim>()
            .add_systems(
                Startup,
                (scene::spawn_table, scene::spawn_balls, shot::spawn_cue),
            )
            .add_systems(
                Update,
                (
                    eight::apply_commands,
                    shot::apply_commands,
                    shot::read_input,
                    eight::suggest_call,
                    sim::advance,
                    sim::after_shot,
                    eight::rule_shots,
                    sim::sync_balls,
                    shot::draw_aim,
                    camera::fit_to_window,
                    hud::update,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                settings::apply_theme.run_if(resource_changed::<Theme>),
            )
            .add_systems(Startup, (camera::spawn, hud::spawn));
    }
}
