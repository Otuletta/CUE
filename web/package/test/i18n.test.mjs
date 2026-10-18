// Every string exists in both languages, with the same shape: a missing
// translation shows up here, not in front of a player.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { formatList, resolveLocale, strings } from '../src/i18n.js';

/** Flattens a strings object to `path → type`. */
function shape(value, path = '', out = new Map()) {
  if (Array.isArray(value)) {
    out.set(path, `array(${value.length})`);
  } else if (value && typeof value === 'object') {
    for (const [key, inner] of Object.entries(value)) shape(inner, path ? `${path}.${key}` : key, out);
  } else {
    out.set(path, typeof value);
  }
  return out;
}

test('English and Spanish define the same strings with the same shapes', () => {
  const en = shape(strings('en'));
  const es = shape(strings('es'));
  const missing = [...en.keys()].filter((k) => !es.has(k));
  const extra = [...es.keys()].filter((k) => !en.has(k));
  assert.deepEqual(missing, [], 'missing in Spanish');
  assert.deepEqual(extra, [], 'missing in English');
  for (const [key, type] of en) assert.equal(es.get(key), type, key);
});

test('every function string returns text for sample arguments', () => {
  for (const locale of ['en', 'es']) {
    const t = strings(locale);
    const visit = (value, path) => {
      if (typeof value === 'function') {
        // `balls` takes a list of numbers; every other string takes words,
        // numbers or a `{ chooser, breaker }` object and must cope.
        const text = path.endsWith('.balls')
          ? value([3, 11])
          : value('Ana', 'Bo', 3, 'x', { chooser: 'Ana', breaker: 'Bo', cueBallOff: false });
        assert.equal(typeof text, 'string', `${locale}.${path}`);
        assert.ok(text.length > 0, `${locale}.${path}`);
      } else if (value && typeof value === 'object' && !Array.isArray(value)) {
        for (const [k, v] of Object.entries(value)) visit(v, `${path}.${k}`);
      }
    };
    visit(t, locale);
  }
});

test('locales resolve with fallbacks', () => {
  assert.equal(resolveLocale('es-AR'), 'es');
  assert.equal(resolveLocale('EN'), 'en');
  assert.ok(['en', 'es'].includes(resolveLocale('fr')), 'falls back to a supported one');
});

test('lists read naturally in both languages', () => {
  assert.equal(formatList(['3', '9', '11'], 'en'), '3, 9, and 11');
  assert.equal(formatList(['3', '9', '11'], 'es'), '3, 9 y 11');
  assert.equal(strings('es').balls([3, 11]), 'la 3 y la 11');
});
