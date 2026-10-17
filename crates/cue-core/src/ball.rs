use std::fmt;

use crate::CoreError;

/// Stable logical identity of a ball.
///
/// `0` is the cue ball; `1..=15` are the numbered object balls. This is the
/// identifier used in reports, events and the public API. It is never an ECS
/// entity id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BallId(u8);

impl BallId {
    /// The cue ball.
    pub const CUE: Self = Self(0);
    /// The eight ball.
    pub const EIGHT: Self = Self(8);
    /// Highest valid ball number.
    pub const MAX_NUMBER: u8 = 15;

    /// Creates an id from a ball number in `0..=15`.
    pub const fn new(number: u8) -> Result<Self, CoreError> {
        if number <= Self::MAX_NUMBER {
            Ok(Self(number))
        } else {
            Err(CoreError::InvalidBallNumber(number))
        }
    }

    /// The number printed on the ball (`0` for the cue ball).
    pub const fn number(self) -> u8 {
        self.0
    }

    /// Whether this is the cue ball.
    pub const fn is_cue(self) -> bool {
        self.0 == 0
    }

    /// The group this ball belongs to in eight-ball.
    pub const fn group(self) -> BallGroup {
        match self.0 {
            0 => BallGroup::Cue,
            1..=7 => BallGroup::Solids,
            8 => BallGroup::Eight,
            _ => BallGroup::Stripes,
        }
    }

    /// Every ball in a standard set, cue ball first, in ascending order.
    pub fn all() -> impl ExactSizeIterator<Item = Self> + Clone {
        (0..=Self::MAX_NUMBER).map(Self)
    }

    /// The fifteen object balls in ascending order.
    pub fn objects() -> impl ExactSizeIterator<Item = Self> + Clone {
        (1..=Self::MAX_NUMBER).map(Self)
    }
}

impl fmt::Display for BallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_cue() {
            f.write_str("cue ball")
        } else {
            write!(f, "{}-ball", self.0)
        }
    }
}

/// Eight-ball grouping of a ball.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BallGroup {
    /// The cue ball.
    Cue,
    /// Balls `1..=7`.
    Solids,
    /// The eight ball.
    Eight,
    /// Balls `9..=15`.
    Stripes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_range_is_zero_to_fifteen() {
        assert!(BallId::new(0).is_ok());
        assert!(BallId::new(15).is_ok());
        assert_eq!(BallId::new(16), Err(CoreError::InvalidBallNumber(16)));
    }

    #[test]
    fn groups_partition_the_set() {
        let count = |g| BallId::all().filter(|b| b.group() == g).count();
        assert_eq!(count(BallGroup::Cue), 1);
        assert_eq!(count(BallGroup::Solids), 7);
        assert_eq!(count(BallGroup::Eight), 1);
        assert_eq!(count(BallGroup::Stripes), 7);
    }

    #[test]
    fn iterators_have_expected_lengths() {
        assert_eq!(BallId::all().len(), 16);
        assert_eq!(BallId::objects().len(), 15);
        assert!(BallId::objects().all(|b| !b.is_cue()));
    }
}
