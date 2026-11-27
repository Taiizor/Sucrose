//! The page calls both web engines run: order, gating and the data push, against scratch theme folders and the calls
//! fixture of the service tests.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use sucrose_core::{
    compatible::{CompatibleKey, DataGroup},
    properties::PropertiesError,
    settings::Stretch,
};
use sucrose_engine_web::{
    js::{self, KIND_COMPATIBLE, KIND_PROPERTY, LEGACY_RUN_ORDER, PageCall},
    page::{Complaints, PageKind, PageState, describe, load_templates, page_calls, push_calls, wanted_groups},
};

/// A scratch theme folder with the given `SucroseProperties.json` and `SucroseCompatible.json` texts (none: no file),
/// removed on drop.
struct Theme(PathBuf);

impl Theme {
    fn new(properties: Option<&str>, compatible: Option<&str>) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-page-calls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch theme");
        for (name, text) in [
            ("SucroseProperties.json", properties),
            ("SucroseCompatible.json", compatible),
        ] {
            if let Some(text) = text {
                std::fs::write(dir.join(name), text).expect("write theme file");
            }
        }
        Self(dir)
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TWO_PROPERTIES: &str = r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{"b":{"type":"checkbox","value":true},"a":{"type":"textbox","value":"x"}}}"#;
const COMPATIBLE: &str =
    r#"{"VolumeLevel":"V({0});","SystemMemory":"M({0});","LoopMode":"L('{0}');","StretchMode":"S('{0}');"}"#;
const WITH_DATA: &str =
    r#"{"LoopMode":"L('{0}');","SystemDate":"D({0});","SystemMemory":"M({0});","SystemBattery":""}"#;

fn fresh_state() -> PageState {
    PageState::new(false, 100, true, true, Stretch::UniformToFill, "Dark")
}

fn keys(calls: &[PageCall]) -> Vec<String> {
    calls.iter().map(|call| format!("{}:{}", call.kind, call.key)).collect()
}

/// The calls fixture of the service tests: ten property controls and a compatible file with data groups.
fn calls_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/sucrose/tests/fixtures/web-themes/calls")
}

#[test]
fn a_loaded_page_gets_its_property_calls_in_file_order_and_a_failed_load_gets_none() {
    let theme = Theme::new(Some(TWO_PROPERTIES), None);
    let (state, complaints) = (fresh_state(), Complaints::default());

    let calls = page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints);
    let failed = page_calls(PageKind::Theme, &theme.0, 404, &state, &complaints);
    let unloaded = page_calls(PageKind::Theme, &theme.0, 0, &state, &complaints);

    assert_eq!(keys(&calls), ["property:b", "property:a"]);
    assert!(failed.is_empty(), "only a page that loaded with status 200 gets calls");
    assert!(unloaded.is_empty(), "a failed navigation (status 0) gets none");
    assert!(!complaints.properties.load(Ordering::SeqCst));
}

#[test]
fn the_compatible_calls_follow_every_property_call_in_the_legacy_order_without_the_data_keys() {
    let theme = Theme::new(Some(TWO_PROPERTIES), Some(COMPATIBLE));
    let (state, complaints) = (fresh_state(), Complaints::default());

    let calls = page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints);

    assert_eq!(
        keys(&calls),
        [
            "property:b",
            "property:a",
            "compatible:LoopMode",
            "compatible:StretchMode",
            "compatible:VolumeLevel"
        ]
    );
    let scripts: Vec<&str> = calls.iter().skip(2).map(|call| call.script.as_str()).collect();
    assert_eq!(scripts, ["L('True');", "S('UniformToFill');", "V(100);"]);
}

#[test]
fn the_compatible_calls_use_the_values_the_engine_was_last_told() {
    let theme = Theme::new(None, Some(COMPATIBLE));
    let state = fresh_state();
    state.set_stretch(Stretch::Fill);
    state.set_looping(false);
    state.set_volume(40);

    let calls = page_calls(PageKind::Theme, &theme.0, 200, &state, &Complaints::default());

    let scripts: Vec<&str> = calls.iter().map(|call| call.script.as_str()).collect();
    assert_eq!(scripts, ["L('False');", "S('Fill');", "V(40);"]);
}

#[test]
fn a_reload_sends_every_key_again() {
    let theme = Theme::new(Some(TWO_PROPERTIES), Some(COMPATIBLE));
    let (state, complaints) = (fresh_state(), Complaints::default());

    let first = page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints);
    let reload = page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints);

    assert_eq!(first, reload);
    assert_eq!(reload.len(), 5);
}

#[test]
fn a_page_that_loads_while_paused_is_paused_after_its_calls() {
    let theme = Theme::new(Some(TWO_PROPERTIES), None);
    let state = fresh_state();
    state.set_paused(true);

    let calls = page_calls(PageKind::Theme, &theme.0, 200, &state, &Complaints::default());

    assert_eq!(keys(&calls), ["property:b", "property:a", "pause:pause"]);
    assert_eq!(calls[2].script, js::pause_script(true));
}

#[test]
fn a_theme_without_files_gets_no_calls_and_no_complaint() {
    let theme = Theme::new(None, None);
    let (state, complaints) = (fresh_state(), Complaints::default());

    assert!(page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints).is_empty());
    assert!(!complaints.properties.load(Ordering::SeqCst) && !complaints.compatible.load(Ordering::SeqCst));
}

#[test]
fn an_unparsable_file_gives_no_calls_for_it_and_is_reported_once_while_the_other_still_runs() {
    let theme = Theme::new(Some("{ nope"), Some(COMPATIBLE));
    let (state, complaints) = (fresh_state(), Complaints::default());

    let calls = page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints);

    assert_eq!(
        keys(&calls),
        [
            "compatible:LoopMode",
            "compatible:StretchMode",
            "compatible:VolumeLevel"
        ],
        "the page keeps playing and gets the other file's calls"
    );
    assert!(
        complaints.properties.load(Ordering::SeqCst),
        "the first failure is the one that is logged"
    );
    assert!(!complaints.compatible.load(Ordering::SeqCst));
    assert_eq!(page_calls(PageKind::Theme, &theme.0, 200, &state, &complaints).len(), 3);
}

#[test]
fn the_state_starts_from_its_values_and_reports_the_legacy_theme_name() {
    let state = PageState::new(false, 250, true, false, Stretch::UniformToFill, "Light");

    let compatible = state.compatible();

    assert!(!compatible.shuffle);
    assert_eq!(compatible.theme_type, "Light");
    assert!(compatible.looping);
    assert_eq!(compatible.volume, 100, "the volume is capped at 100");
}

#[test]
fn the_log_reason_names_the_problem_without_a_path() {
    let missing = describe(&PropertiesError::Missing(PathBuf::from("C:/secret/x.json")));
    let invalid = describe(&PropertiesError::Invalid {
        path: PathBuf::from("C:/secret/x.json"),
        reason: "invalid JSON at line 1 column 3".to_owned(),
    });

    assert!(!missing.contains("secret") && !invalid.contains("secret"));
    assert!(invalid.contains("line 1"));
}

#[test]
fn the_wanted_groups_are_the_data_templates_of_a_page_that_loaded() {
    let theme = Theme::new(None, Some(WITH_DATA));

    assert_eq!(wanted_groups(&theme.0, 200), [DataGroup::Date, DataGroup::Memory]);
    assert!(
        wanted_groups(&theme.0, 404).is_empty(),
        "a page that did not load wants nothing"
    );
    assert!(
        wanted_groups(
            &Theme::new(None, Some(COMPATIBLE.replace("SystemMemory", "Nope").as_str())).0,
            200
        )
        .is_empty()
    );
    assert!(
        wanted_groups(&Theme::new(Some(TWO_PROPERTIES), None).0, 200).is_empty(),
        "no file, no data"
    );
    assert!(
        wanted_groups(&Theme::new(None, Some("{ nope")).0, 200).is_empty(),
        "an unusable file wants nothing"
    );
}

#[test]
fn a_data_push_runs_every_template_in_the_legacy_order_with_the_newest_snapshots() {
    let theme = Theme::new(None, Some(WITH_DATA));
    let state = fresh_state();
    state.store_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);
    state.store_data(vec![(DataGroup::Date, r#"{"Day":2}"#.to_owned())]);

    let templates = load_templates(&theme.0);
    let calls = push_calls(&templates, &state);

    assert_eq!(
        keys(&calls),
        ["compatible:LoopMode", "compatible:SystemDate"],
        "Memory has no snapshot yet"
    );
    assert_eq!(calls[0].script, "L('True');");
    assert_eq!(
        calls[1].script, "D({\r\n  \"Day\": 2\r\n});",
        "the newest snapshot of the group"
    );
}

#[test]
fn a_snapshot_that_is_not_json_is_dropped_and_the_group_keeps_its_last_value() {
    let state = fresh_state();
    state.store_data(vec![(DataGroup::Date, r#"{"Day":1}"#.to_owned())]);

    state.store_data(vec![(DataGroup::Date, "not json".to_owned())]);

    assert_eq!(state.data(), [(DataGroup::Date, serde_json::json!({"Day": 1}))]);
}

#[test]
fn a_new_shuffle_value_reaches_the_next_compatible_push() {
    let theme = Theme::new(None, Some(r#"{"ShuffleMode":"S('{0}');"}"#));
    let state = fresh_state();
    let templates = load_templates(&theme.0);
    assert_eq!(push_calls(&templates, &state)[0].script, "S('True');");

    state.set_shuffle(false);

    assert_eq!(push_calls(&templates, &state)[0].script, "S('False');");
}

#[test]
fn the_calls_fixture_gets_its_property_calls_in_file_order_then_the_non_data_compatible_calls_in_the_legacy_order() {
    let theme = calls_fixture();
    let state = fresh_state();

    let calls = page_calls(PageKind::Theme, &theme, 200, &state, &Complaints::default());

    let property: Vec<&str> = calls
        .iter()
        .filter(|call| call.kind == KIND_PROPERTY)
        .map(|call| call.key.as_str())
        .collect();
    assert_eq!(
        property,
        [
            "title", "press", "amount", "name", "enabled", "mode", "count", "secret", "picture", "tint"
        ],
        "file order, not alphabetical"
    );
    let first_compatible = calls
        .iter()
        .position(|call| call.kind == KIND_COMPATIBLE)
        .expect("the fixture has compatible calls");
    assert_eq!(first_compatible, property.len(), "every property call comes first");
    let compatible: Vec<&str> = calls[first_compatible..].iter().map(|call| call.key.as_str()).collect();
    let expected: Vec<String> = LEGACY_RUN_ORDER
        .iter()
        .filter(|key| {
            matches!(
                key,
                CompatibleKey::LoopMode
                    | CompatibleKey::ThemeType
                    | CompatibleKey::SystemTheme
                    | CompatibleKey::ShuffleMode
                    | CompatibleKey::StretchMode
                    | CompatibleKey::VolumeLevel
            )
        })
        .map(|key| format!("{key:?}"))
        .collect();
    assert_eq!(compatible, expected);
    assert!(
        calls.iter().all(|call| call.kind != js::KIND_PAUSE),
        "not paused, no pause call"
    );
}

#[test]
fn the_calls_fixture_wants_its_five_data_groups_in_the_legacy_payload_order() {
    assert_eq!(
        wanted_groups(&calls_fixture(), 200),
        [
            DataGroup::Bios,
            DataGroup::Date,
            DataGroup::Memory,
            DataGroup::Processor,
            DataGroup::Motherboard
        ]
    );
    assert!(wanted_groups(&calls_fixture(), 404).is_empty());
    assert!(wanted_groups(&calls_fixture(), 0).is_empty());
}

#[test]
fn a_page_without_a_compatible_file_wants_no_data() {
    let theme = Theme::new(Some(TWO_PROPERTIES), None);

    assert!(wanted_groups(&theme.0, 200).is_empty());
}

#[test]
fn a_url_page_gets_no_property_compatible_or_data_calls_even_beside_theme_files() {
    let theme = Theme::new(Some(TWO_PROPERTIES), Some(WITH_DATA));
    let state = fresh_state();

    assert!(page_calls(PageKind::Url, &theme.0, 200, &state, &Complaints::default()).is_empty());

    state.set_paused(true);
    let paused = page_calls(PageKind::Url, &theme.0, 200, &state, &Complaints::default());
    assert_eq!(
        paused.len(),
        1,
        "only the Light pause of a page paused before it loaded"
    );
    assert_eq!(paused[0].kind, js::KIND_PAUSE);
    assert!(page_calls(PageKind::Url, &theme.0, 404, &state, &Complaints::default()).is_empty());
}

#[test]
fn the_page_kind_of_a_wallpaper_decides_its_pages_and_whether_it_takes_input() {
    use sucrose_core::theme::WallpaperType;
    use sucrose_engine_web::args::WebEngine;

    for engine in [WebEngine::Cef, WebEngine::WebView2] {
        let kind = |wallpaper| PageKind::for_wallpaper(wallpaper, engine);
        assert!(kind(WallpaperType::Web).is_theme());
        assert!(matches!(kind(WallpaperType::Url), PageKind::Url));
        assert!(kind(WallpaperType::Gif).generated().is_some());
        assert!(kind(WallpaperType::Video).generated().is_some());
        // Legacy: input only for Web, Url and Application; the generated Gif and Video pages get none.
        assert!(kind(WallpaperType::Web).takes_input());
        assert!(kind(WallpaperType::Url).takes_input());
        assert!(!kind(WallpaperType::Gif).takes_input());
        assert!(!kind(WallpaperType::Video).takes_input());
        assert!(
            !kind(WallpaperType::Url).is_theme(),
            "a Url page has no theme files to read"
        );
    }
}
