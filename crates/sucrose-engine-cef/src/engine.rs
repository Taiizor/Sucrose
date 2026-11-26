//! CEF behind the [`Engine`] trait. CEF is bound to the thread that owns the desktop surface, so the engine is split in
//! two: [`CefEngine`] is the thread-safe handle the IPC and clock threads call, and it only queues commands and wakes
//! the surface thread; [`CefPump`] owns the CEF host, drains the queue and pumps CEF on the surface thread. No CEF
//! function is called from another thread.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded};
use sucrose_core::settings::Stretch;
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started, SurfacePump},
};
use sucrose_engine_web::{
    origin,
    profile::{self, ProfileLease},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings};
use sucrose_platform::{LoopProxy, LoopTimers};

use crate::browser::{CefError, CefHost, CefStart};

/// The timer id of the CEF message pump (D-036): the only timer a CEF renderer sets, armed with the delay
/// [`CefHost::do_work`] returns after each pump call.
pub const CEF_PUMP_TIMER: u32 = 1;

/// How many commands may wait for the surface thread. A full queue drops the newest command and logs it, so the IPC
/// thread never blocks (T-04-13); the engine's state values below always hold the latest of each.
const QUEUE_CAPACITY: usize = 64;

/// What the renderer gives the engine of the surface thread it runs on.
pub struct SurfaceHooks {
    /// Wakes the renderer's message loop; the engine calls it after it queued a command.
    pub proxy: LoopProxy,
    /// The loop's timers; the pump arms its timer through them.
    pub timers: LoopTimers,
    /// Called once, from the surface thread, when the first page finished loading: the first frame the user can see.
    pub on_playing: Arc<dyn Fn() + Send + Sync>,
}

/// A command the IPC threads queue for the surface thread.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    Pause { paused: bool, level: PauseLevel },
    Volume(u8),
    Loop(bool),
    Stretch(Stretch),
}

/// The newest value of each setting, for the page calls of plan 04-09 and for anyone asking what the engine was told.
#[derive(Debug)]
struct State {
    paused: AtomicBool,
    volume: AtomicU8,
    looping: AtomicBool,
    stretch: Mutex<Stretch>,
}

/// The thread-safe handle of a CEF renderer.
pub struct CefEngine {
    commands: Sender<Command>,
    /// Wakes the surface thread's message loop; callable from any thread.
    wake: Arc<dyn Fn() + Send + Sync>,
    state: Arc<State>,
}

impl std::fmt::Debug for CefEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CefEngine")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

/// The surface-thread half of a CEF renderer: owns the browser and is the only thing that calls CEF.
pub struct CefPump {
    host: CefHost,
    commands: Receiver<Command>,
    timers: LoopTimers,
}

/// Why CEF did not start, as the engine failure the renderer reports.
fn start_failure(error: &CefError) -> EngineError {
    match error {
        CefError::Load(_) => EngineError::Missing {
            reason: "CEF was not found.".to_owned(),
            hint: "The CEF files are missing or can't be loaded. Reinstall Sucrose to repair them.".to_owned(),
        },
        other => EngineError::Start {
            reason: other.to_string(),
        },
    }
}

/// The engine name of CEF's web data folders (`<data_dir>/Web/Cef`, D-35) and of its profile lock.
const PROFILE_ENGINE: &str = "Cef";

/// The profile this renderer runs on (D-33): the persistent `<data_dir>/Web/Cef` when no other running CEF renderer holds
/// it, otherwise an emptied ephemeral folder of this slot. A failure to get any folder is logged and leaves the
/// renderer on in-memory caches, so the wallpaper still plays; it just keeps nothing.
fn acquire_profile(slot: u8) -> Option<Arc<ProfileLease>> {
    let web = sucrose_core::paths::web_dir()?;
    let ephemeral = sucrose_core::paths::cache_dir()?.join("WebEphemeral");
    match profile::acquire(&web, PROFILE_ENGINE, &ephemeral, slot) {
        Ok(lease) => {
            tracing::info!(kind = ?lease.kind, engine = PROFILE_ENGINE, path = ?lease.path, slot, "web profile");
            Some(Arc::new(lease))
        }
        Err(error) => {
            tracing::warn!(%error, engine = PROFILE_ENGINE, slot, "no web profile folder; the page keeps nothing");
            None
        }
    }
}

/// Starts CEF on the calling thread, which must be the one that owns the surface `wid` and runs its message loop,
/// showing the page of the assignment. Returns the thread-safe handle and the pump the loop drives.
pub fn start(wid: u64, assignment: &Assignment, hooks: SurfaceHooks) -> Result<Started, EngineError> {
    let SurfaceHooks {
        proxy,
        timers,
        on_playing,
    } = hooks;
    let told = AtomicBool::new(false);
    let wake_proxy = proxy.clone();
    let engine_proxy = proxy;
    let theme_dir = PathBuf::from(&assignment.theme_dir);
    let start = CefStart {
        log_file: sucrose_core::paths::log_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(format!("CefRenderer-{}.log", assignment.slot)),
        subprocess_path: std::env::current_exe().map_err(|error| EngineError::Start {
            reason: format!("cannot find this program's path: {error}"),
        })?,
        wake: Arc::new(move || wake_proxy.wake()),
        roots: [
            Some(theme_dir.clone()),
            sucrose_core::paths::cache_dir().map(|dir| dir.join("Content")),
        ]
        .into_iter()
        .flatten()
        .collect(),
        // The first finished main-frame load is the first frame the user can see.
        on_loaded: Arc::new(move |_url, _status| {
            if !told.swap(true, Ordering::SeqCst) {
                on_playing();
            }
        }),
        profile: acquire_profile(assignment.slot),
    };
    // The page comes from the virtual origin (D-14): the theme folder first, then the generated content.
    let url = origin::page_url(&theme_dir, &assignment.source);
    let host = CefHost::start(wid, assignment.rect, &url, &start).map_err(|error| {
        tracing::error!(%error, "cannot start CEF");
        start_failure(&error)
    })?;
    timers.set(CEF_PUMP_TIMER, host.do_work());

    let (sender, receiver) = bounded(QUEUE_CAPACITY);
    let settings = &assignment.settings;
    let state = Arc::new(State {
        paused: AtomicBool::new(false),
        volume: AtomicU8::new(settings.volume),
        looping: AtomicBool::new(settings.looping),
        stretch: Mutex::new(settings.stretch),
    });
    Ok(Started {
        engine: Arc::new(CefEngine {
            commands: sender,
            wake: Arc::new(move || engine_proxy.wake()),
            state,
        }),
        pump: Some(Box::new(CefPump {
            host,
            commands: receiver,
            timers,
        })),
    })
}

impl CefEngine {
    /// Queues `command` for the surface thread and wakes it. A full queue drops the command (T-04-13).
    fn queue(&self, command: Command) {
        match self.commands.try_send(command) {
            Ok(()) => (self.wake)(),
            Err(TrySendError::Full(command)) => {
                tracing::warn!(?command, "the CEF command queue is full, the command is dropped");
            }
            // The pump closed the browser already; there is nobody left to tell.
            Err(TrySendError::Disconnected(command)) => tracing::debug!(?command, "the CEF pump is gone"),
        }
    }

    /// Whether the last pause command paused.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.state.paused.load(Ordering::SeqCst)
    }

    /// The last volume the engine was given, 0..=100.
    #[must_use]
    pub fn volume(&self) -> u8 {
        self.state.volume.load(Ordering::SeqCst)
    }

    /// Whether the wallpaper loops, as last set.
    #[must_use]
    pub fn looping(&self) -> bool {
        self.state.looping.load(Ordering::SeqCst)
    }

    /// How the wallpaper is fitted, as last set.
    #[must_use]
    pub fn stretch(&self) -> Stretch {
        *self.state.stretch.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Engine for CefEngine {
    fn name(&self) -> &'static str {
        "Cef"
    }

    fn set_paused(&self, paused: bool, level: PauseLevel) -> Result<(), EngineError> {
        self.state.paused.store(paused, Ordering::SeqCst);
        self.queue(Command::Pause { paused, level });
        Ok(())
    }

    fn set_volume(&self, volume: u8) -> Result<(), EngineError> {
        let volume = volume.min(100);
        self.state.volume.store(volume, Ordering::SeqCst);
        self.queue(Command::Volume(volume));
        Ok(())
    }

    fn set_loop(&self, looping: bool) -> Result<(), EngineError> {
        self.state.looping.store(looping, Ordering::SeqCst);
        self.queue(Command::Loop(looping));
        Ok(())
    }

    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        *self.state.stretch.lock().unwrap_or_else(PoisonError::into_inner) = stretch;
        self.queue(Command::Stretch(stretch));
        Ok(())
    }

    /// A running CEF renderer cannot show another page in place: the service starts a new renderer for it.
    fn load_source(&self, _source: &Path, _settings: &PlaybackSettings) -> Result<(), EngineError> {
        Err(EngineError::Unsupported {
            what: "Showing another page in a running CEF renderer".to_owned(),
        })
    }

    /// A page has no timeline to follow; web clock following is Round C.
    fn clock(&self) -> Option<&dyn EngineClock> {
        None
    }
}

impl CefPump {
    /// Applies what the IPC threads queued. The values are kept for the page calls of plan 04-09; until then CEF
    /// itself has nothing to apply, and a command is only noted.
    fn drain(&self) {
        while let Ok(command) = self.commands.try_recv() {
            tracing::debug!(?command, "queued command reached the CEF thread");
        }
    }

    /// Pumps CEF and arms the pump timer with the delay CEF asked for.
    fn pump(&self) {
        self.timers.set(CEF_PUMP_TIMER, self.host.do_work());
    }
}

impl SurfacePump for CefPump {
    fn on_wake(&mut self) {
        // A wake (CEF asked to be pumped, or a command arrived) pumps CEF.
        self.drain();
        self.pump();
    }

    fn on_timer(&mut self, id: u32) -> bool {
        if id != CEF_PUMP_TIMER {
            return false;
        }
        self.pump();
        true
    }

    fn set_rect(&mut self, rect: [i32; 4]) {
        if let Err(error) = self.host.set_rect(rect) {
            tracing::warn!(%error, "cannot move the browser window");
        }
    }

    fn close(self: Box<Self>) {
        self.timers.kill(CEF_PUMP_TIMER);
        // The browser first: its window is a child of the surface.
        self.host.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_cef_is_a_missing_engine_and_other_failures_are_start_failures() {
        let missing = start_failure(&CefError::Load("x".to_owned()));
        let EngineError::Missing { hint, .. } = missing else {
            panic!("a missing library is not Missing: {missing:?}");
        };
        assert!(hint.contains("Reinstall Sucrose"));
        assert!(matches!(
            start_failure(&CefError::CreateBrowser),
            EngineError::Start { .. }
        ));
        assert!(matches!(
            start_failure(&CefError::Unsupported),
            EngineError::Start { .. }
        ));
    }

    /// An engine handle whose pump side is the returned receiver and whose wakes are counted, so the queue can be
    /// looked at without CEF or a message loop.
    fn handle(capacity: usize) -> (CefEngine, Receiver<Command>, Arc<AtomicU8>) {
        let (sender, receiver) = bounded(capacity);
        let wakes = Arc::new(AtomicU8::new(0));
        let counter = Arc::clone(&wakes);
        let engine = CefEngine {
            commands: sender,
            wake: Arc::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }),
            state: Arc::new(State {
                paused: AtomicBool::new(false),
                volume: AtomicU8::new(100),
                looping: AtomicBool::new(true),
                stretch: Mutex::new(Stretch::default()),
            }),
        };
        (engine, receiver, wakes)
    }

    #[test]
    fn commands_reach_the_queue_in_order_and_each_wakes_the_surface_thread() {
        let (engine, queue, wakes) = handle(8);

        engine.set_paused(true, PauseLevel::Heavy).expect("pause");
        engine.set_volume(40).expect("volume");
        engine.set_loop(false).expect("loop");
        engine.set_stretch(Stretch::Fill).expect("stretch");

        let queued: Vec<Command> = queue.try_iter().collect();
        assert_eq!(
            queued,
            [
                Command::Pause {
                    paused: true,
                    level: PauseLevel::Heavy
                },
                Command::Volume(40),
                Command::Loop(false),
                Command::Stretch(Stretch::Fill),
            ]
        );
        assert_eq!(wakes.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn the_engine_keeps_the_newest_value_of_each_setting() {
        let (engine, _queue, _wakes) = handle(8);

        engine.set_paused(true, PauseLevel::Light).expect("pause");
        engine.set_volume(250).expect("volume");
        engine.set_loop(false).expect("loop");
        engine.set_stretch(Stretch::Uniform).expect("stretch");

        assert!(engine.paused());
        assert_eq!(engine.volume(), 100, "the volume is capped at 100");
        assert!(!engine.looping());
        assert_eq!(engine.stretch(), Stretch::Uniform);
    }

    #[test]
    fn a_full_queue_drops_the_command_instead_of_blocking_the_caller() {
        let (engine, queue, wakes) = handle(1);

        engine.set_volume(10).expect("first");
        engine.set_volume(20).expect("second is dropped, not blocked");

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Volume(10)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1, "a dropped command wakes nothing");
        assert_eq!(engine.volume(), 20, "the state still holds the newest value");
    }

    #[test]
    fn a_command_after_the_pump_is_gone_is_not_an_error() {
        let (engine, queue, _wakes) = handle(4);
        drop(queue);

        assert!(engine.set_volume(5).is_ok());
    }

    #[test]
    fn a_page_cannot_be_replaced_in_place_and_there_is_no_clock() {
        let (engine, _queue, _wakes) = handle(4);

        let result = engine.load_source(Path::new("C:/page/index.html"), &PlaybackSettings::default());

        assert!(matches!(result, Err(EngineError::Unsupported { .. })), "{result:?}");
        assert!(engine.clock().is_none());
        assert_eq!(engine.name(), "Cef");
    }
}
