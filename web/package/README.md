# @tuletta/cue

**A billiards engine in Rust and WebAssembly that you embed with one call.**
It provides free play and two-player eight-ball, with WPA or casual rules,
purpose-built physics and accessible controls in English and Spanish.

- [Live demo](https://otuletta.github.io/CUE/)
- [Source and documentation](https://github.com/Otuletta/CUE)
- [API reference](https://github.com/Otuletta/CUE/blob/main/docs/api.md)

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

The package is an ES module. It ships the wrapper, the engine
(`pkg/cue_web_bg.wasm`, about 4.2 MiB with Brotli) and a manifest, and
nothing loads until you call `createCue`.

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
