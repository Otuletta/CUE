//! Instantaneous contact models: ball–cushion and ball–ball.
//!
//! Both follow the same recipe, which keeps them physically honest:
//!
//! 1. Treat the ball(s) as **free spheres** and apply a Newton–Coulomb
//!    impulse at the contact point: normal restitution `e`, and friction that
//!    either stops the tangential slip (stick) or is capped at `μ` times the
//!    normal impulse (slide). For spheres the normal through the contact
//!    passes through the centre, so normal and tangential responses are
//!    decoupled and this is exact.
//! 2. Let the slate absorb any vertical velocity the impulse produced. The
//!    ball stays on the cloth; spin keeps the full effect of the impulse.
//!
//! Step 1 cannot create energy for `e ≤ 1` and `μ ≥ 0`, and step 2 only
//! removes it, so **neither model can ever add kinetic energy** — a property
//! the tests check over random inputs.
//!
//! The cushion contact sits at the nose height `h`, above the ball's equator,
//! at the angle `θ = asin((h − R) / R)` (Han, 2005). That angle is what makes
//! a rolling ball come off a rail with less topspin, and what lets side spin
//! change the rebound angle.

use cue_core::glam::{DVec2, DVec3};

use crate::{BallState, SpeedFriction};

/// Inverse of a free sphere's tangential effective mass at its surface,
/// times its mass: `1 + mR²/I = 7/2`.
const TANGENTIAL: f64 = 3.5;

/// Ball–cushion contact parameters.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CushionContact {
    pub(crate) radius: f64,
    /// `sin θ` and `cos θ` of the contact point above the equator.
    pub(crate) sin_theta: f64,
    pub(crate) cos_theta: f64,
    pub(crate) restitution: f64,
    pub(crate) friction: f64,
}

impl CushionContact {
    pub(crate) fn new(radius: f64, nose_height: f64, restitution: f64, friction: f64) -> Self {
        let sin_theta = ((nose_height - radius) / radius).clamp(-1.0, 1.0);
        Self {
            radius,
            sin_theta,
            cos_theta: (1.0 - sin_theta * sin_theta).sqrt(),
            restitution,
            friction,
        }
    }
}

/// Ball–ball contact parameters. Friction depends on the slip speed at the
/// contact point.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BallContact {
    pub(crate) radius: f64,
    pub(crate) restitution: f64,
    pub(crate) friction: SpeedFriction,
}

fn horizontal(v: DVec3) -> DVec3 {
    DVec3::new(v.x, 0.0, v.z)
}

/// Coulomb friction impulse (per unit mass) against `slip`: the impulse
/// that stops the slip, if friction can supply it, otherwise `μ·normal`
/// opposite the slip. `inverse_mass` is the tangential inverse effective
/// mass times the ball mass.
fn friction_impulse(slip: DVec3, normal_impulse: f64, mu: f64, inverse_mass: f64) -> DVec3 {
    let stick = slip / inverse_mass;
    if stick.length() <= mu * normal_impulse {
        -stick
    } else {
        -mu * normal_impulse * slip.normalize_or_zero()
    }
}

/// Spin change from an impulse `j` (per unit mass) at offset `r` from the
/// centre: `Δω = (r × J) / I` with `I = 2/5·m·R²`.
fn spin_change(r: DVec3, j: DVec3, radius: f64) -> DVec3 {
    r.cross(j) * (2.5 / (radius * radius))
}

/// A ball hitting a cushion whose surface faces `inward` (horizontal unit
/// vector pointing away from the cushion). Returns the new state and the
/// horizontal speed into the cushion before contact.
pub(crate) fn cushion(s: &BallState, inward: DVec2, c: &CushionContact) -> (BallState, f64) {
    let n = DVec3::new(inward.x, 0.0, inward.y);
    // Contact point, from the centre: towards the cushion and up to the nose.
    let r = c.radius * (-c.cos_theta * n + c.sin_theta * DVec3::Y);
    // Contact normal, pointing from the cushion into the ball.
    let normal = -r / c.radius;
    let u = s.velocity + s.angular_velocity.cross(r);
    let closing = (-u.dot(normal)).max(0.0);

    let jn = (1.0 + c.restitution) * closing;
    let slip = u - u.dot(normal) * normal;
    let jt = friction_impulse(slip, jn, c.friction, TANGENTIAL);
    let j = jn * normal + jt;

    let mut out = *s;
    out.velocity = horizontal(s.velocity + j);
    out.angular_velocity += spin_change(r, j, c.radius);
    (out, (-s.velocity.dot(n)).max(0.0))
}

/// Two balls colliding, `a` and `b` of equal mass. Returns the new states and
/// the closing speed along the line of centres before contact.
pub(crate) fn ball_ball(
    a: &BallState,
    b: &BallState,
    c: &BallContact,
) -> (BallState, BallState, f64) {
    let line =
        DVec2::new(b.position.x - a.position.x, b.position.z - a.position.z).normalize_or_zero();
    let n = DVec3::new(line.x, 0.0, line.y);
    let (ra, rb) = (c.radius * n, -c.radius * n);
    // Velocity of a's contact point relative to b's.
    let rel =
        (a.velocity + a.angular_velocity.cross(ra)) - (b.velocity + b.angular_velocity.cross(rb));
    let closing = rel.dot(n).max(0.0);

    // Equal masses: the normal inverse effective mass is 2/m, the tangential
    // one 7/m (each sphere contributes 7/2m).
    let jn = 0.5 * (1.0 + c.restitution) * closing;
    let slip = rel - rel.dot(n) * n;
    let jt = friction_impulse(slip, jn, c.friction.at(slip.length()), 2.0 * TANGENTIAL);
    let ja = -jn * n + jt;

    let mut a_out = *a;
    let mut b_out = *b;
    a_out.velocity = horizontal(a.velocity + ja);
    b_out.velocity = horizontal(b.velocity - ja);
    a_out.angular_velocity += spin_change(ra, ja, c.radius);
    b_out.angular_velocity += spin_change(rb, -ja, c.radius);
    (a_out, b_out, closing)
}

/// Translational plus rotational kinetic energy per unit mass.
#[cfg(test)]
pub(crate) fn energy(s: &BallState, radius: f64) -> f64 {
    0.5 * s.velocity.length_squared() + 0.2 * radius * radius * s.angular_velocity.length_squared()
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    use super::*;
    use crate::Motion;

    const R: f64 = 0.028575;
    const THROW: SpeedFriction = SpeedFriction {
        floor: 9.951e-3,
        extra: 0.108,
        decay: 1.088,
    };

    fn ball(x: f64, z: f64, v: DVec3, w: DVec3) -> BallState {
        BallState {
            position: DVec3::new(x, R, z),
            velocity: v,
            angular_velocity: w,
            motion: Motion::Sliding,
        }
    }

    fn rail(restitution: f64, friction: f64) -> CushionContact {
        CushionContact::new(R, 0.635 * 2.0 * R, restitution, friction)
    }

    #[test]
    fn equator_contact_without_friction_is_a_plain_reflection() {
        let c = CushionContact::new(R, R, 0.8, 0.0);
        let s = ball(
            0.0,
            0.0,
            DVec3::new(1.0, 0.0, 2.0),
            DVec3::new(3.0, 4.0, 5.0),
        );
        // Cushion at +Z, facing −Z.
        let (out, speed) = cushion(&s, DVec2::NEG_Y, &c);
        assert_relative_eq!(speed, 2.0);
        assert_relative_eq!(out.velocity.z, -1.6, epsilon = 1e-12);
        assert_relative_eq!(out.velocity.x, 1.0, epsilon = 1e-12);
        assert!((out.angular_velocity - s.angular_velocity).length() < 1e-12);
    }

    #[test]
    fn nose_height_reduces_the_perpendicular_rebound() {
        // Contact above the equator: part of the impulse goes into the slate,
        // so the horizontal rebound is e·cos²θ-ish, not e.
        let c = rail(0.85, 0.0);
        let s = ball(0.0, 0.0, DVec3::new(0.0, 0.0, 2.0), DVec3::ZERO);
        let (out, _) = cushion(&s, DVec2::NEG_Y, &c);
        let ratio = -out.velocity.z / 2.0;
        assert!(ratio > 0.6 && ratio < 0.85, "rebound ratio {ratio}");
    }

    #[test]
    fn a_rolling_ball_loses_topspin_at_the_rail() {
        // Rolling towards +Z: ω = (v/R, 0, 0). The contact above the equator
        // opposes that spin.
        let c = rail(0.85, 0.2);
        let v = 1.5;
        let s = ball(
            0.0,
            0.0,
            DVec3::new(0.0, 0.0, v),
            DVec3::new(v / R, 0.0, 0.0),
        );
        let (out, _) = cushion(&s, DVec2::NEG_Y, &c);
        assert!(out.velocity.z < 0.0, "must rebound");
        assert!(
            out.angular_velocity.x < s.angular_velocity.x,
            "topspin must drop"
        );
        assert_eq!(out.velocity.y, 0.0);
    }

    #[test]
    fn side_spin_changes_the_rebound_angle_through_rail_friction() {
        // Same incoming path, opposite side spin: friction opposes the slip
        // of the contact point, so the tangential speed after contact differs
        // in the direction the physics predicts.
        let c = rail(0.85, 0.2);
        let incoming = DVec3::new(1.0, 0.0, 1.0);
        let spun = |wy: f64| {
            let s = ball(0.0, 0.0, incoming, DVec3::new(0.0, wy, 0.0));
            cushion(&s, DVec2::NEG_Y, &c).0.velocity.x
        };
        // At the +Z contact, ω_y > 0 moves the contact point towards +X, so
        // friction pushes the ball towards −X.
        assert!(spun(40.0) < spun(0.0));
        assert!(spun(-40.0) > spun(0.0));
    }

    #[test]
    fn head_on_frictionless_ball_collision_matches_the_textbook() {
        let c = BallContact {
            radius: R,
            restitution: 0.95,
            friction: SpeedFriction::NONE,
        };
        let a = ball(0.0, 0.0, DVec3::new(2.0, 0.0, 0.0), DVec3::ZERO);
        let b = ball(2.0 * R, 0.0, DVec3::ZERO, DVec3::ZERO);
        let (a2, b2, closing) = ball_ball(&a, &b, &c);
        assert_relative_eq!(closing, 2.0);
        assert_relative_eq!(a2.velocity.x, 0.05, epsilon = 1e-12);
        assert_relative_eq!(b2.velocity.x, 1.95, epsilon = 1e-12);
    }

    #[test]
    fn cut_shots_throw_the_object_ball_off_the_line_of_centres() {
        // Half-ball stun hit at 1 m/s: the object ball leaves a few degrees
        // off the line of centres, towards the cue ball's direction of travel.
        let c = BallContact {
            radius: R,
            restitution: 0.95,
            friction: THROW,
        };
        // Line of centres at 30° from the cue ball's path (half-ball hit).
        let line = DVec2::from_angle(30f64.to_radians());
        let a = ball(0.0, 0.0, DVec3::new(1.0, 0.0, 0.0), DVec3::ZERO);
        let b = ball(2.0 * R * line.x, 2.0 * R * line.y, DVec3::ZERO, DVec3::ZERO);
        let (_, b2, _) = ball_ball(&a, &b, &c);
        let out = DVec2::new(b2.velocity.x, b2.velocity.z);
        let throw = line.angle_to(out).to_degrees();
        // Thrown towards the cue ball's path, i.e. clockwise from the line.
        assert!(throw < -1.0 && throw > -6.0, "throw {throw}°");
    }

    #[test]
    fn right_english_throws_a_straight_shot_to_the_left() {
        // Shooting along +X, "right" is +Z and right english is ω_y > 0.
        let c = BallContact {
            radius: R,
            restitution: 0.95,
            friction: THROW,
        };
        let a = ball(
            0.0,
            0.0,
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.0, 30.0, 0.0),
        );
        let b = ball(2.0 * R, 0.0, DVec3::ZERO, DVec3::ZERO);
        let (_, b2, _) = ball_ball(&a, &b, &c);
        assert!(b2.velocity.z < 0.0, "object ball must go left (−Z)");
    }

    fn arb_state() -> impl Strategy<Value = (DVec3, DVec3)> {
        let v = (-5.0..5.0f64, -5.0..5.0f64);
        let w = (-300.0..300.0f64, -300.0..300.0f64, -300.0..300.0f64);
        (v, w)
            .prop_map(|((vx, vz), (wx, wy, wz))| (DVec3::new(vx, 0.0, vz), DVec3::new(wx, wy, wz)))
    }

    proptest! {
        #[test]
        fn cushion_impacts_never_add_energy(
            (v, w) in arb_state(),
            e in 0.0..=1.0f64,
            mu in 0.0..1.0f64,
            angle in 0.0..std::f64::consts::TAU,
        ) {
            let c = rail(e, mu);
            let s = ball(0.0, 0.0, v, w);
            let (out, _) = cushion(&s, DVec2::from_angle(angle), &c);
            prop_assert!(energy(&out, R) <= energy(&s, R) + 1e-12);
        }

        #[test]
        fn ball_impacts_never_add_energy_and_conserve_momentum(
            (va, wa) in arb_state(),
            (vb, wb) in arb_state(),
            e in 0.0..=1.0f64,
            angle in 0.0..std::f64::consts::TAU,
        ) {
            let c = BallContact { radius: R, restitution: e, friction: THROW };
            let line = DVec2::from_angle(angle) * 2.0 * R;
            let a = ball(0.0, 0.0, va, wa);
            let b = ball(line.x, line.y, vb, wb);
            let (a2, b2, _) = ball_ball(&a, &b, &c);
            let before = energy(&a, R) + energy(&b, R);
            let after = energy(&a2, R) + energy(&b2, R);
            prop_assert!(after <= before + 1e-12, "{} > {}", after, before);
            let momentum = (a.velocity + b.velocity) - (a2.velocity + b2.velocity);
            prop_assert!(momentum.length() < 1e-9);
        }
    }
}
