//! Procedural ball textures: no image assets, no fonts, no licensing
//! questions.
//!
//! Each texture is an equirectangular map for Bevy's UV sphere, whose poles
//! lie on the mesh's Z axis. Markings are defined on the unit sphere and
//! anti-aliased by angular distance, so they stay crisp at the poles where an
//! equirectangular map is most stretched.
//!
//! Numbers are drawn from a small stroke font (straight strokes and circular
//! arcs) as distance fields, inside two white discs on opposite sides of the
//! ball. Each disc is read upright with the pole axis as "up", so on a stripe
//! the band runs horizontally under the number, as on a real ball. Every
//! texture carries a full mip chain filtered in linear light, so small, distant
//! balls do not shimmer.

use std::f32::consts::{PI, TAU};

use bevy::{
    image::Image,
    math::{Quat, Vec2, Vec3},
};
use cue_core::{BallGroup, BallId};

use crate::textures::{to_linear, with_mips};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 256;

/// Angular radius of the white number disc, radians.
const NUMBER_DISC: f32 = 0.36;
/// Half-height of a stripe band, as a latitude in radians.
const STRIPE_HALF_HEIGHT: f32 = 0.43;
/// Angular radius of the cue ball's marker dots, radians.
const CUE_DOT: f32 = 0.11;
/// Edge softness, about one texel at the equator.
const EDGE: f32 = TAU / WIDTH as f32;
/// Glyph units (a digit is 10 tall) to tangent-plane units on the disc.
const GLYPH_SCALE: f32 = 0.036;
/// Half the stroke width of the digits, in glyph units.
const STROKE: f32 = 0.72;
/// Space between the two digits of 10–15, in glyph units.
const DIGIT_GAP: f32 = 1.2;
/// Size of a two-digit number relative to a single digit.
const TWO_DIGIT_SCALE: f32 = 0.88;

const IVORY: [f32; 3] = [0.97, 0.96, 0.91];
const INK: [f32; 3] = [0.06, 0.06, 0.07];
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

/// The orientation that shows a ball's number from above: the +X disc faces
/// up and reads upright with screen-up towards the far rail, then a small
/// `yaw`, as when a rack is set by hand.
pub(crate) fn resting_orientation(yaw: f32) -> Quat {
    // Mesh X (the disc) → world +Y, mesh Z (the glyph's up) → world −Z.
    let up = Quat::from_mat3(&bevy::math::Mat3::from_cols(Vec3::Y, Vec3::X, Vec3::NEG_Z));
    Quat::from_rotation_y(yaw) * up
}

// --- The stroke font ---------------------------------------------------------

/// A stroke of a digit, in glyph units: a digit is 10 units tall and centred
/// on the origin.
#[derive(Clone, Copy, Debug)]
enum Stroke {
    /// A straight stroke between two points.
    Line(Vec2, Vec2),
    /// A circular arc: centre, radius, start angle and sweep in degrees
    /// (negative sweeps run clockwise).
    Arc(Vec2, f32, f32, f32),
}

const fn p(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// Width between the outer stroke centres of most digits, in glyph units.
const W: f32 = 4.6;

/// The strokes of each digit and its width between stroke centres.
const DIGITS: [(&[Stroke], f32); 10] = {
    use Stroke::{Arc, Line};
    [
        (
            &[
                Arc(p(0.0, 2.0), 2.3, 0.0, 180.0),
                Arc(p(0.0, -2.0), 2.3, 180.0, 180.0),
                Line(p(-2.3, -2.0), p(-2.3, 2.0)),
                Line(p(2.3, -2.0), p(2.3, 2.0)),
            ],
            W,
        ),
        (
            &[
                Line(p(1.1, -4.3), p(1.1, 4.3)),
                Line(p(1.1, 4.3), p(-1.1, 2.5)),
            ],
            2.2,
        ),
        (
            &[
                Arc(p(0.0, 2.0), 2.3, 160.0, -195.0),
                Line(p(1.88, 0.68), p(-2.3, -4.3)),
                Line(p(-2.3, -4.3), p(2.3, -4.3)),
            ],
            W,
        ),
        (
            &[
                Arc(p(0.0, 2.2), 2.1, 155.0, -245.0),
                Arc(p(0.0, -2.1), 2.2, 90.0, -245.0),
            ],
            W,
        ),
        (
            &[
                Line(p(1.0, 4.3), p(-2.3, -1.5)),
                Line(p(-2.3, -1.5), p(2.3, -1.5)),
                Line(p(1.0, 4.3), p(1.0, -4.3)),
            ],
            W,
        ),
        (
            &[
                Line(p(2.1, 4.3), p(-1.8, 4.3)),
                Line(p(-1.8, 4.3), p(-1.4, -0.18)),
                Arc(p(0.0, -2.0), 2.3, 127.5, -267.5),
            ],
            W,
        ),
        (
            &[
                Arc(p(0.0, -2.0), 2.3, 0.0, 360.0),
                Line(p(1.5, 4.3), p(-1.99, -0.85)),
            ],
            W,
        ),
        (
            &[
                Line(p(-2.3, 4.3), p(2.3, 4.3)),
                Line(p(2.3, 4.3), p(-0.7, -4.3)),
            ],
            W,
        ),
        (
            &[
                Arc(p(0.0, 2.25), 2.05, 0.0, 360.0),
                Arc(p(0.0, -1.95), 2.35, 0.0, 360.0),
            ],
            W,
        ),
        (
            &[
                Arc(p(0.0, 2.0), 2.3, 0.0, 360.0),
                Line(p(-1.5, -4.3), p(1.99, 0.85)),
            ],
            W,
        ),
    ]
};

/// The strokes of digit `d` (0–9) and its width between stroke centres.
fn digit(d: u8) -> (&'static [Stroke], f32) {
    DIGITS[usize::from(d % 10)]
}

/// Distance from `q` to a stroke's centre line.
fn stroke_distance(q: Vec2, stroke: &Stroke) -> f32 {
    match *stroke {
        Stroke::Line(a, b) => {
            let ab = b - a;
            let t = ((q - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            q.distance(a + ab * t)
        }
        Stroke::Arc(c, r, start, sweep) => {
            let d = q - c;
            let angle = d.y.atan2(d.x).to_degrees();
            // How far along the sweep `angle` is, in the sweep's direction.
            let along = if sweep >= 0.0 {
                (angle - start).rem_euclid(360.0)
            } else {
                (start - angle).rem_euclid(360.0)
            };
            if along <= sweep.abs() {
                (d.length() - r).abs()
            } else {
                let end = (start + sweep).to_radians();
                let start = start.to_radians();
                let a = c + Vec2::new(start.cos(), start.sin()) * r;
                let b = c + Vec2::new(end.cos(), end.sin()) * r;
                q.distance(a).min(q.distance(b))
            }
        }
    }
}

/// Distance from `q` (glyph units, number centred on the origin) to the ink
/// of `number`'s digits. Two-digit numbers are set a little smaller, so they
/// keep the same margin in the disc.
fn number_distance(q: Vec2, number: u8) -> f32 {
    if number >= 10 {
        return number_distance_at(q / TWO_DIGIT_SCALE, &[number / 10, number % 10])
            * TWO_DIGIT_SCALE;
    }
    number_distance_at(q, &[number])
}

fn number_distance_at(q: Vec2, digits: &[u8]) -> f32 {
    let widths: Vec<f32> = digits.iter().map(|&d| digit(d).1).collect();
    let total = widths.iter().sum::<f32>() + DIGIT_GAP * (digits.len() - 1) as f32;
    let mut left = -total / 2.0;
    let mut best = f32::INFINITY;
    for (&d, &w) in digits.iter().zip(&widths) {
        let centre = left + w / 2.0;
        let local = q - Vec2::new(centre, 0.0);
        for stroke in digit(d).0 {
            best = best.min(stroke_distance(local, stroke));
        }
        left += w + DIGIT_GAP;
    }
    best
}

// --- Shading ---------------------------------------------------------------

/// Builds the texture for one ball, with its mip chain.
pub(crate) fn ball_image(id: BallId) -> Image {
    let colour = base_colour(id);
    let mut level: Vec<[f32; 3]> = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    let lons: Vec<(f32, f32)> = (0..WIDTH)
        .map(|x| {
            let lon = (x as f32 + 0.5) / WIDTH as f32 * TAU;
            (lon.cos(), lon.sin())
        })
        .collect();
    for y in 0..HEIGHT {
        let lat = (0.5 - (y as f32 + 0.5) / HEIGHT as f32) * PI;
        let (sin_lat, cos_lat) = lat.sin_cos();
        for &(cos_lon, sin_lon) in &lons {
            // Bevy's UV sphere: (cos lat cos lon, cos lat sin lon, sin lat).
            let dir = Vec3::new(cos_lat * cos_lon, cos_lat * sin_lon, sin_lat);
            level.push(linear(shade(id, colour, dir, lat)));
        }
    }

    with_mips(WIDTH, HEIGHT, level, false)
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
        BallGroup::Solids | BallGroup::Eight => numbered(colour, dir, id.number()),
        BallGroup::Stripes => {
            let band = coverage(lat.abs(), STRIPE_HALF_HEIGHT);
            numbered(mix(IVORY, colour, band), dir, id.number())
        }
    }
}

/// `body` with a white disc on each side of the X axis, and the number in it.
fn numbered(body: [f32; 3], dir: Vec3, number: u8) -> [f32; 3] {
    let disc = coverage(number_disc_distance(dir), NUMBER_DISC);
    if disc <= 0.0 {
        return body;
    }
    // Tangent-plane coordinates on the disc, read from outside the ball with
    // the pole axis (Z) as up: right is +Y on the +X disc, −Y on the −X disc.
    let x = dir.x.abs().max(1e-3);
    let q = if dir.x >= 0.0 {
        Vec2::new(dir.y / x, dir.z / x)
    } else {
        Vec2::new(-dir.y / x, dir.z / x)
    };
    let distance = number_distance(q / GLYPH_SCALE, number) * GLYPH_SCALE;
    let ink = coverage(distance, STROKE * GLYPH_SCALE);
    mix(body, mix(IVORY, INK, ink), disc)
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

fn linear(c: [f32; 3]) -> [f32; 3] {
    c.map(to_linear)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ink coverage of `number` at glyph-space point `q`.
    fn ink(number: u8, q: Vec2) -> bool {
        number_distance(q, number) < STROKE
    }

    #[test]
    fn every_digit_is_drawn_inside_its_box() {
        for d in 0..=9 {
            let (strokes, width) = digit(d);
            assert!(!strokes.is_empty());
            // Sample a grid: ink only inside the glyph's box plus the stroke.
            for i in -60..=60 {
                for j in -60..=60 {
                    let q = Vec2::new(i as f32 * 0.1, j as f32 * 0.1);
                    if number_distance(q, d) < STROKE {
                        assert!(q.x.abs() <= width / 2.0 + STROKE + 1e-3, "{d} at {q}");
                        assert!(q.y.abs() <= 5.0 + 1e-3, "{d} at {q}");
                    }
                }
            }
        }
    }

    #[test]
    fn digits_are_told_apart_by_their_ink() {
        // A coarse bitmap of each digit must differ from every other one.
        let bitmap = |d: u8| -> Vec<bool> {
            let mut bits = Vec::new();
            for j in -4..=4 {
                for i in -2..=2 {
                    bits.push(ink(d, Vec2::new(i as f32 * 1.1, j as f32 * 1.05)));
                }
            }
            bits
        };
        let maps: Vec<_> = (0..=9).map(bitmap).collect();
        for a in 0..10 {
            for b in a + 1..10 {
                assert_ne!(maps[a], maps[b], "{a} and {b} look the same");
            }
        }
        // Not mirrored: a 2's foot is on the left of its diagonal's top.
        assert!(ink(2, Vec2::new(-2.3, -4.3)) && ink(2, Vec2::new(2.3, -4.3)));
        assert!(ink(7, Vec2::new(2.3, 4.3)) && !ink(7, Vec2::new(-2.0, -4.0)));
    }

    #[test]
    fn two_digit_numbers_fit_in_the_disc() {
        let radius = NUMBER_DISC.tan() / GLYPH_SCALE;
        for n in 10..=15 {
            for i in -80..=80 {
                for j in -60..=60 {
                    let q = Vec2::new(i as f32 * 0.1, j as f32 * 0.1);
                    if number_distance(q, n) < STROKE {
                        assert!(q.length() < radius * 0.9, "{n} spills at {q}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_disc_reads_upright_from_outside() {
        // The top of the 7's bar is at +Z on both discs; its right end is +Y
        // on the +X disc and −Y on the −X disc.
        let at = |dir: Vec3| numbered([1.0, 0.0, 0.0], dir.normalize(), 7);
        let s = GLYPH_SCALE;
        let top_right = |side: f32| Vec3::new(side, side * 2.3 * s, 4.3 * s);
        let top_left = |side: f32| Vec3::new(side, -side * 2.3 * s, 4.3 * s);
        let foot_left = |side: f32| Vec3::new(side, -side * 0.7 * s, -4.3 * s);
        for side in [1.0, -1.0] {
            assert!(
                at(top_right(side))[1] < 0.2,
                "ink at the top right ({side})"
            );
            assert!(at(top_left(side))[1] < 0.2, "ink at the top left ({side})");
            assert!(at(foot_left(side))[1] < 0.2, "ink at the foot ({side})");
            // The other side of the foot is paper.
            let paper = Vec3::new(side, side * 2.0 * s, -4.0 * s);
            assert!(at(paper)[1] > 0.8, "paper at the bottom right ({side})");
        }
    }

    #[test]
    fn the_mip_chain_is_complete() {
        let image = ball_image(BallId::new(11).unwrap());
        let levels = image.texture_descriptor.mip_level_count;
        assert_eq!(levels, 10);
        let expected: u32 = (0..levels)
            .map(|l| ((WIDTH >> l).max(1)) * ((HEIGHT >> l).max(1)) * 4)
            .sum();
        assert_eq!(image.data.as_ref().unwrap().len() as u32, expected);
    }

    #[test]
    fn resting_orientation_shows_the_disc_from_above() {
        let q = resting_orientation(0.0);
        assert!((q * Vec3::X).distance(Vec3::Y) < 1e-6);
        assert!((q * Vec3::Z).distance(Vec3::NEG_Z) < 1e-6);
    }
}
