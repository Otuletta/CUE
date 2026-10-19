//! Shared helpers for procedural textures: colour conversion, mip chains
//! filtered in linear light, the cloth's grain, the rails' wood and the
//! pockets' depth.

use bevy::{
    asset::RenderAssetUsages,
    image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Builds an sRGB image with a full mip chain from a linear-light base level.
/// Each level averages 2×2 texels of the one above, in linear light, so
/// minified textures keep their brightness and do not shimmer.
pub(crate) fn with_mips(width: u32, height: u32, base: Vec<[f32; 3]>, repeat: bool) -> Image {
    build(width, height, base, repeat, true)
}

/// Builds a linear (data, not colour) image with a full mip chain, for maps
/// such as roughness that the shader reads as numbers.
pub(crate) fn with_linear_mips(width: u32, height: u32, base: Vec<[f32; 3]>) -> Image {
    build(width, height, base, false, false)
}

fn build(width: u32, height: u32, base: Vec<[f32; 3]>, repeat: bool, srgb: bool) -> Image {
    debug_assert_eq!(base.len(), (width * height) as usize);
    let encode = |level: &[[f32; 3]], out: &mut Vec<u8>| {
        for c in level {
            out.extend(c.map(if srgb { to_srgb8 } else { to_unorm8 }));
            out.push(255);
        }
    };
    let mut data = Vec::with_capacity((width * height * 4 * 4 / 3) as usize + 4);
    let (mut w, mut h) = (width, height);
    let mut level = base;
    let mut levels = 1;
    encode(&level, &mut data);
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = Vec::with_capacity((nw * nh) as usize);
        for y in 0..nh {
            for x in 0..nw {
                let at = |dx: u32, dy: u32| {
                    level[((2 * y + dy).min(h - 1) * w + (2 * x + dx).min(w - 1)) as usize]
                };
                let quad = [at(0, 0), at(1, 0), at(0, 1), at(1, 1)];
                next.push([0, 1, 2].map(|i| quad.iter().map(|c| c[i]).sum::<f32>() / 4.0));
            }
        }
        encode(&next, &mut data);
        level = next;
        (w, h) = (nw, nh);
        levels += 1;
    }

    let mut image = Image::new_uninit(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        if srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(data);
    if repeat {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
    }
    image
}

/// sRGB component to linear light.
pub(crate) fn to_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light to an 8-bit sRGB component.
fn to_srgb8(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// A value in 0–1 to an 8-bit component, as is.
fn to_unorm8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Side of the cloth grain tile, texels.
pub(crate) const GRAIN_SIZE: u32 = 256;

/// A tileable, near-white grain for the cloth: fine fibres over a faint
/// mottle, within a few per cent of white so the theme's colour comes
/// through. Multiplied with the cloth colour by the material.
pub(crate) fn cloth_grain() -> Image {
    let n = GRAIN_SIZE;
    let mut base = Vec::with_capacity((n * n) as usize);
    for y in 0..n {
        for x in 0..n {
            let (u, v) = (x as f32 / n as f32, y as f32 / n as f32);
            // Octaves on lattices that divide the tile, so it repeats.
            let fibre = value_noise(u, v, 64) * 0.6 + value_noise(u, v, 128) * 0.4;
            let mottle = value_noise(u, v, 8);
            let shade = 0.955 + 0.03 * fibre + 0.015 * mottle;
            base.push([shade; 3]);
        }
    }
    with_mips(n, n, base, true)
}

/// Size of the wood tile, texels: along the grain, then across it.
pub(crate) const WOOD_SIZE: (u32, u32) = (512, 128);

/// A tileable wood grain for the rails: growth rings, warped and stretched
/// along the board, with fine streaks. Between about 62 % and 100 % of
/// white, multiplied with the theme's wood colour by the material.
pub(crate) fn wood_grain() -> Image {
    let (w, h) = WOOD_SIZE;
    let mut base = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            // The texture is four times longer than wide, so noise on a
            // square lattice comes out stretched along the grain.
            let warp = value_noise(u, v, 4) * 1.1 + value_noise(u, v, 16) * 0.25;
            let ring = 0.5 + 0.5 * (std::f32::consts::TAU * (v * 10.0 + warp)).cos();
            let streak = value_noise(u, v, 64) * 0.6 + value_noise(u, v, 128) * 0.4;
            let shade = 0.62 + 0.22 * ring * ring + 0.16 * streak;
            // A little warmer in the light rings, as in real timber.
            base.push([
                shade,
                shade * (0.97 + 0.03 * ring),
                shade * (0.94 + 0.06 * ring),
            ]);
        }
    }
    with_mips(w, h, base, true)
}

/// Side of the pocket texture, texels.
const POCKET_SIZE: u32 = 64;

/// The inside of a pocket seen from above: black in the middle, lifting to
/// a dark brown at the rim, where the liner catches a little light.
pub(crate) fn pocket_depth() -> Image {
    let n = POCKET_SIZE;
    let mut base = Vec::with_capacity((n * n) as usize);
    for y in 0..n {
        for x in 0..n {
            let (dx, dy) = (
                (x as f32 + 0.5) / n as f32 * 2.0 - 1.0,
                (y as f32 + 0.5) / n as f32 * 2.0 - 1.0,
            );
            let r = (dx * dx + dy * dy).sqrt().min(1.0);
            let lift = r.powi(4) * 0.05;
            base.push([lift, lift * 0.75, lift * 0.55]);
        }
    }
    with_mips(n, n, base, false)
}

/// Smooth value noise in `[0, 1]`, periodic over the unit square with
/// `cells` lattice cells per side.
fn value_noise(u: f32, v: f32, cells: u32) -> f32 {
    let (x, y) = (u * cells as f32, v * cells as f32);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sy) = (smooth(fx), smooth(fy));
    let corner = |dx: u32, dy: u32| {
        let ix = (x0 as u32 + dx) % cells;
        let iy = (y0 as u32 + dy) % cells;
        hash(ix, iy, cells)
    };
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * sx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * sx;
    top + (bottom - top) * sy
}

/// A deterministic pseudo-random value in `[0, 1)` per lattice point.
fn hash(x: u32, y: u32, salt: u32) -> f32 {
    let mut h = x
        .wrapping_mul(0x8da6_b343)
        .wrapping_add(y.wrapping_mul(0xd816_3841))
        .wrapping_add(salt.wrapping_mul(0xcb1a_b31f));
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Size of the cue's forearm texture, texels: round the cue, then along it.
const POINTS_SIZE: (u32, u32) = (256, 512);
/// Circumference and length of the forearm, metres, so the inlay's lines
/// keep one width whichever way they run.
const FOREARM: (f32, f32) = (0.072, 0.326);

/// The forearm of the cue: four maple points inlaid in ebony, each edged
/// with a thin brass-coloured veneer, rising from the wrap towards the joint.
/// The texture wraps round the frustum (`u`) and runs from the joint (`v` = 0)
/// to the wrap (`v` = 1).
pub(crate) fn cue_points() -> Image {
    const EBONY: [f32; 3] = [0.09, 0.06, 0.05];
    const MAPLE: [f32; 3] = [0.88, 0.76, 0.56];
    const VENEER: [f32; 3] = [0.86, 0.68, 0.38];
    // Where the points' tips are along the forearm, and their half-width at
    // the wrap as a fraction of the circumference.
    const APEX: f32 = 0.22;
    const BASE: f32 = 0.085;
    // The veneer's width, metres.
    const LINE: f32 = 0.0012;
    let (w, h) = POINTS_SIZE;
    let texel = FOREARM.1 / h as f32;
    let mut base = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            // Across from the nearest point's centre line, metres.
            let across = ((u * 4.0 + 0.5).fract() - 0.5).abs() / 4.0 * FOREARM.0;
            // The point's edge: a straight taper from the tip to the wrap.
            let half = (v - APEX).max(0.0) / (1.0 - APEX) * BASE * FOREARM.0;
            // Signed distance to the edge, roughly: negative inside.
            let slope = BASE * FOREARM.0 / ((1.0 - APEX) * FOREARM.1);
            let edge = if v < APEX {
                ((v - APEX) * FOREARM.1).hypot(across)
            } else {
                (across - half) / slope.hypot(1.0)
            };
            let inside = smooth(-edge / texel);
            let veneer = smooth((LINE - edge.abs()) / texel);
            base.push(mix3(mix3(EBONY, MAPLE, inside), VENEER, veneer).map(to_linear));
        }
    }
    with_mips(w, h, base, false)
}

/// 0 below −½, 1 above ½, smooth between: a one-texel edge.
fn smooth(t: f32) -> f32 {
    let t = (t + 0.5).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grain_tiles_and_stays_close_to_white() {
        let n = GRAIN_SIZE as f32;
        for i in 0..32 {
            let v = i as f32 / 32.0;
            // The left edge continues the right edge.
            let left = value_noise(0.0, v, 64);
            let right = value_noise((n - 1e-4) / n, v, 64);
            assert!((left - right).abs() < 0.05, "seam at {v}");
        }
        let image = cloth_grain();
        let data = image.data.as_ref().unwrap();
        let level0 = &data[..(GRAIN_SIZE * GRAIN_SIZE * 4) as usize];
        let min = level0.chunks(4).map(|p| p[0]).min().unwrap();
        assert!(min > 235, "too dark: {min}");
    }

    #[test]
    fn the_wood_tiles_and_keeps_its_range() {
        let (w, h) = WOOD_SIZE;
        let image = wood_grain();
        let data = image.data.as_ref().unwrap();
        let level0 = &data[..(w * h * 4) as usize];
        let texel = |x: u32, y: u32| level0[((y * w + x) * 4) as usize];
        for y in (0..h).step_by(8) {
            // The last column runs into the first.
            assert!(
                texel(0, y).abs_diff(texel(w - 1, y)) < 24,
                "seam at row {y}"
            );
        }
        let (min, max) = level0
            .chunks(4)
            .fold((255, 0), |(lo, hi), p| (lo.min(p[0]), hi.max(p[0])));
        assert!(min > 150 && max > 240, "range {min}–{max}");
    }

    #[test]
    fn colour_conversion_round_trips() {
        for v in [0u8, 1, 17, 128, 200, 255] {
            let c = f32::from(v) / 255.0;
            assert_eq!(to_srgb8(to_linear(c)), v);
        }
    }
}
