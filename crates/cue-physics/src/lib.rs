//! # cue-physics
//!
//! A purpose-built billiards solver. It does not use a general rigid-body
//! engine: pool balls are spheres on a plane, and the phenomena that make
//! billiards feel right (sliding that turns into rolling, cloth resistance,
//! side spin, cushion rebound at nose height) have well-studied closed-form
//! models that a generic contact solver only approximates.
//!
//! See `docs/adr/0002-custom-billiards-solver.md` for the rationale and
//! `docs/physics-model.md` (Phase 2) for the equations and their sources.
//!
//! The crate depends only on `cue-core`. It never touches Bevy, the GPU or the
//! browser, so every physical behaviour can be tested natively.

#![warn(missing_docs)]

mod profile;
mod state;

pub use profile::{BallBallParams, ClothParams, CushionParams, PhysicsProfile};
pub use state::{BallState, Motion};
