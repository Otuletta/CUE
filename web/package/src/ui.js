// The accessible control layer that sits over the canvas (spec §7, §10, §22).
//
// Everything a player needs is a real HTML control: buttons with names, a
// native range input for power, a keyboard-operable spin selector, dialogs
// that trap and return focus, and a live region that announces shot results.
// The canvas stays the place where you aim, but nothing *requires* it.
//
// The layer renders into its own shadow root, so the host page's CSS cannot
// break it and its CSS cannot leak out.

import { formatList } from './i18n.js';
import { ICONS } from './icons.js';
import { boardMarkup, createMatch, dockMarkup, sheetsMarkup } from './match.js';
import { STYLES } from './styles.js';

const TOAST_MS = 2600;
const MAX_TIP = 0.5;
const SPIN_STEP = 0.05;

const coarsePointer = () => window.matchMedia?.('(pointer: coarse)').matches ?? false;
const escape = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

/**
 * @param {HTMLElement} root  The CUE root element (holds the canvas).
 * @param {object} deps
 */
export function createOverlay(root, deps) {
  const { t, locale, api, settings, onExit, fallbackUrl, rulesUrl } = deps;
  const host = document.createElement('div');
  host.className = 'cue-overlay';
  Object.assign(host.style, { position: 'absolute', inset: '0', pointerEvents: 'none' });
  const shadow = host.attachShadow({ mode: 'open' });
  shadow.innerHTML = markup(t, { exit: Boolean(onExit), locale, rulesUrl });
  adoptStyles(shadow);
  root.appendChild(host);

  const $ = (selector) => shadow.querySelector(selector);
  const ui = $('.ui');
  const live = $('.sr');
  const toastEl = $('.toast');
  const power = $('input[name="power"]');
  const powerOut = $('.power output');
  const spin = $('.spin');
  const dot = $('.spin .dot');
  const shootBtn = $('[data-action="shoot"]');
  const button = (action) => $(`[data-action="${action}"]`);

  let snapshot = null;
  // The overlay's own copy of the tip, so quick successive key presses build
  // on each other instead of on a snapshot that is one frame old.
  let tip = { x: 0, y: 0 };
  let draggingPower = false;
  // The power last set with the slider. The engine applies it on its next
  // frame, so until the snapshot catches up this is the power that counts.
  let askedPower = null;
  let toastTimer = 0;
  let raf = 0;
  let openSheet = null;
  let returnFocus = null;
  let fullscreen = false;

  // --- Announcements ------------------------------------------------------

  function announce(message, { visual = true } = {}) {
    // Clearing first makes repeated identical messages announce again.
    live.textContent = '';
    requestAnimationFrame(() => (live.textContent = message));
    if (!visual) return;
    // Just below the top bar, however many rows it wraps to.
    const top = $('.top').getBoundingClientRect().bottom - ui.getBoundingClientRect().top;
    if (top > 0) toastEl.style.top = `${Math.round(top + 8)}px`;
    toastEl.textContent = message;
    toastEl.setAttribute('data-visible', '');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toastEl.removeAttribute('data-visible'), TOAST_MS);
  }

  // --- Sheets and focus ---------------------------------------------------

  const background = () => [$('.top'), $('.dock'), $('.panels')];
  const canvas = () => root.querySelector('canvas');

  function open(name) {
    if (openSheet === name) return;
    if (openSheet) close({ restore: false });
    else returnFocus = shadow.activeElement ?? document.activeElement;
    const sheet = $(`[data-sheet="${name}"]`);
    sheet.hidden = false;
    openSheet = name;
    background().forEach((el) => (el.inert = true));
    if (name === 'settings') syncSettings();
    sheet.querySelector('.card')?.scrollTo?.(0, 0);
    (sheet.querySelector('[autofocus]') ?? sheet.querySelector('button, input, a'))?.focus({ preventScroll: true });
  }

  function close({ restore = true } = {}) {
    if (!openSheet) return;
    const name = openSheet;
    $(`[data-sheet="${name}"]`).hidden = true;
    openSheet = null;
    background().forEach((el) => (el.inert = false));
    if (name === 'help') deps.onTutorialDone?.();
    if (name === 'paused') api.resume('user');
    if (!restore) return;
    // The control that opened the sheet may have gone (a panel that closed).
    const back = returnFocus?.isConnected && returnFocus.checkVisibility?.() !== false ? returnFocus : canvas();
    back?.focus?.({ preventScroll: true });
  }

  // What Tab can reach in `card`. A radio group is a single stop, at its
  // checked option (the first when none is), as browsers make it.
  const tabStops = (card) =>
    [...card.querySelectorAll('button, input, a[href], [tabindex="0"]')].filter((el) => {
      if (el.disabled || el.checkVisibility?.() === false) return false;
      if (el.type !== 'radio') return true;
      const group = [...card.querySelectorAll(`input[type="radio"][name="${el.name}"]`)];
      return el === (group.find((r) => r.checked) ?? group[0]);
    });

  // Keep Tab inside an open dialog: wrap at either end, and from anything
  // Tab cannot reach (a heading focused on opening) go to the first or last
  // stop. Without this, Shift+Tab would land on the canvas behind.
  shadow.addEventListener('keydown', (e) => {
    if (!openSheet || e.key !== 'Tab') return;
    const stops = tabStops($(`[data-sheet="${openSheet}"] .card`));
    if (!stops.length) return;
    const at = stops.indexOf(shadow.activeElement);
    const last = stops.length - 1;
    const next = e.shiftKey ? (at <= 0 ? last : null) : at === -1 || at === last ? 0 : null;
    if (next === null) return;
    stops[next].focus();
    e.preventDefault();
  });

  // --- Actions ------------------------------------------------------------

  function togglePause() {
    if (openSheet === 'paused') close();
    else if (!openSheet) {
      api.pause('user');
      open('paused');
    }
  }

  function toggleSound() {
    const s = settings.get();
    settings.set({ ...s, sound: { ...s.sound, enabled: !s.sound.enabled } });
    syncSound();
  }

  function shoot() {
    if (!snapshot?.atRest) return;
    if ((askedPower ?? snapshot.power ?? 0) < (snapshot.minPower ?? 0.03)) {
      announce(t.shootHint);
      return;
    }
    api.shoot();
  }

  const match = createMatch(shadow, {
    t,
    locale,
    api,
    root,
    open,
    close,
    announce,
    openSheet: () => openSheet,
    focusTable: () => canvas()?.focus({ preventScroll: true }),
  });

  // --- Framing ------------------------------------------------------------
  // The engine keeps the cloth clear of the top bar and the dock; only the
  // rails run underneath. Re-measured whenever either changes size.
  let insets = null;
  let measuring = 0;
  function measureInsets() {
    measuring = 0;
    const box = ui.getBoundingClientRect();
    if (!box.height || ui.dataset.state !== 'ready') return;
    const top = $('.top').getBoundingClientRect().bottom - box.top;
    const dock = $('.dock').getBoundingClientRect();
    // A panel hides the dock: keep its last size so the table stays put.
    const bottom = dock.height ? box.bottom - dock.top : (insets?.bottom ?? 0);
    const next = { top: Math.max(0, Math.round(top)), bottom: Math.max(0, Math.round(bottom)) };
    if (insets && next.top === insets.top && next.bottom === insets.bottom) return;
    insets = next;
    api.setViewInsets(next.top, next.bottom);
  }
  const scheduleInsets = () => {
    if (!measuring) measuring = requestAnimationFrame(measureInsets);
  };
  const insetObserver = new ResizeObserver(scheduleInsets);
  [ui, $('.top'), $('.dock')].forEach((el) => insetObserver.observe(el));

  const actions = {
    ...match.actions,
    help: () => open('help'),
    undo: () => api.undo(),
    sound: toggleSound,
    settings: () => open('settings'),
    fullscreen: () => (fullscreen ? api.exitFullscreen() : api.enterFullscreen()),
    pause: togglePause,
    resume: () => close(),
    exit: () => onExit?.(),
    reload: () => window.location.reload(),
    shoot,
    close: () => close(),
    resetSettings: () => {
      settings.reset();
      syncSettings();
      syncSound();
    },
  };

  shadow.addEventListener('click', (e) => {
    const el = e.target.closest?.('[data-action]');
    const action = el?.dataset.action;
    if (action && actions[action] && !el.disabled) {
      e.preventDefault();
      actions[action](el);
    }
  });

  // Shortcuts that do not collide with the in-canvas keys (arrows, Space,
  // WASD, IJKL, C, U, R, Esc).
  function onKey(e) {
    if (e.defaultPrevented && e.target !== root.querySelector('canvas')) return;
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === 'Escape' && openSheet && openSheet !== 'loading') {
      close();
      e.preventDefault();
      return;
    }
    const target = e.composedPath?.()[0];
    // Typing a name, or moving a slider, is not a shortcut.
    if (target?.matches?.('input[type="range"], input[type="text"]')) return;
    const key = e.key.toLowerCase();
    const map = { h: 'help', '?': 'help', p: 'pause', m: 'sound', f: 'fullscreen', n: 'game', o: 'settings' };
    if (map[key] && ui.dataset.state === 'ready' && (!openSheet || map[key] === 'pause')) {
      actions[map[key]]();
      e.preventDefault();
    }
  }
  root.addEventListener('keydown', onKey);

  // --- Power and spin -------------------------------------------------------

  power.addEventListener('pointerdown', () => (draggingPower = true));
  power.addEventListener('pointerup', () => (draggingPower = false));
  power.addEventListener('blur', () => (draggingPower = false));
  power.addEventListener('input', () => {
    powerOut.value = `${power.value}%`;
    askedPower = Number(power.value) / 100;
    api.setPower(askedPower);
  });

  function setSpin(x, y) {
    const length = Math.hypot(x, y);
    if (length > MAX_TIP) {
      x *= MAX_TIP / length;
      y *= MAX_TIP / length;
    }
    tip = { x, y };
    api.setTip(x, y);
    drawSpin(x, y);
  }

  function drawSpin(x, y) {
    // The dashed ring sits at 80 % of the face: the miscue limit.
    dot.style.left = `${50 + (x / MAX_TIP) * 40}%`;
    dot.style.top = `${50 - (y / MAX_TIP) * 40}%`;
    spin.setAttribute('aria-valuetext', t.spinValue(x, y));
  }

  function spinFromPointer(e) {
    const r = spin.getBoundingClientRect();
    const nx = ((e.clientX - r.left) / r.width - 0.5) / 0.4;
    const ny = ((e.clientY - r.top) / r.height - 0.5) / 0.4;
    setSpin(nx * MAX_TIP, -ny * MAX_TIP);
  }

  spin.addEventListener('pointerdown', (e) => {
    spin.setPointerCapture(e.pointerId);
    spinFromPointer(e);
  });
  spin.addEventListener('pointermove', (e) => {
    if (spin.hasPointerCapture(e.pointerId)) spinFromPointer(e);
  });
  spin.addEventListener('dblclick', () => setSpin(0, 0));
  spin.addEventListener('keydown', (e) => {
    const { x, y } = tip;
    const moves = {
      ArrowUp: [x, y + SPIN_STEP],
      ArrowDown: [x, y - SPIN_STEP],
      ArrowLeft: [x - SPIN_STEP, y],
      ArrowRight: [x + SPIN_STEP, y],
      Home: [0, 0],
      '0': [0, 0],
    };
    if (moves[e.key]) {
      setSpin(...moves[e.key]);
      e.preventDefault();
      e.stopPropagation();
    }
  });

  // --- Settings dialog ----------------------------------------------------

  function syncSettings() {
    const s = settings.get();
    for (const input of shadow.querySelectorAll('input[name="theme"]')) input.checked = input.value === s.theme;
    for (const input of shadow.querySelectorAll('input[name="assists"]')) input.checked = input.value === s.assists;
    for (const input of shadow.querySelectorAll('input[name="quality"]')) input.checked = input.value === s.quality;
    $('input[name="sound"]').checked = s.sound.enabled;
    $('input[name="volume"]').value = String(Math.round(s.sound.volume * 100));
    ui.dataset.theme = s.theme;
  }

  function syncSound() {
    const on = settings.get().sound.enabled;
    const btn = button('sound');
    btn.innerHTML = on ? ICONS.soundOn : ICONS.soundOff;
    btn.setAttribute('aria-pressed', String(on));
  }

  shadow.addEventListener('change', (e) => {
    const input = e.target;
    const s = settings.get();
    if (input.name === 'theme') settings.set({ ...s, theme: input.value });
    if (input.name === 'assists') settings.set({ ...s, assists: input.value });
    if (input.name === 'quality') settings.set({ ...s, quality: input.value });
    if (input.name === 'sound') settings.set({ ...s, sound: { ...s.sound, enabled: input.checked } });
    if (input.name === 'volume') settings.set({ ...s, sound: { ...s.sound, volume: Number(input.value) / 100 } });
    ui.dataset.theme = settings.get().theme;
    syncSound();
  });

  // --- Engine state ---------------------------------------------------------

  function update(next) {
    if (!next) return;
    const prev = snapshot;
    snapshot = next;
    const changed = (key) => !prev || prev[key] !== next[key];

    if (changed('atRest')) {
      ui.toggleAttribute('data-moving', !next.atRest);
      power.disabled = !next.atRest;
      shootBtn.setAttribute('aria-disabled', String(!next.atRest));
    }
    if (changed('inHand')) ui.toggleAttribute('data-in-hand', next.inHand);
    if (changed('canUndo')) button('undo').disabled = !next.canUndo;
    if (changed('power')) askedPower = null;
    if (changed('power') && !draggingPower) {
      const percent = Math.round(next.power * 100);
      power.value = String(percent);
      powerOut.value = `${percent}%`;
    }
    if (changed('tipX') || changed('tipY')) {
      // Changed by the engine (WASD keys) or echoing our own change.
      tip = { x: next.tipX, y: next.tipY };
      drawSpin(tip.x, tip.y);
    }

    if (!next.eightBall) {
      const left = next.ballsInPlay.filter((n) => n !== 0).length;
      const detail = next.inHand ? t.inHand : next.atRest ? t.ballsLeft(left) : t.rolling;
      const detailEl = $('.status .detail');
      if (detailEl.textContent !== detail) detailEl.textContent = detail;
    }
    match.update(next);
  }

  function loop() {
    update(api.state());
    raf = requestAnimationFrame(loop);
  }

  // --- Public surface -------------------------------------------------------

  syncSettings();
  syncSound();

  return {
    setState(state) {
      ui.dataset.state = state;
      $('[data-sheet="loading"]').hidden = state !== 'loading';
      if (state === 'ready') {
        insets = null;
        scheduleInsets();
        if (!raf) loop();
      }
    },
    setProgress(fraction) {
      const bar = $('.progress');
      const label = $('[data-sheet="loading"] p');
      if (fraction == null) {
        bar.setAttribute('data-indeterminate', '');
        label.textContent = t.loading;
      } else {
        const percent = Math.round(Math.min(1, fraction) * 100);
        bar.removeAttribute('data-indeterminate');
        bar.style.setProperty('--p', `${percent}%`);
        bar.setAttribute('aria-valuenow', String(percent));
        label.textContent = t.loadingProgress(percent);
      }
    },
    showMessage(kind) {
      // A failure can arrive while a dialog is open: the message replaces it.
      if (openSheet) {
        $(`[data-sheet="${openSheet}"]`).hidden = true;
        openSheet = null;
      }
      const card = $('[data-sheet="message"] .card');
      const unsupported = kind === 'unsupported';
      const text = {
        unsupported: [t.unsupportedTitle, t.unsupportedBody],
        reload: [t.reloadTitle, t.reloadBody],
        stopped: [t.stoppedTitle, t.stoppedBody],
      }[kind] ?? [t.errorTitle, t.errorBody];
      card.querySelector('h2').textContent = text[0];
      card.querySelector('p').textContent = text[1];
      const link = card.querySelector('a');
      link.hidden = !fallbackUrl;
      if (fallbackUrl) link.href = fallbackUrl;
      card.querySelector('[data-action="reload"]').hidden = unsupported;
      // Keep the keyboard with CUE if it was there, on the way out.
      const focused = shadow.activeElement !== null || document.activeElement === canvas();
      this.setState(kind);
      $('[data-sheet="message"]').hidden = false;
      if (focused) card.querySelector('button:not([hidden]), a:not([hidden])')?.focus({ preventScroll: true });
    },
    showTutorial() {
      open('help');
    },
    /** Measures the bars again and tells the engine (after a move or restyle). */
    remeasure() {
      insets = null;
      scheduleInsets();
    },
    setFullscreen(on) {
      fullscreen = on;
      const btn = button('fullscreen');
      btn.innerHTML = on ? ICONS.fullscreenExit : ICONS.fullscreen;
      btn.setAttribute('aria-label', on ? t.exitFullscreen : t.fullscreen);
      btn.title = btn.getAttribute('aria-label');
    },
    /** Reacts to engine events with feedback a player can see and hear. */
    onEvent(name, detail) {
      if (match.onEvent(name, detail)) return;
      if (name === 'shot-end' && detail && !detail.abandoned) {
        const balls = detail.pocketed.filter((p) => p.ballId !== 0).map((p) => t.ball(p.ballId));
        if (detail.cueBallPocketed) announce(t.scratch);
        else if (balls.length) announce(t.pocketed(formatList(balls, locale)));
        else announce(t.noPocket, { visual: false });
      } else if (name === 'racked') {
        announce(detail?.cleared ? t.cleared : t.racked);
      } else if (name === 'undo') {
        announce(t.undone);
      } else if (name === 'paused' && openSheet !== 'paused') {
        cancelAnimationFrame(raf);
        raf = 0;
      } else if (name === 'resumed') {
        if (openSheet === 'paused') {
          $('[data-sheet="paused"]').hidden = true;
          openSheet = null;
          background().forEach((el) => (el.inert = false));
        }
        if (!raf && ui.dataset.state === 'ready') loop();
      }
    },
    destroy() {
      cancelAnimationFrame(raf);
      clearTimeout(toastTimer);
      root.removeEventListener('keydown', onKey);
      insetObserver.disconnect();
      cancelAnimationFrame(measuring);
      match.destroy();
      host.remove();
    },
  };
}

/**
 * Styles the shadow root with a constructed stylesheet where the browser has
 * them: unlike a `<style>` element, it needs no `'unsafe-inline'` in a host's
 * Content Security Policy. Older browsers get the `<style>` element.
 */
function adoptStyles(shadow) {
  if ('adoptedStyleSheets' in ShadowRoot.prototype && 'replaceSync' in CSSStyleSheet.prototype) {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(STYLES);
    shadow.adoptedStyleSheets = [sheet];
  } else {
    const style = document.createElement('style');
    style.textContent = STYLES;
    shadow.prepend(style);
  }
}

function markup(t, { exit, locale, rulesUrl }) {
  const icon = (action, label, svg, extra = '') =>
    `<button type="button" class="icon" data-action="${action}" aria-label="${escape(label)}" title="${escape(label)}" ${extra}>${svg}</button>`;
  const steps = (coarsePointer() ? t.howTouch : t.howPointer).map((s) => `<li>${escape(s)}</li>`).join('');
  const shortcuts = coarsePointer()
    ? ''
    : `<h3>${escape(t.shortcutsTitle)}</h3><dl>${t.shortcuts
        .map(([k, v]) => `<dt>${escape(k)}</dt><dd>${escape(v)}</dd>`)
        .join('')}</dl>`;
  const eightBall = t.eightBallHow.map((s) => `<li>${escape(s)}</li>`).join('');
  const radios = (name, values) =>
    Object.entries(values)
      .map(([value, label]) => `<label><input type="radio" name="${name}" value="${value}">${escape(label)}</label>`)
      .join('');

  return `
<div class="ui" data-state="loading" lang="${locale}">
  <div class="top">
    <p class="status"><span class="mode">${escape(t.freePlay)}</span><span class="detail"></span></p>
    ${boardMarkup(t)}
    <div class="toolbar" role="toolbar" aria-label="${escape(t.toolbar)}">
      ${icon('help', t.help, ICONS.help)}
      ${icon('undo', t.undo, ICONS.undo, 'disabled')}
      ${icon('game', t.newGame, ICONS.rack, 'aria-haspopup="dialog"')}
      ${icon('sound', t.sound, ICONS.soundOn, 'aria-pressed="true"')}
      ${icon('settings', t.settings, ICONS.settings)}
      ${icon('fullscreen', t.fullscreen, ICONS.fullscreen)}
      ${icon('pause', t.pause, ICONS.pause)}
      ${exit ? icon('exit', t.exit, ICONS.exit) : ''}
    </div>
  </div>

  <div class="toast" aria-hidden="true"></div>
  <div class="sr" role="status" aria-live="polite"></div>

  <div class="dock">
    ${dockMarkup(t)}
    <div class="spin" role="slider" tabindex="0" aria-label="${escape(t.spin)}" aria-valuetext="${escape(t.spinValue(0, 0))}" title="${escape(t.spin)}"><span class="dot"></span></div>
    <label class="power"><span>${escape(t.power)}</span><input type="range" name="power" min="0" max="100" step="1" value="0"><output>0%</output></label>
    <button type="button" class="shoot" data-action="shoot">${escape(t.shoot)}</button>
  </div>

  <div class="sheet bare" data-sheet="loading">
    <div class="card">
      <p>${escape(t.loading)}</p>
      <div class="progress" role="progressbar" aria-label="${escape(t.loading)}" data-indeterminate aria-valuemin="0" aria-valuemax="100"><span></span></div>
    </div>
  </div>

  <div class="sheet" data-sheet="message" hidden>
    <div class="card" role="alert">
      <h2></h2>
      <p></p>
      <div class="actions">
        <a hidden target="_blank" rel="noopener">${escape(t.learnMore)}</a>
        <button type="button" class="primary" data-action="reload">${escape(t.tryAgain)}</button>
      </div>
    </div>
  </div>

  <div class="sheet" data-sheet="help" role="dialog" aria-modal="true" aria-labelledby="help-title" hidden>
    <div class="card">
      <h2 id="help-title">${escape(t.howTitle)}</h2>
      <ol>${steps}</ol>
      <h3>${escape(t.eightBallTitle)}</h3>
      <ul>${eightBall}</ul>
      ${shortcuts}
      <div class="actions"><button type="button" class="primary" data-action="close" autofocus>${escape(t.done)}</button></div>
    </div>
  </div>

  <div class="sheet" data-sheet="settings" role="dialog" aria-modal="true" aria-labelledby="settings-title" hidden>
    <div class="card">
      <h2 id="settings-title">${escape(t.settingsTitle)}</h2>
      <fieldset><legend>${escape(t.theme)}</legend><div class="choices">${radios('theme', t.themes)}</div></fieldset>
      <fieldset><legend>${escape(t.assists)}</legend><div class="choices">${radios('assists', t.assistsLevels)}</div>
        <p class="note">${escape(t.assistsNote)}</p></fieldset>
      <fieldset><legend>${escape(t.quality)}</legend><div class="choices">${radios('quality', t.qualityLevels)}</div>
        <p class="note" data-note="quality">${escape(t.qualityNote)}</p></fieldset>
      <div class="row"><label><input type="checkbox" name="sound"> ${escape(t.soundToggle)}</label></div>
      <div class="row"><label>${escape(t.volume)} <input type="range" name="volume" min="0" max="100" step="5"></label></div>
      <div class="actions">
        <button type="button" data-action="resetSettings">${escape(t.resetSettings)}</button>
        <button type="button" class="primary" data-action="close" autofocus>${escape(t.close)}</button>
      </div>
    </div>
  </div>

  <div class="sheet" data-sheet="paused" role="dialog" aria-modal="true" aria-labelledby="paused-title" hidden>
    <div class="card">
      <h2 id="paused-title">${escape(t.pausedTitle)}</h2>
      <div class="actions"><button type="button" class="primary" data-action="resume" autofocus>${escape(t.resume)}</button></div>
    </div>
  </div>
  ${sheetsMarkup(t, { rulesUrl })}
</div>`;
}
