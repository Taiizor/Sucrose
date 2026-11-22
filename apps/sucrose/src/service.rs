//! Service mode: owns the endpoint (single instance, D-023), starts a renderer and listens to it.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex, PoisonError},
    thread,
    time::Duration,
};

use anyhow::Context as _;
use crossbeam_channel::Sender;
use sucrose_config::legacy::{LegacySettings, LibrarySettings};
use sucrose_core::{
    paths::{legacy_data_dir, legacy_setting_dir},
    theme::{SucroseInfo, WallpaperType, read_info},
};
use sucrose_ipc::{
    Connection, IpcError, Listener, endpoint_name,
    protocol::{Assignment, PROTOCOL_VERSION, ToRenderer, ToService},
};
use sucrose_platform::{Platform, PlatformError};

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

/// Runs the service until the process is stopped.
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
        os = platform.name(),
        capabilities = ?platform.capabilities(),
        endpoint = %endpoint.name,
        dpi = ?awareness,
        "service started"
    );
    let assignments: Arc<Vec<Assignment>> = Arc::new(plan_wallpapers(platform.as_ref()));

    let (sender, receiver) = crossbeam_channel::unbounded();
    let spawned = Arc::new(Mutex::new(HashSet::new()));
    for assignment in assignments.iter() {
        spawn_renderer(&spawned, assignment.slot)?;
    }
    let accept_spawned = Arc::clone(&spawned);
    let accept_assignments = Arc::clone(&assignments);
    thread::Builder::new()
        .name("ipc-accept".to_owned())
        .spawn(move || accept_loop(&listener, &accept_spawned, &accept_assignments, &sender))
        .context("cannot start the accept thread")?;

    // The accept thread and the reader threads hold the only senders: when they are all gone nothing can arrive.
    for message in receiver {
        match message {
            ToService::Hello { pid, slot, .. } => tracing::info!(pid, slot, "renderer connected"),
        }
    }
    tracing::warn!("the service channel closed, exiting");
    Ok(())
}

/// Starts `sucrose --render <slot>` and remembers its pid as a process this service started.
fn spawn_renderer(spawned: &Mutex<HashSet<u32>>, slot: u8) -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("cannot find the path of this executable")?;
    // Held across the spawn so a fast renderer cannot say Hello before its pid is recorded.
    let mut pids = spawned.lock().unwrap_or_else(PoisonError::into_inner);
    let child = Command::new(exe)
        .arg("--render")
        .arg(slot.to_string())
        .spawn()
        .context("cannot start the renderer")?;
    pids.insert(child.id());
    tracing::info!(pid = child.id(), slot, "renderer started");
    Ok(())
}

/// Accepts connections and gives each one a reader thread.
fn accept_loop(
    listener: &Listener,
    spawned: &Arc<Mutex<HashSet<u32>>>,
    assignments: &Arc<Vec<Assignment>>,
    sender: &Sender<ToService>,
) {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    while failures < MAX_FAILURES {
        match listener.accept() {
            Ok(connection) => {
                failures = 0;
                let spawned = Arc::clone(spawned);
                let assignments = Arc::clone(assignments);
                let sender = sender.clone();
                let started = thread::Builder::new()
                    .name("ipc-reader".to_owned())
                    .spawn(move || read_connection(connection, &spawned, &assignments, &sender));
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
    spawned: &Mutex<HashSet<u32>>,
    assignments: &[Assignment],
    sender: &Sender<ToService>,
) {
    let first: ToService = match connection.recv() {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(%error, "dropping a connection whose first message is not valid");
            return;
        }
    };
    let accepted = {
        let pids = spawned.lock().unwrap_or_else(PoisonError::into_inner);
        accept_hello(&pids, &first)
    };
    let slot = match accepted {
        Ok(slot) => slot,
        Err(refusal) => {
            tracing::warn!(%refusal, "refusing a connection");
            return;
        }
    };
    // The renderer waits for this message; without a job for its slot it has nothing to do, so it is told to stop.
    let reply = match assignments.iter().find(|assignment| assignment.slot == slot) {
        Some(assignment) => ToRenderer::Assign(assignment.clone()),
        None => ToRenderer::Close,
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
    if sender.send(first).is_err() {
        return;
    }
    loop {
        match connection.recv::<ToService>() {
            Ok(message) => {
                if sender.send(message).is_err() {
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

/// Why the service refuses a renderer's Hello.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The renderer speaks another protocol version.
    ProtocolVersion { got: u16, expected: u16 },
    /// The pid is not a process this service started.
    UnknownPid(u32),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtocolVersion { got, expected } => {
                write!(f, "the renderer speaks protocol {got}, this service speaks {expected}")
            }
            Self::UnknownPid(pid) => write!(f, "pid {pid} is not a renderer this service started"),
        }
    }
}

/// Decides whether a connection's first message is acceptable and returns the renderer's slot.
fn accept_hello(spawned: &HashSet<u32>, message: &ToService) -> Result<u8, Refusal> {
    let ToService::Hello {
        protocol_version,
        pid,
        slot,
    } = message;
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

/// Reads the settings of the C# install, read-only from its real folder (D-008), logs what the service will use and
/// returns the wallpaper jobs it leads to: one renderer for the primary monitor when the selected theme is a video
/// this build can play, none otherwise.
fn plan_wallpapers(platform: &dyn Platform) -> Vec<Assignment> {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return Vec::new();
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
    match wallpaper_for(&settings.library, platform) {
        Ok(assignment) => vec![assignment],
        Err(reason) => {
            tracing::warn!(%reason, "no wallpaper starts, the service keeps waiting without a renderer");
            Vec::new()
        }
    }
}

/// The job for the primary monitor, or the reason there is none.
fn wallpaper_for(library: &LibrarySettings, platform: &dyn Platform) -> Result<Assignment, String> {
    if library.selected.is_empty() {
        return Err("no theme is selected in the C# install".to_owned());
    }
    let theme_dir = library.location.join(&library.selected);
    let info = read_info(&theme_dir).map_err(|error| format!("the selected theme cannot be read: {error}"))?;
    tracing::info!(theme = %library.selected, kind = ?info.kind, source = %info.source, "selected theme");
    let source = video_source(&theme_dir, &info)?;
    let monitors = platform.monitors();
    let monitor = monitors
        .iter()
        .find(|monitor| monitor.primary)
        .or_else(|| monitors.first())
        .ok_or_else(|| "the OS reports no monitor".to_owned())?;
    let source = source
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

/// Checks that a theme's `Source` is a relative path inside its folder (T-02-13): no `..`, no address, no drive or
/// UNC prefix. Plan 08 adds canonicalized containment.
fn safe_relative_source(source: &str) -> Result<(), &'static str> {
    if source.is_empty() {
        Err("the theme has no Source")
    } else if source.contains("://") {
        Err("the Source is an address, not a file")
    } else if source.starts_with(['/', '\\']) {
        Err("the Source is an absolute or UNC path")
    } else if source.contains(':') {
        Err("the Source has a drive prefix")
    } else if source.split(['/', '\\']).any(|part| part == "..") {
        Err("the Source leaves the theme folder")
    } else {
        Ok(())
    }
}

/// The absolute path of the video a Video theme plays, or why it cannot be played.
fn video_source(theme_dir: &Path, info: &SucroseInfo) -> Result<PathBuf, String> {
    if info.kind != WallpaperType::Video {
        return Err(format!(
            "the selected theme is a {:?} wallpaper, only Video plays in this build",
            info.kind
        ));
    }
    safe_relative_source(&info.source).map_err(str::to_owned)?;
    let path = theme_dir.join(&info.source);
    if !path.is_file() {
        return Err("the Source file does not exist in the theme folder".to_owned());
    }
    std::path::absolute(&path).map_err(|error| format!("cannot make the Source path absolute: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(protocol_version: u16, pid: u32, slot: u8) -> ToService {
        ToService::Hello {
            protocol_version,
            pid,
            slot,
        }
    }

    fn info(kind: WallpaperType, source: &str) -> SucroseInfo {
        SucroseInfo {
            app_version: serde_json::Value::Null,
            title: String::new(),
            kind,
            source: source.to_owned(),
        }
    }

    /// A fresh theme folder with `clip.mp4` in it, under the target directory of the test run.
    fn theme_folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sucrose-service-test-{}-{name}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).expect("create theme folder");
        std::fs::write(dir.join("clip.mp4"), b"x").expect("write clip");
        std::fs::write(dir.join("sub").join("deep.mp4"), b"x").expect("write deep clip");
        dir
    }

    #[test]
    fn video_source_takes_an_existing_relative_file() {
        let dir = theme_folder("ok");

        let path = video_source(&dir, &info(WallpaperType::Video, "clip.mp4")).expect("playable");
        let nested = video_source(&dir, &info(WallpaperType::Video, "sub/deep.mp4")).expect("playable");

        assert!(path.is_absolute() && path.ends_with("clip.mp4"));
        assert!(nested.is_absolute());
    }

    #[test]
    fn video_source_refuses_paths_that_leave_the_theme_folder() {
        let dir = theme_folder("escape");

        for source in [
            "../clip.mp4",
            "sub/../../x.mp4",
            "sub\\..\\..\\x.mp4",
            "/etc/passwd",
            "\\\\server\\share\\x.mp4",
            "C:\\x.mp4",
            "C:x.mp4",
            "https://example.com/x.mp4",
            "file:///x.mp4",
            "",
        ] {
            assert!(
                video_source(&dir, &info(WallpaperType::Video, source)).is_err(),
                "{source:?} must be refused"
            );
        }
    }

    #[test]
    fn video_source_refuses_a_missing_file_and_other_types() {
        let dir = theme_folder("other");

        assert!(video_source(&dir, &info(WallpaperType::Video, "missing.mp4")).is_err());
        assert!(video_source(&dir, &info(WallpaperType::Video, "sub")).is_err());
        assert!(video_source(&dir, &info(WallpaperType::Gif, "clip.mp4")).is_err());
        assert!(video_source(&dir, &info(WallpaperType::Web, "clip.mp4")).is_err());
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
}
