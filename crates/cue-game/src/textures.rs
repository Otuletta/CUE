//! Shared helpers for procedural textures: colour conversion, mip chains
//! filtered in linear light, and the cloth's grain.

use bevy::{
    asset::RenderAssetUsages,
    image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Builds an sRGB image with a full mip chain from a linear-light base level.
/// Each level averages 2×2 texels of the one above, in linear light, so
/// minified textures keep their brightness and do not shimmer.
pub(crate) fn with_mips(width: u32, height: u32, base: Vec<[f32; 3]>, repeat: bool) -> Image {
    debug_assert_eq!(base.len(), (width * height) as usize);
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
        TextureFormat::Rgba8UnormSrgb,
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

fn encode(level: &[[f32; 3]], out: &mut Vec<u8>) {
    for c in level {
        out.extend(c.map(to_srgb8));
        out.push(255);
    }
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
    fn colour_conversion_round_trips() {
        for v in [0u8, 1, 17, 128, 200, 255] {
            let c = f32::from(v) / 255.0;
            assert_eq!(to_srgb8(to_linear(c)), v);
        }
    }
}
