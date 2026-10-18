//! The simulation as the game sees it: the physics world, the presentation
//! clock, and free-play bookkeeping (ball in hand, undo, re-racking).
//!
//! Everything here is plain Rust on top of `cue-physics`; the Bevy systems at
//! the bottom only move data between the world and the ECS.

use bevy::prelude::*;
use cue_core::{
    BallId, Layout, ShotReport, TableSpec, eight_ball_rack, glam::DVec2, rules::eight_ball::Choice,
};
use cue_physics::{Motion, PhysicsProfile, SimError, SimEvent, Strike, World};

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

/// Game-level happenings the host may want to show.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// A fresh rack is on the table.
    Racked {
        /// `true` when the table was cleared, `false` when asked for.
        cleared: bool,
    },
    /// The last shot (or re-rack) was undone.
    Undone,
    /// A new game started (free play or eight-ball).
    NewGame,
    /// Eight-ball: the rules ruled on a shot; details are in the session.
    Ruled,
    /// Eight-ball: a pending decision was resolved.
    Decided {
        /// The option chosen.
        choice: Choice,
    },
    /// Eight-ball: a ball was put back on the table (WPA 1.5, 1.6).
    Spotted {
        /// The ball spotted.
        ball: BallId,
    },
    /// A shot was refused before it was taken, with a stable code saying why
    /// (see [`EightBall::can_shoot`](crate::EightBall::can_shoot)).
    ShotRefused(&'static str),
    /// Eight-ball: a call was refused, with the
    /// [`CallError`](cue_core::rules::eight_ball::CallError) name.
    CallRefused(&'static str),
}

/// The report of the most recent shot, for the HUD.
#[derive(Resource, Debug, Default, Clone)]
pub struct LastShot(pub Option<ShotReport>);

/// Logical id attached to every rendered ball.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ball(pub BallId);

/// Where a cue ball in hand may be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandRegion {
    /// Anywhere on the playing surface.
    Anywhere,
    /// Only above (behind) the head string.
    AboveHeadString,
}

/// Everything needed to put the table back as it was.
#[derive(Debug, Clone)]
struct Saved {
    world: World,
    rack_seed: u64,
    in_hand: Option<HandRegion>,
}

/// The physics world plus the clock that drives it.
#[derive(Resource, Debug)]
pub struct Simulation {
    /// The authoritative simulation.
    pub world: World,
    /// Presentation time, seconds. Always `≥ world.time()`; the gap is the
    /// part of the current trajectory already shown on screen.
    pub clock: f64,
    rack_seed: u64,
    in_hand: Option<HandRegion>,
    undo: Option<Box<Saved>>,
}

impl Simulation {
    /// A fresh eight-ball rack drawn from `seed`, with the cue ball in hand
    /// anywhere (free play).
    pub fn new(seed: u64) -> Self {
        Self {
            world: Self::racked_world(seed),
            clock: 0.0,
            rack_seed: seed,
            in_hand: Some(HandRegion::Anywhere),
            undo: None,
        }
    }

    fn racked_world(seed: u64) -> World {
        let profile = PhysicsProfile::baseline();
        let layout =
            eight_ball_rack(&profile.table, seed).expect("the standard table always racks");
        World::new(profile, &layout).expect("a standard rack is a valid layout")
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

    /// Whether the player may move the cue ball before the next shot: at the
    /// start of a rack and after a scratch or foul.
    pub fn in_hand(&self) -> bool {
        self.hand_region().is_some()
    }

    /// Where the cue ball in hand may go, if it is in hand right now.
    pub fn hand_region(&self) -> Option<HandRegion> {
        self.in_hand.filter(|_| self.world.is_at_rest())
    }

    /// Positions of every ball in play at the current instant, as the rules
    /// see the table before a shot.
    pub fn layout(&self) -> Layout {
        self.world
            .sample(self.clock)
            .filter(|(_, s)| s.motion != Motion::Pocketed)
            .map(|(id, s)| (id, DVec2::new(s.position.x, s.position.z)))
            .collect()
    }

    /// Racks fifteen fresh balls with the cue ball in hand in `region`, and
    /// forgets the undo history (used to start games and racks).
    pub fn new_rack(&mut self, region: HandRegion) {
        self.next_rack();
        self.in_hand = Some(region);
        self.undo = None;
    }

    /// Forgets the undo history (eight-ball does not allow undo).
    pub fn clear_undo(&mut self) {
        self.undo = None;
    }

    /// Puts the cue ball in hand in `region`, or with `None` takes it out of
    /// hand without moving it (a pocketed cue ball stays off the table while a
    /// decision is pending or the rack is over). A cue ball in hand that is
    /// pocketed or outside its region is moved to the nearest free spot: the
    /// break spot above the head string, the head spot otherwise.
    pub fn give_cue_ball(&mut self, region: Option<HandRegion>) {
        self.in_hand = region;
        let Some(region) = region else {
            return;
        };
        let table = *self.table();
        let preferred = match region {
            HandRegion::AboveHeadString => table.break_spot(),
            HandRegion::Anywhere => table.head_spot(),
        };
        let misplaced = match (self.cue_position(), region) {
            (None, _) => true,
            (Some(p), HandRegion::AboveHeadString) => !table.above_head_string(p),
            (Some(_), HandRegion::Anywhere) => false,
        };
        if misplaced
            && let Some(spot) = self.world.nearest_free_spot(BallId::CUE, preferred)
            && let Err(e) = self.world.place(BallId::CUE, spot)
        {
            warn!("could not place the cue ball: {e}");
        }
    }

    /// Puts a (possibly pocketed) ball back on the table at `at`.
    pub fn spot_ball(&mut self, ball: BallId, at: DVec2) -> Result<(), SimError> {
        self.world.place(ball, at)
    }

    /// Whether there is a shot or re-rack to undo.
    pub fn can_undo(&self) -> bool {
        self.undo.is_some()
    }

    fn snapshot(&self) -> Box<Saved> {
        Box::new(Saved {
            world: self.world.clone(),
            rack_seed: self.rack_seed,
            in_hand: self.in_hand,
        })
    }

    /// Takes a shot, remembering the table so it can be undone.
    pub fn strike(&mut self, strike: Strike) -> Result<(), SimError> {
        let before = self.snapshot();
        self.world.strike(strike)?;
        self.undo = Some(before);
        // The shot starts at the world's time; drop any presentation lag.
        self.clock = self.world.time();
        self.in_hand = None;
        Ok(())
    }

    /// Replaces the table with the next rack. Undoable.
    pub fn rerack(&mut self) {
        self.undo = Some(self.snapshot());
        self.next_rack();
    }

    fn next_rack(&mut self) {
        self.rack_seed = self.rack_seed.wrapping_add(1);
        self.world = Self::racked_world(self.rack_seed);
        self.clock = 0.0;
        self.in_hand = Some(HandRegion::Anywhere);
    }

    /// Puts the table back as it was before the last shot or re-rack, even if
    /// balls are still rolling. Returns `false` if there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        let Some(saved) = self.undo.take() else {
            return false;
        };
        self.world = saved.world;
        self.rack_seed = saved.rack_seed;
        self.in_hand = saved.in_hand;
        self.clock = self.world.time();
        true
    }

    /// Moves the cue ball while it is in hand, within its region.
    pub fn place_cue(&mut self, at: DVec2) -> Result<(), SimError> {
        match self.hand_region() {
            None => Err(SimError::InvalidPlacement("the cue ball is not in hand")),
            Some(HandRegion::AboveHeadString) if !self.table().above_head_string(at) => Err(
                SimError::InvalidPlacement("the cue ball must stay above the head string"),
            ),
            Some(_) => self.world.place(BallId::CUE, at),
        }
    }

    /// Free-play housekeeping once a shot has finished. Returns `Some(true)`
    /// when the table was cleared and re-racked.
    ///
    /// A pocketed cue ball comes back in hand at the free spot nearest the
    /// head spot. The undo snapshot of the shot is kept, so even a shot that
    /// cleared the table can be taken back.
    pub fn after_shot(&mut self, report: &ShotReport) -> Option<bool> {
        if self.world.balls_in_play().all(|b| b.is_cue()) {
            self.next_rack();
            return Some(true);
        }
        if report.cue_ball_pocketed() {
            let head = self.table().head_spot();
            match self.world.nearest_free_spot(BallId::CUE, head) {
                Some(spot) if self.world.place(BallId::CUE, spot).is_ok() => {
                    self.in_hand = Some(HandRegion::Anywhere);
                }
                _ => {
                    self.next_rack();
                    return Some(true);
                }
            }
        }
        None
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

/// Applies free-play rules after each shot.
pub(crate) fn after_shot(
    mut finished: MessageReader<ShotFinished>,
    mode: Res<crate::eight::Mode>,
    mut sim: ResMut<Simulation>,
    mut last: ResMut<LastShot>,
    mut events: MessageWriter<GameEvent>,
) {
    for ShotFinished(report) in finished.read() {
        last.0 = Some(report.clone());
        // Eight-ball has its own rules for what happens next.
        if matches!(*mode, crate::eight::Mode::FreePlay)
            && let Some(cleared) = sim.after_shot(report)
        {
            events.write(GameEvent::Racked { cleared });
        }
    }
}

/// Moves and spins every ball mesh to the simulated state at `clock`.
/// How long a pocketed ball takes to drop out of sight, seconds.
const SINK_TIME: f32 = 0.38;

/// A ball on its way down a pocket: from where it was last drawn to below
/// the pocket's centre.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sink {
    from: Vec3,
    to: Vec3,
    elapsed: f32,
}

/// Where a ball sinking for `t` of [`SINK_TIME`] is drawn, and its scale:
/// it slides to the middle of the pocket while it falls, faster and faster,
/// and shrinks a little into the dark.
pub(crate) fn sink_pose(sink: &Sink, t: f32) -> (Vec3, f32) {
    let t = t.clamp(0.0, 1.0);
    let slide = 1.0 - (1.0 - t) * (1.0 - t);
    let mut at = sink.from.lerp(sink.to, slide);
    at.y = sink.from.y + (sink.to.y - sink.from.y) * t * t;
    (at, 1.0 - 0.25 * t)
}

pub(crate) fn sync_balls(
    time: Res<Time>,
    paused: Res<Paused>,
    sim: Res<Simulation>,
    motion: Res<crate::ReducedMotion>,
    mut sinking: Local<std::collections::HashMap<BallId, Sink>>,
    mut balls: Query<(&Ball, &mut Transform, &mut Visibility)>,
) {
    let dt = if paused.0 { 0.0 } else { time.delta_secs() };
    let states: Vec<_> = sim.world.sample(sim.clock).collect();
    let r = sim.table().ball.radius as f32;
    for (ball, mut transform, mut visibility) in &mut balls {
        let Some((_, s)) = states.iter().find(|(id, _)| *id == ball.0) else {
            continue;
        };
        if s.motion == Motion::Pocketed {
            // A ball that was on show when it dropped sinks out of sight;
            // one already gone (after an undo, say) stays hidden.
            let shown = *visibility != Visibility::Hidden;
            if motion.0 {
                *visibility = Visibility::Hidden;
                continue;
            }
            let sink = sinking.entry(ball.0).or_insert_with(|| {
                let at = DVec2::new(s.position.x, s.position.z);
                let pocket = sim
                    .world
                    .geometry()
                    .pockets()
                    .iter()
                    .min_by(|a, b| a.centre.distance(at).total_cmp(&b.centre.distance(at)))
                    .map_or(at, |p| p.centre);
                Sink {
                    from: transform.translation,
                    to: Vec3::new(pocket.x as f32, -2.5 * r, pocket.y as f32),
                    elapsed: if shown { 0.0 } else { SINK_TIME },
                }
            });
            sink.elapsed += dt;
            if sink.elapsed >= SINK_TIME {
                *visibility = Visibility::Hidden;
            } else {
                let (at, scale) = sink_pose(sink, sink.elapsed / SINK_TIME);
                transform.translation = at;
                transform.scale = Vec3::splat(scale);
                *visibility = Visibility::Inherited;
            }
            continue;
        }
        sinking.remove(&ball.0);
        *visibility = Visibility::Inherited;
        transform.scale = Vec3::ONE;
        transform.translation = s.position.as_vec3();
        // Orientation is presentation only: integrate the sampled spin so the
        // markings visibly roll, slide and spin.
        let spin = Quat::from_scaled_axis(s.angular_velocity.as_vec3() * dt);
        transform.rotation = (spin * transform.rotation).normalize();
    }
}

#[cfg(test)]
mod tests {
    use cue_core::PocketId;

    use super::*;

    fn play_out(sim: &mut Simulation) -> ShotReport {
        sim.world.run_to_rest(1e6).unwrap();
        sim.world.take_report().unwrap()
    }

    #[test]
    fn a_sinking_ball_slides_into_the_pocket_falling_ever_faster() {
        let sink = Sink {
            from: Vec3::new(1.0, 0.03, 0.5),
            to: Vec3::new(1.2, -0.07, 0.6),
            elapsed: 0.0,
        };
        let (start, s0) = sink_pose(&sink, 0.0);
        let (mid, _) = sink_pose(&sink, 0.5);
        let (end, s1) = sink_pose(&sink, 1.0);
        assert_eq!((start, s0), (sink.from, 1.0));
        assert!((end - sink.to).length() < 1e-6 && s1 < 1.0);
        // Most of the slide is done by half time, most of the fall is not.
        assert!((mid.x - start.x) / (end.x - start.x) > 0.7);
        assert!((mid.y - start.y) / (end.y - start.y) < 0.3);
    }

    #[test]
    fn a_new_rack_starts_with_the_cue_ball_in_hand() {
        let mut sim = Simulation::new(3);
        assert!(sim.in_hand());
        let spot = DVec2::new(-0.8, 0.2);
        sim.place_cue(spot).unwrap();
        assert_eq!(sim.cue_position(), Some(spot));
        sim.strike(Strike::centre(DVec2::X, 0.3)).unwrap();
        assert!(!sim.in_hand(), "the first shot ends ball in hand");
        assert!(sim.place_cue(DVec2::new(-0.8, 0.0)).is_err());
    }

    #[test]
    fn undo_restores_the_table_exactly() {
        let mut sim = Simulation::new(5);
        let before: Vec<_> = sim.world.sample(0.0).collect();
        sim.strike(Strike::centre(DVec2::X, 1.0)).unwrap();
        play_out(&mut sim);
        assert!(sim.can_undo());
        assert!(sim.undo());
        let after: Vec<_> = sim.world.sample(sim.clock).collect();
        assert_eq!(before, after);
        assert!(sim.in_hand(), "ball in hand comes back too");
        assert!(!sim.undo(), "one level only");
    }

    #[test]
    fn undo_works_mid_shot_and_after_a_rerack() {
        let mut sim = Simulation::new(5);
        sim.strike(Strike::centre(DVec2::X, 1.0)).unwrap();
        sim.world.advance_to(0.2).unwrap();
        assert!(!sim.world.is_at_rest());
        assert!(sim.undo());
        assert!(sim.world.is_at_rest());

        let rack: Vec<_> = sim.world.sample(0.0).collect();
        sim.rerack();
        assert_ne!(rack, sim.world.sample(0.0).collect::<Vec<_>>());
        assert!(sim.undo());
        assert_eq!(rack, sim.world.sample(0.0).collect::<Vec<_>>());
    }

    #[test]
    fn a_scratch_puts_the_cue_ball_back_in_hand_near_the_head_spot() {
        let mut sim = Simulation::new(1);
        let centre = sim.world.geometry().pocket(PocketId::HeadNear).centre;
        let cue = sim.cue_position().unwrap();
        sim.strike(Strike::centre(centre - cue, 0.3)).unwrap();
        let report = play_out(&mut sim);
        assert!(report.cue_ball_pocketed(), "{report:?}");
        assert_eq!(sim.after_shot(&report), None);
        assert!(sim.in_hand());
        let spot = sim.cue_position().unwrap();
        assert!(spot.distance(sim.table().head_spot()) < 0.2);
    }
}
