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
    portal::{Launch, PORTAL_PROTOCOL_VERSION, PortalEvent, PortalPage, PortalRequest},
    protocol::ToService,
    service_endpoint,
};

/// How many requests wait for the writer thread before a new one is dropped.
const REQUEST_QUEUE: usize = 64;

/// How long the Portal keeps trying to reach a service it has just started.
const CONNECT_WINDOW: Duration = Duration::from_secs(10);

/// How long one reconnect attempt keeps trying once it has started the service again.
const RECONNECT_WINDOW: Duration = Duration::from_secs(5);

/// How many times a lost connection is retried before the Portal gives up (UI-SPEC E6).
pub const MAX_RECONNECTS: u8 = 3;

/// The pause before each reconnect attempt.
const RECONNECT_GAP: Duration = Duration::from_secs(1);

/// How long the Portal waits for the service's first word before it opens its window (D-12): long enough that a
/// second Portal exits on AlreadyOpen without a window flashing.
pub const FIRST_EVENT_WAIT: Duration = Duration::from_secs(2);

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

/// The state after `event`. A lost-connection event after the last attempt is a failure; messages from the service do
/// not change the state, except that a refusal ends in `Failed`.
pub fn next_state(state: ConnectionState, event: &LinkEvent) -> ConnectionState {
    match event {
        LinkEvent::Connecting => ConnectionState::Connecting,
        LinkEvent::Connected => ConnectionState::Connected,
        LinkEvent::Lost { attempt } => match state {
            ConnectionState::Lost { attempt: done } if done >= MAX_RECONNECTS => ConnectionState::Failed,
            _ if *attempt > MAX_RECONNECTS => ConnectionState::Failed,
            _ => ConnectionState::Lost { attempt: *attempt },
        },
        LinkEvent::Failed | LinkEvent::Event(PortalEvent::Refused { .. }) => ConnectionState::Failed,
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
    retry: Sender<()>,
}

impl ServiceLink {
    /// Starts the link threads and returns the handle with the receiver of the link's events. `page` is the page
    /// the Portal was asked to open on.
    pub fn start(page: Option<PortalPage>) -> (Self, async_channel::Receiver<LinkEvent>) {
        let (events, receiver) = async_channel::unbounded();
        let (requests, queue) = bounded(REQUEST_QUEUE);
        let (retry, retries) = bounded(1);
        let slot: Slot = Arc::new(Mutex::new(None));

        let supervisor = Supervisor {
            page,
            events: events.clone(),
            retries,
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
        (Self { requests, retry }, receiver)
    }

    /// Starts a new connection cycle after the link gave up (Try Again). Does nothing while the link is still trying.
    pub fn retry(&self) {
        // A full slot already holds a retry; a closed one means the link is gone.
        let _ = self.retry.try_send(());
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

#[cfg(test)]
impl ServiceLink {
    /// A link with no service behind it: every request stays in the returned queue, for the view tests to read.
    pub fn recording() -> (Self, Receiver<PortalRequest>) {
        let (requests, queue) = bounded(REQUEST_QUEUE);
        let (retry, _) = bounded(1);
        (Self { requests, retry }, queue)
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
    Lost {
        /// Whether the service had registered the Portal (answered Welcome) before.
        welcomed: bool,
    },
}

/// When the link tells the UI it is connecting.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Announce {
    /// Not at all: a reconnect is already shown as Lost.
    Never,
    /// Only when no service answers and one has to be started.
    WhenStarting,
    /// At once: the user asked to try again.
    Now,
}

/// The `portal-ipc` thread.
struct Supervisor {
    page: Option<PortalPage>,
    events: async_channel::Sender<LinkEvent>,
    retries: Receiver<()>,
    slot: Slot,
}

impl Supervisor {
    fn emit(&self, event: LinkEvent) -> Result<(), UiGone> {
        self.events.send_blocking(event).map_err(|_| UiGone)
    }

    /// One cycle per start and per Try Again; between cycles the thread waits for the user.
    fn run(&self) {
        let mut announce = Announce::WhenStarting;
        // An UiGone result means the window is closed: the thread ends with it.
        while self.session(announce).is_ok() {
            // A retry asked for while the link was still busy is stale.
            while self.retries.try_recv().is_ok() {}
            if self.retries.recv().is_err() {
                return;
            }
            announce = Announce::Now;
        }
    }

    /// Connects, serves the connection, and after a loss reconnects [MAX_RECONNECTS] times before giving up.
    fn session(&self, announce: Announce) -> Result<(), UiGone> {
        let Some(mut connection) = self.connect_or_start(CONNECT_WINDOW, announce)? else {
            return self.emit(LinkEvent::Failed);
        };
        let mut used = 0_u8;
        loop {
            match self.serve(connection)? {
                Served::Final => return Ok(()),
                // A connection that lived to be registered earns a fresh set of attempts; one that broke before
                // that does not, so a service that dies at once cannot keep the Portal retrying forever.
                Served::Lost { welcomed } => {
                    if welcomed {
                        used = 0;
                    }
                }
            }
            let mut restored = None;
            while restored.is_none() && used < MAX_RECONNECTS {
                used += 1;
                self.emit(LinkEvent::Lost { attempt: used })?;
                thread::sleep(RECONNECT_GAP);
                restored = self.connect_or_start(RECONNECT_WINDOW, Announce::Never)?;
            }
            match restored {
                Some(next) => connection = next,
                None => return self.emit(LinkEvent::Failed),
            }
        }
    }

    /// Connects to the service; when none answers, starts it once and keeps trying for `window`.
    fn connect_or_start(&self, window: Duration, announce: Announce) -> Result<Option<Connection>, UiGone> {
        let endpoint = match service_endpoint() {
            Ok(endpoint) => endpoint,
            Err(error) => {
                tracing::error!(%error, "cannot name the service endpoint");
                return Ok(None);
            }
        };
        if announce == Announce::Now {
            self.emit(LinkEvent::Connecting)?;
        }
        match connect(&endpoint.name) {
            Ok(connection) => return Ok(Some(connection)),
            Err(error) => tracing::info!(%error, "no service answers, starting one"),
        }
        if announce == Announce::WhenStarting {
            self.emit(LinkEvent::Connecting)?;
        }
        if let Err(error) = start_service() {
            // Another process may be starting the service, so keep trying to connect.
            tracing::error!(%error, "cannot start the service");
        }
        let deadline = Instant::now() + window;
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
        let mut welcomed = false;
        let writer = match reader.try_clone() {
            Ok(writer) => writer,
            Err(error) => {
                tracing::warn!(%error, "cannot share the connection with the writer thread");
                return Ok(Served::Lost { welcomed });
            }
        };
        let hello = ToService::PortalHello {
            portal_protocol_version: PORTAL_PROTOCOL_VERSION,
            pid: std::process::id(),
            page: self.page,
        };
        if let Err(error) = reader.send(&hello) {
            tracing::warn!(%error, "cannot say hello to the service");
            return Ok(Served::Lost { welcomed });
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
                        welcomed = true;
                        self.emit(LinkEvent::Connected)?;
                    }
                    if let PortalEvent::AlreadyOpen { existing_pid } = &event {
                        // This process was just started by the user and holds the foreground right: hand it to the
                        // Portal that is open and ask the service to bring that one to the front, then exit.
                        pass_on_foreground(&mut reader, *existing_pid, self.page);
                    }
                    self.emit(LinkEvent::Event(event))?;
                    if last {
                        break Served::Final;
                    }
                }
                Err(error) => {
                    tracing::warn!(%error, "service lost");
                    break Served::Lost { welcomed };
                }
            }
        };
        *lock(&self.slot) = None;
        Ok(served)
    }
}

/// Passes the foreground right of this just-started Portal to the Portal `existing_pid` and asks the service to focus
/// it. Nothing here may stop the exit: a failed grant only means the existing window can flash instead of coming to
/// the front.
fn pass_on_foreground(connection: &mut Connection, existing_pid: u32, page: Option<PortalPage>) {
    if let Err(error) = sucrose_platform::allow_set_foreground(existing_pid) {
        tracing::warn!(%error, existing_pid, "cannot hand the foreground to the open Portal");
    }
    if let Err(error) = connection.send(&ToService::Launch(Launch::ForegroundGranted { page })) {
        tracing::warn!(%error, "cannot ask the service to focus the open Portal");
    }
}

/// The events that are ready when the Portal starts: waits up to `wait` for the first one, then takes the ones that are
/// already queued. The window opens after this, so AlreadyOpen ends the process before any window exists (D-12).
pub fn first_events(events: &async_channel::Receiver<LinkEvent>, wait: Duration) -> Vec<LinkEvent> {
    let deadline = Instant::now() + wait;
    let mut ready = Vec::new();
    loop {
        match events.try_recv() {
            Ok(event) => ready.push(event),
            Err(async_channel::TryRecvError::Empty) => {
                if !ready.is_empty() || Instant::now() >= deadline {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(async_channel::TryRecvError::Closed) => break,
        }
    }
    ready
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

#[cfg(test)]
mod tests {
    use super::*;

    fn lost(attempt: u8) -> LinkEvent {
        LinkEvent::Lost { attempt }
    }

    #[test]
    fn backoff_starts_at_50_ms_doubles_and_stops_at_500_ms() {
        let steps: Vec<u128> = (0..8).map(|attempt| backoff(attempt).as_millis()).collect();
        assert_eq!(steps, [50, 100, 200, 400, 500, 500, 500, 500]);
        assert_eq!(backoff(u32::MAX).as_millis(), 500);
    }

    #[test]
    fn connecting_becomes_connected_and_a_lost_connection_counts_its_attempts() {
        let connected = next_state(ConnectionState::Connecting, &LinkEvent::Connected);
        assert_eq!(connected, ConnectionState::Connected);
        let first = next_state(connected, &lost(1));
        assert_eq!(first, ConnectionState::Lost { attempt: 1 });
        assert_eq!(next_state(first, &lost(2)), ConnectionState::Lost { attempt: 2 });
    }

    #[test]
    fn a_lost_event_after_the_last_attempt_is_a_failure() {
        let last = ConnectionState::Lost { attempt: 3 };
        assert_eq!(next_state(last, &lost(4)), ConnectionState::Failed);
        assert_eq!(next_state(last, &lost(3)), ConnectionState::Failed);
        assert_eq!(next_state(last, &LinkEvent::Failed), ConnectionState::Failed);
    }

    #[test]
    fn try_again_starts_over_and_a_late_connection_still_counts() {
        assert_eq!(
            next_state(ConnectionState::Failed, &LinkEvent::Connecting),
            ConnectionState::Connecting
        );
        assert_eq!(
            next_state(ConnectionState::Failed, &LinkEvent::Connected),
            ConnectionState::Connected
        );
    }

    #[test]
    fn a_refusal_ends_in_failed_and_other_service_messages_change_nothing() {
        let refused = LinkEvent::Event(PortalEvent::Refused {
            reason: "old".to_owned(),
        });
        assert_eq!(
            next_state(ConnectionState::Connecting, &refused),
            ConnectionState::Failed
        );
        let focus = LinkEvent::Event(PortalEvent::Focus { page: None });
        assert_eq!(
            next_state(ConnectionState::Connected, &focus),
            ConnectionState::Connected
        );
        let already = LinkEvent::Event(PortalEvent::AlreadyOpen { existing_pid: 1 });
        assert_eq!(
            next_state(ConnectionState::Connecting, &already),
            ConnectionState::Connecting
        );
    }
}
