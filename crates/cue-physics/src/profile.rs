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
    /// Coefficient of friction between ball surfaces (drives throw).
    pub friction: f64,
}

/// Ball–cushion interaction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CushionParams {
    /// Coefficient of restitution normal to the cushion.
    pub restitution: f64,
    /// Coefficient of friction between ball and cushion rubber.
    pub friction: f64,
}

/// Cue delivery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CueParams {
    /// Cue-ball speed at full power, m/s. A hard amateur break is around
    /// 7–8 m/s; professionals exceed 10 m/s.
    pub max_speed: f64,
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
    /// Starting point for calibration.
    ///
    /// Values are the typical figures compiled by Dr. Dave Alciatore
    /// (Colorado State University) in *Typical Physical Properties of Pool
    /// Equipment*. They are uncalibrated against CUE's own reference shots;
    /// Phase 2 replaces them with measured values and bumps the id.
    pub const fn baseline() -> Self {
        Self {
            id: "baseline-0",
            table: TableSpec::nine_foot(),
            cloth: ClothParams {
                sliding_friction: 0.2,
                rolling_resistance: 0.01,
                // ≈10 rad/s² spin deceleration, mid-range of the 5–15 reported.
                spinning_friction: 0.0117,
            },
            ball_ball: BallBallParams {
                restitution: 0.95,
                friction: 0.06,
            },
            cushion: CushionParams {
                restitution: 0.75,
                friction: 0.2,
            },
            cue: CueParams { max_speed: 8.0 },
            gravity: 9.80665,
        }
    }
}

impl Default for PhysicsProfile {
    fn default() -> Self {
        Self::baseline()
    }
}
