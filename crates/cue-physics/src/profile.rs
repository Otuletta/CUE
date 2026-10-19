use cue_core::TableSpec;

/// Ball–cloth interaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClothParams {
    /// Coefficient of sliding friction between ball and cloth (dimensionless).
    pub sliding_friction: f64,
    /// Coefficient of rolling resistance (dimensionless).
    pub rolling_resistance: f64,
    /// Coefficient of spinning friction (dimensionless). Vertical-axis spin
    /// decelerates at `α = 5·μ·g / (2·R)`.
    pub spinning_friction: f64,
}

/// Ball–ball interaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallBallParams {
    /// Coefficient of restitution along the line of centres.
    pub restitution: f64,
    /// Friction between ball surfaces, which drives throw and spin transfer.
    pub friction: SpeedFriction,
}

/// A friction coefficient that falls with slip speed:
/// `μ(v) = floor + extra · e^(−decay · v)`.
///
/// Phenolic balls grip more when they slide slowly against each other, which
/// is why slow cut shots throw more than fast ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedFriction {
    /// Coefficient at high slip speed.
    pub floor: f64,
    /// Additional coefficient at zero slip speed.
    pub extra: f64,
    /// Decay rate, s/m.
    pub decay: f64,
}

impl SpeedFriction {
    /// No friction at all.
    pub const NONE: Self = Self {
        floor: 0.0,
        extra: 0.0,
        decay: 0.0,
    };

    /// Coefficient at slip speed `v` (m/s).
    pub fn at(&self, v: f64) -> f64 {
        self.floor + self.extra * (-self.decay * v).exp()
    }
}

/// Ball–cushion interaction. The contact height comes from
/// [`TableSpec::cushion_nose_height`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CushionParams {
    /// Coefficient of restitution along the contact normal.
    pub restitution: f64,
    /// Coefficient of friction between ball and cushion cloth.
    pub friction: f64,
}

/// Cue delivery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CueParams {
    /// Cue-ball speed at full power, m/s. A good amateur break is around
    /// 9 m/s and a professional one 11–13 m/s (Alciatore); full power is a
    /// strong professional break, so that breaks open the rack as they do on
    /// a real table.
    pub max_speed: f64,
    /// Effective mass of the shaft's tip end, kg. It sets **squirt**: the
    /// lighter it is, the less a side-spin shot pushes the cue ball off the
    /// aim line. Typical shafts are 5–10 g; low-deflection shafts are lighter.
    pub shaft_endmass: f64,
}

/// A complete, versioned set of physical parameters.
///
/// Every simulated shot records the profile id it ran with, so a change in
/// tuning is an observable, documented change rather than a silent one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsProfile {
    /// Stable identifier, bumped whenever any value changes.
    pub id: &'static str,
    /// Table and ball geometry.
    pub table: TableSpec,
    /// Ball–cloth parameters.
    pub cloth: ClothParams,
    /// Ball–ball parameters.
    pub ball_ball: BallBallParams,
    /// Ball–cushion parameters.
    pub cushion: CushionParams,
    /// Cue delivery.
    pub cue: CueParams,
    /// Gravitational acceleration, m/s².
    pub gravity: f64,
}

impl PhysicsProfile {
    /// The reference profile, on a 7-foot table: the size most eight-ball is
    /// played on, and the one that shows the balls large enough to read.
    ///
    /// Cloth, restitution and friction values are the typical figures
    /// compiled by Dr. Dave Alciatore (Colorado State University) in *Typical
    /// Physical Properties of Pool Equipment*. The ball–ball friction curve is
    /// his fit to measured throw. Cushion values follow common practice for
    /// the Han (2005) contact model. Rolling resistance sits just above the
    /// typical range, a medium-slow cloth, so a shot settles in 4–6 s as on a
    /// club table rather than gliding for nine. `docs/physics-model.md` lists
    /// the reference behaviours this profile is checked against.
    pub const fn baseline() -> Self {
        Self {
            id: "baseline-4",
            table: TableSpec::seven_foot(),
            cloth: ClothParams {
                sliding_friction: 0.2,
                rolling_resistance: 0.018,
                // ≈10 rad/s² spin deceleration, mid-range of the 5–15 reported.
                spinning_friction: 0.0117,
            },
            ball_ball: BallBallParams {
                restitution: 0.95,
                friction: SpeedFriction {
                    floor: 9.951e-3,
                    extra: 0.108,
                    decay: 1.088,
                },
            },
            cushion: CushionParams {
                restitution: 0.85,
                friction: 0.2,
            },
            cue: CueParams {
                max_speed: 12.5,
                shaft_endmass: 0.007,
            },
            gravity: 9.80665,
        }
    }
}

impl Default for PhysicsProfile {
    fn default() -> Self {
        Self::baseline()
    }
}
