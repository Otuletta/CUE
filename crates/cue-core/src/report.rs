use std::collections::BTreeSet;

use crate::{BallId, PocketId};

/// Contacts closer together than this, in seconds, count as simultaneous.
///
/// The solver resolves events one at a time; two contacts that are physically
/// simultaneous can come out a few nanoseconds apart from rounding alone.
pub const SIMULTANEOUS: f64 = 1e-9;

/// What the rules need to know about one shot, and nothing else.
///
/// Built by the physics from its event log once every ball has stopped, and
/// handed to the rules as an immutable value. Rules never see the solver,
/// the engine or the renderer (spec §9.12).
#[derive(Clone, Debug, PartialEq)]
pub struct ShotReport {
    /// Sequence number of the shot within its world, starting at 1.
    pub shot_id: u32,
    /// Chronological contacts and pocketings. Grazing contacts (pressing
    /// balls, balls resting against a cushion) are already filtered out by
    /// the physics.
    pub events: Vec<ShotEvent>,
}

/// One entry of a [`ShotReport`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotEvent {
    /// Seconds since the cue struck the cue ball.
    pub time: f64,
    /// What happened.
    pub kind: ShotEventKind,
}

/// The kinds of [`ShotEvent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotEventKind {
    /// Two balls touched. `a < b`.
    BallContact {
        /// Lower-numbered ball.
        a: BallId,
        /// Higher-numbered ball.
        b: BallId,
    },
    /// A ball touched a cushion, including the pocket jaws.
    Cushion {
        /// The ball.
        ball: BallId,
    },
    /// A ball dropped into a pocket.
    Pocketed {
        /// The ball.
        ball: BallId,
        /// The pocket.
        pocket: PocketId,
    },
}

impl ShotReport {
    /// Time of the cue ball's first contact with an object ball.
    pub fn first_contact_time(&self) -> Option<f64> {
        self.events.iter().find_map(|e| match e.kind {
            ShotEventKind::BallContact { a, .. } if a.is_cue() => Some(e.time),
            _ => None,
        })
    }

    /// The object ball(s) the cue ball touched first. Usually one; more than
    /// one only when contacts were simultaneous within [`SIMULTANEOUS`]. The
    /// rules decide how to treat that case.
    pub fn first_contacts(&self) -> Vec<BallId> {
        let Some(t0) = self.first_contact_time() else {
            return Vec::new();
        };
        let mut balls: Vec<BallId> = self
            .events
            .iter()
            .take_while(|e| e.time <= t0 + SIMULTANEOUS)
            .filter_map(|e| match e.kind {
                ShotEventKind::BallContact { a, b } if a.is_cue() => Some(b),
                _ => None,
            })
            .collect();
        balls.dedup();
        balls
    }

    /// Balls pocketed during the shot, in order, with their pockets.
    pub fn pocketed(&self) -> impl Iterator<Item = (BallId, PocketId)> + '_ {
        self.events.iter().filter_map(|e| match e.kind {
            ShotEventKind::Pocketed { ball, pocket } => Some((ball, pocket)),
            _ => None,
        })
    }

    /// Whether the cue ball went into a pocket.
    pub fn cue_ball_pocketed(&self) -> bool {
        self.pocketed().any(|(b, _)| b.is_cue())
    }

    /// Whether any ball touched a cushion strictly after the cue ball's first
    /// contact with an object ball. Needed for the "no rail after contact"
    /// foul.
    pub fn cushion_after_first_contact(&self) -> bool {
        let Some(t0) = self.first_contact_time() else {
            return false;
        };
        self.events
            .iter()
            .any(|e| e.time > t0 && matches!(e.kind, ShotEventKind::Cushion { .. }))
    }

    /// Distinct object balls that touched a cushion at any time. Needed for
    /// the legal-break count.
    pub fn object_balls_touching_cushion(&self) -> BTreeSet<BallId> {
        self.events
            .iter()
            .filter_map(|e| match e.kind {
                ShotEventKind::Cushion { ball } if !ball.is_cue() => Some(ball),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ball(n: u8) -> BallId {
        BallId::new(n).unwrap()
    }

    fn at(time: f64, kind: ShotEventKind) -> ShotEvent {
        ShotEvent { time, kind }
    }

    fn contact(a: u8, b: u8) -> ShotEventKind {
        ShotEventKind::BallContact {
            a: ball(a),
            b: ball(b),
        }
    }

    fn cushion(n: u8) -> ShotEventKind {
        ShotEventKind::Cushion { ball: ball(n) }
    }

    #[test]
    fn empty_report_has_no_contacts() {
        let r = ShotReport {
            shot_id: 1,
            events: vec![at(0.4, cushion(0))],
        };
        assert!(r.first_contacts().is_empty());
        assert!(!r.cushion_after_first_contact());
        assert!(!r.cue_ball_pocketed());
    }

    #[test]
    fn first_contact_ignores_earlier_object_ball_contacts_and_cushions() {
        let r = ShotReport {
            shot_id: 1,
            events: vec![
                at(0.10, cushion(0)),
                at(0.20, contact(0, 3)),
                at(0.30, contact(3, 9)),
                at(0.40, contact(0, 9)),
            ],
        };
        assert_eq!(r.first_contacts(), vec![ball(3)]);
        assert_eq!(r.first_contact_time(), Some(0.20));
    }

    #[test]
    fn simultaneous_first_contacts_are_all_reported() {
        let r = ShotReport {
            shot_id: 2,
            events: vec![at(0.25, contact(0, 1)), at(0.25 + 1e-12, contact(0, 8))],
        };
        assert_eq!(r.first_contacts(), vec![ball(1), ball(8)]);
    }

    #[test]
    fn cushion_must_come_after_first_contact() {
        let before_only = ShotReport {
            shot_id: 1,
            events: vec![at(0.1, cushion(0)), at(0.2, contact(0, 5))],
        };
        assert!(!before_only.cushion_after_first_contact());
        let after = ShotReport {
            shot_id: 1,
            events: vec![at(0.2, contact(0, 5)), at(0.6, cushion(5))],
        };
        assert!(after.cushion_after_first_contact());
    }

    #[test]
    fn counts_distinct_object_balls_on_cushions_and_pocketings() {
        let r = ShotReport {
            shot_id: 1,
            events: vec![
                at(0.1, contact(0, 1)),
                at(0.3, cushion(4)),
                at(0.4, cushion(4)),
                at(0.5, cushion(12)),
                at(0.6, cushion(0)),
                at(
                    0.9,
                    ShotEventKind::Pocketed {
                        ball: ball(0),
                        pocket: PocketId::SideNear,
                    },
                ),
            ],
        };
        assert_eq!(
            r.object_balls_touching_cushion()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![ball(4), ball(12)]
        );
        assert!(r.cue_ball_pocketed());
        assert_eq!(r.pocketed().count(), 1);
    }
}
