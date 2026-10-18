// Eight-ball for two players on one device: the scoreboard, the call, the
// decisions after a break and the end of a rack (spec §8.2).
//
// The engine is the referee; this module only shows what it decided. Every
// frame `update(state)` receives the engine snapshot and redraws a part of
// the interface only when the data behind it changed.

import { formatList } from './i18n.js';

/** Pockets in reading order on screen. The camera shows the table
 * lengthwise in landscape and rotated a quarter turn in portrait; the
 * stylesheet places each pocket accordingly. */
const POCKET_ORDER = {
  landscape: ['head-far', 'side-far', 'foot-far', 'head-near', 'side-near', 'foot-near'],
  portrait: ['foot-far', 'foot-near', 'side-far', 'side-near', 'head-far', 'head-near'],
};

const SEATS = ['a', 'b'];
/** Colour classes for a ball: its hue, and whether it is striped. */
const ballClass = (n) => (n === 8 ? 'eight' : `c${n > 8 ? n - 8 : n}${n > 8 ? ' stripe' : ''}`);
const other = (seat) => (seat === 'a' ? 'b' : 'a');
/** Each group's balls, then the eight it must finish on. */
const GROUP_BALLS = { solids: [1, 2, 3, 4, 5, 6, 7, 8], stripes: [9, 10, 11, 12, 13, 14, 15, 8] };

const escape = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

/** A small ball swatch: solid, striped or the black eight. */
function ballMark(n) {
  return `<span class="mark ${ballClass(n)}" aria-hidden="true"></span>`;
}

/** A tiny table with one pocket lit, turned the way the table is shown. */
function pocketIcon(pocket, portrait) {
  const order = POCKET_ORDER[portrait ? 'portrait' : 'landscape'];
  const [w, h] = portrait ? [18, 30] : [30, 18];
  const spots = portrait
    ? [[3, 3], [15, 3], [3, 15], [15, 15], [3, 27], [15, 27]]
    : [[3, 3], [15, 3], [27, 3], [3, 15], [15, 15], [27, 15]];
  const dots = order
    .map((id, i) => {
      const [x, y] = spots[i];
      return id === pocket ? `<circle class="lit" cx="${x}" cy="${y}" r="2.8"/>` : `<circle cx="${x}" cy="${y}" r="1.4"/>`;
    })
    .join('');
  return `<svg class="pocket-icon" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" aria-hidden="true" focusable="false"><rect x="3" y="3" width="${w - 6}" height="${h - 6}" rx="1.5"/>${dots}</svg>`;
}

/** Markup that lives in the top bar. */
export function boardMarkup(t) {
  const player = (seat) => `
      <div class="player" data-seat="${seat}">
        <span class="name"></span><span class="group"></span><span class="score"></span>
      </div>`;
  return `
    <section class="board" aria-label="${escape(t.scoreboard)}" hidden>
      <div class="players">${SEATS.map(player).join('')}</div>
      <p class="line"><span class="rack"></span><span class="turn"></span></p>
    </section>`;
}

/** Markup that lives in the bottom dock, before the spin control. */
export function dockMarkup(t) {
  return `
    <button type="button" class="call" data-action="call" aria-haspopup="dialog" hidden>
      <span class="call-label" aria-hidden="true">${escape(t.callLabel)}</span><span class="call-body" aria-hidden="true"></span><span class="tag" aria-hidden="true"></span>
    </button>
    <button type="button" class="chip" data-action="spot" title="${escape(t.spotNote)}" hidden>${escape(t.spotBall)}</button>`;
}

/** Panels that replace the dock while a decision or a result is shown, and
 * the eight-ball dialogs. */
export function sheetsMarkup(t, { rulesUrl }) {
  const radios = (name, entries) =>
    entries
      .map(
        ([value, label], i) =>
          `<label><input type="radio" name="${name}" value="${escape(value)}"${i === 0 ? ' checked' : ''}><span data-label="${escape(value)}">${escape(label)}</span></label>`,
      )
      .join('');
  return `
  <div class="panels">
    <section class="panel" data-panel="decision" role="dialog" aria-modal="false" aria-labelledby="decision-title" aria-describedby="decision-body" hidden>
      <h2 id="decision-title"></h2>
      <p id="decision-body"></p>
      <div class="options"></div>
    </section>
    <section class="panel" data-panel="over" role="dialog" aria-modal="false" aria-labelledby="over-title" aria-describedby="over-body" hidden>
      <h2 id="over-title"></h2>
      <p id="over-body"></p>
      <p class="tally"></p>
      <div class="options"></div>
    </section>
  </div>

  <div class="sheet" data-sheet="game" role="dialog" aria-modal="true" aria-labelledby="game-title" hidden>
    <div class="card">
      <h2 id="game-title">${escape(t.gameTitle)}</h2>
      <div class="menu"></div>
      <div class="actions"><button type="button" data-action="close">${escape(t.cancel)}</button></div>
    </div>
  </div>

  <div class="sheet" data-sheet="stalemate" role="dialog" aria-modal="true" aria-labelledby="stalemate-title" hidden>
    <div class="card">
      <h2 id="stalemate-title">${escape(t.stalemateTitle)}</h2>
      <p>${escape(t.stalemateBody)}</p>
      <div class="actions">
        <button type="button" data-action="close" autofocus>${escape(t.cancel)}</button>
        <button type="button" class="primary" data-action="confirmStalemate">${escape(t.stalemateConfirm)}</button>
      </div>
    </div>
  </div>

  <div class="sheet" data-sheet="setup" role="dialog" aria-modal="true" aria-labelledby="setup-title" hidden>
    <form class="card" data-form="setup" novalidate>
      <h2 id="setup-title" tabindex="-1" autofocus>${escape(t.setupTitle)}</h2>
      <div class="names">
        ${[1, 2]
          .map(
            (n) =>
              `<label><span>${escape(t.player(n))}</span><input type="text" name="name${n}" maxlength="24" autocomplete="off" autocapitalize="words" spellcheck="false" enterkeyhint="next" placeholder="${escape(t.player(n))}" aria-label="${escape(t.playerName(n))}"></label>`,
          )
          .join('')}
      </div>
      <p class="note">${escape(t.namesNote)}</p>
      <fieldset><legend>${escape(t.rules)}</legend>
        <div class="choices">${radios('ruleset', Object.entries(t.rulesets))}</div>
        <p class="note" data-note="ruleset"></p>
        ${rulesUrl ? `<p class="note"><a href="${escape(rulesUrl)}" target="_blank" rel="noopener">${escape(t.rulesLink)}</a></p>` : ''}
      </fieldset>
      <fieldset><legend>${escape(t.match)}</legend>
        <div class="choices">${radios('raceTo', [1, 3, 5].map((n) => [String(n), t.raceTo(n)]))}</div>
      </fieldset>
      <fieldset><legend>${escape(t.whoBreaks)}</legend>
        <div class="choices">${radios('breaker', [
          ['random', t.randomBreaker],
          ['a', t.player(1)],
          ['b', t.player(2)],
        ])}</div>
      </fieldset>
      <div class="actions">
        <button type="button" data-action="close">${escape(t.cancel)}</button>
        <button type="submit" class="primary">${escape(t.start)}</button>
      </div>
    </form>
  </div>

  <div class="sheet" data-sheet="call" role="dialog" aria-modal="true" aria-labelledby="call-title" hidden>
    <form class="card wide" data-form="call" novalidate>
      <h2 id="call-title">${escape(t.callTitle)}</h2>
      <fieldset class="call-balls"><legend>${escape(t.callBall)}</legend><div class="balls"></div></fieldset>
      <fieldset class="call-pockets"><legend>${escape(t.callPocket)}</legend><div class="pockets"></div></fieldset>
      <div class="actions">
        <button type="button" data-action="callSafety">${escape(t.safety)}</button>
        <button type="button" data-action="callAuto">${escape(t.callAuto)}</button>
        <button type="submit" class="primary" disabled>${escape(t.callIt)}</button>
      </div>
    </form>
  </div>`;
}

/**
 * @param {ShadowRoot} shadow
 * @param {object} deps  `t`, `locale`, `api`, `root`, and the overlay's
 *   `open`, `close`, `openSheet`, `announce` and `focusTable`.
 */
export function createMatch(shadow, deps) {
  const { t, locale, api, root, open, close, announce, banner, celebrate } = deps;
  const $ = (selector) => shadow.querySelector(selector);
  const ui = $('.ui');
  const board = $('.board');
  const callBtn = $('[data-action="call"]');
  const spotBtn = $('[data-action="spot"]');
  const decisionPanel = $('[data-panel="decision"]');
  const overPanel = $('[data-panel="over"]');
  const setupForm = $('[data-form="setup"]');
  const callForm = $('[data-form="call"]');

  let state = null;
  let portrait = false;
  // Rendering keys: a part is redrawn only when its key changes.
  const keys = {};
  const changed = (name, value) => {
    const key = JSON.stringify(value);
    if (keys[name] === key) return false;
    keys[name] = key;
    return true;
  };
  // Set up for the next match: names stay for the session only.
  let lastSetup = { names: ['', ''], ruleset: 'eight-ball-wpa', raceTo: '1', breaker: 'random' };
  let skipSpotted = false;

  const resize = new ResizeObserver(() => {
    const next = root.clientWidth < root.clientHeight;
    if (next === portrait) return;
    portrait = next;
    // Pocket names depend on how the table is shown.
    delete keys.call;
    if (state) render(state);
  });
  resize.observe(root);

  const match = () => state?.eightBall ?? null;
  const nameOf = (m, seat) => m.players[SEATS.indexOf(seat)]?.name || t.player(SEATS.indexOf(seat) + 1);
  const pocketName = (pocket) => t.pockets[portrait ? 'portrait' : 'landscape'][pocket] ?? pocket;
  const needsCall = (m) => m.ruleset === 'eight-ball-wpa' || (m.onTheEight && !m.openTable);

  // --- Scoreboard ------------------------------------------------------------

  function turnText(m, s) {
    const name = nameOf(m, m.shooter);
    if (!s.atRest) return t.rolling;
    switch (m.stage) {
      case 'break':
        return t.breaks(name);
      case 'decision':
        return t.decides(nameOf(m, m.decision.chooser));
      case 'over': {
        const winner = nameOf(m, m.outcome.winner);
        return m.matchOver ? t.winsMatch(winner) : t.winsRack(winner);
      }
      default:
        if (s.handRegion === 'above-head-string') return t.inHandKitchen(name);
        if (s.handRegion === 'anywhere') return t.inHandAnywhere(name);
        if (m.onTheEight && !m.openTable) return t.onTheEight(name);
        return m.openTable ? `${t.toShoot(name)} · ${t.openTable}` : t.toShoot(name);
    }
  }

  function renderBoard(m, s) {
    m.players.forEach((p, i) => {
      const el = board.querySelector(`[data-seat="${SEATS[i]}"]`);
      const active = m.shooter === p.seat && m.stage !== 'over';
      el.toggleAttribute('data-active', active);
      el.querySelector('.name').textContent = nameOf(m, p.seat);
      // The player's balls, built once when the group is set; each one
      // dims, with a little pop, as it goes down.
      const group = el.querySelector('.group');
      if (group.dataset.group !== (p.group ?? '')) {
        group.dataset.group = p.group ?? '';
        const marks = (GROUP_BALLS[p.group] ?? [])
          .map((n) => `<span class="mark mini ${ballClass(n)}" data-ball="${n}"></span>`)
          .join('');
        group.innerHTML = `${marks ? `<span class="rack-balls" aria-hidden="true">${marks}</span>` : ''}<span class="sr"></span>`;
      }
      for (const mark of group.querySelectorAll('[data-ball]')) {
        mark.toggleAttribute('data-gone', !s.ballsInPlay.includes(Number(mark.dataset.ball)));
      }
      group.querySelector('.sr').textContent = p.group ? t.groupLeft(t.groups[p.group], p.remaining) : t.noGroup;
      const score = el.querySelector('.score');
      score.hidden = m.raceTo === 1;
      score.innerHTML = `<span aria-hidden="true">${p.score}</span><span class="sr">${escape(t.racksWon(p.score))}</span>`;
      const tail = active ? `<span class="sr">, ${escape(t.shooting)}</span>` : '';
      el.querySelector('.name').insertAdjacentHTML('beforeend', tail);
    });
    board.querySelector('.rack').textContent = t.rackOf(m.rack, m.raceTo);
    board.querySelector('.turn').textContent = turnText(m, s);
  }

  // --- The call ----------------------------------------------------------------

  function renderCall(m, s) {
    const show = m.stage === 'play' && s.atRest && (needsCall(m) || m.call.kind !== 'none');
    callBtn.hidden = !show;
    if (!show) return;
    const { call } = m;
    let text;
    let body;
    if (call.kind === 'ball') {
      text = t.calledShot(call.ball, pocketName(call.pocket));
      body = `${ballMark(call.ball)}<b>${call.ball}</b><span class="arrow">→</span>${pocketIcon(call.pocket, portrait)}`;
    } else {
      text = call.kind === 'safety' ? t.safety : '—';
      body = `<span>${escape(text)}</span>`;
    }
    callBtn.querySelector('.call-body').innerHTML = body;
    const auto = !m.manualCall && call.kind !== 'none';
    callBtn.querySelector('.tag').textContent = auto ? t.autoTag : '';
    callBtn.title = `${t.callLabel}: ${text}`;
    callBtn.setAttribute('aria-label', `${t.changeCall}. ${t.callLabel}: ${text}${auto ? ` (${t.autoTag})` : ''}`);
  }

  function renderCallSheet(m) {
    const current = m.call.kind === 'ball' ? m.call : null;
    const balls = m.legalTargets.filter((n) => n !== 0);
    callForm.querySelector('.balls').innerHTML = balls
      .map(
        (n) =>
          `<label class="ballchip"><input type="radio" name="ball" value="${n}"${current?.ball === n ? ' checked' : ''}><span class="ball ${ballClass(n)}"><b>${n}</b></span></label>`,
      )
      .join('');
    const pockets = callForm.querySelector('.pockets');
    pockets.toggleAttribute('data-portrait', portrait);
    pockets.innerHTML = POCKET_ORDER[portrait ? 'portrait' : 'landscape']
      .map(
        (id) =>
          `<label class="pocket" data-pocket="${id}"><input type="radio" name="pocket" value="${id}"${current?.pocket === id ? ' checked' : ''}><span class="hole"></span><span class="sr">${escape(pocketName(id))}</span></label>`,
      )
      .join('');
    syncCallForm();
  }

  function syncCallForm() {
    const data = new FormData(callForm);
    callForm.querySelector('[type="submit"]').disabled = !(data.get('ball') && data.get('pocket'));
  }

  callForm.addEventListener('change', syncCallForm);
  callForm.addEventListener('submit', (e) => {
    e.preventDefault();
    const data = new FormData(callForm);
    const ball = Number(data.get('ball'));
    const pocket = data.get('pocket');
    if (!ball || !pocket) return;
    close();
    api.callBall(ball, String(pocket));
  });

  // --- Panels: decisions and results ------------------------------------------

  function renderDecision(m) {
    const d = m.decision;
    const chooser = nameOf(m, d.chooser);
    const breaker = nameOf(m, m.breaker);
    decisionPanel.querySelector('h2').textContent = t.decisionTitles[d.situation] ?? d.situation;
    decisionPanel.querySelector('p').textContent = t.decisionBodies[d.situation]?.(breaker, chooser) ?? '';
    decisionPanel.querySelector('.options').innerHTML = d.options
      .map((choice) => {
        const label = t.choices[choice]?.({ chooser, breaker, cueBallOff: d.cueBallOff }) ?? choice;
        return `<button type="button" class="option" data-action="decide" data-choice="${escape(choice)}">${escape(label)}</button>`;
      })
      .join('');
  }

  function renderOver(m) {
    const o = m.outcome;
    const winner = nameOf(m, o.winner);
    const loser = nameOf(m, other(o.winner));
    overPanel.querySelector('h2').textContent = m.matchOver ? t.matchWon(winner) : t.rackWon(winner);
    overPanel.querySelector('p').textContent = o.loss ? t.lossReasons[o.loss]?.(loser) ?? '' : t.legalWin(winner);
    const [a, b] = m.players;
    const tally = overPanel.querySelector('.tally');
    tally.hidden = m.raceTo === 1;
    tally.textContent = t.tally(nameOf(m, 'a'), a.score, b.score, nameOf(m, 'b'));
    overPanel.querySelector('.options').innerHTML = m.matchOver
      ? `<button type="button" class="option primary" data-action="setup">${escape(t.newMatch)}</button>
         <button type="button" class="option" data-action="freePlay">${escape(t.freePlayOption)}</button>`
      : `<button type="button" class="option primary" data-action="nextRack">${escape(t.nextRack)}</button>`;
  }

  function renderPanels(m, s) {
    const show = s.atRest ? (m.stage === 'decision' ? 'decision' : m.stage === 'over' ? 'over' : null) : null;
    const was = ui.dataset.showing ?? null;
    if (show === 'decision') renderDecision(m);
    if (show === 'over') renderOver(m);
    decisionPanel.hidden = show !== 'decision';
    overPanel.hidden = show !== 'over';
    if (show) ui.dataset.showing = show;
    else delete ui.dataset.showing;
    const panel = show === 'decision' ? decisionPanel : show === 'over' ? overPanel : null;
    if (panel && show !== was && !deps.openSheet()) {
      // Bring keyboard and screen-reader users to the choice.
      panel.querySelector('button')?.focus({ preventScroll: true });
    } else if (!panel && was && shadow.activeElement?.closest?.('.panel')) {
      deps.focusTable();
    }
  }

  // --- Frame update ------------------------------------------------------------

  function render(s) {
    const m = s.eightBall;
    if (changed('mode', s.mode)) {
      ui.dataset.mode = s.mode;
      board.hidden = !m;
    }
    if (!m) {
      callBtn.hidden = true;
      spotBtn.hidden = true;
      if (changed('panels', null)) renderPanels({ stage: 'none' }, { atRest: false });
      return;
    }
    if (
      changed('board', [
        m.players,
        m.shooter,
        m.stage,
        m.rack,
        m.raceTo,
        m.decision?.chooser,
        m.outcome,
        m.matchOver,
        m.onTheEight,
        m.openTable,
        s.atRest,
        s.handRegion,
      ])
    ) {
      renderBoard(m, s);
    }
    if (changed('call', [m.stage, m.call, m.manualCall, m.ruleset, m.onTheEight, m.openTable, s.atRest])) {
      renderCall(m, s);
    }
    if (changed('spot', m.canSpot && s.atRest)) spotBtn.hidden = !(m.canSpot && s.atRest);
    if (changed('panels', [m.stage, m.decision, m.outcome, m.matchOver, m.players, s.atRest])) {
      renderPanels(m, s);
    }
  }

  // --- Announcements -------------------------------------------------------------

  /** Shows the banner a ruling deserves, a foul, a change of turn or the
   * win, and says whether there was one. */
  function bannerFor(r, m) {
    const next = nameOf(m, other(r.shooter));
    if (r.next === 'over' && m.outcome) {
      const winner = nameOf(m, m.outcome.winner);
      banner('win', m.matchOver ? t.bannerMatch(winner) : t.bannerWin(winner));
      celebrate();
    } else if (r.fouls.length) {
      const list = r.fouls.map((f) => t.fouls[f]?.(r.wrongBall) ?? f);
      // What the foul was, and who has the cue ball now.
      const why = formatFouls(list);
      banner('foul', t.bannerFoul, r.next === 'pass' ? `${why} · ${t.bannerInHand(next)}` : why);
    } else if (r.next === 'pass') {
      banner('turn', t.bannerTurn(next));
    } else {
      return false;
    }
    return true;
  }

  function describeRuling(r, m) {
    const shooter = nameOf(m, r.shooter);
    const next = nameOf(m, other(r.shooter));
    const parts = [];
    if (r.fouls.length) {
      const list = r.fouls.map((f) => t.fouls[f]?.(r.wrongBall) ?? f);
      parts.push(t.foulBy(shooter, formatFouls(list)));
    }
    switch (r.next) {
      case 'continue':
        parts.push(r.pocketed.length ? t.keepsShooting(shooter, t.balls(r.pocketed)) : t.nowShoots(shooter));
        break;
      case 'pass':
        if (r.fouls.length) {
          parts.push(r.inHand === 'above-head-string' ? t.hasBallInHandKitchen(next) : t.hasBallInHand(next));
        } else {
          if (r.call.kind === 'safety') parts.push(t.safetyPlayed(shooter));
          else if (r.pocketed.length && r.call.kind === 'ball') parts.push(t.notAsCalled(t.balls(r.pocketed)));
          else if (r.pocketed.length) parts.push(`${t.pocketed(t.balls(r.pocketed))}.`);
          parts.push(t.nowShoots(next));
        }
        break;
      case 'decide':
        if (m.decision) {
          parts.push(`${t.decisionTitles[m.decision.situation]}. ${t.decides(nameOf(m, m.decision.chooser))}.`);
        }
        break;
      case 'over':
        if (m.outcome) {
          const winner = nameOf(m, m.outcome.winner);
          const loser = nameOf(m, other(m.outcome.winner));
          parts.push(m.outcome.loss ? t.lossReasons[m.outcome.loss](loser) : t.legalWin(winner));
          parts.push(m.matchOver ? t.matchWon(winner) : t.rackWon(winner));
        }
        break;
      default:
    }
    if (r.assigned) parts.push(t.takesGroup(nameOf(m, r.assigned.seat), t.groups[r.assigned.group]));
    return parts.join(' ');
  }

  const formatFouls = (list) => formatList(list, locale);

  function turnAnnouncement(m, seat, inHand) {
    const name = nameOf(m, seat);
    if (inHand === 'above-head-string') return t.inHandKitchen(name);
    if (inHand === 'anywhere') return t.inHandAnywhere(name);
    return t.toShoot(name);
  }

  /** Returns `true` when it handled the event, so the overlay stays quiet. */
  function onEvent(name, detail) {
    const s = api.state();
    const m = s?.eightBall;
    switch (name) {
      case 'new-game':
        if (detail?.mode === 'eight-ball' && m) {
          // The banner shows it; the live region still says it in full.
          announce(t.matchStarts(nameOf(m, 'a'), nameOf(m, 'b'), nameOf(m, m.breaker)), { visual: false });
          banner('turn', t.bannerBreak(nameOf(m, m.breaker)));
        } else {
          announce(t.freePlay);
        }
        return true;
      case 'ruling':
        if (m) announce(describeRuling(detail, m), { visual: !bannerFor(detail, m) });
        return true;
      case 'decided':
        if (!m || detail.stage === 'break') return true; // `racked` follows.
        if (detail.choice.startsWith('spot-eight')) {
          skipSpotted = true;
          announce(`${t.spotted(8)} ${turnAnnouncement(m, detail.shooter, detail.inHand)}`);
        } else {
          announce(turnAnnouncement(m, detail.shooter, detail.inHand));
        }
        return true;
      case 'ball-spotted':
        if (skipSpotted) skipSpotted = false;
        else announce(t.spotted(detail.ballId));
        return true;
      case 'racked':
        if (!m) return false;
        announce(m.raceTo > 1 ? t.newRackBreaks(m.rack, nameOf(m, m.breaker)) : t.rerackBreaks(nameOf(m, m.breaker)), {
          visual: false,
        });
        banner('turn', t.bannerBreak(nameOf(m, m.breaker)));
        return true;
      case 'shot-refused':
      case 'call-refused':
        announce(t.refusals[detail?.reason] ?? t.refused);
        return true;
      case 'shot-end':
        // In a match the ruling says what happened.
        return Boolean(m);
      default:
        return false;
    }
  }

  // --- Dialogs -----------------------------------------------------------------

  function openGameMenu() {
    const s = api.state();
    const m = s?.eightBall;
    const option = (action, title, note) =>
      `<button type="button" class="option" data-action="${action}"><strong>${escape(title)}</strong>${note ? `<span>${escape(note)}</span>` : ''}</button>`;
    $('[data-sheet="game"] .menu').innerHTML = m
      ? [
          option('setup', t.newMatchOption, t.newMatchNote),
          option('freePlay', t.freePlayOption),
          m.stage === 'play' && s.atRest ? option('stalemate', t.stalemateOption, t.stalemateNote) : '',
          `<p class="note">${escape(t.leaveNote)}</p>`,
        ].join('')
      : [option('rerackNow', t.rerackOption, t.rerackNote), option('setup', t.eightBallOption, t.eightBallNote)].join('');
    open('game');
  }

  function openSetup() {
    const form = setupForm;
    form.elements.name1.value = lastSetup.names[0];
    form.elements.name2.value = lastSetup.names[1];
    for (const name of ['ruleset', 'raceTo', 'breaker']) {
      for (const input of form.querySelectorAll(`input[name="${name}"]`)) input.checked = input.value === lastSetup[name];
    }
    syncSetup();
    open('setup');
  }

  function syncSetup() {
    const form = setupForm;
    const ruleset = new FormData(form).get('ruleset');
    form.querySelector('[data-note="ruleset"]').textContent = t.rulesNotes[ruleset] ?? '';
    // The breaker choices follow the names as they are typed.
    [1, 2].forEach((n) => {
      const typed = form.elements[`name${n}`].value.trim();
      form.querySelector(`[data-label="${n === 1 ? 'a' : 'b'}"]`).textContent = typed || t.player(n);
    });
  }

  setupForm.addEventListener('input', syncSetup);
  setupForm.addEventListener('change', syncSetup);
  setupForm.addEventListener('submit', (e) => {
    e.preventDefault();
    const data = new FormData(setupForm);
    const names = [String(data.get('name1') ?? '').trim(), String(data.get('name2') ?? '').trim()];
    lastSetup = {
      names,
      ruleset: String(data.get('ruleset')),
      raceTo: String(data.get('raceTo')),
      breaker: String(data.get('breaker')),
    };
    close();
    api.newGame({
      ruleset: lastSetup.ruleset,
      breaker: lastSetup.breaker,
      raceTo: Number(lastSetup.raceTo),
      names: [names[0] || t.player(1), names[1] || t.player(2)],
    });
  });

  const actions = {
    game: openGameMenu,
    rerackNow: () => {
      close();
      api.rerack();
    },
    setup: () => openSetup(),
    freePlay: () => {
      close();
      api.freePlay();
    },
    stalemate: () => open('stalemate'),
    confirmStalemate: () => {
      close();
      api.stalemate();
    },
    call: () => {
      const m = match();
      if (!m) return;
      renderCallSheet(m);
      open('call');
    },
    callSafety: () => {
      close();
      api.callSafety();
    },
    callAuto: () => {
      close();
      api.autoCall();
    },
    spot: () => api.spotRequest(),
    decide: (el) => {
      const choice = el?.dataset.choice;
      if (choice) api.decide(choice);
    },
    nextRack: () => api.nextRack(),
  };

  return {
    actions,
    update(next) {
      state = next;
      render(next);
    },
    onEvent,
    destroy() {
      resize.disconnect();
    },
  };
}
