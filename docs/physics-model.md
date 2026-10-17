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
| Ball–cushion | first root of a quadratic (distance to the cushion line minus `R`) |
| Ball–ball | first root of the quartic `|pᵢ(t) − pⱼ(t)|² − (2R)²` |

Roots are isolated rather than computed in closed form (`poly.rs`): the roots
of `p′` split the window into monotonic intervals, and each interval with a
sign change is bisected to machine precision. A contact that is already
touching *and closing* is reported immediately, so rounding can never let two
balls tunnel into each other.

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

## Collisions (Phase 1 models)

**Ball–ball.** Equal masses, impulse along the line of centres with
restitution `e`. Surfaces are frictionless for now, so spin passes through
the collision unchanged. That is what produces **follow and draw**: after a
head-on hit the cue ball keeps its spin and the cloth turns it into motion.
It also gives the **90° rule** for stun cut shots, which is tested exactly
with `e = 1`.

**Ball–cushion.** The normal velocity reverses with restitution; spin is
preserved, and the cloth realigns the ball afterwards.

**Pressing contacts.** Friction can push two touching balls, or a ball and a
cushion, into each other. With impulses alone this collapses into an
unbounded sequence of ever-smaller bounces. Every contact therefore leaves a
separating speed of at least **0.1 mm/s**. That bounds the cascade and adds
about a nanojoule of energy.

## Strike

The cue delivers a horizontal impulse `J = m·V·d̂` at the tip contact point
`r`, so `v = J/m` and `ω = (r × J) / (2/5·m·R²)`. Tip offsets are limited to
half a radius (the miscue limit). Power `∈ (0, 1]` maps linearly to speed up
to `CueParams::max_speed`; any perceptual curve belongs to the input layer.

## Parameters

`PhysicsProfile::baseline()` (id `baseline-0`) uses the typical values compiled
by Dr. Dave Alciatore. They are **uncalibrated** against CUE's own reference
shots.

| Parameter | Value |
|---|---|
| Sliding friction μs | 0.2 |
| Rolling resistance μr | 0.01 |
| Spinning friction μsp | 0.0117 (≈10 rad/s² spin decay) |
| Ball–ball restitution | 0.95 |
| Ball–cushion restitution | 0.75 |
| Max cue-ball speed | 8 m/s |

## Not modelled yet

| Effect | Planned |
|---|---|
| Pockets and jaws | Phase 2 |
| Cushion nose height and spin–cushion coupling (Han 2005) | Phase 2 |
| Ball–ball friction: throw, spin transfer | Phase 2 |
| Squirt / swerve (cue-ball deflection from side spin) | Phase 2, evaluated |
| Jump and massé shots | Not committed for 1.0 |

## Performance

On an Apple M4 Pro (native, release build), the 1,000-shot stress test
processes **28,421 events in about 70 ms**, roughly 2.5 µs per event,
including scheduling over all ball pairs. The test simulates two hours of play
and runs with `--ignored`.

## References

- W. Leckie, M. Greenspan, *An Event-Based Pool Physics Simulator*, Advances
  in Computer Games, 2005.
- I. Han, *Dynamics in carom and three cushion billiards*, Journal of
  Mechanical Science and Technology, 2005.
- D. Alciatore, *Typical Physical Properties of Pool Equipment* and technical
  proofs, billiards.colostate.edu.
