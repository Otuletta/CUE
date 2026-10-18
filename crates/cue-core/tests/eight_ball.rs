//! The eight-ball case matrix (spec §24.3). Every test builds a shot report by
//! hand, so the rules are exercised without any physics, engine or browser
//! (spec §30.2: "rules can be tested without initialising Bevy").
//!
//! Test names follow the WPA rule they check.

// Helpers outside `#[test]` fns are not covered by `allow-unwrap-in-tests`.
#![allow(clippy::unwrap_used)]

use cue_core::{
    BallId, Layout, PocketId, Rail, ShotEvent, ShotEventKind, ShotReport, TableSpec,
    eight_ball_rack,
    glam::DVec2,
    rules::{
        Group, Ruleset, Seat,
        eight_ball::{
            Call, CallError, Choice, Effect, Foul, Game, InHand, LossReason, Next, Outcome,
            RulesError, Shot, Situation, Stage,
        },
    },
};

fn ball(n: u8) -> BallId {
    BallId::new(n).unwrap()
}

/// A report built from `(time, kind)` pairs.
fn report(events: &[(f64, ShotEventKind)]) -> ShotReport {
    ShotReport {
        shot_id: 1,
        events: events
            .iter()
            .map(|&(time, kind)| ShotEvent { time, kind })
            .collect(),
    }
}

fn hit(a: u8, b: u8) -> ShotEventKind {
    ShotEventKind::BallContact {
        a: ball(a),
        b: ball(b),
    }
}

fn rail(n: u8) -> ShotEventKind {
    ShotEventKind::Cushion {
        ball: ball(n),
        rail: Rail::Foot,
        frozen: false,
    }
}

/// A frozen ball pushed into the rail it was touching (WPA 2.7).
fn pressed(n: u8) -> ShotEventKind {
    ShotEventKind::Cushion {
        ball: ball(n),
        rail: Rail::Near,
        frozen: true,
    }
}

fn sink(n: u8, pocket: PocketId) -> ShotEventKind {
    ShotEventKind::Pocketed {
        ball: ball(n),
        pocket,
    }
}

fn call(n: u8, pocket: PocketId) -> Call {
    Call::Ball {
        ball: ball(n),
        pocket,
    }
}

fn table() -> TableSpec {
    TableSpec::nine_foot()
}

/// The standard rack, cue ball on the head spot (above the head string).
fn rack() -> Layout {
    eight_ball_rack(&table(), 1).unwrap()
}

/// Judges and applies one shot; returns the ruling.
fn play(
    game: &mut Game,
    call: Call,
    events: &[(f64, ShotEventKind)],
    before: &Layout,
) -> cue_core::rules::eight_ball::Ruling {
    let report = report(events);
    let table = table();
    let ruling = game
        .evaluate(&Shot {
            call,
            report: &report,
            before,
            table: &table,
        })
        .unwrap();
    game.apply(&ruling);
    ruling
}

/// A game past a quiet legal break: A broke, four balls reached a rail,
/// nothing dropped, B is at the table.
fn after_quiet_break(ruleset: Ruleset) -> Game {
    let mut game = Game::new(ruleset, Seat::A);
    play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.3, rail(2)),
            (0.3, rail(3)),
            (0.4, rail(4)),
            (0.5, rail(5)),
        ],
        &rack(),
    );
    assert_eq!(game.shooter(), Seat::B);
    game
}

/// A game where A owns solids and is at the table.
fn a_on_solids(ruleset: Ruleset) -> Game {
    let mut game = after_quiet_break(ruleset);
    // B misses: pass to A.
    play(
        &mut game,
        call(9, PocketId::FootNear),
        &[(0.2, hit(0, 9)), (0.5, rail(9))],
        &rack(),
    );
    // A calls the 1 and makes it.
    let r = play(
        &mut game,
        call(1, PocketId::FootFar),
        &[(0.2, hit(0, 1)), (0.6, sink(1, PocketId::FootFar))],
        &rack(),
    );
    assert_eq!(r.assigned, Some((Seat::A, Group::Solids)));
    assert_eq!(game.shooter(), Seat::A);
    game
}

/// A on solids with every solid already off the table.
fn a_on_the_eight(ruleset: Ruleset) -> Game {
    let mut game = a_on_solids(ruleset);
    for n in 2..=7 {
        let r = play(
            &mut game,
            call(n, PocketId::SideNear),
            &[(0.2, hit(0, n)), (0.6, sink(n, PocketId::SideNear))],
            &rack(),
        );
        assert_eq!(r.next, Next::Continue);
    }
    assert!(game.on_the_eight(Seat::A));
    game
}

// --- 4.3 Break -------------------------------------------------------------

#[test]
fn rule_4_3_d_four_rails_without_a_pocket_is_a_legal_break() {
    let game = after_quiet_break(Ruleset::EightBallWpa);
    assert_eq!(game.stage(), &Stage::Play);
    assert!(game.is_open());
    assert_eq!(game.in_hand(), InHand::No);
}

#[test]
fn rule_4_3_d_three_rails_and_no_pocket_is_an_illegal_break_with_three_options() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.3, rail(2)),
            (0.4, rail(3)),
            (0.5, rail(4)),
        ],
        &rack(),
    );
    let Next::Decide(d) = &r.next else {
        panic!("{r:?}")
    };
    assert_eq!(d.situation, Situation::IllegalBreak);
    assert_eq!(d.chooser, Seat::B);
    assert_eq!(
        d.options,
        vec![
            Choice::AcceptTable,
            Choice::RerackAndBreak,
            Choice::RerackOffenderBreaks
        ]
    );
    assert_eq!(game.shooter(), Seat::B, "the chooser is up");

    let mut accept = game.clone();
    assert_eq!(accept.decide(Choice::AcceptTable), Ok(Effect::None));
    assert_eq!((accept.shooter(), accept.in_hand()), (Seat::B, InHand::No));

    let mut rebreak = game.clone();
    assert_eq!(rebreak.decide(Choice::RerackAndBreak), Ok(Effect::Rerack));
    assert_eq!(
        (rebreak.breaker(), rebreak.stage()),
        (Seat::B, &Stage::Break)
    );

    let mut offender = game.clone();
    assert_eq!(
        offender.decide(Choice::RerackOffenderBreaks),
        Ok(Effect::Rerack)
    );
    assert_eq!(offender.breaker(), Seat::A);

    assert_eq!(game.decide(Choice::Rebreak), Err(RulesError::InvalidChoice));
}

#[test]
fn rule_4_3_c_a_ball_on_the_break_keeps_the_breaker_at_the_table_and_the_table_open() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.6, sink(3, PocketId::FootFar)),
            (0.7, sink(12, PocketId::SideNear)),
        ],
        &rack(),
    );
    assert_eq!(r.next, Next::Continue);
    assert!(game.is_open(), "balls on the break never choose groups");
    assert_eq!(game.shooter(), Seat::A);
    assert_eq!(game.remaining(Group::Solids), 6);
}

#[test]
fn rule_4_3_e_the_eight_on_a_legal_break_lets_the_breaker_spot_it_or_rebreak() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[(0.1, hit(0, 1)), (0.6, sink(8, PocketId::SideFar))],
        &rack(),
    );
    let Next::Decide(d) = &r.next else {
        panic!("{r:?}")
    };
    assert_eq!(
        (d.situation, d.chooser),
        (Situation::EightOnLegalBreak, Seat::A)
    );
    assert!(r.fouls.is_empty(), "not a foul");

    let mut spot = game.clone();
    assert_eq!(
        spot.decide(Choice::SpotEightAndContinue),
        Ok(Effect::SpotEight)
    );
    assert_eq!(spot.shooter(), Seat::A);
    assert!(spot.on_table().any(|b| b == BallId::EIGHT));

    assert_eq!(game.decide(Choice::Rebreak), Ok(Effect::Rerack));
    assert_eq!(game.breaker(), Seat::A);
}

#[test]
fn rule_4_3_f_the_eight_on_a_foul_break_gives_the_opponent_the_choice() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.6, sink(8, PocketId::SideFar)),
            (0.9, sink(0, PocketId::HeadNear)),
        ],
        &rack(),
    );
    let Next::Decide(d) = &r.next else {
        panic!("{r:?}")
    };
    assert_eq!(
        (d.situation, d.chooser),
        (Situation::EightOnFoulBreak, Seat::B)
    );
    assert_eq!(
        game.decide(Choice::SpotEightCueInHand),
        Ok(Effect::SpotEight)
    );
    assert_eq!(
        (game.shooter(), game.in_hand()),
        (Seat::B, InHand::AboveHeadString)
    );
}

#[test]
fn rule_4_3_h_a_scratch_on_a_legal_break_gives_ball_in_hand_above_the_head_string() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.6, sink(4, PocketId::FootFar)),
            (0.9, sink(0, PocketId::HeadNear)),
        ],
        &rack(),
    );
    assert!(r.fouls.contains(&Foul::CueBallPocketed));
    assert_eq!(
        r.next,
        Next::Pass {
            in_hand: InHand::AboveHeadString
        }
    );
    assert!(game.is_open());
    assert_eq!(game.remaining(Group::Solids), 6, "the 4 stays down");
}

#[test]
fn an_illegal_break_that_is_also_a_foul_offers_the_options_of_4_3_d_and_h() {
    // Missing the rack entirely: no contact (3.3) and nothing to a rail.
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(&mut game, Call::None, &[(0.4, rail(0))], &rack());
    assert_eq!(r.fouls, vec![Foul::NoContact]);
    let Next::Decide(d) = &r.next else {
        panic!("{r:?}")
    };
    assert_eq!(d.situation, Situation::IllegalBreak);
    assert_eq!(
        d.options,
        vec![
            Choice::AcceptTable,
            Choice::CueInHand,
            Choice::RerackAndBreak,
            Choice::RerackOffenderBreaks
        ]
    );
    game.decide(Choice::CueInHand).unwrap();
    assert_eq!(
        (game.shooter(), game.in_hand()),
        (Seat::B, InHand::AboveHeadString)
    );
}

#[test]
fn an_illegal_break_with_a_scratch_accepted_puts_the_cue_ball_in_hand_above_the_head_string() {
    // CUE's adjudication: 4.3 (d) and (h) combined.
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    play(
        &mut game,
        Call::None,
        &[(0.1, hit(0, 1)), (0.9, sink(0, PocketId::HeadFar))],
        &rack(),
    );
    game.decide(Choice::AcceptTable).unwrap();
    assert_eq!(
        (game.shooter(), game.in_hand()),
        (Seat::B, InHand::AboveHeadString)
    );
}

// --- 4.4 Open table ----------------------------------------------------------

#[test]
fn rule_4_4_groups_are_assigned_only_by_a_called_ball_legally_pocketed() {
    let mut game = after_quiet_break(Ruleset::EightBallWpa);
    // B calls the 10 in the foot-near pocket but it drops in foot-far.
    let r = play(
        &mut game,
        call(10, PocketId::FootNear),
        &[(0.2, hit(0, 10)), (0.7, sink(10, PocketId::FootFar))],
        &rack(),
    );
    assert_eq!(r.assigned, None);
    assert!(game.is_open());
    assert_eq!(game.shooter(), Seat::A, "play passes");
    // A calls the 5 and makes it: A has solids, B stripes.
    let r = play(
        &mut game,
        call(5, PocketId::SideNear),
        &[(0.2, hit(0, 5)), (0.6, sink(5, PocketId::SideNear))],
        &rack(),
    );
    assert_eq!(r.assigned, Some((Seat::A, Group::Solids)));
    assert_eq!(game.group(Seat::B), Some(Group::Stripes));
    assert_eq!(game.shooter(), Seat::A);
}

#[test]
fn rule_4_4_on_an_open_table_any_ball_but_the_eight_may_be_hit_first() {
    let mut game = after_quiet_break(Ruleset::EightBallWpa);
    // Combination: hit a stripe to pocket the called solid.
    let r = play(
        &mut game,
        call(2, PocketId::FootFar),
        &[
            (0.2, hit(0, 11)),
            (0.3, hit(2, 11)),
            (0.7, sink(2, PocketId::FootFar)),
        ],
        &rack(),
    );
    assert!(r.fouls.is_empty());
    assert_eq!(r.assigned, Some((Seat::B, Group::Solids)));
}

#[test]
fn rule_4_4_hitting_the_eight_first_on_an_open_table_is_a_foul() {
    let mut game = after_quiet_break(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootFar),
        &[(0.2, hit(0, 8)), (0.5, rail(8))],
        &rack(),
    );
    assert_eq!(r.fouls, vec![Foul::WrongBallFirst { hit: BallId::EIGHT }]);
    assert_eq!(
        r.next,
        Next::Pass {
            in_hand: InHand::Anywhere
        }
    );
}

#[test]
fn rule_4_4_a_cleared_group_on_an_open_table_may_be_claimed_to_shoot_the_eight() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    // An improbable break: every stripe drops, the table stays open.
    let mut events: Vec<(f64, ShotEventKind)> = (9..=15)
        .map(|n| (0.5, sink(n, PocketId::FootNear)))
        .chain([(0.1, hit(0, 1))])
        .collect();
    events.sort_by(|a, b| a.0.total_cmp(&b.0));
    play(&mut game, Call::None, &events, &rack());
    assert!(game.is_open() && game.on_the_eight(Seat::A));
    assert!(game.legal_targets().contains(&BallId::EIGHT));
    let r = play(
        &mut game,
        call(8, PocketId::SideFar),
        &[(0.2, hit(0, 8)), (0.8, sink(8, PocketId::SideFar))],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::A,
            loss: None
        })
    );
}

// --- 4.5, 4.6 Continuing play and called shots --------------------------------

#[test]
fn rule_4_6_the_called_ball_in_another_pocket_ends_the_inning_without_a_foul() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(3, PocketId::HeadNear),
        &[(0.2, hit(0, 3)), (0.7, sink(3, PocketId::HeadFar))],
        &rack(),
    );
    assert!(r.fouls.is_empty());
    assert_eq!(
        r.next,
        Next::Pass {
            in_hand: InHand::No
        }
    );
    assert_eq!(
        game.remaining(Group::Solids),
        5,
        "no ball is ever spotted (4.7)"
    );
}

#[test]
fn rule_4_6_a_safety_passes_play_and_pocketed_balls_stay_down() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        Call::Safety,
        &[(0.2, hit(0, 2)), (0.7, sink(2, PocketId::SideFar))],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Pass {
            in_hand: InHand::No
        }
    );
    assert_eq!(game.shooter(), Seat::B);
    assert_eq!(game.remaining(Group::Solids), 5);
}

#[test]
fn wpa_requires_a_call_on_every_shot_but_casual_only_on_the_eight() {
    let wpa = a_on_solids(Ruleset::EightBallWpa);
    assert_eq!(
        wpa.validate_call(Call::None),
        Err(RulesError::InvalidCall(CallError::CallRequired))
    );
    let casual = a_on_solids(Ruleset::EightBallCasual);
    assert_eq!(casual.validate_call(Call::None), Ok(()));
    let casual_eight = a_on_the_eight(Ruleset::EightBallCasual);
    assert_eq!(
        casual_eight.validate_call(Call::None),
        Err(RulesError::InvalidCall(CallError::CallTheEightsPocket))
    );
    assert_eq!(
        RulesError::InvalidCall(CallError::CallTheEightsPocket).name(),
        "call-the-eights-pocket"
    );
}

#[test]
fn calls_must_name_a_legal_ball() {
    let game = a_on_solids(Ruleset::EightBallWpa);
    assert!(
        game.validate_call(call(12, PocketId::FootFar)).is_err(),
        "not your group"
    );
    assert!(
        game.validate_call(call(8, PocketId::FootFar)).is_err(),
        "not on the eight yet"
    );
    assert!(
        game.validate_call(call(1, PocketId::FootFar)).is_err(),
        "already pocketed"
    );
    assert_eq!(game.validate_call(call(2, PocketId::FootFar)), Ok(()));
    let open = after_quiet_break(Ruleset::EightBallWpa);
    assert!(
        open.validate_call(call(8, PocketId::FootFar)).is_err(),
        "eight on an open table"
    );
}

#[test]
fn casual_any_ball_of_your_group_keeps_you_at_the_table() {
    let mut game = a_on_solids(Ruleset::EightBallCasual);
    let r = play(
        &mut game,
        Call::None,
        &[(0.2, hit(0, 3)), (0.7, sink(3, PocketId::HeadFar))],
        &rack(),
    );
    assert_eq!(r.next, Next::Continue);
}

#[test]
fn casual_the_first_ball_pocketed_on_an_open_table_decides_the_groups() {
    let mut game = after_quiet_break(Ruleset::EightBallCasual);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.2, hit(0, 4)),
            (0.6, sink(13, PocketId::SideFar)),
            (0.8, sink(4, PocketId::FootNear)),
        ],
        &rack(),
    );
    assert_eq!(r.assigned, Some((Seat::B, Group::Stripes)));
}

// --- 4.8 Losing the rack -----------------------------------------------------

#[test]
fn rule_4_8_a_the_eight_with_a_foul_loses() {
    let mut game = a_on_the_eight(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(8, PocketId::FootNear),
        &[
            (0.2, hit(0, 8)),
            (0.6, sink(8, PocketId::FootNear)),
            (0.9, sink(0, PocketId::SideFar)),
        ],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::B,
            loss: Some(LossReason::EightWithFoul)
        })
    );
}

#[test]
fn rule_4_8_b_the_eight_before_the_group_is_cleared_loses() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[
            (0.2, hit(0, 2)),
            (0.4, hit(2, 8)),
            (0.8, sink(8, PocketId::FootFar)),
        ],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::B,
            loss: Some(LossReason::EightBeforeGroupCleared)
        })
    );
}

#[test]
fn rule_4_8_b_the_last_ball_and_the_eight_together_still_lose() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    for n in 2..=6 {
        play(
            &mut game,
            call(n, PocketId::SideNear),
            &[(0.2, hit(0, n)), (0.6, sink(n, PocketId::SideNear))],
            &rack(),
        );
    }
    let r = play(
        &mut game,
        call(7, PocketId::FootFar),
        &[
            (0.2, hit(0, 7)),
            (0.5, sink(7, PocketId::FootFar)),
            (0.7, sink(8, PocketId::FootNear)),
        ],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::B,
            loss: Some(LossReason::EightBeforeGroupCleared)
        })
    );
}

#[test]
fn rule_4_8_c_the_eight_in_an_uncalled_pocket_loses() {
    let mut game = a_on_the_eight(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(8, PocketId::FootNear),
        &[(0.2, hit(0, 8)), (0.6, sink(8, PocketId::HeadFar))],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::B,
            loss: Some(LossReason::EightInUncalledPocket)
        })
    );
}

#[test]
fn rule_4_8_the_eight_legally_in_the_called_pocket_wins() {
    let mut game = a_on_the_eight(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(8, PocketId::HeadFar),
        &[(0.2, hit(0, 8)), (0.6, sink(8, PocketId::HeadFar))],
        &rack(),
    );
    assert_eq!(
        r.next,
        Next::Over(Outcome {
            winner: Seat::A,
            loss: None
        })
    );
    assert!(matches!(game.stage(), Stage::Over(_)));
    assert_eq!(
        game.validate_call(Call::Safety),
        Err(RulesError::NotExpectingShot)
    );
}

#[test]
fn the_losing_rules_do_not_apply_on_the_break() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[(0.1, hit(0, 1)), (0.6, sink(8, PocketId::FootNear))],
        &rack(),
    );
    assert!(matches!(r.next, Next::Decide(_)));
}

// --- 4.9 Standard fouls --------------------------------------------------------

#[test]
fn rule_3_1_a_scratch_gives_ball_in_hand_anywhere() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.2, hit(0, 2)), (0.9, sink(0, PocketId::SideFar))],
        &rack(),
    );
    assert!(r.fouls.contains(&Foul::CueBallPocketed));
    assert_eq!(game.in_hand(), InHand::Anywhere);
    assert_eq!(game.shooter(), Seat::B);
}

#[test]
fn rule_3_2_the_wrong_ball_first_is_a_foul_once_groups_are_set() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[
            (0.2, hit(0, 12)),
            (0.4, hit(12, 2)),
            (0.9, sink(2, PocketId::FootNear)),
        ],
        &rack(),
    );
    assert_eq!(r.fouls, vec![Foul::WrongBallFirst { hit: ball(12) }]);
    assert_eq!(
        r.next,
        Next::Pass {
            in_hand: InHand::Anywhere
        }
    );
    assert_eq!(
        game.remaining(Group::Solids),
        5,
        "the 2 stays down even on a foul"
    );
}

#[test]
fn rule_3_2_a_simultaneous_hit_on_a_legal_and_an_illegal_ball_is_assumed_legal() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.2, hit(0, 12)), (0.2 + 1e-12, hit(0, 2)), (0.5, rail(2))],
        &rack(),
    );
    assert!(r.fouls.is_empty(), "{r:?}");
}

#[test]
fn rule_3_3_no_ball_to_a_rail_after_contact_is_a_foul() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.2, hit(0, 2))],
        &rack(),
    );
    assert_eq!(r.fouls, vec![Foul::NoRailAfterContact]);
}

#[test]
fn rule_3_3_a_rail_before_the_contact_does_not_count() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.1, rail(0)), (0.3, hit(0, 2))],
        &rack(),
    );
    assert_eq!(r.fouls, vec![Foul::NoRailAfterContact]);
}

#[test]
fn rule_3_3_a_rail_at_the_same_instant_as_the_contact_counts_as_after() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.3 - 1e-12, rail(0)), (0.3, hit(0, 2))],
        &rack(),
    );
    assert!(r.fouls.is_empty(), "{r:?}");
}

#[test]
fn rule_2_7_a_frozen_ball_pushed_into_its_own_rail_is_not_driven_to_it() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let shot = |events: &[(f64, ShotEventKind)]| {
        let mut g = game.clone();
        play(&mut g, call(2, PocketId::FootNear), events, &rack())
    };
    let r = shot(&[(0.2, hit(0, 2)), (0.2, pressed(2))]);
    assert_eq!(r.fouls, vec![Foul::NoRailAfterContact]);
    // Off the rail and back again: driven to it.
    let r = shot(&[(0.2, hit(0, 2)), (0.2, pressed(2)), (0.8, rail(2))]);
    assert!(r.fouls.is_empty(), "{r:?}");
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.2, hit(0, 2)), (0.2, pressed(2)), (0.5, rail(0))],
        &rack(),
    );
    assert!(
        r.fouls.is_empty(),
        "the cue ball reaching a rail counts: {r:?}"
    );
}

#[test]
fn rule_4_3_d_frozen_contacts_do_not_count_towards_a_legal_break() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let r = play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.3, rail(2)),
            (0.3, rail(3)),
            (0.4, rail(4)),
            (0.5, pressed(5)),
        ],
        &rack(),
    );
    assert!(
        matches!(r.next, Next::Decide(_)),
        "three rails: illegal: {r:?}"
    );
}

#[test]
fn rule_3_3_missing_every_ball_is_a_foul() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    let r = play(
        &mut game,
        call(2, PocketId::FootNear),
        &[(0.3, rail(0)), (0.6, rail(0))],
        &rack(),
    );
    assert_eq!(r.fouls, vec![Foul::NoContact]);
}

#[test]
fn rule_3_10_playing_from_below_the_head_string_when_restricted_is_a_foul() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    let mut layout: Vec<_> = rack().iter().collect();
    layout.iter_mut().find(|(b, _)| b.is_cue()).unwrap().1 = DVec2::new(0.0, 0.0);
    let before: Layout = layout.into_iter().collect();
    let r = play(
        &mut game,
        Call::None,
        &[(0.1, hit(0, 1)), (0.5, sink(6, PocketId::FootNear))],
        &before,
    );
    assert!(r.fouls.contains(&Foul::BadCueBallPlacement));
}

/// A layout with the cue ball and the 9 above the head string, after a foul
/// on the break left B with the cue ball in hand there.
fn kitchen_game() -> (Game, Layout) {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    play(
        &mut game,
        Call::None,
        &[
            (0.1, hit(0, 1)),
            (0.6, sink(4, PocketId::FootFar)),
            (0.9, sink(0, PocketId::HeadNear)),
        ],
        &rack(),
    );
    assert_eq!(game.in_hand(), InHand::AboveHeadString);
    let head = table().head_string_x();
    let before: Layout = rack()
        .iter()
        .map(|(b, p)| match b.number() {
            0 => (b, DVec2::new(head - 0.3, 0.0)),
            9 => (b, DVec2::new(head - 0.1, 0.2)),
            _ => (b, p),
        })
        .collect();
    (game, before)
}

#[test]
fn rule_3_11_from_the_kitchen_a_kitchen_ball_needs_the_cue_ball_to_cross_the_string_first() {
    let (game, before) = kitchen_game();
    let shot = |events: &[(f64, ShotEventKind)]| {
        let report = report(events);
        let table = table();
        game.evaluate(&Shot {
            call: call(9, PocketId::HeadNear),
            report: &report,
            before: &before,
            table: &table,
        })
        .unwrap()
    };
    let direct = shot(&[(0.2, hit(0, 9)), (0.5, rail(9))]);
    assert!(direct.fouls.contains(&Foul::BadPlayFromAboveHeadString));
    let around = shot(&[
        (0.1, ShotEventKind::CueCrossedHeadString),
        (0.4, rail(0)),
        (0.6, ShotEventKind::CueCrossedHeadString),
        (0.8, hit(0, 9)),
        (0.9, rail(9)),
    ]);
    assert!(around.fouls.is_empty(), "{around:?}");
}

#[test]
fn rule_1_6_every_legal_ball_in_the_kitchen_may_be_spotted_on_request() {
    let (game, before) = kitchen_game();
    // Not every legal target is in the kitchen here.
    assert_eq!(game.spot_request(&before, &table()), None);
    // Every ball above the head string, at a distinct distance from it: the
    // 15 is closest. (The rules only look at positions, not overlaps.)
    let head = table().head_string_x();
    let all_kitchen: Layout = before
        .iter()
        .map(|(b, _)| {
            let n = f64::from(b.number());
            (b, DVec2::new(head - 0.2 + 0.01 * n, -0.6 + 0.08 * n))
        })
        .collect();
    assert_eq!(game.spot_request(&all_kitchen, &table()), Some(ball(15)));
}

// --- 4.11 Stalemate ------------------------------------------------------------

#[test]
fn rule_4_11_after_a_stalemate_the_original_breaker_breaks_again() {
    let mut game = a_on_solids(Ruleset::EightBallWpa);
    assert_eq!(game.stalemate(), Effect::Rerack);
    assert_eq!(
        (game.breaker(), game.shooter(), game.stage()),
        (Seat::A, Seat::A, &Stage::Break)
    );
    assert!(game.is_open());
    assert_eq!(game.on_table().count(), 15);
}

#[test]
fn no_shot_is_accepted_while_a_decision_is_pending() {
    let mut game = Game::new(Ruleset::EightBallWpa, Seat::A);
    play(&mut game, Call::None, &[(0.1, hit(0, 1))], &rack());
    assert!(matches!(game.stage(), Stage::Decision(_)));
    assert_eq!(
        game.validate_call(Call::Safety),
        Err(RulesError::NotExpectingShot)
    );
}
