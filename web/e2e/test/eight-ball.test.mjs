// Eight-ball and free play driven through the built-in controls, the way a
// player does it (spec §24.1, "flujo real de Free Play y 8-Ball"). The
// controls must agree with the engine: the events, the state and what the
// board shows.
//
// Not covered here: winning a rack. That needs a scripted run of the table,
// and the public state does not expose ball positions to aim with. The
// rules engine's tests cover every way a rack ends (docs/rules-eight-ball.md).
import assert from 'node:assert/strict';
import { after, before, describe, test } from 'node:test';

import { harness, nextEvent } from '../helpers.mjs';

const EVENTS = [
  'new-game',
  'shot-start',
  'shot-end',
  'ruling',
  'foul',
  'groups-assigned',
  'turn-changed',
  'decision-required',
  'decided',
  'game-over',
  'match-over',
  'racked',
  'undo',
  'shot-refused',
  'call-refused',
];

let h;
before(async () => {
  h = await harness();
});
after(async () => {
  await h?.close();
});

/** In CUE's shadow root (Playwright's CSS selectors pierce it). */
const inCue = (selector) => `#a .cue-overlay >> ${selector}`;

const state = (page) => page.evaluate(() => window.game.getState());
const sheet = (page) =>
  page.evaluate(() => window.shadow.querySelector('.sheet:not([hidden])')?.dataset.sheet ?? null);
const count = (page, name) => page.evaluate((n) => window.events.filter(([e]) => e === n).length, name);

/** Sets the power slider as a player drags it, 0–100. */
async function setPowerSlider(page, value) {
  await page.evaluate((v) => {
    const input = window.shadow.querySelector('input[name="power"]');
    input.value = String(v);
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }, value);
}

/** Shoots with the Shoot button and waits for the shot, and its ruling in a match, to end. */
async function shootAndSettle(page, power) {
  const before = await count(page, 'shot-end');
  const rulings = await count(page, 'ruling');
  await setPowerSlider(page, power);
  await page.click(inCue('[data-action="shoot"]'));
  try {
    await page.waitForFunction((n) => window.events.filter(([e]) => e === 'shot-end').length > n, before, {
      timeout: 80_000,
    });
  } catch (error) {
    // Say where things stood, which a timeout alone does not.
    const where = await page.evaluate(() => ({
      state: window.game.getState(),
      sheet: window.shadow.querySelector('.sheet:not([hidden])')?.dataset.sheet ?? null,
      events: window.events.slice(-8).map(([e]) => e),
    }));
    throw new Error(`no shot-end: ${JSON.stringify(where)}`, { cause: error });
  }
  const inMatch = (await state(page)).eightBall !== null;
  if (inMatch) {
    await page.waitForFunction((n) => window.events.filter(([e]) => e === 'ruling').length > n, rulings);
  }
  await page.waitForFunction(() => window.game.getState().atRest);
  // The controls redraw on the next animation frame.
  await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
}

/** What the scoreboard shows: names, who is active, the turn line. */
const board = (page) =>
  page.evaluate(() => {
    const el = window.shadow.querySelector('.board');
    const seat = (s) => el.querySelector(`[data-seat="${s}"]`);
    return {
      shown: !el.hidden,
      names: ['a', 'b'].map((s) => seat(s).querySelector('.name').firstChild?.textContent ?? ''),
      active: ['a', 'b'].filter((s) => seat(s).hasAttribute('data-active')),
      turn: el.querySelector('.turn').textContent,
    };
  });

describe('a match played through the controls', () => {
  let tab;
  let page;
  before(async () => {
    tab = await h.open();
    page = tab.page;
    await page.evaluate(async (names) => {
      window.game = await window.createCue({ container: document.querySelector('#a'), persist: false, locale: 'en' });
      window.record(window.game, names);
      window.shadow = document.querySelector('#a .cue-overlay').shadowRoot;
    }, EVENTS);
    await page.keyboard.press('Escape'); // the first-visit dialog
  });
  after(async () => {
    await tab.close();
  });

  test('sets up a match from the game menu', async () => {
    await page.click(inCue('[data-action="game"]'));
    assert.equal(await sheet(page), 'game');
    await page.click(inCue('[data-sheet="game"] [data-action="setup"]'));
    assert.equal(await sheet(page), 'setup');
    await page.fill(inCue('input[name="name1"]'), 'Ana');
    await page.fill(inCue('input[name="name2"]'), 'Bo');
    await page.check(inCue('input[name="ruleset"][value="eight-ball-wpa"]'));
    await page.check(inCue('input[name="raceTo"][value="3"]'));
    await page.check(inCue('input[name="breaker"][value="a"]'));
    await page.click(inCue('[data-form="setup"] button[type="submit"]'));

    assert.deepEqual(await nextEvent(page, 'new-game'), { mode: 'eight-ball' });
    await page.waitForFunction(() => window.game.getState().eightBall?.canShoot);
    const m = (await state(page)).eightBall;
    assert.deepEqual(
      { ruleset: m.ruleset, raceTo: m.raceTo, breaker: m.breaker, stage: m.stage, names: m.players.map((p) => p.name) },
      { ruleset: 'eight-ball-wpa', raceTo: 3, breaker: 'a', stage: 'break', names: ['Ana', 'Bo'] },
    );
    assert.equal(await sheet(page), null);
    // A banner says who breaks; the live region has the full sentence.
    const banner = await page.evaluate(() => {
      const el = window.shadow.querySelector('.banner');
      return { visible: el.hasAttribute('data-visible'), text: el.querySelector('strong').textContent };
    });
    assert.deepEqual(banner, { visible: true, text: 'Ana breaks' });
    await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
    const shown = await board(page);
    assert.deepEqual({ ...shown, turn: undefined }, { shown: true, names: ['Ana', 'Bo'], active: ['a'], turn: undefined });
    assert.match(shown.turn, /Ana/);
  });

  test('breaks with the power slider and the Shoot button', { timeout: 120_000 }, async () => {
    await shootAndSettle(page, 100);
    const start = await nextEvent(page, 'shot-start');
    assert.equal(start.shooter, 'a');
    const ruling = await nextEvent(page, 'ruling');
    assert.equal(ruling.shooter, 'a');
    assert.ok(['continue', 'pass', 'decide', 'over'].includes(ruling.next), ruling.next);
    assert.equal(await count(page, 'shot-start'), 1);
    assert.equal(await count(page, 'shot-end'), 1);
  });

  test('plays on for several turns, answering what each ruling asks', { timeout: 600_000 }, async () => {
    for (let turn = 0; turn < 6; turn++) {
      const s = await state(page);
      const m = s.eightBall;
      if (m.stage === 'over') break;

      if (m.stage === 'decision') {
        // The decision panel offers exactly the engine's options, as buttons.
        const offered = await page.evaluate(() =>
          [...window.shadow.querySelectorAll('[data-panel="decision"] [data-action="decide"]')].map((b) => b.dataset.choice),
        );
        assert.deepEqual(offered, m.decision.options);
        await page.click(inCue(`[data-panel="decision"] [data-choice="${offered[0]}"]`));
        const decided = await nextEvent(page, 'decided');
        assert.equal(decided.choice, offered[0]);
        await page.waitForFunction(() => window.game.getState().eightBall.stage !== 'decision');
        assert.ok(await page.evaluate(() => window.shadow.querySelector('[data-panel="decision"]').hidden));
        continue;
      }

      // The board marks the player whose turn it is.
      const shown = await board(page);
      assert.deepEqual(shown.active, [m.shooter], `board ${shown.active}, engine ${m.shooter}`);
      assert.ok(m.canShoot, `cannot shoot: ${m.refusal}`);
      await shootAndSettle(page, 45);
    }

    // Every pass of the turn names both players, and follows a ruling that
    // said so.
    const events = await page.evaluate(() => window.events);
    let last = null;
    for (const [name, detail] of events) {
      if (name === 'ruling') last = detail;
      if (name === 'turn-changed') {
        assert.notEqual(detail.shooter, detail.previous);
        assert.equal(detail.previous, last?.shooter, 'the outgoing player is the one who shot');
      }
    }
    assert.equal(await count(page, 'shot-start'), await count(page, 'shot-end'));
    assert.equal(await count(page, 'shot-refused'), 0);
  });

  test('calls a ball and a pocket with the picker, then hands the call back to the aim', async (t) => {
    const m = (await state(page)).eightBall;
    if (m.stage !== 'play') {
      t.skip(`the rack is in stage ${m.stage}`);
      return;
    }
    await page.click(inCue('[data-action="call"]'));
    assert.equal(await sheet(page), 'call');
    const ball = await page.evaluate(() => window.shadow.querySelector('[data-sheet="call"] input[name="ball"]:not(:disabled)').value);
    await page.check(inCue(`[data-sheet="call"] input[name="ball"][value="${ball}"]`));
    const pocket = await page.evaluate(() => window.shadow.querySelector('[data-sheet="call"] input[name="pocket"]').value);
    await page.check(inCue(`[data-sheet="call"] input[name="pocket"][value="${pocket}"]`));
    await page.click(inCue('[data-form="call"] button[type="submit"]'));
    await page.waitForFunction(() => window.game.getState().eightBall.manualCall);
    const called = (await state(page)).eightBall.call;
    assert.deepEqual(called, { kind: 'ball', ball: Number(ball), pocket });

    await page.click(inCue('[data-action="call"]'));
    await page.click(inCue('[data-sheet="call"] [data-action="callAuto"]'));
    await page.waitForFunction(() => !window.game.getState().eightBall.manualCall);
    assert.equal(await count(page, 'call-refused'), 0);
  });

  test('agreeing to a stalemate replays the rack', async (t) => {
    const m = (await state(page)).eightBall;
    if (m.stage !== 'play') {
      t.skip(`the rack is in stage ${m.stage}`);
      return;
    }
    const racked = await count(page, 'racked');
    await page.click(inCue('[data-action="game"]'));
    await page.click(inCue('[data-sheet="game"] [data-action="stalemate"]'));
    assert.equal(await sheet(page), 'stalemate');
    await page.click(inCue('[data-action="confirmStalemate"]'));
    await page.waitForFunction((n) => window.events.filter(([e]) => e === 'racked').length > n, racked);
    await page.waitForFunction(() => window.game.getState().eightBall.stage === 'break');
    assert.equal((await state(page)).eightBall.rack, m.rack, 'a replay, not a new rack');
  });

  test('leaves for free play, where Undo and New rack work from the controls', { timeout: 120_000 }, async () => {
    await page.click(inCue('[data-action="game"]'));
    await page.click(inCue('[data-sheet="game"] [data-action="freePlay"]'));
    await page.waitForFunction(() => window.game.getState().mode === 'free-play');
    assert.ok(await page.evaluate(() => window.shadow.querySelector('.board').hidden), 'the scoreboard goes');

    const undoDisabled = () => page.evaluate(() => window.shadow.querySelector('[data-action="undo"]').disabled);
    await shootAndSettle(page, 40);
    await page.waitForFunction(() => !window.shadow.querySelector('[data-action="undo"]').disabled);
    await page.click(inCue('[data-action="undo"]'));
    await nextEvent(page, 'undo');
    await page.waitForFunction(() => window.shadow.querySelector('[data-action="undo"]').disabled);
    assert.ok(await undoDisabled());

    const racked = await count(page, 'racked');
    await page.click(inCue('[data-action="game"]'));
    await page.click(inCue('[data-sheet="game"] [data-action="rerackNow"]'));
    await page.waitForFunction((n) => window.events.filter(([e]) => e === 'racked').length > n, racked);
    assert.equal(await count(page, 'shot-refused'), 0);
  });
});
