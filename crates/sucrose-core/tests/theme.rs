//! SucroseInfo.json round trip and the D-13 gate, against fixtures taken from real themes and temp folders.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sucrose_core::{
    theme::{SucroseInfo, ThemeRejection, WallpaperType, check_selected, parse_info, write_info},
    version::LegacyVersion,
};

const APP: LegacyVersion = LegacyVersion::new(0, 1, 0, -1);

fn fixture_text(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    fs::read_to_string(path).expect("fixture is readable")
}

fn fixture(name: &str) -> SucroseInfo {
    parse_info(&fixture_text(name)).expect("fixture parses")
}

/// A folder under the system temp folder, removed again when it goes out of scope.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sucrose-theme-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch folder");
        Self(dir)
    }

    /// A library with one theme folder `t` holding `info` and `video.mp4` plus a file and a folder next to it.
    fn library_with(&self, info: &SucroseInfo) -> &Path {
        let theme = self.0.join("t");
        fs::create_dir_all(theme.join("sub")).expect("create theme folder");
        fs::write(theme.join("video.mp4"), b"x").expect("write video");
        fs::write(theme.join("sub").join("deep.mp4"), b"x").expect("write nested video");
        fs::write(self.0.join("outside.mp4"), b"x").expect("write outside file");
        write_info(&theme.join("SucroseInfo.json"), info).expect("write info");
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn with_source(source: &str) -> SucroseInfo {
    SucroseInfo {
        source: source.to_owned(),
        ..fixture("SucroseInfo.video.json")
    }
}

fn rejection_for(name: &str, info: &SucroseInfo) -> ThemeRejection {
    let scratch = Scratch::new(name);
    let library = scratch.library_with(info);
    check_selected(library, "t", &APP).expect_err("the gate must refuse this theme")
}

#[test]
fn every_fixture_round_trips_to_an_equal_value() {
    for name in [
        "SucroseInfo.video.json",
        "SucroseInfo.full.json",
        "SucroseInfo.object-version.json",
    ] {
        let info = fixture(name);
        let scratch = Scratch::new("roundtrip");
        let path = scratch.0.join("SucroseInfo.json");
        write_info(&path, &info).expect("write");
        let back = parse_info(&fs::read_to_string(&path).expect("read back")).expect("parse back");
        assert_eq!(back, info, "{name}");
    }
}

#[test]
fn write_info_reproduces_the_real_theme_file_byte_for_byte() {
    let original = fixture_text("SucroseInfo.video.json")
        .replace("\r\n", "\n")
        .replace('\n', "\r\n");
    let scratch = Scratch::new("bytes");
    let path = scratch.0.join("SucroseInfo.json");

    write_info(&path, &fixture("SucroseInfo.video.json")).expect("write");

    assert_eq!(fs::read(&path).expect("read"), original.into_bytes());
}

#[test]
fn write_info_uses_two_spaces_crlf_and_no_bom() {
    let scratch = Scratch::new("format");
    let path = scratch.0.join("SucroseInfo.json");
    write_info(&path, &fixture("SucroseInfo.video.json")).expect("write");
    let bytes = fs::read(&path).expect("read");

    assert_eq!(&bytes[..3], b"{\r\n");
    assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    let text = String::from_utf8(bytes).expect("utf-8");
    assert!(text.contains("\r\n  \"AppVersion\": \"26.10.19.0\""));
    assert!(!text.replace("\r\n", "").contains('\n'), "every line break is CRLF");
}

#[test]
fn write_info_writes_absent_optional_fields_as_explicit_null() {
    let scratch = Scratch::new("nulls");
    let path = scratch.0.join("SucroseInfo.json");
    let info = SucroseInfo {
        license: None,
        tags: None,
        arguments: None,
        localization: None,
        ..fixture("SucroseInfo.video.json")
    };
    write_info(&path, &info).expect("write");
    let written: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");

    for key in ["License", "Tags", "Arguments", "Localization"] {
        assert_eq!(written.get(key), Some(&Value::Null), "{key} must be present as null");
    }
}

#[test]
fn write_info_writes_the_type_as_an_int_and_the_version_as_a_string() {
    let scratch = Scratch::new("types");
    let path = scratch.0.join("SucroseInfo.json");
    write_info(&path, &fixture("SucroseInfo.object-version.json")).expect("write");
    let written: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");

    assert_eq!(written["Type"], json!(3));
    assert_eq!(written["AppVersion"], json!("26.10.19.0"));
    assert_eq!(written["Version"], json!("1.0"));
}

#[test]
fn unknown_fields_survive_a_round_trip() {
    let info = fixture("SucroseInfo.full.json");
    assert_eq!(info.extra.get("FutureField"), Some(&json!(1)));

    let scratch = Scratch::new("extra");
    let path = scratch.0.join("SucroseInfo.json");
    write_info(&path, &info).expect("write");
    let written: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");

    assert_eq!(written["FutureField"], json!(1));
    assert_eq!(written["Localization"]["TR"]["Title"], json!("Ornek video"));
    assert_eq!(written["Tags"], json!(["nature", "loop"]));
}

#[test]
fn type_is_read_as_an_int_or_a_name() {
    assert_eq!(fixture("SucroseInfo.video.json").kind, WallpaperType::Video);
    assert_eq!(fixture("SucroseInfo.object-version.json").kind, WallpaperType::Video);
}

#[test]
fn version_is_read_as_a_string_or_an_object_with_missing_components_as_minus_one() {
    let object = fixture("SucroseInfo.object-version.json");

    assert_eq!(object.app_version, LegacyVersion::new(26, 10, 19, 0));
    assert_eq!(object.version, LegacyVersion::new(1, 0, -1, -1));
    assert_eq!(fixture("SucroseInfo.full.json").version, LegacyVersion::new(1, 2, 3, 4));
}

#[test]
fn the_gate_accepts_a_playable_video_theme() {
    let scratch = Scratch::new("ok");
    let library = scratch.library_with(&fixture("SucroseInfo.video.json"));

    let selected = check_selected(library, "t", &APP).expect("playable");

    assert!(selected.source_path.is_absolute() && selected.source_path.ends_with("video.mp4"));
    assert_eq!(selected.dir, library.join("t"));

    let nested = Scratch::new("ok-nested");
    let library = nested.library_with(&with_source("sub/deep.mp4"));
    assert!(check_selected(library, "t", &APP).is_ok());
}

#[test]
fn nothing_selected_and_a_missing_folder_are_their_own_reasons() {
    let scratch = Scratch::new("folder");
    let library = scratch.library_with(&fixture("SucroseInfo.video.json"));

    assert_eq!(
        check_selected(library, "", &APP).unwrap_err(),
        ThemeRejection::NothingSelected
    );
    assert!(matches!(
        check_selected(library, "nope", &APP).unwrap_err(),
        ThemeRejection::FolderMissing { .. }
    ));
    for escape in ["..", "t/..", "t\\..", "/t", "sub/x"] {
        assert!(
            matches!(
                check_selected(library, escape, &APP).unwrap_err(),
                ThemeRejection::FolderMissing { .. }
            ),
            "{escape:?} is not a folder name"
        );
    }
}

#[test]
fn a_missing_or_invalid_info_file_is_unreadable() {
    let scratch = Scratch::new("info");
    let library = scratch.library_with(&fixture("SucroseInfo.video.json"));
    let info_path = library.join("t").join("SucroseInfo.json");

    fs::write(&info_path, "{ \"Title\": \"half").expect("truncate");
    assert!(matches!(
        check_selected(library, "t", &APP).unwrap_err(),
        ThemeRejection::InfoUnreadable { .. }
    ));

    fs::remove_file(&info_path).expect("remove");
    let rejection = check_selected(library, "t", &APP).unwrap_err();
    assert!(
        matches!(rejection, ThemeRejection::InfoUnreadable { .. }),
        "{rejection}"
    );
}

#[test]
fn other_wallpaper_types_are_refused() {
    for kind in [WallpaperType::Gif, WallpaperType::Web, WallpaperType::YouTube] {
        let info = SucroseInfo {
            kind,
            ..fixture("SucroseInfo.video.json")
        };
        assert!(matches!(
            rejection_for("type", &info),
            ThemeRejection::NotVideo { found, .. } if found == kind
        ));
    }
}

#[test]
fn a_source_that_is_not_a_relative_file_inside_the_theme_is_refused() {
    for source in [
        "",
        "https://example.com/x.mp4",
        "file:///x.mp4",
        "/etc/passwd",
        "\\\\server\\share\\x.mp4",
        "\\\\?\\C:\\x.mp4",
        "\\\\.\\pipe\\x",
        "C:\\x.mp4",
        "C:x.mp4",
        "../outside.mp4",
        "sub/../../outside.mp4",
        "sub\\..\\..\\outside.mp4",
        "missing.mp4",
        "sub",
    ] {
        assert!(
            matches!(
                rejection_for("source", &with_source(source)),
                ThemeRejection::SourceInvalid { .. }
            ),
            "{source:?} must be refused"
        );
    }
}

#[test]
fn an_absolute_path_to_a_real_file_is_still_refused() {
    let scratch = Scratch::new("absolute");
    let outside = scratch.0.join("outside.mp4");
    let library = scratch.library_with(&with_source(&outside.to_string_lossy()));

    assert!(matches!(
        check_selected(library, "t", &APP).unwrap_err(),
        ThemeRejection::SourceInvalid { .. }
    ));
}

#[test]
fn rejection_texts_name_the_problem_and_the_folder_but_not_the_file() {
    let secret = SucroseInfo {
        title: "secret-title".to_owned(),
        ..with_source("missing.mp4")
    };
    let text = rejection_for("text", &secret).to_string();

    assert!(text.contains(" t "), "names the folder: {text}");
    assert!(text.contains("does not exist"), "names the problem: {text}");
    assert!(
        !text.contains("secret-title") && !text.contains("missing.mp4"),
        "{text}"
    );
    assert!(text.ends_with('.') && !text.contains('\n'));
}
