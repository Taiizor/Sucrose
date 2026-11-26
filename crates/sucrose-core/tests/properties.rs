//! SucroseProperties.json read and write against the maintainer's real files, the legacy showcase files and
//! hand-made cases (FMT-03).

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sucrose_core::properties::{
    Control, ControlKind, PropertiesError, SucroseProperties, parse_properties, read_properties, write_properties,
};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/properties")
}

fn fixture(name: &str) -> PathBuf {
    dir().join(name)
}

fn fixtures_ending(suffix: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir())
        .expect("fixture folder is readable")
        .map(|entry| entry.expect("entry is readable").path())
        .filter(|path| path.to_string_lossy().ends_with(suffix))
        .collect();
    found.sort();
    found
}

/// A file under the system temp folder, removed again when it goes out of scope.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(std::env::temp_dir().join(format!("sucrose-properties-test-{}-{name}", std::process::id())))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn round_trip(path: &Path) -> (SucroseProperties, SucroseProperties, String) {
    let first = read_properties(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let scratch = Scratch::new(&path.file_name().expect("file name").to_string_lossy());
    write_properties(&scratch.0, &first).expect("write succeeds");
    let written = fs::read_to_string(&scratch.0).expect("written file is readable");
    let second = read_properties(&scratch.0).expect("written file parses");
    (first, second, written)
}

fn keys(properties: &SucroseProperties) -> Vec<&str> {
    properties.list.iter().map(|(key, _)| key.as_str()).collect()
}

fn control<'a>(properties: &'a SucroseProperties, key: &str) -> &'a Control {
    &properties
        .list
        .iter()
        .find(|(name, _)| name == key)
        .unwrap_or_else(|| panic!("control {key} exists"))
        .1
}

#[test]
fn every_real_and_showcase_file_round_trips_with_its_key_order() {
    let real = fixtures_ending(".real.json");
    let showcase = fixtures_ending(".showcase.json");
    assert_eq!(real.len(), 7, "the maintainer's library has 7 property files");
    assert_eq!(showcase.len(), 13, "the legacy showcase has 13 property files");
    for path in real.iter().chain(&showcase) {
        let (first, second, written) = round_trip(path);
        assert!(!first.listener.is_empty(), "{} has a listener", path.display());
        assert!(!first.list.is_empty(), "{} has controls", path.display());
        assert_eq!(first, second, "{} changed in the round trip", path.display());
        assert_eq!(keys(&first), keys(&second), "{} reordered its list", path.display());
        assert!(
            first
                .list
                .iter()
                .all(|(_, control)| !matches!(control.kind, ControlKind::Unknown { .. })),
            "{} holds a control type this build does not know",
            path.display()
        );
        assert!(
            written.contains("\r\n  \"PropertyListener\""),
            "two-space indent and CRLF"
        );
        assert!(!written.replace("\r\n", "").contains('\n'), "no bare line feed");
        assert!(!written.ends_with('\n'), "no trailing newline");
    }
}

#[test]
fn the_list_keeps_the_order_of_the_file_not_alphabetical_order() {
    let properties = read_properties(&fixture("all-types.json")).expect("fixture parses");
    assert_eq!(
        keys(&properties),
        [
            "zeta", "alpha", "button1", "name", "enabled", "mode", "amount", "secret", "picture", "tint"
        ]
    );
    let (_, second, _) = round_trip(&fixture("all-types.json"));
    assert_eq!(keys(&second), keys(&properties));
    let cultures: Vec<&str> = properties
        .localization
        .iter()
        .map(|(culture, _)| culture.as_str())
        .collect();
    assert_eq!(cultures, ["EN", "TR"]);
    assert_eq!(
        properties.localization[0].1[0],
        ("loc_alpha".to_owned(), "Alpha".to_owned())
    );
}

#[test]
fn all_ten_control_types_read_their_fields() {
    let properties = read_properties(&fixture("all-types.json")).expect("fixture parses");
    assert_eq!(properties.listener, "SucrosePropertyListener('{0}', {1});");
    assert_eq!(
        control(&properties, "zeta").kind,
        ControlKind::Label {
            value: "loc_zeta".to_owned()
        }
    );
    assert_eq!(control(&properties, "zeta").text, None);
    let alpha = control(&properties, "alpha");
    assert_eq!(alpha.name, "alpha");
    assert_eq!(alpha.text.as_deref(), Some("loc_alpha"));
    assert_eq!(alpha.help.as_deref(), Some("loc_alpha_help"));
    assert_eq!(
        alpha.kind,
        ControlKind::Slider {
            min: -5.0,
            max: 5.0,
            step: 0.5,
            value: 0.0
        },
        "integers are accepted where doubles are expected"
    );
    assert_eq!(
        control(&properties, "button1").kind,
        ControlKind::Button { value: "Go".to_owned() }
    );
    assert_eq!(
        control(&properties, "name").kind,
        ControlKind::Textbox {
            value: "Sucrose".to_owned()
        }
    );
    assert_eq!(
        control(&properties, "enabled").kind,
        ControlKind::Checkbox { value: true }
    );
    assert_eq!(
        control(&properties, "mode").kind,
        ControlKind::Dropdown {
            value: 1,
            items: vec!["Flat".to_owned(), "Vertical".to_owned(), "Horizontal".to_owned()],
            valuetext: "Vertical".to_owned(),
        },
        "valuetext is the chosen item, whatever the file says"
    );
    assert_eq!(
        control(&properties, "amount").kind,
        ControlKind::Numberbox {
            min: 0.0,
            max: 10.0,
            places: 2,
            value: 2.5
        }
    );
    assert_eq!(
        control(&properties, "secret").kind,
        ControlKind::Passwordbox {
            value: "hunter2".to_owned()
        }
    );
    assert_eq!(
        control(&properties, "picture").kind,
        ControlKind::Filedropdown {
            desc: "Images".to_owned(),
            value: "a.png".to_owned(),
            title: "Pick".to_owned(),
            folder: "assets/img".to_owned(),
            filter: "*.png|*.jpg".to_owned(),
            files: Some(vec!["a.png".to_owned(), "b.jpg".to_owned()]),
        }
    );
    let tint = control(&properties, "tint");
    assert_eq!(
        tint.kind,
        ControlKind::Colorpicker {
            value: "#FF00FF00".to_owned()
        }
    );
    assert_eq!(
        tint.extra.get("unit"),
        Some(&json!("argb")),
        "an unknown field of a known type is kept"
    );
    assert_eq!(
        properties.extra.get("Note"),
        Some(&json!("kept")),
        "an unknown top-level key is kept"
    );
}

#[test]
fn page_call_json_follows_the_legacy_serialization() {
    let properties = read_properties(&fixture("all-types.json")).expect("fixture parses");
    assert_eq!(
        control(&properties, "alpha").to_json_value(),
        json!({"type": "slider", "text": "loc_alpha", "help": "loc_alpha_help", "min": -5.0, "max": 5.0, "step": 0.5, "value": 0.0})
    );
    assert_eq!(
        control(&properties, "zeta").to_json_value(),
        json!({"type": "label", "text": "", "help": null, "value": "loc_zeta"}),
        "a label without text gets an empty text"
    );
    let mode = control(&properties, "mode").to_json_value();
    assert_eq!(mode["valuetext"], json!("Vertical"));
    assert_eq!(mode["items"], json!(["Flat", "Vertical", "Horizontal"]));
    assert!(
        control(&properties, "tint").to_json_value().get("unit").is_none(),
        "unknown fields are not sent"
    );
    let picture = control(&properties, "picture").to_json_value();
    assert_eq!(picture["files"], json!(["a.png", "b.jpg"]));
}

#[test]
fn an_unknown_type_and_unknown_fields_survive_a_write() {
    let (first, second, written) = round_trip(&fixture("unknown-type.json"));
    assert_eq!(first, second);
    let odd = control(&first, "odd");
    match &odd.kind {
        ControlKind::Unknown { type_name, raw } => {
            assert_eq!(type_name, "spinner");
            assert_eq!(raw.get("speed"), Some(&json!({"fast": true})));
            assert_eq!(raw.get("labels"), Some(&json!(["a", "b"])));
        }
        other => panic!("expected an unknown control, got {other:?}"),
    }
    let value: Value = serde_json::from_str(&written).expect("written file is JSON");
    assert_eq!(value["PropertyList"]["odd"]["type"], json!("spinner"));
    assert_eq!(value["PropertyList"]["odd"]["speed"], json!({"fast": true}));
    assert_eq!(value["Extra"], json!({"keep": [1, 2, 3]}));
}

#[test]
fn an_empty_list_reads_as_an_empty_list() {
    let properties = read_properties(&fixture("empty-list.json")).expect("fixture parses");
    assert!(properties.list.is_empty());
    assert!(properties.localization.is_empty());
    let (first, second, _) = round_trip(&fixture("empty-list.json"));
    assert_eq!(first, second);
}

#[test]
fn a_file_that_is_not_json_is_an_error_and_stays_untouched() {
    let scratch = Scratch::new("not-json");
    fs::copy(fixture("not-json.json"), &scratch.0).expect("copy fixture");
    let before = fs::read(&scratch.0).expect("read bytes");
    match read_properties(&scratch.0) {
        Err(PropertiesError::Invalid { reason, .. }) => assert!(!reason.is_empty()),
        other => panic!("expected an Invalid error, got {other:?}"),
    }
    assert_eq!(
        fs::read(&scratch.0).expect("file still exists"),
        before,
        "reading must not change the file"
    );
}

#[test]
fn a_missing_file_is_reported_as_missing() {
    assert!(matches!(
        read_properties(&fixture("does-not-exist.json")),
        Err(PropertiesError::Missing(_))
    ));
}

#[test]
fn malformed_controls_give_reasons_not_panics() {
    for (text, expect) in [
        (r#"{"PropertyList": {"a": 5}}"#, "is not an object"),
        (r#"{"PropertyList": {"a": {"text": "x"}}}"#, "has no type"),
        (
            r#"{"PropertyList": {"a": {"type": "slider", "min": []}}}"#,
            "is not a number",
        ),
        (
            r#"{"PropertyList": {"a": {"type": "dropdown", "value": 1.5}}}"#,
            "is not a whole number",
        ),
        (r#"{"PropertyList": [1]}"#, "wrong type"),
        ("[]", "wrong type"),
        ("", "too early"),
    ] {
        let reason = parse_properties(text).expect_err("must be rejected");
        assert!(reason.contains(expect), "{text}: {reason}");
    }
}

#[test]
fn keys_match_in_any_letter_case_like_newtonsoft() {
    let properties = parse_properties(
        r#"{"propertylistener": "L", "PROPERTYLIST": {"a": {"Type": "Slider", "Min": 1, "Max": 2, "Step": 1, "Value": 1}}}"#,
    )
    .expect("parses");
    assert_eq!(properties.listener, "L");
    assert_eq!(
        control(&properties, "a").kind,
        ControlKind::Slider {
            min: 1.0,
            max: 2.0,
            step: 1.0,
            value: 1.0
        }
    );
}

#[test]
fn a_repeated_key_keeps_its_first_place_and_last_value() {
    let properties = parse_properties(
        r#"{"PropertyList": {"a": {"type": "checkbox", "value": false}, "b": {"type": "checkbox", "value": false}, "a": {"type": "checkbox", "value": true}}}"#,
    )
    .expect("parses");
    assert_eq!(keys(&properties), ["a", "b"]);
    assert_eq!(control(&properties, "a").kind, ControlKind::Checkbox { value: true });
}
