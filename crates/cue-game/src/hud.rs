//! Minimal development HUD. The real, accessible interface is HTML outside the
//! canvas (spec §22); this overlay exists to read state and frame rate while
//! the vertical slice is being measured.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    shot::Aim,
    sim::{LastShot, Simulation},
};

/// Below this canvas width (logical px) only the status line is shown.
const COMPACT_WIDTH: f32 = 720.0;

#[derive(Component, Debug)]
pub(crate) struct HudText;

pub(crate) fn spawn(mut commands: Commands) {
    commands.spawn((
        HudText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));
}

pub(crate) fn update(
    sim: Res<Simulation>,
    aim: Res<Aim>,
    diagnostics: Res<DiagnosticsStore>,
    last: Res<LastShot>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut text: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let state = if sim.world.is_at_rest() {
        format!(
            "power {:>3.0}%   spin {:+.2} side {:+.2} height",
            aim.power * 100.0,
            aim.tip.x,
            aim.tip.y
        )
    } else {
        "balls moving".to_owned()
    };
    let shot = last.0.as_ref().map_or_else(String::new, |r| {
        let pocketed: Vec<String> = r
            .pocketed()
            .map(|(b, _)| {
                if b.is_cue() {
                    "cue".into()
                } else {
                    b.number().to_string()
                }
            })
            .collect();
        let first = r
            .first_contacts()
            .iter()
            .map(|b| b.number().to_string())
            .collect::<Vec<_>>()
            .join("+");
        format!(
            "   last shot: first contact {}   pocketed {}",
            if first.is_empty() {
                "none".into()
            } else {
                first
            },
            if pocketed.is_empty() {
                "none".into()
            } else {
                pocketed.join(", ")
            }
        )
    });
    let compact = window.single().is_ok_and(|w| w.width() < COMPACT_WIDTH);
    text.0 = if compact {
        format!("{fps:>3.0} fps   {state}")
    } else {
        format!(
            "CUE - phase 2   {fps:>3.0} fps{shot}\n{state}\n\
             aim: pointer or left/right   power: drag back and release, or up/down + space\n\
             spin: WASD (C centres)   re-rack: R   cancel: Esc or right click"
        )
    };
}
