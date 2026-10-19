//! The cue stick: a two-piece pool cue built from tapered cylinders, from the
//! chalked tip to the rubber bumper. Presentation only: the strike is a
//! [`Strike`](cue_physics::Strike) computed from the aim, and the stick
//! merely shows it.
//!
//! The stick's local frame has its origin at the tip and `+Y` running to the
//! butt, so placing it is a translation to the tip and a rotation of `+Y` onto
//! the cue's axis.

use bevy::prelude::*;

use crate::textures;

/// Marker for the cue stick's root entity.
#[derive(Component, Debug)]
pub(crate) struct CueStick;

/// One section of the cue: where it starts along the stick, its length, its
/// radii at both ends (tip side first) and its finish.
struct Section {
    start: f32,
    length: f32,
    radius: (f32, f32),
    colour: Color,
    roughness: f32,
    metallic: f32,
    /// Whether the section carries the inlaid points of the forearm.
    points: bool,
}

const fn section(
    start: f32,
    length: f32,
    radius: (f32, f32),
    colour: Color,
    roughness: f32,
    metallic: f32,
) -> Section {
    Section {
        start,
        length,
        radius,
        colour,
        roughness,
        metallic,
        points: false,
    }
}

/// Polished brass, as on the table's pocket lips and sight line.
const BRASS: Color = Color::srgb(0.86, 0.68, 0.38);
const EBONY: Color = Color::srgb(0.09, 0.06, 0.05);

/// From the tip to the butt, 1.47 m in all (a standard 58-inch cue). Radii
/// follow a real cue: a 13 mm tip, a shaft tapering to the joint, and a butt
/// thickening to 29 mm. Brass at the joint and in two trim rings, and maple
/// points inlaid in the ebony forearm, as on a fine cue.
const SECTIONS: [Section; 10] = [
    // Leather tip, blue with chalk.
    section(
        0.0,
        0.011,
        (0.0063, 0.0065),
        Color::srgb(0.19, 0.36, 0.62),
        0.9,
        0.0,
    ),
    // Ferrule.
    section(
        0.011,
        0.024,
        (0.0065, 0.0066),
        Color::srgb(0.95, 0.94, 0.9),
        0.3,
        0.0,
    ),
    // Maple shaft, lacquered.
    section(
        0.035,
        0.705,
        (0.0066, 0.0103),
        Color::srgb(0.9, 0.8, 0.62),
        0.3,
        0.0,
    ),
    // Brass joint collar.
    section(0.74, 0.016, (0.0108, 0.0108), BRASS, 0.25, 1.0),
    // Ebony forearm with maple points.
    Section {
        points: true,
        ..section(0.756, 0.326, (0.0108, 0.0123), Color::WHITE, 0.28, 0.0)
    },
    // Brass trim ring.
    section(1.082, 0.004, (0.0123, 0.0123), BRASS, 0.25, 1.0),
    // Linen wrap.
    section(
        1.086,
        0.266,
        (0.0123, 0.0134),
        Color::srgb(0.035, 0.035, 0.04),
        0.85,
        0.0,
    ),
    // Brass trim ring.
    section(1.352, 0.004, (0.0134, 0.0134), BRASS, 0.25, 1.0),
    // Butt sleeve.
    section(1.356, 0.1, (0.0134, 0.0144), EBONY, 0.28, 0.0),
    // Rubber bumper.
    section(
        1.456,
        0.014,
        (0.0144, 0.0138),
        Color::srgb(0.02, 0.02, 0.02),
        0.95,
        0.0,
    ),
];

pub(crate) fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let points = images.add(textures::cue_points());
    commands
        .spawn((CueStick, Transform::default(), Visibility::Hidden))
        .with_children(|stick| {
            for s in &SECTIONS {
                stick.spawn((
                    Mesh3d(
                        meshes.add(
                            ConicalFrustum {
                                // The frustum's top is +Y: the butt side.
                                radius_top: s.radius.1,
                                radius_bottom: s.radius.0,
                                height: s.length,
                            }
                            .mesh()
                            .resolution(24),
                        ),
                    ),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: s.colour,
                        base_color_texture: s.points.then(|| points.clone()),
                        perceptual_roughness: s.roughness,
                        metallic: s.metallic,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, s.start + s.length / 2.0, 0.0),
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sections_join_end_to_end_into_one_cue() {
        let mut at = 0.0;
        let mut radius = SECTIONS[0].radius.0;
        for s in &SECTIONS {
            assert!(
                (s.start - at).abs() < 1e-6,
                "gap before section at {}",
                s.start
            );
            // No step down in thickness from tip to butt except the bumper.
            assert!(s.radius.0 + 1e-4 >= radius, "step at {}", s.start);
            at = s.start + s.length;
            radius = s.radius.1;
        }
        assert!((at - 1.47).abs() < 1e-6, "a 58-inch cue");
    }
}
