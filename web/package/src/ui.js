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
  const { t, locale, api, settings, onExit, fallbackUrl } = deps;
  const host = document.createElement('div');
  host.className = 'cue-overlay';
  Object.assign(host.style, { position: 'absolute', inset: '0', pointerEvents: 'none' });
  const shadow = host.attachShadow({ mode: 'open' });
  shadow.innerHTML = `<style>${STYLES}</style>${markup(t, { exit: Boolean(onExit), locale })}`;
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
    toastEl.textContent = message;
    toastEl.setAttribute('data-visible', '');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toastEl.removeAttribute('data-visible'), TOAST_MS);
  }

  // --- Sheets and focus ---------------------------------------------------

  const background = () => [$('.top'), $('.dock')];

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
    if (name === 'paused') api.resume();
    if (restore) (returnFocus?.isConnected ? returnFocus : root.querySelector('canvas'))?.focus?.();
  }

  // Keep Tab inside an open dialog.
  shadow.addEventListener('keydown', (e) => {
    if (!openSheet || e.key !== 'Tab') return;
    const card = $(`[data-sheet="${openSheet}"] .card`);
    const items = [...card.querySelectorAll('button, input, a[href]')].filter((el) => !el.disabled);
    if (!items.length) return;
    const [first, last] = [items[0], items[items.length - 1]];
    const active = shadow.activeElement;
    if (e.shiftKey && active === first) {
      last.focus();
      e.preventDefault();
    } else if (!e.shiftKey && active === last) {
      first.focus();
      e.preventDefault();
    }
  });

  // --- Actions ------------------------------------------------------------

  function togglePause() {
    if (openSheet === 'paused') close();
    else if (!openSheet) {
      api.pause();
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
    if ((snapshot.power ?? 0) < (snapshot.minPower ?? 0.03)) {
      announce(t.shootHint);
      return;
    }
    api.shoot();
  }

  const actions = {
    help: () => open('help'),
    undo: () => api.undo(),
    rack: () => open('confirm'),
    confirmRack: () => {
      close();
      api.rerack();
    },
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
    const action = e.target.closest?.('[data-action]')?.dataset.action;
    if (action && actions[action]) {
      e.preventDefault();
      actions[action]();
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
    if (target?.matches?.('input[type="range"]')) return;
    const key = e.key.toLowerCase();
    const map = { h: 'help', '?': 'help', p: 'pause', m: 'sound', f: 'fullscreen', n: 'rack', o: 'settings' };
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
    api.setPower(Number(power.value) / 100);
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

    const left = next.ballsInPlay.filter((n) => n !== 0).length;
    const detail = next.inHand ? t.inHand : next.atRest ? t.ballsLeft(left) : t.rolling;
    const detailEl = $('.status .detail');
    if (detailEl.textContent !== detail) detailEl.textContent = detail;
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
      if (state === 'ready' && !raf) loop();
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
      const card = $('[data-sheet="message"] .card');
      const unsupported = kind === 'unsupported';
      card.querySelector('h2').textContent = unsupported ? t.unsupportedTitle : t.errorTitle;
      card.querySelector('p').textContent = unsupported ? t.unsupportedBody : t.errorBody;
      const link = card.querySelector('a');
      link.hidden = !fallbackUrl;
      if (fallbackUrl) link.href = fallbackUrl;
      card.querySelector('[data-action="reload"]').hidden = unsupported;
      this.setState(kind);
      $('[data-sheet="message"]').hidden = false;
    },
    showTutorial() {
      open('help');
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
      host.remove();
    },
  };
}

function markup(t, { exit, locale }) {
  const icon = (action, label, svg, extra = '') =>
    `<button type="button" class="icon" data-action="${action}" aria-label="${escape(label)}" title="${escape(label)}" ${extra}>${svg}</button>`;
  const steps = (coarsePointer() ? t.howTouch : t.howPointer).map((s) => `<li>${escape(s)}</li>`).join('');
  const shortcuts = coarsePointer()
    ? ''
    : `<h3>${escape(t.shortcutsTitle)}</h3><dl>${t.shortcuts
        .map(([k, v]) => `<dt>${escape(k)}</dt><dd>${escape(v)}</dd>`)
        .join('')}</dl>`;
  const radios = (name, values) =>
    Object.entries(values)
      .map(([value, label]) => `<label><input type="radio" name="${name}" value="${value}">${escape(label)}</label>`)
      .join('');

  return `
<div class="ui" data-state="loading" lang="${locale}">
  <div class="top">
    <p class="status"><span class="mode">${escape(t.freePlay)}</span><span class="detail"></span></p>
    <div class="toolbar" role="toolbar" aria-label="${escape(t.toolbar)}">
      ${icon('help', t.help, ICONS.help)}
      ${icon('undo', t.undo, ICONS.undo, 'disabled')}
      ${icon('rack', t.newRack, ICONS.rack)}
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
      <div class="row"><label><input type="checkbox" name="sound"> ${escape(t.soundToggle)}</label></div>
      <div class="row"><label>${escape(t.volume)} <input type="range" name="volume" min="0" max="100" step="5"></label></div>
      <div class="actions">
        <button type="button" data-action="resetSettings">${escape(t.resetSettings)}</button>
        <button type="button" class="primary" data-action="close" autofocus>${escape(t.close)}</button>
      </div>
    </div>
  </div>

  <div class="sheet" data-sheet="confirm" role="dialog" aria-modal="true" aria-labelledby="confirm-title" hidden>
    <div class="card">
      <h2 id="confirm-title">${escape(t.confirmRackTitle)}</h2>
      <p>${escape(t.confirmRackBody)}</p>
      <div class="actions">
        <button type="button" data-action="close" autofocus>${escape(t.cancel)}</button>
        <button type="button" class="primary" data-action="confirmRack">${escape(t.confirmRack)}</button>
      </div>
    </div>
  </div>

  <div class="sheet" data-sheet="paused" role="dialog" aria-modal="true" aria-labelledby="paused-title" hidden>
    <div class="card">
      <h2 id="paused-title">${escape(t.pausedTitle)}</h2>
      <div class="actions"><button type="button" class="primary" data-action="resume" autofocus>${escape(t.resume)}</button></div>
    </div>
  </div>
</div>`;
}
