//! Library scan against fixture libraries (hand-written and sanitized real SucroseInfo.json files) and temp folders.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::json;
use sucrose_core::{
    library::{Compatibility, CorruptReason, LibraryTheme, Scan, display_title, is_url, scan, search, sort},
    paths::legacy_setting_dir,
    settings::{SortKind, SortMode},
    theme::{WallpaperType, parse_info},
    version::{LegacyVersion, app_version},
};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture_scan() -> Scan {
    scan(&fixtures().join("library"), &app_version()).expect("fixture library scans")
}

/// A folder under the system temp folder, removed again when it goes out of scope.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sucrose-library-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch folder");
        Self(dir)
    }

    /// A library of theme folders written from `(folder, title, description, tags, localization)`, each with every
    /// file it names.
    fn library_of(&self, themes: &[(&str, &str, &str, serde_json::Value, serde_json::Value)]) -> &Path {
        for (folder, title, description, tags, localization) in themes {
            let theme = self.0.join(folder);
            fs::create_dir_all(&theme).expect("create theme folder");
            let info = json!({
                "AppVersion": "26.10.19.0", "Title": title, "Thumbnail": "th.jpg", "Preview": "pv.gif",
                "Description": description, "Type": 3, "Source": "v.mp4", "Tags": tags,
                "Localization": localization,
            });
            fs::write(theme.join("SucroseInfo.json"), info.to_string()).expect("write info");
            for name in ["th.jpg", "pv.gif", "v.mp4"] {
                fs::write(theme.join(name), b"x").expect("write file");
            }
        }
        &self.0
    }

    /// Adds a theme folder `t` whose SucroseInfo.json has the given Preview and every file it names.
    fn theme_with_preview(&self, preview: &str) -> &Path {
        let theme = self.0.join("t");
        fs::create_dir_all(&theme).expect("create theme folder");
        let info = format!(
            r#"{{"AppVersion":"26.10.19.0","Title":"T","Thumbnail":"th.jpg","Preview":{},"Type":3,"Source":"v.mp4"}}"#,
            serde_json::to_string(preview).expect("string serializes")
        );
        fs::write(theme.join("SucroseInfo.json"), info).expect("write info");
        for name in ["th.jpg", "v.mp4"] {
            fs::write(theme.join(name), b"x").expect("write file");
        }
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn corrupt_reason(scan: &Scan, folder: &str) -> CorruptReason {
    scan.corrupt
        .iter()
        .find(|c| c.folder == folder)
        .unwrap_or_else(|| panic!("{folder} is not corrupt"))
        .reason
}

#[test]
fn fixture_library_has_six_themes_four_corrupt_and_one_ignored() {
    let scan = fixture_scan();
    let themes: Vec<&str> = scan.themes.iter().map(|t| t.folder.as_str()).collect();
    assert_eq!(
        themes,
        ["gif", "lower-case-type", "newer-version", "url-source", "video", "web"]
    );
    assert_eq!(scan.corrupt.len(), 4);
    assert_eq!(scan.ignored, 1);
}

#[test]
fn a_newer_app_version_is_a_theme_not_a_corrupt_folder() {
    let scan = fixture_scan();
    let theme = scan
        .themes
        .iter()
        .find(|t| t.folder == "newer-version")
        .expect("listed");
    assert_eq!(
        theme.compatibility,
        Compatibility::NeedsNewerApp {
            required: LegacyVersion::new(99, 1, 1, 0)
        }
    );
    let video = scan.themes.iter().find(|t| t.folder == "video").expect("listed");
    assert_eq!(video.compatibility, Compatibility::Compatible);
}

#[test]
fn a_type_in_lower_case_is_a_video_theme() {
    let scan = fixture_scan();
    let theme = scan
        .themes
        .iter()
        .find(|t| t.folder == "lower-case-type")
        .expect("listed");
    assert_eq!(theme.info.kind, WallpaperType::Video);
}

#[test]
fn corrupt_folders_get_their_reason() {
    let scan = fixture_scan();
    assert_eq!(corrupt_reason(&scan, "escaping-preview"), CorruptReason::PathEscapes);
    assert_eq!(corrupt_reason(&scan, "missing-preview"), CorruptReason::PreviewMissing);
    assert_eq!(corrupt_reason(&scan, "broken-json"), CorruptReason::InfoInvalid);
    assert_eq!(corrupt_reason(&scan, "project-only"), CorruptReason::InfoMissing);
}

#[test]
fn a_url_source_needs_no_source_file() {
    let scan = fixture_scan();
    let theme = scan.themes.iter().find(|t| t.folder == "url-source").expect("listed");
    assert_eq!(theme.info.kind, WallpaperType::Url);
    assert!(!theme.dir.join("page").exists());
}

#[test]
fn listed_files_stay_inside_their_theme_folder() {
    let scan = fixture_scan();
    assert!(!scan.themes.is_empty());
    for theme in &scan.themes {
        assert!(theme.thumbnail.starts_with(&theme.dir), "{}", theme.folder);
        assert!(theme.preview.starts_with(&theme.dir), "{}", theme.folder);
        assert!(theme.thumbnail.is_file() && theme.preview.is_file(), "{}", theme.folder);
    }
}

#[test]
fn paths_that_leave_the_theme_folder_make_it_corrupt() {
    for (index, preview) in [
        "../th.jpg",
        "sub/../../th.jpg",
        "/etc/hosts",
        "\\\\server\\share\\p.gif",
        "C:\\Windows\\p.gif",
        "D:p.gif",
    ]
    .iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("escape-{index}"));
        let location = scratch.theme_with_preview(preview);
        let scan = scan(location, &app_version()).expect("scans");
        assert!(scan.themes.is_empty(), "{preview} was accepted");
        assert_eq!(corrupt_reason(&scan, "t"), CorruptReason::PathEscapes, "{preview}");
    }
}

#[test]
fn an_empty_preview_is_a_missing_preview() {
    let scratch = Scratch::new("empty-preview");
    let location = scratch.theme_with_preview("");
    let scan = scan(location, &app_version()).expect("scans");
    assert_eq!(corrupt_reason(&scan, "t"), CorruptReason::PreviewMissing);
}

#[test]
fn a_missing_location_is_an_error_and_an_empty_one_is_an_empty_scan() {
    let scratch = Scratch::new("empty");
    assert!(scan(&scratch.0.join("nope"), &app_version()).is_err());
    let scan = scan(&scratch.0, &app_version()).expect("scans");
    assert!(scan.themes.is_empty() && scan.corrupt.is_empty() && scan.ignored == 0);
}

#[test]
fn the_scan_does_not_change_the_library() {
    let before = tree(&fixtures().join("library"));
    let _ = fixture_scan();
    assert_eq!(tree(&fixtures().join("library")), before);
}

fn tree(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in fs::read_dir(&next).expect("readable") {
            let path = entry.expect("entry").path();
            names.push(path.to_string_lossy().into_owned());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    names.sort();
    names
}

#[test]
fn sanitized_real_theme_files_parse() {
    let expected = [
        ("video", WallpaperType::Video),
        ("web", WallpaperType::Web),
        ("gif", WallpaperType::Gif),
    ];
    for (name, kind) in expected {
        let path = fixtures().join("library-real").join(name).join("SucroseInfo.json");
        let info = parse_info(&fs::read_to_string(path).expect("fixture is readable")).expect("parses");
        assert_eq!(info.kind, kind, "{name}");
    }
}

fn folders(themes: &[LibraryTheme]) -> Vec<&str> {
    themes.iter().map(|t| t.folder.as_str()).collect()
}

fn at(seconds: u64) -> Option<std::time::SystemTime> {
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds))
}

#[test]
fn the_index_holds_title_description_and_tags_lowercased_without_punctuation() {
    let fixture = fixture_scan();
    let video = fixture.themes.iter().find(|t| t.folder == "video").expect("listed");
    assert_eq!(video.index, ["calm", "sea", "waves", "at", "night", "ocean", "blue"]);

    let scratch = Scratch::new("index");
    let location = scratch.library_of(&[
        (
            "a",
            "Rock'n'Roll, Rock!",
            "",
            json!(["ROCK", "Night-Sky"]),
            json!({"TR": {"Title": "Deniz", "Description": "Dalga"}}),
        ),
        ("b", "Ünï İstanbul", "(x) [y]", json!(null), json!(null)),
    ]);
    let scan = scan(location, &app_version()).expect("scans");
    assert_eq!(
        scan.themes[0].index,
        ["rocknroll", "rock", "deniz", "dalga", "nightsky"]
    );
    assert_eq!(scan.themes[1].index, ["ünï", "i\u{307}stanbul", "x", "y"]);
}

#[test]
fn the_title_is_the_localized_one_then_the_base_one_then_the_folder() {
    let scratch = Scratch::new("title");
    let location = scratch.library_of(&[
        ("plain", "Base", "", json!(null), json!(null)),
        (
            "local",
            "Base",
            "",
            json!(null),
            json!({"en": {"Title": "Local", "Description": "d"}}),
        ),
        (
            "other",
            "Base",
            "",
            json!(null),
            json!({"TR": {"Title": "Yerel", "Description": "d"}}),
        ),
        ("empty", "", "", json!(null), json!(null)),
    ]);
    let scan = scan(location, &app_version()).expect("scans");
    let title = |folder: &str| {
        let theme = scan.themes.iter().find(|t| t.folder == folder).expect("listed");
        display_title(&theme.info, &theme.folder).to_owned()
    };
    assert_eq!(title("plain"), "Base");
    assert_eq!(title("local"), "Local");
    assert_eq!(title("other"), "Base");
    assert_eq!(title("empty"), "empty");
}

#[test]
fn sorting_by_name_is_by_title_and_descending_reverses_it() {
    let mut themes = fixture_scan().themes;
    sort(&mut themes, SortMode::Name, SortKind::Ascending);
    assert_eq!(
        folders(&themes),
        ["video", "gif", "url-source", "newer-version", "lower-case-type", "web"]
    );
    sort(&mut themes, SortMode::Name, SortKind::Descending);
    assert_eq!(
        folders(&themes),
        ["web", "lower-case-type", "newer-version", "url-source", "gif", "video"]
    );
}

#[test]
fn equal_keys_keep_folder_name_order_in_both_directions() {
    let mut themes = fixture_scan().themes;
    // Folder order: gif, lower-case-type, newer-version, url-source, video, web.
    for (theme, seconds) in themes.iter_mut().zip([30, 10, 20, 10, 50, 20]) {
        theme.created = at(seconds);
        theme.modified = at(60 - seconds);
    }
    sort(&mut themes, SortMode::Creation, SortKind::Ascending);
    assert_eq!(
        folders(&themes),
        ["lower-case-type", "url-source", "newer-version", "web", "gif", "video"]
    );
    themes.reverse();
    sort(&mut themes, SortMode::Creation, SortKind::Descending);
    assert_eq!(
        folders(&themes),
        ["video", "gif", "newer-version", "web", "lower-case-type", "url-source"]
    );
    sort(&mut themes, SortMode::Modification, SortKind::Ascending);
    assert_eq!(
        folders(&themes),
        ["video", "gif", "newer-version", "web", "lower-case-type", "url-source"]
    );
    sort(&mut themes, SortMode::Modification, SortKind::Descending);
    assert_eq!(
        folders(&themes),
        ["lower-case-type", "url-source", "newer-version", "web", "gif", "video"]
    );
}

fn search_library(scratch: &Scratch) -> Vec<LibraryTheme> {
    let location = scratch.library_of(&[
        ("a", "Night Owl", "", json!(null), json!(null)),
        ("b", "Sea Shell", "A shell", json!(["ocean"]), json!(null)),
        ("c", "Calm Sea", "Waves at night", json!(null), json!(null)),
        ("d", "Mountain", "", json!(null), json!(null)),
    ]);
    scan(location, &app_version()).expect("scans").themes
}

#[test]
fn an_empty_query_returns_every_theme_in_the_current_order() {
    let scratch = Scratch::new("search-empty");
    let themes = search_library(&scratch);
    assert_eq!(search(&themes, ""), [0, 1, 2, 3]);
    assert_eq!(search(&themes, "   "), [0, 1, 2, 3]);
    assert_eq!(search(&themes, "!?"), [0, 1, 2, 3]);
}

#[test]
fn themes_matching_more_words_come_first_and_ties_keep_the_order() {
    let scratch = Scratch::new("search-order");
    let themes = search_library(&scratch);
    assert_eq!(search(&themes, "sea night"), [2, 0, 1]);
    assert_eq!(search(&themes, "night"), [0, 2]);
}

#[test]
fn a_query_word_matches_inside_a_token_ignoring_case_and_punctuation() {
    let scratch = Scratch::new("search-substring");
    let themes = search_library(&scratch);
    assert_eq!(search(&themes, "OCEA"), [1]);
    assert_eq!(search(&themes, "Sea,"), [1, 2]);
    assert!(search(&themes, "zebra").is_empty());
}

#[test]
fn legacy_addresses_are_accepted_and_everything_else_is_not() {
    for url in [
        "http://example.com",
        "https://www.example.com/page?x=1#y",
        "HTTPS://EXAMPLE.COM",
        "http://a-b.c_d.org/path/",
        "https://example.com.",
        "http://sub.example.com:8080/x",
        "http://x.y",
    ] {
        assert!(is_url(url), "{url}");
    }
    for text in [
        "ftp://x.y",
        "http://localhost",
        "index.html",
        "",
        "http://",
        "http://.com",
        "http://a..b",
        "https://exa mple.com",
        "http://example.com/a b",
        "http://example.com/x,",
        "http://example.com\n",
    ] {
        assert!(!is_url(text), "{text:?}");
    }
}

/// Scans the library the maintainer's legacy `Library.json` names. Read-only; run by hand on that machine.
#[test]
#[ignore = "reads the real library of the machine"]
#[allow(clippy::print_stdout)]
fn real_library() {
    let file = legacy_setting_dir()
        .expect("a config folder exists")
        .join("Library.json");
    let text = fs::read_to_string(file).expect("the legacy Library.json is readable");
    let json: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{FEFF}')).expect("valid JSON");
    let location = json["Properties"]["Location"].as_str().expect("Location is set");
    let scan = scan(Path::new(location), &app_version()).expect("the library location scans");
    println!(
        "{} themes, {} corrupt, {} ignored",
        scan.themes.len(),
        scan.corrupt.len(),
        scan.ignored
    );
    assert!(!scan.themes.is_empty(), "no theme found in {location}");
}
