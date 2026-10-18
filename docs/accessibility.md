# Accessibility

CUE applies WCAG 2.2 principles to the interface **around** the canvas. It
does not claim that a real-time 3D game is universally accessible; that would
need validation with assistive technology users, which has not happened yet
(spec §22.1).

## What is in place

| Need | How CUE meets it |
|---|---|
| Operable without a pointer | Full keyboard path: aim, power, spin, shoot, placing the cue ball, undo, pause, settings. The HTML power slider and Shoot button replace the drag gesture. |
| Real controls with names | Toolbar buttons, a native range input, radio groups and checkboxes, all named and localised (English, Spanish). The spin selector is a focusable `slider` whose `aria-valuetext` describes the tip position ("follow 30 %, left 10 %"). |
| Feedback without sound | Every shot result appears as text ("Pocketed 3 and 11", "Scratch: the cue ball is back in hand") and is announced through a polite live region. In eight-ball, each ruling is one announced sentence that names the foul, the groups and who plays next. Sound is never the only signal. |
| Eight-ball without the canvas | Calls are made with native radio groups, offering only legal balls, and a pocket picker whose options are named by screen position ("bottom-right corner"). Decisions and rack results are real buttons in a labelled panel that takes focus when it appears. The scoreboard is text, with visually hidden detail for screen readers ("solids, 5 left", "shooting"). |
| Focus | Visible focus rings. Dialogs trap focus while open, close with `Esc` and return focus to where it was, or to the table if that control has gone. No keyboard trap: Tab leaves CUE normally when no dialog is open. Typing a player's name never triggers a shortcut. |
| Colour | Groups differ by pattern (solid or stripe) as well as colour, on the table, in the call dialog and on the scoreboard. The **High contrast** theme uses a mid-grey cloth that separates the black and the white balls equally well, near-black rails, and a thicker yellow guide. |
| Motion | No camera animation. `prefers-reduced-motion` turns off interface transitions and toasts animate in place. |
| Time | Nothing is timed. Balls rolling never require a reaction. |
| Touch | 44 px targets on coarse pointers. Gestures are captured only on the canvas, never on the host page. |
| Language | `lang` is set on the control layer; strings follow the browser's language or `createCue({ locale })`. |
| Without WebGL | The control layer explains what is missing and links to the project instead of showing a blank box. |

## Known limits

- The table itself is a canvas. A screen reader hears the shot results and
  the state ("Ball in hand", "3 balls left"), but not the positions of the
  balls. An audio or text "table description" mode is possible future work
  (spec §22.3).
- Aiming precisely by keyboard is slower than with a pointer.
- Not yet tested with VoiceOver, NVDA or TalkBack users.

## How to check

1. Unplug the mouse. Tab to the table, play a shot, open Settings, change the
   theme, close it with `Esc`, and confirm focus returns.
2. Turn on a screen reader, play a shot, and confirm the result is announced.
3. Turn on "Reduce motion" in the OS settings and confirm the interface
   no longer animates.
4. Switch to the High contrast theme and check every ball is distinguishable.
