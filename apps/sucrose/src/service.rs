//! Service mode: owns the endpoint (single instance, D-023), starts a renderer and listens to it.

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use crossbeam_channel::Sender;
use sucrose_config::legacy::{EngineSettings, LegacySettings};
use sucrose_core::{
    APP_LONG_NAME,
    paths::{legacy_data_dir, legacy_setting_dir},
    theme::{SelectedTheme, check_selected},
    version::app_version,
};
use sucrose_ipc::{
    Connection, IpcError, Listener, endpoint_name,
    protocol::{Assignment, ClockTick, PROTOCOL_VERSION, PlaybackSettings, Role, ToRenderer, ToService},
};
use sucrose_platform::{
    EventLoop, Flow, GpuPreferenceChange, LoopEnd, LoopEvent, LoopProxy, LoopTimers, MonitorInfo, PlatformError,
    RectPx, Tray,
};
// ScreenArea is read below to choose the surface rectangle.
use sucrose_core::settings::{FullScreenAction, GpuPreference, PauseKind, ScreenArea};

use crate::{
    fullscreen::{ActionScope, FULLSCREEN_POLL_MS, FULLSCREEN_TIMER, Rule, RuleAction, covered_monitors},
    supervisor::{
        Decision, Exhausted, ExitClass, Exited, RestartBudget, START_FAILED, ShellRestart, Supervisor, classify_exit,
        decide_exit,
    },
    topology::{
        Debounce, SlotMap, TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE, TimerCommand, TopologyAction, diff, primary, with_area,
    },
};

/// Where this user's service listens.
pub struct Endpoint {
    /// The endpoint name, shared by the service and its renderers.
    pub name: String,
    /// The account the endpoint is restricted to, where the OS can restrict it (the SID on Windows).
    pub owner: Option<String>,
}

/// The endpoint of the current user. The account SID names it and owns it on Windows; the user name is only the
/// fallback where the platform has no user identity (macOS and Linux, which have no M1 wallpaper).
pub fn endpoint() -> anyhow::Result<Endpoint> {
    match sucrose_platform::current().user_key() {
        Ok(key) => Ok(Endpoint {
            name: endpoint_name(&key),
            owner: Some(key),
        }),
        Err(PlatformError::Unsupported(_)) => {
            let user = std::env::var("USERNAME")
                .or_else(|_| std::env::var("USER"))
                .unwrap_or_else(|_| "user".to_owned());
            Ok(Endpoint {
                name: endpoint_name(&user),
                owner: None,
            })
        }
        Err(error) => Err(error).context("cannot identify the current user"),
    }
}

/// What the threads of the service tell its main thread.
#[derive(Debug)]
enum Event {
    /// A message from the renderer of `slot`, started as process `pid`. Both are the ones verified at its Hello, never
    /// read from the message.
    Renderer { slot: u8, pid: u32, message: ToService },
    /// A renderer process ended.
    Exited(Exited),
    /// The shutdown worker finished closing every renderer.
    ShutdownDone,
}

/// Sends events to the main thread and wakes its message loop, so the loop never polls.
#[derive(Clone)]
struct Notifier {
    sender: Sender<Event>,
    proxy: LoopProxy,
}

impl Notifier {
    /// Queues `event` and wakes the loop. `false` once the main thread is gone.
    fn send(&self, event: Event) -> bool {
        if self.sender.send(event).is_err() {
            return false;
        }
        self.proxy.wake();
        true
    }
}

/// How many messages may wait for one renderer that does not read. The service sends a handful of control messages and
/// one clock reading per second, so a full queue means a renderer that is stuck, not a busy one.
const OUTGOING_CAPACITY: usize = 64;

/// What became of a message offered to a [`Link`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Offer {
    /// Queued; the writer thread sends it.
    Queued,
    /// A clock reading dropped because the renderer is behind: the next reading replaces it anyway.
    Dropped,
    /// Not queued: the queue is full of messages that cannot be skipped, or the writer thread has ended (the
    /// connection is broken).
    Failed,
}

/// The write side of one renderer's connection (WR-13). Messages go into a bounded queue that a writer thread of its
/// own drains, so a renderer that stops reading blocks that thread only: the main thread and the other renderers
/// never wait for it, and nothing blocks while the `Renderers` lock is held. One writer per connection also keeps
/// the order of the messages and never interleaves two frames (WR-12).
struct Link {
    /// The renderer process on the other end, as verified at its Hello. Reports and exits carry a pid too, so one of
    /// a renderer that has been replaced is never taken for its replacement's.
    pid: u32,
    queue: Sender<ToRenderer>,
}

impl Link {
    /// Starts the writer thread `name`, which calls `write` for each queued message and ends when `write` says the
    /// connection is broken or the link is dropped (after sending what was queued).
    fn start(
        name: String,
        pid: u32,
        mut write: impl FnMut(ToRenderer) -> bool + Send + 'static,
    ) -> std::io::Result<Self> {
        let (queue, receiver) = crossbeam_channel::bounded::<ToRenderer>(OUTGOING_CAPACITY);
        thread::Builder::new().name(name).spawn(move || {
            while let Ok(message) = receiver.recv() {
                if !write(message) {
                    break;
                }
            }
        })?;
        Ok(Self { pid, queue })
    }

    /// The writer of `connection`, a clone of the handle the reader thread keeps.
    fn for_connection(slot: u8, pid: u32, mut connection: Connection) -> std::io::Result<Self> {
        Self::start(format!("ipc-writer-{slot}"), pid, move |message| {
            match connection.send(&message) {
                Ok(()) => true,
                Err(error) => {
                    tracing::warn!(%error, slot, "cannot send a message to a renderer, its writer stops");
                    false
                }
            }
        })
    }

    /// Queues `message` without ever waiting. A clock reading that finds the queue full is dropped; any other message
    /// that does is a failure, because losing it would leave the renderer in a wrong state.
    fn offer(&self, message: ToRenderer) -> Offer {
        let skippable = matches!(message, ToRenderer::Clock(_));
        match self.queue.try_send(message) {
            Ok(()) => Offer::Queued,
            Err(crossbeam_channel::TrySendError::Full(_)) if skippable => Offer::Dropped,
            Err(_) => Offer::Failed,
        }
    }
}

/// The channels to the renderers this service started.
#[derive(Default)]
struct Renderers {
    /// The write side of each connected renderer, by slot.
    connections: HashMap<u8, Link>,
    /// Set when shutdown begins: a renderer that connects later is told to close at once.
    closing: bool,
    /// The slot of the master clock (D-02); the others are followers. Kept under the same lock as the connections so
    /// a role is never sent from a stale election.
    master: Option<u8>,
}

type SharedRenderers = Arc<Mutex<Renderers>>;

/// The job of every renderer slot that has a monitor, shared with the threads that answer a renderer's Hello.
type SharedAssignments = Arc<Mutex<HashMap<u8, Assignment>>>;

fn lock_assignments(assignments: &Mutex<HashMap<u8, Assignment>>) -> MutexGuard<'_, HashMap<u8, Assignment>> {
    assignments.lock().unwrap_or_else(PoisonError::into_inner)
}

fn lock(renderers: &Mutex<Renderers>) -> MutexGuard<'_, Renderers> {
    renderers.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The fullscreen rule acts on every monitor, as the legacy app does (D-05). Acting on the affected monitor only is
/// implemented too ([`ActionScope::AffectedMonitor`]) but has no switch in M1: persisting it is Phase 3, its UI PERF-01.
const ACTION_SCOPE: ActionScope = ActionScope::AllMonitors;

/// How long a renderer gets to leave after Close before it is killed.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// The tray icon the service shows (the C# Launcher's image, D-14).
const TRAY_ICON: &[u8] = include_bytes!("../assets/TrayIcon.ico");

/// Why no wallpaper plays while the OS reports no monitor.
const NO_MONITOR: &str = "the OS reports no monitor";

/// The longest reason or hint taken from a renderer, so a misbehaving renderer cannot flood the log (T-02-19).
const MAX_REPORT_CHARS: usize = 300;

/// What the service tells the user about the wallpaper: one value, shown in the tray tooltip and written to the log.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    /// A renderer was started for this theme and has not reported playback yet.
    Starting { title: String },
    /// The wallpaper plays.
    Playing { title: String },
    /// No renderer was started: the theme is rejected (D-13) or no monitor can be used.
    NoWallpaper { reason: String },
    /// The renderer could not start its wallpaper, for example because libmpv is missing (D-20).
    Failed { reason: String, hint: String },
    /// The restart budget of a monitor is spent (D-22): its renderer is not started again until Sucrose restarts.
    RendererStopped { monitor: String, reason: String },
}

impl Status {
    /// The sentence for the log.
    fn text(&self) -> String {
        match self {
            Self::Starting { .. } => APP_LONG_NAME.to_owned(),
            Self::Playing { title } => format!("Sucrose — playing {title}"),
            Self::NoWallpaper { reason } => format!("Sucrose — no wallpaper: {reason}"),
            Self::Failed { reason, hint } => format!("Sucrose — no wallpaper: {reason} {hint}"),
            Self::RendererStopped { monitor, reason } => format!("Sucrose — no wallpaper on {monitor}: {reason}"),
        }
    }

    /// The tray tooltip: the same sentence, cut to what the notification area keeps.
    fn tooltip(&self) -> String {
        sucrose_platform::truncate_tooltip(&self.text())
    }

    /// Whether no renderer runs only because no monitor was connected.
    fn is_waiting_for_a_monitor(&self) -> bool {
        matches!(self, Self::NoWallpaper { reason } if reason == NO_MONITOR)
    }

    /// Whether the user has to be told that no wallpaper plays.
    fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::NoWallpaper { .. } | Self::Failed { .. } | Self::RendererStopped { .. }
        )
    }

    /// The status while a crashed renderer is started again: a playing wallpaper is starting once more, and so is
    /// one that failed (the new renderer may well start). `title` is the selected theme's, which a failure does not
    /// carry.
    fn restarting(&self, title: Option<&str>) -> Option<Self> {
        match self {
            Self::Playing { title } => Some(Self::Starting { title: title.clone() }),
            Self::Failed { .. } => title.map(|title| Self::Starting {
                title: title.to_owned(),
            }),
            _ => None,
        }
    }

    /// The status while the renderer that ended with `class` is started again. A failed start keeps its failure: the
    /// reason the renderer reported is what the user needs, and the restart may fail the same way (WR-17).
    fn restarting_after(&self, class: ExitClass, title: Option<&str>) -> Option<Self> {
        if class == ExitClass::StartFailed && matches!(self, Self::Failed { .. }) {
            return None;
        }
        self.restarting(title)
    }

    /// Why the monitor's renderer is not started again once its budget is spent: the reason of the last failed start
    /// when the status still holds it (the sentence is cut at the end for the tooltip, so it comes first), otherwise
    /// the budget's own.
    fn stopped_reason(&self, exhausted: Exhausted) -> String {
        match self {
            Self::Failed { reason, .. } => format!("{reason} ({})", exhausted.reason()),
            _ => exhausted.reason(),
        }
    }

    /// The status a renderer message leads to, or `None` when it changes nothing. A renderer that plays replaces a
    /// failure too: after a restart, or while another monitor's renderer failed, the wallpaper does play. Only a
    /// status that says no renderer was wanted ([`Status::NoWallpaper`]) stays.
    fn after(&self, message: &ToService, title: Option<&str>) -> Option<Self> {
        match message {
            ToService::Playing => match (self, title) {
                (Self::Starting { title }, _) => Some(Self::Playing { title: title.clone() }),
                (Self::Failed { .. } | Self::RendererStopped { .. }, Some(title)) => Some(Self::Playing {
                    title: title.to_owned(),
                }),
                _ => None,
            },
            ToService::Failed { reason, hint } => Some(Self::Failed {
                reason: limit(reason),
                hint: limit(hint),
            }),
            ToService::Hello { .. } | ToService::Clock(_) => None,
        }
    }
}

/// `text` cut to [`MAX_REPORT_CHARS`] characters, with control characters made visible (see [`printable`]).
fn limit(text: &str) -> String {
    printable(&text.chars().take(MAX_REPORT_CHARS).collect::<String>())
}

/// `text` with every control character (a newline, an escape sequence) replaced by U+FFFD. Text that comes from a
/// theme or a renderer reaches the log and the tray tooltip, where it must not forge a log line or garble a tooltip
/// (IN-08).
fn printable(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

/// Creates the tray when General.json's AppVisible says so (D-15). With AppVisible false `create` is never called:
/// no icon, no menu and none of their resources exist (D-008). A tray that cannot be created is logged and the
/// service runs without one.
fn start_tray<T>(app_visible: bool, create: impl FnOnce() -> Result<T, PlatformError>) -> Option<T> {
    if !app_visible {
        tracing::info!("tray disabled: General.json AppVisible is false");
        return None;
    }
    match create() {
        Ok(tray) => {
            tracing::info!("tray icon ready");
            Some(tray)
        }
        Err(error) => {
            tracing::error!(%error, "cannot show the tray icon, Sucrose can only be stopped from Task Manager");
            None
        }
    }
}

/// Runs the service until Quit is chosen from the tray.
pub fn run() -> anyhow::Result<()> {
    let awareness = sucrose_platform::init_dpi_awareness();
    let endpoint = endpoint()?;
    let listener = match Listener::bind(&endpoint.name, endpoint.owner.as_deref()) {
        Ok(listener) => listener,
        Err(IpcError::AlreadyRunning) => {
            tracing::info!("Sucrose is already running; exiting");
            return Ok(());
        }
        Err(error) => return Err(error).context("cannot open the service endpoint"),
    };

    let platform = sucrose_platform::current();
    tracing::info!(
        pid = std::process::id(),
        version = sucrose_core::VERSION,
        app_version = %app_version(),
        os = platform.name(),
        capabilities = ?platform.capabilities(),
        endpoint = %endpoint.name,
        dpi = ?awareness,
        "service started"
    );
    let plan = plan_wallpapers();
    let assignments = SharedAssignments::default();
    let mut fleet = Fleet::new(plan.job, Arc::clone(&assignments));
    let mut status = plan.status;

    // The main thread only pumps messages and drains the channel (D-012): IPC reads and child waits run on threads
    // of their own and wake it through the proxy.
    let event_loop = EventLoop::new().context("cannot start the message loop")?;
    let (sender, receiver) = crossbeam_channel::unbounded();
    let notifier = Notifier {
        sender,
        proxy: event_loop.proxy(),
    };
    let renderers: SharedRenderers = Arc::default();
    let supervisor = Supervisor::new({
        let notifier = notifier.clone();
        move |exited| {
            notifier.send(Event::Exited(exited));
        }
    })?;
    // One renderer per connected monitor (D-01), each playing the same video on its own monitor's surface.
    if let Some(screen) = fleet.job.as_ref().map(|job| job.screen) {
        // Before the first renderer exists: Windows reads the per-app GPU preference when a process creates its
        // Direct3D device, and every renderer runs this same executable.
        apply_gpu_preference(&*platform, plan.gpu);
        let monitors = with_area(platform.monitors(), screen);
        let ids: Vec<&str> = monitors.iter().map(|monitor| monitor.id.as_str()).collect();
        if monitors.is_empty() {
            status = Status::NoWallpaper {
                reason: NO_MONITOR.to_owned(),
            };
        }
        let primary_id = primary(&monitors).map(|monitor| monitor.id.clone());
        let applied = fleet.apply(
            diff(&[], &monitors),
            &Ctx {
                supervisor: &supervisor,
                renderers: &renderers,
            },
        );
        if applied.started == 0 && applied.failed > 0 {
            anyhow::bail!("cannot start a renderer for any monitor");
        }
        tracing::info!(count = applied.started, monitors = ?ids, primary = ?primary_id, "renderers started");
        fleet.monitors = monitors;
        fleet.elect(&Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        });
    }
    let accept = Accept {
        renderers: Arc::clone(&renderers),
        supervisor: Arc::clone(&supervisor),
        assignments: Arc::clone(&assignments),
        notifier: notifier.clone(),
    };
    thread::Builder::new()
        .name("ipc-accept".to_owned())
        .spawn(move || accept_loop(&listener, &accept))
        .context("cannot start the accept thread")?;

    let tray_proxy = event_loop.proxy();
    let tray = start_tray(plan.app_visible, || Tray::new(TRAY_ICON, &status.tooltip(), tray_proxy));
    if status.is_failure() {
        tracing::warn!(status = %status.text(), "no wallpaper, the service waits in the tray");
    }

    let mut quitting = false;
    let timers = event_loop.timers();
    if fleet.enable_fullscreen(plan.fullscreen, plan.pause) {
        timers.set(FULLSCREEN_TIMER, FULLSCREEN_POLL_MS);
    }
    let loop_end = event_loop.run(|event| match event {
        LoopEvent::Wake => {
            for event in receiver.try_iter() {
                match event {
                    Event::Renderer {
                        slot,
                        message: ToService::Hello { pid, .. },
                        ..
                    } => {
                        tracing::info!(pid, slot, "renderer connected");
                        fleet.catch_up_renderer(slot, &renderers);
                    }
                    Event::Renderer { slot, pid, message } => {
                        let link_pid = lock(&renderers).connections.get(&slot).map(|link| link.pid);
                        // A report can be handled after the Exited of the same renderer (two threads feed the
                        // channel), or after the slot got another renderer; handle_report decides what still
                        // counts (WR-15, WR-18).
                        let Some(effect) = handle_report(
                            &status,
                            link_pid,
                            &supervisor.pids(),
                            slot,
                            pid,
                            &message,
                            fleet.job.as_ref().map(|job| job.title.as_str()),
                        ) else {
                            tracing::debug!(
                                slot,
                                pid,
                                "ignoring a report of a renderer that is gone or was replaced"
                            );
                            continue;
                        };
                        if effect.first_frame && fleet.playing.insert(slot) && !quitting {
                            // Its first frame is shown: only now can it be the master clock.
                            tracing::debug!(slot, "a renderer shows its first frame");
                            fleet.elect(&Ctx {
                                supervisor: &supervisor,
                                renderers: &renderers,
                            });
                        }
                        if let Some(next) = effect.status {
                            status = next;
                            if status.is_failure() {
                                tracing::warn!(status = %status.text(), "no wallpaper, the service waits in the tray");
                            } else {
                                tracing::info!(status = %status.text(), "wallpaper status");
                            }
                            if let Some(tray) = &tray {
                                tray.set_tooltip(&status.tooltip());
                            }
                        }
                    }
                    Event::Exited(exited) => {
                        let old_link_removed = {
                            let mut state = lock(&renderers);
                            let is_old = state
                                .connections
                                .get(&exited.slot)
                                .is_some_and(|link| link.pid == exited.pid);
                            if is_old {
                                state.connections.remove(&exited.slot);
                            }
                            is_old
                        };
                        // A replacement already runs for the slot (it was started in the gap between the old one
                        // ending and this event): this exit says nothing about the new renderer.
                        if exit_is_stale(&supervisor.pids(), exited.slot, exited.pid) {
                            tracing::info!(
                                slot = exited.slot,
                                pid = exited.pid,
                                "a replaced renderer ended, nothing to do"
                            );
                            // The old renderer's connection was still registered, so its replacement could not
                            // connect yet and its first frame cannot have been reported: the entry in `playing` is
                            // the old renderer's and must not count for the replacement (WR-16).
                            if forget_replaced_playing(&mut fleet.playing, exited.slot, old_link_removed) && !quitting {
                                fleet.elect(&Ctx {
                                    supervisor: &supervisor,
                                    renderers: &renderers,
                                });
                            }
                            continue;
                        }
                        // A renderer that ended plays no more and cannot stay the master; its replacement counts
                        // again once it shows its first frame.
                        let was_playing = fleet.playing.remove(&exited.slot);
                        if quitting {
                            continue;
                        }
                        if was_playing {
                            fleet.elect(&Ctx {
                                supervisor: &supervisor,
                                renderers: &renderers,
                            });
                        }
                        if fleet.slots.release(exited.slot) {
                            // The monitor was unplugged: this exit is the answer to the Close, nothing to restart.
                            fleet.budgets.remove(&exited.slot);
                            fleet.stopped.remove(&exited.slot);
                            fleet.shell.forget(exited.slot);
                            fleet.rule_closed.forget(exited.slot);
                            tracing::info!(slot = exited.slot, "the renderer of a removed monitor ended");
                            continue;
                        }
                        if fleet.rule_closed.note_exit(exited.slot) {
                            // The fullscreen rule closed this renderer: it is neither a crash nor restarted by the
                            // supervisor, the rule starts it again when the fullscreen app is gone (D-06).
                            fleet.stopped.insert(exited.slot);
                            fleet.shell.forget(exited.slot);
                            tracing::info!(slot = exited.slot, "the renderer closed for the fullscreen app ended");
                            continue;
                        }
                        let budget = fleet.budgets.entry(exited.slot).or_default();
                        let class = classify_exit(exited.code);
                        let decision = decide_exit(budget, &mut fleet.shell, class, exited.slot, Instant::now());
                        tracing::info!(slot = exited.slot, ?class, ?decision, "a renderer ended");
                        match decision {
                            Decision::Leave | Decision::GiveUp(_) => {
                                fleet.stopped.insert(exited.slot);
                            }
                            Decision::Restart(after) => {
                                fleet.stopped.remove(&exited.slot);
                                tracing::warn!(slot = exited.slot, ?class, ?after, "restarting the renderer");
                                supervisor.restart_after(exited.slot, after, i32::from(START_FAILED));
                            }
                        }
                        let monitor = fleet
                            .slots
                            .monitor_of(exited.slot)
                            .map_or_else(|| format!("slot {}", exited.slot), str::to_owned);
                        let next = status_after_exit(
                            &status,
                            decision,
                            class,
                            fleet.job.as_ref().map(|job| job.title.as_str()),
                            monitor,
                        );
                        if let (Some(next), Decision::GiveUp(_)) = (&next, decision) {
                            tracing::warn!(status = %next.text(), "no wallpaper, the service waits in the tray");
                        }
                        if let Some(next) = next {
                            status = next;
                            if let Some(tray) = &tray {
                                tray.set_tooltip(&status.tooltip());
                            }
                        }
                    }
                    Event::ShutdownDone => return Flow::Exit,
                }
            }
            Flow::Continue
        }
        LoopEvent::Quit => {
            if !quitting {
                quitting = true;
                tracing::info!("Quit chosen, closing the renderers");
                // No monitor change is applied while the renderers are being closed.
                for id in [TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE, FULLSCREEN_TIMER] {
                    timers.kill(id);
                }
                start_shutdown(&renderers, &supervisor, &notifier);
            }
            Flow::Continue
        }
        // Monitors and DPI are the service's to watch: a surface parented into the desktop no longer hears about
        // them. The events come in bursts, so they only (re)start the debounce.
        LoopEvent::DisplayChanged => {
            if !quitting && fleet.job.is_some() {
                run_timers(&timers, fleet.debounce.display_changed().timers);
            }
            Flow::Continue
        }
        LoopEvent::Timer(FULLSCREEN_TIMER) => {
            if quitting {
                timers.kill(FULLSCREEN_TIMER);
            } else {
                fleet.fullscreen_tick(
                    platform.as_ref(),
                    &Ctx {
                        supervisor: &supervisor,
                        renderers: &renderers,
                    },
                );
            }
            Flow::Continue
        }
        LoopEvent::Timer(id) => {
            let Some(screen) = fleet.job.as_ref().map(|job| job.screen) else {
                timers.kill(id);
                return Flow::Continue;
            };
            let step = match id {
                TOPOLOGY_DEBOUNCE => fleet.debounce.debounce_fired(with_area(platform.monitors(), screen)),
                TOPOLOGY_SETTLE => fleet.debounce.settle_fired(with_area(platform.monitors(), screen)),
                other => {
                    tracing::debug!(id = other, "an unknown timer fired, stopping it");
                    timers.kill(other);
                    return Flow::Continue;
                }
            };
            run_timers(&timers, step.timers);
            if let (Some(monitors), false) = (step.apply, quitting) {
                let applied = fleet.apply_snapshot(
                    monitors,
                    &Ctx {
                        supervisor: &supervisor,
                        renderers: &renderers,
                    },
                );
                if applied.started > 0 && status.is_waiting_for_a_monitor() {
                    if let Some(job) = &fleet.job {
                        status = Status::Starting {
                            title: job.title.clone(),
                        };
                    }
                    if let Some(tray) = &tray {
                        tray.set_tooltip(&status.tooltip());
                    }
                }
            }
            Flow::Continue
        }
        // explorer.exe started again: the surfaces have to be replaced, so each monitor gets a new renderer.
        LoopEvent::ShellRestarted => {
            if !quitting && fleet.job.is_some() {
                let acted = fleet.shell_restarted(&Ctx {
                    supervisor: &supervisor,
                    renderers: &renderers,
                });
                if acted && let Some(next) = status.restarting(fleet.job.as_ref().map(|job| job.title.as_str())) {
                    status = next;
                    if let Some(tray) = &tray {
                        tray.set_tooltip(&status.tooltip());
                    }
                }
            }
            Flow::Continue
        }
    });
    drop(tray);
    if loop_end == LoopEnd::Failed {
        // Not a Quit: the renderers die with the job object, and the failure must reach the exit code and the log.
        anyhow::bail!("the message loop failed, the service cannot run without it");
    }
    tracing::info!("the service stopped");
    Ok(())
}

/// Closes every renderer on a worker thread, so the message loop (and with it the tray) stays responsive while a
/// renderer takes its time (T-02-20).
fn start_shutdown(renderers: &SharedRenderers, supervisor: &Arc<Supervisor>, notifier: &Notifier) {
    let renderers = Arc::clone(renderers);
    let supervisor = Arc::clone(supervisor);
    let notifier = notifier.clone();
    let started = thread::Builder::new().name("shutdown".to_owned()).spawn({
        let notifier = notifier.clone();
        move || {
            let mut control = {
                let mut state = lock(&renderers);
                state.closing = true;
                supervisor.begin_shutdown();
                ProcessRenderers {
                    supervisor,
                    connections: std::mem::take(&mut state.connections),
                }
            };
            let had_renderers = !control.slots().is_empty();
            shutdown(&mut control, SHUTDOWN_GRACE);
            if had_renderers {
                // The renderers' surfaces are gone: make the shell paint the normal wallpaper again.
                sucrose_platform::refresh_desktop();
            }
            notifier.send(Event::ShutdownDone);
        }
    });
    if let Err(error) = started {
        // Without the worker nothing can wait for the children, so leave at once: the OS ends children that hold a
        // connection to a dead service when they notice the close.
        tracing::error!(%error, "cannot start the shutdown thread, exiting without waiting for the renderers");
        notifier.send(Event::ShutdownDone);
    }
}

/// What shutdown needs from the renderers, as a trait so the order of the steps can be tested.
trait RendererControl {
    /// The slots of the renderers that were started.
    fn slots(&self) -> Vec<u8>;
    /// Asks the renderer to stop; best effort.
    fn close(&mut self, slot: u8);
    /// Waits up to `timeout` for the renderer to be gone; `true` if it is.
    fn wait_exit(&mut self, slot: u8, timeout: Duration) -> bool;
    /// Stops the renderer by force.
    fn kill(&mut self, slot: u8);
}

/// Asks every renderer to close, gives them `grace` in total to leave, then kills only the ones still alive.
fn shutdown(control: &mut impl RendererControl, grace: Duration) {
    let slots = control.slots();
    for slot in &slots {
        control.close(*slot);
    }
    let deadline = Instant::now() + grace;
    let mut survivors = Vec::new();
    for slot in slots {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if !control.wait_exit(slot, remaining) {
            survivors.push(slot);
        }
    }
    for slot in survivors {
        tracing::warn!(slot, "a renderer did not leave in time, stopping it");
        control.kill(slot);
    }
}

/// The real renderers: only the processes this service spawned are ever touched (D-14).
struct ProcessRenderers {
    supervisor: Arc<Supervisor>,
    connections: HashMap<u8, Link>,
}

impl RendererControl for ProcessRenderers {
    fn slots(&self) -> Vec<u8> {
        self.supervisor.slots()
    }

    fn close(&mut self, slot: u8) {
        match self.connections.get(&slot) {
            Some(link) => {
                if link.offer(ToRenderer::Close) == Offer::Failed {
                    tracing::warn!(slot, "cannot send Close to a renderer");
                }
            }
            None => tracing::debug!(slot, "no connection to send Close to, the renderer is still starting"),
        }
    }

    fn wait_exit(&mut self, slot: u8, timeout: Duration) -> bool {
        self.supervisor.wait_exit(slot, timeout)
    }

    fn kill(&mut self, _slot: u8) {
        // Only the renderers that ignored Close are left when this runs, so ending the job ends exactly those.
        self.supervisor.kill_survivors();
    }
}

/// What the accept thread and the reader threads need.
struct Accept {
    renderers: SharedRenderers,
    supervisor: Arc<Supervisor>,
    assignments: SharedAssignments,
    notifier: Notifier,
}

/// Accepts connections and gives each one a reader thread.
fn accept_loop(listener: &Listener, accept: &Accept) {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    while failures < MAX_FAILURES {
        match listener.accept() {
            Ok(connection) => {
                failures = 0;
                let renderers = Arc::clone(&accept.renderers);
                let supervisor = Arc::clone(&accept.supervisor);
                let assignments = Arc::clone(&accept.assignments);
                let notifier = accept.notifier.clone();
                let started = thread::Builder::new()
                    .name("ipc-reader".to_owned())
                    .spawn(move || read_connection(connection, &renderers, &supervisor, &assignments, &notifier));
                if let Err(error) = started {
                    tracing::warn!(%error, "cannot start a reader thread, dropping the connection");
                }
            }
            Err(error) => {
                failures += 1;
                tracing::warn!(%error, failures, "accepting a connection failed");
                thread::sleep(Duration::from_millis(200));
            }
        }
    }
    tracing::error!("accepting connections keeps failing, the service accepts no more renderers");
}

/// Reads one connection until it ends. Whatever goes wrong here ends this connection only.
fn read_connection(
    mut connection: Connection,
    renderers: &Mutex<Renderers>,
    supervisor: &Supervisor,
    assignments: &Mutex<HashMap<u8, Assignment>>,
    notifier: &Notifier,
) {
    let first: ToService = match connection.recv() {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(%error, "dropping a connection whose first message is not valid");
            return;
        }
    };
    let (accepted, closing) = {
        let state = lock(renderers);
        let connected: HashSet<u8> = state.connections.keys().copied().collect();
        (accept_hello(&supervisor.pids(), &connected, &first), state.closing)
    };
    let slot = match accepted {
        Ok(slot) => slot,
        Err(refusal) => {
            tracing::warn!(%refusal, "refusing a connection");
            return;
        }
    };
    let ToService::Hello { pid, .. } = &first else {
        return;
    };
    let pid = *pid;
    // The renderer waits for this message; without a job for its slot, or once shutdown has begun, it has nothing
    // to do, so it is told to stop.
    let assignment = lock_assignments(assignments).get(&slot).cloned();
    let reply = match assignment {
        Some(assignment) if !closing => ToRenderer::Assign(assignment),
        _ => ToRenderer::Close,
    };
    if let Err(error) = connection.send(&reply) {
        tracing::warn!(%error, slot, "cannot answer the Hello");
        return;
    }
    match &reply {
        ToRenderer::Assign(assignment) => tracing::info!(
            slot,
            monitor = %assignment.monitor,
            rect = ?assignment.rect,
            "Hello answered with an Assign"
        ),
        ToRenderer::Close => tracing::warn!(slot, "no assignment for this slot, the renderer was told to close"),
        // The reply to a Hello is only ever an Assign or a Close.
        _ => {}
    }
    if !register_writer(renderers, slot, pid, &mut connection) {
        return;
    }
    if matches!(reply, ToRenderer::Assign(_)) {
        send_role(renderers, slot);
    }
    // A change that came between the reply and the registration found no connection to send to: catch up.
    if let ToRenderer::Assign(sent) = &reply {
        // The assignments lock is released before the send takes the renderers lock.
        let current = lock_assignments(assignments)
            .get(&slot)
            .map(|assignment| assignment.rect);
        match current {
            // Through the registered writer, under the lock every other sender takes (WR-12).
            Some(rect) if rect != sent.rect => {
                send_to(renderers, slot, &ToRenderer::SetRect(rect));
            }
            Some(_) => {}
            None => {
                send_to(renderers, slot, &ToRenderer::Close);
            }
        }
    }
    if !notifier.send(Event::Renderer {
        slot,
        pid,
        message: first,
    }) {
        return;
    }
    loop {
        match connection.recv::<ToService>() {
            // The clock goes straight from this thread to the followers: it is frequent and changes no status.
            Ok(ToService::Clock(tick)) => relay_clock(renderers, slot, tick),
            Ok(message) => {
                if !notifier.send(Event::Renderer { slot, pid, message }) {
                    return;
                }
            }
            Err(IpcError::Closed) => {
                tracing::info!("a renderer disconnected");
                return;
            }
            Err(error) => {
                tracing::warn!(%error, "dropping a connection after an invalid message");
                return;
            }
        }
    }
}

/// Remembers the write side of a renderer's connection so shutdown can send Close while this thread is blocked
/// reading. When shutdown began in the meantime, the Close goes out right here instead. `false` when the connection
/// must not be served: its write handle cannot be kept, or another renderer got the slot first (a slot's connection is
/// never replaced).
fn register_writer(renderers: &Mutex<Renderers>, slot: u8, pid: u32, connection: &mut Connection) -> bool {
    let writer = match connection
        .try_clone()
        .map_err(|error| error.to_string())
        .and_then(|clone| Link::for_connection(slot, pid, clone).map_err(|error| error.to_string()))
    {
        Ok(writer) => writer,
        Err(error) => {
            tracing::warn!(%error, slot, "cannot keep a write handle for a renderer");
            return false;
        }
    };
    let mut state = lock(renderers);
    if state.closing {
        drop(state);
        if let Err(error) = connection.send(&ToRenderer::Close) {
            tracing::warn!(%error, slot, "cannot send Close to a renderer");
        }
        true
    } else if let Entry::Vacant(vacant) = state.connections.entry(slot) {
        vacant.insert(writer);
        true
    } else {
        drop(state);
        tracing::warn!(
            slot,
            "refusing a second connection for a slot that already has a renderer"
        );
        false
    }
}

/// Tells the renderer of `slot` whether it is the master clock or a follower, through the connection kept for the
/// service's own sends so that it cannot interleave with another message.
fn send_role(renderers: &Mutex<Renderers>, slot: u8) {
    let state = lock(renderers);
    let role = if state.master == Some(slot) {
        Role::Master
    } else {
        Role::Follower
    };
    if let Some(link) = state.connections.get(&slot) {
        match link.offer(ToRenderer::SetRole(role)) {
            Offer::Failed => tracing::warn!(slot, "cannot send the role to a renderer"),
            Offer::Queued | Offer::Dropped => tracing::info!(slot, ?role, "role sent to a renderer"),
        }
    }
}

/// Which slots get a clock reading that the renderer of `from` published: every other connected renderer when
/// `from` is the master, nobody otherwise (T-02-30).
fn clock_targets(master: Option<u8>, from: u8, connected: impl IntoIterator<Item = u8>) -> Vec<u8> {
    if master != Some(from) {
        return Vec::new();
    }
    let mut targets: Vec<u8> = connected.into_iter().filter(|slot| *slot != from).collect();
    targets.sort_unstable();
    targets
}

/// Hands the master's clock reading to the followers; a reading from any other renderer is ignored.
fn relay_clock(renderers: &Mutex<Renderers>, from: u8, tick: ClockTick) {
    let state = lock(renderers);
    if state.master != Some(from) {
        tracing::debug!(
            slot = from,
            "ignoring a clock reading from a renderer that is not the master"
        );
        return;
    }
    let targets = clock_targets(state.master, from, state.connections.keys().copied());
    for slot in targets {
        if let Some(link) = state.connections.get(&slot) {
            match link.offer(ToRenderer::Clock(tick)) {
                Offer::Queued => {}
                Offer::Dropped => tracing::debug!(slot, "a renderer is behind, a clock reading was skipped"),
                Offer::Failed => tracing::warn!(slot, "cannot send the clock to a renderer"),
            }
        }
    }
}

/// The monitor whose renderer is the master clock (D-030). `current` is the monitor of the present master and
/// `playing` the monitors whose renderer shows its first frame. The master is sticky: the present one keeps the role
/// while its monitor is still among `playing`, so plugging a monitor, even one that becomes the primary, never moves
/// the clock or the audio. With no master, or when the master's monitor is gone or its renderer stopped, the primary
/// monitor among `playing` takes over, else the lowest device name.
fn choose_master(current: Option<&str>, playing: &[MonitorInfo]) -> Option<String> {
    if let Some(current) = current
        && playing.iter().any(|monitor| monitor.id == current)
    {
        return Some(current.to_owned());
    }
    playing
        .iter()
        .find(|monitor| monitor.primary)
        .or_else(|| playing.iter().min_by(|a, b| a.id.cmp(&b.id)))
        .map(|monitor| monitor.id.clone())
}

/// What changes for the renderers when the master moves from `old` to `new`: the new master is told to take the audio
/// at `volume`, and the old one, if it still has a monitor (`active`), goes back to following.
fn role_changes(old: Option<u8>, new: Option<u8>, active: &[u8], volume: u8) -> Vec<(u8, ToRenderer)> {
    let Some(new) = new else {
        return Vec::new();
    };
    if old == Some(new) {
        return Vec::new();
    }
    let mut changes = vec![
        (new, ToRenderer::SetRole(Role::Master)),
        (new, ToRenderer::SetVolume(volume)),
    ];
    if let Some(old) = old.filter(|slot| active.contains(slot)) {
        changes.push((old, ToRenderer::SetRole(Role::Follower)));
    }
    changes
}

/// What the main thread does about a report that still counts.
#[derive(Debug, PartialEq, Eq)]
struct ReportEffect {
    /// The report says the renderer shows its first frame: it may become the master clock.
    first_frame: bool,
    /// The new status, `None` when the report changes nothing.
    status: Option<Status>,
}

/// Decides what a report of process `pid` on `slot` does, or `None` when it is dropped. `link_pid` is the process
/// whose connection the slot holds and `running` the processes the supervisor runs (pid to slot). A renderer registers
/// its connection before its first report is queued and the connection goes when the renderer ended, so:
/// - a report of a renderer that was replaced (another connection, or another process runs for the slot) is dropped;
/// - a report of a renderer that is gone and has no replacement can still be its last word: `Failed` is the one
///   report a renderer sends right before it ends, and its exit may be handled first (two threads feed the channel),
///   so it is kept (WR-18). A late `Playing` would claim a wallpaper nobody shows and is dropped (WR-15).
fn handle_report(
    status: &Status,
    link_pid: Option<u32>,
    running: &HashMap<u32, u8>,
    slot: u8,
    pid: u32,
    message: &ToService,
    title: Option<&str>,
) -> Option<ReportEffect> {
    let replaced = link_pid.is_some_and(|current| current != pid) || exit_is_stale(running, slot, pid);
    let late = link_pid != Some(pid);
    if replaced || (late && matches!(message, ToService::Playing)) {
        return None;
    }
    // The restart budget's verdict stays when the last words of the renderer that spent it arrive after its exit:
    // the status already says the monitor gets no more renderers.
    let kept = late && matches!(message, ToService::Failed { .. }) && matches!(status, Status::RendererStopped { .. });
    Some(ReportEffect {
        first_frame: matches!(message, ToService::Playing),
        status: if kept { None } else { status.after(message, title) },
    })
}

/// The status after a renderer ended and `decision` was taken: a restart keeps a failed start's reason, a spent budget
/// names the monitor and why (WR-17). `None` when nothing changes.
fn status_after_exit(
    status: &Status,
    decision: Decision,
    class: ExitClass,
    title: Option<&str>,
    monitor: String,
) -> Option<Status> {
    match decision {
        Decision::Leave => None,
        Decision::Restart(_) => status.restarting_after(class, title),
        Decision::GiveUp(exhausted) => Some(Status::RendererStopped {
            monitor,
            reason: status.stopped_reason(exhausted),
        }),
    }
}

/// Removes the `playing` entry of `slot` when the renderer that ended was the one whose connection the slot held
/// (`old_link_removed`) and a replacement already runs: the entry is the old renderer's. `true` when an entry was
/// removed, so that the master has to be elected again.
fn forget_replaced_playing(playing: &mut HashSet<u8>, slot: u8, old_link_removed: bool) -> bool {
    old_link_removed && playing.remove(&slot)
}

/// Whether the end of process `pid` is that of a renderer that was replaced: another process (`running`, pid to slot)
/// already runs for `slot`.
fn exit_is_stale(running: &HashMap<u32, u8>, slot: u8, pid: u32) -> bool {
    running
        .iter()
        .any(|(other, other_slot)| *other_slot == slot && *other != pid)
}

/// Why the service refuses a renderer's Hello.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The renderer speaks another protocol version.
    ProtocolVersion { got: u16, expected: u16 },
    /// The pid is not a process this service started.
    UnknownPid(u32),
    /// The pid was started for another slot than the one the Hello claims.
    WrongSlot { pid: u32, claimed: u8, started_for: u8 },
    /// A renderer is already connected for the claimed slot.
    AlreadyConnected(u8),
    /// The first message of a connection is not a Hello.
    NotHello,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolVersion { got, expected } => {
                write!(f, "the renderer speaks protocol {got}, this service speaks {expected}")
            }
            Self::UnknownPid(pid) => write!(f, "pid {pid} is not a renderer this service started"),
            Self::WrongSlot {
                pid,
                claimed,
                started_for,
            } => write!(
                f,
                "pid {pid} claims slot {claimed} but was started for slot {started_for}"
            ),
            Self::AlreadyConnected(slot) => write!(f, "slot {slot} already has a connected renderer"),
            Self::NotHello => write!(f, "the first message is not a Hello"),
        }
    }
}

/// Decides whether a connection's first message is acceptable and returns the renderer's slot. The slot is the one the
/// pid was started for, so a peer cannot claim another monitor's slot, and a slot that already has a connected renderer
/// is never taken over by a second Hello.
fn accept_hello(spawned: &HashMap<u32, u8>, connected: &HashSet<u8>, message: &ToService) -> Result<u8, Refusal> {
    let ToService::Hello {
        protocol_version,
        pid,
        slot,
    } = message
    else {
        return Err(Refusal::NotHello);
    };
    if *protocol_version != PROTOCOL_VERSION {
        return Err(Refusal::ProtocolVersion {
            got: *protocol_version,
            expected: PROTOCOL_VERSION,
        });
    }
    let Some(started_for) = spawned.get(pid).copied() else {
        return Err(Refusal::UnknownPid(*pid));
    };
    if started_for != *slot {
        return Err(Refusal::WrongSlot {
            pid: *pid,
            claimed: *slot,
            started_for,
        });
    }
    if connected.contains(slot) {
        return Err(Refusal::AlreadyConnected(*slot));
    }
    Ok(started_for)
}

/// What the settings and the selected theme lead to.
struct Plan {
    /// What every renderer plays and how, when the selected theme is a video this build can play.
    job: Option<Job>,
    /// What the tray says before any renderer has reported.
    status: Status,
    /// General.json AppVisible: whether the tray exists at all (D-15).
    app_visible: bool,
    /// Backgroundog.json FullScreenPerformance: what a fullscreen app in front does to the wallpaper.
    fullscreen: FullScreenAction,
    /// Backgroundog.json PausePerformanceType.
    pause: PauseKind,
    /// Engine.json GraphicPreference: the GPU Windows is asked to give the renderers (DESK-12).
    gpu: GpuPreference,
}

/// What every renderer plays: the same video with the same settings on each monitor (D-01).
struct Job {
    /// The theme's title, for the tooltip.
    title: String,
    /// The absolute path of the video.
    source: String,
    /// How it is played.
    settings: PlaybackSettings,
    /// Whether a surface covers its whole monitor or only the work area (Engine.json ScreenType).
    screen: ScreenArea,
}

impl Job {
    /// The assignment of the renderer `slot` for `monitor`, whose rectangle is already the area to cover.
    fn assignment(&self, slot: u8, monitor: &MonitorInfo) -> Assignment {
        let area = monitor.rect;
        Assignment {
            slot,
            monitor: monitor.id.clone(),
            rect: [area.left, area.top, area.right, area.bottom],
            source: self.source.clone(),
            settings: self.settings,
        }
    }
}

/// Sends `message` to the renderer of `slot`. `false` when it has no connection (it is still starting) or the send
/// failed.
fn send_to(renderers: &Mutex<Renderers>, slot: u8, message: &ToRenderer) -> bool {
    match lock(renderers).connections.get(&slot) {
        Some(link) => match link.offer(message.clone()) {
            Offer::Queued | Offer::Dropped => true,
            Offer::Failed => {
                tracing::warn!(slot, "cannot send a message to a renderer");
                false
            }
        },
        None => false,
    }
}

/// Starts and stops loop timers as the debounce asks.
fn run_timers(timers: &LoopTimers, commands: Vec<TimerCommand>) {
    for command in commands {
        match command {
            TimerCommand::Arm { id, ms } => timers.set(id, ms),
            TimerCommand::Kill(id) => timers.kill(id),
        }
    }
}

/// What the fleet needs from the rest of the service to act on a change.
struct Ctx<'a> {
    supervisor: &'a Arc<Supervisor>,
    renderers: &'a SharedRenderers,
}

/// How many renderers an application of monitor changes started and could not start.
#[derive(Debug, Default, PartialEq, Eq)]
struct Applied {
    started: usize,
    failed: usize,
}

/// The renderers of the monitors: one slot per monitor, keyed by the monitor's device name so that the same monitors
/// in another enumeration order change nothing, and nothing can start twice for one monitor.
struct Fleet {
    /// `None` when no renderer is wanted (the theme is rejected).
    job: Option<Job>,
    slots: SlotMap,
    /// The monitors the renderers follow, as last applied, with the rectangle of the area they cover.
    monitors: Vec<MonitorInfo>,
    assignments: SharedAssignments,
    debounce: Debounce,
    /// One restart budget per monitor slot (D-22).
    budgets: HashMap<u8, RestartBudget>,
    /// Slots whose renderer ended for good (nothing to restart): no exit event will ever come for them again.
    stopped: HashSet<u8>,
    /// The recovery after explorer.exe restarted (D-11).
    shell: ShellRestart,
    /// The fullscreen rule; `None` when FullScreenPerformance is Resume (the legacy default) or nothing plays.
    fullscreen: Option<Rule>,
    /// The renderers the fullscreen rule closed.
    rule_closed: RuleClosed,
    /// The slots whose renderer showed its first frame and is still running: only these can be the master clock.
    playing: HashSet<u8>,
}

/// The renderers the fullscreen rule closed (FullScreenPerformance = Close) and has to bring back (D-06). A slot is
/// only started again once the exit event of its old renderer has been seen, so the old and the new renderer can
/// never be confused.
#[derive(Debug, Default)]
struct RuleClosed {
    /// Slots whose renderer was closed by the rule.
    closed: HashSet<u8>,
    /// The closed slots whose renderer has ended.
    ended: HashSet<u8>,
    /// The closed slots the rule wants back.
    back: HashSet<u8>,
}

impl RuleClosed {
    fn close(&mut self, slot: u8, already_ended: bool) {
        self.closed.insert(slot);
        self.back.remove(&slot);
        if already_ended {
            self.ended.insert(slot);
        } else {
            self.ended.remove(&slot);
        }
    }

    fn is_closed(&self, slot: u8) -> bool {
        self.closed.contains(&slot)
    }

    /// The renderer of `slot` ended; `true` when that is the exit the rule asked for.
    fn note_exit(&mut self, slot: u8) -> bool {
        if self.closed.contains(&slot) {
            self.ended.insert(slot);
            true
        } else {
            false
        }
    }

    /// Whether the renderer of `slot` still has to say it ended, so a Close has to reach it.
    fn awaiting_exit(&self, slot: u8) -> bool {
        self.closed.contains(&slot) && !self.ended.contains(&slot)
    }

    fn want_back(&mut self, slot: u8) {
        if self.closed.contains(&slot) {
            self.back.insert(slot);
        }
    }

    /// The slots that are wanted back and whose old renderer is gone.
    fn ready(&self) -> Vec<u8> {
        let mut ready: Vec<u8> = self.back.intersection(&self.ended).copied().collect();
        ready.sort_unstable();
        ready
    }

    fn forget(&mut self, slot: u8) {
        self.closed.remove(&slot);
        self.ended.remove(&slot);
        self.back.remove(&slot);
    }
}

impl Fleet {
    fn new(job: Option<Job>, assignments: SharedAssignments) -> Self {
        Self {
            job,
            slots: SlotMap::default(),
            monitors: Vec::new(),
            assignments,
            debounce: Debounce::default(),
            budgets: HashMap::new(),
            stopped: HashSet::new(),
            shell: ShellRestart::default(),
            fullscreen: None,
            rule_closed: RuleClosed::default(),
            playing: HashSet::new(),
        }
    }

    /// Switches the fullscreen rule on for `action` unless it is Resume, which disables it: then no timer and no
    /// state exist at all (D-008). `true` when the rule is on and its timer has to run.
    fn enable_fullscreen(&mut self, action: FullScreenAction, pause: PauseKind) -> bool {
        if action == FullScreenAction::Resume || self.job.is_none() {
            tracing::info!("fullscreen rule off (FullScreenPerformance = Resume)");
            return false;
        }
        if pause == PauseKind::Heavy {
            tracing::info!("PausePerformanceType Heavy is handled like Light in M1");
        }
        tracing::info!(?action, "fullscreen rule on");
        self.fullscreen = Some(Rule::new(action, ACTION_SCOPE));
        true
    }

    /// One 1 s sample of the foreground window: after enough equal samples the renderers are paused, resumed, closed
    /// or started again.
    fn fullscreen_tick(&mut self, platform: &dyn sucrose_platform::Platform, ctx: &Ctx<'_>) {
        let Some(rule) = self.fullscreen.as_mut() else {
            return;
        };
        let mut own_pids: Vec<u32> = ctx.supervisor.pids().into_keys().collect();
        own_pids.push(std::process::id());
        let foreground = platform.foreground_window();
        // The whole monitor rectangles, not the work areas the surfaces may cover: a maximized window must not count.
        let covered = covered_monitors(foreground.as_ref(), &platform.monitors(), &own_pids);
        let all: Vec<String> = self.monitors.iter().map(|monitor| monitor.id.clone()).collect();
        let actions = rule.tick(&covered, &all);
        if !actions.is_empty() {
            tracing::info!(action = ?actions.first().map(|(_, step)| *step), monitors = ?covered, renderers = actions.len(), "fullscreen rule changed");
        }
        for (monitor, action) in actions {
            let Some(slot) = self.slots.slot_of(&monitor) else {
                continue;
            };
            match action {
                RuleAction::Pause | RuleAction::Resume => {
                    send_to(ctx.renderers, slot, &ToRenderer::Pause(action == RuleAction::Pause));
                }
                RuleAction::Close => {
                    // A slot with nothing running has nothing to close and nothing to bring back.
                    if ctx.supervisor.slots().contains(&slot) {
                        self.rule_closed.close(slot, false);
                        send_to(ctx.renderers, slot, &ToRenderer::Close);
                    }
                }
                RuleAction::Respawn => self.rule_closed.want_back(slot),
            }
        }
        self.respawn_closed(ctx);
    }

    /// Starts the renderers the rule closed and wants back, once their old renderer has ended. Retried on every tick
    /// until it works.
    fn respawn_closed(&mut self, ctx: &Ctx<'_>) {
        for slot in self.rule_closed.ready() {
            if self.job.is_none() || self.slots.monitor_of(slot).is_none() {
                self.rule_closed.forget(slot);
                continue;
            }
            self.budgets.remove(&slot);
            self.stopped.remove(&slot);
            match ctx.supervisor.spawn(slot) {
                Ok(()) => {
                    tracing::info!(slot, "the fullscreen app is gone, starting the renderer again");
                    self.rule_closed.forget(slot);
                }
                Err(error) => {
                    tracing::warn!(%error, slot, "cannot start the renderer after the fullscreen app, will retry")
                }
            }
        }
    }

    /// A renderer said Hello: it is told what the fullscreen rule decided before it connected.
    fn catch_up_renderer(&self, slot: u8, renderers: &SharedRenderers) {
        if self.rule_closed.awaiting_exit(slot) {
            send_to(renderers, slot, &ToRenderer::Close);
        } else if let (Some(rule), Some(monitor)) = (&self.fullscreen, self.slots.monitor_of(slot))
            && rule.pauses(monitor)
        {
            send_to(renderers, slot, &ToRenderer::Pause(true));
        }
    }

    /// The slot of the master clock: the renderer of the monitor `choose_master` picks among those whose renderer
    /// plays, `current` being the slot of the present master. A renderer that has not shown its first frame has no
    /// clock yet, so it is never elected.
    fn master_slot(&self, current: Option<u8>) -> Option<u8> {
        let candidates: Vec<MonitorInfo> = self
            .monitors
            .iter()
            .filter(|monitor| {
                self.slots
                    .slot_of(&monitor.id)
                    .is_some_and(|slot| self.playing.contains(&slot))
            })
            .cloned()
            .collect();
        let current = current.and_then(|slot| self.slots.monitor_of(slot));
        choose_master(current, &candidates).and_then(|id| self.slots.slot_of(&id))
    }

    /// Elects the master again for the monitors as they are now. When it changed, the new master is told to take the
    /// audio at the configured volume and the old one, if it still has a monitor, to follow (D-02). A renderer that
    /// is still starting has no connection yet and gets its role when it says Hello.
    fn elect(&self, ctx: &Ctx<'_>) {
        let volume = self.job.as_ref().map_or(100, |job| job.settings.volume);
        let mut state = lock(ctx.renderers);
        let old = state.master;
        let wanted = self.master_slot(old);
        if old == wanted {
            return;
        }
        state.master = wanted;
        tracing::info!(?old, new = ?wanted, "master clock elected");
        for (slot, message) in role_changes(old, wanted, &self.slots.active_slots(), volume) {
            if let Some(link) = state.connections.get(&slot)
                && link.offer(message) == Offer::Failed
            {
                tracing::warn!(slot, "cannot send a role change to a renderer");
            }
        }
    }

    /// Applies the monitors of a snapshot that two looks agreed on. An empty snapshot is the OS in the middle of a
    /// hot-plug and changes nothing: the renderers stay until a snapshot with monitors arrives.
    fn apply_snapshot(&mut self, monitors: Vec<MonitorInfo>, ctx: &Ctx<'_>) -> Applied {
        if monitors.is_empty() {
            tracing::info!("the OS reports no monitor, the renderers stay as they are");
            return Applied::default();
        }
        let actions = diff(&self.monitors, &monitors);
        if actions.is_empty() {
            tracing::debug!("the monitors did not change in a way that matters");
            return Applied::default();
        }
        tracing::info!(actions = actions.len(), "applying a monitor change");
        let applied = self.apply(actions, ctx);
        self.monitors = monitors;
        self.elect(ctx);
        applied
    }

    /// Carries out `actions` (see [`diff`]).
    fn apply(&mut self, actions: Vec<TopologyAction>, ctx: &Ctx<'_>) -> Applied {
        let mut applied = Applied::default();
        for action in actions {
            match action {
                TopologyAction::Spawn(monitor) => {
                    if self.spawn(&monitor, ctx) {
                        applied.started += 1;
                    } else {
                        applied.failed += 1;
                    }
                }
                TopologyAction::Close(id) => self.close(&id, ctx),
                TopologyAction::SetRect(id, rect) => self.move_to(&id, rect, ctx),
            }
        }
        applied
    }

    /// Starts the renderer for `monitor`; `false` when it could not be started.
    fn spawn(&mut self, monitor: &MonitorInfo, ctx: &Ctx<'_>) -> bool {
        let Some(job) = &self.job else {
            return false;
        };
        let Some(slot) = self.slots.assign(&monitor.id) else {
            tracing::error!(monitor = %monitor.id, "no free renderer slot");
            return false;
        };
        lock_assignments(&self.assignments).insert(slot, job.assignment(slot, monitor));
        self.budgets.remove(&slot);
        self.stopped.remove(&slot);
        if self.fullscreen.as_ref().is_some_and(|rule| rule.closes(&monitor.id)) {
            // A monitor that appears while a fullscreen app has the renderers closed gets its renderer with the others.
            self.rule_closed.close(slot, true);
            tracing::info!(slot, monitor = %monitor.id, "a fullscreen app is in front, the renderer of the new monitor waits");
            return true;
        }
        match ctx.supervisor.spawn(slot) {
            Ok(()) => true,
            Err(error) => {
                tracing::error!(%error, slot, monitor = %monitor.id, "cannot start the renderer");
                false
            }
        }
    }

    /// explorer.exe started again. Renderers that still run are closed (their surfaces have to be replaced) and each
    /// monitor's renderer is started once more when its predecessor has ended; a monitor without a running renderer
    /// gets a fresh one at once. `false` when restarts come too fast and this one was ignored.
    fn shell_restarted(&mut self, ctx: &Ctx<'_>) -> bool {
        let now = Instant::now();
        let running = ctx.supervisor.slots();
        let mut to_close = Vec::new();
        let mut to_start = Vec::new();
        for slot in self.slots.active_slots() {
            // A renderer the fullscreen rule closed comes back with the rule, not with the shell.
            if self.rule_closed.is_closed(slot) {
                continue;
            }
            if !running.contains(&slot) {
                to_start.push(slot);
            // Without a connection the renderer is still starting: it attaches to the new shell by itself, or ends
            // with the surface-lost code inside the restart window and is replaced for free.
            } else if lock(ctx.renderers).connections.contains_key(&slot) {
                to_close.push(slot);
            }
        }
        if !self.shell.begin(now, to_close.iter().copied()) {
            tracing::warn!("the shell restarted again right away, not acting on it");
            return false;
        }
        tracing::info!(closing = ?to_close, starting = ?to_start, "the shell restarted, replacing the renderers");
        for slot in to_close {
            send_to(ctx.renderers, slot, &ToRenderer::Close);
        }
        for slot in to_start {
            self.budgets.remove(&slot);
            self.stopped.remove(&slot);
            if let Err(error) = ctx.supervisor.spawn(slot) {
                tracing::warn!(%error, slot, "cannot start a renderer after the shell restarted");
            }
        }
        true
    }

    /// The monitor `id` is gone: its renderer is closed and only it. The slot stays reserved until that renderer
    /// has ended, so a monitor plugged in right now cannot get a slot that is still in use.
    fn close(&mut self, id: &str, ctx: &Ctx<'_>) {
        let Some(slot) = self.slots.retire(id) else {
            return;
        };
        lock_assignments(&self.assignments).remove(&slot);
        self.rule_closed.forget(slot);
        // A renderer that has not said Hello yet has no connection: it is told to close when it does, because its
        // assignment is gone.
        let told = send_to(ctx.renderers, slot, &ToRenderer::Close);
        tracing::info!(slot, monitor = %id, told, "the monitor is gone, closing its renderer");
        if self.stopped.remove(&slot) && !ctx.supervisor.slots().contains(&slot) {
            // Nothing runs and nothing will restart, so no exit event will come to free the slot.
            self.slots.release(slot);
            self.budgets.remove(&slot);
        }
    }

    /// The monitor `id` moved, was resized or changed its DPI: its renderer keeps playing and moves its surface.
    fn move_to(&mut self, id: &str, rect: RectPx, ctx: &Ctx<'_>) {
        let Some(slot) = self.slots.slot_of(id) else {
            return;
        };
        let rect = [rect.left, rect.top, rect.right, rect.bottom];
        if let Some(assignment) = lock_assignments(&self.assignments).get_mut(&slot) {
            // A renderer started later, or restarted, uses the new rectangle.
            assignment.rect = rect;
        }
        let told = send_to(ctx.renderers, slot, &ToRenderer::SetRect(rect));
        tracing::info!(slot, monitor = %id, ?rect, told, "the monitor changed, moving its surface");
    }
}

/// Reads the settings of the C# install, read-only from its real folder (D-008), logs what the service will use and
/// returns what they lead to.
fn plan_wallpapers() -> Plan {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return Plan {
            job: None,
            status: Status::NoWallpaper {
                reason: "the OS reports no config folder, so the settings cannot be read".to_owned(),
            },
            app_visible: true,
            fullscreen: FullScreenAction::default(),
            pause: PauseKind::default(),
            gpu: GpuPreference::default(),
        };
    };
    let settings = LegacySettings::load(&setting_dir, &root);
    // Values only: no paths and no theme names (D-08, D-09). The legacy DisplayScreenType and ScreenDevice are not
    // read: every monitor plays the same video (D-01).
    tracing::info!(
        volume = settings.engine.volume,
        looping = settings.engine.looping,
        stretch = ?settings.engine.stretch,
        hardware_acceleration = settings.engine.hardware_acceleration,
        screen = ?settings.engine.screen,
        gpu_preference = ?settings.engine.gpu_preference,
        fullscreen = ?settings.backgroundog.fullscreen,
        pause = ?settings.backgroundog.pause,
        app_visible = settings.general.app_visible,
        "legacy settings"
    );
    let (job, status) = match plan_for(&settings.library.location, &settings.library.selected, &settings.engine) {
        Ok(job) => {
            let status = Status::Starting {
                title: job.title.clone(),
            };
            (Some(job), status)
        }
        Err(reason) => (None, Status::NoWallpaper { reason }),
    };
    Plan {
        job,
        status,
        app_visible: settings.general.app_visible,
        fullscreen: settings.backgroundog.fullscreen,
        pause: settings.backgroundog.pause,
        gpu: settings.engine.gpu_preference,
    }
}

/// Writes the GPU preference for this executable, which the renderers are spawned from (legacy
/// `GraphicPreference.EnsureHighPerformance`). A failure only costs the default GPU: it is logged and never stops the
/// service. The executable's path is not logged (T-02-40).
fn apply_gpu_preference(platform: &dyn sucrose_platform::Platform, preference: GpuPreference) {
    if !platform.capabilities().gpu_preference {
        tracing::debug!("this OS has no per-app GPU preference");
        return;
    }
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            tracing::warn!(%error, "cannot set the GPU preference, the renderers use the default GPU");
            return;
        }
    };
    match platform.set_gpu_preference(&exe, preference) {
        Ok(GpuPreferenceChange::Written) => tracing::info!(?preference, "GPU preference written for the renderers"),
        Ok(GpuPreferenceChange::Unchanged) => {
            tracing::info!(?preference, "GPU preference already set for the renderers");
        }
        Err(error) => {
            tracing::warn!(%error, ?preference, "cannot set the GPU preference, the renderers use the default GPU");
        }
    }
}

/// Runs the gate (D-13) on the selected theme and, when it passes, builds what every renderer plays. The error is
/// one English sentence for the log and the tooltip: the gate's own reason.
fn plan_for(location: &std::path::Path, selected: &str, engine: &EngineSettings) -> Result<Job, String> {
    let theme =
        check_selected(location, selected, &app_version()).map_err(|rejection| printable(&rejection.to_string()))?;
    // Debug formatting for the text a theme or the settings supply: control characters show as escapes.
    tracing::info!(theme = ?selected, kind = ?theme.info.kind, source = ?theme.info.source, "selected theme");
    job_for(&theme, engine)
}

/// What every renderer plays for a theme that passed the gate, with the playback settings of Engine.json.
fn job_for(theme: &SelectedTheme, engine: &EngineSettings) -> Result<Job, String> {
    let source = theme
        .source_path
        .to_str()
        .ok_or_else(|| "the video path is not valid UTF-8".to_owned())?
        .to_owned();
    Ok(Job {
        title: printable(&theme.info.title),
        source,
        settings: playback_settings(engine),
        screen: engine.screen,
    })
}

/// The player settings Engine.json leads to.
fn playback_settings(engine: &EngineSettings) -> PlaybackSettings {
    PlaybackSettings {
        volume: engine.volume,
        looping: engine.looping,
        stretch: engine.stretch,
        hardware_decoding: engine.hardware_acceleration,
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::theme::ThemeRejection;

    use super::*;
    use crate::supervisor::ENGINE_MISSING;

    fn hello(protocol_version: u16, pid: u32, slot: u8) -> ToService {
        ToService::Hello {
            protocol_version,
            pid,
            slot,
        }
    }

    #[test]
    fn accept_hello_takes_a_renderer_the_service_started() {
        let spawned = HashMap::from([(100, 0), (200, 7)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::new(), &hello(PROTOCOL_VERSION, 200, 7)),
            Ok(7)
        );
    }

    fn failed_report() -> ToService {
        ToService::Failed {
            reason: "mpv could not open the file".to_owned(),
            hint: "Check the file.".to_owned(),
        }
    }

    fn failed_reported() -> Status {
        Status::Failed {
            reason: "mpv could not open the file".to_owned(),
            hint: "Check the file.".to_owned(),
        }
    }

    /// The main thread's handling of a renderer that sent `Failed` and ended with `code`, the exit handled first: the
    /// exit takes its decision and removes the connection, then the report finds no link and no replacement.
    fn exit_then_failed(status: Status, budget: &mut RestartBudget, code: u8) -> Status {
        let class = classify_exit(Some(i32::from(code)));
        let decision = decide_exit(budget, &mut ShellRestart::default(), class, 0, Instant::now());
        let mut status = status_after_exit(&status, decision, class, TITLE, "Monitor 1".to_owned()).unwrap_or(status);
        let effect = handle_report(&status, None, &HashMap::new(), 0, 10, &failed_report(), TITLE)
            .expect("the last words of a renderer that ended with nobody to replace it count");
        assert!(!effect.first_frame);
        if let Some(next) = effect.status {
            status = next;
        }
        status
    }

    #[test]
    fn a_report_counts_only_while_its_renderer_holds_the_slot() {
        let none = HashMap::new();
        let counts = |link_pid, running: &HashMap<u32, u8>, message: &ToService| {
            handle_report(&starting(), link_pid, running, 0, 7, message, TITLE)
        };

        let playing = counts(Some(7), &none, &ToService::Playing).expect("the current renderer");
        assert!(playing.first_frame);
        assert_eq!(
            playing.status,
            Some(Status::Playing {
                title: "Cyber Samurai".to_owned()
            })
        );
        assert!(
            counts(Some(8), &none, &ToService::Playing).is_none(),
            "a replacement is connected"
        );
        assert!(
            counts(Some(8), &none, &failed_report()).is_none(),
            "a replacement is connected"
        );
    }

    #[test]
    fn a_late_playing_of_an_ended_renderer_is_dropped() {
        // The renderer played, died and was restarted: the status is Starting. Its Playing report, handled after the
        // Exited, finds no connection for its pid and must not claim the wallpaper (WR-15).
        assert_eq!(
            handle_report(&starting(), None, &HashMap::new(), 0, 7, &ToService::Playing, TITLE),
            None
        );
    }

    #[test]
    fn a_failed_of_a_replaced_renderer_is_dropped_whether_or_not_the_replacement_connected() {
        let running = HashMap::from([(20, 0)]);

        assert_eq!(
            handle_report(&starting(), None, &running, 0, 10, &failed_report(), TITLE),
            None,
            "pid 20 runs for the slot but has not connected yet"
        );
        assert_eq!(
            handle_report(&starting(), Some(20), &running, 0, 10, &failed_report(), TITLE),
            None,
            "pid 20 is connected"
        );
        assert!(
            handle_report(&starting(), None, &running, 1, 10, &failed_report(), TITLE).is_some(),
            "the replacement runs for another slot"
        );
    }

    #[test]
    fn an_engine_failure_whose_exit_is_handled_first_still_names_its_reason() {
        // Exit code 2: Decision::Leave leaves the status Starting, the late Failed sets the reason (WR-18).
        let mut budget = RestartBudget::default();

        assert_eq!(
            exit_then_failed(starting(), &mut budget, ENGINE_MISSING),
            failed_reported()
        );
    }

    #[test]
    fn a_failed_start_whose_exit_is_handled_first_keeps_its_reason_through_the_restart() {
        // Exit code 4: the restart is scheduled and the status is still Starting when the late Failed arrives (WR-18).
        let mut budget = RestartBudget::default();
        let status = exit_then_failed(starting(), &mut budget, START_FAILED);
        assert_eq!(status, failed_reported());

        // The restarted renderer fails the same way until the budget is spent: the reason leads the final status.
        let mut status = status;
        let mut spent = None;
        for _ in 0..10 {
            let class = ExitClass::StartFailed;
            let decision = decide_exit(&mut budget, &mut ShellRestart::default(), class, 0, Instant::now());
            if let Some(next) = status_after_exit(&status, decision, class, TITLE, "Monitor 1".to_owned()) {
                status = next;
            }
            if matches!(decision, Decision::GiveUp(_)) {
                spent = Some(status.clone());
                break;
            }
        }
        let spent = spent.expect("the budget runs out");
        assert!(
            matches!(&spent, Status::RendererStopped { reason, .. } if reason.starts_with("mpv could not open the file ("))
        );
    }

    #[test]
    fn a_late_failed_does_not_replace_the_verdict_of_the_spent_budget() {
        let mut budget = RestartBudget::default();
        let mut status = starting();
        for _ in 0..10 {
            let class = ExitClass::StartFailed;
            let decision = decide_exit(&mut budget, &mut ShellRestart::default(), class, 0, Instant::now());
            if let Some(next) = status_after_exit(&status, decision, class, TITLE, "Monitor 1".to_owned()) {
                status = next;
            }
            if matches!(decision, Decision::GiveUp(_)) {
                break;
            }
        }
        assert!(matches!(status, Status::RendererStopped { .. }));

        let effect = handle_report(&status, None, &HashMap::new(), 0, 10, &failed_report(), TITLE).expect("it counts");

        assert_eq!(
            effect.status, None,
            "the status keeps saying that the monitor gets no more renderers"
        );
        let live =
            handle_report(&status, Some(10), &HashMap::new(), 0, 10, &failed_report(), TITLE).expect("it counts");
        assert_eq!(
            live.status,
            Some(failed_reported()),
            "a connected renderer's failure still reports"
        );
    }

    #[test]
    fn a_replaced_renderer_that_played_leaves_no_playing_entry_for_its_replacement() {
        let mut playing = HashSet::from([0, 1]);

        assert!(
            !forget_replaced_playing(&mut playing, 0, false),
            "its connection was not the registered one: the entry is not its to remove"
        );
        assert!(playing.contains(&0));
        assert!(forget_replaced_playing(&mut playing, 0, true));
        assert_eq!(playing, HashSet::from([1]));
        assert!(
            !forget_replaced_playing(&mut playing, 0, true),
            "nothing left, no new election"
        );
    }

    #[test]
    fn an_exit_is_stale_only_when_a_replacement_runs_for_the_slot() {
        let running = HashMap::from([(20, 0), (30, 1)]);

        assert!(exit_is_stale(&running, 0, 10), "pid 20 replaced pid 10 on slot 0");
        assert!(
            !exit_is_stale(&running, 0, 20),
            "the renderer that runs ended: not stale"
        );
        assert!(!exit_is_stale(&running, 2, 40), "nothing runs for slot 2");
        assert!(!exit_is_stale(&HashMap::new(), 0, 10), "the ordinary exit");
    }

    #[test]
    fn accept_hello_refuses_a_slot_the_pid_was_not_started_for() {
        let spawned = HashMap::from([(100, 0), (200, 7)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::new(), &hello(PROTOCOL_VERSION, 200, 0)),
            Err(Refusal::WrongSlot {
                pid: 200,
                claimed: 0,
                started_for: 7
            })
        );
    }

    #[test]
    fn accept_hello_never_takes_over_a_slot_that_has_a_renderer() {
        let spawned = HashMap::from([(100, 0)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::from([0]), &hello(PROTOCOL_VERSION, 100, 0)),
            Err(Refusal::AlreadyConnected(0))
        );
    }

    #[test]
    fn accept_hello_refuses_a_pid_the_service_did_not_start() {
        let spawned = HashMap::from([(100, 0)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::new(), &hello(PROTOCOL_VERSION, 999, 0)),
            Err(Refusal::UnknownPid(999))
        );
    }

    #[test]
    fn accept_hello_refuses_another_protocol_version() {
        let spawned = HashMap::from([(100, 0)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::new(), &hello(PROTOCOL_VERSION + 1, 100, 0)),
            Err(Refusal::ProtocolVersion {
                got: PROTOCOL_VERSION + 1,
                expected: PROTOCOL_VERSION
            })
        );
    }

    #[test]
    fn accept_hello_refuses_with_an_empty_set() {
        assert_eq!(
            accept_hello(&HashMap::new(), &HashSet::new(), &hello(PROTOCOL_VERSION, 100, 0)),
            Err(Refusal::UnknownPid(100))
        );
    }

    fn monitor(id: &str, primary: bool) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            rect: RectPx::default(),
            work: RectPx::default(),
            dpi: 96,
            primary,
        }
    }

    #[test]
    fn master_starts_on_the_primary_monitor_once_it_plays() {
        let playing = [monitor("\\\\.\\DISPLAY2", false), monitor("\\\\.\\DISPLAY1", true)];

        assert_eq!(choose_master(None, &playing).as_deref(), Some("\\\\.\\DISPLAY1"));
    }

    #[test]
    fn the_master_moves_when_its_monitor_goes_away() {
        let playing = [monitor("\\\\.\\DISPLAY18", true), monitor("\\\\.\\DISPLAY2", false)];

        assert_eq!(
            choose_master(Some("\\\\.\\DISPLAY1"), &playing).as_deref(),
            Some("\\\\.\\DISPLAY18")
        );
    }

    #[test]
    fn the_master_moves_when_its_renderer_stops_playing() {
        let playing = [monitor("\\\\.\\DISPLAY3", false), monitor("\\\\.\\DISPLAY2", false)];

        assert_eq!(
            choose_master(Some("\\\\.\\DISPLAY1"), &playing).as_deref(),
            Some("\\\\.\\DISPLAY2")
        );
    }

    #[test]
    fn with_nobody_playing_there_is_no_master() {
        assert_eq!(choose_master(Some("\\\\.\\DISPLAY1"), &[]), None);
        assert_eq!(choose_master(None, &[]), None);
    }

    #[test]
    fn plugging_a_monitor_keeps_the_playing_master() {
        let playing = [monitor("\\\\.\\DISPLAY1", false), monitor("\\\\.\\DISPLAY18", true)];

        assert_eq!(
            choose_master(Some("\\\\.\\DISPLAY1"), &playing).as_deref(),
            Some("\\\\.\\DISPLAY1")
        );
    }

    #[test]
    fn a_sticky_master_survives_a_new_primary_through_the_fleet() {
        let mut fleet = fleet_of(
            &[("\\\\.\\DISPLAY1", true), ("\\\\.\\DISPLAY18", false)],
            &["\\\\.\\DISPLAY1", "\\\\.\\DISPLAY18"],
        );
        let master = fleet.slots.slot_of("\\\\.\\DISPLAY1");
        for monitor in &mut fleet.monitors {
            monitor.primary = monitor.id.ends_with("DISPLAY18");
        }

        assert_eq!(fleet.master_slot(master), master);
    }

    #[test]
    fn a_promoted_renderer_takes_the_audio_at_the_configured_volume() {
        let changes = role_changes(Some(0), Some(1), &[1], 40);

        assert_eq!(
            changes,
            vec![(1, ToRenderer::SetRole(Role::Master)), (1, ToRenderer::SetVolume(40))]
        );
    }

    #[test]
    fn a_demoted_master_that_still_has_a_monitor_goes_back_to_following() {
        let changes = role_changes(Some(0), Some(1), &[0, 1], 100);

        assert_eq!(
            changes,
            vec![
                (1, ToRenderer::SetRole(Role::Master)),
                (1, ToRenderer::SetVolume(100)),
                (0, ToRenderer::SetRole(Role::Follower)),
            ]
        );
    }

    #[test]
    fn an_unchanged_master_or_no_master_changes_nothing() {
        assert!(role_changes(Some(1), Some(1), &[0, 1], 100).is_empty());
        assert!(role_changes(Some(1), None, &[0], 100).is_empty());
    }

    #[test]
    fn only_the_masters_clock_reaches_the_other_renderers() {
        assert_eq!(clock_targets(Some(1), 1, [0, 1, 2]), vec![0, 2]);
        assert!(clock_targets(Some(1), 2, [0, 1, 2]).is_empty());
        assert!(clock_targets(None, 0, [0, 1]).is_empty());
    }

    const TITLE: Option<&str> = Some("Cyber Samurai");

    fn starting() -> Status {
        Status::Starting {
            title: "Cyber Samurai".to_owned(),
        }
    }

    #[test]
    fn a_starting_service_shows_the_app_name() {
        assert_eq!(starting().tooltip(), APP_LONG_NAME);
    }

    #[test]
    fn a_playing_report_names_the_theme_in_the_tooltip() {
        let playing = starting().after(&ToService::Playing, TITLE).expect("a status change");

        assert_eq!(playing.tooltip(), "Sucrose — playing Cyber Samurai");
    }

    #[test]
    fn a_playing_report_changes_nothing_when_no_renderer_was_expected() {
        let rejected = Status::NoWallpaper { reason: "x".to_owned() };

        assert_eq!(rejected.after(&ToService::Playing, TITLE), None);
        assert_eq!(starting().after(&hello(PROTOCOL_VERSION, 1, 0), TITLE), None);
    }

    #[test]
    fn a_rejected_theme_shows_the_gate_reason_in_the_tooltip() {
        let reason = ThemeRejection::NothingSelected.to_string();
        let status = Status::NoWallpaper { reason: reason.clone() };

        assert_eq!(status.tooltip(), format!("Sucrose — no wallpaper: {reason}"));
        assert!(status.is_failure());
    }

    #[test]
    fn a_failed_renderer_shows_its_reason_and_hint_in_the_tooltip() {
        let failed = starting()
            .after(
                &ToService::Failed {
                    reason: "libmpv could not be found.".to_owned(),
                    hint: "Reinstall Sucrose.".to_owned(),
                },
                TITLE,
            )
            .expect("a status change");

        assert_eq!(
            failed.tooltip(),
            "Sucrose — no wallpaper: libmpv could not be found. Reinstall Sucrose."
        );
        assert!(failed.is_failure());
    }

    #[test]
    fn a_spent_restart_budget_names_the_monitor_in_the_tooltip() {
        let stopped = Status::RendererStopped {
            monitor: "DISPLAY1".to_owned(),
            reason: Exhausted::Crashes.reason(),
        };

        assert_eq!(
            stopped.tooltip(),
            "Sucrose — no wallpaper on DISPLAY1: the renderer stopped after 3 crashes in 60 s"
        );
        assert!(stopped.is_failure());
    }

    #[test]
    fn a_restarting_renderer_shows_the_app_name_until_it_plays_again() {
        let playing = starting().after(&ToService::Playing, TITLE).expect("a status change");

        let restarting = playing.restarting(TITLE).expect("a status change");

        assert_eq!(restarting, starting());
        assert_eq!(restarting.restarting(TITLE), None);
    }

    fn failed() -> Status {
        Status::Failed {
            reason: "the surface could not be attached".to_owned(),
            hint: "Check the log for details.".to_owned(),
        }
    }

    #[test]
    fn a_failed_renderer_that_is_restarted_and_then_plays_leaves_no_failure_behind() {
        let restarting = failed().restarting(TITLE).expect("a failed renderer is started again");
        assert_eq!(restarting, starting());

        let playing = restarting.after(&ToService::Playing, TITLE).expect("a status change");

        assert_eq!(playing.tooltip(), "Sucrose — playing Cyber Samurai");
        assert!(!playing.is_failure());
    }

    #[test]
    fn a_failed_start_keeps_its_reason_while_the_renderer_is_started_again() {
        assert_eq!(
            failed().restarting_after(ExitClass::StartFailed, TITLE),
            None,
            "the failure stays in the tooltip"
        );
        assert_eq!(
            failed().restarting_after(ExitClass::Crashed(1), TITLE),
            Some(starting()),
            "a crash after the failure is a new attempt"
        );
        let playing = starting().after(&ToService::Playing, TITLE).expect("a status change");
        assert_eq!(
            playing.restarting_after(ExitClass::StartFailed, TITLE),
            Some(starting()),
            "nothing failed on screen yet"
        );
    }

    #[test]
    fn a_spent_budget_after_failed_starts_names_the_failure() {
        let reason = failed().stopped_reason(Exhausted::Crashes);

        assert_eq!(
            reason,
            "the surface could not be attached (the renderer stopped after 3 crashes in 60 s)"
        );
        assert_eq!(
            starting().stopped_reason(Exhausted::Crashes),
            Exhausted::Crashes.reason()
        );
    }

    #[test]
    fn a_renderer_that_plays_replaces_a_failure_of_another_monitor_or_a_stopped_one() {
        let stopped = Status::RendererStopped {
            monitor: "DISPLAY2".to_owned(),
            reason: Exhausted::Crashes.reason(),
        };

        for status in [failed(), stopped] {
            let playing = status.after(&ToService::Playing, TITLE).expect("a status change");
            assert_eq!(playing.tooltip(), "Sucrose — playing Cyber Samurai");
        }
        assert_eq!(
            failed().after(&ToService::Playing, None),
            None,
            "no theme, no wallpaper"
        );
    }

    #[test]
    fn control_characters_of_a_theme_or_renderer_text_are_made_visible() {
        assert_eq!(
            printable("a\nb\r\u{1b}[31mc\t"),
            "a\u{fffd}b\u{fffd}\u{fffd}[31mc\u{fffd}"
        );
        assert_eq!(printable("Cyber Samurai — 夜"), "Cyber Samurai — 夜");
        assert_eq!(limit("reason\nforged line"), "reason\u{fffd}forged line");
        let rejection = ThemeRejection::FolderMissing {
            folder: "x\ninjected".to_owned(),
        };
        assert!(!printable(&rejection.to_string()).contains('\n'));
    }

    #[test]
    fn a_long_reason_is_cut_to_127_utf16_units_in_the_tooltip() {
        let failed = starting()
            .after(
                &ToService::Failed {
                    reason: "r".repeat(300),
                    hint: "h".repeat(300),
                },
                TITLE,
            )
            .expect("a status change");

        assert_eq!(failed.tooltip().encode_utf16().count(), 127);
        // What a renderer sends is also limited before it reaches the log.
        let Status::Failed { reason, hint } = failed else {
            panic!("expected Failed");
        };
        assert_eq!(reason.chars().count(), MAX_REPORT_CHARS);
        assert_eq!(hint.chars().count(), MAX_REPORT_CHARS);
    }

    /// A link whose writer first announces it holds a message, then waits until `release` is dropped or fed, and records
    /// what it was asked to write.
    fn gated_link() -> Gated {
        let (holding_tx, holding_rx) = crossbeam_channel::unbounded();
        let (release_tx, release_rx) = crossbeam_channel::unbounded::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&written);
        let link = Link::start("test-writer".to_owned(), 0, move |message| {
            let _ = holding_tx.send(());
            // Blocks like a pipe write to a renderer that does not read; ends when the test lets go.
            let _ = release_rx.recv();
            record.lock().unwrap_or_else(PoisonError::into_inner).push(message);
            true
        })
        .expect("the writer thread starts");
        Gated {
            link,
            holding: holding_rx,
            release: release_tx,
            written,
        }
    }

    /// What `gated_link` returns.
    struct Gated {
        link: Link,
        /// Receives a unit each time the writer takes a message.
        holding: crossbeam_channel::Receiver<()>,
        /// Dropping it lets the writer through.
        release: Sender<()>,
        written: Arc<Mutex<Vec<ToRenderer>>>,
    }

    fn tick() -> ClockTick {
        ClockTick {
            position_s: 1.0,
            stamp_ns: 0,
            speed: 1.0,
            paused: false,
            loop_gen: 0,
        }
    }

    #[test]
    fn a_renderer_that_does_not_read_never_blocks_the_sender_and_loses_only_clock_readings() {
        let Gated {
            link,
            holding,
            release,
            written,
        } = gated_link();
        // The writer takes the first message and is stuck writing it.
        assert_eq!(link.offer(ToRenderer::Pause(true)), Offer::Queued);
        holding
            .recv_timeout(Duration::from_secs(5))
            .expect("the writer holds a message");

        // The queue fills with readings; the sender is never made to wait.
        for _ in 0..OUTGOING_CAPACITY {
            assert_eq!(link.offer(ToRenderer::Clock(tick())), Offer::Queued);
        }
        assert_eq!(
            link.offer(ToRenderer::Clock(tick())),
            Offer::Dropped,
            "a reading is skipped"
        );
        assert_eq!(
            link.offer(ToRenderer::Close),
            Offer::Failed,
            "a control message is never skipped silently"
        );

        // Letting the renderer read again delivers what was queued, in order.
        drop(release);
        let deadline = Instant::now() + Duration::from_secs(5);
        while written.lock().unwrap_or_else(PoisonError::into_inner).len() < 1 + OUTGOING_CAPACITY {
            assert!(Instant::now() < deadline, "the queued messages were not written");
            thread::sleep(Duration::from_millis(10));
        }
        let written = written.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(written.first(), Some(&ToRenderer::Pause(true)));
        assert!(
            written[1..]
                .iter()
                .all(|message| matches!(message, ToRenderer::Clock(_)))
        );
    }

    #[test]
    fn messages_are_written_in_the_order_they_were_offered() {
        let written = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&written);
        let link = Link::start("test-writer".to_owned(), 0, move |message| {
            record.lock().unwrap_or_else(PoisonError::into_inner).push(message);
            true
        })
        .expect("the writer thread starts");

        let sent = [
            ToRenderer::SetVolume(10),
            ToRenderer::Pause(true),
            ToRenderer::Pause(false),
            ToRenderer::Close,
        ];
        for message in sent.clone() {
            assert_eq!(link.offer(message), Offer::Queued);
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        while written.lock().unwrap_or_else(PoisonError::into_inner).len() < sent.len() {
            assert!(Instant::now() < deadline, "the messages were not written");
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(*written.lock().unwrap_or_else(PoisonError::into_inner), sent);
    }

    #[test]
    fn a_broken_connection_fails_every_later_offer() {
        let link = Link::start("test-writer".to_owned(), 0, |_| false).expect("the writer thread starts");
        assert_eq!(link.offer(ToRenderer::Close), Offer::Queued);

        let deadline = Instant::now() + Duration::from_secs(5);
        while link.offer(ToRenderer::Pause(true)) != Offer::Failed {
            assert!(Instant::now() < deadline, "the broken link kept accepting messages");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn with_app_visible_false_no_tray_is_built() {
        let mut built = false;

        let tray = start_tray(false, || {
            built = true;
            Ok(())
        });

        assert!(tray.is_none());
        assert!(!built, "the tray must not be created at all");
    }

    #[test]
    fn with_app_visible_true_the_tray_is_built() {
        assert_eq!(start_tray(true, || Ok(7)), Some(7));
    }

    #[test]
    fn a_tray_that_cannot_be_created_leaves_the_service_without_one() {
        let tray: Option<()> = start_tray(true, || Err(PlatformError::Unsupported("tray icon")));

        assert!(tray.is_none());
    }

    #[test]
    fn accept_hello_refuses_a_first_message_that_is_not_a_hello() {
        let spawned = HashMap::from([(100, 0)]);

        assert_eq!(
            accept_hello(&spawned, &HashSet::new(), &ToService::Playing),
            Err(Refusal::NotHello)
        );
    }

    /// Records every call and says which slots are still alive after Close.
    struct Fake {
        slots: Vec<u8>,
        stubborn: Vec<u8>,
        calls: Vec<String>,
    }

    impl RendererControl for Fake {
        fn slots(&self) -> Vec<u8> {
            self.slots.clone()
        }

        fn close(&mut self, slot: u8) {
            self.calls.push(format!("close {slot}"));
        }

        fn wait_exit(&mut self, slot: u8, _timeout: Duration) -> bool {
            self.calls.push(format!("wait {slot}"));
            !self.stubborn.contains(&slot)
        }

        fn kill(&mut self, slot: u8) {
            self.calls.push(format!("kill {slot}"));
        }
    }

    #[test]
    fn shutdown_closes_all_then_waits_then_kills_only_the_survivors() {
        let mut fake = Fake {
            slots: vec![0, 1, 2],
            stubborn: vec![1],
            calls: Vec::new(),
        };

        shutdown(&mut fake, Duration::from_secs(2));

        assert_eq!(
            fake.calls,
            ["close 0", "close 1", "close 2", "wait 0", "wait 1", "wait 2", "kill 1"]
        );
    }

    #[test]
    fn shutdown_with_no_renderer_does_nothing() {
        let mut fake = Fake {
            slots: Vec::new(),
            stubborn: Vec::new(),
            calls: Vec::new(),
        };

        shutdown(&mut fake, Duration::from_secs(2));

        assert!(fake.calls.is_empty());
    }

    #[test]
    fn a_closed_slot_comes_back_only_after_its_old_renderer_ended() {
        let mut closed = RuleClosed::default();
        closed.close(0, false);
        closed.want_back(0);

        assert!(closed.ready().is_empty());
        assert!(closed.awaiting_exit(0));
        assert!(closed.note_exit(0));
        assert_eq!(closed.ready(), [0]);
        closed.forget(0);
        assert!(closed.ready().is_empty());
        assert!(!closed.is_closed(0));
    }

    #[test]
    fn an_exit_the_rule_did_not_ask_for_is_not_its_business() {
        let mut closed = RuleClosed::default();

        assert!(!closed.note_exit(3));
        closed.want_back(3);
        assert!(closed.ready().is_empty());
    }

    #[test]
    fn a_slot_for_a_monitor_that_appeared_while_closed_is_ready_when_wanted_back() {
        let mut closed = RuleClosed::default();
        closed.close(2, true);

        assert!(!closed.awaiting_exit(2));
        assert!(closed.ready().is_empty());
        closed.want_back(2);
        assert_eq!(closed.ready(), [2]);
    }

    /// The maintainer's real C# install, read-only: the selected theme must pass the gate.
    #[test]
    #[ignore = "reads the real C# install read-only"]
    fn real_install_selected_theme_passes_the_gate() {
        let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
            panic!("the OS reports no config folder");
        };
        let settings = LegacySettings::load(&setting_dir, &root);

        let selected = check_selected(&settings.library.location, &settings.library.selected, &app_version());

        let theme = selected.unwrap_or_else(|rejection| panic!("the real selected theme is refused: {rejection}"));
        assert!(theme.source_path.is_file());
    }

    fn fleet_with_a_job() -> Fleet {
        let job = Job {
            title: "test".to_owned(),
            source: "C:/wallpaper.mp4".to_owned(),
            settings: PlaybackSettings::default(),
            screen: ScreenArea::default(),
        };
        Fleet::new(Some(job), SharedAssignments::default())
    }

    /// A fleet whose monitors each have a slot; `playing` names the monitors whose renderer shows its first frame.
    fn fleet_of(monitors: &[(&str, bool)], playing: &[&str]) -> Fleet {
        let mut fleet = fleet_with_a_job();
        for (id, primary) in monitors {
            fleet.monitors.push(monitor(id, *primary));
            fleet.slots.assign(id).expect("a free slot");
        }
        for id in playing {
            fleet
                .playing
                .insert(fleet.slots.slot_of(id).expect("a slot for a playing monitor"));
        }
        fleet
    }

    #[test]
    fn a_renderer_that_does_not_play_yet_is_never_the_master() {
        let fleet = fleet_of(
            &[("\\\\.\\DISPLAY1", false), ("\\\\.\\DISPLAY18", true)],
            &["\\\\.\\DISPLAY1"],
        );

        assert_eq!(fleet.master_slot(None), fleet.slots.slot_of("\\\\.\\DISPLAY1"));
    }

    #[test]
    fn with_no_renderer_playing_there_is_no_master() {
        let fleet = fleet_of(&[("\\\\.\\DISPLAY1", true), ("\\\\.\\DISPLAY2", false)], &[]);

        assert_eq!(fleet.master_slot(None), None);
    }

    #[test]
    fn the_primary_renderer_is_master_once_it_plays() {
        let fleet = fleet_of(
            &[("\\\\.\\DISPLAY1", false), ("\\\\.\\DISPLAY18", true)],
            &["\\\\.\\DISPLAY1", "\\\\.\\DISPLAY18"],
        );

        assert_eq!(fleet.master_slot(None), fleet.slots.slot_of("\\\\.\\DISPLAY18"));
    }

    #[test]
    fn enable_fullscreen_resume_keeps_the_rule_and_its_timer_off() {
        let mut fleet = fleet_with_a_job();

        assert!(!fleet.enable_fullscreen(FullScreenAction::Resume, PauseKind::Light));
        assert!(fleet.fullscreen.is_none(), "a Resume rule left state behind");
    }

    #[test]
    fn enable_fullscreen_pause_turns_the_rule_and_its_timer_on() {
        let mut fleet = fleet_with_a_job();

        assert!(fleet.enable_fullscreen(FullScreenAction::Pause, PauseKind::Light));
        assert!(fleet.fullscreen.is_some());
    }

    #[test]
    fn enable_fullscreen_close_turns_the_rule_and_its_timer_on_for_heavy_pause() {
        let mut fleet = fleet_with_a_job();

        assert!(fleet.enable_fullscreen(FullScreenAction::Close, PauseKind::Heavy));
        assert!(fleet.fullscreen.is_some());
    }

    #[test]
    fn enable_fullscreen_without_a_job_stays_off() {
        let mut fleet = Fleet::new(None, SharedAssignments::default());

        assert!(!fleet.enable_fullscreen(FullScreenAction::Pause, PauseKind::Light));
        assert!(fleet.fullscreen.is_none());
    }

    /// A Platform that records every GPU preference write and answers with a configured capability and result.
    struct GpuFake {
        capable: bool,
        fail: bool,
        writes: std::sync::Mutex<Vec<(std::path::PathBuf, GpuPreference)>>,
    }

    impl GpuFake {
        fn new(capable: bool, fail: bool) -> Self {
            Self {
                capable,
                fail,
                writes: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn writes(&self) -> Vec<(std::path::PathBuf, GpuPreference)> {
            self.writes.lock().expect("the fake's lock").clone()
        }
    }

    impl sucrose_platform::Platform for GpuFake {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn capabilities(&self) -> sucrose_platform::Capabilities {
            sucrose_platform::Capabilities {
                gpu_preference: self.capable,
                ..sucrose_platform::Capabilities::default()
            }
        }

        fn set_gpu_preference(
            &self,
            exe: &std::path::Path,
            preference: GpuPreference,
        ) -> Result<GpuPreferenceChange, PlatformError> {
            self.writes
                .lock()
                .expect("the fake's lock")
                .push((exe.to_path_buf(), preference));
            if self.fail {
                Err(PlatformError::Os {
                    context: "set the GPU preference",
                    code: 5,
                })
            } else {
                Ok(GpuPreferenceChange::Written)
            }
        }
    }

    #[test]
    fn apply_gpu_preference_writes_nothing_when_the_os_has_no_gpu_preference() {
        let fake = GpuFake::new(false, false);

        apply_gpu_preference(&fake, GpuPreference::HighPerformance);

        assert!(
            fake.writes().is_empty(),
            "a non-capable OS must not write any preference"
        );
    }

    #[test]
    fn apply_gpu_preference_writes_once_for_this_executable_with_the_given_preference() {
        let fake = GpuFake::new(true, false);

        apply_gpu_preference(&fake, GpuPreference::PowerSaving);

        let expected_exe = std::env::current_exe().expect("the test executable path");
        assert_eq!(fake.writes(), vec![(expected_exe, GpuPreference::PowerSaving)]);
    }

    #[test]
    fn a_failed_gpu_preference_write_returns_normally_and_is_attempted_once() {
        let fake = GpuFake::new(true, true);

        apply_gpu_preference(&fake, GpuPreference::HighPerformance);

        assert_eq!(fake.writes().len(), 1, "a failed write must not be retried");
    }
}
