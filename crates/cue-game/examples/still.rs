//! Renders a still image of the table without opening a window:
//! `cargo run -p cue-game --example still --release`.
//!
//! For visual checks and documentation, and on machines where a window cannot
//! be shown (CI, a locked screen):
//!
//! - `CUE_STILL=<path.png>` is where the image goes (required).
//! - `CUE_VIEW=table|rack|cue` picks the camera: the whole table, a close-up
//!   of the rack's numbers, or the cue behind the cue ball.
//! - `CUE_SIZE=<width>x<height>` sets the image size (default 1920x1080).
//! - `CUE_THEME=classic|studio|contrast` picks the presentation.

use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, save_to_disk},
    },
    window::ExitCondition,
    winit::WinitPlugin,
};
use cue_game::{CuePlugin, Theme};

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

/// Points the table camera at an image instead of a window, from the chosen
/// viewpoint.
fn aim_camera(
    mut commands: Commands,
    mut still: ResMut<Still>,
    mut images: ResMut<Assets<Image>>,
    mut camera: Query<(Entity, &mut Transform), With<Camera3d>>,
) {
    let Ok((entity, mut transform)) = camera.single_mut() else {
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
    *transform = match still.view.as_str() {
        // Low over the foot of the table, looking at the rack.
        "rack" => {
            Transform::from_xyz(0.42, 0.42, 0.34).looking_at(Vec3::new(0.7, 0.0, 0.0), Vec3::Y)
        }
        // Behind and above the cue ball at the break spot.
        "cue" => {
            Transform::from_xyz(-1.12, 0.3, 0.22).looking_at(Vec3::new(-0.72, 0.02, 0.0), Vec3::Y)
        }
        // The whole table, as the game frames it on a 16:9 screen.
        _ => Transform::from_xyz(0.0, 3.05, 0.62).looking_at(Vec3::ZERO, Vec3::Y),
    };
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
