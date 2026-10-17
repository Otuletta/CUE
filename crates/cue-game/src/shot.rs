//! Aiming, charging and taking a shot, with mouse, touch or keyboard.
//!
//! Phase 1 gesture (to be validated with real users, ADR-010):
//!
//! - **Aim:** move the mouse, or touch the point to shoot towards.
//! - **Power:** press and drag *back*, away from the aim direction.
//! - **Shoot:** release. Releasing with almost no pull cancels instead, so a
//!   click or a tap never fires a shot by accident.
//! - **Cancel:** `Esc` or right click.
//!
//! The keyboard path is complete on its own: arrows aim and set power,
//! `Space` shoots, `W`/`A`/`S`/`D` move the tip for spin.

use bevy::{input::touch::Touches, prelude::*, window::PrimaryWindow};
use cue_core::{
    BallId,
    glam::{DVec2, DVec3},
};
use cue_physics::{MAX_TIP_OFFSET, Motion, Strike};

use crate::sim::{Paused, Simulation};

/// Drag distance on the table, in metres, that maps to full power.
const FULL_PULL: f64 = 0.45;
/// Releases below this power cancel rather than shoot.
const MIN_SHOT_POWER: f64 = 0.03;
/// Perceptual power curve: more resolution for soft shots.
const POWER_EXPONENT: f64 = 1.6;
/// Keyboard aim speed, radians per second (and with `Shift` held).
const KEY_AIM_SPEED: (f64, f64) = (1.0, 0.08);
/// Keyboard power change per second.
const KEY_POWER_SPEED: f64 = 0.6;
/// Keyboard tip movement per second, in ball radii.
const KEY_TIP_SPEED: f64 = 0.6;

const CUE_LENGTH: f32 = 1.45;
const CUE_TIP_RADIUS: f32 = 0.0065;
const CUE_BUTT_RADIUS: f32 = 0.015;
/// Butt elevation, radians: a level cue would clip the rail.
const CUE_ELEVATION: f32 = 0.07;
/// How far the stick pulls back at full power, metres.
const CUE_MAX_DRAWBACK: f32 = 0.22;

/// The shot being prepared.
#[derive(Resource, Debug, Clone)]
pub struct Aim {
    /// Aim direction on the table plane, unit length.
    pub direction: DVec2,
    /// Charged power in `[0, 1]`, before the perceptual curve.
    pub power: f64,
    /// Tip offset in ball radii (`x` side, `y` follow/draw).
    pub tip: DVec2,
    drag: Option<Drag>,
}

impl Default for Aim {
    fn default() -> Self {
        Self {
            direction: DVec2::X,
            power: 0.0,
            tip: DVec2::ZERO,
            drag: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Drag {
    anchor: DVec2,
    /// Touch id, or `None` for the mouse.
    touch: Option<u64>,
}

/// Marker for the cue stick mesh.
#[derive(Component, Debug)]
pub(crate) struct CueStick;

pub(crate) fn spawn_cue(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CueStick,
        Mesh3d(meshes.add(ConicalFrustum {
            radius_top: CUE_BUTT_RADIUS,
            radius_bottom: CUE_TIP_RADIUS,
            height: CUE_LENGTH,
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.78, 0.62, 0.42),
            perceptual_roughness: 0.35,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
    ));
}

/// Projects a screen position onto the plane through the ball centres.
fn table_point(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    screen: Vec2,
    height: f32,
) -> Option<DVec2> {
    let ray = camera.viewport_to_world(camera_transform, screen).ok()?;
    let p =
        ray.plane_intersection_point(Vec3::new(0.0, height, 0.0), InfinitePlane3d::new(Vec3::Y))?;
    Some(DVec2::new(f64::from(p.x), f64::from(p.z)))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn read_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    paused: Res<Paused>,
    mut sim: ResMut<Simulation>,
    mut aim: ResMut<Aim>,
) {
    if paused.0 {
        aim.drag = None;
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        sim.rerack();
        *aim = Aim {
            tip: aim.tip,
            ..Aim::default()
        };
        return;
    }
    let Some(cue) = sim.cue_position().filter(|_| sim.world.is_at_rest()) else {
        aim.drag = None;
        return;
    };
    let (Ok(window), Ok((camera, camera_transform))) = (window.single(), camera.single()) else {
        return;
    };
    let height = sim.table().ball.radius as f32;
    let to_table = |screen: Vec2| table_point(camera, camera_transform, screen, height);
    let aim_at = |aim: &mut Aim, p: DVec2| {
        if let Some(d) = (p - cue).try_normalize().filter(|_| p.distance(cue) > 1e-3) {
            aim.direction = d;
        }
    };

    // Cancel.
    if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        aim.drag = None;
        aim.power = 0.0;
    }

    // Press: start charging.
    if aim.drag.is_none() {
        if let Some(t) = touches.iter_just_pressed().next() {
            if let Some(p) = to_table(t.position()) {
                aim_at(&mut aim, p);
                aim.drag = Some(Drag {
                    anchor: p,
                    touch: Some(t.id()),
                });
            }
        } else if mouse.just_pressed(MouseButton::Left) {
            if let Some(p) = window.cursor_position().and_then(to_table) {
                aim.drag = Some(Drag {
                    anchor: p,
                    touch: None,
                });
            }
        } else if let Some(p) = window.cursor_position().and_then(to_table) {
            // Hover aims (mouse only; touch has no hover).
            if touches.iter().next().is_none() {
                aim_at(&mut aim, p);
            }
        }
    }

    // Drag: pull back for power. Release: shoot or cancel.
    if let Some(drag) = aim.drag {
        let (current, released) = match drag.touch {
            Some(id) => match touches.get_pressed(id) {
                Some(t) => (to_table(t.position()), false),
                None => (
                    touches
                        .iter_just_released()
                        .find(|t| t.id() == id)
                        .and_then(|t| to_table(t.position())),
                    true,
                ),
            },
            None => (
                window.cursor_position().and_then(to_table),
                mouse.just_released(MouseButton::Left) || !mouse.pressed(MouseButton::Left),
            ),
        };
        if let Some(p) = current {
            let pull = (drag.anchor - p).dot(aim.direction);
            aim.power = (pull / FULL_PULL).clamp(0.0, 1.0);
        }
        if released {
            aim.drag = None;
            if aim.power >= MIN_SHOT_POWER {
                fire(&mut sim, &mut aim);
            } else {
                aim.power = 0.0;
            }
            return;
        }
    }

    // Keyboard.
    let dt = time.delta_secs_f64();
    let aim_speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        KEY_AIM_SPEED.1
    } else {
        KEY_AIM_SPEED.0
    };
    let turn = f64::from(
        i8::from(keys.pressed(KeyCode::ArrowLeft)) - i8::from(keys.pressed(KeyCode::ArrowRight)),
    );
    if turn != 0.0 {
        aim.direction = DVec2::from_angle(turn * aim_speed * dt).rotate(aim.direction);
    }
    let charge = f64::from(
        i8::from(keys.pressed(KeyCode::ArrowUp)) - i8::from(keys.pressed(KeyCode::ArrowDown)),
    );
    if charge != 0.0 {
        aim.power = (aim.power + charge * KEY_POWER_SPEED * dt).clamp(0.0, 1.0);
    }
    let tip_move = DVec2::new(
        f64::from(i8::from(keys.pressed(KeyCode::KeyD)) - i8::from(keys.pressed(KeyCode::KeyA))),
        f64::from(i8::from(keys.pressed(KeyCode::KeyW)) - i8::from(keys.pressed(KeyCode::KeyS))),
    );
    if tip_move != DVec2::ZERO {
        aim.tip = (aim.tip + tip_move * KEY_TIP_SPEED * dt).clamp_length_max(MAX_TIP_OFFSET);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        aim.tip = DVec2::ZERO;
    }
    if keys.just_pressed(KeyCode::Space) && aim.power >= MIN_SHOT_POWER {
        fire(&mut sim, &mut aim);
    }
}

fn fire(sim: &mut Simulation, aim: &mut Aim) {
    let strike = Strike {
        direction: aim.direction,
        power: aim.power.powf(POWER_EXPONENT),
        tip_offset: aim.tip,
    };
    match sim.world.strike(strike) {
        Ok(()) => {
            // The shot starts at the world's time; drop any presentation lag.
            sim.clock = sim.world.time();
            aim.power = 0.0;
        }
        Err(e) => warn!("strike rejected: {e}"),
    }
}

/// Places the cue stick and draws the aiming guide.
pub(crate) fn draw_aim(
    sim: Res<Simulation>,
    aim: Res<Aim>,
    mut gizmos: Gizmos,
    mut stick: Query<(&mut Transform, &mut Visibility), With<CueStick>>,
) {
    let Ok((mut transform, mut visibility)) = stick.single_mut() else {
        return;
    };
    let Some(cue) = sim.cue_position().filter(|_| sim.world.is_at_rest()) else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;

    let r = sim.table().ball.radius;
    let centre = DVec3::new(cue.x, r, cue.y).as_vec3();
    let dir = DVec3::new(aim.direction.x, 0.0, aim.direction.y).as_vec3();

    // Stick: tip just behind the ball, drawn back with power, butt raised.
    let axis = (-dir * CUE_ELEVATION.cos() + Vec3::Y * CUE_ELEVATION.sin()).normalize();
    let side = dir.cross(Vec3::Y);
    let tip_point = centre + (side * aim.tip.x as f32 + Vec3::Y * aim.tip.y as f32) * r as f32;
    let gap = 0.012 + CUE_MAX_DRAWBACK * aim.power as f32;
    let tip = tip_point + axis * (r as f32 + gap);
    transform.translation = tip + axis * CUE_LENGTH / 2.0;
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, axis);

    // Guide: line to the first ball or cushion, ghost ball, object path.
    let guide = Color::srgba(1.0, 1.0, 1.0, 0.55);
    let (end, hit) = first_contact(&sim, cue, aim.direction);
    let end3 = DVec3::new(end.x, r, end.y).as_vec3();
    gizmos.line(centre, end3, guide);
    let flat = Isometry3d::new(end3, Quat::from_rotation_arc(Vec3::Z, Vec3::Y));
    gizmos.circle(flat, r as f32, guide);
    if let Some(object) = hit {
        let push = (object - end).normalize_or_zero() * 0.3;
        let from = DVec3::new(object.x, r, object.y).as_vec3();
        gizmos.line(
            from,
            from + DVec3::new(push.x, 0.0, push.y).as_vec3(),
            Color::srgba(1.0, 0.85, 0.4, 0.7),
        );
    }
}

/// Where the cue ball would first touch something if it travelled straight.
/// Returns the cue-ball centre at contact and the struck ball's centre.
///
/// This is an assist, not a prediction: it ignores spin, curve and throw.
fn first_contact(sim: &Simulation, from: DVec2, dir: DVec2) -> (DVec2, Option<DVec2>) {
    let r = sim.table().ball.radius;
    let limit = sim.table().half_extents() - DVec2::splat(r);
    let wall = [
        (limit.x - from.x) / dir.x,
        (-limit.x - from.x) / dir.x,
        (limit.y - from.y) / dir.y,
        (-limit.y - from.y) / dir.y,
    ]
    .into_iter()
    .filter(|t| t.is_finite() && *t > 0.0)
    .fold(f64::INFINITY, f64::min);

    let mut best = (wall, None);
    for (id, s) in sim.world.sample(sim.clock) {
        if id == BallId::CUE || s.motion == Motion::Pocketed {
            continue;
        }
        let p = DVec2::new(s.position.x, s.position.z);
        // |from + dir·t − p| = 2r, smallest positive t.
        let m = from - p;
        let b = m.dot(dir);
        let c = m.length_squared() - 4.0 * r * r;
        let disc = b * b - c;
        if disc >= 0.0 {
            let t = -b - disc.sqrt();
            if t > 0.0 && t < best.0 {
                best = (t, Some(p));
            }
        }
    }
    (from + dir * best.0.min(10.0), best.1)
}
