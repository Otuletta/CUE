//! Procedural ball textures: no image assets, no licensing questions.
//!
//! Each texture is an equirectangular map for Bevy's UV sphere. Markings are
//! defined on the unit sphere and anti-aliased by angular distance, so they
//! stay crisp at the poles where an equirectangular map is most stretched.
//! Number glyphs arrive with the final ball materials (Phase 5); until then
//! the white number discs, stripe bands and the cue ball's dots are enough to
//! make rolling, sliding and spin readable.

use std::f32::consts::{PI, TAU};

use bevy::{
    asset::RenderAssetUsages,
    image::Image,
    math::Vec3,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use cue_core::{BallGroup, BallId};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 128;

/// Angular radius of the white number disc, radians.
const NUMBER_DISC: f32 = 0.36;
/// Half-height of a stripe band, as a latitude in radians.
const STRIPE_HALF_HEIGHT: f32 = 0.43;
/// Angular radius of the cue ball's marker dots, radians.
const CUE_DOT: f32 = 0.11;
/// Edge softness, about one texel at the equator.
const EDGE: f32 = TAU / WIDTH as f32;

const IVORY: [f32; 3] = [0.97, 0.96, 0.91];
const DOT_RED: [f32; 3] = [0.75, 0.10, 0.12];
const EIGHT_BLACK: [f32; 3] = [0.04, 0.04, 0.045];

/// Conventional pool ball hues for 1–7 (repeated for 9–15), sRGB.
const HUES: [[f32; 3]; 7] = [
    [0.96, 0.76, 0.08], // yellow
    [0.08, 0.24, 0.72], // blue
    [0.80, 0.10, 0.10], // red
    [0.36, 0.14, 0.52], // purple
    [0.96, 0.44, 0.08], // orange
    [0.06, 0.46, 0.24], // green
    [0.48, 0.12, 0.08], // maroon
];

/// The base colour of a ball, for UI swatches and fallbacks.
pub(crate) fn base_colour(id: BallId) -> [f32; 3] {
    match id.group() {
        BallGroup::Cue => IVORY,
        BallGroup::Eight => EIGHT_BLACK,
        BallGroup::Solids => HUES[usize::from(id.number() - 1)],
        BallGroup::Stripes => HUES[usize::from(id.number() - 9)],
    }
}

/// Builds the texture for one ball.
pub(crate) fn ball_image(id: BallId) -> Image {
    let colour = base_colour(id);
    let mut data = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let lon = (x as f32 + 0.5) / WIDTH as f32 * TAU;
            let lat = (0.5 - (y as f32 + 0.5) / HEIGHT as f32) * PI;
            let dir = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
            let rgb = shade(id, colour, dir, lat);
            data.extend(
                rgb.iter()
                    .map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8),
            );
            data.push(255);
        }
    }
    Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn shade(id: BallId, colour: [f32; 3], dir: Vec3, lat: f32) -> [f32; 3] {
    match id.group() {
        BallGroup::Cue => {
            // Six dots on the axes, like a measle practice ball: every
            // rotation axis is visible.
            let near = [Vec3::X, Vec3::Y, Vec3::Z]
                .iter()
                .map(|axis| dir.dot(*axis).abs().clamp(-1.0, 1.0).acos())
                .fold(f32::MAX, f32::min);
            mix(IVORY, DOT_RED, coverage(near, CUE_DOT))
        }
        BallGroup::Solids | BallGroup::Eight => mix(
            colour,
            IVORY,
            coverage(number_disc_distance(dir), NUMBER_DISC),
        ),
        BallGroup::Stripes => {
            let band = coverage(lat.abs(), STRIPE_HALF_HEIGHT);
            let body = mix(IVORY, colour, band);
            mix(
                body,
                IVORY,
                coverage(number_disc_distance(dir), NUMBER_DISC),
            )
        }
    }
}

/// Angle to the nearer of the two number discs on opposite sides of the
/// equator.
fn number_disc_distance(dir: Vec3) -> f32 {
    dir.x.abs().clamp(-1.0, 1.0).acos()
}

/// 1 inside a disc of angular `radius`, 0 outside, smooth across the edge.
fn coverage(distance: f32, radius: f32) -> f32 {
    let t = ((radius + EDGE * 0.5 - distance) / EDGE).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}
