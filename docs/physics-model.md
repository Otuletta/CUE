# Physics model

CUE simulates pool with a purpose-built, **event-driven** solver
(`crates/cue-physics`). This page documents what is modelled, the equations
used, and what is deliberately not modelled yet. Rationale:
[ADR-0002](adr/0002-custom-billiards-solver.md).

## Conventions

SI units throughout. Origin at the centre of the playing surface, `+X`
towards the foot rail, `+Y` up. Balls are spheres of radius `R` resting on
the plane `y = R`; their planar state is `(x, z)`.

## Motion between events

Every ball is in one of five regimes. Each regime has a closed-form solution,
so the solver never integrates forces frame by frame.

The slip velocity of the cloth contact point is

```
u = v + ω × (0, −R, 0)          (planar components: vx + R·ωz,  vz − R·ωx)
```

| Regime | Condition | Equations | Duration |
|---|---|---|---|
| Sliding | `|u| > 0` | `v(t) = v₀ − μs·g·t·û`, `ω̇⊥ = (5μs·g / 2R)·(ûz, 0, −ûx)` | `2|u₀| / (7μs·g)` |
| Rolling | `u = 0, v ≠ 0` | `v(t) = v₀ − μr·g·t·v̂`, `ω⊥` locked to `v` | `|v₀| / (μr·g)` |
| Spinning | `v = 0, ωy ≠ 0` | `|ωy|` decreases at `5μsp·g / 2R` | `|ωy| · 2R / (5μsp·g)` |
| Stationary | all zero | — | ∞ |
| Pocketed | removed (Phase 2) | — | ∞ |

`û` (slip direction) is constant while sliding, which is why the path is a
parabola and positions are quadratic in time: `p(t) = c + b·t + a·t²`.
Vertical spin `ωy` decays in parallel in every moving regime without affecting
the path on its own.

Classic results that the tests check:

- a stun shot (no initial spin) starts rolling at **5/7** of its launch speed;
- a rolling ball stops after `v² / (2μr·g)`;
- a tip height of **+2R/5** produces natural roll straight away.

## Events

The next event is the earliest of:

| Event | Found as |
|---|---|
| Regime change | the durations above |
| Ball–cushion segment | first root of a quadratic (distance to the segment's line minus `R`), accepted only if the contact point lies on the segment |
| Ball–jaw point | first root of the quartic `|p(t) − v|² − R²` |
| Ball into pocket | first root of the quartic `|p(t) − c|² − r²` (centre enters the drop region) |
| Ball–ball | first root of the quartic `|pᵢ(t) − pⱼ(t)|² − (2R)²` |

Roots are isolated rather than computed in closed form (`poly.rs`): the roots
of `p′` split the window into monotonic intervals, and each interval with a
sign change is bisected to machine precision. Candidates are offered in time
order to an acceptance test, so touching a cushion segment's *line* beyond its
end is not a contact. A contact that is already touching *and closing* is
reported immediately, so rounding can never let two balls tunnel into each
other.

Candidates are visited in a fixed order and only a strictly earlier one wins,
so ties always resolve the same way. Pairs that cannot meet within the window
are rejected cheaply from a bound on how far each ball can travel.

### Determinism and frame-rate independence

The authoritative state changes **only at events**. Rendering calls
`World::sample(t)`, which evaluates the closed-form trajectories without
mutating anything. The same strike therefore produces the same shot,
bit-for-bit, whether the game runs at 7, 30, 60 or 144 FPS or with irregular
frame pacing. `break_outcome_is_identical_at_any_frame_rate` asserts exactly
this on a full 16-ball break.

## Table geometry

`TableGeometry` (`geometry.rs`) derives the collision model from the
`TableSpec`:

- **Six rail sections**, the cushion noses between pocket mouths.
- **Twelve facings**, two per pocket. They run from each jaw point into the
  rail at the WPA facing angle: 142° for corners, 104° for sides. Above 90°
  the facings converge, so every throat is narrower than its mouth (corner
  mouth 4.56 in, throat about 3.8 in).
- **24 sharp vertices**: each jaw point and each facing end can be struck on
  its own.
- **Six drop regions.** A circle behind each mouth, set back by the WPA shelf
  depth. A ball whose **centre** enters it falls.

Construction refuses any geometry whose drop region does not cover the whole
throat, so no ball can slip between the facings and leave the table. The
same outlines drive the rendered cushions; the solver never reads meshes.

## Contacts

Both contact models follow one recipe (`impact.rs`). It keeps them simple to
reason about and provably dissipative:

1. Treat the ball(s) as **free spheres** and apply a Newton–Coulomb impulse
   at the contact point: normal restitution `e`, and friction that either
   stops the tangential slip (stick) or is capped at `μ` times the normal
   impulse (slide). For spheres the contact normal passes through the
   centre, so normal and tangential responses decouple and this step is
   exact.
2. **Let the slate absorb** any vertical velocity the impulse produced. The
   ball stays on the cloth, and its spin keeps the full effect of the impulse.

Step 1 cannot create energy for `e ≤ 1`, `μ ≥ 0`, and step 2 only removes
it. Property tests check this over random states and also check momentum
conservation in ball–ball impacts.

**Ball–cushion (after Han, 2005).** The contact point is at the cushion nose
height `h`, above the equator, at `θ = asin((h − R)/R)`, which is about 16°
for a nose at 63.5 % of the ball's diameter. Consequences that the tests
verify:

- a perpendicular stun shot rebounds at about 0.7 of its speed;
- a rolling ball loses topspin at the rail and leaves it sliding;
- side spin changes the rebound angle through cushion friction.

Han's paper parameterises the impulse by its normal component. Here friction
opposes the *actual* slip of the contact point, which is equivalent for a
free sphere and avoids assuming the slip direction.

**Ball–ball with friction.** Equal masses, restitution along the line of
centres, plus friction at the contact. The friction coefficient falls with
slip speed, `μ(v) = 0.00995 + 0.108·e^(−1.088·v)` (Alciatore's fit to
measured throw). This produces:

- **Cut-induced throw.** A slow half-ball stun shot throws the object ball
  1–6° off the line of centres, in the direction of the cue ball's path. Fast
  shots throw less.
- **Spin-induced throw.** Right english throws a straight shot to the left.
- **Spin transfer**, from the same friction impulse.

With the friction off and `e = 1`, the 90° rule holds exactly, and the tests
assert it. With realistic values the separation angle is a degree or two
smaller, as on a real table.

**Follow and draw** emerge on their own: spin survives the collision, and the
cloth turns it into motion.

**Pressing contacts.** Friction can push two touching balls, or a ball and a
cushion, into each other. With impulses alone this collapses into an
unbounded sequence of ever-smaller bounces. Every contact therefore leaves a
separating speed of at least **0.1 mm/s**. That bounds the cascade and adds
about a nanojoule of energy. Contacts below **1 mm/s** are simulated but left
out of the `ShotReport`, so a ball nudging another or resting against a rail
is never mistaken for a hit.

## Strike

The cue delivers a horizontal impulse `J = m·V·d̂` at the tip contact point
`r`, so `v = J/m` and `ω = (r × J) / (2/5·m·R²)`. Tip offsets are limited to
half a radius (the miscue limit). Power `∈ (0, 1]` maps linearly to speed up
to `CueParams::max_speed`; any perceptual curve belongs to the input layer.

**Squirt.** Side spin pushes the cue ball off the cue's line, away from the
side that was struck:

```
tan α = (5/2)·x·√(1 − x²) / (1 + m/mₑ + (5/2)·(1 − x²))
```

Here `x` is the side offset in radii and `mₑ` the shaft's effective endmass
(Alciatore, TP A.31). With a 7 g endmass, half a tip of english squirts the
ball about 2.3°.

## Shot report

When every ball has stopped, the world produces an immutable `ShotReport`
(`cue-core`): a chronological list of ball contacts, cushion contacts and
pocketings, with times relative to the strike. The rules read only this
([eight-ball rules](rules-eight-ball.md)). They can ask:

- which ball was contacted first (all of them, when contacts are
  simultaneous within 1 ns);
- which balls were pocketed, and where;
- whether the cue ball scratched;
- whether any ball was driven to a rail after the first contact;
- how many distinct object balls were driven to a rail, which is what the
  legal break needs.

Two facts in the report exist only for the rules:

- **Head-string crossings.** A sensor event fires each time the cue ball's
  centre crosses the head string. It is found by the same root finding as a
  contact, so it is exact and does not change the motion. WPA 3.11 needs it.
- **Frozen balls.** Each cushion contact carries its rail and a `frozen`
  flag. When the shot starts, the world records which rails each ball is
  touching, to within 10 µm. A ball stays frozen to a rail until its centre
  gets more than 10 µm farther from that rail's nose line. Between events the
  distance is a quadratic in time, so the check is an exact maximum, with no
  sampling. A contact by a ball that has not left its rail is reported as
  frozen, and the rules do not count it as driving the ball to a rail
  (WPA 2.7). Pocket facings and jaws belong to the rail they are cut into.

## Parameters

`PhysicsProfile::baseline()` is id **`baseline-3`**. Any change to a value
bumps the id, so a change in behaviour is always visible.

| Parameter | Value | Source |
|---|---|---|
| Sliding friction μs | 0.2 | Alciatore, typical |
| Rolling resistance μr | 0.018 | A medium-slow cloth, for the pace of play (see "Pace") |
| Spinning friction μsp | 0.0117 (≈10 rad/s² spin decay) | Alciatore, typical |
| Ball–ball restitution | 0.95 | Alciatore, typical |
| Ball–ball friction | `0.00995 + 0.108·e^(−1.088 v)` | Alciatore, fit to measured throw |
| Ball–cushion restitution | 0.85 | Common for the Han model |
| Ball–cushion friction | 0.2 | Common for the Han model |
| Cushion nose height | 63.5 % of ball diameter | WPA midpoint |
| Max cue-ball speed | 12.5 m/s | A strong break (Alciatore: good amateurs ≈ 9, professionals 11–13) |
| Shaft endmass | 7 g | Typical shaft |

## The break

`baseline-1` broke poorly: the pack barely opened. Two causes, each
measured on 20 racks with a full-power break struck square on the apex:

- **The rack was perfectly frozen.** Five microns between balls let the
  solver pass the impulse along the rows, like a Newton's cradle: the back
  balls flew and the rest stayed. Real racks are never that tight. The rack
  now leaves about half a millimetre between neighbours, varied ball by
  ball from the rack's seed (`RACK_SPACING`). The apex stays exactly on the
  foot spot.
- **Full power was too slow.** At 8 m/s, about four balls reached a rail. A
  strong real break is 10–12 m/s.

| Profile | Balls to a rail | Balls moved > 15 cm | Pocketed | Legal breaks |
|---|---:|---:|---:|---:|
| `baseline-1`: 8 m/s, frozen rack | 4.0 | 6.0 | 0.00 | 20/20 (only just) |
| `baseline-2`: 12 m/s, spaced rack | 5.7 | 8.6 | 0.87 | 20/20 |
| `baseline-3`: 12.5 m/s, slower cloth | 5.4 | 8.7 | 0.85 | 20/20 |

A wider spacing (1–2 mm) did not spread the rack further. A hit a few
millimetres off the apex still loses part of the break, as on a real table.
`a_full_power_break_opens_the_rack` holds these figures.

**Only the break gets the full stroke.** The input layer maps the power
slider differently on the break, a fresh rack's first strike:

| | Full power | Half | Quarter | Curve |
|---|---:|---:|---:|---|
| The break | 12.5 m/s | 3.1 m/s | 0.78 m/s | `p²` |
| Every other shot | 7 m/s | 2.3 m/s | 0.76 m/s | `p^1.6` |

As on a real table, nothing but the break is struck that hard. A full-power
shot at 12 m/s ran the cue ball into eight cushions, which players found
"far too strong". At 7 m/s it is a hard power shot of about five. The
physics profile's top speed is the break's.

## Pace

On `baseline-2`, with the typical rolling resistance of 0.01, a shot took
7–9 seconds to settle, most of it a long, slow roll. Asked how play should
feel, the owner chose "calibrated realism": shots that settle in 4–6
seconds, power that builds progressively, and a break that is strong but
believable.

`baseline-3` raises the rolling resistance to 0.018: a medium-slow cloth,
just above the typical range, which brings the long roll to a stop
sooner. Alone, it weakened the break, which then pocketed a ball only a
third of the time. The break's full power rises from 12 to 12.5 m/s, a
strong professional break, which restores it (table above). Other shots
keep their 7 m/s top.

Time until every ball is under 2 cm/s, for a lone cue ball struck from the
head spot, centre ball:

| Speed | `baseline-2` | `baseline-3` |
|---|---:|---:|
| 1 m/s (a touch) | 4.4–5.4 s | 2.7–4.1 s |
| 2 m/s (a medium shot) | 6.1–7.5 s | 4.1–5.2 s |
| 3 m/s | 5.4–8.3 s | 4.9–5.9 s |
| 4.5 m/s (a firm shot) | 6.2–8.8 s | 4.3–6.3 s |
| 7 m/s (full power) | 4.5–10.3 s | 5.2–7.1 s |
| The break, median | 6.8 s | 5.2 s |

Ranges span three directions (along the table, 25° and 60°). The break's
median is over 20 racks for `baseline-2` and 40 for `baseline-3`.

**Calibration status.** The profile is validated against the textbook and
published behaviours listed above, which the test suite asserts. It has
**not** been fitted to video of a real table. Until it is, the honest claim is
*convincing game physics with the right qualitative and order-of-magnitude
behaviour*, not a competitive-grade replica (spec §11.10).

## Not modelled

| Effect | Status |
|---|---|
| Jump and massé shots (cue elevation, ball leaving the cloth) | Not committed for 1.0 |
| Swerve (curve from side spin with an elevated cue) | Needs cue elevation; not committed |
| Rounded jaw tips and rattling against pocket liners | Jaws are sharp; the drop region decides |
| Ball–cushion "throw" from cushion compression time | Instantaneous impact |
| Cloth wear, humidity, ball dirt | Out of scope |

## Performance

On an Apple M4 Pro (native, release build), the 1,000-shot stress test runs
**18,452 events in about 90 ms**. That includes four racks, 204 balls
pocketed, scheduling against 18 cushion segments, 24 vertices, 6 pockets and
every ball pair, and a full invariant check after every event. It simulates
about 100 minutes of play.

## References

- W. Leckie, M. Greenspan, *An Event-Based Pool Physics Simulator*, Advances
  in Computer Games, 2005.
- I. Han, *Dynamics in carom and three cushion billiards*, Journal of
  Mechanical Science and Technology, 2005.
- D. Alciatore, *Typical Physical Properties of Pool Equipment* and technical
  proofs (TP A.14 throw, TP A.31 squirt), billiards.colostate.edu.
- World Pool-Billiard Association, *Equipment Specifications* (table, pocket
  and cushion dimensions).
