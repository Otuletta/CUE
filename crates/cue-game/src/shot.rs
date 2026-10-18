//! Aiming, charging and taking a shot, with mouse, touch, keyboard or the
//! host page's controls.
//!
//! In-canvas gesture (ADR-008):
//!
//! - **Aim:** move the mouse, or touch the point to shoot towards.
//! - **Power:** press and drag *back*, away from the aim direction.
//! - **Shoot:** release. Releasing with almost no pull cancels instead, so a
//!   click or a tap never fires a shot by accident.
//! - **Cancel:** `Esc` or right click.
//! - **Ball in hand** (start of a rack, after a scratch): press *on* the cue
//!   ball and drag it to a free spot.
//!
//! Every gesture has a non-pointer equivalent, so the game is playable from
//! the keyboard alone or from the host's accessible controls: arrows aim and
//! set power, `Space` shoots, `W`/`A`/`S`/`D` move the tip, `I`/`J`/`K`/`L` move
//! a cue ball in hand, `U` undoes, `R` re-racks (free play only: eight-ball
//! has no undo).

use bevy::{input::touch::Touches, prelude::*, window::PrimaryWindow};
use cue_core::{
    BallId,
    glam::{DVec2, DVec3},
    rules::eight_ball::Call,
};
use cue_physics::{MAX_TIP_OFFSET, Motion, Strike};

use crate::{
    eight::{self, Mode},
    settings::{Assists, DevHud, GameCommand, Theme},
    sim::{GameEvent, HandRegion, Paused, Simulation},
};

/// Drag distance on the table, in metres, that maps to full power.
const FULL_PULL: f64 = 0.45;
/// Shots below this power are refused, so a stray click never fires.
pub const MIN_SHOT_POWER: f64 = 0.03;
/// Perceptual power curve: more resolution for soft shots.
const POWER_EXPONENT: f64 = 1.6;
/// Keyboard aim speed, radians per second (and with `Shift` held).
const KEY_AIM_SPEED: (f64, f64) = (1.0, 0.08);
/// Keyboard power change per second.
const KEY_POWER_SPEED: f64 = 0.6;
/// Keyboard tip movement per second, in ball radii.
const KEY_TIP_SPEED: f64 = 0.6;
/// Keyboard cue-ball movement while in hand, m/s.
const KEY_PLACE_SPEED: f64 = 0.4;
/// A press within this many radii of the cue ball picks it up when in hand.
const PICK_RADII: f64 = 1.6;

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
    gesture: Option<Gesture>,
}

impl Default for Aim {
    fn default() -> Self {
        Self {
            direction: DVec2::X,
            power: 0.0,
            tip: DVec2::ZERO,
            gesture: None,
        }
    }
}

impl Aim {
    /// Whether the player is currently dragging the cue ball.
    pub fn placing(&self) -> bool {
        matches!(self.gesture, Some(Gesture::Place { .. }))
    }
}

/// A pointer gesture in progress. `touch` is the touch id, `None` for the
/// mouse.
#[derive(Debug, Clone, Copy)]
enum Gesture {
    Charge { anchor: DVec2, touch: Option<u64> },
    Place { touch: Option<u64>, valid: bool },
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

/// Shoots with the current aim. Refuses soft "shots" below
/// [`MIN_SHOT_POWER`] and, in eight-ball, shots the rules cannot take yet.
fn fire(
    sim: &mut Simulation,
    aim: &mut Aim,
    mode: &mut Mode,
    events: &mut MessageWriter<GameEvent>,
) {
    if aim.power < MIN_SHOT_POWER {
        return;
    }
    let before = match mode {
        Mode::FreePlay => None,
        Mode::EightBall(session) => {
            // The call follows the aim up to the moment of the shot.
            if !session.manual_call {
                session.call = eight::auto_call(session, sim, aim.direction);
            }
            if let Err(why) = session.can_shoot() {
                events.write(GameEvent::ShotRefused(why));
                return;
            }
            Some(sim.layout())
        }
    };
    let strike = Strike {
        direction: aim.direction,
        power: aim.power.powf(POWER_EXPONENT),
        tip_offset: aim.tip,
    };
    match sim.strike(strike) {
        Ok(()) => {
            aim.power = 0.0;
            if let (Mode::EightBall(session), Some(before)) = (mode, before) {
                session.begin_shot(before);
                // Eight-ball has no undo.
                sim.clear_undo();
            }
        }
        Err(e) => warn!("strike rejected: {e}"),
    }
}

fn undo(sim: &mut Simulation, aim: &mut Aim, mode: &Mode, events: &mut MessageWriter<GameEvent>) {
    if matches!(mode, Mode::FreePlay) && sim.undo() {
        aim.gesture = None;
        aim.power = 0.0;
        events.write(GameEvent::Undone);
    }
}

fn rerack(sim: &mut Simulation, aim: &mut Aim, mode: &Mode, events: &mut MessageWriter<GameEvent>) {
    if !matches!(mode, Mode::FreePlay) {
        return;
    }
    sim.rerack();
    *aim = Aim {
        tip: aim.tip,
        ..Aim::default()
    };
    events.write(GameEvent::Racked { cleared: false });
}

/// Applies [`GameCommand`]s sent from outside the canvas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_commands(
    mut commands: MessageReader<GameCommand>,
    paused: Res<Paused>,
    mut sim: ResMut<Simulation>,
    mut aim: ResMut<Aim>,
    mut mode: ResMut<Mode>,
    mut theme: ResMut<Theme>,
    mut assists: ResMut<Assists>,
    mut dev: ResMut<DevHud>,
    mut events: MessageWriter<GameEvent>,
) {
    for command in commands.read() {
        match command {
            GameCommand::SetTheme(t) if *theme != *t => *theme = *t,
            GameCommand::SetAssists(a) if *assists != *a => *assists = *a,
            GameCommand::ToggleDevHud => dev.0 = !dev.0,
            GameCommand::SetTip(t) if t.is_finite() => {
                aim.tip = t.clamp_length_max(MAX_TIP_OFFSET);
            }
            // A new game or rack (set up by `eight::apply_commands`, paused or
            // not) starts with a fresh aim.
            GameCommand::NewGame(_) | GameCommand::FreePlay | GameCommand::NextRack => {
                *aim = Aim {
                    tip: aim.tip,
                    ..Aim::default()
                };
            }
            // Everything below acts on the table, which a paused game must not.
            _ if paused.0 => {}
            GameCommand::SetPower(p) if p.is_finite() && sim.world.is_at_rest() => {
                aim.power = p.clamp(0.0, 1.0);
            }
            GameCommand::Shoot if sim.world.is_at_rest() => {
                fire(&mut sim, &mut aim, &mut mode, &mut events);
            }
            GameCommand::Undo => undo(&mut sim, &mut aim, &mode, &mut events),
            GameCommand::Rerack => rerack(&mut sim, &mut aim, &mode, &mut events),
            _ => {}
        }
    }
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
    mut mode: ResMut<Mode>,
    mut dev: ResMut<DevHud>,
    mut events: MessageWriter<GameEvent>,
) {
    if keys.just_pressed(KeyCode::Backquote) {
        dev.0 = !dev.0;
    }
    if paused.0 {
        aim.gesture = None;
        return;
    }
    let control = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    if keys.just_pressed(KeyCode::KeyU) || (control && keys.just_pressed(KeyCode::KeyZ)) {
        undo(&mut sim, &mut aim, &mode, &mut events);
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) && !control && matches!(*mode, Mode::FreePlay) {
        rerack(&mut sim, &mut aim, &mode, &mut events);
        return;
    }
    let Some(cue) = sim.cue_position().filter(|_| sim.world.is_at_rest()) else {
        aim.gesture = None;
        return;
    };
    // Nothing to aim at while players decide or after the rack.
    if mode.eight_ball().is_some_and(|s| !s.accepts_shot()) {
        aim.gesture = None;
        return;
    }
    let (Ok(window), Ok((camera, camera_transform))) = (window.single(), camera.single()) else {
        return;
    };
    let r = sim.table().ball.radius;
    let to_table = |screen: Vec2| table_point(camera, camera_transform, screen, r as f32);
    let aim_at = |aim: &mut Aim, p: DVec2| {
        if let Some(d) = (p - cue).try_normalize().filter(|_| p.distance(cue) > 1e-3) {
            aim.direction = d;
        }
    };

    // Cancel.
    if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        aim.gesture = None;
        aim.power = 0.0;
    }

    // Press: pick up the cue ball (in hand) or start charging.
    if aim.gesture.is_none() {
        let press = touches
            .iter_just_pressed()
            .next()
            .map(|t| (t.position(), Some(t.id())))
            .or_else(|| {
                mouse
                    .just_pressed(MouseButton::Left)
                    .then(|| window.cursor_position().map(|p| (p, None)))
                    .flatten()
            });
        if let Some((screen, touch)) = press {
            if let Some(p) = to_table(screen) {
                if sim.in_hand() && p.distance(cue) < PICK_RADII * r {
                    aim.gesture = Some(Gesture::Place { touch, valid: true });
                } else {
                    if touch.is_some() {
                        aim_at(&mut aim, p);
                    }
                    aim.gesture = Some(Gesture::Charge { anchor: p, touch });
                }
            }
        } else if touches.iter().next().is_none()
            && let Some(p) = window.cursor_position().and_then(to_table)
        {
            // Hover aims (mouse only; touch has no hover).
            aim_at(&mut aim, p);
        }
    }

    // Drag and release.
    let pointer = |touch: Option<u64>| -> (Option<DVec2>, bool) {
        match touch {
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
        }
    };
    match aim.gesture {
        Some(Gesture::Charge { anchor, touch }) => {
            let (current, released) = pointer(touch);
            if let Some(p) = current {
                let pull = (anchor - p).dot(aim.direction);
                aim.power = (pull / FULL_PULL).clamp(0.0, 1.0);
            }
            if released {
                aim.gesture = None;
                fire(&mut sim, &mut aim, &mut mode, &mut events);
                aim.power = 0.0;
                return;
            }
        }
        Some(Gesture::Place { touch, .. }) => {
            let (current, released) = pointer(touch);
            let valid = current.is_some_and(|p| sim.place_cue(p).is_ok());
            aim.gesture = (!released).then_some(Gesture::Place { touch, valid });
            return;
        }
        None => {}
    }

    // Keyboard.
    let dt = time.delta_secs_f64();
    let pressed = |k| i8::from(keys.pressed(k));
    let axis = |neg, pos| f64::from(pressed(pos) - pressed(neg));
    let fine = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let turn = axis(KeyCode::ArrowRight, KeyCode::ArrowLeft);
    if turn != 0.0 {
        let speed = if fine {
            KEY_AIM_SPEED.1
        } else {
            KEY_AIM_SPEED.0
        };
        aim.direction = DVec2::from_angle(turn * speed * dt).rotate(aim.direction);
    }
    let charge = axis(KeyCode::ArrowDown, KeyCode::ArrowUp);
    if charge != 0.0 {
        aim.power = (aim.power + charge * KEY_POWER_SPEED * dt).clamp(0.0, 1.0);
    }
    let tip = DVec2::new(
        axis(KeyCode::KeyA, KeyCode::KeyD),
        axis(KeyCode::KeyS, KeyCode::KeyW),
    );
    if tip != DVec2::ZERO {
        aim.tip = (aim.tip + tip * KEY_TIP_SPEED * dt).clamp_length_max(MAX_TIP_OFFSET);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        aim.tip = DVec2::ZERO;
    }
    if sim.in_hand() {
        // On screen, up is −Z: the far rail.
        let nudge = DVec2::new(
            axis(KeyCode::KeyJ, KeyCode::KeyL),
            axis(KeyCode::KeyI, KeyCode::KeyK),
        );
        if nudge != DVec2::ZERO {
            let _ = sim.place_cue(cue + nudge * KEY_PLACE_SPEED * dt);
        }
    }
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        fire(&mut sim, &mut aim, &mut mode, &mut events);
    }
}

/// Places the cue stick and draws the aiming guide, plus what the rules need
/// the player to see: the head string while the cue ball must stay above it,
/// and the called ball and pocket. Those are rules, not assists, so they show
/// at every assist level.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_aim(
    sim: Res<Simulation>,
    aim: Res<Aim>,
    mode: Res<Mode>,
    assists: Res<Assists>,
    theme: Res<Theme>,
    mut gizmos: Gizmos,
    mut stick: Query<(&mut Transform, &mut Visibility), With<CueStick>>,
) {
    let Ok((mut transform, mut visibility)) = stick.single_mut() else {
        return;
    };
    let Some(cue) = sim
        .cue_position()
        .filter(|_| sim.world.is_at_rest())
        .filter(|_| mode.eight_ball().is_none_or(|s| s.accepts_shot()))
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    let palette = theme.palette();
    let r = sim.table().ball.radius;
    let at = |p: DVec2| DVec3::new(p.x, r, p.y).as_vec3();
    let flat = |p: Vec3| Isometry3d::new(p, Quat::from_rotation_arc(Vec3::Z, Vec3::Y));
    let centre = at(cue);

    if sim.hand_region() == Some(HandRegion::AboveHeadString) {
        draw_head_string(&sim, &mut gizmos, palette.guide);
    }
    if let Some(session) = mode.eight_ball()
        && let Call::Ball { ball, pocket } = session.effective_call()
    {
        if let Some(s) = sim
            .world
            .state(ball)
            .filter(|s| s.motion != Motion::Pocketed)
        {
            let p = DVec2::new(s.position.x, s.position.z);
            gizmos.circle(flat(at(p)), (1.3 * r) as f32, palette.guide_accent);
        }
        let mouth = sim.world.geometry().pocket(pocket);
        let on_cloth = DVec3::new(mouth.centre.x, 0.002, mouth.centre.y).as_vec3();
        gizmos.circle(flat(on_cloth), mouth.radius as f32, palette.guide_accent);
    }

    if sim.in_hand() {
        // A ring marks the cue ball as movable; red while the drag is over
        // an invalid spot.
        let colour = match aim.gesture {
            Some(Gesture::Place { valid: false, .. }) => Color::srgb(1.0, 0.25, 0.25),
            _ => palette.guide,
        };
        gizmos.circle(flat(centre), (1.45 * r) as f32, colour);
    }
    if aim.placing() {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Inherited;

    let dir = DVec3::new(aim.direction.x, 0.0, aim.direction.y).as_vec3();
    // Stick: tip just behind the ball, drawn back with power, butt raised.
    let axis = (-dir * CUE_ELEVATION.cos() + Vec3::Y * CUE_ELEVATION.sin()).normalize();
    let side = dir.cross(Vec3::Y);
    let tip_point = centre + (side * aim.tip.x as f32 + Vec3::Y * aim.tip.y as f32) * r as f32;
    let gap = 0.012 + CUE_MAX_DRAWBACK * aim.power as f32;
    let tip = tip_point + axis * (r as f32 + gap);
    transform.translation = tip + axis * CUE_LENGTH / 2.0;
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, axis);

    if *assists == Assists::Off {
        return;
    }
    // Guide: line to the first ball or cushion, and a ghost ball there.
    let Target { end, object, .. } = first_contact(&sim, cue, aim.direction);
    gizmos.line(centre, at(end), palette.guide);
    gizmos.circle(flat(at(end)), r as f32, palette.guide);
    if *assists == Assists::Assisted
        && let Some(object) = object
    {
        // Object ball leaves along the line of centres; a stun cue ball
        // leaves along the tangent, on the side it was not pushed to.
        let out = (object - end).normalize_or_zero();
        gizmos.line(at(object), at(object + out * 0.3), palette.guide_accent);
        let tangent = out.perp() * out.perp().dot(aim.direction).signum();
        gizmos.line(at(end), at(end + tangent * 0.2), palette.guide_accent);
    }
}

/// A dashed line across the cloth at the head string.
fn draw_head_string(sim: &Simulation, gizmos: &mut Gizmos, colour: Color) {
    const DASH: f64 = 0.03;
    let table = sim.table();
    let x = table.head_string_x();
    let half = table.half_extents().y;
    let on_cloth = |z: f64| DVec3::new(x, 0.002, z).as_vec3();
    let mut z = -half;
    while z < half {
        gizmos.line(on_cloth(z), on_cloth((z + DASH).min(half)), colour);
        z += 2.0 * DASH;
    }
}

/// What the cue ball would meet first travelling straight from where it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Target {
    /// The cue-ball centre at contact (a ghost ball).
    pub(crate) end: DVec2,
    /// The struck ball's centre, if it meets a ball before a cushion.
    pub(crate) object: Option<DVec2>,
    /// The struck ball, if any.
    pub(crate) ball: Option<BallId>,
}

/// Where the cue ball would first touch something if it travelled straight.
///
/// This is an assist, not a prediction: it ignores spin, curve and throw.
pub(crate) fn first_contact(sim: &Simulation, from: DVec2, dir: DVec2) -> Target {
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
                best = (t, Some((id, p)));
            }
        }
    }
    Target {
        end: from + dir * best.0.min(10.0),
        object: best.1.map(|(_, p)| p),
        ball: best.1.map(|(id, _)| id),
    }
}
