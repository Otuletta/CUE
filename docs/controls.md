# Controls

Every action can be done in at least two ways: with a pointer on the table,
and with real HTML controls or the keyboard. Nothing *requires* the canvas
(spec §10, §22). The rationale is in
[ADR-0008](adr/0008-controls-and-accessible-overlay.md).

## On the table

| Action | Mouse | Touch |
|---|---|---|
| Aim | Move the pointer | Touch where the cue ball should go |
| Power and shot | Press, drag **back** (away from the aim), release | Keep the finger down, drag towards the cue ball, release |
| Cancel a shot | Release with almost no pull, `Esc`, or right click | Release with almost no pull |
| Ball in hand | Press **on** the cue ball and drag it | Same |

A release with almost no pull never fires, so a click or a tap cannot shoot
by accident.

## Control layer

| Control | What it does |
|---|---|
| Spin selector (white ball) | Drag the red dot to where the tip should strike. Arrow keys move it, `Home` or `0` centres it, double-click centres it. The dashed ring is the miscue limit. |
| Power slider | Sets the power, like dragging back on the table. Native range input: arrows, Page Up/Down, Home/End. |
| Shoot | Shoots with the current aim, power and spin. If the power is still at zero, it says so instead of firing. |
| Toolbar | How to play · Undo · New rack · Sound · Settings · Full screen · Pause · (Exit, if the host provides one) |

## Keyboard

With the table focused (click it, or Tab to it):

| Keys | Action |
|---|---|
| `←` `→` | Aim; hold `Shift` for fine aim |
| `↑` `↓` | Power |
| `Space` or `Enter` | Shoot |
| `W` `A` `S` `D` | Move the tip (spin); `C` centres it |
| `I` `J` `K` `L` | Move the cue ball while it is in hand |
| `U` (or `Ctrl`/`⌘`+`Z`) | Undo the last shot or re-rack |
| `R` | Re-rack immediately (undoable) |
| `Esc` | Cancel a shot being charged |

Anywhere in CUE:

| Key | Action |
|---|---|
| `H` or `?` | How to play |
| `P` | Pause / resume |
| `M` | Sound on / off |
| `F` | Full screen |
| `N` | New rack (asks first) |
| `O` | Settings |
| `` ` `` | Developer overlay (frame rate, state) |

## Free play

- A new rack starts with the cue ball **in hand**: place it anywhere free
  before the break.
- After a scratch the cue ball comes back in hand near the head spot.
- Clearing the table racks a new set automatically.
- **Undo** puts the table back exactly as it was before the last shot or
  re-rack, even mid-shot. It restores a full snapshot of the simulation; it
  does not rewind physics (spec §8.1).

## Aiming guide

Settings → Aiming guide:

| Level | Shows |
|---|---|
| Off | Only the cue stick |
| Basic | A line to the first ball or cushion, and a ghost ball |
| Assisted | Also the object ball's path and the cue ball's stun tangent |

The guide is an estimate. It ignores spin, throw, squirt and curve, and says
so in the settings.
