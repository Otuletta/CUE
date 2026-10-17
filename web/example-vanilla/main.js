// Reference integration, deliberately framework-free. It also collects the
// Phase 1 measurements; `window.__cueMetrics` exposes them to test tooling.

const $ = (id) => document.getElementById(id);
const root = $('cue-root');
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
  const entry = performance
    .getEntriesByType('resource')
    .find((e) => pattern.test(e.name));
  return entry ? `${(entry.transferSize / 1024).toFixed(0)} KiB transferred, ${(entry.decodedBodySize / 1024).toFixed(0)} KiB decoded` : 'n/a';
}

/** Frame rate as the page sees it, sampled over one-second windows. */
function startFpsMeter() {
  let frames = 0;
  let since = performance.now();
  const samples = [];
  const tick = (now) => {
    frames += 1;
    if (now - since >= 1000) {
      const fps = (frames * 1000) / (now - since);
      samples.push(fps);
      record('fps (page rAF, last 1 s)', fps.toFixed(1));
      const sorted = [...samples].sort((a, b) => a - b);
      record('fps p5 over session', sorted[Math.floor(sorted.length * 0.05)].toFixed(1));
      frames = 0;
      since = now;
    }
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

let game = null;

$('play').addEventListener('click', async () => {
  $('play').disabled = true;
  root.dataset.state = 'loading';
  const t0 = performance.now();
  try {
    const { createCue } = await import('./package/index.js');
    game = await createCue({ container: root });
  } catch (err) {
    root.dataset.state = 'error';
    log(`error: ${err.message}`);
    $('play').disabled = false;
    return;
  }
  root.dataset.state = 'running';
  record('time to ready (click → first frame)', `${(performance.now() - t0).toFixed(0)} ms`);
  record('cue_web_bg.wasm', resourceSize(/cue_web_bg\.wasm/));
  record('cue_web.js', resourceSize(/cue_web\.js/));
  record('engine version', game.version);
  log('ready');

  for (const name of ['shot-start', 'shot-end', 'paused', 'resumed', 'destroyed']) {
    game.on(name, () => log(name));
  }
  $('pause').disabled = false;
  $('rerack').disabled = false;
  $('destroy').disabled = false;
  root.querySelector('canvas')?.focus();
  startFpsMeter();
});

$('pause').addEventListener('click', () => {
  const paused = $('pause').textContent === 'Resume';
  if (paused) game.resume();
  else game.pause();
  $('pause').textContent = paused ? 'Pause' : 'Resume';
});

$('rerack').addEventListener('click', () => game.rerack());

$('destroy').addEventListener('click', async () => {
  for (const id of ['pause', 'rerack', 'destroy']) $(id).disabled = true;
  game.destroy();
  await game.destroy(); // idempotent by contract: same promise
  root.dataset.state = 'destroyed';
  log('canvas removed');
});

// A hidden tab should cost nothing.
document.addEventListener('visibilitychange', () => {
  if (!game || root.dataset.state !== 'running') return;
  if (document.hidden) game.pause();
  else if ($('pause').textContent === 'Pause') game.resume();
});
