//! The Portal's link to the service (D-009, D-033): finds the service or starts it, says PortalHello, and hands every
//! event to the UI thread.
//!
//! No I/O runs on the GPUI thread (D-012): one thread owns the connection and reads it, one writes it, and events
//! reach the UI through an async channel that the UI awaits inside `cx.spawn`.

use std::{
    io,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded};
use sucrose_ipc::{
    Connection, connect,
    portal::{PORTAL_PROTOCOL_VERSION, PortalEvent, PortalPage, PortalRequest},
    protocol::ToService,
    service_endpoint,
};

/// How many requests wait for the writer thread before a new one is dropped.
const REQUEST_QUEUE: usize = 64;

/// How long the Portal keeps trying to reach a service it has just started.
const CONNECT_WINDOW: Duration = Duration::from_secs(10);

/// The argument the Portal starts the service with: the service must not open a second Portal (D-12).
const SERVICE_ARGUMENT: &str = "--no-portal";

/// What the link tells the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    /// The link is looking for the service, or starting it.
    Connecting,
    /// The service accepted the Portal and answered with Welcome.
    Connected,
    /// A message from the service.
    Event(PortalEvent),
    /// The connection to the service ended. `attempt` is the reconnect attempt that is about to start.
    Lost {
        /// Which reconnect attempt this is, counting from 1.
        attempt: u8,
    },
    /// The service cannot be reached.
    Failed,
}

/// Where the connection to the service stands, as the UI shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Looking for the service, or starting it.
    Connecting,
    /// Registered with the service.
    Connected,
    /// The connection ended and the link is trying to restore it.
    Lost {
        /// The reconnect attempt in progress.
        attempt: u8,
    },
    /// The service cannot be reached.
    Failed,
}

/// The state after `event`. Messages from the service do not change it, except that a refusal ends in `Failed`.
pub fn next_state(state: ConnectionState, event: &LinkEvent) -> ConnectionState {
    match event {
        LinkEvent::Connecting => ConnectionState::Connecting,
        LinkEvent::Connected => ConnectionState::Connected,
        LinkEvent::Lost { attempt } => ConnectionState::Lost { attempt: *attempt },
        LinkEvent::Failed => ConnectionState::Failed,
        LinkEvent::Event(_) => state,
    }
}

/// How long to wait before connect try number `attempt` (from 0): 50 ms, doubling, at most 500 ms.
pub fn backoff(attempt: u32) -> Duration {
    let millis = 50_u64.saturating_mul(1_u64 << attempt.min(8));
    Duration::from_millis(millis.min(500))
}

/// The handle the UI keeps: sends requests to the service. Cloning it shares the same connection.
#[derive(Debug, Clone)]
pub struct ServiceLink {
    requests: Sender<PortalRequest>,
}

impl ServiceLink {
    /// Starts the link threads and returns the handle with the receiver of the link's events. `page` is the page
    /// the Portal was asked to open on.
    pub fn start(page: Option<PortalPage>) -> (Self, async_channel::Receiver<LinkEvent>) {
        let (events, receiver) = async_channel::unbounded();
        let (requests, queue) = bounded(REQUEST_QUEUE);
        let slot: Slot = Arc::new(Mutex::new(None));

        let supervisor = Supervisor {
            page,
            events: events.clone(),
            slot: Arc::clone(&slot),
        };
        let started = thread::Builder::new()
            .name("portal-ipc".to_owned())
            .spawn(move || supervisor.run())
            .and_then(|_| {
                thread::Builder::new()
                    .name("portal-ipc-writer".to_owned())
                    .spawn(move || write_requests(&queue, &slot))
            });
        if let Err(error) = started {
            tracing::error!(%error, "cannot start the threads that talk to the service");
            // The channel is unbounded and its receiver is alive, so this cannot fail.
            let _ = events.try_send(LinkEvent::Failed);
        }
        (Self { requests }, receiver)
    }

    /// Queues a request for the service. Never blocks the caller: when the queue is full or the writer is gone the
    /// request is dropped and logged.
    #[allow(
        dead_code,
        reason = "the library and settings pages of the following plans send requests"
    )]
    pub fn send(&self, request: PortalRequest) {
        match self.requests.try_send(request) {
            Ok(()) => {}
            Err(TrySendError::Full(request)) => tracing::warn!(?request, "the request queue is full, dropped"),
            Err(TrySendError::Disconnected(request)) => {
                tracing::warn!(?request, "the writer thread is gone, dropped");
            }
        }
    }
}

/// The write half of the current connection, shared by the reader thread (which fills and clears it) and the writer.
type Slot = Arc<Mutex<Option<Connection>>>;

fn lock(slot: &Slot) -> MutexGuard<'_, Option<Connection>> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The writer thread: sends queued requests through the current connection, if there is one.
fn write_requests(queue: &Receiver<PortalRequest>, slot: &Slot) {
    while let Ok(request) = queue.recv() {
        let mut guard = lock(slot);
        match guard.as_mut() {
            Some(connection) => {
                if let Err(error) = connection.send(&request) {
                    tracing::warn!(%error, "cannot send a request to the service");
                    *guard = None;
                }
            }
            None => tracing::warn!(?request, "the service is not connected, request dropped"),
        }
    }
}

/// The UI stopped listening: the link has nothing left to do.
struct UiGone;

/// How a connection ended.
enum Served {
    /// The service answered AlreadyOpen, Refused or Close: it ended the connection on purpose.
    Final,
    /// The connection broke.
    Lost,
}

/// The `portal-ipc` thread.
struct Supervisor {
    page: Option<PortalPage>,
    events: async_channel::Sender<LinkEvent>,
    slot: Slot,
}

impl Supervisor {
    fn emit(&self, event: LinkEvent) -> Result<(), UiGone> {
        self.events.send_blocking(event).map_err(|_| UiGone)
    }

    fn run(&self) {
        // An UiGone result means the window is closed: the thread ends with it.
        let _ = self.session();
    }

    fn session(&self) -> Result<(), UiGone> {
        let Some(connection) = self.connect_or_start()? else {
            return self.emit(LinkEvent::Failed);
        };
        match self.serve(connection)? {
            Served::Final => Ok(()),
            Served::Lost => self.emit(LinkEvent::Failed),
        }
    }

    /// Connects to the service; when none answers, starts it once and keeps trying for [CONNECT_WINDOW].
    fn connect_or_start(&self) -> Result<Option<Connection>, UiGone> {
        let endpoint = match service_endpoint() {
            Ok(endpoint) => endpoint,
            Err(error) => {
                tracing::error!(%error, "cannot name the service endpoint");
                return Ok(None);
            }
        };
        match connect(&endpoint.name) {
            Ok(connection) => return Ok(Some(connection)),
            Err(error) => tracing::info!(%error, "no service answers, starting one"),
        }
        self.emit(LinkEvent::Connecting)?;
        if let Err(error) = start_service() {
            // Another process may be starting the service, so keep trying to connect.
            tracing::error!(%error, "cannot start the service");
        }
        let deadline = Instant::now() + CONNECT_WINDOW;
        for attempt in 0_u32.. {
            thread::sleep(backoff(attempt));
            match connect(&endpoint.name) {
                Ok(connection) => return Ok(Some(connection)),
                Err(error) => tracing::debug!(%error, attempt, "the service does not answer yet"),
            }
            if Instant::now() >= deadline {
                break;
            }
        }
        tracing::error!("the service did not answer in time");
        Ok(None)
    }

    /// Says PortalHello and forwards the service's events until the connection ends.
    fn serve(&self, mut reader: Connection) -> Result<Served, UiGone> {
        let writer = match reader.try_clone() {
            Ok(writer) => writer,
            Err(error) => {
                tracing::warn!(%error, "cannot share the connection with the writer thread");
                return Ok(Served::Lost);
            }
        };
        let hello = ToService::PortalHello {
            portal_protocol_version: PORTAL_PROTOCOL_VERSION,
            pid: std::process::id(),
            page: self.page,
        };
        if let Err(error) = reader.send(&hello) {
            tracing::warn!(%error, "cannot say hello to the service");
            return Ok(Served::Lost);
        }
        *lock(&self.slot) = Some(writer);

        let served = loop {
            match reader.recv::<PortalEvent>() {
                Ok(event) => {
                    let last = matches!(
                        event,
                        PortalEvent::AlreadyOpen { .. } | PortalEvent::Refused { .. } | PortalEvent::Close
                    );
                    if matches!(event, PortalEvent::Welcome { .. }) {
                        tracing::info!("connected to the service");
                        self.emit(LinkEvent::Connected)?;
                    }
                    self.emit(LinkEvent::Event(event))?;
                    if last {
                        break Served::Final;
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "service lost");
                    break Served::Lost;
                }
            }
        };
        *lock(&self.slot) = None;
        Ok(served)
    }
}

/// The service binary: the fixed name next to this executable (T-03-19). Never a PATH lookup.
fn service_path() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let folder = exe
        .parent()
        .ok_or_else(|| io::Error::other("the Portal executable has no folder"))?;
    Ok(folder.join(format!("sucrose{}", std::env::consts::EXE_SUFFIX)))
}

/// Starts the service that does not open a Portal itself, from an argument vector (never a shell string).
fn start_service() -> io::Result<()> {
    let path = service_path()?;
    let mut command = Command::new(&path);
    command.arg(SERVICE_ARGUMENT);
    sucrose_platform::spawn_detached(&mut command)?;
    tracing::info!(path = %path.display(), "started the service");
    Ok(())
}
