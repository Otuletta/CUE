// CUE web wrapper: the only API host pages use.
//
//   import { createCue } from './package/index.js';
//   const game = await createCue({ container: document.querySelector('#cue') });
//   const off = game.on('shot-end', (shot) => { ... });
//   game.pause(); game.resume(); game.undo(); game.rerack();
//   await game.destroy();
//
// The wrapper owns one root element inside the host's container (a <canvas>
// plus an optional accessible control layer in a shadow root) and nothing
// else: no global styles, no listeners outside that root, no network requests
// beyond its own module, binary and manifest.

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
]);

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
 * @param {{enabled?: boolean, volume?: number}} [options.audio]  Overrides saved sound settings.
 * @param {boolean} [options.persist=true]  Remember preferences in localStorage.
 * @param {() => void} [options.onExit]  Shows an Exit button that calls this.
 * @param {string} [options.fallbackUrl]  Link offered when the browser is unsupported.
 * @param {string|URL} [options.moduleUrl]  wasm-bindgen module; defaults to `./pkg/cue_web.js`.
 * @returns {Promise<object>} Resolves once the first frame has run.
 */
export async function createCue(options = {}) {
  const { container, ui = true, persist = true, onExit, fallbackUrl } = options;
  if (!(container instanceof HTMLElement)) {
    throw new TypeError('createCue: `container` must be an HTMLElement');
  }
  if (live) throw new Error('INSTANCE_ALREADY_MOUNTED');
  live = {};

  const locale = resolveLocale(options.locale);
  const t = strings(locale);
  let settings = sanitize({
    ...loadSettings(persist),
    ...(options.theme && { theme: options.theme }),
    ...(options.assists && { assists: options.assists }),
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

  const applySettings = () => {
    if (!wasm) return;
    wasm.setTheme(settings.theme);
    wasm.setAssists(settings.assists);
    wasm.setAudio(settings.sound.enabled, settings.sound.volume);
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
      onTutorialDone: () => markTutorialSeen(persist),
    });
    overlay.setState('loading');
  }

  const fail = (kind, error) => {
    overlay?.showMessage(kind);
    live = null;
    emit('error', { code: error.message });
    return error;
  };

  if (typeof WebAssembly !== 'object' || !hasWebGL2()) {
    throw fail('unsupported', new Error('UNSUPPORTED_BROWSER'));
  }

  // Load the module, then stream the binary with honest progress.
  try {
    const moduleUrl = options.moduleUrl ?? new URL('./pkg/cue_web.js', import.meta.url);
    const [module, manifest] = await Promise.all([
      import(/* @vite-ignore */ String(moduleUrl)),
      fetchManifest(),
    ]);
    const wasmUrl = new URL(manifest?.wasm?.file ?? 'cue_web_bg.wasm', new URL('./pkg/', import.meta.url));
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

  const instance = {
    /** Build version reported by the engine. */
    version: wasm.version(),
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
    setTheme: guard((theme) => settingsStore.set({ ...settings, theme })),
    setAssists: guard((assists) => settingsStore.set({ ...settings, assists })),
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

async function fetchManifest() {
  try {
    const res = await fetch(new URL('./manifest.json', import.meta.url));
    return res.ok ? await res.json() : null;
  } catch {
    return null;
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
