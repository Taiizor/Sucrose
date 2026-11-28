//! The real service binary as a Portal client (D-033). Each test starts `sucrose` on its own scratch folder, so it
//! never meets another test's service or the user's own.

mod common;

use std::{collections::BTreeMap, path::PathBuf};

use common::{
    PortalClient, TestService, copy_fixture_library, copy_web_fixture, service_runs, skipped, write_gif_theme,
};
use serde_json::{Value, json};
use sucrose_core::{
    library::scan,
    paths::legacy_data_dir,
    settings::{DisplayScreenType, EngineType, ThemeMode},
    theme::{WallpaperType, check_selected},
    version::app_version,
};
use sucrose_ipc::{
    connect,
    portal::{
        EngineReportView, EngineState, FallbackView, Launch, LaunchReply, MaintainReport, MoveOutcome, MoveRefusal,
        PortalEvent, PortalPage, PortalRequest, RuntimeState, SettingChange, StartupInfo, StartupMode, UseOutcome,
        UseRefusal,
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
fn use_of_an_application_theme_is_refused_and_the_selection_is_unchanged() {
    if !service_runs("use_of_an_application_theme_is_refused_and_the_selection_is_unchanged") {
        return;
    }
    let service = TestService::start("use-application", &json!({}));
    copy_fixture_library(&service);
    // An Application theme: no fixture holds one, and the type is the one no engine plays yet (Round E). Gif plays
    // since 04-30, Url since 04-34 and YouTube since 04-35.
    let theme = service.dir.join("Library").join("application-source");
    std::fs::create_dir_all(&theme).expect("the theme folder can be created");
    std::fs::write(
        theme.join("SucroseInfo.json"),
        r#"{"AppVersion":"26.10.19.0","Title":"Clip","Type":5,"Source":"clip.exe"}"#,
    )
    .expect("the theme file can be written");
    std::fs::write(theme.join("clip.exe"), b"not a program").expect("the source file can be written");
    let (mut portal, _) = PortalClient::hello(&service, None);

    let outcome = use_theme(&mut portal, "application-source");

    assert_eq!(outcome, UseOutcome::Refused(UseRefusal::TypeNotPlayable));
    assert_eq!(service.settings_file()["Library"]["Selected"], "");
}

/// The slot of a renderer log file (`Renderer-<slot>.<date>.log`); `CefRenderer-` and `Service.` files are not renderer logs.
fn slot_of(file_name: &str) -> Option<u32> {
    let (slot, _) = file_name.strip_prefix("Renderer-")?.split_once('.')?;
    slot.parse().ok()
}

/// Turns (file name, text) pairs into the text of each renderer slot: the files of one slot are joined in file-name
/// (date) order, each ending in a line break, so the lines of two files never merge.
fn logs_by_slot(mut files: Vec<(String, String)>) -> BTreeMap<u32, String> {
    files.sort();
    let mut by_slot = BTreeMap::new();
    for (name, text) in files {
        let Some(slot) = slot_of(&name) else { continue };
        let log: &mut String = by_slot.entry(slot).or_default();
        log.push_str(&text);
        if !text.ends_with('\n') {
            log.push('\n');
        }
    }
    by_slot
}

/// The renderer logs of `service` (`<data>/Log/Renderer-<slot>.<date>.log`) by slot, empty while there are none yet.
fn renderer_logs_by_slot(service: &TestService) -> BTreeMap<u32, String> {
    let Ok(entries) = std::fs::read_dir(service.dir.join("Log")) else {
        return BTreeMap::new();
    };
    let files = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            slot_of(&name)?;
            Some((name, std::fs::read_to_string(entry.path()).ok()?))
        })
        .collect();
    logs_by_slot(files)
}

/// The text of the renderer logs of `service` joined over all slots, for presence checks that hold with any number of
/// monitors; counts belong to one slot's log.
fn renderer_logs(service: &TestService) -> String {
    renderer_logs_by_slot(service).into_values().collect()
}

/// The engine-start lines of one slot's log, in log order.
fn starts(log: &str) -> Vec<&str> {
    log.lines().filter(|line| line.contains("engine started")).collect()
}

/// Whether the renderer of every monitor slot logged exactly one engine start.
fn exactly_one_start_per_slot(by_slot: &BTreeMap<u32, String>) -> bool {
    by_slot.len() == monitor_slots() && by_slot.values().all(|log| starts(log).len() == 1)
}

/// Whether at least `slots` slots have at least `count` engine starts each.
fn every_slot_started(by_slot: &BTreeMap<u32, String>, slots: usize, count: usize) -> bool {
    by_slot.values().filter(|log| starts(log).len() >= count).count() >= slots
}

/// How many renderer slots a Use starts: the service starts one renderer per connected monitor (apps/sucrose/src/service.rs),
/// and the OS list reaches this process too. Never less than one.
fn monitor_slots() -> usize {
    sucrose_platform::current().monitors().len().max(1)
}

/// Waits until the renderer of every monitor slot logged at least `count` engine starts and returns the logs by slot.
fn wait_for_engine_starts_on_every_slot(service: &TestService, count: usize) -> BTreeMap<u32, String> {
    wait_for_engine_starts_polling(service, count, std::time::Duration::from_millis(250))
}

/// The wait of [`wait_for_engine_starts_on_every_slot`] with its own poll interval, for a test that has to act right
/// after the starts.
fn wait_for_engine_starts_polling(
    service: &TestService,
    count: usize,
    poll: std::time::Duration,
) -> BTreeMap<u32, String> {
    let slots = monitor_slots();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let by_slot = renderer_logs_by_slot(service);
        if every_slot_started(&by_slot, slots, count) {
            return by_slot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{slots} slot(s) had to log {count} start(s) each, seen {:?}: {by_slot:?}",
            by_slot
                .iter()
                .map(|(slot, log)| (*slot, starts(log).len()))
                .collect::<Vec<_>>()
        );
        std::thread::sleep(poll);
    }
}

/// The renderer log file names carry the slot (`Renderer-<slot>.<date>.log`); the files of one slot are joined in date
/// order, and the engine-start lines of a slot's log are told apart from the rest.
#[test]
fn renderer_logs_are_grouped_by_the_slot_in_their_file_name() {
    assert_eq!(slot_of("Renderer-0.2026-11-27.log"), Some(0));
    assert_eq!(slot_of("Renderer-12.2026-11-28.log"), Some(12));
    assert_eq!(slot_of("CefRenderer-0.log"), None);
    assert_eq!(slot_of("Service.2026-11-27.log"), None);
    assert_eq!(slot_of("Renderer-x.2026-11-27.log"), None);
    assert_eq!(slot_of("Renderer-.log"), None);

    let by_slot = logs_by_slot(vec![
        ("Renderer-1.2026-11-28.log".to_string(), "b2".to_string()),
        ("Service.2026-11-27.log".to_string(), "svc".to_string()),
        ("Renderer-0.2026-11-27.log".to_string(), "a0".to_string()),
        ("CefRenderer-0.log".to_string(), "cef".to_string()),
        ("Renderer-1.2026-11-27.log".to_string(), "b1".to_string()),
    ]);
    assert_eq!(by_slot.keys().copied().collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(by_slot[&0], "a0\n");
    assert_eq!(by_slot[&1], "b1\nb2\n");

    let log = "t engine started engine=\"Cef\"\nassignment received slot=0\nchromium switches x\nt engine started engine=\"WebView2\"\n";
    assert_eq!(
        starts(log),
        vec![
            "t engine started engine=\"Cef\"",
            "t engine started engine=\"WebView2\""
        ]
    );
}

/// The 04-37 shape: two monitors, one CEF renderer each. The joined count already read 2 after the first Use, so a wait
/// for the restart returned before the second renderer existed; every slot has to reach the count on its own.
#[test]
fn every_monitor_slot_has_to_reach_the_count() {
    let start = "t engine started engine=\"Cef\"\n";
    let two_slots_one_start = BTreeMap::from([(0, start.to_string()), (1, start.to_string())]);
    assert!(every_slot_started(&two_slots_one_start, 2, 1));
    assert!(!every_slot_started(&two_slots_one_start, 2, 2));

    let one_slot_two_starts = BTreeMap::from([(0, start.repeat(2))]);
    assert!(
        !every_slot_started(&one_slot_two_starts, 2, 1),
        "the second monitor has no renderer yet"
    );
    assert!(every_slot_started(&one_slot_two_starts, 1, 2));

    assert!(!every_slot_started(&BTreeMap::new(), 1, 1));
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

/// A running Web wallpaper restarts on the engine just chosen: with WebView2 installed, choosing it ends the CEF renderer
/// and starts a new one on WebView2, so nothing falls back. (The fallback itself is covered where an engine is missing:
/// `engines::tests::a_missing_configured_engine_falls_back_for_this_run_and_says_why`.) It plays a real wallpaper, so
/// it skips where CEF or WebView2 is not available. With several monitors every monitor's renderer restarts on the engine
/// chosen.
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
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    for (slot, log) in &first {
        assert!(
            starts(log)[0].contains("engine=\"Cef\""),
            "the default engine is CEF on slot {slot}: {log}"
        );
    }

    portal.request(&PortalRequest::Change(SettingChange::WebEngine(
        EngineType::WebViewLive,
    )));

    let after = wait_for_engine_starts_on_every_slot(&service, 2);
    for (slot, log) in &after {
        let restarted = &starts(log)[1..];
        assert!(
            restarted.iter().all(|line| line.contains("engine=\"WebView2\"")),
            "the renderer of slot {slot} after the change plays on WebView2: {log}"
        );
    }
}

/// An engine change for Gif (D-10, D-11): with mpv chosen, a GIF theme plays on mpv; choosing the built-in choice (written
/// as `XavierLive`) ends that renderer and starts one on the built-in GIF renderer with no fallback; choosing libVLC,
/// which this build lacks, keeps the stored value, plays on the built-in renderer and reports the fallback. It plays a real
/// wallpaper on the desktop from a scratch data folder, so it runs on purpose and skips where mpv is not installed.
#[test]
#[ignore = "plays a GIF wallpaper on the desktop from a scratch data folder; plan 36 runs it"]
fn a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer() {
    const NAME: &str = "a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer";
    if !service_runs(NAME) {
        return;
    }
    let service = TestService::start("gif-engine", &json!({ "Engine": { "Gif": "MpvPlayerLive" } }));
    write_gif_theme(&service, "gif-anim");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    let mpv_ready = report
        .engines
        .iter()
        .any(|engine| engine.engine == EngineType::MpvPlayerLive && matches!(engine.state, EngineState::Ready { .. }));
    if !report.playable.contains(&WallpaperType::Gif) || !mpv_ready {
        skipped(NAME, "libmpv is not installed next to the executable");
        return;
    }
    assert_eq!(use_theme(&mut portal, "gif-anim"), UseOutcome::Applied);
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    for (slot, log) in &first {
        assert!(
            starts(log)[0].contains("engine=\"Mpv\""),
            "the configured engine is mpv on slot {slot}: {log}"
        );
    }

    portal.request(&PortalRequest::Change(SettingChange::GifEngine(EngineType::XavierLive)));

    let after = wait_for_engine_starts_on_every_slot(&service, 2);
    for (slot, log) in &after {
        let started = |engine: &str| starts(log).iter().filter(|line| line.contains(engine)).count();
        assert_eq!(
            started("engine=\"BuiltInGif\""),
            1,
            "the second renderer of slot {slot} plays on the built-in one: {log}"
        );
        assert_eq!(started("engine=\"Mpv\""), 1, "mpv started once on slot {slot}: {log}");
    }
    assert_eq!(service.settings_file()["Engine"]["Gif"], "XavierLive");
    // Whatever reports followed the restart, the last one names no fallback for Gif.
    portal.request(&PortalRequest::DetectEngines);
    // The service answers in order, so the report of DetectEngines is the last one; Memory events keep coming, so the
    // read ends at the deadline, not at silence.
    let mut last = None;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if let Some(PortalEvent::Engines(report)) = portal.next_event_any() {
            last = Some(report);
        }
    }
    let last = last.expect("an engine report followed the change");
    assert!(
        !last
            .fallbacks
            .iter()
            .any(|fallback| fallback.kind == WallpaperType::Gif),
        "the built-in renderer is the configured engine, so nothing falls back: {last:?}"
    );

    portal.request(&PortalRequest::Change(SettingChange::GifEngine(
        EngineType::VlcPlayerLive,
    )));

    let fallback = wait_for_fallback(&portal, |fallback| fallback.kind == WallpaperType::Gif);
    assert_eq!(fallback.configured, EngineType::VlcPlayerLive);
    assert_eq!(
        fallback.playing,
        EngineType::XavierLive,
        "the built-in renderer plays instead (its one canonical name)"
    );
    assert_eq!(
        service.settings_file()["Engine"]["Gif"],
        "VlcPlayerLive",
        "the fallback keeps the setting (D-11)"
    );
}

/// What the Portal last heard of the monitors: the displays of the newest Runtime event (or of the Welcome), read until
/// `wanted` holds for them. Returns that runtime state.
fn wait_for_runtime(
    portal: &PortalClient,
    first: Option<RuntimeState>,
    wanted: impl Fn(&RuntimeState) -> bool,
) -> RuntimeState {
    let mut seen = first;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(runtime) = &seen
            && wanted(runtime)
        {
            return runtime.clone();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the runtime state never matched, the last one was {seen:?}"
        );
        if let Some(PortalEvent::Runtime(runtime)) = portal.next_event_any() {
            seen = Some(runtime);
        }
    }
}

/// Per display on two monitors: the Portal sees both of them with their themes (`RuntimeState.displays`), moves the target
/// monitor with `ScreenDevice`, uses a theme there (only that monitor's renderer restarts) and removes it again with
/// `RemoveWallpaper` (its renderer ends and its entry leaves the file; the other monitor plays on). It plays real
/// wallpapers on two monitors from a scratch data folder, so it runs on purpose and skips with fewer than two.
#[test]
#[ignore = "plays wallpapers on two monitors from a scratch data folder; plan 42 runs it"]
#[allow(clippy::print_stderr)]
fn the_target_monitor_gets_the_use_and_remove_frees_it() {
    const NAME: &str = "the_target_monitor_gets_the_use_and_remove_frees_it";
    if !service_runs(NAME) {
        return;
    }
    let mut monitors = sucrose_platform::current().monitors();
    monitors.sort_by(|a, b| a.id.cmp(&b.id));
    if monitors.len() < 2 {
        eprintln!("{NAME}: skipped: fewer than two monitors are connected");
        return;
    }
    let (first, last) = (monitors[0].clone(), monitors[monitors.len() - 1].clone());
    let entry = |monitor: &sucrose_platform::MonitorInfo, theme: &str| json!({ "MonitorId": monitor.stable_id(), "DeviceName": monitor.id, "Theme": theme });
    // Only these two monitors play: a third one would have no entry and no renderer.
    let service = TestService::start(
        "target-monitor",
        &json!({
            "Library": { "Selected": "gif-anim" },
            "Engine": {
                "Gif": "XavierLive",
                "DisplayScreenType": "PerDisplay",
                "ScreenDevice": "",
                "DisplayAssignments": [entry(&first, "gif-anim"), entry(&last, "gif-two")],
            },
        }),
    );
    // The themes exist before the service reads its settings, so it places both wallpapers at its start.
    for theme in ["gif-anim", "gif-two", "gif-three"] {
        write_gif_theme(&service, theme);
    }
    let service = service.restart();
    let (mut portal, welcome) = PortalClient::hello(&service, None);
    let PortalEvent::Welcome { runtime, .. } = welcome else {
        panic!("the first event is the Welcome: {welcome:?}");
    };

    // Both monitors play their own theme, and the Portal is told which.
    let started = wait_for_engine_starts_on_every_slot(&service, 1);
    assert_eq!(started.len(), 2, "one renderer per assigned monitor: {started:?}");
    let both = |runtime: &RuntimeState| {
        runtime.displays.len() == 2
            && runtime
                .displays
                .iter()
                .all(|display| display.connected && display.running)
    };
    let runtime = wait_for_runtime(&portal, Some(runtime), both);
    let theme_of = |runtime: &RuntimeState, monitor: &sucrose_platform::MonitorInfo| {
        runtime
            .displays
            .iter()
            .find(|display| display.device_name == monitor.id)
            .and_then(|display| display.theme.clone())
    };
    assert_eq!(theme_of(&runtime, &first).as_deref(), Some("gif-anim"));
    assert_eq!(theme_of(&runtime, &last).as_deref(), Some("gif-two"));
    assert!(
        runtime.displays[0].rect[0] <= runtime.displays[1].rect[0],
        "left to right: {:?}",
        runtime.displays
    );
    assert_eq!(
        runtime.selected.as_deref(),
        Some("gif-anim"),
        "the target is the first monitor"
    );

    // The target moves to the other monitor: only the setting and what is selected change.
    portal.request(&PortalRequest::Change(SettingChange::ScreenDevice(last.id.clone())));
    let runtime = wait_for_runtime(&portal, None, |runtime| runtime.selected.as_deref() == Some("gif-two"));
    assert!(both(&runtime), "no renderer changed: {runtime:?}");
    assert_eq!(service.settings_file()["Engine"]["ScreenDevice"], last.id.as_str());
    assert!(
        renderer_logs_by_slot(&service)
            .values()
            .all(|log| starts(log).len() == 1),
        "no renderer restarted for a new target"
    );

    // A Use goes to the target monitor: its renderer restarts on the new theme, the other one is not touched.
    assert_eq!(use_theme(&mut portal, "gif-three"), UseOutcome::Applied);
    let runtime = wait_for_runtime(&portal, None, |runtime| {
        both(runtime) && runtime.selected.as_deref() == Some("gif-three")
    });
    assert_eq!(theme_of(&runtime, &last).as_deref(), Some("gif-three"));
    assert_eq!(theme_of(&runtime, &first).as_deref(), Some("gif-anim"));
    // The renderer of the target monitor ends and comes back on the new theme: wait for its second engine start.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let after = loop {
        let by_slot = renderer_logs_by_slot(&service);
        if by_slot.values().any(|log| starts(log).len() >= 2) {
            break by_slot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no renderer started a second time: {by_slot:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    };
    let restarted: Vec<u32> = after
        .iter()
        .filter(|(_, log)| starts(log).len() >= 2)
        .map(|(slot, _)| *slot)
        .collect();
    assert_eq!(restarted.len(), 1, "exactly one renderer restarted: {after:?}");
    assert!(
        after
            .iter()
            .filter(|(slot, _)| !restarted.contains(slot))
            .all(|(_, log)| starts(log).len() == 1),
        "the other monitor kept its single engine start: {after:?}"
    );
    let stored = &service.settings_file()["Engine"]["DisplayAssignments"];
    let themes: Vec<(&str, &str)> = stored
        .as_array()
        .expect("the assignments are stored")
        .iter()
        .map(|entry| {
            (
                entry["DeviceName"].as_str().unwrap_or(""),
                entry["Theme"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert!(themes.contains(&(last.id.as_str(), "gif-three")), "{themes:?}");
    assert!(themes.contains(&(first.id.as_str(), "gif-anim")), "{themes:?}");

    // Remove Wallpaper frees the target monitor: its renderer ends, its entry leaves the file, the other plays on.
    portal.request(&PortalRequest::RemoveWallpaper {
        monitor_id: Some(last.stable_id().to_owned()),
    });
    let runtime = wait_for_runtime(&portal, None, |runtime| {
        runtime
            .displays
            .iter()
            .any(|display| display.device_name == last.id && display.theme.is_none() && !display.running)
    });
    assert!(
        runtime.displays.iter().any(|display| display.device_name == first.id
            && display.running
            && display.theme.as_deref() == Some("gif-anim")),
        "the other monitor plays on: {runtime:?}"
    );
    assert_eq!(runtime.selected, None, "the target monitor shows nothing now");
    let stored = service.settings_file();
    let entries = stored["Engine"]["DisplayAssignments"]
        .as_array()
        .expect("the assignments are stored");
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["DeviceName"], first.id.as_str());
    // The theme itself is untouched.
    assert!(
        service
            .dir
            .join("Library")
            .join("gif-three")
            .join("SucroseInfo.json")
            .is_file()
    );
}

/// The text of the service logs of `service` (`<data>/Log/Service.<date>.log`), empty while there is none yet.
fn service_log(service: &TestService) -> String {
    let Ok(entries) = std::fs::read_dir(service.dir.join("Log")) else {
        return String::new();
    };
    let mut files: Vec<(String, String)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("Service.").then_some(())?;
            Some((name, std::fs::read_to_string(entry.path()).ok()?))
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, text)| text).collect()
}

/// Waits until `done` holds for the service log and returns it; fails after 30 s.
fn wait_for_service_log(service: &TestService, what: &str, done: impl Fn(&str) -> bool) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let log = service_log(service);
        if done(&log) {
            return log;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the service log never showed {what}: {log}"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

/// The engine starts of every renderer log together.
fn total_engine_starts(service: &TestService) -> usize {
    renderer_logs_by_slot(service)
        .values()
        .map(|log| starts(log).len())
        .sum()
}

/// Waits until the renderer logs hold `count` engine starts altogether, and returns the logs by slot.
fn wait_for_total_engine_starts(service: &TestService, count: usize) -> BTreeMap<u32, String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if total_engine_starts(service) >= count {
            return renderer_logs_by_slot(service);
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{count} engine starts were expected, seen {}: {:?}",
            total_engine_starts(service),
            renderer_logs_by_slot(service)
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

/// Per display on two monitors, then Span, Duplicate and Per display again through `SettingChange::DisplayScreenType`:
/// Span runs one renderer over the union of the monitors (the service log's `wallpaper placed` line names `span:all`
/// and the union), Duplicate one renderer on each monitor with the selected theme, and Per display gives each monitor
/// its own theme back. It plays real wallpapers on two monitors from a scratch data folder, so it runs on purpose and
/// skips with fewer than two.
#[test]
#[ignore = "plays wallpapers on two monitors from a scratch data folder; plan 43 runs it"]
#[allow(clippy::print_stderr)]
fn a_display_mode_change_moves_the_wallpaper_to_the_union_and_back() {
    const NAME: &str = "a_display_mode_change_moves_the_wallpaper_to_the_union_and_back";
    if !service_runs(NAME) {
        return;
    }
    let mut monitors = sucrose_platform::current().monitors();
    monitors.sort_by(|a, b| a.id.cmp(&b.id));
    if monitors.len() < 2 {
        eprintln!("{NAME}: skipped: fewer than two monitors are connected");
        return;
    }
    // Both monitors are assigned, so Per display plays one renderer on each.
    let (first, last) = (monitors[0].clone(), monitors[monitors.len() - 1].clone());
    let entry = |monitor: &sucrose_platform::MonitorInfo, theme: &str| json!({ "MonitorId": monitor.stable_id(), "DeviceName": monitor.id, "Theme": theme });
    let service = TestService::start(
        "display-mode",
        &json!({
            "Library": { "Selected": "gif-anim" },
            "Engine": {
                "Gif": "XavierLive",
                "DisplayScreenType": "PerDisplay",
                "ScreenDevice": "",
                "DisplayAssignments": [entry(&first, "gif-anim"), entry(&last, "gif-two")],
            },
        }),
    );
    for theme in ["gif-anim", "gif-two"] {
        write_gif_theme(&service, theme);
    }
    let service = service.restart();
    let (mut portal, welcome) = PortalClient::hello(&service, None);
    let PortalEvent::Welcome { runtime, .. } = welcome else {
        panic!("the first event is the Welcome: {welcome:?}");
    };
    let slots = monitors.len();
    let theme_of = |runtime: &RuntimeState, monitor: &sucrose_platform::MonitorInfo| {
        runtime
            .displays
            .iter()
            .find(|display| display.device_name == monitor.id)
            .and_then(|display| display.theme.clone())
    };

    // Per display: every connected monitor plays, the first its own theme and the last another one.
    let after_start = wait_for_total_engine_starts(&service, slots);
    assert_eq!(after_start.len(), slots, "one renderer per monitor: {after_start:?}");
    let runtime = wait_for_runtime(&portal, Some(runtime), |runtime| {
        runtime.displays.len() == slots && runtime.displays.iter().all(|display| display.running)
    });
    assert_eq!(theme_of(&runtime, &first).as_deref(), Some("gif-anim"));
    assert_eq!(theme_of(&runtime, &last).as_deref(), Some("gif-two"));

    // Span: one renderer over the union of the monitors, playing the selected theme.
    portal.request(&PortalRequest::Change(SettingChange::DisplayScreenType(
        DisplayScreenType::SpanAcross,
    )));
    let union = {
        let left = monitors
            .iter()
            .map(|monitor| monitor.rect.left)
            .min()
            .expect("a monitor");
        let top = monitors
            .iter()
            .map(|monitor| monitor.rect.top)
            .min()
            .expect("a monitor");
        let right = monitors
            .iter()
            .map(|monitor| monitor.rect.right)
            .max()
            .expect("a monitor");
        let bottom = monitors
            .iter()
            .map(|monitor| monitor.rect.bottom)
            .max()
            .expect("a monitor");
        format!("rect=[{left}, {top}, {right}, {bottom}]")
    };
    let log = wait_for_service_log(&service, "the span placement", |log| {
        log.lines()
            .any(|line| line.contains("wallpaper placed") && line.contains("monitor=span:all"))
    });
    let placed = log
        .lines()
        .rev()
        .find(|line| line.contains("wallpaper placed") && line.contains("monitor=span:all"))
        .expect("the span line");
    assert!(
        placed.contains(&union),
        "the span covers the union of the monitors ({union}): {placed}"
    );
    let runtime = wait_for_runtime(&portal, None, |runtime| {
        runtime
            .displays
            .iter()
            .all(|display| display.running && display.theme.as_deref() == Some("gif-anim"))
    });
    assert_eq!(runtime.displays.len(), slots);
    let by_slot = wait_for_total_engine_starts(&service, slots + 1);
    assert_eq!(
        total_engine_starts(&service),
        slots + 1,
        "exactly one renderer started for the span: {by_slot:?}"
    );
    assert!(
        by_slot.values().any(|log| log.contains("monitor=span:all")),
        "the new renderer was assigned the span target: {by_slot:?}"
    );
    assert_eq!(service.settings_file()["Engine"]["DisplayScreenType"], "SpanAcross");

    // Duplicate: the span renderer ends and every monitor gets a renderer with the selected theme.
    portal.request(&PortalRequest::Change(SettingChange::DisplayScreenType(
        DisplayScreenType::SameDuplicate,
    )));
    let runtime = wait_for_runtime(&portal, None, |runtime| {
        runtime.displays.len() == slots
            && runtime
                .displays
                .iter()
                .all(|display| display.running && display.theme.as_deref() == Some("gif-anim"))
            && runtime.displays.iter().all(|display| display.engine.is_some())
    });
    assert_eq!(
        theme_of(&runtime, &last).as_deref(),
        Some("gif-anim"),
        "both monitors play the selected theme"
    );
    wait_for_total_engine_starts(&service, slots + 1 + slots);
    let log = service_log(&service);
    for monitor in &monitors {
        assert!(
            log.lines().any(|line| line.contains("wallpaper placed")
                && line.contains(&format!("monitor={}", monitor.id))
                && line.contains("theme=\"gif-anim\"")),
            "{} was placed with the selected theme: {log}",
            monitor.id
        );
    }
    assert_eq!(service.settings_file()["Engine"]["DisplayScreenType"], "SameDuplicate");

    // Per display again: the stored assignments come back.
    portal.request(&PortalRequest::Change(SettingChange::DisplayScreenType(
        DisplayScreenType::PerDisplay,
    )));
    let runtime = wait_for_runtime(&portal, None, |runtime| {
        runtime.displays.len() == slots
            && runtime.displays.iter().all(|display| display.running)
            && runtime
                .displays
                .iter()
                .find(|display| display.device_name == last.id)
                .is_some_and(|display| display.theme.as_deref() == Some("gif-two"))
    });
    assert_eq!(theme_of(&runtime, &first).as_deref(), Some("gif-anim"));
    assert_eq!(service.settings_file()["Engine"]["DisplayScreenType"], "PerDisplay");
}

/// Two theme switches in a row on WebView2 (the Round A CEF close lesson): the renderer of each theme closes its
/// controller before the surface goes, the service starts the next one, and the logs show the close and no WARN. It plays
/// real wallpapers, so it skips where CEF or WebView2 is not available. With several monitors every monitor's renderer is
/// checked.
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
    wait_for_engine_starts_on_every_slot(&service, 1);
    assert_eq!(use_theme(&mut portal, "storage"), UseOutcome::Applied);
    wait_for_engine_starts_on_every_slot(&service, 2);
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);

    let by_slot = wait_for_engine_starts_on_every_slot(&service, 3);

    for (slot, logs) in &by_slot {
        let closed = logs.matches("webview2 closed").count();
        assert!(
            closed >= 2,
            "the first two renderers of slot {slot} logged their close ({closed}): {logs}"
        );
        assert!(
            starts(logs).iter().all(|line| line.contains("engine=\"WebView2\"")),
            "every renderer of slot {slot} plays on WebView2: {logs}"
        );
        assert!(!logs.contains(" WARN "), "no renderer of slot {slot} warned: {logs}");
    }
}

/// A WebView2 renderer closed just after its start on a profile folder that was never used lets go of it at once
/// (debug session webview2-browser-exit-slow, D-041). With Edge SmartScreen on, the first navigation of a new folder sent
/// a SmartScreen check, and a close while it was in flight kept the browser process about 6 s, so a quick theme switch
/// waited that long for the next page. Every folder here is new (the scratch folder is made again and an ephemeral
/// folder is emptied at every start), and the GIF theme that follows starts no browser on them. "At once" is within the
/// exit-wait bound (`BROWSER_EXIT_WAIT`, 1.5 s): while the whole test suite runs, such a close nearly always takes the
/// Runtime's own slower shutdown path, which ended 0.82-1.25 s after the close (debug session
/// webview2-browser-exit-can-exce). It plays real wallpapers, so it skips where WebView2 is not available.
#[test]
fn a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once() {
    // The renderer's `BROWSER_EXIT_WAIT` (crates/sucrose-engine-webview2/src/host.rs), in milliseconds.
    const BROWSER_EXIT_WAIT_MS: u64 = 1500;
    const TEST: &str = "a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once";
    if !service_runs(TEST) {
        return;
    }
    let service = TestService::start("webview2-early-close", &json!({ "Engine": { "Web": "WebViewLive" } }));
    copy_web_fixture(&service, "hello");
    write_gif_theme(&service, "gif");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(TEST, "the WebView2 Runtime is missing");
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let first = wait_for_engine_starts_polling(&service, 1, std::time::Duration::from_millis(10));
    if first
        .values()
        .filter_map(|log| starts(log).first().copied())
        .any(|line| !line.contains("engine=\"WebView2\""))
    {
        skipped(TEST, "WebView2 did not start the wallpaper here");
        return;
    }
    // Every monitor's renderer started (the wait is per slot and polls fast): they are closed just after their own start,
    // while the first navigation is under way.
    assert_eq!(use_theme(&mut portal, "gif"), UseOutcome::Applied);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let by_slot = loop {
        let by_slot = renderer_logs_by_slot(&service);
        if first
            .keys()
            .all(|slot| by_slot.get(slot).is_some_and(|log| log.contains("webview2 closed")))
        {
            break by_slot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the WebView2 renderer of every slot did not close: {by_slot:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    };

    for slot in first.keys() {
        let logs = &by_slot[slot];
        let exits: Vec<u64> = logs
            .lines()
            .filter(|line| line.contains("webview2 browser process exited"))
            .filter_map(|line| line.split("elapsed_ms=").nth(1))
            .filter_map(|rest| rest.split_whitespace().next()?.parse().ok())
            .collect();
        assert_eq!(
            exits.len(),
            1,
            "the closed renderer of slot {slot} saw its browser process exit within the bound: {logs}"
        );
        assert!(
            exits.iter().all(|elapsed| *elapsed < BROWSER_EXIT_WAIT_MS),
            "the browser process of slot {slot} exited within the exit-wait bound of the close: {exits:?}"
        );
        assert!(!logs.contains("did not exit in time"), "slot {slot}: {logs}");
        assert!(!logs.contains("no web profile folder"), "slot {slot}: {logs}");
    }
}

/// The WebView2 SmartScreen setting is read from the file at the start (D-041): with `Engine.WebViewSmartScreen` true a
/// Web wallpaper on WebView2 starts without `msSmartScreenProtection` in its switches (the legacy `disable-features`
/// items stay) and its `chromium switches` line says `webview_smartscreen=true`. It plays real wallpapers, so it skips
/// where WebView2 is not available.
#[test]
fn the_webview2_smartscreen_setting_leaves_the_smartscreen_switch_out_of_the_renderer() {
    const TEST: &str = "the_webview2_smartscreen_setting_leaves_the_smartscreen_switch_out_of_the_renderer";
    if !service_runs(TEST) {
        return;
    }
    let service = TestService::start(
        "webview2-smartscreen-on",
        &json!({ "Engine": { "Web": "WebViewLive", "WebViewSmartScreen": true } }),
    );
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(TEST, "the WebView2 Runtime is missing");
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    if first
        .values()
        .filter_map(|log| starts(log).first().copied())
        .any(|line| !line.contains("engine=\"WebView2\""))
    {
        skipped(TEST, "WebView2 did not start the wallpaper here");
        return;
    }
    for (slot, logs) in &first {
        let switches: Vec<&str> = logs.lines().filter(|line| line.contains("chromium switches")).collect();
        assert_eq!(
            switches.len(),
            1,
            "one renderer of slot {slot} logged its switches: {logs}"
        );
        let line = switches[0];
        assert!(line.contains("webview_smartscreen=true"), "slot {slot}: {line}");
        assert!(
            line.contains("IsolateOrigins"),
            "the legacy items stay on slot {slot}: {line}"
        );
        assert!(
            !line.contains("msSmartScreenProtection"),
            "SmartScreen is left on on slot {slot}: {line}"
        );
    }
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
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    for (slot, log) in &first {
        assert!(
            log.contains("allow_invalid_certificates=false"),
            "certificates are validated by default on slot {slot}: {log}"
        );
    }
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
    let by_slot = renderer_logs_by_slot(&service);
    assert!(
        exactly_one_start_per_slot(&by_slot),
        "a saved setting never restarts the wallpaper by itself, one start on every slot: {by_slot:?}"
    );

    // Another engine's restart changes nothing.
    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::MpvPlayerLive],
    });
    std::thread::sleep(std::time::Duration::from_secs(2));
    let by_slot = renderer_logs_by_slot(&service);
    assert!(
        exactly_one_start_per_slot(&by_slot),
        "a restart of mpv leaves the CEF renderer alone, one start on every slot: {by_slot:?}"
    );

    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::CefSharpLive],
    });
    let after = wait_for_engine_starts_on_every_slot(&service, 2);
    for (slot, log) in &after {
        assert!(
            log.contains("allow_invalid_certificates=true"),
            "the new renderer of slot {slot} reads the switch: {log}"
        );
    }
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
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    if first
        .values()
        .filter_map(|log| starts(log).first().copied())
        .any(|line| !line.contains("engine=\"WebView2\""))
    {
        skipped(
            TEST,
            "WebView2 did not start the wallpaper here (the service fell back to another engine)",
        );
        return;
    }
    for (slot, log) in &first {
        assert!(
            !log.contains("Translate"),
            "the legacy defaults do not name Translate on slot {slot}: {log}"
        );
    }

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
    let by_slot = renderer_logs_by_slot(&service);
    assert!(
        exactly_one_start_per_slot(&by_slot),
        "a saved setting never restarts the wallpaper by itself, one start on every slot: {by_slot:?}"
    );

    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::WebViewLive],
    });
    let after = wait_for_engine_starts_on_every_slot(&service, 2);
    for (slot, logs) in &after {
        let switches: Vec<&str> = logs.lines().filter(|line| line.contains("chromium switches")).collect();
        assert_eq!(
            switches.len(),
            2,
            "two renderers of slot {slot} logged their switches: {logs}"
        );
        assert!(
            switches[1].contains("Translate") && switches[1].contains("disable-features="),
            "the second renderer of slot {slot} has the saved switch: {logs}"
        );
    }
}

/// The WebView2 SmartScreen switch of the Portal (D-041): with a Web wallpaper on WebView2, `WebViewSmartScreen(true)` is
/// saved, broadcast and written to `Engine.WebViewSmartScreen`, and restarts nothing by itself. A restart asked for CEF
/// leaves the renderer alone; `RestartWallpapers` for WebView2 starts a second renderer per slot whose `chromium
/// switches` line has no `msSmartScreenProtection` while the first one had it. It plays a real wallpaper, so it runs on
/// purpose and skips where WebView2 cannot play Web here.
#[test]
#[ignore = "plays a Web wallpaper on the desktop from a scratch data folder; quick 261128-0du runs it"]
fn webview2_smartscreen_saved_from_the_portal_restarts_only_the_webview2_wallpapers() {
    const TEST: &str = "webview2_smartscreen_saved_from_the_portal_restarts_only_the_webview2_wallpapers";
    if !service_runs(TEST) {
        return;
    }
    let service = TestService::start("webview2-smartscreen", &json!({ "Engine": { "Web": "WebViewLive" } }));
    copy_web_fixture(&service, "hello");
    let (mut portal, _) = PortalClient::hello(&service, None);
    let report = wait_for_engines(&portal);
    if !report.playable.contains(&WallpaperType::Web) || !webview2_is_ready(&report) {
        skipped(TEST, "the WebView2 Runtime is missing");
        return;
    }
    assert_eq!(use_theme(&mut portal, "hello"), UseOutcome::Applied);
    let first = wait_for_engine_starts_on_every_slot(&service, 1);
    if first
        .values()
        .filter_map(|log| starts(log).first().copied())
        .any(|line| !line.contains("engine=\"WebView2\""))
    {
        skipped(
            TEST,
            "WebView2 did not start the wallpaper here (the service fell back to another engine)",
        );
        return;
    }
    for (slot, log) in &first {
        assert!(
            log.contains("msSmartScreenProtection") && log.contains("webview_smartscreen=false"),
            "SmartScreen is off by default on slot {slot}: {log}"
        );
    }

    portal.request(&PortalRequest::Change(SettingChange::WebViewSmartScreen(true)));
    loop {
        match portal.next_event_any() {
            Some(PortalEvent::Changed(SettingChange::WebViewSmartScreen(true))) => break,
            Some(PortalEvent::Rejected { reason, .. }) => panic!("the SmartScreen switch was refused: {reason}"),
            Some(_) => {}
            None => panic!("the SmartScreen switch was not broadcast"),
        }
    }
    assert_eq!(service.settings_file()["Engine"]["WebViewSmartScreen"], true);
    let by_slot = renderer_logs_by_slot(&service);
    assert!(
        exactly_one_start_per_slot(&by_slot),
        "a saved setting never restarts the wallpaper by itself, one start on every slot: {by_slot:?}"
    );

    // A restart asked for CEF changes nothing: the wallpaper runs on WebView2.
    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::CefSharpLive],
    });
    std::thread::sleep(std::time::Duration::from_secs(2));
    let by_slot = renderer_logs_by_slot(&service);
    assert!(
        exactly_one_start_per_slot(&by_slot),
        "a restart of CEF leaves the WebView2 renderer alone, one start on every slot: {by_slot:?}"
    );

    portal.request(&PortalRequest::RestartWallpapers {
        engines: vec![EngineType::WebViewLive],
    });
    let after = wait_for_engine_starts_on_every_slot(&service, 2);
    for (slot, logs) in &after {
        let switches: Vec<&str> = logs.lines().filter(|line| line.contains("chromium switches")).collect();
        assert_eq!(
            switches.len(),
            2,
            "two renderers of slot {slot} logged their switches: {logs}"
        );
        assert!(
            switches[0].contains("msSmartScreenProtection"),
            "the first renderer of slot {slot} had SmartScreen off: {}",
            switches[0]
        );
        assert!(
            switches[1].contains("webview_smartscreen=true") && !switches[1].contains("msSmartScreenProtection"),
            "the second renderer of slot {slot} reads the saved setting: {}",
            switches[1]
        );
    }
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
