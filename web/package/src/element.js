// <cue-game>: CUE as one HTML tag, for any page or framework.
//
//   <script type="module" src="https://cdn.jsdelivr.net/npm/@tuletta/cue/element.js"></script>
//   <cue-game mode="eight-ball-local" locale="es"></cue-game>
//
// or, with a bundler, `import '@tuletta/cue/element';`.
//
// The element sizes itself (16:9, taller on a phone held upright; any CSS the
// page sets wins), downloads the engine only as it nears the viewport, pauses
// while it is scrolled away, while the tab is hidden and while it is out of
// the document, and dispatches every engine event as a `cue-<name>` DOM event.
// Reference: docs/api.md, "The <cue-game> element".
//
// A page runs one engine, and a destroyed one cannot start again before a
// reload (ADR-0006). So when an element leaves the document its game waits,
// paused, and the next <cue-game> to connect adopts it: a single-page app can
// route away and back as often as it likes.

import { createCue } from './index.js';

const TAG = 'cue-game';

// `:host` rules lose to any rule the page writes for the element, so these
// are only defaults.
const STYLES = `
:host {
  display: block;
  position: relative;
  aspect-ratio: 16 / 9;
  max-block-size: 100vh;
  max-block-size: 100svh;
  overflow: hidden;
  background: #08090b;
}
:host([hidden]) { display: none; }
@media (orientation: portrait) and (max-width: 640px) {
  :host { aspect-ratio: 3 / 4; }
}
`;

// Attributes that also apply while the game runs.
const LIVE = ['mode', 'ruleset', 'breaker', 'race-to', 'player-a', 'player-b', 'theme', 'assists', 'quality', 'muted', 'volume'];
const MATCH = new Set(['mode', 'ruleset', 'breaker', 'race-to', 'player-a', 'player-b']);
// Why an element wants the game paused; only `detached` ignores `autopause`.
const AUTO = new Set(['offscreen', 'hidden']);
// How far ahead of the viewport a lazy element starts loading.
const LOOKAHEAD = '50% 0px';

// The page's one game, its load while it runs, and the element showing it.
let game = null;
let loading = null;
let owner = null;
// Set while the element paused the game itself, so it only ever resumes
// its own pauses, never the player's.
let autoPaused = false;

const Base = globalThis.HTMLElement ?? class {};

/** The `<cue-game>` element. */
export class CueGameElement extends Base {
  static observedAttributes = LIVE;

  #hidden = new Set();
  #near = null;
  #seen = null;
  #park = 0;
  #matchQueued = false;

  constructor() {
    super();
    const shadow = this.attachShadow({ mode: 'open' });
    shadow.append(document.createElement('slot'));
    if ('adoptedStyleSheets' in ShadowRoot.prototype && 'replaceSync' in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet();
      sheet.replaceSync(STYLES);
      shadow.adoptedStyleSheets = [sheet];
    } else {
      const style = document.createElement('style');
      style.textContent = STYLES;
      shadow.prepend(style);
    }
  }

  /** The running game, while this element shows it; otherwise `null`. */
  get game() {
    return owner === this ? game : null;
  }

  /**
   * Starts the engine now rather than when the element nears the viewport.
   * Resolves with the game (the same object as `.game`).
   */
  start() {
    if (owner && owner !== this) {
      if (owner.isConnected) {
        const message = 'Another <cue-game> on this page is showing the game.';
        this.#emit('error', { code: 'INSTANCE_ALREADY_MOUNTED', message });
        return Promise.reject(new Error('INSTANCE_ALREADY_MOUNTED'));
      }
      this.#adopt();
    }
    if (game) return Promise.resolve(game);
    if (loading) return loading;
    owner = this;
    this.#near?.disconnect();
    let reported = false;
    loading = createCue({
      ...options(this),
      container: this,
      onEvent: (name, detail) => {
        if (name === 'error') reported = true;
        // `ready` comes once `.game` is set, below.
        if (name !== 'ready') owner?.#emit(name, detail);
        // Destroyed by the host: the page cannot start another engine, and
        // the next element to start says so (`RELOAD_REQUIRED`).
        if (name === 'destroyed') [game, owner, autoPaused] = [null, null, false];
      },
    }).then(
      (g) => {
        game = g;
        loading = null;
        // Adopted by another element while it loaded: that one's attributes win.
        if (owner !== this) {
          g.attach(owner);
          owner.#applyAll();
        }
        owner.#sync('visible');
        owner.#emit('ready', g.version);
        return g;
      },
      (error) => {
        loading = null;
        if (owner === this) owner = null;
        const code = String(error?.message ?? error).split(':')[0];
        if (!reported) this.#emit('error', { code, message: String(error?.message ?? error) });
        throw error;
      },
    );
    return loading;
  }

  connectedCallback() {
    clearTimeout(this.#park); // moved within the document, not removed
    this.#watch();
    if (owner === this) this.#show('detached');
    else if (owner && !owner.isConnected) this.#adopt();
    else if (this.getAttribute('loading') === 'eager') this.start().catch(() => {});
  }

  disconnectedCallback() {
    this.#near?.disconnect();
    this.#seen?.disconnect();
    document.removeEventListener('visibilitychange', this.#onVisibility);
    if (owner !== this) return;
    // A move fires `connectedCallback` straight after; wait to tell.
    this.#park = setTimeout(() => this.#hide('detached'), 0);
  }

  attributeChangedCallback(name, before, after) {
    if (before === after || !game || owner !== this) return;
    if (MATCH.has(name)) this.#applyMatchSoon();
    else this.#apply(name);
  }

  /** Takes the page's game over from an element that left the document. */
  #adopt() {
    const previous = owner;
    owner = this;
    previous.#hidden.clear();
    if (game) {
      game.attach(this);
      this.#applyAll();
      this.#sync('detached');
      this.#emit('ready', game.version);
    }
  }

  #watch() {
    if (typeof IntersectionObserver !== 'function') {
      if (this.getAttribute('loading') !== 'manual') this.start().catch(() => {});
      return;
    }
    if (!game && !loading && !['eager', 'manual'].includes(this.getAttribute('loading'))) {
      this.#near ??= new IntersectionObserver(
        (entries) => {
          if (entries.some((e) => e.isIntersecting)) this.start().catch(() => {});
        },
        { rootMargin: LOOKAHEAD },
      );
      this.#near.observe(this);
    }
    this.#seen ??= new IntersectionObserver((entries) => {
      const entry = entries[entries.length - 1];
      if (entry.isIntersecting) this.#show('offscreen');
      else this.#hide('offscreen');
    });
    this.#seen.observe(this);
    document.addEventListener('visibilitychange', this.#onVisibility);
    this.#onVisibility();
  }

  #onVisibility = () => {
    if (document.hidden) this.#hide('hidden');
    else this.#show('hidden');
  };

  #hide(reason) {
    this.#hidden.add(reason);
    this.#sync(reason);
  }

  #show(reason) {
    this.#hidden.delete(reason);
    this.#sync(reason);
  }

  /** Pauses or resumes the game to match why this element is out of sight. */
  #sync(reason) {
    if (!game || owner !== this) return;
    const why = [...this.#hidden].filter((r) => !AUTO.has(r) || autopause(this));
    const away = why.length > 0;
    // Pausing, say why the element is away; resuming, what just came back.
    if (away) reason = why[0];
    try {
      if (away && !autoPaused && !game.getState()?.paused) {
        autoPaused = true;
        game.pause(reason);
      } else if (!away && autoPaused) {
        autoPaused = false;
        game.resume(reason);
      }
    } catch {
      // A destroyed game: nothing left to pause.
    }
  }

  #emit(name, detail) {
    this.dispatchEvent(new CustomEvent(`cue-${name}`, { detail, bubbles: true, composed: true }));
  }

  /** Applies the live attributes this element carries to an adopted game. */
  #applyAll() {
    for (const name of ['theme', 'assists', 'quality', 'muted', 'volume']) {
      if (this.hasAttribute(name)) this.#apply(name);
    }
    if (this.hasAttribute('mode') && wantedMode(this) !== game.getState()?.mode) this.#applyMatchSoon();
  }

  #apply(name) {
    const value = this.getAttribute(name);
    try {
      if (name === 'theme' && value) game.setTheme(value);
      else if (name === 'assists' && value) game.setAssists(value);
      else if (name === 'quality' && value) game.setQuality(value);
      else if (name === 'muted' || name === 'volume') game.setAudio(audio(this) ?? { enabled: true });
    } catch (error) {
      this.#emit('error', { code: 'INVALID_ATTRIBUTE', message: `${name}: ${error.message}` });
    }
  }

  /** Several match attributes often change together: start one match for them. */
  #applyMatchSoon() {
    if (this.#matchQueued) return;
    this.#matchQueued = true;
    queueMicrotask(() => {
      this.#matchQueued = false;
      if (!game || owner !== this) return;
      // Free play has no setup: only a change into it restarts anything.
      if (wantedMode(this) === 'free-play' && game.getState()?.mode === 'free-play') return;
      try {
        game.setMode(this.getAttribute('mode') || 'free-play', match(this));
      } catch (error) {
        this.#emit('error', { code: 'INVALID_ATTRIBUTE', message: `mode: ${error.message}` });
      }
    });
  }
}

const has = (el, name) => el.hasAttribute(name) && !['false', 'off'].includes(el.getAttribute(name));
const text = (el, name) => el.getAttribute(name) || undefined;
const autopause = (el) => el.getAttribute('autopause') !== 'off';
/** The mode the attributes ask for, as the game's state names it. */
const wantedMode = (el) => (el.getAttribute('mode') === 'eight-ball-local' ? 'eight-ball' : 'free-play');

function audio(el) {
  const volume = Number.parseFloat(el.getAttribute('volume') ?? '');
  if (!el.hasAttribute('muted') && !Number.isFinite(volume)) return undefined;
  return {
    enabled: !has(el, 'muted'),
    ...(Number.isFinite(volume) && { volume: Math.min(1, Math.max(0, volume)) }),
  };
}

function match(el) {
  const raceTo = Number.parseInt(el.getAttribute('race-to') ?? '', 10);
  return {
    ...(text(el, 'ruleset') && { ruleset: text(el, 'ruleset') }),
    ...(text(el, 'breaker') && { breaker: text(el, 'breaker') }),
    ...(Number.isFinite(raceTo) && { raceTo }),
    names: [el.getAttribute('player-a') ?? '', el.getAttribute('player-b') ?? ''],
  };
}

/** `createCue` options from the element's attributes. */
function options(el) {
  const cap = Number.parseFloat(el.getAttribute('pixel-ratio-cap') ?? '');
  return {
    ui: !['off', 'none', 'false'].includes(el.getAttribute('ui')),
    persist: el.getAttribute('persist') !== 'off',
    locale: text(el, 'locale'),
    theme: text(el, 'theme'),
    assists: text(el, 'assists'),
    quality: text(el, 'quality'),
    audio: audio(el),
    mode: text(el, 'mode'),
    match: match(el),
    assetBase: text(el, 'asset-base'),
    fallbackUrl: text(el, 'fallback-url'),
    rulesUrl: text(el, 'rules-url'),
    ...(Number.isFinite(cap) && { render: { pixelRatioCap: cap } }),
    accessibility: {
      ...(el.hasAttribute('reduced-motion') && { reducedMotion: has(el, 'reduced-motion') }),
      highContrast: has(el, 'high-contrast'),
    },
  };
}

if (globalThis.customElements && !customElements.get(TAG)) customElements.define(TAG, CueGameElement);
