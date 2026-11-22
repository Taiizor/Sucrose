//! Service mode: owns the endpoint (single instance, D-023), starts a renderer and listens to it.

use std::{
    collections::{HashMap, HashSet},
    process::{Child, Command},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use crossbeam_channel::Sender;
use sucrose_config::legacy::LegacySettings;
use sucrose_core::{
    APP_LONG_NAME,
    paths::{legacy_data_dir, legacy_setting_dir},
    theme::{SelectedTheme, check_selected},
    version::app_version,
};
use sucrose_ipc::{
    Connection, IpcError, Listener, endpoint_name,
    protocol::{Assignment, PROTOCOL_VERSION, ToRenderer, ToService},
};
use sucrose_platform::{EventLoop, Flow, LoopEvent, LoopProxy, Platform, PlatformError, Tray};

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
    /// A message from a renderer.
    Renderer(ToService),
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

/// The renderers this service started and the channels to them.
#[derive(Default)]
struct Renderers {
    /// The pids of the processes this service started: the only ones whose Hello is accepted.
    pids: HashSet<u32>,
    children: Vec<(u8, Child)>,
    /// The write side of each connected renderer, by slot.
    connections: HashMap<u8, Connection>,
    /// Set when shutdown begins: a renderer that connects later is told to close at once.
    closing: bool,
}

type SharedRenderers = Arc<Mutex<Renderers>>;

fn lock(renderers: &Mutex<Renderers>) -> MutexGuard<'_, Renderers> {
    renderers.lock().unwrap_or_else(PoisonError::into_inner)
}

/// How long a renderer gets to leave after Close before it is killed.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// The tray icon the service shows (the C# Launcher's image, D-14).
const TRAY_ICON: &[u8] = include_bytes!("../assets/TrayIcon.ico");

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
}

impl Status {
    /// The sentence for the log.
    fn text(&self) -> String {
        match self {
            Self::Starting { .. } => APP_LONG_NAME.to_owned(),
            Self::Playing { title } => format!("Sucrose — playing {title}"),
            Self::NoWallpaper { reason } => format!("Sucrose — no wallpaper: {reason}"),
            Self::Failed { reason, hint } => format!("Sucrose — no wallpaper: {reason} {hint}"),
        }
    }

    /// The tray tooltip: the same sentence, cut to what the notification area keeps.
    fn tooltip(&self) -> String {
        sucrose_platform::truncate_tooltip(&self.text())
    }

    /// Whether the user has to be told that no wallpaper plays.
    fn is_failure(&self) -> bool {
        matches!(self, Self::NoWallpaper { .. } | Self::Failed { .. })
    }

    /// The status a renderer message leads to, or `None` when it changes nothing.
    fn after(&self, message: &ToService) -> Option<Self> {
        match message {
            ToService::Playing => match self {
                Self::Starting { title } => Some(Self::Playing { title: title.clone() }),
                _ => None,
            },
            ToService::Failed { reason, hint } => Some(Self::Failed {
                reason: limit(reason),
                hint: limit(hint),
            }),
            ToService::Hello { .. } => None,
        }
    }
}

/// `text` cut to [`MAX_REPORT_CHARS`] characters.
fn limit(text: &str) -> String {
    text.chars().take(MAX_REPORT_CHARS).collect()
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
        version = sucrose_core::VERSION,
        app_version = %app_version(),
        os = platform.name(),
        capabilities = ?platform.capabilities(),
        endpoint = %endpoint.name,
        dpi = ?awareness,
        "service started"
    );
    let plan = plan_wallpapers(platform.as_ref());
    let assignments: Arc<Vec<Assignment>> = Arc::new(plan.assignments);
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
    for assignment in assignments.iter() {
        spawn_renderer(&renderers, assignment.slot)?;
    }
    let accept_renderers = Arc::clone(&renderers);
    let accept_assignments = Arc::clone(&assignments);
    let accept_notifier = notifier.clone();
    thread::Builder::new()
        .name("ipc-accept".to_owned())
        .spawn(move || accept_loop(&listener, &accept_renderers, &accept_assignments, &accept_notifier))
        .context("cannot start the accept thread")?;

    let tray_proxy = event_loop.proxy();
    let tray = start_tray(plan.app_visible, || Tray::new(TRAY_ICON, &status.tooltip(), tray_proxy));
    if status.is_failure() {
        tracing::warn!(status = %status.text(), "no wallpaper, the service waits in the tray");
    }

    let mut quitting = false;
    event_loop.run(|event| match event {
        LoopEvent::Wake => {
            for event in receiver.try_iter() {
                match event {
                    Event::Renderer(ToService::Hello { pid, slot, .. }) => {
                        tracing::info!(pid, slot, "renderer connected");
                    }
                    Event::Renderer(message) => {
                        if let Some(next) = status.after(&message) {
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
                    Event::ShutdownDone => return Flow::Exit,
                }
            }
            Flow::Continue
        }
        LoopEvent::Quit => {
            if !quitting {
                quitting = true;
                tracing::info!("Quit chosen, closing the renderers");
                start_shutdown(&renderers, &notifier);
            }
            Flow::Continue
        }
    });
    drop(tray);
    tracing::info!("the service stopped");
    Ok(())
}

/// Closes every renderer on a worker thread, so the message loop (and with it the tray) stays responsive while a
/// renderer takes its time (T-02-20).
fn start_shutdown(renderers: &SharedRenderers, notifier: &Notifier) {
    let renderers = Arc::clone(renderers);
    let notifier = notifier.clone();
    let started = thread::Builder::new().name("shutdown".to_owned()).spawn({
        let notifier = notifier.clone();
        move || {
            let mut control = {
                let mut state = lock(&renderers);
                state.closing = true;
                ProcessRenderers {
                    children: std::mem::take(&mut state.children),
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

/// The real renderers: only the child handles this service spawned are ever touched (D-14).
struct ProcessRenderers {
    children: Vec<(u8, Child)>,
    connections: HashMap<u8, Connection>,
}

impl ProcessRenderers {
    fn child(&mut self, slot: u8) -> Option<&mut Child> {
        self.children
            .iter_mut()
            .find(|(candidate, _)| *candidate == slot)
            .map(|(_, child)| child)
    }
}

impl RendererControl for ProcessRenderers {
    fn slots(&self) -> Vec<u8> {
        self.children.iter().map(|(slot, _)| *slot).collect()
    }

    fn close(&mut self, slot: u8) {
        match self.connections.get_mut(&slot) {
            Some(connection) => {
                if let Err(error) = connection.send(&ToRenderer::Close) {
                    tracing::warn!(%error, slot, "cannot send Close to a renderer");
                }
            }
            None => tracing::debug!(slot, "no connection to send Close to, the renderer is still starting"),
        }
    }

    fn wait_exit(&mut self, slot: u8, timeout: Duration) -> bool {
        let Some(child) = self.child(slot) else {
            return true;
        };
        let deadline = Instant::now() + timeout;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::info!(slot, %status, "a renderer exited");
                    return true;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(%error, slot, "cannot wait for a renderer");
                    return false;
                }
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn kill(&mut self, slot: u8) {
        if let Some(child) = self.child(slot) {
            if let Err(error) = child.kill() {
                tracing::warn!(%error, slot, "cannot stop a renderer");
            }
            // Reaps the process so no handle is left behind.
            let _ = child.wait();
        }
    }
}

/// Starts `sucrose --render <slot>` and remembers it as a process this service started.
fn spawn_renderer(renderers: &Mutex<Renderers>, slot: u8) -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("cannot find the path of this executable")?;
    // Held across the spawn so a fast renderer cannot say Hello before its pid is recorded.
    let mut state = lock(renderers);
    let child = Command::new(exe)
        .arg("--render")
        .arg(slot.to_string())
        .spawn()
        .context("cannot start the renderer")?;
    state.pids.insert(child.id());
    tracing::info!(pid = child.id(), slot, "renderer started");
    state.children.push((slot, child));
    Ok(())
}

/// Accepts connections and gives each one a reader thread.
fn accept_loop(
    listener: &Listener,
    renderers: &SharedRenderers,
    assignments: &Arc<Vec<Assignment>>,
    notifier: &Notifier,
) {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    while failures < MAX_FAILURES {
        match listener.accept() {
            Ok(connection) => {
                failures = 0;
                let renderers = Arc::clone(renderers);
                let assignments = Arc::clone(assignments);
                let notifier = notifier.clone();
                let started = thread::Builder::new()
                    .name("ipc-reader".to_owned())
                    .spawn(move || read_connection(connection, &renderers, &assignments, &notifier));
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
    assignments: &[Assignment],
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
        (accept_hello(&state.pids, &first), state.closing)
    };
    let slot = match accepted {
        Ok(slot) => slot,
        Err(refusal) => {
            tracing::warn!(%refusal, "refusing a connection");
            return;
        }
    };
    // The renderer waits for this message; without a job for its slot, or once shutdown has begun, it has nothing
    // to do, so it is told to stop.
    let reply = match assignments.iter().find(|assignment| assignment.slot == slot) {
        Some(assignment) if !closing => ToRenderer::Assign(assignment.clone()),
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
    }
    register_writer(renderers, slot, &mut connection);
    if !notifier.send(Event::Renderer(first)) {
        return;
    }
    loop {
        match connection.recv::<ToService>() {
            Ok(message) => {
                if !notifier.send(Event::Renderer(message)) {
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
/// reading. When shutdown began in the meantime, the Close goes out right here instead.
fn register_writer(renderers: &Mutex<Renderers>, slot: u8, connection: &mut Connection) {
    let writer = match connection.try_clone() {
        Ok(writer) => writer,
        Err(error) => {
            tracing::warn!(%error, slot, "cannot keep a write handle for a renderer");
            return;
        }
    };
    let mut state = lock(renderers);
    if state.closing {
        drop(state);
        if let Err(error) = connection.send(&ToRenderer::Close) {
            tracing::warn!(%error, slot, "cannot send Close to a renderer");
        }
    } else {
        state.connections.insert(slot, writer);
    }
}

/// Why the service refuses a renderer's Hello.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The renderer speaks another protocol version.
    ProtocolVersion { got: u16, expected: u16 },
    /// The pid is not a process this service started.
    UnknownPid(u32),
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
            Self::NotHello => write!(f, "the first message is not a Hello"),
        }
    }
}

/// Decides whether a connection's first message is acceptable and returns the renderer's slot.
fn accept_hello(spawned: &HashSet<u32>, message: &ToService) -> Result<u8, Refusal> {
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
    if !spawned.contains(pid) {
        return Err(Refusal::UnknownPid(*pid));
    }
    Ok(*slot)
}

/// What the settings and the selected theme lead to.
struct Plan {
    /// The renderers to start: one for the primary monitor when the selected theme is a video this build can play,
    /// none otherwise.
    assignments: Vec<Assignment>,
    /// What the tray says before any renderer has reported.
    status: Status,
    /// General.json AppVisible: whether the tray exists at all (D-15).
    app_visible: bool,
}

/// Reads the settings of the C# install, read-only from its real folder (D-008), logs what the service will use and
/// returns what they lead to.
fn plan_wallpapers(platform: &dyn Platform) -> Plan {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return Plan {
            assignments: Vec::new(),
            status: Status::NoWallpaper {
                reason: "the OS reports no config folder, so the settings cannot be read".to_owned(),
            },
            app_visible: true,
        };
    };
    let settings = LegacySettings::load(&setting_dir, &root);
    // Values only: no paths and no theme names (D-08, D-09).
    tracing::info!(
        volume = settings.engine.volume,
        looping = settings.engine.looping,
        stretch = ?settings.engine.stretch,
        hardware_acceleration = settings.engine.hardware_acceleration,
        screen = ?settings.engine.screen,
        fullscreen = ?settings.backgroundog.fullscreen,
        pause = ?settings.backgroundog.pause,
        app_visible = settings.general.app_visible,
        "legacy settings"
    );
    let (assignments, status) = match plan_for(&settings.library.location, &settings.library.selected, platform) {
        Ok(planned) => (vec![planned.assignment], Status::Starting { title: planned.title }),
        Err(reason) => (Vec::new(), Status::NoWallpaper { reason }),
    };
    Plan {
        assignments,
        status,
        app_visible: settings.general.app_visible,
    }
}

/// A theme that passed the gate and the job for it.
struct Planned {
    assignment: Assignment,
    /// The theme's title, for the tooltip.
    title: String,
}

/// Runs the gate (D-13) on the selected theme and, when it passes, builds the job for the primary monitor. The error
/// is one English sentence for the log and the tooltip: the gate's own reason, or why no monitor could be used.
fn plan_for(location: &std::path::Path, selected: &str, platform: &dyn Platform) -> Result<Planned, String> {
    let theme = check_selected(location, selected, &app_version()).map_err(|rejection| rejection.to_string())?;
    tracing::info!(theme = %selected, kind = ?theme.info.kind, source = %theme.info.source, "selected theme");
    Ok(Planned {
        assignment: assignment_for(&theme, platform)?,
        title: theme.info.title.clone(),
    })
}

/// The job for the primary monitor playing a theme that passed the gate.
fn assignment_for(theme: &SelectedTheme, platform: &dyn Platform) -> Result<Assignment, String> {
    let monitors = platform.monitors();
    let monitor = monitors
        .iter()
        .find(|monitor| monitor.primary)
        .or_else(|| monitors.first())
        .ok_or_else(|| "the OS reports no monitor".to_owned())?;
    let source = theme
        .source_path
        .to_str()
        .ok_or_else(|| "the video path is not valid UTF-8".to_owned())?
        .to_owned();
    Ok(Assignment {
        slot: 0,
        monitor: monitor.id.clone(),
        rect: [
            monitor.rect.left,
            monitor.rect.top,
            monitor.rect.right,
            monitor.rect.bottom,
        ],
        source,
    })
}

#[cfg(test)]
mod tests {
    use sucrose_core::theme::ThemeRejection;

    use super::*;

    fn hello(protocol_version: u16, pid: u32, slot: u8) -> ToService {
        ToService::Hello {
            protocol_version,
            pid,
            slot,
        }
    }

    #[test]
    fn accept_hello_takes_a_renderer_the_service_started() {
        let spawned = HashSet::from([100, 200]);

        assert_eq!(accept_hello(&spawned, &hello(PROTOCOL_VERSION, 200, 7)), Ok(7));
    }

    #[test]
    fn accept_hello_refuses_a_pid_the_service_did_not_start() {
        let spawned = HashSet::from([100]);

        assert_eq!(
            accept_hello(&spawned, &hello(PROTOCOL_VERSION, 999, 0)),
            Err(Refusal::UnknownPid(999))
        );
    }

    #[test]
    fn accept_hello_refuses_another_protocol_version() {
        let spawned = HashSet::from([100]);

        assert_eq!(
            accept_hello(&spawned, &hello(PROTOCOL_VERSION + 1, 100, 0)),
            Err(Refusal::ProtocolVersion {
                got: PROTOCOL_VERSION + 1,
                expected: PROTOCOL_VERSION
            })
        );
    }

    #[test]
    fn accept_hello_refuses_with_an_empty_set() {
        assert_eq!(
            accept_hello(&HashSet::new(), &hello(PROTOCOL_VERSION, 100, 0)),
            Err(Refusal::UnknownPid(100))
        );
    }

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
        let playing = starting().after(&ToService::Playing).expect("a status change");

        assert_eq!(playing.tooltip(), "Sucrose — playing Cyber Samurai");
    }

    #[test]
    fn a_playing_report_changes_nothing_when_no_renderer_was_expected() {
        let rejected = Status::NoWallpaper { reason: "x".to_owned() };

        assert_eq!(rejected.after(&ToService::Playing), None);
        assert_eq!(starting().after(&hello(PROTOCOL_VERSION, 1, 0)), None);
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
            .after(&ToService::Failed {
                reason: "libmpv could not be found.".to_owned(),
                hint: "Reinstall Sucrose.".to_owned(),
            })
            .expect("a status change");

        assert_eq!(
            failed.tooltip(),
            "Sucrose — no wallpaper: libmpv could not be found. Reinstall Sucrose."
        );
        assert!(failed.is_failure());
    }

    #[test]
    fn a_long_reason_is_cut_to_127_utf16_units_in_the_tooltip() {
        let failed = starting()
            .after(&ToService::Failed {
                reason: "r".repeat(300),
                hint: "h".repeat(300),
            })
            .expect("a status change");

        assert_eq!(failed.tooltip().encode_utf16().count(), 127);
        // What a renderer sends is also limited before it reaches the log.
        let Status::Failed { reason, hint } = failed else {
            panic!("expected Failed");
        };
        assert_eq!(reason.chars().count(), MAX_REPORT_CHARS);
        assert_eq!(hint.chars().count(), MAX_REPORT_CHARS);
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
        let spawned = HashSet::from([100]);

        assert_eq!(accept_hello(&spawned, &ToService::Playing), Err(Refusal::NotHello));
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
}
