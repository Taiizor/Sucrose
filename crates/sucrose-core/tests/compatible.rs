//! SucroseCompatible.json read and write against the maintainer's real file, the legacy showcase files and
//! hand-made cases (FMT-03).

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sucrose_core::compatible::{
    CompatibleKey, DataGroup, SucroseCompatible, parse_compatible, read_compatible, write_compatible,
};
use sucrose_core::properties::PropertiesError;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compatible")
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
        Self(std::env::temp_dir().join(format!("sucrose-compatible-test-{}-{name}", std::process::id())))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn round_trip(path: &Path) -> (SucroseCompatible, SucroseCompatible, String) {
    let first = read_compatible(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let scratch = Scratch::new(&path.file_name().expect("file name").to_string_lossy());
    write_compatible(&scratch.0, &first).expect("write succeeds");
    let written = fs::read_to_string(&scratch.0).expect("written file is readable");
    let second = read_compatible(&scratch.0).expect("written file parses");
    (first, second, written)
}

#[test]
fn the_real_file_and_every_showcase_file_round_trip() {
    let real = fixtures_ending(".real.json");
    let showcase = fixtures_ending(".showcase.json");
    assert_eq!(real.len(), 1, "the maintainer's library has 1 compatible file");
    assert_eq!(showcase.len(), 10, "the legacy showcase has 10 compatible files");
    for path in real.iter().chain(&showcase) {
        let (first, second, written) = round_trip(path);
        assert_eq!(first, second, "{} changed in the round trip", path.display());
        assert!(!first.templates().is_empty(), "{} has templates", path.display());
        for (key, template) in first.templates() {
            assert!(
                template.contains("{0}"),
                "{}: {} has no {{0}}",
                path.display(),
                key.name()
            );
        }
        assert_eq!(
            first.unknown_entries().count(),
            0,
            "{} has a key this build does not know",
            path.display()
        );
        assert!(!written.replace("\r\n", "").contains('\n'), "no bare line feed");
        assert!(!written.ends_with('\n'), "no trailing newline");
        if !first.templates().is_empty() {
            assert!(written.starts_with("{\r\n  \""), "two-space indent and CRLF");
        }
    }
}

#[test]
fn the_real_file_defines_all_sixteen_keys_in_table_order() {
    let compatible = read_compatible(&fixture("1.real.json")).expect("fixture parses");
    let keys: Vec<CompatibleKey> = compatible.templates().into_iter().map(|(key, _)| key).collect();
    assert_eq!(keys, CompatibleKey::ALL);
    assert_eq!(
        compatible.get(CompatibleKey::SystemMemory),
        "Sucrose('SystemMemory', {0});"
    );
    assert_eq!(compatible.data_groups(), DataGroup::ALL);
}

#[test]
fn templates_come_in_table_order_whatever_the_file_order() {
    let compatible = parse_compatible(
        r#"{"SystemMotherboard": "M({0});", "StretchMode": "S('{0}');", "LoopMode": "", "ThemeType": "T('{0}');"}"#,
    )
    .expect("parses");
    assert_eq!(
        compatible.templates(),
        [
            (CompatibleKey::ThemeType, "T('{0}');"),
            (CompatibleKey::StretchMode, "S('{0}');"),
            (CompatibleKey::SystemMotherboard, "M({0});"),
        ],
        "empty keys are left out"
    );
    assert_eq!(compatible.get(CompatibleKey::LoopMode), "");
    assert_eq!(compatible.get(CompatibleKey::SystemAudio), "", "a missing key is empty");
}

#[test]
fn data_groups_name_what_the_data_hub_samples() {
    let two = parse_compatible(r#"{"SystemMemory": "M({0});", "SystemDate": "D({0});"}"#).expect("parses");
    assert_eq!(two.data_groups(), [DataGroup::Date, DataGroup::Memory]);
    let none = parse_compatible(r#"{"LoopMode": "L('{0}');"}"#).expect("parses");
    assert_eq!(none.data_groups(), []);
    let hub_free = parse_compatible(
        r#"{"LoopMode": "a", "ThemeType": "b", "ShuffleMode": "c", "StretchMode": "d", "SystemTheme": "e", "VolumeLevel": "f"}"#,
    )
    .expect("parses");
    assert_eq!(hub_free.data_groups(), [], "the engine's own state needs no data group");
    for (key, group) in [
        (CompatibleKey::SystemBios, DataGroup::Bios),
        (CompatibleKey::SystemAudio, DataGroup::Audio),
        (CompatibleKey::SystemBattery, DataGroup::Battery),
        (CompatibleKey::SystemGraphic, DataGroup::Graphic),
        (CompatibleKey::SystemNetwork, DataGroup::Network),
        (CompatibleKey::SystemStorage, DataGroup::Storage),
        (CompatibleKey::SystemProcessor, DataGroup::Processor),
    ] {
        assert_eq!(key.data_group(), Some(group));
    }
}

#[test]
fn unknown_keys_survive_in_file_order_and_known_keys_keep_their_place() {
    let (first, second, written) = round_trip(&fixture("unknown-key.json"));
    assert_eq!(first, second);
    let unknown: Vec<(&str, &Value)> = first.unknown_entries().collect();
    assert_eq!(unknown, [("Custom", &json!({"a": 1}))]);
    // "loopmode" matched in another letter case and is written under its legacy spelling, at its place.
    assert_eq!(first.get(CompatibleKey::LoopMode), "L('{0}');");
    let order: Vec<usize> = ["StretchMode", "Custom", "SystemDate", "LoopMode"]
        .iter()
        .map(|key| written.find(&format!("\"{key}\"")).expect("key is written"))
        .collect();
    assert!(
        order.windows(2).all(|pair| pair[0] < pair[1]),
        "file order kept: {order:?}"
    );
}

#[test]
fn an_empty_object_reads_and_writes_as_an_empty_object() {
    let (first, second, written) = round_trip(&fixture("empty.json"));
    assert_eq!(first, second);
    assert_eq!(first, SucroseCompatible::new());
    assert!(first.templates().is_empty());
    assert_eq!(written, "{}");
}

#[test]
fn set_replaces_in_place_or_appends() {
    let mut compatible = parse_compatible(r#"{"StretchMode": "a", "Custom": 1}"#).expect("parses");
    compatible.set(CompatibleKey::StretchMode, "b");
    compatible.set(CompatibleKey::SystemDate, "D({0});");
    assert_eq!(compatible.get(CompatibleKey::StretchMode), "b");
    let scratch = Scratch::new("set");
    write_compatible(&scratch.0, &compatible).expect("write succeeds");
    let written = fs::read_to_string(&scratch.0).expect("readable");
    let value: Value = serde_json::from_str(&written).expect("JSON");
    assert_eq!(value, json!({"StretchMode": "b", "Custom": 1, "SystemDate": "D({0});"}));
}

#[test]
fn bad_files_give_reasons_and_leave_the_file_alone() {
    for (text, expect) in [
        ("[]", "wrong type"),
        ("", "too early"),
        ("{ nope", "invalid JSON"),
        (r#"{"LoopMode": 5}"#, "LoopMode"),
    ] {
        let reason = parse_compatible(text).expect_err("must be rejected");
        assert!(reason.contains(expect), "{text}: {reason}");
    }
    let scratch = Scratch::new("bad");
    fs::write(&scratch.0, b"{ nope").expect("write scratch");
    let before = fs::read(&scratch.0).expect("read bytes");
    assert!(matches!(
        read_compatible(&scratch.0),
        Err(PropertiesError::Invalid { .. })
    ));
    assert_eq!(
        fs::read(&scratch.0).expect("file still exists"),
        before,
        "reading must not change the file"
    );
    assert!(matches!(
        read_compatible(&fixture("missing.json")),
        Err(PropertiesError::Missing(_))
    ));
}

#[test]
fn null_reads_as_empty_and_is_written_back_as_null() {
    let compatible = parse_compatible(r#"{"LoopMode": null}"#).expect("parses");
    assert_eq!(compatible.get(CompatibleKey::LoopMode), "");
    assert!(compatible.templates().is_empty());
    let scratch = Scratch::new("null");
    write_compatible(&scratch.0, &compatible).expect("write succeeds");
    let value: Value = serde_json::from_str(&fs::read_to_string(&scratch.0).expect("readable")).expect("JSON");
    assert_eq!(value, json!({"LoopMode": null}));
}
