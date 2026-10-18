// CUE web wrapper: the only API host pages use. Reference: docs/api.md.
//
//   import { createCue } from '@tuletta/cue';
//   const game = await createCue({ container: document.querySelector('#cue') });
//   const off = game.on('shot-end', (shot) => { ... });
//   game.pause(); game.resume(); game.undo(); game.rerack();
//   game.newGame({ ruleset: 'eight-ball-wpa', raceTo: 3, names: ['Ana', 'Bo'] });
//   await game.destroy();
//
// The wrapper owns one root element inside the host's container (a <canvas>
// plus an optional accessible control layer in a shadow root) and nothing
// else: no global styles, no listeners outside that root, no network requests
// beyond its own module, binary and manifest.
//
// Its own files are referenced with static `import()` and
// `new URL(…, import.meta.url)` expressions, so bundlers such as Vite find,
// copy and fingerprint them without configuration.

import { resolveLocale, strings } from './i18n.js';
import {
  DEFAULTS,
  clearSettings,
  loadSettings,
  markTutorialSeen,
  sanitize,
  saveSettings,
  tutorialSeen,
} from './settings.js';
import { createOverlay } from './ui.js';

const EVENTS = new Set([
  'ready',
  'shot-start',
  'ball-pocketed',
  'shot-end',
  'racked',
  'undo',
  'paused',
  'resumed',
  'fullscreen-change',
  'settings-change',
  'destroyed',
  'error',
  // Eight-ball (docs/rules-eight-ball.md).
  'new-game',
  'ruling',
  'foul',
  'groups-assigned',
  'turn-changed',
  'decision-required',
  'decided',
  'rack-over',
  'match-over',
  'ball-spotted',
  'shot-refused',
  'call-refused',
]);

const RULESETS = new Set(['eight-ball-wpa', 'eight-ball-casual']);
const MODES = new Set(['free-play', 'eight-ball-local']);

let live = null;

/**
 * Mounts CUE inside `container`.
 *
 * @param {object} options
 * @param {HTMLElement} options.container  Element the game fills. It must have a size.
 * @param {boolean} [options.ui=true]  Render the built-in accessible controls.
 * @param {'en'|'es'} [options.locale]  Interface language; defaults to the browser's.
 * @param {'classic'|'studio'|'contrast'} [options.theme]  Overrides the saved theme.
 * @param {'off'|'basic'|'assisted'} [options.assists]  Overrides the saved guide level.
 * @param {'auto'|'high'|'medium'|'low'} [options.quality]  Overrides the saved graphics quality.
 * @param {{enabled?: boolean, volume?: number}} [options.audio]  Overrides saved sound settings.
 * @param {boolean} [options.persist=true]  Remember preferences in localStorage.
 * @param {() => void} [options.onExit]  Shows an Exit button that calls this.
 * @param {string} [options.fallbackUrl]  Link offered when the browser is unsupported.
 * @param {string} [options.rulesUrl]  Link to the eight-ball rules notes, shown in the match setup.
 * @param {'free-play'|'eight-ball-local'} [options.mode='free-play']  The game to start with.
 * @param {object} [options.match]  Setup for `mode: 'eight-ball-local'`, as for `newGame`.
 * @param {{reducedMotion?: boolean, highContrast?: boolean}} [options.accessibility]
 *   `reducedMotion` overrides the device setting; `highContrast` picks the contrast theme.
 * @param {string|URL} [options.assetBase]  Where `manifest.json` and `pkg/` are served, when
 *   not next to this module (a CDN, or a copy in the host's public folder).
 * @returns {Promise<object>} Resolves once the first frame has run.
 */
export async function createCue(options = {}) {
  const { container, ui = true, persist = true, onExit, fallbackUrl, rulesUrl } = options;
  if (!(container instanceof HTMLElement)) {
    throw new TypeError('createCue: `container` must be an HTMLElement');
  }
  if (live) throw new Error('INSTANCE_ALREADY_MOUNTED');
  live = {};

  const locale = resolveLocale(options.locale);
  const t = strings(locale);
  const access = options.accessibility ?? {};
  let settings = sanitize({
    ...loadSettings(persist),
    ...(access.highContrast && { theme: 'contrast' }),
    ...(options.theme && { theme: options.theme }),
    ...(options.assists && { assists: options.assists }),
    ...(options.quality && { quality: options.quality }),
  });
  if (options.mode && !MODES.has(options.mode)) {
    throw new TypeError(`createCue: unknown mode "${options.mode}"`);
  }
  if (options.audio) {
    settings = sanitize({ ...settings, sound: { ...settings.sound, ...options.audio } });
  }

  const listeners = new Map([...EVENTS].map((name) => [name, new Set()]));
  let overlay = null;
  const emit = (name, detail) => {
    overlay?.onEvent(name, detail);
    for (const fn of listeners.get(name) ?? []) {
      try {
        fn(detail);
      } catch (err) {
        // A failing host handler must not break the game or other handlers.
        queueMicrotask(() => {
          throw err;
        });
      }
    }
  };

  // Root: canvas plus overlay, filling the container.
  const root = document.createElement('div');
  root.className = 'cue-root';
  Object.assign(root.style, {
    position: 'relative',
    width: '100%',
    height: '100%',
    overflow: 'hidden',
  });
  const canvas = document.createElement('canvas');
  canvas.id = `cue-canvas-${Math.random().toString(36).slice(2, 10)}`;
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-label', locale === 'es' ? 'Mesa de billar de CUE' : 'CUE pool table');
  Object.assign(canvas.style, {
    display: 'block',
    width: '100%',
    height: '100%',
    touchAction: 'none',
    outline: 'none',
  });
  root.appendChild(canvas);
  container.appendChild(root);

  let wasm = null;
  let destroyed = false;
  let teardown = null;
  const fs = fullscreenController(root, (on) => {
    overlay?.setFullscreen(on);
    emit('fullscreen-change', { fullscreen: on });
  });

  // The visitor's reduced-motion preference turns off the cue's swing,
  // unless the host decides.
  const motionQuery = window.matchMedia?.('(prefers-reduced-motion: reduce)');
  const applyMotion = () => {
    const reduced = access.reducedMotion ?? Boolean(motionQuery?.matches);
    if (wasm && !destroyed) wasm.setReducedMotion(reduced);
  };
  motionQuery?.addEventListener?.('change', applyMotion);

  const applySettings = () => {
    if (!wasm) return;
    wasm.setTheme(settings.theme);
    wasm.setAssists(settings.assists);
    wasm.setQuality(settings.quality);
    wasm.setAudio(settings.sound.enabled, settings.sound.volume);
    applyMotion();
  };

  const guard =
    (fn) =>
    (...args) => {
      if (destroyed) throw new Error('INSTANCE_DESTROYED');
      return fn(...args);
    };

  const api = {
    pause: guard(() => wasm?.pause()),
    resume: guard(() => wasm?.resume()),
    rerack: guard(() => wasm?.rerack()),
    undo: guard(() => wasm?.undo()),
    shoot: guard(() => wasm?.shoot()),
    setPower: guard((p) => wasm?.setPower(Number(p))),
    setTip: guard((x, y) => wasm?.setTip(Number(x), Number(y))),
    newGame: guard((setup = {}) => {
      const { ruleset = 'eight-ball-wpa', breaker = 'random', raceTo = 1, names = [] } = setup;
      if (!RULESETS.has(ruleset)) throw new TypeError(`newGame: unknown ruleset "${ruleset}"`);
      const name = (i) => String(names[i] ?? '').trim() || t.player(i + 1);
      wasm?.newGame(ruleset, String(breaker), Math.round(Number(raceTo)) || 1, name(0), name(1));
    }),
    freePlay: guard(() => wasm?.freePlay()),
    callBall: guard((ball, pocket) => wasm?.callBall(Number(ball), String(pocket))),
    callSafety: guard(() => wasm?.callSafety()),
    autoCall: guard(() => wasm?.autoCall()),
    decide: guard((choice) => wasm?.decide(String(choice))),
    nextRack: guard(() => wasm?.nextRack()),
    stalemate: guard(() => wasm?.stalemate()),
    spotRequest: guard(() => wasm?.spotRequest()),
    setViewInsets: guard((top, bottom) => wasm?.setViewInsets(Number(top) || 0, Number(bottom) || 0)),
    state: () => (wasm && !destroyed ? wasm.state() : null),
    enterFullscreen: () => fs.enter(),
    exitFullscreen: () => fs.exit(),
  };

  const settingsStore = {
    get: () => settings,
    set(next) {
      settings = sanitize(next);
      saveSettings(settings, persist);
      applySettings();
      emit('settings-change', structuredClone(settings));
    },
    reset() {
      clearSettings();
      this.set(DEFAULTS);
    },
  };

  if (ui) {
    overlay = createOverlay(root, {
      t,
      locale,
      api,
      settings: settingsStore,
      onExit,
      fallbackUrl,
      rulesUrl,
      onTutorialDone: () => markTutorialSeen(persist),
    });
    overlay.setState('loading');
  }

  const fail = (kind, error) => {
    overlay?.showMessage(error.message === 'RELOAD_REQUIRED' ? 'reload' : kind);
    live = null;
    emit('error', { code: error.message });
    return error;
  };

  if (typeof WebAssembly !== 'object' || !hasWebGL2()) {
    throw fail('unsupported', new Error('UNSUPPORTED_BROWSER'));
  }

  // Load the module, then stream the binary with honest progress.
  try {
    const base = options.assetBase ? new URL(String(options.assetBase), document.baseURI) : null;
    const wasmUrl = base ? new URL('pkg/cue_web_bg.wasm', base) : new URL('./pkg/cue_web_bg.wasm', import.meta.url);
    const [module, manifest] = await Promise.all([
      base ? import(/* @vite-ignore */ new URL('pkg/cue_web.js', base).href) : import('./pkg/cue_web.js'),
      (base ? import(/* @vite-ignore */ new URL('manifest.js', base).href) : import('./manifest.js')).then(
        (m) => m.default,
        () => null,
      ),
    ]);
    const response = await fetch(wasmUrl);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    await module.default({
      module_or_path: withProgress(response, manifest?.wasm?.bytes, (p) => overlay?.setProgress(p)),
    });
    wasm = module;
  } catch (err) {
    canvas.remove();
    throw fail('error', new Error(`LOAD_FAILED: ${err?.message ?? err}`));
  }

  const ready = new Promise((resolve) => listeners.get('ready').add(resolve));
  wasm.setListener((name, detail) => emit(name, detail));
  try {
    wasm.mount(`#${canvas.id}`);
  } catch (err) {
    wasm.setListener(null);
    throw fail('error', err);
  }
  applySettings();

  // Browsers only allow audio from inside a user gesture.
  const unlock = () => {
    try {
      wasm.unlockAudio();
      root.removeEventListener('pointerdown', unlock, true);
      root.removeEventListener('keydown', unlock, true);
    } catch {
      // Not allowed yet; the next gesture will try again.
    }
  };
  root.addEventListener('pointerdown', unlock, true);
  root.addEventListener('keydown', unlock, true);

  const setMode = guard((mode, match = {}) => {
    if (mode === 'free-play') api.freePlay();
    else if (mode === 'eight-ball-local') api.newGame(match);
    else throw new TypeError(`setMode: unknown mode "${mode}"`);
  });

  const instance = {
    /** Build version reported by the engine. */
    version: wasm.version(),
    /**
     * Resolves once the game runs. `createCue` already starts it, so this
     * only resumes a paused instance; it never creates a second one.
     */
    start: guard(async () => {
      if (api.state()?.paused) api.resume();
    }),
    /** Starts free play, or a local eight-ball match with a `newGame` setup. */
    setMode,
    /**
     * Moves the game into another container, for single-page apps that
     * re-create the element CUE lives in. The instance, its state and its GPU
     * surface carry over: pair with `pause()` when hiding and `resume()` after.
     */
    attach: guard((next) => {
      if (!(next instanceof HTMLElement)) throw new TypeError('attach: `container` must be an HTMLElement');
      if (root.parentElement !== next) next.appendChild(root);
      overlay?.remeasure();
    }),
    /**
     * Re-measures the layout. The canvas follows its container on its own;
     * call this after moving or restyling it in ways a resize does not cover.
     */
    resize: guard(() => overlay?.remeasure()),
    /** Subscribes to an event; returns a function that unsubscribes. */
    on(name, handler) {
      const set = listeners.get(name);
      if (!set) throw new TypeError(`createCue: unknown event "${name}"`);
      set.add(handler);
      return () => set.delete(handler);
    },
    pause: api.pause,
    resume: api.resume,
    rerack: api.rerack,
    /** Alias of `rerack`, the name used in the specification. */
    reset: api.rerack,
    undo: api.undo,
    shoot: api.shoot,
    setPower: api.setPower,
    setTip: api.setTip,
    /**
     * Starts a local eight-ball match for two players on this device.
     * `ruleset`: 'eight-ball-wpa' (every shot called, the default) or
     * 'eight-ball-casual' (only the 8). `breaker`: 'a', 'b' or 'random'.
     * `raceTo`: racks needed to win, 1–15. `names` are display-only and never
     * stored.
     */
    newGame: api.newGame,
    /** Leaves any match and goes back to free play. */
    freePlay: api.freePlay,
    /** Eight-ball: calls `ball` into `pocket` ('foot-near', 'side-far'…). */
    callBall: api.callBall,
    /** Eight-ball: calls a safety for the next shot. */
    callSafety: api.callSafety,
    /** Eight-ball: lets the call follow the aim again. */
    autoCall: api.autoCall,
    /** Eight-ball: answers a pending decision with one of its option names. */
    decide: api.decide,
    /** Eight-ball: racks the next game once a rack is over. */
    nextRack: api.nextRack,
    /** Eight-ball: both players agree to a stalemate; the rack is replayed. */
    stalemate: api.stalemate,
    /** Eight-ball: spots the legal ball nearest the head string (WPA 1.6). */
    spotRequest: api.spotRequest,
    /**
     * CSS pixels at the top and bottom of the canvas covered by the host's own
     * controls; the cloth is framed clear of them. The built-in controls set
     * this themselves.
     */
    setViewInsets: api.setViewInsets,
    setTheme: guard((theme) => settingsStore.set({ ...settings, theme })),
    setAssists: guard((assists) => settingsStore.set({ ...settings, assists })),
    /** Graphics quality: 'auto', 'high', 'medium' or 'low'. Never changes the physics. */
    setQuality: guard((quality) => settingsStore.set({ ...settings, quality })),
    setAudio: guard((audio) =>
      settingsStore.set({ ...settings, sound: { ...settings.sound, ...audio } }),
    ),
    /** Current preferences. */
    getSettings: () => structuredClone(settings),
    /** Read-only snapshot of the engine state, or `null` once destroyed. */
    getState: () => api.state(),
    enterFullscreen: api.enterFullscreen,
    exitFullscreen: api.exitFullscreen,
    /** Shows the how-to-play dialog (built-in UI only). */
    showHelp: () => overlay?.showTutorial(),
    /** Engine counters, for diagnostics and tests. */
    stats: () => ({ frames: wasm.frameCount() }),
    /**
     * Stops the game and removes everything CUE added to the container. Safe
     * to call more than once; resolves after the engine released its surface.
     */
    destroy() {
      if (destroyed) return teardown;
      destroyed = true;
      teardown = new Promise((resolve) => {
        const finish = () => {
          clearTimeout(timer);
          wasm.setListener(null);
          motionQuery?.removeEventListener?.('change', applyMotion);
          fs.exit();
          fs.dispose();
          overlay?.destroy();
          root.remove();
          live = null;
          resolve();
        };
        // The engine applies the command on its next frame; don't wait
        // forever if the tab is hidden and frames are suspended.
        const timer = setTimeout(finish, 1000);
        listeners.get('destroyed').add(finish);
      });
      wasm.destroy();
      return teardown;
    },
  };

  live = instance;
  await ready;
  overlay?.setState('ready');
  if (options.mode === 'eight-ball-local') api.newGame(options.match ?? {});
  if (ui && !tutorialSeen(persist)) overlay.showTutorial();
  return instance;
}

function hasWebGL2() {
  try {
    const gl = document.createElement('canvas').getContext('webgl2');
    // Release the probe context right away; browsers cap live contexts.
    gl?.getExtension('WEBGL_lose_context')?.loseContext();
    return !!gl;
  } catch {
    return false;
  }
}

/**
 * Wraps a fetch response so reading it reports progress. `total` is the
 * uncompressed size from the manifest: with HTTP compression, Content-Length
 * counts compressed bytes while the stream yields decompressed ones.
 */
function withProgress(response, total, onProgress) {
  onProgress(total ? 0 : null);
  if (!response.body || !total) return response;
  let loaded = 0;
  const reader = response.body.getReader();
  const stream = new ReadableStream({
    async pull(controller) {
      const { done, value } = await reader.read();
      if (done) {
        onProgress(1);
        controller.close();
        return;
      }
      loaded += value.byteLength;
      onProgress(loaded / total);
      controller.enqueue(value);
    },
    cancel(reason) {
      return reader.cancel(reason);
    },
  });
  return new Response(stream, { headers: { 'Content-Type': 'application/wasm' } });
}

/**
 * Real fullscreen where the browser allows it on an element, otherwise an
 * "immersive" fixed layout (iPhone Safari only fullscreens video). Every
 * style it changes is restored on exit (spec §18.3, §18.5).
 */
function fullscreenController(root, onChange) {
  let immersive = null;
  const native = () => document.fullscreenElement === root;
  const notify = () => onChange(native() || immersive !== null);
  document.addEventListener('fullscreenchange', notify);

  return {
    async enter() {
      if (root.requestFullscreen && document.fullscreenEnabled) {
        try {
          await root.requestFullscreen({ navigationUI: 'hide' });
          return;
        } catch {
          // Denied: fall back to the immersive layout.
        }
      }
      if (immersive) return;
      immersive = { root: root.getAttribute('style'), body: document.body.style.overflow };
      Object.assign(root.style, {
        position: 'fixed',
        inset: '0',
        width: '100%',
        height: '100%',
        zIndex: '2147483000',
        background: '#000',
      });
      document.body.style.overflow = 'hidden';
      notify();
    },
    async exit() {
      if (native()) await document.exitFullscreen().catch(() => {});
      if (immersive) {
        root.setAttribute('style', immersive.root ?? '');
        document.body.style.overflow = immersive.body;
        immersive = null;
      }
      notify();
    },
    dispose() {
      document.removeEventListener('fullscreenchange', notify);
    },
  };
}
