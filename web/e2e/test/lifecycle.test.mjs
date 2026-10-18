// The page lifecycle in a real browser, against the built bundle in
// web/dist: mounting, pausing, moving, destroying, cancelling and failing
// (docs/api.md, spec §17.3 and §18.4). Every page runs under the strict
// Content Security Policy and must report no violations and no uncaught
// errors.
import assert from 'node:assert/strict';
import { after, before, describe, test } from 'node:test';

import { launch } from '../browser.mjs';
import { serve } from '../server.mjs';

let server;
let browser;

before(async () => {
  server = await serve();
  browser = await launch();
});

after(async () => {
  await browser?.close();
  await server?.close();
});

/** A fresh page on the test host, in `using` (the shared browser by default). */
async function open({ using = browser, viewport = { width: 800, height: 900 } } = {}) {
  const context = await using.newContext({ viewport });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  await page.goto(`${server.origin}/e2e/host.html`);
  await page.waitForFunction(() => window.hostReady);
  return {
    page,
    /** Checks the page stayed clean, then closes it. */
    async close({ allowErrors = false } = {}) {
      const violations = await page.evaluate(() => window.violations);
      await context.close();
      assert.deepEqual(violations, [], 'Content Security Policy violations');
      if (!allowErrors) assert.deepEqual(errors, [], 'uncaught page errors');
      else if (errors.length) console.log(`page errors, allowed here: ${errors.join(' | ')}`);
    },
  };
}

const frameCount = (page) => page.evaluate(() => window.game.stats().frames);

/** Waits until the engine has run `n` more frames. */
async function framesPass(page, n = 3) {
  const start = await frameCount(page);
  await page.waitForFunction((target) => window.game.stats().frames >= target, start + n);
}

/** Waits for an event in `window.events` and returns its detail. */
async function nextEvent(page, name, timeout = 30_000) {
  await page.waitForFunction((n) => window.events.some(([e]) => e === n), name, { timeout });
  return page.evaluate((n) => window.events.find(([e]) => e === n)[1], name);
}

describe('one instance, from mount to destroy', () => {
  let tab;
  before(async () => {
    tab = await open();
  });
  after(async () => {
    await tab.close();
  });

  test('mounts in its container and runs', async () => {
    const result = await tab.page.evaluate(async () => {
      const container = document.querySelector('#a');
      window.game = await window.createCue({ container, ui: false, persist: false });
      window.record(window.game, ['shot-start', 'shot-end', 'racked', 'paused', 'resumed', 'destroyed', 'error']);
      const state = window.game.getState();
      return {
        version: window.game.version,
        roots: container.querySelectorAll(':scope > .cue-root').length,
        mode: state.mode,
        paused: state.paused,
      };
    });
    assert.match(result.version, /^\d+\.\d+\.\d+/);
    assert.deepEqual({ ...result, version: undefined }, { version: undefined, roots: 1, mode: 'free-play', paused: false });
    await framesPass(tab.page);
  });

  test('refuses a second instance and keeps the first running', async () => {
    const result = await tab.page.evaluate(async () => {
      const b = document.querySelector('#b');
      const code = await window.createCue({ container: b }).then(
        () => 'resolved',
        (error) => error.message,
      );
      return { code, children: b.childElementCount };
    });
    assert.deepEqual(result, { code: 'INSTANCE_ALREADY_MOUNTED', children: 0 });
    await framesPass(tab.page);
  });

  test('runs no frames while paused, and both changes report why', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.game.pause('test');
    });
    await page.waitForFunction(() => window.game.getState().paused);
    const paused = await frameCount(page);
    await page.waitForTimeout(600);
    const later = await frameCount(page);
    assert.ok(later - paused <= 1, `${later - paused} frames ran while paused`);

    await page.evaluate(() => window.game.resume());
    await framesPass(page);
    const changes = await page.evaluate(() => window.events.filter(([e]) => e === 'paused' || e === 'resumed'));
    assert.deepEqual(changes, [
      ['paused', { reason: 'test' }],
      ['resumed', { reason: 'host' }],
    ]);
  });

  test('reports a shot with one start and one end that share its id', { timeout: 90_000 }, async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.game.setPower(0.3);
      window.game.shoot();
    });
    const end = await nextEvent(page, 'shot-end', 80_000);
    const events = await page.evaluate(() => window.events.map(([e, detail]) => [e, detail]));
    const starts = events.filter(([e]) => e === 'shot-start');
    assert.equal(starts.length, 1);
    assert.equal(events.filter(([e]) => e === 'shot-end').length, 1);
    assert.deepEqual(starts[0][1], { shotId: end.shotId, shooter: null });
    assert.ok(end.shotId >= 1);
  });

  test('re-racks', async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.game.rerack();
    });
    assert.deepEqual(await nextEvent(page, 'racked'), { cleared: false });
  });

  test('moves to another container and keeps running', async () => {
    const { page } = tab;
    const result = await page.evaluate(() => {
      window.game.attach(document.querySelector('#b'));
      return {
        a: document.querySelector('#a').childElementCount,
        b: document.querySelectorAll('#b > .cue-root').length,
      };
    });
    assert.deepEqual(result, { a: 0, b: 1 });
    await framesPass(page);
  });

  test('destroys once, removes everything, then refuses every call', async () => {
    const result = await tab.page.evaluate(async () => {
      const { game } = window;
      window.events.length = 0;
      const first = game.destroy();
      const same = first === game.destroy();
      await first;
      let threw = null;
      try {
        game.pause();
      } catch (error) {
        threw = error.message;
      }
      return {
        same,
        threw,
        state: game.getState(),
        roots: document.querySelectorAll('.cue-root').length,
        events: window.events.map(([e]) => e),
      };
    });
    assert.deepEqual(result, {
      same: true,
      threw: 'INSTANCE_DESTROYED',
      state: null,
      roots: 0,
      events: ['destroyed'],
    });
  });

  test('needs a reload for another instance, and says so at once', async () => {
    const result = await tab.page.evaluate(async () => {
      const a = document.querySelector('#a');
      const started = performance.now();
      const code = await window.createCue({ container: a, ui: false }).then(
        () => 'resolved',
        (error) => error.message,
      );
      return { code, ms: performance.now() - started, children: a.childElementCount };
    });
    assert.equal(result.code, 'RELOAD_REQUIRED');
    assert.equal(result.children, 0);
    assert.ok(result.ms < 500, `took ${result.ms} ms: it should not download the engine again`);
  });
});

test('cancelling before the engine starts leaves nothing and frees the page', async () => {
  const tab = await open();
  const result = await tab.page.evaluate(async () => {
    const a = document.querySelector('#a');
    const outcome = (promise) =>
      promise.then(
        () => 'resolved',
        (error) => `${error.name} ${error.message}`,
      );

    const early = new AbortController();
    early.abort();
    const before = await outcome(window.createCue({ container: a, signal: early.signal }));
    const leftBefore = a.childElementCount;

    // Cancel, then start again in the same task, as React's StrictMode
    // remounts in development.
    const controller = new AbortController();
    const first = window.createCue({ container: a, signal: controller.signal });
    controller.abort();
    const second = window.createCue({ container: a, ui: false, persist: false });
    const cancelled = await outcome(first);
    window.game = await second;
    return {
      before,
      leftBefore,
      cancelled,
      roots: a.querySelectorAll(':scope > .cue-root').length,
      running: window.game.getState() !== null,
    };
  });
  assert.deepEqual(result, {
    before: 'AbortError ABORTED',
    leftBefore: 0,
    cancelled: 'AbortError ABORTED',
    roots: 1,
    running: true,
  });
  await framesPass(tab.page);
  await tab.close();
});

test('a bad option is refused before CUE claims the page', async () => {
  const tab = await open();
  const result = await tab.page.evaluate(async () => {
    const a = document.querySelector('#a');
    const refused = await window
      .createCue({ container: a, mode: 'eight-ball-local', match: { ruleset: 'nine-ball' } })
      .then(
        () => 'resolved',
        (error) => `${error.name}: ${error.message}`,
      );
    const leftover = a.childElementCount;
    window.game = await window.createCue({ container: a, ui: false, persist: false });
    return { refused, leftover, running: window.game.getState() !== null };
  });
  assert.deepEqual(result, {
    refused: 'TypeError: newGame: unknown ruleset "nine-ball"',
    leftover: 0,
    running: true,
  });
  await tab.close();
});

test('cancelling stops the download itself', async () => {
  const tab = await open();
  const { page } = tab;
  // Hold the binary so the load is caught mid-download.
  await page.route('**/cue_web_bg.wasm', () => {});
  const failed = page.waitForEvent('requestfailed', (r) => r.url().endsWith('cue_web_bg.wasm'));
  await page.evaluate(() => {
    window.controller = new AbortController();
    window.load = window.createCue({ container: document.querySelector('#a'), signal: window.controller.signal }).then(
      () => 'resolved',
      (error) => error.message,
    );
  });
  await page.waitForRequest((r) => r.url().endsWith('cue_web_bg.wasm'));
  const code = await page.evaluate(() => {
    window.controller.abort();
    return window.load;
  });
  assert.equal(code, 'ABORTED');
  await failed;
  assert.equal(await page.evaluate(() => document.querySelector('#a').childElementCount), 0);
  await tab.close();
});

test('a failed download is explained, leaves the page working, and can be retried', async () => {
  const tab = await open();
  const { page } = tab;
  await page.route('**/cue_web_bg.wasm', (route) => route.fulfill({ status: 404, body: '' }));
  const result = await page.evaluate(async () => {
    const a = document.querySelector('#a');
    const code = await window.createCue({ container: a, persist: false }).then(
      () => 'resolved',
      (error) => error.message,
    );
    const shadow = a.querySelector('.cue-overlay').shadowRoot;
    const message = shadow.querySelector('[data-sheet="message"]');
    return { code, shown: !message.hidden, title: message.querySelector('h2').textContent };
  });
  assert.deepEqual(result, { code: 'LOAD_FAILED: HTTP 404', shown: true, title: 'CUE couldn’t start' });

  await page.click('#host-button');
  assert.equal(await page.evaluate(() => window.hostClicks), 1);

  await page.unroute('**/cue_web_bg.wasm');
  const version = await page.evaluate(async () => {
    window.game = await window.createCue({ container: document.querySelector('#b'), ui: false, persist: false });
    return window.game.version;
  });
  assert.match(version, /^\d+\.\d+\.\d+/);
  await framesPass(page);
  await tab.close();
});

test('a lost WebGL context stops the instance without breaking the host', async () => {
  const tab = await open();
  const { page } = tab;
  await page.evaluate(async () => {
    window.game = await window.createCue({ container: document.querySelector('#a'), persist: false });
    window.record(window.game, ['error']);
    const gl = document.querySelector('#a canvas').getContext('webgl2');
    gl.getExtension('WEBGL_lose_context').loseContext();
  });
  const error = await nextEvent(page, 'error');
  assert.equal(error.code, 'CONTEXT_LOST');
  assert.equal(typeof error.message, 'string');
  assert.ok(error.message.length > 0);

  const result = await page.evaluate(async () => {
    const { game } = window;
    // Methods do nothing rather than throw, so the host's code goes on.
    game.pause('route');
    game.setPower(0.5);
    game.shoot();
    const shadow = document.querySelector('#a .cue-overlay').shadowRoot;
    const message = shadow.querySelector('[data-sheet="message"]');
    const state = game.getState();
    await game.destroy();
    return {
      state,
      shown: !message.hidden,
      title: message.querySelector('h2').textContent,
      roots: document.querySelectorAll('.cue-root').length,
      errors: window.events.filter(([e]) => e === 'error').length,
    };
  });
  assert.deepEqual(result, { state: null, shown: true, title: 'CUE stopped working', roots: 0, errors: 1 });
  await page.click('#host-button');
  assert.equal(await page.evaluate(() => window.hostClicks), 1);
  // The graphics stack may log or throw on its way down; the host is fine.
  await tab.close({ allowErrors: true });
});

test('the built-in pause control reports the player as the reason', async () => {
  const tab = await open();
  const { page } = tab;
  await page.evaluate(async () => {
    window.game = await window.createCue({ container: document.querySelector('#a'), persist: false });
    window.record(window.game, ['paused', 'resumed']);
  });
  // The how-to-play dialog opens on a first visit.
  await page.keyboard.press('Escape');
  await page.focus('#a canvas');
  await page.keyboard.press('p');
  await nextEvent(page, 'paused');
  await page.keyboard.press('p');
  await nextEvent(page, 'resumed');
  assert.deepEqual(await page.evaluate(() => window.events), [
    ['paused', { reason: 'user' }],
    ['resumed', { reason: 'user' }],
  ]);
  await tab.close();
});

test('in a match, shot-start names the shooter', { timeout: 90_000 }, async () => {
  const tab = await open();
  const { page } = tab;
  await page.evaluate(async () => {
    window.game = await window.createCue({
      container: document.querySelector('#a'),
      ui: false,
      persist: false,
      mode: 'eight-ball-local',
      match: { breaker: 'b' },
    });
    window.record(window.game, ['shot-start']);
  });
  await page.waitForFunction(() => window.game.getState().eightBall?.canShoot);
  await page.evaluate(() => {
    window.game.setPower(0.6);
    window.game.shoot();
  });
  const start = await nextEvent(page, 'shot-start');
  assert.equal(start.shooter, 'b');
  assert.equal(typeof start.shotId, 'number');
  await tab.close();
});

test('caps the pixel ratio, and a click still lands where it looks', async () => {
  // A real 3× screen. Emulating one (Playwright's deviceScaleFactor) changes
  // devicePixelRatio but not the device-pixel box the canvas is sized from.
  const dense = await launch(['--force-device-scale-factor=3', '--window-size=900,900']);
  const tab = await open({ using: dense, viewport: null });
  const { page } = tab;
  const buffer = () =>
    page.evaluate(() => {
      const canvas = document.querySelector('#a canvas');
      return [canvas.width, canvas.height];
    });
  const near = ([w, h], [W, H]) => Math.abs(w - W) <= 2 && Math.abs(h - H) <= 2;

  const sizes = await page.evaluate(async () => {
    window.game = await window.createCue({ container: document.querySelector('#a'), ui: false, persist: false });
    const canvas = document.querySelector('#a canvas');
    const box = canvas.getBoundingClientRect();
    return {
      ratio: window.devicePixelRatio,
      // The first frame has run: it must not have rendered a larger buffer.
      first: [canvas.width, canvas.height],
      visible: [box.width, box.height],
      own: [canvas.clientWidth, canvas.clientHeight],
    };
  });
  assert.equal(sizes.ratio, 3);
  assert.ok(near(sizes.first, [1280, 720]), `first frame at ${sizes.first}`);
  // Fills the container, to layout rounding.
  assert.ok(near(sizes.visible, [640, 360]), `visible ${sizes.visible}`);
  // 640 × 360 CSS pixels at the default cap of 2, not the screen's 3.
  await page.waitForFunction(() => document.querySelector('#a canvas').width > 300);
  assert.ok(near(await buffer(), [1280, 720]), `buffer ${await buffer()}`);

  // The engine reads pointers in the canvas's own coordinates (offsetX/Y),
  // which the scaling transform must not shift.
  await page.evaluate(() => {
    document.querySelector('#a canvas').addEventListener(
      'pointerdown',
      (e) => (window.offset = [e.offsetX, e.offsetY]),
      { once: true },
    );
  });
  await page.mouse.click(320, 180);
  const offset = await page.evaluate(() => window.offset);
  assert.ok(near(offset, [sizes.own[0] / 2, sizes.own[1] / 2]), `click at ${offset} in ${sizes.own}`);

  // The low tier renders at most 1.5 pixels per CSS pixel.
  await page.evaluate(() => window.game.setQuality('low'));
  await page.waitForFunction(() => document.querySelector('#a canvas').width < 1200);
  assert.ok(near(await buffer(), [960, 540]), `buffer ${await buffer()}`);
  await tab.close();
  await dense.close();
});
