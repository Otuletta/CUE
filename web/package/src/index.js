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
  'game-over',
  'match-over',
  'ball-spotted',
  'shot-refused',
  'call-refused',
]);

const RULESETS = new Set(['eight-ball-wpa', 'eight-ball-casual']);
const BREAKERS = new Set(['a', 'b', 'random']);

/** Throws for a match setup the engine would refuse. */
function checkMatch({ ruleset = 'eight-ball-wpa', breaker = 'random' } = {}) {
  if (!RULESETS.has(ruleset)) throw new TypeError(`newGame: unknown ruleset "${ruleset}"`);
  if (!BREAKERS.has(String(breaker))) throw new TypeError(`newGame: unknown breaker "${breaker}"`);
}

/** Safe, plain descriptions for `error` events: no paths, stacks or host data. */
const ERROR_MESSAGES = {
  UNSUPPORTED_BROWSER: 'This browser cannot run CUE: it needs WebAssembly and WebGL 2.',
  LOAD_FAILED: 'The game engine could not be downloaded or started.',
  RELOAD_REQUIRED: 'CUE was closed on this page. Reload the page to start it again.',
  ENGINE_FAILED: 'The game engine stopped after an internal error. Reload the page to play again.',
  CONTEXT_LOST: 'The browser stopped the graphics. Reload the page to play again.',
};

/** Pixels per CSS pixel at most: the default `render.pixelRatioCap`. */
const PIXEL_RATIO_CAP = 2;
/** The low quality tier renders at most this many, whatever the cap. */
const LOW_TIER_PIXEL_RATIO = 1.5;

/** Errors after which the instance cannot go on: only a reload helps. */
const FATAL = new Set(['ENGINE_FAILED', 'CONTEXT_LOST']);

/** `{ code, message }` for an error whose message is a code. */
function describeError(error) {
  const code = String(error?.message ?? error);
  const key = code.split(':')[0];
  return { code, message: ERROR_MESSAGES[key] ?? 'CUE could not start.' };
}

/** The rejection for a cancelled load: code `ABORTED`, named like a DOM abort. */
function abortError(signal) {
  const error = new Error('ABORTED', { cause: signal.reason });
  error.name = 'AbortError';
  return error;
}

const isSignal = (value) =>
  typeof value?.aborted === 'boolean' && typeof value.addEventListener === 'function';

/** A reason given for pausing or resuming: a short word, `host` by default. */
const reasonOf = (reason) => (typeof reason === 'string' && reason ? reason.slice(0, 32) : 'host');

const MODES = new Set(['free-play', 'eight-ball-local']);

// The page's one instance, or a token for the `createCue` call loading it.
let live = null;
// An instance was destroyed on this page: the engine cannot start again
// before a reload (ADR-0006), so later calls fail fast without downloading.
let spent = false;

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
 * @param {AbortSignal} [options.signal]  Cancels the load: rejects with `ABORTED`.
 * @param {{pixelRatioCap?: number}} [options.render]  `pixelRatioCap`: pixels rendered per
 *   CSS pixel at most (default 2), on screens denser than that.
 * @returns {Promise<object>} Resolves once the first frame has run.
 */
export async function createCue(options = {}) {
  // Every option is checked before CUE claims the page's one instance.
  const { container, signal } = options;
  if (!(container instanceof HTMLElement)) {
    throw new TypeError('createCue: `container` must be an HTMLElement');
  }
  if (signal != null && !isSignal(signal)) {
    throw new TypeError('createCue: `signal` must be an AbortSignal');
  }
  if (options.mode && !MODES.has(options.mode)) {
    throw new TypeError(`createCue: unknown mode "${options.mode}"`);
  }
  if (options.mode === 'eight-ball-local') checkMatch(options.match);
  const cap = options.render?.pixelRatioCap ?? PIXEL_RATIO_CAP;
  if (!(typeof cap === 'number' && cap >= 0.5)) {
    throw new TypeError('createCue: `render.pixelRatioCap` must be a number of at least 0.5');
  }
  if (live) throw new Error('INSTANCE_ALREADY_MOUNTED');
  if (signal?.aborted) throw abortError(signal);
  const claim = {};
  live = claim;
  try {
    return await mount(options, claim);
  } finally {
    // An error before the instance existed must not keep the page's slot.
    if (live === claim) live = null;
  }
}

/** The body of `createCue`, once `claim` holds the page's instance slot. */
async function mount(options, claim) {
  const { container, ui = true, persist = true, onExit, fallbackUrl, rulesUrl, signal } = options;
  const pixelRatioCap = options.render?.pixelRatioCap ?? PIXEL_RATIO_CAP;
  // Who holds the slot: `claim`, then the instance once it exists.
  let owner = claim;
  let instance = null;

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
  // The canvas sits in a frame that sets how many pixels it renders
  // (`applyResolution`).
  const frame = document.createElement('div');
  Object.assign(frame.style, {
    position: 'absolute',
    left: '0',
    top: '0',
    width: '100%',
    height: '100%',
    transformOrigin: '0 0',
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
  frame.appendChild(canvas);
  root.appendChild(frame);
  container.appendChild(root);

  let wasm = null;
  let destroyed = false;
  // Set by a fatal error: the instance stays, quiet and harmless, until
  // destroyed. `engineFailed`: the engine itself stopped and can take no
  // more commands.
  let failed = false;
  let engineFailed = false;
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
    if (wasm && !destroyed && !failed) wasm.setReducedMotion(reduced);
  };
  motionQuery?.addEventListener?.('change', applyMotion);

  // Insets the host or the controls asked for, in visible CSS pixels.
  let insets = [0, 0];
  // The canvas's CSS size over the visible size: 1, or less past the cap.
  let resolution = 1;
  const sendInsets = () => {
    if (wasm && !destroyed && !failed) wasm.setViewInsets(insets[0] * resolution, insets[1] * resolution);
  };

  // Renders at most `pixelRatioCap` pixels per CSS pixel, and fewer at the
  // low tier (spec §21.4). Past the cap the canvas gets a smaller CSS box, so
  // the browser sizes its drawing buffer to the cap times the visible box,
  // and a transform scales it back up. Pointer input still lands right: the
  // engine reads it in the canvas's own, untransformed, coordinates.
  let tier = null;
  let ratioQuery = null;
  const applyResolution = () => {
    const ratio = window.devicePixelRatio || 1;
    // Follow the ratio when the window moves to another screen or zooms.
    ratioQuery?.removeEventListener('change', applyResolution);
    ratioQuery = window.matchMedia?.(`(resolution: ${ratio}dppx)`) ?? null;
    ratioQuery?.addEventListener('change', applyResolution);
    const cap = Math.min(pixelRatioCap, tier === 'low' ? LOW_TIER_PIXEL_RATIO : Infinity);
    const next = Math.min(1, cap / ratio);
    if (next === resolution) return;
    resolution = next;
    Object.assign(frame.style, {
      width: `${next * 100}%`,
      height: `${next * 100}%`,
      transform: next < 1 ? `scale(${1 / next})` : '',
    });
    sendInsets();
  };
  applyResolution();

  const applySettings = () => {
    if (!wasm) return;
    wasm.setTheme(settings.theme);
    wasm.setAssists(settings.assists);
    wasm.setQuality(settings.quality);
    wasm.setAudio(settings.sound.enabled, settings.sound.volume);
    applyMotion();
  };

  // After a fatal error, methods do nothing rather than throw, so a host's
  // own code (a router hiding CUE, say) keeps working.
  const guard =
    (fn) =>
    (...args) => {
      if (destroyed) throw new Error('INSTANCE_DESTROYED');
      if (failed) return undefined;
      try {
        return fn(...args);
      } catch (err) {
        if (failed) return undefined; // the engine failed in this very call
        throw err;
      }
    };

  // A fatal error reported by the engine, or found by the wrapper.
  const stop = (code) => {
    if (failed || destroyed) return;
    failed = true;
    engineFailed = code === 'ENGINE_FAILED';
    try {
      wasm?.pause('error');
    } catch {
      // The engine is gone; there is nothing left to pause.
    }
    overlay?.showMessage('stopped');
    emit('error', describeError(new Error(code)));
  };
  // Fine aiming: Shift held while the mouse moves. Pointer events carry the
  // modifier even when the canvas lacks keyboard focus, which the engine's
  // own key input needs.
  let fineAim = false;
  const onPointerMove = (e) => {
    if (e.shiftKey === fineAim || !wasm || destroyed || failed) return;
    fineAim = e.shiftKey;
    wasm.setFineAim(fineAim);
  };
  canvas.addEventListener('pointermove', onPointerMove, { passive: true });

  // The browser can drop the WebGL context (a GPU reset, too many tabs). The
  // renderer cannot rebuild its resources, so this ends the instance.
  const onContextLost = () => stop('CONTEXT_LOST');
  canvas.addEventListener('webglcontextlost', onContextLost);

  const api = {
    pause: guard((reason) => wasm?.pause(reasonOf(reason))),
    resume: guard((reason) => wasm?.resume(reasonOf(reason))),
    rerack: guard(() => wasm?.rerack()),
    undo: guard(() => wasm?.undo()),
    shoot: guard(() => wasm?.shoot()),
    setPower: guard((p) => wasm?.setPower(Number(p))),
    setTip: guard((x, y) => wasm?.setTip(Number(x), Number(y))),
    turnAim: guard((radians) => wasm?.turnAim(Number(radians) || 0)),
    newGame: guard((setup = {}) => {
      checkMatch(setup);
      const { ruleset = 'eight-ball-wpa', breaker = 'random', raceTo = 1, names = [] } = setup;
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
    setViewInsets: guard((top, bottom) => {
      insets = [Number(top) || 0, Number(bottom) || 0];
      sendInsets();
    }),
    state: () => (wasm && !destroyed && !failed ? wasm.state() : null),
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

  // Removes everything CUE added: shared by `destroy` and a cancelled load.
  const release = () => {
    canvas.removeEventListener('pointermove', onPointerMove);
    ratioQuery?.removeEventListener('change', applyResolution);
    canvas.removeEventListener('webglcontextlost', onContextLost);
    motionQuery?.removeEventListener?.('change', applyMotion);
    fs.exit();
    fs.dispose();
    overlay?.destroy();
    root.remove();
    if (live === owner) live = null;
  };

  // `signal`: cancelling before the engine starts leaves nothing behind and
  // frees the page for another `createCue` at once, even within the same
  // task (React's StrictMode remounts that way). Once the engine runs, it
  // destroys the instance. Either way the promise rejects with `ABORTED`.
  let rejectAbort;
  const aborted = new Promise((_, reject) => (rejectAbort = reject));
  aborted.catch(() => {});
  const onAbort = () => {
    rejectAbort(abortError(signal));
    if (instance) instance.destroy();
    else release();
  };
  signal?.addEventListener('abort', onAbort, { once: true });
  // Waits for one loading step, and stops the load if it was cancelled meanwhile.
  const step = async (promise) => {
    const value = await (signal ? Promise.race([promise, aborted]) : promise);
    if (signal?.aborted) throw abortError(signal);
    return value;
  };

  // Without the built-in controls there is nothing to explain the failure,
  // so CUE takes its root away; with them, the root shows a message.
  const fail = (kind, error) => {
    signal?.removeEventListener('abort', onAbort);
    if (overlay) overlay.showMessage(error.message === 'RELOAD_REQUIRED' ? 'reload' : kind);
    else root.remove();
    if (live === owner) live = null;
    emit('error', describeError(error));
    return error;
  };

  if (typeof WebAssembly !== 'object' || !hasWebGL2()) {
    throw fail('unsupported', new Error('UNSUPPORTED_BROWSER'));
  }
  if (spent) throw fail('reload', new Error('RELOAD_REQUIRED'));

  // Load the module, then stream the binary with honest progress.
  try {
    const base = options.assetBase ? new URL(String(options.assetBase), document.baseURI) : null;
    const wasmUrl = base ? new URL('pkg/cue_web_bg.wasm', base) : new URL('./pkg/cue_web_bg.wasm', import.meta.url);
    const [module, manifest] = await step(
      Promise.all([
        base ? import(/* @vite-ignore */ new URL('pkg/cue_web.js', base).href) : import('./pkg/cue_web.js'),
        (base ? import(/* @vite-ignore */ new URL('manifest.js', base).href) : import('./manifest.js')).then(
          (m) => m.default,
          () => null,
        ),
      ]),
    );
    const response = await step(fetch(wasmUrl, { signal }));
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    await step(
      module.default({
        module_or_path: withProgress(response, manifest?.wasm?.bytes, (p) => overlay?.setProgress(p)),
      }),
    );
    wasm = module;
  } catch (err) {
    if (signal?.aborted) throw abortError(signal); // `onAbort` cleaned up
    frame.remove();
    throw fail('error', new Error(`LOAD_FAILED: ${err?.message ?? err}`));
  }

  const ready = new Promise((resolve) => listeners.get('ready').add(resolve));
  wasm.setListener((name, detail) => {
    if (name === 'error' && FATAL.has(detail?.code)) stop(detail.code);
    else if (name === 'tier') {
      tier = detail;
      applyResolution();
    } else emit(name, detail);
  });
  try {
    // Its size now, so the first frame renders at the size the page shows.
    wasm.mount(`#${canvas.id}`, Math.round(canvas.clientWidth), Math.round(canvas.clientHeight));
  } catch (err) {
    wasm.setListener(null);
    frame.remove();
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

  instance = {
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
     * Turns the aim by `radians`, counter-clockwise seen from above: fine
     * aiming for the host's own controls. Ignored while balls move.
     */
    turnAim: api.turnAim,
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
      spent = true;
      teardown = new Promise((resolve) => {
        const finish = () => {
          clearTimeout(timer);
          try {
            wasm.setListener(null);
          } catch {
            // A failed engine may refuse even this; it emits nothing anyway.
          }
          release();
          resolve();
        };
        // The engine applies the command on its next frame. Don't wait
        // forever if the tab is hidden and frames are suspended, nor at all
        // for an engine that failed: report `destroyed` on its behalf.
        const timer = setTimeout(() => emit('destroyed'), engineFailed ? 0 : 1000);
        listeners.get('destroyed').add(finish);
      });
      try {
        wasm.destroy();
      } catch {
        // A failed engine cannot take the command; the timer finishes.
      }
      return teardown;
    },
  };

  owner = instance;
  live = instance;
  try {
    await step(ready);
  } catch (err) {
    await instance.destroy(); // cancelled after the engine started
    throw err;
  }
  signal?.removeEventListener('abort', onAbort);
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
      // CSSOM, not the style attribute: a strict `style-src` allows only the former.
      immersive = { root: root.style.cssText, body: document.body.style.overflow };
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
        root.style.cssText = immersive.root;
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
