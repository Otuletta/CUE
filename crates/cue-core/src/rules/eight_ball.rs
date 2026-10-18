//! Eight-ball, after WPA *Rules of Play* (effective 2025-09-15), section 4.
//!
//! The engine is a state machine. [`Game::evaluate`] turns the state, the
//! shooter's [`Call`] and the shot's [`ShotReport`] into a [`Ruling`], without
//! changing anything; [`Game::apply`] commits it. Situations where a player
//! must choose (an illegal break, the eight on the break) become a
//! [`Decision`], resolved with [`Game::decide`].
//!
//! Rule numbers in comments refer to the WPA document. Rules that cannot
//! happen in digital play (feet on the floor, touching balls, push shots…)
//! are listed in `docs/rules-eight-ball.md` rather than pretended.

use std::collections::BTreeSet;

use glam::DVec2;

use super::{Group, Ruleset, Seat};
use crate::{BallId, Layout, PocketId, SIMULTANEOUS, ShotReport, TableSpec};

/// Where the incoming player may place the cue ball.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InHand {
    /// Not in hand: play it where it lies.
    #[default]
    No,
    /// Anywhere on the playing surface (WPA 1.6, 4.9).
    Anywhere,
    /// Only above (behind) the head string, as for the break (WPA 4.3).
    AboveHeadString,
}

impl InHand {
    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::No => "no",
            Self::Anywhere => "anywhere",
            Self::AboveHeadString => "above-head-string",
        }
    }
}

/// What the shooter declares before the shot (WPA 1.7, 4.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Call {
    /// Nothing called: the break, or a casual-rules shot not on the eight.
    #[default]
    None,
    /// The intended ball and pocket.
    Ball {
        /// The called ball.
        ball: BallId,
        /// The called pocket.
        pocket: PocketId,
    },
    /// A safety: play passes after the shot whatever drops (WPA 2.16).
    Safety,
}

/// A standard foul (WPA 3, as listed for 8-Ball in 4.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foul {
    /// 3.1 The cue ball was pocketed.
    CueBallPocketed,
    /// 3.3 The cue ball touched no object ball and nothing was pocketed.
    NoContact,
    /// 3.2 The first ball contacted was not a legal target.
    WrongBallFirst {
        /// The ball contacted first.
        hit: BallId,
    },
    /// 3.3 After the first contact, no ball was pocketed or driven to a rail.
    NoRailAfterContact,
    /// 3.10 The cue ball was played from on or below the head string when it
    /// had to be above it.
    BadCueBallPlacement,
    /// 3.11 From above the head string, the first ball contacted was also
    /// above it and the cue ball had not crossed the string first.
    BadPlayFromAboveHeadString,
}

impl Foul {
    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::CueBallPocketed => "cue-ball-pocketed",
            Self::NoContact => "no-contact",
            Self::WrongBallFirst { .. } => "wrong-ball-first",
            Self::NoRailAfterContact => "no-rail-after-contact",
            Self::BadCueBallPlacement => "bad-cue-ball-placement",
            Self::BadPlayFromAboveHeadString => "bad-play-from-above-head-string",
        }
    }
}

/// Why a rack was lost (WPA 4.8). These are the serious fouls of 4.10.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossReason {
    /// 4.8 (a) The eight was pocketed on a foul.
    EightWithFoul,
    /// 4.8 (b) The eight was pocketed before the shooter's group was cleared.
    EightBeforeGroupCleared,
    /// 4.8 (c) The eight went into a pocket that was not called.
    EightInUncalledPocket,
}

impl LossReason {
    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::EightWithFoul => "eight-with-foul",
            Self::EightBeforeGroupCleared => "eight-before-group-cleared",
            Self::EightInUncalledPocket => "eight-in-uncalled-pocket",
        }
    }
}

/// How a rack ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Who won the rack.
    pub winner: Seat,
    /// `None` when the winner pocketed the eight legally; otherwise how the
    /// loser lost it.
    pub loss: Option<LossReason>,
}

/// A situation in which a player must choose how play continues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Situation {
    /// 4.3 (d) No ball pocketed and fewer than four object balls driven to a
    /// rail.
    IllegalBreak,
    /// 4.3 (e) The eight dropped on a legal break.
    EightOnLegalBreak,
    /// 4.3 (f) The eight dropped on a break that was also a foul.
    EightOnFoulBreak,
    /// 4.3 (h) Another foul on the break, with the cue ball still on the
    /// table.
    FoulOnBreak,
}

impl Situation {
    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::IllegalBreak => "illegal-break",
            Self::EightOnLegalBreak => "eight-on-legal-break",
            Self::EightOnFoulBreak => "eight-on-foul-break",
            Self::FoulOnBreak => "foul-on-break",
        }
    }
}

/// An option offered in a [`Decision`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Play the table as it lies.
    AcceptTable,
    /// Re-rack, and the chooser breaks.
    RerackAndBreak,
    /// Re-rack, and the player who broke breaks again.
    RerackOffenderBreaks,
    /// Spot the eight and carry on as things are.
    SpotEightAndContinue,
    /// Re-rack and break again.
    Rebreak,
    /// Spot the eight and shoot with the cue ball in hand above the head
    /// string.
    SpotEightCueInHand,
    /// Take the cue ball in hand above the head string.
    CueInHand,
}

impl Choice {
    /// Stable name used by the public API.
    pub const fn name(self) -> &'static str {
        match self {
            Self::AcceptTable => "accept-table",
            Self::RerackAndBreak => "rerack-and-break",
            Self::RerackOffenderBreaks => "rerack-offender-breaks",
            Self::SpotEightAndContinue => "spot-eight-and-continue",
            Self::Rebreak => "rebreak",
            Self::SpotEightCueInHand => "spot-eight-cue-in-hand",
            Self::CueInHand => "cue-in-hand",
        }
    }

    /// Parses a name from [`Choice::name`].
    pub fn parse(name: &str) -> Option<Self> {
        [
            Self::AcceptTable,
            Self::RerackAndBreak,
            Self::RerackOffenderBreaks,
            Self::SpotEightAndContinue,
            Self::Rebreak,
            Self::SpotEightCueInHand,
            Self::CueInHand,
        ]
        .into_iter()
        .find(|c| c.name() == name)
    }
}

/// A choice a player has to make before play continues.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    /// Who chooses.
    pub chooser: Seat,
    /// Why.
    pub situation: Situation,
    /// The options, in the order the rules list them.
    pub options: Vec<Choice>,
    /// Whether the cue ball left the table on the shot that caused this.
    pub cue_ball_off: bool,
}

/// What happens after a shot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    /// The shooter shoots again.
    Continue,
    /// The opponent comes to the table.
    Pass {
        /// Where the cue ball may be placed.
        in_hand: InHand,
    },
    /// Someone must choose first.
    Decide(Decision),
    /// The rack is over.
    Over(Outcome),
}

/// The referee's verdict on one shot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ruling {
    /// Who took the shot.
    pub shooter: Seat,
    /// Standard fouls detected, in the order checked. Only the most serious
    /// consequence is enforced (WPA 3).
    pub fouls: Vec<Foul>,
    /// Object balls pocketed, in order.
    pub pocketed: Vec<BallId>,
    /// Groups assigned by this shot: `(player, their group)`.
    pub assigned: Option<(Seat, Group)>,
    /// What happens next.
    pub next: Next,
}

/// Physical follow-up the game must carry out after a [`Game::decide`] or a
/// stalemate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Nothing to do on the table.
    None,
    /// Rack all fifteen balls again; the cue ball is in hand above the head
    /// string.
    Rerack,
    /// Put the eight back on the table with [`super::spot_position`].
    SpotEight,
}

/// Why a call does not fit the next shot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CallError {
    /// Nothing is called on the break (WPA 4.3).
    #[error("nothing is called on the break")]
    NoCallOnBreak,
    /// The called ball has already been pocketed.
    #[error("that ball is not on the table")]
    BallNotOnTable,
    /// The called ball belongs to the opponent's group (WPA 4.5).
    #[error("the called ball must be from your group")]
    NotYourGroup,
    /// The shooter's group is cleared, so only the eight can be called.
    #[error("your group is cleared: call the eight")]
    CallTheEight,
    /// The eight is not a legal target on an open table (WPA 4.4).
    #[error("the eight cannot be called on an open table")]
    EightOnOpenTable,
    /// These rules require a call on every shot but the break (WPA 4.6).
    #[error("every shot must be called")]
    CallRequired,
    /// Even casual rules call the eight's pocket.
    #[error("call the eight's pocket")]
    CallTheEightsPocket,
}

impl CallError {
    /// Stable name used in public events.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoCallOnBreak => "no-call-on-break",
            Self::BallNotOnTable => "ball-not-on-table",
            Self::NotYourGroup => "not-your-group",
            Self::CallTheEight => "call-the-eight",
            Self::EightOnOpenTable => "eight-on-open-table",
            Self::CallRequired => "call-required",
            Self::CallTheEightsPocket => "call-the-eights-pocket",
        }
    }
}

/// Why the rules refused an input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RulesError {
    /// The game is waiting for a decision, or is over.
    #[error("the game is not expecting a shot")]
    NotExpectingShot,
    /// The call does not fit this shot under these rules.
    #[error("invalid call: {0}")]
    InvalidCall(CallError),
    /// No decision is pending, or the choice is not one of its options.
    #[error("that choice is not available")]
    InvalidChoice,
}

impl RulesError {
    /// Stable name used in public events: the call error's name for an
    /// invalid call.
    pub const fn name(self) -> &'static str {
        match self {
            Self::NotExpectingShot => "not-expecting-shot",
            Self::InvalidCall(e) => e.name(),
            Self::InvalidChoice => "choice-unavailable",
        }
    }
}

/// Where the game stands between shots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The next shot is the break.
    Break,
    /// Normal play.
    Play,
    /// Waiting for a player's choice.
    Decision(Decision),
    /// The rack is over.
    Over(Outcome),
}

impl Stage {
    /// Stable name used by the public API.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Break => "break",
            Self::Play => "play",
            Self::Decision(_) => "decision",
            Self::Over(_) => "over",
        }
    }
}

/// What the rules need to know about a shot.
#[derive(Clone, Copy, Debug)]
pub struct Shot<'a> {
    /// What the shooter declared.
    pub call: Call,
    /// What happened.
    pub report: &'a ShotReport,
    /// Ball positions when the shot began.
    pub before: &'a Layout,
    /// The table, for the head string.
    pub table: &'a TableSpec,
}

/// One rack of eight-ball between two players.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    ruleset: Ruleset,
    breaker: Seat,
    shooter: Seat,
    /// The group of player A, once assigned; B has the other.
    a_group: Option<Group>,
    on_table: BTreeSet<BallId>,
    in_hand: InHand,
    stage: Stage,
}

impl Game {
    /// A fresh rack, with `breaker` to break.
    pub fn new(ruleset: Ruleset, breaker: Seat) -> Self {
        Self {
            ruleset,
            breaker,
            shooter: breaker,
            a_group: None,
            on_table: BallId::objects().collect(),
            in_hand: InHand::AboveHeadString,
            stage: Stage::Break,
        }
    }

    /// The rules in force.
    pub fn ruleset(&self) -> Ruleset {
        self.ruleset
    }

    /// Who broke this rack.
    pub fn breaker(&self) -> Seat {
        self.breaker
    }

    /// Whose turn it is (or who must decide, during a decision).
    pub fn shooter(&self) -> Seat {
        match &self.stage {
            Stage::Decision(d) => d.chooser,
            _ => self.shooter,
        }
    }

    /// Where the game stands.
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// Where the cue ball may be placed before the next shot.
    pub fn in_hand(&self) -> InHand {
        self.in_hand
    }

    /// A player's group, once assigned.
    pub fn group(&self, seat: Seat) -> Option<Group> {
        self.a_group
            .map(|g| if seat == Seat::A { g } else { g.other() })
    }

    /// Whether the table is still open (WPA 4.4).
    pub fn is_open(&self) -> bool {
        self.a_group.is_none()
    }

    /// Object balls still on the table.
    pub fn on_table(&self) -> impl Iterator<Item = BallId> + '_ {
        self.on_table.iter().copied()
    }

    /// Balls of `group` still on the table.
    pub fn remaining(&self, group: Group) -> usize {
        self.on_table
            .iter()
            .filter(|b| Group::of(**b) == Some(group))
            .count()
    }

    fn group_cleared(&self, group: Group) -> bool {
        self.remaining(group) == 0
    }

    /// Whether `seat` is shooting at the eight: their group is cleared, or
    /// the table is open and a whole group is already off the table, which
    /// they may claim (WPA 4.4).
    pub fn on_the_eight(&self, seat: Seat) -> bool {
        match self.group(seat) {
            Some(g) => self.group_cleared(g),
            None => self.group_cleared(Group::Solids) || self.group_cleared(Group::Stripes),
        }
    }

    /// The balls the current shooter may contact first (WPA 3.2, 4.4).
    pub fn legal_targets(&self) -> Vec<BallId> {
        if self.stage == Stage::Break {
            return self.on_table.iter().copied().collect();
        }
        let eight_allowed = self.on_the_eight(self.shooter);
        match self.group(self.shooter) {
            Some(g) if !self.group_cleared(g) => self
                .on_table
                .iter()
                .copied()
                .filter(|b| Group::of(*b) == Some(g))
                .collect(),
            Some(_) => vec![BallId::EIGHT],
            None => self
                .on_table
                .iter()
                .copied()
                .filter(|b| *b != BallId::EIGHT || eight_allowed)
                .collect(),
        }
    }

    /// Whether `call` is acceptable for the next shot.
    pub fn validate_call(&self, call: Call) -> Result<(), RulesError> {
        self.check_call(call).map_err(|e| match e {
            Some(e) => RulesError::InvalidCall(e),
            None => RulesError::NotExpectingShot,
        })
    }

    /// [`Game::validate_call`], with `None` for "not expecting a shot".
    fn check_call(&self, call: Call) -> Result<(), Option<CallError>> {
        match (&self.stage, call) {
            (Stage::Break, Call::None) => Ok(()),
            (Stage::Break, _) => Err(Some(CallError::NoCallOnBreak)),
            (Stage::Play, Call::Safety) => Ok(()),
            (Stage::Play, Call::Ball { ball, .. }) => {
                if !self.on_table.contains(&ball) {
                    return Err(Some(CallError::BallNotOnTable));
                }
                let on_eight = self.on_the_eight(self.shooter);
                match self.group(self.shooter) {
                    Some(g) if !self.group_cleared(g) && Group::of(ball) != Some(g) => {
                        Err(Some(CallError::NotYourGroup))
                    }
                    Some(_) if on_eight && ball != BallId::EIGHT => {
                        Err(Some(CallError::CallTheEight))
                    }
                    None if ball == BallId::EIGHT && !on_eight => {
                        Err(Some(CallError::EightOnOpenTable))
                    }
                    _ => Ok(()),
                }
            }
            (Stage::Play, Call::None) => {
                if self.ruleset.calls_every_shot() {
                    Err(Some(CallError::CallRequired))
                } else if self.on_the_eight(self.shooter) && self.group(self.shooter).is_some() {
                    Err(Some(CallError::CallTheEightsPocket))
                } else {
                    Ok(())
                }
            }
            _ => Err(None),
        }
    }

    /// Judges a shot without changing the game.
    pub fn evaluate(&self, shot: &Shot<'_>) -> Result<Ruling, RulesError> {
        self.validate_call(shot.call)?;
        match self.stage {
            Stage::Break => Ok(self.evaluate_break(shot)),
            Stage::Play => Ok(self.evaluate_play(shot)),
            _ => Err(RulesError::NotExpectingShot),
        }
    }

    /// Standard fouls that apply to any shot (WPA 3.1–3.3, 3.10, 3.11).
    fn standard_fouls(&self, shot: &Shot<'_>, check_targets: bool) -> Vec<Foul> {
        let report = shot.report;
        let mut fouls = Vec::new();
        let head = shot.table.head_string_x();
        let above = |b: BallId| shot.before.get(b).is_some_and(|p| p.x < head);

        if self.in_hand == InHand::AboveHeadString && !above(BallId::CUE) {
            fouls.push(Foul::BadCueBallPlacement);
        }
        if report.cue_ball_pocketed() {
            fouls.push(Foul::CueBallPocketed);
        }
        let first = report.first_contacts();
        let anything_pocketed = report.pocketed().next().is_some();
        match first.first() {
            None if !anything_pocketed => fouls.push(Foul::NoContact),
            None => {}
            Some(&hit) => {
                let legal = self.legal_targets();
                // 3.2: on a simultaneous hit, assume the legal ball came first.
                if check_targets && !first.iter().any(|b| legal.contains(b)) {
                    fouls.push(Foul::WrongBallFirst { hit });
                }
                // 3.11: from the kitchen, a first ball also in the kitchen
                // needs the cue ball to have crossed the string first.
                if self.in_hand == InHand::AboveHeadString {
                    let t0 = report.first_contact_time().unwrap_or(0.0);
                    if above(hit) && !report.cue_crossed_head_string_before(t0) {
                        fouls.push(Foul::BadPlayFromAboveHeadString);
                    }
                }
                // 3.3, with 2.7: a pocketed ball counts as driven to a rail,
                // a frozen ball pushed into its own rail does not, and a rail
                // touched together with the first contact counts as after it.
                if !anything_pocketed {
                    let after = report.events.iter().any(|e| {
                        e.time >= t0_of(report) - SIMULTANEOUS && e.kind.drives_to_a_rail()
                    });
                    if !after {
                        fouls.push(Foul::NoRailAfterContact);
                    }
                }
            }
        }
        fouls
    }

    fn evaluate_break(&self, shot: &Shot<'_>) -> Ruling {
        let report = shot.report;
        let fouls = self.standard_fouls(shot, false);
        let pocketed = object_balls_pocketed(report);
        let eight = pocketed.contains(&BallId::EIGHT);
        let foul = !fouls.is_empty();
        let opponent = self.shooter.other();
        let cue_ball_off = report.cue_ball_pocketed();
        let decide = |chooser, situation, options: &[Choice]| {
            Next::Decide(Decision {
                chooser,
                situation,
                options: options.to_vec(),
                cue_ball_off,
            })
        };

        // 4.3 (d): a ball pocketed, or four object balls driven to a rail.
        let legal = !pocketed.is_empty() || report.object_balls_driven_to_a_rail().len() >= 4;
        let next = if !legal && foul && !cue_ball_off {
            // An illegal break that is also a foul (missing the rack, 3.3):
            // the options of 4.3 (d) and (h) together. CUE's adjudication;
            // WPA does not address the combination.
            decide(
                opponent,
                Situation::IllegalBreak,
                &[
                    Choice::AcceptTable,
                    Choice::CueInHand,
                    Choice::RerackAndBreak,
                    Choice::RerackOffenderBreaks,
                ],
            )
        } else if !legal {
            decide(
                opponent,
                Situation::IllegalBreak,
                &[
                    Choice::AcceptTable,
                    Choice::RerackAndBreak,
                    Choice::RerackOffenderBreaks,
                ],
            )
        } else if eight && !foul {
            decide(
                self.shooter,
                Situation::EightOnLegalBreak,
                &[Choice::SpotEightAndContinue, Choice::Rebreak],
            )
        } else if eight {
            decide(
                opponent,
                Situation::EightOnFoulBreak,
                &[Choice::SpotEightCueInHand, Choice::Rebreak],
            )
        } else if foul && cue_ball_off {
            // 4.3 (h) with the cue ball gone: the only meaningful option.
            Next::Pass {
                in_hand: InHand::AboveHeadString,
            }
        } else if foul {
            decide(
                opponent,
                Situation::FoulOnBreak,
                &[Choice::AcceptTable, Choice::CueInHand],
            )
        } else if pocketed.is_empty() {
            Next::Pass {
                in_hand: InHand::No,
            }
        } else {
            // 4.3 (c): the breaker continues; the table stays open.
            Next::Continue
        };
        Ruling {
            shooter: self.shooter,
            fouls,
            pocketed,
            assigned: None,
            next,
        }
    }

    fn evaluate_play(&self, shot: &Shot<'_>) -> Ruling {
        let report = shot.report;
        let shooter = self.shooter;
        let opponent = shooter.other();
        let fouls = self.standard_fouls(shot, true);
        let foul = !fouls.is_empty();
        let pocketed = object_balls_pocketed(report);
        let ruling = |next, assigned| Ruling {
            shooter,
            fouls: fouls.clone(),
            pocketed: pocketed.clone(),
            assigned,
            next,
        };
        let lost = |reason| {
            Next::Over(Outcome {
                winner: opponent,
                loss: Some(reason),
            })
        };

        // 4.8: the eight decides the rack, one way or the other.
        if let Some(eight_pocket) = pocket_of(report, BallId::EIGHT) {
            let next = if foul {
                lost(LossReason::EightWithFoul)
            } else if !self.on_the_eight(shooter) {
                lost(LossReason::EightBeforeGroupCleared)
            } else if shot.call
                == (Call::Ball {
                    ball: BallId::EIGHT,
                    pocket: eight_pocket,
                })
            {
                Next::Over(Outcome {
                    winner: shooter,
                    loss: None,
                })
            } else {
                lost(LossReason::EightInUncalledPocket)
            };
            return ruling(next, None);
        }

        // 4.9: a standard foul passes the table with ball in hand anywhere.
        // A foul never assigns groups: nothing was pocketed legally.
        if foul {
            return ruling(
                Next::Pass {
                    in_hand: InHand::Anywhere,
                },
                None,
            );
        }
        if shot.call == Call::Safety {
            return ruling(
                Next::Pass {
                    in_hand: InHand::No,
                },
                None,
            );
        }

        match (self.group(shooter), self.ruleset.calls_every_shot()) {
            // 4.4: on an open table the called ball, legally pocketed, decides.
            (None, true) => match shot.call {
                Call::Ball { ball, pocket } if pocket_of(report, ball) == Some(pocket) => {
                    let group = Group::of(ball).expect("the eight was handled above");
                    ruling(Next::Continue, Some((shooter, group)))
                }
                _ => ruling(
                    Next::Pass {
                        in_hand: InHand::No,
                    },
                    None,
                ),
            },
            // Casual: the first ball pocketed decides.
            (None, false) => match pocketed.iter().find_map(|b| Group::of(*b)) {
                Some(group) => ruling(Next::Continue, Some((shooter, group))),
                None => ruling(
                    Next::Pass {
                        in_hand: InHand::No,
                    },
                    None,
                ),
            },
            // 4.5, 4.6: the called ball in the called pocket keeps you there.
            (Some(_), true) => match shot.call {
                Call::Ball { ball, pocket } if pocket_of(report, ball) == Some(pocket) => {
                    ruling(Next::Continue, None)
                }
                _ => ruling(
                    Next::Pass {
                        in_hand: InHand::No,
                    },
                    None,
                ),
            },
            // Casual: any ball of your group keeps you there.
            (Some(group), false) => {
                let own = pocketed.iter().any(|b| Group::of(*b) == Some(group));
                let next = if own {
                    Next::Continue
                } else {
                    Next::Pass {
                        in_hand: InHand::No,
                    }
                };
                ruling(next, None)
            }
        }
    }

    /// Commits a ruling from [`Game::evaluate`].
    pub fn apply(&mut self, ruling: &Ruling) {
        for ball in &ruling.pocketed {
            self.on_table.remove(ball);
        }
        if let Some((seat, group)) = ruling.assigned {
            self.a_group = Some(if seat == Seat::A {
                group
            } else {
                group.other()
            });
        }
        match &ruling.next {
            Next::Continue => {
                self.in_hand = InHand::No;
                self.stage = Stage::Play;
            }
            Next::Pass { in_hand } => {
                self.shooter = ruling.shooter.other();
                self.in_hand = *in_hand;
                self.stage = Stage::Play;
            }
            Next::Decide(decision) => self.stage = Stage::Decision(decision.clone()),
            Next::Over(outcome) => self.stage = Stage::Over(*outcome),
        }
    }

    /// Resolves a pending [`Decision`]. Returns what the game must do on the
    /// table.
    pub fn decide(&mut self, choice: Choice) -> Result<Effect, RulesError> {
        let Stage::Decision(decision) = &self.stage else {
            return Err(RulesError::InvalidChoice);
        };
        if !decision.options.contains(&choice) {
            return Err(RulesError::InvalidChoice);
        }
        let chooser = decision.chooser;
        let cue_ball_off = decision.cue_ball_off;
        let breaker = self.breaker;
        let effect = match choice {
            Choice::AcceptTable => {
                self.shooter = chooser;
                // Accepting a table without a cue ball: place it as for the
                // break (CUE's adjudication of 4.3 (d) combined with a scratch).
                self.in_hand = if cue_ball_off {
                    InHand::AboveHeadString
                } else {
                    InHand::No
                };
                self.stage = Stage::Play;
                Effect::None
            }
            Choice::CueInHand => {
                self.shooter = chooser;
                self.in_hand = InHand::AboveHeadString;
                self.stage = Stage::Play;
                Effect::None
            }
            Choice::RerackAndBreak => self.rerack(chooser),
            Choice::RerackOffenderBreaks => self.rerack(breaker),
            Choice::Rebreak => self.rerack(chooser),
            Choice::SpotEightAndContinue => {
                self.on_table.insert(BallId::EIGHT);
                self.shooter = chooser;
                self.in_hand = InHand::No;
                self.stage = Stage::Play;
                Effect::SpotEight
            }
            Choice::SpotEightCueInHand => {
                self.on_table.insert(BallId::EIGHT);
                self.shooter = chooser;
                self.in_hand = InHand::AboveHeadString;
                self.stage = Stage::Play;
                Effect::SpotEight
            }
        };
        Ok(effect)
    }

    fn rerack(&mut self, breaker: Seat) -> Effect {
        *self = Self::new(self.ruleset, breaker);
        Effect::Rerack
    }

    /// Both players agree no progress is being made (WPA 1.13): the original
    /// breaker breaks again (4.11). Digital play has no referee, so this needs
    /// the players' agreement rather than an observation of no progress.
    pub fn stalemate(&mut self) -> Effect {
        let breaker = self.breaker;
        self.rerack(breaker)
    }

    /// WPA 1.6: with the cue ball in hand above the head string and every
    /// legal target above it too, the shooter may ask for the legal ball
    /// nearest the head string to be spotted. Returns that ball, if the
    /// request is allowed. Equidistant balls resolve to the lowest number;
    /// the rules let the shooter choose, which a host may offer.
    pub fn spot_request(&self, before: &Layout, table: &TableSpec) -> Option<BallId> {
        if self.in_hand != InHand::AboveHeadString || self.stage != Stage::Play {
            return None;
        }
        let head = table.head_string_x();
        let targets: Vec<(BallId, DVec2)> = self
            .legal_targets()
            .into_iter()
            .filter_map(|b| before.get(b).map(|p| (b, p)))
            .collect();
        if targets.is_empty() || targets.iter().any(|(_, p)| p.x >= head) {
            return None;
        }
        targets
            .into_iter()
            .min_by(|a, b| (head - a.1.x).total_cmp(&(head - b.1.x)))
            .map(|(b, _)| b)
    }
}

fn t0_of(report: &ShotReport) -> f64 {
    report.first_contact_time().unwrap_or(f64::INFINITY)
}

fn object_balls_pocketed(report: &ShotReport) -> Vec<BallId> {
    report
        .pocketed()
        .map(|(b, _)| b)
        .filter(|b| !b.is_cue())
        .collect()
}

fn pocket_of(report: &ShotReport, ball: BallId) -> Option<PocketId> {
    report.pocketed().find(|(b, _)| *b == ball).map(|(_, p)| p)
}
