// Local preferences (spec §23.2). Stored under a versioned key in
// localStorage, validated on every read, and never required: private modes,
// blocked storage or quota errors simply fall back to the defaults.

const SETTINGS_KEY = 'cue:v1:settings';
const TUTORIAL_KEY = 'cue:v1:tutorial-seen';

export const THEMES = ['classic', 'studio', 'contrast'];
export const ASSISTS = ['off', 'basic', 'assisted'];
export const QUALITIES = ['auto', 'high', 'medium', 'low'];

export const DEFAULTS = Object.freeze({
  theme: 'studio',
  assists: 'basic',
  quality: 'auto',
  sound: Object.freeze({ enabled: true, volume: 0.7 }),
});

/** Returns a clean settings object from any input, falling back per field. */
export function sanitize(input, base = DEFAULTS) {
  const s = input && typeof input === 'object' ? input : {};
  const sound = s.sound && typeof s.sound === 'object' ? s.sound : {};
  const volume = Number(sound.volume);
  return {
    theme: THEMES.includes(s.theme) ? s.theme : base.theme,
    assists: ASSISTS.includes(s.assists) ? s.assists : base.assists,
    quality: QUALITIES.includes(s.quality) ? s.quality : base.quality,
    sound: {
      enabled: typeof sound.enabled === 'boolean' ? sound.enabled : base.sound.enabled,
      volume: Number.isFinite(volume) ? Math.min(1, Math.max(0, volume)) : base.sound.volume,
    },
  };
}

function read(key) {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key, value) {
  try {
    if (value === null) window.localStorage.removeItem(key);
    else window.localStorage.setItem(key, value);
  } catch {
    // Storage unavailable: preferences last for this visit only.
  }
}

export function loadSettings(persist) {
  if (!persist) return sanitize(null);
  try {
    return sanitize(JSON.parse(read(SETTINGS_KEY) ?? 'null'));
  } catch {
    return sanitize(null);
  }
}

export function saveSettings(settings, persist) {
  if (persist) write(SETTINGS_KEY, JSON.stringify(sanitize(settings)));
}

export function clearSettings() {
  write(SETTINGS_KEY, null);
  write(TUTORIAL_KEY, null);
}

export function tutorialSeen(persist) {
  return persist && read(TUTORIAL_KEY) === '1';
}

export function markTutorialSeen(persist) {
  if (persist) write(TUTORIAL_KEY, '1');
}
