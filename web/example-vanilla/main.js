// Reference integration, deliberately framework-free. It also collects
// measurements; `window.__cueMetrics` exposes them to test tooling.

const $ = (id) => document.getElementById(id);
const stage = $('cue-root');
const play = $('play');
const metrics = {};
window.__cueMetrics = metrics;

function record(name, value) {
  metrics[name] = value;
  let row = document.querySelector(`[data-metric="${name}"]`);
  if (!row) {
    row = document.createElement('div');
    row.dataset.metric = name;
    row.append(document.createElement('dt'), document.createElement('dd'));
    row.firstChild.textContent = name;
    $('metrics').append(row);
  }
  row.lastChild.textContent = String(value);
}

function log(text) {
  const item = document.createElement('li');
  item.textContent = `${(performance.now() / 1000).toFixed(2)} s  ${text}`;
  $('log').prepend(item);
}

function resourceSize(pattern) {
  const entry = performance.getEntriesByType('resource').find((e) => pattern.test(e.name));
  return entry
    ? `${(entry.transferSize / 1024).toFixed(0)} KiB transferred, ${(entry.decodedBodySize / 1024).toFixed(0)} KiB decoded`
    : 'n/a';
}

let game = null;

async function start() {
  play.disabled = true;
  stage.dataset.state = 'loading';
  const t0 = performance.now();
  try {
    const { createCue } = await import('./package/index.js');
    game = await createCue({
      container: stage,
      fallbackUrl: 'https://github.com/Otuletta/CUE',
      rulesUrl: 'https://github.com/Otuletta/CUE/blob/main/docs/rules-eight-ball.md',
      // An SPA would route away here. ADR-0006: pause and hide instead of
      // destroying, so coming back is instant.
      onExit: () => {
        game.pause();
        stage.dataset.state = 'exited';
        play.textContent = 'Back to the table';
        play.disabled = false;
        play.focus();
        log('exit (paused)');
      },
    });
  } catch (err) {
    stage.dataset.state = 'error';
    log(`error: ${err.message}`);
    return;
  }
  stage.dataset.state = 'running';
  record('time to ready (click → first frame)', `${(performance.now() - t0).toFixed(0)} ms`);
  record('cue_web_bg.wasm', resourceSize(/cue_web_bg\.wasm/));
  record('cue_web.js', resourceSize(/cue_web\.js/));
  record('engine version', game.version);
  log('ready');

  for (const name of ['shot-start', 'paused', 'resumed', 'undo', 'destroyed']) {
    game.on(name, () => log(name));
  }
  game.on('racked', ({ cleared }) => log(`racked${cleared ? ' (table cleared)' : ''}`));
  game.on('ball-pocketed', ({ ballId, pocketId }) =>
    log(`ball-pocketed ${ballId === 0 ? 'cue ball' : ballId} → ${pocketId}`),
  );
  game.on('shot-end', (shot) => {
    if (shot?.abandoned) return log('shot-end (abandoned)');
    const first = shot.firstContacts.join('+') || 'none';
    log(`shot-end #${shot.shotId}: first contact ${first}, ${shot.pocketed.length} pocketed`);
  });
  game.on('settings-change', (s) =>
    log(`settings ${s.theme} · ${s.assists} · sound ${s.sound.enabled ? 'on' : 'off'}`),
  );
  // Eight-ball: the referee's side of the API.
  game.on('new-game', ({ mode }) => log(`new-game ${mode}`));
  game.on('ruling', (r) => {
    const call = r.call.kind === 'ball' ? `${r.call.ball}→${r.call.pocket}` : r.call.kind;
    const fouls = r.fouls.length ? ` fouls [${r.fouls.join(', ')}]` : '';
    log(`ruling ${r.shooter}: call ${call}, pocketed [${r.pocketed.join(', ')}]${fouls} → ${r.next}`);
  });
  game.on('groups-assigned', ({ seat, group }) => log(`groups-assigned ${seat} ${group}`));
  game.on('decision-required', (d) => log(`decision-required ${d.situation}: ${d.chooser} [${d.options.join(', ')}]`));
  game.on('decided', (d) => log(`decided ${d.choice} → ${d.shooter} (${d.inHand})`));
  game.on('rack-over', (o) => log(`rack-over ${o.winner}${o.loss ? ` (${o.loss})` : ''} ${o.score.join('–')}`));
  game.on('match-over', (o) => log(`match-over ${o.winner} ${o.score.join('–')}`));
  game.on('ball-spotted', ({ ballId }) => log(`ball-spotted ${ballId}`));
  game.on('shot-refused', ({ reason }) => log(`shot-refused ${reason}`));
  game.on('call-refused', ({ reason }) => log(`call-refused ${reason}`));
  $('destroy').disabled = false;
  stage.querySelector('canvas')?.focus();
}

play.addEventListener('click', () => {
  if (stage.dataset.state === 'exited') {
    stage.dataset.state = 'running';
    game.resume();
    stage.querySelector('canvas')?.focus();
    return;
  }
  start();
});

$('destroy').addEventListener('click', async () => {
  $('destroy').disabled = true;
  game.destroy();
  await game.destroy(); // idempotent by contract: same promise
  stage.dataset.state = 'destroyed';
  log('instance destroyed; reload the page to play again');
});

// A hidden tab should cost nothing.
document.addEventListener('visibilitychange', () => {
  if (!game || stage.dataset.state !== 'running') return;
  if (document.hidden) game.pause();
  else game.resume();
});
