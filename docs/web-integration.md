# Web integration

How to put CUE in a page and keep it well behaved there. The method-by-method
reference is [api.md](api.md).

## The contract

**CUE owns one element.** It adds a single root `div` to your container,
holding a `<canvas>` and the controls in a shadow root. It never changes
`body`, global styles or scrolling. The only exception is immersive
fullscreen, which restores every style it changed on exit. It listens for
input only on its own root.

CUE also guarantees the following:

- **No network beyond its own files.** It fetches no analytics, fonts or
  assets.
- **One instance per page.** Bevy's windowing layer (winit) allows one event
  loop per page. A second `createCue` rejects with `INSTANCE_ALREADY_MOUNTED`.
- **Pausing is free.** A paused instance runs 0 frames: no CPU, GPU or audio.
- **`destroy()` is final** for the page. After it, `createCue` rejects with
  `RELOAD_REQUIRED`. Single-page apps should pause and move the instance
  instead (see [below](#single-page-apps)).

## Plain HTML (no bundler)

Serve the package folder (`node_modules/@tuletta/cue`, or `web/dist/package`
from a build) as static files:

```html
<div id="cue" style="width: 100%; aspect-ratio: 16 / 9"></div>
<button id="play">Play</button>
<script type="module">
  document.querySelector('#play').addEventListener('click', async () => {
    const { createCue } = await import('/cue/index.js');
    await createCue({ container: document.querySelector('#cue') });
  });
</script>
```

[`web/example-vanilla`](../web/example-vanilla) is the full reference host,
the one deployed as the live demo.

## Vite

```sh
npm install @tuletta/cue
```

```js
// vite.config.js: keep CUE out of the dev server's dependency pre-bundling,
// which would move the module away from its WebAssembly.
export default { optimizeDeps: { exclude: ['@tuletta/cue'] } };
```

```js
const { createCue } = await import('@tuletta/cue'); // a chunk of its own
const game = await createCue({ container });
```

Production builds need nothing else. Vite copies and fingerprints the engine
(`cue_web_bg-[hash].wasm`), the glue and the manifest, because the wrapper
names them with static `import()` and `new URL(…, import.meta.url)`
expressions. [`web/example-vite`](../web/example-vite) is a working example.
CI installs the npm tarball into a fresh copy of it and builds it on every
pull request (`scripts/consumer-test.sh`).

## Files elsewhere (CDN)

Serve `manifest.js`, `manifest.json` and `pkg/` together and point CUE at
them:

```js
createCue({ container, assetBase: 'https://cdn.example.com/cue/1.0.0/' });
```

## Hosting

| Concern | What to do |
|---|---|
| MIME type | Serve `.wasm` as `application/wasm`, which streaming compilation needs. |
| Compression | The engine is 29 MB raw. It is 6.8 MiB with gzip and 4.2 MiB with Brotli; prefer Brotli where the host offers it. |
| Caching | Versioned paths (`/cue/1.0.0/`) or Vite's hashed names can be cached forever. |
| Integrity | `manifest.json` lists each file's size and SHA-256, for subresource integrity or a deployment check. |
| HTTPS | Required in practice; browsers restrict audio and fullscreen elsewhere. |

## Content Security Policy

CUE runs under a strict policy. Tested in Chromium with zero violations:

```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';
style-src 'self';
img-src 'self';
connect-src 'self';
object-src 'none'; base-uri 'none'
```

- `'wasm-unsafe-eval'` is what compiling WebAssembly needs; plain JavaScript
  `eval` is never used.
- No `'unsafe-inline'` is needed. The controls are styled with constructed
  stylesheets, and the manifest is a module, not a fetched `data:` URL. A
  browser without constructed stylesheets falls back to a `<style>` element
  and then needs `style-src 'unsafe-inline'`.
- With `assetBase` on another origin, add that origin to `script-src` and
  `connect-src`.

## Single-page apps

Keep one instance for the whole visit. Pause it and move it with the route,
instead of destroying and re-creating it:

```js
// cue-host.js: module scope survives route changes.
let cue;

export async function show(container) {
  if (cue) {
    cue.attach(container);
    cue.resume();
  } else {
    const { createCue } = await import('@tuletta/cue');
    cue = await createCue({ container });
  }
}

export function hide() {
  cue?.pause();
}
```

### React

```jsx
import { useEffect, useRef } from 'react';
import { show, hide } from './cue-host.js';

export function CueTable() {
  const ref = useRef(null);
  useEffect(() => {
    show(ref.current);
    return hide; // also covers StrictMode's double mount in development
  }, []);
  return <div ref={ref} style={{ width: '100%', aspectRatio: '16 / 9' }} />;
}
```

### Vue

```vue
<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { show, hide } from './cue-host.js';

const el = ref(null);
onMounted(() => show(el.value));
onBeforeUnmount(hide);
</script>

<template><div ref="el" style="width: 100%; aspect-ratio: 16 / 9" /></template>
```

**Leaving the page for good:** call `destroy()` only when CUE will not come
back before a reload, for example in a page's `pagehide` handler.

## Your own controls

With `ui: false`, CUE draws only the table. Build controls on the methods and
state in [api.md](api.md):

- Read `getState()` every frame, for example in `requestAnimationFrame`.
- React to events for announcements.
- Call `setViewInsets(top, bottom)` with the heights of your bars, so the
  camera keeps the balls clear of them.
- You don't need to unlock audio. CUE does it on the first `pointerdown` or
  `keydown` on its root, as browsers require a gesture.

## Privacy

CUE stores only preferences, under `cue:v1:settings` and
`cue:v1:tutorial-seen` in `localStorage`, and never stores player names. It
sends nothing anywhere. Your host's own logging and analytics are yours to
disclose (spec §23.3).
