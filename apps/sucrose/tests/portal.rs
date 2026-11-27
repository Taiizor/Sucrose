//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use std::path::PathBuf;

use common::{PortalClient, TestService, copy_fixture_library, copy_web_fixture, service_runs, skipped};
use serde_json::{Value, json};
use sucrose_core::{
    library::scan,
    paths::legacy_data_dir,
    settings::{EngineType, ThemeMode},
    theme::{WallpaperType, check_selected},
    version::app_version,
};
use sucrose_ipc::{
    connect,
    portal::{
        EngineReportView, EngineState, FallbackView, Launch, LaunchReply, MaintainReport, MoveOutcome, MoveRefusal,
        PortalEvent, PortalPage, PortalRequest, SettingChange, StartupInfo, StartupMode, UseOutcome, UseRefusal,
    },
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

/// Reads events until the engine report arrives, skipping the others (Memory and the like).
fn wait_for_engines(portal: &PortalClient) -> EngineReportView {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        match portal.next_event_any() {
            Some(PortalEvent::Engines(report)) => return report,
            Some(_) => {}
            None => break,
        }
    }
    panic!("no engine report arrived");
}

#[test]
fn the_engine_report_follows_the_welcome_and_comes_again_on_request() {
    if !service_runs("the_engine_report_follows_the_welcome_and_comes_again_on_request") {
        return;
    }
    let service = TestService::start("engines", &json!({}));
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    let report = wait_for_engines(&portal);

    assert_eq!(report.engines.len(), 7, "{report:?}");
    assert!(
        report
            .engines
            .iter()
            .any(|engine| engine.engine == EngineType::VlcPlayerLive
                && engine.state == EngineState::Unsupported { architecture: false }),
        "libVLC is not in this build: {report:?}"
    );
    assert!(report.fallbacks.is_empty(), "nothing plays, so no fallback: {report:?}");

    portal.request(&PortalRequest::DetectEngines);

    assert_eq!(wait_for_engines(&portal), report);
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
    let mut service = TestService::start("tray-quick", &json!({}));
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

    // With the tray hidden and nothing to play, the service ends once the Portal is gone (D-13); that only happens when
    // the tray ended as the last value says.
    assert!(
        service.exits_within(std::time::Duration::from_secs(3)),
        "the tray ended visible although the last value hides it"
    );
}

#[test]
fn a_second_portal_is_told_the_first_is_open_and_the_first_is_focused() {
    if !service_runs("a_second_portal_is_told_the_first_is_open_and_the_first_is_focused") {
        return;
    }
    let service = TestService::start("second", &json!({}));
    let (first, first_event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(first_event, PortalEvent::Welcome { .. }), "{first_event:?}");

    let (mut second, second_event) = PortalClient::hello_as(&service, 1002, Some(PortalPage::General));

    assert_eq!(second_event, PortalEvent::AlreadyOpen { existing_pid: 1001 });
    // The first one is focused once the second has passed on the foreground right, not before.
    second
        .connection()
        .send(&ToService::Launch(Launch::ForegroundGranted {
            page: Some(PortalPage::General),
        }))
        .expect("the grant is sent");
    assert_eq!(
        first.next_event(),
        Some(PortalEvent::Focus {
            page: Some(PortalPage::General)
        })
    );
}

#[test]
fn a_launch_with_a_portal_registered_gets_portal_running() {
    if !service_runs("a_launch_with_a_portal_registered_gets_portal_running") {
        return;
    }
    let service = TestService::start("launch", &json!({}));
    let (_portal, first_event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(first_event, PortalEvent::Welcome { .. }), "{first_event:?}");

    let mut launcher = service.connect().expect("the service accepts a launcher");
    launcher
        .send(&ToService::Launch(Launch::OpenPortal { page: None }))
        .expect("the request is sent");
    let reply: LaunchReply = launcher.recv().expect("the service answers a launcher");

    assert_eq!(reply, LaunchReply::PortalRunning { pid: 1001 });
}

#[test]
fn a_service_with_the_tray_hidden_and_nothing_to_play_exits_when_its_only_portal_leaves() {
    if !service_runs("a_service_with_the_tray_hidden_and_nothing_to_play_exits_when_its_only_portal_leaves") {
        return;
    }
    let mut service = TestService::start("exit-hidden", &json!({}));
    let (portal, event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(event, PortalEvent::Welcome { .. }), "{event:?}");
    assert!(
        !service.exits_within(std::time::Duration::from_millis(500)),
        "the service stays while the Portal is connected"
    );
    drop(portal);

    assert!(
        service.exits_within(std::time::Duration::from_secs(3)),
        "nothing needs the service any more, it exits"
    );
}

#[test]
fn a_service_with_the_tray_visible_stays_when_its_only_portal_leaves() {
    if !service_runs("a_service_with_the_tray_visible_stays_when_its_only_portal_leaves") {
        return;
    }
    let mut service = TestService::start("stay-visible", &json!({ "General": { "AppVisible": true } }));
    let (portal, event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(event, PortalEvent::Welcome { .. }), "{event:?}");
    drop(portal);

    assert!(
        !service.exits_within(std::time::Duration::from_secs(2)),
        "the tray keeps the service until Exit"
    );
}

#[test]
fn a_launcher_that_passes_on_the_foreground_gets_the_open_portal_focused() {
    if !service_runs("a_launcher_that_passes_on_the_foreground_gets_the_open_portal_focused") {
        return;
    }
    let service = TestService::start("grant", &json!({}));
    let (portal, first_event) = PortalClient::hello_as(&service, 1001, None);
    assert!(matches!(first_event, PortalEvent::Welcome { .. }), "{first_event:?}");

    let mut launcher = service.connect().expect("the service accepts a launcher");
    launcher
        .send(&ToService::Launch(Launch::OpenPortal { page: None }))
        .expect("the request is sent");
    let reply: LaunchReply = launcher.recv().expect("the service answers a launcher");
    assert_eq!(reply, LaunchReply::PortalRunning { pid: 1001 });
    launcher
        .send(&ToService::Launch(Launch::ForegroundGranted { page: None }))
        .expect("the grant is sent");

    assert_eq!(portal.next_event(), Some(PortalEvent::Focus { page: None }));
}

#[test]
fn a_portal_that_left_makes_room_for_the_next() {
    if !service_runs("a_portal_that_left_makes_room_for_the_next") {
        return;
    }
    // The tray keeps the service running after the first Portal leaves (D-13).
    let service = TestService::start("left", &json!({ "General": { "AppVisible": true } }));
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
    // The tray keeps the service running after the first Portal leaves (D-13).
    let service = TestService::start(
        "readonly",
        &json!({ "Portal": { "LibraryPagination": 12 }, "General": { "AppVisible": true } }),
    );
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
fn use_of_a_url_theme_is_refused_and_the_selection_is_unchanged() {
    if !service_runs("use_of_a_url_theme_is_refused_and_the_selection_is_unchanged") {
        return;
    }
    let service = TestService::start("use-url", &json!({}));
    copy_fixture_library(&service);
    let (mut portal, _) = PortalClient::hello(&service, None);

    // Gif plays since 04-30; a Url theme is the type no engine plays yet (04-35 adds it).
    let outcome = use_theme(&mut portal, "url-source");

    assert_eq!(outcome, UseOutcome::Refused(UseRefusal::TypeNotPlayable));
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

/// The text of the renderer logs of `service` (`<data>/Log/Renderer-*.log`), empty when there are none yet.
fn renderer_logs(service: &TestService) -> String {
    let Ok(entries) = std::fs::read_dir(service.dir.join("Log")) else {
        return String::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("Renderer"))
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .collect()
}

/// The whole chain of a Web theme: the engine report says Web plays, Use of the `hello` fixture is applied, the settings
/// select it and its renderer starts CEF. It plays a real wallpaper, so it skips where CEF is not installed next to
/// the executable (the report then lists Web as not playable), and it closes what it started.
#[test]
fn use_plays_a_web_theme() {
    if !service_runs("use_plays_a_web_theme") {
        return;
    }
    let service = TestService::start("use-web", &json!({}));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    assert!(report.supported.contains(&WallpaperType::Web), "{report:?}");
    if !report.playable.contains(&WallpaperType::Web) {
        skipped(
            "use_plays_a_web_theme",
            &format!("CEF is not installed next to the executable: {report:?}"),
        );
        return;
    }

    let outcome = use_theme(&mut portal, "hello");

    assert_eq!(outcome, UseOutcome::Applied);
    assert_eq!(service.settings_file()["Library"]["Selected"], "hello");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let started = loop {
        let logs = renderer_logs(&service);
        if logs.contains("engine started") {
            break logs;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no renderer started its engine: {logs}"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    };
    assert!(started.contains("Cef"), "the engine that started is CEF: {started}");
}

/// Turning Loop Mode off in the Portal reaches the playing wallpaper (PLAY-01): the switch's change is saved, comes back
/// as Changed, and the renderer logs `loop changed` with `looping=false`. The scratch folder has no settings file, so the
/// service migrates the maintainer's legacy settings read-only and plays the Video theme they select; that opens a real
/// wallpaper on the desktop, so the test only runs when asked for (`--include-ignored`) and the service is stopped
/// when it ends.
#[test]
#[ignore = "plays the maintainer's selected Video theme on the desktop from a scratch data folder; plan 14 runs it"]
fn a_loop_change_reaches_the_running_renderer() {
    if !service_runs("a_loop_change_reaches_the_running_renderer") {
        return;
    }
    let service = TestService::start_migrated("loop-live");
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let logs = renderer_logs(&service);
        if logs.contains("engine started") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no renderer started: the legacy settings of this machine select no playable Video theme? {logs}"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    portal.request(&PortalRequest::Change(SettingChange::WallpaperLoop(false)));

    let mut answered = false;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline && !answered {
        match portal.next_event() {
            Some(PortalEvent::Changed(SettingChange::WallpaperLoop(false))) => answered = true,
            Some(PortalEvent::Rejected { change, reason }) => panic!("{change:?} was rejected: {reason}"),
            Some(_) => {}
            None => break,
        }
    }
    assert!(answered, "the change came back as Changed");
    assert_eq!(service.settings_file()["Engine"]["WallpaperLoop"], false);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let logs = renderer_logs(&service);
        if logs
            .lines()
            .any(|line| line.contains("loop changed") && line.contains("looping=false"))
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the renderer never logged the loop change: {logs}"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[test]
fn an_engine_choice_is_saved_and_broadcast_and_one_outside_the_list_of_the_type_is_rejected() {
    if !service_runs("an_engine_choice_is_saved_and_broadcast_and_one_outside_the_list_of_the_type_is_rejected") {
        return;
    }
    let service = TestService::start("engine-choice", &json!({}));
    let (mut portal, first) = PortalClient::hello(&service, None);
    let PortalEvent::Welcome { settings, .. } = first else {
        panic!("the first event is not a Welcome: {first:?}");
    };
    assert_eq!(settings.web_engine, EngineType::CefSharpLive, "the legacy default");

    portal.request(&PortalRequest::Change(SettingChange::WebEngine(
        EngineType::WebViewLive,
    )));
    let change = SettingChange::VideoEngine(EngineType::AuroraLive);
    portal.request(&PortalRequest::Change(change.clone()));

    let mut changed = false;
    let mut rejected = false;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !(changed && rejected) {
        assert!(
            std::time::Instant::now() < deadline,
            "changed {changed}, rejected {rejected}"
        );
        match portal.next_event() {
            Some(PortalEvent::Changed(SettingChange::WebEngine(EngineType::WebViewLive))) => changed = true,
            Some(PortalEvent::Rejected { change: refused, .. }) if refused == change => rejected = true,
            Some(_) => {}
            None => panic!("changed {changed}, rejected {rejected}"),
        }
    }
    let file = service.settings_file();
    assert_eq!(file["Engine"]["Web"], "WebViewLive");
    assert_ne!(file["Engine"]["Video"], "AuroraLive");
}

/// Whether the engine report lists WebView2 as ready (its runtime is installed and new enough).
fn webview2_is_ready(report: &EngineReportView) -> bool {
    report
        .engines
        .iter()
        .any(|engine| engine.engine == EngineType::WebViewLive && matches!(engine.state, EngineState::Ready { .. }))
}

/// Waits until the renderer logs have at least `count` `engine started` lines.
fn wait_for_engine_starts(service: &TestService, count: usize) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let logs = renderer_logs(service);
        if logs.matches("engine started").count() >= count {
            return logs;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the renderer did not start {count} time(s): {logs}"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

/// A running Web wallpaper restarts on the engine just chosen: with WebView2 installed, choosing it ends the CEF renderer
/// and starts a new one on WebView2, so nothing falls back. (The fallback itself is covered where an engine is missing:
/// `engines::tests::a_missing_configured_engine_falls_back_for_this_run_and_says_why`.) It plays a real wallpaper, so
/// it skips where CEF or WebView2 is not available.
#[test]
fn an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen() {
    if !service_runs("an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen") {
        return;
    }
    let service = TestService::start("engine-restart", &json!({}));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(
            "an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen",
            "CEF is not installed next to the executable or the WebView2 Runtime is missing",
        );
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let first = wait_for_engine_starts(&service, 1);
    assert!(first.contains("engine=\"Cef\""), "the default engine is CEF: {first}");

    portal.request(&PortalRequest::Change(SettingChange::WebEngine(
        EngineType::WebViewLive,
    )));

    let logs = wait_for_engine_starts(&service, 2);
    assert!(
        logs.lines()
            .any(|line| line.contains("engine started") && line.contains("engine=\"WebView2\"")),
        "the second renderer plays on WebView2: {logs}"
    );
}

/// Two theme switches in a row on WebView2 (the Round A CEF close lesson): the renderer of each theme closes its
/// controller before the surface goes, the service starts the next one, and the logs show the close and no WARN. It plays
/// real wallpapers, so it skips where CEF or WebView2 is not available.
#[test]
fn switching_web_themes_on_webview2_closes_each_renderer_cleanly() {
    if !service_runs("switching_web_themes_on_webview2_closes_each_renderer_cleanly") {
        return;
    }
    let service = TestService::start("webview2-switch", &json!({ "Engine": { "Web": "WebViewLive" } }));
    copy_web_fixture(&service, "hello");
    copy_web_fixture(&service, "storage");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(
            "switching_web_themes_on_webview2_closes_each_renderer_cleanly",
            "the WebView2 Runtime is missing",
        );
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    wait_for_engine_starts(&service, 1);
    assert_eq!(use_theme(&mut portal, "storage"), UseOutcome::Applied);
    wait_for_engine_starts(&service, 2);
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);

    let logs = wait_for_engine_starts(&service, 3);

    let closed = logs.matches("webview2 closed").count();
    assert!(
        closed >= 2,
        "the first two renderers logged their close ({closed}): {logs}"
    );
    let started = |engine: &str| {
        logs.lines()
            .filter(|line| line.contains("engine started") && line.contains(engine))
            .count()
    };
    assert_eq!(
        started("engine=\"WebView2\""),
        started(""),
        "every renderer plays on WebView2: {logs}"
    );
    assert!(!logs.contains(" WARN "), "no renderer warned: {logs}");
}

/// The restart prompt's "Restart now" (D-17): with a Web wallpaper running on CEF, turning Allow invalid certificates on
/// is saved and broadcast, the runtime state names CEF among the engines that run, and RestartWallpapers for CEF ends that
/// renderer and starts a new one that reads the switch (its log says so). A restart asked for another engine leaves the
/// renderer alone. It plays a real wallpaper, so it runs on purpose and skips where CEF is not installed.
#[test]
#[ignore = "plays a Web wallpaper on the desktop from a scratch data folder; plan 16 runs it"]
fn restart_now_restarts_only_the_affected_wallpapers() {
    if !service_runs("restart_now_restarts_only_the_affected_wallpapers") {
        return;
    }
    let service = TestService::start("restart-now", &json!({}));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) {
        skipped(
            "restart_now_restarts_only_the_affected_wallpapers",
            "CEF is not installed next to the executable",
        );
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let started = |count: usize| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let logs = renderer_logs(&service);
            if logs.matches("engine started").count() >= count {
                return logs;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the renderer did not start {count} time(s): {logs}"
            );
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    };
    let first = started(1);
    assert!(
        first.contains("allow_invalid_certificates=false"),
        "certificates are validated by default: {first}"
    );
    // The runtime state names the engine of the wallpaper that runs, once its renderer is up.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "no runtime state with the running engine"
        );
        match portal.next_event_any() {
            Some(PortalEvent::Runtime(runtime)) if !runtime.running_engines.is_empty() => {
                assert!(
                    runtime
                        .running_engines
                        .iter()
                        .all(|(_, engine)| *engine == EngineType::CefSharpLive),
                    "{runtime:?}"
                );
                break;
            }
            Some(_) => {}
            None => panic!("no runtime state with the running engine"),
        }
    }

    // Saved first: the change comes back as Changed, whatever happens to the wallpaper.
    portal.request(&PortalRequest::Change(SettingChange::AllowInvalidCertificates(true)));
    loop {
        match portal.next_event_any() {
            Some(PortalEvent::Changed(SettingChange::AllowInvalidCertificates(true))) => break,
            Some(_) => {}
            None => panic!("the certificate switch was not broadcast"),
        }
    }
    assert_eq!(service.settings_file()["Engine"]["AllowInvalidCertificates"], true);
    assert_eq!(
        renderer_logs(&service).matches("engine started").count(),
        1,
        "a saved setting never restarts the wallpaper by itself"
    );

    // Another engine's restart changes nothing.
    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::MpvPlayerLive],
    });
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(
        renderer_logs(&service).matches("engine started").count(),
        1,
        "a restart of mpv leaves the CEF renderer alone"
    );

    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::CefSharpLive],
    });
    let logs = started(2);
    assert!(
        logs.contains("allow_invalid_certificates=true"),
        "the new renderer reads the switch: {logs}"
    );
}

/// WebView2 switches saved from a Portal client reach the next renderer (D-17, WAPI-10): with a Web wallpaper on
/// WebView2, the legacy defaults plus `Translate` in `disable-features` are saved through `WebArguments`, stored in the
/// legacy list form, and `RestartWallpapers` for WebView2 starts a renderer whose `chromium switches` line has the
/// merged value, while the first renderer did not have it. It plays a real wallpaper, so it runs on purpose and skips
/// where WebView2 cannot play Web here.
#[test]
#[ignore = "plays a Web wallpaper on the desktop from a scratch data folder; plan 26 runs it"]
fn saved_webview2_arguments_reach_the_restarted_renderer() {
    const TEST: &str = "saved_webview2_arguments_reach_the_restarted_renderer";
    if !service_runs(TEST) {
        return;
    }
    let service = TestService::start("webview2-arguments", &json!({ "Engine": { "Web": "WebViewLive" } }));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(TEST, "the WebView2 Runtime is missing");
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let first = wait_for_engine_starts(&service, 1);
    if !first.contains("engine started engine=\"WebView2\"") {
        skipped(
            TEST,
            "WebView2 did not start the wallpaper here (the service fell back to another engine)",
        );
        return;
    }
    assert!(
        !first.contains("Translate"),
        "the legacy defaults do not name Translate: {first}"
    );

    // The list the editor would start from: the legacy defaults, with Translate added to disable-features.
    portal.request(&PortalRequest::GetArguments {
        engine: EngineType::WebViewLive,
    });
    let mut list = loop {
        match portal.next_event_any() {
            Some(PortalEvent::Arguments {
                engine: EngineType::WebViewLive,
                user,
                ..
            }) => break user,
            Some(_) => {}
            None => panic!("no Arguments answer for WebView2"),
        }
    };
    let disabled = list
        .iter_mut()
        .find(|switch| switch.name == "disable-features")
        .expect("the legacy defaults carry disable-features");
    let merged = format!("{},Translate", disabled.value.as_deref().unwrap_or_default());
    disabled.value = Some(merged);

    portal.request(&PortalRequest::Change(SettingChange::WebArguments(list.clone())));
    loop {
        match portal.next_event_any() {
            Some(PortalEvent::Changed(SettingChange::WebArguments(stored))) => {
                assert_eq!(stored, list, "the service stores what it was sent");
                break;
            }
            Some(PortalEvent::Rejected { reason, .. }) => panic!("the list was refused: {reason}"),
            Some(_) => {}
            None => panic!("the WebView2 switches were not broadcast"),
        }
    }
    let stored = &service.settings_file()["Engine"]["WebArguments"];
    let stored = stored.as_array().expect("WebArguments is the legacy list");
    assert!(
        stored.iter().any(|entry| entry == "--disable-web-security"),
        "a flag is stored as --name: {stored:?}"
    );
    assert!(
        stored
            .iter()
            .filter_map(Value::as_str)
            .any(|entry| entry.starts_with("--disable-features=") && entry.ends_with(",Translate")),
        "a valued switch is stored as --name=value: {stored:?}"
    );
    assert_eq!(
        renderer_logs(&service).matches("engine started").count(),
        1,
        "a saved setting never restarts the wallpaper by itself"
    );

    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::WebViewLive],
    });
    let logs = wait_for_engine_starts(&service, 2);
    let switches: Vec<&str> = logs.lines().filter(|line| line.contains("chromium switches")).collect();
    assert_eq!(switches.len(), 2, "two renderers logged their switches: {logs}");
    let second = switches
        .iter()
        .find(|line| line.contains("Translate"))
        .unwrap_or_else(|| panic!("the second renderer has the saved switch: {logs}"));
    assert!(second.contains("disable-features="), "{second}");
}

/// Open DevTools (D-13): with a Web wallpaper running on CEF, the Portal's `OpenDevTools` for its monitor reaches that
/// monitor's renderer, which opens the developer tools, logs `devtools opened` and confirms, so the answer is `opened: true`.
/// A monitor without a wallpaper is answered `opened: false` and nothing opens. Nothing opens the developer tools before
/// the request. It plays a real wallpaper and opens a real DevTools window, so it runs on purpose and skips where CEF is
/// not installed; the service and its renderers are stopped when the test ends, which closes the window.
#[test]
#[ignore = "plays a Web wallpaper and opens DevTools on the desktop from a scratch data folder; plan 17 runs it"]
fn open_devtools_reaches_the_web_renderer() {
    open_devtools_roundtrip(
        "open_devtools_reaches_the_web_renderer",
        "open-devtools",
        &json!({}),
        None,
    );
}

/// The same steps with the web engine WebView2 (plan 25, D-13): skipped when WebView2 cannot play Web here (the service
/// then plays it with CEF, which the first test covers). Closing what it started is stopping the service, never input.
#[test]
#[ignore = "plays a Web wallpaper and opens DevTools on the desktop from a scratch data folder; plan 25 runs it"]
fn open_devtools_reaches_the_webview2_renderer() {
    open_devtools_roundtrip(
        "open_devtools_reaches_the_webview2_renderer",
        "open-devtools-webview2",
        &json!({ "Engine": { "Web": "WebViewLive" } }),
        Some("WebView2"),
    );
}

/// Plays the `hello` Web fixture on a scratch service with `overrides`, asks an unknown monitor and the real one to open
/// the developer tools and checks the answers and the renderer log. `engine` names the engine the renderer has to start;
/// the run is skipped when another one starts.
fn open_devtools_roundtrip(test: &str, scratch: &str, overrides: &Value, engine: Option<&str>) {
    if !service_runs(test) {
        return;
    }
    let service = TestService::start(scratch, overrides);
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) {
        skipped(test, "no web engine is installed next to the executable");
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !renderer_logs(&service).contains("engine started") {
        assert!(
            std::time::Instant::now() < deadline,
            "the renderer did not start its engine: {}",
            renderer_logs(&service)
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    if let Some(engine) = engine
        && !renderer_logs(&service).contains(&format!("engine started engine=\"{engine}\""))
    {
        skipped(
            test,
            &format!("{engine} did not start the wallpaper here (the service fell back to another engine)"),
        );
        return;
    }
    assert!(
        !renderer_logs(&service).contains("devtools opened"),
        "nothing opens the developer tools by itself (D-13)"
    );
    // The monitor the wallpaper plays on, from the runtime state.
    let monitor = loop {
        match portal.next_event_any() {
            Some(PortalEvent::Runtime(runtime)) if !runtime.running_engines.is_empty() => {
                break runtime.running_engines[0].0.clone();
            }
            Some(_) => {}
            None => panic!("no runtime state with the running engine"),
        }
    };
    let answer = |portal: &mut PortalClient| loop {
        match portal.next_event_any() {
            Some(PortalEvent::DevToolsResult { opened }) => return opened,
            Some(_) => {}
            None => panic!("no DevToolsResult"),
        }
    };

    // A monitor without a wallpaper has nothing to open.
    portal.request(&PortalRequest::OpenDevTools {
        monitor: "NO-SUCH-MONITOR".to_owned(),
    });
    assert!(!answer(&mut portal), "an unknown monitor opens nothing");
    assert!(!renderer_logs(&service).contains("devtools opened"));

    portal.request(&PortalRequest::OpenDevTools { monitor });
    assert!(
        answer(&mut portal),
        "the web renderer confirms that it opened the developer tools"
    );
    assert!(
        renderer_logs(&service).contains("devtools opened"),
        "the renderer logs it: {}",
        renderer_logs(&service)
    );
}

/// Waits for an engine report whose first fallback satisfies `wanted`, skipping the other events.
fn wait_for_fallback(portal: &PortalClient, wanted: impl Fn(&FallbackView) -> bool) -> FallbackView {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        match portal.next_event_any() {
            Some(PortalEvent::Engines(report)) => {
                if let Some(fallback) = report.fallbacks.iter().find(|fallback| wanted(fallback)) {
                    return *fallback;
                }
            }
            Some(_) => {}
            None => break,
        }
    }
    panic!("no engine report with the expected fallback");
}

/// "Don't remind me for a day" (D-16): a Web wallpaper whose configured engine is WebView2 plays with CEF, the Portal is
/// told to notify; after SnoozeFallback the Warehouse field holds a quiet time and the report says notify false, also
/// after a restart. It plays a real wallpaper, so it runs on purpose and skips where CEF is not installed.
#[test]
#[ignore = "plays a Web wallpaper on the desktop from a scratch data folder; plan 15 runs it"]
fn a_snoozed_fallback_is_not_announced_again() {
    if !service_runs("a_snoozed_fallback_is_not_announced_again") {
        return;
    }
    let service = TestService::start("snooze", &json!({ "Engine": { "Web": "WebViewLive" } }));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) {
        skipped(
            "a_snoozed_fallback_is_not_announced_again",
            "CEF is not installed next to the executable",
        );
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    // `use_theme` skips the engine reports that came with the Use, so ask for the current one.
    portal.request(&PortalRequest::DetectEngines);
    let fallback = wait_for_fallback(&portal, |_| true);
    assert_eq!(fallback.configured, EngineType::WebViewLive);
    assert!(fallback.notify, "nothing was snoozed yet");

    portal.request(&PortalRequest::SnoozeFallback {
        engine: EngineType::WebViewLive,
    });

    let fallback = wait_for_fallback(&portal, |fallback| !fallback.notify);
    assert_eq!(fallback.playing, EngineType::CefSharpLive, "the fallback itself stays");
    let stored = service.settings_file()["Warehouse"]["WebViewTime"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(
        !stored.is_empty() && !stored.starts_with("0001"),
        "the quiet time is stored: {stored:?}"
    );
    assert_eq!(
        service.settings_file()["Warehouse"]["CefSharpTime"],
        "0001-01-01T00:00:00",
        "the other engine's field is untouched"
    );
    drop(portal);

    let service = service.restart();
    let (mut portal, _) = PortalClient::hello(&service, None);
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    portal.request(&PortalRequest::DetectEngines);
    let fallback = wait_for_fallback(&portal, |_| true);
    assert_eq!(fallback.configured, EngineType::WebViewLive);
    assert!(!fallback.notify, "the snooze survives a restart");
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
        .filter(|theme| check_selected(&location, &theme.folder, &version, &[WallpaperType::Video]).is_ok())
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

/// Sends a MaintainLibrary and returns the report the service answers with, skipping the other events.
fn maintain(portal: &mut PortalClient) -> MaintainReport {
    portal.request(&PortalRequest::MaintainLibrary);
    loop {
        match portal.next_event() {
            Some(PortalEvent::LibraryMaintained(report)) => return report,
            Some(_) => {}
            None => panic!("no LibraryMaintained"),
        }
    }
}

#[test]
fn showcase_themes_are_copied_once_and_recorded_in_the_settings() {
    if !service_runs("showcase_themes_are_copied_once_and_recorded_in_the_settings") {
        return;
    }
    let service = TestService::start("showcase-once", &json!({ "Warehouse": { "Showcase": ["Listed"] } }));
    for name in ["Listed", "Fresh"] {
        let theme = service.dir.join("Showcase").join(name);
        std::fs::create_dir_all(&theme).expect("the showcase theme folder can be created");
        std::fs::write(theme.join("SucroseInfo.json"), b"{}").expect("the showcase file can be written");
    }
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    let report = maintain(&mut portal);

    assert_eq!(report.showcase_copied, 1);
    assert_eq!(report.corrupt_moved, 0);
    assert!(service.dir.join("Library/Fresh/SucroseInfo.json").is_file());
    assert!(
        !service.dir.join("Library/Listed").exists(),
        "a listed name is not copied"
    );
    let recorded = service.settings_file()["Warehouse"]["Showcase"].clone();
    assert_eq!(recorded, json!(["Listed", "Fresh"]));

    // The second run finds every name recorded and copies nothing.
    assert_eq!(maintain(&mut portal).showcase_copied, 0);
}

/// Sends a MoveLibrary and returns the service's answer, skipping the other events (Runtime and the like).
/// The next Startup answer. Memory events arrive every second, so a bare read loop would never time out.
fn next_startup(portal: &PortalClient) -> StartupInfo {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        match portal.next_event() {
            Some(PortalEvent::Startup(info)) => return info,
            Some(_) => {}
            None => break,
        }
    }
    panic!("no Startup answer");
}

#[test]
fn refresh_startup_answers_the_stored_mode() {
    if !service_runs("refresh_startup_answers_the_stored_mode") {
        return;
    }
    let service = TestService::start("startup-refresh", &json!({ "General": { "RunStartup": 3 } }));
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    portal.request(&PortalRequest::RefreshStartup);
    let info = next_startup(&portal);

    assert_eq!(info.mode, StartupMode::Scheduler);
    assert!(
        !info.entry_present,
        "Scheduler writes no entry of Sucrose's own: {info:?}"
    );
}

#[test]
fn set_startup_priority_and_scheduler_are_refused_and_change_nothing() {
    if !service_runs("set_startup_priority_and_scheduler_are_refused_and_change_nothing") {
        return;
    }
    let service = TestService::start("startup-refused", &json!({ "General": { "RunStartup": 3 } }));
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    for mode in [StartupMode::Scheduler, StartupMode::Priority] {
        portal.request(&PortalRequest::SetStartup(mode));
        let info = next_startup(&portal);

        assert_eq!(
            info.mode,
            StartupMode::Scheduler,
            "asked for {mode:?}: the stored mode stays"
        );
        assert!(!info.entry_present, "{info:?}");
    }
    assert_eq!(service.settings_file()["General"]["RunStartup"], 3);
}

fn move_library(portal: &mut PortalClient, destination: &std::path::Path) -> MoveOutcome {
    portal.request(&PortalRequest::MoveLibrary {
        destination: destination.to_string_lossy().into_owned(),
    });
    // Memory events arrive every second, so a bare read loop would never time out.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        match portal.next_event() {
            Some(PortalEvent::MoveResult(outcome)) => return outcome,
            Some(_) => {}
            None => break,
        }
    }
    panic!("no MoveResult");
}

/// The names inside `folder`, sorted.
fn names_in(folder: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .expect("the folder can be read")
        .map(|entry| {
            entry
                .expect("the entry can be read")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn a_library_moves_to_an_empty_folder_and_the_location_follows() {
    if !service_runs("a_library_moves_to_an_empty_folder_and_the_location_follows") {
        return;
    }
    let service = TestService::start("move-ok", &json!({ "Library": { "Move": true } }));
    copy_fixture_library(&service);
    let old = service.dir.join("Library");
    let before = names_in(&old);
    assert!(!before.is_empty(), "the fixture library has content");
    std::fs::write(old.join("loose.txt"), b"loose").expect("a loose file can be written");
    let new = service.dir.join("NewLibrary");
    std::fs::create_dir_all(&new).expect("the new folder can be created");
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    let outcome = move_library(&mut portal, &new);

    assert_eq!(outcome, MoveOutcome::Moved);
    assert_eq!(
        service.settings_file()["Library"]["Location"],
        json!(new.to_string_lossy())
    );
    assert!(!old.exists(), "the old folder is gone");
    let mut expected = before;
    expected.push("loose.txt".to_owned());
    expected.sort();
    assert_eq!(names_in(&new), expected, "every entry, themes and loose files, arrived");
    assert_eq!(
        std::fs::read(new.join("loose.txt")).ok().as_deref(),
        Some(b"loose".as_slice())
    );
}

#[test]
fn a_destination_that_is_not_empty_is_refused_and_nothing_changes() {
    if !service_runs("a_destination_that_is_not_empty_is_refused_and_nothing_changes") {
        return;
    }
    let service = TestService::start("move-refused", &json!({ "Library": { "Move": true } }));
    copy_fixture_library(&service);
    let old = service.dir.join("Library");
    let inside = old.join("Inner");
    std::fs::create_dir_all(&inside).expect("a folder inside the library can be created");
    let before = names_in(&old);
    let full = service.dir.join("Full");
    std::fs::create_dir_all(&full).expect("the folder can be created");
    std::fs::write(full.join("mine.txt"), b"mine").expect("the file can be written");
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    assert_eq!(
        move_library(&mut portal, &full),
        MoveOutcome::Refused(MoveRefusal::NotEmpty)
    );
    assert_eq!(move_library(&mut portal, &old), MoveOutcome::Refused(MoveRefusal::Same));
    assert_eq!(
        move_library(&mut portal, &inside),
        MoveOutcome::Refused(MoveRefusal::Nested)
    );
    assert_eq!(
        move_library(&mut portal, &service.dir.join("Nowhere")),
        MoveOutcome::Refused(MoveRefusal::Missing)
    );
    assert_eq!(
        move_library(&mut portal, std::path::Path::new("relative")),
        MoveOutcome::Refused(MoveRefusal::Missing)
    );

    assert_eq!(names_in(&old), before, "the library is untouched");
    assert_eq!(names_in(&full), ["mine.txt"]);
    assert_eq!(
        service.settings_file()["Library"]["Location"],
        json!(old.to_string_lossy()),
        "the location is unchanged"
    );
}

#[test]
fn with_move_off_only_the_location_changes() {
    if !service_runs("with_move_off_only_the_location_changes") {
        return;
    }
    let service = TestService::start("move-off", &json!({ "Library": { "Move": false } }));
    copy_fixture_library(&service);
    let old = service.dir.join("Library");
    let before = names_in(&old);
    // Not empty: with Move off a folder that already holds themes is a fine location (legacy).
    let other = service.dir.join("Other");
    std::fs::create_dir_all(&other).expect("the folder can be created");
    std::fs::write(other.join("theme.txt"), b"t").expect("the file can be written");
    let (mut portal, first) = PortalClient::hello(&service, None);
    assert!(matches!(first, PortalEvent::Welcome { .. }), "{first:?}");

    let outcome = move_library(&mut portal, &other);

    assert_eq!(outcome, MoveOutcome::Changed);
    assert_eq!(names_in(&old), before, "the themes stay in the old folder");
    assert_eq!(names_in(&other), ["theme.txt"]);
    assert_eq!(
        service.settings_file()["Library"]["Location"],
        json!(other.to_string_lossy())
    );
}
