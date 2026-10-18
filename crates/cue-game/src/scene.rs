//! Table and ball meshes.
//!
//! Cushions and rails are built from the outlines in the physics
//! [`TableGeometry`](cue_physics::TableGeometry), so what you see lines up
//! with what the balls bounce off. The meshes are still presentation only:
//! the solver never reads them.

use bevy::{
    asset::RenderAssetUsages,
    light::CascadeShadowConfigBuilder,
    math::Affine2,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use cue_core::{BallId, glam::DVec2};
use cue_physics::Cushion;

use crate::{
    ball_texture,
    settings::{TableMaterials, Theme},
    sim::{Ball, GameEvent, Simulation},
    textures,
};

/// Width of the wooden rail outside the cushion, metres.
pub(crate) const RAIL_WIDTH: f32 = 0.12;
/// Height of the cushion top above the cloth, metres.
const CUSHION_HEIGHT: f32 = 0.044;
/// Height of the wooden rail top above the cloth, metres.
const RAIL_HEIGHT: f32 = 0.05;
/// Height of the pocket surround (the leather around the hole), metres.
const POCKET_HEIGHT: f32 = 0.02;
/// Width and height of the leather lip round each pocket, metres.
const POCKET_LIP: f32 = 0.006;
/// One repeat of the cloth grain, metres.
const GRAIN_TILE: f32 = 0.25;
/// One repeat of the rails' wood, along and across the grain, metres.
const WOOD_TILE: Vec2 = Vec2::new(0.9, 0.225);
/// Half-diagonals of a diamond sight, across and along its rail, metres.
const DIAMOND: (f32, f32) = (0.009, 0.005);
/// Radius of the foot-spot mark, metres.
const SPOT_RADIUS: f32 = 0.006;
/// Height of the lamp's shades above the cloth, metres.
const LAMP_HEIGHT: f32 = 1.0;
/// Along the table, where the lamp's three shades hang, metres from centre.
const LAMP_SHADES: [f32; 3] = [-0.75, 0.0, 0.75];
/// Luminous power of each shade, lumens. Together they light the middle of
/// the cloth to about the key light's level and let the corners and the room
/// fall into shadow.
const LAMP_LUMENS: f32 = 70_000.0;
/// The table's body below the rails, and the floor, metres below the cloth.
const BODY_DEPTH: f32 = 0.2;
const FLOOR_DEPTH: f32 = 0.76;
/// Farthest distance from the camera that receives shadows, metres. Bevy's
/// default spreads WebGL2's single shadow cascade over 150 m, too coarse for
/// a ball's shadow; the camera is never farther than a few metres away.
const SHADOW_DISTANCE: f32 = 7.0;

fn flat(p: DVec2) -> Vec2 {
    p.as_vec2()
}

pub(crate) fn spawn_table(
    mut commands: Commands,
    sim: Res<Simulation>,
    theme: Res<Theme>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
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
    // The cloth's grain repeats every 25 cm, square on both axes.
    let cloth = materials.add(StandardMaterial {
        base_color: palette.cloth,
        base_color_texture: Some(images.add(textures::cloth_grain())),
        uv_transform: Affine2::from_scale(Vec2::new(
            2.0 * outer.x / GRAIN_TILE,
            2.0 * outer.y / GRAIN_TILE,
        )),
        perceptual_roughness: 0.92,
        reflectance: 0.25,
        ..default()
    });
    let cushion = material(&mut materials, palette.cushion, 0.9);
    // Varnished: the lamp's shades glide along the rails.
    let wood = materials.add(StandardMaterial {
        base_color: palette.wood,
        base_color_texture: Some(images.add(textures::wood_grain())),
        perceptual_roughness: 0.42,
        reflectance: 0.3,
        ..default()
    });
    let leather = material(&mut materials, palette.leather, 0.55);
    let floor = material(&mut materials, palette.floor, 0.85);
    commands.insert_resource(TableMaterials {
        cloth: cloth.clone(),
        cushion: cushion.clone(),
        wood: wood.clone(),
        leather: leather.clone(),
        floor: floor.clone(),
    });
    let hole = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(textures::pocket_depth())),
        unlit: true,
        ..default()
    });
    let pearl = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.91, 0.86),
        perceptual_roughness: 0.25,
        reflectance: 0.5,
        ..default()
    });
    let chalk = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.86, 0.82),
        perceptual_roughness: 0.95,
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
    // The body under the rails, so the table stands on something, and the
    // floor of a dim room around it.
    spawn(
        Cuboid::new(2.0 * rim.x, BODY_DEPTH, 2.0 * rim.y)
            .mesh()
            .build(),
        &wood,
        Vec3::new(0.0, -0.03 - BODY_DEPTH / 2.0, 0.0),
    );
    spawn(
        Plane3d::default().mesh().size(14.0, 14.0).build(),
        &floor,
        Vec3::new(0.0, -FLOOR_DEPTH, 0.0),
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
        // Wood behind it, from the back of the cushion to the rim, with the
        // grain along the rail.
        let normal = flat(segment.normal);
        let out = -normal * RAIL_WIDTH;
        let back = [quad[3], quad[2], quad[2] + out, quad[3] + out];
        spawn(
            prism_mapped(&back, 0.0, RAIL_HEIGHT, normal.perp(), WOOD_TILE),
            &wood,
            Vec3::ZERO,
        );
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
        // The rail's wood runs round the pocket, a little lower, as on a
        // real table; only the lip is leather.
        spawn(
            prism_mapped(&surround, 0.0, POCKET_HEIGHT, Vec2::X, WOOD_TILE),
            &wood,
            Vec3::ZERO,
        );
        // The drop, dark towards its middle, inside a leather lip that
        // catches the lamp.
        let centre = flat(pocket.centre);
        let radius = pocket.radius as f32;
        spawn(
            Cylinder::new(radius, 0.001).mesh().resolution(48).build(),
            &hole,
            Vec3::new(centre.x, POCKET_HEIGHT + 0.0005, centre.y),
        );
        spawn(
            Extrusion::new(Annulus::new(radius, radius + POCKET_LIP), POCKET_LIP)
                .mesh()
                .resolution(48)
                .build()
                .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            &leather,
            Vec3::new(centre.x, POCKET_HEIGHT + POCKET_LIP / 2.0, centre.y),
        );
    }

    // Diamond sights on the rail tops, at eighths of the length and quarters
    // of the width measured nose to nose (WPA 2.1); none at the side pockets.
    let table = sim.table();
    let half = table.half_extents().as_vec2();
    let band = |inner: f32| inner + RAIL_WIDTH * 0.5;
    let mut sights = Vec::new();
    for k in [1, 2, 3, 5, 6, 7] {
        let x = -half.x + k as f32 * half.x / 4.0;
        for z in [-band(outer.y), band(outer.y)] {
            sights.push((Vec2::new(x, z), Vec2::Y));
        }
    }
    for k in [1, 2, 3] {
        let z = -half.y + k as f32 * half.y / 2.0;
        for x in [-band(outer.x), band(outer.x)] {
            sights.push((Vec2::new(x, z), Vec2::X));
        }
    }
    for (at, across) in sights {
        let along = across.perp();
        let outline = [
            at + across * DIAMOND.0,
            at + along * DIAMOND.1,
            at - across * DIAMOND.0,
            at - along * DIAMOND.1,
        ];
        spawn(
            prism(&outline, RAIL_HEIGHT, RAIL_HEIGHT + 0.0006),
            &pearl,
            Vec3::ZERO,
        );
    }

    // The foot spot is marked: the rack's apex and spotted balls go there.
    let foot = table.foot_spot().as_vec2();
    spawn(
        Cylinder::new(SPOT_RADIUS, 0.0004)
            .mesh()
            .resolution(24)
            .build(),
        &chalk,
        Vec3::new(foot.x, 0.0002, foot.y),
    );

    // A pool-table lamp: three shades over the playing surface. They light
    // the middle of the cloth more than its corners, give each ball the
    // shades' highlights, and leave the room dim. They cast no shadows, which
    // would cost a shadow map each on WebGL2.
    for x in LAMP_SHADES {
        commands.spawn((
            SpotLight {
                color: Color::srgb(1.0, 0.95, 0.87),
                intensity: LAMP_LUMENS,
                range: 4.0,
                radius: 0.1,
                shadow_maps_enabled: false,
                inner_angle: 0.55,
                outer_angle: 1.05,
                ..default()
            },
            Transform::from_xyz(x, LAMP_HEIGHT, 0.0).looking_at(Vec3::new(x, 0.0, 0.0), Vec3::X),
        ));
    }

    // A soft key light from above, for the balls' contact shadows, with its
    // shadow cascade fitted to the table rather than to a landscape.
    commands.spawn((
        DirectionalLight {
            illuminance: 3_500.0,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            minimum_distance: 0.1,
            maximum_distance: SHADOW_DISTANCE,
            first_cascade_far_bound: SHADOW_DISTANCE,
            overlap_proportion: 0.2,
        }
        .build(),
        Transform::from_xyz(0.4, 4.0, 1.2).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// A vertical prism over a convex outline on the table plane, from `bottom`
/// to `top`. Flat-shaded; winding is fixed up from the geometry, so the
/// outline may be given in either orientation.
fn prism(outline: &[Vec2], bottom: f32, top: f32) -> Mesh {
    prism_mapped(outline, bottom, top, Vec2::X, Vec2::ONE)
}

/// [`prism`] with texture coordinates: `u` runs along `along` and `v`
/// across it on the top face, and round the outline and up on the sides,
/// one repeat per `tile` metres.
fn prism_mapped(outline: &[Vec2], bottom: f32, top: f32, along: Vec2, tile: Vec2) -> Mesh {
    let centre = outline.iter().copied().sum::<Vec2>() / outline.len() as f32;
    let centre3 = Vec3::new(centre.x, (top + bottom) / 2.0, centre.y);
    let across = along.perp();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    // Each triangle is emitted so that its normal points away from the
    // prism's centre.
    let mut triangle = |corners: [(Vec3, Vec2); 3]| {
        let [(a, ua), (mut b, mut ub), (mut c, mut uc)] = corners;
        let mut n = (b - a).cross(c - a).normalize_or_zero();
        if n.dot((a + b + c) / 3.0 - centre3) < 0.0 {
            n = -n;
            (b, c) = (c, b);
            (ub, uc) = (uc, ub);
        }
        for (p, uv) in [(a, ua), (b, ub), (c, uc)] {
            positions.push(p.to_array());
            normals.push(n.to_array());
            uvs.push(uv.to_array());
        }
    };
    let up = |p: Vec2| Vec3::new(p.x, top, p.y);
    let down = |p: Vec2| Vec3::new(p.x, bottom, p.y);
    let plan = |p: Vec2| Vec2::new(p.dot(along), p.dot(across)) / tile;
    for i in 1..outline.len() - 1 {
        let [p, q, r] = [outline[0], outline[i], outline[i + 1]];
        triangle([(up(p), plan(p)), (up(q), plan(q)), (up(r), plan(r))]);
    }
    // The sides unroll round the outline, so the grain wraps the edge.
    let mut run = 0.0;
    for i in 0..outline.len() {
        let (p, q) = (outline[i], outline[(i + 1) % outline.len()]);
        let next = run + p.distance(q);
        let side = |s: f32, y: f32| Vec2::new(s / tile.x, y / tile.y);
        let (pb, qb, qt, pt) = (
            side(run, bottom),
            side(next, bottom),
            side(next, top),
            side(run, top),
        );
        triangle([(down(p), pb), (down(q), qb), (up(q), qt)]);
        triangle([(down(p), pb), (up(q), qt), (up(p), pt)]);
        run = next;
    }
    let count = positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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
            Transform::from_translation(state.position.as_vec3())
                .with_rotation(ball_texture::resting_orientation(rack_yaw(id))),
            Visibility::Inherited,
        ));
    }
}

/// A small, fixed twist per ball, so a fresh rack looks set by hand rather
/// than printed: within ±0.35 rad, spread by the golden angle.
fn rack_yaw(id: BallId) -> f32 {
    ((f32::from(id.number()) * 0.618_034).fract() - 0.5) * 0.7
}

/// Shows the numbers from above whenever balls are racked or spotted. Once
/// they roll, their orientation follows the simulation's spin.
pub(crate) fn orient_racked_balls(
    mut events: MessageReader<GameEvent>,
    mut balls: Query<(&Ball, &mut Transform)>,
) {
    for event in events.read() {
        let only = match event {
            GameEvent::Racked { .. } | GameEvent::NewGame => None,
            GameEvent::Spotted { ball } => Some(*ball),
            _ => continue,
        };
        for (ball, mut transform) in &mut balls {
            if only.is_none_or(|b| b == ball.0) {
                transform.rotation = ball_texture::resting_orientation(rack_yaw(ball.0));
            }
        }
    }
}
