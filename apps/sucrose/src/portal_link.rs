//! The Portal's side of the service (D-009, D-033): the one registered Portal, its ordered writer, the admission
//! decision for a connecting Portal and the view of the settings it gets.
//!
//! The registry lives on the service's main thread, so two Portals that connect at the same moment are decided in one
//! place: exactly one is registered, the other is told who is open.

use std::io;
use std::thread;

use crossbeam_channel::Sender;
use sucrose_config::model::Settings;
use sucrose_ipc::{
    Connection,
    portal::{
        MAX_PATH_TEXT, PORTAL_PROTOCOL_VERSION, PortalEvent, PortalPage, PortalSettings, RuleState, RuntimeState,
    },
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

    /// Queues `event` without ever waiting. A full queue is a failure: the Portal is stuck, and losing an event
    /// would leave it showing a wrong state.
    pub fn offer(&self, event: PortalEvent) -> Offer {
        match self.queue.try_send(event) {
            Ok(()) => Offer::Queued,
            Err(_) => Offer::Failed,
        }
    }
}

/// What the service does with a Portal that says hello.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalAdmission {
    /// It is the one Portal: register it and send Welcome.
    Register,
    /// Another Portal is registered: tell the newcomer, and the registered one to focus.
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
        page: Option<PortalPage>,
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
                self.send(PortalEvent::Focus { page });
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

/// The text of a request kind for the log: never its content, which can hold a theme name or a path (D-08).
pub fn request_kind(request: &sucrose_ipc::portal::PortalRequest) -> &'static str {
    use sucrose_ipc::portal::PortalRequest;
    match request {
        PortalRequest::Use { .. } => "use",
        PortalRequest::Change(_) => "change",
        PortalRequest::SetStartup(_) => "set startup",
        PortalRequest::RefreshStartup => "refresh startup",
        PortalRequest::MaintainLibrary => "maintain library",
        PortalRequest::MoveLibrary { .. } => "move library",
    }
}

/// `text` cut to [`MAX_PATH_TEXT`] bytes at a character boundary.
fn clip(text: &str) -> String {
    let mut end = text.len().min(MAX_PATH_TEXT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.get(..end).unwrap_or_default().to_owned()
}

/// The settings the Portal sees: a copy of the values it shows, never the file (D-11).
pub fn portal_view(settings: &Settings) -> PortalSettings {
    PortalSettings {
        app_visible: settings.general.app_visible,
        app_exit: settings.general.app_exit,
        run_startup: settings.general.run_startup,
        theme_mode: settings.portal.theme_mode,
        backdrop: settings.portal.backdrop,
        background_image: clip(&settings.portal.background_image),
        background_stretch: settings.portal.background_stretch,
        background_opacity: settings.portal.background_opacity,
        library_pagination: settings.portal.library_pagination,
        adaptive_layout: settings.portal.adaptive_layout,
        adaptive_margin: settings.portal.adaptive_margin,
        library_preview: settings.portal.library_preview,
        library_preview_hide: settings.portal.library_preview_hide,
        library_sort_mode: settings.portal.library_sort_mode,
        library_sort_kind: settings.portal.library_sort_kind,
        library_location: clip(&settings.library.location),
        library_move: settings.library.move_library,
        delete_corrupt: settings.library.delete_corrupt,
        hint_tray_icon: settings.warehouse.hint_tray_icon,
    }
}

/// What the service tells the Portal it is doing: the selected theme from the settings, whether a renderer runs and
/// whether the tray exists. No rule and no library move exist yet.
pub fn runtime_state(settings: &Settings, running: bool, tray_visible: bool) -> RuntimeState {
    RuntimeState {
        selected: Some(settings.library.selected.as_str())
            .filter(|selected| !selected.is_empty())
            .map(clip),
        running,
        rule: RuleState::None,
        library_moving: false,
        tray_visible,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex, PoisonError},
        time::{Duration, Instant},
    };

    use sucrose_core::settings::ThemeMode;
    use sucrose_ipc::portal::MemoryReport;

    use super::*;

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
    fn the_registry_registers_one_portal_and_focuses_it_for_the_second() {
        let mut registry = PortalRegistry::default();
        let (first, first_written) = recording_link(1, 100);
        let (second, second_written) = recording_link(2, 200);

        let one = registry.admit(first, PORTAL_PROTOCOL_VERSION, None, false, || PortalEvent::Close);
        let two = registry.admit(
            second,
            PORTAL_PROTOCOL_VERSION,
            Some(PortalPage::General),
            false,
            welcome,
        );

        assert_eq!(one, PortalAdmission::Register);
        assert_eq!(two, PortalAdmission::AlreadyOpen { existing_pid: 100 });
        assert!(registry.is_registered(1));
        assert!(!registry.is_registered(2));
        assert_eq!(
            wait_for(&second_written, 1),
            [PortalEvent::AlreadyOpen { existing_pid: 100 }]
        );
        assert_eq!(
            wait_for(&first_written, 2),
            [
                PortalEvent::Close,
                PortalEvent::Focus {
                    page: Some(PortalPage::General)
                }
            ]
        );
    }

    #[test]
    fn a_registered_portal_that_goes_frees_the_place_for_the_next() {
        let mut registry = PortalRegistry::default();
        let (first, _) = recording_link(1, 100);
        registry.admit(first, PORTAL_PROTOCOL_VERSION, None, false, welcome);

        assert!(!registry.gone(2), "another connection's end changes nothing");
        assert!(registry.gone(1));

        let (next, _) = recording_link(3, 100);
        assert_eq!(
            registry.admit(next, PORTAL_PROTOCOL_VERSION, None, false, welcome),
            PortalAdmission::Register
        );
    }

    #[test]
    fn a_portal_that_connects_while_the_service_stops_gets_close_not_welcome() {
        let mut registry = PortalRegistry::default();
        let (link, written) = recording_link(1, 100);

        let admission = registry.admit(link, PORTAL_PROTOCOL_VERSION, None, true, || {
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

        registry.admit(link, PORTAL_PROTOCOL_VERSION + 1, None, false, welcome);

        let events = wait_for(&written, 1);
        assert!(matches!(events.as_slice(), [PortalEvent::Refused { .. }]), "{events:?}");
        assert!(!registry.is_registered(1));
    }

    #[test]
    fn the_view_carries_the_values_the_portal_shows() {
        let mut settings = Settings::default();
        settings.portal.library_pagination = 17;
        settings.portal.theme_mode = ThemeMode::Dark;
        settings.general.app_visible = false;
        settings.library.location = "D:\\Themes".to_owned();
        settings.library.selected = "abc".to_owned();

        let view = portal_view(&settings);
        let runtime = runtime_state(&settings, true, false);

        assert_eq!(view.library_pagination, 17);
        assert_eq!(view.theme_mode, ThemeMode::Dark);
        assert!(!view.app_visible);
        assert_eq!(view.library_location, "D:\\Themes");
        assert_eq!(runtime.selected.as_deref(), Some("abc"));
        assert!(runtime.running && !runtime.tray_visible);
    }

    #[test]
    fn no_selected_theme_is_none_and_long_paths_are_cut_at_a_character() {
        let mut settings = Settings::default();
        settings.portal.background_image = "\u{20ac}".repeat(MAX_PATH_TEXT);

        let view = portal_view(&settings);

        assert_eq!(runtime_state(&settings, false, false).selected, None);
        assert!(view.background_image.len() <= MAX_PATH_TEXT);
        assert!(view.background_image.chars().all(|c| c == '\u{20ac}'));
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
