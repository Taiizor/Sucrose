//! Fixture tests for the legacy settings reader (FMT-08): defaults, clamps, parsing quirks and read-only access.
//!
//! The fixtures under `tests/fixtures` are copies of a real C# install's `Setting` files with personal data removed.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::SystemTime,
};

use sucrose_config::legacy::{LegacyFile, LegacySettings, read};
use sucrose_core::settings::{FullScreenAction, GpuPreference, PauseKind, ScreenArea, Stretch};

const ROOT: &str = "legacy-root";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// A scratch `Setting` folder under the target folder, with the given files; never a real legacy folder.
fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("legacy-settings")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the scratch folder under target can be created");
    for (file, content) in files {
        fs::write(dir.join(file), content).expect("the scratch file can be written");
    }
    dir
}

/// A folder holding one fixture under its legacy name, so the same fixture can stand in for any setting file.
fn with_fixture(name: &str, fixture: &str, as_file: LegacyFile) -> PathBuf {
    let content = fs::read_to_string(fixtures().join(fixture)).expect("the fixture exists");
    scratch(name, &[(as_file.file_name(), &content)])
}

fn load(dir: &Path) -> LegacySettings {
    LegacySettings::load(dir, Path::new(ROOT))
}

fn engine(json: &str) -> sucrose_config::legacy::EngineSettings {
    // One folder per call: tests run in parallel and must not overwrite each other's Engine.json.
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    let name = format!("engine-inline-{}", CALLS.fetch_add(1, Ordering::Relaxed));
    load(&scratch(&name, &[("Engine.json", json)])).engine
}

#[test]
fn library_fixture_gives_location_and_selected() {
    let library = load(&fixtures()).library;
    assert_eq!(library.location, PathBuf::from("C:\\Sucrose\\Library"));
    assert_eq!(library.selected, "AAAAAAAAAAAAAAAAAAAAAAAAA");
}

#[test]
fn missing_location_falls_back_to_the_legacy_library_folder() {
    let dir = scratch(
        "library-no-location",
        &[("Library.json", r#"{"Properties":{"Selected":"x"}}"#)],
    );
    let library = load(&dir).library;
    assert_eq!(library.location, Path::new(ROOT).join("Library"));
    assert_eq!(library.selected, "x");
}

#[test]
fn bom_and_crlf_parse_like_the_plain_file() {
    let plain = load(&fixtures()).library;
    let bom = load(&with_fixture("library-bom", "Library.bom.json", LegacyFile::Library)).library;
    assert_eq!(bom, plain);

    // The same text with CRLF forced, whatever the checkout's line endings are.
    let text = fs::read_to_string(fixtures().join("Library.json")).expect("the fixture exists");
    let crlf = format!("\u{feff}{}", text.replace("\r\n", "\n").replace('\n', "\r\n"));
    let dir = scratch("library-bom-crlf", &[("Library.json", &crlf)]);
    assert_eq!(load(&dir).library, plain);
}

#[test]
fn general_fixture_and_missing_key() {
    assert!(load(&fixtures()).general.app_visible);
    let hidden = scratch(
        "general-hidden",
        &[("General.json", r#"{"Properties":{"AppVisible":false}}"#)],
    );
    assert!(!load(&hidden).general.app_visible);
    let missing = scratch(
        "general-missing",
        &[("General.json", r#"{"Properties":{"Culture":"EN"}}"#)],
    );
    assert!(load(&missing).general.app_visible);
}

#[test]
fn engine_fixture_values() {
    let engine = load(&fixtures()).engine;
    assert_eq!(engine.volume, 100);
    assert!(engine.looping);
    assert_eq!(engine.stretch, Stretch::UniformToFill);
    assert!(engine.hardware_acceleration);
    assert_eq!(engine.screen, ScreenArea::DisplayBound);
}

#[test]
fn invalid_engine_values_fall_back_to_the_defaults() {
    let dir = with_fixture("engine-invalid", "Engine.invalid.json", LegacyFile::Engine);
    let engine = load(&dir).engine;
    assert_eq!(engine.volume, 100, "250 is clamped to 100");
    assert_eq!(engine.stretch, Stretch::UniformToFill, "an unknown name is the default");
    assert!(engine.hardware_acceleration, "a string is not a boolean");
    assert_eq!(
        engine.screen,
        ScreenArea::DisplayBound,
        "a number is not a name for this enum"
    );
}

#[test]
fn graphic_preference_defaults_to_high_performance() {
    // The real-file fixture has no GraphicPreference key.
    assert_eq!(load(&fixtures()).engine.gpu_preference, GpuPreference::HighPerformance);
    assert_eq!(
        engine(r#"{"Properties":{"GraphicPreference":"Turbo"}}"#).gpu_preference,
        GpuPreference::HighPerformance,
        "an unknown name is the default"
    );
    assert_eq!(
        engine(r#"{"Properties":{"GraphicPreference":"3"}}"#).gpu_preference,
        GpuPreference::HighPerformance,
        "an ordinal past the enum is the default"
    );
}

#[test]
fn graphic_preference_is_read_by_name() {
    let dir = with_fixture("engine-gpu", "Engine.gpu.json", LegacyFile::Engine);
    assert_eq!(load(&dir).engine.gpu_preference, GpuPreference::PowerSaving);
}

#[test]
fn graphic_preference_accepts_the_ordinal() {
    assert_eq!(
        engine(r#"{"Properties":{"GraphicPreference":"0"}}"#).gpu_preference,
        GpuPreference::Default
    );
    assert_eq!(
        engine(r#"{"Properties":{"GraphicPreference":1}}"#).gpu_preference,
        GpuPreference::PowerSaving
    );
}

#[test]
fn volume_accepts_numeric_strings_and_clamps_both_ways() {
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":"40"}}"#).volume, 40);
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":-5}}"#).volume, 0);
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":250}}"#).volume, 100);
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":12}}"#).volume, 12);
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":"loud"}}"#).volume, 100);
    assert_eq!(engine(r#"{"Properties":{"WallpaperVolume":true}}"#).volume, 100);
}

#[test]
fn engine_enums_accept_names_and_documented_ordinals() {
    assert_eq!(
        engine(r#"{"Properties":{"StretchType":"Fill"}}"#).stretch,
        Stretch::Fill
    );
    assert_eq!(
        engine(r#"{"Properties":{"StretchType":"2"}}"#).stretch,
        Stretch::Uniform
    );
    assert_eq!(engine(r#"{"Properties":{"StretchType":0}}"#).stretch, Stretch::None);
    assert_eq!(
        engine(r#"{"Properties":{"ScreenType":"WorkingArea"}}"#).screen,
        ScreenArea::WorkingArea
    );
    assert_eq!(
        engine(r#"{"Properties":{"ScreenType":"1"}}"#).screen,
        ScreenArea::DisplayBound
    );
}

#[test]
fn backgroundog_defaults_and_pause() {
    let defaults = load(&fixtures()).backgroundog;
    assert_eq!(defaults.fullscreen, FullScreenAction::Resume);
    assert_eq!(defaults.pause, PauseKind::Light);

    let dir = with_fixture(
        "backgroundog-pause",
        "Backgroundog.pause.json",
        LegacyFile::Backgroundog,
    );
    let paused = load(&dir).backgroundog;
    assert_eq!(paused.fullscreen, FullScreenAction::Pause);
    assert_eq!(paused.pause, PauseKind::Heavy);
}

#[test]
fn enum_names_are_case_sensitive() {
    let dir = scratch(
        "case-sensitive",
        &[(
            "Backgroundog.json",
            r#"{"Properties":{"FullScreenPerformance":"pause","PausePerformanceType":"heavy"}}"#,
        )],
    );
    let backgroundog = load(&dir).backgroundog;
    assert_eq!(backgroundog.fullscreen, FullScreenAction::Resume);
    assert_eq!(backgroundog.pause, PauseKind::Light);
}

#[test]
fn fullscreen_action_accepts_documented_ordinals() {
    let dir = scratch(
        "fullscreen-ordinal",
        &[("Backgroundog.json", r#"{"Properties":{"FullScreenPerformance":"0"}}"#)],
    );
    assert_eq!(load(&dir).backgroundog.fullscreen, FullScreenAction::Close);
}

#[test]
fn a_missing_folder_gives_every_legacy_default() {
    let settings = load(&fixtures().join("no-such-folder"));
    assert_eq!(settings.library.location, Path::new(ROOT).join("Library"));
    assert_eq!(settings.library.selected, "");
    assert!(settings.general.app_visible);
    assert_eq!(settings.engine.volume, 100);
    assert!(settings.engine.looping);
    assert_eq!(settings.engine.stretch, Stretch::UniformToFill);
    assert!(settings.engine.hardware_acceleration);
    assert_eq!(settings.engine.screen, ScreenArea::DisplayBound);
    assert_eq!(settings.engine.gpu_preference, GpuPreference::HighPerformance);
    assert_eq!(settings.backgroundog.fullscreen, FullScreenAction::Resume);
    assert_eq!(settings.backgroundog.pause, PauseKind::Light);
}

#[test]
fn empty_and_broken_files_read_as_defaults() {
    let missing = load(&fixtures().join("no-such-folder"));
    let dir = scratch(
        "broken-files",
        &[
            ("Library.json", ""),
            ("General.json", "{ \"Properties\": { \"AppVisible\": tru"),
            ("Engine.json", "[1, 2]"),
            ("Backgroundog.json", "\r\n  \r\n"),
        ],
    );
    assert_eq!(load(&dir), missing);
}

#[test]
fn keys_outside_the_m1_set_are_not_read() {
    // DisplayScreenType and ScreenDevice (D-01), CrashExplorer (D-11), the Video engine choice and StayAwake.
    let dir = scratch(
        "unrelated-keys",
        &[(
            "Engine.json",
            r#"{"Properties":{"DisplayScreenType":"SpanAcross","ScreenDevice":"X","CrashExplorer":true,"Video":"VlcPlayerLive","StayAwake":true}}"#,
        )],
    );
    assert_eq!(load(&dir), load(&fixtures().join("no-such-folder")));
}

#[test]
fn reading_leaves_the_files_untouched() {
    let modified = |dir: &Path| -> Vec<Option<SystemTime>> {
        ["Library", "General", "Engine", "Backgroundog"]
            .iter()
            .map(|name| {
                fs::metadata(dir.join(format!("{name}.json")))
                    .and_then(|meta| meta.modified())
                    .ok()
            })
            .collect()
    };
    let before = modified(&fixtures());
    let _ = load(&fixtures());
    let _ = read(&fixtures(), LegacyFile::Engine);
    assert_eq!(modified(&fixtures()), before);
}

#[test]
fn fixtures_contain_no_profile_paths() {
    let mut checked = 0;
    for entry in fs::read_dir(fixtures()).expect("the fixtures folder exists") {
        let path = entry.expect("a directory entry").path();
        let text = fs::read_to_string(&path).expect("a fixture is UTF-8").to_lowercase();
        for forbidden in [r"\users\", r"\\users\\", "/users/", "appdata"] {
            assert!(!text.contains(forbidden), "{} contains {forbidden}", path.display());
        }
        checked += 1;
    }
    assert!(checked >= 7, "expected the 7 committed fixtures, found {checked}");
}
