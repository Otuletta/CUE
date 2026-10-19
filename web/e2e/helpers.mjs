// Shared pieces of the browser tests: the server and browser for a test
// file, fresh pages on the test host, and waits on the engine.
import assert from 'node:assert/strict';

import { launch } from './browser.mjs';
import { serve } from './server.mjs';

/**
 * Starts the server and the browser for one test file. `open` gives a fresh
 * page on the test host; its `close` checks that the page stayed clean.
 */
export async function harness() {
  const server = await serve();
  const browser = await launch();

  /**
   * A fresh page on a test host (`path`, `host.html` by default), in `using`
   * (this file's browser by default).
   */
  async function open({ using = browser, viewport = { width: 800, height: 900 }, path = 'host.html' } = {}) {
    const context = await using.newContext({ viewport });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(String(error)));
    await page.goto(`${server.origin}/e2e/${path}`);
    await page.waitForFunction(() => window.hostReady);
    return {
      page,
      /** Checks for CSP violations and uncaught errors, then closes the page. */
      async close({ allowErrors = false } = {}) {
        const violations = await page.evaluate(() => window.violations);
        await context.close();
        assert.deepEqual(violations, [], 'Content Security Policy violations');
        if (!allowErrors) assert.deepEqual(errors, [], 'uncaught page errors');
        else if (errors.length) console.log(`page errors, allowed here: ${errors.join(' | ')}`);
      },
    };
  }

  return {
    server,
    browser,
    open,
    async close() {
      await browser.close();
      await server.close();
    },
  };
}

export const frameCount = (page) => page.evaluate(() => window.game.stats().frames);

/** Waits until the engine has run `n` more frames. */
export async function framesPass(page, n = 3) {
  const start = await frameCount(page);
  await page.waitForFunction((target) => window.game.stats().frames >= target, start + n);
}

/** Waits for an event in `window.events` and returns its latest detail. */
export async function nextEvent(page, name, timeout = 30_000) {
  await page.waitForFunction((n) => window.events.some(([e]) => e === n), name, { timeout });
  return page.evaluate((n) => window.events.findLast(([e]) => e === n)[1], name);
}
