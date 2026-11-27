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
    theme::WallpaperType,
};
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started, SurfaceHooks, SurfacePump},
};
use sucrose_engine_web::{
    args::{self, SucroseInputs, Switch, WebEngine},
    js,
    moves::MoveSlot,
    origin,
    page::{self, Complaints, PageKind, PageState},
    pages::{self, GeneratedPage},
    profile::{self, ProfileKind, ProfileLease},
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings, WebOptions};
use sucrose_platform::input::{InputEvent, focus_click_messages, key_message, mouse_message};

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
    /// Whether this renderer plays the sound changed (D-02): a generated page is told the level it plays at.
    Audio(bool),
    Loop(bool),
    /// The value of the `ShuffleMode` call changed (PLAY-01).
    Shuffle(bool),
    Stretch(Stretch),
    /// A data snapshot arrived: run the templates with the newest values (kept in the [`PageState`]).
    Data,
    /// The Portal's Open DevTools button (D-13).
    OpenDevTools,
    /// A mouse move is waiting in the shared pending slot; one marker covers a whole burst of moves.
    MouseMove,
    /// One input event the renderer's filters let through.
    Input(InputEvent),
}

/// The newest mouse move not yet handed to the page: a mouse moves a thousand times a second, so moves are coalesced in
/// this slot instead of filling the command queue.
type PendingMove = Arc<MoveSlot<InputEvent>>;

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
    pending_move: PendingMove,
    /// Whether the page takes forwarded input: a Web or Url page does, a generated Gif, Video or YouTube page does not.
    takes_input: bool,
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
    pending_move: PendingMove,
    /// The top left corner of the surface on the virtual screen: input positions are made relative to it.
    origin: (i32, i32),
    theme_dir: PathBuf,
    /// The generated page this renderer shows (a Gif, Video or YouTube theme), `None` for a theme's own page.
    generated: Option<GeneratedPage>,
    /// Whether this renderer shows the address of a Url theme: such a page gets no compatible calls and no data.
    url_page: bool,
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

/// The profile this renderer runs on (D-33, D-040): the persistent `<data_dir>/Web/WebView2` when the service told this
/// renderer (the primary monitor's) to ask for it and no other running WebView2 renderer holds it, otherwise an emptied
/// ephemeral folder of this slot. A failure to get any folder is logged and the
/// renderer falls back to a scratch folder, so the wallpaper still plays; it just keeps nothing.
fn acquire_profile(slot: u8, monitor: &str, wanted: ProfileKind) -> Option<Arc<ProfileLease>> {
    let web = sucrose_core::paths::web_dir()?;
    let ephemeral = sucrose_core::paths::cache_dir()?.join("WebEphemeral");
    match profile::acquire(&web, PROFILE_ENGINE, &ephemeral, slot, wanted, profile::START_WAIT) {
        Ok(lease) => {
            tracing::info!(
                kind = ?lease.kind,
                wanted = ?wanted,
                monitor,
                engine = PROFILE_ENGINE,
                path = ?lease.path,
                slot,
                "web profile"
            );
            if wanted == ProfileKind::Persistent && lease.kind == ProfileKind::Ephemeral {
                tracing::info!(
                    slot,
                    monitor,
                    "the persistent web profile is still held by another renderer, so this page keeps nothing across runs"
                );
            }
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

/// The Chromium switch that accepts every invalid TLS certificate: the switch CEF gets for Allow invalid certificates
/// (`sucrose_engine_cef::browser::IGNORE_CERTIFICATE_ERRORS_SWITCH`).
const IGNORE_CERTIFICATE_ERRORS: &str = "ignore-certificate-errors";

/// The `AdditionalBrowserArguments` text of a renderer: the user's `WebArguments` (the service already put the legacy
/// defaults there for an empty list) merged with Sucrose's own switches, never stored (D-17, WAPI-10). Names that are
/// not switch names are left out and logged. Allow invalid certificates adds `ignore-certificate-errors`, as for CEF
/// (D-12): an error allowed through the certificate event instead holds each request about ten seconds in WebView2.
/// The SmartScreen switch follows `WebOptions.webview_smartscreen` (D-041): it is there unless the setting is on.
fn browser_arguments(web: &WebOptions, port: Option<u16>) -> String {
    let inputs = SucroseInputs {
        hardware_acceleration: web.hardware_acceleration,
        debugging_port: port,
        windows_build: sucrose_platform::os_build(),
        engine: WebEngine::WebView2,
        webview_smartscreen: web.webview_smartscreen,
    };
    let mut plan = args::plan_webview2_command_line(&web.arguments, &inputs);
    for (name, error) in &plan.invalid {
        tracing::warn!(name = %name, %error, "a switch name is not valid, the switch is left out");
    }
    if web.allow_invalid_certificates
        && !plan
            .applied
            .iter()
            .any(|switch| switch.name == IGNORE_CERTIFICATE_ERRORS)
    {
        plan.applied.push(Switch::flag(IGNORE_CERTIFICATE_ERRORS));
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
    let generated = GeneratedPage::for_kind(assignment.kind);
    let page_kind = PageKind::for_wallpaper(assignment.kind, WebEngine::WebView2);
    let url_page = assignment.kind == WallpaperType::Url;
    let port = debugging_port(web.developer_port, assignment.slot);
    if let Some(port) = port {
        tracing::info!(port, slot = assignment.slot, "remote debugging");
    }
    let browser_arguments = browser_arguments(web, port);
    tracing::info!(
        switches = %browser_arguments,
        allow_invalid_certificates = web.allow_invalid_certificates,
        user_agent = ?web.user_agent,
        webview_smartscreen = web.webview_smartscreen,
        "chromium switches"
    );

    let profile = acquire_profile(assignment.slot, &assignment.monitor, assignment.web_profile);
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
        user_agent: web.user_agent.clone(),
        page_calls: Arc::new(move |_url, status| {
            page::page_calls(page_kind, &calls_dir, status, &calls_state, &complaints)
        }),
        // The first finished main-frame load is the first frame the user can see.
        on_loaded: Arc::new(move |_url, status| {
            if !told.swap(true, Ordering::SeqCst) {
                on_playing();
            }
            // Once per renderer, after the page and its calls: a reload keeps the data it already asked for.
            if status == 200 && !asked.swap(true, Ordering::SeqCst) {
                // Only a Web theme's own page asks for system data: a generated page and a URL page ask for none.
                let groups = if page_kind.is_theme() {
                    page::wanted_groups(&wanted_dir, status)
                } else {
                    Vec::new()
                };
                if !groups.is_empty() {
                    tracing::info!(groups = ?groups, "data wanted");
                    on_data_wanted(groups);
                }
            }
        }),
    };
    // The page comes from the virtual origin (D-14): the theme folder first, then the generated content. A Gif, Video
    // or YouTube theme plays through a generated page that is written into the content folder.
    let url = if url_page {
        // A Url theme navigates straight to its address, which the theme gate verified (http or https only).
        assignment.source.clone()
    } else {
        match pages::prepare(assignment.kind, &theme_dir, &assignment.source) {
            Ok(Some(url)) => url,
            Ok(None) => origin::page_url(&theme_dir, &assignment.source),
            Err(error) => {
                tracing::error!(%error, kind = ?assignment.kind, "cannot prepare the generated page");
                remove_scratch();
                return Err(EngineError::Start {
                    reason: error.to_string(),
                });
            }
        }
    };
    let host = Host::start(wid, assignment.rect, &url, &host_start).map_err(|error| {
        tracing::error!(%error, "cannot start WebView2");
        remove_scratch();
        start_failure(&error)
    })?;

    let (sender, receiver) = bounded(QUEUE_CAPACITY);
    let pending_move = PendingMove::default();
    Ok(Started {
        engine: Arc::new(WebView2Engine {
            commands: sender,
            wake: Arc::new(move || proxy.wake()),
            state: Arc::clone(&state),
            pending_move: Arc::clone(&pending_move),
            takes_input: page_kind.takes_input(),
        }),
        pump: Some(Box::new(WebView2Pump {
            host,
            commands: receiver,
            state,
            pending_move,
            origin: (assignment.rect[0], assignment.rect[1]),
            theme_dir,
            generated,
            url_page,
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

    /// A theme's page gets its `ShuffleMode` call with the next compatible push, which the command triggers; the YouTube
    /// page gets `setShuffle` at once.
    fn set_shuffle(&self, shuffle: bool) {
        self.state.set_shuffle(shuffle);
        self.queue(Command::Shuffle(shuffle));
    }

    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        self.state.set_stretch(stretch);
        self.queue(Command::Stretch(stretch));
        Ok(())
    }

    /// A generated page plays at the engine volume only while this renderer plays the sound (D-02, D-06); a theme's own
    /// page keeps its Round A behaviour (its volume is Round D).
    fn set_audio(&self, on: bool) -> Result<(), EngineError> {
        self.state.set_audio(on);
        self.queue(Command::Audio(on));
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

    fn takes_input(&self) -> bool {
        self.takes_input
    }

    /// Keeps the newest mouse move and queues one marker for a burst of consecutive moves; every other event is queued as
    /// it is, behind the move that came before it, so input keeps its order. A full queue drops the event without a word
    /// about its content (T-04-45).
    fn forward_input(&self, event: &InputEvent) {
        if let InputEvent::MouseMove { .. } = event {
            if self.pending_move.offer(*event) {
                match self.commands.try_send(Command::MouseMove) {
                    Ok(()) => (self.wake)(),
                    // Nobody will take it: the next move must queue a marker again.
                    Err(_) => self.pending_move.marker_dropped(),
                }
            }
            return;
        }
        // A move still waiting goes first: its marker would otherwise hand it to the page after this event.
        if let Some(moved) = self.pending_move.flush_before_other()
            && self.commands.try_send(Command::Input(moved)).is_err()
        {
            tracing::debug!("the WebView2 command queue is full or closed, an input event is dropped");
        }
        match self.commands.try_send(Command::Input(*event)) {
            Ok(()) => (self.wake)(),
            Err(_) => tracing::debug!("the WebView2 command queue is full or closed, an input event is dropped"),
        }
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
            // Input is never logged: it is what the user types and moves (T-04-45).
            match command {
                Command::MouseMove => {
                    if let Some(event) = self.pending_move.take_at_marker() {
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
            tracing::debug!(?command, "queued command reached the WebView2 thread");
            match command {
                Command::Data | Command::MouseMove | Command::Input(_) => {}
                Command::OpenDevTools => {
                    if let Err(error) = self.host.open_devtools() {
                        tracing::warn!(%error, "cannot open the WebView2 developer tools");
                    }
                }
                Command::Pause { paused, .. } => self.run(&js::pause_call(paused)),
                // A generated page is told the new value; a theme's own page gets the compatible calls again.
                Command::Loop(on) => match self.generated {
                    Some(page) => self.run(&js::PageCall {
                        kind: js::KIND_ENGINE,
                        key: "loop".to_owned(),
                        script: pages::loop_script(page, on),
                    }),
                    None => data_pushed = true,
                },
                // The YouTube page is told the new value; a theme's own page gets the compatible calls again.
                Command::Shuffle(on) => match self.generated {
                    Some(page) => {
                        if let Some(call) = page::shuffle_call(page, on) {
                            self.run(&call);
                        }
                    }
                    None => data_pushed = true,
                },
                Command::Stretch(stretch) => {
                    if let Some(page) = self.generated {
                        self.run(&js::PageCall {
                            kind: js::KIND_ENGINE,
                            key: "stretch".to_owned(),
                            script: pages::stretch_script(page, stretch),
                        });
                    }
                }
                Command::Volume(_) | Command::Audio(_) => {
                    if self.generated.is_some() {
                        self.run(&page::volume_call(&self.state));
                    }
                }
            }
        }
        if data_pushed {
            self.push_data();
        }
    }

    /// Runs one call in the page, logged as a `page call` at debug.
    fn run(&self, call: &js::PageCall) {
        tracing::debug!(kind = call.kind, key = ?call.key, "page call");
        self.host.execute_script(&call.script);
    }

    /// Hands one input event to the page: the legacy message to the Chromium widget, positions relative to the surface;
    /// the wheel goes through the DevTools protocol, as in the legacy WebView engine.
    fn post_input(&mut self, event: &InputEvent) {
        match *event {
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

    /// Runs the templates with the newest data in the page: the non-data keys again and each data key that has a
    /// snapshot.
    fn push_data(&mut self) {
        if self.generated.is_some() || self.url_page {
            return;
        }
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
        self.origin = (rect[0], rect[1]);
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
        // The profile folder is free for the next instance only once WebView2 no longer writes to it: `host.close()`
        // returned after WebView2's browser process exited (or the bounded wait ended), so the lease ends here (D10).
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
            pending_move: PendingMove::default(),
            takes_input: true,
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
    fn a_web_page_takes_input() {
        let (engine, _queue, _) = handle(1);

        assert!(engine.takes_input());
    }

    #[test]
    fn a_burst_of_moves_queues_one_marker_and_keeps_the_newest_move() {
        let (engine, queue, wakes) = handle(8);

        for x in 1..=5 {
            engine.forward_input(&InputEvent::MouseMove { x, y: 2 });
        }

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::MouseMove]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert_eq!(
            engine.pending_move.pending(),
            Some(InputEvent::MouseMove { x: 5, y: 2 })
        );
    }

    #[test]
    fn a_click_keeps_its_place_between_the_moves_around_it() {
        let (engine, queue, _) = handle(8);
        let key = InputEvent::Key {
            vk: 0x41,
            scan: 0x1E,
            down: true,
            extended: false,
        };
        let first = InputEvent::MouseMove { x: 1, y: 1 };
        let second = InputEvent::MouseMove { x: 2, y: 2 };

        engine.forward_input(&first);
        engine.forward_input(&key);
        engine.forward_input(&second);

        // The move before the key is queued ahead of it and its marker is skipped; the move after it has its own.
        assert_eq!(
            queue.try_iter().collect::<Vec<_>>(),
            [
                Command::MouseMove,
                Command::Input(first),
                Command::Input(key),
                Command::MouseMove
            ]
        );
        assert_eq!(
            engine.pending_move.take_at_marker(),
            None,
            "the first marker was left behind"
        );
        assert_eq!(engine.pending_move.take_at_marker(), Some(second));
    }

    #[test]
    fn a_generated_page_takes_no_input_and_the_handle_reports_what_it_was_started_with() {
        let (mut engine, _queue, _) = handle(1);
        assert!(engine.takes_input());

        engine.takes_input = false;

        assert!(!engine.takes_input());
    }

    #[test]
    fn a_dropped_move_marker_lets_the_next_move_queue_a_new_one() {
        let (engine, queue, _) = handle(1);
        engine.set_volume(1).expect("fills the queue");

        engine.forward_input(&InputEvent::MouseMove { x: 1, y: 1 });

        assert_eq!(engine.pending_move.pending(), None);
        assert_eq!(queue.try_iter().count(), 1, "only the volume command is queued");
    }

    #[test]
    fn other_events_are_queued_as_they_are_and_a_full_queue_drops_them() {
        let (engine, queue, wakes) = handle(1);
        let key = InputEvent::Key {
            vk: 0x41,
            scan: 0x1E,
            down: true,
            extended: false,
        };

        engine.forward_input(&key);
        engine.forward_input(&InputEvent::FocusClick);

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Input(key)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_change_of_the_sound_role_is_kept_and_queued_so_a_generated_page_is_told_its_level() {
        let (engine, queue, wakes) = handle(4);
        assert!(
            engine.state.audio(),
            "a renderer plays the sound until it is told otherwise"
        );

        engine.set_audio(false).expect("audio");

        assert_eq!(queue.try_iter().collect::<Vec<_>>(), [Command::Audio(false)]);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        assert!(!engine.state.audio());
        assert_eq!(page::volume_call(&engine.state).script, "SucroseVolume(0);");
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
    fn smartscreen_on_leaves_the_smartscreen_feature_out_of_the_browser_arguments() {
        let off = WebOptions {
            arguments: args::from_web_list(&args::legacy_defaults_web()),
            ..WebOptions::default()
        };
        let on = WebOptions {
            webview_smartscreen: true,
            ..off.clone()
        };

        let text_off = browser_arguments(&off, None);
        let text_on = browser_arguments(&on, None);

        assert!(text_off.contains("msSmartScreenProtection"), "{text_off}");
        assert!(!text_on.contains("msSmartScreenProtection"), "{text_on}");
        assert!(text_on.contains("IsolateOrigins"), "{text_on}");
    }

    #[test]
    fn allow_invalid_certificates_gives_webview2_the_switch_cef_gets_and_only_then() {
        let allowed = WebOptions {
            allow_invalid_certificates: true,
            ..WebOptions::default()
        };
        let listed = WebOptions {
            allow_invalid_certificates: true,
            arguments: args::from_web_list(&["--ignore-certificate-errors".to_owned()]),
            ..WebOptions::default()
        };

        let text = browser_arguments(&allowed, None);

        assert_eq!(text.matches("--ignore-certificate-errors").count(), 1, "{text}");
        assert!(
            !browser_arguments(&WebOptions::default(), None).contains("ignore-certificate-errors"),
            "D-12: certificates are validated by default"
        );
        assert_eq!(
            browser_arguments(&listed, None)
                .matches("ignore-certificate-errors")
                .count(),
            1,
            "a switch the user already lists is not added twice"
        );
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
