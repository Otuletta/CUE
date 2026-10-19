// The <cue-game> element in a real browser, against the built bundle in
// web/dist (docs/api.md, "The <cue-game> element"): it sizes itself, loads
// only near the viewport, pauses out of sight, follows its attributes and
// hands its game to the next element when it leaves the document. Every page
// runs under the strict Content Security Policy.
import assert from 'node:assert/strict';
import { after, before, describe, test } from 'node:test';

import { harness } from '../helpers.mjs';

let h;
before(async () => {
  h = await harness();
});
after(async () => {
  await h?.close();
});

/** Waits for a `cue-<name>` event (from element `id`, if given); returns its detail. */
async function nextEvent(page, name, id, timeout = 60_000) {
  await page.waitForFunction(
    ([n, i]) => window.events.some(([e, , t]) => e === n && (!i || t === i)),
    [name, id],
    { timeout },
  );
  return page.evaluate(
    ([n, i]) => window.events.findLast(([e, , t]) => e === n && (!i || t === i))[1],
    [name, id],
  );
}

const frames = (page, id) => page.evaluate((i) => document.getElementById(i).game.stats().frames, id);

describe('<cue-game>', () => {
  let tab;
  before(async () => {
    tab = await h.open({ path: 'element.html' });
  });
  after(async () => {
    await tab.close();
  });

  test('sizes itself and loads nothing while out of view', async () => {
    const { page } = tab;
    await page.waitForTimeout(800);
    const result = await page.evaluate(() => {
      const el = document.getElementById('game');
      const box = el.getBoundingClientRect();
      return {
        defined: customElements.get('cue-game') === el.constructor,
        size: [box.width, box.height],
        game: el.game,
        wasm: window.wasmRequested(),
      };
    });
    assert.deepEqual(result, { defined: true, size: [640, 360], game: null, wasm: false });
  });

  test('starts as it nears the viewport and announces itself', async () => {
    const { page } = tab;
    await page.evaluate(() => document.getElementById('game').scrollIntoView({ block: 'center' }));
    const version = await nextEvent(page, 'ready', 'game');
    const result = await page.evaluate(() => {
      const el = document.getElementById('game');
      window.first = el.game;
      return { version: el.game.version, inside: el.querySelectorAll(':scope > .cue-root').length };
    });
    assert.equal(version, result.version);
    assert.equal(result.inside, 1);
    const start = await frames(page, 'game');
    await page.waitForFunction((n) => document.getElementById('game').game.stats().frames > n + 2, start);
  });

  test('pauses while scrolled away and resumes when back', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.scrollTo(0, 0);
    });
    assert.deepEqual(await nextEvent(page, 'paused', 'game'), { reason: 'offscreen' });
    await page.evaluate(() => document.getElementById('game').scrollIntoView({ block: 'center' }));
    assert.deepEqual(await nextEvent(page, 'resumed', 'game'), { reason: 'offscreen' });
  });

  test('never resumes a pause it did not make', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      document.getElementById('game').game.pause('user');
      window.scrollTo(0, 0);
    });
    await nextEvent(page, 'paused', 'game');
    await page.evaluate(() => document.getElementById('game').scrollIntoView({ block: 'center' }));
    await page.waitForTimeout(800);
    const result = await page.evaluate(() => ({
      paused: document.getElementById('game').game.getState().paused,
      resumed: window.events.some(([e]) => e === 'resumed'),
    }));
    assert.deepEqual(result, { paused: true, resumed: false });
    await page.evaluate(() => document.getElementById('game').game.resume('user'));
    await nextEvent(page, 'resumed', 'game');
  });

  test('follows its attributes while it runs', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      document.getElementById('game').setAttribute('theme', 'classic');
    });
    const settings = await nextEvent(page, 'settings-change', 'game');
    assert.equal(settings.theme, 'classic');

    await page.evaluate(() => {
      const el = document.getElementById('game');
      el.setAttribute('player-a', 'Ana');
      el.setAttribute('player-b', 'Bo');
      el.setAttribute('mode', 'eight-ball-local');
    });
    assert.deepEqual(await nextEvent(page, 'new-game', 'game'), { mode: 'eight-ball' });
    const state = await page.evaluate(() => document.getElementById('game').game.getState());
    assert.equal(state.mode, 'eight-ball');
    // Three attribute changes, one match.
    const games = await page.evaluate(() => window.events.filter(([e]) => e === 'new-game').length);
    assert.equal(games, 1);
  });

  test('leaves its game, paused, for the next element to adopt', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.old = document.getElementById('game');
      // Out of the document, its events no longer bubble to the page.
      window.old.addEventListener('cue-paused', (e) => window.events.push(['paused', e.detail, 'game']));
      window.old.remove();
    });
    assert.deepEqual(await nextEvent(page, 'paused', 'game'), { reason: 'detached' });

    await page.evaluate(() => {
      const next = document.createElement('cue-game');
      next.id = 'next';
      next.setAttribute('ui', 'off');
      const host = document.getElementById('elsewhere');
      host.append(next);
      next.scrollIntoView({ block: 'center' });
    });
    await nextEvent(page, 'ready', 'next');
    assert.deepEqual(await nextEvent(page, 'resumed', 'next'), { reason: 'detached' });
    const result = await page.evaluate(() => {
      const next = document.getElementById('next');
      return {
        same: next.game === window.first,
        oldGame: window.old.game,
        moved: next.querySelectorAll(':scope > .cue-root').length,
        left: window.old.querySelectorAll('.cue-root').length,
        mode: next.game.getState().mode,
      };
    });
    // The match carries over: the new element asks for no mode.
    assert.deepEqual(result, { same: true, oldGame: null, moved: 1, left: 0, mode: 'eight-ball' });
    const start = await frames(page, 'next');
    await page.waitForFunction((n) => document.getElementById('next').game.stats().frames > n + 2, start);
  });

  test('a second element in view says the game is taken', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      const third = document.createElement('cue-game');
      third.id = 'third';
      third.setAttribute('loading', 'eager');
      document.body.append(third);
    });
    const error = await nextEvent(page, 'error', 'third');
    assert.equal(error.code, 'INSTANCE_ALREADY_MOUNTED');
    const still = await page.evaluate(() => document.getElementById('next').game === window.first);
    assert.equal(still, true);
  });
});
