use bevy::prelude::*;
use cue_core::{BallId, ShotReport, TableSpec, eight_ball_rack, glam::DVec2};
use cue_physics::{Motion, PhysicsProfile, SimEvent, World};

/// Longest slice of wall-clock time fed to the simulation in one frame.
///
/// A hidden tab or a debugger pause must not make balls jump: time beyond
/// this is simply dropped. Because the solver is event-driven, how the time
/// is sliced never changes the outcome — only how long it takes to watch.
const MAX_FRAME_SECONDS: f64 = 0.1;

/// While `true`, simulated time stands still and input is ignored. The host
/// page sets it through `cue-web`; rendering is throttled separately there.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Paused(pub bool);

/// Every simulation event, as it happens: collisions, cushion contacts,
/// pocketings. Audio and the web bridge listen to these.
#[derive(Message, Debug, Clone, Copy)]
pub struct Contact(pub SimEvent);

/// A shot has finished: every ball is at rest.
#[derive(Message, Debug, Clone)]
pub struct ShotFinished(pub ShotReport);

/// The report of the most recent shot, for the HUD.
#[derive(Resource, Debug, Default, Clone)]
pub struct LastShot(pub Option<ShotReport>);

/// Logical id attached to every rendered ball.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ball(pub BallId);

/// The physics world plus the clock that drives it.
#[derive(Resource, Debug)]
pub struct Simulation {
    /// The authoritative simulation.
    pub world: World,
    /// Presentation time, seconds. Always `≥ world.time()`; the gap is the
    /// part of the current trajectory already shown on screen.
    pub clock: f64,
    rack_seed: u64,
}

impl Simulation {
    /// A fresh eight-ball rack drawn from `seed`.
    pub fn new(seed: u64) -> Self {
        let profile = PhysicsProfile::baseline();
        let layout =
            eight_ball_rack(&profile.table, seed).expect("the standard table always racks");
        Self {
            world: World::new(profile, &layout).expect("a standard rack is a valid layout"),
            clock: 0.0,
            rack_seed: seed,
        }
    }

    /// Replaces the table with the next rack.
    pub fn rerack(&mut self) {
        *self = Self::new(self.rack_seed.wrapping_add(1));
    }

    /// The table being simulated.
    pub fn table(&self) -> &TableSpec {
        &self.world.profile().table
    }

    /// Cue-ball position on the table plane, if it is in play.
    pub fn cue_position(&self) -> Option<DVec2> {
        self.world
            .state(BallId::CUE)
            .filter(|s| s.motion != Motion::Pocketed)
            .map(|s| DVec2::new(s.position.x, s.position.z))
    }
}

/// Feeds elapsed time to the solver and forwards what happened.
pub(crate) fn advance(
    time: Res<Time>,
    paused: Res<Paused>,
    mut sim: ResMut<Simulation>,
    mut contacts: MessageWriter<Contact>,
    mut finished: MessageWriter<ShotFinished>,
) {
    if !paused.0 && !sim.world.is_at_rest() {
        let dt = time.delta_secs_f64().min(MAX_FRAME_SECONDS);
        let target = sim.clock + dt;
        sim.clock = target;
        if let Err(e) = sim.world.advance_to(target) {
            // Never expected; the property tests exist to keep it that way.
            error!("simulation failed, re-racking: {e}");
            sim.rerack();
            return;
        }
    }
    // Drained every frame, so the world's log never grows unbounded.
    contacts.write_batch(sim.world.drain_events().map(Contact));
    if let Some(report) = sim.world.take_report() {
        finished.write(ShotFinished(report));
    }
}

/// Free-play housekeeping after a shot: a pocketed cue ball goes back to the
/// nearest free spot to the head spot, and an empty table is re-racked.
pub(crate) fn after_shot(
    mut finished: MessageReader<ShotFinished>,
    mut sim: ResMut<Simulation>,
    mut last: ResMut<LastShot>,
) {
    for ShotFinished(report) in finished.read() {
        last.0 = Some(report.clone());
        if sim.world.balls_in_play().all(|b| b.is_cue()) {
            sim.rerack();
            continue;
        }
        if report.cue_ball_pocketed() {
            let head = sim.table().head_spot();
            match sim.world.nearest_free_spot(BallId::CUE, head) {
                Some(spot) => {
                    if let Err(e) = sim.world.place(BallId::CUE, spot) {
                        error!("could not re-spot the cue ball: {e}");
                    }
                }
                None => sim.rerack(),
            }
        }
    }
}

/// Moves and spins every ball mesh to the simulated state at `clock`.
pub(crate) fn sync_balls(
    time: Res<Time>,
    paused: Res<Paused>,
    sim: Res<Simulation>,
    mut balls: Query<(&Ball, &mut Transform, &mut Visibility)>,
) {
    let dt = if paused.0 { 0.0 } else { time.delta_secs() };
    let states: Vec<_> = sim.world.sample(sim.clock).collect();
    for (ball, mut transform, mut visibility) in &mut balls {
        let Some((_, s)) = states.iter().find(|(id, _)| *id == ball.0) else {
            continue;
        };
        *visibility = if s.motion == Motion::Pocketed {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        transform.translation = s.position.as_vec3();
        // Orientation is presentation only: integrate the sampled spin so the
        // markings visibly roll, slide and spin.
        let spin = Quat::from_scaled_axis(s.angular_velocity.as_vec3() * dt);
        transform.rotation = (spin * transform.rotation).normalize();
    }
}
