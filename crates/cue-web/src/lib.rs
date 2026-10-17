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
//!   small snapshot (aim, power, ball in hand…) that the host's controls read
//!   cheaply without reaching into the ECS.
//! - **Pausing stops rendering.** A paused instance switches winit to
//!   reactive mode and sleeps until the host resumes it, so a backgrounded or
//!   hidden game costs no CPU or GPU.
//!
//! The browser is single-threaded for this module, which is why the listener
//! and the audio engine live in thread-locals.

mod audio;

use std::{
    cell::RefCell,
    sync::{
        Mutex, PoisonError,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use bevy::{
    asset::AssetMetaCheck,
    log::{Level, LogPlugin},
    prelude::*,
    winit::{EventLoopProxyWrapper, UpdateMode, WinitSettings, WinitUserEvent},
};
use cue_core::{ShotReport, glam::DVec2};
use cue_game::{
    Aim, Assists, Contact, CuePlugin, GameCommand, GameEvent, LastShot, MIN_SHOT_POWER, Paused,
    ShotFinished, Simulation, Theme,
};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    Pause,
    Resume,
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
    can_undo: bool,
    paused: bool,
    theme: &'static str,
    assists: &'static str,
    shots: u32,
    balls_in_play: Vec<u8>,
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
/// The canvas is sized to its parent element. Returns immediately; the game
/// runs on the browser's animation frames and emits `ready` once its first
/// frame has run.
///
/// Errors with `INSTANCE_ALREADY_MOUNTED` if an instance is live, or
/// `RELOAD_REQUIRED` after [`destroy`]: winit allows one event loop per
/// page, so a destroyed instance cannot be re-created without a reload. Hosts
/// that need repeated mounting should pause and resume instead, or isolate
/// CUE in an iframe (ADR-0006).
#[wasm_bindgen]
pub fn mount(canvas_selector: &str) -> Result<(), JsError> {
    match lifecycle() {
        Lifecycle::Mounted => return Err(JsError::new("INSTANCE_ALREADY_MOUNTED")),
        Lifecycle::Destroyed => return Err(JsError::new("RELOAD_REQUIRED")),
        Lifecycle::Idle => {}
    }
    set_lifecycle(Lifecycle::Mounted);

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some(canvas_selector.to_owned()),
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
        .add_systems(Last, (emit_events, publish_state))
        .run();
    Ok(())
}

/// Pauses simulation, input, sound and rendering.
#[wasm_bindgen]
pub fn pause() -> Result<(), JsError> {
    send(Command::Pause)
}

/// Resumes a paused instance.
#[wasm_bindgen]
pub fn resume() -> Result<(), JsError> {
    send(Command::Resume)
}

/// Racks the balls again. Undoable.
#[wasm_bindgen]
pub fn rerack() -> Result<(), JsError> {
    game(GameCommand::Rerack)
}

/// Undoes the last shot or re-rack.
#[wasm_bindgen]
pub fn undo() -> Result<(), JsError> {
    game(GameCommand::Undo)
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
#[wasm_bindgen]
pub fn destroy() {
    if lifecycle() == Lifecycle::Mounted {
        let _ = send(Command::Destroy);
    }
}

/// Number of frames the engine has run since it was mounted.
#[wasm_bindgen(js_name = frameCount)]
pub fn frame_count() -> u32 {
    FRAMES.load(Ordering::Relaxed)
}

/// The engine's state as of its last frame, as a plain object:
/// `{ power, tipX, tipY, atRest, inHand, canUndo, paused, theme, assists,
/// shots, ballsInPlay, minPower }`. `null` before the first frame.
#[wasm_bindgen]
pub fn state() -> JsValue {
    let Some(s) = lock(&SNAPSHOT).clone() else {
        return JsValue::NULL;
    };
    let balls: js_sys::Array = s.balls_in_play.iter().map(|n| JsValue::from(*n)).collect();
    object(&[
        ("power", s.power.into()),
        ("tipX", s.tip.x.into()),
        ("tipY", s.tip.y.into()),
        ("atRest", s.at_rest.into()),
        ("inHand", s.in_hand.into()),
        ("canUndo", s.can_undo.into()),
        ("paused", s.paused.into()),
        ("theme", s.theme.into()),
        ("assists", s.assists.into()),
        ("shots", s.shots.into()),
        ("ballsInPlay", balls.into()),
        ("minPower", MIN_SHOT_POWER.into()),
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
            Command::Pause if !paused.0 => {
                paused.0 = true;
                let sleep = UpdateMode::reactive_low_power(Duration::MAX);
                *settings = WinitSettings {
                    focused_mode: sleep,
                    unfocused_mode: sleep,
                };
                audio::set_running(false);
                emit("paused", &JsValue::UNDEFINED);
            }
            Command::Resume if paused.0 => {
                paused.0 = false;
                *settings = WinitSettings::game();
                audio::set_running(true);
                emit("resumed", &JsValue::UNDEFINED);
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
            Command::Pause | Command::Resume => {}
        }
    }
}

/// Builds a plain JS object from `(key, value)` pairs.
fn object(entries: &[(&str, JsValue)]) -> JsValue {
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
            JsValue::from(report.object_balls_touching_cushion().len() as u32),
        ),
    ])
}

/// Translates engine happenings into host events and sound.
///
/// Per shot the host sees exactly one `shot-start` and one `shot-end`, with
/// any number of `ball-pocketed` in between (spec §16.4). A shot interrupted
/// by an undo or re-rack still ends, with `{ abandoned: true }`.
fn emit_events(
    sim: Res<Simulation>,
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
    // report also counts as the start of a shot.
    let reports: Vec<&ShotReport> = finished.read().map(|f| &f.0).collect();
    if !*in_shot && (!sim.world.is_at_rest() || !reports.is_empty()) {
        *in_shot = true;
        emit("shot-start", &JsValue::UNDEFINED);
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
    for event in game_events.read() {
        match event {
            GameEvent::Racked { cleared } => {
                emit("racked", &object(&[("cleared", JsValue::from(*cleared))]));
            }
            GameEvent::Undone => emit("undo", &JsValue::UNDEFINED),
        }
    }
}

fn publish_state(
    sim: Res<Simulation>,
    aim: Res<Aim>,
    paused: Res<Paused>,
    theme: Res<Theme>,
    assists: Res<Assists>,
    last: Res<LastShot>,
) {
    let snapshot = Snapshot {
        power: aim.power,
        tip: aim.tip,
        at_rest: sim.world.is_at_rest(),
        in_hand: sim.in_hand(),
        can_undo: sim.can_undo(),
        paused: paused.0,
        theme: theme.name(),
        assists: assists.name(),
        shots: last.0.as_ref().map_or(0, |r| r.shot_id),
        balls_in_play: sim.world.balls_in_play().map(|b| b.number()).collect(),
    };
    let mut slot = lock(&SNAPSHOT);
    if slot.as_ref() != Some(&snapshot) {
        *slot = Some(snapshot);
    }
}
