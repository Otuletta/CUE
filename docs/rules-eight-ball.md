# Eight-ball rules

CUE's eight-ball follows the **World Pool-Billiard Association, *Rules of
Play*, effective 2025-09-15**: section 4 (8-Ball) and the general rules,
definitions and fouls it refers to
([official PDF](https://www.wpapool.com/wp-content/uploads/2026/01/2026.01.02-WPA-Rules.pdf)).
This page goes through it rule by rule and says what CUE does with each.

CUE is not endorsed by the WPA. "WPA" here names the rules CUE follows, not a
certification.

## Rule profiles

| Profile | Calls | Group assignment | Turn continues on |
|---|---|---|---|
| `eight-ball-wpa` | Every shot except the break: a ball and a pocket, or a safety | The first *called* ball legally pocketed after the break | The called ball in the called pocket |
| `eight-ball-casual` | Only the 8, with its pocket | The first ball pocketed without a foul after the break | Any ball of your group |

Everything else (break options, fouls, ball in hand, losing the rack,
stalemate) is identical in both. Profiles are never mixed: a match keeps the
profile it started with (spec §9.12).

**When CUE calls a profile "WPA".** Every rule of section 4 that can happen in
digital play is implemented and has a test. Rules that cannot happen with
CUE's controls or physics are listed below as *not applicable*. Where the WPA
relies on a referee or leaves a case open, CUE makes an explicit, tested
decision, listed under [adjudications](#cues-adjudications). If a gap is ever
found, it is listed here and the profile is labelled with it until it is
fixed (spec §9.1).

## Section 4: 8-Ball

| Rule | In short | CUE | Tests |
|---|---|---|---|
| 4 | Pocket your group of seven, then the 8. Shots are called. | Implemented. | the whole matrix |
| 4.1 | The lag winner decides who breaks first. | **Adapted:** there is no lag. The players choose the breaker, or CUE draws one at random. | — |
| 4.2 | Tight triangle, apex on the foot spot, the 8 directly below the apex, one ball of each group in the back corners, the rest without pattern. | Implemented. The rest are shuffled from a seed. | `rack.rs`: `apex_is_on_the_foot_spot…`, `eight_ball_is_centred_in_the_third_row`, `back_corners_hold_one_ball_of_each_group` |
| 4.3 (a) | The cue ball starts in hand above the head string. | Implemented. The interface keeps the cue ball above the string and draws the string. | `a_new_game_breaks_from_the_kitchen` |
| 4.3 (b) | Nothing is called on the break; any ball may be hit first. | Implemented. | `calls_must_name_a_legal_ball` |
| 4.3 (c) | A ball pocketed without a foul keeps the breaker at the table; the table stays open. | Implemented. | `rule_4_3_c_…` |
| 4.3 (d) | No ball pocketed and fewer than four object balls driven to a rail: illegal break. The incoming player accepts the table, re-racks and breaks, or has the breaker break again. | Implemented, with 2.7 for frozen balls. | `rule_4_3_d_…` (three tests) |
| 4.3 (e) | The 8 on a legal break is not a foul. The breaker spots it and continues, or re-breaks. | Implemented. The 8 is spotted with 1.5. | `rule_4_3_e_…`, `a_spotted_eight_goes_back_on_the_long_string` |
| 4.3 (f) | The 8 on a foul break: the opponent spots it and takes ball in hand above the head string, or re-breaks. | Implemented. | `rule_4_3_f_…` |
| 4.3 (g) | An object ball driven off the table on the break. | **Not applicable:** in CUE's physics balls stay on the table plane. They leave it only through a pocket. | — |
| 4.3 (h) | Any other foul on the break: the next player accepts the table or takes ball in hand above the head string. | Implemented. After a scratch both options leave the same table, so CUE applies it without asking ([adjudication](#cues-adjudications)). | `rule_4_3_h_…` |
| 4.4 | The table is open after the break. Legally pocketing a called ball wins that group. On an open table any ball but the 8 may be hit first; the 8 may be hit first only once a group is already off the table, which the shooter may claim. | Implemented. | `rule_4_4_…` (four tests) |
| 4.5 | The shooter stays at the table while pocketing balls of the group legally, or by winning with the 8. | Implemented. | `rule_4_6_the_called_ball_in_another_pocket…` |
| 4.6 | Every shot but the break is called (1.7). The called ball must be from the shooter's group, and the 8 once the group is cleared. A safety passes play; balls pocketed on it stay down. | Implemented. The call is made before the shot and kept with it (spec §9.6). | `rule_4_6_…`, `wpa_requires_a_call…`, `calls_must_name_a_legal_ball` |
| 4.7 | Only the 8 is ever spotted, and only on the break. | Implemented. The 1.6 request is the one exception (see 1.6). | `rule_4_6_the_called_ball_in_another_pocket…` |
| 4.8 (a)–(c) | The shooter loses by pocketing the 8 with a foul, before the group is cleared, or in an uncalled pocket. Never on the break. | Implemented. | `rule_4_8_…` (five tests), `the_losing_rules_do_not_apply_on_the_break`, `the_eight_in_another_pocket_loses_the_rack` |
| 4.8 (d) | Driving the 8 off the table loses. | **Not applicable** (see 4.3 (g)). | — |
| 4.9 | A foul passes play with ball in hand anywhere. The standard fouls are listed below. | Implemented. | `rule_3_1_…`, `a_scratch_gives_the_opponent_ball_in_hand_anywhere` |
| 4.10 | The 4.8 fouls lose the rack; unsportsmanlike conduct is up to the referee. | Implemented for 4.8. Conduct: see 3.16. | `rule_4_8_…` |
| 4.11 | After a stalemate the original breaker breaks again. | Implemented. | `rule_4_11_…` |

## General rules

| Rule | In short | CUE |
|---|---|---|
| 1.2 | The lag. | **Adapted** (see 4.1). |
| 1.3 | In games scored by the rack the break alternates. | Implemented: the break alternates rack by rack. Tested in `the_aim_calls_the_eight_and_a_win_scores_and_alternates_the_break`. |
| 1.5 | Spot on the long string, as close as possible to the foot spot and towards the foot rail; touching an interfering ball, never the cue ball; above the foot spot if the line is full. | Implemented (`spot.rs`, four tests). "Touching" leaves the same 5 µm gap as a racked ball; the cue ball keeps 2 mm of clearance. |
| 1.6 | Ball in hand anywhere, movable until the shot. With the cue ball in hand above the head string and every legal ball above it too, the shooter may have the nearest one spotted; on equal distances the shooter picks. A ball on the string is playable. | Implemented. A **Spot a ball** button appears when the request is allowed. Equal distances: see [adjudications](#cues-adjudications). Tested in `rule_1_6_…` and `a_kitchen_shooter_may_ask…`. |
| 1.7 | The ball and the pocket are called unless obvious. Only one ball per shot. A safety may be called instead. | Implemented. The call is always explicit, and CUE proposes it from the aim ([adjudication](#cues-adjudications)). |
| 1.8 | Balls settling into a pocket. | **Not applicable:** a ball at rest in CUE never moves again. |
| 1.9–1.11 | Restoring positions, outside interference, protests. | **Not applicable:** there is no referee and no outside interference. The simulation is exact and replayable. |
| 1.12 | Conceding loses the match. | Not modelled. Leaving a match (a new match, or free play) ends it without a result. |
| 1.13 | A referee declares a stalemate after three more turns each, or the players accept one by agreement. | **Agreement only.** With no referee, CUE offers **Declare a stalemate** in the game menu, to be used when both players agree. |

## Definitions CUE depends on

| Definition | CUE |
|---|---|
| 2.1 Above the head string excludes the string itself | A ball centred exactly on the string is not above it, both for placing the cue ball and for 3.11. |
| 2.2 Ball pocketed | A ball whose centre enters a pocket's drop region falls; balls never rebound out of a pocket ([ADR-0007](adr/0007-cushion-and-pocket-model.md)). |
| 2.7 Driven to a rail | A pocketed ball counts as driven to a rail. A ball **frozen** to a rail when the shot begins is not driven to that rail unless it leaves it and comes back. The physics measures it: a ball within 10 µm of a cushion is frozen, and it has left once its centre gets more than 10 µm farther away. Pocket facings and jaws belong to the rail they are cut into. Tested in `rule_2_7_…`, `rule_4_3_d_frozen_contacts…` and, in the physics, `a_frozen_ball_…` (two tests). |
| 2.13 Position of a ball | The projection of its centre on the cloth, which is exactly what CUE stores. |
| 2.16 Safety | Declared before the shot, from the call panel. |
| 2.19 Shot | A shot ends when every ball has stopped; only then is it judged. |

## Fouls (section 3, as listed in 4.9)

| Rule | CUE |
|---|---|
| 3.1 Cue ball scratch or off the table | Implemented for pocketing. Off the table: not applicable. |
| 3.2 Wrong ball first | Implemented. Contacts within 1 ns count as simultaneous, and then the legal ball is assumed first. |
| 3.3 No rail after contact | Implemented, with 2.7. A rail touched within 1 ns of the first contact counts as after it. |
| 3.4 No foot on floor | Not applicable. |
| 3.5 Ball driven off the table | Not applicable. |
| 3.6 Touched ball | Not applicable: only the cue ball can be moved, and only while in hand. |
| 3.7 Double hit, frozen balls | A double hit cannot happen: the cue strikes once, as an impulse. Shooting into a touching ball counts as contacting it, and playing away from it does not. |
| 3.8 Push shot | Not applicable, for the same reason. |
| 3.9 Balls still moving | Prevented: no shot is accepted until every ball is at rest. |
| 3.10 Bad cue-ball placement | Prevented by the interface, which keeps the cue ball above the string. The rules check it anyway. |
| 3.11 Bad play from above the head string | Implemented. A physics sensor reports every time the cue ball crosses the head string. The second paragraph (the cue ball must cross the string or hit a ball on or below it) is covered by 3.1 and 3.3. |
| 3.12 Playing out of turn | Prevented: the engine keeps the turn. On a shared device, whoever shoots plays for the player shown as shooting. |
| 3.13 Three consecutive fouls | Not part of 8-ball (the WPA says so). |
| 3.14 Slow play | Not applicable: there is no shot clock. |
| 3.15 Ball rack template | Not applicable. |
| 3.16 Unsportsmanlike conduct | Not modelled: it needs a referee's judgement (spec §8.2). |

When a shot has several fouls, CUE lists them all and enforces only the most
serious one (section 3).

## CUE's adjudications

These are decisions where the WPA text leaves the case open, or assumes a
referee. Each is tested.

1. **An illegal break with a scratch, accepted.** 4.3 (d) and (h) both apply.
   Accepting the table puts the cue ball in hand above the head string, as
   (h) would. (`an_illegal_break_with_a_scratch_accepted…`)
2. **An illegal break that is also another foul** (for example missing the
   rack). The incoming player gets the options of 4.3 (d) and (h) together:
   accept the table, ball in hand above the head string, re-rack and break,
   or have the breaker break again. (`an_illegal_break_that_is_also_a_foul…`)
3. **A scratch on a legal break.** 4.3 (h) offers "accept the table" or "ball
   in hand above the head string". With the cue ball off the table, both give
   the same table, so CUE applies it without asking. (`rule_4_3_h_…`)
4. **Calling the shot.** The call is always explicit, never inferred after
   the shot. Before each shot CUE proposes the ball the cue ball would hit
   first and the pocket that ball heads for, if that is a legal call. If
   nothing legal is in line, it proposes a safety. The player can change the
   call at any time. The call on screen when the shot is taken is the one
   judged. This stands in for 1.7's "obvious" shots and an opponent asking
   for a call.
5. **Equal distances for a 1.6 request.** The rule lets the shooter choose.
   CUE spots the lowest-numbered ball; in a continuous simulation an exact
   tie is practically impossible.
6. **Simultaneity.** Physically simultaneous contacts can be resolved a few
   nanoseconds apart by rounding, so contacts within **1 ns** are treated as
   simultaneous (3.2, 3.3).
7. **Frozen balls.** WPA 2.7 assumes a ball is not frozen unless someone
   declares it. CUE has exact geometry, so it measures instead, with a 10 µm
   tolerance (see 2.7).
8. **Stalemate.** Only by agreement (see 1.13).

## Where it lives

- **Rules:** `crates/cue-core/src/rules/`. This is a pure state machine over
  `ShotReport`s, with no engine and no physics; see
  [ADR-0010](adr/0010-eight-ball-rules-and-call-shot.md).
- **Case matrix:** `crates/cue-core/tests/eight_ball.rs`. Every test builds a
  shot report by hand and is named after the rule it checks.
- **Facts the rules need from the physics:** `crates/cue-physics`
  (`tests/pockets.rs`), covering the head-string sensor and frozen balls.
- **Turns, spotting and the table:** `crates/cue-game/src/eight.rs`. Its
  tests play real shots through the physics.
