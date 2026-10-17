//! Table and ball meshes.
//!
//! Render geometry is derived from the simulation's [`TableSpec`] but is
//! never used for collision: the solver has its own exact model.

use bevy::prelude::*;
use cue_core::BallId;

use crate::{
    ball_texture,
    sim::{Ball, Simulation},
};

/// Width of the cushion rubber, metres.
pub(crate) const CUSHION_WIDTH: f32 = 0.05;
/// Width of the wooden rail outside the cushion, metres.
pub(crate) const RAIL_WIDTH: f32 = 0.12;
/// Height of the cushion top above the cloth, metres.
const CUSHION_HEIGHT: f32 = 0.044;

pub(crate) fn spawn_table(
    mut commands: Commands,
    sim: Res<Simulation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = sim.table();
    let (l, w) = (t.playfield_length as f32, t.playfield_width as f32);
    let (cw, rw) = (CUSHION_WIDTH, RAIL_WIDTH);

    let cloth = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.30, 0.24),
        perceptual_roughness: 0.92,
        reflectance: 0.2,
        ..default()
    });
    let cushion = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.25, 0.20),
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    });
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.09, 0.05),
        perceptual_roughness: 0.45,
        reflectance: 0.35,
        ..default()
    });

    let mut slab = |size: Vec3, at: Vec3, material: &Handle<StandardMaterial>| {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at),
        ));
    };

    // Cloth: top surface at y = 0, extending under the cushions.
    slab(
        Vec3::new(l + 2.0 * cw, 0.03, w + 2.0 * cw),
        Vec3::new(0.0, -0.015, 0.0),
        &cloth,
    );

    // Cushions along each side of the playing surface.
    let h = CUSHION_HEIGHT;
    for side in [-1.0, 1.0] {
        slab(
            Vec3::new(l, h, cw),
            Vec3::new(0.0, h / 2.0, side * (w + cw) / 2.0),
            &cushion,
        );
        slab(
            Vec3::new(cw, h, w),
            Vec3::new(side * (l + cw) / 2.0, h / 2.0, 0.0),
            &cushion,
        );
    }

    // Wooden rails, slightly proud of the cushions.
    let rh = h + 0.012;
    let outer = Vec2::new(l + 2.0 * cw, w + 2.0 * cw);
    for side in [-1.0, 1.0] {
        slab(
            Vec3::new(outer.x + 2.0 * rw, rh, rw),
            Vec3::new(0.0, rh / 2.0 - 0.006, side * (outer.y + rw) / 2.0),
            &wood,
        );
        slab(
            Vec3::new(rw, rh, outer.y),
            Vec3::new(side * (outer.x + rw) / 2.0, rh / 2.0 - 0.006, 0.0),
            &wood,
        );
    }

    // A soft key light from above, as over a real table.
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.4, 4.0, 1.2).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub(crate) fn spawn_balls(
    mut commands: Commands,
    sim: Res<Simulation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = sim.table().ball.radius as f32;
    let mesh = meshes.add(Sphere::new(radius).mesh().uv(48, 24));
    for id in BallId::all() {
        let Some(state) = sim.world.state(id) else {
            continue;
        };
        commands.spawn((
            Ball(id),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(images.add(ball_texture::ball_image(id))),
                perceptual_roughness: 0.12,
                reflectance: 0.6,
                ..default()
            })),
            Transform::from_translation(state.position.as_vec3()),
            Visibility::Inherited,
        ));
    }
}
