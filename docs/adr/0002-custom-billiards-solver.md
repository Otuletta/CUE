# ADR-0002 — Purpose-built billiards solver instead of Rapier

- **Status:** accepted (supersedes the spec's proposal of `bevy_rapier3d` 0.37)
- **Date:** 2026-10-17

## Context

The specification (§11, §14.5) proposed starting with Rapier 3D and keeping a
dedicated solver as a fallback if spin, cushion and rolling behaviour failed
acceptance tests.

Rapier is an excellent general rigid-body engine, but the behaviours that
decide whether billiards *feels* right are exactly the ones a generic contact
solver approximates poorly:

- the transition from **sliding to rolling** after a stun or draw shot;
- **rolling resistance** and **spin decay** on cloth, which must be
  frame-rate independent;
- **cushion rebound** at nose height (≈63.5 % of the ball diameter), which
  couples linear and angular velocity in a specific way;
- **throw** and spin transfer between balls.

These phenomena have well-studied closed-form models, for example:

- Leckie & Greenspan (2005), *An event-based pool physics simulator* —
  piecewise-analytic ball motion between discrete events;
- Han (2005), *Dynamics in carom and three cushion billiards* — cushion
  impulse model;
- Alciatore, *The Illustrated Principles of Pool and Billiards* and technical
  proofs — physical parameters and sanity checks.

## Decision

Implement the physics in `cue-physics` as a purpose-built solver in pure Rust,
with no dependency on Rapier or Bevy.

Balls are modelled as spheres on a plane in one of five regimes: stationary,
spinning, rolling, sliding or pocketed. Each regime has analytic equations of
motion. Collisions with balls and cushions are resolved by dedicated impulse
models. All parameters live in a versioned `PhysicsProfile`.

The **time-integration strategy** is decided by the Phase 1 spike (ADR-0007):

1. **Event-driven** (preferred): advance every ball analytically to the next
   event (collision, regime change, pocket), resolve it, repeat. This avoids
   tunnelling by construction, is independent of frame rate and produces
   exact event times for the `ShotReport`.
2. **Fixed-step with sub-steps**: simpler, but requires continuous collision
   detection and careful tolerances.

## Consequences

**Positive**

- Physics that can be calibrated against published values and real shots.
- Determinism under our control: no third-party solver ordering, no hidden
  state. Seeds, profile id and inputs fully define a shot.
- Smaller WASM: no general-purpose collision pipeline, broad phase or joints.
- Physics tests run natively in microseconds and need no ECS.
- Rendering samples ball trajectories at any instant, so visuals are smooth at
  any refresh rate without interpolation hacks.

**Costs and risks**

- More code to write and own: collision-time root finding (quartic for
  accelerating balls), pocket jaw geometry, and edge cases (frozen balls,
  simultaneous contacts).
- Correctness depends on our own tests. Mitigation: the physics case catalogue
  in spec §24.2 becomes mandatory automated tests before Phase 2 exits.

## Review conditions

Revisit only if the solver cannot pass the critical cases of spec §24.2 within
a reasonable effort. In that case, document the failing cases and evaluate
Rapier for broad-phase or collision only, keeping the `cue-physics` contracts.
