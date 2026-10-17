//! Reference shots from spec §24.2. Each test is a physical claim CUE makes;
//! if one fails, the claim is no longer true.

// Helpers outside `#[test]` fns are not covered by `allow-unwrap-in-tests`.
#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use cue_core::{
    BallId, Layout, TableSpec, eight_ball_rack,
    glam::{DVec2, DVec3},
};
use cue_physics::{
    BallState, Motion, PhysicsProfile, SimError, SimEvent, SimEventKind, Strike, World,
};
use proptest::prelude::*;

fn ball(n: u8) -> BallId {
    BallId::new(n).unwrap()
}

fn layout(balls: &[(u8, f64, f64)]) -> Layout {
    balls
        .iter()
        .map(|&(n, x, z)| (ball(n), DVec2::new(x, z)))
        .collect()
}

fn world(balls: &[(u8, f64, f64)]) -> World {
    World::new(PhysicsProfile::baseline(), &layout(balls)).unwrap()
}

fn planar(v: DVec3) -> DVec2 {
    DVec2::new(v.x, v.z)
}

/// Advances event by event until `pick` matches one; returns it.
fn until_event(w: &mut World, pick: impl Fn(&SimEvent) -> bool) -> SimEvent {
    loop {
        let t = w.next_event_time().expect("the event never happened");
        w.advance_to(t).unwrap();
        if let Some(e) = w.drain_events().find(|e| pick(e)) {
            return e;
        }
    }
}

fn is_ball_ball(e: &SimEvent) -> bool {
    matches!(e.kind, SimEventKind::BallBall { .. })
}

#[test]
fn head_on_stun_shot_stops_the_cue_ball() {
    let mut w = world(&[(0, -0.3, 0.0), (1, 0.0, 0.0)]);
    w.strike(Strike::centre(DVec2::X, 1.0)).unwrap();
    let hit = until_event(&mut w, is_ball_ball);
    let SimEventKind::BallBall { normal_speed, .. } = hit.kind else {
        unreachable!()
    };

    let e = w.profile().ball_ball.restitution;
    let cue = w.state(BallId::CUE).unwrap().velocity.x;
    let object = w.state(ball(1)).unwrap().velocity.x;
    assert_relative_eq!(object, 0.5 * (1.0 + e) * normal_speed, epsilon = 1e-9);
    assert_relative_eq!(cue, 0.5 * (1.0 - e) * normal_speed, epsilon = 1e-9);
    // Momentum along the line of centres is conserved.
    assert_relative_eq!(cue + object, normal_speed, epsilon = 1e-9);
}

#[test]
fn stun_cut_obeys_the_ninety_degree_rule() {
    let mut profile = PhysicsProfile::baseline();
    profile.ball_ball.restitution = 1.0;
    let r = profile.table.ball.radius;
    // Half-ball hit: aim at a point one radius off the object's centre.
    let l = layout(&[(0, -0.4, 0.0), (1, 0.0, r)]);
    let mut w = World::new(profile, &l).unwrap();
    w.strike(Strike::centre(DVec2::X, 1.0)).unwrap();
    until_event(&mut w, is_ball_ball);

    let cue = planar(w.state(BallId::CUE).unwrap().velocity);
    let object = planar(w.state(ball(1)).unwrap().velocity);
    let angle = cue.angle_to(object).abs().to_degrees();
    assert_relative_eq!(angle, 90.0, epsilon = 1e-6);
}

#[test]
fn follow_carries_the_cue_ball_forward_and_draw_pulls_it_back() {
    // After a head-on hit the cue ball keeps its spin; cloth friction then
    // turns that spin into motion. Measure the velocity once it rolls again.
    let rolling_velocity_after_contact = |tip: f64| {
        let mut w = world(&[(0, -0.6, 0.0), (1, 0.0, 0.0)]);
        w.strike(Strike {
            direction: DVec2::X,
            power: 0.4,
            tip_offset: DVec2::new(0.0, tip),
        })
        .unwrap();
        until_event(&mut w, is_ball_ball);
        until_event(
            &mut w,
            |e| matches!(e.kind, SimEventKind::Transition { ball, from: Motion::Sliding, .. } if ball.is_cue()),
        );
        w.state(BallId::CUE).unwrap().velocity.x
    };
    let follow = rolling_velocity_after_contact(0.4);
    let stun = rolling_velocity_after_contact(0.0);
    let draw = rolling_velocity_after_contact(-0.5);
    assert!(follow > 0.3, "follow must roll forward: {follow}");
    assert!(draw < -0.3, "draw must come back: {draw}");
    assert!(
        stun.abs() < follow.abs() && stun.abs() < draw.abs(),
        "stun: {stun}"
    );
}

#[test]
fn cushion_reverses_the_normal_component_with_restitution() {
    let mut w = world(&[(0, 0.0, 0.0)]);
    w.strike(Strike::centre(DVec2::new(1.0, 0.2).normalize(), 0.6))
        .unwrap();
    loop {
        let t = w.next_event_time().unwrap();
        // Sampling at the event time gives the state just before resolution.
        let incoming = w.sample(t).next().unwrap().1.velocity;
        w.advance_to(t).unwrap();
        if w.drain_events()
            .any(|e| matches!(e.kind, SimEventKind::Cushion { .. }))
        {
            let after = w.state(BallId::CUE).unwrap().velocity;
            let e = w.profile().cushion.restitution;
            assert!(incoming.x > 0.0, "the foot rail is reached first");
            assert_relative_eq!(after.x, -e * incoming.x, epsilon = 1e-12);
            assert_relative_eq!(after.z, incoming.z, epsilon = 1e-12);
            return;
        }
    }
}

#[test]
fn cannot_strike_while_balls_move() {
    let mut w = world(&[(0, 0.0, 0.0)]);
    w.strike(Strike::centre(DVec2::X, 0.3)).unwrap();
    assert_eq!(
        w.strike(Strike::centre(DVec2::X, 0.3)),
        Err(SimError::NotAtRest)
    );
}

#[test]
fn sampling_never_changes_the_state() {
    let mut w = world(&[(0, 0.0, 0.0)]);
    w.strike(Strike::centre(DVec2::X, 0.3)).unwrap();
    let before = *w.state(BallId::CUE).unwrap();
    let mid = w.next_event_time().unwrap() * 0.5;
    let sampled: Vec<_> = w.sample(mid).collect();
    assert_eq!(*w.state(BallId::CUE).unwrap(), before);
    assert!(sampled[0].1.position.x > before.position.x);
}

#[test]
fn rejects_overlapping_or_off_table_layouts() {
    let p = PhysicsProfile::baseline();
    assert!(World::new(p, &layout(&[(0, 0.0, 0.0), (1, 0.01, 0.0)])).is_err());
    assert!(World::new(p, &layout(&[(0, 5.0, 0.0)])).is_err());
}

/// Plays a break, advancing with a given sequence of frame lengths, and
/// returns every event and the final state.
fn play_break(frame: impl Fn(usize) -> f64) -> (Vec<SimEvent>, Vec<(BallId, BallState)>) {
    let table = TableSpec::nine_foot();
    let mut w = World::new(
        PhysicsProfile::baseline(),
        &eight_ball_rack(&table, 7).unwrap(),
    )
    .unwrap();
    w.strike(Strike {
        direction: DVec2::new(1.0, 0.004),
        power: 1.0,
        tip_offset: DVec2::new(0.0, 0.1),
    })
    .unwrap();
    let mut events = Vec::new();
    let mut now = 0.0;
    let mut n = 0;
    while !w.is_at_rest() {
        now += frame(n);
        n += 1;
        w.advance_to(now).unwrap();
        w.check_invariants().unwrap();
        // Rendering would sample here; it must not perturb anything.
        let _ = w.sample(now).count();
        events.extend(w.drain_events());
        assert!(now < 120.0, "break never came to rest");
    }
    (events, w.sample(now).collect())
}

#[test]
fn break_outcome_is_identical_at_any_frame_rate() {
    let reference = play_break(|_| 1.0 / 60.0);
    assert!(reference.0.len() > 50, "a break should produce many events");
    for frame in [1.0 / 30.0, 1.0 / 144.0, 1.0 / 7.0] {
        assert_eq!(play_break(|_| frame), reference, "frame = {frame}");
    }
    // Irregular frame pacing, as on a real browser.
    let jitter = |n: usize| [0.004, 0.031, 0.0167, 0.009, 0.05][n % 5];
    assert_eq!(play_break(jitter), reference);
}

#[test]
fn break_moves_the_rack_and_settles() {
    let (events, end) = play_break(|_| 1.0 / 60.0);
    let collisions = events.iter().filter(|e| is_ball_ball(e)).count();
    let cushions = events
        .iter()
        .filter(|e| matches!(e.kind, SimEventKind::Cushion { .. }))
        .count();
    assert!(collisions > 15, "only {collisions} collisions");
    assert!(cushions > 4, "only {cushions} cushion contacts");
    assert!(end.iter().all(|(_, s)| s.motion == Motion::Stationary));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Any legal strike on a full rack keeps every invariant at every event
    /// and eventually comes to rest.
    #[test]
    fn random_strikes_never_break_invariants(
        angle in 0.0..std::f64::consts::TAU,
        power in 0.05..=1.0f64,
        tip_r in 0.0..=0.5f64,
        tip_a in 0.0..std::f64::consts::TAU,
        seed: u64,
    ) {
        let table = TableSpec::nine_foot();
        let mut w = World::new(PhysicsProfile::baseline(), &eight_ball_rack(&table, seed).unwrap()).unwrap();
        w.strike(Strike {
            direction: DVec2::from_angle(angle),
            power,
            tip_offset: DVec2::from_angle(tip_a) * tip_r,
        }).unwrap();
        while let Some(t) = w.next_event_time() {
            prop_assert!(t < 120.0, "did not come to rest");
            w.advance_to(t).unwrap();
            if let Err(e) = w.check_invariants() {
                prop_assert!(false, "{}", e);
            }
        }
    }
}

/// Spec §24.2 case 14: a thousand consecutive shots on one table, each taken
/// from wherever the cue ball stopped. Slow in debug builds, so opt-in:
/// `cargo test -p cue-physics --release -- --ignored`.
#[test]
#[ignore = "stress test; run with --release -- --ignored"]
fn a_thousand_consecutive_shots_stay_physical() {
    let table = TableSpec::nine_foot();
    let mut w = World::new(
        PhysicsProfile::baseline(),
        &eight_ball_rack(&table, 1).unwrap(),
    )
    .unwrap();
    let mut rng = cue_core::SplitMix64::new(2026);
    let mut unit = move || rng.next_u64() as f64 / u64::MAX as f64;
    let mut events = 0usize;
    for shot in 0..1000 {
        w.strike(Strike {
            direction: DVec2::from_angle(unit() * std::f64::consts::TAU),
            power: 0.05 + 0.95 * unit(),
            tip_offset: DVec2::from_angle(unit() * std::f64::consts::TAU) * 0.5 * unit(),
        })
        .unwrap();
        let start = w.time();
        while let Some(t) = w.next_event_time() {
            assert!(t - start < 120.0, "shot {shot} never came to rest");
            w.advance_to(t).unwrap();
            if let Err(e) = w.check_invariants() {
                panic!("shot {shot}: {e}");
            }
        }
        events += w.drain_events().count();
    }
    eprintln!("1000 shots, {events} events, {:.1} s simulated", w.time());
}
