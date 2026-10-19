// The smallest useful host: one tag, then listen to it. The element module is
// small; the engine itself downloads only as the table nears the viewport.
import '@tuletta/cue/element';

const cue = document.querySelector('cue-game');
cue.addEventListener('cue-ready', () => {
  window.cue = cue.game; // for poking around in the console
});
cue.addEventListener('cue-shot-end', ({ detail }) => console.info('shot', detail));
cue.addEventListener('cue-game-over', ({ detail }) => console.info(`rack to ${detail.winner}`, detail.score));
