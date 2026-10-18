// The smallest useful host: load CUE on demand, then listen to it.
const play = document.getElementById('play');

play.addEventListener('click', async () => {
  play.disabled = true;
  // A dynamic import keeps the engine out of the page's first download.
  const { createCue } = await import('@tuletta/cue');
  const game = await createCue({ container: document.getElementById('cue') });
  play.remove();
  game.on('shot-end', (shot) => console.info('shot', shot));
  game.on('rack-over', ({ winner, score }) => console.info(`rack to ${winner}`, score));
  window.cue = game; // for poking around in the console
});
