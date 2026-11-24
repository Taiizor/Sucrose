//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use common::{PortalClient, TestService, service_runs};
use serde_json::json;
use sucrose_core::settings::ThemeMode;
use sucrose_ipc::{
    portal::{PortalEvent, PortalPage, PortalRequest, SettingChange},
    protocol::ToService,
};

#[test]
fn welcome_carries_the_settings_view() {
    if !service_runs("welcome_carries_the_settings_view") {
        return;
    }
    let service = TestService::start(
        "welcome",
        &json!({ "Portal": { "LibraryPagination": 17, "ThemeMode": "Dark" } }),
    );

    let (_portal, first) = PortalClient::hello(&service, None);

    let PortalEvent::Welcome { settings, runtime } = first else {
        panic!("the first event is not a Welcome: {first:?}");
    };
    assert_eq!(settings.library_pagination, 17);
    assert_eq!(settings.theme_mode, ThemeMode::Dark);
    assert!(!settings.app_visible);
    assert_eq!(runtime.selected, None);
    assert!(!runtime.running);
    assert!(!runtime.tray_visible);
}

#[test]
fn a_second_portal_is_told_the_first_is_open_and_the_first_is_focused() {
    if !service_runs("a_second_portal_is_told_the_first_is_open_and_the_first_is_focused") {
        return;
    }
    let service = TestService::start("second", &json!({}));
    let (first, first_event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(first_event, PortalEvent::Welcome { .. }), "{first_event:?}");

    let (_second, second_event) = PortalClient::hello_as(&service, 1002, Some(PortalPage::General));

    assert_eq!(second_event, PortalEvent::AlreadyOpen { existing_pid: 1001 });
    assert_eq!(
        first.next_event(),
        Some(PortalEvent::Focus {
            page: Some(PortalPage::General)
        })
    );
}

#[test]
fn a_portal_that_left_makes_room_for_the_next() {
    if !service_runs("a_portal_that_left_makes_room_for_the_next") {
        return;
    }
    let service = TestService::start("left", &json!({}));
    let (first, first_event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(first_event, PortalEvent::Welcome { .. }), "{first_event:?}");
    drop(first);

    // The service notices the closed connection on its reader thread; a Portal that comes a moment too early is
    // told AlreadyOpen, so the next one tries again.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let (_next, event) = PortalClient::hello_as(&service, 1002, None);
        if matches!(event, PortalEvent::Welcome { .. }) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no Welcome after the first Portal left: {event:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn a_connection_whose_first_frame_is_garbage_is_dropped_and_a_portal_still_gets_welcome() {
    if !service_runs("a_connection_whose_first_frame_is_garbage_is_dropped_and_a_portal_still_gets_welcome") {
        return;
    }
    let service = TestService::start("garbage", &json!({}));

    // A frame whose payload is no message, one that is a message but not a hello, and a message of another peer.
    let garbage = PortalClient::connect_with_garbage(&service);
    let not_a_hello = PortalClient::connect_with(&service, &ToService::Playing);

    assert_eq!(garbage.next_event(), None, "the garbage connection is closed");
    assert_eq!(
        not_a_hello.next_event(),
        None,
        "a first message that is no hello is closed"
    );
    let (_portal, event) = PortalClient::hello(&service, None);
    assert!(matches!(event, PortalEvent::Welcome { .. }), "{event:?}");
}

#[test]
fn a_change_survives_a_restart() {
    if !service_runs("a_change_survives_a_restart") {
        return;
    }
    let service = TestService::start("restart", &json!({}));
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    portal.request(&PortalRequest::Change(SettingChange::LibraryPagination(42)));

    assert_eq!(
        portal.next_event(),
        Some(PortalEvent::Changed(SettingChange::LibraryPagination(42)))
    );
    assert_eq!(service.settings_file()["Portal"]["LibraryPagination"], 42);
    drop(portal);

    let service = service.restart();
    let (_portal, event) = PortalClient::hello(&service, None);

    let PortalEvent::Welcome { settings, .. } = event else {
        panic!("the first event is not a Welcome: {event:?}");
    };
    assert_eq!(settings.library_pagination, 42);
}

#[test]
fn changes_are_applied_in_arrival_order_and_the_last_one_wins() {
    if !service_runs("changes_are_applied_in_arrival_order_and_the_last_one_wins") {
        return;
    }
    let service = TestService::start("order", &json!({}));
    let (mut portal, _) = PortalClient::hello(&service, None);

    portal.request(&PortalRequest::Change(SettingChange::LibraryPagination(10)));
    portal.request(&PortalRequest::Change(SettingChange::LibraryPagination(20)));

    assert_eq!(
        portal.next_event(),
        Some(PortalEvent::Changed(SettingChange::LibraryPagination(10)))
    );
    assert_eq!(
        portal.next_event(),
        Some(PortalEvent::Changed(SettingChange::LibraryPagination(20)))
    );
    assert_eq!(service.settings_file()["Portal"]["LibraryPagination"], 20);
}

#[test]
fn an_out_of_range_change_comes_back_clamped() {
    if !service_runs("an_out_of_range_change_comes_back_clamped") {
        return;
    }
    let service = TestService::start("clamp", &json!({}));
    let (mut portal, _) = PortalClient::hello(&service, None);

    portal.request(&PortalRequest::Change(SettingChange::BackgroundOpacity(150)));

    assert_eq!(
        portal.next_event(),
        Some(PortalEvent::Changed(SettingChange::BackgroundOpacity(100)))
    );
    assert_eq!(service.settings_file()["Portal"]["BackgroundOpacity"], 100);
}

#[test]
fn a_refused_image_is_rejected_and_nothing_is_stored() {
    if !service_runs("a_refused_image_is_rejected_and_nothing_is_stored") {
        return;
    }
    let service = TestService::start("image", &json!({}));
    let (mut portal, _) = PortalClient::hello(&service, None);
    let change = SettingChange::BackgroundImage("relative.png".to_owned());

    portal.request(&PortalRequest::Change(change.clone()));

    let Some(PortalEvent::Rejected { change: refused, .. }) = portal.next_event() else {
        panic!("the change is not answered Rejected");
    };
    assert_eq!(refused, change);
    assert_eq!(service.settings_file()["Portal"]["BackgroundImage"], "");
}

/// Makes a file writable again when dropped, so a failed assertion cannot leave a read-only file in the scratch
/// folder that the next run could not remove.
struct ReadOnly(std::path::PathBuf);

impl ReadOnly {
    fn new(path: std::path::PathBuf) -> Self {
        let mut permissions = std::fs::metadata(&path).expect("the file exists").permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&path, permissions).expect("the file can be made read-only");
        Self(path)
    }
}

impl Drop for ReadOnly {
    #[allow(clippy::permissions_set_readonly_false)] // Windows: this only clears the read-only attribute.
    fn drop(&mut self) {
        if let Ok(metadata) = std::fs::metadata(&self.0) {
            let mut permissions = metadata.permissions();
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&self.0, permissions);
        }
    }
}

#[test]
fn a_failed_write_is_rejected_with_the_not_saved_text_and_nothing_is_broadcast() {
    if !service_runs("a_failed_write_is_rejected_with_the_not_saved_text_and_nothing_is_broadcast") {
        return;
    }
    let service = TestService::start("readonly", &json!({ "Portal": { "LibraryPagination": 12 } }));
    let (mut portal, _) = PortalClient::hello(&service, None);
    let _readonly = ReadOnly::new(service.dir.join("Setting").join("Sucrose.json"));

    portal.request(&PortalRequest::Change(SettingChange::LibraryPagination(30)));

    // The first event after the Welcome is the rejection: no Changed came before it.
    assert_eq!(
        portal.next_event(),
        Some(PortalEvent::Rejected {
            change: SettingChange::LibraryPagination(30),
            reason: "Your setting couldn't be saved. Try again.".to_owned(),
        })
    );
    drop(portal);

    // The next Portal still sees the old value; it may have to wait until the service noticed the first one left.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let (_next, event) = PortalClient::hello(&service, None);
        if let PortalEvent::Welcome { settings, .. } = event {
            assert_eq!(settings.library_pagination, 12);
            break;
        }
        assert!(std::time::Instant::now() < deadline, "no Welcome: {event:?}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
