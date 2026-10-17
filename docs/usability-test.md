# Usability test script (Phase 3 exit)

**Exit criterion (spec §27, Phase 3):** a visitor can take a shot, understand
the result, reset and leave **without any explanation from you**.

Run this with at least **five people who have never seen CUE**: three on a
phone and two on a laptop. Mix people who play pool with people who do not.
It takes about ten minutes each.

## Setup

- Use the live demo, <https://otuletta.github.io/CUE/>, in a private window
  so the first-visit tutorial appears.
- Say only: *"This is a pool game on a web page. Please think out loud. I
  can't help you, but everything you find hard is useful."*
- Do not point, hint or answer questions about controls. If someone is stuck
  for two minutes, note it and move on to the next task.

## Tasks

| # | Ask them to… | Watch for |
|---|---|---|
| 1 | Start the game. | Do they find Play? Do they read the tutorial or skip it? |
| 2 | Take any shot. | Time to first shot. Accidental shots. Do they discover drag-back, or the slider and Shoot? |
| 3 | Try to pot a ball. | Aim precision. Do they find or understand the guide? |
| 4 | Tell me what happened in the last shot. | Do they notice the toast? Can they say which balls went in? |
| 5 | Put spin on the ball. | Do they find the spin selector? Do they understand it? |
| 6 | Undo that shot. | Do they find Undo? |
| 7 | Start again with all the balls. | New rack button, the `R` key, or neither? |
| 8 | Turn off the sound. | — |
| 9 | Leave the game. | Exit button, browser back, or confusion? |

## Record for each person

- Device and browser.
- Seconds to the first shot, and the number of accidental shots.
- Every place they hesitated for more than 5 seconds, in their own words.
- Tasks failed.
- One thing they liked and one they disliked.

## Decision rule

- **Pass:** every participant completes tasks 2, 4, 7 and 9 unaided, and no
  task fails for more than one participant.
- Otherwise, fix the top issue, record the change in ADR-0008, and run the
  test again with new people.

Findings go in `docs/benchmarks/phase-3-usability.md`.
