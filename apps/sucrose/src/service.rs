//! Service mode: owns the endpoint (single instance, D-023), starts a renderer and listens to it.

use std::{
    collections::HashSet,
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
    theme::read_info,
};
use sucrose_ipc::{
    Connection, IpcError, Listener, endpoint_name,
    protocol::{PROTOCOL_VERSION, ToService},
};

/// The endpoint name of this user's service, shared by the service and its renderers.
pub fn endpoint() -> String {
    // Replaced by the account SID where the platform provides one.
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_owned());
    endpoint_name(&user)
}

/// Runs the service until the process is stopped.
pub fn run() -> anyhow::Result<()> {
    let endpoint = endpoint();
    let listener = match Listener::bind(&endpoint, None) {
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
        %endpoint,
        "service started"
    );
    log_legacy_settings();

    let (sender, receiver) = crossbeam_channel::unbounded();
    let spawned = Arc::new(Mutex::new(HashSet::new()));
    spawn_renderer(&spawned, 0)?;
    thread::Builder::new()
        .name("ipc-accept".to_owned())
        .spawn(move || accept_loop(&listener, &sender))
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
fn accept_loop(listener: &Listener, sender: &Sender<ToService>) {
    const MAX_FAILURES: u32 = 10;
    let mut failures = 0;
    while failures < MAX_FAILURES {
        match listener.accept() {
            Ok(connection) => {
                failures = 0;
                let sender = sender.clone();
                let started = thread::Builder::new()
                    .name("ipc-reader".to_owned())
                    .spawn(move || read_connection(connection, &sender));
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
fn read_connection(mut connection: Connection, sender: &Sender<ToService>) {
    let first: ToService = match connection.recv() {
        Ok(message) => message,
        Err(error) => {
            tracing::warn!(%error, "dropping a connection whose first message is not valid");
            return;
        }
    };
    if first_is_wrong(&first) {
        return;
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

/// Logs and reports a first message the service refuses.
fn first_is_wrong(first: &ToService) -> bool {
    let ToService::Hello { protocol_version, .. } = first;
    if *protocol_version == PROTOCOL_VERSION {
        return false;
    }
    tracing::warn!(
        protocol_version,
        expected = PROTOCOL_VERSION,
        "refusing a renderer that speaks another protocol version"
    );
    true
}

/// Reads the settings of the C# install, read-only from its real folder (D-008), and logs what the service will use.
fn log_legacy_settings() {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return;
    };
    let settings = LegacySettings::load(&setting_dir, &root);
    log_selected_theme(&settings.library);
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
}

/// Logs the theme selected in the C# install.
fn log_selected_theme(library: &LibrarySettings) {
    if library.selected.is_empty() {
        tracing::info!("no theme selected in the C# install");
        return;
    }
    match read_info(&library.location.join(&library.selected)) {
        Ok(info) => tracing::info!(
            theme = %library.selected,
            kind = ?info.kind,
            source = %info.source,
            "selected theme"
        ),
        Err(error) => tracing::warn!(%error, "selected theme cannot be read"),
    }
}
