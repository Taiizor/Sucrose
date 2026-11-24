//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use std::path::PathBuf;

use common::{PortalClient, TestService, copy_fixture_library, service_runs};
use serde_json::json;
use sucrose_core::{
    library::scan, paths::legacy_data_dir, settings::ThemeMode, theme::check_selected, version::app_version,
};
use sucrose_ipc::{
    connect,
    portal::{PortalEvent, PortalPage, PortalRequest, SettingChange, UseOutcome, UseRefusal},
    protocol::ToService,
    service_endpoint_for,
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
fn memory_reports_arrive_while_connected() {
    if !service_runs("memory_reports_arrive_while_connected") {
        return;
    }
    let service = TestService::start("memory", &json!({}));
    let (portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");
    let started = std::time::Instant::now();

    let mut reports = Vec::new();
    while reports.len() < 2 {
        match portal.next_event() {
            Some(PortalEvent::Memory(report)) => reports.push(report),
            Some(_) => {}
            None => panic!("only {} Memory events arrived", reports.len()),
        }
    }

    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "two reports took {:?}",
        started.elapsed()
    );
    for report in &reports {
        assert!(report.service > 0, "the service figure is above 0: {report:?}");
        assert!(
            report.renderers.is_empty(),
            "no wallpaper plays, so no renderer: {report:?}"
        );
        assert!(
            report.portal > 0,
            "the Portal (this test process) is counted: {report:?}"
        );
    }
}

/// Reads events until a Runtime with this tray state arrives, skipping the others (Memory and the like).
fn wait_for_tray(portal: &PortalClient, visible: bool) {
    // Memory events arrive every second, so a bare read loop would never time out.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        match portal.next_event() {
            Some(PortalEvent::Runtime(runtime)) if runtime.tray_visible == visible => return,
            Some(_) => {}
            None => break,
        }
    }
    panic!("no Runtime with tray_visible {visible}");
}

#[test]
fn the_tray_follows_the_notification_area_switch_live() {
    if !service_runs("the_tray_follows_the_notification_area_switch_live") {
        return;
    }
    let service = TestService::start("tray-live", &json!({}));
    let (mut portal, first) = PortalClient::hello(&service, None);
    let PortalEvent::Welcome { runtime, .. } = first else {
        panic!("the first event is not a Welcome: {first:?}");
    };
    assert!(!runtime.tray_visible, "AppVisible is false: no tray at start");

    portal.request(&PortalRequest::Change(SettingChange::AppVisible(true)));
    wait_for_tray(&portal, true);

    // The tray is removed again, so no icon is left behind when the service is killed at the end of the test.
    portal.request(&PortalRequest::Change(SettingChange::AppVisible(false)));
    wait_for_tray(&portal, false);
    assert_eq!(service.settings_file()["General"]["AppVisible"], false);
}

#[test]
fn quick_toggles_leave_the_state_of_the_last_value() {
    if !service_runs("quick_toggles_leave_the_state_of_the_last_value") {
        return;
    }
    let service = TestService::start("tray-quick", &json!({}));
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    for visible in [true, false, true, false] {
        portal.request(&PortalRequest::Change(SettingChange::AppVisible(visible)));
    }
    let mut changed = 0;
    let answered_by = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while changed < 4 {
        assert!(
            std::time::Instant::now() < answered_by,
            "only {changed} of 4 changes were answered"
        );
        match portal.next_event() {
            Some(PortalEvent::Changed(SettingChange::AppVisible(_))) => changed += 1,
            Some(_) => {}
            None => panic!("only {changed} of 4 changes were answered"),
        }
    }
    assert_eq!(service.settings_file()["General"]["AppVisible"], false);
    drop(portal);

    // A Welcome describes the state the service is in now, so it shows where the toggles ended.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let (_next, event) = PortalClient::hello(&service, None);
        if let PortalEvent::Welcome { settings, runtime } = event {
            assert!(!settings.app_visible);
            assert!(!runtime.tray_visible, "the tray ended as the last value says");
            break;
        }
        assert!(std::time::Instant::now() < deadline, "no Welcome: {event:?}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
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
    // The default is not written out, so the key is absent or empty; either way the refused path is not there.
    let stored = service.settings_file();
    assert_eq!(stored["Portal"]["BackgroundImage"].as_str().unwrap_or(""), "");
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

/// Sends a Use and returns the service's answer to it, skipping the other events (Runtime and the like).
fn use_theme(portal: &mut PortalClient, folder: &str) -> UseOutcome {
    portal.request(&PortalRequest::Use {
        folder: folder.to_owned(),
    });
    loop {
        match portal.next_event() {
            Some(PortalEvent::UseResult {
                folder: answered,
                outcome,
            }) => {
                assert_eq!(answered, folder, "the answer names the folder it is for");
                return outcome;
            }
            Some(_) => {}
            None => panic!("no UseResult for {folder:?}"),
        }
    }
}

#[test]
fn use_of_a_web_theme_is_refused_and_the_selection_is_unchanged() {
    if !service_runs("use_of_a_web_theme_is_refused_and_the_selection_is_unchanged") {
        return;
    }
    let service = TestService::start("use-web", &json!({}));
    copy_fixture_library(&service);
    let (mut portal, _) = PortalClient::hello(&service, None);

    let outcome = use_theme(&mut portal, "web");

    assert_eq!(outcome, UseOutcome::Refused(UseRefusal::NotVideo));
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

#[test]
fn use_of_a_theme_for_a_newer_sucrose_is_refused_and_the_selection_is_unchanged() {
    if !service_runs("use_of_a_theme_for_a_newer_sucrose_is_refused_and_the_selection_is_unchanged") {
        return;
    }
    let service = TestService::start("use-newer", &json!({}));
    copy_fixture_library(&service);
    let (mut portal, _) = PortalClient::hello(&service, None);

    let outcome = use_theme(&mut portal, "newer-version");

    assert_eq!(outcome, UseOutcome::Refused(UseRefusal::NewerVersion));
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

#[test]
fn use_of_a_name_that_leaves_the_library_or_does_not_exist_is_refused_as_not_found() {
    if !service_runs("use_of_a_name_that_leaves_the_library_or_does_not_exist_is_refused_as_not_found") {
        return;
    }
    let service = TestService::start("use-escape", &json!({}));
    copy_fixture_library(&service);
    let (mut portal, _) = PortalClient::hello(&service, None);

    for folder in ["../escape", "web/../video", "no-such-theme", ""] {
        let outcome = use_theme(&mut portal, folder);

        assert_eq!(outcome, UseOutcome::Refused(UseRefusal::NotFound), "folder {folder:?}");
    }
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

#[test]
fn use_of_a_damaged_theme_is_refused_as_corrupt() {
    if !service_runs("use_of_a_damaged_theme_is_refused_as_corrupt") {
        return;
    }
    let service = TestService::start("use-corrupt", &json!({}));
    copy_fixture_library(&service);
    let (mut portal, _) = PortalClient::hello(&service, None);

    let outcome = use_theme(&mut portal, "broken-json");

    assert_eq!(outcome, UseOutcome::Refused(UseRefusal::Corrupt));
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

/// Plan 11 runs this against the real service on the maintainer's screen; it needs two Video themes the gate accepts
/// in the library of `SUCROSE_DATA_DIR`. `SUCROSE_SWITCH_COUNT` (default 10) Uses alternate between them,
/// `SUCROSE_SWITCH_SECONDS` (default 10) apart, after `SUCROSE_SWITCH_DELAY` seconds (default 30) to start recording.
/// These are knobs of this test, not flags of the app.
#[test]
#[ignore = "drives the real service of SUCROSE_DATA_DIR and changes its wallpaper; plan 11 runs it"]
#[allow(clippy::print_stdout)]
fn use_switches_between_video_themes() {
    let count = env_number("SUCROSE_SWITCH_COUNT", 10);
    let seconds = env_number("SUCROSE_SWITCH_SECONDS", 10);
    let delay = env_number("SUCROSE_SWITCH_DELAY", 30);
    let data_dir = std::env::var_os("SUCROSE_DATA_DIR").expect("SUCROSE_DATA_DIR names the data folder of the service");
    let endpoint = service_endpoint_for(Some(data_dir.as_os_str())).expect("the endpoint of that folder is known");
    let connection = connect(&endpoint.name).expect("a service runs on that data folder");
    let (mut portal, first) = PortalClient::hello_on(connection, std::process::id());
    let PortalEvent::Welcome { settings, runtime } = first else {
        panic!("the first event is not a Welcome: {first:?}");
    };

    let location = if settings.library_location.is_empty() {
        legacy_data_dir().expect("the OS reports a data folder").join("Library")
    } else {
        PathBuf::from(&settings.library_location)
    };
    let version = app_version();
    let scan = scan(&location, &version).expect("the library can be scanned");
    let mut playable: Vec<String> = scan
        .themes
        .iter()
        .filter(|theme| check_selected(&location, &theme.folder, &version).is_ok())
        .map(|theme| theme.folder.clone())
        .collect();
    // The current selection first, when it is one of them.
    if let Some(selected) = &runtime.selected
        && let Some(at) = playable.iter().position(|folder| folder == selected)
    {
        playable.swap(0, at);
    }
    assert!(
        playable.len() >= 2,
        "the library needs two Video themes the gate accepts, it has {}",
        playable.len()
    );
    println!(
        "{} start recording: {} themes, {count} switches {seconds} s apart",
        utc_now(),
        playable.len()
    );
    std::thread::sleep(std::time::Duration::from_secs(delay));
    for index in 0..count {
        // The first Use goes to the second theme: the first one is what plays now.
        let folder = &playable[usize::from(index % 2 == 0)];
        let outcome = use_theme(&mut portal, folder);
        println!("{} use {index}: {outcome:?}", utc_now());
        assert_eq!(outcome, UseOutcome::Applied, "switch {index}");
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }
    println!("{} done", utc_now());
}

/// A whole number from the environment, or `default`.
fn env_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(default)
}

/// The current UTC time as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
fn utc_now() -> String {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let (days, rest) = (since_epoch.as_secs() / 86_400, since_epoch.as_secs() % 86_400);
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let shifted = i64::try_from(days).unwrap_or_default() + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = if month_part < 10 {
        month_part + 3
    } else {
        month_part - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60,
        since_epoch.subsec_millis()
    )
}
