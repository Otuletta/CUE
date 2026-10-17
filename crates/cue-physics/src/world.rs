//! The event-driven simulation.
//!
//! The authoritative state only ever changes **at events**: a collision, a
//! cushion contact, a ball dropping into a pocket, or a regime change.
//! Between events every ball follows a closed-form trajectory, so the
//! renderer can [`sample`](World::sample) any instant without touching the
//! state. As a consequence, the outcome of a shot is bit-for-bit independent
//! of the frame rate that drives it.

use cue_core::{
    BallId, Layout, PocketId, ShotEvent, ShotEventKind, ShotReport,
    glam::{DVec2, DVec3},
};
use thiserror::Error;

use crate::{
    BallState, Motion, PhysicsProfile, Strike,
    geometry::{Cushion, Segment, TableGeometry},
    impact::{self, BallContact, CushionContact},
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

/// Contacts slower than this, m/s, are pressing-contact artefacts (balls
/// nudging each other or resting against a cushion), not hits. They are
/// simulated but not reported to the rules.
const REPORT_MIN_SPEED: f64 = 1e-3;

/// How far behind a cushion line a ball may be, from rounding alone, and
/// still count as touching it, metres.
const CONTACT_TOLERANCE: f64 = 1e-9;

/// Upper bound on events processed by a single [`World::advance_to`] call.
/// Real shots stay orders of magnitude below it; reaching it means a bug.
const MAX_EVENTS_PER_ADVANCE: usize = 100_000;

/// Errors raised by the simulation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SimError {
    /// The table or initial layout cannot be simulated.
    #[error("invalid layout: {0}")]
    InvalidLayout(String),
    /// A strike was rejected before touching the state.
    #[error("invalid strike: {0}")]
    InvalidStrike(&'static str),
    /// A ball cannot be placed where requested.
    #[error("invalid placement: {0}")]
    InvalidPlacement(&'static str),
    /// A strike or placement was attempted while balls are still moving.
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
    /// A ball hit a cushion or a pocket jaw.
    Cushion {
        /// The ball.
        ball: BallId,
        /// What it hit.
        cushion: Cushion,
        /// Horizontal speed into the cushion, m/s.
        normal_speed: f64,
    },
    /// A ball dropped into a pocket.
    Pocketed {
        /// The ball.
        ball: BallId,
        /// The pocket.
        pocket: PocketId,
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

/// Indices into the world's balls and the geometry's lists.
#[derive(Clone, Copy, Debug)]
enum Scheduled {
    Transition(usize),
    Segment(usize, usize),
    Vertex(usize, usize),
    Pocket(usize, usize),
    Pair(usize, usize),
}

/// The shot being recorded.
#[derive(Clone, Debug)]
struct ShotLog {
    id: u32,
    start: f64,
    events: Vec<ShotEvent>,
}

/// A table of balls and the rules of motion that govern them.
#[derive(Clone, Debug)]
pub struct World {
    profile: PhysicsProfile,
    geometry: TableGeometry,
    k: Kinematics,
    cushion_contact: CushionContact,
    ball_contact: BallContact,
    /// Ball mass over shaft endmass, for squirt.
    mass_ratio: f64,
    /// Ball ids, ascending. Index `i` here is index `i` in `balls`.
    ids: Vec<BallId>,
    balls: Vec<BallState>,
    time: f64,
    next: Option<Pending>,
    log: Vec<SimEvent>,
    shots: u32,
    shot: Option<ShotLog>,
    report: Option<ShotReport>,
}

impl World {
    /// Builds a world with every ball of `layout` at rest.
    pub fn new(profile: PhysicsProfile, layout: &Layout) -> Result<Self, SimError> {
        let geometry = TableGeometry::new(&profile.table)?;
        let table = &profile.table;
        let r = table.ball.radius;
        let (ids, balls): (Vec<_>, Vec<_>) = layout
            .iter()
            .map(|(id, p)| (id, BallState::at_rest(p.x, p.y, r)))
            .unzip();

        for (i, (id, p)) in layout.iter().enumerate() {
            if !table.contains_ball(p) {
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
            cushion_contact: CushionContact::new(
                r,
                table.cushion_nose_height,
                profile.cushion.restitution,
                profile.cushion.friction,
            ),
            ball_contact: BallContact {
                radius: r,
                restitution: profile.ball_ball.restitution,
                friction: profile.ball_ball.friction,
            },
            mass_ratio: table.ball.mass / profile.cue.shaft_endmass,
            geometry,
            profile,
            ids,
            balls,
            time: 0.0,
            next: None,
            log: Vec::new(),
            shots: 0,
            shot: None,
            report: None,
        })
    }

    /// The parameters this world runs with.
    pub fn profile(&self) -> &PhysicsProfile {
        &self.profile
    }

    /// The cushion and pocket geometry.
    pub fn geometry(&self) -> &TableGeometry {
        &self.geometry
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

    /// Ids of balls still in play, ascending.
    pub fn balls_in_play(&self) -> impl Iterator<Item = BallId> + '_ {
        self.ids
            .iter()
            .zip(&self.balls)
            .filter(|(_, s)| s.motion != Motion::Pocketed)
            .map(|(id, _)| *id)
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
        let (v, w) = strike.velocities(speed, self.k.radius, self.mass_ratio);
        let mut cue = self.balls[i];
        cue.velocity = v;
        cue.angular_velocity = w;
        self.balls[i] = motion::settle(&cue, &self.k);

        self.shots += 1;
        self.shot = Some(ShotLog {
            id: self.shots,
            start: self.time,
            events: Vec::new(),
        });
        self.report = None;
        self.push(SimEventKind::Struck { speed });
        self.next = self.schedule();
        self.finish_shot_if_at_rest();
        Ok(())
    }

    /// Places a ball at rest at `at`: ball in hand, re-spotting, practice
    /// setups. The ball may currently be pocketed. Fails if balls are moving,
    /// if `at` is off the playing surface or if it overlaps another ball.
    pub fn place(&mut self, id: BallId, at: DVec2) -> Result<(), SimError> {
        if !self.is_at_rest() {
            return Err(SimError::NotAtRest);
        }
        let i = self
            .index(id)
            .ok_or(SimError::InvalidPlacement("no such ball on this table"))?;
        if !self.profile.table.contains_ball(at) {
            return Err(SimError::InvalidPlacement("off the playing surface"));
        }
        if !self.is_free(id, at) {
            return Err(SimError::InvalidPlacement("overlaps another ball"));
        }
        self.balls[i] = BallState::at_rest(at.x, at.y, self.k.radius);
        Ok(())
    }

    /// The free position nearest to `preferred` where `id` could be placed,
    /// searching outwards in rings. Deterministic.
    pub fn nearest_free_spot(&self, id: BallId, preferred: DVec2) -> Option<DVec2> {
        let table = &self.profile.table;
        let ok = |p: DVec2| table.contains_ball(p) && self.is_free(id, p);
        if ok(preferred) {
            return Some(preferred);
        }
        let step = self.k.radius / 2.0;
        let rings = (table.playfield_length / step).ceil() as usize;
        (1..=rings).find_map(|ring| {
            let radius = ring as f64 * step;
            let points = (std::f64::consts::TAU * ring as f64).ceil() as usize;
            (0..points)
                .map(|k| {
                    preferred
                        + DVec2::from_angle(k as f64 / points as f64 * std::f64::consts::TAU)
                            * radius
                })
                .find(|p| ok(*p))
        })
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
            self.finish_shot_if_at_rest();
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

    /// Takes the report of the last completed shot, if it has not been taken.
    pub fn take_report(&mut self) -> Option<ShotReport> {
        self.report.take()
    }

    /// Checks the physical invariants: finite state, no interpenetration of
    /// balls or cushions, no ball escaped the table, and no ball in play
    /// sitting inside a drop region. Returns the first violation.
    pub fn check_invariants(&self) -> Result<(), String> {
        const TOLERANCE: f64 = 1e-9;
        let r = self.k.radius;
        let outer = self.geometry.outer_half_extents();
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
            if p.x.abs() > outer.x || p.y.abs() > outer.y {
                return Err(format!("{id} escaped the table at {p}"));
            }
            if let Some((cushion, depth)) = self.geometry.penetration(p, r)
                && depth > TOLERANCE
            {
                return Err(format!("{id} is {depth:.3e} m inside {cushion:?} at {p}"));
            }
            if let Some(pocket) = self
                .geometry
                .pockets()
                .iter()
                .find(|k| p.distance(k.centre) < k.radius - TOLERANCE)
            {
                return Err(format!(
                    "{id} is inside the {} drop region but in play",
                    pocket.id
                ));
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

    fn is_free(&self, id: BallId, at: DVec2) -> bool {
        let contact = 2.0 * self.k.radius;
        self.ids.iter().zip(&self.balls).all(|(other, s)| {
            *other == id
                || s.motion == Motion::Pocketed
                || planar(s.position).distance(at) >= contact
        })
    }

    /// Records an event in the simulation log and, when it matters to the
    /// rules, in the current shot.
    fn push(&mut self, kind: SimEventKind) {
        self.log.push(SimEvent {
            time: self.time,
            kind,
        });
        let reported = match kind {
            SimEventKind::BallBall { a, b, normal_speed } if normal_speed >= REPORT_MIN_SPEED => {
                Some(ShotEventKind::BallContact { a, b })
            }
            SimEventKind::Cushion {
                ball, normal_speed, ..
            } if normal_speed >= REPORT_MIN_SPEED => Some(ShotEventKind::Cushion { ball }),
            SimEventKind::Pocketed { ball, pocket } => {
                Some(ShotEventKind::Pocketed { ball, pocket })
            }
            _ => None,
        };
        if let (Some(kind), Some(shot)) = (reported, self.shot.as_mut()) {
            shot.events.push(ShotEvent {
                time: self.time - shot.start,
                kind,
            });
        }
    }

    fn finish_shot_if_at_rest(&mut self) {
        if self.next.is_none()
            && let Some(shot) = self.shot.take()
        {
            self.report = Some(ShotReport {
                shot_id: shot.id,
                events: shot.events,
            });
        }
    }

    /// Finds the earliest upcoming event. Candidates are examined in a fixed
    /// order and only a strictly earlier one replaces the current best, so
    /// ties always resolve the same way.
    fn schedule(&self) -> Option<Pending> {
        let r = self.k.radius;
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
        let window = |end: f64, best: &Option<Pending>| best.map_or(end, |b| b.dt.min(end));

        for (i, t) in trajectories.iter().enumerate() {
            if t.end.is_finite() {
                offer(t.end, Scheduled::Transition(i), &mut best);
            }
        }

        for (i, t) in trajectories.iter().enumerate() {
            if !t.is_moving() || self.balls[i].motion == Motion::Pocketed {
                continue;
            }
            for (k, segment) in self.geometry.segments().iter().enumerate() {
                let w = window(t.end, &best);
                let gap = (t.c - segment.a).dot(segment.normal) - r;
                if gap > t.reach(w) || gap < -r {
                    continue;
                }
                let touches = |dt: f64| on_segment(segment, t.at(dt), r);
                if let Some(dt) = segment_gap(t, segment, r).first_contact(0.0, w, touches) {
                    offer(dt, Scheduled::Segment(i, k), &mut best);
                }
            }
            for (k, vertex) in self.geometry.vertices().iter().enumerate() {
                let w = window(t.end, &best);
                if t.c.distance(vertex.at) - r > t.reach(w) {
                    continue;
                }
                if let Some(dt) = point_gap(t, vertex.at, r).first_falling_root(0.0, w) {
                    offer(dt, Scheduled::Vertex(i, k), &mut best);
                }
            }
            for (k, pocket) in self.geometry.pockets().iter().enumerate() {
                let w = window(t.end, &best);
                if t.c.distance(pocket.centre) - pocket.radius > t.reach(w) {
                    continue;
                }
                if let Some(dt) =
                    point_gap(t, pocket.centre, pocket.radius).first_falling_root(0.0, w)
                {
                    offer(dt, Scheduled::Pocket(i, k), &mut best);
                }
            }
        }

        let contact = 2.0 * r;
        for (i, ti) in trajectories.iter().enumerate() {
            if self.balls[i].motion == Motion::Pocketed {
                continue;
            }
            for (j, tj) in trajectories.iter().enumerate().skip(i + 1) {
                if self.balls[j].motion == Motion::Pocketed || !(ti.is_moving() || tj.is_moving()) {
                    continue;
                }
                let w = window(ti.end.min(tj.end), &best);
                // Cheap rejection: too far apart to meet within the window.
                if ti.c.distance(tj.c) - contact > ti.reach(w) + tj.reach(w) {
                    continue;
                }
                if let Some(dt) = pair_gap(ti, tj, contact).first_falling_root(0.0, w) {
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
            Scheduled::Segment(i, k) => {
                let segment = self.geometry.segments()[k];
                let normal_speed = self.cushion_hit(i, segment.normal);
                self.push(SimEventKind::Cushion {
                    ball: self.ids[i],
                    cushion: segment.cushion,
                    normal_speed,
                });
            }
            Scheduled::Vertex(i, k) => {
                let vertex = self.geometry.vertices()[k];
                let normal = (planar(self.balls[i].position) - vertex.at).normalize_or_zero();
                let normal_speed = self.cushion_hit(i, normal);
                self.push(SimEventKind::Cushion {
                    ball: self.ids[i],
                    cushion: vertex.cushion,
                    normal_speed,
                });
            }
            Scheduled::Pocket(i, k) => {
                let pocket = self.geometry.pockets()[k].id;
                let s = &mut self.balls[i];
                s.velocity = DVec3::ZERO;
                s.angular_velocity = DVec3::ZERO;
                s.motion = Motion::Pocketed;
                self.push(SimEventKind::Pocketed {
                    ball: self.ids[i],
                    pocket,
                });
            }
            Scheduled::Pair(i, j) => {
                let normal_speed = self.ball_hit(i, j);
                self.push(SimEventKind::BallBall {
                    a: self.ids[i],
                    b: self.ids[j],
                    normal_speed,
                });
            }
        }
    }

    /// Ball–cushion contact through the Han-style model in [`impact`], then
    /// the minimum separation guarantee.
    fn cushion_hit(&mut self, i: usize, normal: DVec2) -> f64 {
        let (mut out, speed) = impact::cushion(&self.balls[i], normal, &self.cushion_contact);
        let separating = planar(out.velocity).dot(normal);
        if separating < MIN_SEPARATION_SPEED {
            out.velocity +=
                DVec3::new(normal.x, 0.0, normal.y) * (MIN_SEPARATION_SPEED - separating);
        }
        self.balls[i] = motion::settle(&out, &self.k);
        speed
    }

    /// Ball–ball contact with friction (throw, spin transfer), then the
    /// minimum separation guarantee.
    fn ball_hit(&mut self, i: usize, j: usize) -> f64 {
        let (mut a, mut b, closing) =
            impact::ball_ball(&self.balls[i], &self.balls[j], &self.ball_contact);
        let n = (planar(b.position) - planar(a.position)).normalize_or_zero();
        let separating = (planar(b.velocity) - planar(a.velocity)).dot(n);
        if separating < MIN_SEPARATION_SPEED {
            let fix = DVec3::new(n.x, 0.0, n.y) * 0.5 * (MIN_SEPARATION_SPEED - separating);
            a.velocity -= fix;
            b.velocity += fix;
        }
        self.balls[i] = motion::settle(&a, &self.k);
        self.balls[j] = motion::settle(&b, &self.k);
        closing
    }
}

/// Whether a ball centred at `p` is touching `segment` itself rather than the
/// infinite line through it.
fn on_segment(segment: &Segment, p: DVec2, r: f64) -> bool {
    let along = (p - segment.a).dot(segment.direction());
    let gap = (p - segment.a).dot(segment.normal) - r;
    (0.0..=segment.length()).contains(&along) && gap >= -CONTACT_TOLERANCE
}

/// Signed clearance between a ball and the line through a cushion segment;
/// positive while the ball is clear of it.
fn segment_gap(t: &Trajectory, segment: &Segment, r: f64) -> Poly {
    let n = segment.normal;
    Poly::new(&[(t.c - segment.a).dot(n) - r, t.b.dot(n), t.a.dot(n)])
}

/// `|p(t) − q|² − d²`: positive while a ball is farther than `d` from `q`.
fn point_gap(t: &Trajectory, q: DVec2, d: f64) -> Poly {
    let c = t.c - q;
    Poly::new(&[
        c.dot(c) - d * d,
        2.0 * t.b.dot(c),
        t.b.dot(t.b) + 2.0 * t.a.dot(c),
        2.0 * t.a.dot(t.b),
        t.a.dot(t.a),
    ])
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
