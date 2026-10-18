// Fine aiming and the dialogs' fit, driven as a player would.
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

/** The angle from `a` to `b`, in (−π, π]. */
const turned = (a, b) => Math.atan2(Math.sin(b - a), Math.cos(b - a));
const aimAngle = (page) => page.evaluate(() => window.game.getState().aimAngle);
const frames = (page) => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
const DEG = Math.PI / 180;

describe('fine aiming', () => {
  let tab;
  let page;
  before(async () => {
    tab = await h.open();
    page = tab.page;
    await page.evaluate(async () => {
      window.game = await window.createCue({ container: document.querySelector('#a'), persist: false, locale: 'en' });
      window.shadow = document.querySelector('#a .cue-overlay').shadowRoot;
    });
    await page.keyboard.press('Escape'); // the first-visit dialog
  });
  after(async () => {
    await tab.close();
  });

  test('turnAim turns the aim by the angle asked, and aimAngle reports it', async () => {
    const start = await aimAngle(page);
    await page.evaluate(() => window.game.turnAim(0.01));
    await page.waitForFunction((a) => window.game.getState().aimAngle !== a, start);
    assert.ok(Math.abs(turned(start, await aimAngle(page)) - 0.01) < 1e-9);
  });

  test('the fine-aim strip turns the aim a little per pixel, clockwise to the right', async () => {
    const strip = await page.locator('#a .cue-overlay >> .jog').boundingBox();
    const start = await aimAngle(page);
    await page.mouse.move(strip.x + strip.width / 2, strip.y + strip.height / 2);
    await page.mouse.down();
    await page.mouse.move(strip.x + strip.width / 2 + 40, strip.y + strip.height / 2, { steps: 4 });
    await page.mouse.up();
    await frames(page);
    const by = turned(start, await aimAngle(page));
    // 40 px at 0.05° each: 2°, clockwise on screen (a negative angle).
    assert.ok(Math.abs(by + 2 * DEG) < 0.05 * DEG, `turned ${by / DEG}°`);
  });

  test('the strip takes arrow keys: a quarter of a degree, a twentieth with Shift', async () => {
    await page.focus('#a .cue-overlay >> .jog');
    let start = await aimAngle(page);
    await page.keyboard.press('ArrowLeft');
    await frames(page);
    assert.ok(Math.abs(turned(start, await aimAngle(page)) - 0.25 * DEG) < 1e-6);
    start = await aimAngle(page);
    await page.keyboard.press('Shift+ArrowRight');
    await frames(page);
    assert.ok(Math.abs(turned(start, await aimAngle(page)) + 0.05 * DEG) < 1e-6);
  });

  test('Shift turns the aim a tenth as much as the mouse moves, and the aim then stays', async () => {
    const box = await page.locator('#a canvas').boundingBox();
    const at = (dx, dy) => [box.x + box.width * dx, box.y + box.height * dy];
    // Aim at a point well away from the cue ball.
    await page.mouse.move(...at(0.8, 0.3));
    await frames(page);
    const free = await aimAngle(page);
    await page.mouse.move(...at(0.8, 0.5));
    await frames(page);
    const full = turned(free, await aimAngle(page));

    await page.mouse.move(...at(0.8, 0.3));
    await frames(page);
    const before = await aimAngle(page);
    await page.keyboard.down('Shift');
    await page.mouse.move(...at(0.8, 0.5), { steps: 8 });
    await frames(page);
    await page.keyboard.up('Shift');
    const fine = turned(before, await aimAngle(page));
    assert.ok(Math.abs(fine - full / 10) < Math.abs(full) * 0.02, `full ${full / DEG}°, fine ${fine / DEG}°`);

    // Releasing Shift without moving keeps the fine aim.
    await frames(page);
    assert.ok(Math.abs(turned(before, await aimAngle(page)) - fine) < 1e-9);
  });
});

test('a long dialog keeps its button in view on a small stage', async () => {
  const tab = await h.open();
  const { page } = tab;
  const fit = await page.evaluate(async () => {
    const a = document.querySelector('#a'); // 640 × 360
    window.game = await window.createCue({ container: a, persist: false, locale: 'en' });
    const shadow = a.querySelector('.cue-overlay').shadowRoot;
    const card = shadow.querySelector('[data-sheet="help"] .card');
    const button = card.querySelector('.actions .primary');
    const [c, b] = [card.getBoundingClientRect(), button.getBoundingClientRect()];
    return { scrolls: card.scrollHeight > card.clientHeight, inside: b.top >= c.top && b.bottom <= c.bottom + 0.5 };
  });
  assert.deepEqual(fit, { scrolls: true, inside: true });
  await tab.close();
});
