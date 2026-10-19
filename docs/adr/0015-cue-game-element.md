# ADR-0015 — The `<cue-game>` element

- **Status:** accepted
- **Date:** 2026-10-19

Extends [ADR-0012](0012-distribution-and-integration.md) with a second way to
integrate CUE. `createCue` is unchanged.

## Context

`createCue` gives hosts full control, but every host repeats the same work:

- give the container a size;
- load the engine only when the visitor gets near it, not with the page;
- pause it when it scrolls away, when the tab is hidden and when a router
  removes it;
- bring it back after a route change. ADR-0006 rules out a remount: a page
  can start the engine once.

The owner asked for CUE to fit any platform it is put on "without having to
touch the code much". Offered an iframe, a web component or a poster with a
play button, they chose the **web component**.

## Decision

**`@tuletta/cue/element` defines `<cue-game>`**, a custom element over
`createCue`:

- **Sizes itself:** 16:9 by default, and 3:4 on a phone held upright, so the
  table turns to portrait and fills it. The defaults are `:host` rules in
  the element's shadow root, so any CSS the page writes for the element wins.
- **Loads lazily:** the engine downloads as the element nears the viewport
  (half a screen ahead). `loading="eager"` starts at once and
  `loading="manual"` waits for `start()`.
- **Pauses out of sight:** when scrolled away, when the tab is hidden and
  when out of the document. It resumes only pauses it made itself, never
  the player's. `autopause="off"` keeps it running except when detached.
- **Attributes are options:** `mode`, the match setup, `theme`, `assists`,
  `quality`, `muted` and `volume` also apply live. Several match attributes
  changed in one task start one match.
- **Events are DOM events:** every engine event is dispatched on the element
  as `cue-<name>`, bubbling and composed, with the same detail. `.game` is
  the `createCue` instance, so the whole API stays available.
- **Adoption instead of remount:** the game lives in the element's light
  DOM; Bevy finds its canvas with `document.querySelector`. When an element
  leaves the document, its game waits, paused. The next `<cue-game>` to
  connect adopts it with `attach`, applies its own attributes and resumes.
  A route change away and back costs nothing and needs no reload.
- **One per page,** as ADR-0006 requires: a second element in view while the
  first shows the game reports `INSTANCE_ALREADY_MOUNTED` as a `cue-error`.
- **Works without a bundler:** the module imports its siblings by relative
  URL, so it works from a CDN with one `<script type="module">`. It is safe
  to import during server-side rendering: nothing is defined where
  `customElements` does not exist.

To feed the element, `createCue` gains one option, `onEvent(name, detail)`:
every event, from the start of the load, errors before the instance exists
included.

## Consequences

**Positive**

- One tag is a complete integration: size, loading, pausing and routing are
  handled.
- Custom elements are framework-neutral: frameworks that render them
  (React 19, Vue, Svelte, Astro) need no adapter. Only plain HTML and Vite
  are tested so far.
- No new runtime dependency, and no global styles or listeners beyond the
  element's own (one `visibilitychange` listener while connected).

**Costs and risks**

- The control layer's download budget rises to 45 KiB (was 40) for the
  element's 3.3 KiB. Hosts that use `createCue` alone never download it.
- Events from an element out of the document no longer bubble to the page;
  listen on the element itself for those.
- The package is no longer free of side effects: `element.js` registers the
  element, and `package.json` says so (`sideEffects`), so bundlers keep it.

## Evidence

- `web/e2e/test/element.test.mjs`, in a real browser under the strict CSP:
  - the element sizes itself and downloads nothing while out of view;
  - it starts near the viewport and dispatches `cue-ready`;
  - it pauses while scrolled away and resumes when back, but never resumes
    a pause it did not make;
  - live attributes apply, with one match for several changes;
  - a removed element's game is adopted, still in its match, by the next
    element;
  - a second element in view reports `INSTANCE_ALREADY_MOUNTED`.
- `scripts/consumer-test.sh`: the packed tarball builds in a clean Vite
  project whose page is one `<cue-game>` tag, and the element survives the
  build.

## Review conditions

- Several instances per page become possible (spec §31 ADR-018).
- A host framework cannot render custom elements with attributes and
  events.
