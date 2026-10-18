# API reference

The package is `@tuletta/cue`, under the npm organization `tuletta`. It has one
entry point, `createCue`, and the types in
[`index.d.ts`](../web/package/src/index.d.ts) mirror this page. For hosting,
frameworks and the page lifecycle, see [web-integration.md](web-integration.md).

```js
import { createCue } from '@tuletta/cue';

const game = await createCue({ container: document.querySelector('#cue') });
const off = game.on('shot-end', (shot) => console.log(shot));
```

## `createCue(options)`

Mounts CUE in `options.container` and resolves with an instance once the
first frame has run. The container must have a size; CUE fills it.

| Option | Type | Default | Notes |
|---|---|---|---|
| `container` | `HTMLElement` | required | |
| `ui` | `boolean` | `true` | The built-in accessible controls. With `false`, the host draws its own and calls the methods below. |
| `locale` | `'en' \| 'es'` | browser | Interface language. |
| `theme` | `'classic' \| 'studio' \| 'contrast'` | saved, else `studio` | Presentation only. |
| `assists` | `'off' \| 'basic' \| 'assisted'` | saved, else `basic` | Aiming guide. |
| `quality` | `'auto' \| 'high' \| 'medium' \| 'low'` | saved, else `auto` | Anti-aliasing, shadows and, at `low`, resolution; never the physics. |
| `render` | `{ pixelRatioCap? }` | `{ pixelRatioCap: 2 }` | Pixels rendered per CSS pixel at most, on denser screens: a 3× phone renders 2×, which costs about half as much. At least 0.5; `Infinity` renders every device pixel. The `low` tier renders at most 1.5. |
| `audio` | `{ enabled?, volume? }` | saved, else on at 0.7 | |
| `persist` | `boolean` | `true` | Save preferences in `localStorage` (`cue:v1:settings`). |
| `mode` | `'free-play' \| 'eight-ball-local'` | `free-play` | The game to start with. |
| `match` | `MatchSetup` | | For `mode: 'eight-ball-local'`, as for `newGame`. |
| `accessibility` | `{ reducedMotion?, highContrast? }` | | `reducedMotion` overrides `prefers-reduced-motion` (no cue swing). `highContrast` starts with the contrast theme. |
| `onExit` | `() => void` | | Shows an Exit button that calls it. |
| `fallbackUrl` | `string` | | Link shown when the browser cannot run CUE. |
| `rulesUrl` | `string` | | Link to the rules notes, shown in the match setup. |
| `assetBase` | `string \| URL` | the module's folder | Where `manifest.js`, `manifest.json` and `pkg/` are served (a CDN, or a copy in a public folder). |
| `signal` | `AbortSignal` | | Cancels the load ([below](#cancelling-a-load)). |

### Rejections

The promise rejects with an `Error` whose message is one of these codes:

| Code | When |
|---|---|
| `INSTANCE_ALREADY_MOUNTED` | An instance is live on this page. Move it with `attach` instead. |
| `RELOAD_REQUIRED` | An instance was destroyed on this page; only a reload can create another. |
| `UNSUPPORTED_BROWSER` | No WebAssembly or WebGL 2. The built-in UI shows an explanation. |
| `LOAD_FAILED: …` | The module or binary could not be loaded, with the reason after the colon. |
| `ABORTED` | The load was cancelled with `signal`. The error's `name` is `AbortError` and its `cause` the signal's reason. |

### Cancelling a load

Pass an `AbortSignal` to stop a load the visitor no longer wants, for
example when they leave the page while CUE downloads:

```js
const controller = new AbortController();
cancelButton.onclick = () => controller.abort();
try {
  const game = await createCue({ container, signal: controller.signal });
} catch (error) {
  if (error.name !== 'AbortError') throw error;
}
```

- **Before the engine starts,** which is nearly all of the load, aborting
  stops the download and removes everything CUE added. The page can call
  `createCue` again straight away, even in the same task.
- **In the last frames before the promise settles,** the engine is already
  running, so aborting destroys it: as after any `destroy()`, a new
  instance needs a reload.
- **Once `createCue` has resolved,** the signal has no effect. Use
  `pause()`, or `destroy()` when CUE will not come back.

To hide CUE during a route change, keep the load going and pause the
instance when it arrives, as in
[web-integration.md](web-integration.md#single-page-apps).

### Failures after start

Two failures can end a running instance. Each is reported by an `error`
event, and the built-in UI replaces the table with a message and a Reload
button:

| Code | When |
|---|---|
| `ENGINE_FAILED` | The engine hit an internal error. The details go to the console; the event carries only the code. |
| `CONTEXT_LOST` | The browser dropped the WebGL context, for example after a GPU reset. |

After either one, methods do nothing instead of throwing, so the host's own
code keeps working; `getState()` returns `null`. `destroy()` still removes
everything. A new instance needs a reload.

## Instance

### Lifecycle

| Method | Does |
|---|---|
| `start(): Promise<void>` | Resolves once the game runs. `createCue` already starts it, so this only resumes a paused instance. It never creates a second one. |
| `pause(reason?)` | Stops simulation, input, sound and rendering. A paused instance runs **0 frames**. `reason`, a short word such as `hidden` or `route`, is reported by the `paused` event; the default is `host`. |
| `resume(reason?)` | Continues a paused instance. `reason` is reported by the `resumed` event, as for `pause`. |
| `attach(container)` | Moves the game into another element. State, preferences and the GPU surface carry over. For single-page apps. |
| `resize()` | Re-measures the layout. The canvas follows its container by itself; call this after moves or restyles that a resize does not cover. |
| `destroy(): Promise<void>` | Stops everything and removes all that CUE added. Idempotent: it returns the same promise every time. Afterwards every method throws `INSTANCE_DESTROYED`, `getState()` returns `null`, and `createCue` rejects with `RELOAD_REQUIRED`. |

### The table

| Method | Does |
|---|---|
| `setMode(mode, match?)` | `'free-play'` or `'eight-ball-local'` (with a `MatchSetup`). |
| `shoot()` | Shoots with the current aim, power and spin. Ignored while balls move or below `minPower`. |
| `setPower(power)` | `0..1`, as the slider. |
| `setTip(x, y)` | The tip offset in ball radii: `x` is side spin, `y` follow (+) or draw (−). Clamped to the miscue limit. |
| `turnAim(radians)` | Turns the aim by a small angle, counter-clockwise seen from above: fine aiming for your own controls. Ignored while balls move. |
| `rerack()`, alias `reset()` | Free play: a new rack, undoable. |
| `undo()` | Free play: undoes the last shot or re-rack. |

### Eight-ball

| Method | Does |
|---|---|
| `newGame(setup?)` | Starts a match. `setup`: `{ ruleset?, breaker?, raceTo?, names? }`. `ruleset` is `eight-ball-wpa` (the default) or `eight-ball-casual`; `breaker` is `a`, `b` or `random`; `raceTo` is 1–15. `names` are for display only and never stored. |
| `freePlay()` | Leaves the match. |
| `callBall(ball, pocket)` | Calls a ball and a pocket for the next shot. An invalid call emits `call-refused`. |
| `callSafety()` | Calls a safety. |
| `autoCall()` | Lets the call follow the aim again (the default). |
| `decide(choice)` | Answers a pending decision with one of `state.eightBall.decision.options`. |
| `nextRack()` | Racks the next game once a rack is over. The break alternates. |
| `stalemate()` | Both players agree to a stalemate; the rack is replayed (WPA 4.11). |
| `spotRequest()` | WPA 1.6: spots the legal ball nearest the head string. Allowed when `state.eightBall.canSpot` is true. |

Pockets are `head-far`, `head-near`, `side-far`, `side-near`, `foot-far` and
`foot-near`. The head is the break end. In landscape, the far rail is at the
top of the screen and the foot is on the right.

### Preferences and display

| Method | Does |
|---|---|
| `setTheme(theme)`, `setAssists(level)`, `setQuality(quality)`, `setAudio({ enabled?, volume? })` | Change and save a preference. Each emits `settings-change`. |
| `getSettings()` | A copy of the preferences. |
| `getState()` | A snapshot ([below](#state)), or `null` once destroyed. |
| `setViewInsets(top, bottom)` | CSS pixels at the top and bottom covered by the host's own controls, for `ui: false`. The camera keeps the balls clear of them. |
| `enterFullscreen()`, `exitFullscreen()` | Real fullscreen where allowed, else an immersive layout. Every style changed is restored on exit. |
| `showHelp()` | Opens the how-to-play dialog (built-in UI). |
| `on(name, handler): () => void` | Subscribes to an event. Returns a function that unsubscribes. Unknown names throw. |
| `stats()` | `{ frames }`, the frames run since mounting. |
| `version` | The build version, for example `1.0.0-beta.3`. |

## Events

Handlers receive one `detail` argument. A handler that throws does not stop
the game or other handlers; the error is rethrown asynchronously.

| Event | Detail |
|---|---|
| `ready` | The build version. |
| `shot-start` | `{ shotId, shooter }`: the shot's id, which its `shot-end` carries too, and the seat that shot (`a` or `b`), or `null` in free play. |
| `ball-pocketed` | `{ ballId, pocketId }`, where `ballId` 0 is the cue ball. |
| `shot-end` | `{ shotId, firstContacts, pocketed: [{ ballId, pocketId }], cueBallPocketed, objectBallsToCushion }`, or `{ abandoned: true }` after an undo or a re-rack mid-shot. |
| `racked` | `{ cleared }`: `true` when free play re-racked a cleared table. |
| `undo`, `destroyed` | — |
| `paused` / `resumed` | `{ reason }`: `user` when the player used CUE's pause control, else the reason given to `pause` or `resume` (`host` by default). |
| `settings-change` | The preferences. |
| `fullscreen-change` | `{ fullscreen }` |
| `error` | `{ code, message }`: `code` is one of the rejection codes above; `message` is a plain description, safe to show, with no paths or page data. |
| `new-game` | `{ mode: 'free-play' \| 'eight-ball' }` |
| `ruling` | `{ shooter, call, fouls, wrongBall, pocketed, assigned, next, inHand }`: the referee's verdict on a shot. `next` is `continue`, `pass`, `decide` or `over`. |
| `foul` | `{ shooter, fouls, wrongBall }` |
| `groups-assigned` | `{ seat, group }` |
| `turn-changed` | `{ shooter, previous, inHand }`: the incoming and the outgoing player. |
| `decision-required` | `{ chooser, situation, options, cueBallOff }` |
| `decided` | `{ choice, shooter, inHand, stage }` |
| `game-over` / `match-over` | `{ winner, loss, score }` once a rack, or the match, is decided: `loss` is `null` for a legal win, else why the other player lost; `score` is `[a, b]`. |
| `ball-spotted` | `{ ballId }` |
| `shot-refused` / `call-refused` | `{ reason }`, a refusal code ([below](#codes)). |

Per shot there is exactly one `shot-start` and one `shot-end`, with any
number of `ball-pocketed` events between them. In a match, `ruling` follows
`shot-end`, and the more specific events follow the ruling.

Event names and detail shapes are part of the versioned API: within a major
version, details only gain fields.

## State

`getState()` returns a plain object, refreshed every frame. It holds no
references into the engine.

| Field | Meaning |
|---|---|
| `mode` | `free-play` or `eight-ball`. |
| `aimAngle` | The aim on the table, radians: 0 points at the foot rail, π/2 at the near rail. |
| `power`, `tipX`, `tipY` | The charged power and tip offset. |
| `atRest`, `inHand`, `handRegion` | Whether balls are still; whether the cue ball can be moved, and where (`anywhere` or `above-head-string`). |
| `canUndo`, `paused`, `shots`, `ballsInPlay`, `minPower` | |
| `theme`, `assists`, `quality`, `graphics` | Preferences. `graphics` is the tier rendered now (`high`, `medium` or `low`). |
| `eightBall` | The match, or `null`. |

The `eightBall` object has these fields:

- `ruleset`, `raceTo`, `rack`, `breaker`;
- `shooter`: who plays, or who decides during a decision;
- `stage`: `break`, `play`, `decision` or `over`;
- `players`: `[{ seat, name, score, group, remaining }, …]`;
- `openTable`, `onTheEight`;
- `call` (`{ kind, ball?, pocket? }`) and `manualCall`;
- `legalTargets`;
- `decision`, `outcome` and `matchOver`;
- `canShoot`, with `refusal` saying why not;
- `canSpot`;
- `lastRuling`.

## Codes

These names are stable codes, meant for the host to translate.

- **Fouls:** `cue-ball-pocketed`, `no-contact`, `wrong-ball-first`,
  `no-rail-after-contact`, `bad-cue-ball-placement`,
  `bad-play-from-above-head-string`.
- **Losses:** `eight-with-foul`, `eight-before-group-cleared`,
  `eight-in-uncalled-pocket`.
- **Decisions:** `illegal-break`, `eight-on-legal-break`,
  `eight-on-foul-break`, `foul-on-break`.
- **Choices:** `accept-table`, `rerack-and-break`, `rerack-offender-breaks`,
  `spot-eight-and-continue`, `rebreak`, `spot-eight-cue-in-hand`,
  `cue-in-hand`.
- **Refusals:** `decision-pending`, `rack-over`, `not-expecting-shot`,
  `choice-unavailable`, `no-call-on-break`, `ball-not-on-table`,
  `not-your-group`, `call-the-eight`, `eight-on-open-table`,
  `call-required`, `call-the-eights-pocket`.
- **Errors thrown by methods:** `INSTANCE_DESTROYED`,
  `INSTANCE_NOT_MOUNTED`, and for invalid names `UNKNOWN_THEME`,
  `UNKNOWN_ASSISTS`, `UNKNOWN_QUALITY`, `UNKNOWN_RULESET`, `UNKNOWN_SEAT`,
  `UNKNOWN_BALL`, `UNKNOWN_POCKET`, `UNKNOWN_CHOICE`.

## Versioning

The API follows SemVer (spec §26.3). A breaking change to a method, an
option, an event, a code or the state's meaning needs a major version.

A physics change that alters shot outcomes is visible behaviour even if no
signature changes. It bumps the physics profile id (now `baseline-2`) and is
recorded in the [changelog](../CHANGELOG.md).
