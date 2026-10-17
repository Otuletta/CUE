# ADR-0007 — Cushion, pocket and contact models

- **Status:** accepted (resolves open decision ADR-008 in spec §31: friction and spin model)
- **Date:** 2026-10-18

## Context

Phase 2 needed pockets, cushions that respond to spin, and ball–ball
friction. The literature offers a spectrum of options:

- Han (2005): an instantaneous impulse at the cushion nose.
- Mathavan et al. (2010): numerical integration through the impact, with
  table friction.
- Full compliant-contact simulations.

Each step up costs complexity and makes determinism harder to guarantee.

## Decision

**Geometry.** Cushions are straight segments: the six rail sections plus two
facings per pocket at the WPA facing angles (142° corners, 104° sides). Every
jaw point and facing end is a sharp vertex. Each pocket is a **drop region**:
a circle set back by the WPA shelf depth, entered by the ball's *centre*.
Construction rejects any table whose drop region does not cover the whole
throat.

**Contacts.** Both ball–cushion and ball–ball use one recipe: a free-sphere
Newton–Coulomb impulse at the true contact point, with stick or slide
friction, after which the slate absorbs any vertical velocity.

- The cushion contact sits at nose height, after Han (2005).
- Ball–ball friction falls with slip speed, using Alciatore's fit to measured
  throw.

**Squirt** follows Alciatore's endmass formula. Swerve, jump and massé are not
modelled.

## Consequences

**Positive**

- Throw, spin transfer, side-spin rebound angles and rail-induced spin all
  come out of the model; none is a special case.
- Energy can never increase, and ball–ball momentum is conserved. Property
  tests check both over random states.
- Pockets behave like pockets: straight-in and rail-frozen shots drop, a shot
  aimed at the point bounces out, and a side pocket takes 45° but rejects 15°.
- The full Phase 2 physics adds about 11 KiB (Brotli) to the WASM bundle.

**Costs and risks**

- Instantaneous impacts ignore the compression time of the cushion rubber, so
  effects that depend on it, like the slight "throw" off a rail, are absent.
- Jaws are sharp and the drop region is a circle. Rattles are approximate,
  and there is no pocket liner for a ball to bounce off once it is past the
  throat.
- Parameters are validated against textbook behaviour, not fitted to video of
  a real table. The honest claim is convincing game physics, not a replica
  (spec §11.10).

## Evidence

- 44 unit tests and 19 reference-shot and pocket scenarios in `cue-physics`,
  including the energy and momentum property tests.
- 1,000-shot stress test with pockets, re-spotting and re-racking: 18,452
  events in about 90 ms native, with every invariant checked after every
  event.
- Bit-identical outcome at any frame rate, still asserted on a full break.

## Review conditions

Revisit if a calibration against filmed reference shots shows rebound angles
or pocket acceptance off by more than the tolerances that the Phase 5
calibration sets. The candidate upgrade is Mathavan (2010) for the cushion,
which keeps the same interfaces.
