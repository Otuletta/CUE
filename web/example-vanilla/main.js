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
