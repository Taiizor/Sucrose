//! The generated pages: their addresses, their files in the content folder and what they contain.

use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use sucrose_core::theme::WallpaperType;
use sucrose_engine_web::pages::{
    GeneratedPage, PageError, ensure_file, ensure_file_with, ensure_written, generated_url, prepare_in,
};

/// A scratch folder, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sucrose-pages-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch folder");
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_video_address_carries_the_path_relative_to_the_theme_folder_as_one_encoded_query_value() {
    let url = generated_url(
        GeneratedPage::Video,
        Path::new("C:/lib/t"),
        "C:/lib/t/media/clip one.mp4",
    )
    .expect("a source inside the theme");

    assert_eq!(
        url,
        "https://sucrose.wallpaper.engine/VideoContent.html?src=media%2Fclip%20one.mp4"
    );
}

#[test]
fn backslashes_case_and_the_verbatim_prefix_do_not_change_the_address() {
    let url = generated_url(
        GeneratedPage::Video,
        Path::new("C:\\Lib\\T"),
        "\\\\?\\c:\\lib\\t\\clip.mp4",
    )
    .expect("the same folder spelled differently");

    assert_eq!(url, "https://sucrose.wallpaper.engine/VideoContent.html?src=clip.mp4");
}

#[test]
fn a_relative_source_is_taken_as_it_is_and_special_characters_are_escaped() {
    let url = generated_url(GeneratedPage::Video, Path::new("C:/lib/t"), "a&b/c#d?.mp4").expect("relative");

    assert_eq!(
        url,
        "https://sucrose.wallpaper.engine/VideoContent.html?src=a%26b%2Fc%23d%3F.mp4"
    );
}

#[test]
fn a_source_outside_the_theme_folder_is_an_error() {
    let theme = Path::new("C:/lib/t");
    for source in [
        "C:/lib/other/clip.mp4",
        "C:/lib/t2/clip.mp4",
        "D:/lib/t/clip.mp4",
        "/etc/passwd",
        "../clip.mp4",
        "media/../../clip.mp4",
        "C:/lib/t/../other/clip.mp4",
    ] {
        let result = generated_url(GeneratedPage::Video, theme, source);
        assert!(matches!(result, Err(PageError::OutsideTheme)), "{source}: {result:?}");
    }
    assert!(matches!(
        generated_url(GeneratedPage::Video, theme, ""),
        Err(PageError::Empty)
    ));
    assert!(matches!(
        generated_url(GeneratedPage::Video, theme, "C:/lib/t"),
        Err(PageError::OutsideTheme)
    ));
}

#[test]
fn generated_pages_exist_for_a_video_theme_only_so_far() {
    assert_eq!(
        GeneratedPage::for_kind(WallpaperType::Video),
        Some(GeneratedPage::Video)
    );
    for kind in [
        WallpaperType::Web,
        WallpaperType::Url,
        WallpaperType::YouTube,
        WallpaperType::Application,
    ] {
        assert_eq!(GeneratedPage::for_kind(kind), None, "{kind:?}");
    }
}

#[test]
fn ensure_written_creates_the_pages_in_an_empty_folder_and_leaves_identical_ones_alone() {
    let scratch = Scratch::new();
    let content = scratch.0.join("Content");

    let first = ensure_written(&content).expect("first write");
    let written = std::fs::read_to_string(content.join("VideoContent.html")).expect("page exists");
    let before = std::fs::metadata(content.join("VideoContent.html"))
        .and_then(|meta| meta.modified())
        .expect("modified time");
    std::thread::sleep(std::time::Duration::from_millis(30));
    let second = ensure_written(&content).expect("second write");
    let after = std::fs::metadata(content.join("VideoContent.html"))
        .and_then(|meta| meta.modified())
        .expect("modified time");

    assert_eq!(first, [GeneratedPage::Video]);
    assert_eq!(written, GeneratedPage::Video.html());
    assert!(second.is_empty(), "an identical page is not written again");
    assert_eq!(before, after, "an identical page is not touched");
}

#[test]
fn a_different_page_is_replaced_through_a_temporary_file_and_no_temporary_file_stays() {
    let scratch = Scratch::new();
    std::fs::write(scratch.0.join("VideoContent.html"), "old page").expect("old page");

    let written = ensure_written(&scratch.0).expect("replace");

    assert_eq!(written, [GeneratedPage::Video]);
    assert_eq!(
        std::fs::read_to_string(scratch.0.join("VideoContent.html")).expect("page"),
        GeneratedPage::Video.html()
    );
    let names: Vec<String> = std::fs::read_dir(&scratch.0)
        .expect("list")
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    assert_eq!(names, ["VideoContent.html"], "no temporary file is left");
}

#[test]
fn a_rename_refused_because_the_file_is_open_is_success_when_the_file_holds_the_same_bytes() {
    let scratch = Scratch::new();
    let target = scratch.0.join("page.html");
    // Another renderer wrote the same page between our check and our rename.
    let other = target.clone();
    let refused = move |_: &Path, _: &Path| -> io::Result<()> {
        std::fs::write(&other, b"new").expect("the other renderer writes");
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    };

    let result = ensure_file_with(&target, b"new", &refused);

    assert!(matches!(result, Ok(false)), "{result:?}");
    assert_eq!(std::fs::read(&target).expect("page"), b"new");
    assert_eq!(
        std::fs::read_dir(&scratch.0).expect("list").count(),
        1,
        "the temporary file is removed"
    );
}

#[test]
fn a_rename_refused_while_the_file_stays_different_is_an_error() {
    let scratch = Scratch::new();
    let target = scratch.0.join("page.html");
    std::fs::write(&target, b"old").expect("old");
    let refused = |_: &Path, _: &Path| Err(io::Error::from(io::ErrorKind::PermissionDenied));

    let result = ensure_file_with(&target, b"new", &refused);

    assert!(result.is_err(), "{result:?}");
    assert_eq!(std::fs::read(&target).expect("page"), b"old");
    assert!(ensure_file(&target, b"new").expect("a plain write works"));
}

#[test]
fn prepare_writes_the_pages_and_answers_the_address_and_a_theme_page_needs_neither() {
    let scratch = Scratch::new();
    let theme = scratch.0.join("theme");
    let content = scratch.0.join("Content");

    let video = prepare_in(
        &content,
        WallpaperType::Video,
        &theme,
        &theme.join("clip.mp4").to_string_lossy(),
    )
    .expect("prepared");
    let web = prepare_in(&content, WallpaperType::Web, &theme, "index.html").expect("web");

    assert_eq!(
        video.as_deref(),
        Some("https://sucrose.wallpaper.engine/VideoContent.html?src=clip.mp4")
    );
    assert!(content.join("VideoContent.html").is_file());
    assert_eq!(web, None);
}

#[test]
fn a_bad_source_is_refused_before_anything_is_written() {
    let scratch = Scratch::new();
    let content = scratch.0.join("Content");

    let result = prepare_in(
        &content,
        WallpaperType::Video,
        Path::new("C:/lib/t"),
        "C:/elsewhere/clip.mp4",
    );

    assert!(matches!(result, Err(PageError::OutsideTheme)), "{result:?}");
    assert!(!content.exists());
}

#[test]
fn the_video_page_has_the_legacy_functions_and_reads_its_media_from_the_query() {
    let html = GeneratedPage::Video.html();

    for name in [
        "SucrosePropertyListener",
        "SucrosePlay",
        "SucrosePause",
        "SucroseLoopMode",
        "SucroseStretchMode",
        "SucroseVolume",
    ] {
        assert!(
            html.contains(&format!("function {name}(")) || html.contains(&format!("function {name} (")),
            "{name}"
        );
    }
    assert!(html.contains("new URLSearchParams(location.search)"));
    assert!(!html.contains("innerHTML"), "the media path is never written as markup");
    assert!(
        !html.contains("http:") && !html.contains("https:"),
        "the page loads nothing from the network"
    );
}
