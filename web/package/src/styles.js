// Styles for the overlay. They live inside its shadow root, so nothing here
// can leak into the host page and nothing from the host can break it.

export const STYLES = /* css */ `
:host { all: initial; }

/* Design: a dark club room. Smoked glass with hairline edges, one warm
   champagne accent, a serif for titles, and soft, quick motion. */
.ui {
  --glass: rgba(15, 16, 20, 0.66);
  --glass-strong: rgba(17, 18, 23, 0.95);
  --line: rgba(255, 255, 255, 0.1);
  --line-strong: rgba(255, 255, 255, 0.2);
  --hi: inset 0 1px 0 rgba(255, 255, 255, 0.07);
  --lift: 0 10px 30px rgba(0, 0, 0, 0.34);
  --blur: blur(14px) saturate(140%);
  --fg: #f5f2eb;
  --muted: #b8b2a7;
  --accent: #d9b56c;
  --accent-hi: #efd597;
  --accent-lo: #b88c43;
  --accent-fg: #1c160a;
  --gold: linear-gradient(180deg, var(--accent-hi), var(--accent-lo));
  --glow: 0 6px 18px rgba(217, 181, 108, 0.24);
  --danger: #ff6b6b;
  --radius: 14px;
  --tap: 40px;
  --serif: 'Iowan Old Style', 'Palatino Linotype', Palatino, 'Book Antiqua', Georgia, ui-serif, serif;
  --ease: cubic-bezier(0.2, 0.8, 0.2, 1);
  position: absolute;
  inset: 0;
  container-type: size;
  pointer-events: none;
  color: var(--fg);
  font: 14px/1.45 ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  letter-spacing: 0.005em;
  -webkit-font-smoothing: antialiased;
  user-select: none;
  -webkit-user-select: none;
}

.ui[data-theme='contrast'] {
  --glass: rgba(0, 0, 0, 0.92);
  --glass-strong: #000;
  --line: #fff;
  --line-strong: #fff;
  --muted: #fff;
  --accent: #ffe100;
  --accent-hi: #ffe100;
  --accent-lo: #ffe100;
  --accent-fg: #000;
  --gold: #ffe100;
  --glow: none;
  --hi: none;
  font-size: 15px;
}

@media (pointer: coarse) {
  .ui { --tap: 44px; }
}

.ui :is(button, input, a, label, [tabindex], .card) { pointer-events: auto; }

button {
  font: inherit;
  color: inherit;
  cursor: pointer;
}

button:focus-visible,
input:focus-visible,
a:focus-visible,
[tabindex]:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

/* Top bar ------------------------------------------------------------- */

.top {
  position: absolute;
  top: max(8px, env(safe-area-inset-top));
  left: max(8px, env(safe-area-inset-left));
  right: max(8px, env(safe-area-inset-right));
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  flex-wrap: wrap-reverse;
}

/* Glass surfaces share one recipe. */
:is(.status, .toolbar, .controls, .player, .board .line, .call, .chip, .toast) {
  background: var(--glass);
  border: 1px solid var(--line);
  box-shadow: var(--hi), var(--lift);
  backdrop-filter: var(--blur);
  -webkit-backdrop-filter: var(--blur);
}

.status {
  margin: 0;
  padding: 8px 15px;
  border-radius: 999px;
  font-size: 13px;
  display: flex;
  gap: 10px;
  align-items: baseline;
  max-width: 100%;
}

.status .mode {
  font-weight: 650;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  font-size: 11.5px;
  color: var(--accent);
}
.status .detail { color: var(--muted); }
.ui[data-in-hand] .status .detail { color: var(--accent); }

/* One pill holds every tool, with a hairline before the pause group. */
.toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: auto;
  padding: 3px;
  border-radius: 999px;
}

.icon {
  position: relative;
  width: calc(var(--tap) - 2px);
  height: calc(var(--tap) - 2px);
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 999px;
  background: transparent;
  transition: background 0.16s var(--ease), color 0.16s var(--ease), transform 0.1s;
}

.icon:hover { background: rgba(255, 255, 255, 0.09); }
.icon:active { transform: scale(0.94); }
.icon:disabled { opacity: 0.32; cursor: default; background: transparent; }
.icon[aria-pressed='false'] { color: var(--muted); }

.icon[data-action='pause'] { margin-left: 7px; }
.icon[data-action='pause']::before {
  content: '';
  position: absolute;
  left: -5px;
  top: 25%;
  bottom: 25%;
  width: 1px;
  background: var(--line-strong);
}

/* Bottom dock --------------------------------------------------------- */

.dock {
  position: absolute;
  bottom: max(8px, env(safe-area-inset-bottom));
  left: max(8px, env(safe-area-inset-left));
  right: max(8px, env(safe-area-inset-right));
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  transition: opacity 0.2s;
}

.ui[data-moving] .dock { opacity: 0.45; }

.controls {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px;
  border-radius: 999px;
  min-width: 0;
}

.spin {
  --size: 52px;
  position: relative;
  width: var(--size);
  height: var(--size);
  flex: none;
  border-radius: 50%;
  background: radial-gradient(circle at 36% 30%, #ffffff, #ece8dd 48%, #b7b1a2 100%);
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.5), 0 4px 12px rgba(0, 0, 0, 0.45),
    inset 0 -6px 10px rgba(0, 0, 0, 0.12);
  touch-action: none;
  cursor: crosshair;
}

@media (pointer: coarse) {
  .spin { --size: 54px; }
}

/* Fine aim: a ridged strip dragged sideways, like a jog wheel. */
.jog {
  position: relative;
  flex: none;
  width: clamp(64px, 14vw, 120px);
  height: var(--tap);
  overflow: hidden;
  border-radius: 999px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.02));
  box-shadow: inset 0 0 0 1px var(--line), inset 0 2px 6px rgba(0, 0, 0, 0.35);
  touch-action: none;
  cursor: ew-resize;
}

.jog:hover { box-shadow: inset 0 0 0 1px var(--line-strong), inset 0 2px 6px rgba(0, 0, 0, 0.35); }

.jog .ridges {
  position: absolute;
  inset: 8px -24px;
  background: repeating-linear-gradient(90deg, rgba(239, 213, 151, 0.42) 0 1.5px, transparent 1.5px 7px);
  transform: translateX(var(--shift, 0px));
  pointer-events: none;
  /* Fades out at both ends, as a wheel curving away. */
  -webkit-mask-image: linear-gradient(90deg, transparent, #000 30%, #000 70%, transparent);
  mask-image: linear-gradient(90deg, transparent, #000 30%, #000 70%, transparent);
}

.spin::before {
  /* The miscue limit: the tip may go anywhere inside this ring. */
  content: '';
  position: absolute;
  inset: 10%;
  border-radius: 50%;
  border: 1px dashed rgba(0, 0, 0, 0.25);
}

.spin .dot {
  position: absolute;
  left: 50%;
  top: 50%;
  width: 12px;
  height: 12px;
  margin: -6px 0 0 -6px;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 35%, #ff6b5a, #c41f1f);
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.9), 0 1px 4px rgba(0, 0, 0, 0.4);
  pointer-events: none;
}

.power {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  height: var(--tap);
  padding: 0 6px 0 10px;
}

.power > span:first-child {
  font-size: 11px;
  font-weight: 650;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}

/* The side gauge's drawing; the native slider does the work. */
.power .gauge { display: none; }

.power output {
  min-width: 3.2em;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* Sliders: a hairline track filled in gold up to the value (--fill, set
   by the controls), and a gold thumb. */
input[type='range'] {
  width: clamp(110px, 26vw, 260px);
  height: 28px;
  margin: 0;
  background: transparent;
  -webkit-appearance: none;
  appearance: none;
  touch-action: none;
  cursor: pointer;
}

input[type='range']::-webkit-slider-runnable-track {
  height: 6px;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--accent) var(--fill, 0%), rgba(255, 255, 255, 0.14) var(--fill, 0%));
}

input[type='range']::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  margin-top: -6px;
  border: 0;
  border-radius: 50%;
  background: var(--gold);
  box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.35), var(--hi);
}

input[type='range']::-moz-range-track {
  height: 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.14);
}

input[type='range']::-moz-range-progress {
  height: 6px;
  border-radius: 999px;
  background: var(--accent);
}

input[type='range']::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border: 0;
  border-radius: 50%;
  background: var(--gold);
  box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.35);
}

/* Wide stages: the power slider stands at the side of the table, pulled down
   for more power and released to shoot. */
@container (orientation: landscape) and (min-width: 720px) {
  /* Placed from the dock, which sits 8 px above the stage's bottom: this
     centres the bar on the stage's height. */
  .power {
    position: absolute;
    left: 2px;
    bottom: calc(24cqh - 8px);
    height: 52cqh;
    width: 54px;
    padding: 14px 0 12px;
    flex-direction: column;
    justify-content: space-between;
    gap: 10px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid var(--line);
    box-shadow: var(--hi), var(--lift);
    backdrop-filter: var(--blur);
    -webkit-backdrop-filter: var(--blur);
  }
  .power > span:first-child { display: none; }
  /* A backdrop filter would make the pill the gauge's containing block and
     keep it from reaching the stage's side: the pill goes solid instead. */
  .dock .controls { backdrop-filter: none; -webkit-backdrop-filter: none; background: var(--glass-strong); }
  /* A gauge filling downwards, warming towards full power, with ticks. */
  .power .gauge {
    display: block;
    position: relative;
    flex: 1;
    width: 14px;
    min-height: 0;
    border-radius: 999px;
    background:
      repeating-linear-gradient(180deg, transparent 0 calc(10% - 1px), rgba(255, 255, 255, 0.16) calc(10% - 1px) 10%),
      rgba(255, 255, 255, 0.06);
    box-shadow: inset 0 0 0 1px var(--line), inset 0 2px 6px rgba(0, 0, 0, 0.45);
    overflow: hidden;
    pointer-events: none;
  }
  .power .gauge .fill {
    position: absolute;
    inset: 0 0 auto;
    height: var(--fill, 0%);
    border-radius: 999px;
    background: linear-gradient(180deg, var(--accent-hi), var(--accent) 55%, #e07a3c);
    box-shadow: 0 0 12px rgba(239, 213, 151, 0.35);
  }
  /* The slider itself lies over the gauge, unseen, and takes the drag. */
  .power input[type='range'] {
    writing-mode: vertical-lr;
    position: absolute;
    inset: 14px 0 34px;
    width: 100%;
    height: auto;
    opacity: 0;
    cursor: ns-resize;
  }
  .power:has(input:focus-visible) .gauge { outline: 2px solid var(--accent); outline-offset: 3px; }
  .power output { min-width: 0; font-size: 12px; font-weight: 650; text-align: center; font-variant-numeric: tabular-nums; }
}

.shoot {
  height: var(--tap);
  padding: 0 22px;
  border: 0;
  border-radius: 999px;
  background: var(--gold);
  color: var(--accent-fg);
  font-weight: 750;
  font-size: 12.5px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  box-shadow: var(--glow), inset 0 1px 0 rgba(255, 255, 255, 0.45);
  transition: filter 0.16s var(--ease), transform 0.1s;
}

.shoot:hover { filter: brightness(1.06); }
.shoot:active { transform: scale(0.97); }
.shoot[aria-disabled='true'] { filter: saturate(0.35) brightness(0.75); box-shadow: none; }

/* Toast ----------------------------------------------------------------- */

.toast {
  position: absolute;
  /* Centred by margins, not a 50% offset, so it may use the full width. */
  left: 0;
  right: 0;
  margin-inline: auto;
  width: fit-content;
  box-sizing: border-box;
  top: calc(max(8px, env(safe-area-inset-top)) + 56px);
  transform: translateY(-6px);
  max-width: min(90%, 460px);
  padding: 9px 18px;
  border-radius: 999px;
  text-wrap: balance;
  font-weight: 550;
  text-align: center;
  opacity: 0;
  transition: opacity 0.2s, transform 0.2s;
}

.toast[data-visible] {
  opacity: 1;
  transform: translateY(0);
}

/* Sheets and dialogs ------------------------------------------------------ */

.sheet {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 16px;
  background: radial-gradient(ellipse at 50% 40%, rgba(6, 7, 9, 0.35), rgba(6, 7, 9, 0.7));
  backdrop-filter: blur(3px);
  -webkit-backdrop-filter: blur(3px);
  pointer-events: auto;
}

.sheet[hidden] { display: none; }
.sheet.bare { background: transparent; pointer-events: none; }

.card {
  --pad-y: 20px;
  --pad-x: 22px;
  --card: #16171c;
  position: relative;
  box-sizing: border-box;
  width: min(100%, 460px);
  max-height: calc(100% - 16px);
  overflow: auto;
  padding: var(--pad-y) var(--pad-x);
  border-radius: 22px;
  background: linear-gradient(180deg, #1d1e24, var(--card) 140px);
  border: 1px solid var(--line);
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55), var(--hi);
  animation: rise 0.22s var(--ease);
  user-select: text;
  -webkit-user-select: text;
}

.ui[data-theme='contrast'] .card { --card: #000; background: #000; }

@keyframes rise {
  from { opacity: 0; transform: translateY(10px) scale(0.985); }
}

/* Close, in the corner of a dialog. */
.card .x {
  position: absolute;
  top: 14px;
  right: 14px;
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  color: var(--muted);
  font-size: 20px;
  line-height: 1;
  transition: background 0.16s var(--ease), color 0.16s var(--ease);
}

.card .x:hover { background: rgba(255, 255, 255, 0.12); color: var(--fg); }

/* A card's way out stays in view while a long card scrolls on a small
   stage. */
.card > .actions {
  position: sticky;
  bottom: calc(-1 * var(--pad-y));
  margin: 18px calc(-1 * var(--pad-x)) calc(-1 * var(--pad-y));
  padding: 12px var(--pad-x) var(--pad-y);
  background: var(--card);
  border-top: 1px solid var(--line);
}

.card h2 {
  margin: 0 44px 14px 0;
  font: 600 23px/1.2 var(--serif);
  letter-spacing: 0.005em;
}

.card h3 {
  margin: 18px 0 8px;
  font-size: 11px;
  font-weight: 700;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.14em;
}

.card p { margin: 0 0 12px; color: var(--muted); }

.card :is(ol, ul) {
  margin: 0;
  padding-left: 1.3em;
}

.card li { margin: 0 0 8px; }

.card dl {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 4px 14px;
  margin: 0;
}

.card dt {
  font: 600 11.5px/1.7 ui-monospace, SFMono-Regular, Menlo, monospace;
  color: var(--fg);
}

.card ol { counter-reset: step; list-style: none; padding-left: 0; }
.card ol > li { counter-increment: step; position: relative; padding-left: 2.1em; }
.card ol > li::before {
  content: counter(step);
  position: absolute;
  left: 0;
  top: 0.05em;
  width: 1.45em;
  height: 1.45em;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: rgba(217, 181, 108, 0.16);
  color: var(--accent);
  font: 650 11.5px/1 var(--serif);
}

.card dd { margin: 0; color: var(--muted); }

.card a { color: var(--accent); }

.card fieldset {
  margin: 0 0 16px;
  padding: 0;
  border: 0;
}

.card legend {
  margin-bottom: 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
}

/* Segmented controls: the radios stay real for keyboards and screen
   readers, and the chosen segment wears the accent. */
/* One row, always: a long name wraps inside its own segment. */
.choices {
  display: flex;
  flex-wrap: nowrap;
  gap: 3px;
  padding: 3px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 0 0 1px var(--line);
}

.choices label {
  position: relative;
  flex: 1 1 0;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 38px;
  padding: 5px 12px;
  text-align: center;
  line-height: 1.2;
  border-radius: 11px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.16s var(--ease), color 0.16s var(--ease);
}

.choices input {
  position: absolute;
  opacity: 0;
  width: 1px;
  height: 1px;
  margin: 0;
  pointer-events: none;
}

.choices label:hover { color: var(--fg); background: rgba(255, 255, 255, 0.05); }

.choices label:has(input:checked) {
  color: var(--accent-fg);
  background: var(--gold);
  box-shadow: var(--glow), inset 0 1px 0 rgba(255, 255, 255, 0.4);
  font-weight: 650;
}

.choices label:has(input:focus-visible) { outline: 2px solid var(--accent); outline-offset: 2px; }

/* A theme's swatch: its cloth framed in its wood. */
.swatch-theme[data-swatch='classic'] { --cloth: #0f5a45; --wood: #6e3f22; }
.swatch-theme[data-swatch='studio'] { --cloth: #0d3a66; --wood: #4a3122; }
.swatch-theme[data-swatch='contrast'] { --cloth: #8a8f96; --wood: #050505; }

.choices .swatch-theme {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--cloth);
  box-shadow: 0 0 0 2.5px var(--wood), 0 0 0 3.5px rgba(255, 255, 255, 0.25);
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 0 0 12px;
}

/* Switches: a checkbox with role="switch", drawn as a pill. */
.row label:has(input[type='checkbox']) {
  flex: 1;
  display: flex;
  flex-direction: row-reverse;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
}

input[type='checkbox'] {
  position: relative;
  flex: none;
  width: 42px;
  height: 25px;
  margin: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.16);
  -webkit-appearance: none;
  appearance: none;
  cursor: pointer;
  transition: background 0.2s var(--ease);
}

input[type='checkbox']::after {
  content: '';
  position: absolute;
  top: 3px;
  left: 3px;
  width: 19px;
  height: 19px;
  border-radius: 50%;
  background: #f5f2eb;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.45);
  transition: transform 0.2s var(--ease);
}

input[type='checkbox']:checked { background: var(--accent); }
input[type='checkbox']:checked::after { transform: translateX(17px); }
.row label:has(input[type='range']) { flex: 1; display: flex; align-items: center; justify-content: space-between; gap: 12px; }

.note { font-size: 12px; }

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
  flex-wrap: wrap;
}

.actions button,
.actions a {
  min-height: var(--tap);
  padding: 0 20px;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.05);
  color: var(--fg);
  font: inherit;
  font-weight: 550;
  display: inline-flex;
  align-items: center;
  text-decoration: none;
  transition: background 0.16s var(--ease), filter 0.16s var(--ease), transform 0.1s;
}

.actions button:hover,
.actions a:hover { background: rgba(255, 255, 255, 0.1); }
.actions button:active { transform: scale(0.97); }

.actions .primary {
  background: var(--gold);
  border-color: transparent;
  color: var(--accent-fg);
  font-weight: 700;
  box-shadow: var(--glow), inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

.actions .primary:hover { background: var(--gold); filter: brightness(1.06); }

.progress {
  height: 6px;
  margin-top: 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.12);
  overflow: hidden;
}

.progress span {
  display: block;
  height: 100%;
  width: var(--p, 30%);
  background: var(--gold);
  transition: width 0.15s;
}

.progress[data-indeterminate] span {
  width: 30%;
  animation: slide 1.1s ease-in-out infinite;
}

@keyframes slide {
  from { transform: translateX(-100%); }
  to { transform: translateX(340%); }
}

/* Eight-ball: scoreboard ----------------------------------------------- */

.ui[data-mode='eight-ball'] :is(.status, [data-action='undo']) { display: none; }

.board {
  display: grid;
  gap: 4px;
  justify-items: start;
  max-width: 100%;
  min-width: 0;
}

.board[hidden] { display: none; }

.players {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-width: 100%;
}

.player {
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: 38px;
  padding: 4px 10px 4px 4px;
  border-radius: 999px;
  font-size: 13px;
  min-width: 0;
  transition: border-color 0.25s var(--ease), box-shadow 0.25s var(--ease);
}

/* The player's initial, gold on their turn. */
.player::before {
  content: attr(data-initial);
  flex: none;
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.08);
  color: var(--fg);
  font: 600 14px/1 var(--serif);
  transition: background 0.25s var(--ease), color 0.25s var(--ease);
}

.player[data-active] {
  border-color: rgba(217, 181, 108, 0.55);
  box-shadow: 0 0 0 1px rgba(217, 181, 108, 0.25), 0 0 26px rgba(217, 181, 108, 0.16), var(--hi), var(--lift);
}

.player[data-active]::before { background: var(--gold); color: var(--accent-fg); }

.player .name {
  font-weight: 600;
  max-width: 14ch;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.player .group {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.player .group:empty { display: none; }

.player .score {
  min-width: 1.8em;
  padding: 2px 6px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.08);
  text-align: center;
  font: 650 13px/1.3 var(--serif);
  font-variant-numeric: tabular-nums;
}

.player .score[hidden] { display: none; }

.rack-balls { display: inline-flex; gap: 3px; }

.mark.mini {
  width: 11px;
  height: 11px;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.35);
  transition: opacity 0.3s, transform 0.3s;
}

/* A pocketed ball pops, then fades back. */
.mark.mini[data-gone] {
  opacity: 0.2;
  transform: scale(0.72);
  animation: pocketed 0.55s ease-out;
}

@keyframes pocketed {
  0% { opacity: 1; transform: scale(1.6); box-shadow: 0 0 0 3px var(--accent); }
  100% { opacity: 0.2; transform: scale(0.72); }
}

/* Banners: a few big words for a turn, a foul or a win. */
.banner {
  position: absolute;
  left: 0;
  right: 0;
  top: 34%;
  margin-inline: auto;
  width: fit-content;
  max-width: calc(100% - 32px);
  padding: 14px 30px;
  border-radius: 20px;
  background: linear-gradient(180deg, rgba(30, 31, 37, 0.94), rgba(16, 17, 22, 0.94));
  border: 1px solid var(--line);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), var(--hi);
  backdrop-filter: var(--blur);
  -webkit-backdrop-filter: var(--blur);
  text-align: center;
  pointer-events: none;
  visibility: hidden;
  opacity: 0;
}

.banner[data-visible] {
  visibility: visible;
  opacity: 1;
  animation: banner-in 0.34s cubic-bezier(0.2, 0.9, 0.25, 1.15);
}

.banner[data-leaving] { opacity: 0; transform: translateY(6px); transition: opacity 0.35s, transform 0.35s; }

@keyframes banner-in {
  0% { opacity: 0; transform: translateY(-14px) scale(0.9); }
  100% { opacity: 1; transform: none; }
}

.banner strong {
  display: block;
  font: 600 clamp(20px, 4.4cqw, 32px)/1.15 var(--serif);
  letter-spacing: 0.01em;
}

.banner span { display: block; margin-top: 2px; font-size: 13px; color: var(--muted); }
.banner span[hidden] { display: none; }
.banner[data-kind='foul'] { border-color: rgba(255, 92, 96, 0.7); }
.banner[data-kind='foul'] strong { color: #ff6b6f; }
.banner[data-kind='win'] { border-color: var(--accent); }
.banner[data-kind='win'] strong { color: var(--accent); }

/* The win: a short burst of confetti over the table. */
.confetti { position: absolute; inset: 0; overflow: hidden; pointer-events: none; }

.confetti i {
  position: absolute;
  top: -12px;
  left: var(--x);
  width: 7px;
  height: 11px;
  border-radius: 2px;
  opacity: 0;
  animation: confetti 2.4s cubic-bezier(0.25, 0.6, 0.5, 1) var(--delay) forwards;
}

@keyframes confetti {
  0% { opacity: 1; transform: translate(0, 0) rotate(0); }
  100% { opacity: 0; transform: translate(var(--drift), 105cqh) rotate(var(--turn)); }
}

.swatch {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  border: 1.5px solid var(--fg);
}

.swatch.solids { background: var(--fg); }
.swatch.stripes { background: linear-gradient(transparent 0 30%, var(--fg) 30% 70%, transparent 70%); }

.board .line {
  margin: 0;
  padding: 5px 12px;
  border-radius: var(--radius);
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  font-size: 12.5px;
  color: var(--muted);
  display: flex;
  flex-wrap: wrap;
  gap: 0 10px;
  max-width: 100%;
}

.board .line .rack:empty { display: none; }
.board .line .turn { color: var(--fg); }

/* Eight-ball: the call and the spot request --------------------------- */

.call,
.chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: var(--tap);
  padding: 0 14px;
  border-radius: 999px;
  min-width: 0;
}

.call { border-color: rgba(217, 181, 108, 0.55); margin-right: auto; max-width: 100%; }
.chip { flex: none; }
:is(.call, .chip)[hidden] { display: none; }

.call .call-label { color: var(--muted); font-size: 12.5px; }

.call .call-body {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.call .call-body b { font-variant-numeric: tabular-nums; }
.call .arrow { color: var(--muted); }

.pocket-icon { display: block; flex: none; }
.pocket-icon rect { fill: none; stroke: currentColor; stroke-width: 1.3; opacity: 0.6; }
.pocket-icon circle { fill: currentColor; opacity: 0.45; }
.pocket-icon circle.lit { fill: var(--accent); opacity: 1; }

.call .tag {
  font-size: 10.5px;
  font-weight: 650;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}

.call .tag:empty { display: none; }

/* Ball colours, as on the table (cue-game's ball_texture.rs). */
.c1 { --c: #f5c214; }
.c2 { --c: #143db8; }
.c3 { --c: #cc1a1a; }
.c4 { --c: #5c2485; }
.c5 { --c: #f57014; }
.c6 { --c: #0f753d; }
.c7 { --c: #7a1f14; }
.eight { --c: #0a0a0b; }

.mark {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--c);
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.45);
}

.mark.stripe,
.ball.stripe { background: linear-gradient(#f7f5e8 0 25%, var(--c) 25% 75%, #f7f5e8 75%); }

/* Eight-ball: panels over the dock ------------------------------------ */

.panels {
  position: absolute;
  left: max(8px, env(safe-area-inset-left));
  right: max(8px, env(safe-area-inset-right));
  bottom: max(8px, env(safe-area-inset-bottom));
  display: grid;
  justify-items: center;
  pointer-events: none;
}

.panel {
  width: min(100%, 520px);
  box-sizing: border-box;
  padding: 16px 18px;
  border-radius: 22px;
  background: linear-gradient(180deg, rgba(30, 31, 37, 0.95), rgba(16, 17, 22, 0.95));
  border: 1px solid var(--line);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), var(--hi);
  backdrop-filter: var(--blur);
  -webkit-backdrop-filter: var(--blur);
  pointer-events: auto;
  animation: rise 0.22s var(--ease);
}

.panel[hidden] { display: none; }
.panel h2 { margin: 0 0 4px; font: 600 20px/1.2 var(--serif); }
.panel p { margin: 0 0 10px; color: var(--muted); }
.panel .tally { color: var(--fg); font-weight: 600; font-variant-numeric: tabular-nums; }
.panel .tally[hidden] { display: none; }

.ui[data-showing] .dock { display: none; }

.options,
.menu {
  display: grid;
  gap: 6px;
}

.option {
  min-height: var(--tap);
  padding: 11px 16px;
  border-radius: 14px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.04);
  text-align: left;
  display: grid;
  gap: 3px;
  transition: border-color 0.16s var(--ease), background 0.16s var(--ease), transform 0.1s;
}

.option:hover { border-color: rgba(217, 181, 108, 0.6); background: rgba(255, 255, 255, 0.07); }
.option:active { transform: scale(0.99); }
.option span { font-size: 12.5px; color: var(--muted); }
.option strong { font-weight: 600; }

.option.primary {
  background: var(--gold);
  border-color: transparent;
  color: var(--accent-fg);
  font-weight: 700;
  box-shadow: var(--glow), inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

.option.primary span { color: rgba(28, 22, 10, 0.75); }

.menu .note { margin: 4px 0 0; }

/* Eight-ball: setup and call dialogs ---------------------------------- */

.card h2[tabindex]:focus { outline: none; }

.card.wide { width: min(100%, 540px); }

.names {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 6px;
}

.names label {
  display: grid;
  gap: 4px;
  font-size: 12.5px;
  color: var(--muted);
  min-width: 0;
}

input[type='text'] {
  box-sizing: border-box;
  width: 100%;
  min-height: var(--tap);
  padding: 0 12px;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.05);
  color: var(--fg);
  font-family: inherit;
  transition: border-color 0.16s var(--ease), background 0.16s var(--ease);
  /* 16px keeps iOS Safari from zooming in on focus. */
  font-size: 16px;
  line-height: 1.2;
  user-select: text;
  -webkit-user-select: text;
}

input[type='text']::placeholder { color: rgba(255, 255, 255, 0.35); }
input[type='text']:focus { border-color: rgba(217, 181, 108, 0.7); background: rgba(255, 255, 255, 0.07); outline: none; }

.balls {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.ballchip,
.pocket { position: relative; cursor: pointer; }

:is(.ballchip, .pocket) input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}

.ball {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: var(--c);
  box-shadow: inset -4px -5px 9px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(255, 255, 255, 0.25);
}

.ball b {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: #f7f5e8;
  color: #111;
  font-size: 11px;
  font-weight: 700;
}

.ballchip input:checked + .ball {
  box-shadow: 0 0 0 3px var(--accent), inset -4px -5px 9px rgba(0, 0, 0, 0.35);
}

:is(.ballchip, .pocket) input:focus-visible + :is(.ball, .hole) {
  outline: 2px solid var(--accent);
  outline-offset: 3px;
}

/* A small table: the pockets sit where they are on screen. */
.pockets {
  position: relative;
  width: min(calc(100% - 40px), 340px);
  aspect-ratio: 2 / 1;
  margin: 20px auto 16px;
  border-radius: 6px;
  background: #0f3d66;
  box-shadow: 0 0 0 9px #26272b;
}

.ui[data-theme='classic'] .pockets { background: #0f4c3d; }
.ui[data-theme='contrast'] .pockets { background: #4d5259; box-shadow: 0 0 0 9px #fff; }

.pockets[data-portrait] {
  width: auto;
  height: 220px;
  aspect-ratio: 1 / 2;
}

.pocket {
  position: absolute;
  width: 44px;
  height: 44px;
  display: grid;
  place-items: center;
  transform: translate(-50%, -50%);
}

.hole {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #050505;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.3);
}

.pocket input:checked + .hole {
  background: var(--accent);
  box-shadow: 0 0 0 3px var(--accent), 0 0 14px var(--accent);
}

[data-pocket='head-far'] { left: 0; top: 0; }
[data-pocket='side-far'] { left: 50%; top: 0; }
[data-pocket='foot-far'] { left: 100%; top: 0; }
[data-pocket='head-near'] { left: 0; top: 100%; }
[data-pocket='side-near'] { left: 50%; top: 100%; }
[data-pocket='foot-near'] { left: 100%; top: 100%; }

[data-portrait] [data-pocket='foot-far'] { left: 0; top: 0; }
[data-portrait] [data-pocket='foot-near'] { left: 100%; top: 0; }
[data-portrait] [data-pocket='side-far'] { left: 0; top: 50%; }
[data-portrait] [data-pocket='side-near'] { left: 100%; top: 50%; }
[data-portrait] [data-pocket='head-far'] { left: 0; top: 100%; }
[data-portrait] [data-pocket='head-near'] { left: 100%; top: 100%; }

.actions button:disabled { opacity: 0.45; cursor: default; }

/* States ----------------------------------------------------------------- */

.ui:not([data-state='ready']) :is(.top, .dock) { visibility: hidden; }

/* Narrow stages: a phone, or a small embed on a large screen. Sized by the
   stage, not the window, because CUE may sit in any box on the host page. */

@container (max-width: 560px) {
  .status { font-size: 12px; padding: 6px 10px; }
  .status .mode { display: none; }
  .player { font-size: 12px; gap: 6px; padding-left: 10px; }
  .player .word { display: none; }
  .player .name { max-width: 9ch; }
  .power span { display: none; }
  .power { padding: 0 10px; gap: 8px; }
  .shoot { padding: 0 14px; }
}

@container (max-width: 480px) {
  /* Container queries style descendants, not the container itself. */
  .top, .dock { --tap: 40px; }
  .toolbar {
    width: 100%;
    justify-content: space-between;
    gap: 2px;
  }
  .dock { gap: 6px; }
  .spin { --size: 46px; }
  .power { flex: 1; min-width: 0; }
  .power input[type='range'] { width: 100%; min-width: 48px; }
  .power output { min-width: 2.6em; font-size: 12px; }
  /* The turn line drops its box: the top bar stays short, so the camera
     can keep the table large. */
  .board .line {
    background: none;
    border: 0;
    box-shadow: none;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    padding: 0 4px;
    font-size: 12px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.9);
  }
  /* The call gets a row of its own above the cue controls. */
  .dock { flex-wrap: wrap; }
  .call { order: -1; flex: 1 1 auto; margin-right: 0; }
  .chip { order: -1; }
  .names { grid-template-columns: 1fr; }
}

/* Short stages (an embedded 16:9 box on a laptop, a phone in landscape). */

@container (max-height: 520px) {
  .card { --pad-y: 14px; --pad-x: 18px; font-size: 13px; }
  .panel { padding: 10px 14px; }
  .panel p { margin-bottom: 6px; }
  .options { grid-auto-flow: column; grid-auto-columns: 1fr; }
  /* Balls and pockets side by side, so the dialog fits without scrolling. */
  [data-form='call'] {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    column-gap: 20px;
    align-items: start;
  }
  [data-form='call'] :is(h2, .actions) { grid-column: 1 / -1; }
  .ball { width: 34px; height: 34px; }
  .pockets { width: 200px; margin: 14px 14px 10px; }
  .pockets[data-portrait] { width: auto; height: 160px; }
  .card h2 { font-size: 16px; margin-bottom: 8px; }
  .card h3 { margin-top: 10px; }
  .card li { margin-bottom: 4px; }
  .card dl { gap: 2px 12px; }
  .actions { margin-top: 12px; }
}

@media (prefers-reduced-motion: reduce) {
  * { transition: none !important; animation: none !important; }
}

/* The host asked for reduced motion (\`accessibility.reducedMotion\`). */
.ui[data-reduced-motion] * { transition: none !important; animation: none !important; }
`;
