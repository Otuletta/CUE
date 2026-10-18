//! The eight-ball half of the host API: the match snapshot hosts draw from
//! and the events a ruling turns into.
//!
//! Every name that crosses into JavaScript (`"solids"`, `"cue-ball-pocketed"`,
//! `"rerack-and-break"`…) comes from a `name()` in `cue-core`, so the wire
//! format cannot drift from the rules.

use cue_core::{
    BallId,
    rules::{
        Group, Seat,
        eight_ball::{Call, Choice, Decision, Foul, Next, Outcome, Ruling, Stage},
    },
};
use cue_game::{EightBall, Simulation};
use wasm_bindgen::JsValue;

use crate::object;

/// What a host needs to draw an eight-ball match, refreshed every frame.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatchSnapshot {
    ruleset: &'static str,
    names: [String; 2],
    score: [u8; 2],
    race_to: u8,
    rack: u32,
    breaker: Seat,
    shooter: Seat,
    stage: &'static str,
    groups: [Option<(Group, usize)>; 2],
    on_the_eight: bool,
    call: Call,
    manual_call: bool,
    legal_targets: Vec<u8>,
    decision: Option<Decision>,
    outcome: Option<Outcome>,
    match_over: bool,
    refusal: Option<&'static str>,
    can_spot: bool,
    last: Option<(Ruling, Call)>,
}

impl MatchSnapshot {
    pub(crate) fn new(session: &EightBall, sim: &Simulation) -> Self {
        let game = &session.game;
        let shooter = game.shooter();
        let (decision, outcome) = match game.stage() {
            Stage::Decision(d) => (Some(d.clone()), None),
            Stage::Over(o) => (None, Some(*o)),
            Stage::Break | Stage::Play => (None, None),
        };
        Self {
            ruleset: game.ruleset().name(),
            names: session.names.clone(),
            score: session.score,
            race_to: session.race_to,
            rack: session.rack,
            breaker: game.breaker(),
            shooter,
            stage: game.stage().name(),
            groups: [session.remaining(Seat::A), session.remaining(Seat::B)],
            on_the_eight: game.on_the_eight(shooter),
            call: session.effective_call(),
            manual_call: session.manual_call,
            legal_targets: game.legal_targets().iter().map(|b| b.number()).collect(),
            decision,
            outcome,
            match_over: session.match_over(),
            refusal: session.can_shoot().err(),
            can_spot: sim.world.is_at_rest()
                && game.spot_request(&sim.layout(), sim.table()).is_some(),
            last: session.last.clone().map(|r| (r, session.last_call)),
        }
    }

    /// The `eightBall` member of `state()`.
    pub(crate) fn to_js(&self) -> JsValue {
        let players: js_sys::Array = [Seat::A, Seat::B]
            .into_iter()
            .map(|seat| {
                let i = seat.index();
                let (group, remaining) = match self.groups[i] {
                    Some((g, n)) => (JsValue::from_str(g.name()), JsValue::from(n as u32)),
                    None => (JsValue::NULL, JsValue::NULL),
                };
                object(&[
                    ("seat", seat.name().into()),
                    ("name", JsValue::from_str(&self.names[i])),
                    ("score", self.score[i].into()),
                    ("group", group),
                    ("remaining", remaining),
                ])
            })
            .collect();
        object(&[
            ("ruleset", self.ruleset.into()),
            ("players", players.into()),
            ("raceTo", self.race_to.into()),
            ("rack", self.rack.into()),
            ("breaker", self.breaker.name().into()),
            ("shooter", self.shooter.name().into()),
            ("stage", self.stage.into()),
            ("openTable", self.groups[0].is_none().into()),
            ("onTheEight", self.on_the_eight.into()),
            ("call", call(self.call)),
            ("manualCall", self.manual_call.into()),
            ("legalTargets", numbers(&self.legal_targets)),
            (
                "decision",
                self.decision.as_ref().map_or(JsValue::NULL, decision),
            ),
            ("outcome", self.outcome.map_or(JsValue::NULL, outcome)),
            ("matchOver", self.match_over.into()),
            ("canShoot", self.refusal.is_none().into()),
            (
                "refusal",
                self.refusal.map_or(JsValue::NULL, JsValue::from_str),
            ),
            ("canSpot", self.can_spot.into()),
            (
                "lastRuling",
                self.last
                    .as_ref()
                    .map_or(JsValue::NULL, |(r, c)| ruling(r, *c)),
            ),
        ])
    }
}

fn numbers(balls: &[u8]) -> JsValue {
    balls
        .iter()
        .map(|n| JsValue::from(*n))
        .collect::<js_sys::Array>()
        .into()
}

fn ball_numbers(balls: &[BallId]) -> JsValue {
    numbers(&balls.iter().map(|b| b.number()).collect::<Vec<_>>())
}

/// `{ kind: "none" | "safety" }` or `{ kind: "ball", ball, pocket }`.
fn call(call: Call) -> JsValue {
    match call {
        Call::None => object(&[("kind", "none".into())]),
        Call::Safety => object(&[("kind", "safety".into())]),
        Call::Ball { ball, pocket } => object(&[
            ("kind", "ball".into()),
            ("ball", ball.number().into()),
            ("pocket", pocket.name().into()),
        ]),
    }
}

fn decision(d: &Decision) -> JsValue {
    let options: js_sys::Array = d
        .options
        .iter()
        .map(|c| JsValue::from_str(c.name()))
        .collect();
    object(&[
        ("chooser", d.chooser.name().into()),
        ("situation", d.situation.name().into()),
        ("options", options.into()),
        ("cueBallOff", d.cue_ball_off.into()),
    ])
}

fn outcome(o: Outcome) -> JsValue {
    object(&[
        ("winner", o.winner.name().into()),
        (
            "loss",
            o.loss
                .map_or(JsValue::NULL, |l| JsValue::from_str(l.name())),
        ),
    ])
}

fn fouls(fouls: &[Foul]) -> JsValue {
    fouls
        .iter()
        .map(|f| JsValue::from_str(f.name()))
        .collect::<js_sys::Array>()
        .into()
}

/// The ball a [`Foul::WrongBallFirst`] hit, if that was one of the fouls.
fn wrong_ball(fouls: &[Foul]) -> JsValue {
    fouls
        .iter()
        .find_map(|f| match f {
            Foul::WrongBallFirst { hit } => Some(JsValue::from(hit.number())),
            _ => None,
        })
        .unwrap_or(JsValue::NULL)
}

/// The `ruling` event payload, also `state().eightBall.lastRuling`. `called`
/// is what the shooter called for that shot.
fn ruling(r: &Ruling, called: Call) -> JsValue {
    let (next, in_hand) = match &r.next {
        Next::Continue => ("continue", JsValue::NULL),
        Next::Pass { in_hand } => ("pass", JsValue::from_str(in_hand.name())),
        Next::Decide(_) => ("decide", JsValue::NULL),
        Next::Over(_) => ("over", JsValue::NULL),
    };
    let assigned = r.assigned.map_or(JsValue::NULL, |(seat, group)| {
        object(&[("seat", seat.name().into()), ("group", group.name().into())])
    });
    object(&[
        ("shooter", r.shooter.name().into()),
        ("call", call(called)),
        ("fouls", fouls(&r.fouls)),
        ("wrongBall", wrong_ball(&r.fouls)),
        ("pocketed", ball_numbers(&r.pocketed)),
        ("assigned", assigned),
        ("next", next.into()),
        ("inHand", in_hand),
    ])
}

/// The events one ruling produces, in the order a commentator would say
/// them: the ruling itself, then a foul, groups, and who plays next or how
/// the game (the rack) ended.
pub(crate) fn ruling_events(
    session: &EightBall,
    r: &Ruling,
    called: Call,
) -> Vec<(&'static str, JsValue)> {
    let mut events = vec![("ruling", ruling(r, called))];
    if !r.fouls.is_empty() {
        events.push((
            "foul",
            object(&[
                ("shooter", r.shooter.name().into()),
                ("fouls", fouls(&r.fouls)),
                ("wrongBall", wrong_ball(&r.fouls)),
            ]),
        ));
    }
    if let Some((seat, group)) = r.assigned {
        events.push((
            "groups-assigned",
            object(&[("seat", seat.name().into()), ("group", group.name().into())]),
        ));
    }
    match &r.next {
        Next::Continue => {}
        Next::Pass { in_hand } => events.push((
            "turn-changed",
            object(&[
                ("shooter", r.shooter.other().name().into()),
                ("previous", r.shooter.name().into()),
                ("inHand", in_hand.name().into()),
            ]),
        )),
        Next::Decide(d) => events.push(("decision-required", decision(d))),
        Next::Over(o) => {
            let score: js_sys::Array = session.score.iter().map(|s| JsValue::from(*s)).collect();
            let payload = object(&[
                ("winner", o.winner.name().into()),
                (
                    "loss",
                    o.loss
                        .map_or(JsValue::NULL, |l| JsValue::from_str(l.name())),
                ),
                ("score", score.into()),
            ]);
            events.push(("game-over", payload.clone()));
            if session.match_over() {
                events.push(("match-over", payload));
            }
        }
    }
    events
}

/// The `decided` payload: the option chosen and who plays next, from where.
pub(crate) fn decided(session: &EightBall, choice: Choice) -> JsValue {
    object(&[
        ("choice", choice.name().into()),
        ("shooter", session.game.shooter().name().into()),
        ("inHand", session.game.in_hand().name().into()),
        ("stage", session.game.stage().name().into()),
    ])
}
