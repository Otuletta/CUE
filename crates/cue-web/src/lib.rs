//! # cue-web
//!
//! The boundary between CUE and a host web page. The JavaScript wrapper in
//! `web/package` calls into this crate; nothing here may assume anything about
//! the host's router, styles or framework.
//!
//! ## Model
//!
//! - **One instance per page** (spec §17.6). A second [`mount`] while one is
//!   live fails with an explicit error rather than misbehaving.
//! - **Commands in, events out.** Host calls enqueue commands that a Bevy
//!   system applies at the start of the next frame, so the host never touches
//!   ECS state directly. Game events go to a single JS listener.
//! - **State is published, not queried.** Once per frame the engine writes a
//!   small snapshot (aim, power, ball in hand, the eight-ball match…) that the
//!   host's controls read cheaply without reaching into the ECS.
//! - **Pausing stops rendering.** A paused instance switches winit to
//!   reactive mode and sleeps until the host resumes it, so a backgrounded or
//!   hidden game costs no CPU or GPU.
//!
//! The browser is single-threaded for this module, which is why the listener
//! and the audio engine live in thread-locals.

mod audio;
mod eight;

use std::{
    cell::RefCell,
    sync::{
        Mutex, PoisonError, TryLockError,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    prelude::*,
    window::WindowResolution,
    winit::{EventLoopProxyWrapper, UpdateMode, WinitSettings, WinitUserEvent},
};
use cue_audio::Sound;
use cue_core::{
    BallId, PocketId, ShotReport,
    glam::DVec2,
    rules::{
        Ruleset, Seat,
        eight_ball::{Call, Choice, Next},
    },
};
use cue_game::{
    Aim, Assists, Contact, CuePlugin, GameCommand, GameEvent, GameSetup, Graphics, HandRegion,
    LastShot, MIN_SHOT_POWER, Mode, Paused, Quality, ShotFinished, Simulation, Theme, ViewInsets,
};
use wasm_bindgen::prelude::*;

use crate::eight::MatchSnapshot;

#[derive(Debug, Clone, PartialEq)]
enum Command {
    /// With the reason reported by the `paused` event.
    Pause(String),
    /// With the reason reported by the `resumed` event.
    Resume(String),
    Destroy,
    Game(GameCommand),
}

/// Lifecycle of the page's single instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Idle,
    Mounted,
    /// Bevy's event loop cannot be created twice in one page; see [`mount`].
    Destroyed,
}

/// What the host's controls need to draw, refreshed every frame.
#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    power: f64,
    tip: DVec2,
    at_rest: bool,
    in_hand: bool,
    hand_region: Option<HandRegion>,
    can_undo: bool,
    paused: bool,
    theme: &'static str,
    assists: &'static str,
    quality: &'static str,
    graphics: &'static str,
    shots: u32,
    balls_in_play: Vec<u8>,
    eight_ball: Option<MatchSnapshot>,
}

static LIFECYCLE: Mutex<Lifecycle> = Mutex::new(Lifecycle::Idle);
static INBOX: Mutex<Vec<Command>> = Mutex::new(Vec::new());
static SNAPSHOT: Mutex<Option<Snapshot>> = Mutex::new(None);
/// Wakes a sleeping (paused) event loop so queued commands are seen.
static WAKER: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
/// Frames run by the engine, so hosts and tests can verify that a paused
/// instance really stops working.
static FRAMES: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static LISTENER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

fn lifecycle() -> Lifecycle {
    *lock(&LIFECYCLE)
}

fn set_lifecycle(state: Lifecycle) {
    *lock(&LIFECYCLE) = state;
}

fn send(command: Command) -> Result<(), JsError> {
    if lifecycle() != Lifecycle::Mounted {
        return Err(JsError::new("INSTANCE_NOT_MOUNTED"));
    }
    lock(&INBOX).push(command);
    if let Some(wake) = lock(&WAKER).as_ref() {
        wake();
    }
    Ok(())
}

fn game(command: GameCommand) -> Result<(), JsError> {
    send(Command::Game(command))
}

/// Version of the CUE build, for diagnostics and the `ready` event.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Starts CUE, rendering into the `<canvas>` matched by `canvas_selector`.
///
/// The canvas is sized to its parent element. `width` and `height` are its
/// current size in CSS pixels, so the first frame already renders at the size
/// the page shows: Bevy would otherwise start at 1280 × 720, which on a dense
/// phone screen is a burst of GPU memory before the canvas settles. Returns
/// immediately; the game
/// runs on the browser's animation frames and emits `ready` once its first
/// frame has run.
///
/// Errors with `INSTANCE_ALREADY_MOUNTED` if an instance is live, or
/// `RELOAD_REQUIRED` after [`destroy`]: winit allows one event loop per
/// page, so a destroyed instance cannot be re-created without a reload. Hosts
/// that need repeated mounting should pause and resume instead, or isolate
/// CUE in an iframe (ADR-0006).
#[wasm_bindgen]
pub fn mount(canvas_selector: &str, width: u32, height: u32) -> Result<(), JsError> {
    match lifecycle() {
        Lifecycle::Mounted => return Err(JsError::new("INSTANCE_ALREADY_MOUNTED")),
        Lifecycle::Destroyed => return Err(JsError::new("RELOAD_REQUIRED")),
        Lifecycle::Idle => {}
    }
    set_lifecycle(Lifecycle::Mounted);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some(canvas_selector.to_owned()),
                    // Logical, at the starting scale factor of 1: winit
                    // applies the screen's pixel ratio.
                    resolution: WindowResolution::new(width.max(1), height.max(1)),
                    fit_canvas_to_parent: true,
                    // Only events aimed at the canvas are captured.
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            // CUE ships no `.meta` files; don't probe the server for them.
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            // Keep the host's console quiet: warnings and errors only,
            // minus the expected "feature unavailable on WebGL2" notices.
            .set(LogPlugin {
                level: Level::WARN,
                filter: "wgpu=error,naga=error,bevy_core_pipeline=error,bevy_pbr=error".into(),
                ..default()
            }),
    )
    .add_plugins(CuePlugin)
    .insert_resource(WinitSettings::game())
    .add_systems(Startup, install_waker)
    .add_systems(First, apply_commands)
    // Published first, so a listener that reads `state()` while handling
    // an event sees the state that event left behind.
    .add_systems(Last, (publish_state, emit_events, report_tier).chain());
    // After the plugins, so it wraps the console hook Bevy installs.
    report_panics();
    app.run();
    Ok(())
}

/// Tells the host when the engine panics, after the previous hook has logged
/// the details to the console. WebAssembly aborts right after, so the
/// instance cannot go on: the event lets the wrapper say so instead of
/// leaving a frozen table. The event carries only a code, never the panic
/// message.
fn report_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        // Nothing here may lock or borrow what the panic interrupted.
        let listener = LISTENER
            .try_with(|l| l.try_borrow().ok().and_then(|l| l.clone()))
            .ok()
            .flatten();
        if let Some(f) = listener {
            let detail = object(&[("code", "ENGINE_FAILED".into())]);
            let _ = f.call2(&JsValue::NULL, &JsValue::from_str("error"), &detail);
        }
        // Later calls get `INSTANCE_NOT_MOUNTED` and a new mount
        // `RELOAD_REQUIRED`, rather than running a broken engine.
        match LIFECYCLE.try_lock() {
            Ok(mut state) => *state = Lifecycle::Destroyed,
            Err(TryLockError::Poisoned(state)) => *state.into_inner() = Lifecycle::Destroyed,
            Err(TryLockError::WouldBlock) => {}
        }
    }));
}

/// Pauses simulation, input, sound and rendering. `reason` (default `host`)
/// is reported by the `paused` event.
#[wasm_bindgen]
pub fn pause(reason: Option<String>) -> Result<(), JsError> {
    send(Command::Pause(reason.unwrap_or_else(|| "host".into())))
}

/// Resumes a paused instance. `reason` (default `host`) is reported by the
/// `resumed` event.
#[wasm_bindgen]
pub fn resume(reason: Option<String>) -> Result<(), JsError> {
    send(Command::Resume(reason.unwrap_or_else(|| "host".into())))
}

/// Racks the balls again. Undoable. Free play only.
#[wasm_bindgen]
pub fn rerack() -> Result<(), JsError> {
    game(GameCommand::Rerack)
}

/// Undoes the last shot or re-rack. Free play only.
#[wasm_bindgen]
pub fn undo() -> Result<(), JsError> {
    game(GameCommand::Undo)
}

/// Starts a match of eight-ball for two players on this device.
///
/// `ruleset` is `eight-ball-wpa` (every shot called) or `eight-ball-casual`
/// (only the eight's pocket); `breaker` is `a`, `b` or `random`; `race_to` is
/// the racks needed to win, clamped to `1..=15`. Names are display-only,
/// trimmed to 24 characters and never stored. Emits `new-game`.
#[wasm_bindgen(js_name = newGame)]
pub fn new_game(
    ruleset: &str,
    breaker: &str,
    race_to: u32,
    name_a: &str,
    name_b: &str,
) -> Result<(), JsError> {
    let ruleset = Ruleset::parse(ruleset).ok_or_else(|| JsError::new("UNKNOWN_RULESET"))?;
    let breaker = match breaker {
        "random" => None,
        seat => Some(Seat::parse(seat).ok_or_else(|| JsError::new("UNKNOWN_SEAT"))?),
    };
    game(GameCommand::NewGame(GameSetup {
        ruleset,
        breaker,
        race_to: u8::try_from(race_to.clamp(1, 15)).unwrap_or(1),
        names: [name_a.to_owned(), name_b.to_owned()],
    }))
}

/// Leaves any match and goes back to free play on a fresh rack. Emits
/// `new-game`.
#[wasm_bindgen(js_name = freePlay)]
pub fn free_play() -> Result<(), JsError> {
    game(GameCommand::FreePlay)
}

/// Eight-ball: calls `ball` into `pocket` (a pocket name such as
/// `foot-near`) for the next shot, overriding the call that follows the aim.
/// An invalid call emits `call-refused`.
#[wasm_bindgen(js_name = callBall)]
pub fn call_ball(ball: u8, pocket: &str) -> Result<(), JsError> {
    let ball = BallId::new(ball).map_err(|_| JsError::new("UNKNOWN_BALL"))?;
    let pocket = PocketId::parse(pocket).ok_or_else(|| JsError::new("UNKNOWN_POCKET"))?;
    game(GameCommand::SetCall(Call::Ball { ball, pocket }))
}

/// Eight-ball: calls a safety for the next shot: play passes whatever drops.
#[wasm_bindgen(js_name = callSafety)]
pub fn call_safety() -> Result<(), JsError> {
    game(GameCommand::SetCall(Call::Safety))
}

/// Eight-ball: lets the call follow the aim again.
#[wasm_bindgen(js_name = autoCall)]
pub fn auto_call() -> Result<(), JsError> {
    game(GameCommand::AutoCall)
}

/// Eight-ball: answers a pending decision with one of the option names in
/// `state().eightBall.decision.options`.
#[wasm_bindgen]
pub fn decide(choice: &str) -> Result<(), JsError> {
    let choice = Choice::parse(choice).ok_or_else(|| JsError::new("UNKNOWN_CHOICE"))?;
    game(GameCommand::Decide(choice))
}

/// Eight-ball: racks the next game of the match once a rack is over. The
/// break alternates.
#[wasm_bindgen(js_name = nextRack)]
pub fn next_rack() -> Result<(), JsError> {
    game(GameCommand::NextRack)
}

/// Eight-ball: both players agree the rack is a stalemate (WPA 4.11). The
/// rack is replayed and the same player breaks.
#[wasm_bindgen]
pub fn stalemate() -> Result<(), JsError> {
    game(GameCommand::Stalemate)
}

/// Eight-ball: with the cue ball in hand above the head string and every
/// legal ball above it too, spots the one nearest the string (WPA 1.6).
/// Available when `state().eightBall.canSpot` is true.
#[wasm_bindgen(js_name = spotRequest)]
pub fn spot_request() -> Result<(), JsError> {
    game(GameCommand::SpotRequest)
}

/// Sets the charged power, `0..=1`, as a slider would.
#[wasm_bindgen(js_name = setPower)]
pub fn set_power(power: f64) -> Result<(), JsError> {
    game(GameCommand::SetPower(power))
}

/// Shoots with the current aim, power and spin. Ignored while balls move or
/// if the power is too low to be intentional.
#[wasm_bindgen]
pub fn shoot() -> Result<(), JsError> {
    game(GameCommand::Shoot)
}

/// Sets where the tip strikes the cue ball, in ball radii: `x` is side spin
/// (positive right), `y` follow (positive) or draw (negative).
#[wasm_bindgen(js_name = setTip)]
pub fn set_tip(x: f64, y: f64) -> Result<(), JsError> {
    game(GameCommand::SetTip(DVec2::new(x, y)))
}

/// Sets the visual theme: `classic`, `studio` or `contrast`.
#[wasm_bindgen(js_name = setTheme)]
pub fn set_theme(name: &str) -> Result<(), JsError> {
    let theme = Theme::parse(name).ok_or_else(|| JsError::new("UNKNOWN_THEME"))?;
    game(GameCommand::SetTheme(theme))
}

/// Sets the aiming assistance: `off`, `basic` or `assisted`.
#[wasm_bindgen(js_name = setAssists)]
pub fn set_assists(name: &str) -> Result<(), JsError> {
    let assists = Assists::parse(name).ok_or_else(|| JsError::new("UNKNOWN_ASSISTS"))?;
    game(GameCommand::SetAssists(assists))
}

/// Tells the engine how many CSS pixels at the top and bottom of the canvas
/// the host's controls cover. The cloth is framed clear of them; only the
/// rails may run underneath.
#[wasm_bindgen(js_name = setViewInsets)]
pub fn set_view_insets(top: f64, bottom: f64) -> Result<(), JsError> {
    let clean = |v: f64| {
        if v.is_finite() {
            v.max(0.0) as f32
        } else {
            0.0
        }
    };
    game(GameCommand::SetViewInsets(ViewInsets {
        top: clean(top),
        bottom: clean(bottom),
    }))
}

/// Sets the graphics quality: `auto`, `high`, `medium` or `low`. Quality
/// changes anti-aliasing and shadows, never the simulation.
#[wasm_bindgen(js_name = setQuality)]
pub fn set_quality(name: &str) -> Result<(), JsError> {
    let quality = Quality::parse(name).ok_or_else(|| JsError::new("UNKNOWN_QUALITY"))?;
    game(GameCommand::SetQuality(quality))
}

/// Reduced motion: when on, the cue strikes without its short swing. The
/// wrapper follows the visitor's `prefers-reduced-motion` setting.
#[wasm_bindgen(js_name = setReducedMotion)]
pub fn set_reduced_motion(on: bool) -> Result<(), JsError> {
    game(GameCommand::SetReducedMotion(on))
}

/// Shows or hides the developer overlay.
#[wasm_bindgen(js_name = toggleDevOverlay)]
pub fn toggle_dev_overlay() -> Result<(), JsError> {
    game(GameCommand::ToggleDevHud)
}

/// Turns sound on or off and sets its volume, `0..=1`. Takes effect
/// immediately, also before audio is unlocked.
#[wasm_bindgen(js_name = setAudio)]
pub fn set_audio(enabled: bool, volume: f64) {
    audio::configure(enabled, volume as f32);
}

/// Creates or resumes the audio context. Call it from inside a user gesture
/// (`pointerdown`, `keydown`), as browsers require.
#[wasm_bindgen(js_name = unlockAudio)]
pub fn unlock_audio() -> Result<(), JsValue> {
    audio::unlock()
}

/// Stops the game loop and releases the GPU surface. Idempotent.
///
/// The page counts as destroyed at once, so a [`mount`] that comes before
/// the engine's last frame (a hidden tab runs none) already gets
/// `RELOAD_REQUIRED`.
#[wasm_bindgen]
pub fn destroy() {
    if lifecycle() == Lifecycle::Mounted {
        let _ = send(Command::Destroy);
        set_lifecycle(Lifecycle::Destroyed);
    }
}

/// Number of frames the engine has run since it was mounted.
#[wasm_bindgen(js_name = frameCount)]
pub fn frame_count() -> u32 {
    FRAMES.load(Ordering::Relaxed)
}

/// The engine's state as of its last frame, as a plain object:
/// `{ mode, power, tipX, tipY, atRest, inHand, handRegion, canUndo, paused,
/// theme, assists, quality, graphics, shots, ballsInPlay, minPower,
/// eightBall }`. `quality` is the setting and `graphics` the tier it renders
/// (`high`, `medium` or `low`). `mode` is
/// `free-play` or `eight-ball`; `eightBall` is the match (players, score,
/// shooter, stage, call, decision…) or `null`. `null` before the first frame.
#[wasm_bindgen]
pub fn state() -> JsValue {
    let Some(s) = lock(&SNAPSHOT).clone() else {
        return JsValue::NULL;
    };
    let balls: js_sys::Array = s.balls_in_play.iter().map(|n| JsValue::from(*n)).collect();
    let region = s.hand_region.map_or(JsValue::NULL, |r| {
        JsValue::from_str(match r {
            HandRegion::Anywhere => "anywhere",
            HandRegion::AboveHeadString => "above-head-string",
        })
    });
    let mode = if s.eight_ball.is_some() {
        "eight-ball"
    } else {
        "free-play"
    };
    object(&[
        ("mode", mode.into()),
        ("power", s.power.into()),
        ("tipX", s.tip.x.into()),
        ("tipY", s.tip.y.into()),
        ("atRest", s.at_rest.into()),
        ("inHand", s.in_hand.into()),
        ("handRegion", region),
        ("canUndo", s.can_undo.into()),
        ("paused", s.paused.into()),
        ("theme", s.theme.into()),
        ("assists", s.assists.into()),
        ("quality", s.quality.into()),
        ("graphics", s.graphics.into()),
        ("shots", s.shots.into()),
        ("ballsInPlay", balls.into()),
        ("minPower", MIN_SHOT_POWER.into()),
        (
            "eightBall",
            s.eight_ball
                .as_ref()
                .map_or(JsValue::NULL, MatchSnapshot::to_js),
        ),
    ])
}

/// Registers the function that receives `(name, detail)` for every event.
/// Passing `null` or `undefined` removes it.
#[wasm_bindgen(js_name = setListener)]
pub fn set_listener(listener: Option<js_sys::Function>) {
    LISTENER.with(|l| *l.borrow_mut() = listener);
}

fn emit(name: &str, detail: &JsValue) {
    // Release the borrow before calling out: a listener may re-enter this
    // module (for example `setListener(null)` while handling `destroyed`).
    let listener = LISTENER.with(|l| l.borrow().clone());
    if let Some(f) = listener {
        // A throwing host listener must not take the game down.
        let _ = f.call2(&JsValue::NULL, &JsValue::from_str(name), detail);
    }
}

fn install_waker(proxy: Res<EventLoopProxyWrapper>) {
    let proxy = (**proxy).clone();
    *lock(&WAKER) = Some(Box::new(move || {
        let _ = proxy.send_event(WinitUserEvent::WakeUp);
    }));
}

fn apply_commands(
    mut paused: ResMut<Paused>,
    mut settings: ResMut<WinitSettings>,
    mut exit: MessageWriter<AppExit>,
    mut game: MessageWriter<GameCommand>,
) {
    let commands: Vec<_> = lock(&INBOX).drain(..).collect();
    for command in commands {
        match command {
            Command::Pause(reason) if !paused.0 => {
                paused.0 = true;
                let sleep = UpdateMode::reactive_low_power(Duration::MAX);
                *settings = WinitSettings {
                    focused_mode: sleep,
                    unfocused_mode: sleep,
                };
                audio::set_running(false);
                emit("paused", &object(&[("reason", reason.into())]));
            }
            Command::Resume(reason) if paused.0 => {
                paused.0 = false;
                *settings = WinitSettings::game();
                audio::set_running(true);
                emit("resumed", &object(&[("reason", reason.into())]));
            }
            Command::Destroy => {
                exit.write(AppExit::Success);
                *lock(&WAKER) = None;
                audio::set_running(false);
                set_lifecycle(Lifecycle::Destroyed);
                emit("destroyed", &JsValue::UNDEFINED);
            }
            Command::Game(g) => {
                game.write(g);
            }
            Command::Pause(_) | Command::Resume(_) => {}
        }
    }
}

/// Builds a plain JS object from `(key, value)` pairs.
pub(crate) fn object(entries: &[(&str, JsValue)]) -> JsValue {
    let o = js_sys::Object::new();
    for (key, value) in entries {
        // Setting a property on a fresh plain object cannot fail.
        let _ = js_sys::Reflect::set(&o, &JsValue::from_str(key), value);
    }
    o.into()
}

/// The `shot-end` payload: what a host might show or log about a shot.
fn shot_summary(report: &ShotReport) -> JsValue {
    let first: js_sys::Array = report
        .first_contacts()
        .iter()
        .map(|b| JsValue::from(b.number()))
        .collect();
    let pocketed: js_sys::Array = report
        .pocketed()
        .map(|(b, p)| {
            object(&[
                ("ballId", JsValue::from(b.number())),
                ("pocketId", JsValue::from_str(p.name())),
            ])
        })
        .collect();
    object(&[
        ("shotId", JsValue::from(report.shot_id)),
        ("firstContacts", first.into()),
        ("pocketed", pocketed.into()),
        ("cueBallPocketed", JsValue::from(report.cue_ball_pocketed())),
        (
            "objectBallsToCushion",
            JsValue::from(report.object_balls_driven_to_a_rail().len() as u32),
        ),
    ])
}

/// Tells the wrapper which tier renders whenever it changes, in an internal
/// `tier` event it does not pass on. The low tier also renders fewer pixels,
/// which only the page can arrange (`render.pixelRatioCap`).
fn report_tier(graphics: Res<Graphics>, mut reported: Local<Option<&'static str>>) {
    let tier = graphics.tier.name();
    if *reported != Some(tier) {
        *reported = Some(tier);
        emit("tier", &JsValue::from_str(tier));
    }
}

/// Translates engine happenings into host events and sound.
///
/// Per shot the host sees exactly one `shot-start` and one `shot-end`, with
/// any number of `ball-pocketed` in between (spec §16.4). A shot interrupted
/// by an undo or re-rack still ends, with `{ abandoned: true }`.
fn emit_events(
    sim: Res<Simulation>,
    mode: Res<Mode>,
    mut contacts: MessageReader<Contact>,
    mut finished: MessageReader<ShotFinished>,
    mut game_events: MessageReader<GameEvent>,
    mut started: Local<bool>,
    mut in_shot: Local<bool>,
) {
    FRAMES.fetch_add(1, Ordering::Relaxed);
    if !*started {
        *started = true;
        emit("ready", &JsValue::from_str(&version()));
    }
    // A very soft shot can start and finish within one frame, so a finished
    // report also counts as the start of a shot. Then the rules have already
    // ruled, and the shooter is the ruling's.
    let reports: Vec<&ShotReport> = finished.read().map(|f| &f.0).collect();
    if !*in_shot && (!sim.world.is_at_rest() || !reports.is_empty()) {
        *in_shot = true;
        let shot_id = reports.first().map_or(sim.world.shots(), |r| r.shot_id);
        let shooter = mode
            .eight_ball()
            .map(|session| match (&session.last, reports.first()) {
                (Some(ruling), Some(_)) => ruling.shooter,
                _ => session.game.shooter(),
            });
        emit(
            "shot-start",
            &object(&[
                ("shotId", shot_id.into()),
                (
                    "shooter",
                    shooter.map_or(JsValue::NULL, |s| JsValue::from_str(s.name())),
                ),
            ]),
        );
    }
    let mut voiced = 0;
    for Contact(event) in contacts.read() {
        if voiced < audio::MAX_PER_FRAME && audio::play_event(&event.kind) {
            voiced += 1;
        }
        if let cue_physics::SimEventKind::Pocketed { ball, pocket } = event.kind {
            emit(
                "ball-pocketed",
                &object(&[
                    ("ballId", JsValue::from(ball.number())),
                    ("pocketId", JsValue::from_str(pocket.name())),
                ]),
            );
        }
    }
    if let Some(report) = reports.last() {
        emit("shot-end", &shot_summary(report));
        *in_shot = false;
    } else if *in_shot && sim.world.is_at_rest() {
        emit("shot-end", &object(&[("abandoned", JsValue::TRUE)]));
        *in_shot = false;
    }
    let reason = |why: &str| object(&[("reason", JsValue::from_str(why))]);
    for event in game_events.read() {
        match event {
            GameEvent::Racked { cleared } => {
                emit("racked", &object(&[("cleared", JsValue::from(*cleared))]));
            }
            GameEvent::Undone => emit("undo", &JsValue::UNDEFINED),
            GameEvent::NewGame => {
                let name = if mode.eight_ball().is_some() {
                    "eight-ball"
                } else {
                    "free-play"
                };
                emit("new-game", &object(&[("mode", JsValue::from_str(name))]));
            }
            GameEvent::Ruled => {
                if let Some(session) = mode.eight_ball()
                    && let Some(ruling) = &session.last
                {
                    // One cue per ruling: the end of a rack outranks a foul,
                    // which outranks a plain change of turn.
                    let cue = match &ruling.next {
                        Next::Over(_) => Some(Sound::Win),
                        _ if !ruling.fouls.is_empty() => Some(Sound::Foul),
                        Next::Pass { .. } | Next::Decide(_) => Some(Sound::Turn),
                        Next::Continue => None,
                    };
                    if let Some(cue) = cue {
                        audio::play_cue(cue);
                    }
                    for (name, detail) in eight::ruling_events(session, ruling, session.last_call) {
                        emit(name, &detail);
                    }
                }
            }
            GameEvent::Decided { choice } => {
                if let Some(session) = mode.eight_ball() {
                    emit("decided", &eight::decided(session, *choice));
                }
            }
            GameEvent::Spotted { ball } => {
                emit(
                    "ball-spotted",
                    &object(&[("ballId", JsValue::from(ball.number()))]),
                );
            }
            GameEvent::ShotRefused(why) => emit("shot-refused", &reason(why)),
            GameEvent::CallRefused(why) => emit("call-refused", &reason(why)),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn publish_state(
    sim: Res<Simulation>,
    aim: Res<Aim>,
    mode: Res<Mode>,
    paused: Res<Paused>,
    theme: Res<Theme>,
    assists: Res<Assists>,
    graphics: Res<Graphics>,
    last: Res<LastShot>,
) {
    let snapshot = Snapshot {
        power: aim.power,
        tip: aim.tip,
        at_rest: sim.world.is_at_rest(),
        in_hand: sim.in_hand(),
        hand_region: sim.hand_region(),
        can_undo: sim.can_undo(),
        paused: paused.0,
        theme: theme.name(),
        assists: assists.name(),
        quality: graphics.setting.name(),
        graphics: graphics.tier.name(),
        shots: last.0.as_ref().map_or(0, |r| r.shot_id),
        balls_in_play: sim.world.balls_in_play().map(|b| b.number()).collect(),
        eight_ball: mode
            .eight_ball()
            .map(|session| MatchSnapshot::new(session, &sim)),
    };
    let mut slot = lock(&SNAPSHOT);
    if slot.as_ref() != Some(&snapshot) {
        *slot = Some(snapshot);
    }
}
