//! WebView2 behind the [`Engine`] trait. WebView2 is bound to the thread that owns the desktop surface, so the engine
//! is split in two, like CEF: [`WebView2Engine`] is the thread-safe handle the IPC and clock threads call, and it only
//! queues commands and wakes the surface thread; [`WebView2Pump`] owns the host, drains the queue and moves the
//! controller on the surface thread. No WebView2 function is called from another thread.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
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
    engine::{Engine, EngineClock, EngineError, Started, SurfaceHooks, SurfacePump},
};
use sucrose_engine_web::{
    args::{self, SucroseInputs, WebEngine},
    js, origin,
    page::{self, Complaints, PageState},
    profile::{self, ProfileLease},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings, WebOptions};

use crate::host::{Host, HostError, HostStart};

/// How many commands may wait for the surface thread. A full queue drops the newest command and logs it, so the IPC
/// thread never blocks (T-04-13); the engine's state values below always hold the latest of each.
const QUEUE_CAPACITY: usize = 64;

/// The engine name of WebView2's web data folders (`<data_dir>/Web/WebView2`, D-35) and of its profile lock.
const PROFILE_ENGINE: &str = "WebView2";

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
    /// A data snapshot arrived: run the templates with the newest values (kept in the [`PageState`]).
    Data,
    /// The Portal's Open DevTools button (D-13).
    OpenDevTools,
}

/// The state a renderer starts with: not paused, the playback settings and the web options of its assignment.
fn initial_state(settings: &PlaybackSettings, theme_type: &str, shuffle: bool) -> PageState {
    PageState::new(
        false,
        settings.volume,
        settings.looping,
        shuffle,
        settings.stretch,
        theme_type,
    )
}

/// The thread-safe handle of a WebView2 renderer.
pub struct WebView2Engine {
    commands: Sender<Command>,
    /// Wakes the surface thread's message loop; callable from any thread.
    wake: Arc<dyn Fn() + Send + Sync>,
    state: Arc<PageState>,
}

impl std::fmt::Debug for WebView2Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebView2Engine")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

/// The surface-thread half of a WebView2 renderer: owns the controller and is the only thing that calls WebView2.
pub struct WebView2Pump {
    host: Host,
    commands: Receiver<Command>,
    state: Arc<PageState>,
    theme_dir: PathBuf,
    /// The non-empty templates of the theme, read when the first snapshot arrives.
    templates: Option<Vec<(CompatibleKey, String)>>,
    /// Held until the host is closed: the profile folder is free for the next instance only once WebView2 no longer
    /// uses it.
    profile: Option<Arc<ProfileLease>>,
    /// The folder used when no profile could be leased; removed when the renderer closes.
    scratch: Option<PathBuf>,
}

/// Why WebView2 did not start, as the engine failure the renderer reports.
fn start_failure(error: &HostError) -> EngineError {
    match error {
        HostError::RuntimeMissing => EngineError::Missing {
            reason: "The WebView2 Runtime was not found.".to_owned(),
            hint: "Install the Microsoft Edge WebView2 Runtime, or play web wallpapers with CEF.".to_owned(),
        },
        HostError::Unsupported => EngineError::Unsupported {
            what: "WebView2 on this system".to_owned(),
        },
        other => EngineError::Start {
            reason: other.to_string(),
        },
    }
}

/// The profile this renderer runs on (D-33): the persistent `<data_dir>/Web/WebView2` when no other running WebView2
/// renderer holds it, otherwise an emptied ephemeral folder of this slot. A failure to get any folder is logged and the
/// renderer falls back to a scratch folder, so the wallpaper still plays; it just keeps nothing.
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
/// bind the same port (Pitfall 11). A port outside the range Chromium accepts is logged and left off, never wrapped
/// around.
fn debugging_port(developer_port: u16, slot: u8) -> Option<u16> {
    match args::debugging_port(developer_port, slot) {
        Ok(port) => port,
        Err(error) => {
            tracing::warn!(%error, slot, "no remote debugging port for this renderer");
            None
        }
    }
}

/// The `AdditionalBrowserArguments` text of a renderer: the user's `WebArguments` (the service already put the legacy
/// defaults there for an empty list) merged with Sucrose's own switches, never stored (D-17, WAPI-10). Names that are
/// not switch names are left out and logged.
fn browser_arguments(web: &WebOptions, port: Option<u16>) -> String {
    let inputs = SucroseInputs {
        hardware_acceleration: web.hardware_acceleration,
        debugging_port: port,
        windows_build: sucrose_platform::os_build(),
        engine: WebEngine::WebView2,
    };
    let plan = args::plan_webview2_command_line(&web.arguments, &inputs);
    for (name, error) in &plan.invalid {
        tracing::warn!(name = %name, %error, "a switch name is not valid, the switch is left out");
    }
    args::browser_arguments(&plan.applied)
}

/// Starts WebView2 on the calling thread, which must be the one that owns the surface `wid` and runs its message loop,
/// showing the page of the assignment. Returns the thread-safe handle and the pump the loop drives.
///
/// # Errors
/// [`EngineError::Missing`] when no WebView2 Runtime is installed, [`EngineError::Unsupported`] on a system without
/// WebView2 and [`EngineError::Start`] for every other failure.
pub fn start(wid: u64, assignment: &Assignment, hooks: SurfaceHooks) -> Result<Started, EngineError> {
    let SurfaceHooks {
        proxy,
        timers: _,
        on_playing,
        on_data_wanted,
    } = hooks;
    let told = AtomicBool::new(false);
    let asked = AtomicBool::new(false);
    let complaints = Complaints::default();
    let theme_dir = PathBuf::from(&assignment.theme_dir);
    let calls_dir = theme_dir.clone();
    let wanted_dir = theme_dir.clone();
    let web = &assignment.web;
    let state = Arc::new(initial_state(&assignment.settings, &web.theme_type, web.shuffle));
    let calls_state = Arc::clone(&state);
    let port = debugging_port(web.developer_port, assignment.slot);
    if let Some(port) = port {
        tracing::info!(port, slot = assignment.slot, "remote debugging");
    }
    let browser_arguments = browser_arguments(web, port);
    tracing::info!(
        switches = %browser_arguments,
        allow_invalid_certificates = web.allow_invalid_certificates,
        "chromium switches"
    );

    let profile = acquire_profile(assignment.slot);
    let scratch = profile.is_none().then(|| {
        std::env::temp_dir()
            .join("SucroseWebView2")
            .join(std::process::id().to_string())
    });
    let user_data_folder = match (&profile, &scratch) {
        (Some(lease), _) => lease.path.clone(),
        (None, Some(folder)) => folder.clone(),
        (None, None) => unreachable!("a renderer without a lease always has a scratch folder"),
    };
    let remove_scratch = || {
        if let Some(folder) = &scratch {
            let _ = std::fs::remove_dir_all(folder);
        }
    };

    let host_start = HostStart {
        roots: [
            Some(theme_dir.clone()),
            sucrose_core::paths::cache_dir().map(|dir| dir.join("Content")),
        ]
        .into_iter()
        .flatten()
        .collect(),
        user_data_folder,
        browser_arguments,
        developer_mode: web.developer_mode,
        allow_invalid_certificates: web.allow_invalid_certificates,
        page_calls: Arc::new(move |_url, status| page::page_calls(&calls_dir, status, &calls_state, &complaints)),
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
    };
    // The page comes from the virtual origin (D-14): the theme folder first, then the generated content.
    let url = origin::page_url(&theme_dir, &assignment.source);
    let host = Host::start(wid, assignment.rect, &url, &host_start).map_err(|error| {
        tracing::error!(%error, "cannot start WebView2");
        remove_scratch();
        start_failure(&error)
    })?;

    let (sender, receiver) = bounded(QUEUE_CAPACITY);
    Ok(Started {
        engine: Arc::new(WebView2Engine {
            commands: sender,
            wake: Arc::new(move || proxy.wake()),
            state: Arc::clone(&state),
        }),
        pump: Some(Box::new(WebView2Pump {
            host,
            commands: receiver,
            state,
            theme_dir,
            templates: None,
            profile,
            scratch,
        })),
    })
}

impl WebView2Engine {
    /// Queues `command` for the surface thread and wakes it. A full queue drops the command (T-04-13).
    fn queue(&self, command: Command) {
        match self.commands.try_send(command) {
            Ok(()) => (self.wake)(),
            // The snapshot itself is kept in the state, so a dropped marker loses nothing: the next one runs it.
            Err(TrySendError::Full(Command::Data)) => {
                tracing::debug!("the WebView2 command queue is full, a data push waits");
            }
            Err(TrySendError::Full(command)) => {
                tracing::warn!(?command, "the WebView2 command queue is full, the command is dropped");
            }
            // The pump closed the controller already; there is nobody left to tell.
            Err(TrySendError::Disconnected(command)) => tracing::debug!(?command, "the WebView2 pump is gone"),
        }
    }

    /// Whether the last pause command paused.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.state.paused()
    }

    /// The last volume the engine was given, 0..=100.
    #[must_use]
    pub fn volume(&self) -> u8 {
        self.state.volume()
    }

    /// Whether the wallpaper loops, as last set.
    #[must_use]
    pub fn looping(&self) -> bool {
        self.state.looping()
    }

    /// The value of the `ShuffleMode` call, as last set.
    #[must_use]
    pub fn shuffle(&self) -> bool {
        self.state.shuffle()
    }

    /// How the wallpaper is fitted, as last set.
    #[must_use]
    pub fn stretch(&self) -> Stretch {
        self.state.stretch()
    }
}

impl Engine for WebView2Engine {
    fn name(&self) -> &'static str {
        "WebView2"
    }

    fn set_paused(&self, paused: bool, level: PauseLevel) -> Result<(), EngineError> {
        self.state.set_paused(paused);
        self.queue(Command::Pause { paused, level });
        Ok(())
    }

    fn set_volume(&self, volume: u8) -> Result<(), EngineError> {
        let volume = volume.min(100);
        self.state.set_volume(volume);
        self.queue(Command::Volume(volume));
        Ok(())
    }

    fn set_loop(&self, looping: bool) -> Result<(), EngineError> {
        self.state.set_looping(looping);
        self.queue(Command::Loop(looping));
        Ok(())
    }

    /// The page's `ShuffleMode` call follows with the next compatible push, which the command triggers.
    fn set_shuffle(&self, shuffle: bool) {
        self.state.set_shuffle(shuffle);
        self.queue(Command::Shuffle(shuffle));
    }

    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        self.state.set_stretch(stretch);
        self.queue(Command::Stretch(stretch));
        Ok(())
    }

    /// A running WebView2 renderer cannot show another page in place: the service starts a new renderer for it.
    fn load_source(&self, _source: &Path, _settings: &PlaybackSettings) -> Result<(), EngineError> {
        Err(EngineError::Unsupported {
            what: "Showing another page in a running WebView2 renderer".to_owned(),
        })
    }

    /// A page has no timeline to follow.
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
        self.state.store_data(data);
        self.queue(Command::Data);
    }
}

impl WebView2Pump {
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
            tracing::debug!(?command, "queued command reached the WebView2 thread");
            match command {
                Command::Data | Command::Volume(_) | Command::Stretch(_) => {}
                Command::OpenDevTools => {
                    if let Err(error) = self.host.open_devtools() {
                        tracing::warn!(%error, "cannot open the WebView2 developer tools");
                    }
                }
                Command::Pause { paused, .. } => {
                    let call = js::pause_call(paused);
                    tracing::debug!(kind = call.kind, key = ?call.key, "page call");
                    self.host.execute_script(&call.script);
                }
                Command::Loop(_) | Command::Shuffle(_) => data_pushed = true,
            }
        }
        if data_pushed {
            self.push_data();
        }
    }

    /// Runs the templates with the newest data in the page: the non-data keys again and each data key that has a
    /// snapshot.
    fn push_data(&mut self) {
        let templates = self
            .templates
            .get_or_insert_with(|| page::load_templates(&self.theme_dir));
        for call in page::push_calls(templates, &self.state) {
            self.host.execute_script(&call.script);
        }
    }
}

impl SurfacePump for WebView2Pump {
    fn on_wake(&mut self) {
        self.drain();
    }

    fn on_timer(&mut self, _id: u32) -> bool {
        false
    }

    fn set_rect(&mut self, rect: [i32; 4]) {
        if let Err(error) = self.host.set_rect(rect) {
            tracing::warn!(%error, "cannot move the WebView2 controller");
        }
    }

    fn close(self: Box<Self>) {
        let Self {
            host, profile, scratch, ..
        } = *self;
        // The controller first: its window is a child of the surface.
        host.close();
        // The profile folder is free for the next instance only once WebView2 no longer writes to it.
        drop(profile);
        if let Some(folder) = scratch {
            let _ = std::fs::remove_dir_all(folder);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU8;

    use super::*;

    /// An engine handle whose pump side is the returned receiver and whose wakes are counted, so the queue can be looked
    /// at without WebView2 or a message loop.
    fn handle(capacity: usize) -> (WebView2Engine, Receiver<Command>, Arc<AtomicU8>) {
        let (sender, receiver) = bounded(capacity);
        let wakes = Arc::new(AtomicU8::new(0));
        let counter = Arc::clone(&wakes);
        let engine = WebView2Engine {
            commands: sender,
            wake: Arc::new(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }),
            state: Arc::new(initial_state(&PlaybackSettings::default(), "Dark", true)),
        };
        (engine, receiver, wakes)
    }

    #[test]
    fn a_missing_runtime_is_a_missing_engine_and_other_failures_are_start_failures() {
        let missing = start_failure(&HostError::RuntimeMissing);
        let EngineError::Missing { hint, .. } = missing else {
            panic!("a missing runtime is not Missing: {missing:?}");
        };
        assert!(hint.contains("WebView2 Runtime"));
        assert!(matches!(
            start_failure(&HostError::Environment("x".to_owned())),
            EngineError::Start { .. }
        ));
        assert!(matches!(
            start_failure(&HostError::Unsupported),
            EngineError::Unsupported { .. }
        ));
    }

    #[test]
    fn commands_reach_the_queue_in_order_and_each_wakes_the_surface_thread() {
        let (engine, queue, wakes) = handle(8);

        engine.set_paused(true, PauseLevel::Heavy).expect("pause");
        engine.set_volume(40).expect("volume");
        engine.set_loop(false).expect("loop");
        engine.set_shuffle(false);
        engine.set_stretch(Stretch::Fill).expect("stretch");

        assert_eq!(
            queue.try_iter().collect::<Vec<_>>(),
            [
                Command::Pause {
                    paused: true,
                    level: PauseLevel::Heavy
                },
                Command::Volume(40),
                Command::Loop(false),
                Command::Shuffle(false),
                Command::Stretch(Stretch::Fill),
            ]
        );
        assert_eq!(wakes.load(Ordering::SeqCst), 5);
        assert!(engine.paused() && !engine.looping() && !engine.shuffle());
        assert_eq!((engine.volume(), engine.stretch()), (40, Stretch::Fill));
    }

    #[test]
    fn a_full_queue_drops_the_command_without_blocking_and_the_state_keeps_the_newest_value() {
        let (engine, queue, wakes) = handle(1);

        engine.set_volume(10).expect("volume");
        engine.set_volume(70).expect("volume");

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Volume(10)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert_eq!(engine.volume(), 70);
    }

    #[test]
    fn open_devtools_queues_a_command_for_the_surface_thread_and_wakes_it() {
        let (engine, queue, wakes) = handle(4);

        assert!(engine.open_devtools().is_ok());

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::OpenDevTools]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn the_volume_is_capped_at_100() {
        let (engine, _queue, _) = handle(2);

        engine.set_volume(250).expect("volume");

        assert_eq!(engine.volume(), 100);
    }

    /// The stored snapshots as compact JSON text.
    fn snapshots(engine: &WebView2Engine) -> Vec<(DataGroup, String)> {
        engine
            .state
            .data()
            .into_iter()
            .map(|(group, value)| (group, value.to_string()))
            .collect()
    }

    #[test]
    fn a_data_push_keeps_the_newest_snapshot_and_queues_one_marker_each() {
        let (engine, queue, wakes) = handle(8);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Data, Command::Data]);
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        assert_eq!(snapshots(&engine), [(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);
    }

    #[test]
    fn a_full_queue_does_not_lose_the_newest_snapshot() {
        let (engine, queue, _) = handle(1);

        engine.push_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
        engine.push_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

        assert_eq!(queue.try_iter().count(), 1, "the second marker was dropped");
        assert_eq!(snapshots(&engine), [(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);
    }

    #[test]
    fn the_browser_arguments_are_the_users_switches_and_sucroses_own_on_one_line() {
        let web = WebOptions {
            arguments: args::from_web_list(&args::legacy_defaults_web()),
            hardware_acceleration: true,
            ..WebOptions::default()
        };

        let text = browser_arguments(&web, None);

        assert!(text.contains("--autoplay-policy=no-user-gesture-required"), "{text}");
        assert!(text.contains("--disable-web-security"), "{text}");
        assert!(text.contains("--enable-gpu"), "{text}");
        assert!(!text.contains("remote-debugging-port"), "{text}");
        assert!(!text.contains('\n'));
    }

    #[test]
    fn a_debugging_port_adds_the_port_and_the_one_merged_feature_switch() {
        let web = WebOptions::default();

        let text = browser_arguments(&web, debugging_port(9222, 1));

        assert!(text.contains("--remote-debugging-port=9223"), "{text}");
        assert!(text.contains("--remote-allow-origins=*"), "{text}");
        assert_eq!(text.matches("--enable-features=").count(), 1, "{text}");
        assert!(text.contains("msEdgeDevToolsWdpRemoteDebugging"), "{text}");
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
    fn a_web_page_cannot_be_replaced_in_place_and_has_no_clock() {
        let (engine, _queue, _) = handle(1);

        let error = engine
            .load_source(Path::new("index.html"), &PlaybackSettings::default())
            .expect_err("the service restarts the renderer");

        assert!(matches!(error, EngineError::Unsupported { .. }));
        assert!(engine.clock().is_none());
        assert_eq!(engine.name(), "WebView2");
    }
}
