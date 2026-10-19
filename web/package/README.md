# @tuletta/cue

**A billiards engine in Rust and WebAssembly that you embed with one tag.**
It provides free play and two-player eight-ball, with WPA or casual rules,
purpose-built physics and accessible controls in English and Spanish.

- [Live demo](https://otuletta.github.io/CUE/)
- [Source and documentation](https://github.com/Otuletta/CUE)
- [API reference](https://github.com/Otuletta/CUE/blob/main/docs/api.md)

```html
<script type="module" src="https://cdn.jsdelivr.net/npm/@tuletta/cue/element.js"></script>
<cue-game mode="eight-ball-local" locale="es"></cue-game>
```

`<cue-game>` sizes itself, downloads the engine only as it nears the
viewport, pauses while out of sight and survives a single-page app's route
changes. Every event arrives as a `cue-<name>` DOM event, and `.game` is the
full API. Attributes: `mode`, `ruleset`, `breaker`, `race-to`, `player-a`,
`player-b`, `theme`, `assists`, `quality`, `muted`, `volume`, `locale`,
`ui="off"`, `persist="off"`, `loading`, `autopause` and more
([reference](https://github.com/Otuletta/CUE/blob/main/docs/api.md#the-cue-game-element)).

For full control, mount it yourself:

```js
import { createCue } from '@tuletta/cue';

const game = await createCue({ container: document.querySelector('#cue') });
game.on('shot-end', (shot) => console.log(shot.pocketed));
```

The container must have a size. CUE fills it with a canvas and its own
controls, and touches nothing else on the page.

## Install

```sh
npm install @tuletta/cue
```

The package is an ES module. It ships the wrapper, the element, the engine
(`pkg/cue_web_bg.wasm`, about 4.2 MiB with Brotli) and a manifest. The
engine loads only when you call `createCue`, or when a `<cue-game>` nears
the viewport.

```js
import '@tuletta/cue/element'; // defines <cue-game>
```

### Vite

The wrapper references its files with `import()` and
`new URL(…, import.meta.url)`, which Vite copies and fingerprints on its own
at build time. Keep the package out of the dev-server pre-bundling, which
would move the module away from its files:

```js
// vite.config.js
export default { optimizeDeps: { exclude: ['@tuletta/cue'] } };
```

Load it on demand, so your first page does not pay for the engine:

```js
playButton.addEventListener('click', async () => {
  const { createCue } = await import('@tuletta/cue');
  await createCue({ container });
});
```

### Without a bundler

Serve the package folder as static files and import `index.js`. Serve
`.wasm` files as `application/wasm`.

### Files on a CDN or another path

```js
createCue({ container, assetBase: 'https://cdn.example.com/cue/1.0.0/' });
```

## Single-page apps

`<cue-game>` handles route changes itself: a new element adopts the running
game. With `createCue`:

**There is one instance per page,** and `destroy()` is final: winit, the
windowing layer, allows one event loop per page. Keep the instance and move
it instead:

```js
let cue; // module scope: survives route changes. A promise, so overlapping calls share one.

export function show(container) {
  cue ??= createCue({ container });
  return cue.then((game) => {
    game.attach(container);
    game.resume('route');
  });
}

export function hide() {
  cue?.then((game) => game.pause('route'), () => {}); // 0 frames while paused
}
```

## Requirements

- WebAssembly and WebGL 2. Without them, CUE shows a message and the host
  page keeps working.
- No backend, no accounts and no network requests beyond its own files.
  Preferences stay in `localStorage` under `cue:v1:*`.

## Licence

MIT or Apache-2.0, at your option. CUE ships no third-party assets: every
texture, mesh and sound is generated in code.
