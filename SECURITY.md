# Security policy

## Supported versions

Fixes go into the latest release. Before 1.0, that means the latest beta.

## Reporting a vulnerability

Report privately, through GitHub's **private vulnerability reporting** for
this repository: the *Security* tab, then *Report a vulnerability*. Please
do not open a public issue for a vulnerability.

Include what you found, how to reproduce it, and the impact you expect. You
will get an acknowledgement, and a fix or an assessment, as soon as
possible. Credit is given in the release notes unless you prefer otherwise.

## What CUE does and does not do

These facts bound the risk:

- **No backend, no accounts, no network.** CUE fetches only its own files
  and sends nothing anywhere.
- **Local storage holds preferences only**, under `cue:v1:*`. Player names
  are never stored.
- **Host input is untrusted.** Options, names and commands are validated.
  Player names are inserted as text, never as markup; names are
  validated in Rust too, and capped at 24 characters.
- **Strict CSP compatible:** it needs `script-src 'self' 'wasm-unsafe-eval'`
  and no `'unsafe-inline'` ([web-integration.md](docs/web-integration.md#content-security-policy)).
- **No secrets** in the code, the WASM or the package.
- **Errors do not leak local paths:** source paths in the binary are
  rewritten at build time (`scripts/build-web.sh`).
- **Supply chain:**
  - `cargo deny` checks licences, advisories and sources on every pull
    request.
  - The wrapper has no runtime npm dependencies.
  - `manifest.json` and the release's `SHA256SUMS` let hosts check the files
    they serve.
