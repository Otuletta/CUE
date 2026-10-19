//! Local eight-ball for two players on one device (spec §8.2).
//!
//! The session owns the rules state and turns rulings into what happens on
//! the table: who shoots, where the cue ball may go, re-racks and spotted
//! balls. It never judges a shot itself; `cue_core::rules` does.

use crate::{
    settings::GameCommand,
    sim::{GameEvent, HandRegion, ShotFinished, Simulation},
};
use bevy::prelude::*;
use cue_core::{
    BallId, Layout, PocketId, ShotReport,
    glam::DVec2,
    rules::{
        Group, Ruleset, Seat,
        eight_ball::{Call, Choice, Effect, Game, InHand, Next, RulesError, Ruling, Shot, Stage},
        spot_position,
    },
};

/// Longest player name accepted, in characters.
const MAX_NAME: usize = 24;

/// How a new game is set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSetup {
    /// The rules to play.
    pub ruleset: Ruleset,
    /// Who breaks the first rack; `None` picks at random.
    pub breaker: Option<Seat>,
    /// Racks needed to win the match (1 for a single rack).
    pub race_to: u8,
    /// Local, temporary display names (spec §8.2). Never stored.
    pub names: [String; 2],
}

/// Which game is on the table.
#[derive(Resource, Debug, Default)]
pub enum Mode {
    /// Practice: no turns, no fouls.
    #[default]
    FreePlay,
    /// Two players, eight-ball rules.
    EightBall(Box<EightBall>),
}

impl Mode {
    /// The eight-ball session, if one is running.
    pub fn eight_ball(&self) -> Option<&EightBall> {
        match self {
            Self::EightBall(session) => Some(session),
            Self::FreePlay => None,
        }
    }
}

/// A shot in flight: what was called and where the balls were.
#[derive(Debug, Clone)]
struct InFlight {
    call: Call,
    before: Layout,
}

/// A match of eight-ball.
#[derive(Debug, Clone)]
pub struct EightBall {
    /// The rules state of the current rack.
    pub game: Game,
    /// Display names, A then B.
    pub names: [String; 2],
    /// Racks needed to win.
    pub race_to: u8,
    /// Racks won, A then B.
    pub score: [u8; 2],
    /// What the shooter is calling for the next shot.
    pub call: Call,
    /// Whether the player chose the call; otherwise it follows the aim.
    pub manual_call: bool,
    /// The most recent ruling, for the interface.
    pub last: Option<Ruling>,
    /// What was called for the shot behind [`EightBall::last`].
    pub last_call: Call,
    /// Racks played, including the current one.
    pub rack: u32,
    in_flight: Option<InFlight>,
}

impl EightBall {
    pub(crate) fn new(setup: &GameSetup, coin: bool) -> Self {
        let breaker = setup
            .breaker
            .unwrap_or(if coin { Seat::A } else { Seat::B });
        Self {
            game: Game::new(setup.ruleset, breaker),
            names: setup.names.clone().map(|n| clean_name(&n)),
            race_to: setup.race_to.clamp(1, 15),
            score: [0, 0],
            call: Call::None,
            manual_call: false,
            last: None,
            last_call: Call::None,
            rack: 1,
            in_flight: None,
        }
    }

    /// Whether the match is decided.
    pub fn match_over(&self) -> bool {
        self.score.iter().any(|s| *s >= self.race_to)
    }

    /// Display name of a player.
    pub fn name(&self, seat: Seat) -> &str {
        &self.names[seat.index()]
    }

    /// A player's group and how many of its balls are still on the table,
    /// once groups are assigned.
    pub fn remaining(&self, seat: Seat) -> Option<(Group, usize)> {
        self.game.group(seat).map(|g| (g, self.game.remaining(g)))
    }

    /// Whether the table is in play: false while a decision is pending and
    /// once the rack is over, when there is nothing to aim at.
    pub fn accepts_shot(&self) -> bool {
        matches!(self.game.stage(), Stage::Break | Stage::Play)
    }

    /// Whether a shot may be taken now, and if not a stable code saying why:
    /// `decision-pending`, `rack-over` or a [`CallError`] name.
    ///
    /// [`CallError`]: cue_core::rules::eight_ball::CallError
    pub fn can_shoot(&self) -> Result<(), &'static str> {
        match self.game.stage() {
            Stage::Decision(_) => Err("decision-pending"),
            Stage::Over(_) => Err("rack-over"),
            Stage::Break | Stage::Play => self
                .game
                .validate_call(self.effective_call())
                .map_err(reason),
        }
    }

    /// The call that will be used: nothing on the break.
    pub fn effective_call(&self) -> Call {
        if *self.game.stage() == Stage::Break {
            Call::None
        } else {
            self.call
        }
    }

    /// Records the shot about to be taken.
    pub(crate) fn begin_shot(&mut self, before: Layout) {
        self.in_flight = Some(InFlight {
            call: self.effective_call(),
            before,
        });
    }

    /// Judges a finished shot and carries out the ruling on the table.
    fn after_shot(&mut self, sim: &mut Simulation, report: &ShotReport) -> Option<Ruling> {
        let flight = self.in_flight.take()?;
        let table = *sim.table();
        let shot = Shot {
            call: flight.call,
            report,
            before: &flight.before,
            table: &table,
        };
        let ruling = match self.game.evaluate(&shot) {
            Ok(r) => r,
            Err(e) => {
                error!("rules refused a finished shot: {e}");
                return None;
            }
        };
        self.game.apply(&ruling);
        match &ruling.next {
            Next::Continue | Next::Pass { .. } => sim.give_cue_ball(region(self.game.in_hand())),
            // The table waits as it is until someone decides.
            Next::Decide(_) => sim.give_cue_ball(None),
            Next::Over(outcome) => {
                self.score[outcome.winner.index()] += 1;
                sim.give_cue_ball(None);
            }
        }
        self.manual_call = false;
        self.call = Call::None;
        self.last = Some(ruling.clone());
        self.last_call = flight.call;
        Some(ruling)
    }

    /// Resolves a pending decision. Returns what it did on the table, or
    /// `None` if the choice was not on offer.
    fn decide(&mut self, sim: &mut Simulation, choice: Choice) -> Option<Effect> {
        let effect = self.game.decide(choice).ok()?;
        match effect {
            Effect::None => {}
            Effect::Rerack => {
                self.last = None;
                sim.new_rack(HandRegion::AboveHeadString);
            }
            Effect::SpotEight => spot(sim, BallId::EIGHT),
        }
        sim.give_cue_ball(region(self.game.in_hand()));
        Some(effect)
    }

    /// Starts the next rack of the match; the break alternates (WPA 1.3).
    fn next_rack(&mut self, sim: &mut Simulation) -> bool {
        if !matches!(self.game.stage(), Stage::Over(_)) || self.match_over() {
            return false;
        }
        let breaker = self.game.breaker().other();
        self.game = Game::new(self.game.ruleset(), breaker);
        self.rack += 1;
        self.last = None;
        sim.new_rack(HandRegion::AboveHeadString);
        true
    }

    /// Both players agreed to a stalemate (WPA 1.13, 4.11).
    fn stalemate(&mut self, sim: &mut Simulation) {
        self.game.stalemate();
        self.last = None;
        sim.new_rack(HandRegion::AboveHeadString);
    }

    /// WPA 1.6: spot the legal ball nearest the head string on request.
    /// Returns the ball spotted, if the request was allowed.
    fn spot_request(&mut self, sim: &mut Simulation) -> Option<BallId> {
        let ball = self.game.spot_request(&sim.layout(), sim.table())?;
        spot(sim, ball);
        Some(ball)
    }
}

/// The stable code of a refusal, for the host to explain in its language.
fn reason(e: RulesError) -> &'static str {
    e.name()
}

fn clean_name(name: &str) -> String {
    name.trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_NAME)
        .collect()
}

fn region(in_hand: InHand) -> Option<HandRegion> {
    match in_hand {
        InHand::No => None,
        InHand::Anywhere => Some(HandRegion::Anywhere),
        InHand::AboveHeadString => Some(HandRegion::AboveHeadString),
    }
}

/// Moves `ball` to its spot (WPA 1.5), clear of every other ball.
fn spot(sim: &mut Simulation, ball: BallId) {
    let layout = sim.layout();
    let occupied: Vec<DVec2> = layout
        .iter()
        .filter(|(b, _)| *b != ball && !b.is_cue())
        .map(|(_, p)| p)
        .collect();
    let at = spot_position(sim.table(), &occupied, layout.get(BallId::CUE));
    if let Err(e) = sim.spot_ball(ball, at) {
        error!("could not spot {ball}: {e}");
    }
}

/// The pocket an object ball would head for if struck from `ghost`, the cue
/// ball's centre at contact: the one closest to the line of centres, ahead of
/// the ball.
pub(crate) fn pocket_for(sim: &Simulation, ghost: DVec2, object: DVec2) -> Option<PocketId> {
    let path = (object - ghost).try_normalize()?;
    sim.world
        .geometry()
        .pockets()
        .iter()
        .filter_map(|p| {
            let to = (p.centre - object).try_normalize()?;
            let alignment = to.dot(path);
            (alignment > 0.0).then_some((p.id, alignment))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Starts a match in `mode` without the Bevy plumbing, for other modules'
/// tests.
#[cfg(test)]
pub(crate) fn start_for_tests(mode: &mut Mode, setup: &GameSetup) {
    *mode = Mode::EightBall(Box::new(EightBall::new(setup, true)));
}

/// Applies eight-ball commands. Unlike shots, these work while paused: they
/// set up the table rather than let time pass.
pub(crate) fn apply_commands(
    mut commands: MessageReader<GameCommand>,
    mut mode: ResMut<Mode>,
    mut sim: ResMut<Simulation>,
    time: Res<Time>,
    stroke: Res<crate::shot::Stroke>,
    mut events: MessageWriter<GameEvent>,
) {
    for command in commands.read() {
        match (command, mode.as_mut()) {
            (GameCommand::NewGame(setup), _) => {
                // A coin flip from the clock is plenty for "who breaks".
                let coin = time.elapsed().subsec_nanos() % 2 == 0;
                *mode = Mode::EightBall(Box::new(EightBall::new(setup, coin)));
                sim.new_rack(HandRegion::AboveHeadString);
                events.write(GameEvent::NewGame);
            }
            (GameCommand::FreePlay, _) => {
                *mode = Mode::FreePlay;
                sim.new_rack(HandRegion::Anywhere);
                events.write(GameEvent::NewGame);
            }
            // The rest only means something between eight-ball shots.
            (_, Mode::EightBall(_)) if !sim.world.is_at_rest() || stroke.swinging() => {}
            (GameCommand::SetCall(call), Mode::EightBall(session)) => {
                match session.game.validate_call(*call) {
                    Ok(()) => {
                        session.call = *call;
                        session.manual_call = true;
                    }
                    Err(e) => {
                        events.write(GameEvent::CallRefused(reason(e)));
                    }
                }
            }
            (GameCommand::AutoCall, Mode::EightBall(session)) => session.manual_call = false,
            (GameCommand::Decide(choice), Mode::EightBall(session)) => {
                match session.decide(&mut sim, *choice) {
                    Some(effect) => {
                        events.write(GameEvent::Decided { choice: *choice });
                        match effect {
                            Effect::Rerack => {
                                events.write(GameEvent::Racked { cleared: false });
                            }
                            Effect::SpotEight => {
                                events.write(GameEvent::Spotted {
                                    ball: BallId::EIGHT,
                                });
                            }
                            Effect::None => {}
                        }
                    }
                    None => warn!("{} is not on offer", choice.name()),
                }
            }
            (GameCommand::NextRack, Mode::EightBall(session)) => {
                if session.next_rack(&mut sim) {
                    events.write(GameEvent::Racked { cleared: false });
                }
            }
            (GameCommand::Stalemate, Mode::EightBall(session)) => {
                if matches!(session.game.stage(), Stage::Play) {
                    session.stalemate(&mut sim);
                    events.write(GameEvent::Racked { cleared: false });
                }
            }
            (GameCommand::SpotRequest, Mode::EightBall(session)) => {
                if let Some(ball) = session.spot_request(&mut sim) {
                    events.write(GameEvent::Spotted { ball });
                }
            }
            _ => {}
        }
    }
}

/// Judges every finished shot in eight-ball.
pub(crate) fn rule_shots(
    mut finished: MessageReader<ShotFinished>,
    mut mode: ResMut<Mode>,
    mut sim: ResMut<Simulation>,
    mut events: MessageWriter<GameEvent>,
) {
    // Always drained, so a free-play shot is never judged after a switch.
    for ShotFinished(report) in finished.read() {
        if let Mode::EightBall(session) = mode.as_mut()
            && session.after_shot(&mut sim, report).is_some()
        {
            events.write(GameEvent::Ruled);
        }
    }
}

/// The call the aim implies (WPA 1.7: obvious shots need no explicit call):
/// the ball the cue ball would hit first and the pocket it would head for,
/// if that is a legal call. Otherwise a safety where a call is required, and
/// nothing where it is not.
pub(crate) fn auto_call(session: &EightBall, sim: &Simulation, direction: DVec2) -> Call {
    let game = &session.game;
    if *game.stage() != Stage::Play {
        return Call::None;
    }
    let shooter = game.shooter();
    let must_call = game.ruleset().calls_every_shot()
        || (game.on_the_eight(shooter) && game.group(shooter).is_some());
    let candidate = sim.cue_position().and_then(|cue| {
        let target = crate::shot::first_contact(sim, cue, direction);
        let (ball, object) = target.ball.zip(target.object)?;
        let pocket = pocket_for(sim, target.end, object)?;
        Some(Call::Ball { ball, pocket })
    });
    match candidate {
        Some(call @ Call::Ball { ball, .. }) if game.validate_call(call).is_ok() => {
            // Casual rules only call the eight.
            if must_call || ball == BallId::EIGHT {
                call
            } else {
                Call::None
            }
        }
        _ if must_call => Call::Safety,
        _ => Call::None,
    }
}

/// Keeps the call in step with the aim unless the player chose one.
pub(crate) fn suggest_call(
    mut mode: ResMut<Mode>,
    sim: Res<Simulation>,
    aim: Res<crate::shot::Aim>,
) {
    let Some(session) = mode.eight_ball() else {
        return;
    };
    if session.manual_call || !sim.world.is_at_rest() {
        return;
    }
    let call = auto_call(session, &sim, aim.direction);
    if session.call != call
        && let Mode::EightBall(session) = mode.as_mut()
    {
        session.call = call;
    }
}

#[cfg(test)]
mod tests {
    use cue_core::{
        rules::eight_ball::{Decision, Foul, Outcome, Situation},
        rules::spot_position,
    };
    use cue_physics::{PhysicsProfile, Strike, World};

    use super::*;

    fn ball(n: u8) -> BallId {
        BallId::new(n).unwrap()
    }

    fn start(ruleset: Ruleset) -> (EightBall, Simulation) {
        let setup = GameSetup {
            ruleset,
            breaker: Some(Seat::A),
            race_to: 2,
            names: ["  Ana\u{7}  ".into(), "Bo".into()],
        };
        let mut sim = Simulation::new(7);
        sim.new_rack(HandRegion::AboveHeadString);
        (EightBall::new(&setup, false), sim)
    }

    /// Replaces the balls on the table, keeping the rules state.
    fn lay_out(sim: &mut Simulation, balls: &[(u8, DVec2)]) {
        let layout: Layout = balls.iter().map(|&(n, p)| (ball(n), p)).collect();
        sim.world = World::new(PhysicsProfile::baseline(), &layout).unwrap();
        sim.clock = 0.0;
    }

    /// Commits a made-up ruling, to put the game in a given state.
    fn rule(session: &mut EightBall, sim: &mut Simulation, ruling: Ruling) {
        session.game.apply(&ruling);
        sim.give_cue_ball(region(session.game.in_hand()));
    }

    /// Takes a shot the way the game does and plays it to rest.
    fn shoot(session: &mut EightBall, sim: &mut Simulation, strike: Strike) -> Ruling {
        session.can_shoot().unwrap();
        let before = sim.layout();
        sim.strike(strike).unwrap();
        session.begin_shot(before);
        sim.world.run_to_rest(1e6).unwrap();
        sim.clock = sim.world.time();
        let report = sim.world.take_report().unwrap();
        session.after_shot(sim, &report).unwrap()
    }

    /// Player A on solids with every solid gone: shooting at the eight.
    fn a_on_the_eight(session: &mut EightBall, sim: &mut Simulation) {
        rule(
            session,
            sim,
            Ruling {
                shooter: Seat::A,
                fouls: vec![],
                pocketed: (1..=7).map(ball).collect(),
                assigned: Some((Seat::A, Group::Solids)),
                next: Next::Continue,
            },
        );
    }

    /// The eight lined up with the near foot corner, the cue ball behind it.
    fn eight_to_the_corner(sim: &mut Simulation) -> DVec2 {
        let corner = sim.world.geometry().pocket(PocketId::FootNear).centre;
        let line = DVec2::splat(std::f64::consts::FRAC_1_SQRT_2);
        lay_out(
            sim,
            &[
                (0, corner - line * 0.7),
                (8, corner - line * 0.35),
                (12, DVec2::ZERO),
            ],
        );
        line
    }

    #[test]
    fn a_new_game_breaks_from_the_kitchen() {
        let (session, mut sim) = start(Ruleset::EightBallWpa);
        assert_eq!(
            session.name(Seat::A),
            "Ana",
            "names are trimmed and cleaned"
        );
        assert_eq!(session.game.shooter(), Seat::A);
        assert_eq!(sim.hand_region(), Some(HandRegion::AboveHeadString));
        assert_eq!(session.effective_call(), Call::None);
        assert!(session.can_shoot().is_ok());
        let head = sim.table().head_string_x();
        assert!(sim.place_cue(DVec2::new(head + 0.05, 0.0)).is_err());
        assert!(sim.place_cue(DVec2::new(head - 0.05, 0.2)).is_ok());
    }

    #[test]
    fn the_table_follows_the_ruling_on_the_break() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        let ruling = shoot(&mut session, &mut sim, Strike::centre(DVec2::X, 1.0));
        assert_eq!(ruling.shooter, Seat::A);
        assert_eq!(session.last.as_ref(), Some(&ruling));
        match &ruling.next {
            Next::Continue => assert_eq!(session.game.shooter(), Seat::A),
            Next::Pass { .. } => assert_eq!(session.game.shooter(), Seat::B),
            Next::Decide(d) => {
                assert_eq!(session.game.shooter(), d.chooser);
                assert_eq!(session.can_shoot(), Err("decision-pending"));
            }
            Next::Over(_) => unreachable!("nothing on the break ends the rack"),
        }
        assert_eq!(sim.hand_region(), region(session.game.in_hand()));
        let gone = ruling.pocketed.len();
        assert_eq!(session.game.on_table().count(), 15 - gone);
    }

    #[test]
    fn a_scratch_gives_the_opponent_ball_in_hand_anywhere() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        let quiet_break = Ruling {
            shooter: Seat::A,
            fouls: vec![],
            pocketed: vec![],
            assigned: None,
            next: Next::Pass {
                in_hand: InHand::No,
            },
        };
        rule(&mut session, &mut sim, quiet_break);
        assert_eq!(sim.hand_region(), None);
        session.call = Call::Safety;
        let cue = sim.cue_position().unwrap();
        let corner = sim.world.geometry().pocket(PocketId::HeadNear).centre;
        let ruling = shoot(&mut session, &mut sim, Strike::centre(corner - cue, 0.3));
        assert!(ruling.fouls.contains(&Foul::CueBallPocketed), "{ruling:?}");
        assert_eq!(
            ruling.next,
            Next::Pass {
                in_hand: InHand::Anywhere
            }
        );
        assert_eq!(session.game.shooter(), Seat::A);
        assert_eq!(sim.hand_region(), Some(HandRegion::Anywhere));
        assert!(
            sim.cue_position().is_some(),
            "the cue ball is back on the table"
        );
        assert_eq!(session.call, Call::None, "a call lasts one shot");
    }

    #[test]
    fn a_decision_blocks_play_until_the_chooser_picks_an_offered_option() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        let illegal = Ruling {
            shooter: Seat::A,
            fouls: vec![],
            pocketed: vec![],
            assigned: None,
            next: Next::Decide(Decision {
                chooser: Seat::B,
                situation: Situation::IllegalBreak,
                options: vec![
                    Choice::AcceptTable,
                    Choice::RerackAndBreak,
                    Choice::RerackOffenderBreaks,
                ],
                cue_ball_off: false,
            }),
        };
        session.game.apply(&illegal);
        sim.give_cue_ball(None);
        assert!(session.can_shoot().is_err());
        assert_eq!(session.decide(&mut sim, Choice::Rebreak), None);
        assert_eq!(
            session.decide(&mut sim, Choice::RerackAndBreak),
            Some(Effect::Rerack)
        );
        assert_eq!(*session.game.stage(), Stage::Break);
        assert_eq!(session.game.breaker(), Seat::B);
        assert_eq!(sim.hand_region(), Some(HandRegion::AboveHeadString));
        assert_eq!(sim.layout().iter().count(), 16);
    }

    #[test]
    fn a_spotted_eight_goes_back_on_the_long_string() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        // Drop the eight for real, then rule it a legal break.
        let line = eight_to_the_corner(&mut sim);
        sim.strike(Strike::centre(line, 0.3)).unwrap();
        sim.world.run_to_rest(1e6).unwrap();
        sim.clock = sim.world.time();
        let report = sim.world.take_report().unwrap();
        assert!(
            report.pocketed().any(|(b, _)| b == BallId::EIGHT),
            "{report:?}"
        );
        let eight_on_break = Ruling {
            shooter: Seat::A,
            fouls: vec![],
            pocketed: vec![BallId::EIGHT],
            assigned: None,
            next: Next::Decide(Decision {
                chooser: Seat::A,
                situation: Situation::EightOnLegalBreak,
                options: vec![Choice::SpotEightAndContinue, Choice::Rebreak],
                cue_ball_off: false,
            }),
        };
        session.game.apply(&eight_on_break);
        sim.give_cue_ball(None);
        assert_eq!(
            session.decide(&mut sim, Choice::SpotEightAndContinue),
            Some(Effect::SpotEight)
        );
        let others: Vec<DVec2> = sim
            .layout()
            .iter()
            .filter(|(b, _)| !b.is_cue() && *b != BallId::EIGHT)
            .map(|(_, p)| p)
            .collect();
        let expected = spot_position(sim.table(), &others, sim.cue_position());
        assert_eq!(sim.layout().get(BallId::EIGHT), Some(expected));
        assert_eq!(expected.y, 0.0, "on the long string");
        assert!(session.game.on_table().any(|b| b == BallId::EIGHT));
        assert_eq!(
            sim.hand_region(),
            None,
            "the breaker plays on from where it lies"
        );
    }

    #[test]
    fn the_aim_calls_the_eight_and_a_win_scores_and_alternates_the_break() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        a_on_the_eight(&mut session, &mut sim);
        let line = eight_to_the_corner(&mut sim);
        let call = auto_call(&session, &sim, line);
        assert_eq!(
            call,
            Call::Ball {
                ball: BallId::EIGHT,
                pocket: PocketId::FootNear
            }
        );
        session.call = call;
        let draw = Strike {
            direction: line,
            // 2.4 m/s: a soft draw that stops the cue ball short of the pocket.
            power: 0.2,
            tip_offset: DVec2::new(0.0, -0.4),
        };
        let ruling = shoot(&mut session, &mut sim, draw);
        assert_eq!(
            ruling.next,
            Next::Over(Outcome {
                winner: Seat::A,
                loss: None
            }),
            "{ruling:?}"
        );
        assert_eq!(session.score, [1, 0]);
        assert!(!session.match_over(), "a race to two");
        assert!(session.can_shoot().is_err());

        assert!(session.next_rack(&mut sim));
        assert_eq!(session.rack, 2);
        assert_eq!(session.game.breaker(), Seat::B, "the break alternates");
        assert_eq!(*session.game.stage(), Stage::Break);
        assert_eq!(sim.hand_region(), Some(HandRegion::AboveHeadString));
        assert_eq!(sim.layout().iter().count(), 16);
    }

    #[test]
    fn the_eight_in_another_pocket_loses_the_rack() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        a_on_the_eight(&mut session, &mut sim);
        let line = eight_to_the_corner(&mut sim);
        session.call = Call::Ball {
            ball: BallId::EIGHT,
            pocket: PocketId::FootFar,
        };
        session.manual_call = true;
        let ruling = shoot(&mut session, &mut sim, Strike::centre(line, 0.3));
        assert_eq!(
            ruling.next,
            Next::Over(Outcome {
                winner: Seat::B,
                loss: Some(cue_core::rules::eight_ball::LossReason::EightInUncalledPocket)
            }),
            "{ruling:?}"
        );
        assert_eq!(session.score, [0, 1]);
    }

    #[test]
    fn casual_rules_call_only_the_eight_and_wpa_falls_back_to_a_safety() {
        for (ruleset, expected) in [
            (Ruleset::EightBallCasual, Call::None),
            (Ruleset::EightBallWpa, Call::Safety),
        ] {
            let (mut session, mut sim) = start(ruleset);
            rule(
                &mut session,
                &mut sim,
                Ruling {
                    shooter: Seat::A,
                    fouls: vec![],
                    pocketed: vec![ball(1)],
                    assigned: Some((Seat::A, Group::Solids)),
                    next: Next::Continue,
                },
            );
            // Aiming at a stripe: nothing legal in line.
            lay_out(&mut sim, &[(0, DVec2::new(-0.5, 0.0)), (12, DVec2::ZERO)]);
            assert_eq!(auto_call(&session, &sim, DVec2::X), expected, "{ruleset:?}");
            // Aiming at a solid: casual calls nothing, WPA the ball and pocket.
            lay_out(&mut sim, &[(0, DVec2::new(-0.5, 0.0)), (3, DVec2::ZERO)]);
            let call = auto_call(&session, &sim, DVec2::X);
            match ruleset {
                Ruleset::EightBallCasual => assert_eq!(call, Call::None),
                Ruleset::EightBallWpa => {
                    assert!(matches!(call, Call::Ball { ball, .. } if ball == self::ball(3)));
                }
            }
        }
    }

    #[test]
    fn a_kitchen_shooter_may_ask_for_the_nearest_legal_ball_to_be_spotted() {
        let (mut session, mut sim) = start(Ruleset::EightBallWpa);
        rule(
            &mut session,
            &mut sim,
            Ruling {
                shooter: Seat::A,
                fouls: vec![Foul::CueBallPocketed],
                pocketed: vec![],
                assigned: None,
                next: Next::Pass {
                    in_hand: InHand::AboveHeadString,
                },
            },
        );
        let head = sim.table().head_string_x();
        lay_out(
            &mut sim,
            &[
                (0, DVec2::new(head - 0.4, 0.3)),
                (3, DVec2::new(head - 0.1, -0.2)),
                (12, DVec2::new(head - 0.3, 0.1)),
            ],
        );
        sim.give_cue_ball(Some(HandRegion::AboveHeadString));
        assert_eq!(session.spot_request(&mut sim), Some(ball(3)));
        assert_eq!(sim.layout().get(ball(3)), Some(sim.table().foot_spot()));
        assert_eq!(
            session.spot_request(&mut sim),
            None,
            "only when every target is above"
        );
    }
}
