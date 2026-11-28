//! Service mode: owns the endpoint (single instance, D-023), starts a renderer and listens to it.

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use crossbeam_channel::Sender;
use sucrose_config::{
    migrate::migrate,
    model::{Engine, Settings, Warehouse},
};
use sucrose_core::{
    APP_LONG_NAME,
    compatible::DataGroup,
    paths::{legacy_data_dir, legacy_setting_dir, setting_dir},
    theme::{SelectedTheme, ThemeRejection, ThemeSource, WallpaperType, check_selected},
    version::app_version,
};
use sucrose_ipc::{
    Connection, IpcError, Listener, blob,
    portal::{
        EngineReportView, Launch, LaunchReply, MAX_PATH_TEXT, MaintainReport, MoveOutcome, MoveRefusal, PortalEvent,
        PortalPage, PortalRequest, RuleState, RuntimeState, SettingChange, UseOutcome, UseRefusal,
    },
    protocol::{
        Assignment, ClockTick, InputSettings, PROTOCOL_VERSION, PlaybackSettings, Role, ToRenderer, ToService,
        WebOptions,
    },
    service_endpoint,
};
use sucrose_platform::{
    EventLoop, Flow, GpuPreferenceChange, LoopEnd, LoopEvent, LoopProxy, LoopTimers, MonitorInfo, PlatformError,
    RectPx, Tray, TrayAction, TrayLabels,
};
// ScreenArea is read below to choose the surface rectangle.
use sucrose_core::settings::{
    EngineType, FullScreenAction, FullScreenScope, GpuPreference, PauseKind, ScreenArea, ThemeMode,
};
use sucrose_engine::EngineId;
use sucrose_engine_web::{args, profile::ProfileKind};

use crate::{
    data_hub::{DataModule, Delivery, Thumbnails, system_sampling_with_thumbnails},
    engines::{self, EngineReport, FallbackNote, Fallbacks},
    fullscreen::{ActionScope, FULLSCREEN_POLL_MS, FULLSCREEN_TIMER, Rule, RuleAction, covered_monitors},
    install::{InstallStage, InstallWorker, Prerequisite, SystemSteps},
    library_ops::{self, FinishStats, MaintainInput, MoveFailure, MovePlanDone, MoveStats, RealOps},
    lifecycle::{Closing, OPEN_LABEL, exit_label, should_exit, start_actions},
    modules::{
        MEMORY_TIMER, MemoryModule, ModuleAction, PortalEnd, end_portal, module_off, pids_by_slot, start_module,
        tray_action,
    },
    portal_link::{
        PortalAdmission, PortalLink, PortalRegistry, ask_service_to_open_portal, portal_args, runtime_state,
        spawn_pending,
    },
    settings_host::{SettingsHost, arguments_answer, describe},
    startup::{SystemStartup, set_startup, startup_info},
    supervisor::{
        Decision, Exhausted, ExitClass, Exited, RestartBudget, START_FAILED, ShellRestart, Supervisor, classify_exit,
        decide_exit,
    },
    topology::{
        Debounce, SlotMap, TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE, TimerCommand, TopologyAction, diff, primary,
        replace_all, web_profile_owner, with_area,
    },
};

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
    /// The user chose an item of the tray menu, or double-clicked the icon.
    Tray(TrayAction),
    /// A Portal said hello on a connection of its own. The main thread decides whether it is the one Portal.
    PortalConnected {
        /// The writer of that connection.
        link: PortalLink,
        /// The Portal protocol version it announced.
        version: u16,
        /// Where the main thread tells the reader thread whether this Portal was turned away because another is
        /// open (`true`), in which case the connection is read for the foreground grant instead of for requests.
        turned_away: Sender<bool>,
    },
    /// The foreground right was passed on (a second `sucrose` or a second Portal): the registered Portal comes to the
    /// front.
    PortalFocus {
        /// The page to show.
        page: Option<PortalPage>,
    },
    /// A request of the Portal on `connection`.
    Portal {
        /// Identifies the connection, as in [`PortalLink::connection`].
        connection: u64,
        /// What it asks.
        request: PortalRequest,
    },
    /// The Portal on `connection` disconnected.
    PortalGone {
        /// Identifies the connection.
        connection: u64,
    },
    /// A launcher (a second `sucrose`) asked the service to open the Portal. The answer goes back through `reply`.
    Launch {
        /// The page the Portal is to show.
        page: Option<PortalPage>,
        /// Where the main thread puts the answer for the launcher's connection.
        reply: Sender<LaunchReply>,
    },
    /// The library-ops worker finished a maintenance run.
    LibraryOps(LibraryDone),
    /// The library-ops worker finished a step of the library move.
    LibraryMove(MoveStep),
    /// The startup worker found out whether the C# app has a start entry.
    StartupLegacy(bool),
    /// The install worker reached a stage of the install the Portal asked for.
    Install(InstallStage),
}

/// What the library-ops worker reports: the counts for the Portal and the showcase names for the settings.
#[derive(Debug)]
struct LibraryDone {
    report: MaintainReport,
    /// Showcase folder names that are in the library now and must be recorded in `Warehouse.Showcase`.
    copied: Vec<String>,
}

/// The report of a maintenance that did nothing.
const NOTHING_MAINTAINED: MaintainReport = MaintainReport {
    showcase_copied: 0,
    corrupt_moved: 0,
    corrupt_skipped: 0,
};

/// What a MaintainLibrary request does given what else touches the library (WR-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaintenanceDecision {
    /// Start a library-ops run.
    Start,
    /// A run is going: its report answers this request too.
    Wait,
    /// A library move runs: nothing is maintained and the Portal is told so at once.
    AnswerNothing,
}

/// Logs one `monitor identified` line per monitor, so the ids settings remember a monitor by (D-08) can be read from
/// the Service log.
fn log_monitors_identified(monitors: &[MonitorInfo]) {
    for monitor in monitors {
        tracing::info!(
            device = %monitor.id,
            stable_id = %monitor.stable_id(),
            edid = ?monitor.edid,
            friendly = ?monitor.friendly,
            "monitor identified"
        );
    }
}

/// What a MaintainLibrary request does: maintenance and a library move never run together (WR-01); the move side is
/// `MoveEnv::start`'s Busy refusal. `running` is a library-ops run that is going, `moving` a library move.
fn maintenance_decision(running: bool, moving: bool) -> MaintenanceDecision {
    if running {
        MaintenanceDecision::Wait
    } else if moving {
        MaintenanceDecision::AnswerNothing
    } else {
        MaintenanceDecision::Start
    }
}

/// What the main thread does when a Portal connection ends (WR-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PortalGoneEffect {
    /// Re-check the exit rule: the service may have been kept alive only by this Portal.
    check_exit: bool,
    /// Tell the closing sequence its Portal is gone. Only the registered Portal is the one it waits for.
    tell_closing: bool,
}

/// What a Portal connection that ended asks of the main thread. `was_registered` is whether it was the registered
/// one: every ended connection re-checks the exit rule, but only the registered Portal counts for the closing
/// sequence.
fn portal_gone_effect(was_registered: bool) -> PortalGoneEffect {
    PortalGoneEffect {
        check_exit: true,
        tell_closing: was_registered,
    }
}

/// The OS recycle bin or trash for the library-ops worker.
struct OsBin;

impl library_ops::Trash for OsBin {
    fn move_to_trash(&self, path: &std::path::Path) -> Result<(), String> {
        sucrose_platform::move_to_trash(path).map_err(|error| error.to_string())
    }
}

/// Starts the `library-ops` thread for one maintenance run (T-03-50): copying and moving never run on the main thread.
/// The result comes back as `Event::LibraryOps`, also when the worker panics. `false` when the thread cannot start.
fn start_library_ops(input: MaintainInput, notifier: &Notifier) -> bool {
    let notifier = notifier.clone();
    let started = thread::Builder::new().name("library-ops".to_owned()).spawn(move || {
        let done = std::panic::catch_unwind(|| library_ops::maintain(&input, &OsBin)).map_or_else(
            |_| {
                tracing::error!("the library maintenance panicked");
                LibraryDone {
                    report: NOTHING_MAINTAINED,
                    copied: Vec::new(),
                }
            },
            |(report, copied)| LibraryDone { report, copied },
        );
        notifier.send(Event::LibraryOps(done));
    });
    match started {
        Ok(_) => true,
        Err(error) => {
            tracing::warn!(%error, "cannot start the library-ops thread");
            false
        }
    }
}

/// Starts the `startup-legacy` thread that looks for a start entry of the C# app (it may start a short-lived process, so
/// it never runs on the main thread). The answer comes back as `Event::StartupLegacy`. `false` when the thread cannot
/// start.
fn start_legacy_check(notifier: &Notifier) -> bool {
    let notifier = notifier.clone();
    let started = thread::Builder::new().name("startup-legacy".to_owned()).spawn(move || {
        let legacy = std::panic::catch_unwind(sucrose_platform::legacy_entry).unwrap_or_else(|_| {
            tracing::error!("the check for the C# start entry panicked");
            false
        });
        notifier.send(Event::StartupLegacy(legacy));
    });
    match started {
        Ok(_) => true,
        Err(error) => {
            tracing::warn!(%error, "cannot start the startup-legacy thread");
            false
        }
    }
}

/// The loop timer of a library move (1 and 2 belong to the topology debounce, 3 to the fullscreen rule, 4 to the memory
/// report, 5 to Close Applications): the renderers get this long to leave before the move ends them.
const MOVE_TIMER: u32 = 6;

/// How long the renderers get to close for a library move before they are ended.
const MOVE_CLOSE_GRACE_MS: u32 = 4000;

/// The loop timer that ends the wait for a renderer's answer to Open DevTools (7; 1 to 6 are taken above).
const DEVTOOLS_TIMER: u32 = 7;

/// How long the Portal's Open DevTools waits for the renderer to say that the developer tools are open; later it is
/// answered as failed.
const DEVTOOLS_WAIT_MS: u32 = 5000;

/// What the Portal is told when the service has no library folder to work with.
const LIBRARY_UNAVAILABLE: &str = "the library folder is not available";

/// A library move that is going on (LIB-09, D-07). The renderers close first so no file is held open, then the content
/// moves on the library-ops worker, the setting is written on the main thread, the old copy is removed and the
/// wallpaper starts again from wherever the library is now.
#[derive(Debug)]
struct LibraryMove {
    source: PathBuf,
    destination: PathBuf,
    /// Whether the content moves along (`Library.Move`); otherwise only the location changes.
    carry: bool,
    stage: MoveStage,
    /// The renderer slots this move closed: the ones it brings back.
    closed: Vec<u8>,
}

/// Where a [`LibraryMove`] waits.
#[derive(Debug)]
enum MoveStage {
    /// For the renderers to end.
    Closing,
    /// For the worker to move the content.
    Copying,
    /// For the worker to remove the old copy, after the setting was written.
    Finishing,
    /// For the worker to put the content back after the setting could not be written.
    RollingBack { reason: String },
}

/// What the library-ops worker reports of a move step.
#[derive(Debug)]
enum MoveStep {
    /// The content is in the destination and verified, or why not (and everything is back).
    Copied(Result<MovePlanDone, MoveFailure>),
    /// The old copy is removed.
    Finished { moved: MoveStats, finish: FinishStats },
    /// The content is back in the old folder, or not entirely.
    RolledBack { restored: bool },
}

/// Runs `work` on a `library-ops` thread and reports its step. `on_panic` is reported when it panics. `false` when the
/// thread cannot start.
fn spawn_move_step(notifier: &Notifier, on_panic: MoveStep, work: impl FnOnce() -> MoveStep + Send + 'static) -> bool {
    let notifier = notifier.clone();
    let started = thread::Builder::new().name("library-ops".to_owned()).spawn(move || {
        let step = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|_| {
            tracing::error!("a step of the library move panicked");
            on_panic
        });
        notifier.send(Event::LibraryMove(step));
    });
    match started {
        Ok(_) => true,
        Err(error) => {
            tracing::warn!(%error, "cannot start the library-ops thread");
            false
        }
    }
}

/// The folder a Portal's text names: an absolute path of a sane length without `..` parts. Anything else cannot be a
/// folder the service moves into (T-03-53).
fn parse_destination(text: &str) -> Result<PathBuf, MoveOutcome> {
    let path = Path::new(text);
    let plain = text.len() <= MAX_PATH_TEXT
        && !text.contains('\0')
        && path.is_absolute()
        && !path.components().any(|part| matches!(part, Component::ParentDir));
    if plain {
        Ok(path.to_path_buf())
    } else {
        Err(MoveOutcome::Refused(MoveRefusal::Missing))
    }
}

/// Whether the service acts on a Portal request once it is closing (WR-02): only the startup read, which changes nothing.
/// Nothing that starts, moves or changes anything is acted on after the renderers were told to close.
fn acts_while_quitting(request: &PortalRequest) -> bool {
    matches!(request, PortalRequest::RefreshStartup)
}

/// The answer a Portal gets to a request that is not acted on while the service closes: a move was asked for and is
/// refused Busy, so the Portal does not wait for a result that never comes; the other requests have no answer.
fn quitting_answer(request: &PortalRequest) -> Option<PortalEvent> {
    match request {
        PortalRequest::MoveLibrary { .. } => Some(PortalEvent::MoveResult(MoveOutcome::Refused(MoveRefusal::Busy))),
        _ => None,
    }
}

/// Whether the service may exit: what it waits for is over and no library move runs. Leaving in the middle of a move
/// could leave the library split between two folders.
fn may_exit(done: bool, moving: bool) -> bool {
    done && !moving
}

/// The exit check of the main loop: [`may_exit`], saying once that the exit waits for a library move.
fn settle_exit(done: bool, moving: bool, held: &mut bool) -> bool {
    if may_exit(done, moving) {
        return true;
    }
    if done && !*held {
        *held = true;
        tracing::info!("the service waits for the library move before it exits");
    }
    false
}

/// Whether the end of a library move starts the wallpaper again: not while the service closes, the renderers are gone
/// for good and nothing may start.
fn resume_after_move(quitting: bool) -> bool {
    !quitting
}

/// Everything a step of a library move touches on the main thread.
struct MoveEnv<'a> {
    host: &'a mut SettingsHost,
    fleet: &'a mut Fleet,
    legacy_root: Option<&'a Path>,
    ctx: &'a Ctx<'a>,
    notifier: &'a Notifier,
    timers: &'a LoopTimers,
    portals: &'a mut PortalRegistry,
    /// The service is closing: the end of the move starts no renderer.
    quitting: bool,
}

/// What a step of a library move changed for the main loop.
#[derive(Default)]
struct MoveProgress {
    /// The move is over, whatever its outcome.
    ended: bool,
    /// The tray status after the wallpaper started again.
    status: Option<Status>,
}

impl MoveEnv<'_> {
    /// Handles the Portal's MoveLibrary: a refusal is answered at once, otherwise the move starts by closing the
    /// renderers and its answer comes when it ends. `maintenance_running` is a library-ops run that is going.
    fn begin(&mut self, destination: &str, maintenance_running: bool) {
        if let Err(outcome) = self.start(destination, maintenance_running) {
            tracing::info!(?outcome, "library move refused");
            self.portals.send(PortalEvent::MoveResult(outcome));
        }
    }

    fn start(&mut self, destination: &str, maintenance_running: bool) -> Result<(), MoveOutcome> {
        if self.fleet.library_move.is_some() || maintenance_running {
            return Err(MoveOutcome::Refused(MoveRefusal::Busy));
        }
        let Some(root) = self.legacy_root else {
            return Err(MoveOutcome::Failed {
                reason: LIBRARY_UNAVAILABLE.to_owned(),
            });
        };
        let destination = parse_destination(destination)?;
        let (source, carry) = {
            let settings = self.host.settings();
            (settings.library_location(root), settings.library.move_library)
        };
        let checked = if carry {
            library_ops::check_destination(&source, &destination)
        } else {
            library_ops::check_location(&source, &destination)
        };
        checked.map_err(MoveOutcome::Refused)?;
        let closed = close_for_move(self.fleet, self.ctx);
        if !closed.is_empty() {
            self.timers.set(MOVE_TIMER, MOVE_CLOSE_GRACE_MS);
        }
        tracing::info!(carry, renderers = closed.len(), "library move started");
        self.fleet.library_move = Some(LibraryMove {
            source,
            destination,
            carry,
            stage: MoveStage::Closing,
            closed,
        });
        Ok(())
    }

    /// Goes on once every renderer has ended: the content moves, or with `Move` off only the location is written.
    fn drive(&mut self) -> MoveProgress {
        let closing = self
            .fleet
            .library_move
            .as_ref()
            .is_some_and(|flow| matches!(flow.stage, MoveStage::Closing));
        if !closing || self.fleet.rule_closed.any_awaiting() {
            return MoveProgress::default();
        }
        self.timers.kill(MOVE_TIMER);
        let Some(flow) = self.fleet.library_move.as_mut() else {
            return MoveProgress::default();
        };
        let (source, destination, carry) = (flow.source.clone(), flow.destination.clone(), flow.carry);
        if !carry {
            let location = destination.to_string_lossy().into_owned();
            let outcome = match self.host.update(|settings| settings.library.location = location) {
                Ok(()) => MoveOutcome::Changed,
                Err(reason) => MoveOutcome::Failed { reason },
            };
            return self.end(outcome);
        }
        flow.stage = MoveStage::Copying;
        let started = spawn_move_step(
            self.notifier,
            MoveStep::Copied(Err(MoveFailure {
                reason: "the move stopped unexpectedly".to_owned(),
            })),
            move || MoveStep::Copied(library_ops::move_library(&source, &destination, &RealOps)),
        );
        if started {
            MoveProgress::default()
        } else {
            self.end(MoveOutcome::Failed {
                reason: "the move could not be started".to_owned(),
            })
        }
    }

    /// Handles what the worker reported.
    fn step(&mut self, step: MoveStep) -> MoveProgress {
        match step {
            MoveStep::Copied(Ok(done)) => self.commit(done),
            MoveStep::Copied(Err(failure)) => {
                tracing::warn!(reason = %failure.reason, "the library move failed and was rolled back");
                self.end(MoveOutcome::Failed { reason: failure.reason })
            }
            MoveStep::Finished { moved, finish } => {
                tracing::info!(
                    entries = moved.entries,
                    renamed = moved.renamed,
                    files = moved.files,
                    bytes = moved.bytes,
                    left_behind = finish.failed,
                    "library moved"
                );
                self.end(MoveOutcome::Moved)
            }
            MoveStep::RolledBack { restored } => {
                if !restored {
                    tracing::warn!("the library move was rolled back only in part");
                }
                let reason = match self.fleet.library_move.as_ref().map(|flow| &flow.stage) {
                    Some(MoveStage::RollingBack { reason }) => reason.clone(),
                    _ => "the move failed".to_owned(),
                };
                self.end(MoveOutcome::Failed { reason })
            }
        }
    }

    /// The content is in the destination and verified: the location is written now, and only after that the old copy
    /// goes. When the write fails everything is put back.
    fn commit(&mut self, done: MovePlanDone) -> MoveProgress {
        let Some(flow) = self.fleet.library_move.as_mut() else {
            // Nobody waits for this move any more; put the content back rather than leave it where nothing looks.
            tracing::error!("a library move finished without a move in progress, putting it back");
            library_ops::rollback_move(done, &RealOps);
            return MoveProgress::default();
        };
        let moved = done.stats();
        let location = flow.destination.to_string_lossy().into_owned();
        let shared = Arc::new(Mutex::new(Some(done)));
        match self.host.update(|settings| settings.library.location = location) {
            Ok(()) => {
                flow.stage = MoveStage::Finishing;
                let worker = Arc::clone(&shared);
                let started = spawn_move_step(
                    self.notifier,
                    MoveStep::Finished {
                        moved,
                        finish: FinishStats::default(),
                    },
                    move || {
                        let finish = take_done(&worker).map(|done| library_ops::finish_move(done, &RealOps));
                        MoveStep::Finished {
                            moved,
                            finish: finish.unwrap_or_default(),
                        }
                    },
                );
                if started {
                    MoveProgress::default()
                } else {
                    // The new location is saved and the content is there: the old copy stays, nothing is lost.
                    tracing::warn!("the old library folder could not be cleaned up");
                    self.end(MoveOutcome::Moved)
                }
            }
            Err(reason) => {
                tracing::warn!(%reason, "the new location cannot be saved, putting the library back");
                flow.stage = MoveStage::RollingBack { reason: reason.clone() };
                let worker = Arc::clone(&shared);
                let started = spawn_move_step(self.notifier, MoveStep::RolledBack { restored: false }, move || {
                    let restored = take_done(&worker).is_none_or(|done| library_ops::rollback_move(done, &RealOps));
                    MoveStep::RolledBack { restored }
                });
                if started {
                    MoveProgress::default()
                } else if let Some(done) = take_done(&shared) {
                    // The content must not stay where the settings do not look: put it back here.
                    let restored = library_ops::rollback_move(done, &RealOps);
                    self.step(MoveStep::RolledBack { restored })
                } else {
                    self.end(MoveOutcome::Failed { reason })
                }
            }
        }
    }

    /// Ends the move: the renderers it closed start again from the library as the settings say now, `library_moving`
    /// clears and the Portal gets its answer.
    fn end(&mut self, outcome: MoveOutcome) -> MoveProgress {
        self.timers.kill(MOVE_TIMER);
        let closed = self
            .fleet
            .library_move
            .take()
            .map(|flow| flow.closed)
            .unwrap_or_default();
        let status = if resume_after_move(self.quitting) {
            resume_wallpaper(self.host, self.fleet, self.legacy_root, &closed, self.ctx)
        } else {
            // The renderers are being closed for good: nothing is kept for later and nothing starts.
            self.fleet.pending_snapshot = None;
            None
        };
        self.portals.send(PortalEvent::MoveResult(outcome));
        MoveProgress { ended: true, status }
    }
}

/// The content of `cell`, once.
fn take_done(cell: &Mutex<Option<MovePlanDone>>) -> Option<MovePlanDone> {
    cell.lock().unwrap_or_else(PoisonError::into_inner).take()
}

/// Closes every renderer that runs, so no file of the library is held open, and returns their slots. A renderer the
/// fullscreen rule closed already is not touched: the rule brings it back, not the move.
fn close_for_move(fleet: &mut Fleet, ctx: &Ctx<'_>) -> Vec<u8> {
    let mut closed = Vec::new();
    for slot in ctx.supervisor.slots() {
        if fleet.rule_closed.is_closed(slot) {
            continue;
        }
        fleet.rule_closed.close(slot, false);
        // A renderer that has not connected yet is told at its Hello (catch_up_renderer).
        send_to(ctx.renderers, slot, &ToRenderer::Close);
        closed.push(slot);
    }
    closed
}

/// Starts the wallpaper again after a library move, from the library the settings name now: the job is made again for
/// the selected theme and the renderers the move closed come back. A theme that is not in that library any more leaves
/// no wallpaper, which the status says. Returns the new tray status when something was closed.
fn resume_wallpaper(
    host: &SettingsHost,
    fleet: &mut Fleet,
    legacy_root: Option<&Path>,
    closed: &[u8],
    ctx: &Ctx<'_>,
) -> Option<Status> {
    let root = legacy_root?;
    let gate = selected_job(host.settings(), root, &fleet.engines);
    for slot in closed {
        fleet.rule_closed.want_back(*slot);
    }
    let (title, no_wallpaper) = match gate {
        Ok(job) => {
            let title = (job.title.clone(), job.fallback.clone());
            if fleet.job.is_some() || !closed.is_empty() {
                fleet.adopt_job(job);
            }
            (Some(title), None)
        }
        Err(rejection) => {
            fleet.job = None;
            fleet.fallbacks.clear();
            (None, Some(rejection.to_string()))
        }
    };
    // The job names the new library now: the monitor change of the move is applied and the closed slots come back (with
    // no job they are forgotten instead of started).
    let applied = fleet.finish_move(ctx);
    match (title, no_wallpaper) {
        (Some((title, fallback)), _) if !closed.is_empty() || applied.started > 0 => {
            Some(Status::Starting { title, fallback })
        }
        (None, Some(reason)) if !closed.is_empty() => Some(Status::NoWallpaper { reason }),
        _ => None,
    }
}

/// Folds what a library move step did into the tray status and the exit check.
fn settle_move(progress: MoveProgress, status: &mut Status, tray: Option<&Tray>, check_exit: &mut bool) {
    if let Some(next) = progress.status {
        *status = next;
        if status.is_failure() {
            tracing::warn!(status = %status.text(), "no wallpaper, the service waits in the tray");
        } else {
            tracing::info!(status = %status.text(), "wallpaper status");
        }
        if let Some(tray) = tray {
            tray.set_tooltip(&status.tooltip());
        }
    }
    if progress.ended {
        // The service may have been kept alive only by the move.
        *check_exit = true;
    }
}

/// The job for the theme the settings select, in the library the settings name.
fn selected_job(settings: &Settings, legacy_root: &Path, engines: &EngineReport) -> Result<Job, ThemeRejection> {
    let folder = &settings.library.selected;
    check_selected(
        &settings.library_location(legacy_root),
        folder,
        &app_version(),
        PLAYABLE,
    )
    .and_then(|theme| {
        job_for(
            &theme,
            &settings.engine,
            settings.backgroundog.pause,
            settings.portal.theme_mode,
            &settings.general.user_agent,
            engines,
        )
        .map_err(|reason| ThemeRejection::SourceInvalid {
            folder: folder.clone(),
            reason,
        })
    })
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
pub(crate) const OUTGOING_CAPACITY: usize = 64;

/// What became of a message offered to a [`Link`] or a `PortalLink`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Offer {
    /// Queued; the writer thread sends it.
    Queued,
    /// A reading dropped because the peer is behind: the next reading replaces it anyway.
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

    /// Queues `message` without ever waiting. A clock reading or data snapshot that finds the queue full is dropped; any other message
    /// that does is a failure, because losing it would leave the renderer in a wrong state.
    fn offer(&self, message: ToRenderer) -> Offer {
        // A clock reading or a data snapshot that finds the queue full is replaced by the next one anyway.
        let skippable = matches!(message, ToRenderer::Clock(_) | ToRenderer::Data(_));
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

/// How long a renderer gets to leave after Close before it is killed.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// The tray icon the service shows (the C# Launcher's image, D-14).
const TRAY_ICON: &[u8] = include_bytes!("../assets/TrayIcon.ico");

/// Why no wallpaper plays while the OS reports no monitor.
const NO_MONITOR: &str = "the OS reports no monitor";

/// The hint for a failure that has no repair step of its own.
const CHECK_THE_LOG: &str = "Check the log for details.";

/// The longest reason or hint taken from a renderer, so a misbehaving renderer cannot flood the log (T-02-19).
const MAX_REPORT_CHARS: usize = 300;

/// The theme that plays now, as the status transitions need it: its title and the fallback it plays with.
#[derive(Debug, Clone, Copy)]
struct Current<'a> {
    title: &'a str,
    fallback: Option<&'a FallbackNote>,
}

/// What the service tells the user about the wallpaper: one value, shown in the tray tooltip and written to the log.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    /// A renderer was started for this theme and has not reported playback yet. `fallback` is set when another engine
    /// than the configured one plays it (D-11).
    Starting {
        title: String,
        fallback: Option<FallbackNote>,
    },
    /// The wallpaper plays, with the fallback engine when `fallback` is set.
    Playing {
        title: String,
        fallback: Option<FallbackNote>,
    },
    /// No renderer was started: the theme is rejected (D-13) or no monitor can be used.
    NoWallpaper { reason: String },
    /// The renderer could not start its wallpaper, for example because libmpv is missing (D-20).
    Failed { reason: String, hint: String },
    /// The restart budget of a monitor is spent (D-22): its renderer is not started again until Sucrose restarts.
    RendererStopped { monitor: String, reason: String },
}

impl Status {
    /// The sentence for the log. Every free-text field passes through [`printable`] here, in the one place that builds
    /// the sentence, so no constructor can forget it (WR-07): a theme name from `Library.Selected` cannot forge a log
    /// line or garble the tooltip.
    fn text(&self) -> String {
        match self {
            Self::Starting { .. } => APP_LONG_NAME.to_owned(),
            Self::Playing { title, .. } => format!("Sucrose — playing {}", printable(title)),
            Self::NoWallpaper { reason } => format!("Sucrose — no wallpaper: {}", printable(reason)),
            Self::Failed { reason, hint } => {
                format!("Sucrose — no wallpaper: {} {}", printable(reason), printable(hint))
            }
            Self::RendererStopped { monitor, reason } => {
                format!(
                    "Sucrose — no wallpaper on {}: {}",
                    printable(monitor),
                    printable(reason)
                )
            }
        }
    }

    /// The tray tooltip: the same sentence, cut to what the notification area keeps. While another engine than the
    /// configured one plays, it says so instead (D-11); the theme title is left out, so nothing important is cut.
    fn tooltip(&self) -> String {
        match self {
            Self::Starting {
                fallback: Some(note), ..
            }
            | Self::Playing {
                fallback: Some(note), ..
            } => sucrose_platform::truncate_tooltip(&format!(
                "Playing with {} ({} isn't available)",
                engines::label(note.playing),
                engines::label(note.configured)
            )),
            _ => sucrose_platform::truncate_tooltip(&self.text()),
        }
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
    fn restarting(&self, current: Option<Current<'_>>) -> Option<Self> {
        match self {
            Self::Playing { title, fallback } => Some(Self::Starting {
                title: title.clone(),
                fallback: fallback.clone(),
            }),
            Self::Failed { .. } => current.map(|current| Self::Starting {
                title: current.title.to_owned(),
                fallback: current.fallback.cloned(),
            }),
            _ => None,
        }
    }

    /// The status while the renderer that ended with `class` is started again. A failed start keeps its failure: the
    /// reason the renderer reported is what the user needs, and the restart may fail the same way (WR-17).
    fn restarting_after(&self, class: ExitClass, current: Option<Current<'_>>) -> Option<Self> {
        if class == ExitClass::StartFailed && matches!(self, Self::Failed { .. }) {
            return None;
        }
        self.restarting(current)
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
    fn after(&self, message: &ToService, current: Option<Current<'_>>) -> Option<Self> {
        match message {
            ToService::Playing => match (self, current) {
                (Self::Starting { title, fallback }, _) => Some(Self::Playing {
                    title: title.clone(),
                    fallback: fallback.clone(),
                }),
                (Self::Failed { .. } | Self::RendererStopped { .. }, Some(current)) => Some(Self::Playing {
                    title: current.title.to_owned(),
                    fallback: current.fallback.cloned(),
                }),
                _ => None,
            },
            ToService::Failed { reason, hint } => Some(Self::Failed {
                reason: limit(reason),
                hint: limit(hint),
            }),
            ToService::Hello { .. }
            | ToService::Clock(_)
            | ToService::PortalHello { .. }
            | ToService::Launch(_)
            | ToService::DataWanted(_)
            | ToService::DevToolsOpened(_) => None,
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

/// Starts the fullscreen rule through the module gate (D-008): with FullScreenPerformance Resume, or with no wallpaper
/// to act on, nothing is built and the module logs that it is off. `true` when the rule is on and its timer has to run.
fn start_fullscreen(fleet: &mut Fleet, action: FullScreenAction, scope: ActionScope, pause: PauseKind) -> bool {
    let wanted = action != FullScreenAction::Resume && fleet.job.is_some();
    start_module("fullscreen", wanted, || {
        Ok(fleet.enable_fullscreen(action, scope, pause))
    })
    .unwrap_or(false)
}

/// The loop timer of Close Applications (1 and 2 belong to the topology debounce, 3 to the fullscreen rule, 4 to the
/// memory report): the Portal gets this long to close before it is ended.
const CLOSE_TIMER: u32 = 5;

/// How long the Portal has to close after it was told to (TRAY-05, about 2 s).
const CLOSE_GRACE_MS: u32 = 2000;

/// What the tray tooltip says when Open Sucrose cannot start the Portal (UI-SPEC E14).
const OPEN_FAILED_TOOLTIP: &str = "Sucrose couldn't open its window";

/// Builds the tray icon with the tooltip of `status` and the exit item labelled for `app_exit`. What the user
/// chooses reaches the main thread as `Event::Tray`. Without the icon Sucrose can only be stopped from Task Manager,
/// so a failure says so.
fn create_tray(status: &Status, app_exit: bool, notifier: &Notifier) -> Result<Tray, PlatformError> {
    let labels = TrayLabels {
        open: OPEN_LABEL.to_owned(),
        exit: exit_label(app_exit).to_owned(),
    };
    let notifier = notifier.clone();
    let tray = Tray::new(TRAY_ICON, &status.tooltip(), &labels, move |action| {
        notifier.send(Event::Tray(action));
    })
    .inspect_err(|_| {
        tracing::error!("without the tray icon Sucrose can only be stopped from Task Manager");
    })?;
    tracing::info!(open = %labels.open, exit = %labels.exit, "tray icon ready");
    Ok(tray)
}

/// The tray as a module (D-008): not built at all when `enabled` is false.
fn build_tray(enabled: bool, status: &Status, app_exit: bool, notifier: &Notifier) -> Option<Tray> {
    start_module("tray", enabled, || create_tray(status, app_exit, notifier))
}

/// The Portal executable that sits next to this one.
fn portal_path() -> std::io::Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    let folder = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("the Sucrose executable has no folder"))?;
    Ok(folder.join(format!("sucrose-portal{}", std::env::consts::EXE_SUFFIX)))
}

/// Starts the Portal that sits next to this executable (T-03-34): the fixed name, the argument vector chosen from a
/// page enum here, never text from a request, and never a PATH lookup.
fn open_portal(page: Option<PortalPage>) -> std::io::Result<()> {
    let path = portal_path()?;
    let mut command = std::process::Command::new(&path);
    command.args(portal_args(page));
    sucrose_platform::spawn_detached(&mut command)?;
    tracing::info!(path = %path.display(), ?page, "started the Portal");
    Ok(())
}

/// Runs the service until Close Applications is chosen from the tray or nothing needs it any more (D-13).
/// `portal_wanted` is false for a service the Portal started itself (`--no-portal`, D-12): neither it nor a second `sucrose` of that kind asks for another Portal.
pub fn run(portal_wanted: bool) -> anyhow::Result<()> {
    let awareness = sucrose_platform::init_dpi_awareness();
    let endpoint = service_endpoint().context("cannot identify the current user")?;
    let listener = match Listener::bind(&endpoint.name, endpoint.owner.as_deref()) {
        Ok(listener) => listener,
        Err(IpcError::AlreadyRunning) => {
            if portal_wanted {
                // This is a second `sucrose` (D-12): the running service opens or raises the Portal for it.
                match ask_service_to_open_portal(&endpoint.name) {
                    Ok(reply) => tracing::info!(?reply, "Sucrose is already running"),
                    Err(error) => tracing::warn!(%error, "Sucrose is already running, but its answer was not read"),
                }
            } else {
                tracing::info!("Sucrose is already running; exiting");
            }
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
    // The settings host owns the file for the whole run: it is the only writer (D-11).
    // What can play, from files and the registry only: no engine is loaded here (D-028).
    let engines = engines::detect(&engines::exe_dir(), &*platform);
    let (plan, mut host) = plan_wallpapers(&engines);
    let assignments = SharedAssignments::default();
    let mut fleet = Fleet::new(plan.job, Arc::clone(&assignments), engines);
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
        log_monitors_identified(&monitors);
        let primary_id = primary(&monitors).map(|monitor| monitor.id.clone());
        fleet.web_owner = web_profile_owner(&monitors).map(|monitor| monitor.id.clone());
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
        tracing::info!(
            count = applied.started,
            monitors = ?ids,
            primary = ?primary_id,
            web_profile_owner = ?fleet.web_owner,
            "renderers started"
        );
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
    // General.json AppVisible (D-15): with it false no icon, no menu and none of their resources exist (D-008).
    let mut tray = build_tray(plan.app_visible, &status, host.settings().general.app_exit, &notifier);
    // Whether the Open item is enabled: it is greyed out while the Portal is open.
    let mut open_enabled = true;
    // Startup with the OS: the entry is always this executable's own, and the C# entry is looked up only when the
    // Portal asks (the General page opens), so the last answer is kept for the other replies.
    let startup_exe = std::env::current_exe().unwrap_or_default();
    let mut legacy_startup = false;
    if status.is_failure() {
        tracing::warn!(status = %status.text(), "no wallpaper, the service waits in the tray");
    }

    let mut quitting = false;
    // Set while Close Applications waits for the renderers and the Portal; the service exits when it is done.
    let mut closing: Option<Closing> = None;
    // The pid of the Portal that Close Applications told to close, so it can be ended after the grace even when its
    // connection was already given up.
    let mut closing_portal: Option<u32> = None;
    // The renderers ended on the Quit path, which has no `closing`; the service still waits for a library move.
    let mut quit_done = false;
    // Whether the exit waiting for a library move was logged already.
    let mut exit_held = false;
    // The one Portal that is open, if any; owned by this thread so two that connect together are decided here.
    let mut portals = PortalRegistry::default();
    // When this service last started a Portal that has not registered yet, so a launcher that comes in the meantime
    // does not start a second one (T-03-35).
    let mut portal_spawned: Option<Instant> = None;
    // Whether the library-ops worker runs; one maintenance at a time.
    let mut library_job = false;
    // The prerequisite install the Portal asked for, while it runs: one at a time (T-04-44), and the stage it reached,
    // so a second request is answered with where the running one is.
    let mut install: Option<InstallWorker> = None;
    let mut install_stage: Option<InstallStage> = None;
    // The legacy Launcher rule (D-12): what the service itself starts besides the wallpaper that the plan already
    // started above.
    let start = start_actions(plan.selected_exists, plan.app_visible, portal_wanted);
    tracing::info!(?start, "start rule");
    if start.open_portal {
        match open_portal(None) {
            Ok(()) => portal_spawned = Some(Instant::now()),
            Err(error) => tracing::warn!(%error, "cannot open the Portal at start"),
        }
    }
    // The slot whose renderer was asked to open its developer tools and has not answered yet (Open DevTools, D-13).
    let mut devtools_pending: Option<u8> = None;
    // The runtime state last offered to the Portal, so that only a change is sent (D-23).
    let mut last_runtime: Option<RuntimeState> = None;
    // The engine report last offered to the Portal, so that only a change is sent; reset when a Portal registers or asks.
    let mut last_engines: Option<EngineReportView> = None;
    let legacy_root = legacy_data_dir();
    let timers = event_loop.timers();
    // The memory report exists only while a Portal is connected (D-19): nothing runs until one registers.
    let mut memory = MemoryModule::default();
    module_off("memory");
    // The data hub exists only while a running page needs system data (D-24): nothing runs until one asks.
    let (data_sample, data_release, data_take) = system_sampling_with_thumbnails();
    let mut data = DataModule::new(data_sample, {
        let renderers = Arc::clone(&renderers);
        Arc::new(move |slot, batch| deliver_data(&renderers, slot, batch))
    })
    .with_release(data_release)
    .with_thumbnails(Thumbnails {
        take: data_take,
        deliver: {
            let renderers = Arc::clone(&renderers);
            Arc::new(move |slot, id, bytes| deliver_blob(&renderers, slot, id, bytes))
        },
    });
    if start_fullscreen(&mut fleet, plan.fullscreen, plan.scope, plan.pause) {
        timers.set(FULLSCREEN_TIMER, FULLSCREEN_POLL_MS);
    }
    let loop_end = event_loop.run(|event| match event {
        LoopEvent::Wake => {
            // Set by what can leave the service with nothing to do: the Portal going, a renderer ending, the tray
            // switch (D-13).
            let mut check_exit = false;
            for event in receiver.try_iter() {
                match event {
                    Event::Renderer {
                        slot,
                        message: ToService::Hello { pid, .. },
                        ..
                    } => {
                        tracing::info!(pid, slot, "renderer connected");
                        // A renderer that reconnects asks for its data again once its page has loaded.
                        data.release(slot, None);
                        fleet.catch_up_renderer(slot, &renderers);
                    }
                    Event::Renderer {
                        slot,
                        pid,
                        message: ToService::DataWanted(groups),
                    } => {
                        // Only the renderer that holds the slot counts, and not while the service quits.
                        let holds = lock(&renderers)
                            .connections
                            .get(&slot)
                            .is_some_and(|link| link.pid == pid);
                        if holds && !quitting {
                            data.want(slot, pid, &groups);
                        } else {
                            tracing::debug!(slot, pid, "ignoring a data request of a renderer that is gone or replaced");
                        }
                    }
                    Event::Renderer {
                        slot,
                        pid,
                        message: ToService::DevToolsOpened(opened),
                    } => {
                        // Only the renderer that was asked counts; a late answer after the timeout is dropped.
                        let holds = lock(&renderers)
                            .connections
                            .get(&slot)
                            .is_some_and(|link| link.pid == pid);
                        if holds && devtools_pending == Some(slot) {
                            devtools_pending = None;
                            timers.kill(DEVTOOLS_TIMER);
                            tracing::info!(slot, opened, "the renderer answered Open DevTools");
                            portals.send(PortalEvent::DevToolsResult { opened });
                        } else {
                            tracing::debug!(slot, pid, "ignoring an answer to Open DevTools that nobody waits for");
                        }
                    }
                    Event::Renderer { slot, pid, message } => {
                        let link_pid = lock(&renderers).connections.get(&slot).map(|link| link.pid);
                        // A report can be handled after the Exited of the same renderer (two threads feed the
                        // channel), or after the slot got another renderer; handle_report decides what still
                        // counts (WR-15, WR-18).
                        let running = supervisor.pids();
                        let holder = SlotState {
                            link_pid,
                            running: &running,
                            assigned: fleet.slot_assigned(slot),
                        };
                        let Some(effect) = handle_report(
                            &status,
                            &holder,
                            slot,
                            pid,
                            &message,
                            fleet.job.as_ref().map(Job::current),
                        ) else {
                            tracing::debug!(
                                slot,
                                pid,
                                assigned = holder.assigned,
                                "ignoring a report of a renderer that is gone, was replaced or lost its monitor"
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
                        check_exit = true;
                        // Its data is no longer wanted; the hub stops with the last renderer that wanted any.
                        data.release(exited.slot, Some(exited.pid));
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
                            fleet.note_exit_while_quitting(exited.slot);
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
                            // A switch by restart (D-26) wants its renderers back as soon as the old ones are gone.
                            fleet.respawn_closed(&Ctx {
                                supervisor: &supervisor,
                                renderers: &renderers,
                            });
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
                            fleet.job.as_ref().map(Job::current),
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
                    // The Quit path leaves once nothing else holds it; Close Applications also waits for the Portal. Both
                    // wait for a library move (the exit check at the end of this pass).
                    Event::ShutdownDone => match &mut closing {
                        Some(closing) => closing.renderers_ended(),
                        None => quit_done = true,
                    },
                    Event::PortalConnected {
                        link,
                        version,
                        turned_away,
                    } => {
                        let pid = link.pid();
                        let runtime = current_runtime(&host, &fleet, &supervisor, tray.is_some());
                        let admission = portals.admit(link, version, quitting, || PortalEvent::Welcome {
                            settings: host.view(),
                            runtime: runtime.clone(),
                        });
                        // The reader thread of this connection waits for this; it never matters to anything else.
                        let _ = turned_away.send(matches!(admission, PortalAdmission::AlreadyOpen { .. }));
                        match admission {
                            PortalAdmission::Register => {
                                // The Welcome carries this state, so it is what the Portal knows.
                                last_runtime = Some(runtime);
                                // The engine report follows the Welcome (announced at the end of this pass).
                                last_engines = None;
                                portal_spawned = None;
                                tracing::info!(pid, "portal registered");
                            }
                            PortalAdmission::AlreadyOpen { existing_pid } => {
                                tracing::info!(pid, existing_pid, "a second portal was told the first one is open");
                            }
                            PortalAdmission::Close => {
                                tracing::info!(pid, "a portal connected while the service stops, it was told to close");
                            }
                            PortalAdmission::Refuse(reason) => tracing::warn!(pid, %reason, "refusing a portal"),
                        }
                    }
                    Event::Portal { connection, request } => {
                        if !portals.is_registered(connection) {
                            tracing::debug!("ignoring a request of a portal that is not registered");
                        } else if quitting && !acts_while_quitting(&request) {
                            // WR-02: nothing starts or changes once the renderers were told to close.
                            tracing::debug!("a portal request was ignored, the service is closing");
                            if let Some(answer) = quitting_answer(&request) {
                                portals.send(answer);
                            }
                        } else {
                            match request {
                                PortalRequest::Change(change) => {
                                    // One request at a time on this thread: the last change wins, and the Portal
                                    // only shows what comes back as Changed.
                                    match host.apply(change.clone()) {
                                        Ok(stored) => {
                                            tracing::info!(setting = %describe(&stored), "setting changed");
                                            // Notification Area Icon: the tray appears or goes at once. Changes are
                                            // handled one at a time here, so quick toggles end in the last value.
                                            if let SettingChange::AppVisible(visible) = &stored {
                                                check_exit = true;
                                                match tray_action(tray.is_some(), *visible) {
                                                    ModuleAction::Create => {
                                                        tray = build_tray(
                                                            true,
                                                            &status,
                                                            host.settings().general.app_exit,
                                                            &notifier,
                                                        );
                                                        // A new item is enabled; the sync after the events fixes it.
                                                        open_enabled = true;
                                                    }
                                                    ModuleAction::Drop => {
                                                        tray = None;
                                                        module_off("tray");
                                                    }
                                                    ModuleAction::Keep => {}
                                                }
                                            }
                                            // The exit item says Close Applications or Exit, live.
                                            if let SettingChange::AppExit(app_exit) = &stored
                                                && let Some(tray) = &tray
                                            {
                                                tray.set_exit_label(exit_label(*app_exit));
                                            }
                                            // The wallpaper of that type restarts on the new engine, and the report
                                            // (fallbacks) is sent again at the end of this pass.
                                            if let Some(kind) = engine_change_kind(&stored) {
                                                restart_for_engine(
                                                    kind,
                                                    &host,
                                                    &mut fleet,
                                                    legacy_root.as_deref(),
                                                    &renderers,
                                                );
                                                last_engines = None;
                                            }
                                            // Loop Mode and Shuffle Mode reach the playing wallpapers at once; the other
                                            // Wallpaper settings are read when the next wallpaper starts.
                                            if let Some(message) = live_message(&stored) {
                                                let told = fleet.apply_live(&message, &renderers);
                                                tracing::info!(renderers = told.len(), "a live setting was sent to the renderers");
                                            }
                                            portals.send(PortalEvent::Changed(stored));
                                        }
                                        Err(reason) => {
                                            tracing::warn!(setting = %describe(&change), %reason, "setting change refused");
                                            portals.send(PortalEvent::Rejected { change, reason });
                                        }
                                    }
                                }
                                PortalRequest::Use { folder } => {
                                    let done = handle_use(
                                        &folder,
                                        &mut host,
                                        &mut fleet,
                                        legacy_root.as_deref(),
                                        platform.as_ref(),
                                        &Ctx {
                                            supervisor: &supervisor,
                                            renderers: &renderers,
                                        },
                                    );
                                    if done.arm_fullscreen {
                                        timers.set(FULLSCREEN_TIMER, FULLSCREEN_POLL_MS);
                                    }
                                    if let Some(next) = done.status {
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
                                    portals.send(PortalEvent::UseResult {
                                        folder,
                                        outcome: done.outcome,
                                    });
                                }
                                PortalRequest::DetectEngines => {
                                    // Look again: an engine installed or repaired meanwhile is used from the next start of
                                    // a wallpaper (D-11); the answer is sent at the end of this pass.
                                    fleet.engines = engines::detect(&engines::exe_dir(), platform.as_ref());
                                    last_engines = None;
                                }
                                PortalRequest::InstallPrerequisite { prerequisite } => {
                                    // The user confirmed in the install dialog (D-15). One install runs at a time: a
                                    // second request is told where the running one is.
                                    if install.is_some() {
                                        if let Some(stage) = install_stage {
                                            portals.send(PortalEvent::InstallProgress(stage.view()));
                                        }
                                    } else if let Some(root) = sucrose_core::paths::cache_dir() {
                                        let prerequisite = Prerequisite::from_view(prerequisite);
                                        let reporter = notifier.clone();
                                        match InstallWorker::start(prerequisite, Arc::new(SystemSteps), root, move |stage| {
                                            reporter.send(Event::Install(stage));
                                        }) {
                                            Ok(worker) => {
                                                tracing::info!(?prerequisite, "install started");
                                                install = Some(worker);
                                                install_stage = None;
                                            }
                                            Err(error) => {
                                                tracing::warn!(%error, "cannot start the install thread");
                                                portals.send(PortalEvent::InstallProgress(
                                                    InstallStage::Failed(crate::install::InstallFailure::Installer).view(),
                                                ));
                                            }
                                        }
                                    } else {
                                        tracing::warn!("no data folder, so no installer can be downloaded");
                                        portals.send(PortalEvent::InstallProgress(
                                            InstallStage::Failed(crate::install::InstallFailure::Download).view(),
                                        ));
                                    }
                                }
                                PortalRequest::CancelInstall => {
                                    if let Some(worker) = &install {
                                        worker.cancel();
                                    }
                                }
                                PortalRequest::SnoozeFallback { engine } => {
                                    // "Don't remind me for a day" (D-16): the quiet time goes into the Warehouse field of
                                    // that engine, by this thread, the only writer. Any other engine changes nothing on disk
                                    // (T-04-34); the report is sent again at the end of this pass with notify false.
                                    let engine = EngineId::from_legacy(engine);
                                    if engines::can_snooze(engine) {
                                        let now = engines::unix_now();
                                        match host.update(|settings| {
                                            engines::snooze(&mut settings.warehouse, engine, now);
                                        }) {
                                            Ok(()) => {
                                                tracing::info!(engine = engines::label(engine), "fallback toast snoozed for a day");
                                            }
                                            Err(reason) => tracing::warn!(%reason, "the snooze could not be saved"),
                                        }
                                    }
                                    last_engines = None;
                                }
                                PortalRequest::MaintainLibrary => {
                                    match maintenance_decision(library_job, fleet.library_move.is_some()) {
                                        MaintenanceDecision::Wait => {
                                            // The one registered Portal gets the report of the run that is going.
                                            tracing::debug!("library maintenance is running, the answer comes when it ends");
                                        }
                                        MaintenanceDecision::AnswerNothing => {
                                            tracing::debug!("library maintenance skipped: the library is being moved");
                                            portals.send(PortalEvent::LibraryMaintained(NOTHING_MAINTAINED));
                                        }
                                        MaintenanceDecision::Start => {
                                            if let Some(root) = legacy_root.as_deref() {
                                                let input = MaintainInput::from_settings(host.settings(), root);
                                                if start_library_ops(input, &notifier) {
                                                    library_job = true;
                                                } else {
                                                    portals.send(PortalEvent::LibraryMaintained(NOTHING_MAINTAINED));
                                                }
                                            } else {
                                                portals.send(PortalEvent::LibraryMaintained(NOTHING_MAINTAINED));
                                            }
                                        }
                                    }
                                }
                                PortalRequest::MoveLibrary { destination } => {
                                    let mut env = MoveEnv {
                                        host: &mut host,
                                        fleet: &mut fleet,
                                        legacy_root: legacy_root.as_deref(),
                                        ctx: &Ctx {
                                            supervisor: &supervisor,
                                            renderers: &renderers,
                                        },
                                        notifier: &notifier,
                                        timers: &timers,
                                        portals: &mut portals,
                                        quitting,
                                    };
                                    env.begin(&destination, library_job);
                                }
                                PortalRequest::SetStartup(mode) => {
                                    set_startup(mode, &startup_exe, &SystemStartup, &mut host);
                                    portals.send(PortalEvent::Startup(startup_info(
                                        &host,
                                        &startup_exe,
                                        &SystemStartup,
                                        legacy_startup,
                                    )));
                                }
                                PortalRequest::RefreshStartup => {
                                    // The answer comes with the C# lookup; without a thread the last one is used.
                                    if !start_legacy_check(&notifier) {
                                        portals.send(PortalEvent::Startup(startup_info(
                                            &host,
                                            &startup_exe,
                                            &SystemStartup,
                                            legacy_startup,
                                        )));
                                    }
                                }
                                PortalRequest::GetArguments { engine } => {
                                    // The rules and the storage format of the switches are this service's (D-17): the
                                    // Portal gets the lists ready to show.
                                    portals.send(arguments_answer(engine, host.settings(), sucrose_platform::os_build()));
                                }
                                PortalRequest::OpenDevTools { monitor } => {
                                    // Only this button opens DevTools (D-13): the renderer of that monitor, and only when
                                    // it plays a web wallpaper on a web engine (T-04-39).
                                    let running = supervisor.slots();
                                    let job = fleet.job.as_ref().map(|job| (job.kind, job.engine));
                                    let target = devtools_slot(job, &fleet.slots, &running, &monitor);
                                    match target {
                                        // A second click while the renderer is still answering waits for the same answer.
                                        Some(slot) if devtools_pending == Some(slot) => {
                                            tracing::debug!(slot, "Open DevTools is already on its way to this renderer");
                                        }
                                        Some(slot) if devtools_pending.is_none() && send_to(&renderers, slot, &ToRenderer::OpenDevTools) => {
                                            devtools_pending = Some(slot);
                                            timers.set(DEVTOOLS_TIMER, DEVTOOLS_WAIT_MS);
                                            tracing::info!(slot, "Open DevTools was sent to the renderer");
                                        }
                                        _ => {
                                            tracing::info!(%monitor, "Open DevTools has no web wallpaper to open");
                                            portals.send(PortalEvent::DevToolsResult { opened: false });
                                        }
                                    }
                                }
                                PortalRequest::RestartWallpapers { engines } => {
                                    // "Restart now" of the restart prompt: only the wallpapers on the listed engines
                                    // restart, with the settings as saved now (T-04-38).
                                    match restart_wallpapers(
                                        &engines,
                                        &host,
                                        &mut fleet,
                                        legacy_root.as_deref(),
                                        &renderers,
                                    ) {
                                        Ok(closed) => tracing::info!(
                                            ?engines,
                                            renderers = closed,
                                            "wallpapers restart to apply a saved setting"
                                        ),
                                        Err(reason) => {
                                            tracing::warn!(?engines, %reason, "the wallpapers could not be restarted");
                                            portals.send(PortalEvent::RestartFailed { reason });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Event::LibraryOps(done) => {
                        library_job = false;
                        // Recorded only now, after the copies succeeded; a failed save only means the next run finds the
                        // folders there and records them.
                        if !done.copied.is_empty()
                            && let Err(reason) =
                                host.update(|settings| library_ops::record_names(&mut settings.warehouse.showcase, &done.copied))
                        {
                            tracing::warn!(%reason, "the copied showcase names could not be saved");
                        }
                        tracing::info!(
                            showcase_copied = done.report.showcase_copied,
                            corrupt_moved = done.report.corrupt_moved,
                            corrupt_skipped = done.report.corrupt_skipped,
                            "library maintained"
                        );
                        portals.send(PortalEvent::LibraryMaintained(done.report));
                    }
                    Event::Install(stage) => {
                        portals.send(PortalEvent::InstallProgress(stage.view()));
                        if stage.is_final() {
                            install = None;
                            install_stage = None;
                            if stage == InstallStage::Done {
                                // Look again: the engine is used from the next start of a wallpaper (D-11); the new
                                // report is sent at the end of this pass.
                                fleet.engines = engines::detect(&engines::exe_dir(), platform.as_ref());
                                last_engines = None;
                            }
                        } else {
                            install_stage = Some(stage);
                        }
                    }
                    Event::StartupLegacy(legacy) => {
                        legacy_startup = legacy;
                        portals.send(PortalEvent::Startup(startup_info(
                            &host,
                            &startup_exe,
                            &SystemStartup,
                            legacy_startup,
                        )));
                    }
                    Event::LibraryMove(step) => {
                        let progress = MoveEnv {
                            host: &mut host,
                            fleet: &mut fleet,
                            legacy_root: legacy_root.as_deref(),
                            ctx: &Ctx {
                                supervisor: &supervisor,
                                renderers: &renderers,
                            },
                            notifier: &notifier,
                            timers: &timers,
                            portals: &mut portals,
                            quitting,
                        }
                        .step(step);
                        settle_move(progress, &mut status, tray.as_ref(), &mut check_exit);
                    }
                    Event::PortalGone { connection } => {
                        let was_registered = portals.gone(connection);
                        let effect = portal_gone_effect(was_registered);
                        if was_registered {
                            tracing::info!("the portal disconnected");
                        } else {
                            tracing::debug!("a portal connection ended that was not the registered one");
                        }
                        // A Portal the service dropped (WR-04) or never registered can still be the only thing that
                        // kept the service alive: the exit rule is checked whichever connection ended.
                        check_exit |= effect.check_exit;
                        if effect.tell_closing
                            && let Some(closing) = &mut closing
                        {
                            closing.portal_gone();
                        }
                    }
                    Event::PortalFocus { page } => {
                        portals.send(PortalEvent::Focus { page });
                    }
                    Event::Tray(TrayAction::Open) => {
                        if quitting {
                            tracing::debug!("Open chosen while the service stops, ignored");
                        } else if portals.registered_pid().is_some() {
                            // The item is greyed out while the Portal is open, but a double-click is not: bring the
                            // open Portal to the front.
                            portals.send(PortalEvent::Focus { page: None });
                        } else if spawn_pending(portal_spawned, Instant::now()) {
                            tracing::debug!("Open chosen while the Portal is starting, ignored");
                        } else {
                            match open_portal(None) {
                                Ok(()) => portal_spawned = Some(Instant::now()),
                                Err(error) => {
                                    tracing::error!(%error, "cannot open the Portal from the tray");
                                    if let Some(tray) = &tray {
                                        tray.set_tooltip(OPEN_FAILED_TOOLTIP);
                                    }
                                }
                            }
                        }
                    }
                    Event::Tray(TrayAction::Exit) => {
                        if quitting {
                            tracing::debug!("Exit chosen while the service stops, ignored");
                        } else if fleet.library_move.is_some() {
                            // Leaving in the middle of a move could leave the library split between two folders.
                            tracing::info!("Exit chosen while the library moves, ignored");
                        } else if host.settings().general.app_exit {
                            // Close Applications (TRAY-05): the renderers, the Portal, then the service.
                            quitting = true;
                            if let Some(worker) = &install {
                                worker.cancel();
                            }
                            tracing::info!("Close Applications chosen, closing the renderers and the Portal");
                            for id in [TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE, FULLSCREEN_TIMER, MEMORY_TIMER] {
                                timers.kill(id);
                            }
                            closing_portal = portals.registered_pid();
                            closing = Some(Closing::new(supervisor.pids().len(), closing_portal.is_some()));
                            start_shutdown(&renderers, &supervisor, &notifier);
                            if closing_portal.is_some() {
                                portals.send(PortalEvent::Close);
                                timers.set(CLOSE_TIMER, CLOSE_GRACE_MS);
                            }
                        } else {
                            // Exit (AppExit off): only the tray goes. The wallpaper keeps playing and the exit rule
                            // (D-13) decides about the service.
                            tracing::info!("Exit chosen, removing the tray");
                            tray = None;
                            module_off("tray");
                            check_exit = true;
                        }
                    }
                    Event::Launch { page, reply } => {
                        // A launch while the tray is wanted but was removed by Exit shows it again (the legacy
                        // relaunch does the same).
                        if !quitting && tray.is_none() && host.settings().general.app_visible {
                            tray = build_tray(true, &status, host.settings().general.app_exit, &notifier);
                            open_enabled = true;
                        }
                        let answer = if quitting {
                            LaunchReply::Refused {
                                reason: "Sucrose is stopping".to_owned(),
                            }
                        } else if let Some(pid) = portals.registered_pid() {
                            LaunchReply::PortalRunning { pid }
                        } else if spawn_pending(portal_spawned, Instant::now()) {
                            LaunchReply::Opened
                        } else {
                            match open_portal(page) {
                                Ok(()) => {
                                    portal_spawned = Some(Instant::now());
                                    LaunchReply::Opened
                                }
                                Err(error) => {
                                    tracing::warn!(%error, "cannot start the Portal for a launcher");
                                    LaunchReply::Refused {
                                        reason: "the Portal could not be started".to_owned(),
                                    }
                                }
                            }
                        };
                        // The launcher's connection thread may have given up waiting; nothing to do then.
                        let _ = reply.send(answer);
                    }
                }
            }
            // A library move goes on once the renderers it closed have ended.
            let progress = MoveEnv {
                host: &mut host,
                fleet: &mut fleet,
                legacy_root: legacy_root.as_deref(),
                ctx: &Ctx {
                    supervisor: &supervisor,
                    renderers: &renderers,
                },
                notifier: &notifier,
                timers: &timers,
                portals: &mut portals,
                quitting,
            }
            .drive();
            settle_move(progress, &mut status, tray.as_ref(), &mut check_exit);
            let finished = quit_done || closing.as_ref().is_some_and(Closing::done);
            if settle_exit(finished, fleet.library_move.is_some(), &mut exit_held) {
                tracing::info!("the renderers ended and the Portal is gone, the service exits");
                return Flow::Exit;
            }
            if check_exit
                && !quitting
                && fleet.library_move.is_none()
                && should_exit(
                    tray.is_some(),
                    fleet.wallpaper_wanted(),
                    portals.registered_pid().is_some() || spawn_pending(portal_spawned, Instant::now()),
                )
            {
                tracing::info!("nothing needs the service, exiting");
                tray_proxy.quit();
            }
            // The Open item is greyed out exactly while a Portal is registered.
            let portal_open = portals.registered_pid().is_some();
            if open_enabled == portal_open {
                open_enabled = !portal_open;
                if let Some(tray) = &tray {
                    tray.set_open_enabled(open_enabled);
                }
            }
            // The memory report follows the Portal: started when one is registered, stopped when it is gone.
            if !quitting {
                run_timers(&timers, memory.follow(portals.registered_pid().is_some()).into_iter().collect());
            }
            // Whatever the events changed (a Use, an exit, a first frame, a setting) reaches the Portal once.
            announce_runtime(&mut portals, &mut last_runtime, &host, &fleet, &supervisor, tray.is_some());
            publish_engines(&mut portals, &mut last_engines, fleet.engine_view(&host.settings().warehouse));
            Flow::Continue
        }
        LoopEvent::Quit => {
            if !quitting {
                quitting = true;
                if let Some(worker) = &install {
                    worker.cancel();
                }
                tracing::info!("Quit chosen, closing the renderers");
                // No monitor change is applied while the renderers are being closed.
                for id in [TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE, FULLSCREEN_TIMER, MEMORY_TIMER] {
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
        // The Portal had its grace to close: end it if it is still there and is the sibling Portal executable.
        LoopEvent::Timer(CLOSE_TIMER) => {
            timers.kill(CLOSE_TIMER);
            if let Some(closing) = &mut closing {
                if !closing.portal_done
                    && let Some(pid) = closing_portal
                {
                    match portal_path() {
                        Ok(sibling) => {
                            let end = end_portal(pid, &sibling);
                            match end {
                                PortalEnd::Ended => tracing::warn!(pid, "the Portal did not close in time, it was ended"),
                                PortalEnd::Gone => tracing::info!(pid, "the Portal was already gone"),
                                PortalEnd::NotThePortal => {
                                    tracing::warn!(pid, "the process is not the Portal executable, it is left alone");
                                }
                                PortalEnd::Failed => tracing::warn!(pid, "the Portal could not be ended"),
                            }
                        }
                        Err(error) => tracing::warn!(pid, %error, "cannot tell where the Portal lives, it is left alone"),
                    }
                    // Whatever the outcome, there is nothing more to wait for: a process that is not ours is not
                    // waited for, and one that was ended is gone.
                    closing.portal_gone();
                }
                if settle_exit(closing.done(), fleet.library_move.is_some(), &mut exit_held) {
                    tracing::info!("Close Applications is done, the service exits");
                    return Flow::Exit;
                }
            }
            Flow::Continue
        }
        // The renderers did not leave for the library move in time: end the ones that are left.
        LoopEvent::Timer(MOVE_TIMER) => {
            timers.kill(MOVE_TIMER);
            if fleet.library_move.is_some() && fleet.rule_closed.any_awaiting() {
                tracing::warn!("the renderers did not close for the library move in time, ending them");
                supervisor.kill_survivors();
            }
            Flow::Continue
        }
        LoopEvent::Timer(MEMORY_TIMER) => {
            let portal = portals.registered_pid();
            if quitting {
                timers.kill(MEMORY_TIMER);
            } else if portal.is_none() {
                // The Portal went away between two messages: the module stops with it.
                run_timers(&timers, memory.follow(false).into_iter().collect());
            } else if let Some(report) =
                memory.sample(std::process::id(), &pids_by_slot(&supervisor.pids()), portal)
            {
                // The writer drops a Memory event when the Portal is slow, the next reading replaces it (D-19).
                portals.send(PortalEvent::Memory(report));
            }
            Flow::Continue
        }
        LoopEvent::Timer(FULLSCREEN_TIMER) => {
            if quitting {
                timers.kill(FULLSCREEN_TIMER);
            } else if fleet.library_move.is_none() {
                fleet.fullscreen_tick(
                    platform.as_ref(),
                    &Ctx {
                        supervisor: &supervisor,
                        renderers: &renderers,
                    },
                );
            }
            announce_runtime(&mut portals, &mut last_runtime, &host, &fleet, &supervisor, tray.is_some());
            Flow::Continue
        }
        LoopEvent::Timer(DEVTOOLS_TIMER) => {
            timers.kill(DEVTOOLS_TIMER);
            // The renderer did not confirm in time: the Portal is told it failed instead of waiting for ever.
            if devtools_pending.take().is_some() {
                tracing::warn!("a renderer did not answer Open DevTools in time");
                portals.send(PortalEvent::DevToolsResult { opened: false });
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
            // No renderer starts for a monitor while the library moves: it would open files of the old folder. The
            // snapshot waits for the end of the move.
            if let Some(monitors) = step.apply {
                let applied = fleet.apply_or_defer(
                    monitors,
                    quitting,
                    &Ctx {
                        supervisor: &supervisor,
                        renderers: &renderers,
                    },
                );
                if applied.started > 0 && status.is_waiting_for_a_monitor() {
                    if let Some(job) = &fleet.job {
                        status = Status::Starting {
                            title: job.title.clone(),
                            fallback: job.fallback.clone(),
                        };
                    }
                    if let Some(tray) = &tray {
                        tray.set_tooltip(&status.tooltip());
                    }
                }
            }
            announce_runtime(&mut portals, &mut last_runtime, &host, &fleet, &supervisor, tray.is_some());
            Flow::Continue
        }
        // Only a renderer registers for raw input; the service never does.
        LoopEvent::Input(_) => Flow::Continue,
        // explorer.exe started again: the surfaces have to be replaced, so each monitor gets a new renderer.
        LoopEvent::ShellRestarted => {
            if !quitting && fleet.job.is_some() && fleet.library_move.is_none() {
                let acted = fleet.shell_restarted(&Ctx {
                    supervisor: &supervisor,
                    renderers: &renderers,
                });
                if acted && let Some(next) = status.restarting(fleet.job.as_ref().map(Job::current)) {
                    status = next;
                    if let Some(tray) = &tray {
                        tray.set_tooltip(&status.tooltip());
                    }
                }
            }
            announce_runtime(&mut portals, &mut last_runtime, &host, &fleet, &supervisor, tray.is_some());
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
    // The first message says which kind of peer this is (D-033); a renderer's Hello continues below.
    match &first {
        ToService::PortalHello {
            portal_protocol_version,
            pid,
            ..
        } => {
            serve_portal(connection, notifier, *portal_protocol_version, *pid);
            return;
        }
        ToService::Launch(launch) => {
            serve_launcher(connection, notifier, *launch);
            return;
        }
        _ => {}
    }
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
        let current = lock_assignments(assignments).get(&slot).cloned();
        match current {
            // Through the registered writer, under the lock every other sender takes (WR-12).
            Some(current) => {
                if current.rect != sent.rect {
                    send_to(renderers, slot, &ToRenderer::SetRect(current.rect));
                }
                // A Use (D-26) that came in the same gap: the renderer plays the theme it was assigned, not the new one.
                if current.source != sent.source || current.settings != sent.settings {
                    send_to(
                        renderers,
                        slot,
                        &ToRenderer::LoadSource {
                            source: current.source,
                            settings: current.settings,
                        },
                    );
                }
            }
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

/// How long a launcher waits for the service's answer; the main thread answers at once, so a longer wait means it is
/// busy shutting down.
const LAUNCH_REPLY_WAIT: Duration = Duration::from_secs(5);

/// Serves one launcher's connection: hands its request to the main thread and writes the answer back. The connection
/// ends with the launcher.
fn serve_launcher(mut connection: Connection, notifier: &Notifier, launch: Launch) {
    let Launch::OpenPortal { page } = launch else {
        tracing::warn!("a launcher started with a foreground grant instead of a request, dropping the connection");
        return;
    };
    let (reply, answer) = crossbeam_channel::bounded(1);
    if !notifier.send(Event::Launch { page, reply }) {
        return;
    }
    let reply = answer
        .recv_timeout(LAUNCH_REPLY_WAIT)
        .unwrap_or_else(|_| LaunchReply::Refused {
            reason: "the service did not answer in time".to_owned(),
        });
    if let Err(error) = connection.send(&reply) {
        tracing::warn!(%error, "cannot answer a launcher");
        return;
    }
    // A launcher that found the Portal open passes the foreground right on and expects it to come to the front.
    if matches!(reply, LaunchReply::PortalRunning { .. }) {
        read_foreground_grant(&mut connection, notifier);
    }
}

/// Reads the one message that follows a refusal or a `PortalRunning` answer: the foreground grant. The Portal that is
/// open is told to come to the front. Anything else, or a peer that leaves without it, changes nothing.
fn read_foreground_grant(connection: &mut Connection, notifier: &Notifier) {
    match connection.recv::<ToService>() {
        Ok(ToService::Launch(Launch::ForegroundGranted { page })) => {
            notifier.send(Event::PortalFocus { page });
        }
        Ok(_) => tracing::debug!("a peer sent something else instead of the foreground grant, ignored"),
        Err(IpcError::Closed) => tracing::debug!("a peer left without passing on the foreground"),
        Err(error) => tracing::debug!(%error, "cannot read the foreground grant"),
    }
}

/// Numbers the Portal connections, so an event is never attributed to another connection that claims the same pid.
static NEXT_PORTAL_CONNECTION: AtomicU64 = AtomicU64::new(1);

/// Serves one Portal's connection: hands its writer to the main thread, which decides whether it is the one Portal,
/// then forwards its requests until it goes. Whatever goes wrong here ends this connection only (T-03-14).
fn serve_portal(mut connection: Connection, notifier: &Notifier, version: u16, pid: u32) {
    let id = NEXT_PORTAL_CONNECTION.fetch_add(1, Ordering::Relaxed);
    let link = match connection
        .try_clone()
        .map_err(|error| error.to_string())
        .and_then(|clone| PortalLink::for_connection(id, pid, clone).map_err(|error| error.to_string()))
    {
        Ok(link) => link,
        Err(error) => {
            tracing::warn!(%error, "cannot keep a write handle for a portal");
            return;
        }
    };
    let (turned_away, verdict) = crossbeam_channel::bounded(1);
    if !notifier.send(Event::PortalConnected {
        link,
        version,
        turned_away,
    }) {
        return;
    }
    // A Portal that is not the one open passes the foreground right on before it exits; the one open focuses then.
    if verdict.recv_timeout(LAUNCH_REPLY_WAIT).unwrap_or(false) {
        read_foreground_grant(&mut connection, notifier);
        notifier.send(Event::PortalGone { connection: id });
        return;
    }
    loop {
        match connection.recv::<PortalRequest>() {
            Ok(request) => {
                if !notifier.send(Event::Portal {
                    connection: id,
                    request,
                }) {
                    return;
                }
            }
            Err(IpcError::Closed) => break,
            Err(error) => {
                tracing::warn!(%error, "dropping a portal connection after an invalid message");
                break;
            }
        }
    }
    notifier.send(Event::PortalGone { connection: id });
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

/// Who holds a report's slot when the report is handled.
struct SlotState<'a> {
    /// The process whose connection the slot holds.
    link_pid: Option<u32>,
    /// The processes the supervisor runs (pid to slot).
    running: &'a HashMap<u32, u8>,
    /// The slot belongs to a connected monitor and the fullscreen rule has not closed its renderer.
    assigned: bool,
}

/// Decides what a report of process `pid` on `slot` does, or `None` when it is dropped. `holder` says who holds the
/// slot (see [`SlotState`]). A renderer registers its connection before its first report is queued and the
/// connection goes when the renderer ended, so:
/// - a report of a renderer that was replaced (another connection, or another process runs for the slot) is dropped;
/// - a report for a slot whose monitor was removed, or whose renderer the fullscreen rule closed, is dropped, whether
///   or not the renderer is still connected: the single tray status must not say "no wallpaper" while the other
///   monitors play (WR-19);
/// - a report of a renderer that is gone and has no replacement can still be its last word: `Failed` is the one
///   report a renderer sends right before it ends, and its exit may be handled first (two threads feed the channel),
///   so it is kept (WR-18). A late `Playing` would claim a wallpaper nobody shows and is dropped (WR-15).
fn handle_report(
    status: &Status,
    holder: &SlotState<'_>,
    slot: u8,
    pid: u32,
    message: &ToService,
    current: Option<Current<'_>>,
) -> Option<ReportEffect> {
    let replaced = holder.link_pid.is_some_and(|current| current != pid)
        || exit_is_stale(holder.running, slot, pid)
        || !holder.assigned;
    let late = holder.link_pid != Some(pid);
    if replaced || (late && matches!(message, ToService::Playing)) {
        return None;
    }
    // The restart budget's verdict stays when the last words of the renderer that spent it arrive after its exit:
    // the status already says the monitor gets no more renderers.
    let kept = late && matches!(message, ToService::Failed { .. }) && matches!(status, Status::RendererStopped { .. });
    Some(ReportEffect {
        first_frame: matches!(message, ToService::Playing),
        status: if kept { None } else { status.after(message, current) },
    })
}

/// The status after a renderer ended and `decision` was taken: a restart keeps a failed start's reason, a spent budget
/// names the monitor and why (WR-17). `None` when nothing changes.
fn status_after_exit(
    status: &Status,
    decision: Decision,
    class: ExitClass,
    current: Option<Current<'_>>,
    monitor: String,
) -> Option<Status> {
    match decision {
        Decision::Leave => None,
        Decision::Restart(_) => status.restarting_after(class, current),
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
    /// Whether the selected theme has a folder in the library. A theme the gate rejects still counts, like in the
    /// legacy Launcher (D-12).
    selected_exists: bool,
    /// Backgroundog.json FullScreenPerformance: what a fullscreen app in front does to the wallpaper.
    fullscreen: FullScreenAction,
    /// Backgroundog.FullScreenScope: which monitors the fullscreen rule acts on.
    scope: ActionScope,
    /// Backgroundog.json PausePerformanceType.
    pause: PauseKind,
    /// Engine.json GraphicPreference: the GPU Windows is asked to give the renderers (DESK-12).
    gpu: GpuPreference,
}

/// The wallpaper types the service plays in this build (D-31 slices 1 to 3); later slices extend it. Gif plays on the
/// built-in renderer or, as the migrated settings say, on mpv; Url and YouTube play on the web engines.
const PLAYABLE: &[WallpaperType] = &[
    WallpaperType::Video,
    WallpaperType::Web,
    WallpaperType::Gif,
    WallpaperType::Url,
    WallpaperType::YouTube,
];

/// Whether a running renderer can load another wallpaper in place: only mpv can. A renderer is one process with one
/// engine, so a Use that changes the engine (mpv, the built-in GIF renderer, a web engine) needs a new renderer; and
/// neither a web engine nor the built-in renderer switches to another file in place yet (their `load_source` is a
/// restart until later plans), so a Use while one of them runs starts a new renderer too.
fn loads_in_place(running: EngineType, next: EngineType) -> bool {
    EngineId::from_legacy(running) == EngineId::Mpv && EngineId::from_legacy(next) == EngineId::Mpv
}

/// What every renderer plays: the same video or web page with the same settings on each monitor (D-01).
#[derive(Debug, Clone)]
struct Job {
    /// The theme's title, for the tooltip.
    title: String,
    /// The absolute path of the video, or of the page of a Web theme.
    source: String,
    /// How it is played.
    settings: PlaybackSettings,
    /// Whether a surface covers its whole monitor or only the work area (Engine.json ScreenType).
    screen: ScreenArea,
    /// The theme's wallpaper type.
    kind: WallpaperType,
    /// The canonical legacy name of the engine that plays it, chosen by `choose_engine` (D-11).
    engine: EngineType,
    /// Set when another engine than the configured one plays it, for this run only (D-11).
    fallback: Option<FallbackNote>,
    /// The absolute path of the theme folder.
    theme_dir: String,
    /// How the web engines behave for this job (D-12).
    web: WebOptions,
    /// Which input a web wallpaper gets (WAPI-07).
    input: InputSettings,
}

impl Job {
    /// The title and fallback the status transitions need.
    fn current(&self) -> Current<'_> {
        Current {
            title: &self.title,
            fallback: self.fallback.as_ref(),
        }
    }

    /// The assignment of the renderer `slot` for `monitor`, whose rectangle is already the area to cover.
    fn assignment(&self, slot: u8, monitor: &MonitorInfo, web_profile: ProfileKind) -> Assignment {
        let area = monitor.rect;
        Assignment {
            slot,
            monitor: monitor.id.clone(),
            rect: [area.left, area.top, area.right, area.bottom],
            source: self.source.clone(),
            settings: self.settings,
            kind: self.kind,
            engine: self.engine,
            theme_dir: self.theme_dir.clone(),
            web: self.web.clone(),
            input: self.input,
            web_profile,
        }
    }
}

/// Offers a data snapshot to the renderer of `slot` through its writer queue, never waiting for it (T-04-24).
fn deliver_data(renderers: &Mutex<Renderers>, slot: u8, batch: Vec<(DataGroup, String)>) -> Delivery {
    match lock(renderers).connections.get(&slot) {
        Some(link) => match link.offer(ToRenderer::Data(batch)) {
            Offer::Queued => Delivery::Sent,
            Offer::Dropped => Delivery::Dropped,
            Offer::Failed => Delivery::Gone,
        },
        None => Delivery::Gone,
    }
}

/// Offers the album art `bytes` (blob `id`) to the renderer of `slot` as chunks through its writer queue, never waiting
/// for it. The chunks are not droppable like a snapshot; if the queue cannot take all of them the delivery is
/// [`Delivery::Dropped`] and the hub offers the art again as a new blob (the renderer drops the partial one).
fn deliver_blob(renderers: &Mutex<Renderers>, slot: u8, id: u32, bytes: &[u8]) -> Delivery {
    let messages = blob::chunks(id, bytes);
    if messages.is_empty() {
        // Over the cap: the hub never offers it, but a chunkless blob must not be reported as sent.
        return Delivery::Dropped;
    }
    let renderers = lock(renderers);
    let Some(link) = renderers.connections.get(&slot) else {
        return Delivery::Gone;
    };
    for message in messages {
        if link.offer(message) != Offer::Queued {
            return Delivery::Dropped;
        }
    }
    Delivery::Sent
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
    /// What the machine can play with, as detected at the start or when a Portal last asked (ENG-06).
    engines: EngineReport,
    /// The fallbacks that run now (D-11); nothing is stored.
    fallbacks: Fallbacks,
    slots: SlotMap,
    /// The monitors the renderers follow, as last applied, with the rectangle of the area they cover.
    monitors: Vec<MonitorInfo>,
    /// The monitor whose renderer asks for the persistent web profile (D-040): the primary of the snapshot being
    /// applied, set before its spawns because the monitor list above is updated after them.
    web_owner: Option<String>,
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
    /// The library move that is going on, if any (`RuntimeState.library_moving`).
    library_move: Option<LibraryMove>,
    /// A monitor change that settled while the library moved, applied when the move ends; the latest one wins (WR-03).
    pending_snapshot: Option<Vec<MonitorInfo>>,
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

    /// Whether any closed slot's renderer has still to say it ended.
    fn any_awaiting(&self) -> bool {
        self.closed.iter().any(|slot| !self.ended.contains(slot))
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
    /// Whether `slot` belongs to a connected monitor and the fullscreen rule has not closed its renderer: only then
    /// do the reports of its renderer count for the tray status (WR-19).
    fn slot_assigned(&self, slot: u8) -> bool {
        self.slots.monitor_of(slot).is_some() && !self.rule_closed.is_closed(slot)
    }

    fn new(job: Option<Job>, assignments: SharedAssignments, engines: EngineReport) -> Self {
        let mut fallbacks = Fallbacks::default();
        if let Some(job) = &job {
            fallbacks.set(job.kind, job.fallback.clone());
        }
        Self {
            job,
            engines,
            fallbacks,
            slots: SlotMap::default(),
            monitors: Vec::new(),
            web_owner: None,
            assignments,
            debounce: Debounce::default(),
            budgets: HashMap::new(),
            stopped: HashSet::new(),
            shell: ShellRestart::default(),
            fullscreen: None,
            rule_closed: RuleClosed::default(),
            playing: HashSet::new(),
            library_move: None,
            pending_snapshot: None,
        }
    }

    /// Switches the fullscreen rule on for `action` unless it is Resume, which disables it: then no timer and no
    /// state exist at all (D-008). `true` when the rule is on and its timer has to run.
    fn enable_fullscreen(&mut self, action: FullScreenAction, scope: ActionScope, pause: PauseKind) -> bool {
        if action == FullScreenAction::Resume || self.job.is_none() {
            return false;
        }
        if pause == PauseKind::Heavy {
            tracing::info!("PausePerformanceType Heavy is handled like Light in M1");
        }
        tracing::info!(?action, "fullscreen rule on");
        self.fullscreen = Some(Rule::new(action, scope));
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
        // While the library moves nothing starts: it would open files of the old folder (WR-03). The move's end calls
        // this again, and the slots stay wanted until then.
        if self.library_move.is_some() {
            return;
        }
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

    /// A renderer ended while the service quits. A library move that closed it still has to see the exit: without it
    /// the move never leaves `Closing`, and the exit waits for the move forever (WR-10). Nothing else is bookkept
    /// while quitting.
    fn note_exit_while_quitting(&mut self, slot: u8) {
        if self.library_move.is_some() {
            self.rule_closed.note_exit(slot);
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

    /// Handles a monitor snapshot that settled: while the service quits it is dropped, while the library moves it is
    /// kept for the end of the move (the latest wins, WR-03: no renderer may start on the old folder), otherwise it is
    /// applied.
    fn apply_or_defer(&mut self, monitors: Vec<MonitorInfo>, quitting: bool, ctx: &Ctx<'_>) -> Applied {
        if quitting {
            return Applied::default();
        }
        if self.library_move.is_some() {
            tracing::debug!("a monitor change settled while the library moves, applying it when the move ends");
            self.pending_snapshot = Some(monitors);
            return Applied::default();
        }
        self.apply_snapshot(monitors, ctx)
    }

    /// The fleet catches up after a library move: the monitor change that settled meanwhile is applied (the job is
    /// the new library's by now) and the closed renderers that still have a monitor start again.
    fn finish_move(&mut self, ctx: &Ctx<'_>) -> Applied {
        let mut applied = Applied::default();
        if let Some(monitors) = self.pending_snapshot.take() {
            // With no job (the theme is gone) the topology is ignored, as it is outside a move.
            if self.job.is_some() {
                applied = self.apply_snapshot(monitors, ctx);
            }
        }
        self.respawn_closed(ctx);
        applied
    }

    /// Applies the monitors of a snapshot that two looks agreed on and re-places every renderer's surface, changed or
    /// not (legacy parity); the master is elected again only when monitors changed. An empty snapshot is the OS in the
    /// middle of a hot-plug and changes nothing: the renderers stay until a snapshot with monitors arrives.
    fn apply_snapshot(&mut self, monitors: Vec<MonitorInfo>, ctx: &Ctx<'_>) -> Applied {
        if monitors.is_empty() {
            tracing::info!("the OS reports no monitor, the renderers stay as they are");
            return Applied::default();
        }
        log_monitors_identified(&monitors);
        let changed = diff(&self.monitors, &monitors);
        let actions = replace_all(&self.monitors, &monitors);
        let surfaces = actions
            .iter()
            .filter(|action| matches!(action, TopologyAction::SetRect(..)))
            .count();
        if !changed.is_empty() {
            tracing::info!(actions = changed.len(), "applying a monitor change");
        }
        // Every surface is put back after a display change, whether its monitor changed or not: explorer may have
        // moved the parent window the surface's position depends on (legacy parity).
        tracing::info!(changed = changed.len(), surfaces, "re-placing every surface");
        self.web_owner = web_profile_owner(&monitors).map(|monitor| monitor.id.clone());
        let applied = self.apply(actions, ctx);
        self.monitors = monitors;
        self.refresh_web_profiles();
        if !changed.is_empty() {
            // Unchanged monitors cannot change the master.
            self.elect(ctx);
        }
        applied
    }

    /// The profile the renderer of `monitor` asks for: the persistent one when it is the web profile owner.
    fn web_profile_for(&self, monitor: &str) -> ProfileKind {
        if self.web_owner.as_deref() == Some(monitor) {
            ProfileKind::Persistent
        } else {
            ProfileKind::Ephemeral
        }
    }

    /// Makes every stored assignment ask for the kind the current web profile owner implies, so the next start of each
    /// slot follows the primary monitor; a running renderer keeps its lease until it ends.
    fn refresh_web_profiles(&mut self) {
        let owner = self.web_owner.clone();
        let mut assignments = lock_assignments(&self.assignments);
        for assignment in assignments.values_mut() {
            assignment.web_profile = if owner.as_deref() == Some(assignment.monitor.as_str()) {
                ProfileKind::Persistent
            } else {
                ProfileKind::Ephemeral
            };
        }
    }

    /// Carries out `actions` (see [`replace_all`]).
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
        let web_profile = self.web_profile_for(&monitor.id);
        lock_assignments(&self.assignments).insert(slot, job.assignment(slot, monitor, web_profile));
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

    /// The surface of monitor `id` is put on `rect`, its monitor's rectangle after a display change (it may be the
    /// same one as before): its renderer keeps playing and re-places its surface.
    fn move_to(&mut self, id: &str, rect: RectPx, ctx: &Ctx<'_>) {
        let Some(slot) = self.slots.slot_of(id) else {
            return;
        };
        let changed = self
            .monitors
            .iter()
            .find(|monitor| monitor.id == id)
            .is_none_or(|monitor| monitor.rect != rect);
        let rect = [rect.left, rect.top, rect.right, rect.bottom];
        if let Some(assignment) = lock_assignments(&self.assignments).get_mut(&slot) {
            // A renderer started later, or restarted, uses the new rectangle.
            assignment.rect = rect;
        }
        let told = send_to(ctx.renderers, slot, &ToRenderer::SetRect(rect));
        tracing::info!(slot, monitor = %id, ?rect, told, changed, "re-placing its surface");
    }
}

/// How an accepted Use reaches the renderers that run (D-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "Restart is the fallback USE_TRANSITION is switched to if the maintainer sees a black frame on a switch"
)]
enum UseTransition {
    /// The running renderers load the new video in place; nothing restarts.
    LoadSource,
    /// The renderers are closed and new ones start with the new job.
    Restart,
}

/// The transition a Use takes: the renderers keep running and load the new video (D-26, RESEARCH assumption A5).
const USE_TRANSITION: UseTransition = UseTransition::LoadSource;

/// What decide_use needs to know about a Use request.
#[derive(Debug)]
struct UseInput<'a> {
    /// The folder the Portal named.
    folder: &'a str,
    /// What the theme gate made of the folder: the job to play, or why the theme is refused.
    gate: Result<Job, ThemeRejection>,
    /// The folder selected now.
    selected: &'a str,
    /// Whether renderers run.
    running: bool,
    /// The engine of the job the renderers play now, when the fleet has a job.
    running_engine: Option<EngineType>,
    /// What the fullscreen rule does to the wallpaper now.
    rule: RuleState,
    /// Whether the library is being moved.
    moving: bool,
}

/// What a Use request leads to.
#[derive(Debug)]
enum UseDecision {
    /// Nothing changes; the Portal is told why.
    Refuse(UseRefusal),
    /// The theme is the selected one and plays: nothing is reloaded.
    AlreadyApplied,
    /// Renderers run: they switch to this job.
    Load(Job),
    /// Renderers run on another engine than this job needs: they are closed and new ones start for it.
    Restart(Job),
    /// No renderer runs: the fleet starts for this job.
    Start(Job),
}

/// The refusal the Portal is told for a theme the gate rejected.
fn refusal_of(rejection: &ThemeRejection) -> UseRefusal {
    match rejection {
        ThemeRejection::NothingSelected | ThemeRejection::FolderMissing { .. } => UseRefusal::NotFound,
        ThemeRejection::TypeNotPlayable { .. } => UseRefusal::TypeNotPlayable,
        ThemeRejection::NeedsNewerApp { .. } => UseRefusal::NewerVersion,
        // No available engine for the type is not a damaged theme (UI-SPEC, Use-disabled reason 4).
        ThemeRejection::SourceInvalid { reason, .. } if reason.starts_with(engines::NO_ENGINE) => UseRefusal::NoEngine,
        ThemeRejection::InfoUnreadable { .. } | ThemeRejection::SourceInvalid { .. } => UseRefusal::Corrupt,
    }
}

/// Decides a Use request without touching anything (D-17, D-23, LIB-08): a refusal changes neither the selection nor
/// the wallpaper. The library move comes first, then the fullscreen rule, then the gate's verdict on the theme.
fn decide_use(input: &UseInput<'_>) -> UseDecision {
    if input.moving {
        return UseDecision::Refuse(UseRefusal::Moving);
    }
    match input.rule {
        RuleState::Paused => return UseDecision::Refuse(UseRefusal::RulePaused),
        RuleState::Closed => return UseDecision::Refuse(UseRefusal::RuleClosed),
        RuleState::None => {}
    }
    let job = match &input.gate {
        Ok(job) => job,
        Err(rejection) => return UseDecision::Refuse(refusal_of(rejection)),
    };
    if input.running && input.folder == input.selected {
        return UseDecision::AlreadyApplied;
    }
    if input.running {
        // A new renderer when the engine changes, and whenever either engine cannot load another file in place.
        if input
            .running_engine
            .is_some_and(|running| loads_in_place(running, job.engine))
        {
            UseDecision::Load(job.clone())
        } else {
            UseDecision::Restart(job.clone())
        }
    } else {
        UseDecision::Start(job.clone())
    }
}

/// What a Use did, for the main loop to fold into the tray and the timers.
struct UseDone {
    /// The answer for the Portal.
    outcome: UseOutcome,
    /// The new tray status, when it changed.
    status: Option<Status>,
    /// Whether the fullscreen timer has to start: the rule was off because no theme played, and is on now.
    arm_fullscreen: bool,
}

impl UseDone {
    fn refused(reason: UseRefusal) -> Self {
        Self {
            outcome: UseOutcome::Refused(reason),
            status: None,
            arm_fullscreen: false,
        }
    }

    fn applied(status: Option<Status>) -> Self {
        Self {
            outcome: UseOutcome::Applied,
            status,
            arm_fullscreen: false,
        }
    }

    fn failed(reason: String) -> Self {
        Self {
            outcome: UseOutcome::Failed { reason },
            status: None,
            arm_fullscreen: false,
        }
    }
}

/// Handles the Portal's Use: the theme gate decides (T-03-22), the selection is stored, and the renderers switch to
/// the new video (D-26) or, when none runs, start for it like a service start does. A refused Use changes nothing.
fn handle_use(
    folder: &str,
    host: &mut SettingsHost,
    fleet: &mut Fleet,
    legacy_root: Option<&std::path::Path>,
    platform: &dyn sucrose_platform::Platform,
    ctx: &Ctx<'_>,
) -> UseDone {
    let Some(root) = legacy_root else {
        return UseDone::refused(UseRefusal::Unavailable);
    };
    let (decision, kind) = {
        let settings = host.settings();
        let theme = check_selected(&settings.library_location(root), folder, &app_version(), PLAYABLE);
        let kind = theme.as_ref().ok().map(|theme| theme.info.kind);
        let gate = theme.and_then(|theme| {
            job_for(
                &theme,
                &settings.engine,
                settings.backgroundog.pause,
                settings.portal.theme_mode,
                &settings.general.user_agent,
                &fleet.engines,
            )
            .map_err(|reason| ThemeRejection::SourceInvalid {
                folder: folder.to_owned(),
                reason,
            })
        });
        let input = UseInput {
            folder,
            gate,
            selected: &settings.library.selected,
            running: fleet.job.is_some() && !ctx.supervisor.pids().is_empty(),
            running_engine: fleet.job.as_ref().map(|job| job.engine),
            rule: fleet.rule_state(),
            moving: fleet.library_move.is_some(),
        };
        (decide_use(&input), kind)
    };
    let (job, starting, transition) = match decision {
        UseDecision::Refuse(reason) => {
            tracing::info!(?reason, "use refused");
            return UseDone::refused(reason);
        }
        UseDecision::AlreadyApplied => {
            tracing::debug!("the theme is already in use, nothing is reloaded");
            return UseDone::applied(None);
        }
        UseDecision::Load(job) => (job, false, USE_TRANSITION),
        UseDecision::Restart(job) => (job, false, UseTransition::Restart),
        UseDecision::Start(job) => (job, true, USE_TRANSITION),
    };
    // Looked at before anything changes: a Use that cannot show anything is refused, not half applied.
    let monitors = if starting {
        let monitors = with_area(platform.monitors(), job.screen);
        if monitors.is_empty() {
            tracing::info!("use refused: the OS reports no monitor");
            return UseDone::refused(UseRefusal::Unavailable);
        }
        monitors
    } else {
        Vec::new()
    };
    if let Err(reason) = host.update(|settings| settings.library.selected = folder.to_owned()) {
        tracing::warn!(%reason, "the selection cannot be saved, nothing was changed");
        return UseDone::failed(reason);
    }
    let title = job.title.clone();
    let fallback = job.fallback.clone();
    if !starting {
        let reached = match transition {
            UseTransition::LoadSource => fleet.load_source(job, ctx.renderers),
            UseTransition::Restart => fleet.restart_with(job, ctx.renderers),
        };
        fleet.revive_stopped(ctx.supervisor);
        tracing::info!(?kind, ?transition, renderers = reached.len(), "theme used");
        return UseDone::applied(Some(Status::Starting { title, fallback }));
    }
    apply_gpu_preference(platform, host.settings().engine.gpu_preference);
    fleet.adopt_job(job);
    fleet.web_owner = web_profile_owner(&monitors).map(|monitor| monitor.id.clone());
    let applied = fleet.apply(diff(&[], &monitors), ctx);
    fleet.monitors = monitors;
    fleet.elect(ctx);
    let arm_fullscreen = fleet.fullscreen.is_none() && {
        let settings = host.settings();
        start_fullscreen(
            fleet,
            settings.backgroundog.fullscreen,
            action_scope(settings.backgroundog.full_screen_scope),
            settings.backgroundog.pause,
        )
    };
    tracing::info!(?kind, started = applied.started, failed = applied.failed, "theme used");
    if applied.started == 0 {
        let reason = "no renderer could be started for the theme".to_owned();
        return UseDone {
            outcome: UseOutcome::Failed { reason: reason.clone() },
            status: Some(Status::Failed {
                reason,
                hint: CHECK_THE_LOG.to_owned(),
            }),
            arm_fullscreen,
        };
    }
    UseDone {
        arm_fullscreen,
        ..UseDone::applied(Some(Status::Starting { title, fallback }))
    }
}

/// What the Portal is told the service is doing now.
fn current_runtime(host: &SettingsHost, fleet: &Fleet, supervisor: &Supervisor, tray_visible: bool) -> RuntimeState {
    let running = fleet.job.is_some() && !supervisor.pids().is_empty();
    runtime_state(
        host.settings(),
        running,
        fleet.rule_state(),
        fleet.library_move.is_some(),
        tray_visible,
        engines_running(fleet.job.as_ref(), &fleet.slots, &supervisor.slots()),
    )
}

/// The monitor and the engine of each wallpaper that runs: the slots with a running renderer, by monitor, on the engine
/// of the job (one job plays on every monitor, D-01). The Portal reads it to know whether a saved setting needs the
/// restart prompt.
fn engines_running(job: Option<&Job>, slots: &SlotMap, running: &[u8]) -> Vec<(String, EngineType)> {
    let Some(job) = job else {
        return Vec::new();
    };
    running
        .iter()
        .filter_map(|slot| slots.monitor_of(*slot).map(|monitor| (monitor.to_owned(), job.engine)))
        .collect()
}

/// Whether a wallpaper of `kind` on `engine` has developer tools to open: a Web, Url or YouTube wallpaper on a web engine.
fn has_developer_tools(kind: WallpaperType, engine: EngineType) -> bool {
    matches!(kind, WallpaperType::Web | WallpaperType::Url | WallpaperType::YouTube)
        && matches!(engine, EngineType::WebViewLive | EngineType::CefSharpLive)
}

/// The slot whose renderer Open DevTools goes to: the renderer that runs on `monitor` while the job is a wallpaper with
/// developer tools (see [`has_developer_tools`]). `None` for a monitor without a running renderer, another kind of
/// wallpaper or no wallpaper at all.
fn devtools_slot(
    job: Option<(WallpaperType, EngineType)>,
    slots: &SlotMap,
    running: &[u8],
    monitor: &str,
) -> Option<u8> {
    let (kind, engine) = job?;
    if !has_developer_tools(kind, engine) {
        return None;
    }
    slots.slot_of(monitor).filter(|slot| running.contains(slot))
}

/// Whether a restart asked for the engines `listed` touches the wallpaper that plays on `running`: only a wallpaper on
/// a listed engine restarts.
fn restart_affects(running: Option<EngineType>, listed: &[EngineType]) -> bool {
    running.is_some_and(|engine| listed.contains(&engine))
}

/// What the Portal is told when a restart is asked for while the library moves.
const RESTART_BUSY: &str = "Another change is still running. Try again in a moment.";

/// The restart of the Portal's prompt: when the wallpaper plays on one of `engines` its job is built again from the
/// settings as saved now (so the new certificate switch, arguments or hardware acceleration are read) and the renderers
/// restart on it, through the path of the engine change. Wallpapers on other engines and no wallpaper at all restart
/// nothing. Returns how many renderers were closed to come back.
fn restart_wallpapers(
    engines: &[EngineType],
    host: &SettingsHost,
    fleet: &mut Fleet,
    legacy_root: Option<&Path>,
    renderers: &SharedRenderers,
) -> Result<usize, String> {
    if !restart_affects(fleet.job.as_ref().map(|job| job.engine), engines) {
        return Ok(0);
    }
    if fleet.library_move.is_some() {
        return Err(RESTART_BUSY.to_owned());
    }
    let Some(root) = legacy_root else {
        return Err("the library folder cannot be found".to_owned());
    };
    let settings = host.settings();
    let job = plan_for(
        &settings.library_location(root),
        &settings.library.selected,
        &settings.engine,
        settings.backgroundog.pause,
        settings.portal.theme_mode,
        &settings.general.user_agent,
        &fleet.engines,
    )?;
    Ok(fleet.restart_with(job, renderers).len())
}

/// Sends the Portal `runtime` when it differs from the last one sent (and remembers it either way, so a Portal that
/// connects later is told the current state by its Welcome). `true` when a Runtime event was offered. The runtime
/// state is never written to Sucrose.json (D-23, ARCH-04).
fn publish_runtime(portals: &mut PortalRegistry, last: &mut Option<RuntimeState>, runtime: RuntimeState) -> bool {
    if last.as_ref() == Some(&runtime) {
        return false;
    }
    portals.send(PortalEvent::Runtime(runtime.clone()));
    *last = Some(runtime);
    true
}

/// Sends the Portal the engine report when it differs from the last one sent (D-11, ENG-06). `true` when it was offered.
fn publish_engines(portals: &mut PortalRegistry, last: &mut Option<EngineReportView>, view: EngineReportView) -> bool {
    if last.as_ref() == Some(&view) {
        return false;
    }
    portals.send(PortalEvent::Engines(view.clone()));
    *last = Some(view);
    true
}

/// Offers the Portal the current runtime state when it changed since the last offer.
fn announce_runtime(
    portals: &mut PortalRegistry,
    last: &mut Option<RuntimeState>,
    host: &SettingsHost,
    fleet: &Fleet,
    supervisor: &Supervisor,
    tray_visible: bool,
) {
    publish_runtime(portals, last, current_runtime(host, fleet, supervisor, tray_visible));
}

impl Fleet {
    /// What the Portal is told about the engines: their states, the types that can play and the fallbacks that run.
    fn engine_view(&self, warehouse: &Warehouse) -> EngineReportView {
        self.engines
            .view(PLAYABLE, &self.fallbacks, warehouse, engines::unix_now())
    }

    /// Whether a wallpaper is still wanted: a theme is selected and at least one monitor's renderer runs, is about to
    /// start, or waits for the fullscreen rule to bring it back. A renderer that ended for good does not count.
    fn wallpaper_wanted(&self) -> bool {
        if self.job.is_none() {
            return false;
        }
        let slots = self.slots.active_slots();
        slots.is_empty()
            || slots
                .iter()
                .any(|slot| !self.stopped.contains(slot) || self.rule_closed.is_closed(*slot))
    }

    /// What the fullscreen rule does to the wallpaper now, read from the rule itself: no second record to keep in step.
    fn rule_state(&self) -> RuleState {
        let Some(rule) = &self.fullscreen else {
            return RuleState::None;
        };
        if self.monitors.iter().any(|monitor| rule.closes(&monitor.id)) {
            RuleState::Closed
        } else if self.monitors.iter().any(|monitor| rule.pauses(&monitor.id)) {
            RuleState::Paused
        } else {
            RuleState::None
        }
    }

    /// Makes `job` what every renderer plays from now on: the shared assignments (so a renderer started or restarted
    /// later plays it) and the restart budgets. An explicit Use resets the budgets: a user who replaced a bad theme is
    /// not locked out by its old crashes (a renderer that keeps crashing is still stopped after 3 crashes in 60 s).
    fn adopt_job(&mut self, job: Job) {
        {
            let mut assignments = lock_assignments(&self.assignments);
            for assignment in assignments.values_mut() {
                assignment.source.clone_from(&job.source);
                assignment.settings = job.settings;
                assignment.kind = job.kind;
                assignment.theme_dir.clone_from(&job.theme_dir);
                // A restart reads the engine and the web options as saved now: the certificate switch, the switches and
                // hardware acceleration apply when the renderer starts again (D-17, WAPI-10).
                assignment.engine = job.engine;
                assignment.web.clone_from(&job.web);
            }
        }
        self.budgets.clear();
        self.fallbacks.set(job.kind, job.fallback.clone());
        self.job = Some(job);
    }

    /// Applies a live setting `message` (loop or shuffle) to what plays: the job and the shared assignments, so a renderer
    /// started or restarted later gets it too, and every connected renderer. It is no Use: nothing restarts and the
    /// restart budgets stay. Returns the slots that were told.
    fn apply_live(&mut self, message: &ToRenderer, renderers: &SharedRenderers) -> Vec<u8> {
        {
            let mut assignments = lock_assignments(&self.assignments);
            for assignment in assignments.values_mut() {
                put_live(&mut assignment.settings, &mut assignment.web, message);
            }
        }
        if let Some(job) = self.job.as_mut() {
            put_live(&mut job.settings, &mut job.web, message);
        }
        self.slots
            .active_slots()
            .into_iter()
            .filter(|slot| send_to(renderers, *slot, message))
            .collect()
    }

    /// Switches the running renderers to `job` without restarting them (D-26): the assignments are updated first,
    /// then every connected renderer is sent LoadSource. A renderer that has not connected yet reads the new
    /// assignment at its Hello. Returns the slots that were told.
    fn load_source(&mut self, job: Job, renderers: &SharedRenderers) -> Vec<u8> {
        let message = ToRenderer::LoadSource {
            source: job.source.clone(),
            settings: job.settings,
        };
        self.adopt_job(job);
        self.slots
            .active_slots()
            .into_iter()
            .filter(|slot| send_to(renderers, *slot, &message))
            .collect()
    }

    /// The Restart transition: every connected renderer is closed and comes back once its old one has ended (the
    /// path of the fullscreen rule's Close), playing `job`. Returns the slots that were closed.
    fn restart_with(&mut self, job: Job, renderers: &SharedRenderers) -> Vec<u8> {
        self.adopt_job(job);
        let mut closed = Vec::new();
        for slot in self.slots.active_slots() {
            if !lock(renderers).connections.contains_key(&slot) {
                continue;
            }
            self.rule_closed.close(slot, false);
            self.rule_closed.want_back(slot);
            if send_to(renderers, slot, &ToRenderer::Close) {
                closed.push(slot);
            }
        }
        closed
    }

    /// Starts the renderers that had given up (budget spent, or ended for good) for the theme just selected: an
    /// explicit Use is a new start for them too.
    fn revive_stopped(&mut self, supervisor: &Arc<Supervisor>) {
        let running = supervisor.slots();
        let mut slots: Vec<u8> = self.stopped.iter().copied().collect();
        slots.sort_unstable();
        for slot in slots {
            if running.contains(&slot) || self.slots.monitor_of(slot).is_none() || self.rule_closed.is_closed(slot) {
                continue;
            }
            self.stopped.remove(&slot);
            match supervisor.spawn(slot) {
                Ok(()) => tracing::info!(slot, "a renderer that had stopped starts again for the new theme"),
                Err(error) => {
                    tracing::warn!(%error, slot, "cannot start a stopped renderer for the new theme");
                    self.stopped.insert(slot);
                }
            }
        }
    }
}

/// Opens the settings file (migrating the legacy files read-only the first time, D-03), logs what the service will
/// use and returns what they lead to and the settings host that owns the file.
fn plan_wallpapers(engines: &EngineReport) -> (Plan, SettingsHost) {
    let (Some(root), Some(legacy_setting)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the settings cannot be found");
        let plan = Plan {
            job: None,
            status: Status::NoWallpaper {
                reason: "the OS reports no config folder, so the settings cannot be read".to_owned(),
            },
            app_visible: true,
            selected_exists: false,
            fullscreen: FullScreenAction::default(),
            scope: ActionScope::AllMonitors,
            pause: PauseKind::default(),
            gpu: GpuPreference::default(),
        };
        return (plan, SettingsHost::unsaved(Settings::default()));
    };
    let host = load_settings(&root, &legacy_setting);
    (plan_from(host.settings(), &root, engines), host)
}

/// Opens the settings host. When the folder cannot be opened or written the legacy files are still migrated, in
/// memory only, so the wallpaper plays.
fn load_settings(root: &std::path::Path, legacy_setting: &std::path::Path) -> SettingsHost {
    let Some(dir) = setting_dir() else {
        tracing::error!("the OS reports no config folder; using the legacy settings without saving");
        let host = SettingsHost::unsaved(migrate(legacy_setting, root));
        log_settings(host.settings(), &"Unsaved");
        return host;
    };
    let (host, outcome) = SettingsHost::open(&dir, Some((legacy_setting, root)));
    if host.persists() {
        log_settings(host.settings(), &outcome);
    } else {
        log_settings(host.settings(), &"Unsaved");
    }
    host
}

/// Values only: no paths and no theme names (D-08, D-09). The legacy DisplayScreenType and ScreenDevice are not read:
/// every monitor plays the same video (D-01).
fn log_settings(settings: &Settings, outcome: &dyn std::fmt::Debug) {
    tracing::info!(
        outcome = ?outcome,
        volume = settings.engine.volume,
        looping = settings.engine.looping,
        stretch = ?settings.engine.stretch,
        hardware_acceleration = settings.engine.hardware_acceleration,
        screen = ?settings.engine.screen,
        gpu_preference = ?settings.engine.gpu_preference,
        fullscreen = ?settings.backgroundog.fullscreen,
        pause = ?settings.backgroundog.pause,
        app_visible = settings.general.app_visible,
        "settings loaded"
    );
}

/// What the settings and the selected theme lead to; `legacy_root` is the C# data folder, the default library location.
fn plan_from(settings: &Settings, legacy_root: &std::path::Path, engines: &EngineReport) -> Plan {
    let location = settings.library_location(legacy_root);
    let (job, status) = match plan_for(
        &location,
        &settings.library.selected,
        &settings.engine,
        settings.backgroundog.pause,
        settings.portal.theme_mode,
        &settings.general.user_agent,
        engines,
    ) {
        Ok(job) => {
            let status = Status::Starting {
                title: job.title.clone(),
                fallback: job.fallback.clone(),
            };
            (Some(job), status)
        }
        Err(reason) => (None, Status::NoWallpaper { reason }),
    };
    Plan {
        job,
        status,
        app_visible: settings.general.app_visible,
        selected_exists: !settings.library.selected.is_empty() && location.join(&settings.library.selected).is_dir(),
        fullscreen: settings.backgroundog.fullscreen,
        scope: action_scope(settings.backgroundog.full_screen_scope),
        pause: settings.backgroundog.pause,
        gpu: settings.engine.gpu_preference,
    }
}

/// The monitors the fullscreen rule acts on for `Backgroundog.FullScreenScope` (Phase 2 D-05).
fn action_scope(scope: FullScreenScope) -> ActionScope {
    match scope {
        FullScreenScope::AllMonitors => ActionScope::AllMonitors,
        FullScreenScope::AffectedMonitor => ActionScope::AffectedMonitor,
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

/// Runs the gate (D-13) on the selected theme and, when it passes, builds what every renderer plays: the engine
/// settings, the pause rule, the theme mode and `General.UserAgent` (`user_agent`) are read here. The error is one
/// English sentence for the log and the tooltip: the gate's own reason.
fn plan_for(
    location: &std::path::Path,
    selected: &str,
    engine: &Engine,
    pause: PauseKind,
    theme_mode: ThemeMode,
    user_agent: &str,
    engines: &EngineReport,
) -> Result<Job, String> {
    let theme = check_selected(location, selected, &app_version(), PLAYABLE)
        .map_err(|rejection| printable(&rejection.to_string()))?;
    // Debug formatting for the text a theme or the settings supply: control characters show as escapes.
    tracing::info!(theme = ?selected, kind = ?theme.info.kind, source = ?theme.info.source, "selected theme");
    job_for(&theme, engine, pause, theme_mode, user_agent, engines)
}

/// The wallpaper type whose engine `change` sets, `None` for every other setting.
fn engine_change_kind(change: &SettingChange) -> Option<WallpaperType> {
    match change {
        SettingChange::GifEngine(_) => Some(WallpaperType::Gif),
        SettingChange::UrlEngine(_) => Some(WallpaperType::Url),
        SettingChange::WebEngine(_) => Some(WallpaperType::Web),
        SettingChange::VideoEngine(_) => Some(WallpaperType::Video),
        SettingChange::YouTubeEngine(_) => Some(WallpaperType::YouTube),
        SettingChange::ApplicationEngine(_) => Some(WallpaperType::Application),
        _ => None,
    }
}

/// The message a saved setting sends to the running renderers at once, `None` for a setting that is read when the next
/// wallpaper starts (PLAY-01): Loop Mode and Shuffle Mode reach what plays, while Hardware Acceleration and the
/// interaction settings are read by the next renderer that starts.
fn live_message(change: &SettingChange) -> Option<ToRenderer> {
    match change {
        SettingChange::WallpaperLoop(looping) => Some(ToRenderer::SetLoop(*looping)),
        SettingChange::WallpaperShuffle(shuffle) => Some(ToRenderer::SetShuffle(*shuffle)),
        _ => None,
    }
}

/// Puts the value of a live setting `message` (loop or shuffle) into the playback settings and web options a job or an
/// assignment carries; every other message changes nothing.
fn put_live(settings: &mut PlaybackSettings, web: &mut WebOptions, message: &ToRenderer) {
    match message {
        ToRenderer::SetLoop(looping) => settings.looping = *looping,
        ToRenderer::SetShuffle(shuffle) => web.shuffle = *shuffle,
        _ => {}
    }
}

/// Whether a changed engine of type `changed` restarts the wallpaper that plays now (of type `running`, if any): only a
/// wallpaper of the changed type does (T-04-32).
fn engine_restart_wanted(running: Option<WallpaperType>, changed: WallpaperType) -> bool {
    running == Some(changed)
}

/// An engine of type `kind` was just saved: when a wallpaper of that type plays, its renderers restart on the engine now
/// chosen, with no prompt (UI-SPEC Engine select). The job is built again from the selected theme, so the engine and
/// its fallback follow the new setting. A wallpaper of another type is left alone. `true` when the job was replaced.
fn restart_for_engine(
    kind: WallpaperType,
    host: &SettingsHost,
    fleet: &mut Fleet,
    legacy_root: Option<&Path>,
    renderers: &SharedRenderers,
) -> bool {
    if !engine_restart_wanted(fleet.job.as_ref().map(|job| job.kind), kind) {
        return false;
    }
    let Some(root) = legacy_root else {
        return false;
    };
    let settings = host.settings();
    let job = match plan_for(
        &settings.library_location(root),
        &settings.library.selected,
        &settings.engine,
        settings.backgroundog.pause,
        settings.portal.theme_mode,
        &settings.general.user_agent,
        &fleet.engines,
    ) {
        Ok(job) => job,
        Err(reason) => {
            // The setting is saved; what plays keeps playing until a start can use the new choice.
            tracing::warn!(?kind, %reason, "the wallpaper keeps its engine, the new choice cannot be used now");
            return false;
        }
    };
    let engine = job.engine;
    let closed = fleet.restart_with(job, renderers);
    tracing::info!(
        ?kind,
        ?engine,
        renderers = closed.len(),
        "engine changed, the wallpaper restarts"
    );
    true
}

/// The engine the Engine section configures for a wallpaper type.
fn configured_engine(engine: &Engine, kind: WallpaperType) -> EngineType {
    match kind {
        WallpaperType::Gif => engine.gif,
        WallpaperType::Url => engine.url,
        WallpaperType::Web => engine.web,
        WallpaperType::Video => engine.video,
        WallpaperType::YouTube => engine.youtube,
        WallpaperType::Application => engine.application,
    }
}

/// What every renderer plays for a theme that passed the gate, with the playback settings of Engine.json. The engine
/// is chosen here from the detected engines, with its fallback (D-11, ENG-07): the configured one when it is available,
/// else the first that is. The setting itself is never changed. `user_agent` is `General.UserAgent`; only a web engine
/// gets it (see [`web_user_agent`]).
fn job_for(
    theme: &SelectedTheme,
    engine: &Engine,
    pause: PauseKind,
    theme_mode: ThemeMode,
    user_agent: &str,
    engines: &EngineReport,
) -> Result<Job, String> {
    let source = match &theme.source {
        ThemeSource::File(path) => path
            .to_str()
            .ok_or_else(|| "the Source path is not valid UTF-8".to_owned())?
            .to_owned(),
        ThemeSource::Address(address) => address.clone(),
    };
    let theme_dir = std::path::absolute(&theme.dir)
        .map_err(|_| "the theme folder path cannot be made absolute".to_owned())?
        .to_str()
        .ok_or_else(|| "the theme folder path is not valid UTF-8".to_owned())?
        .to_owned();
    let kind = theme.info.kind;
    let configured = configured_engine(engine, kind);
    let picked = engines::pick(engines, kind, configured)?;
    let chosen = picked.engine;
    tracing::info!(
        kind = ?kind,
        configured = ?configured,
        engine = ?chosen,
        fell_back = picked.fallback.is_some(),
        "engine chosen"
    );
    Ok(Job {
        title: printable(&theme.info.title),
        source,
        settings: playback_settings(engine, pause),
        screen: engine.screen,
        kind,
        engine: chosen.to_legacy(),
        fallback: picked.fallback,
        theme_dir,
        web: WebOptions {
            theme_type: legacy_theme_type(theme_mode).to_owned(),
            user_agent: web_user_agent(chosen, user_agent),
            ..web_options(engine, chosen)
        },
        input: input_settings(engine),
    })
}

/// The legacy `ThemeType` name the compatible calls `ThemeType` and `SystemTheme` carry for the migrated theme mode.
/// The legacy app stored Dark or Light and, with nothing stored, asked the OS for the app theme through a package that
/// is not part of this repository; reading the OS theme is not done yet, so `System` is the legacy default name
/// `Dark` until it is (docs/FEATURES.md).
fn legacy_theme_type(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Light",
        ThemeMode::Dark | ThemeMode::System => "Dark",
    }
}

/// The web engine options the Engine section leads to: certificates are validated unless the user allowed invalid ones,
/// the switches are the user's for the chosen web engine (an empty list means the legacy defaults, as in the legacy
/// app; D-17), and the developer settings follow `DeveloperMode` and `DeveloperPort` (D-13). Read when the job is
/// made, so a change saved while a renderer runs applies at its next start (WAPI-10). `Engine.WebViewSmartScreen` goes to
/// every engine's options and only the WebView2 renderer reads it (D-041), as with the certificate setting.
fn web_options(engine: &Engine, chosen: EngineId) -> WebOptions {
    let arguments = match chosen {
        EngineId::Cef if engine.cef_arguments.is_empty() => args::from_cef_map(&args::legacy_defaults_cef()),
        EngineId::Cef => args::from_cef_map(&engine.cef_arguments),
        EngineId::WebView2 if engine.web_arguments.is_empty() => args::from_web_list(&args::legacy_defaults_web()),
        EngineId::WebView2 => args::from_web_list(&engine.web_arguments),
        _ => Vec::new(),
    };
    WebOptions {
        allow_invalid_certificates: engine.allow_invalid_certificates,
        arguments,
        hardware_acceleration: engine.hardware_acceleration,
        developer_mode: engine.developer_mode,
        developer_port: engine.developer_port,
        shuffle: engine.wallpaper_shuffle,
        theme_type: WebOptions::default().theme_type,
        // `job_for` fills it from General.UserAgent, as it does `theme_type`.
        user_agent: String::new(),
        webview_smartscreen: engine.webview_smartscreen,
    }
}

/// The user agent a job hands to its engine, from `General.UserAgent` (legacy: `CefSettings.UserAgent` and
/// `CoreWebView2.Settings.UserAgent`). Only the two web engines get one: the value as stored, never trimmed. An empty
/// or blank value, or one with a control character (CR, LF, NUL, tab), gives an empty string, which both engines read
/// as their own user agent; the control-character case warns with the length only, never the text (T-uov-01, T-uov-02).
fn web_user_agent(chosen: EngineId, configured: &str) -> String {
    if !matches!(chosen, EngineId::Cef | EngineId::WebView2) || configured.trim().is_empty() {
        return String::new();
    }
    if configured.chars().any(char::is_control) {
        tracing::warn!(
            len = configured.len(),
            "the user agent has a control character, the web engines keep their own"
        );
        return String::new();
    }
    configured.to_owned()
}

/// The input forwarding settings the Engine section leads to (WAPI-07): read when the job is made, so a change saved
/// while a renderer runs applies at its next start.
fn input_settings(engine: &Engine) -> InputSettings {
    InputSettings {
        input_type: engine.input_type,
        desktop_only: engine.input_desktop,
    }
}

/// The player settings the Engine section leads to.
fn playback_settings(engine: &Engine, pause: PauseKind) -> PlaybackSettings {
    PlaybackSettings {
        volume: engine.volume,
        looping: engine.looping,
        stretch: engine.stretch,
        hardware_decoding: engine.hardware_acceleration,
        pause,
    }
}

#[cfg(test)]
mod tests {
    use sucrose_core::{
        settings::{InputModuleType, InputType},
        theme::ThemeRejection,
    };

    use super::*;
    use crate::supervisor::ENGINE_MISSING;
    use sucrose_engine::Availability;

    /// A report in which every engine is installed, so a test is not at the mercy of the machine it runs on.
    fn installed_engines() -> EngineReport {
        EngineReport::from_fn(|_| Availability::Installed { version: None })
    }

    fn fallback_note() -> FallbackNote {
        FallbackNote {
            kind: WallpaperType::Web,
            configured: EngineId::WebView2,
            playing: EngineId::Cef,
            reason: "this build of Sucrose does not play this wallpaper with WebView2".to_owned(),
        }
    }

    #[test]
    fn the_tooltip_of_a_fallback_names_the_engine_that_plays_and_the_one_that_is_missing() {
        let playing = Status::Playing {
            title: "Cyber Samurai".to_owned(),
            fallback: Some(fallback_note()),
        };

        assert_eq!(playing.tooltip(), "Playing with CEF (WebView2 isn't available)");
        assert!(playing.tooltip().chars().count() <= 127);
        assert_eq!(playing.text(), "Sucrose — playing Cyber Samurai");
    }

    #[test]
    fn a_status_without_a_fallback_keeps_its_own_tooltip() {
        assert_eq!(starting().tooltip(), APP_LONG_NAME);
    }

    #[test]
    fn the_fallback_survives_the_start_the_first_frame_and_a_restart() {
        let starting = Status::Starting {
            title: "Cyber Samurai".to_owned(),
            fallback: Some(fallback_note()),
        };

        let playing = starting.after(&ToService::Playing, None).expect("a status change");
        let restarting = playing.restarting(None).expect("a status change");

        assert_eq!(playing.tooltip(), restarting.tooltip());
        assert!(restarting.tooltip().starts_with("Playing with CEF"));
    }

    #[test]
    fn a_fleet_remembers_the_fallback_of_its_job_for_this_run_only() {
        let mut fleet = fleet_with_a_job();
        assert!(fleet.fallbacks.notes().is_empty());
        let job = Job {
            kind: WallpaperType::Web,
            fallback: Some(fallback_note()),
            ..use_job("C:/themes/page/index.html")
        };

        fleet.adopt_job(job);
        assert_eq!(fleet.fallbacks.notes(), [fallback_note()]);
        fleet.adopt_job(Job {
            kind: WallpaperType::Web,
            ..use_job("C:/themes/page/index.html")
        });

        assert!(fleet.fallbacks.notes().is_empty());
    }

    #[test]
    fn an_engine_setting_names_the_wallpaper_type_it_is_for() {
        let cases = [
            (SettingChange::GifEngine(EngineType::MpvPlayerLive), WallpaperType::Gif),
            (SettingChange::UrlEngine(EngineType::CefSharpLive), WallpaperType::Url),
            (SettingChange::WebEngine(EngineType::CefSharpLive), WallpaperType::Web),
            (
                SettingChange::VideoEngine(EngineType::MpvPlayerLive),
                WallpaperType::Video,
            ),
            (
                SettingChange::YouTubeEngine(EngineType::CefSharpLive),
                WallpaperType::YouTube,
            ),
            (
                SettingChange::ApplicationEngine(EngineType::AuroraLive),
                WallpaperType::Application,
            ),
        ];
        for (change, kind) in cases {
            assert_eq!(engine_change_kind(&change), Some(kind), "{change:?}");
        }
        assert_eq!(engine_change_kind(&SettingChange::AppVisible(true)), None);
        assert_eq!(engine_change_kind(&SettingChange::LibraryPagination(5)), None);
    }

    #[test]
    fn only_a_wallpaper_of_the_changed_type_restarts_on_an_engine_change() {
        assert!(engine_restart_wanted(Some(WallpaperType::Web), WallpaperType::Web));
        assert!(!engine_restart_wanted(Some(WallpaperType::Video), WallpaperType::Web));
        assert!(!engine_restart_wanted(None, WallpaperType::Web));
    }

    #[test]
    fn a_use_without_an_engine_is_refused_for_that_reason_and_not_as_a_damaged_theme() {
        let rejection = ThemeRejection::SourceInvalid {
            folder: "new".to_owned(),
            reason: format!("{} {}", engines::NO_ENGINE, "The files of mpv are missing."),
        };

        assert_eq!(refusal_of(&rejection), UseRefusal::NoEngine);
        let damaged = ThemeRejection::SourceInvalid {
            folder: "new".to_owned(),
            reason: "the Source path is not valid UTF-8".to_owned(),
        };
        assert_eq!(refusal_of(&damaged), UseRefusal::Corrupt);
    }

    #[test]
    fn the_compatible_calls_get_the_shuffle_setting_and_the_legacy_theme_name() {
        let mut engine = Engine::default();
        assert!(
            web_options(&engine, EngineId::Cef).shuffle,
            "WallpaperShuffle defaults to on"
        );

        engine.wallpaper_shuffle = false;

        assert!(!web_options(&engine, EngineId::Cef).shuffle);
        assert_eq!(legacy_theme_type(ThemeMode::Light), "Light");
        assert_eq!(legacy_theme_type(ThemeMode::Dark), "Dark");
        assert_eq!(
            legacy_theme_type(ThemeMode::System),
            "Dark",
            "the OS theme is not read yet, so System is the legacy default name"
        );
    }

    #[test]
    fn web_jobs_validate_certificates_unless_the_engine_section_allows_invalid_ones() {
        let mut engine = Engine::default();
        assert!(
            !web_options(&engine, EngineId::Cef).allow_invalid_certificates,
            "D-12: validation is the default"
        );

        engine.allow_invalid_certificates = true;

        assert!(web_options(&engine, EngineId::Cef).allow_invalid_certificates);
    }

    #[test]
    fn web_jobs_carry_the_webview2_smartscreen_setting() {
        let mut engine = Engine::default();
        assert!(
            !web_options(&engine, EngineId::WebView2).webview_smartscreen,
            "D-041: SmartScreen is off by default"
        );

        engine.webview_smartscreen = true;

        assert!(web_options(&engine, EngineId::WebView2).webview_smartscreen);
    }

    #[test]
    fn web_jobs_carry_the_users_cef_switches_or_the_legacy_defaults_when_there_are_none() {
        let defaults = web_options(&Engine::default(), EngineId::Cef);
        let engine = Engine {
            cef_arguments: std::collections::BTreeMap::from([("no-sandbox".to_owned(), String::new())]),
            ..Engine::default()
        };
        let custom = web_options(&engine, EngineId::Cef);

        assert_eq!(
            args::to_cef_map(&defaults.arguments),
            args::legacy_defaults_cef(),
            "an empty dictionary means the legacy defaults"
        );
        assert_eq!(custom.arguments, [args::Switch::flag("no-sandbox")]);
    }

    #[test]
    fn web_jobs_carry_the_web_list_for_webview_and_nothing_for_a_player() {
        let engine = Engine {
            web_arguments: vec!["--no-sandbox".to_owned()],
            ..Engine::default()
        };

        assert_eq!(
            web_options(&engine, EngineId::WebView2).arguments,
            [args::Switch::flag("no-sandbox")]
        );
        assert!(web_options(&engine, EngineId::Mpv).arguments.is_empty());
        assert_eq!(
            args::to_web_list(&web_options(&Engine::default(), EngineId::WebView2).arguments),
            args::legacy_defaults_web()
        );
    }

    #[test]
    fn web_jobs_carry_the_developer_settings_and_hardware_acceleration() {
        let engine = Engine {
            developer_mode: true,
            developer_port: 9222,
            hardware_acceleration: false,
            ..Engine::default()
        };

        let options = web_options(&engine, EngineId::Cef);

        assert!(options.developer_mode);
        assert_eq!(options.developer_port, 9222);
        assert!(!options.hardware_acceleration);
        assert!(
            !web_options(&Engine::default(), EngineId::Cef).developer_mode,
            "off by default"
        );
    }

    #[test]
    fn web_jobs_carry_the_general_user_agent_and_player_jobs_none() {
        use sucrose_config::model::DEFAULT_USER_AGENT;

        assert_eq!(web_user_agent(EngineId::Cef, DEFAULT_USER_AGENT), DEFAULT_USER_AGENT);
        assert_eq!(
            web_user_agent(EngineId::WebView2, "My Agent 1.0"),
            "My Agent 1.0",
            "exactly as stored, not trimmed"
        );
        assert_eq!(web_user_agent(EngineId::WebView2, " padded "), " padded ");
        for player in [
            EngineId::Mpv,
            EngineId::Vlc,
            EngineId::BuiltInGif,
            EngineId::BuiltInVideo,
            EngineId::AppEmbedder,
        ] {
            assert_eq!(
                web_user_agent(player, DEFAULT_USER_AGENT),
                "",
                "{player:?} has no user agent"
            );
        }
        assert_eq!(
            web_options(&Engine::default(), EngineId::Cef).user_agent,
            "",
            "job_for fills it"
        );
    }

    #[test]
    fn a_blank_user_agent_or_one_with_a_control_character_leaves_the_engines_own() {
        for blank in ["", "   ", " \t "] {
            assert_eq!(web_user_agent(EngineId::Cef, blank), "", "{blank:?}");
            assert_eq!(web_user_agent(EngineId::WebView2, blank), "", "{blank:?}");
        }
        for bad in ["Bad\r\nX-Injected: 1", "A\u{0}B", "tab\tin"] {
            assert_eq!(web_user_agent(EngineId::Cef, bad), "", "{bad:?}");
            assert_eq!(web_user_agent(EngineId::WebView2, bad), "", "{bad:?}");
        }
    }

    #[test]
    fn the_input_settings_of_a_job_follow_the_engine_section() {
        let engine = Engine {
            input_type: sucrose_core::settings::InputType::OnlyMouse,
            input_desktop: true,
            ..Engine::default()
        };

        assert_eq!(
            input_settings(&engine),
            InputSettings {
                input_type: sucrose_core::settings::InputType::OnlyMouse,
                desktop_only: true
            }
        );
        assert_eq!(
            input_settings(&Engine::default()),
            InputSettings::default(),
            "the legacy defaults: mouse and keyboard, desktop or not"
        );
    }

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

    /// The holder of a slot that belongs to a connected monitor.
    fn holder(link_pid: Option<u32>, running: &HashMap<u32, u8>) -> SlotState<'_> {
        SlotState {
            link_pid,
            running,
            assigned: true,
        }
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
        let effect = handle_report(&status, &holder(None, &HashMap::new()), 0, 10, &failed_report(), TITLE)
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
            handle_report(&starting(), &holder(link_pid, running), 0, 7, message, TITLE)
        };

        let playing = counts(Some(7), &none, &ToService::Playing).expect("the current renderer");
        assert!(playing.first_frame);
        assert_eq!(
            playing.status,
            Some(Status::Playing {
                title: "Cyber Samurai".to_owned(),
                fallback: None,
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
            handle_report(
                &starting(),
                &holder(None, &HashMap::new()),
                0,
                7,
                &ToService::Playing,
                TITLE
            ),
            None
        );
    }

    #[test]
    fn a_failed_of_a_replaced_renderer_is_dropped_whether_or_not_the_replacement_connected() {
        let running = HashMap::from([(20, 0)]);

        assert_eq!(
            handle_report(&starting(), &holder(None, &running), 0, 10, &failed_report(), TITLE),
            None,
            "pid 20 runs for the slot but has not connected yet"
        );
        assert_eq!(
            handle_report(&starting(), &holder(Some(20), &running), 0, 10, &failed_report(), TITLE),
            None,
            "pid 20 is connected"
        );
        assert!(
            handle_report(&starting(), &holder(None, &running), 1, 10, &failed_report(), TITLE).is_some(),
            "the replacement runs for another slot"
        );
    }

    #[test]
    fn a_late_failed_of_a_removed_monitor_is_dropped() {
        // The monitor was unplugged: its slot is retired, then the renderer's exit and a queued Failed arrive. The
        // tooltip of the monitors that still play must not switch to "no wallpaper" (WR-19).
        let none = HashMap::new();
        let removed = SlotState {
            link_pid: None,
            running: &none,
            assigned: false,
        };

        assert_eq!(
            handle_report(&starting(), &removed, 0, 10, &failed_report(), TITLE),
            None
        );
    }

    #[test]
    fn a_report_of_a_renderer_whose_slot_is_not_assigned_is_dropped_even_while_connected() {
        // The monitor is retired, or the fullscreen rule closed the renderer, and the renderer is still connected: it
        // changes neither the status nor the master (WR-19).
        let none = HashMap::new();
        let connected = SlotState {
            link_pid: Some(10),
            running: &none,
            assigned: false,
        };

        assert_eq!(
            handle_report(&starting(), &connected, 0, 10, &failed_report(), TITLE),
            None
        );
        assert_eq!(
            handle_report(&starting(), &connected, 0, 10, &ToService::Playing, TITLE),
            None
        );
    }

    #[test]
    fn a_late_failed_of_an_assigned_slot_still_counts() {
        // A renderer of a connected monitor ended with nobody replacing it: its last words still set the reason, so the
        // WR-19 drop cannot hide a real failure (WR-18).
        let none = HashMap::new();
        let ended = SlotState {
            link_pid: None,
            running: &none,
            assigned: true,
        };

        let effect = handle_report(&starting(), &ended, 0, 10, &failed_report(), TITLE).expect("it counts");

        assert_eq!(effect.status, Some(failed_reported()));
    }

    #[test]
    fn slot_assigned_follows_the_monitor_and_the_fullscreen_rule() {
        let mut fleet = fleet_with_a_job();
        let slot = fleet.slots.assign("DISPLAY1").expect("a free slot");
        assert!(fleet.slot_assigned(slot), "a connected monitor's slot");

        fleet.rule_closed.close(slot, false);
        assert!(!fleet.slot_assigned(slot), "the fullscreen rule closed its renderer");

        fleet.rule_closed.forget(slot);
        assert!(fleet.slot_assigned(slot), "the renderer is back");

        assert_eq!(fleet.slots.retire("DISPLAY1"), Some(slot));
        assert!(!fleet.slot_assigned(slot), "the monitor was removed");

        assert!(fleet.slots.release(slot));
        assert!(!fleet.slot_assigned(slot), "the slot is free");
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

        let effect =
            handle_report(&status, &holder(None, &HashMap::new()), 0, 10, &failed_report(), TITLE).expect("it counts");

        assert_eq!(
            effect.status, None,
            "the status keeps saying that the monitor gets no more renderers"
        );
        let live = handle_report(
            &status,
            &holder(Some(10), &HashMap::new()),
            0,
            10,
            &failed_report(),
            TITLE,
        )
        .expect("it counts");
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
            device_path: None,
            edid: None,
            friendly: None,
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

    const TITLE: Option<Current<'static>> = Some(Current {
        title: "Cyber Samurai",
        fallback: None,
    });

    fn starting() -> Status {
        Status::Starting {
            title: "Cyber Samurai".to_owned(),
            fallback: None,
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
    fn a_no_wallpaper_reason_with_control_characters_is_printable() {
        let forged = "Rain\n2026 ERROR forged\u{1b}[31m";
        let status = Status::NoWallpaper {
            reason: forged.to_owned(),
        };

        for sentence in [status.text(), status.tooltip()] {
            assert!(!sentence.chars().any(char::is_control), "{sentence:?}");
        }
        assert_eq!(
            status.text(),
            "Sucrose — no wallpaper: Rain\u{fffd}2026 ERROR forged\u{fffd}[31m"
        );
    }

    #[test]
    fn every_free_text_field_of_a_status_is_printable() {
        let dirty = "x\ny\u{1b}z";
        let statuses = [
            Status::Playing {
                title: dirty.to_owned(),
                fallback: None,
            },
            Status::Starting {
                title: dirty.to_owned(),
                fallback: None,
            },
            Status::Failed {
                reason: dirty.to_owned(),
                hint: dirty.to_owned(),
            },
            Status::RendererStopped {
                monitor: dirty.to_owned(),
                reason: dirty.to_owned(),
            },
        ];

        for status in statuses {
            for sentence in [status.text(), status.tooltip()] {
                assert!(!sentence.chars().any(char::is_control), "{status:?}: {sentence:?}");
            }
        }
        let failed = Status::Failed {
            reason: dirty.to_owned(),
            hint: dirty.to_owned(),
        };
        assert_eq!(
            failed.text(),
            "Sucrose — no wallpaper: x\u{fffd}y\u{fffd}z x\u{fffd}y\u{fffd}z"
        );
    }

    #[test]
    fn the_monitor_wait_is_still_recognized_after_printable_text() {
        let waiting = Status::NoWallpaper {
            reason: NO_MONITOR.to_owned(),
        };

        assert!(waiting.is_waiting_for_a_monitor());
        assert_eq!(
            printable(NO_MONITOR),
            NO_MONITOR,
            "the reason has no control character to lose"
        );
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
        let settings = migrate(&setting_dir, &root);

        let selected = check_selected(
            &settings.library_location(&root),
            &settings.library.selected,
            &app_version(),
            PLAYABLE,
        );

        let theme = selected.unwrap_or_else(|rejection| panic!("the real selected theme is refused: {rejection}"));
        assert!(matches!(&theme.source, ThemeSource::File(path) if path.is_file()));
    }

    fn fleet_with_a_job() -> Fleet {
        let job = Job {
            title: "test".to_owned(),
            source: "C:/wallpaper.mp4".to_owned(),
            settings: PlaybackSettings::default(),
            screen: ScreenArea::default(),
            kind: WallpaperType::Video,
            engine: EngineType::MpvPlayerLive,
            fallback: None,
            theme_dir: "C:/".to_owned(),
            web: WebOptions::default(),
            input: InputSettings::default(),
        };
        Fleet::new(Some(job), SharedAssignments::default(), installed_engines())
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
    fn no_job_means_no_wallpaper_is_wanted() {
        let mut fleet = fleet_of(&[("a", true)], &[]);
        fleet.job = None;
        assert!(!fleet.wallpaper_wanted());
    }

    #[test]
    fn a_job_with_a_renderer_that_runs_or_will_start_is_wanted() {
        assert!(fleet_with_a_job().wallpaper_wanted(), "no monitor yet, one may come");
        assert!(fleet_of(&[("a", true)], &[]).wallpaper_wanted());
    }

    #[test]
    fn a_job_whose_renderers_all_gave_up_is_not_wanted_but_one_alive_keeps_it() {
        let mut fleet = fleet_of(&[("a", true), ("b", false)], &[]);
        let a = fleet.slots.slot_of("a").expect("a slot");
        let b = fleet.slots.slot_of("b").expect("a slot");
        fleet.stopped.insert(a);
        assert!(fleet.wallpaper_wanted(), "the other monitor still plays");
        fleet.stopped.insert(b);
        assert!(!fleet.wallpaper_wanted(), "every renderer ended for good");
    }

    #[test]
    fn a_renderer_closed_by_the_fullscreen_rule_is_still_wanted() {
        let mut fleet = fleet_of(&[("a", true)], &[]);
        let a = fleet.slots.slot_of("a").expect("a slot");
        fleet.stopped.insert(a);
        fleet.rule_closed.close(a, true);
        assert!(fleet.wallpaper_wanted());
    }

    /// A monitor with an explicit rectangle (its work area is the same).
    fn monitor_at(id: &str, primary: bool, area: [i32; 4]) -> MonitorInfo {
        let rect = RectPx {
            left: area[0],
            top: area[1],
            right: area[2],
            bottom: area[3],
        };
        MonitorInfo {
            rect,
            work: rect,
            ..monitor(id, primary)
        }
    }

    /// What each renderer connection of a test fleet was sent, by slot.
    type SentLogs = HashMap<u8, Arc<Mutex<Vec<ToRenderer>>>>;

    /// A fleet whose monitors each have a slot, an assignment and a renderer connection that records what it is sent,
    /// without any renderer process: applying snapshots of these monitors never spawns.
    fn recording_fleet(monitors: &[MonitorInfo]) -> (Fleet, SharedRenderers, SentLogs) {
        let mut fleet = fleet_with_a_job();
        fleet.web_owner = web_profile_owner(monitors).map(|monitor| monitor.id.clone());
        let renderers = SharedRenderers::default();
        let mut logs = HashMap::new();
        for monitor in monitors {
            fleet.monitors.push(monitor.clone());
            let slot = fleet.slots.assign(&monitor.id).expect("a free slot");
            let web_profile = fleet.web_profile_for(&monitor.id);
            let assignment = fleet
                .job
                .as_ref()
                .expect("a job")
                .assignment(slot, monitor, web_profile);
            lock_assignments(&fleet.assignments).insert(slot, assignment);
            let log = Arc::new(Mutex::new(Vec::new()));
            let record = Arc::clone(&log);
            let link = Link::start(format!("test-writer-{slot}"), 0, move |message| {
                record.lock().unwrap_or_else(PoisonError::into_inner).push(message);
                true
            })
            .expect("the writer thread starts");
            lock(&renderers).connections.insert(slot, link);
            logs.insert(slot, log);
        }
        (fleet, renderers, logs)
    }

    /// Everything the service sent to the renderer of `slot` so far: a Close goes through the same queue behind it and
    /// is waited for, so nothing queued earlier can still be on its way; the Close itself is left out.
    fn sent_to(renderers: &SharedRenderers, logs: &SentLogs, slot: u8) -> Vec<ToRenderer> {
        assert!(send_to(renderers, slot, &ToRenderer::Close), "the sentinel is queued");
        let log = &logs[&slot];
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let written = log.lock().unwrap_or_else(PoisonError::into_inner).clone();
            if written.last() == Some(&ToRenderer::Close) {
                return written[..written.len() - 1].to_vec();
            }
            assert!(Instant::now() < deadline, "the messages were not written");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn replace_all_an_origin_shift_reaches_the_unchanged_monitors_renderer() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY9";
        let (mut fleet, renderers, logs) = recording_fleet(&[
            monitor_at(first, true, [0, 0, 1920, 1080]),
            monitor_at(second, false, [1920, 0, 3840, 1080]),
        ]);
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };

        let applied = fleet.apply_snapshot(
            vec![
                monitor_at(first, true, [0, 0, 1920, 1080]),
                monitor_at(second, false, [-1920, 0, 0, 1080]),
            ],
            &ctx,
        );

        assert_eq!(applied, Applied::default());
        let slot_first = fleet.slots.slot_of(first).expect("a slot");
        let slot_second = fleet.slots.slot_of(second).expect("a slot");
        let unchanged = sent_to(&renderers, &logs, slot_first);
        let moved = sent_to(&renderers, &logs, slot_second);
        assert!(
            unchanged.contains(&ToRenderer::SetRect([0, 0, 1920, 1080])),
            "the unchanged monitor was not re-placed: {unchanged:?}"
        );
        assert!(moved.contains(&ToRenderer::SetRect([-1920, 0, 0, 1080])), "{moved:?}");
        let assignments = lock_assignments(&fleet.assignments);
        assert_eq!(assignments[&slot_first].rect, [0, 0, 1920, 1080]);
        assert_eq!(assignments[&slot_second].rect, [-1920, 0, 0, 1080]);
    }

    #[test]
    fn the_primary_monitor_gets_the_persistent_web_profile_in_any_list_order() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY9";
        let (fleet, _renderers, _logs) = recording_fleet(&[
            monitor_at(second, false, [1920, 0, 3840, 1080]),
            monitor_at(first, true, [0, 0, 1920, 1080]),
        ]);

        let assignments = lock_assignments(&fleet.assignments);

        let slot_first = fleet.slots.slot_of(first).expect("a slot");
        let slot_second = fleet.slots.slot_of(second).expect("a slot");
        assert_eq!(assignments[&slot_first].web_profile, ProfileKind::Persistent);
        assert_eq!(assignments[&slot_second].web_profile, ProfileKind::Ephemeral);
    }

    #[test]
    fn a_snapshot_that_moves_the_primary_flag_flips_the_stored_web_profiles() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY9";
        let (mut fleet, renderers, _logs) = recording_fleet(&[
            monitor_at(first, true, [0, 0, 1920, 1080]),
            monitor_at(second, false, [1920, 0, 3840, 1080]),
        ]);
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };

        fleet.apply_snapshot(
            vec![
                monitor_at(first, false, [0, 0, 1920, 1080]),
                monitor_at(second, true, [1920, 0, 3840, 1080]),
            ],
            &ctx,
        );

        let slot_first = fleet.slots.slot_of(first).expect("a slot");
        let slot_second = fleet.slots.slot_of(second).expect("a slot");
        let assignments = lock_assignments(&fleet.assignments);
        assert_eq!(assignments[&slot_first].web_profile, ProfileKind::Ephemeral);
        assert_eq!(assignments[&slot_second].web_profile, ProfileKind::Persistent);
    }

    #[test]
    fn a_use_restart_keeps_the_web_profile_of_each_slot() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY9";
        let (mut fleet, _renderers, _logs) = recording_fleet(&[
            monitor_at(second, false, [1920, 0, 3840, 1080]),
            monitor_at(first, true, [0, 0, 1920, 1080]),
        ]);
        let job = fleet.job.clone().expect("a job");

        fleet.adopt_job(job);

        let slot_first = fleet.slots.slot_of(first).expect("a slot");
        let slot_second = fleet.slots.slot_of(second).expect("a slot");
        let assignments = lock_assignments(&fleet.assignments);
        assert_eq!(assignments[&slot_first].web_profile, ProfileKind::Persistent);
        assert_eq!(assignments[&slot_second].web_profile, ProfileKind::Ephemeral);
    }

    #[test]
    fn replace_all_an_unchanged_snapshot_re_places_every_renderer() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY2";
        let layout = || {
            vec![
                monitor_at(first, true, [0, 0, 1920, 1080]),
                monitor_at(second, false, [1920, 0, 3840, 1080]),
            ]
        };
        let (mut fleet, renderers, logs) = recording_fleet(&layout());
        // A playing monitor gives an election something to do, so a wrongly run one would send a role.
        fleet
            .playing
            .insert(fleet.slots.slot_of(first).expect("a slot for the first monitor"));
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };

        let applied = fleet.apply_snapshot(layout(), &ctx);

        assert_eq!(applied, Applied { started: 0, failed: 0 });
        for (id, rect) in [(first, [0, 0, 1920, 1080]), (second, [1920, 0, 3840, 1080])] {
            let slot = fleet.slots.slot_of(id).expect("a slot");
            assert_eq!(sent_to(&renderers, &logs, slot), [ToRenderer::SetRect(rect)]);
        }
        assert_eq!(lock(&renderers).master, None, "no election ran");
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

        assert!(!fleet.enable_fullscreen(FullScreenAction::Resume, ActionScope::AllMonitors, PauseKind::Light));
        assert!(fleet.fullscreen.is_none(), "a Resume rule left state behind");
    }

    #[test]
    fn enable_fullscreen_pause_turns_the_rule_and_its_timer_on() {
        let mut fleet = fleet_with_a_job();

        assert!(fleet.enable_fullscreen(FullScreenAction::Pause, ActionScope::AllMonitors, PauseKind::Light));
        assert!(fleet.fullscreen.is_some());
    }

    #[test]
    fn enable_fullscreen_close_turns_the_rule_and_its_timer_on_for_heavy_pause() {
        let mut fleet = fleet_with_a_job();

        assert!(fleet.enable_fullscreen(FullScreenAction::Close, ActionScope::AllMonitors, PauseKind::Heavy));
        assert!(fleet.fullscreen.is_some());
    }

    #[test]
    fn enable_fullscreen_without_a_job_stays_off() {
        let mut fleet = Fleet::new(None, SharedAssignments::default(), installed_engines());

        assert!(!fleet.enable_fullscreen(FullScreenAction::Pause, ActionScope::AllMonitors, PauseKind::Light));
        assert!(fleet.fullscreen.is_none());
    }

    #[test]
    fn the_plan_carries_the_fullscreen_scope_of_the_settings() {
        let mut settings = Settings::default();
        let root = std::path::Path::new("no-such-legacy-root");

        assert_eq!(
            plan_from(&settings, root, &installed_engines()).scope,
            ActionScope::AllMonitors
        );

        settings.backgroundog.full_screen_scope = FullScreenScope::AffectedMonitor;
        assert_eq!(
            plan_from(&settings, root, &installed_engines()).scope,
            ActionScope::AffectedMonitor
        );
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

    // --- Use (plan 03-09) ---

    fn use_job(source: &str) -> Job {
        Job {
            title: "Used".to_owned(),
            source: source.to_owned(),
            settings: PlaybackSettings::default(),
            screen: ScreenArea::default(),
            kind: WallpaperType::Video,
            engine: EngineType::MpvPlayerLive,
            fallback: None,
            theme_dir: "C:/themes/new".to_owned(),
            web: WebOptions::default(),
            input: InputSettings::default(),
        }
    }

    fn web_job() -> Job {
        Job {
            kind: WallpaperType::Web,
            engine: EngineType::CefSharpLive,
            theme_dir: "C:/themes/page".to_owned(),
            ..use_job("C:/themes/page/index.html")
        }
    }

    /// A Use of "new" while "old" is selected, whose theme passed the gate.
    fn use_input() -> UseInput<'static> {
        UseInput {
            folder: "new",
            gate: Ok(use_job("C:/themes/new/video.mp4")),
            selected: "old",
            running: true,
            running_engine: Some(EngineType::MpvPlayerLive),
            rule: RuleState::None,
            moving: false,
        }
    }

    fn refused(decision: UseDecision) -> UseRefusal {
        match decision {
            UseDecision::Refuse(reason) => reason,
            other => panic!("the Use was not refused: {other:?}"),
        }
    }

    fn rejected(rejection: ThemeRejection) -> UseInput<'static> {
        UseInput {
            gate: Err(rejection),
            ..use_input()
        }
    }

    #[test]
    fn maintenance_is_answered_with_nothing_during_a_library_move() {
        assert_eq!(maintenance_decision(false, true), MaintenanceDecision::AnswerNothing);
    }

    #[test]
    fn maintenance_waits_for_the_run_that_is_going_during_no_library_move() {
        assert_eq!(maintenance_decision(true, false), MaintenanceDecision::Wait);
        assert_eq!(
            maintenance_decision(true, true),
            MaintenanceDecision::Wait,
            "the run that is going answers"
        );
    }

    #[test]
    fn maintenance_starts_when_nothing_else_touches_the_library_move_free() {
        assert_eq!(maintenance_decision(false, false), MaintenanceDecision::Start);
    }

    /// A library move that has not ended.
    fn a_move() -> LibraryMove {
        LibraryMove {
            source: PathBuf::from("source"),
            destination: PathBuf::from("destination"),
            carry: false,
            stage: MoveStage::Closing,
            closed: Vec::new(),
        }
    }

    #[test]
    fn respawn_closed_waits_while_the_library_move_runs() {
        let id = "\\\\.\\DISPLAY1";
        let (mut fleet, renderers, _logs) = recording_fleet(&[monitor_at(id, true, [0, 0, 1920, 1080])]);
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        // A spawn would fail instead of starting a process: what counts is whether it is tried.
        supervisor.begin_shutdown();
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };
        let slot = fleet.slots.slot_of(id).expect("a slot");
        fleet.rule_closed.close(slot, true);
        fleet.rule_closed.want_back(slot);
        fleet.stopped.insert(slot);
        fleet.library_move = Some(a_move());

        fleet.respawn_closed(&ctx);

        assert!(fleet.stopped.contains(&slot), "a start was tried during the move");
        assert_eq!(fleet.rule_closed.ready(), [slot], "the slot is still wanted back");
        // The monitor going during the move changes nothing either: the slot is dealt with when the move ends.
        fleet.slots.retire(id);
        fleet.respawn_closed(&ctx);
        assert_eq!(fleet.rule_closed.ready(), [slot]);
        fleet.library_move = None;
        fleet.respawn_closed(&ctx);
        assert!(
            fleet.rule_closed.ready().is_empty(),
            "the slot of a gone monitor is forgotten"
        );
    }

    #[test]
    fn a_renderer_exit_while_quitting_still_lets_a_closing_move_go_on() {
        let (mut fleet, _renderers, _logs) = recording_fleet(&[monitor_at("D1", true, [0, 0, 100, 100])]);
        fleet.library_move = Some(a_move());
        fleet.rule_closed.close(1, false);
        fleet.rule_closed.close(2, false);
        assert!(fleet.rule_closed.any_awaiting());

        fleet.note_exit_while_quitting(1);
        assert!(fleet.rule_closed.any_awaiting(), "slot 2 has not ended");
        fleet.note_exit_while_quitting(2);

        assert!(
            !fleet.rule_closed.any_awaiting(),
            "every renderer ended, the move can leave Closing and the service can exit"
        );
        assert!(
            !may_exit(true, true),
            "the exit still waits until the move itself has ended"
        );
        fleet.library_move = None;
        assert!(may_exit(true, false));
    }

    #[test]
    fn a_renderer_exit_while_quitting_without_a_move_changes_nothing() {
        let (mut fleet, _renderers, _logs) = recording_fleet(&[monitor_at("D1", true, [0, 0, 100, 100])]);
        fleet.rule_closed.close(1, false);

        fleet.note_exit_while_quitting(1);

        assert!(fleet.rule_closed.awaiting_exit(1), "no move, no bookkeeping");
    }

    #[test]
    fn a_monitor_change_during_a_library_move_is_applied_when_the_move_ends() {
        let first = "\\\\.\\DISPLAY1";
        let second = "\\\\.\\DISPLAY9";
        let (mut fleet, renderers, logs) = recording_fleet(&[
            monitor_at(first, true, [0, 0, 1920, 1080]),
            monitor_at(second, false, [1920, 0, 3840, 1080]),
        ]);
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };
        let slot_first = fleet.slots.slot_of(first).expect("a slot");
        let slot_second = fleet.slots.slot_of(second).expect("a slot");
        fleet.library_move = Some(a_move());

        // Two snapshots settle during the move: the second one replaces the first.
        let during = [
            vec![monitor_at(first, true, [0, 0, 1280, 720])],
            vec![monitor_at(first, true, [0, 0, 2560, 1440])],
        ];
        for monitors in during {
            assert_eq!(fleet.apply_or_defer(monitors, false, &ctx), Applied::default());
        }

        assert_eq!(
            fleet.slots.slot_of(second),
            Some(slot_second),
            "nothing was applied yet"
        );
        assert!(
            logs[&slot_first]
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_empty(),
            "a renderer was told during the move"
        );
        fleet.library_move = None;
        fleet.finish_move(&ctx);

        assert_eq!(
            sent_to(&renderers, &logs, slot_first),
            [ToRenderer::SetRect([0, 0, 2560, 1440])],
            "the latest snapshot is the one that counts"
        );
        assert_eq!(fleet.slots.slot_of(second), None, "the unplugged monitor lost its slot");
        assert_eq!(fleet.slots.slot_of(first), Some(slot_first));
        assert!(fleet.pending_snapshot.is_none());
    }

    #[test]
    fn a_monitor_change_that_settles_while_quitting_is_dropped() {
        let id = "\\\\.\\DISPLAY1";
        let (mut fleet, renderers, logs) = recording_fleet(&[monitor_at(id, true, [0, 0, 1920, 1080])]);
        let supervisor = Supervisor::new(|_| {}).expect("the job object is created");
        let ctx = Ctx {
            supervisor: &supervisor,
            renderers: &renderers,
        };
        let slot = fleet.slots.slot_of(id).expect("a slot");
        fleet.library_move = Some(a_move());

        let applied = fleet.apply_or_defer(vec![monitor_at(id, true, [0, 0, 1280, 720])], true, &ctx);

        assert_eq!(applied, Applied::default());
        assert!(fleet.pending_snapshot.is_none(), "nothing is kept while quitting");
        assert!(sent_to(&renderers, &logs, slot).is_empty());
    }

    mod decide_use {
        use super::*;

        #[test]
        fn loads_the_theme_into_the_renderers_that_run() {
            let decision = decide_use(&use_input());

            let UseDecision::Load(job) = decision else {
                panic!("a Use while renderers run is not a Load: {decision:?}");
            };
            assert_eq!(job.source, "C:/themes/new/video.mp4");
        }

        #[test]
        fn decide_use_restarts_when_the_engine_changes() {
            // Video -> Web and Web -> Video switch engines; a renderer is one process with one engine.
            let to_web = decide_use(&UseInput {
                gate: Ok(web_job()),
                ..use_input()
            });
            let UseDecision::Restart(job) = to_web else {
                panic!("Video to Web is not a Restart: {to_web:?}");
            };
            assert_eq!(job.kind, WallpaperType::Web);

            let to_video = decide_use(&UseInput {
                running_engine: Some(EngineType::CefSharpLive),
                ..use_input()
            });
            assert!(matches!(to_video, UseDecision::Restart(_)), "{to_video:?}");
        }

        fn gif_job(engine: EngineType) -> Job {
            Job {
                kind: WallpaperType::Gif,
                engine,
                ..use_job("C:/themes/anim/animation.gif")
            }
        }

        #[test]
        fn decide_use_restarts_whenever_the_built_in_gif_renderer_is_on_either_side() {
            // mpv -> built-in GIF, built-in GIF -> mpv, built-in GIF -> built-in GIF, built-in GIF -> web: a GIF set to
            // the built-in engine never ends up in an mpv renderer, and the built-in renderer is not told to load in place.
            let cases = [
                (EngineType::MpvPlayerLive, gif_job(EngineType::XavierLive)),
                (EngineType::XavierLive, use_job("C:/themes/new/video.mp4")),
                (EngineType::XavierLive, gif_job(EngineType::VexanaLive)),
                (EngineType::VexanaLive, gif_job(EngineType::XavierLive)),
                (EngineType::XavierLive, web_job()),
                (EngineType::WebViewLive, gif_job(EngineType::XavierLive)),
            ];
            for (running, job) in cases {
                let decision = decide_use(&UseInput {
                    gate: Ok(job.clone()),
                    running_engine: Some(running),
                    ..use_input()
                });
                assert!(
                    matches!(decision, UseDecision::Restart(_)),
                    "{running:?} -> {:?}: {decision:?}",
                    job.engine
                );
            }
        }

        #[test]
        fn decide_use_loads_in_place_only_between_two_mpv_wallpapers() {
            let gif_on_mpv = decide_use(&UseInput {
                gate: Ok(gif_job(EngineType::MpvPlayerLive)),
                ..use_input()
            });
            assert!(matches!(gif_on_mpv, UseDecision::Load(_)), "{gif_on_mpv:?}");
        }

        #[test]
        fn gif_is_playable_so_gif_cards_can_be_used() {
            assert!(PLAYABLE.contains(&WallpaperType::Gif));
            let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("crates")
                .join("sucrose-core")
                .join("tests")
                .join("fixtures")
                .join("library");
            let theme = check_selected(&library, "gif", &app_version(), PLAYABLE).expect("a Gif theme passes the gate");
            assert_eq!(theme.info.kind, WallpaperType::Gif);
        }

        fn url_job(address: &str, engine: EngineType) -> Job {
            Job {
                kind: WallpaperType::Url,
                engine,
                theme_dir: "C:/themes/site".to_owned(),
                ..use_job(address)
            }
        }

        #[test]
        fn youtube_is_playable_and_the_gate_gives_its_address_as_the_source() {
            assert!(PLAYABLE.contains(&WallpaperType::YouTube));
            let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("web-themes");
            let theme =
                check_selected(&library, "youtube", &app_version(), PLAYABLE).expect("the YouTube fixture passes");
            assert_eq!(
                theme.source,
                ThemeSource::Address("https://www.youtube.com/watch?v=znNllvBt_Jg".to_owned())
            );
        }

        #[test]
        fn url_is_playable_and_its_job_carries_the_address_as_the_source() {
            assert!(PLAYABLE.contains(&WallpaperType::Url));
            let info = sucrose_core::theme::parse_info(
                r#"{"AppVersion":"26.6.19.0","Title":"Site","Type":1,"Source":"https://example.com/"}"#,
            )
            .expect("a Url SucroseInfo parses");
            let theme = SelectedTheme {
                dir: std::path::PathBuf::from("C:/themes/site"),
                info,
                source: ThemeSource::Address("https://example.com/".to_owned()),
            };

            let job = job_for(
                &theme,
                &Engine::default(),
                PauseKind::default(),
                ThemeMode::Dark,
                "Probe/1",
                &installed_engines(),
            )
            .expect("a Url theme makes a job");

            assert_eq!(job.source, "https://example.com/");
            assert_eq!(
                job.web.user_agent, "Probe/1",
                "a Url job carries the user agent of the General section"
            );
            assert_eq!(job.kind, WallpaperType::Url);
            assert!(matches!(job.engine, EngineType::CefSharpLive | EngineType::WebViewLive));
        }

        #[test]
        fn two_url_themes_with_the_same_address_are_two_themes_and_only_the_same_folder_is_already_applied() {
            // The folder is the identity (TYPE-04): the same address in another folder restarts the renderer like any
            // other web-engine Use, and only the folder that plays is AlreadyApplied.
            let same_address = url_job("https://example.com/", EngineType::CefSharpLive);
            let other = decide_use(&UseInput {
                folder: "site-b",
                gate: Ok(same_address.clone()),
                selected: "site-a",
                running_engine: Some(EngineType::CefSharpLive),
                ..use_input()
            });
            assert!(matches!(other, UseDecision::Restart(_)), "{other:?}");

            let same = decide_use(&UseInput {
                folder: "site-a",
                gate: Ok(same_address),
                selected: "site-a",
                running_engine: Some(EngineType::CefSharpLive),
                ..use_input()
            });
            assert!(matches!(same, UseDecision::AlreadyApplied), "{same:?}");
        }

        #[test]
        fn restarts_between_web_themes_because_cef_cannot_load_another_page_in_place() {
            let web_to_web = decide_use(&UseInput {
                gate: Ok(web_job()),
                running_engine: Some(EngineType::CefSharpLive),
                folder: "other",
                ..use_input()
            });
            assert!(matches!(web_to_web, UseDecision::Restart(_)), "{web_to_web:?}");
        }

        #[test]
        fn keeps_the_load_source_path_between_themes_of_the_same_engine() {
            let video_to_video = decide_use(&use_input());
            assert!(matches!(video_to_video, UseDecision::Load(_)), "{video_to_video:?}");
        }

        #[test]
        fn of_the_playing_web_theme_changes_nothing() {
            let decision = decide_use(&UseInput {
                folder: "old",
                gate: Ok(web_job()),
                running_engine: Some(EngineType::CefSharpLive),
                ..use_input()
            });

            assert!(matches!(decision, UseDecision::AlreadyApplied), "{decision:?}");
        }

        #[test]
        fn starts_the_fleet_when_nothing_runs() {
            let decision = decide_use(&UseInput {
                running: false,
                ..use_input()
            });

            assert!(matches!(decision, UseDecision::Start(_)), "{decision:?}");
        }

        #[test]
        fn of_the_selected_theme_that_plays_reloads_nothing() {
            let decision = decide_use(&UseInput {
                folder: "old",
                ..use_input()
            });

            assert!(matches!(decision, UseDecision::AlreadyApplied), "{decision:?}");
        }

        #[test]
        fn of_the_selected_theme_that_does_not_play_starts_it() {
            let decision = decide_use(&UseInput {
                folder: "old",
                running: false,
                ..use_input()
            });

            assert!(matches!(decision, UseDecision::Start(_)), "{decision:?}");
        }

        #[test]
        fn refuses_while_the_library_moves() {
            let decision = decide_use(&UseInput {
                moving: true,
                ..use_input()
            });

            assert_eq!(refused(decision), UseRefusal::Moving);
        }

        #[test]
        fn refuses_while_the_fullscreen_rule_paused_or_closed_the_wallpaper() {
            for (rule, expected) in [
                (RuleState::Paused, UseRefusal::RulePaused),
                (RuleState::Closed, UseRefusal::RuleClosed),
            ] {
                let decision = decide_use(&UseInput { rule, ..use_input() });

                assert_eq!(refused(decision), expected);
            }
        }

        #[test]
        fn refuses_a_theme_that_is_not_a_video() {
            let decision = decide_use(&rejected(ThemeRejection::TypeNotPlayable {
                folder: "new".to_owned(),
                found: sucrose_core::theme::WallpaperType::Web,
            }));

            assert_eq!(refused(decision), UseRefusal::TypeNotPlayable);
        }

        #[test]
        fn refuses_a_theme_for_a_newer_sucrose() {
            let version = app_version();
            let decision = decide_use(&rejected(ThemeRejection::NeedsNewerApp {
                folder: "new".to_owned(),
                required: version,
                current: version,
            }));

            assert_eq!(refused(decision), UseRefusal::NewerVersion);
        }

        #[test]
        fn refuses_a_folder_that_does_not_exist_as_not_found() {
            for rejection in [
                ThemeRejection::FolderMissing {
                    folder: "new".to_owned(),
                },
                ThemeRejection::NothingSelected,
            ] {
                assert_eq!(refused(decide_use(&rejected(rejection))), UseRefusal::NotFound);
            }
        }

        #[test]
        fn refuses_a_damaged_theme_as_corrupt() {
            for rejection in [
                ThemeRejection::InfoUnreadable {
                    folder: "new".to_owned(),
                    reason: "the file is missing".to_owned(),
                },
                ThemeRejection::SourceInvalid {
                    folder: "new".to_owned(),
                    reason: "the file is missing".to_owned(),
                },
            ] {
                assert_eq!(refused(decide_use(&rejected(rejection))), UseRefusal::Corrupt);
            }
        }

        #[test]
        fn checks_the_theme_even_when_it_is_the_selected_one() {
            let decision = decide_use(&UseInput {
                folder: "old",
                gate: Err(ThemeRejection::FolderMissing {
                    folder: "old".to_owned(),
                }),
                ..use_input()
            });

            assert_eq!(refused(decision), UseRefusal::NotFound);
        }

        #[test]
        fn names_a_library_move_before_anything_else() {
            let decision = decide_use(&UseInput {
                moving: true,
                rule: RuleState::Closed,
                gate: Err(ThemeRejection::NothingSelected),
                ..use_input()
            });

            assert_eq!(refused(decision), UseRefusal::Moving);
        }

        #[test]
        fn maps_the_real_gate_for_names_that_would_leave_the_library() {
            let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("crates")
                .join("sucrose-core")
                .join("tests")
                .join("fixtures")
                .join("library");
            // A Url theme plays since 04-34: it passes the gate as an address.
            let url = check_selected(&library, "url-source", &app_version(), PLAYABLE).expect("a Url theme passes");
            assert_eq!(
                url.source,
                ThemeSource::Address("https://www.example.com/page".to_owned())
            );
            for (folder, expected) in [
                ("../x", UseRefusal::NotFound),
                ("a/b", UseRefusal::NotFound),
                ("a\\b", UseRefusal::NotFound),
                ("", UseRefusal::NotFound),
                ("no-such-folder", UseRefusal::NotFound),
                ("newer-version", UseRefusal::NewerVersion),
                ("broken-json", UseRefusal::Corrupt),
            ] {
                let gate = check_selected(&library, folder, &app_version(), PLAYABLE)
                    .map(|_| use_job("C:/themes/new/video.mp4"));
                let decision = decide_use(&UseInput {
                    folder,
                    gate,
                    ..use_input()
                });

                assert_eq!(refused(decision), expected, "folder {folder:?}");
            }
        }
    }

    #[test]
    fn load_source_sends_one_load_source_to_every_connected_renderer_and_updates_what_a_restart_would_play() {
        let monitors = [
            monitor_at("D1", true, [0, 0, 100, 100]),
            monitor_at("D2", false, [100, 0, 200, 100]),
        ];
        let (mut fleet, renderers, logs) = recording_fleet(&monitors);
        for slot in fleet.slots.active_slots() {
            fleet.budgets.insert(slot, RestartBudget::default());
        }
        let job = use_job("C:/themes/new/video.mp4");
        let message = ToRenderer::LoadSource {
            source: job.source.clone(),
            settings: job.settings,
        };

        let told = fleet.load_source(job, &renderers);

        assert_eq!(told, fleet.slots.active_slots());
        for slot in fleet.slots.active_slots() {
            assert_eq!(sent_to(&renderers, &logs, slot), vec![message.clone()], "slot {slot}");
            let assignments = lock_assignments(&fleet.assignments);
            assert_eq!(assignments[&slot].source, "C:/themes/new/video.mp4", "slot {slot}");
        }
        assert!(fleet.budgets.is_empty(), "an explicit Use resets every restart budget");
        assert_eq!(
            fleet.job.as_ref().map(|job| job.source.as_str()),
            Some("C:/themes/new/video.mp4")
        );
    }

    #[test]
    fn live_message_maps_loop_and_shuffle_and_nothing_else() {
        assert_eq!(
            live_message(&SettingChange::WallpaperLoop(false)),
            Some(ToRenderer::SetLoop(false))
        );
        assert_eq!(
            live_message(&SettingChange::WallpaperShuffle(true)),
            Some(ToRenderer::SetShuffle(true))
        );
        for change in [
            SettingChange::HardwareAcceleration(false),
            SettingChange::InputType(InputType::Close),
            SettingChange::InputModuleType(InputModuleType::RawInput),
            SettingChange::InputDesktop(true),
            SettingChange::LibraryPagination(5),
            SettingChange::VideoEngine(EngineType::MpvPlayerLive),
        ] {
            assert_eq!(live_message(&change), None, "{change:?}");
        }
    }

    #[test]
    fn a_loop_change_reaches_every_connected_renderer_and_the_assignments_without_a_restart() {
        let monitors = [
            monitor_at("D1", true, [0, 0, 100, 100]),
            monitor_at("D2", false, [100, 0, 200, 100]),
        ];
        let (mut fleet, renderers, logs) = recording_fleet(&monitors);
        for slot in fleet.slots.active_slots() {
            fleet.budgets.insert(slot, RestartBudget::default());
        }

        let told = fleet.apply_live(&ToRenderer::SetLoop(false), &renderers);

        assert_eq!(told, fleet.slots.active_slots());
        for slot in fleet.slots.active_slots() {
            assert_eq!(
                sent_to(&renderers, &logs, slot),
                vec![ToRenderer::SetLoop(false)],
                "slot {slot}"
            );
            assert!(
                !lock_assignments(&fleet.assignments)[&slot].settings.looping,
                "a renderer that restarts later keeps the new value, slot {slot}"
            );
        }
        assert!(!fleet.job.as_ref().expect("a job").settings.looping);
        assert_eq!(
            fleet.budgets.len(),
            fleet.slots.active_slots().len(),
            "a live setting is no Use: the restart budgets stay"
        );
    }

    #[test]
    fn a_shuffle_change_reaches_the_renderers_and_the_web_options_of_the_job() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, logs) = recording_fleet(&monitors);
        let slot = fleet.slots.slot_of("D1").expect("a slot");
        assert!(
            fleet.job.as_ref().expect("a job").web.shuffle,
            "shuffle is on by default"
        );

        let told = fleet.apply_live(&ToRenderer::SetShuffle(false), &renderers);

        assert_eq!(told, vec![slot]);
        assert_eq!(sent_to(&renderers, &logs, slot), vec![ToRenderer::SetShuffle(false)]);
        assert!(!lock_assignments(&fleet.assignments)[&slot].web.shuffle);
        assert!(!fleet.job.as_ref().expect("a job").web.shuffle);
    }

    #[test]
    fn a_live_setting_with_no_connected_renderer_still_updates_the_assignments() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, _logs) = recording_fleet(&monitors);
        let slot = fleet.slots.slot_of("D1").expect("a slot");
        lock(&renderers).connections.remove(&slot);

        let told = fleet.apply_live(&ToRenderer::SetLoop(false), &renderers);

        assert!(told.is_empty(), "nobody is connected to be told");
        assert!(!lock_assignments(&fleet.assignments)[&slot].settings.looping);
    }

    #[test]
    fn load_source_leaves_a_renderer_that_has_not_connected_to_read_the_new_assignment_at_its_hello() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, _logs) = recording_fleet(&monitors);
        let slot = fleet.slots.slot_of("D1").expect("a slot");
        lock(&renderers).connections.remove(&slot);

        let told = fleet.load_source(use_job("C:/themes/new/video.mp4"), &renderers);

        assert!(told.is_empty(), "nobody is connected to be told");
        assert_eq!(
            lock_assignments(&fleet.assignments)[&slot].source,
            "C:/themes/new/video.mp4"
        );
    }

    #[test]
    fn load_source_keeps_the_playback_settings_of_the_new_job_in_the_assignments() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, _logs) = recording_fleet(&monitors);
        let mut job = use_job("C:/themes/new/video.mp4");
        job.settings.volume = 12;

        fleet.load_source(job, &renderers);

        let slot = fleet.slots.slot_of("D1").expect("a slot");
        assert_eq!(lock_assignments(&fleet.assignments)[&slot].settings.volume, 12);
    }

    #[test]
    fn the_restart_transition_closes_every_connected_renderer_and_wants_it_back() {
        let monitors = [
            monitor_at("D1", true, [0, 0, 100, 100]),
            monitor_at("D2", false, [100, 0, 200, 100]),
        ];
        let (mut fleet, renderers, logs) = recording_fleet(&monitors);

        let closed = fleet.restart_with(use_job("C:/themes/new/video.mp4"), &renderers);

        assert_eq!(closed, fleet.slots.active_slots());
        for slot in fleet.slots.active_slots() {
            // sent_to would stop at the Close this very test looks for, so the log is read directly.
            let deadline = Instant::now() + Duration::from_secs(5);
            while logs[&slot].lock().unwrap_or_else(PoisonError::into_inner).is_empty() {
                assert!(Instant::now() < deadline, "the Close of slot {slot} was not written");
                thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(
                *logs[&slot].lock().unwrap_or_else(PoisonError::into_inner),
                vec![ToRenderer::Close],
                "slot {slot}"
            );
            assert!(
                fleet.rule_closed.awaiting_exit(slot),
                "slot {slot} waits for its old renderer to end"
            );
            // The exit of the old renderer makes it ready to start again with the new assignment.
            assert!(fleet.rule_closed.note_exit(slot));
            assert_eq!(fleet.rule_closed.ready(), vec![slot]);
            fleet.rule_closed.forget(slot);
            assert_eq!(
                lock_assignments(&fleet.assignments)[&slot].source,
                "C:/themes/new/video.mp4"
            );
        }
    }

    #[test]
    fn a_restart_hands_the_new_engine_and_web_options_to_the_assignments_of_every_slot() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, _logs) = recording_fleet(&monitors);
        let mut job = web_job();
        job.web.allow_invalid_certificates = true;
        job.web.hardware_acceleration = false;
        job.web.user_agent = "Probe/1".to_owned();

        fleet.restart_with(job, &renderers);

        let slot = fleet.slots.slot_of("D1").expect("a slot");
        let assignment = lock_assignments(&fleet.assignments)[&slot].clone();
        assert_eq!(assignment.engine, EngineType::CefSharpLive);
        assert!(
            assignment.web.allow_invalid_certificates,
            "a restarted renderer reads the saved switch"
        );
        assert!(!assignment.web.hardware_acceleration);
        assert_eq!(
            assignment.web.user_agent, "Probe/1",
            "a restart carries the user agent saved now"
        );
    }

    #[test]
    fn engines_running_pairs_each_running_slot_with_its_monitor_and_the_engine_of_the_job() {
        let fleet = fleet_of(&[("D1", true), ("D2", false), ("D3", false)], &[]);
        let slot_of = |id: &str| fleet.slots.slot_of(id).expect("a slot");

        let running = engines_running(fleet.job.as_ref(), &fleet.slots, &[slot_of("D1"), slot_of("D3"), 99]);

        assert_eq!(
            running,
            vec![
                ("D1".to_owned(), EngineType::MpvPlayerLive),
                ("D3".to_owned(), EngineType::MpvPlayerLive)
            ],
            "a slot without a monitor is left out"
        );
        assert!(engines_running(None, &fleet.slots, &[slot_of("D1")]).is_empty());
        assert!(engines_running(fleet.job.as_ref(), &fleet.slots, &[]).is_empty());
    }

    #[test]
    fn open_devtools_goes_only_to_a_running_web_wallpaper_on_a_web_engine() {
        let mut slots = SlotMap::default();
        let d1 = slots.assign("D1").expect("a slot");
        let d3 = slots.assign("D3").expect("a slot");
        let web = Some((WallpaperType::Web, EngineType::CefSharpLive));

        assert_eq!(devtools_slot(web, &slots, &[d1, d3], "D3"), Some(d3));
        for kind in [WallpaperType::Url, WallpaperType::YouTube] {
            assert_eq!(
                devtools_slot(Some((kind, EngineType::WebViewLive)), &slots, &[d1], "D1"),
                Some(d1),
                "{kind:?} is a web wallpaper too"
            );
        }
        assert_eq!(devtools_slot(web, &slots, &[d1], "D3"), None, "no renderer runs on D3");
        assert_eq!(devtools_slot(web, &slots, &[d1], "D9"), None, "an unknown monitor");
        assert_eq!(devtools_slot(None, &slots, &[d1], "D1"), None, "nothing plays");
        let video = Some((WallpaperType::Video, EngineType::MpvPlayerLive));
        assert_eq!(
            devtools_slot(video, &slots, &[d1], "D1"),
            None,
            "a video has no developer tools"
        );
        let video_on_cef = Some((WallpaperType::Video, EngineType::CefSharpLive));
        assert_eq!(
            devtools_slot(video_on_cef, &slots, &[d1], "D1"),
            None,
            "only web wallpapers count"
        );
        let gif = Some((WallpaperType::Gif, EngineType::VexanaLive));
        assert_eq!(devtools_slot(gif, &slots, &[d1], "D1"), None);
    }

    #[test]
    fn a_restart_touches_only_the_wallpaper_on_a_listed_engine() {
        let cef = Some(EngineType::CefSharpLive);

        assert!(restart_affects(cef, &[EngineType::CefSharpLive]));
        assert!(restart_affects(
            cef,
            &[EngineType::WebViewLive, EngineType::CefSharpLive]
        ));
        assert!(!restart_affects(cef, &[EngineType::MpvPlayerLive]));
        assert!(!restart_affects(cef, &[]));
        assert!(!restart_affects(None, &[EngineType::CefSharpLive]));
    }

    #[test]
    fn a_restart_of_another_engine_closes_nothing_and_one_during_a_library_move_is_refused() {
        let monitors = [monitor_at("D1", true, [0, 0, 100, 100])];
        let (mut fleet, renderers, logs) = recording_fleet(&monitors);
        let host = SettingsHost::unsaved(Settings::default());

        // The job plays on mpv: a restart of the CEF wallpapers leaves it alone.
        let untouched = restart_wallpapers(&[EngineType::CefSharpLive], &host, &mut fleet, None, &renderers);
        assert_eq!(untouched, Ok(0));
        let slot = fleet.slots.slot_of("D1").expect("a slot");
        assert!(
            sent_to(&renderers, &logs, slot).is_empty(),
            "nothing was sent to the renderer"
        );
        assert!(!fleet.rule_closed.is_closed(slot));

        // Listed, but the library has no folder to build the job from.
        let no_library = restart_wallpapers(&[EngineType::MpvPlayerLive], &host, &mut fleet, None, &renderers);
        assert!(no_library.is_err());
        assert!(sent_to(&renderers, &logs, slot).is_empty());
    }

    #[test]
    fn the_use_transition_is_the_one_that_keeps_the_renderers_running() {
        assert_eq!(USE_TRANSITION, UseTransition::LoadSource);
    }

    /// Feeds the rule fullscreen samples on every monitor until it acts.
    fn trigger(rule: &mut Rule, monitors: &[&str]) {
        let all: Vec<String> = monitors.iter().map(|id| (*id).to_owned()).collect();
        for _ in 0..50 {
            if !rule.tick(&all, &all).is_empty() {
                return;
            }
        }
        panic!("the rule never acted");
    }

    #[test]
    fn the_fullscreen_rule_state_is_none_without_a_rule_or_while_it_is_quiet() {
        let mut fleet = fleet_of(&[("D1", true)], &[]);
        assert_eq!(fleet.rule_state(), RuleState::None);

        fleet.fullscreen = Some(Rule::new(FullScreenAction::Pause, ActionScope::AllMonitors));

        assert_eq!(fleet.rule_state(), RuleState::None);
    }

    #[test]
    fn the_fullscreen_rule_state_reads_paused_and_closed_from_the_rule() {
        for (action, expected) in [
            (FullScreenAction::Pause, RuleState::Paused),
            (FullScreenAction::Close, RuleState::Closed),
        ] {
            let mut fleet = fleet_of(&[("D1", true)], &[]);
            let mut rule = Rule::new(action, ActionScope::AllMonitors);
            trigger(&mut rule, &["D1"]);
            fleet.fullscreen = Some(rule);

            assert_eq!(fleet.rule_state(), expected, "{action:?}");
        }
    }

    #[test]
    fn a_portal_that_leaves_always_rechecks_the_exit_rule() {
        assert_eq!(
            portal_gone_effect(true),
            PortalGoneEffect {
                check_exit: true,
                tell_closing: true
            }
        );
        assert_eq!(
            portal_gone_effect(false),
            PortalGoneEffect {
                check_exit: true,
                tell_closing: false
            },
            "a Portal that was never registered cannot keep the service alive either, but the closing sequence does not wait for it"
        );
    }

    /// A registry with one Portal whose events are recorded; the first recorded event is its Welcome stand-in.
    fn registry_with_a_portal() -> (PortalRegistry, Arc<Mutex<Vec<PortalEvent>>>) {
        let written = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&written);
        let link = PortalLink::start("test-portal-writer".to_owned(), 1, 1, move |event| {
            record.lock().unwrap_or_else(PoisonError::into_inner).push(event);
            true
        })
        .expect("the writer thread starts");
        let mut registry = PortalRegistry::default();
        let admission = registry.admit(link, sucrose_ipc::portal::PORTAL_PROTOCOL_VERSION, false, || {
            PortalEvent::Focus { page: None }
        });
        assert_eq!(admission, PortalAdmission::Register);
        (registry, written)
    }

    fn runtime(selected: &str, rule: RuleState) -> RuntimeState {
        RuntimeState {
            selected: Some(selected.to_owned()),
            running: true,
            rule,
            library_moving: false,
            tray_visible: false,
            running_engines: Vec::new(),
        }
    }

    #[test]
    fn the_runtime_state_is_published_only_when_it_differs_from_the_last_one() {
        let (mut portals, written) = registry_with_a_portal();
        let mut last = None;

        assert!(publish_runtime(&mut portals, &mut last, runtime("a", RuleState::None)));
        assert!(!publish_runtime(&mut portals, &mut last, runtime("a", RuleState::None)));
        assert!(publish_runtime(
            &mut portals,
            &mut last,
            runtime("a", RuleState::Paused)
        ));
        assert!(!publish_runtime(
            &mut portals,
            &mut last,
            runtime("a", RuleState::Paused)
        ));

        let expected = vec![
            PortalEvent::Focus { page: None },
            PortalEvent::Runtime(runtime("a", RuleState::None)),
            PortalEvent::Runtime(runtime("a", RuleState::Paused)),
        ];
        let deadline = Instant::now() + Duration::from_secs(5);
        while written.lock().unwrap_or_else(PoisonError::into_inner).len() < expected.len() {
            assert!(Instant::now() < deadline, "the events were not written");
            thread::sleep(Duration::from_millis(10));
        }
        // Time for an event that must not exist to show up.
        thread::sleep(Duration::from_millis(100));
        assert_eq!(*written.lock().unwrap_or_else(PoisonError::into_inner), expected);
    }

    #[test]
    fn a_destination_text_must_be_an_absolute_path_without_parent_parts() {
        let absolute = if cfg!(windows) {
            r"C:\Libraries\New"
        } else {
            "/libraries/new"
        };
        assert_eq!(parse_destination(absolute), Ok(PathBuf::from(absolute)));

        let relative = parse_destination("New");
        let parent = parse_destination(&format!(
            "{absolute}{}..{}x",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        ));
        let nul = parse_destination(&format!("{absolute}\0"));
        let long = parse_destination(&format!("{absolute}{}", "a".repeat(MAX_PATH_TEXT)));
        for refused in [relative, parent, nul, long] {
            assert_eq!(refused, Err(MoveOutcome::Refused(MoveRefusal::Missing)));
        }
    }

    #[test]
    fn a_move_waits_for_every_closed_renderer_to_have_ended() {
        let mut closed = RuleClosed::default();
        assert!(!closed.any_awaiting(), "nothing closed, nothing to wait for");

        closed.close(1, false);
        closed.close(2, true);
        assert!(closed.any_awaiting(), "slot 1 has not ended");

        assert!(closed.note_exit(1));
        assert!(!closed.any_awaiting());

        closed.close(3, false);
        closed.forget(3);
        assert!(
            !closed.any_awaiting(),
            "a slot that was forgotten (its monitor went) is not waited for"
        );
    }

    #[test]
    fn only_refresh_startup_acts_while_quitting() {
        let change = PortalRequest::Change(SettingChange::AppVisible(true));
        let used = PortalRequest::Use {
            folder: "Theme".to_owned(),
        };
        let moved = PortalRequest::MoveLibrary {
            destination: "C:/Library".to_owned(),
        };
        let startup = PortalRequest::SetStartup(sucrose_ipc::portal::StartupMode::Normal);
        for request in [change, used, PortalRequest::MaintainLibrary, moved, startup] {
            assert!(
                !acts_while_quitting(&request),
                "{request:?} must not be acted on while quitting"
            );
        }
        assert!(acts_while_quitting(&PortalRequest::RefreshStartup));
    }

    #[test]
    fn a_move_asked_for_while_quitting_is_refused_busy() {
        let moved = PortalRequest::MoveLibrary {
            destination: "C:/Library".to_owned(),
        };

        assert_eq!(
            quitting_answer(&moved),
            Some(PortalEvent::MoveResult(MoveOutcome::Refused(MoveRefusal::Busy)))
        );
        let used = PortalRequest::Use {
            folder: "Theme".to_owned(),
        };
        assert_eq!(
            quitting_answer(&used),
            None,
            "a request that is only dropped has no answer"
        );
        assert_eq!(quitting_answer(&PortalRequest::RefreshStartup), None);
    }

    #[test]
    fn quitting_waits_for_the_library_move_before_exit() {
        assert!(!may_exit(true, true), "a move that runs holds the exit");
        assert!(may_exit(true, false));
        assert!(!may_exit(false, false), "nothing is done yet");
        assert!(!may_exit(false, true));
    }

    #[test]
    fn a_move_that_ends_while_quitting_starts_no_renderer() {
        assert!(!resume_after_move(true));
        assert!(resume_after_move(false));
    }
}
