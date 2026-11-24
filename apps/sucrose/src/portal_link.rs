//! The Portal's side of the service (D-009, D-033): the one registered Portal, its ordered writer, the admission
//! decision for a connecting Portal and the view of the settings it gets.
//!
//! The registry lives on the service's main thread, so two Portals that connect at the same moment are decided in one
//! place: exactly one is registered, the other is told who is open.

use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use sucrose_config::model::Settings;
use sucrose_ipc::{
    Connection, connect,
    portal::{
        Launch, LaunchReply, MAX_PATH_TEXT, PORTAL_PROTOCOL_VERSION, PortalEvent, PortalPage, RuleState, RuntimeState,
    },
    protocol::ToService,
};

use crate::service::{OUTGOING_CAPACITY, Offer};

/// The write side of one Portal's connection. Events go into a bounded queue that a writer thread of its own drains,
/// so a Portal that stops reading blocks that thread only, never the main thread (T-03-14). One writer per connection
/// keeps the order of the events.
#[derive(Debug)]
pub struct PortalLink {
    /// Identifies this connection, so a Portal that claims a pid of another one is never taken for it.
    connection: u64,
    /// The pid the Portal reported in its `PortalHello`.
    pid: u32,
    queue: Sender<PortalEvent>,
}

impl PortalLink {
    /// Starts the writer thread `name`, which calls `write` for each queued event and ends when `write` says the
    /// connection is broken or the link is dropped (after sending what was queued).
    pub fn start(
        name: String,
        connection: u64,
        pid: u32,
        mut write: impl FnMut(PortalEvent) -> bool + Send + 'static,
    ) -> io::Result<Self> {
        let (queue, receiver) = crossbeam_channel::bounded::<PortalEvent>(OUTGOING_CAPACITY);
        thread::Builder::new().name(name).spawn(move || {
            while let Ok(event) = receiver.recv() {
                if !write(event) {
                    break;
                }
            }
        })?;
        Ok(Self { connection, pid, queue })
    }

    /// The writer of `connection`, a clone of the handle the reader thread keeps.
    pub fn for_connection(id: u64, pid: u32, mut connection: Connection) -> io::Result<Self> {
        Self::start(format!("portal-writer-{id}"), id, pid, move |event| {
            match connection.send(&event) {
                Ok(()) => true,
                Err(error) => {
                    tracing::warn!(%error, "cannot send an event to the portal, its writer stops");
                    false
                }
            }
        })
    }

    /// The connection this link writes to.
    pub fn connection(&self) -> u64 {
        self.connection
    }

    /// The pid the Portal reported.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Queues `event` without ever waiting. A memory reading that finds the queue full is dropped: the next one
    /// replaces it. Any other event that does is a failure, because the Portal is stuck and losing the event would
    /// leave it showing a wrong state; the connection is given up and the Portal reconnects.
    pub fn offer(&self, event: PortalEvent) -> Offer {
        let skippable = matches!(event, PortalEvent::Memory(_));
        match self.queue.try_send(event) {
            Ok(()) => Offer::Queued,
            Err(crossbeam_channel::TrySendError::Full(_)) if skippable => Offer::Dropped,
            Err(_) => Offer::Failed,
        }
    }
}

/// What the service does with a Portal that says hello.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalAdmission {
    /// It is the one Portal: register it and send Welcome.
    Register,
    /// Another Portal is registered: tell the newcomer. The registered one is told to focus once the newcomer has
    /// passed on the foreground right (`Launch::ForegroundGranted`).
    AlreadyOpen {
        /// Process id of the registered Portal.
        existing_pid: u32,
    },
    /// The service is stopping: tell it to close.
    Close,
    /// The service cannot talk to it: tell it why.
    Refuse(String),
}

/// Decides what happens to a connecting Portal. `registered` is the pid of the Portal that is registered now.
pub fn accept_portal(registered: Option<u32>, closing: bool, version: u16) -> PortalAdmission {
    if version != PORTAL_PROTOCOL_VERSION {
        return PortalAdmission::Refuse(format!(
            "the portal speaks protocol {version}, this service speaks {PORTAL_PROTOCOL_VERSION}"
        ));
    }
    if closing {
        return PortalAdmission::Close;
    }
    match registered {
        Some(existing_pid) => PortalAdmission::AlreadyOpen { existing_pid },
        None => PortalAdmission::Register,
    }
}

/// The one registered Portal, owned by the main thread.
#[derive(Debug, Default)]
pub struct PortalRegistry {
    registered: Option<PortalLink>,
}

impl PortalRegistry {
    /// The pid of the registered Portal, if one is.
    pub fn registered_pid(&self) -> Option<u32> {
        self.registered.as_ref().map(PortalLink::pid)
    }

    /// Whether `connection` is the registered Portal's.
    pub fn is_registered(&self, connection: u64) -> bool {
        self.registered
            .as_ref()
            .is_some_and(|link| link.connection() == connection)
    }

    /// Admits or turns away a Portal whose connection is `newcomer`, answering it through its own writer.
    /// `welcome` builds the Welcome event, only when the Portal is registered.
    pub fn admit(
        &mut self,
        newcomer: PortalLink,
        version: u16,
        closing: bool,
        welcome: impl FnOnce() -> PortalEvent,
    ) -> PortalAdmission {
        let admission = accept_portal(self.registered.as_ref().map(PortalLink::pid), closing, version);
        match &admission {
            PortalAdmission::Register => {
                if newcomer.offer(welcome()) == Offer::Queued {
                    self.registered = Some(newcomer);
                } else {
                    tracing::warn!("cannot queue Welcome for a portal, it is not registered");
                }
            }
            PortalAdmission::AlreadyOpen { existing_pid } => {
                newcomer.offer(PortalEvent::AlreadyOpen {
                    existing_pid: *existing_pid,
                });
            }
            PortalAdmission::Close => {
                newcomer.offer(PortalEvent::Close);
            }
            PortalAdmission::Refuse(reason) => {
                newcomer.offer(PortalEvent::Refused { reason: reason.clone() });
            }
        }
        admission
    }

    /// Sends `event` to the registered Portal. A Portal whose queue is full is stuck: its connection is given up and
    /// the slot is free for a Portal that reconnects.
    pub fn send(&mut self, event: PortalEvent) {
        let Some(link) = &self.registered else {
            return;
        };
        if link.offer(event) == Offer::Failed {
            tracing::warn!(
                pid = link.pid(),
                "the portal does not keep up or is gone, its connection is dropped"
            );
            self.registered = None;
        }
    }

    /// Forgets the Portal of `connection` once it has gone. `true` when it was the registered one.
    pub fn gone(&mut self, connection: u64) -> bool {
        if self.is_registered(connection) {
            self.registered = None;
            true
        } else {
            false
        }
    }
}

/// `text` cut to [`MAX_PATH_TEXT`] bytes at a character boundary.
pub(crate) fn clip(text: &str) -> String {
    let mut end = text.len().min(MAX_PATH_TEXT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.get(..end).unwrap_or_default().to_owned()
}

/// How long a Portal the service just started has to register before the service starts another one for a launcher
/// (T-03-35).
const SPAWN_WINDOW: Duration = Duration::from_secs(10);

/// Whether a Portal started at `spawned` is still expected to register: a launcher that comes now must not start
/// another one.
pub fn spawn_pending(spawned: Option<Instant>, now: Instant) -> bool {
    spawned.is_some_and(|at| now.saturating_duration_since(at) < SPAWN_WINDOW)
}

/// The arguments the Portal starts with for `page`. Chosen here from the enum, never from text a client sent
/// (T-03-34); the Portal reads no argument for the other pages yet.
pub fn portal_args(page: Option<PortalPage>) -> Vec<&'static str> {
    match page {
        Some(PortalPage::General) => vec!["GeneralSetting"],
        Some(PortalPage::Library | PortalPage::Personal) | None => Vec::new(),
    }
}

/// How long a second `sucrose` waits for the running service's answer.
const LAUNCH_WAIT: Duration = Duration::from_secs(5);

/// The launcher side of a second `sucrose` (D-12): asks the service on `endpoint` to open or raise the Portal and
/// returns its answer. The exchange runs on a thread of its own because a pipe read has no timeout; a service that
/// does not answer within [`LAUNCH_WAIT`] is an error, never a hang.
pub fn ask_service_to_open_portal(endpoint: &str) -> Result<LaunchReply, String> {
    let (result, answer) = crossbeam_channel::bounded(1);
    let endpoint = endpoint.to_owned();
    thread::Builder::new()
        .name("launcher".to_owned())
        .spawn(move || {
            // The receiver may be gone after a timeout; the answer is then of no use.
            let _ = result.send(exchange(&endpoint));
        })
        .map_err(|error| format!("cannot start the launcher thread: {error}"))?;
    answer
        .recv_timeout(LAUNCH_WAIT)
        .map_err(|_| "the service did not answer in time".to_owned())?
}

/// One request and one reply on a connection of its own.
fn exchange(endpoint: &str) -> Result<LaunchReply, String> {
    let mut connection = connect(endpoint).map_err(|error| format!("cannot reach the service: {error}"))?;
    connection
        .send(&ToService::Launch(Launch::OpenPortal { page: None }))
        .map_err(|error| format!("cannot send the request: {error}"))?;
    let reply = connection
        .recv::<LaunchReply>()
        .map_err(|error| format!("cannot read the answer: {error}"))?;
    if let LaunchReply::PortalRunning { pid } = &reply {
        // This process was just started by the user, so it may hand the foreground to the Portal that is open; the
        // service then tells that Portal to come to the front. A failed grant only means the window may flash instead.
        if let Err(error) = sucrose_platform::allow_set_foreground(*pid) {
            tracing::warn!(%error, pid, "cannot hand the foreground to the Portal");
        }
        connection
            .send(&ToService::Launch(Launch::ForegroundGranted { page: None }))
            .map_err(|error| format!("cannot pass on the foreground: {error}"))?;
    }
    Ok(reply)
}

/// What the service tells the Portal it is doing: the selected theme from the settings, whether a renderer runs, what
/// the fullscreen rule does to the wallpaper, whether the library is being moved and whether the tray exists. It is
/// never written to Sucrose.json (D-23).
pub fn runtime_state(
    settings: &Settings,
    running: bool,
    rule: RuleState,
    library_moving: bool,
    tray_visible: bool,
) -> RuntimeState {
    RuntimeState {
        selected: Some(settings.library.selected.as_str())
            .filter(|selected| !selected.is_empty())
            .map(clip),
        running,
        rule,
        library_moving,
        tray_visible,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex, PoisonError},
        time::{Duration, Instant},
    };

    use sucrose_ipc::portal::MemoryReport;

    use super::*;

    #[test]
    fn a_spawned_portal_is_expected_for_ten_seconds_only() {
        let start = Instant::now();
        assert!(!spawn_pending(None, start));
        assert!(spawn_pending(Some(start), start + Duration::from_secs(9)));
        assert!(!spawn_pending(Some(start), start + Duration::from_secs(10)));
    }

    #[test]
    fn the_portal_argument_comes_from_the_page_enum() {
        assert_eq!(portal_args(None), Vec::<&str>::new());
        assert_eq!(portal_args(Some(PortalPage::Library)), Vec::<&str>::new());
        assert_eq!(portal_args(Some(PortalPage::General)), ["GeneralSetting"]);
    }

    fn recording_link(connection: u64, pid: u32) -> (PortalLink, Arc<Mutex<Vec<PortalEvent>>>) {
        let written = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&written);
        let link = PortalLink::start("test-portal-writer".to_owned(), connection, pid, move |event| {
            record.lock().unwrap_or_else(PoisonError::into_inner).push(event);
            true
        })
        .expect("the writer thread starts");
        (link, written)
    }

    fn wait_for(written: &Mutex<Vec<PortalEvent>>, count: usize) -> Vec<PortalEvent> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let events = written.lock().unwrap_or_else(PoisonError::into_inner).clone();
            if events.len() >= count {
                return events;
            }
            assert!(Instant::now() < deadline, "the events were not written");
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// A writer that takes one event and then blocks, like a pipe write to a Portal that does not read.
    struct Gated {
        link: PortalLink,
        holding: crossbeam_channel::Receiver<()>,
        release: Sender<()>,
        written: Arc<Mutex<Vec<PortalEvent>>>,
    }

    fn gated_link() -> Gated {
        let (holding_tx, holding) = crossbeam_channel::unbounded::<()>();
        let (release, release_rx) = crossbeam_channel::unbounded::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&written);
        let link = PortalLink::start("test-portal-writer".to_owned(), 1, 100, move |event| {
            let _ = holding_tx.send(());
            let _ = release_rx.recv();
            record.lock().unwrap_or_else(PoisonError::into_inner).push(event);
            true
        })
        .expect("the writer thread starts");
        Gated {
            link,
            holding,
            release,
            written,
        }
    }

    fn memory() -> PortalEvent {
        PortalEvent::Memory(MemoryReport {
            service: 1,
            renderers: Vec::new(),
            portal: 0,
        })
    }

    #[test]
    fn a_portal_that_does_not_read_never_blocks_the_sender_and_loses_only_memory_events() {
        let Gated {
            link,
            holding,
            release,
            written,
        } = gated_link();
        // The writer takes the first event and is stuck writing it.
        assert_eq!(link.offer(PortalEvent::Focus { page: None }), Offer::Queued);
        holding
            .recv_timeout(Duration::from_secs(5))
            .expect("the writer holds an event");

        for _ in 0..OUTGOING_CAPACITY {
            assert_eq!(link.offer(memory()), Offer::Queued);
        }
        assert_eq!(link.offer(memory()), Offer::Dropped, "a memory reading is skipped");
        assert_eq!(
            link.offer(PortalEvent::Close),
            Offer::Failed,
            "any other event ends the connection"
        );

        // Letting the Portal read again delivers what was queued, in order.
        drop(release);
        let events = wait_for(&written, 1 + OUTGOING_CAPACITY);
        assert_eq!(events.first(), Some(&PortalEvent::Focus { page: None }));
        assert!(events[1..].iter().all(|event| matches!(event, PortalEvent::Memory(_))));
    }

    #[test]
    fn a_broken_connection_fails_every_later_offer() {
        let link =
            PortalLink::start("test-portal-writer".to_owned(), 1, 100, |_| false).expect("the writer thread starts");
        assert_eq!(link.offer(PortalEvent::Close), Offer::Queued);

        let deadline = Instant::now() + Duration::from_secs(5);
        while link.offer(PortalEvent::Close) != Offer::Failed {
            assert!(Instant::now() < deadline, "the broken link kept accepting events");
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn welcome() -> PortalEvent {
        PortalEvent::Close
    }

    #[test]
    fn accept_portal_registers_the_first_portal() {
        assert_eq!(
            accept_portal(None, false, PORTAL_PROTOCOL_VERSION),
            PortalAdmission::Register
        );
    }

    #[test]
    fn accept_portal_tells_a_second_portal_who_is_open() {
        assert_eq!(
            accept_portal(Some(4242), false, PORTAL_PROTOCOL_VERSION),
            PortalAdmission::AlreadyOpen { existing_pid: 4242 }
        );
    }

    #[test]
    fn accept_portal_closes_a_portal_that_connects_while_the_service_stops() {
        assert_eq!(
            accept_portal(None, true, PORTAL_PROTOCOL_VERSION),
            PortalAdmission::Close
        );
        assert_eq!(
            accept_portal(Some(1), true, PORTAL_PROTOCOL_VERSION),
            PortalAdmission::Close
        );
    }

    #[test]
    fn accept_portal_refuses_another_protocol_version() {
        let admission = accept_portal(None, false, PORTAL_PROTOCOL_VERSION + 1);

        assert!(matches!(admission, PortalAdmission::Refuse(_)), "{admission:?}");
    }

    #[test]
    fn the_registry_registers_one_portal_and_turns_the_second_away_without_focusing_the_first() {
        let mut registry = PortalRegistry::default();
        let (first, first_written) = recording_link(1, 100);
        let (second, second_written) = recording_link(2, 200);

        let one = registry.admit(first, PORTAL_PROTOCOL_VERSION, false, || PortalEvent::Close);
        let two = registry.admit(second, PORTAL_PROTOCOL_VERSION, false, welcome);

        assert_eq!(one, PortalAdmission::Register);
        assert_eq!(two, PortalAdmission::AlreadyOpen { existing_pid: 100 });
        assert!(registry.is_registered(1));
        assert!(!registry.is_registered(2));
        assert_eq!(
            wait_for(&second_written, 1),
            [PortalEvent::AlreadyOpen { existing_pid: 100 }]
        );
        // The first Portal is only told to focus after the newcomer passed on the foreground right.
        assert_eq!(wait_for(&first_written, 1), [PortalEvent::Close]);
    }

    #[test]
    fn a_registered_portal_that_goes_frees_the_place_for_the_next() {
        let mut registry = PortalRegistry::default();
        let (first, _) = recording_link(1, 100);
        registry.admit(first, PORTAL_PROTOCOL_VERSION, false, welcome);

        assert!(!registry.gone(2), "another connection's end changes nothing");
        assert!(registry.gone(1));

        let (next, _) = recording_link(3, 100);
        assert_eq!(
            registry.admit(next, PORTAL_PROTOCOL_VERSION, false, welcome),
            PortalAdmission::Register
        );
    }

    #[test]
    fn a_portal_that_connects_while_the_service_stops_gets_close_not_welcome() {
        let mut registry = PortalRegistry::default();
        let (link, written) = recording_link(1, 100);

        let admission = registry.admit(link, PORTAL_PROTOCOL_VERSION, true, || {
            panic!("Welcome must not be built while the service stops")
        });

        assert_eq!(admission, PortalAdmission::Close);
        assert!(!registry.is_registered(1));
        assert_eq!(wait_for(&written, 1), [PortalEvent::Close]);
    }

    #[test]
    fn a_portal_with_another_protocol_version_is_told_why() {
        let mut registry = PortalRegistry::default();
        let (link, written) = recording_link(1, 100);

        registry.admit(link, PORTAL_PROTOCOL_VERSION + 1, false, welcome);

        let events = wait_for(&written, 1);
        assert!(matches!(events.as_slice(), [PortalEvent::Refused { .. }]), "{events:?}");
        assert!(!registry.is_registered(1));
    }

    #[test]
    fn the_runtime_state_names_the_selected_theme() {
        let mut settings = Settings::default();
        settings.library.selected = "abc".to_owned();

        let runtime = runtime_state(&settings, true, RuleState::None, false, false);

        assert_eq!(runtime.selected.as_deref(), Some("abc"));
        assert!(runtime.running && !runtime.tray_visible);
    }

    #[test]
    fn the_runtime_state_says_when_the_library_moves() {
        let settings = Settings::default();

        assert!(runtime_state(&settings, false, RuleState::None, true, false).library_moving);
        assert!(!runtime_state(&settings, false, RuleState::None, false, false).library_moving);
    }

    #[test]
    fn no_selected_theme_is_none() {
        let settings = Settings::default();

        assert_eq!(
            runtime_state(&settings, false, RuleState::None, false, false).selected,
            None
        );
    }

    #[test]
    fn events_are_written_in_the_order_they_were_offered() {
        let (link, written) = recording_link(1, 100);
        let sent = [
            PortalEvent::Focus { page: None },
            PortalEvent::Memory(MemoryReport {
                service: 1,
                renderers: Vec::new(),
                portal: 0,
            }),
            PortalEvent::Close,
        ];

        for event in sent.clone() {
            assert_eq!(link.offer(event), Offer::Queued);
        }

        assert_eq!(wait_for(&written, sent.len()), sent);
    }
}
