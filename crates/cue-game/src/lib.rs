//! # cue-game
//!
//! The Bevy application. It owns presentation and orchestration: it turns
//! player input into shot commands, hands them to `cue-physics`, feeds the
//! resulting reports to the rules in `cue-core`, and draws the outcome. It
//! never makes a physical or regulatory decision of its own.

use bevy::prelude::*;
use cue_core::{BallGroup, BallId, TableSpec, eight_ball_rack};

/// Everything CUE adds to a Bevy [`App`].
#[derive(Debug, Default)]
pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.06, 0.06, 0.07)))
            .add_systems(Startup, spawn_placeholder_scene);
    }
}

/// Logical id attached to every rendered ball.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ball(pub BallId);

/// Placeholder scene: a cloth slab, sixteen balls in a legal rack, a light and
/// a camera. Real geometry, materials and cue arrive in later phases.
fn spawn_placeholder_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let table = TableSpec::nine_foot();
    let half = table.half_extents();
    let radius = table.ball.radius as f32;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            table.playfield_length as f32,
            0.02,
            table.playfield_width as f32,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.32, 0.22),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    let ball_mesh = meshes.add(Sphere::new(radius).mesh().uv(48, 24));
    let layout = eight_ball_rack(&table, 0x00C0_FFEE).expect("the standard table always racks");
    for (id, p) in layout.iter() {
        commands.spawn((
            Ball(id),
            Mesh3d(ball_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: placeholder_colour(id),
                perceptual_roughness: 0.15,
                ..default()
            })),
            Transform::from_xyz(p.x as f32, radius, p.y as f32),
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.6, 3.0, 0.8).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let eye_height = (half.x as f32) * 2.1;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, eye_height, eye_height * 0.45).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Flat placeholder colours. Stripes are drawn lighter until real ball
/// materials (with the white band and numbers) land in Phase 5.
fn placeholder_colour(id: BallId) -> Color {
    const HUES: [(f32, f32, f32); 7] = [
        (0.95, 0.78, 0.10), // 1 / 9   yellow
        (0.10, 0.25, 0.75), // 2 / 10  blue
        (0.80, 0.12, 0.12), // 3 / 11  red
        (0.40, 0.15, 0.55), // 4 / 12  purple
        (0.95, 0.45, 0.10), // 5 / 13  orange
        (0.10, 0.50, 0.25), // 6 / 14  green
        (0.45, 0.12, 0.10), // 7 / 15  maroon
    ];
    match id.group() {
        BallGroup::Cue => Color::srgb(0.96, 0.95, 0.90),
        BallGroup::Eight => Color::srgb(0.03, 0.03, 0.03),
        BallGroup::Solids => {
            let (r, g, b) = HUES[usize::from(id.number() - 1)];
            Color::srgb(r, g, b)
        }
        BallGroup::Stripes => {
            let (r, g, b) = HUES[usize::from(id.number() - 9)];
            Color::srgb(r, g, b).mix(&Color::WHITE, 0.35)
        }
    }
}
