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
//! - **Pausing stops rendering.** A paused instance switches winit to
//!   reactive mode and sleeps until the host resumes it, so a backgrounded or
//!   hidden game costs no CPU or GPU.
//!
//! The browser is single-threaded for this module, which is why the listener
//! lives in a thread-local.

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
use cue_core::ShotReport;
use cue_game::{Contact, CuePlugin, Paused, ShotFinished, Simulation};
use cue_physics::SimEventKind;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Pause,
    Resume,
    Rerack,
    Destroy,
}

/// Lifecycle of the page's single instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Idle,
    Mounted,
    /// Bevy's event loop cannot be created twice in one page; see [`mount`].
    Destroyed,
}

static LIFECYCLE: Mutex<Lifecycle> = Mutex::new(Lifecycle::Idle);
static INBOX: Mutex<Vec<Command>> = Mutex::new(Vec::new());
/// Wakes a sleeping (paused) event loop so queued commands are seen.
static WAKER: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
/// Frames run by the engine, so hosts and tests can verify that a paused
/// instance really stops working.
static FRAMES: AtomicU32 = AtomicU32::new(0);

thread_local! {
    static LISTENER: RefCell<Option<js_sys::Function>> = const { RefCell::new(None) };
}

fn lifecycle() -> Lifecycle {
    *LIFECYCLE.lock().unwrap_or_else(PoisonError::into_inner)
}

fn set_lifecycle(state: Lifecycle) {
    *LIFECYCLE.lock().unwrap_or_else(PoisonError::into_inner) = state;
}

fn send(command: Command) -> Result<(), JsError> {
    if lifecycle() != Lifecycle::Mounted {
        return Err(JsError::new("INSTANCE_NOT_MOUNTED"));
    }
    INBOX
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(command);
    if let Some(wake) = WAKER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
    {
        wake();
    }
    Ok(())
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
/// CUE in an iframe (decision recorded in ADR-0005).
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
        .add_systems(Last, emit_events)
        .run();
    Ok(())
}

/// Pauses simulation, input and rendering.
#[wasm_bindgen]
pub fn pause() -> Result<(), JsError> {
    send(Command::Pause)
}

/// Resumes a paused instance.
#[wasm_bindgen]
pub fn resume() -> Result<(), JsError> {
    send(Command::Resume)
}

/// Starts over with a fresh rack.
#[wasm_bindgen]
pub fn rerack() -> Result<(), JsError> {
    send(Command::Rerack)
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
    *WAKER.lock().unwrap_or_else(PoisonError::into_inner) = Some(Box::new(move || {
        let _ = proxy.send_event(WinitUserEvent::WakeUp);
    }));
}

fn apply_commands(
    mut paused: ResMut<Paused>,
    mut sim: ResMut<Simulation>,
    mut settings: ResMut<WinitSettings>,
    mut exit: MessageWriter<AppExit>,
) {
    let commands: Vec<_> = INBOX
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .drain(..)
        .collect();
    for command in commands {
        match command {
            Command::Pause if !paused.0 => {
                paused.0 = true;
                let sleep = UpdateMode::reactive_low_power(Duration::MAX);
                *settings = WinitSettings {
                    focused_mode: sleep,
                    unfocused_mode: sleep,
                };
                emit("paused", &JsValue::UNDEFINED);
            }
            Command::Resume if paused.0 => {
                paused.0 = false;
                *settings = WinitSettings::game();
                emit("resumed", &JsValue::UNDEFINED);
            }
            Command::Rerack => sim.rerack(),
            Command::Destroy => {
                exit.write(AppExit::Success);
                *WAKER.lock().unwrap_or_else(PoisonError::into_inner) = None;
                set_lifecycle(Lifecycle::Destroyed);
                emit("destroyed", &JsValue::UNDEFINED);
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

/// Translates simulation state changes into host events.
///
/// Per shot the host sees exactly one `shot-start` and one `shot-end`, with
/// any number of `ball-pocketed` in between (spec §16.4). A shot abandoned
/// by a re-rack still ends, with `{ abandoned: true }`.
fn emit_events(
    sim: Res<Simulation>,
    mut contacts: MessageReader<Contact>,
    mut finished: MessageReader<ShotFinished>,
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
    for Contact(event) in contacts.read() {
        if let SimEventKind::Pocketed { ball, pocket } = event.kind {
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
}
