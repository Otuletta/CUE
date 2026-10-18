# Controls

Every action can be done in at least two ways: with a pointer on the table,
and with real HTML controls or the keyboard. Nothing *requires* the canvas
(spec §10, §22). The rationale is in
[ADR-0008](adr/0008-controls-and-accessible-overlay.md).

## On the table

| Action | Mouse | Touch |
|---|---|---|
| Aim | Move the pointer | Touch where the cue ball should go |
| Fine aim | Hold `Shift` while moving: the aim turns a tenth as much | Drag the ridged **fine aim** strip in the dock sideways |
| Power and shot | Press, drag **back** (away from the aim), release | Keep the finger down, drag towards the cue ball, release |
| Cancel a shot | Release with almost no pull, `Esc`, or right click | Release with almost no pull |
| Ball in hand | Press **on** the cue ball and drag it | Same |

A release with almost no pull never fires, so a click or a tap cannot shoot
by accident.

## Control layer

| Control | What it does |
|---|---|
| Fine aim (ridged strip) | Drag it sideways to turn the aim by a twentieth of a degree per pixel; dragging right turns it clockwise. `←` `→` turn a quarter of a degree, a twentieth with `Shift`. |
| Spin selector (white ball) | Drag the red dot to where the tip should strike. Arrow keys move it, `Home` or `0` centres it, double-click centres it. The dashed ring is the miscue limit. |
| Power slider | Sets the power, like dragging back on the table. Full power is a hard 7 m/s shot, or a 12 m/s stroke on the break. On a wide stage it stands at the side of the table: **pull it down and let go to shoot**, as pool games do; letting go with almost no power cancels. On a narrow stage it lies in the dock and only sets the power. Native range input: arrows, Page Up/Down, Home/End, and `Enter` shoots. |
| Shoot | Shoots with the current aim, power and spin. If the power is still at zero, it says so instead of firing. On touch devices that can vibrate, the strike and each pocketed ball give a short pulse while sound is on. |
| Toolbar | How to play · Undo (free play) · New game · Sound · Settings · Full screen · Pause · (Exit, if the host provides one) |
| Call (eight-ball) | Shows the call for the next shot: ball, then the pocket lit on a tiny table, marked *auto* while it follows the aim. Opens the call dialog. |
| Spot a ball (eight-ball) | Appears only when WPA 1.6 allows it: the cue ball in hand above the head string, and every legal ball above it too. |

## Keyboard

With the table focused (click it, or Tab to it):

| Keys | Action |
|---|---|
| `←` `→` | Aim; hold `Shift` for fine aim |
| `↑` `↓` | Power |
| `Space` or `Enter` | Shoot |
| `W` `A` `S` `D` | Move the tip (spin); `C` centres it |
| `I` `J` `K` `L` | Move the cue ball while it is in hand |
| `U` (or `Ctrl`/`⌘`+`Z`) | Undo the last shot or re-rack (free play) |
| `R` | Re-rack immediately (free play, undoable) |
| `Esc` | Cancel a shot being charged |

Anywhere in CUE:

| Key | Action |
|---|---|
| `H` or `?` | How to play |
| `P` | Pause / resume |
| `M` | Sound on / off |
| `F` | Full screen |
| `N` | New game: re-rack, start or leave a match |
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

## Eight-ball

**New game → Eight-ball for two** sets up a match on this device: two names
(display only, never stored), the rules (WPA or casual), the length (one
rack, or a race to 3 or 5) and who breaks. The rules are explained in
[rules-eight-ball.md](rules-eight-ball.md).

- **The break.** The cue ball is in hand above the head string, which is drawn
  on the cloth. You can't place it anywhere else.
- **Calling.** With WPA rules, every shot after the break is called. CUE
  proposes the call from your aim: the ball the cue ball would hit first, and
  the pocket that ball heads for. If no legal ball is in line, it proposes a
  safety. The called ball and pocket are ringed on the table. Use the **Call**
  button to pick a ball and a pocket yourself, call a **Safety**, or go back
  to **Follow my aim**. With casual rules only the 8 is called, the same way.
- **Turns.** The scoreboard shows both players, their group and how many
  balls they still need, the score in a race, and whose turn it is. Every
  ruling is shown and announced in one sentence, such as "Foul by Ana: cue
  ball pocketed. Bo has ball in hand."
- **Decisions.** After some breaks the rules let a player choose: accept the
  table, re-rack, spot the 8 and so on. A panel over the cue controls says
  who chooses and lists exactly the options the rules allow. The table stays
  visible behind it.
- **End of a rack.** The same panel names the winner and why, and offers the
  next rack. The break alternates. Once the match is decided, it offers a new
  match or free play.
- **Stalemate.** If both players agree that nobody can make progress, choose
  **New game → Declare a stalemate**. The rack is replayed and the same
  player breaks.
- There is no undo in a match. **New game** offers a new match or free play.

## The stroke

Shooting fixes the shot: its direction, power, spin and, in eight-ball, the
call. The cue then swings in from where it was drawn back, and the ball is
struck as the tip meets it. A hard shot swings in about 50 ms, a soft one up
to 300 ms, and the stick follows through a few centimetres. Nothing can
change the shot during the swing.

If the device asks for reduced motion (`prefers-reduced-motion`), there is
no swing: the ball is struck at once.

## Aiming guide

Settings → Aiming guide:

| Level | Shows |
|---|---|
| Off | Only the cue stick |
| Basic | A line to the first ball or cushion and a ghost ball there, then a short stretch of what follows: the object ball's path and the cue ball's stun tangent, or the bounce off the cushion |
| Assisted | The same, longer: each path runs to its next contact, up to 1.2 m |

The guide is an estimate. It ignores spin, throw, squirt and curve, and says
so in the settings.

## Graphics

Settings → Graphics:

- **Auto** (the default) starts at High and steps down if the game keeps
  running slowly.
- **High** has anti-aliasing and fine shadows.
- **Medium** has anti-aliasing and coarser shadows.
- **Low** has neither.

Graphics never change how the balls move.
