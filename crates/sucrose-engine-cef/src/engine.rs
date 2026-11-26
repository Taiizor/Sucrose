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
use serde_json::Value;
use sucrose_core::{
    compatible::{COMPATIBLE_FILE, CompatibleKey, DataGroup, read_compatible},
    properties::PropertiesError,
    settings::Stretch,
};
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started, SurfacePump},
};
use sucrose_engine_web::{
    args::{self, SucroseInputs, WebEngine},
    js::{self, CompatibleState, PageCall},
    origin,
    profile::{self, ProfileLease},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings, WebOptions};
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
    /// Called once, from the surface thread, after the first page loaded with the system data groups its
    /// `SucroseCompatible.json` needs; never called with an empty list (D-24).
    pub on_data_wanted: Arc<dyn Fn(Vec<DataGroup>) + Send + Sync>,
}

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
    /// A data snapshot arrived: run the templates with the newest values (kept in [`State::data`]).
    Data,
}

/// The newest value of each setting, for the page calls and for anyone asking what the engine was told.
#[derive(Debug)]
struct State {
    paused: AtomicBool,
    volume: AtomicU8,
    looping: AtomicBool,
    stretch: Mutex<Stretch>,
    /// `Engine.WallpaperShuffle`: the value of the `ShuffleMode` call, as the assignment set it and the Portal's Shuffle
    /// Mode switch changed it since.
    shuffle: AtomicBool,
    /// The legacy `ThemeType` name the `ThemeType` and `SystemTheme` calls carry, fixed for the life of the renderer.
    theme_type: String,
    /// The newest snapshot of each system data group the service pushed.
    data: Mutex<Vec<(DataGroup, Value)>>,
}

impl State {
    /// The state a renderer starts with: not paused, the playback settings and the web options of its assignment.
    fn new(settings: &PlaybackSettings, web: &WebOptions) -> Self {
        Self {
            paused: AtomicBool::new(false),
            volume: AtomicU8::new(settings.volume.min(100)),
            looping: AtomicBool::new(settings.looping),
            stretch: Mutex::new(settings.stretch),
            shuffle: AtomicBool::new(web.shuffle),
            theme_type: web.theme_type.clone(),
            data: Mutex::new(Vec::new()),
        }
    }

    /// What the non-data compatible calls are built from, as the engine was last told (WAPI-02).
    fn compatible(&self) -> CompatibleState {
        CompatibleState {
            looping: self.looping.load(Ordering::SeqCst),
            shuffle: self.shuffle.load(Ordering::SeqCst),
            stretch: *self.stretch.lock().unwrap_or_else(PoisonError::into_inner),
            theme_type: self.theme_type.clone(),
            volume: self.volume.load(Ordering::SeqCst),
        }
    }
}

/// Which theme files were already reported as unusable, so each is logged once per renderer.
#[derive(Debug, Default)]
struct Complaints {
    properties: AtomicBool,
    compatible: AtomicBool,
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

/// A reason for a log line that names what is wrong with a theme file without its path or its text (D-08).
fn describe(error: &PropertiesError) -> String {
    match error {
        PropertiesError::Missing(_) => "the file is missing".to_owned(),
        PropertiesError::Unreadable { source, .. } => format!("the file cannot be read ({:?})", source.kind()),
        PropertiesError::Invalid { reason, .. } => reason.clone(),
    }
}

/// The scripts a page gets after its main frame loaded, in the order they must run (WAPI-01, WAPI-02): the property
/// calls of `SucroseProperties.json`, one per control in file order; then the non-data compatible calls of
/// `SucroseCompatible.json` in the legacy run order, so they come after every property call and before the first data
/// push; then, when the renderer was paused before the page finished loading, the pause. Only a loaded page (HTTP 200)
/// gets them; a theme without a file or with nothing in it gets none, and a file that cannot be used is logged once
/// per renderer (`complaints`) while the page still plays. A reload runs this again, so every key is sent again.
fn page_calls(theme_dir: &Path, status: i32, state: &State, complaints: &Complaints) -> Vec<PageCall> {
    if status != 200 {
        tracing::debug!(status, "no page calls: the main frame did not load with status 200");
        return Vec::new();
    }
    let mut calls = match js::property_calls(theme_dir) {
        Ok(calls) => calls,
        Err(error) => {
            if !complaints.properties.swap(true, Ordering::SeqCst) {
                tracing::warn!(reason = %describe(&error), "SucroseProperties.json cannot be used, the page gets no property calls");
            }
            Vec::new()
        }
    };
    match js::compatible_calls(theme_dir, &state.compatible()) {
        Ok(compatible) => calls.extend(compatible),
        Err(error) => {
            if !complaints.compatible.swap(true, Ordering::SeqCst) {
                tracing::warn!(reason = %describe(&error), "SucroseCompatible.json cannot be used, the page gets no compatible calls");
            }
        }
    }
    if state.paused.load(Ordering::SeqCst) {
        calls.push(js::pause_call(true));
    }
    calls
}

/// The system data groups the theme's `SucroseCompatible.json` asks for, in the legacy payload order; empty for a theme
/// without the file, with nothing in it or with a file that cannot be used (`page_calls` reports that), and for a page
/// that did not load with status 200.
fn wanted_groups(theme_dir: &Path, status: i32) -> Vec<DataGroup> {
    if status != 200 {
        return Vec::new();
    }
    read_compatible(&theme_dir.join(COMPATIBLE_FILE)).map_or_else(|_| Vec::new(), |compatible| compatible.data_groups())
}

/// The non-empty templates of the theme's `SucroseCompatible.json`; none when the file is missing or unusable.
fn load_templates(theme_dir: &Path) -> Vec<(CompatibleKey, String)> {
    read_compatible(&theme_dir.join(COMPATIBLE_FILE)).map_or_else(
        |_| Vec::new(),
        |compatible| {
            compatible
                .templates()
                .into_iter()
                .map(|(key, template)| (key, template.to_owned()))
                .collect()
        },
    )
}

/// The calls of one data push: the templates run in the legacy order with the newest value of everything (the legacy
/// engine runs all of them for every message it receives).
fn push_calls(templates: &[(CompatibleKey, String)], state: &State) -> Vec<PageCall> {
    let templates: Vec<(CompatibleKey, &str)> = templates.iter().map(|(key, text)| (*key, text.as_str())).collect();
    let data = state.data.lock().unwrap_or_else(PoisonError::into_inner).clone();
    js::push_calls(&templates, &state.compatible(), &data)
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
        page_calls: Arc::new(move |_url, status| page_calls(&calls_dir, status, &calls_state, &complaints)),
        // The first finished main-frame load is the first frame the user can see.
        on_loaded: Arc::new(move |_url, status| {
            if !told.swap(true, Ordering::SeqCst) {
                on_playing();
            }
            // Once per renderer, after the page and its calls: a reload keeps the data it already asked for.
            if status == 200 && !asked.swap(true, Ordering::SeqCst) {
                let groups = wanted_groups(&wanted_dir, status);
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
    // The page comes from the virtual origin (D-14): the theme folder first, then the generated content.
    let url = origin::page_url(&theme_dir, &assignment.source);
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

    /// The value of the `ShuffleMode` call, as last set.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.state.shuffle.load(Ordering::SeqCst)
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

    /// The page's `ShuffleMode` call follows with the next compatible push, which the command triggers.
    fn set_shuffle(&self, shuffle: bool) {
        self.state.shuffle.store(shuffle, Ordering::SeqCst);
        self.queue(Command::Shuffle(shuffle));
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

    /// DevTools open in a window of their own on the surface thread, only when asked (D-13).
    fn open_devtools(&self) -> Result<(), EngineError> {
        self.queue(Command::OpenDevTools);
        Ok(())
    }

    /// Keeps the newest snapshot of each group and queues one push for the surface thread. A text that is not JSON is
    /// dropped with a log line and the group keeps its last value.
    fn push_data(&self, data: Vec<(DataGroup, String)>) {
        {
            let mut stored = self.state.data.lock().unwrap_or_else(PoisonError::into_inner);
            for (group, text) in data {
                let value = match serde_json::from_str::<Value>(&text) {
                    Ok(value) => value,
                    Err(error) => {
                        tracing::warn!(?group, %error, "a data snapshot is not valid JSON and is dropped");
                        continue;
                    }
                };
                match stored.iter_mut().find(|(known, _)| *known == group) {
                    Some(slot) => slot.1 = value,
                    None => stored.push((group, value)),
                }
            }
        }
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
            tracing::debug!(?command, "queued command reached the CEF thread");
            match command {
                Command::Data => {}
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

    /// Runs the templates with the newest data in the page: the non-data keys again and each data key that has a snapshot.
    fn push_data(&mut self) {
        let templates = self.templates.get_or_insert_with(|| load_templates(&self.theme_dir));
        for call in push_calls(templates, &self.state) {
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
            state: Arc::new(State::new(&PlaybackSettings::default(), &WebOptions::default())),
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
    fn a_shuffle_change_is_kept_by_the_engine_queued_for_the_surface_thread_and_part_of_the_compatible_state() {
        let (engine, queue, wakes) = handle(8);
        assert!(engine.shuffle(), "the legacy default is on");

        engine.set_shuffle(false);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Shuffle(false)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert!(!engine.shuffle());
        assert!(
            !engine.state.compatible().shuffle,
            "the ShuffleMode call carries the new value"
        );
    }

    #[test]
    fn a_page_gets_the_new_shuffle_value_with_the_next_compatible_push() {
        let theme = Theme::new(None, Some(r#"{"ShuffleMode":"S('{0}');"}"#));
        let (engine, _queue, _wakes) = handle(8);
        let templates = load_templates(&theme.0);
        assert_eq!(push_calls(&templates, &engine.state)[0].script, "S('True');");

        engine.set_shuffle(false);

        assert_eq!(push_calls(&templates, &engine.state)[0].script, "S('False');");
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

    /// A scratch theme folder with the given `SucroseProperties.json` and `SucroseCompatible.json` texts (none: no
    /// file), removed on drop.
    struct Theme(PathBuf);

    impl Theme {
        fn new(properties: Option<&str>, compatible: Option<&str>) -> Self {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "sucrose-cef-calls-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create scratch theme");
            for (name, text) in [
                ("SucroseProperties.json", properties),
                ("SucroseCompatible.json", compatible),
            ] {
                if let Some(text) = text {
                    std::fs::write(dir.join(name), text).expect("write theme file");
                }
            }
            Self(dir)
        }
    }

    impl Drop for Theme {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const TWO_PROPERTIES: &str = r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{"b":{"type":"checkbox","value":true},"a":{"type":"textbox","value":"x"}}}"#;
    const COMPATIBLE: &str =
        r#"{"VolumeLevel":"V({0});","SystemMemory":"M({0});","LoopMode":"L('{0}');","StretchMode":"S('{0}');"}"#;

    fn fresh_state() -> State {
        State::new(&PlaybackSettings::default(), &WebOptions::default())
    }

    fn keys(calls: &[PageCall]) -> Vec<String> {
        calls.iter().map(|call| format!("{}:{}", call.kind, call.key)).collect()
    }

    #[test]
    fn a_loaded_page_gets_its_property_calls_in_file_order_and_a_failed_load_gets_none() {
        let theme = Theme::new(Some(TWO_PROPERTIES), None);
        let (state, complaints) = (fresh_state(), Complaints::default());

        let calls = page_calls(&theme.0, 200, &state, &complaints);
        let failed = page_calls(&theme.0, 404, &state, &complaints);

        assert_eq!(keys(&calls), ["property:b", "property:a"]);
        assert!(failed.is_empty(), "only a page that loaded with status 200 gets calls");
        assert!(!complaints.properties.load(Ordering::SeqCst));
    }

    #[test]
    fn the_compatible_calls_follow_every_property_call_in_the_legacy_order_without_the_data_keys() {
        let theme = Theme::new(Some(TWO_PROPERTIES), Some(COMPATIBLE));
        let (state, complaints) = (fresh_state(), Complaints::default());

        let calls = page_calls(&theme.0, 200, &state, &complaints);

        assert_eq!(
            keys(&calls),
            [
                "property:b",
                "property:a",
                "compatible:LoopMode",
                "compatible:StretchMode",
                "compatible:VolumeLevel"
            ]
        );
        let scripts: Vec<&str> = calls.iter().skip(2).map(|call| call.script.as_str()).collect();
        assert_eq!(scripts, ["L('True');", "S('UniformToFill');", "V(100);"]);
    }

    #[test]
    fn the_compatible_calls_use_the_values_the_engine_was_last_told() {
        let theme = Theme::new(None, Some(COMPATIBLE));
        let state = fresh_state();
        *state.stretch.lock().expect("lock") = Stretch::Fill;
        state.looping.store(false, Ordering::SeqCst);
        state.volume.store(40, Ordering::SeqCst);

        let calls = page_calls(&theme.0, 200, &state, &Complaints::default());

        let scripts: Vec<&str> = calls.iter().map(|call| call.script.as_str()).collect();
        assert_eq!(scripts, ["L('False');", "S('Fill');", "V(40);"]);
    }

    #[test]
    fn a_reload_sends_every_key_again() {
        let theme = Theme::new(Some(TWO_PROPERTIES), Some(COMPATIBLE));
        let (state, complaints) = (fresh_state(), Complaints::default());

        let first = page_calls(&theme.0, 200, &state, &complaints);
        let reload = page_calls(&theme.0, 200, &state, &complaints);

        assert_eq!(first, reload);
        assert_eq!(reload.len(), 5);
    }

    #[test]
    fn a_page_that_loads_while_paused_is_paused_after_its_calls() {
        let theme = Theme::new(Some(TWO_PROPERTIES), None);
        let state = fresh_state();
        state.paused.store(true, Ordering::SeqCst);

        let calls = page_calls(&theme.0, 200, &state, &Complaints::default());

        assert_eq!(keys(&calls), ["property:b", "property:a", "pause:pause"]);
        assert_eq!(calls[2].script, js::pause_script(true));
    }

    #[test]
    fn a_theme_without_files_gets_no_calls_and_no_complaint() {
        let theme = Theme::new(None, None);
        let (state, complaints) = (fresh_state(), Complaints::default());

        assert!(page_calls(&theme.0, 200, &state, &complaints).is_empty());
        assert!(!complaints.properties.load(Ordering::SeqCst) && !complaints.compatible.load(Ordering::SeqCst));
    }

    #[test]
    fn an_unparsable_file_gives_no_calls_for_it_and_is_reported_once_while_the_other_still_runs() {
        let theme = Theme::new(Some("{ nope"), Some(COMPATIBLE));
        let (state, complaints) = (fresh_state(), Complaints::default());

        let calls = page_calls(&theme.0, 200, &state, &complaints);

        assert_eq!(
            keys(&calls),
            [
                "compatible:LoopMode",
                "compatible:StretchMode",
                "compatible:VolumeLevel"
            ],
            "the page keeps playing and gets the other file's calls"
        );
        assert!(
            complaints.properties.load(Ordering::SeqCst),
            "the first failure is the one that is logged"
        );
        assert!(!complaints.compatible.load(Ordering::SeqCst));
        assert_eq!(page_calls(&theme.0, 200, &state, &complaints).len(), 3);
    }

    #[test]
    fn the_state_starts_from_the_assignment_and_reports_the_legacy_theme_name() {
        let web = WebOptions {
            shuffle: false,
            theme_type: "Light".to_owned(),
            ..WebOptions::default()
        };
        let state = State::new(&PlaybackSettings::default(), &web);

        let compatible = state.compatible();

        assert!(!compatible.shuffle);
        assert_eq!(compatible.theme_type, "Light");
        assert!(compatible.looping);
        assert_eq!(compatible.volume, 100);
    }

    #[test]
    fn the_log_reason_names_the_problem_without_a_path() {
        let missing = describe(&PropertiesError::Missing(PathBuf::from("C:/secret/x.json")));
        let invalid = describe(&PropertiesError::Invalid {
            path: PathBuf::from("C:/secret/x.json"),
            reason: "invalid JSON at line 1 column 3".to_owned(),
        });

        assert!(!missing.contains("secret") && !invalid.contains("secret"));
        assert!(invalid.contains("line 1"));
    }

    const WITH_DATA: &str =
        r#"{"LoopMode":"L('{0}');","SystemDate":"D({0});","SystemMemory":"M({0});","SystemBattery":""}"#;

    #[test]
    fn the_wanted_groups_are_the_data_templates_of_a_page_that_loaded() {
        let theme = Theme::new(None, Some(WITH_DATA));

        assert_eq!(wanted_groups(&theme.0, 200), [DataGroup::Date, DataGroup::Memory]);
        assert!(
            wanted_groups(&theme.0, 404).is_empty(),
            "a page that did not load wants nothing"
        );
        assert!(
            wanted_groups(
                &Theme::new(None, Some(COMPATIBLE.replace("SystemMemory", "Nope").as_str())).0,
                200
            )
            .is_empty()
        );
        assert!(
            wanted_groups(&Theme::new(Some(TWO_PROPERTIES), None).0, 200).is_empty(),
            "no file, no data"
        );
        assert!(
            wanted_groups(&Theme::new(None, Some("{ nope")).0, 200).is_empty(),
            "an unusable file wants nothing"
        );
    }

    #[test]
    fn a_data_push_runs_every_template_in_the_legacy_order_with_the_newest_snapshots() {
        let theme = Theme::new(None, Some(WITH_DATA));
        let (engine, queue, wakes) = handle(8);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Data, Command::Data]);
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        let templates = load_templates(&theme.0);
        let calls = push_calls(&templates, &engine.state);
        assert_eq!(
            keys(&calls),
            ["compatible:LoopMode", "compatible:SystemDate"],
            "Memory has no snapshot yet"
        );
        assert_eq!(calls[0].script, "L('True');");
        assert_eq!(
            calls[1].script, "D({\r\n  \"Day\": 2\r\n});",
            "the newest snapshot of the group"
        );
    }

    #[test]
    fn a_snapshot_that_is_not_json_is_dropped_and_the_group_keeps_its_last_value() {
        let (engine, _queue, _wakes) = handle(8);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);

        engine.push_data(vec![(DataGroup::Date, "not json".to_owned())]);

        let stored = engine.state.data.lock().expect("lock").clone();
        assert_eq!(stored, [(DataGroup::Date, serde_json::json!({"Day": 1}))]);
    }

    #[test]
    fn a_full_queue_does_not_lose_the_newest_snapshot() {
        let (engine, queue, _wakes) = handle(1);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().count(), 1, "the second marker was dropped");
        let stored = engine.state.data.lock().expect("lock").clone();
        assert_eq!(stored, [(DataGroup::Date, serde_json::json!({"Day": 2}))]);
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
