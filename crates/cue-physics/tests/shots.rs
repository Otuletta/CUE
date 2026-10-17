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
    BallState, Cushion, Motion, PhysicsProfile, Rail, SimError, SimEvent, SimEventKind,
    SpeedFriction, Strike, World,
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

/// Cue and object ball directions right after a half-ball stun hit.
fn half_ball_stun(profile: PhysicsProfile) -> (DVec2, DVec2) {
    let r = profile.table.ball.radius;
    // Aim at a point one radius off the object's centre.
    let l = layout(&[(0, -0.4, 0.0), (1, 0.0, r)]);
    let mut w = World::new(profile, &l).unwrap();
    w.strike(Strike::centre(DVec2::X, 1.0)).unwrap();
    until_event(&mut w, is_ball_ball);
    (
        planar(w.state(BallId::CUE).unwrap().velocity),
        planar(w.state(ball(1)).unwrap().velocity),
    )
}

#[test]
fn stun_cut_obeys_the_ninety_degree_rule() {
    // The textbook rule holds exactly for frictionless, perfectly elastic
    // balls...
    let mut ideal = PhysicsProfile::baseline();
    ideal.ball_ball.restitution = 1.0;
    ideal.ball_ball.friction = SpeedFriction::NONE;
    let (cue, object) = half_ball_stun(ideal);
    assert_relative_eq!(
        cue.angle_to(object).abs().to_degrees(),
        90.0,
        epsilon = 1e-6
    );

    // ...and approximately for real ones: inelasticity and throw close the
    // angle by a degree or two.
    let (cue, object) = half_ball_stun(PhysicsProfile::baseline());
    let real = cue.angle_to(object).abs().to_degrees();
    assert!(real < 90.0 && real > 85.0, "separation angle {real}°");
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
fn cushion_rebound_loses_speed_and_keeps_the_tangential_direction() {
    // A stun shot into the foot rail at a shallow angle.
    let mut w = world(&[(0, 0.0, 0.0)]);
    w.strike(Strike::centre(DVec2::new(1.0, 0.2).normalize(), 0.6))
        .unwrap();
    loop {
        let t = w.next_event_time().unwrap();
        // Sampling at the event time gives the state just before resolution.
        let incoming = w.sample(t).next().unwrap().1;
        w.advance_to(t).unwrap();
        let hit = w.drain_events().find_map(|e| match e.kind {
            SimEventKind::Cushion { cushion, .. } => Some(cushion),
            _ => None,
        });
        if let Some(cushion) = hit {
            assert_eq!(cushion, Cushion::Rail(Rail::Foot));
            let after = w.state(BallId::CUE).unwrap();
            let ratio = -after.velocity.x / incoming.velocity.x;
            // Han-style contact at nose height with e = 0.85: the horizontal
            // rebound ratio lands between 0.6 and 0.85.
            assert!(ratio > 0.6 && ratio < 0.85, "rebound ratio {ratio}");
            assert!(after.velocity.z > 0.0, "keeps travelling along the rail");
            // The contact above the equator gives the ball spin it did not
            // have, so it leaves the cushion sliding.
            assert_eq!(after.motion, Motion::Sliding);
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
    assert!(
        end.iter()
            .all(|(_, s)| matches!(s.motion, Motion::Stationary | Motion::Pocketed))
    );
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

/// Spec §24.2 case 14: a thousand consecutive shots, each taken from
/// wherever the cue ball stopped. A scratched cue ball is re-spotted and an
/// empty table re-racked, as in free play. Slow in debug builds, so opt-in:
/// `cargo test -p cue-physics --release -- --ignored`.
#[test]
#[ignore = "stress test; run with --release -- --ignored"]
fn a_thousand_consecutive_shots_stay_physical() {
    let table = TableSpec::nine_foot();
    let profile = PhysicsProfile::baseline();
    let mut racks = 1u64;
    let mut w = World::new(profile, &eight_ball_rack(&table, racks).unwrap()).unwrap();
    let mut rng = cue_core::SplitMix64::new(2026);
    let mut unit = move || rng.next_u64() as f64 / u64::MAX as f64;
    let (mut events, mut pocketed, mut simulated) = (0usize, 0usize, 0.0);
    let started = std::time::Instant::now();
    for shot in 0..1000 {
        if w.balls_in_play().all(|b| b.is_cue()) {
            simulated += w.time();
            racks += 1;
            w = World::new(profile, &eight_ball_rack(&table, racks).unwrap()).unwrap();
        }
        if w.state(BallId::CUE).unwrap().motion == Motion::Pocketed {
            let spot = w.nearest_free_spot(BallId::CUE, table.head_spot()).unwrap();
            w.place(BallId::CUE, spot).unwrap();
        }
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
        pocketed += w.take_report().unwrap().pocketed().count();
    }
    simulated += w.time();
    eprintln!(
        "1000 shots, {racks} racks, {pocketed} balls pocketed, {events} events, \
         {simulated:.0} s simulated in {:.0} ms",
        started.elapsed().as_secs_f64() * 1e3
    );
}
