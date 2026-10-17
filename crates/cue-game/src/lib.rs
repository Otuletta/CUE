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
mod hud;
mod scene;
mod shot;
mod sim;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub use shot::Aim;
pub use sim::{Ball, Contact, LastShot, Paused, ShotFinished, Simulation};

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
            .add_message::<Contact>()
            .add_message::<ShotFinished>()
            .init_resource::<Aim>()
            .add_systems(
                Startup,
                (scene::spawn_table, scene::spawn_balls, shot::spawn_cue),
            )
            .add_systems(
                Update,
                (
                    shot::read_input,
                    sim::advance,
                    sim::after_shot,
                    sim::sync_balls,
                    shot::draw_aim,
                    camera::fit_to_window,
                    hud::update,
                )
                    .chain(),
            )
            .add_systems(Startup, (camera::spawn, hud::spawn));
    }
}
