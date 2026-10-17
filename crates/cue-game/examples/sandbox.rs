//! Native development sandbox: `cargo run -p cue-game --example sandbox`.
//!
//! For unattended visual checks:
//!
//! - `CUE_THEME=classic|studio|contrast` and `CUE_ASSISTS=off|basic|assisted`
//!   pick the presentation.
//! - `CUE_SHOT=<angle_deg>,<power>` strikes the cue ball on the first frame.
//! - `CUE_SCREENSHOT=<path>` saves a screenshot after `CUE_SCREENSHOT_AT`
//!   seconds (default 1.5) and exits shortly after.

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use cue_core::glam::DVec2;
use cue_game::{Assists, CuePlugin, DevHud, Simulation, Theme};
use cue_physics::Strike;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CUE sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CuePlugin)
        .insert_resource(DevHud(true))
        .insert_resource(
            std::env::var("CUE_THEME")
                .ok()
                .and_then(|t| Theme::parse(&t))
                .unwrap_or_default(),
        )
        .insert_resource(
            std::env::var("CUE_ASSISTS")
                .ok()
                .and_then(|a| Assists::parse(&a))
                .unwrap_or_default(),
        )
        .add_systems(Update, (scripted_shot, scripted_screenshot))
        .run()
}

fn scripted_shot(mut sim: ResMut<Simulation>, mut done: Local<bool>) {
    if *done {
        return;
    }
    *done = true;
    let Some((angle, power)) = std::env::var("CUE_SHOT").ok().and_then(|v| {
        let (a, p) = v.split_once(',')?;
        Some((a.parse::<f64>().ok()?, p.parse::<f64>().ok()?))
    }) else {
        return;
    };
    let strike = Strike::centre(DVec2::from_angle(angle.to_radians()), power);
    if let Err(e) = sim.strike(strike) {
        warn!("scripted shot rejected: {e}");
    }
}

fn scripted_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    mut taken: Local<Option<f32>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(path) = std::env::var("CUE_SCREENSHOT") else {
        return;
    };
    let at = std::env::var("CUE_SCREENSHOT_AT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.5);
    let now = time.elapsed_secs();
    match *taken {
        None if now >= at => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            *taken = Some(now);
        }
        Some(t) if now > t + 0.5 => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
