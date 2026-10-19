//! Table and ball meshes.
//!
//! Cushions and rails are built from the outlines in the physics
//! [`TableGeometry`](cue_physics::TableGeometry), so what you see lines up
//! with what the balls bounce off. The meshes are still presentation only:
//! the solver never reads them.

use std::f32::consts::TAU;

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
/// Height of the cushion's top at its nose and at its back, metres. The top
/// slopes up to the rail, so the lamp lights it as a band that reads as a
/// raised cushion from straight above.
const CUSHION_SLOPE: (f32, f32) = (0.03, 0.048);
/// Height of the wooden rail top above the cloth, metres.
const RAIL_HEIGHT: f32 = 0.05;
/// Width of the leather cap round each pocket, on the rail, and how far it
/// stands proud of the wood, metres.
const POCKET_CAP: (f32, f32) = (0.03, 0.002);
/// Width of the brass bead at the pocket's edge, on the cap, metres.
const POCKET_BEAD: f32 = 0.0035;
/// Distance from the cushion and width of the brass inlay on the rails, and
/// how far it stands proud of the wood, metres.
const INLAY: (f32, f32) = (0.004, 0.0004);
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
const LAMP_LUMENS: f32 = 100_000.0;
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
    // Polished brass for the pocket lips and the sight line along the rails.
    let brass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.68, 0.38),
        metallic: 1.0,
        perceptual_roughness: 0.28,
        ..default()
    });
    // The room's floor, seen in the margins round the table. Unlit: from
    // straight above it is a backdrop, and lit it would show the edge of the
    // shadow map's reach as a hard rectangle.
    let floor = materials.add(StandardMaterial {
        base_color: palette.floor,
        unlit: true,
        ..default()
    });
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
        spawn(cushion_mesh(&quad), &cushion, Vec3::ZERO);
        // Wood behind it, from the back of the cushion to the rim, with the
        // grain along the rail.
        let normal = flat(segment.normal);
        // The rim's bevel (`frame_bevel`) takes the last stretch of the width.
        let out = -normal * (RAIL_WIDTH - RIM_BEVEL.0);
        let back = [quad[3], quad[2], quad[2] + out, quad[3] + out];
        spawn(rail_mesh(&back, normal, RAIL_HEIGHT), &wood, Vec3::ZERO);
        // A thin brass line inlaid where the wood meets the cushion.
        let inlay = -normal * INLAY.0;
        let line = [
            quad[3] + inlay,
            quad[2] + inlay,
            quad[2] + inlay * 2.0,
            quad[3] + inlay * 2.0,
        ];
        spawn(
            prism(&line, RAIL_HEIGHT, RAIL_HEIGHT + INLAY.1),
            &brass,
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
        let inner = rim - Vec2::splat(RIM_BEVEL.0);
        let surround: Vec<Vec2> = if pocket.id.is_corner() {
            // Long-rail facing end, out to the rim, round the corner, back in
            // to the short-rail facing end.
            vec![
                ends[0],
                Vec2::new(ends[0].x, sz * inner.y),
                Vec2::new(sx * inner.x, sz * inner.y),
                Vec2::new(sx * inner.x, ends[1].y),
                ends[1],
            ]
        } else {
            vec![
                ends[0],
                ends[1],
                Vec2::new(ends[1].x, sz * inner.y),
                Vec2::new(ends[0].x, sz * inner.y),
            ]
        };
        // The rail's wood runs on round the pocket at the rail's height, cut
        // round the hole.
        let centre = flat(pocket.centre);
        let radius = pocket.radius as f32;
        spawn(
            surround_mesh(&surround, centre, radius, f32::INFINITY, RAIL_HEIGHT),
            &wood,
            Vec3::ZERO,
        );
        // On the rail round the hole, a leather cap with a brass bead at its
        // edge, as on a fine table. Both stay on the wood: the cushion's jaws
        // run up to the hole between them.
        spawn(
            cap_mesh(
                centre,
                radius,
                radius + POCKET_CAP.0,
                outer,
                inner,
                RAIL_HEIGHT + POCKET_CAP.1,
            ),
            &leather,
            Vec3::ZERO,
        );
        spawn(
            cap_mesh(
                centre,
                radius,
                radius + POCKET_BEAD,
                outer,
                inner,
                RAIL_HEIGHT + POCKET_CAP.1 + 0.0004,
            ),
            &brass,
            Vec3::ZERO,
        );
        // The drop, dark towards its middle, down at the cloth.
        spawn(
            Cylinder::new(radius, 0.001).mesh().resolution(64).build(),
            &hole,
            Vec3::new(centre.x, 0.0005, centre.y),
        );
    }

    spawn(frame_bevel(rim, RAIL_HEIGHT), &wood, Vec3::ZERO);

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
                radius: 0.15,
                shadow_maps_enabled: false,
                inner_angle: 0.45,
                outer_angle: 0.95,
                ..default()
            },
            Transform::from_xyz(x, LAMP_HEIGHT, 0.0).looking_at(Vec3::new(x, 0.0, 0.0), Vec3::X),
        ));
    }

    // A soft key light from above, for the balls' contact shadows, with its
    // shadow cascade fitted to the table rather than to a landscape.
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.0,
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

    // A low light from the room, off the screen's top left: every ball gets
    // a bright highlight on its shoulder, clear of the number, and a lit side
    // that shows its roundness from straight above. A distant point light
    // rather than a directional one, so it casts no shadows at all: a light
    // this low would draw long ones across the cloth.
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.97, 0.92),
            intensity: ROOM_LUMENS,
            range: 12.0,
            radius: 0.3,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(ROOM_LIGHT),
    ));
}

/// Where the room light hangs: off the far rail and the head, a little above
/// the table, so it skims the balls at about 8°. Its power, lumens.
const ROOM_LIGHT: Vec3 = Vec3::new(-3.0, 0.6, -3.0);
const ROOM_LUMENS: f32 = 1_400_000.0;

/// How far the rail's edges are rounded off, as `(inner, outer)` chamfers,
/// metres. Seen from above the lamp catches the inner one and the outer one
/// falls into shade, so the rail reads as turned wood, not a flat band.
const RAIL_CHAMFER: (f32, f32) = (0.012, 0.0);
/// Width of the bevel that runs round the whole frame's outer edge, and how
/// far it drops, metres. One continuous piece, so rails and pocket
/// surrounds read as a single frame of wood.
const RIM_BEVEL: (f32, f32) = (0.024, 0.018);

/// A ring round a pocket's hole, from `radius` to `reach` round `centre`, on
/// the rails only: outside the back of the cushions (`outer`) and inside the
/// frame's outer bevel (`inner`), at `height`. Swept as rays from the centre
/// in one piece, so it has no seams where two rails meet.
fn cap_mesh(centre: Vec2, radius: f32, reach: f32, outer: Vec2, inner: Vec2, height: f32) -> Mesh {
    const STEPS: usize = 360;
    let rect = |h: Vec2| vec![-h, Vec2::new(h.x, -h.y), h, Vec2::new(-h.x, h.y)];
    let (cushions, frame) = (rect(outer), rect(inner));
    let mut angles: Vec<f32> = (0..STEPS).map(|k| k as f32 / STEPS as f32 * TAU).collect();
    angles.extend(
        cushions
            .iter()
            .map(|v| (*v - centre).to_angle().rem_euclid(TAU)),
    );
    angles.sort_by(f32::total_cmp);
    angles.dedup_by(|a, b| (*a - *b).abs() < 1e-5);
    let span = |a: f32| {
        let dir = Vec2::from_angle(a);
        let (_, out) = ray_in_convex(&frame, centre, dir)?;
        let end = out.min(reach);
        let (from, to) = match ray_in_convex(&cushions, centre, dir) {
            // The ray reaches the cushions after leaving the hole: the cap
            // runs up to them.
            Some((enter, _)) if enter > radius => (radius, end.min(enter)),
            // It starts over the cushions: the cap begins where it leaves them.
            Some((_, exit)) => (radius.max(exit), end),
            None => (radius, end),
        };
        (to > from + 1e-5).then_some((centre + dir * from, centre + dir * to))
    };
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for i in 0..angles.len() {
        let (Some((a0, a1)), Some((b0, b1))) =
            (span(angles[i]), span(angles[(i + 1) % angles.len()]))
        else {
            continue;
        };
        // Counter-clockwise seen from above, as in `surround_mesh`.
        for p in [a0, b1, a1, a0, b0, b1] {
            positions.push([p.x, height, p.y]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 0.0]; count])
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}

/// A cushion section: `quad` is its two nose points, then the two ends
/// behind them. The top slopes from the nose up to the back
/// ([`CUSHION_SLOPE`]); the sides stand straight.
fn cushion_mesh(quad: &[Vec2; 4]) -> Mesh {
    let (a, b) = (quad[0], quad[1]);
    let across = (b - a).perp().normalize_or_zero();
    let depth = |p: Vec2| (p - a).dot(across).abs();
    let full = depth(quad[2]).max(depth(quad[3])).max(1e-6);
    let (nose, back) = CUSHION_SLOPE;
    prism_shaped(
        quad,
        0.0,
        &|p| nose + (back - nose) * (depth(p) / full).clamp(0.0, 1.0),
        Vec2::X,
        Vec2::ONE,
    )
}

/// The part of a ray from `origin` along `dir` inside a convex polygon, as
/// `(enter, exit)` distances (Cyrus–Beck clipping).
fn ray_in_convex(polygon: &[Vec2], origin: Vec2, dir: Vec2) -> Option<(f32, f32)> {
    let n = polygon.len();
    let area: f32 = (0..n)
        .map(|i| polygon[i].perp_dot(polygon[(i + 1) % n]))
        .sum();
    let (mut enter, mut exit) = (f32::NEG_INFINITY, f32::INFINITY);
    for i in 0..n {
        let (a, b) = (polygon[i], polygon[(i + 1) % n]);
        let e = b - a;
        let outward = if area > 0.0 {
            Vec2::new(e.y, -e.x)
        } else {
            Vec2::new(-e.y, e.x)
        };
        let (denom, num) = (outward.dot(dir), outward.dot(a - origin));
        if denom.abs() < 1e-9 {
            if num < 0.0 {
                return None;
            }
        } else if denom > 0.0 {
            exit = exit.min(num / denom);
        } else {
            enter = enter.max(num / denom);
        }
    }
    (enter < exit).then_some((enter.max(0.0), exit))
}

/// The flat top of a convex `outline` at `height`, with a hole of `radius`
/// round `centre` cut out of it and nothing farther than `reach` from the
/// centre: swept as rays from the centre, through every corner of the
/// outline and every few degrees between them.
fn surround_mesh(outline: &[Vec2], centre: Vec2, radius: f32, reach: f32, height: f32) -> Mesh {
    const STEPS: usize = 160;
    let mut angles: Vec<f32> = (0..STEPS).map(|k| k as f32 / STEPS as f32 * TAU).collect();
    angles.extend(
        outline
            .iter()
            .map(|v| (*v - centre).to_angle().rem_euclid(TAU)),
    );
    angles.sort_by(f32::total_cmp);
    angles.dedup_by(|a, b| (*a - *b).abs() < 1e-5);
    let span = |a: f32| {
        let dir = Vec2::from_angle(a);
        ray_in_convex(outline, centre, dir).and_then(|(enter, exit)| {
            let (from, to) = (enter.max(radius), exit.min(reach));
            (to > from + 1e-5).then_some((centre + dir * from, centre + dir * to))
        })
    };
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut put = |p: Vec2| {
        positions.push([p.x, height, p.y]);
        uvs.push((p / WOOD_TILE).to_array());
    };
    for i in 0..angles.len() {
        let (Some((a0, a1)), Some((b0, b1))) =
            (span(angles[i]), span(angles[(i + 1) % angles.len()]))
        else {
            continue;
        };
        // Counter-clockwise seen from above (+Y up, Z towards the viewer's
        // bottom): the winding that faces up.
        for p in [a0, b1, a1, a0, b0, b1] {
            put(p);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}

/// The bevel round the frame's outer edge: four slopes, mitred at the
/// corners, from the top at the rim's inset down to the rim.
fn frame_bevel(rim: Vec2, height: f32) -> Mesh {
    let (width, fall) = RIM_BEVEL;
    let inner = rim - Vec2::splat(width);
    let corner = |v: Vec2, sx: f32, sz: f32| Vec2::new(sx * v.x, sz * v.y);
    let signs = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    for i in 0..4 {
        let (s0, s1) = (signs[i], signs[(i + 1) % 4]);
        let (q0, q1) = (corner(inner, s0.0, s0.1), corner(inner, s1.0, s1.1));
        let (p0, p1) = (corner(rim, s0.0, s0.1), corner(rim, s1.0, s1.1));
        let top = |p: Vec2| Vec3::new(p.x, height, p.y);
        let low = |p: Vec2| Vec3::new(p.x, height - fall, p.y);
        let quad = [top(q0), low(p0), low(p1), top(q1)];
        let mut n = (quad[1] - quad[0])
            .cross(quad[2] - quad[0])
            .normalize_or_zero();
        let mut tris = [0, 1, 2, 0, 2, 3];
        if n.y < 0.0 {
            n = -n;
            tris = [0, 2, 1, 0, 3, 2];
        }
        for k in tris {
            positions.push(quad[k].to_array());
            normals.push(n.to_array());
            uvs.push((Vec2::new(quad[k].x, quad[k].z) / WOOD_TILE).to_array());
        }
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

/// A rail section: `quad` is `[inner start, inner end, outer end, outer
/// start]` on the table plane, `normal` points from the rail to the cloth.
/// The top is flat between two chamfers that run along the rail; the ends,
/// cut at the pocket angles, stay vertical. Wood grain runs along the rail.
fn rail_mesh(quad: &[Vec2; 4], normal: Vec2, height: f32) -> Mesh {
    let along = normal.perp();
    let (ci, co) = RAIL_CHAMFER;
    let [ia, ib, ob, oa] = *quad;
    // Points on the top, stepped in from each long edge.
    let inset = |p: Vec2, by: f32| p - normal * by;
    let top = [inset(ia, ci), inset(ib, ci), inset(ob, -co), inset(oa, -co)];
    let at = |p: Vec2, y: f32| Vec3::new(p.x, y, p.y);
    let uv = |p: Vec2| Vec2::new(p.dot(along), p.dot(normal)) / WOOD_TILE;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    // Flat shading: each face carries its own normal, oriented outwards.
    let mut face = |corners: &[(Vec3, Vec2)], outward: Vec3| {
        let (a, b, c) = (corners[0].0, corners[1].0, corners[2].0);
        let n = (b - a).cross(c - a).normalize_or_zero();
        if n == Vec3::ZERO {
            return; // a chamfer of zero width
        }
        let flip = n.dot(outward) < 0.0;
        let n = if flip { -n } else { n };
        for i in 1..corners.len() - 1 {
            let mut tri = [corners[0], corners[i], corners[i + 1]];
            if flip {
                tri.swap(1, 2);
            }
            for (p, t) in tri {
                positions.push(p.to_array());
                normals.push(n.to_array());
                uvs.push(t.to_array());
            }
        }
    };
    let n3 = Vec3::new(normal.x, 0.0, normal.y);
    let drop = |c: f32| height - c;
    // Top.
    face(&top.map(|p| (at(p, height), uv(p))), Vec3::Y);
    // Inner chamfer, facing the cloth and the lamp.
    face(
        &[
            (at(ia, drop(ci)), uv(ia)),
            (at(ib, drop(ci)), uv(ib)),
            (at(top[1], height), uv(top[1])),
            (at(top[0], height), uv(top[0])),
        ],
        Vec3::Y + n3,
    );
    // Outer chamfer, facing away from the table.
    face(
        &[
            (at(top[3], height), uv(top[3])),
            (at(top[2], height), uv(top[2])),
            (at(ob, drop(co)), uv(ob)),
            (at(oa, drop(co)), uv(oa)),
        ],
        Vec3::Y - n3,
    );
    // Sides, down to the cloth: the long ones under the chamfers, the ends as
    // cut, following the chamfered profile.
    let side = |p: Vec2, q: Vec2, y: f32| {
        [
            (at(p, 0.0), uv(p)),
            (at(q, 0.0), uv(q)),
            (at(q, y), uv(q)),
            (at(p, y), uv(p)),
        ]
    };
    face(&side(ia, ib, drop(ci)), n3);
    face(&side(oa, ob, drop(co)), -n3);
    for (inner, step, outer) in [(ia, top[0], oa), (ib, top[1], ob)] {
        let other = if inner == ia { ib } else { ia };
        let outward = Vec3::new(inner.x - other.x, 0.0, inner.y - other.y);
        let step_out = if inner == ia { top[3] } else { top[2] };
        face(
            &[
                (at(inner, 0.0), uv(inner)),
                (at(inner, drop(ci)), uv(inner)),
                (at(step, height), uv(step)),
                (at(step_out, height), uv(step_out)),
                (at(outer, drop(co)), uv(outer)),
                (at(outer, 0.0), uv(outer)),
            ],
            outward,
        );
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
    prism_shaped(outline, bottom, &|_| top, along, tile)
}

/// [`prism_mapped`] with a top whose height varies over the outline: `top`
/// gives it at each corner.
fn prism_shaped(
    outline: &[Vec2],
    bottom: f32,
    top: &dyn Fn(Vec2) -> f32,
    along: Vec2,
    tile: Vec2,
) -> Mesh {
    let centre = outline.iter().copied().sum::<Vec2>() / outline.len() as f32;
    let centre3 = Vec3::new(centre.x, (top(centre) + bottom) / 2.0, centre.y);
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
    let up = |p: Vec2| Vec3::new(p.x, top(p), p.y);
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
            side(next, top(q)),
            side(run, top(p)),
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
        let finish = ball_texture::finish_image(id).map(|image| images.add(image));
        commands.spawn((
            Ball(id),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(images.add(ball_texture::ball_image(id))),
                // Lacquered, mirroring the lamp's shades, with satin ink: the
                // finish map scales the roughness, so the numbered balls
                // carry the lacquer's value in the map and 1.0 here.
                metallic_roughness_texture: finish.clone(),
                perceptual_roughness: if finish.is_some() {
                    1.0
                } else {
                    ball_texture::LACQUER_ROUGHNESS
                },
                reflectance: 0.55,
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
