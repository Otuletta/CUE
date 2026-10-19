// The host page for the <cue-game> tests: one tag, the built package's
// element module, and a record of the element's DOM events.
import '/package/element.js';

window.violations = [];
document.addEventListener('securitypolicyviolation', (e) => {
  window.violations.push(`${e.violatedDirective}: ${e.blockedURI || e.sample}`);
});

// `cue-*` events in arrival order, as [name, detail, element id] pairs.
window.events = [];
for (const name of ['ready', 'paused', 'resumed', 'settings-change', 'new-game', 'error']) {
  document.addEventListener(`cue-${name}`, (e) =>
    window.events.push([name, e.detail ?? null, e.target.id]),
  );
}

// Whether the engine's binary has been requested yet.
window.wasmRequested = () =>
  performance.getEntriesByType('resource').some((e) => /cue_web_bg\.wasm/.test(e.name));

window.hostReady = true;
