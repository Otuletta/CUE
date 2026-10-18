// Styles for the overlay. They live inside its shadow root, so nothing here
// can leak into the host page and nothing from the host can break it.

export const STYLES = /* css */ `
:host { all: initial; }

.ui {
  --glass: rgba(12, 13, 17, 0.62);
  --glass-strong: rgba(12, 13, 17, 0.9);
  --line: rgba(255, 255, 255, 0.14);
  --fg: #f3f1ec;
  --muted: #b9b6ae;
  --accent: #e2bb74;
  --accent-fg: #17140f;
  --danger: #ff6b6b;
  --radius: 12px;
  --tap: 40px;
  position: absolute;
  inset: 0;
  container-type: size;
  pointer-events: none;
  color: var(--fg);
  font: 14px/1.45 ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
  -webkit-font-smoothing: antialiased;
  user-select: none;
  -webkit-user-select: none;
}

.ui[data-theme='contrast'] {
  --glass: rgba(0, 0, 0, 0.92);
  --glass-strong: #000;
  --line: #fff;
  --muted: #fff;
  --accent: #ffe100;
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

.status {
  margin: 0;
  padding: 7px 12px;
  border-radius: var(--radius);
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  font-size: 13px;
  display: flex;
  gap: 10px;
  align-items: baseline;
  max-width: 100%;
}

.status .mode { font-weight: 600; letter-spacing: 0.02em; }
.status .detail { color: var(--muted); }
.ui[data-in-hand] .status .detail { color: var(--accent); }

.toolbar {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

.icon {
  width: var(--tap);
  height: var(--tap);
  display: grid;
  place-items: center;
  padding: 0;
  border-radius: 10px;
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.icon:hover { background: var(--glass-strong); }
.icon:disabled { opacity: 0.35; cursor: default; }
.icon[aria-pressed='false'] { color: var(--muted); }

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

.spin {
  --size: 52px;
  position: relative;
  width: var(--size);
  height: var(--size);
  flex: none;
  border-radius: 50%;
  background: radial-gradient(circle at 38% 32%, #ffffff, #e9e6dc 55%, #b9b5a8);
  box-shadow: 0 0 0 1px var(--line), 0 2px 10px rgba(0, 0, 0, 0.4);
  touch-action: none;
  cursor: crosshair;
}

@media (pointer: coarse) {
  .spin { --size: 56px; }
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
  background: #d22b2b;
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.85);
  pointer-events: none;
}

.power {
  display: flex;
  align-items: center;
  gap: 10px;
  height: var(--tap);
  padding: 0 12px;
  border-radius: var(--radius);
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.power span { font-size: 13px; color: var(--muted); }

.power output {
  min-width: 3.2em;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

input[type='range'] {
  width: clamp(110px, 26vw, 260px);
  accent-color: var(--accent);
  touch-action: none;
}

.shoot {
  height: var(--tap);
  padding: 0 18px;
  border: 0;
  border-radius: var(--radius);
  background: var(--accent);
  color: var(--accent-fg);
  font-weight: 650;
  letter-spacing: 0.02em;
}

.shoot[aria-disabled='true'] { filter: saturate(0.4) brightness(0.8); }

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
  padding: 9px 16px;
  border-radius: 14px;
  text-wrap: balance;
  background: var(--glass-strong);
  border: 1px solid var(--line);
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
  background: rgba(0, 0, 0, 0.5);
  pointer-events: auto;
}

.sheet[hidden] { display: none; }
.sheet.bare { background: transparent; pointer-events: none; }

.card {
  width: min(100%, 440px);
  max-height: calc(100% - 16px);
  overflow: auto;
  padding: 20px 22px;
  border-radius: 16px;
  background: #15171c;
  border: 1px solid var(--line);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  user-select: text;
  -webkit-user-select: text;
}

.ui[data-theme='contrast'] .card { background: #000; }

.card h2 {
  margin: 0 0 12px;
  font-size: 18px;
  font-weight: 650;
}

.card h3 {
  margin: 16px 0 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
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
  font: 12px/1.6 ui-monospace, SFMono-Regular, Menlo, monospace;
  color: var(--accent);
}

.card dd { margin: 0; color: var(--muted); }

.card a { color: var(--accent); }

.card fieldset {
  margin: 0 0 14px;
  padding: 0;
  border: 0;
}

.card legend {
  margin-bottom: 6px;
  font-weight: 600;
}

.choices {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.choices label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 36px;
  padding: 0 12px;
  border-radius: 999px;
  border: 1px solid var(--line);
  cursor: pointer;
}

.choices input { accent-color: var(--accent); }
.choices label:has(input:checked) { border-color: var(--accent); }

.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 12px;
}

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
  padding: 0 16px;
  border-radius: 10px;
  border: 1px solid var(--line);
  background: transparent;
  color: var(--fg);
  font: inherit;
  display: inline-flex;
  align-items: center;
  text-decoration: none;
}

.actions .primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
  font-weight: 650;
}

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
  background: var(--accent);
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
  gap: 8px;
  min-height: 34px;
  padding: 3px 6px 3px 12px;
  border-radius: var(--radius);
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  font-size: 13px;
  min-width: 0;
}

.player[data-active] {
  border-color: var(--accent);
  box-shadow: inset 3px 0 0 var(--accent);
}

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
  background: rgba(255, 255, 255, 0.1);
  text-align: center;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.player .score[hidden] { display: none; }

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
  padding: 0 12px;
  border-radius: var(--radius);
  background: var(--glass);
  border: 1px solid var(--line);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  min-width: 0;
}

.call { border-color: var(--accent); margin-right: auto; max-width: 100%; }
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
  padding: 14px 16px;
  border-radius: 16px;
  background: var(--glass-strong);
  border: 1px solid var(--line);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  pointer-events: auto;
}

.panel[hidden] { display: none; }
.panel h2 { margin: 0 0 4px; font-size: 16px; font-weight: 650; }
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
  padding: 8px 14px;
  border-radius: 10px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.05);
  text-align: left;
  display: grid;
  gap: 2px;
}

.option:hover { border-color: var(--accent); }
.option span { font-size: 12.5px; color: var(--muted); }
.option strong { font-weight: 600; }

.option.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
  font-weight: 650;
}

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
  border-radius: 10px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.06);
  color: var(--fg);
  font-family: inherit;
  /* 16px keeps iOS Safari from zooming in on focus. */
  font-size: 16px;
  line-height: 1.2;
  user-select: text;
  -webkit-user-select: text;
}

input[type='text']::placeholder { color: rgba(255, 255, 255, 0.35); }

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
  .card { padding: 14px 18px; font-size: 13px; }
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
`;
