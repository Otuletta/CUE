// Accessibility of the built-in controls, automated (spec §24.1): an axe
// audit of every dialog and state in both languages, and the keyboard paths
// a player depends on. What stays manual is in docs/accessibility.md.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { after, before, describe, test } from 'node:test';

import { harness, nextEvent } from '../helpers.mjs';

const axeSource = readFileSync(createRequire(import.meta.url).resolve('axe-core/axe.min.js'), 'utf8');

let h;
before(async () => {
  h = await harness();
});
after(async () => {
  await h?.close();
});

/** A page with CUE mounted with its controls, in `locale`, and axe loaded. */
async function mounted(locale, options = {}) {
  const tab = await h.open();
  // Evaluated through the DevTools protocol, which the page's CSP does not
  // govern; a <script> tag would be blocked, as it should be.
  await tab.page.evaluate(axeSource);
  await tab.page.evaluate(
    async ({ locale, options }) => {
      window.game = await window.createCue({ container: document.querySelector('#a'), persist: false, locale, ...options });
      window.record(window.game, ['shot-end', 'paused', 'resumed']);
      window.shadow = document.querySelector('#a .cue-overlay').shadowRoot;
    },
    { locale, options },
  );
  return tab;
}

/** Serious and critical axe findings on the page, as readable lines. */
async function audit(page) {
  // Dialogs fade in: judge what the player sees once they have settled.
  await page.waitForFunction(() => {
    const root = document.querySelector('#a .cue-overlay')?.shadowRoot ?? document;
    return root
      .getAnimations()
      .every((a) => a.playState !== 'running' || a.effect?.getTiming().iterations === Infinity);
  });
  const found = await page.evaluate(async () => {
    const result = await window.axe.run(document, { resultTypes: ['violations'] });
    return result.violations
      .filter((v) => v.impact === 'serious' || v.impact === 'critical')
      .map((v) => `${v.id} (${v.impact}): ${v.nodes.map((n) => n.target.flat().join(' ')).join(', ')}`);
  });
  return found;
}

/** The open dialog or sheet, by its `data-sheet` name, or null. */
const openSheet = (page) =>
  page.evaluate(() => window.shadow.querySelector('.sheet:not([hidden])')?.dataset.sheet ?? null);

/** What has keyboard focus inside CUE: a short description. */
const focused = (page) =>
  page.evaluate(() => {
    const el = window.shadow.activeElement ?? document.activeElement;
    if (!el) return null;
    if (el.tagName === 'CANVAS') return 'canvas';
    return el.dataset?.action ?? el.getAttribute('name') ?? el.className ?? el.tagName;
  });

const pressInCue = async (page, key) => {
  await page.keyboard.press(key);
  await page.waitForTimeout(50);
};

test('the audit reaches inside the shadow root', async () => {
  const tab = await mounted('en');
  // With a dialog open the rest is inert and rightly skipped: close it.
  await pressInCue(tab.page, 'Escape');
  const { reached, visible } = await tab.page.evaluate(async () => {
    const result = await window.axe.run(document, { runOnly: ['button-name'] });
    return {
      // A node inside a shadow root is targeted by a path: [host, selector].
      reached: result.passes.flatMap((rule) => rule.nodes).filter((node) => Array.isArray(node.target[0])).length,
      visible: [...window.shadow.querySelectorAll('button')].filter((b) => b.checkVisibility()).length,
    };
  });
  assert.ok(visible >= 6, `${visible} visible buttons`);
  assert.equal(reached, visible, 'every visible button in the shadow root is audited');
  await tab.close();
});

for (const locale of ['en', 'es']) {
  describe(`every state passes an audit (${locale})`, () => {
    let tab;
    before(async () => {
      tab = await mounted(locale);
    });
    after(async () => {
      await tab.close();
    });

    test('the how-to-play dialog that opens on a first visit', async () => {
      assert.equal(await openSheet(tab.page), 'help');
      assert.deepEqual(await audit(tab.page), []);
    });

    test('the table with its toolbar and dock', async () => {
      await pressInCue(tab.page, 'Escape');
      assert.equal(await openSheet(tab.page), null);
      assert.deepEqual(await audit(tab.page), []);
    });

    for (const [key, sheet] of [
      ['o', 'settings'],
      ['p', 'paused'],
      ['n', 'game'],
      ['h', 'help'],
    ]) {
      test(`the ${sheet} sheet`, async () => {
        const { page } = tab;
        await page.focus('#a canvas');
        await pressInCue(page, key);
        assert.equal(await openSheet(page), sheet);
        assert.deepEqual(await audit(page), []);
        await pressInCue(page, key === 'p' ? 'p' : 'Escape');
        assert.equal(await openSheet(page), null);
      });
    }

    test('the match setup, and a match in progress', async () => {
      const { page } = tab;
      await page.focus('#a canvas');
      await pressInCue(page, 'n');
      await page.evaluate(() => window.shadow.querySelector('[data-sheet="game"] [data-action="setup"]').click());
      assert.equal(await openSheet(page), 'setup');
      assert.deepEqual(await audit(page), [], 'setup');

      await page.evaluate(() => window.shadow.querySelector('[data-form="setup"] button[type="submit"]').click());
      await page.waitForFunction(() => window.game.getState().eightBall?.canShoot);
      assert.equal(await openSheet(page), null);
      assert.deepEqual(await audit(page), [], 'match');
      await page.evaluate(() => window.game.freePlay());
    });
  });
}

test('the message shown when the engine cannot load passes an audit', async () => {
  const tab = await h.open();
  await tab.page.evaluate(axeSource);
  await tab.page.route('**/cue_web_bg.wasm', (route) => route.fulfill({ status: 404, body: '' }));
  await tab.page.evaluate(() =>
    window.createCue({ container: document.querySelector('#a'), persist: false }).catch(() => {}),
  );
  assert.deepEqual(await audit(tab.page), []);
  await tab.close();
});

describe('keyboard paths', () => {
  let tab;
  before(async () => {
    tab = await mounted('en');
    await pressInCue(tab.page, 'Escape'); // the first-visit dialog
  });
  after(async () => {
    await tab.close();
  });

  for (const sheet of ['settings', 'setup']) {
    test(`Tab stays inside the ${sheet} dialog, both ways`, async () => {
      const { page } = tab;
      await page.focus('#a canvas');
      if (sheet === 'settings') {
        await pressInCue(page, 'o');
      } else {
        // Opens with its heading focused, which Tab itself cannot reach.
        await pressInCue(page, 'n');
        await page.evaluate(() => window.shadow.querySelector('[data-sheet="game"] [data-action="setup"]').click());
      }
      assert.equal(await openSheet(page), sheet);
      const inside = () =>
        page.evaluate((name) => window.shadow.querySelector(`[data-sheet="${name}"]`).contains(window.shadow.activeElement), sheet);
      for (const key of ['Shift+Tab', 'Tab']) {
        for (let i = 0; i < 20; i++) {
          await page.keyboard.press(key);
          assert.ok(await inside(), `${key} ${i + 1} left the dialog for ${await focused(page)}`);
        }
      }
      await pressInCue(page, 'Escape');
      assert.equal(await openSheet(page), null);
    });
  }

  test('Escape closes a dialog and gives focus back to the control that opened it', async () => {
    const { page } = tab;
    await page.click('#a .cue-overlay >> [data-action="settings"]');
    assert.equal(await openSheet(page), 'settings');
    await pressInCue(page, 'Escape');
    assert.equal(await openSheet(page), null);
    assert.equal(await focused(page), 'settings');
  });

  test('the shortcuts work from the table', async () => {
    const { page } = tab;
    await page.focus('#a canvas');
    await pressInCue(page, 'h');
    assert.equal(await openSheet(page), 'help');
    await pressInCue(page, 'Escape');
    assert.equal(await focused(page), 'canvas');

    const pressed = () => page.evaluate(() => window.shadow.querySelector('[data-action="sound"]').getAttribute('aria-pressed'));
    const before = await pressed();
    await pressInCue(page, 'm');
    assert.notEqual(await pressed(), before, 'M toggles the sound');
    await pressInCue(page, 'm');
    assert.equal(await pressed(), before);

    await pressInCue(page, 'p');
    assert.equal(await openSheet(page), 'paused');
    await pressInCue(page, 'p');
    assert.equal(await openSheet(page), null);
  });

  test('the result of a shot is announced to screen readers', { timeout: 90_000 }, async () => {
    const { page } = tab;
    await page.evaluate(() => {
      window.events.length = 0;
      window.game.setPower(0.3);
      window.game.shoot();
    });
    await nextEvent(page, 'shot-end', 80_000);
    await page.waitForFunction(() => window.shadow.querySelector('[role="status"]').textContent.trim().length > 0);
    const live = await page.evaluate(() => {
      const region = window.shadow.querySelector('[role="status"]');
      return { polite: region.getAttribute('aria-live'), text: region.textContent.trim() };
    });
    assert.equal(live.polite, 'polite');
    assert.ok(live.text.length > 3, `announced "${live.text}"`);
  });
});
