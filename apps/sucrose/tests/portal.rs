//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use common::{PortalClient, TestService, service_runs};
use serde_json::json;
use sucrose_core::settings::ThemeMode;
use sucrose_ipc::{
    portal::{PortalEvent, PortalPage},
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
