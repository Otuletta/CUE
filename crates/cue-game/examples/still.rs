//! Renders a still image of the table without opening a window:
//! `cargo run -p cue-game --example still --release`.
//!
//! For visual checks and documentation, and on machines where a window cannot
//! be shown (CI, a locked screen):
//!
//! - `CUE_STILL=<path.png>` is where the image goes (required).
//! - `CUE_VIEW=table|rack|cue` picks the framing, always from straight above
//!   as in the game: the whole table, a close-up of the rack's numbers, or
//!   the cue ball and the cue.
//! - `CUE_SIZE=<width>x<height>` sets the image size (default 1920x1080).
//! - `CUE_THEME=classic|studio|contrast` picks the presentation.

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    camera::{RenderTarget, ScalingMode},
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, save_to_disk},
    },
    window::ExitCondition,
    winit::WinitPlugin,
};
use cue_game::{CuePlugin, Simulation, Theme};

/// Frames rendered before the capture, so shaders compile and textures load.
const SETTLE_FRAMES: u32 = 40;

#[derive(Resource)]
struct Still {
    path: String,
    size: UVec2,
    view: String,
    target: Option<Handle<Image>>,
}

fn main() -> AppExit {
    let path = std::env::var("CUE_STILL").expect("set CUE_STILL to the output .png path");
    let size = std::env::var("CUE_SIZE")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or(UVec2::new(1920, 1080));
    let view = std::env::var("CUE_VIEW").unwrap_or_else(|_| "table".into());
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(CuePlugin)
        .insert_resource(
            std::env::var("CUE_THEME")
                .ok()
                .and_then(|t| Theme::parse(&t))
                .unwrap_or_default(),
        )
        .insert_resource(Still {
            path,
            size,
            view,
            target: None,
        })
        .add_systems(PostStartup, aim_camera)
        .add_systems(Update, capture)
        .run()
}

/// Points the table camera at an image instead of a window, and frames the
/// chosen view from straight above.
fn aim_camera(
    mut commands: Commands,
    mut still: ResMut<Still>,
    mut images: ResMut<Assets<Image>>,
    sim: Res<Simulation>,
    mut camera: Query<(Entity, &mut Transform, &mut Projection), With<Camera3d>>,
) {
    let Ok((entity, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    let target = images.add(Image::new_target_texture(
        still.size.x,
        still.size.y,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands
        .entity(entity)
        .insert(RenderTarget::Image(target.clone().into()));
    still.target = Some(target);
    // The centre of the view and the half-extents it must show, in metres,
    // from the table the game plays on.
    let table = *sim.table();
    let (foot, break_spot) = (table.foot_spot().as_vec2(), table.break_spot().as_vec2());
    let (centre, half) = match still.view.as_str() {
        // The rack at the foot spot.
        "rack" => (Vec3::new(foot.x + 0.065, 0.0, 0.0), Vec2::new(0.18, 0.16)),
        // The cue ball at the break spot, with the cue behind it as far as
        // its forearm.
        "cue" => (
            Vec3::new(break_spot.x - 0.5, 0.0, 0.0),
            Vec2::new(0.62, 0.2),
        ),
        // The whole table with its rails (12 cm) and the game's margin (6 cm).
        _ => (
            Vec3::ZERO,
            sim.world.geometry().outer_half_extents().as_vec2() + Vec2::splat(0.18),
        ),
    };
    let aspect = still.size.x as f32 / still.size.y.max(1) as f32;
    let width = (half.x * 2.0).max(half.y * 2.0 * aspect);
    *transform =
        Transform::from_translation(centre + Vec3::Y * 3.0).looking_to(Vec3::NEG_Y, Vec3::NEG_Z);
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        ortho.scaling_mode = ScalingMode::Fixed {
            width,
            height: width / aspect,
        };
    }
}

fn capture(
    mut commands: Commands,
    still: Res<Still>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;
    if *frames == SETTLE_FRAMES
        && let Some(target) = &still.target
    {
        commands
            .spawn(Screenshot::image(target.clone()))
            .observe(save_to_disk(still.path.clone()));
    }
    if *frames == SETTLE_FRAMES + 20 {
        exit.write(AppExit::Success);
    }
}
