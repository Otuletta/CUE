// Type declarations for CUE's web API. The reference with every rule and
// example is docs/api.md; these types mirror it exactly.

/** Interface languages. */
export type Locale = 'en' | 'es';
export type Theme = 'classic' | 'studio' | 'contrast';
export type Assists = 'off' | 'basic' | 'assisted';
/** Graphics quality: changes anti-aliasing and shadows, never the physics. */
export type Quality = 'auto' | 'high' | 'medium' | 'low';
/** The tier the engine renders; `auto` settles on one of these. */
export type Tier = 'high' | 'medium' | 'low';
export type Mode = 'free-play' | 'eight-ball-local';

/** Eight-ball rule profiles (docs/rules-eight-ball.md). */
export type Ruleset = 'eight-ball-wpa' | 'eight-ball-casual';
/** The two players: A breaks first unless chosen otherwise. */
export type Seat = 'a' | 'b';
export type Group = 'solids' | 'stripes';
/** Head = the break end (−X); near = the rail at +Z (bottom of a landscape view). */
export type PocketId = 'head-far' | 'head-near' | 'side-far' | 'side-near' | 'foot-far' | 'foot-near';
/** Where a cue ball in hand may go. */
export type InHand = 'no' | 'anywhere' | 'above-head-string';
export type Stage = 'break' | 'play' | 'decision' | 'over';
export type Foul =
  | 'cue-ball-pocketed'
  | 'no-contact'
  | 'wrong-ball-first'
  | 'no-rail-after-contact'
  | 'bad-cue-ball-placement'
  | 'bad-play-from-above-head-string';
export type LossReason = 'eight-with-foul' | 'eight-before-group-cleared' | 'eight-in-uncalled-pocket';
export type Situation = 'illegal-break' | 'eight-on-legal-break' | 'eight-on-foul-break' | 'foul-on-break';
export type Choice =
  | 'accept-table'
  | 'rerack-and-break'
  | 'rerack-offender-breaks'
  | 'spot-eight-and-continue'
  | 'rebreak'
  | 'spot-eight-cue-in-hand'
  | 'cue-in-hand';
/** Why a shot or a call was refused. */
export type Refusal =
  | 'decision-pending'
  | 'rack-over'
  | 'not-expecting-shot'
  | 'choice-unavailable'
  | 'no-call-on-break'
  | 'ball-not-on-table'
  | 'not-your-group'
  | 'call-the-eight'
  | 'eight-on-open-table'
  | 'call-required'
  | 'call-the-eights-pocket';

/** Setup of a local eight-ball match. */
export interface MatchSetup {
  /** Default `eight-ball-wpa`. */
  ruleset?: Ruleset;
  /** Default `random`. */
  breaker?: Seat | 'random';
  /** Racks needed to win, 1–15. Default 1. */
  raceTo?: number;
  /** Display names, never stored. Default "Player 1" and "Player 2", localised. */
  names?: [string?, string?];
}

export interface AudioSettings {
  enabled: boolean;
  /** 0–1. */
  volume: number;
}

/** Preferences, saved under `cue:v1:settings` unless `persist` is false. */
export interface Settings {
  theme: Theme;
  assists: Assists;
  quality: Quality;
  sound: AudioSettings;
}

export interface CueOptions {
  /** The element CUE fills. It must have a size. */
  container: HTMLElement;
  /** The built-in accessible controls. Default `true`. */
  ui?: boolean;
  /** Default: the browser's language, else English. */
  locale?: Locale;
  theme?: Theme;
  assists?: Assists;
  quality?: Quality;
  audio?: Partial<AudioSettings>;
  /** Remember preferences in localStorage. Default `true`. */
  persist?: boolean;
  /** The game to start with. Default `free-play`. */
  mode?: Mode;
  /** The match to start when `mode` is `eight-ball-local`. */
  match?: MatchSetup;
  accessibility?: {
    /** Overrides `prefers-reduced-motion`: no cue swing. */
    reducedMotion?: boolean;
    /** Starts with the high-contrast theme. */
    highContrast?: boolean;
  };
  /** Shows an Exit button that calls this. */
  onExit?: () => void;
  /** Link shown when the browser cannot run CUE. */
  fallbackUrl?: string;
  /** Link to the eight-ball rules notes, shown in the match setup. */
  rulesUrl?: string;
  /** Where `manifest.json` and `pkg/` are served, when not next to the module. */
  assetBase?: string | URL;
}

export interface Call {
  kind: 'none' | 'safety' | 'ball';
  ball?: number;
  pocket?: PocketId;
}

export interface DecisionDetail {
  chooser: Seat;
  situation: Situation;
  options: Choice[];
  /** The cue ball left the table on the shot that caused this. */
  cueBallOff: boolean;
}

export interface OutcomeDetail {
  winner: Seat;
  /** `null` when the winner pocketed the 8 legally. */
  loss: LossReason | null;
}

export interface RulingDetail {
  shooter: Seat;
  /** What was called for the shot. */
  call: Call;
  fouls: Foul[];
  /** The ball hit first, when `wrong-ball-first` is among the fouls. */
  wrongBall: number | null;
  /** Object balls pocketed, in order. */
  pocketed: number[];
  assigned: { seat: Seat; group: Group } | null;
  next: 'continue' | 'pass' | 'decide' | 'over';
  /** For `pass`: where the incoming player may put the cue ball. */
  inHand: InHand | null;
}

export interface PlayerState {
  seat: Seat;
  name: string;
  /** Racks won. */
  score: number;
  group: Group | null;
  /** Balls of `group` still on the table. */
  remaining: number | null;
}

export interface MatchState {
  ruleset: Ruleset;
  players: [PlayerState, PlayerState];
  raceTo: number;
  /** Racks played, including the current one. */
  rack: number;
  breaker: Seat;
  /** Whose turn it is, or who decides. */
  shooter: Seat;
  stage: Stage;
  openTable: boolean;
  onTheEight: boolean;
  call: Call;
  /** The player chose the call; otherwise it follows the aim. */
  manualCall: boolean;
  /** Balls the shooter may hit first. */
  legalTargets: number[];
  decision: DecisionDetail | null;
  outcome: OutcomeDetail | null;
  matchOver: boolean;
  canShoot: boolean;
  /** Why a shot would be refused now. */
  refusal: Refusal | null;
  /** A 1.6 spot request is allowed now. */
  canSpot: boolean;
  lastRuling: RulingDetail | null;
}

/** A read-only snapshot, refreshed every frame. */
export interface CueState {
  mode: 'free-play' | 'eight-ball';
  /** Charged power, 0–1. */
  power: number;
  /** Tip offset in ball radii: side, follow (+) or draw (−). */
  tipX: number;
  tipY: number;
  atRest: boolean;
  inHand: boolean;
  handRegion: 'anywhere' | 'above-head-string' | null;
  canUndo: boolean;
  paused: boolean;
  theme: Theme;
  assists: Assists;
  quality: Quality;
  graphics: Tier;
  /** Shots taken on this table. */
  shots: number;
  ballsInPlay: number[];
  /** Shots below this power are refused. */
  minPower: number;
  eightBall: MatchState | null;
}

export interface PocketedBall {
  ballId: number;
  pocketId: PocketId;
}

export interface ShotSummary {
  shotId: number;
  firstContacts: number[];
  pocketed: PocketedBall[];
  cueBallPocketed: boolean;
  /** Distinct object balls driven to a rail (WPA 2.7). */
  objectBallsToCushion: number;
}

export interface RackResult extends OutcomeDetail {
  /** Racks won, A then B. */
  score: [number, number];
}

/** Every event and its detail. */
export interface CueEvents {
  /** The build version. */
  ready: string;
  'shot-start': undefined;
  'ball-pocketed': PocketedBall;
  'shot-end': ShotSummary | { abandoned: true };
  racked: { cleared: boolean };
  undo: undefined;
  paused: undefined;
  resumed: undefined;
  'fullscreen-change': { fullscreen: boolean };
  'settings-change': Settings;
  destroyed: undefined;
  error: { code: string };
  'new-game': { mode: 'free-play' | 'eight-ball' };
  ruling: RulingDetail;
  foul: { shooter: Seat; fouls: Foul[]; wrongBall: number | null };
  'groups-assigned': { seat: Seat; group: Group };
  'turn-changed': { shooter: Seat; inHand: InHand };
  'decision-required': DecisionDetail;
  decided: { choice: Choice; shooter: Seat; inHand: InHand; stage: Stage };
  'rack-over': RackResult;
  'match-over': RackResult;
  'ball-spotted': { ballId: number };
  'shot-refused': { reason: Refusal };
  'call-refused': { reason: Refusal };
}

export interface CueInstance {
  /** The engine's build version. */
  readonly version: string;
  /** Subscribes; returns a function that unsubscribes. */
  on<K extends keyof CueEvents>(name: K, handler: (detail: CueEvents[K]) => void): () => void;
  /** Resolves once the game runs; resumes a paused instance. */
  start(): Promise<void>;
  /** Stops simulation, input, sound and rendering (0 frames). */
  pause(): void;
  resume(): void;
  /** Moves the game into another container (single-page apps). */
  attach(container: HTMLElement): void;
  /** Re-measures the layout after moves or restyles. */
  resize(): void;
  setMode(mode: Mode, match?: MatchSetup): void;

  /** Free play: racks the balls again (undoable). */
  rerack(): void;
  /** Alias of `rerack`. */
  reset(): void;
  /** Free play: undoes the last shot or re-rack. */
  undo(): void;
  shoot(): void;
  /** 0–1. */
  setPower(power: number): void;
  /** Tip offset in ball radii, within the miscue limit. */
  setTip(x: number, y: number): void;

  newGame(setup?: MatchSetup): void;
  freePlay(): void;
  callBall(ball: number, pocket: PocketId): void;
  callSafety(): void;
  /** Lets the call follow the aim again. */
  autoCall(): void;
  decide(choice: Choice): void;
  nextRack(): void;
  /** Both players agree to a stalemate (WPA 4.11). */
  stalemate(): void;
  /** WPA 1.6: spots the legal ball nearest the head string. */
  spotRequest(): void;

  /** CSS pixels covered by the host's own controls (with `ui: false`). */
  setViewInsets(top: number, bottom: number): void;
  setTheme(theme: Theme): void;
  setAssists(assists: Assists): void;
  setQuality(quality: Quality): void;
  setAudio(audio: Partial<AudioSettings>): void;
  getSettings(): Settings;
  /** `null` once destroyed. */
  getState(): CueState | null;
  enterFullscreen(): Promise<void>;
  exitFullscreen(): Promise<void>;
  /** Shows the how-to-play dialog (built-in controls only). */
  showHelp(): void;
  stats(): { frames: number };
  /**
   * Stops the game and removes everything CUE added. Idempotent. A page
   * cannot mount CUE again after this: use `pause` and `attach` instead.
   */
  destroy(): Promise<void>;
}

/**
 * Mounts CUE in `options.container` and resolves once the first frame has
 * run. Rejects with `INSTANCE_ALREADY_MOUNTED`, `UNSUPPORTED_BROWSER`,
 * `LOAD_FAILED: …` or `RELOAD_REQUIRED`.
 */
export function createCue(options: CueOptions): Promise<CueInstance>;
