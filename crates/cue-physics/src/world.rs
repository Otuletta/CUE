//! The event-driven simulation.
//!
//! The authoritative state only ever changes **at events**: a collision, a
//! cushion contact or a regime change. Between events every ball follows a
//! closed-form trajectory, so the renderer can [`sample`](World::sample) any
//! instant without touching the state. As a consequence, the outcome of a shot
//! is bit-for-bit independent of the frame rate that drives it.

use cue_core::{
    BallId, Layout,
    glam::{DVec2, DVec3},
};
use thiserror::Error;

use crate::{
    BallState, Motion, PhysicsProfile, Strike,
    motion::{self, Kinematics, Trajectory, planar},
    poly::Poly,
};

/// Smallest separating speed after any contact, m/s.
///
/// Friction can press two touching balls (or a ball and a cushion) together.
/// An impulse model answers each press with a smaller bounce, which would
/// otherwise collapse into an unbounded sequence of ever-shorter collisions.
/// Guaranteeing 0.1 mm/s of separation bounds that cascade while adding about
/// a nanojoule — far below anything visible or audible.
const MIN_SEPARATION_SPEED: f64 = 1e-4;

/// Upper bound on events processed by a single [`World::advance_to`] call.
/// Real shots stay orders of magnitude below it; reaching it means a bug.
const MAX_EVENTS_PER_ADVANCE: usize = 100_000;

/// Errors raised by the simulation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SimError {
    /// The initial layout cannot be simulated.
    #[error("invalid layout: {0}")]
    InvalidLayout(String),
    /// A strike was rejected before touching the state.
    #[error("invalid strike: {0}")]
    InvalidStrike(&'static str),
    /// A strike was attempted while balls are still moving.
    #[error("balls are still moving")]
    NotAtRest,
    /// The cue ball is not on the table.
    #[error("the cue ball is not on the table")]
    CueBallUnavailable,
    /// Time cannot run backwards.
    #[error("cannot advance to {target} s: simulation is already at {now} s")]
    TimeTravel {
        /// Requested time.
        target: f64,
        /// Current simulation time.
        now: f64,
    },
    /// The event loop failed to make progress. Always a bug.
    #[error("simulation stalled after {0} events at t = {1} s")]
    Stalled(usize, f64),
}

/// One of the four cushions of the rectangular playing surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rail {
    /// Short rail behind the head string (`−X`).
    Head,
    /// Short rail behind the rack (`+X`).
    Foot,
    /// Long rail at `−Z`.
    Far,
    /// Long rail at `+Z`.
    Near,
}

impl Rail {
    const ALL: [Self; 4] = [Self::Head, Self::Foot, Self::Far, Self::Near];

    /// Unit normal pointing into the playing surface.
    fn inward_normal(self) -> DVec2 {
        match self {
            Self::Head => DVec2::X,
            Self::Foot => DVec2::NEG_X,
            Self::Far => DVec2::Y,
            Self::Near => DVec2::NEG_Y,
        }
    }
}

/// Something that happened during the simulation, with its exact time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimEvent {
    /// Simulation time in seconds.
    pub time: f64,
    /// What happened.
    pub kind: SimEventKind,
}

/// The kinds of [`SimEvent`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimEventKind {
    /// The cue struck the cue ball.
    Struck {
        /// Cue-ball speed right after the hit, m/s.
        speed: f64,
    },
    /// Two balls collided.
    BallBall {
        /// Lower-numbered ball.
        a: BallId,
        /// Higher-numbered ball.
        b: BallId,
        /// Closing speed along the line of centres, m/s.
        normal_speed: f64,
    },
    /// A ball hit a cushion.
    Cushion {
        /// The ball.
        ball: BallId,
        /// The cushion.
        rail: Rail,
        /// Speed into the cushion, m/s.
        normal_speed: f64,
    },
    /// A ball changed motion regime.
    Transition {
        /// The ball.
        ball: BallId,
        /// Previous regime.
        from: Motion,
        /// New regime.
        to: Motion,
    },
}

/// The next scheduled event, relative to the current simulation time.
#[derive(Clone, Copy, Debug)]
struct Pending {
    dt: f64,
    what: Scheduled,
}

#[derive(Clone, Copy, Debug)]
enum Scheduled {
    Transition(usize),
    Cushion(usize, Rail),
    Pair(usize, usize),
}

/// A table of balls and the rules of motion that govern them.
#[derive(Clone, Debug)]
pub struct World {
    profile: PhysicsProfile,
    k: Kinematics,
    /// Ball ids, ascending. Index `i` here is index `i` in `balls`.
    ids: Vec<BallId>,
    balls: Vec<BallState>,
    time: f64,
    next: Option<Pending>,
    log: Vec<SimEvent>,
}

impl World {
    /// Builds a world with every ball of `layout` at rest.
    pub fn new(profile: PhysicsProfile, layout: &Layout) -> Result<Self, SimError> {
        profile
            .table
            .validate()
            .map_err(|e| SimError::InvalidLayout(e.to_string()))?;
        let r = profile.table.ball.radius;
        let (ids, balls): (Vec<_>, Vec<_>) = layout
            .iter()
            .map(|(id, p)| (id, BallState::at_rest(p.x, p.y, r)))
            .unzip();

        for (i, (id, p)) in layout.iter().enumerate() {
            if !profile.table.contains_ball(p) {
                return Err(SimError::InvalidLayout(format!(
                    "{id} is off the playing surface"
                )));
            }
            for (other, q) in layout.iter().skip(i + 1) {
                if p.distance(q) < 2.0 * r {
                    return Err(SimError::InvalidLayout(format!("{id} overlaps {other}")));
                }
            }
        }

        Ok(Self {
            k: Kinematics::new(&profile),
            profile,
            ids,
            balls,
            time: 0.0,
            next: None,
            log: Vec::new(),
        })
    }

    /// The parameters this world runs with.
    pub fn profile(&self) -> &PhysicsProfile {
        &self.profile
    }

    /// Current simulation time in seconds.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Whether every ball has stopped.
    pub fn is_at_rest(&self) -> bool {
        self.next.is_none()
    }

    /// Absolute time of the next scheduled event, if any.
    pub fn next_event_time(&self) -> Option<f64> {
        self.next.map(|p| self.time + p.dt)
    }

    /// Authoritative state of a ball at the current simulation time.
    pub fn state(&self, id: BallId) -> Option<&BallState> {
        self.index(id).map(|i| &self.balls[i])
    }

    /// Hits the cue ball. Only allowed when everything is at rest.
    pub fn strike(&mut self, strike: Strike) -> Result<(), SimError> {
        strike.validate()?;
        if !self.is_at_rest() {
            return Err(SimError::NotAtRest);
        }
        let i = self
            .index(BallId::CUE)
            .filter(|&i| self.balls[i].motion != Motion::Pocketed)
            .ok_or(SimError::CueBallUnavailable)?;

        let speed = strike.power * self.profile.cue.max_speed;
        let (v, w) = strike.velocities(speed, self.k.radius);
        let mut cue = self.balls[i];
        cue.velocity = v;
        cue.angular_velocity = w;
        self.balls[i] = motion::settle(&cue, &self.k);

        self.push(SimEventKind::Struck { speed });
        self.next = self.schedule();
        Ok(())
    }

    /// Processes every event up to and including `target` seconds.
    ///
    /// The state is left at the last processed event, not at `target`; use
    /// [`sample`](Self::sample) to draw intermediate instants.
    pub fn advance_to(&mut self, target: f64) -> Result<(), SimError> {
        if target < self.time {
            return Err(SimError::TimeTravel {
                target,
                now: self.time,
            });
        }
        let mut processed = 0;
        while let Some(event) = self.next.filter(|p| self.time + p.dt <= target) {
            processed += 1;
            if processed > MAX_EVENTS_PER_ADVANCE {
                return Err(SimError::Stalled(processed, self.time));
            }
            for ball in &mut self.balls {
                *ball = motion::evolve(ball, event.dt, &self.k);
            }
            self.time += event.dt;
            self.resolve(event.what);
            self.next = self.schedule();
        }
        Ok(())
    }

    /// Runs until every ball stops, or until `limit` seconds of simulated
    /// time. Intended for tests, tools and headless evaluation.
    pub fn run_to_rest(&mut self, limit: f64) -> Result<(), SimError> {
        while let Some(t) = self.next_event_time() {
            if t > limit {
                break;
            }
            self.advance_to(t)?;
        }
        Ok(())
    }

    /// Every ball's state at time `t`, without changing the simulation.
    ///
    /// `t` is clamped to `[time(), next_event_time()]`, the interval over
    /// which the closed-form trajectories are valid.
    pub fn sample(&self, t: f64) -> impl Iterator<Item = (BallId, BallState)> + '_ {
        let horizon = self.next_event_time().unwrap_or(f64::INFINITY);
        let dt = (t.min(horizon) - self.time).max(0.0);
        self.ids
            .iter()
            .zip(&self.balls)
            .map(move |(id, s)| (*id, motion::evolve(s, dt, &self.k)))
    }

    /// Takes the events recorded since the last call.
    pub fn drain_events(&mut self) -> std::vec::Drain<'_, SimEvent> {
        self.log.drain(..)
    }

    /// Checks the physical invariants: finite state, no interpenetration and
    /// every ball on the playing surface. Returns a description of the first
    /// violation.
    pub fn check_invariants(&self) -> Result<(), String> {
        const TOLERANCE: f64 = 1e-9;
        let r = self.k.radius;
        let limit = self.profile.table.half_extents() - DVec2::splat(r);
        let live: Vec<_> = self
            .ids
            .iter()
            .zip(&self.balls)
            .filter(|(_, s)| s.motion != Motion::Pocketed)
            .collect();
        for (n, (id, s)) in live.iter().enumerate() {
            if !s.is_finite() {
                return Err(format!("{id} has a non-finite state: {s:?}"));
            }
            let p = planar(s.position);
            if p.x.abs() > limit.x + TOLERANCE || p.y.abs() > limit.y + TOLERANCE {
                return Err(format!("{id} left the playing surface at {p}"));
            }
            for (other, q) in &live[n + 1..] {
                let gap = p.distance(planar(q.position)) - 2.0 * r;
                if gap < -TOLERANCE {
                    return Err(format!("{id} and {other} interpenetrate by {:.3e} m", -gap));
                }
            }
        }
        Ok(())
    }

    fn index(&self, id: BallId) -> Option<usize> {
        self.ids.binary_search(&id).ok()
    }

    fn push(&mut self, kind: SimEventKind) {
        self.log.push(SimEvent {
            time: self.time,
            kind,
        });
    }

    /// Finds the earliest upcoming event. Candidates are examined in a fixed
    /// order and only a strictly earlier one replaces the current best, so
    /// ties always resolve the same way.
    fn schedule(&self) -> Option<Pending> {
        let trajectories: Vec<Trajectory> = self
            .balls
            .iter()
            .map(|s| motion::trajectory(s, &self.k))
            .collect();
        let mut best: Option<Pending> = None;
        let offer = |dt: f64, what: Scheduled, best: &mut Option<Pending>| {
            if best.is_none_or(|b| dt < b.dt) {
                *best = Some(Pending { dt, what });
            }
        };

        for (i, t) in trajectories.iter().enumerate() {
            if t.end.is_finite() {
                offer(t.end, Scheduled::Transition(i), &mut best);
            }
        }

        let limit = self.profile.table.half_extents() - DVec2::splat(self.k.radius);
        for (i, t) in trajectories
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_moving())
        {
            for rail in Rail::ALL {
                let window = best.map_or(t.end, |b| b.dt.min(t.end));
                if let Some(dt) = rail_gap(t, rail, limit).first_falling_root(0.0, window) {
                    offer(dt, Scheduled::Cushion(i, rail), &mut best);
                }
            }
        }

        let contact = 2.0 * self.k.radius;
        for (i, ti) in trajectories.iter().enumerate() {
            if self.balls[i].motion == Motion::Pocketed {
                continue;
            }
            for (j, tj) in trajectories.iter().enumerate().skip(i + 1) {
                if self.balls[j].motion == Motion::Pocketed || !(ti.is_moving() || tj.is_moving()) {
                    continue;
                }
                let window = best.map_or(ti.end.min(tj.end), |b| b.dt.min(ti.end).min(tj.end));
                // Cheap rejection: too far apart to meet within the window.
                if ti.c.distance(tj.c) - contact > ti.reach(window) + tj.reach(window) {
                    continue;
                }
                if let Some(dt) = pair_gap(ti, tj, contact).first_falling_root(0.0, window) {
                    offer(dt, Scheduled::Pair(i, j), &mut best);
                }
            }
        }
        best
    }

    fn resolve(&mut self, what: Scheduled) {
        match what {
            Scheduled::Transition(i) => {
                let from = self.balls[i].motion;
                self.balls[i] = motion::transition(&self.balls[i], &self.k);
                let to = self.balls[i].motion;
                self.push(SimEventKind::Transition {
                    ball: self.ids[i],
                    from,
                    to,
                });
            }
            Scheduled::Cushion(i, rail) => {
                let normal_speed = self.cushion_impulse(i, rail);
                self.push(SimEventKind::Cushion {
                    ball: self.ids[i],
                    rail,
                    normal_speed,
                });
            }
            Scheduled::Pair(i, j) => {
                let normal_speed = self.ball_impulse(i, j);
                self.push(SimEventKind::BallBall {
                    a: self.ids[i],
                    b: self.ids[j],
                    normal_speed,
                });
            }
        }
    }

    /// Ball–cushion contact: the normal component reverses with restitution.
    ///
    /// Phase 1 model. Spin is preserved, so the ball leaves the cushion
    /// sliding and cloth friction realigns it. The nose-height model of
    /// Han (2005), which couples spin and rebound, replaces this in Phase 2.
    fn cushion_impulse(&mut self, i: usize, rail: Rail) -> f64 {
        let n = rail.inward_normal();
        let s = &mut self.balls[i];
        let v = planar(s.velocity);
        let vn = v.dot(n);
        let rebound = (-vn * self.profile.cushion.restitution).max(MIN_SEPARATION_SPEED);
        if vn < rebound {
            let v = v + (rebound - vn) * n;
            s.velocity = DVec3::new(v.x, 0.0, v.y);
        }
        self.balls[i] = motion::settle(&self.balls[i], &self.k);
        (-vn).max(0.0)
    }

    /// Ball–ball contact between equal masses: an impulse along the line of
    /// centres with restitution. Surfaces are treated as frictionless in
    /// Phase 1, so spin carries through the collision unchanged — which is
    /// exactly what produces follow and draw once the cloth acts on it.
    fn ball_impulse(&mut self, i: usize, j: usize) -> f64 {
        let (pi, pj) = (
            planar(self.balls[i].position),
            planar(self.balls[j].position),
        );
        let n = (pj - pi).normalize_or_zero();
        let closing = (planar(self.balls[i].velocity) - planar(self.balls[j].velocity)).dot(n);
        // Equal masses: each ball's normal velocity changes by half of the
        // total change in relative normal velocity.
        let separating = (closing * self.profile.ball_ball.restitution).max(MIN_SEPARATION_SPEED);
        if -closing < separating {
            let dv = DVec3::new(n.x, 0.0, n.y) * 0.5 * (closing + separating);
            self.balls[i].velocity -= dv;
            self.balls[j].velocity += dv;
        }
        self.balls[i] = motion::settle(&self.balls[i], &self.k);
        self.balls[j] = motion::settle(&self.balls[j], &self.k);
        closing.max(0.0)
    }
}

/// Signed clearance between a ball and a cushion over time; positive while
/// the ball is clear of it.
fn rail_gap(t: &Trajectory, rail: Rail, limit: DVec2) -> Poly {
    match rail {
        Rail::Foot => Poly::new(&[limit.x - t.c.x, -t.b.x, -t.a.x]),
        Rail::Head => Poly::new(&[limit.x + t.c.x, t.b.x, t.a.x]),
        Rail::Near => Poly::new(&[limit.y - t.c.y, -t.b.y, -t.a.y]),
        Rail::Far => Poly::new(&[limit.y + t.c.y, t.b.y, t.a.y]),
    }
}

/// `|pᵢ(t) − pⱼ(t)|² − d²`: positive while two balls are apart.
fn pair_gap(ti: &Trajectory, tj: &Trajectory, contact: f64) -> Poly {
    let (a, b, c) = (ti.a - tj.a, ti.b - tj.b, ti.c - tj.c);
    Poly::new(&[
        c.dot(c) - contact * contact,
        2.0 * b.dot(c),
        b.dot(b) + 2.0 * a.dot(c),
        2.0 * a.dot(b),
        a.dot(a),
    ])
}
