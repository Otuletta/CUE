// Minimal line icons, 24×24, drawn for CUE. `currentColor` strokes so they
// follow the button's colour in every theme and in forced-colours mode.

const svg = (body) =>
  `<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;

export const ICONS = {
  help: svg('<circle cx="12" cy="12" r="9"/><path d="M9.6 9.3a2.5 2.5 0 1 1 3.3 2.4c-.6.3-.9.8-.9 1.4v.6"/><circle cx="12" cy="17" r=".6" fill="currentColor"/>'),
  undo: svg('<path d="M9 14 4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/>'),
  rack: svg('<path d="M12 4 4 18h16Z"/><circle cx="12" cy="14" r="1.6"/>'),
  soundOn: svg('<path d="M4 10v4h3l5 4V6L7 10Z"/><path d="M16 9a4 4 0 0 1 0 6"/><path d="M18.5 6.5a7.5 7.5 0 0 1 0 11"/>'),
  soundOff: svg('<path d="M4 10v4h3l5 4V6L7 10Z"/><path d="m16 9 5 6M21 9l-5 6"/>'),
  settings: svg('<circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M5.3 18.7l2.1-2.1M16.6 7.4l2.1-2.1"/>'),
  fullscreen: svg('<path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/>'),
  fullscreenExit: svg('<path d="M9 4v5H4M15 4v5h5M9 20v-5H4M15 20v-5h5"/>'),
  pause: svg('<path d="M9 5v14M15 5v14"/>'),
  play: svg('<path d="M8 5v14l11-7Z"/>'),
  exit: svg('<path d="M6 6l12 12M18 6 6 18"/>'),
};
