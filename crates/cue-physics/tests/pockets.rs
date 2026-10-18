//! Pockets, shot reports and ball placement (spec §11.10, §24.2 cases 7–8,
//! §24.3). Each test states a behaviour a player would recognise.

// Helpers outside `#[test]` fns are not covered by `allow-unwrap-in-tests`.
#![allow(clippy::unwrap_used)]

use std::f64::consts::FRAC_1_SQRT_2;

use cue_core::{BallId, Layout, PocketId, ShotReport, TableSpec, eight_ball_rack, glam::DVec2};
use cue_physics::{Motion, PhysicsProfile, SimError, Strike, World};

const R: f64 = 0.028575;

fn ball(n: u8) -> BallId {
    BallId::new(n).unwrap()
}

fn world(balls: &[(u8, DVec2)]) -> World {
    let layout: Layout = balls.iter().map(|&(n, p)| (ball(n), p)).collect();
    World::new(PhysicsProfile::baseline(), &layout).unwrap()
}

/// Plays one shot to rest and returns its report.
fn play(w: &mut World, strike: Strike) -> ShotReport {
    w.strike(strike).unwrap();
    while let Some(t) = w.next_event_time() {
        w.advance_to(t).unwrap();
        w.check_invariants().unwrap();
    }
    w.take_report().unwrap()
}

/// A rolling (natural-roll) shot.
fn rolling(direction: DVec2, power: f64) -> Strike {
    Strike {
        direction: direction.normalize(),
        power,
        tip_offset: DVec2::new(0.0, 0.4),
    }
}

fn half() -> DVec2 {
    TableSpec::nine_foot().half_extents()
}

#[test]
fn a_ball_rolled_straight_at_a_corner_pocket_drops() {
    let probe = world(&[(0, DVec2::ZERO)]);
    let centre = probe.geometry().pocket(PocketId::FootNear).centre;
    let start = centre - DVec2::splat(0.6 * FRAC_1_SQRT_2);
    let mut w = world(&[(0, start)]);
    let report = play(&mut w, rolling(centre - start, 0.25));
    assert_eq!(
        report.pocketed().collect::<Vec<_>>(),
        vec![(BallId::CUE, PocketId::FootNear)]
    );
    assert!(report.cue_ball_pocketed());
    assert_eq!(w.state(BallId::CUE).unwrap().motion, Motion::Pocketed);
}

#[test]
fn a_ball_aimed_at_the_point_of_the_pocket_bounces_out() {
    let jaw = world(&[(0, DVec2::ZERO)])
        .geometry()
        .pocket(PocketId::FootNear)
        .jaws[0];
    let start = jaw - DVec2::splat(0.5 * FRAC_1_SQRT_2);
    let mut w = world(&[(0, start)]);
    let report = play(&mut w, rolling(jaw - start, 0.25));
    assert_eq!(report.pocketed().count(), 0, "{report:?}");
    assert!(!report.events.is_empty(), "it must have hit the cushion");
}

#[test]
fn a_ball_rolling_along_the_rail_drops_in_the_corner() {
    // Frozen to the near rail, rolled at medium speed towards the foot.
    let z = half().y - R - 1e-6;
    let mut w = world(&[(0, DVec2::new(0.3, z))]);
    let report = play(&mut w, rolling(DVec2::X, 0.3));
    assert_eq!(
        report.pocketed().collect::<Vec<_>>(),
        vec![(BallId::CUE, PocketId::FootNear)]
    );
}

#[test]
fn a_ball_rolled_straight_into_a_side_pocket_drops() {
    let mut w = world(&[(0, DVec2::new(0.0, 0.1))]);
    let report = play(&mut w, rolling(DVec2::Y, 0.25));
    assert_eq!(
        report.pocketed().collect::<Vec<_>>(),
        vec![(BallId::CUE, PocketId::SideNear)]
    );
}

#[test]
fn side_pockets_take_moderate_angles_and_reject_shallow_ones() {
    // Both shots cross the cushion-nose line at the middle of the mouth; only
    // the steeper one fits past the far jaw.
    let side = |degrees: f64| {
        let line = DVec2::new(0.0, half().y - R);
        let dir = DVec2::from_angle(degrees.to_radians());
        let start = line - dir * 0.6;
        let mut w = world(&[(0, start)]);
        play(&mut w, rolling(dir, 0.15)).pocketed().count()
    };
    assert_eq!(side(45.0), 1, "45° into the side pocket should drop");
    assert_eq!(side(15.0), 0, "15° along the rail should be rejected");
}

#[test]
fn break_report_names_the_first_ball_hit_and_the_cushion_count() {
    let table = TableSpec::nine_foot();
    let rack = eight_ball_rack(&table, 11).unwrap();
    let apex = rack
        .iter()
        .filter(|(b, _)| !b.is_cue())
        .min_by(|a, b| a.1.x.total_cmp(&b.1.x))
        .unwrap()
        .0;
    let mut w = World::new(PhysicsProfile::baseline(), &rack).unwrap();
    let report = play(
        &mut w,
        Strike {
            direction: DVec2::X,
            power: 1.0,
            tip_offset: DVec2::new(0.0, 0.1),
        },
    );
    assert_eq!(report.shot_id, 1);
    assert_eq!(report.first_contacts(), vec![apex]);
    let to_cushion = report.object_balls_driven_to_a_rail().len();
    assert!(
        to_cushion >= 4,
        "a full-power break drives {to_cushion} balls to a rail"
    );
    assert!(report.cushion_after_first_contact());
    // Times are relative to the strike and in order.
    assert!(report.events.windows(2).all(|w| w[0].time <= w[1].time));
    assert!(report.events.first().unwrap().time > 0.0);
}

#[test]
fn a_scratched_cue_ball_must_be_placed_before_the_next_shot() {
    let probe = world(&[(0, DVec2::ZERO)]);
    let centre = probe.geometry().pocket(PocketId::SideNear).centre;
    let mut w = world(&[(0, DVec2::new(0.0, 0.2)), (1, DVec2::new(-0.6, 0.0))]);
    let report = play(&mut w, rolling(centre - DVec2::new(0.0, 0.2), 0.3));
    assert!(report.cue_ball_pocketed());
    assert_eq!(
        w.strike(Strike::centre(DVec2::X, 0.5)),
        Err(SimError::CueBallUnavailable)
    );
    let table = TableSpec::nine_foot();
    let spot = w.nearest_free_spot(BallId::CUE, table.head_spot()).unwrap();
    w.place(BallId::CUE, spot).unwrap();
    let second = play(&mut w, Strike::centre(DVec2::X, 0.2));
    assert_eq!(second.shot_id, 2);
}

#[test]
fn placement_is_validated() {
    let mut w = world(&[(0, DVec2::new(-0.5, 0.0)), (1, DVec2::new(0.3, 0.0))]);
    assert!(
        w.place(BallId::CUE, DVec2::new(5.0, 0.0)).is_err(),
        "off the table"
    );
    assert!(
        w.place(BallId::CUE, DVec2::new(0.3 + R, 0.0)).is_err(),
        "overlapping"
    );
    assert!(w.place(BallId::CUE, DVec2::new(0.0, 0.3)).is_ok());
    w.strike(Strike::centre(DVec2::X, 0.3)).unwrap();
    assert_eq!(
        w.place(BallId::CUE, DVec2::new(-0.5, 0.0)),
        Err(SimError::NotAtRest)
    );
}

#[test]
fn nearest_free_spot_steps_around_an_occupied_spot() {
    let table = TableSpec::nine_foot();
    let w = world(&[(0, DVec2::new(0.5, 0.0)), (1, table.head_spot())]);
    let spot = w.nearest_free_spot(BallId::CUE, table.head_spot()).unwrap();
    assert!(spot.distance(table.head_spot()) >= 2.0 * R);
    assert!(
        spot.distance(table.head_spot()) < 4.0 * R,
        "should be close"
    );
}

#[test]
fn the_head_string_sensor_reports_each_crossing_of_the_cue_ball() {
    use cue_core::ShotEventKind;
    let crossings = |r: &ShotReport| {
        r.events
            .iter()
            .filter(|e| e.kind == ShotEventKind::CueCrossedHeadString)
            .count()
    };
    let table = TableSpec::nine_foot();

    // A break: the cue ball leaves the kitchen before it reaches the rack.
    let mut w = World::new(
        PhysicsProfile::baseline(),
        &eight_ball_rack(&table, 3).unwrap(),
    )
    .unwrap();
    let report = play(&mut w, Strike::centre(DVec2::X, 1.0));
    let t0 = report.first_contact_time().unwrap();
    assert!(report.cue_crossed_head_string_before(t0));

    // From just below the string, into the kitchen, off the head rail and
    // back out: two crossings.
    let start = DVec2::new(table.head_string_x() + 0.1, 0.0);
    let mut w = world(&[(0, start)]);
    let report = play(&mut w, Strike::centre(DVec2::NEG_X, 0.25));
    let end = w.state(BallId::CUE).unwrap().position.x;
    assert!(
        end > table.head_string_x(),
        "rolled back out of the kitchen"
    );
    assert_eq!(crossings(&report), 2, "{report:?}");
}

/// Cushion contacts in a report as `(ball, rail, frozen)`.
fn cushions(report: &ShotReport) -> Vec<(u8, cue_core::Rail, bool)> {
    report
        .events
        .iter()
        .filter_map(|e| match e.kind {
            cue_core::ShotEventKind::Cushion { ball, rail, frozen } => {
                Some((ball.number(), rail, frozen))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_frozen_ball_struck_into_its_own_rail_is_reported_as_frozen() {
    use cue_core::Rail;
    // The 5 rests against the near rail; the cue ball hits it full into it.
    // The 5 is pinned between the rail and the cue ball: it bounces between
    // them without ever leaving the rail, so none of its contacts drive it to
    // a rail (WPA 2.7).
    let z = half().y - R - 1e-6;
    let mut w = world(&[(0, DVec2::new(0.4, z - 0.4)), (5, DVec2::new(0.4, z))]);
    let report = play(&mut w, Strike::centre(DVec2::Y, 0.15));
    let contacts = cushions(&report);
    let pinned: Vec<_> = contacts.iter().filter(|c| c.0 == 5).collect();
    assert!(!pinned.is_empty(), "{report:?}");
    assert!(
        pinned.iter().all(|c| c.1 == Rail::Near && c.2),
        "{contacts:?}"
    );
    assert!(
        !report.cushion_after_first_contact(),
        "no ball was driven to a rail"
    );
}

#[test]
fn a_frozen_ball_that_leaves_its_rail_and_returns_is_driven_to_it() {
    use cue_core::Rail;
    // The cue ball rests against the near rail and is driven into it: the
    // first contact is a press, every later one a real rail.
    let z = half().y - R - 1e-6;
    let mut w = world(&[(0, DVec2::new(0.4, z))]);
    let report = play(&mut w, Strike::centre(DVec2::Y, 0.6));
    let contacts = cushions(&report);
    assert_eq!(
        contacts.first(),
        Some(&(0, Rail::Near, true)),
        "{contacts:?}"
    );
    assert!(
        contacts.len() >= 3,
        "it crosses the table and back: {contacts:?}"
    );
    assert!(contacts[1..].iter().all(|c| !c.2), "{contacts:?}");
    assert!(contacts.contains(&(0, Rail::Far, false)));
    assert!(contacts[1..].contains(&(0, Rail::Near, false)));

    // A ball not touching any rail is never frozen.
    let mut w = world(&[(0, DVec2::new(0.4, 0.0))]);
    let report = play(&mut w, Strike::centre(DVec2::Y, 0.2));
    assert!(cushions(&report).iter().all(|c| !c.2));
}
