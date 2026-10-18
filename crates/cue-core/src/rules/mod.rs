//! Game rules: pure functions from a game state, a call and a
//! [`ShotReport`](crate::ShotReport) to a ruling (spec §9.12).
//!
//! Nothing here knows about the solver, the engine, the camera or sound. The
//! rules see exactly what a referee would need, and every decision they make
//! can be tested by constructing a report by hand.

pub mod eight_ball;
mod spot;

pub use spot::spot_position;

use crate::{BallGroup, BallId};

/// One of the two players at the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Seat {
    /// The first player.
    A,
    /// The second player.
    B,
}

impl Seat {
    /// The other player.
    pub const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }

    /// `0` for A, `1` for B.
    pub const fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    /// Stable name used by the public API: `"a"` or `"b"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }

    /// Parses a name from [`Seat::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [Self::A, Self::B].into_iter().find(|s| s.name() == name)
    }
}

/// One of the two groups of seven object balls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    /// Balls 1–7.
    Solids,
    /// Balls 9–15.
    Stripes,
}

impl Group {
    /// The group a ball belongs to; `None` for the cue ball and the eight.
    pub const fn of(ball: BallId) -> Option<Self> {
        match ball.group() {
            BallGroup::Solids => Some(Self::Solids),
            BallGroup::Stripes => Some(Self::Stripes),
            BallGroup::Cue | BallGroup::Eight => None,
        }
    }

    /// The other group.
    pub const fn other(self) -> Self {
        match self {
            Self::Solids => Self::Stripes,
            Self::Stripes => Self::Solids,
        }
    }

    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Solids => "solids",
            Self::Stripes => "stripes",
        }
    }
}

/// Which rules a game follows. Rule sets are versioned profiles; they are
/// never mixed silently (spec §9.12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ruleset {
    /// WPA *Rules of Play* (effective 2025-09-15), section 4, as far as it
    /// applies to digital play. Every shot except the break is called.
    /// `docs/rules-eight-ball.md` lists rule by rule what is implemented,
    /// what cannot happen digitally and what is left to the players.
    EightBallWpa,
    /// The same rules with one relaxation: only the eight ball's pocket is
    /// called. Any ball of your group that drops keeps you at the table, and
    /// on an open table the first ball pocketed decides the groups.
    EightBallCasual,
}

impl Ruleset {
    /// Stable name used by the public API.
    pub const fn name(self) -> &'static str {
        match self {
            Self::EightBallWpa => "eight-ball-wpa",
            Self::EightBallCasual => "eight-ball-casual",
        }
    }

    /// Parses a name from [`Ruleset::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [Self::EightBallWpa, Self::EightBallCasual]
            .into_iter()
            .find(|r| r.name() == name)
    }

    /// Whether every shot (not only the eight) must be called.
    pub const fn calls_every_shot(self) -> bool {
        matches!(self, Self::EightBallWpa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for seat in [Seat::A, Seat::B] {
            assert_eq!(Seat::parse(seat.name()), Some(seat));
            assert_eq!(seat.other().other(), seat);
        }
        for ruleset in [Ruleset::EightBallWpa, Ruleset::EightBallCasual] {
            assert_eq!(Ruleset::parse(ruleset.name()), Some(ruleset));
        }
        assert_eq!(Seat::parse("c"), None);
        assert_eq!(Ruleset::parse("nine-ball"), None);
    }
}
