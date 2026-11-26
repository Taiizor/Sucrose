//! Starts the real `sucrose` binary on a prepared scratch folder and talks to it as a Portal would.
//!
//! The scratch folder lives under the target directory, never in the user's data: its settings select nothing and hide
//! the tray, so the service plays no wallpaper and shows no icon.

// Every test file of this package includes this module and uses a part of it.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use sucrose_ipc::{
    Connection, connect,
    portal::{PORTAL_PROTOCOL_VERSION, PortalEvent, PortalPage, PortalRequest},
    protocol::ToService,
    service_endpoint_for,
};

/// How long the service gets to open its endpoint.
const START_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a test waits for the next event of the service.
pub const EVENT_TIMEOUT: Duration = Duration::from_secs(5);

/// Whether this OS runs the service's event loop. The Linux and macOS loops are stubs, so a test that needs the
/// service says so and returns instead of failing there.
#[allow(clippy::print_stderr)]
pub fn service_runs(test: &str) -> bool {
    let supported = sucrose_platform::current().capabilities().desktop_wallpaper;
    if !supported {
        eprintln!("{test}: skipped, this OS has no desktop wallpaper support yet");
    }
    supported
}

/// Says on stderr that a test returned early and why, so a skipped run is visible in the output.
#[allow(clippy::print_stderr)]
pub fn skipped(test: &str, why: &str) {
    eprintln!("{test}: skipped, {why}");
}

/// A running service on its own data folder; killed when dropped.
pub struct TestService {
    /// The data folder the service runs on.
    pub dir: PathBuf,
    child: Child,
}

impl TestService {
    /// Prepares `<target tmp>/portal/<name>` with a `Setting/Sucrose.json` (nothing selected, tray hidden, an empty
    /// library) merged with `overrides` section by section, starts the service on it and waits for its endpoint.
    pub fn start(name: &str, overrides: &Value) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("portal").join(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("the scratch folder of an earlier run can be removed");
        }
        let library = dir.join("Library");
        fs::create_dir_all(&library).expect("the scratch library can be created");
        fs::create_dir_all(dir.join("Setting")).expect("the scratch settings folder can be created");
        let settings = merged(
            json!({
                "Library": { "Location": library.to_string_lossy(), "Selected": "" },
                "General": { "AppVisible": false },
            }),
            overrides,
        );
        fs::write(
            dir.join("Setting").join("Sucrose.json"),
            serde_json::to_vec_pretty(&settings).expect("the settings encode"),
        )
        .expect("the scratch settings can be written");

        Self::spawn_on(dir)
    }

    /// Starts the service on `<target tmp>/portal/<name>` with no settings file at all, so it migrates the legacy
    /// settings of this machine (read only) and plays the Video theme they select. It plays a real wallpaper on the
    /// desktop: only an ignored test that says so may use it.
    pub fn start_migrated(name: &str) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("portal").join(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("the scratch folder of an earlier run can be removed");
        }
        fs::create_dir_all(&dir).expect("the scratch folder can be created");
        Self::spawn_on(dir)
    }

    /// Stops this service and starts a new one on the same data folder, as a restart of Sucrose would.
    pub fn restart(self) -> Self {
        let dir = self.dir.clone();
        // Dropping kills the process and waits for it, so its endpoint is gone before the next one opens.
        drop(self);
        Self::spawn_on(dir)
    }

    fn spawn_on(dir: PathBuf) -> Self {
        // `--no-portal`: the start rule opens a Portal otherwise, and no test may open a window.
        let child = Command::new(env!("CARGO_BIN_EXE_sucrose"))
            .arg("--no-portal")
            .env("SUCROSE_DATA_DIR", &dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the sucrose binary starts");
        let mut service = Self { dir, child };
        service.wait_for_endpoint();
        service
    }

    /// Whether the service process ends within `timeout`.
    pub fn exits_within(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if self.child.try_wait().expect("the child can be polled").is_some() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// The saved settings file as the service wrote it.
    pub fn settings_file(&self) -> Value {
        let bytes = fs::read(self.dir.join("Setting").join("Sucrose.json")).expect("the settings file can be read");
        serde_json::from_slice(&bytes).expect("the settings file is JSON")
    }

    fn wait_for_endpoint(&mut self) {
        let deadline = Instant::now() + START_TIMEOUT;
        loop {
            if let Ok(connection) = self.connect() {
                drop(connection);
                return;
            }
            assert!(
                self.child.try_wait().expect("the child can be polled").is_none(),
                "the service exited before it opened its endpoint"
            );
            assert!(
                Instant::now() < deadline,
                "the service did not open its endpoint in time"
            );
            thread::sleep(Duration::from_millis(50));
        }
    }

    /// A new connection to this service's endpoint.
    pub fn connect(&self) -> Result<Connection, sucrose_ipc::IpcError> {
        let endpoint = service_endpoint_for(Some(self.dir.as_os_str()))?;
        connect(&endpoint.name)
    }
}

impl Drop for TestService {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Copies the fixture library of sucrose-core (every kind of theme, damaged ones included) into the scratch library of
/// `service`. Its video theme is not playable, so no test may Use it: a renderer must not flash on the desktop.
pub fn copy_fixture_library(service: &TestService) {
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join("sucrose-core")
        .join("tests")
        .join("fixtures")
        .join("library");
    copy_tree(&from, &service.dir.join("Library"));
}

/// Copies the fixture Web theme `name` of `apps/sucrose/tests/fixtures/web-themes` into the scratch library of `service`.
pub fn copy_web_fixture(service: &TestService, name: &str) {
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("web-themes")
        .join(name);
    copy_tree(&from, &service.dir.join("Library").join(name));
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("the destination folder can be created");
    for entry in fs::read_dir(from).expect("the fixture folder can be read") {
        let entry = entry.expect("the fixture entry can be read");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("the fixture type is known").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("the fixture file can be copied");
        }
    }
}

/// `base` with every section of `overrides` merged in key by key.
fn merged(mut base: Value, overrides: &Value) -> Value {
    if let (Some(base), Some(overrides)) = (base.as_object_mut(), overrides.as_object()) {
        for (section, value) in overrides {
            match (base.get_mut(section).and_then(Value::as_object_mut), value.as_object()) {
                (Some(target), Some(keys)) => target.extend(keys.iter().map(|(k, v)| (k.clone(), v.clone()))),
                _ => {
                    base.insert(section.clone(), value.clone());
                }
            }
        }
    }
    base
}

/// A Portal as the service sees it: one connection, its events read on a thread of its own.
pub struct PortalClient {
    connection: Connection,
    events: Receiver<PortalEvent>,
}

impl PortalClient {
    /// Connects, says `PortalHello` as process `pid` and returns the client with the first event of the service.
    pub fn hello_as(service: &TestService, pid: u32, page: Option<PortalPage>) -> (Self, PortalEvent) {
        let client = Self::connect_with(
            service,
            &ToService::PortalHello {
                portal_protocol_version: PORTAL_PROTOCOL_VERSION,
                pid,
                page,
            },
        );
        let first = client.next_event().expect("the service answers a PortalHello");
        (client, first)
    }

    /// [`PortalClient::hello_as`] with this process's own pid.
    pub fn hello(service: &TestService, page: Option<PortalPage>) -> (Self, PortalEvent) {
        Self::hello_as(service, std::process::id(), page)
    }

    /// Says `PortalHello` as process `pid` on a connection that is already open (for a service this test did not
    /// start) and returns the client with the first event of the service.
    pub fn hello_on(mut connection: Connection, pid: u32) -> (Self, PortalEvent) {
        connection
            .send(&ToService::PortalHello {
                portal_protocol_version: PORTAL_PROTOCOL_VERSION,
                pid,
                page: None,
            })
            .expect("the PortalHello is sent");
        let client = Self::listen(connection);
        let first = client.next_event().expect("the service answers a PortalHello");
        (client, first)
    }

    /// Connects and sends `first` as the first message, whatever it is.
    pub fn connect_with(service: &TestService, first: &ToService) -> Self {
        let mut connection = service.connect().expect("the service accepts a connection");
        connection.send(first).expect("the first message is sent");
        Self::listen(connection)
    }

    /// Connects and sends a frame whose payload is no message at all.
    pub fn connect_with_garbage(service: &TestService) -> Self {
        let mut connection = service.connect().expect("the service accepts a connection");
        connection.send(&vec![0xFF_u8; 64]).expect("the garbage frame is sent");
        Self::listen(connection)
    }

    fn listen(connection: Connection) -> Self {
        let mut reader = connection.try_clone().expect("the connection can be cloned");
        let (sender, events) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(event) = reader.recv::<PortalEvent>() {
                if sender.send(event).is_err() {
                    break;
                }
            }
        });
        Self { connection, events }
    }

    /// The next event, or `None` when none arrives within [`EVENT_TIMEOUT`] or the connection ended. The engine report
    /// the service sends after the Welcome is skipped, so a test sees the events its own requests lead to; read it with
    /// [`PortalClient::next_event_any`].
    pub fn next_event(&self) -> Option<PortalEvent> {
        loop {
            match self.next_event_any()? {
                PortalEvent::Engines(_) => {}
                event => return Some(event),
            }
        }
    }

    /// The next event of any kind, or `None` when none arrives within [`EVENT_TIMEOUT`] or the connection ended.
    pub fn next_event_any(&self) -> Option<PortalEvent> {
        self.events.recv_timeout(EVENT_TIMEOUT).ok()
    }

    /// Sends one request to the service.
    pub fn request(&mut self, request: &PortalRequest) {
        self.connection.send(request).expect("the request is sent");
    }

    /// The connection, to send raw bytes or messages on.
    pub fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}
