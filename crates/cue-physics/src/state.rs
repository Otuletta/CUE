use cue_core::glam::DVec3;

/// Velocity threshold below which motion is treated as finished, in m/s.
const LINEAR_EPSILON: f64 = 1.0e-9;
/// Angular velocity threshold below which spin is treated as finished, in rad/s.
const ANGULAR_EPSILON: f64 = 1.0e-9;

/// Which motion regime a ball is in.
///
/// Each regime has its own closed-form equations of motion, which is what
/// lets the solver advance balls analytically between events instead of
/// integrating forces frame by frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Motion {
    /// At rest: no translation, no spin.
    Stationary,
    /// Not translating but still spinning about the vertical axis.
    Spinning,
    /// Translating with no slip at the cloth contact point.
    Rolling,
    /// Translating with slip at the cloth contact point.
    Sliding,
    /// Removed from play through a pocket.
    Pocketed,
}

/// Kinematic state of one ball, in world coordinates (see `cue-core`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallState {
    /// Centre position, metres.
    pub position: DVec3,
    /// Linear velocity, m/s.
    pub velocity: DVec3,
    /// Angular velocity, rad/s.
    pub angular_velocity: DVec3,
    /// Current motion regime.
    pub motion: Motion,
}

impl BallState {
    /// A ball resting on the cloth at `(x, z)`.
    pub fn at_rest(x: f64, z: f64, radius: f64) -> Self {
        Self {
            position: DVec3::new(x, radius, z),
            velocity: DVec3::ZERO,
            angular_velocity: DVec3::ZERO,
            motion: Motion::Stationary,
        }
    }

    /// Velocity of the material point touching the cloth.
    ///
    /// `u = v + ω × r`, with `r = (0, −R, 0)` from centre to contact. When
    /// this is zero the ball rolls without slipping.
    pub fn contact_velocity(&self, radius: f64) -> DVec3 {
        self.velocity + self.angular_velocity.cross(DVec3::new(0.0, -radius, 0.0))
    }

    /// Classifies the motion regime from the kinematic state alone.
    pub fn classify(&self, radius: f64) -> Motion {
        if self.motion == Motion::Pocketed {
            Motion::Pocketed
        } else if self.contact_velocity(radius).length() > LINEAR_EPSILON {
            Motion::Sliding
        } else if self.velocity.length() > LINEAR_EPSILON {
            Motion::Rolling
        } else if self.angular_velocity.y.abs() > ANGULAR_EPSILON {
            Motion::Spinning
        } else {
            Motion::Stationary
        }
    }

    /// Whether every component is finite.
    pub fn is_finite(&self) -> bool {
        self.position.is_finite() && self.velocity.is_finite() && self.angular_velocity.is_finite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f64 = 0.028575;

    fn moving(v: DVec3, w: DVec3) -> BallState {
        BallState {
            velocity: v,
            angular_velocity: w,
            ..BallState::at_rest(0.0, 0.0, R)
        }
    }

    #[test]
    fn resting_ball_is_stationary() {
        assert_eq!(
            BallState::at_rest(0.1, 0.2, R).classify(R),
            Motion::Stationary
        );
    }

    #[test]
    fn natural_roll_has_zero_contact_velocity() {
        // Rolling along +X requires ω = (0, 0, −v/R).
        let v = 1.5;
        let ball = moving(DVec3::new(v, 0.0, 0.0), DVec3::new(0.0, 0.0, -v / R));
        assert!(ball.contact_velocity(R).length() < 1e-12);
        assert_eq!(ball.classify(R), Motion::Rolling);
    }

    #[test]
    fn stun_shot_slides() {
        // A centre-ball hit: translation with no spin slips on the cloth.
        let ball = moving(DVec3::new(2.0, 0.0, 0.0), DVec3::ZERO);
        assert_eq!(ball.classify(R), Motion::Sliding);
    }

    #[test]
    fn pure_english_in_place_is_spinning() {
        let ball = moving(DVec3::ZERO, DVec3::new(0.0, 10.0, 0.0));
        assert_eq!(ball.classify(R), Motion::Spinning);
    }

    #[test]
    fn pocketed_is_sticky() {
        let mut ball = moving(DVec3::new(1.0, 0.0, 0.0), DVec3::ZERO);
        ball.motion = Motion::Pocketed;
        assert_eq!(ball.classify(R), Motion::Pocketed);
    }

    #[test]
    fn nan_is_detected() {
        let ball = moving(DVec3::new(f64::NAN, 0.0, 0.0), DVec3::ZERO);
        assert!(!ball.is_finite());
    }
}
