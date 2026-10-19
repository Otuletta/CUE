// Types for `@tuletta/cue/element`: the <cue-game> element. Reference:
// docs/api.md, "The <cue-game> element".
import type { CueEvents, CueInstance } from './index.js';

/**
 * `<cue-game>`: CUE as one tag. Attributes (all optional):
 *
 * - `mode`: `free-play` (default) or `eight-ball-local`; with `ruleset`,
 *   `breaker`, `race-to`, `player-a`, `player-b` for the match. Live.
 * - `theme`, `assists`, `quality`, `muted`, `volume` (0–1). Live.
 * - `locale` (`en`, `es`), `ui="off"`, `persist="off"`, `asset-base`,
 *   `fallback-url`, `rules-url`, `pixel-ratio-cap`, `reduced-motion`,
 *   `high-contrast`: read when the engine starts.
 * - `loading`: `lazy` (default: as the element nears the viewport), `eager`
 *   (as soon as it is in the document) or `manual` (on `start()`).
 * - `autopause="off"`: keep playing while scrolled away or in a hidden tab.
 *
 * Every engine event is dispatched on the element as `cue-<name>`
 * (`cue-ready`, `cue-shot-end`, `cue-error`…), bubbling, with the event's
 * detail.
 */
export declare class CueGameElement extends HTMLElement {
  /** The running game while this element shows it, else `null`. */
  readonly game: CueInstance | null;
  /** Starts the engine now; resolves with the game. */
  start(): Promise<CueInstance>;
}

/** The DOM events of `<cue-game>`: each engine event, prefixed `cue-`. */
export type CueGameEventMap = {
  [K in keyof CueEvents as `cue-${K & string}`]: CustomEvent<CueEvents[K]>;
};

declare global {
  interface HTMLElementTagNameMap {
    'cue-game': CueGameElement;
  }
}
