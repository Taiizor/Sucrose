//! Renderer mode (`sucrose --render <slot>`): connects to the service, receives its assignment and keeps a surface
//! behind the desktop icons until it is told to stop.

use std::{
    path::Path,
    process::ExitCode,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context as _, bail};
use sucrose_core::compatible::DataGroup;
use sucrose_core::settings::InputType;
use sucrose_engine::{
    Engine, EngineClock, EngineError, EngineId, PauseLevel, Started,
    sync::{Follower, SyncAction, drift_s},
};
use sucrose_engine_cef::engine::SurfaceHooks;
use sucrose_ipc::{
    Connection, IpcError, connect,
    protocol::{Assignment, ClockTick, PROTOCOL_VERSION, PlaybackSettings, Role, ToRenderer, ToService},
};
use sucrose_platform::{
    DesktopSurface, DpiAwareness, EventLoop, Flow, LoopEnd, LoopEvent, LoopProxy, PlacementVerdict, Platform, RectPx,
    input::{InputEvent, InputFilter, cursor_inside, devices, is_desktop_class, should_forward},
    monotonic_ns,
};

/// How often and how long the renderer retries while the service is still opening its endpoint.
const CONNECT_ATTEMPTS: u32 = 20;
const CONNECT_PAUSE: Duration = Duration::from_millis(100);

/// Exit code for "the desktop surface could not be attached", reported to the service like any other start failure.
const EXIT_START_FAILURE: u8 = crate::supervisor::START_FAILED;

/// The hint for a failure that has no repair step of its own.
const CHECK_THE_LOG: &str = "Check the log for details.";

/// Exit code for "the message loop failed": the service counts it as a crash and starts a new renderer within the
/// restart budget, instead of reading a vanished wallpaper as a normal stop.
const EXIT_LOOP_FAILED: u8 = crate::supervisor::LOOP_FAILED;

/// Exit code for "libmpv is missing or cannot be loaded" (D-20).
const EXIT_ENGINE_MISSING: u8 = crate::supervisor::ENGINE_MISSING;

/// Exit code for "Windows destroyed the desktop surface" (a WM_DESTROY this renderer did not cause): the service
/// starts a new renderer, which does not count as a crash. A killed explorer.exe sends no such message, and a shell
/// restart is handled by the service, which closes and replaces every renderer on TaskbarCreated (D-11), so nothing
/// waits for this exit.
const EXIT_SURFACE_LOST: u8 = crate::supervisor::SURFACE_LOST;

/// How often the master publishes its clock when nothing happened, and how often the clock thread looks for work.
const CLOCK_PERIOD: Duration = Duration::from_secs(1);
const CLOCK_POLL: Duration = Duration::from_millis(100);

/// What the threads of a renderer share about its part in the master/follower scheme (D-02).
#[derive(Debug)]
struct SyncState {
    /// Whether this renderer is the master clock.
    master: AtomicBool,
    /// The volume the master plays at: the assignment's, or the service's latest SetVolume.
    volume: AtomicU8,
    /// Set when the clock thread should publish a reading without waiting for its period.
    kick: AtomicBool,
    /// Whether the video loops, so that a follower treats the loop end as a wrap and not as drift. A new source can
    /// change it.
    looping: AtomicBool,
    /// Set when a new source was loaded: the clock thread tells the service again that the wallpaper plays once the
    /// first frame of the new file is shown.
    reloaded: AtomicBool,
    /// Set by the clock thread when the file could not be played: the renderer exits with the start-failure code.
    start_failed: AtomicBool,
}

/// The newest rectangle the service asked for, waiting for the thread that owns the surface to apply it.
type PendingRect = Arc<Mutex<Option<RectPx>>>;

/// Runs the renderer for `slot` until the service closes the connection or asks it to stop.
pub fn run(slot: u8) -> anyhow::Result<ExitCode> {
    // Before any window or thread exists: engine threads inherit the process default (A15).
    let awareness = sucrose_platform::init_dpi_awareness();
    if awareness == DpiAwareness::PerMonitorV2 {
        tracing::info!(?awareness, "DPI awareness");
    } else {
        tracing::error!(
            ?awareness,
            "the renderer is not Per-Monitor-V2 aware, wallpaper rectangles may be scaled"
        );
    }

    let endpoint = sucrose_ipc::service_endpoint()?;
    let mut connection = connect_with_retry(&endpoint.name)?;
    connection.send(&ToService::Hello {
        protocol_version: PROTOCOL_VERSION,
        pid: std::process::id(),
        slot,
    })?;
    tracing::info!(slot, protocol_version = PROTOCOL_VERSION, "Hello sent to the service");

    let Some(assignment) = wait_for_assignment(&mut connection)? else {
        return Ok(ExitCode::SUCCESS);
    };
    let [left, top, right, bottom] = assignment.rect;
    let rect = RectPx {
        left,
        top,
        right,
        bottom,
    };
    tracing::info!(slot = assignment.slot, monitor = %assignment.monitor, ?rect, "assignment received");

    let event_loop = EventLoop::new().context("cannot prepare the message loop")?;
    let surface = match DesktopSurface::attach(rect) {
        Ok(surface) => surface,
        Err(error) => {
            tracing::error!(%error, "cannot attach the desktop surface");
            report_failure(
                &mut connection,
                format!("the desktop surface could not be attached: {error}"),
                CHECK_THE_LOG,
            );
            return Ok(ExitCode::from(EXIT_START_FAILURE));
        }
    };

    log_video_adapter();

    // The service names the engine; the renderer starts it or refuses one its build does not contain (T-04-12).
    let id = EngineId::from_legacy(assignment.engine);
    let proxy = event_loop.proxy();
    // An engine without a clock (a page) reports its own first frame, from the thread that owns the surface, through
    // its own handle on the connection.
    let playing_writer = connection.try_clone().ok().map(|writer| Arc::new(Mutex::new(writer)));
    let on_playing: Arc<dyn Fn() + Send + Sync> = {
        let writer = playing_writer.clone();
        Arc::new(move || {
            if let Some(writer) = &writer {
                tell_playing(&mut writer.lock().unwrap_or_else(PoisonError::into_inner));
            }
        })
    };
    let on_data_wanted: Arc<dyn Fn(Vec<DataGroup>) + Send + Sync> = {
        let writer = playing_writer.clone();
        Arc::new(move |groups| {
            if let Some(writer) = &writer {
                tell_data_wanted(&mut writer.lock().unwrap_or_else(PoisonError::into_inner), groups);
            }
        })
    };
    let hooks = || SurfaceHooks {
        proxy: proxy.clone(),
        timers: event_loop.timers(),
        on_playing,
        on_data_wanted,
    };

    // Started here, on the thread that pumps the surface's messages: mpv hooks that thread to resize its child, and
    // CEF is bound to it.
    let Started { engine, pump } = match start_engine(id, surface.native_handle(), &assignment, hooks) {
        Ok(started) => started,
        Err(error) => {
            let (reason, hint, code) = engine_failure(&error);
            tracing::error!(%error, "cannot start the engine");
            report_failure(&mut connection, reason, hint);
            return Ok(ExitCode::from(code));
        }
    };
    tracing::info!(engine = engine.name(), "engine started");
    if engine.clock().is_none() && playing_writer.is_none() {
        // Nothing would ever tell the service that this wallpaper plays.
        tracing::error!("cannot keep a write handle for the playing report");
        if let Some(pump) = pump {
            pump.close();
        }
        report_failure(
            &mut connection,
            "the renderer could not prepare its connection".to_owned(),
            CHECK_THE_LOG,
        );
        return Ok(ExitCode::from(EXIT_START_FAILURE));
    }
    drop(playing_writer);
    let mut pump = pump;
    let sync = Arc::new(SyncState {
        master: AtomicBool::new(false),
        volume: AtomicU8::new(assignment.settings.volume),
        kick: AtomicBool::new(false),
        looping: AtomicBool::new(assignment.settings.looping),
        reloaded: AtomicBool::new(false),
        start_failed: AtomicBool::new(false),
    });
    // The clock thread tells the service the wallpaper plays once the first frame is shown, and publishes the master's
    // clock. It is started before the connection moves into the reader thread, and joined before the engine is
    // dropped, so the engine still ends before the surface it is embedded in. An engine without a clock reports
    // its own first frame and has no clock thread.
    let has_clock = engine.clock().is_some();
    let clock_stop = Arc::new(AtomicBool::new(false));
    let clock_thread = match connection.try_clone() {
        Ok(writer) if has_clock => {
            let clock_engine = Arc::clone(&engine);
            let clock_sync = Arc::clone(&sync);
            let stop = Arc::clone(&clock_stop);
            let clock_proxy = proxy.clone();
            let spawned = thread::Builder::new()
                .name("clock".to_owned())
                .spawn(move || publish_clock(writer, clock_engine.as_ref(), &clock_sync, &stop, &clock_proxy));
            match spawned {
                Ok(handle) => Some(handle),
                Err(error) => {
                    tracing::warn!(%error, "cannot start the clock thread, this renderer cannot be the master");
                    None
                }
            }
        }
        Ok(_) => None,
        Err(error) => {
            tracing::warn!(%error, "cannot keep a write handle for the clock, this renderer cannot be the master");
            None
        }
    };
    if has_clock && clock_thread.is_none() {
        // Nobody else will announce it: without a clock thread the wallpaper is reported as playing right away.
        tell_playing(&mut connection);
    }
    // The reader thread only waits for the end of the session; the loop below must stay on this thread because the
    // surface was created here.
    let pending_rect = PendingRect::default();
    let reader_engine = Arc::clone(&engine);
    let reader_rect = Arc::clone(&pending_rect);
    let reader_proxy = proxy.clone();
    let reader_sync = Arc::clone(&sync);
    let pause_level = PauseLevel::from(assignment.settings.pause);
    thread::Builder::new()
        .name("ipc-reader".to_owned())
        .spawn(move || {
            wait_for_close(
                &mut connection,
                reader_engine.as_ref(),
                &reader_rect,
                &reader_proxy,
                &reader_sync,
                pause_level,
            );
            // Released before the loop is told to stop, so that the main thread drops the last handle and the
            // engine ends before the surface it is embedded in.
            drop(reader_engine);
            proxy.quit();
        })
        .context("cannot start the reader thread")?;
    // Raw input only for an engine whose page uses it and only for the devices the InputType leaves on: a renderer that
    // plays a video registers nothing (D-008).
    let input_filter = InputFilter {
        input_type: assignment.input.input_type,
        desktop_only: assignment.input.desktop_only,
    };
    let (want_mouse, want_keyboard) = devices(input_filter.input_type);
    let forwards_input = engine.takes_input() && (want_mouse || want_keyboard);
    if forwards_input {
        match event_loop.register_raw_input(want_mouse, want_keyboard) {
            Ok(()) => tracing::info!(mouse = want_mouse, keyboard = want_keyboard, "raw input registered"),
            Err(error) => tracing::warn!(%error, "cannot register raw input, the page gets no mouse or keyboard"),
        }
    }
    let mut current_rect = assignment.rect;
    let mut desktop_focus = DesktopFocus::new();
    // A keyboard-only wallpaper gets one click before its first key so the page has the keyboard focus (legacy).
    let mut focus_click_due = input_filter.input_type == InputType::OnlyKeyboard;
    // A wake means a new rectangle or, for an engine with a pump, queued commands: the surface is moved here because
    // only this thread may touch it, and the pump is the only thing that calls an engine bound to this thread. The
    // loop ends when the reader thread asks for it, or when Windows destroys the surface (the surface posts the quit
    // itself). A killed explorer.exe sends no such message; the service replaces the renderers on TaskbarCreated.
    let loop_end = event_loop.run(|event| match event {
        LoopEvent::Wake => {
            let rect = pending_rect.lock().unwrap_or_else(PoisonError::into_inner).take();
            if let Some(rect) = rect {
                current_rect = [rect.left, rect.top, rect.right, rect.bottom];
                move_surface(&surface, rect);
                if let Some(pump) = pump.as_mut() {
                    pump.set_rect([rect.left, rect.top, rect.right, rect.bottom]);
                }
            }
            if let Some(pump) = pump.as_mut() {
                pump.on_wake();
            }
            Flow::Continue
        }
        LoopEvent::Timer(id) => {
            if let Some(pump) = pump.as_mut() {
                pump.on_timer(id);
            }
            Flow::Continue
        }
        LoopEvent::Quit => Flow::Exit,
        // Only this renderer's own monitor gets input: the cursor must be inside its rectangle (WAPI-07).
        LoopEvent::Input(event) => {
            if forwards_input {
                let position = event.position().or_else(sucrose_platform::cursor_position);
                let inside = position.is_some_and(|position| cursor_inside(current_rect, position));
                // Only looked up while InputDesktop is on, and at most once a second.
                let desktop = !input_filter.desktop_only || desktop_focus.is_desktop();
                if should_forward(&input_filter, &event, inside, desktop) {
                    if focus_click_due && matches!(event, InputEvent::Key { .. }) {
                        focus_click_due = false;
                        engine.forward_input(&InputEvent::FocusClick);
                    }
                    engine.forward_input(&event);
                }
            }
            Flow::Continue
        }
        // The service watches the monitors and the shell; a renderer learns about both through its own surface.
        LoopEvent::DisplayChanged | LoopEvent::ShellRestarted => Flow::Continue,
    });
    let surface_lost = surface.destroyed_externally();
    clock_stop.store(true, Ordering::SeqCst);
    if let Some(thread) = clock_thread
        && thread.join().is_err()
    {
        tracing::warn!("the clock thread panicked");
    }
    // The engine first: its window lives inside the surface.
    if let Some(pump) = pump {
        pump.close();
    }
    drop(engine);
    drop(surface);
    if sync.start_failed.load(Ordering::SeqCst) {
        tracing::error!("the video could not be played, the renderer exits as a failed start");
        return Ok(ExitCode::from(EXIT_START_FAILURE));
    }
    if loop_end == LoopEnd::Failed {
        tracing::error!("the message loop failed, the renderer exits so that the service starts a new one");
        return Ok(ExitCode::from(EXIT_LOOP_FAILED));
    }
    if surface_lost {
        // Not an error: Windows destroyed the surface (explorer.exe killed sends no such message; the service
        // replaces the renderers on TaskbarCreated), and mpv ending with its parent window is part of that.
        tracing::info!("Windows destroyed the desktop surface, the service starts a new renderer");
        return Ok(ExitCode::from(EXIT_SURFACE_LOST));
    }
    tracing::info!("the renderer stopped");
    Ok(ExitCode::SUCCESS)
}

/// How long the answer to "is the desktop in front" is kept, as in the legacy app (it re-checks every second).
const DESKTOP_RECHECK: Duration = Duration::from_secs(1);

/// Whether the desktop is the foreground window, for `InputDesktop`. A lookup reads the foreground window's class, so
/// the answer is kept for [`DESKTOP_RECHECK`] instead of being asked for every mouse movement.
struct DesktopFocus {
    platform: Box<dyn Platform>,
    last: Option<(Instant, bool)>,
}

impl DesktopFocus {
    fn new() -> Self {
        Self {
            platform: sucrose_platform::current(),
            last: None,
        }
    }

    fn is_desktop(&mut self) -> bool {
        if let Some((at, answer)) = self.last
            && at.elapsed() < DESKTOP_RECHECK
        {
            return answer;
        }
        let answer = self
            .platform
            .foreground_window()
            .is_some_and(|window| is_desktop_class(&window.class_name));
        self.last = Some((Instant::now(), answer));
        answer
    }
}

/// Moves the surface onto `rect` and logs where it landed.
fn move_surface(surface: &DesktopSurface, rect: RectPx) {
    match surface.set_rect(rect) {
        Ok(placement) => {
            tracing::info!(
                target = ?placement.target,
                local = ?placement.local,
                parent = ?placement.parent,
                verdict = ?placement.verdict,
                "surface moved"
            );
            match placement.verdict {
                PlacementVerdict::Shown => {}
                PlacementVerdict::Misplaced => tracing::warn!(
                    target = ?placement.target,
                    actual = ?placement.actual,
                    "the surface did not land on its target"
                ),
                PlacementVerdict::PartlyHidden { .. } | PlacementVerdict::Hidden => tracing::warn!(
                    target = ?placement.target,
                    parent = ?placement.parent,
                    virtual_screen = ?placement.virtual_screen,
                    "the surface lies outside the desktop window"
                ),
            }
        }
        Err(error) => tracing::warn!(%error, ?rect, "cannot move the surface"),
    }
}

/// The reason and the repair hint the service shows the user (the tray tooltip) and logs for an engine that did not
/// start, and the exit code it reads as the cause: a missing engine, or a failed start (D-20).
fn engine_failure(error: &EngineError) -> (String, String, u8) {
    match error {
        EngineError::Missing { reason, hint } => (reason.clone(), hint.clone(), EXIT_ENGINE_MISSING),
        EngineError::Start { reason } => (reason.clone(), CHECK_THE_LOG.to_owned(), EXIT_START_FAILURE),
        EngineError::Unsupported { what } => (
            format!("{what} is not supported by this build."),
            "Choose another engine in Wallpaper settings.".to_owned(),
            EXIT_START_FAILURE,
        ),
    }
}

/// Tells the service why this renderer is about to exit. A failed send only costs the user the explanation: the
/// renderer exits with its code either way.
fn report_failure(connection: &mut Connection, reason: String, hint: impl Into<String>) {
    if let Err(error) = connection.send(&ToService::Failed {
        reason,
        hint: hint.into(),
    }) {
        tracing::warn!(%error, "cannot tell the service why the wallpaper did not start");
    }
}

/// Logs the GPU adapter the video device of this renderer gets. libmpv creates its Direct3D 11 device with no adapter
/// named (config.rs sets no `d3d11-adapter`), and so does `default_gpu_adapter`, so both get the adapter Windows picks
/// for this process from its GPU preference. The order of the adapter enumeration list says nothing about that, so
/// no entry of it is logged as the adapter in use. Finding it creates a Direct3D device (driver load, a device on the
/// discrete GPU), which a renderer with a low resource budget must not pay just for a log line, so it only runs when
/// debug logging is on (`SUCROSE_LOG=debug`).
fn log_video_adapter() {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    match sucrose_platform::current().default_gpu_adapter() {
        Some(adapter) => tracing::debug!(
            name = %adapter.name,
            luid = %adapter.luid,
            dedicated_mb = adapter.dedicated_video_memory_mb,
            "video GPU adapter"
        ),
        None => tracing::debug!("the default GPU adapter is unknown on this OS"),
    }
}

/// Starts the engine `id` on the surface window `wid`. An engine this build does not contain is refused, never
/// replaced by another one (T-04-12).
fn start_engine(
    id: EngineId,
    wid: u64,
    assignment: &Assignment,
    hooks: impl FnOnce() -> SurfaceHooks,
) -> Result<Started, EngineError> {
    match id {
        EngineId::Mpv => sucrose_engine_mpv::engine::start(wid, assignment),
        EngineId::Cef => sucrose_engine_cef::engine::start(wid, assignment, hooks()),
        other => Err(EngineError::Unsupported {
            what: format!("The {other:?} engine"),
        }),
    }
}

/// What one reply of the service to the Hello leads to.
#[derive(Debug)]
enum HelloReply {
    /// The renderer has its job.
    Assigned(Assignment),
    /// The service asked the renderer to stop.
    Stop,
    /// A message that means nothing before the assignment: the wait goes on.
    Skip,
}

/// Reads one reply to the Hello. A connection that ends before an assignment came is an error, not a normal stop:
/// the service drops the connection of a Hello it refuses, and a renderer that read that as a clean exit would stay
/// unrestarted for good (WR-16). A real stop is the `Close` message.
fn judge_hello_reply(reply: Result<ToRenderer, IpcError>) -> anyhow::Result<HelloReply> {
    match reply {
        Ok(ToRenderer::Assign(assignment)) => Ok(HelloReply::Assigned(assignment)),
        Ok(ToRenderer::Close) => {
            tracing::info!("the service asked this renderer to close");
            Ok(HelloReply::Stop)
        }
        Err(IpcError::Closed) => bail!("the service closed the connection before it assigned a wallpaper"),
        // Controls only make sense once a wallpaper plays; before that the service has nothing to control. The
        // protocol never sends one first, so it is skipped and the wait goes on, as the log says.
        Ok(
            ToRenderer::Pause(_)
            | ToRenderer::SetVolume(_)
            | ToRenderer::SetRect(_)
            | ToRenderer::SetRole(_)
            | ToRenderer::Clock(_)
            | ToRenderer::LoadSource { .. }
            | ToRenderer::OpenDevTools
            | ToRenderer::Data(_)
            | ToRenderer::SetLoop(_)
            | ToRenderer::SetShuffle(_),
        ) => {
            tracing::warn!("ignoring a control message that arrived before the assignment");
            Ok(HelloReply::Skip)
        }
        Err(error) => bail!("the service sent no valid assignment: {error}"),
    }
}

/// Waits for the service's answer to the Hello: `Some` for a job, `None` when the service asks to stop. An error
/// when the connection ends first (see [`judge_hello_reply`]), so the renderer exits with a code the budget counts.
fn wait_for_assignment(connection: &mut Connection) -> anyhow::Result<Option<Assignment>> {
    loop {
        match judge_hello_reply(connection.recv::<ToRenderer>())? {
            HelloReply::Assigned(assignment) => return Ok(Some(assignment)),
            HelloReply::Stop => return Ok(None),
            HelloReply::Skip => {}
        }
    }
}

/// Applies the service's pause and volume commands to `engine` until the service asks the renderer to stop or the
/// connection ends. A new rectangle is handed to the main thread through `pending_rect` and `proxy`. `pause_level` is
/// the level every pause is carried out at; a new source brings the level of its own settings.
fn wait_for_close(
    connection: &mut Connection,
    engine: &dyn Engine,
    pending_rect: &PendingRect,
    proxy: &LoopProxy,
    sync: &SyncState,
    mut pause_level: PauseLevel,
) {
    let mut following = FollowState::default();
    loop {
        match connection.recv::<ToRenderer>() {
            Ok(ToRenderer::Close) => {
                tracing::info!("the service asked this renderer to close");
                return;
            }
            Ok(ToRenderer::Pause(paused)) => {
                if let Err(error) = engine.set_paused(paused, pause_level) {
                    tracing::warn!(%error, paused, "cannot change the pause state");
                }
            }
            Ok(ToRenderer::SetVolume(volume)) => {
                sync.volume.store(volume, Ordering::SeqCst);
                if let Err(error) = engine.set_volume(volume) {
                    tracing::warn!(%error, volume, "cannot change the volume");
                }
            }
            Ok(ToRenderer::SetRect([left, top, right, bottom])) => {
                let rect = RectPx {
                    left,
                    top,
                    right,
                    bottom,
                };
                // Only the newest rectangle matters, so a burst of changes collapses into one move.
                *pending_rect.lock().unwrap_or_else(PoisonError::into_inner) = Some(rect);
                proxy.wake();
            }
            Ok(ToRenderer::OpenDevTools) => {
                // Only the Portal's Open DevTools button sends this (D-13); the service hears how it went.
                let opened = match engine.open_devtools() {
                    Ok(()) => {
                        tracing::info!(engine = engine.name(), "devtools opened");
                        true
                    }
                    // An engine without developer tools answers here; nothing else changes (D-13).
                    Err(error) => {
                        tracing::info!(engine = engine.name(), %error, "no developer tools to open");
                        false
                    }
                };
                if let Err(error) = connection.send(&ToService::DevToolsOpened(opened)) {
                    tracing::warn!(%error, "the service cannot be told how opening the developer tools went");
                }
            }
            // Handed to the engine as it is; an engine without a page ignores it.
            Ok(ToRenderer::Data(data)) => engine.push_data(data),
            Ok(ToRenderer::SetRole(role)) => {
                apply_role(role, engine, sync);
                following.reset(engine.clock().map_or(1.0, EngineClock::speed));
            }
            Ok(ToRenderer::Clock(tick)) => {
                if let Some(clock) = engine.clock() {
                    follow_clock(&tick, clock, sync, &mut following);
                }
            }
            Ok(ToRenderer::LoadSource { source, settings }) => {
                pause_level = PauseLevel::from(settings.pause);
                if let Err(error) = switch_source(&source, &settings, engine, sync, &mut following) {
                    // The supervisor and its restart budget take over, as for any failed start (Phase 2 D-13).
                    tracing::error!(%error, "the new source cannot be loaded, the renderer exits as a failed start");
                    report_failure(
                        connection,
                        format!("the new video could not be loaded: {error}"),
                        CHECK_THE_LOG,
                    );
                    sync.start_failed.store(true, Ordering::SeqCst);
                    proxy.quit();
                    return;
                }
            }
            // Loop Mode: the follower's wrap detection and the engine follow at once (PLAY-01).
            Ok(ToRenderer::SetLoop(looping)) => {
                sync.looping.store(looping, Ordering::SeqCst);
                match engine.set_loop(looping) {
                    Ok(()) => tracing::info!(looping, "loop changed"),
                    Err(error) => tracing::warn!(%error, looping, "cannot change the loop mode"),
                }
            }
            // Shuffle Mode: only a web page has a call for it; the other engines ignore it.
            Ok(ToRenderer::SetShuffle(shuffle)) => {
                engine.set_shuffle(shuffle);
                tracing::info!(shuffle, "shuffle changed");
            }
            // A second assignment is not part of this plan; later plans act on it.
            Ok(ToRenderer::Assign(_)) => tracing::warn!("ignoring an unexpected second assignment"),
            Err(IpcError::Closed) => {
                tracing::info!("the service closed the connection");
                return;
            }
            Err(error) => {
                tracing::warn!(%error, "the service channel failed");
                return;
            }
        }
    }
}

/// Plays `source` on the running player instead of the current file (D-26): the renderer, its surface and its
/// connection stay. The follower's controller starts afresh (its drift belongs to the old file), the speed goes back
/// to normal, and the clock thread is told to announce the new file's first frame. The path is not logged.
fn switch_source(
    source: &str,
    settings: &PlaybackSettings,
    engine: &dyn Engine,
    sync: &SyncState,
    following: &mut FollowState,
) -> Result<(), EngineError> {
    // The audio switch the role chose (aid) is a player option and carries over to the next file by itself; only a
    // speed the drift correction left behind has to be put back.
    if let Some(clock) = engine.clock()
        && let Err(error) = clock.set_speed(1.0)
    {
        tracing::warn!(%error, "cannot restore the normal speed for the new source");
    }
    engine.load_source(Path::new(source), settings)?;
    sync.volume.store(settings.volume, Ordering::SeqCst);
    sync.looping.store(settings.looping, Ordering::SeqCst);
    *following = FollowState::default();
    following.reset(1.0);
    sync.reloaded.store(true, Ordering::SeqCst);
    tracing::info!("source loaded");
    Ok(())
}

/// Takes on the role the service chose: the master plays the audio at the configured volume and publishes its clock,
/// a follower plays none (D-02).
fn apply_role(role: Role, engine: &dyn Engine, sync: &SyncState) {
    let master = role == Role::Master;
    sync.master.store(master, Ordering::SeqCst);
    if let Err(error) = engine.set_audio(master) {
        tracing::warn!(%error, ?role, "cannot switch the audio for the new role");
    }
    if master {
        // A promoted follower may be running at a corrected speed; the clock itself plays at normal speed.
        if let Some(clock) = engine.clock()
            && let Err(error) = clock.set_speed(1.0)
        {
            tracing::warn!(%error, "cannot restore the normal speed for the master");
        }
        let volume = sync.volume.load(Ordering::SeqCst);
        if let Err(error) = engine.set_volume(volume) {
            tracing::warn!(%error, volume, "cannot set the master's volume");
        }
        // Followers should not wait a whole period for the first reading of a new master.
        sync.kick.store(true, Ordering::SeqCst);
    }
    tracing::info!(?role, "role");
}

/// How many clock ticks a follower's drift summary covers: about 30 s, as the master ticks once per second.
const SUMMARY_TICKS: u32 = 30;

/// A follower's controller with what it knows about the video it follows.
#[derive(Debug, Default)]
struct FollowState {
    follower: Follower,
    /// One frame of the video in seconds: the container's value is read once, an estimate is read again.
    frame_s: Option<f64>,
    /// Whether `frame_s` came from the container's frame rate and so cannot change; an estimate or the default is
    /// read again on the next readings.
    frame_final: bool,
    /// Ticks followed since the last drift summary was logged.
    ticks_since_summary: u32,
}

impl FollowState {
    /// A new role starts the controller afresh at the speed the player really runs at; the video, and so its
    /// frame period, stays the same.
    fn reset(&mut self, speed: f64) {
        self.follower = Follower::at_speed(speed);
        self.ticks_since_summary = 0;
    }
}

/// The frame period for this follower: the container's once known, else the best current guess, read again on each
/// reading and logged when it changes.
fn current_frame_period(clock: &dyn EngineClock, state: &mut FollowState) -> f64 {
    if let (true, Some(period)) = (state.frame_final, state.frame_s) {
        return period;
    }
    let (period, is_final) = clock.frame_period();
    if state.frame_s.is_none_or(|known| (known - period).abs() > known * 0.01) {
        tracing::info!(frame_period_s = period, is_final, "frame period");
    }
    state.frame_s = Some(period);
    state.frame_final = is_final;
    period
}

/// Corrects this follower's drift against the master's `tick` and logs its drift summary every [SUMMARY_TICKS].
fn follow_clock(tick: &ClockTick, clock: &dyn EngineClock, sync: &SyncState, state: &mut FollowState) {
    // A paused follower (the fullscreen rule on this monitor alone) has nothing to correct until it plays again; the
    // next reading after that realigns it.
    // Before its first frame this player has no position to compare either.
    if sync.master.load(Ordering::SeqCst) || clock.is_paused() || !clock.has_started() {
        return;
    }
    let Some(local) = clock.position_s() else {
        return;
    };
    let duration = if sync.looping.load(Ordering::SeqCst) {
        clock.duration_s()
    } else {
        None
    };
    let frame_s = current_frame_period(clock, state);
    let now_ns = monotonic_ns();
    let action = state.follower.step(tick, now_ns, local, duration, frame_s);
    let applied = match action {
        SyncAction::None => Ok(()),
        SyncAction::Speed(speed) => {
            tracing::debug!(
                speed,
                drift_s = drift_s(tick, now_ns, local, duration),
                "follower speed"
            );
            let result = clock.set_speed(speed);
            if result.is_err() {
                // The controller must not take the speed for sent: the next reading sends it again.
                state.follower.speed_failed(clock.speed());
            }
            result
        }
        SyncAction::Seek(position) => {
            tracing::info!(local, position, "drift beyond half a second, seeking");
            clock.seek_exact(position)
        }
    };
    if let Err(error) = applied {
        tracing::warn!(%error, ?action, "cannot correct the drift");
    }
    state.ticks_since_summary += 1;
    if state.ticks_since_summary >= SUMMARY_TICKS {
        state.ticks_since_summary = 0;
        if let Some(summary) = state.follower.take_summary() {
            tracing::info!(
                ticks = summary.ticks,
                mean_drift_frames = summary.mean_abs_s / frame_s,
                max_drift_frames = summary.max_abs_s / frame_s,
                speed_changes = summary.speed_changes,
                seeks = summary.seeks,
                "follower drift summary"
            );
        }
    }
}

/// One reading of this player's clock, or None while it has no position yet or the OS clock fails.
fn read_tick(clock: &dyn EngineClock) -> Option<ClockTick> {
    let position_s = clock.position_s()?;
    let stamp_ns = monotonic_ns();
    (stamp_ns > 0).then(|| ClockTick {
        position_s,
        stamp_ns,
        speed: clock.speed(),
        paused: clock.is_paused(),
        loop_gen: clock.loop_gen(),
    })
}

/// Tells the service that this renderer shows its first frame. A failed send only delays the tooltip and the master
/// election, so it is logged and nothing else.
fn tell_playing(connection: &mut Connection) {
    if let Err(error) = connection.send(&ToService::Playing) {
        tracing::warn!(%error, "cannot tell the service that the wallpaper is playing");
    }
}

/// Tells the service which system data groups the page needs (D-24). A failed send only means the page gets no data,
/// so it is logged and nothing else.
fn tell_data_wanted(connection: &mut Connection, groups: Vec<DataGroup>) {
    if let Err(error) = connection.send(&ToService::DataWanted(groups)) {
        tracing::warn!(%error, "cannot tell the service which data the page needs");
    }
}

/// Tells the service once that the first frame is shown (the service elects the master only among such renderers),
/// then publishes the master's clock about once per period, and at once after a seek, loop wrap, pause, resume or a
/// promotion. Does nothing while this renderer is a follower or has not shown its first frame. When the video cannot
/// be played it reports that, stops the message loop through `proxy` and ends. Runs until `stop` is set.
fn publish_clock(
    mut connection: Connection,
    engine: &dyn Engine,
    sync: &SyncState,
    stop: &AtomicBool,
    proxy: &LoopProxy,
) {
    let Some(clock) = engine.clock() else {
        return;
    };
    let mut last = Instant::now();
    let mut sent_once = false;
    let mut announced = false;
    while !stop.load(Ordering::SeqCst) {
        // A new source: its first frame is announced again, and the master's clock starts afresh with it.
        if sync.reloaded.swap(false, Ordering::SeqCst) {
            announced = false;
            sent_once = false;
        }
        let mut due = sync.kick.swap(false, Ordering::SeqCst);
        if clock.wait_event(CLOCK_POLL) {
            due = true;
        }
        // A file that mpv cannot open or decode never shows a frame: say why and end the renderer, so that the
        // tray does not wait for a wallpaper that will not come (the restart budget applies like any failed start).
        if let Some(error) = clock.start_failure() {
            tracing::error!(%error, "the video cannot be played");
            report_failure(
                &mut connection,
                format!("the video could not be played: {error}"),
                CHECK_THE_LOG,
            );
            sync.start_failed.store(true, Ordering::SeqCst);
            proxy.quit();
            return;
        }
        // The Started event can come before this thread took the receiver, so the flag is looked at as well.
        if !clock.has_started() {
            continue;
        }
        if !announced {
            tell_playing(&mut connection);
            announced = true;
        }
        if !sync.master.load(Ordering::SeqCst) {
            sent_once = false;
            continue;
        }
        if !(due || !sent_once || last.elapsed() >= CLOCK_PERIOD) {
            continue;
        }
        let Some(tick) = read_tick(clock) else {
            continue;
        };
        if let Err(error) = connection.send(&ToService::Clock(tick)) {
            tracing::warn!(%error, "cannot publish the clock, the service is gone");
            return;
        }
        sent_once = true;
        last = Instant::now();
    }
}

/// Connects to the service, retrying while it starts.
fn connect_with_retry(endpoint: &str) -> Result<Connection, IpcError> {
    let mut attempt = 1;
    loop {
        match connect(endpoint) {
            Ok(connection) => return Ok(connection),
            Err(error) if attempt >= CONNECT_ATTEMPTS => return Err(error),
            Err(error) => {
                tracing::debug!(attempt, %error, "the service is not reachable yet");
                attempt += 1;
                thread::sleep(CONNECT_PAUSE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::theme::WallpaperType;
    use sucrose_ipc::protocol::{InputSettings, WebOptions};

    use super::*;

    #[test]
    fn a_missing_engine_exits_with_code_2() {
        let error = EngineError::Missing {
            reason: "libmpv was not found.".to_owned(),
            hint: "Reinstall Sucrose.".to_owned(),
        };

        let (reason, hint, code) = engine_failure(&error);

        assert_eq!(code, EXIT_ENGINE_MISSING);
        assert_eq!(code, 2);
        assert_eq!(reason, "libmpv was not found.");
        assert_eq!(hint, "Reinstall Sucrose.");
    }

    #[test]
    fn a_failed_start_exits_with_code_4_and_points_to_the_log() {
        let error = EngineError::Start {
            reason: "the player could not be created".to_owned(),
        };

        let (reason, hint, code) = engine_failure(&error);

        assert_eq!(code, EXIT_START_FAILURE);
        assert_eq!(code, 4);
        assert!(!reason.is_empty());
        assert_eq!(hint, CHECK_THE_LOG);
    }

    #[test]
    fn an_engine_the_build_does_not_contain_is_refused_with_a_hint_to_the_settings() {
        let (reason, hint, code) = engine_failure(&EngineError::Unsupported {
            what: "The Vlc engine".to_owned(),
        });

        assert_eq!(code, EXIT_START_FAILURE);
        assert!(reason.contains("Vlc"), "{reason}");
        assert!(hint.contains("Wallpaper settings"), "{hint}");
    }

    #[test]
    fn the_factory_refuses_an_engine_it_does_not_start_instead_of_guessing() {
        let assignment = Assignment {
            slot: 0,
            monitor: "M".to_owned(),
            rect: [0, 0, 10, 10],
            source: "C:/a.mp4".to_owned(),
            settings: PlaybackSettings::default(),
            kind: WallpaperType::Video,
            engine: sucrose_core::settings::EngineType::VlcPlayerLive,
            theme_dir: "C:/".to_owned(),
            web: WebOptions::default(),
            input: InputSettings::default(),
        };

        let error = start_engine(EngineId::Vlc, 0, &assignment, || {
            unreachable!("only the CEF engine takes the surface hooks")
        })
        .expect_err("libVLC is not in this build");

        assert!(matches!(error, EngineError::Unsupported { .. }), "{error:?}");
    }

    #[test]
    fn a_connection_closed_before_the_assignment_is_an_error_not_a_clean_stop() {
        let error = judge_hello_reply(Err(IpcError::Closed)).expect_err("a refused Hello must not exit with code 0");

        assert!(error.to_string().contains("before it assigned"), "{error}");
    }

    #[test]
    fn close_stops_the_renderer_and_a_control_message_waits_on() {
        assert!(matches!(judge_hello_reply(Ok(ToRenderer::Close)), Ok(HelloReply::Stop)));
        assert!(matches!(
            judge_hello_reply(Ok(ToRenderer::Pause(true))),
            Ok(HelloReply::Skip)
        ));
    }

    #[test]
    fn a_loop_or_shuffle_change_before_the_assignment_waits_on_like_any_control_message() {
        for message in [ToRenderer::SetLoop(false), ToRenderer::SetShuffle(true)] {
            assert!(matches!(judge_hello_reply(Ok(message)), Ok(HelloReply::Skip)));
        }
    }

    #[test]
    fn a_new_source_before_the_assignment_waits_on_like_any_control_message() {
        let reply = judge_hello_reply(Ok(ToRenderer::LoadSource {
            source: "C:/clip.mp4".to_owned(),
            settings: PlaybackSettings::default(),
        }));

        assert!(matches!(reply, Ok(HelloReply::Skip)));
    }
}
