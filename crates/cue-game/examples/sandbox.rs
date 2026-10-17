//! Native development sandbox: `cargo run -p cue-game --example sandbox`.

use bevy::prelude::*;
use cue_game::CuePlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CUE sandbox".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CuePlugin)
        .run()
}
