# ADR-0012 — Distribution, integration and releases

- **Status:** accepted
- **Date:** 2026-10-18

Resolves spec §31 ADR-015 (the distribution namespace), with the name still
provisional, and settles how Phase 6 delivers a library that someone outside
the project can integrate.

> **Amended (2026-10-18): the name is confirmed.** The owner reserved the
> scope as the npm organization `tuletta`, with two-factor authentication
> enforced for its members, so the package is `@tuletta/cue`.

## Context

The exit criterion for Phase 6 is that someone outside the project can
integrate and run CUE using only the public documentation. The spec asks for:

- an ES module wrapper plus the WASM (§17.1);
- testing in a Vite host and a plain host before fixing the signature
  (§17.2);
- a clean remount in single-page apps (§18.4);
- a strict Content Security Policy that is not weakened blindly (§23.4);
- tested packaging, checksums and a release pipeline (§26.2).

Three facts constrain this:

- **One loop per page.** winit allows one event loop per page, so a destroyed
  instance cannot be re-created without a reload (ADR-0006).
- **Bundlers move files.** They fingerprint and relocate assets. A wrapper
  that builds URLs at runtime breaks under them.
- **Inline code and CSP.** Inline `<style>` elements and fetched `data:` URLs
  need policy exceptions that hosts should not have to grant.

## Decision

**One package, one version.** `@tuletta/cue` is a provisional name. It
contains:

- the wrapper (ES modules) and TypeScript declarations;
- the WASM and its glue;
- a manifest as JSON (for integrity checks) and as a module;
- the licences.

`Cargo.toml` holds the only version number, for the crates, the binary and
the package alike. Releases start at `1.0.0-beta.1`, the beta channel of
§26.4.

**Bundler-safe by construction.** The wrapper names its own files only with
static `import('./pkg/cue_web.js')`, `import('./manifest.js')` and
`new URL('./pkg/cue_web_bg.wasm', import.meta.url)` expressions. Vite (and
other bundlers that follow the same convention) copies and fingerprints them
with no plugin. Development servers that pre-bundle dependencies are told to
leave the package alone (`optimizeDeps.exclude`). `assetBase` covers files
served from a CDN or a public folder.

**Strict CSP without exceptions.** The controls are styled through
constructed stylesheets, falling back to a `<style>` element in old
browsers. The manifest is imported, not fetched, so nothing becomes a
`data:` fetch. The policy CUE needs is
`script-src 'self' 'wasm-unsafe-eval'`, with no `'unsafe-inline'`. A test
page with that policy loads with zero violations.

**Remounting means moving, not re-creating.** The instance gains:

- `attach(container)`, which moves the live root, canvas and GPU surface
  into a new element;
- `start()`, `resize()` and `setMode()`, which complete the §17.3 contract.

Single-page apps keep one instance, pause it when hidden (0 frames) and
attach it to the next view. `destroy()` stays final and says so
(`RELOAD_REQUIRED`, with a message that asks for a reload). Isolating CUE in
an iframe per mount was considered and rejected for now:

- it costs a full engine per mount;
- under a bundler, there is no reliable way to load the same module into the
  frame without inline script.

**Tested like a consumer.** On every pull request, CI:

- runs the wrapper's unit tests, including a check that every interface
  string exists in both languages;
- packs the npm tarball;
- installs it into a fresh copy of `web/example-vite` and builds it with
  Vite;
- checks that the engine and the wrapper are in the output.

**Releases from tags; npm by hand.** Pushing `vX.Y.Z` runs the `Release`
workflow. It checks the tag against the version, runs every test, builds,
runs the consumer test, and publishes a GitHub release with the tarball, a
zip of the demo, `SHA256SUMS` and notes taken from `CHANGELOG.md`. Publishing
to npm stays a manual step for the account owner (`docs/releasing.md`).
Credentials never live in CI for a step that cannot be undone.

## Consequences

**Positive**

- A Vite app integrates CUE with one install, one line of config and a
  dynamic import. A plain host needs nothing but static files.
- Hosts keep a strict CSP.
- Typed API: editors and TypeScript hosts get the options, methods, events
  and state shapes.
- Releases are reproducible and verifiable, and keep previous artefacts for
  rollbacks.

**Costs**

- One instance per page remains a v1 limit. Hosts that truly need to
  re-create CUE must reload, or host it in a page of their own inside an
  iframe.
- The npm name is provisional until the owner reserves the scope. Changing
  it before the first npm publish costs only a find-and-replace.
- The `.tgz` is 7.2 MB, because the engine is 29 MB uncompressed. Hosts serve
  it compressed (4.2 MiB with Brotli).

## Review conditions

- If winit gains support for re-creating event loops on the web,
  `destroy()` can stop being final.
- Before 1.0: test on the device matrix, and in at least one more framework
  host (React, through the documented pattern).
