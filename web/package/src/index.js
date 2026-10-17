// CUE web wrapper: the only API host pages use.
//
//   const game = await createCue({ container: document.querySelector('#cue') });
//   const off = game.on('shot-end', () => { ... });
//   game.pause(); game.resume(); game.rerack();
//   game.destroy();
//
// The wrapper owns one <canvas> inside the host's container and nothing else:
// no global styles, no listeners outside the canvas, no network requests
// beyond loading its own module and WebAssembly binary.

const EVENTS = new Set([
  'ready',
  'shot-start',
  'ball-pocketed',
  'shot-end',
  'paused',
  'resumed',
  'destroyed',
  'error',
]);

let live = null;

/**
 * Mounts CUE inside `container`.
 *
 * @param {object} options
 * @param {HTMLElement} options.container  Element the game fills. It must have a size.
 * @param {string|URL} [options.moduleUrl] Where the wasm-bindgen module lives.
 *   Defaults to `./pkg/cue_web.js` next to this file.
 * @returns {Promise<CueInstance>} Resolves once the first frame has run.
 */
export async function createCue({ container, moduleUrl } = {}) {
  if (!(container instanceof HTMLElement)) {
    throw new TypeError('createCue: `container` must be an HTMLElement');
  }
  if (live) {
    throw new Error('INSTANCE_ALREADY_MOUNTED');
  }
  if (typeof WebAssembly !== 'object' || !hasWebGL2()) {
    throw new Error('UNSUPPORTED_BROWSER');
  }

  const listeners = new Map([...EVENTS].map((name) => [name, new Set()]));
  const emit = (name, detail) => {
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

  const canvas = document.createElement('canvas');
  canvas.id = `cue-canvas-${Math.random().toString(36).slice(2, 10)}`;
  canvas.className = 'cue-canvas';
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-label', 'CUE pool table');
  Object.assign(canvas.style, {
    display: 'block',
    width: '100%',
    height: '100%',
    touchAction: 'none',
    outline: 'none',
  });
  container.appendChild(canvas);

  const url = moduleUrl ?? new URL('./pkg/cue_web.js', import.meta.url);
  let wasm;
  try {
    wasm = await import(/* @vite-ignore */ String(url));
    await wasm.default();
  } catch (err) {
    canvas.remove();
    throw new Error(`LOAD_FAILED: ${err?.message ?? err}`);
  }

  let destroyed = false;
  let teardown = null;
  const ready = new Promise((resolve) => listeners.get('ready').add(resolve));
  wasm.setListener((name, detail) => emit(name, detail));

  try {
    wasm.mount(`#${canvas.id}`);
  } catch (err) {
    wasm.setListener(null);
    canvas.remove();
    throw err;
  }

  const guard = (fn) => () => {
    if (destroyed) throw new Error('INSTANCE_DESTROYED');
    fn();
  };

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
    pause: guard(() => wasm.pause()),
    resume: guard(() => wasm.resume()),
    rerack: guard(() => wasm.rerack()),
    /** Engine counters, for diagnostics and tests. */
    stats: () => ({ frames: wasm.frameCount() }),
    /**
     * Stops the game and removes its canvas. Safe to call more than once.
     * Resolves after the engine has released its surface and emitted
     * `destroyed`.
     */
    destroy() {
      if (destroyed) return teardown;
      destroyed = true;
      teardown = new Promise((resolve) => {
        const finish = () => {
          clearTimeout(timer);
          wasm.setListener(null);
          canvas.remove();
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
