# ADR-0004 — No backend

- **Status:** accepted
- **Date:** 2026-10-17

## Decision

CUE ships as static files only: `.wasm`, JS glue, assets and a manifest. It
has no server, database, accounts, analytics or network calls of its own.
Local preferences, if any, use `localStorage` under a versioned prefix
(`cue:v1:*`).

## Consequences

- Nothing to operate, secure or back up beyond the static host.
- Privacy by construction: CUE collects no personal data.
- Online multiplayer, global leaderboards and synced progress are out of scope
  for 1.0. Revisiting this needs a new ADR.
