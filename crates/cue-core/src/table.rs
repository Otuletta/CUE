use std::f64::consts::PI;

use glam::DVec2;

use crate::{CoreError, PocketSpec};

/// Metres per inch, exact by definition.
const INCH: f64 = 0.0254;
/// Radians per degree.
const DEGREE: f64 = PI / 180.0;

/// Physical description of a ball.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallSpec {
    /// Radius in metres.
    pub radius: f64,
    /// Mass in kilograms.
    pub mass: f64,
}

impl BallSpec {
    /// A standard pool ball: 2¼ in diameter, 6 oz.
    ///
    /// The WPA allows 5½–6 oz; mass is a calibration parameter, not a rule.
    pub const fn standard() -> Self {
        Self {
            radius: 2.25 * INCH / 2.0,
            mass: 0.170,
        }
    }

    /// Diameter in metres.
    pub const fn diameter(&self) -> f64 {
        self.radius * 2.0
    }
}

/// Geometry of a pool table as seen by the simulation.
///
/// Dimensions describe the *playing surface*: the area bounded by the cushion
/// noses. Rendering geometry (rails, frame, legs) is a separate concern and
/// must never be used for collision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableSpec {
    /// Length of the playing surface along `X`, in metres.
    pub playfield_length: f64,
    /// Width of the playing surface along `Z`, in metres.
    pub playfield_width: f64,
    /// The balls played on this table.
    pub ball: BallSpec,
    /// The four corner pockets.
    pub corner_pocket: PocketSpec,
    /// The two side pockets.
    pub side_pocket: PocketSpec,
    /// Height of the cushion nose above the cloth, in metres.
    pub cushion_nose_height: f64,
    /// Depth of the cushion from its nose to the wooden rail, in metres. The
    /// pocket facings run through this depth.
    pub cushion_width: f64,
}

impl TableSpec {
    /// A 9-foot table using the midpoints of the WPA equipment tolerances.
    ///
    /// | Dimension | WPA tolerance | Used |
    /// |---|---|---|
    /// | Playing surface | 100 × 50 in ± ⅛ | 100 × 50 in |
    /// | Corner mouth / facing angle | 4½–4⅝ in / 142° ± 1° | 4.5625 in / 142° |
    /// | Side mouth / facing angle | 5–5⅛ in / 104° ± 1° | 5.0625 in / 104° |
    /// | Corner / side shelf | 1–2¼ in / 0–⅜ in | 1.625 in / 0.1875 in |
    /// | Cushion nose height | 62.5–64.5 % of ball diameter | 63.5 % |
    ///
    /// The cushion width (2 in) and the drop radii are modelling choices,
    /// not WPA dimensions. All of these are starting values for calibration;
    /// matching them is not a claim of conformity.
    pub const fn nine_foot() -> Self {
        let ball = BallSpec::standard();
        Self {
            playfield_length: 100.0 * INCH,
            playfield_width: 50.0 * INCH,
            ball,
            corner_pocket: PocketSpec {
                mouth: 4.5625 * INCH,
                facing_angle: 142.0 * DEGREE,
                shelf: 1.625 * INCH,
                drop_radius: 0.054,
            },
            side_pocket: PocketSpec {
                mouth: 5.0625 * INCH,
                facing_angle: 104.0 * DEGREE,
                shelf: 0.1875 * INCH,
                drop_radius: 0.058,
            },
            cushion_nose_height: 0.635 * ball.diameter(),
            cushion_width: 2.0 * INCH,
        }
    }

    /// Checks that every dimension is finite, positive and mutually coherent.
    pub fn validate(&self) -> Result<(), CoreError> {
        let (c, sd) = (&self.corner_pocket, &self.side_pocket);
        let fields = [
            self.playfield_length,
            self.playfield_width,
            self.ball.radius,
            self.ball.mass,
            c.mouth,
            c.facing_angle,
            c.drop_radius,
            sd.mouth,
            sd.facing_angle,
            sd.drop_radius,
            self.cushion_nose_height,
            self.cushion_width,
        ];
        if !fields.iter().all(|v| v.is_finite() && *v > 0.0) {
            return Err(CoreError::InvalidTableSpec(
                "all dimensions must be finite and positive",
            ));
        }
        if self.playfield_width > self.playfield_length {
            return Err(CoreError::InvalidTableSpec(
                "the playing surface must be longer than it is wide",
            ));
        }
        if !(c.shelf.is_finite() && c.shelf >= 0.0 && sd.shelf.is_finite() && sd.shelf >= 0.0) {
            return Err(CoreError::InvalidTableSpec(
                "pocket shelves must be finite and non-negative",
            ));
        }
        let d = self.ball.diameter();
        if c.mouth <= d || sd.mouth <= d {
            return Err(CoreError::InvalidTableSpec(
                "pocket mouths must be wider than a ball",
            ));
        }
        let right = PI / 2.0;
        if !(right..PI).contains(&c.facing_angle) || !(right..PI).contains(&sd.facing_angle) {
            return Err(CoreError::InvalidTableSpec(
                "pocket facings must make an angle in [90°, 180°) with the cushion",
            ));
        }
        if c.drop_radius <= self.ball.radius || sd.drop_radius <= self.ball.radius {
            return Err(CoreError::InvalidTableSpec(
                "pocket drop regions must be larger than a ball",
            ));
        }
        if self.cushion_nose_height <= self.ball.radius || self.cushion_nose_height >= d {
            return Err(CoreError::InvalidTableSpec(
                "cushion nose must sit between the ball's equator and its top",
            ));
        }
        Ok(())
    }

    /// Half extents of the playing surface on the table plane.
    pub fn half_extents(&self) -> DVec2 {
        DVec2::new(self.playfield_length, self.playfield_width) / 2.0
    }

    /// The foot spot, where the apex ball of the rack is placed.
    pub fn foot_spot(&self) -> DVec2 {
        DVec2::new(self.playfield_length / 4.0, 0.0)
    }

    /// The head spot, where the head string meets the long string.
    pub fn head_spot(&self) -> DVec2 {
        DVec2::new(self.head_string_x(), 0.0)
    }

    /// Default cue-ball position for a break: on the long string, two ball
    /// diameters above the head string.
    ///
    /// Not the head spot itself: the cue ball must start *above* the head
    /// string, and the string is not part of that area (WPA 2.1, 4.3 a).
    pub fn break_spot(&self) -> DVec2 {
        DVec2::new(self.head_string_x() - 2.0 * self.ball.diameter(), 0.0)
    }

    /// `X` coordinate of the head string. The break is taken from above
    /// (behind) it, that is from `x < head_string_x()`.
    pub fn head_string_x(&self) -> f64 {
        -self.playfield_length / 4.0
    }

    /// Whether a ball centred at `p` is above the head string, the area the
    /// cue ball is restricted to for the break (WPA 2.1). A ball on the
    /// string is not above it.
    pub fn above_head_string(&self, p: DVec2) -> bool {
        p.x < self.head_string_x()
    }

    /// Whether a ball centred at `p` lies fully on the playing surface,
    /// without overlapping any cushion nose.
    pub fn contains_ball(&self, p: DVec2) -> bool {
        let limit = self.half_extents() - DVec2::splat(self.ball.radius);
        p.is_finite() && p.x.abs() <= limit.x && p.y.abs() <= limit.y
    }
}

impl Default for TableSpec {
    fn default() -> Self {
        Self::nine_foot()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn nine_foot_table_is_valid_and_two_to_one() {
        let t = TableSpec::nine_foot();
        assert_eq!(t.validate(), Ok(()));
        assert_relative_eq!(t.playfield_length, 2.54);
        assert_relative_eq!(t.playfield_width, 1.27);
        assert_relative_eq!(t.playfield_length / t.playfield_width, 2.0);
    }

    #[test]
    fn standard_ball_matches_spec() {
        let b = BallSpec::standard();
        assert_relative_eq!(b.diameter(), 0.05715);
        assert_relative_eq!(b.radius, 0.028575);
    }

    #[test]
    fn spots_sit_on_the_long_axis_quarters() {
        let t = TableSpec::nine_foot();
        assert_relative_eq!(t.foot_spot().x, 0.635);
        assert_relative_eq!(t.head_spot().x, -0.635);
        assert_relative_eq!(t.foot_spot().y, 0.0);
    }

    #[test]
    fn rejects_degenerate_specs() {
        let ok = TableSpec::nine_foot();
        let cases = [
            TableSpec {
                playfield_length: 0.0,
                ..ok
            },
            TableSpec {
                playfield_width: f64::NAN,
                ..ok
            },
            TableSpec {
                playfield_width: 3.0,
                ..ok
            },
            TableSpec {
                corner_pocket: PocketSpec {
                    mouth: 0.01,
                    ..ok.corner_pocket
                },
                ..ok
            },
            TableSpec {
                side_pocket: PocketSpec {
                    facing_angle: 0.5,
                    ..ok.side_pocket
                },
                ..ok
            },
            TableSpec {
                corner_pocket: PocketSpec {
                    shelf: -0.01,
                    ..ok.corner_pocket
                },
                ..ok
            },
            TableSpec {
                cushion_nose_height: 0.01,
                ..ok
            },
            TableSpec {
                ball: BallSpec {
                    mass: -1.0,
                    ..ok.ball
                },
                ..ok
            },
        ];
        for spec in cases {
            assert!(spec.validate().is_err(), "{spec:?} should be rejected");
        }
    }

    #[test]
    fn contains_ball_respects_radius_and_rejects_nan() {
        let t = TableSpec::nine_foot();
        let edge = t.half_extents() - DVec2::splat(t.ball.radius);
        assert!(t.contains_ball(DVec2::ZERO));
        assert!(t.contains_ball(edge));
        assert!(!t.contains_ball(edge + DVec2::new(1e-6, 0.0)));
        assert!(!t.contains_ball(DVec2::new(f64::NAN, 0.0)));
    }
}
