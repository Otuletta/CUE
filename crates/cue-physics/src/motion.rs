//! Closed-form ball motion on cloth.
//!
//! Between events every ball follows one of four regimes, each with an exact
//! solution (Leckie & Greenspan, 2005):
//!
//! | Regime   | Linear motion                          | Ends when                    |
//! |----------|----------------------------------------|------------------------------|
//! | Sliding  | decelerates at `μs·g` along the slip   | slip vanishes: `2|u|/(7μs·g)`|
//! | Rolling  | decelerates at `μr·g` along `v`        | `v = 0`: `|v|/(μr·g)`        |
//! | Spinning | none                                   | `ωy = 0`                     |
//!
//! Vertical-axis spin `ωy` decays at `5·μsp·g/(2R)` in every moving regime and
//! never changes the trajectory on its own. Positions are quadratic in time,
//! which is what keeps collision detection polynomial.

use cue_core::glam::{DVec2, DVec3};

use crate::{BallState, Motion, PhysicsProfile};

/// Derived constants used by the equations of motion.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Kinematics {
    pub(crate) radius: f64,
    /// Linear deceleration while sliding, m/s².
    slide_decel: f64,
    /// Linear deceleration while rolling, m/s².
    roll_decel: f64,
    /// Angular deceleration of `ωy`, rad/s².
    spin_decel: f64,
}

impl Kinematics {
    pub(crate) fn new(p: &PhysicsProfile) -> Self {
        let radius = p.table.ball.radius;
        Self {
            radius,
            slide_decel: p.cloth.sliding_friction * p.gravity,
            roll_decel: p.cloth.rolling_resistance * p.gravity,
            spin_decel: 5.0 * p.cloth.spinning_friction * p.gravity / (2.0 * radius),
        }
    }
}

/// Projects a world vector onto the table plane (`x`, `z`).
pub(crate) fn planar(v: DVec3) -> DVec2 {
    DVec2::new(v.x, v.z)
}

/// In-plane velocity of the cloth contact point: `u = v + ω × (0, −R, 0)`.
fn slip(s: &BallState, r: f64) -> DVec2 {
    DVec2::new(
        s.velocity.x + r * s.angular_velocity.z,
        s.velocity.z - r * s.angular_velocity.x,
    )
}

/// A ball's path over its current regime: `p(t) = c + b·t + a·t²` on the
/// table plane, valid for `t ∈ [0, end]`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Trajectory {
    pub(crate) a: DVec2,
    pub(crate) b: DVec2,
    pub(crate) c: DVec2,
    pub(crate) end: f64,
}

impl Trajectory {
    pub(crate) fn is_moving(&self) -> bool {
        self.b != DVec2::ZERO || self.a != DVec2::ZERO
    }

    /// Upper bound on distance travelled within `t` seconds.
    pub(crate) fn reach(&self, t: f64) -> f64 {
        self.b.length() * t + self.a.length() * t * t
    }
}

pub(crate) fn trajectory(s: &BallState, k: &Kinematics) -> Trajectory {
    let c = planar(s.position);
    let b = planar(s.velocity);
    let still = |end| Trajectory {
        a: DVec2::ZERO,
        b: DVec2::ZERO,
        c,
        end,
    };
    match s.motion {
        Motion::Sliding => {
            let u = slip(s, k.radius);
            Trajectory {
                a: -0.5 * k.slide_decel * u.normalize_or_zero(),
                b,
                c,
                end: 2.0 * u.length() / (7.0 * k.slide_decel),
            }
        }
        Motion::Rolling => Trajectory {
            a: -0.5 * k.roll_decel * b.normalize_or_zero(),
            b,
            c,
            end: b.length() / k.roll_decel,
        },
        Motion::Spinning => still(s.angular_velocity.y.abs() / k.spin_decel),
        Motion::Stationary | Motion::Pocketed => still(f64::INFINITY),
    }
}

/// Advances a ball by `dt ≤ trajectory().end` within its current regime.
pub(crate) fn evolve(s: &BallState, dt: f64, k: &Kinematics) -> BallState {
    if dt == 0.0 || matches!(s.motion, Motion::Stationary | Motion::Pocketed) {
        return *s;
    }
    let mut next = *s;
    let wy = s.angular_velocity.y;
    next.angular_velocity.y = wy.signum() * (wy.abs() - k.spin_decel * dt).max(0.0);

    match s.motion {
        Motion::Sliding => {
            let u_hat = slip(s, k.radius).normalize_or_zero();
            let dv = -k.slide_decel * dt * u_hat;
            let v = planar(s.velocity);
            let p = planar(s.position) + v * dt + 0.5 * dv * dt;
            // Friction torque about the centre: (5·μs·g / 2R) · (ûz, 0, −ûx).
            let dw = 2.5 * k.slide_decel / k.radius * dt;
            next.position = DVec3::new(p.x, k.radius, p.y);
            next.velocity = DVec3::new(v.x + dv.x, 0.0, v.y + dv.y);
            next.angular_velocity.x += dw * u_hat.y;
            next.angular_velocity.z -= dw * u_hat.x;
        }
        Motion::Rolling => {
            let v = planar(s.velocity);
            let dv = -k.roll_decel * dt * v.normalize_or_zero();
            let p = planar(s.position) + v * dt + 0.5 * dv * dt;
            next.position = DVec3::new(p.x, k.radius, p.y);
            next.velocity = DVec3::new(v.x + dv.x, 0.0, v.y + dv.y);
            roll_spin_from_velocity(&mut next, k.radius);
        }
        Motion::Spinning | Motion::Stationary | Motion::Pocketed => {}
    }
    next
}

/// Applies the regime change that happens at the end of the current regime.
pub(crate) fn transition(s: &BallState, k: &Kinematics) -> BallState {
    let mut next = *s;
    match s.motion {
        Motion::Sliding => {
            // Slip has vanished: the ball now rolls (or, after a stop-shot,
            // only spins in place).
            roll_spin_from_velocity(&mut next, k.radius);
            return settle(&next, k);
        }
        Motion::Rolling => {
            next.velocity = DVec3::ZERO;
            next.angular_velocity.x = 0.0;
            next.angular_velocity.z = 0.0;
            next.motion = Motion::Spinning;
            return settle(&next, k);
        }
        Motion::Spinning => {
            next.angular_velocity = DVec3::ZERO;
            next.motion = Motion::Stationary;
        }
        Motion::Stationary | Motion::Pocketed => {}
    }
    next
}

/// Reclassifies a ball after an impulse and snaps its state to the exact
/// representation of the new regime, so no regime carries numerical residue
/// (a "stationary" ball with 1e-12 m/s, a "rolling" ball that slightly slips).
pub(crate) fn settle(s: &BallState, k: &Kinematics) -> BallState {
    let mut next = *s;
    next.position.y = k.radius;
    next.velocity.y = 0.0;
    next.motion = s.classify(k.radius);
    match next.motion {
        Motion::Sliding => {}
        Motion::Rolling => roll_spin_from_velocity(&mut next, k.radius),
        Motion::Spinning => {
            next.velocity = DVec3::ZERO;
            next.angular_velocity.x = 0.0;
            next.angular_velocity.z = 0.0;
        }
        Motion::Stationary => {
            next.velocity = DVec3::ZERO;
            next.angular_velocity = DVec3::ZERO;
        }
        Motion::Pocketed => {}
    }
    next
}

/// Sets the in-plane spin that makes the contact point stationary.
fn roll_spin_from_velocity(s: &mut BallState, r: f64) {
    s.angular_velocity.x = s.velocity.z / r;
    s.angular_velocity.z = -s.velocity.x / r;
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn setup() -> (Kinematics, PhysicsProfile) {
        let p = PhysicsProfile::baseline();
        (Kinematics::new(&p), p)
    }

    fn launched(v: DVec3, w: DVec3, k: &Kinematics) -> BallState {
        settle(
            &BallState {
                velocity: v,
                angular_velocity: w,
                ..BallState::at_rest(0.0, 0.0, k.radius)
            },
            k,
        )
    }

    /// Runs a ball through its regimes until it stops; returns the final state
    /// and the regimes visited.
    fn run_to_rest(mut s: BallState, k: &Kinematics) -> (BallState, Vec<Motion>) {
        let mut visited = vec![s.motion];
        while s.motion != Motion::Stationary {
            let end = trajectory(&s, k).end;
            s = transition(&evolve(&s, end, k), k);
            visited.push(s.motion);
            assert!(visited.len() < 10, "regimes must terminate: {visited:?}");
        }
        (s, visited)
    }

    #[test]
    fn stun_shot_rolls_at_five_sevenths_of_launch_speed() {
        // Classic result: a ball launched without spin starts rolling once it
        // has lost 2/7 of its speed.
        let (k, _) = setup();
        let v0 = 2.0;
        let s = launched(DVec3::new(v0, 0.0, 0.0), DVec3::ZERO, &k);
        assert_eq!(s.motion, Motion::Sliding);
        let end = trajectory(&s, &k).end;
        let rolling = transition(&evolve(&s, end, &k), &k);
        assert_eq!(rolling.motion, Motion::Rolling);
        assert_relative_eq!(rolling.velocity.x, v0 * 5.0 / 7.0, epsilon = 1e-12);
        assert!(rolling.contact_velocity(k.radius).length() < 1e-12);
    }

    #[test]
    fn evolve_stays_consistent_with_the_trajectory_polynomial() {
        let (k, _) = setup();
        let s = launched(DVec3::new(1.2, 0.0, -0.7), DVec3::new(30.0, 4.0, 10.0), &k);
        let traj = trajectory(&s, &k);
        for frac in [0.1, 0.5, 0.9, 1.0] {
            let t = traj.end * frac;
            let p = planar(evolve(&s, t, &k).position);
            let poly = traj.c + traj.b * t + traj.a * t * t;
            assert_relative_eq!(p.x, poly.x, epsilon = 1e-12);
            assert_relative_eq!(p.y, poly.y, epsilon = 1e-12);
        }
    }

    #[test]
    fn rolling_ball_stops_after_the_textbook_distance() {
        // d = v² / (2·μr·g).
        let (k, p) = setup();
        let v0 = 1.0;
        let s = launched(
            DVec3::new(v0, 0.0, 0.0),
            DVec3::new(0.0, 0.0, -v0 / k.radius),
            &k,
        );
        assert_eq!(s.motion, Motion::Rolling);
        let (end, visited) = run_to_rest(s, &k);
        let expected = v0 * v0 / (2.0 * p.cloth.rolling_resistance * p.gravity);
        assert_relative_eq!(end.position.x, expected, epsilon = 1e-9);
        assert_eq!(visited, [Motion::Rolling, Motion::Stationary]);
    }

    #[test]
    fn draw_shot_reverses_spin_before_rolling_forward() {
        // Strong backspin: the ball slides, the slip flips the spin, and it
        // ends up rolling forward at a reduced speed.
        let (k, _) = setup();
        let v0 = 2.0;
        let s = launched(
            DVec3::new(v0, 0.0, 0.0),
            DVec3::new(0.0, 0.0, 1.5 * v0 / k.radius),
            &k,
        );
        let end = trajectory(&s, &k).end;
        let rolling = transition(&evolve(&s, end, &k), &k);
        assert!(rolling.velocity.x > 0.0 && rolling.velocity.x < v0 * 5.0 / 7.0);
        assert!(
            rolling.angular_velocity.z < 0.0,
            "spin must now be forward roll"
        );
    }

    #[test]
    fn side_spin_decays_without_bending_a_rolling_path() {
        let (k, _) = setup();
        let v0 = 1.0;
        let s = launched(
            DVec3::new(v0, 0.0, 0.0),
            DVec3::new(0.0, 25.0, -v0 / k.radius),
            &k,
        );
        let (end, _) = run_to_rest(s, &k);
        assert_relative_eq!(end.position.z, 0.0);
        assert_eq!(end.angular_velocity, DVec3::ZERO);
    }

    #[test]
    fn settle_removes_numerical_residue() {
        let (k, _) = setup();
        let s = launched(DVec3::new(1e-12, 0.0, 0.0), DVec3::new(0.0, 1e-12, 0.0), &k);
        assert_eq!(s.motion, Motion::Stationary);
        assert_eq!(s.velocity, DVec3::ZERO);
        assert_eq!(s.angular_velocity, DVec3::ZERO);
    }
}
