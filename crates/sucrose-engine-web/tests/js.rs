//! The legacy-exact script strings of the wallpaper JavaScript API: C# composite formatting, Newtonsoft's indented JSON,
//! property calls for every control type and the filedropdown listing with its containment rule.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use serde_json::json;
use sucrose_core::{compatible::CompatibleKey, settings::Stretch};
use sucrose_engine_web::js::{
    CompatibleState, CompatibleValue, FormatError, KIND_COMPATIBLE, KIND_PAUSE, KIND_PROPERTY, LEGACY_RUN_ORDER,
    compatible_call, compatible_calls, csharp_format, filedropdown_files, indented_json, non_data_calls, pause_call,
    pause_script, property_call, property_calls, try_csharp_format,
};

/// A scratch theme folder under the temp folder, removed on drop.
struct Theme(PathBuf);

impl Theme {
    fn new(name: &str) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-js-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch theme");
        Self(dir)
    }

    fn file(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create folder");
        fs::write(path, body).expect("write file");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The text with CRLF line ends, as Newtonsoft writes it.
fn crlf(text: &str) -> String {
    text.replace('\n', "\r\n")
}

fn all_types() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../sucrose-core/tests/fixtures/properties/all-types.json"),
    )
    .expect("the all-types fixture of sucrose-core")
}

#[test]
fn format_replaces_placeholders_and_keeps_the_legacy_listener_text() {
    assert_eq!(
        csharp_format("SucrosePropertyListener('{0}', {1});", &["volume", "{...}"]),
        "SucrosePropertyListener('volume', {...});"
    );
    assert_eq!(
        csharp_format("{1}-{0}-{1}", &["a", "b"]),
        "b-a-b",
        "an index may repeat or reorder"
    );
    assert_eq!(csharp_format("no placeholder", &["a"]), "no placeholder");
}

#[test]
fn format_turns_doubled_braces_into_single_ones() {
    assert_eq!(csharp_format("{{x}}", &[]), "{x}");
    assert_eq!(csharp_format("{{{0}}}", &["v"]), "{v}");
    assert_eq!(csharp_format("a {{ b }} c", &[]), "a { b } c");
}

#[test]
fn format_supports_alignment_and_ignores_a_format_part_for_text() {
    assert_eq!(csharp_format("[{0,5}]", &["ab"]), "[   ab]");
    assert_eq!(csharp_format("[{0,-5}]", &["ab"]), "[ab   ]");
    assert_eq!(csharp_format("[{0:N2}]", &["ab"]), "[ab]");
    assert_eq!(csharp_format("[{0, 3:x}]", &["ab"]), "[ ab]");
}

#[test]
fn format_errors_are_the_cases_where_dotnet_throws_and_the_call_is_skipped() {
    assert_eq!(try_csharp_format("{0", &["a"]), Err(FormatError::Brace));
    assert_eq!(try_csharp_format("a}", &["a"]), Err(FormatError::Brace));
    assert_eq!(try_csharp_format("{}", &["a"]), Err(FormatError::Placeholder));
    assert_eq!(try_csharp_format("{x}", &["a"]), Err(FormatError::Placeholder));
    assert_eq!(try_csharp_format("{0{}", &["a"]), Err(FormatError::Placeholder));
    assert_eq!(try_csharp_format("{2}", &["a", "b"]), Err(FormatError::Index(2)));
    assert_eq!(csharp_format("{2}", &["a", "b"]), "", "a skipped call has no script");
}

#[test]
fn indented_json_uses_two_spaces_crlf_and_the_key_order_of_the_input() {
    let value = json!({"type": "slider", "value": 5.0, "min": 0.0});

    assert_eq!(
        indented_json(&value),
        "{\r\n  \"type\": \"slider\",\r\n  \"value\": 5.0,\r\n  \"min\": 0.0\r\n}"
    );
}

#[test]
fn indented_json_writes_nested_lists_and_empty_containers_like_newtonsoft() {
    let value = json!({"items": ["a", "b"], "none": [], "empty": {}, "flag": true, "gone": null});

    assert_eq!(
        indented_json(&value),
        crlf(
            "{\n  \"items\": [\n    \"a\",\n    \"b\"\n  ],\n  \"none\": [],\n  \"empty\": {},\n  \"flag\": true,\n  \"gone\": null\n}"
        )
    );
}

#[test]
fn indented_json_escapes_line_breaks_inside_strings_so_only_structure_gets_crlf() {
    let text = indented_json(&json!({"text": "a\nb"}));

    assert_eq!(text, "{\r\n  \"text\": \"a\\nb\"\r\n}");
}

#[test]
fn property_call_for_each_of_the_ten_control_types_matches_the_stored_text() {
    let theme = Theme::new("types");
    theme.file("SucroseProperties.json", &all_types());
    theme.file("assets/img/b.jpg", "");
    theme.file("assets/img/A.png", "");
    theme.file("assets/img/notes.txt", "");
    fs::create_dir_all(theme.path().join("assets/img/inner.png")).expect("a folder is not a file");

    let calls = property_calls(theme.path()).expect("the fixture is valid");

    let expected: [(&str, &str); 10] = [
        (
            "zeta",
            "SucrosePropertyListener('zeta', {\n  \"type\": \"label\",\n  \"text\": \"\",\n  \"help\": null,\n  \"value\": \"loc_zeta\"\n});",
        ),
        (
            "alpha",
            "SucrosePropertyListener('alpha', {\n  \"type\": \"slider\",\n  \"text\": \"loc_alpha\",\n  \"help\": \"loc_alpha_help\",\n  \"min\": -5.0,\n  \"max\": 5.0,\n  \"step\": 0.5,\n  \"value\": 0.0\n});",
        ),
        (
            "button1",
            "SucrosePropertyListener('button1', {\n  \"type\": \"button\",\n  \"text\": \"loc_button\",\n  \"help\": null,\n  \"value\": \"Go\"\n});",
        ),
        (
            "name",
            "SucrosePropertyListener('name', {\n  \"type\": \"textbox\",\n  \"text\": \"loc_name\",\n  \"help\": null,\n  \"value\": \"Sucrose\"\n});",
        ),
        (
            "enabled",
            "SucrosePropertyListener('enabled', {\n  \"type\": \"checkbox\",\n  \"text\": \"loc_enabled\",\n  \"help\": null,\n  \"value\": true\n});",
        ),
        (
            "mode",
            "SucrosePropertyListener('mode', {\n  \"type\": \"dropdown\",\n  \"text\": \"loc_mode\",\n  \"help\": null,\n  \"value\": 1,\n  \"items\": [\n    \"Flat\",\n    \"Vertical\",\n    \"Horizontal\"\n  ],\n  \"valuetext\": \"Vertical\"\n});",
        ),
        (
            "amount",
            "SucrosePropertyListener('amount', {\n  \"type\": \"numberbox\",\n  \"text\": \"loc_amount\",\n  \"help\": null,\n  \"min\": 0.0,\n  \"max\": 10.0,\n  \"places\": 2,\n  \"value\": 2.5\n});",
        ),
        (
            "secret",
            "SucrosePropertyListener('secret', {\n  \"type\": \"passwordbox\",\n  \"text\": \"loc_secret\",\n  \"help\": null,\n  \"value\": \"hunter2\"\n});",
        ),
        (
            "picture",
            "SucrosePropertyListener('picture', {\n  \"type\": \"filedropdown\",\n  \"text\": \"loc_picture\",\n  \"help\": null,\n  \"desc\": \"Images\",\n  \"value\": \"a.png\",\n  \"title\": \"Pick\",\n  \"folder\": \"assets/img\",\n  \"filter\": \"*.png|*.jpg\",\n  \"files\": [\n    \"A.png\",\n    \"b.jpg\"\n  ]\n});",
        ),
        (
            "tint",
            "SucrosePropertyListener('tint', {\n  \"type\": \"colorpicker\",\n  \"text\": \"loc_tint\",\n  \"help\": null,\n  \"value\": \"#FF00FF00\"\n});",
        ),
    ];
    assert_eq!(calls.len(), 10);
    for (call, (key, text)) in calls.iter().zip(expected) {
        assert_eq!(call.kind, KIND_PROPERTY);
        assert_eq!(call.key, key, "file order, not alphabetical order");
        assert_eq!(call.script, crlf(text), "{key}");
    }
}

#[test]
fn property_call_takes_the_listener_of_the_theme_whatever_it_looks_like() {
    let control = json!({"type": "textbox", "value": "x"});

    assert_eq!(
        property_call("window.on('{0}', {1})", "k", &control),
        crlf("window.on('k', {\n  \"type\": \"textbox\",\n  \"value\": \"x\"\n})")
    );
    assert_eq!(property_call("only('{0}')", "k", &control), "only('k')");
    assert_eq!(
        property_call("{0", "k", &control),
        "",
        "an unformattable listener runs nothing"
    );
}

#[test]
fn a_theme_without_properties_or_with_an_empty_list_gets_no_calls() {
    let missing = Theme::new("missing");
    let empty = Theme::new("empty");
    empty.file(
        "SucroseProperties.json",
        r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{}}"#,
    );

    assert_eq!(property_calls(missing.path()).expect("not an error"), []);
    assert_eq!(property_calls(empty.path()).expect("not an error"), []);
}

#[test]
fn an_unparsable_properties_file_is_an_error_and_stays_untouched() {
    let theme = Theme::new("broken");
    theme.file("SucroseProperties.json", "{ not json");

    assert!(property_calls(theme.path()).is_err());
    assert_eq!(
        fs::read_to_string(theme.path().join("SucroseProperties.json")).expect("still there"),
        "{ not json",
        "reading never changes the theme (D-017)"
    );
}

#[test]
fn an_empty_listener_runs_no_calls() {
    let theme = Theme::new("silent");
    theme.file(
        "SucroseProperties.json",
        r#"{"PropertyListener":"","PropertyList":{"a":{"type":"checkbox","value":true}}}"#,
    );

    assert_eq!(property_calls(theme.path()).expect("valid"), []);
}

#[test]
fn a_filedropdown_with_no_folder_or_filter_gets_null_files_like_the_legacy_listing() {
    let theme = Theme::new("blank");
    theme.file(
        "SucroseProperties.json",
        r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{"pick":{"type":"filedropdown","folder":"","filter":"*","files":["stale"]}}}"#,
    );

    let calls = property_calls(theme.path()).expect("valid");

    assert!(calls[0].script.contains("\"files\": null"), "{}", calls[0].script);
    assert!(!calls[0].script.contains("stale"));
}

#[test]
fn filedropdown_lists_the_matching_files_of_the_folder_sorted_ignoring_case() {
    let theme = Theme::new("list");
    for name in ["two.png", "one.txt", "Three.PNG", "skip.jpg"] {
        theme.file(&format!("media/{name}"), "");
    }
    fs::create_dir_all(theme.path().join("media/folder.png")).expect("a folder");

    assert_eq!(
        filedropdown_files(theme.path(), "media", "*.txt|*.png"),
        ["one.txt", "Three.PNG", "two.png"]
    );
    assert_eq!(
        filedropdown_files(theme.path(), "media", " *.TXT | "),
        ["one.txt"],
        "blank parts are dropped"
    );
}

#[test]
fn filedropdown_star_and_star_dot_star_mean_every_file() {
    let theme = Theme::new("all");
    theme.file("media/a.txt", "");
    theme.file("media/noextension", "");
    fs::create_dir_all(theme.path().join("media/inner")).expect("a folder");

    assert_eq!(filedropdown_files(theme.path(), "media", "*"), ["a.txt", "noextension"]);
    assert_eq!(
        filedropdown_files(theme.path(), "media", "*.*"),
        ["a.txt", "noextension"]
    );
    assert_eq!(
        filedropdown_files(theme.path(), "media", "*.png|*"),
        ["a.txt", "noextension"]
    );
}

#[test]
fn filedropdown_folder_that_is_the_theme_itself_lists_the_theme_root() {
    let theme = Theme::new("root");
    theme.file("a.txt", "");
    theme.file("media/b.txt", "");

    assert_eq!(filedropdown_files(theme.path(), ".", "*.txt"), ["a.txt"]);
    assert_eq!(filedropdown_files(theme.path(), "media/..", "*.txt"), ["a.txt"]);
    assert_eq!(filedropdown_files(theme.path(), "media/../media", "*.txt"), ["b.txt"]);
}

#[test]
fn filedropdown_folder_outside_the_theme_or_missing_gives_an_empty_list() {
    let parent = Theme::new("outside");
    parent.file("secret/stolen.txt", "");
    parent.file("theme/inside.txt", "");
    let theme = parent.path().join("theme");

    assert_eq!(filedropdown_files(&theme, "../secret", "*"), Vec::<String>::new());
    assert_eq!(
        filedropdown_files(&theme, "media/../../secret", "*"),
        Vec::<String>::new()
    );
    let absolute = parent.path().join("secret");
    assert_eq!(
        filedropdown_files(&theme, absolute.to_str().expect("utf-8 temp path"), "*"),
        Vec::<String>::new(),
        "an absolute folder is never listed"
    );
    assert_eq!(filedropdown_files(&theme, "nothing-here", "*"), Vec::<String>::new());
    assert_eq!(
        filedropdown_files(&theme, "", "*"),
        ["inside.txt"],
        "an empty folder is the theme folder"
    );
}

#[cfg(unix)]
#[test]
fn filedropdown_folder_reached_through_a_link_that_leaves_the_theme_gives_an_empty_list() {
    let parent = Theme::new("link");
    parent.file("secret/stolen.txt", "");
    parent.file("theme/inside.txt", "");
    let theme = parent.path().join("theme");
    std::os::unix::fs::symlink(parent.path().join("secret"), theme.join("escape")).expect("symlink");

    assert_eq!(filedropdown_files(&theme, "escape", "*"), Vec::<String>::new());
}

#[test]
fn every_renderer_of_a_theme_gets_its_own_complete_sequence() {
    let theme = Theme::new("twice");
    theme.file("SucroseProperties.json", &all_types());

    let first = property_calls(theme.path()).expect("valid");
    let second = property_calls(theme.path()).expect("valid");

    assert_eq!(
        first, second,
        "reading is repeatable: a reload or a second renderer sends every key again"
    );
    assert_eq!(first.len(), 10);
}

fn state() -> CompatibleState {
    CompatibleState {
        looping: true,
        shuffle: false,
        stretch: Stretch::UniformToFill,
        theme_type: "Dark".to_owned(),
        volume: 40,
    }
}

#[test]
fn compatible_call_writes_bools_as_csharp_does_and_enums_by_name() {
    assert_eq!(
        compatible_call("SucroseLoopMode('{0}');", &CompatibleValue::Bool(true)),
        "SucroseLoopMode('True');"
    );
    assert_eq!(
        compatible_call("SucroseLoopMode('{0}');", &CompatibleValue::Bool(false)),
        "SucroseLoopMode('False');"
    );
    assert_eq!(
        compatible_call(
            "SucroseStretchMode('{0}');",
            &CompatibleValue::Name("UniformToFill".to_owned())
        ),
        "SucroseStretchMode('UniformToFill');"
    );
    assert_eq!(
        compatible_call("SucroseVolumeLevel({0});", &CompatibleValue::Int(40)),
        "SucroseVolumeLevel(40);"
    );
    assert_eq!(
        compatible_call(
            "SucroseMemoryData({0});",
            &CompatibleValue::Json(json!({"MemoryLoad": 41.5, "Name": "RAM"}))
        ),
        crlf("SucroseMemoryData({\n  \"MemoryLoad\": 41.5,\n  \"Name\": \"RAM\"\n});")
    );
    assert_eq!(
        compatible_call("{0", &CompatibleValue::Int(1)),
        "",
        "a template that cannot be formatted runs nothing"
    );
}

#[test]
fn the_non_data_calls_come_out_in_the_legacy_run_order_whatever_the_file_order() {
    let theme = Theme::new("order");
    theme.file(
        "SucroseCompatible.json",
        r#"{
  "VolumeLevel": "SucroseVolumeLevel({0});",
  "SystemMemory": "SucroseMemoryData({0});",
  "StretchMode": "SucroseStretchMode('{0}');",
  "ShuffleMode": "SucroseShuffleMode('{0}');",
  "SystemTheme": "SucroseSystemTheme('{0}');",
  "ThemeType": "SucroseThemeType('{0}');",
  "SystemDate": "SucroseDateData({0});",
  "LoopMode": "SucroseLoopMode('{0}');"
}"#,
    );

    let calls = compatible_calls(theme.path(), &state()).expect("valid");

    assert!(calls.iter().all(|call| call.kind == KIND_COMPATIBLE));
    assert_eq!(
        calls.iter().map(|call| call.key.as_str()).collect::<Vec<_>>(),
        [
            "LoopMode",
            "ThemeType",
            "SystemTheme",
            "ShuffleMode",
            "StretchMode",
            "VolumeLevel"
        ],
        "the data keys are left to the data pushes"
    );
    assert_eq!(
        calls.iter().map(|call| call.script.as_str()).collect::<Vec<_>>(),
        [
            "SucroseLoopMode('True');",
            "SucroseThemeType('Dark');",
            "SucroseSystemTheme('Dark');",
            "SucroseShuffleMode('False');",
            "SucroseStretchMode('UniformToFill');",
            "SucroseVolumeLevel(40);",
        ]
    );
}

#[test]
fn the_legacy_run_order_lists_each_key_once_and_puts_system_theme_before_shuffle() {
    let mut sorted: Vec<&str> = LEGACY_RUN_ORDER.iter().map(|key| key.name()).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let mut all: Vec<&str> = CompatibleKey::ALL.iter().map(|key| key.name()).collect();
    all.sort_unstable();

    assert_eq!(sorted, all, "the same sixteen keys as the table");
    let position = |wanted: CompatibleKey| LEGACY_RUN_ORDER.iter().position(|key| *key == wanted).expect("listed");
    assert!(position(CompatibleKey::SystemTheme) < position(CompatibleKey::ShuffleMode));
    assert!(position(CompatibleKey::StretchMode) < position(CompatibleKey::VolumeLevel));
}

#[test]
fn an_empty_or_unformattable_template_and_a_missing_file_give_no_call() {
    let theme = Theme::new("empty-template");
    theme.file(
        "SucroseCompatible.json",
        r#"{"LoopMode":"","ThemeType":null,"ShuffleMode":"{0","StretchMode":"S('{0}');"}"#,
    );
    let missing = Theme::new("no-compatible");

    let calls = compatible_calls(theme.path(), &state()).expect("valid");

    assert_eq!(
        calls.iter().map(|call| call.key.as_str()).collect::<Vec<_>>(),
        ["StretchMode"]
    );
    assert_eq!(compatible_calls(missing.path(), &state()).expect("not an error"), []);
}

#[test]
fn an_unparsable_compatible_file_is_an_error_and_stays_untouched() {
    let theme = Theme::new("broken-compatible");
    theme.file("SucroseCompatible.json", "[1, 2");

    assert!(compatible_calls(theme.path(), &state()).is_err());
    assert_eq!(
        fs::read_to_string(theme.path().join("SucroseCompatible.json")).expect("still there"),
        "[1, 2"
    );
}

#[test]
fn the_state_gives_each_non_data_key_its_value_and_leaves_the_data_keys_alone() {
    let mut state = state();
    state.stretch = Stretch::Fill;
    state.theme_type = "Light".to_owned();
    state.looping = false;
    state.shuffle = true;

    assert_eq!(state.value(CompatibleKey::LoopMode), Some(CompatibleValue::Bool(false)));
    assert_eq!(
        state.value(CompatibleKey::ShuffleMode),
        Some(CompatibleValue::Bool(true))
    );
    assert_eq!(
        state.value(CompatibleKey::StretchMode),
        Some(CompatibleValue::Name("Fill".to_owned()))
    );
    assert_eq!(
        state.value(CompatibleKey::ThemeType),
        Some(CompatibleValue::Name("Light".to_owned()))
    );
    assert_eq!(
        state.value(CompatibleKey::SystemTheme),
        Some(CompatibleValue::Name("Light".to_owned()))
    );
    assert_eq!(state.value(CompatibleKey::VolumeLevel), Some(CompatibleValue::Int(40)));
    for key in [
        CompatibleKey::SystemBios,
        CompatibleKey::SystemDate,
        CompatibleKey::SystemAudio,
        CompatibleKey::SystemMemory,
        CompatibleKey::SystemBattery,
        CompatibleKey::SystemGraphic,
        CompatibleKey::SystemNetwork,
        CompatibleKey::SystemStorage,
        CompatibleKey::SystemProcessor,
        CompatibleKey::SystemMotherboard,
    ] {
        assert_eq!(state.value(key), None, "{}", key.name());
    }
}

#[test]
fn every_stretch_has_its_legacy_name_in_the_call() {
    let template = [(CompatibleKey::StretchMode, "S('{0}');")];
    let names: Vec<String> = Stretch::ALL
        .iter()
        .map(|stretch| {
            let state = CompatibleState {
                stretch: *stretch,
                ..state()
            };
            non_data_calls(&template, &state).remove(0).script
        })
        .collect();

    assert_eq!(
        names,
        ["S('None');", "S('Fill');", "S('Uniform');", "S('UniformToFill');"]
    );
}

#[test]
fn the_pause_script_pauses_playing_media_hides_the_page_and_calls_sucrose_pause() {
    let script = pause_script(true);

    assert!(script.contains(".pause()"), "playing media pause");
    assert!(
        script.contains("element.paused"),
        "only media that plays is paused, and remembered"
    );
    assert!(
        script.contains("visibilityState") && script.contains("\"hidden\""),
        "the page is hidden"
    );
    assert!(script.contains("visibilitychange"), "the page is told");
    assert!(
        script.contains("typeof SucrosePause === \"function\""),
        "SucrosePause only when the page defines it"
    );
    assert!(!script.contains("SucrosePlay"));
}

#[test]
fn the_resume_script_restores_what_the_pause_changed_and_calls_sucrose_play() {
    let script = pause_script(false);

    assert!(script.contains(".play()"), "the remembered media play again");
    assert!(script.contains("delete document.visibilityState") && script.contains("delete document.hidden"));
    assert!(script.contains("visibilitychange"));
    assert!(
        script.contains("typeof SucrosePlay === \"function\""),
        "SucrosePlay only when the page defines it"
    );
    assert!(!script.contains("SucrosePause"));
    assert_ne!(pause_script(true), pause_script(false));
}

#[test]
fn the_pause_call_is_logged_as_pause_or_resume_and_gives_the_page_no_host_object() {
    let pause = pause_call(true);
    let resume = pause_call(false);

    assert_eq!((pause.kind, pause.key.as_str()), (KIND_PAUSE, "pause"));
    assert_eq!((resume.kind, resume.key.as_str()), (KIND_PAUSE, "resume"));
    for script in [pause.script, resume.script] {
        for forbidden in [
            "chrome.webview",
            "postMessage",
            "addHostObject",
            "cefQuery",
            "window.external",
        ] {
            assert!(!script.contains(forbidden), "{forbidden}");
        }
    }
}
