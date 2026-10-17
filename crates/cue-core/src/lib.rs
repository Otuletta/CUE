//! # cue-core
//!
//! The billiards domain model for CUE. This crate is deliberately free of any
//! engine, GPU, audio or browser dependency so that everything in it can be
//! tested natively and reasoned about in isolation.
//!
//! ## Units and coordinates
//!
//! All quantities are SI: metres, kilograms, seconds.
//!
//! The world origin is the centre of the playing surface. `+X` runs along the
//! long axis towards the **foot rail** (where the rack sits), `Z` runs across
//! the table and `+Y` points up. Points *on the table plane* are expressed as
//! [`glam::DVec2`] where `.x` is world `X` and `.y` is world `Z`.

#![warn(missing_docs)]

mod ball;
mod error;
mod pocket;
mod rack;
mod report;
mod rng;
mod table;

pub use ball::{BallGroup, BallId};
pub use error::CoreError;
pub use pocket::{PocketId, PocketSpec};
pub use rack::{Layout, RACK_GAP, eight_ball_rack};
pub use report::{SIMULTANEOUS, ShotEvent, ShotEventKind, ShotReport};
pub use rng::SplitMix64;
pub use table::{BallSpec, TableSpec};

/// Re-exported so downstream crates use the exact same math types.
pub use glam;
