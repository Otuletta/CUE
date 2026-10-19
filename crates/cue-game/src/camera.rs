//! A single, stable camera straight above the table, framing all of it at
//! any aspect ratio (ADR-0014).
//!
//! The projection is orthographic: every part of the cloth is drawn at the
//! same scale, so angles on screen are angles on the table and the table
//! reads as a crisp, designed object. Light, shadows and gloss stay 3D.
//!
//! Landscape screens see the table lengthwise; portrait screens see it rotated
//! so it still fills the view.
//!
//! The host's controls cover bands at the top and bottom of the canvas. The
//! camera keeps the playing surface clear of those bands, where the balls
//! are, and lets the cushions and rails run under them. Where the bands are
//! taller than the rails, the table shrinks and moves to the middle of the
//! free area: the camera slides along the table, which an orthographic view
//! allows without changing anything else.

use bevy::{camera::ScalingMode, prelude::*, render::view::Tonemapping, window::PrimaryWindow};

use crate::{scene::RAIL_WIDTH, sim::Simulation};

/// Height of the camera over the cloth, metres. With an orthographic view
/// it only has to clear the lamp's light cones and keep the floor in range.
const HEIGHT: f32 = 3.0;
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
        Projection::Orthographic(OrthographicProjection {
            // Replaced on the first frame by `fit_to_window`.
            scaling_mode: ScalingMode::Fixed {
                width: 3.0,
                height: 1.7,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, HEIGHT, 0.0).looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
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
    mut camera: Query<(&mut Transform, &mut Projection), With<TableCamera>>,
) {
    let (Ok(window), Ok((mut transform, mut projection))) = (window.single(), camera.single_mut())
    else {
        return;
    };
    let Projection::Orthographic(ortho) = projection.as_mut() else {
        return;
    };
    let (width, height) = (window.width(), window.height().max(1.0));
    let portrait = width < height;
    let cloth = sim.table().half_extents().as_vec2();
    let table =
        sim.world.geometry().outer_half_extents().as_vec2() + Vec2::splat(RAIL_WIDTH + MARGIN);
    let screen = |v: Vec2| if portrait { Vec2::new(v.y, v.x) } else { v };
    let f = framing(width, height, *insets, screen(cloth), screen(table));

    // Where the cloth's centre goes, in pixels from the top, and how far
    // that is below the canvas's middle, in metres on the table.
    let centre = f.full_height / 2.0 - f.offset;
    let shift = (centre - height / 2.0) / f.scale;
    // Screen up is the far rail in landscape and the foot in portrait. The
    // camera slides up the screen for the table to sit lower on it.
    let up = if portrait { Vec3::X } else { Vec3::NEG_Z };
    let next =
        Transform::from_translation(Vec3::Y * HEIGHT + up * shift).looking_to(Vec3::NEG_Y, up);
    if *transform != next {
        *transform = next;
    }
    let (w, h) = (width / f.scale, height / f.scale);
    // Written only on change, so the projection is not marked changed each frame.
    let same = matches!(ortho.scaling_mode, ScalingMode::Fixed { width, height } if (width - w).abs() < 1e-6 && (height - h).abs() < 1e-6);
    if !same {
        ortho.scaling_mode = ScalingMode::Fixed {
            width: w,
            height: h,
        };
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
