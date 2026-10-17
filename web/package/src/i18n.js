// Interface strings. English and Spanish; the locale defaults to the
// browser's language and can be forced with `createCue({ locale })`.

const STRINGS = {
  en: {
    loading: 'Loading CUE…',
    loadingProgress: (p) => `Loading CUE… ${p}%`,
    unsupportedTitle: 'CUE can’t run in this browser',
    unsupportedBody: 'It needs WebAssembly and WebGL 2. Try a current version of Chrome, Edge, Firefox or Safari.',
    errorTitle: 'CUE couldn’t start',
    errorBody: 'Something went wrong while loading the game.',
    tryAgain: 'Try again',
    learnMore: 'About the project',
    freePlay: 'Free play',
    ballsLeft: (n) => (n === 1 ? '1 ball left' : `${n} balls left`),
    inHand: 'Ball in hand: drag the cue ball to place it',
    rolling: 'Balls rolling…',
    toolbar: 'Game controls',
    help: 'How to play (H)',
    undo: 'Undo last shot (U)',
    newRack: 'New rack (N)',
    sound: 'Sound (M)',
    settings: 'Settings (O)',
    fullscreen: 'Full screen (F)',
    exitFullscreen: 'Exit full screen (F)',
    pause: 'Pause (P)',
    resume: 'Resume (P)',
    exit: 'Exit',
    spin: 'Spin',
    spinValue: (x, y) => describeSpin(x, y, 'en'),
    power: 'Power',
    shoot: 'Shoot',
    shootHint: 'Set the power first: use the slider or drag back on the table.',
    pausedTitle: 'Paused',
    confirmRackTitle: 'Start a new rack?',
    confirmRackBody: 'The current table will be replaced. You can undo this.',
    confirmRack: 'New rack',
    cancel: 'Cancel',
    close: 'Close',
    done: 'Let’s play',
    howTitle: 'How to play',
    howPointer: [
      'Point where you want the cue ball to go.',
      'Press, drag back to set the power, and release to shoot. Letting go with almost no pull cancels.',
      'Or set the power with the slider and press Shoot.',
      'For spin, drag the red dot on the white ball.',
    ],
    howTouch: [
      'Touch where you want the cue ball to go.',
      'Keep your finger down, drag back towards the cue ball, and release to shoot.',
      'Or set the power with the slider and tap Shoot.',
      'For spin, drag the red dot on the white ball.',
    ],
    shortcutsTitle: 'Keyboard',
    shortcuts: [
      ['← →', 'aim (hold Shift for fine)'],
      ['↑ ↓', 'power'],
      ['Space', 'shoot'],
      ['W A S D', 'spin · C centres it'],
      ['I J K L', 'move the cue ball in hand'],
      ['U · N · P · M', 'undo · new rack · pause · sound'],
    ],
    settingsTitle: 'Settings',
    theme: 'Theme',
    themes: { classic: 'Classic', studio: 'Studio', contrast: 'High contrast' },
    assists: 'Aiming guide',
    assistsLevels: { off: 'Off', basic: 'Basic', assisted: 'Assisted' },
    assistsNote: 'The guide is an estimate: spin, throw and curve can change the result.',
    soundToggle: 'Sound',
    volume: 'Volume',
    resetSettings: 'Reset preferences',
    pocketed: (list) => `Pocketed ${list}`,
    scratch: 'Scratch: the cue ball is back in hand',
    noPocket: 'No balls pocketed',
    cleared: 'Table cleared! New rack.',
    racked: 'New rack',
    undone: 'Shot undone',
    ball: (n) => `${n}`,
  },
  es: {
    loading: 'Cargando CUE…',
    loadingProgress: (p) => `Cargando CUE… ${p} %`,
    unsupportedTitle: 'CUE no puede funcionar en este navegador',
    unsupportedBody: 'Necesita WebAssembly y WebGL 2. Prueba una versión actual de Chrome, Edge, Firefox o Safari.',
    errorTitle: 'CUE no ha podido arrancar',
    errorBody: 'Algo ha fallado al cargar el juego.',
    tryAgain: 'Reintentar',
    learnMore: 'Sobre el proyecto',
    freePlay: 'Juego libre',
    ballsLeft: (n) => (n === 1 ? 'Queda 1 bola' : `Quedan ${n} bolas`),
    inHand: 'Bola en mano: arrastra la blanca para colocarla',
    rolling: 'Bolas en movimiento…',
    toolbar: 'Controles del juego',
    help: 'Cómo se juega (H)',
    undo: 'Deshacer el último tiro (U)',
    newRack: 'Nuevo rack (N)',
    sound: 'Sonido (M)',
    settings: 'Ajustes (O)',
    fullscreen: 'Pantalla completa (F)',
    exitFullscreen: 'Salir de pantalla completa (F)',
    pause: 'Pausa (P)',
    resume: 'Continuar (P)',
    exit: 'Salir',
    spin: 'Efecto',
    spinValue: (x, y) => describeSpin(x, y, 'es'),
    power: 'Potencia',
    shoot: 'Golpear',
    shootHint: 'Primero elige la potencia: usa la barra o arrastra hacia atrás sobre la mesa.',
    pausedTitle: 'En pausa',
    confirmRackTitle: '¿Empezar un nuevo rack?',
    confirmRackBody: 'Se reemplazará la mesa actual. Puedes deshacerlo.',
    confirmRack: 'Nuevo rack',
    cancel: 'Cancelar',
    close: 'Cerrar',
    done: 'A jugar',
    howTitle: 'Cómo se juega',
    howPointer: [
      'Apunta con el ratón hacia donde quieres enviar la blanca.',
      'Pulsa, arrastra hacia atrás para elegir la potencia y suelta para golpear. Si sueltas casi sin tirar, se cancela.',
      'O elige la potencia con la barra y pulsa Golpear.',
      'Para dar efecto, arrastra el punto rojo de la bola blanca.',
    ],
    howTouch: [
      'Toca hacia donde quieres enviar la blanca.',
      'Sin levantar el dedo, arrastra hacia la blanca y suelta para golpear.',
      'O elige la potencia con la barra y toca Golpear.',
      'Para dar efecto, arrastra el punto rojo de la bola blanca.',
    ],
    shortcutsTitle: 'Teclado',
    shortcuts: [
      ['← →', 'apuntar (Mayús para afinar)'],
      ['↑ ↓', 'potencia'],
      ['Espacio', 'golpear'],
      ['W A S D', 'efecto · C lo centra'],
      ['I J K L', 'mover la blanca en mano'],
      ['U · N · P · M', 'deshacer · nuevo rack · pausa · sonido'],
    ],
    settingsTitle: 'Ajustes',
    theme: 'Tema',
    themes: { classic: 'Clásico', studio: 'Estudio', contrast: 'Alto contraste' },
    assists: 'Guía de apuntado',
    assistsLevels: { off: 'Desactivada', basic: 'Básica', assisted: 'Asistida' },
    assistsNote: 'La guía es una estimación: el efecto, el throw y la curva pueden cambiar el resultado.',
    soundToggle: 'Sonido',
    volume: 'Volumen',
    resetSettings: 'Restablecer preferencias',
    pocketed: (list) => `Metida: ${list}`,
    scratch: 'Falta: la blanca vuelve a estar en mano',
    noPocket: 'Ninguna bola metida',
    cleared: '¡Mesa limpia! Nuevo rack.',
    racked: 'Nuevo rack',
    undone: 'Tiro deshecho',
    ball: (n) => `${n}`,
  },
};

function describeSpin(x, y, lang) {
  const words = {
    en: { centre: 'centre', left: 'left', right: 'right', follow: 'follow', draw: 'draw' },
    es: { centre: 'centro', left: 'izquierda', right: 'derecha', follow: 'avance', draw: 'retroceso' },
  }[lang];
  const parts = [];
  if (Math.abs(y) >= 0.05) parts.push(`${y > 0 ? words.follow : words.draw} ${Math.round(Math.abs(y) * 200)} %`);
  if (Math.abs(x) >= 0.05) parts.push(`${x > 0 ? words.right : words.left} ${Math.round(Math.abs(x) * 200)} %`);
  return parts.length ? parts.join(', ') : words.centre;
}

/** Picks a supported locale: the requested one, else the browser's, else English. */
export function resolveLocale(requested) {
  const candidates = [requested, ...(navigator.languages ?? [navigator.language])];
  for (const tag of candidates) {
    const lang = String(tag ?? '').slice(0, 2).toLowerCase();
    if (lang in STRINGS) return lang;
  }
  return 'en';
}

export function strings(locale) {
  return STRINGS[resolveLocale(locale)];
}

/** "3, 9 and 11" in the given locale. */
export function formatList(items, locale) {
  try {
    return new Intl.ListFormat(resolveLocale(locale), { style: 'long', type: 'conjunction' }).format(items);
  } catch {
    return items.join(', ');
  }
}
