# ADR-0010 — Eight-ball rules engine, profiles and call-shot interface

- **Status:** accepted
- **Date:** 2026-10-18

Resolves two open questions in spec §31: the ball-and-pocket call
interface, and whether eight-ball starts casual or goes straight to full
WPA.

## Context

Phase 4 adds local eight-ball for two players on one device (spec §8.2, §9).
The spec asks for four things. Rules must be testable without the engine
(§9.12, §30.2). The player's intent (the call) must be fixed before the
simulation runs (§9.6). Judgement calls must not be faked (§8.2). And CUE
must not claim WPA conformance with known gaps (§9.1).

The reference is the WPA *Rules of Play* effective 2025-09-15. Several of its
rules assume a referee, a human body, or balls that can leave the table.
Others need facts about the shot that only the physics knows: which ball the
cue ball hit first, whether the cue ball crossed the head string, and
whether a ball was frozen to a rail.

## Decision

### A pure rules engine over shot reports

`cue_core::rules::eight_ball::Game` is a state machine with no dependency on
the solver, Bevy or the browser:

- `evaluate(shot) -> Ruling` judges a shot without changing anything. The
  shot is made of the call, the `ShotReport`, the layout before the shot and
  the table.
- `apply(&ruling)` commits a ruling.
- `decide(choice) -> Effect` resolves the options the rules give a player
  after the break, such as accept the table, re-rack or spot the 8.
- Stalemates and 1.6 spot requests are explicit calls.

A `Ruling` lists every foul detected, the balls pocketed, any group
assignment, and what happens next: continue, pass (with where the cue ball
may go), a decision, or the end of the rack. Only the most serious
consequence is enforced, as section 3 requires.

The physics supplies the facts as part of the immutable `ShotReport`. Two
additions were needed for this phase:

- **A head-string sensor:** an event each time the cue ball's centre crosses
  the string. It is found with the same polynomial root finding as contacts,
  so it is exact (3.11).
- **Frozen rail contacts:** every cushion contact carries its rail and
  whether the ball was frozen to that rail and has not left it (2.7). The
  physics knows each trajectory in closed form, so "has it left the rail?"
  is an exact maximum of a quadratic between events. No sampling is involved.

Everything above is tested natively with hand-built reports, one test per
rule (`crates/cue-core/tests/eight_ball.rs`). The game layer has its own
tests, which play real shots through the solver.

### Two named profiles, never mixed

- **`eight-ball-wpa`**: section 4 as it applies to digital play, with every
  shot called.
- **`eight-ball-casual`**: the same, except that only the 8 is called. Any
  ball of your group keeps you at the table, and the first ball pocketed
  decides the groups.

The profile is chosen when the match is set up and is shown in the setup
dialog. `docs/rules-eight-ball.md` goes through the WPA rules one by one:
implemented, not applicable to digital play, or adapted, with the test for
each and every adjudication CUE makes. The WPA profile carries that name
only because that table has no open gaps. One gap was found while writing it
(frozen balls, 2.7) and closed before release.

### The call is explicit, proposed from the aim

Spec §31 offered two options: a visual overlay on the table, or accessible
selectors. CUE does both, and makes calling cost nothing for the common
case:

- Before each shot the game **proposes a call from the aim**: the ball the
  cue ball would hit first, and the pocket most in line with that ball's
  path, if that is a legal call. If nothing legal is in line, it proposes a
  safety where a call is required. This is the digital form of WPA 1.7's
  "obvious" shots.
- The proposal is drawn on the table (rings round the called ball and
  pocket) and shown in a call button in the dock: ball, an arrow and a tiny
  table with the pocket lit, marked "auto".
- The button opens the **call dialog**. It has native radio groups for the
  ball (only legal targets are offered) and the pocket (a small table, turned
  the same way as the screen), plus **Safety** and **Follow my aim**. A
  manual call holds until the shot. Every control has a localised, readable
  name; pockets are named by screen position ("bottom-right corner").
- The call used is the one in force at the instant of the shot. It is
  recomputed from the final aim, recorded with the shot before the
  simulation runs, and never reinterpreted afterwards (§9.6).

### Decisions and results do not hide the table

A break that needs a decision opens a **panel docked over the cue
controls**, not a modal dialog. Whether to accept a table depends on seeing
it, and nothing can be shot until someone decides. The panel says why it
appeared and who chooses, offers exactly the options the rules allow, and
takes keyboard focus. The end of a rack uses the same panel, with **Next
rack**, or **New match** and **Free play** once the match is decided. Every
ruling is also announced as one sentence through the live region; for
example: "Foul by Ana: cue ball pocketed. Bo has ball in hand."

## Consequences

**Positive**

- Every rule transition is unit-tested in milliseconds, without a GPU or a
  browser, which is the Phase 4 exit criterion.
- The same engine could serve a different interface, a referee tool or
  replays, because it only reads `ShotReport`s.
- Calling is one tap or none in most shots, yet every call is explicit and
  visible before the shot.
- The conformance claim is auditable rule by rule.

**Costs**

- The proposed pocket is a geometric estimate. It ignores throw and
  cut-induced spin, so near-equal pockets can be proposed "wrong". The player
  sees the proposal and can change it before shooting.
- Rules that need a referee (stalemate observation, unsportsmanlike conduct)
  are left to the players. That is honest, but it relies on them.
- The physics report gained two concepts that only the rules use: the
  head-string sensor and frozen rail contacts. They are facts about the
  shot, not rules, so they live in the report.

## Evidence

- `cargo test -p cue-core`: the eight-ball case matrix (38 tests), the
  spotting rules and the report helpers.
- `cargo test -p cue-physics --test pockets`: the head-string sensor and
  frozen balls, on real trajectories.
- `cargo test -p cue-game`: sessions playing real shots, covering the break,
  scratch and ball in hand, decisions, spotting, the proposed call, winning,
  losing and the alternating break.

## Review conditions

- A new WPA edition, or a report of a ruling that differs from the rules.
- Usability findings on calling: if players often shoot with a wrong
  proposed pocket, the proposal needs to become stricter or more visible.
- Online play (out of scope) would need the call and decisions to become
  messages, which the engine already models as values.
