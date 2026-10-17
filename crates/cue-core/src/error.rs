use thiserror::Error;

/// Errors produced by the domain model.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CoreError {
    /// A ball number outside `0..=15` was requested.
    #[error("ball number {0} is outside the valid range 0..=15")]
    InvalidBallNumber(u8),

    /// A table specification failed validation.
    #[error("invalid table specification: {0}")]
    InvalidTableSpec(&'static str),
}
