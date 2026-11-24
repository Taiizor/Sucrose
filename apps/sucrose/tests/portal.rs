//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use common::{PortalClient, TestService, service_runs};
use serde_json::json;
use sucrose_core::settings::ThemeMode;
use sucrose_ipc::portal::PortalEvent;

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
