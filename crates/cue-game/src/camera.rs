//! A single, stable camera that frames the whole table at any aspect ratio.
//!
//! Landscape screens see the table lengthwise; portrait screens see it rotated
//! so it still fills the view. The tilt is slight: enough to read the balls as
//! spheres, not so much that it distorts aiming (ADR-0013).
//!
//! The host's controls cover bands at the top and bottom of the canvas. The
//! camera keeps the playing surface clear of those bands, where the balls
//! are, and lets the cushions and rails run under them. Where the bands are taller than the rails,
//! the table shrinks and moves to the middle of the free area. It moves with
//! an off-axis projection (a Bevy sub view), so the view direction, and with
//! it aiming, is unchanged.

use bevy::{camera::SubCameraView, prelude::*, render::view::Tonemapping, window::PrimaryWindow};

use crate::{scene::RAIL_WIDTH, sim::Simulation};

/// Tilt away from straight down, radians.
const TILT: f32 = 0.2;
/// Empty space around the rails, metres.
const MARGIN: f32 = 0.06;
/// Largest share of the canvas height an inset may take.
const MAX_INSET: f32 = 0.4;

/// Logical pixels at the top and bottom of the canvas covered by the host's
/// controls.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewInsets {
    /// Covered band at the top.
    pub top: f32,
    /// Covered band at the bottom.
    pub bottom: f32,
}

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

/// How the table sits in a `width` × `height` canvas with `insets`, in
/// logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Framing {
    /// Height of the full, virtual image whose middle is the optical axis.
    full_height: f32,
    /// Where the canvas starts within it, from the top.
    offset: f32,
    /// Pixels per metre on the table plane.
    scale: f32,
}

/// `cloth` and `table` are half extents, as `(across, up)` on screen: the
/// part that must stay clear of the insets, and the whole table with its
/// rails and margin, which must stay on the canvas.
fn framing(width: f32, height: f32, insets: ViewInsets, cloth: Vec2, table: Vec2) -> Framing {
    let top = insets.top.clamp(0.0, height * MAX_INSET);
    let bottom = insets.bottom.clamp(0.0, height * MAX_INSET);
    let free = height - top - bottom;
    // The table's centre goes to the middle of the free band.
    let centre = top + free / 2.0;
    let scale = [
        (width / 2.0) / table.x,
        centre.min(height - centre) / table.y,
        (free / 2.0) / cloth.y,
    ]
    .into_iter()
    .fold(f32::INFINITY, f32::min);
    // A virtual image tall enough to put the axis at `centre`.
    let full_height = 2.0 * centre.max(height - centre);
    Framing {
        full_height,
        offset: full_height / 2.0 - centre,
        scale,
    }
}

pub(crate) fn fit_to_window(
    sim: Res<Simulation>,
    insets: Res<ViewInsets>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut camera: Query<(&mut Transform, &mut Camera, &Projection), With<TableCamera>>,
) {
    let (Ok(window), Ok((mut transform, mut camera, projection))) =
        (window.single(), camera.single_mut())
    else {
        return;
    };
    let Projection::Perspective(p) = projection else {
        return;
    };
    let (width, height) = (window.width(), window.height().max(1.0));
    let portrait = width < height;
    let cloth = sim.table().half_extents().as_vec2();
    let table =
        sim.world.geometry().outer_half_extents().as_vec2() + Vec2::splat(RAIL_WIDTH + MARGIN);
    let screen = |v: Vec2| if portrait { Vec2::new(v.y, v.x) } else { v };
    let f = framing(width, height, *insets, screen(cloth), screen(table));

    let tan_v = (p.fov / 2.0).tan();
    // The tilt foreshortens the near-far extent and pushes the near edge
    // towards the camera; a small allowance keeps both rails in frame.
    let distance = (f.full_height / 2.0) / (tan_v * f.scale) * 1.04 + 0.1;
    let offset = Vec3::new(0.0, TILT.cos(), TILT.sin()) * distance;
    let position = if portrait {
        Vec3::new(-offset.z, offset.y, 0.0)
    } else {
        offset
    };
    let next = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
    if *transform != next {
        *transform = next;
    }

    // Off-axis framing, in physical pixels (only the ratios matter).
    let sub_view = (f.full_height > height + 0.5).then(|| {
        let px = |v: f32| (v * window.scale_factor()).round().max(1.0);
        SubCameraView {
            full_size: UVec2::new(px(width) as u32, px(f.full_height) as u32),
            offset: Vec2::new(0.0, px(f.offset)),
            size: UVec2::new(px(width) as u32, px(height) as u32),
        }
    });
    if camera.sub_camera_view != sub_view {
        camera.sub_camera_view = sub_view;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOTH: Vec2 = Vec2::new(1.33, 0.7);
    const TABLE: Vec2 = Vec2::new(1.51, 0.88);

    #[test]
    fn without_insets_the_table_fills_the_canvas_as_before() {
        let f = framing(800.0, 450.0, ViewInsets::default(), CLOTH, TABLE);
        assert_eq!(f.full_height, 450.0);
        assert_eq!(f.offset, 0.0);
        let expected = (400.0 / TABLE.x).min(225.0 / TABLE.y);
        assert!((f.scale - expected).abs() < 1e-3);
    }

    #[test]
    fn controls_that_fit_over_the_rails_change_nothing() {
        let insets = ViewInsets {
            top: 20.0,
            bottom: 20.0,
        };
        let f = framing(800.0, 450.0, insets, CLOTH, TABLE);
        let plain = framing(800.0, 450.0, ViewInsets::default(), CLOTH, TABLE);
        assert_eq!(f.scale, plain.scale, "the rail band is wider than 20 px");
    }

    #[test]
    fn tall_controls_keep_the_cloth_clear_and_centre_it_in_the_free_band() {
        // A phone in portrait: a tall top bar, a shorter dock.
        let (w, h) = (343.0, 585.0);
        let insets = ViewInsets {
            top: 170.0,
            bottom: 60.0,
        };
        let (cloth, table) = (Vec2::new(CLOTH.y, CLOTH.x), Vec2::new(TABLE.y, TABLE.x));
        let f = framing(w, h, insets, cloth, table);
        let centre = 170.0 + (h - 230.0) / 2.0;
        // The cloth spans exactly the free band, or less.
        assert!(f.scale * cloth.y <= (h - 230.0) / 2.0 + 1e-3);
        // The axis sits at the centre of the free band.
        assert!((f.full_height / 2.0 - f.offset - centre).abs() < 1e-3);
        assert!(f.offset >= 0.0 && f.offset + h <= f.full_height + 1e-3);
    }

    #[test]
    fn absurd_insets_are_clamped() {
        let insets = ViewInsets {
            top: 10_000.0,
            bottom: 10_000.0,
        };
        let f = framing(800.0, 450.0, insets, CLOTH, TABLE);
        assert!(f.scale.is_finite() && f.scale > 0.0);
    }
}
