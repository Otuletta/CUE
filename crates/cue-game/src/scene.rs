//! Table and ball meshes.
//!
//! Cushions and rails are built from the outlines in the physics
//! [`TableGeometry`](cue_physics::TableGeometry), so what you see lines up
//! with what the balls bounce off. The meshes are still presentation only:
//! the solver never reads them.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use cue_core::{BallId, glam::DVec2};
use cue_physics::Cushion;

use crate::{
    ball_texture,
    settings::{TableMaterials, Theme},
    sim::{Ball, Simulation},
};

/// Width of the wooden rail outside the cushion, metres.
pub(crate) const RAIL_WIDTH: f32 = 0.12;
/// Height of the cushion top above the cloth, metres.
const CUSHION_HEIGHT: f32 = 0.044;
/// Height of the wooden rail top above the cloth, metres.
const RAIL_HEIGHT: f32 = 0.05;
/// Height of the pocket surround (the leather around the hole), metres.
const POCKET_HEIGHT: f32 = 0.02;

fn flat(p: DVec2) -> Vec2 {
    p.as_vec2()
}

pub(crate) fn spawn_table(
    mut commands: Commands,
    sim: Res<Simulation>,
    theme: Res<Theme>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let geometry = sim.world.geometry();
    let outer = geometry.outer_half_extents().as_vec2();
    let rim = outer + Vec2::splat(RAIL_WIDTH);

    // Colours come from the theme and are updated in place when it changes.
    let palette = theme.palette();
    let material = |materials: &mut Assets<StandardMaterial>, colour: Color, roughness: f32| {
        materials.add(StandardMaterial {
            base_color: colour,
            perceptual_roughness: roughness,
            reflectance: 0.25,
            ..default()
        })
    };
    let cloth = material(&mut materials, palette.cloth, 0.92);
    let cushion = material(&mut materials, palette.cushion, 0.9);
    let wood = material(&mut materials, palette.wood, 0.45);
    let leather = material(&mut materials, palette.leather, 0.7);
    commands.insert_resource(TableMaterials {
        cloth: cloth.clone(),
        cushion: cushion.clone(),
        wood: wood.clone(),
        leather: leather.clone(),
    });
    let hole = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });

    let mut spawn = |mesh: Mesh, material: &Handle<StandardMaterial>, at: Vec3| {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at),
        ));
    };

    // Cloth: top surface at y = 0, reaching under the cushions.
    spawn(
        Cuboid::new(2.0 * outer.x, 0.03, 2.0 * outer.y)
            .mesh()
            .build(),
        &cloth,
        Vec3::new(0.0, -0.015, 0.0),
    );

    for (outline, segment) in geometry.cushion_outlines().iter().zip(
        geometry
            .segments()
            .iter()
            .filter(|s| matches!(s.cushion, Cushion::Rail(_))),
    ) {
        // Cushion rubber, exactly over the collision outline.
        let quad = outline.map(flat);
        spawn(prism(&quad, 0.0, CUSHION_HEIGHT), &cushion, Vec3::ZERO);
        // Wood behind it, from the back of the cushion to the rim.
        let out = -flat(segment.normal) * RAIL_WIDTH;
        let back = [quad[3], quad[2], quad[2] + out, quad[3] + out];
        spawn(prism(&back, 0.0, RAIL_HEIGHT), &wood, Vec3::ZERO);
    }

    // Pocket surrounds fill the gaps the rail sections leave, and a black
    // disc marks the drop region.
    for pocket in geometry.pockets() {
        let ends: Vec<Vec2> = geometry
            .segments()
            .iter()
            .filter(|s| s.cushion == Cushion::Jaw(pocket.id))
            .map(|s| flat(s.b))
            .collect();
        let (sx, sz) = pocket.id.signs();
        let (sx, sz) = (f32::from(sx), f32::from(sz));
        let surround: Vec<Vec2> = if pocket.id.is_corner() {
            // Long-rail facing end, out to the rim, round the corner, back in
            // to the short-rail facing end.
            vec![
                ends[0],
                Vec2::new(ends[0].x, sz * rim.y),
                Vec2::new(sx * rim.x, sz * rim.y),
                Vec2::new(sx * rim.x, ends[1].y),
                ends[1],
            ]
        } else {
            vec![
                ends[0],
                ends[1],
                Vec2::new(ends[1].x, sz * rim.y),
                Vec2::new(ends[0].x, sz * rim.y),
            ]
        };
        spawn(prism(&surround, 0.0, POCKET_HEIGHT), &leather, Vec3::ZERO);
        let centre = flat(pocket.centre);
        spawn(
            Cylinder::new(pocket.radius as f32, 0.001)
                .mesh()
                .resolution(48)
                .build(),
            &hole,
            Vec3::new(centre.x, POCKET_HEIGHT + 0.0005, centre.y),
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

/// A vertical prism over a convex outline on the table plane, from `bottom`
/// to `top`. Flat-shaded; winding is fixed up from the geometry, so the
/// outline may be given in either orientation.
fn prism(outline: &[Vec2], bottom: f32, top: f32) -> Mesh {
    let centre = outline.iter().copied().sum::<Vec2>() / outline.len() as f32;
    let centre3 = Vec3::new(centre.x, (top + bottom) / 2.0, centre.y);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    // Each triangle is emitted so that its normal points away from the
    // prism's centre.
    let mut triangle = |a: Vec3, b: Vec3, c: Vec3| {
        let mut n = (b - a).cross(c - a).normalize_or_zero();
        let (b, c) = if n.dot((a + b + c) / 3.0 - centre3) < 0.0 {
            n = -n;
            (c, b)
        } else {
            (b, c)
        };
        for p in [a, b, c] {
            positions.push(p.to_array());
            normals.push(n.to_array());
        }
    };
    let up = |p: Vec2| Vec3::new(p.x, top, p.y);
    let down = |p: Vec2| Vec3::new(p.x, bottom, p.y);
    for i in 1..outline.len() - 1 {
        triangle(up(outline[0]), up(outline[i]), up(outline[i + 1]));
    }
    for i in 0..outline.len() {
        let (p, q) = (outline[i], outline[(i + 1) % outline.len()]);
        triangle(down(p), down(q), up(q));
        triangle(down(p), up(q), up(p));
    }
    let count = positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 0.0]; count as usize])
    .with_inserted_indices(Indices::U32((0..count).collect()))
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
