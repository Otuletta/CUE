//! A single, stable camera that frames the whole table at any aspect ratio.
//!
//! Landscape screens see the table lengthwise; portrait screens see it rotated
//! so it still fills the view. The tilt is slight: enough to read the balls as
//! spheres, not so much that it distorts aiming (ADR-005 is still open).

use bevy::{prelude::*, render::view::Tonemapping, window::PrimaryWindow};

use crate::{scene::RAIL_WIDTH, sim::Simulation};

/// Tilt away from straight down, radians.
const TILT: f32 = 0.2;
/// Empty space around the rails, metres.
const MARGIN: f32 = 0.06;

#[derive(Component, Debug)]
pub(crate) struct TableCamera;

pub(crate) fn spawn(mut commands: Commands) {
    commands.spawn((
        TableCamera,
        Camera3d::default(),
        // LUT-free operator: avoids shipping tonemapping lookup tables.
        Tonemapping::AcesFitted,
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_4,
            ..default()
        }),
        Transform::from_xyz(0.0, 3.0, 0.6).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub(crate) fn fit_to_window(
    sim: Res<Simulation>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut Transform, &Projection), With<TableCamera>>,
) {
    let (Ok(window), Ok((mut transform, projection))) = (window.single(), camera.single_mut())
    else {
        return;
    };
    let Projection::Perspective(p) = projection else {
        return;
    };
    let aspect = window.width() / window.height().max(1.0);
    let half =
        sim.world.geometry().outer_half_extents().as_vec2() + Vec2::splat(RAIL_WIDTH + MARGIN);
    let portrait = aspect < 1.0;
    let (across, up) = if portrait {
        (half.y, half.x)
    } else {
        (half.x, half.y)
    };

    let tan_v = (p.fov / 2.0).tan();
    // The tilt foreshortens the near-far extent and pushes the near edge
    // towards the camera; a small allowance keeps both rails in frame.
    let distance = (up / tan_v).max(across / (tan_v * aspect)) * 1.04 + 0.1;
    let offset = Vec3::new(0.0, TILT.cos(), TILT.sin()) * distance;
    let position = if portrait {
        Vec3::new(-offset.z, offset.y, 0.0)
    } else {
        offset
    };
    *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
}
