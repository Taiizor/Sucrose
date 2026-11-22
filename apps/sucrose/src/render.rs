//! Renderer mode (`sucrose --render <slot>`): connects to the service, receives its assignment and keeps a surface
//! behind the desktop icons until it is told to stop.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context as _, bail};
use sucrose_core::paths::{cache_dir, legacy_mpv_cache_dir};
use sucrose_engine_mpv::{
    config::{embedding_options, generated_options, shader_cache_options, user_config_path},
    loader::{self, LoadError},
    player::{OpenOptions, Player, PlayerError, PlayerEvent},
    sync::{Follower, SyncAction, drift_s, frame_period},
};
use sucrose_ipc::{
    Connection, IpcError, connect,
    protocol::{Assignment, ClockTick, PROTOCOL_VERSION, Role, ToRenderer, ToService},
};
use sucrose_platform::{DesktopSurface, DpiAwareness, EventLoop, Flow, LoopEvent, LoopProxy, RectPx, monotonic_ns};

/// How often and how long the renderer retries while the service is still opening its endpoint.
const CONNECT_ATTEMPTS: u32 = 20;
const CONNECT_PAUSE: Duration = Duration::from_millis(100);

/// Exit code for "the desktop surface could not be attached", reported to the service like any other start failure.
const EXIT_START_FAILURE: u8 = crate::supervisor::START_FAILED;

/// The hint for a failure that has no repair step of its own.
const CHECK_THE_LOG: &str = "Check the log for details.";

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
    /// Whether the video loops, so that a follower treats the loop end as a wrap and not as drift.
    looping: bool,
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

    let endpoint = crate::service::endpoint()?;
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

    // Opened here, on the thread that pumps the surface's messages: mpv hooks that thread to resize its child.
    let player = match start_engine(surface.native_handle(), &assignment) {
        Ok(player) => Arc::new(player),
        Err(error) => {
            error.log();
            let (reason, hint) = error.report();
            report_failure(&mut connection, reason, hint);
            return Ok(ExitCode::from(error.exit_code()));
        }
    };
    let sync = Arc::new(SyncState {
        master: AtomicBool::new(false),
        volume: AtomicU8::new(assignment.settings.volume),
        kick: AtomicBool::new(false),
        looping: assignment.settings.looping,
    });
    // The clock thread tells the service the wallpaper plays once the first frame is shown, and publishes the master's
    // clock. It is started before the connection moves into the reader thread, and joined before the player is
    // dropped, so the player still ends before the surface it is embedded in.
    let clock_stop = Arc::new(AtomicBool::new(false));
    let clock_thread = match connection.try_clone() {
        Ok(writer) => {
            let clock_player = Arc::clone(&player);
            let clock_sync = Arc::clone(&sync);
            let stop = Arc::clone(&clock_stop);
            let spawned = thread::Builder::new()
                .name("clock".to_owned())
                .spawn(move || publish_clock(writer, &clock_player, &clock_sync, &stop));
            match spawned {
                Ok(handle) => Some(handle),
                Err(error) => {
                    tracing::warn!(%error, "cannot start the clock thread, this renderer cannot be the master");
                    None
                }
            }
        }
        Err(error) => {
            tracing::warn!(%error, "cannot keep a write handle for the clock, this renderer cannot be the master");
            None
        }
    };
    if clock_thread.is_none() {
        // Nobody else will announce it: without a clock thread the wallpaper is reported as playing right away.
        tell_playing(&mut connection);
    }
    // The reader thread only waits for the end of the session; the loop below must stay on this thread because the
    // surface was created here.
    let proxy = event_loop.proxy();
    let pending_rect = PendingRect::default();
    let reader_player = Arc::clone(&player);
    let reader_rect = Arc::clone(&pending_rect);
    let reader_proxy = proxy.clone();
    let reader_sync = Arc::clone(&sync);
    thread::Builder::new()
        .name("ipc-reader".to_owned())
        .spawn(move || {
            wait_for_close(
                &mut connection,
                &reader_player,
                &reader_rect,
                &reader_proxy,
                &reader_sync,
            );
            // Released before the loop is told to stop, so that the main thread drops the last handle and the
            // player ends before the surface it is embedded in.
            drop(reader_player);
            proxy.quit();
        })
        .context("cannot start the reader thread")?;
    // A wake means a new rectangle: the surface is moved here because only this thread may touch it. The loop ends
    // when the reader thread asks for it, or when Windows destroys the surface (the surface posts the quit itself).
    // A killed explorer.exe sends no such message; the service replaces the renderers on TaskbarCreated instead.
    event_loop.run(|event| match event {
        LoopEvent::Wake => {
            let rect = pending_rect.lock().unwrap_or_else(PoisonError::into_inner).take();
            if let Some(rect) = rect {
                match surface.set_rect(rect) {
                    Ok(()) => tracing::info!(?rect, "surface moved"),
                    Err(error) => tracing::warn!(%error, ?rect, "cannot move the surface"),
                }
            }
            Flow::Continue
        }
        LoopEvent::Quit => Flow::Exit,
        // The service watches the monitors and the shell; a renderer learns about both through its own surface.
        LoopEvent::DisplayChanged | LoopEvent::ShellRestarted | LoopEvent::Timer(_) => Flow::Continue,
    });
    let surface_lost = surface.destroyed_externally();
    clock_stop.store(true, Ordering::SeqCst);
    if let Some(thread) = clock_thread
        && thread.join().is_err()
    {
        tracing::warn!("the clock thread panicked");
    }
    // The player first: its child window lives inside the surface.
    drop(player);
    drop(surface);
    if surface_lost {
        // Not an error: Windows destroyed the surface (explorer.exe killed sends no such message; the service
        // replaces the renderers on TaskbarCreated), and mpv ending with its parent window is part of that.
        tracing::info!("the desktop surface was destroyed by the shell, the service starts a new renderer");
        return Ok(ExitCode::from(EXIT_SURFACE_LOST));
    }
    tracing::info!("the renderer stopped");
    Ok(ExitCode::SUCCESS)
}

/// Why the engine did not start. Every variant is a value with one log line and one exit code, never a panic (D-20).
#[derive(Debug)]
enum EngineError {
    /// libmpv is missing or cannot be loaded.
    Load(LoadError),
    /// libmpv loaded but the player could not be created or could not start the file.
    Player(PlayerError),
}

impl EngineError {
    /// The exit code the service reads as the reason the renderer ended.
    fn exit_code(&self) -> u8 {
        match self {
            Self::Load(_) => EXIT_ENGINE_MISSING,
            Self::Player(_) => EXIT_START_FAILURE,
        }
    }

    /// The reason and the repair hint the service shows the user (the tray tooltip) and logs.
    fn report(&self) -> (String, String) {
        match self {
            // The full search list is in the log; the user only needs to know the file is missing.
            Self::Load(LoadError::NotFound { .. }) => ("libmpv was not found.".to_owned(), self.hint()),
            Self::Load(error) => (error.to_string(), self.hint()),
            Self::Player(error) => (error.to_string(), self.hint()),
        }
    }

    /// The repair hint: the loader's own for a library problem, else a pointer to the log.
    fn hint(&self) -> String {
        match self {
            Self::Load(error) => error.hint().to_owned(),
            Self::Player(_) => CHECK_THE_LOG.to_owned(),
        }
    }

    /// Logs the failure once, at error level, with the repair hint when there is one.
    fn log(&self) {
        match self {
            Self::Load(error) => tracing::error!(%error, hint = error.hint(), "libmpv cannot be loaded"),
            Self::Player(error) => tracing::error!(%error, "cannot start the player"),
        }
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
/// no entry of it is logged as the adapter in use.
fn log_video_adapter() {
    match sucrose_platform::current().default_gpu_adapter() {
        Some(adapter) => tracing::info!(
            name = %adapter.name,
            luid = %adapter.luid,
            dedicated_mb = adapter.dedicated_video_memory_mb,
            "video GPU adapter"
        ),
        None => tracing::debug!("the default GPU adapter is unknown on this OS"),
    }
}

/// Loads the bundled libmpv (only the renderer does, D-17) and starts the player embedded in the window `wid`, with
/// the options the assignment's settings lead to, or the user's own mpv config when there is one (D-10).
fn start_engine(wid: u64, assignment: &Assignment) -> Result<Player, EngineError> {
    let mpv = Arc::new(loader::load_default().map_err(EngineError::Load)?);
    let shader_cache = shader_cache_dir();
    // The shader cache is always redirected, so that a user config cannot send it into the C# app's folder.
    let mut embedding = embedding_options();
    embedding.extend(shader_cache_options(&shader_cache));
    // No audio is decoded until the service names this renderer the master clock (D-02): followers never play it.
    embedding.push(("aid", "no".to_owned()));
    let generated = generated_options(&assignment.settings, &shader_cache);
    // Only the path is taken from the C# app's folder; mpv reads the file and nothing is written there (D-08).
    let user_config = legacy_mpv_cache_dir().and_then(|dir| user_config_path(&dir));
    let options = OpenOptions {
        embedding: &embedding,
        generated: &generated,
        user_config: user_config.as_deref(),
    };
    Player::open(mpv, wid, Path::new(&assignment.source), &options).map_err(EngineError::Player)
}

/// The folder for mpv's shader cache, inside Sucrose's own data (SUCROSE_DATA_DIR when set), created when missing.
fn shader_cache_dir() -> PathBuf {
    let dir = cache_dir().unwrap_or_else(std::env::temp_dir).join("mpv-shaders");
    if let Err(error) = std::fs::create_dir_all(&dir) {
        tracing::warn!(%error, path = %dir.display(), "cannot create the shader cache folder");
    }
    dir
}

/// Waits for the service's answer to the Hello: `Some` for a job, `None` when the service asks to stop.
fn wait_for_assignment(connection: &mut Connection) -> anyhow::Result<Option<Assignment>> {
    match connection.recv::<ToRenderer>() {
        Ok(ToRenderer::Assign(assignment)) => Ok(Some(assignment)),
        Ok(ToRenderer::Close) => {
            tracing::info!("the service asked this renderer to close");
            Ok(None)
        }
        Err(IpcError::Closed) => {
            tracing::info!("the service closed the connection");
            Ok(None)
        }
        // Controls only make sense once a wallpaper plays; before that the service has nothing to control.
        Ok(
            ToRenderer::Pause(_)
            | ToRenderer::SetVolume(_)
            | ToRenderer::SetRect(_)
            | ToRenderer::SetRole(_)
            | ToRenderer::Clock(_),
        ) => {
            tracing::warn!("ignoring a control message that arrived before the assignment");
            Ok(None)
        }
        Err(error) => bail!("the service sent no valid assignment: {error}"),
    }
}

/// Applies the service's pause and volume commands to `player` until the service asks the renderer to stop or the
/// connection ends. A new rectangle is handed to the main thread through `pending_rect` and `proxy`.
fn wait_for_close(
    connection: &mut Connection,
    player: &Player,
    pending_rect: &PendingRect,
    proxy: &LoopProxy,
    sync: &SyncState,
) {
    let mut following = FollowState::default();
    loop {
        match connection.recv::<ToRenderer>() {
            Ok(ToRenderer::Close) => {
                tracing::info!("the service asked this renderer to close");
                return;
            }
            Ok(ToRenderer::Pause(paused)) => {
                if let Err(error) = player.set_paused(paused) {
                    tracing::warn!(%error, paused, "cannot change the pause state");
                }
            }
            Ok(ToRenderer::SetVolume(volume)) => {
                sync.volume.store(volume, Ordering::SeqCst);
                if let Err(error) = player.set_volume(volume) {
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
            Ok(ToRenderer::SetRole(role)) => {
                apply_role(role, player, sync);
                following.reset();
            }
            Ok(ToRenderer::Clock(tick)) => follow_clock(&tick, player, sync, &mut following),
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

/// Takes on the role the service chose: the master plays the audio at the configured volume and publishes its clock,
/// a follower plays none (D-02).
fn apply_role(role: Role, player: &Player, sync: &SyncState) {
    let master = role == Role::Master;
    sync.master.store(master, Ordering::SeqCst);
    if let Err(error) = player.set_audio(master) {
        tracing::warn!(%error, ?role, "cannot switch the audio for the new role");
    }
    if master {
        // A promoted follower may be running at a corrected speed; the clock itself plays at normal speed.
        if let Err(error) = player.set_speed(1.0) {
            tracing::warn!(%error, "cannot restore the normal speed for the master");
        }
        let volume = sync.volume.load(Ordering::SeqCst);
        if let Err(error) = player.set_volume(volume) {
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
    /// One frame of the video in seconds, read once the first frame is shown.
    frame_s: Option<f64>,
    /// Ticks followed since the last drift summary was logged.
    ticks_since_summary: u32,
}

impl FollowState {
    /// A new role starts the controller afresh; the video, and so its frame period, stays the same.
    fn reset(&mut self) {
        self.follower = Follower::new();
        self.ticks_since_summary = 0;
    }
}

/// One frame of the playing video in seconds: from its container frame rate, else mpv's estimate, else a default.
fn video_frame_period(player: &Player) -> f64 {
    let fps = ["container-fps", "estimated-vf-fps"].into_iter().find_map(|name| {
        player
            .get_property_string(name)
            .and_then(|value| value.trim().parse::<f64>().ok())
            .filter(|fps| fps.is_finite() && *fps > 0.0)
    });
    let period = frame_period(fps);
    tracing::info!(?fps, frame_period_s = period, "frame period");
    period
}

/// Corrects this follower's drift against the master's `tick` and logs its drift summary every [SUMMARY_TICKS].
fn follow_clock(tick: &ClockTick, player: &Player, sync: &SyncState, state: &mut FollowState) {
    // A paused follower (the fullscreen rule on this monitor alone) has nothing to correct until it plays again; the
    // next reading after that realigns it.
    // Before its first frame this player has no position to compare either.
    if sync.master.load(Ordering::SeqCst) || player.is_paused() || !player.has_started() {
        return;
    }
    let Some(local) = player.time_pos() else {
        return;
    };
    let duration = if sync.looping { player.duration() } else { None };
    let frame_s = *state.frame_s.get_or_insert_with(|| video_frame_period(player));
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
            player.set_speed(speed)
        }
        SyncAction::Seek(position) => {
            tracing::info!(local, position, "drift beyond half a second, seeking");
            player.seek_exact(position)
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
fn read_tick(player: &Player) -> Option<ClockTick> {
    let position_s = player.time_pos()?;
    let stamp_ns = monotonic_ns();
    (stamp_ns > 0).then(|| ClockTick {
        position_s,
        stamp_ns,
        speed: player.speed(),
        paused: player.is_paused(),
        loop_gen: player.loop_gen(),
    })
}

/// Tells the service that this renderer shows its first frame. A failed send only delays the tooltip and the master
/// election, so it is logged and nothing else.
fn tell_playing(connection: &mut Connection) {
    if let Err(error) = connection.send(&ToService::Playing) {
        tracing::warn!(%error, "cannot tell the service that the wallpaper is playing");
    }
}

/// Tells the service once that the first frame is shown (the service elects the master only among such renderers),
/// then publishes the master's clock about once per period, and at once after a seek, loop wrap, pause, resume or a
/// promotion. Does nothing while this renderer is a follower or has not shown its first frame. Runs until `stop`
/// is set.
fn publish_clock(mut connection: Connection, player: &Player, sync: &SyncState, stop: &AtomicBool) {
    let events: Option<Receiver<PlayerEvent>> = player.take_events();
    let mut last = Instant::now();
    let mut sent_once = false;
    let mut announced = false;
    while !stop.load(Ordering::SeqCst) {
        let mut due = sync.kick.swap(false, Ordering::SeqCst);
        match &events {
            Some(events) => match events.recv_timeout(CLOCK_POLL) {
                Ok(_) => {
                    while events.try_recv().is_ok() {}
                    due = true;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => thread::sleep(CLOCK_POLL),
            },
            None => thread::sleep(CLOCK_POLL),
        }
        // The Started event can come before this thread took the receiver, so the flag is looked at as well.
        if !player.has_started() {
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
        let Some(tick) = read_tick(player) else {
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
    use super::*;

    #[test]
    fn a_missing_engine_exits_with_code_2() {
        let error = EngineError::Load(LoadError::NotFound { searched: Vec::new() });
        assert_eq!(error.exit_code(), EXIT_ENGINE_MISSING);
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn a_player_failure_exits_with_code_4() {
        let error = EngineError::Player(PlayerError::CreateFailed);
        assert_eq!(error.exit_code(), EXIT_START_FAILURE);
        assert_eq!(error.exit_code(), 4);
    }

    #[test]
    fn a_missing_engine_reports_its_reason_and_repair_hint() {
        let error = EngineError::Load(LoadError::NotFound { searched: Vec::new() });

        let (reason, hint) = error.report();

        assert_eq!(reason, "libmpv was not found.");
        assert!(hint.contains("fetch-engines") || hint.contains("reinstall"), "{hint}");
    }

    #[test]
    fn a_player_failure_points_to_the_log() {
        let (reason, hint) = EngineError::Player(PlayerError::CreateFailed).report();

        assert!(!reason.is_empty());
        assert_eq!(hint, CHECK_THE_LOG);
    }

    #[test]
    fn logging_a_failure_does_not_panic() {
        EngineError::Load(LoadError::NotFound { searched: Vec::new() }).log();
        EngineError::Player(PlayerError::CreateFailed).log();
    }
}
