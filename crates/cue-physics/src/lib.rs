//! # cue-physics
//!
//! A purpose-built, event-driven billiards solver. It does not use a general
//! rigid-body engine: pool balls are spheres on a plane, and the phenomena
//! that make billiards feel right (sliding that turns into rolling, cloth
//! resistance, side spin, cushion rebound) have well-studied closed-form
//! models that a generic contact solver only approximates.
//!
//! Between events each ball follows an exact trajectory (see `motion`).
//! Events — collisions, cushion contacts, regime changes — are found as the
//! roots of low-degree polynomials (see `poly`) and resolved one at a time in
//! a fixed order. The result does not depend on the frame rate, and the same
//! inputs always produce the same shot.
//!
//! See `docs/adr/0002-custom-billiards-solver.md` for the rationale.
//!
//! The crate depends only on `cue-core`. It never touches Bevy, the GPU or
//! the browser, so every physical behaviour is tested natively.

#![warn(missing_docs)]

mod geometry;
mod impact;
mod motion;
mod poly;
mod profile;
mod state;
mod strike;
mod world;

pub use geometry::{Cushion, Pocket, Rail, Segment, TableGeometry, Vertex};
pub use profile::{
    BallBallParams, ClothParams, CueParams, CushionParams, PhysicsProfile, SpeedFriction,
};
pub use state::{BallState, Motion};
pub use strike::{MAX_TIP_OFFSET, Strike};
pub use world::{SimError, SimEvent, SimEventKind, World};
