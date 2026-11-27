//! CEF behind the [`Engine`] trait. CEF is bound to the thread that owns the desktop surface, so the engine is split in
//! two: [`CefEngine`] is the thread-safe handle the IPC and clock threads call, and it only queues commands and wakes
//! the surface thread; [`CefPump`] owns the CEF host, drains the queue and pumps CEF on the surface thread. No CEF
//! function is called from another thread.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded};
use sucrose_core::{
    compatible::{CompatibleKey, DataGroup},
    settings::Stretch,
};
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started, SurfacePump},
};
use sucrose_engine_web::{
    args::{self, SucroseInputs, WebEngine},
    js, origin,
    page::{self, Complaints, PageState},
    pages,
    profile::{self, ProfileLease},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings, WebOptions};
use sucrose_platform::{
    LoopTimers,
    input::{InputEvent, focus_click_messages, key_message, mouse_message},
};

use crate::browser::{CefError, CefHost, CefStart};

/// The timer id of the CEF message pump (D-036): the only timer a CEF renderer sets, armed with the delay
/// [`CefHost::do_work`] returns after each pump call.
pub const CEF_PUMP_TIMER: u32 = 1;

/// How many commands may wait for the surface thread. A full queue drops the newest command and logs it, so the IPC
/// thread never blocks (T-04-13); the engine's state values below always hold the latest of each.
const QUEUE_CAPACITY: usize = 64;

/// What the renderer gives the engine of the surface thread it runs on: it moved to `sucrose_engine` so every engine that
/// runs on the surface thread takes the same hooks.
pub use sucrose_engine::engine::SurfaceHooks;

/// A command the IPC threads queue for the surface thread.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    Pause {
        paused: bool,
        level: PauseLevel,
    },
    Volume(u8),
    Loop(bool),
    /// The value of the `ShuffleMode` call changed (PLAY-01).
    Shuffle(bool),
    Stretch(Stretch),
    /// Show DevTools: the Portal's Open DevTools button (D-13).
    OpenDevTools,
    /// A data snapshot arrived: run the templates with the newest values (kept in the [`PageState`]).
    Data,
    /// A mouse move is waiting in [`State::pending_move`]; one marker covers a whole burst of moves.
    MouseMove,
    /// A button, wheel or key event to hand to the page.
    Input(InputEvent),
}

/// What the engine of a renderer keeps: the page state both web engines share and the newest mouse move not yet
/// handed to the page.
#[derive(Debug)]
struct State {
    /// The newest value of each setting and system data group, for the page calls and for anyone asking what the engine
    /// was told.
    page: PageState,
    /// The newest mouse move not yet handed to the page: a mouse moves a thousand times a second, so moves are
    /// coalesced here instead of filling the command queue.
    pending_move: Mutex<Option<InputEvent>>,
}

impl State {
    /// The state a renderer starts with: not paused, the playback settings and the web options of its assignment.
    fn new(settings: &PlaybackSettings, web: &WebOptions) -> Self {
        Self {
            page: PageState::new(
                false,
                settings.volume,
                settings.looping,
                web.shuffle,
                settings.stretch,
                web.theme_type.clone(),
            ),
            pending_move: Mutex::new(None),
        }
    }
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
    state: Arc<State>,
    theme_dir: PathBuf,
    /// The non-empty templates of the theme, read when the first snapshot arrives.
    templates: Option<Vec<(CompatibleKey, String)>>,
    /// The left and top of the surface on the virtual screen: forwarded positions are made relative to it.
    origin: (i32, i32),
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

/// The debugging port of the renderer of `slot`: the configured `DeveloperPort` plus the slot, so two renderers never
/// bind the same port (Pitfall 11). A port outside the range CEF accepts is logged and left off, never wrapped around.
fn debugging_port(developer_port: u16, slot: u8) -> Option<u16> {
    match args::debugging_port(developer_port, slot) {
        Ok(port) => port,
        Err(error) => {
            tracing::warn!(%error, slot, "no remote debugging port for this renderer");
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
        on_data_wanted,
    } = hooks;
    let told = AtomicBool::new(false);
    let asked = AtomicBool::new(false);
    let complaints = Complaints::default();
    let wake_proxy = proxy.clone();
    let engine_proxy = proxy;
    let theme_dir = PathBuf::from(&assignment.theme_dir);
    let calls_dir = theme_dir.clone();
    let wanted_dir = theme_dir.clone();
    let web = &assignment.web;
    let state = Arc::new(State::new(&assignment.settings, web));
    let calls_state = Arc::clone(&state);
    let debugging_port = debugging_port(web.developer_port, assignment.slot);
    if let Some(port) = debugging_port {
        tracing::info!(port, slot = assignment.slot, "remote debugging");
    }
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
        page_calls: Arc::new(move |_url, status| page::page_calls(&calls_dir, status, &calls_state.page, &complaints)),
        // The first finished main-frame load is the first frame the user can see.
        on_loaded: Arc::new(move |_url, status| {
            if !told.swap(true, Ordering::SeqCst) {
                on_playing();
            }
            // Once per renderer, after the page and its calls: a reload keeps the data it already asked for.
            if status == 200 && !asked.swap(true, Ordering::SeqCst) {
                let groups = page::wanted_groups(&wanted_dir, status);
                if !groups.is_empty() {
                    tracing::info!(groups = ?groups, "data wanted");
                    on_data_wanted(groups);
                }
            }
        }),
        profile: acquire_profile(assignment.slot),
        allow_invalid_certificates: web.allow_invalid_certificates,
        arguments: web.arguments.clone(),
        sucrose: SucroseInputs {
            hardware_acceleration: web.hardware_acceleration,
            debugging_port,
            windows_build: sucrose_platform::os_build(),
            engine: WebEngine::Cef,
        },
        developer_mode: web.developer_mode,
    };
    // The page comes from the virtual origin (D-14): the theme folder first, then the generated content. A Gif or Video
    // theme plays through a generated page that is written into the content folder.
    let url = match pages::prepare(assignment.kind, &theme_dir, &assignment.source) {
        Ok(Some(url)) => url,
        Ok(None) => origin::page_url(&theme_dir, &assignment.source),
        Err(error) => {
            tracing::error!(%error, kind = ?assignment.kind, "cannot prepare the generated page");
            return Err(EngineError::Start {
                reason: error.to_string(),
            });
        }
    };
    let host = CefHost::start(wid, assignment.rect, &url, &start).map_err(|error| {
        tracing::error!(%error, "cannot start CEF");
        start_failure(&error)
    })?;
    timers.set(CEF_PUMP_TIMER, host.do_work());

    let (sender, receiver) = bounded(QUEUE_CAPACITY);
    Ok(Started {
        engine: Arc::new(CefEngine {
            commands: sender,
            wake: Arc::new(move || engine_proxy.wake()),
            state: Arc::clone(&state),
        }),
        pump: Some(Box::new(CefPump {
            host,
            commands: receiver,
            timers,
            state,
            theme_dir,
            templates: None,
            origin: (assignment.rect[0], assignment.rect[1]),
        })),
    })
}

impl CefEngine {
    /// Queues `command` for the surface thread and wakes it. A full queue drops the command (T-04-13).
    fn queue(&self, command: Command) {
        match self.commands.try_send(command) {
            Ok(()) => (self.wake)(),
            // The snapshot itself is kept in the state, so a dropped marker loses nothing: the next one runs it.
            Err(TrySendError::Full(Command::Data)) => {
                tracing::debug!("the CEF command queue is full, a data push waits")
            }
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
        self.state.page.paused()
    }

    /// The last volume the engine was given, 0..=100.
    #[must_use]
    pub fn volume(&self) -> u8 {
        self.state.page.volume()
    }

    /// Whether the wallpaper loops, as last set.
    #[must_use]
    pub fn looping(&self) -> bool {
        self.state.page.looping()
    }

    /// The value of the `ShuffleMode` call, as last set.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.state.page.shuffle()
    }

    /// How the wallpaper is fitted, as last set.
    #[must_use]
    pub fn stretch(&self) -> Stretch {
        self.state.page.stretch()
    }
}

impl Engine for CefEngine {
    fn name(&self) -> &'static str {
        "Cef"
    }

    fn set_paused(&self, paused: bool, level: PauseLevel) -> Result<(), EngineError> {
        self.state.page.set_paused(paused);
        self.queue(Command::Pause { paused, level });
        Ok(())
    }

    fn set_volume(&self, volume: u8) -> Result<(), EngineError> {
        let volume = volume.min(100);
        self.state.page.set_volume(volume);
        self.queue(Command::Volume(volume));
        Ok(())
    }

    fn set_loop(&self, looping: bool) -> Result<(), EngineError> {
        self.state.page.set_looping(looping);
        self.queue(Command::Loop(looping));
        Ok(())
    }

    /// The page's `ShuffleMode` call follows with the next compatible push, which the command triggers.
    fn set_shuffle(&self, shuffle: bool) {
        self.state.page.set_shuffle(shuffle);
        self.queue(Command::Shuffle(shuffle));
    }

    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        self.state.page.set_stretch(stretch);
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

    /// DevTools open in a window of their own on the surface thread, only when asked (D-13).
    fn open_devtools(&self) -> Result<(), EngineError> {
        self.queue(Command::OpenDevTools);
        Ok(())
    }

    fn takes_input(&self) -> bool {
        true
    }

    /// Keeps the newest mouse move and queues one marker for a burst of them; every other event is queued as it is. A
    /// full queue drops the event without a word about its content (T-04-45).
    fn forward_input(&self, event: &InputEvent) {
        if let InputEvent::MouseMove { .. } = event {
            let first = self
                .state
                .pending_move
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .replace(*event)
                .is_none();
            if first {
                match self.commands.try_send(Command::MouseMove) {
                    Ok(()) => (self.wake)(),
                    Err(_) => {
                        // Nobody will take it: the next move must queue a marker again.
                        *self.state.pending_move.lock().unwrap_or_else(PoisonError::into_inner) = None;
                    }
                }
            }
            return;
        }
        match self.commands.try_send(Command::Input(*event)) {
            Ok(()) => (self.wake)(),
            Err(_) => tracing::debug!("the CEF command queue is full or closed, an input event is dropped"),
        }
    }

    /// Keeps the newest snapshot of each group and queues one push for the surface thread. A text that is not JSON is
    /// dropped with a log line and the group keeps its last value.
    fn push_data(&self, data: Vec<(DataGroup, String)>) {
        self.state.page.store_data(data);
        self.queue(Command::Data);
    }
}

impl CefPump {
    /// Applies what the IPC threads queued. A pause or resume runs inside the page (PLAY-05, D-22): Light and Heavy are
    /// alike until Round D adds the process suspension, and no process is suspended here. The volume, loop, shuffle and
    /// stretch values were kept by the handle and the pages get them with the compatible calls. A changed loop or shuffle
    /// value runs those calls again at once (PLAY-01), the same push a data snapshot triggers.
    fn drain(&mut self) {
        // Several data markers in one wake run once: the state holds the newest snapshot of each group.
        let mut data_pushed = false;
        while let Ok(command) = self.commands.try_recv() {
            if command == Command::Data {
                data_pushed = true;
                continue;
            }
            // Input is never logged: it is what the user types and moves (T-04-45).
            match command {
                Command::MouseMove => {
                    let moved = self
                        .state
                        .pending_move
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .take();
                    if let Some(event) = moved {
                        self.post_input(&event);
                    }
                    continue;
                }
                Command::Input(event) => {
                    self.post_input(&event);
                    continue;
                }
                _ => {}
            }
            tracing::debug!(?command, "queued command reached the CEF thread");
            match command {
                Command::Data | Command::MouseMove | Command::Input(_) => {}
                Command::OpenDevTools => self.host.open_devtools(),
                Command::Pause { paused, .. } => {
                    let call = js::pause_call(paused);
                    tracing::debug!(kind = call.kind, key = ?call.key, "page call");
                    self.host.execute_script(&call.script);
                }
                Command::Loop(_) | Command::Shuffle(_) => data_pushed = true,
                Command::Volume(_) | Command::Stretch(_) => {}
            }
        }
        if data_pushed {
            self.push_data();
        }
    }

    /// Hands one input event to the page: the legacy message to the Chromium widget, positions relative to the
    /// surface.
    fn post_input(&mut self, event: &InputEvent) {
        match *event {
            // The wheel goes through the browser host, as in the legacy CefSharp engine.
            InputEvent::Wheel { delta, x, y } => {
                self.host
                    .send_mouse_wheel(x.saturating_sub(self.origin.0), y.saturating_sub(self.origin.1), delta);
            }
            InputEvent::FocusClick => {
                for (message, wparam, lparam) in focus_click_messages() {
                    self.host.post_to_widget(message, wparam, lparam);
                }
            }
            _ => {
                if let Some((message, wparam, lparam)) =
                    mouse_message(event, self.origin).or_else(|| key_message(event))
                {
                    self.host.post_to_widget(message, wparam, lparam);
                }
            }
        }
    }

    /// Runs the templates with the newest data in the page: the non-data keys again and each data key that has a snapshot.
    fn push_data(&mut self) {
        let templates = self
            .templates
            .get_or_insert_with(|| page::load_templates(&self.theme_dir));
        for call in page::push_calls(templates, &self.state.page) {
            self.host.execute_script(&call.script);
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
        self.origin = (rect[0], rect[1]);
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
    use std::sync::atomic::AtomicU8;

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
            state: Arc::new(State::new(&PlaybackSettings::default(), &WebOptions::default())),
        };
        (engine, receiver, wakes)
    }

    #[test]
    fn a_burst_of_mouse_moves_queues_one_marker_and_keeps_the_newest_position() {
        let (engine, queue, wakes) = handle(4);

        for step in 0..100 {
            engine.forward_input(&InputEvent::MouseMove { x: step, y: 2 * step });
        }

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::MouseMove]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert_eq!(
            *engine.state.pending_move.lock().expect("lock"),
            Some(InputEvent::MouseMove { x: 99, y: 198 })
        );
    }

    #[test]
    fn a_move_that_found_the_queue_full_does_not_block_later_moves() {
        let (engine, queue, _) = handle(1);
        engine.queue(Command::Volume(1));

        engine.forward_input(&InputEvent::MouseMove { x: 1, y: 1 });
        assert_eq!(
            *engine.state.pending_move.lock().expect("lock"),
            None,
            "nothing will take it"
        );

        queue.try_iter().for_each(drop);
        engine.forward_input(&InputEvent::MouseMove { x: 2, y: 2 });
        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::MouseMove]);
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
    fn a_shuffle_change_is_kept_by_the_engine_queued_for_the_surface_thread_and_part_of_the_compatible_state() {
        let (engine, queue, wakes) = handle(8);
        assert!(engine.shuffle(), "the legacy default is on");

        engine.set_shuffle(false);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Shuffle(false)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert!(!engine.shuffle());
        assert!(
            !engine.state.page.compatible().shuffle,
            "the ShuffleMode call carries the new value"
        );
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
    fn open_devtools_queues_a_command_for_the_surface_thread_and_wakes_it() {
        let (engine, queue, wakes) = handle(4);

        engine.open_devtools().expect("open devtools");

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::OpenDevTools]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn each_renderer_slot_gets_its_own_debugging_port_and_zero_means_none() {
        assert_eq!(debugging_port(0, 5), None, "DeveloperPort 0 is no debugging at all");
        assert_eq!(debugging_port(9222, 0), Some(9222));
        assert_eq!(debugging_port(9222, 1), Some(9223));
        assert_eq!(
            debugging_port(65535, 1),
            None,
            "a sum above 65535 is refused, not wrapped"
        );
    }

    #[test]
    fn a_command_after_the_pump_is_gone_is_not_an_error() {
        let (engine, queue, _wakes) = handle(4);
        drop(queue);

        assert!(engine.set_volume(5).is_ok());
    }

    #[test]
    fn a_data_push_keeps_the_newest_snapshot_and_queues_one_marker_each() {
        let (engine, queue, wakes) = handle(8);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Data, Command::Data]);
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        assert_eq!(
            engine.state.page.data(),
            [(DataGroup::Date, serde_json::json!({"Day": 2}))],
            "the newest snapshot of the group"
        );
    }

    #[test]
    fn a_full_queue_does_not_lose_the_newest_snapshot() {
        let (engine, queue, _wakes) = handle(1);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().count(), 1, "the second marker was dropped");
        assert_eq!(
            engine.state.page.data(),
            [(DataGroup::Date, serde_json::json!({"Day": 2}))]
        );
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
