//! SucroseInfo.json round trip and the D-13 gate, against fixtures taken from real themes and temp folders.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sucrose_core::{
    theme::{
        SucroseInfo, ThemeRejection, ThemeSource, WallpaperType, YouTubeIds, check_selected, parse_info, write_info,
        youtube_ids,
    },
    version::{LegacyVersion, app_version},
};

/// The types the service plays in this build (D-31 slice 1), as in `apps/sucrose`.
const PLAYABLE: &[WallpaperType] = &[WallpaperType::Video, WallpaperType::Web];

/// The version this build reports; every gate test runs against it.
fn app() -> LegacyVersion {
    app_version()
}

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
    check_selected(library, "t", &app(), PLAYABLE).expect_err("the gate must refuse this theme")
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

    let selected = check_selected(library, "t", &app(), PLAYABLE).expect("playable");

    assert!(matches!(&selected.source, ThemeSource::File(path) if path.is_absolute() && path.ends_with("video.mp4")));
    assert_eq!(selected.dir, library.join("t"));

    let nested = Scratch::new("ok-nested");
    let library = nested.library_with(&with_source("sub/deep.mp4"));
    assert!(check_selected(library, "t", &app(), PLAYABLE).is_ok());
}

#[test]
fn nothing_selected_and_a_missing_folder_are_their_own_reasons() {
    let scratch = Scratch::new("folder");
    let library = scratch.library_with(&fixture("SucroseInfo.video.json"));

    assert_eq!(
        check_selected(library, "", &app(), PLAYABLE).unwrap_err(),
        ThemeRejection::NothingSelected
    );
    assert!(matches!(
        check_selected(library, "nope", &app(), PLAYABLE).unwrap_err(),
        ThemeRejection::FolderMissing { .. }
    ));
    for escape in ["..", "t/..", "t\\..", "/t", "sub/x"] {
        assert!(
            matches!(
                check_selected(library, escape, &app(), PLAYABLE).unwrap_err(),
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
        check_selected(library, "t", &app(), PLAYABLE).unwrap_err(),
        ThemeRejection::InfoUnreadable { .. }
    ));

    fs::remove_file(&info_path).expect("remove");
    let rejection = check_selected(library, "t", &app(), PLAYABLE).unwrap_err();
    assert!(
        matches!(rejection, ThemeRejection::InfoUnreadable { .. }),
        "{rejection}"
    );
}

#[test]
fn other_wallpaper_types_are_refused() {
    for kind in [
        WallpaperType::Gif,
        WallpaperType::Url,
        WallpaperType::YouTube,
        WallpaperType::Application,
    ] {
        let info = SucroseInfo {
            kind,
            ..fixture("SucroseInfo.video.json")
        };
        assert!(matches!(
            rejection_for("type", &info),
            ThemeRejection::TypeNotPlayable { found, .. } if found == kind
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
        check_selected(library, "t", &app(), PLAYABLE).unwrap_err(),
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

/// A library whose theme asks for `required` as its AppVersion.
fn rejection_with_app_version(name: &str, required: LegacyVersion) -> Result<(), ThemeRejection> {
    let scratch = Scratch::new(name);
    let info = SucroseInfo {
        app_version: required,
        ..fixture("SucroseInfo.video.json")
    };
    let library = scratch.library_with(&info);
    check_selected(library, "t", &app(), PLAYABLE).map(|_| ())
}

#[test]
fn a_theme_for_exactly_this_app_version_is_accepted() {
    assert!(rejection_with_app_version("same-version", app()).is_ok());
}

#[test]
fn one_more_revision_needs_a_newer_app() {
    let mut newer = app();
    newer.revision += 1;

    let rejection = rejection_with_app_version("newer-version", newer).unwrap_err();

    assert_eq!(
        rejection,
        ThemeRejection::NeedsNewerApp {
            folder: "t".to_owned(),
            required: newer,
            current: app()
        }
    );
    let text = rejection.to_string();
    assert!(text.contains("t") && text.contains(&newer.to_string()), "{text}");
}

#[test]
fn the_maintainers_theme_passes_with_the_date_based_version() {
    let info = fixture("SucroseInfo.video.json");
    assert_eq!(info.app_version, LegacyVersion::new(26, 10, 19, 0));
    let scratch = Scratch::new("maintainer");
    let library = scratch.library_with(&info);

    assert!(check_selected(library, "t", &app(), PLAYABLE).is_ok());
}

/// A Web theme whose page is `index.html`, with `source` as its Source.
fn web_info(source: &str) -> SucroseInfo {
    SucroseInfo {
        kind: WallpaperType::Web,
        source: source.to_owned(),
        ..fixture("SucroseInfo.video.json")
    }
}

#[test]
fn the_gate_accepts_a_web_theme_and_returns_its_page() {
    let scratch = Scratch::new("web-ok");
    let library = scratch.library_with(&web_info("index.html"));
    fs::write(library.join("t").join("index.html"), b"<html></html>").expect("write page");

    let selected = check_selected(library, "t", &app(), PLAYABLE).expect("playable");

    assert_eq!(selected.info.kind, WallpaperType::Web);
    assert!(matches!(&selected.source, ThemeSource::File(path) if path.is_absolute() && path.ends_with("index.html")));
}

#[test]
fn a_web_theme_is_refused_when_the_build_plays_only_video() {
    let scratch = Scratch::new("web-not-playable");
    let library = scratch.library_with(&web_info("video.mp4"));

    let rejection = check_selected(library, "t", &app(), &[WallpaperType::Video]).unwrap_err();

    assert!(matches!(
        rejection,
        ThemeRejection::TypeNotPlayable {
            found: WallpaperType::Web,
            ..
        }
    ));
    assert!(rejection.to_string().contains("does not play yet"));
}

#[test]
fn a_web_theme_whose_source_is_empty_missing_a_folder_outside_or_an_address_is_refused() {
    for source in [
        "",
        "missing.html",
        "sub",
        "../outside.mp4",
        "https://example.com/index.html",
        "http://example.com/",
    ] {
        let scratch = Scratch::new("web-source");
        let library = scratch.library_with(&web_info(source));
        let rejection = check_selected(library, "t", &app(), PLAYABLE).unwrap_err();
        assert!(
            matches!(rejection, ThemeRejection::SourceInvalid { .. }),
            "{source:?}: {rejection}"
        );
    }
}

/// Every type the gate knows, for the table of which source each one takes.
const EVERY_TYPE: [WallpaperType; 6] = [
    WallpaperType::Gif,
    WallpaperType::Url,
    WallpaperType::Web,
    WallpaperType::Video,
    WallpaperType::YouTube,
    WallpaperType::Application,
];

fn typed(kind: WallpaperType, source: &str) -> SucroseInfo {
    SucroseInfo {
        kind,
        source: source.to_owned(),
        ..fixture("SucroseInfo.video.json")
    }
}

/// The text of the refusal for `source` on a theme of `kind` (every type playable).
fn refusal(kind: WallpaperType, source: &str) -> String {
    // The tests run in parallel: each thread gets a folder of its own.
    let scratch = Scratch::new(&format!("refusal-{:?}", std::thread::current().id()));
    let library = scratch.library_with(&typed(kind, source));
    match check_selected(library, "t", &app(), EVERY_TYPE_PLAYABLE) {
        Err(ThemeRejection::SourceInvalid { reason, .. }) => reason,
        other => panic!("{kind:?} {source:?} must be refused as an invalid source: {other:?}"),
    }
}

const EVERY_TYPE_PLAYABLE: &[WallpaperType] = &EVERY_TYPE;

#[test]
fn a_url_theme_with_an_http_or_https_address_plays_that_address() {
    for address in [
        "https://example.com/",
        "http://example.com",
        "HTTPS://Example.com/Path?q=1#frag",
        "https://user@example.com:8443/p",
        "https://[::1]:8080/",
        "https://b\u{fc}cher.example/",
    ] {
        let scratch = Scratch::new("url-ok");
        let library = scratch.library_with(&typed(WallpaperType::Url, address));

        let selected = check_selected(library, "t", &app(), EVERY_TYPE_PLAYABLE).expect(address);

        assert_eq!(selected.source, ThemeSource::Address(address.to_owned()), "{address}");
    }
}

#[test]
fn an_address_of_exactly_the_limit_passes_and_one_byte_more_is_refused() {
    let base = "https://example.com/";
    let at_limit = format!("{base}{}", "a".repeat(2048 - base.len()));
    let scratch = Scratch::new("url-limit");
    let library = scratch.library_with(&typed(WallpaperType::Url, &at_limit));
    assert!(check_selected(library, "t", &app(), EVERY_TYPE_PLAYABLE).is_ok());

    assert!(refusal(WallpaperType::Url, &format!("{at_limit}a")).contains("2048"));
}

#[test]
fn a_url_theme_refuses_every_other_scheme_a_relative_text_and_an_address_without_a_host_each_with_its_own_reason() {
    let empty = refusal(WallpaperType::Url, "");
    let scheme: Vec<String> = [
        "file:///C:/x",
        "javascript:alert(1)",
        "data:text/html,x",
        "about:blank",
        "ftp://example.com/",
    ]
    .iter()
    .map(|source| refusal(WallpaperType::Url, source))
    .collect();
    let relative: Vec<String> = ["example.com", "/index.html", "index.html", "..\\x"]
        .iter()
        .map(|source| refusal(WallpaperType::Url, source))
        .collect();
    let hostless: Vec<String> = ["https://", "https:///x", "https://:80/", "https://@/x", "http://?q=1"]
        .iter()
        .map(|source| refusal(WallpaperType::Url, source))
        .collect();
    let spaced: Vec<String> = [
        "https:// space.example",
        "https://a.example/\tx",
        "https://a.example/\nx",
    ]
    .iter()
    .map(|source| refusal(WallpaperType::Url, source))
    .collect();

    assert!(empty.contains("no Source"), "{empty}");
    assert!(
        scheme.iter().all(|reason| reason.contains("http or https")),
        "{scheme:?}"
    );
    assert!(
        relative.iter().all(|reason| reason.contains("absolute")),
        "{relative:?}"
    );
    assert!(hostless.iter().all(|reason| reason.contains("no host")), "{hostless:?}");
    assert!(spaced.iter().all(|reason| reason.contains("whitespace")), "{spaced:?}");
    let firsts = [&empty, &scheme[0], &relative[0], &hostless[0], &spaced[0]];
    for (index, first) in firsts.iter().enumerate() {
        for other in &firsts[index + 1..] {
            assert_ne!(first, other, "two classes of refusal share a reason");
        }
    }
}

#[test]
fn a_video_theme_with_an_address_is_still_refused_by_the_file_rule_and_a_url_theme_with_a_file_name_by_the_address_rule()
 {
    assert!(refusal(WallpaperType::Video, "https://example.com/x.mp4").contains("address, not a file"));
    assert!(refusal(WallpaperType::Url, "video.mp4").contains("absolute"));
}

#[test]
fn every_type_takes_the_source_its_kind_decides() {
    let youtube = "https://www.youtube.com/watch?v=znNllvBt_Jg";
    for kind in EVERY_TYPE {
        let scratch = Scratch::new("table");
        let file = scratch.library_with(&typed(kind, "video.mp4"));
        let with_file = check_selected(file, "t", &app(), EVERY_TYPE_PLAYABLE);
        let scratch = Scratch::new("table-address");
        let address = scratch.library_with(&typed(kind, youtube));
        let with_address = check_selected(address, "t", &app(), EVERY_TYPE_PLAYABLE);

        match kind {
            WallpaperType::Url | WallpaperType::YouTube => {
                assert!(with_file.is_err(), "{kind:?} must not take a file");
                assert_eq!(
                    with_address.expect("an address").source,
                    ThemeSource::Address(youtube.to_owned()),
                    "{kind:?}"
                );
            }
            _ => {
                assert!(
                    matches!(with_file.expect("a file").source, ThemeSource::File(path) if path.is_absolute()),
                    "{kind:?}"
                );
                assert!(with_address.is_err(), "{kind:?} must not take an address");
            }
        }
    }
}

#[test]
fn a_youtube_theme_needs_a_v_or_a_list_id_in_its_address() {
    for good in [
        "https://www.youtube.com/watch?v=znNllvBt_Jg",
        "https://www.youtube.com/playlist?list=PLx_y-1",
        "https://www.youtube.com/watch?v=a1&list=PLb2",
        "http://youtube.com/watch?x=1&v=abc#t=5",
    ] {
        let scratch = Scratch::new("yt-ok");
        let library = scratch.library_with(&typed(WallpaperType::YouTube, good));
        let selected = check_selected(library, "t", &app(), EVERY_TYPE_PLAYABLE).expect(good);
        assert_eq!(selected.source, ThemeSource::Address(good.to_owned()));
    }
    let long = format!("https://www.youtube.com/watch?v={}", "a".repeat(65));
    let neither = refusal(WallpaperType::YouTube, "https://www.youtube.com/");
    let bad: Vec<String> = [
        "https://www.youtube.com/watch?v=a<b",
        "https://www.youtube.com/watch?v=",
        "https://www.youtube.com/watch?v=a&v=b",
        "https://www.youtube.com/watch?list=a%20b",
        long.as_str(),
    ]
    .iter()
    .map(|source| refusal(WallpaperType::YouTube, source))
    .collect();

    assert!(neither.contains("neither"), "{neither}");
    assert!(bad.iter().all(|reason| reason.contains("not a valid")), "{bad:?}");
    assert_ne!(neither, bad[0]);
    assert!(refusal(WallpaperType::YouTube, "file:///x?v=abc").contains("http or https"));
}

#[test]
fn youtube_ids_reads_the_v_and_list_parameters_as_the_legacy_app_does() {
    let ids = |address: &str| youtube_ids(address);
    assert_eq!(
        ids("https://www.youtube.com/watch?v=znNllvBt_Jg"),
        Some(YouTubeIds {
            video: Some("znNllvBt_Jg".to_owned()),
            list: None
        })
    );
    assert_eq!(
        ids("https://www.youtube.com/playlist?list=PLx_y-1"),
        Some(YouTubeIds {
            video: None,
            list: Some("PLx_y-1".to_owned())
        })
    );
    assert_eq!(
        ids("https://www.youtube.com/watch?v=a1&list=PLb2"),
        Some(YouTubeIds {
            video: Some("a1".to_owned()),
            list: Some("PLb2".to_owned())
        })
    );
    assert_eq!(ids("https://www.youtube.com/"), None);
    assert_eq!(ids("https://www.youtube.com/watch?v=a<b"), None);
    assert_eq!(
        ids(&format!("https://www.youtube.com/watch?v={}", "a".repeat(64))).map(|ids| ids.video.map(|v| v.len())),
        Some(Some(64))
    );
    assert_eq!(
        ids(&format!("https://www.youtube.com/watch?v={}", "a".repeat(65))),
        None
    );
}
