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
  left: 50%;
  top: calc(max(8px, env(safe-area-inset-top)) + 56px);
  transform: translate(-50%, -6px);
  max-width: min(90%, 460px);
  padding: 9px 16px;
  border-radius: 999px;
  background: var(--glass-strong);
  border: 1px solid var(--line);
  font-weight: 550;
  text-align: center;
  opacity: 0;
  transition: opacity 0.2s, transform 0.2s;
}

.toast[data-visible] {
  opacity: 1;
  transform: translate(-50%, 0);
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

.card ol {
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

/* States ----------------------------------------------------------------- */

.ui:not([data-state='ready']) :is(.top, .dock) { visibility: hidden; }

/* Narrow stages: a phone, or a small embed on a large screen. Sized by the
   stage, not the window, because CUE may sit in any box on the host page. */

@container (max-width: 560px) {
  .status { font-size: 12px; padding: 6px 10px; }
  .status .mode { display: none; }
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
}

/* Short stages (an embedded 16:9 box on a laptop, a phone in landscape). */

@container (max-height: 520px) {
  .card { padding: 14px 18px; font-size: 13px; }
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
