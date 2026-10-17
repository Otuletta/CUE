use cue_core::glam::{DVec2, DVec3};

use crate::SimError;

/// Largest tip offset from the centre of the cue ball, as a fraction of its
/// radius. Beyond roughly half a radius a real cue miscues.
pub const MAX_TIP_OFFSET: f64 = 0.5;

/// A validated instruction to hit the cue ball.
///
/// This is the only way energy enters the simulation. It is applied once, as
/// an instantaneous impulse; nothing about it depends on frame timing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    /// Aim direction on the table plane (`x`, `z`). Normalised on use.
    pub direction: DVec2,
    /// Shot strength in `(0, 1]`, mapped linearly to cue-ball speed up to
    /// [`CueParams::max_speed`](crate::CueParams::max_speed). Any perceptual
    /// curve belongs to the input layer, not here.
    pub power: f64,
    /// Where the tip meets the ball, in ball radii, seen from behind the
    /// cue: `x` is side spin (positive = right), `y` is height (positive =
    /// follow, negative = draw). Must lie within [`MAX_TIP_OFFSET`].
    pub tip_offset: DVec2,
}

impl Strike {
    /// A centre-ball hit.
    pub fn centre(direction: DVec2, power: f64) -> Self {
        Self {
            direction,
            power,
            tip_offset: DVec2::ZERO,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), SimError> {
        if !self.direction.is_finite() || self.direction.length_squared() < 1e-12 {
            return Err(SimError::InvalidStrike(
                "direction must be finite and non-zero",
            ));
        }
        if !(self.power.is_finite() && self.power > 0.0 && self.power <= 1.0) {
            return Err(SimError::InvalidStrike("power must be in (0, 1]"));
        }
        if !self.tip_offset.is_finite() || self.tip_offset.length() > MAX_TIP_OFFSET + 1e-12 {
            return Err(SimError::InvalidStrike(
                "tip offset is outside the miscue limit",
            ));
        }
        Ok(())
    }

    /// Linear and angular velocity imparted to the cue ball.
    ///
    /// The cue delivers a horizontal impulse `J = m·V·d̂` at the contact
    /// point `r` on the ball's surface. Then `v = J/m` and `ω = (r × J)/I`
    /// with `I = 2/5·m·R²`. A tip height of `+2/5·R` gives natural roll.
    pub(crate) fn velocities(&self, speed: f64, radius: f64) -> (DVec3, DVec3) {
        let d = self.direction.normalize();
        let forward = DVec3::new(d.x, 0.0, d.y);
        let right = forward.cross(DVec3::Y);
        let (a, b) = (self.tip_offset.x, self.tip_offset.y);
        let depth = (1.0 - a * a - b * b).max(0.0).sqrt();
        let r = radius * (a * right + b * DVec3::Y - depth * forward);

        let v = speed * forward;
        let w = 2.5 * speed / (radius * radius) * r.cross(forward);
        (v, w)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    const R: f64 = 0.028575;

    #[test]
    fn centre_hit_has_no_spin() {
        let (v, w) = Strike::centre(DVec2::X, 1.0).velocities(3.0, R);
        assert_relative_eq!(v.x, 3.0);
        assert!(w.length() < 1e-12);
    }

    #[test]
    fn two_fifths_above_centre_is_natural_roll() {
        let s = Strike {
            direction: DVec2::X,
            power: 1.0,
            tip_offset: DVec2::new(0.0, 0.4),
        };
        let (v, w) = s.velocities(2.0, R);
        let contact = v + w.cross(DVec3::new(0.0, -R, 0.0));
        assert!(contact.length() < 1e-12, "contact point must be at rest");
    }

    #[test]
    fn side_spin_is_about_the_vertical_axis_only() {
        let s = Strike {
            direction: DVec2::new(0.0, 1.0),
            power: 1.0,
            tip_offset: DVec2::new(0.3, 0.0),
        };
        let (_, w) = s.velocities(2.0, R);
        assert!(w.y.abs() > 1.0);
        assert!(w.x.abs() < 1e-12 && w.z.abs() < 1e-12);
    }

    #[test]
    fn rejects_bad_input() {
        let ok = Strike::centre(DVec2::X, 0.5);
        assert!(ok.validate().is_ok());
        for bad in [
            Strike {
                direction: DVec2::ZERO,
                ..ok
            },
            Strike {
                direction: DVec2::new(f64::NAN, 1.0),
                ..ok
            },
            Strike { power: 0.0, ..ok },
            Strike { power: 1.5, ..ok },
            Strike {
                power: f64::INFINITY,
                ..ok
            },
            Strike {
                tip_offset: DVec2::new(0.6, 0.0),
                ..ok
            },
        ] {
            assert!(bad.validate().is_err(), "{bad:?} should be rejected");
        }
    }
}
