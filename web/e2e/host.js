// The host page the browser tests drive: it loads the built package the way
// a site would and exposes a little to the tests.
import { createCue } from '/package/index.js';

window.createCue = createCue;

// Every Content Security Policy violation, for the tests to assert none.
window.violations = [];
document.addEventListener('securitypolicyviolation', (e) => {
  window.violations.push(`${e.violatedDirective}: ${e.blockedURI || e.sample}`);
});

// Events in arrival order, as [name, detail] pairs.
window.events = [];
window.record = (game, names) => {
  for (const name of names) game.on(name, (detail) => window.events.push([name, detail ?? null]));
};

// The host's own control: it must keep working whatever CUE does.
window.hostClicks = 0;
document.querySelector('#host-button').addEventListener('click', () => window.hostClicks++);

window.hostReady = true;
