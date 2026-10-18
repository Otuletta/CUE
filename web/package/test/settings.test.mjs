// Preferences come from localStorage, which anyone can edit: every field is
// validated on the way in.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { DEFAULTS, sanitize } from '../src/settings.js';

test('anything that is not a settings object gives the defaults', () => {
  for (const input of [null, undefined, 42, 'studio', [], () => {}]) {
    assert.deepEqual(sanitize(input), { ...DEFAULTS, sound: { ...DEFAULTS.sound } });
  }
});

test('each field falls back on its own', () => {
  const s = sanitize({ theme: 'neon', assists: 'assisted', quality: 'ultra', sound: { enabled: 'yes', volume: 2 } });
  assert.equal(s.theme, DEFAULTS.theme);
  assert.equal(s.assists, 'assisted');
  assert.equal(s.quality, DEFAULTS.quality);
  assert.equal(s.sound.enabled, DEFAULTS.sound.enabled);
  assert.equal(s.sound.volume, 1, 'volume is clamped');
  assert.equal(sanitize({ sound: { volume: -1 } }).sound.volume, 0);
  assert.equal(sanitize({ sound: { volume: 'loud' } }).sound.volume, DEFAULTS.sound.volume);
});

test('valid settings come through untouched, and nothing else does', () => {
  const valid = { theme: 'contrast', assists: 'off', quality: 'low', sound: { enabled: false, volume: 0.25 } };
  assert.deepEqual(sanitize({ ...valid, injected: '<img onerror=alert(1)>' }), valid);
});
