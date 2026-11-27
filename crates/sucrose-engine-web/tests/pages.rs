//! The generated pages: their addresses, their files in the content folder and what they contain.

use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

use serde_json::Value;
use sucrose_core::{
    properties::{ControlKind, SucroseProperties, parse_properties},
    settings::Stretch,
    theme::WallpaperType,
};
use sucrose_engine_web::{
    args::WebEngine,
    js::{self, KIND_ENGINE, KIND_PROPERTY},
    page::{Complaints, PageKind, PageState, page_calls},
    pages::{
        GeneratedPage, PageError, builtin_set, effective_volume, engine_volume_level, ensure_file, ensure_file_with,
        ensure_written, generated_url, loop_script, prepare_in, shuffle_script, stretch_script, volume_script,
    },
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
fn generated_pages_exist_for_gif_video_and_youtube_themes_only_so_far() {
    assert_eq!(
        GeneratedPage::for_kind(WallpaperType::Video),
        Some(GeneratedPage::Video)
    );
    assert_eq!(GeneratedPage::for_kind(WallpaperType::Gif), Some(GeneratedPage::Gif));
    assert_eq!(
        GeneratedPage::for_kind(WallpaperType::YouTube),
        Some(GeneratedPage::YouTube)
    );
    assert_eq!(GeneratedPage::YouTube.file_name(), "YouTubeContent.html");
    for kind in [WallpaperType::Web, WallpaperType::Url, WallpaperType::Application] {
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

    assert_eq!(
        first,
        [GeneratedPage::Video, GeneratedPage::Gif, GeneratedPage::YouTube]
    );
    assert_eq!(written, GeneratedPage::Video.html());
    assert_eq!(
        std::fs::read_to_string(content.join("YouTubeContent.html")).expect("youtube page exists"),
        GeneratedPage::YouTube.html()
    );
    assert_eq!(
        std::fs::read_to_string(content.join("GifContent.html")).expect("gif page exists"),
        GeneratedPage::Gif.html()
    );
    assert!(second.is_empty(), "an identical page is not written again");
    assert_eq!(before, after, "an identical page is not touched");
}

#[test]
fn a_different_page_is_replaced_through_a_temporary_file_and_no_temporary_file_stays() {
    let scratch = Scratch::new();
    std::fs::write(scratch.0.join("VideoContent.html"), "old page").expect("old page");

    let written = ensure_written(&scratch.0).expect("replace");

    assert_eq!(
        written,
        [GeneratedPage::Video, GeneratedPage::Gif, GeneratedPage::YouTube]
    );
    assert_eq!(
        std::fs::read_to_string(scratch.0.join("VideoContent.html")).expect("page"),
        GeneratedPage::Video.html()
    );
    let mut names: Vec<String> = std::fs::read_dir(&scratch.0)
        .expect("list")
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    names.sort();
    assert_eq!(
        names,
        ["GifContent.html", "VideoContent.html", "YouTubeContent.html"],
        "no temporary file is left"
    );
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

#[test]
fn a_gif_address_points_at_the_gif_page_with_the_encoded_source() {
    let url = generated_url(GeneratedPage::Gif, Path::new("C:/lib/t"), "C:/lib/t/animation.gif").expect("inside");

    assert_eq!(
        url,
        "https://sucrose.wallpaper.engine/GifContent.html?src=animation.gif"
    );
}

#[test]
fn the_gif_page_has_every_legacy_function_and_its_decoder_with_the_license_notice() {
    let html = GeneratedPage::Gif.html();

    for name in [
        "SucrosePropertyListener",
        "SucroseStretchMode",
        "SucroseLoopMode",
        "SucroseSpeedRate",
        "SucrosePlay",
        "SucrosePause",
        "SucrosePlayPause",
        "SucroseVolume",
    ] {
        assert!(html.contains(&format!("function {name}(")), "{name}");
    }
    assert!(
        html.contains("setTimeout(SucrosePlay,1500)"),
        "the legacy delayed start"
    );
    assert!(html.contains("new URLSearchParams(location.search)"));
    assert!(html.contains("The MIT License (MIT)") && html.contains("Copyright (c) 2015 Matt Way"));
    assert!(!html.contains("innerHTML"), "the media path is never written as markup");
    assert!(!html.contains("{Source}"), "the legacy placeholder is gone");
    assert!(
        !html.contains("http:") && !html.contains("https:"),
        "the page loads nothing from the network"
    );
}

#[test]
fn a_gif_theme_prepares_the_gif_page() {
    let scratch = Scratch::new();
    let theme = scratch.0.join("theme");

    let url = prepare_in(
        &scratch.0.join("Content"),
        WallpaperType::Gif,
        &theme,
        &theme.join("animation.gif").to_string_lossy(),
    )
    .expect("prepared");

    assert_eq!(
        url.as_deref(),
        Some("https://sucrose.wallpaper.engine/GifContent.html?src=animation.gif")
    );
}

#[test]
fn a_youtube_address_carries_the_ids_of_the_theme_source() {
    let theme = Path::new("C:/lib/t");
    let url = |source: &str| generated_url(GeneratedPage::YouTube, theme, source);

    assert_eq!(
        url("https://www.youtube.com/watch?v=znNllvBt_Jg").expect("a video"),
        "https://sucrose.wallpaper.engine/YouTubeContent.html?v=znNllvBt_Jg"
    );
    assert_eq!(
        url("https://www.youtube.com/watch?v=abc&list=PL-1_x").expect("both"),
        "https://sucrose.wallpaper.engine/YouTubeContent.html?v=abc&list=PL-1_x"
    );
    assert_eq!(
        url("https://www.youtube.com/playlist?list=PL-1_x").expect("a list"),
        "https://sucrose.wallpaper.engine/YouTubeContent.html?list=PL-1_x"
    );
    for refused in [
        "https://www.youtube.com/",
        "https://www.youtube.com/watch?v=a%20b",
        "https://www.youtube.com/watch?v=a&v=b",
        "",
    ] {
        assert!(
            matches!(url(refused), Err(PageError::NoYouTubeId)),
            "{refused:?} carries no usable id"
        );
    }
}

#[test]
fn a_youtube_theme_prepares_the_youtube_page_and_a_bad_address_writes_nothing() {
    let scratch = Scratch::new();
    let theme = scratch.0.join("theme");
    let content = scratch.0.join("Content");

    let refused = prepare_in(&content, WallpaperType::YouTube, &theme, "https://www.youtube.com/");
    assert!(matches!(refused, Err(PageError::NoYouTubeId)), "{refused:?}");
    assert!(!content.exists(), "nothing is written for a refused address");

    let url = prepare_in(
        &content,
        WallpaperType::YouTube,
        &theme,
        "https://www.youtube.com/watch?v=znNllvBt_Jg",
    )
    .expect("prepared");

    assert_eq!(
        url.as_deref(),
        Some("https://sucrose.wallpaper.engine/YouTubeContent.html?v=znNllvBt_Jg")
    );
    assert!(content.join("YouTubeContent.html").is_file());
}

#[test]
fn the_youtube_page_has_the_legacy_functions_the_legacy_player_vars_and_only_two_addresses() {
    let html = GeneratedPage::YouTube.html();

    for name in [
        "onYouTubeIframeAPIReady",
        "onPlayerReady",
        "onPlayerStateChange",
        "onPlayerError",
        "SucrosePropertyListener",
        "playFirst",
        "playVideo",
        "pauseVideo",
        "setVolume",
        "setShuffle",
        "forceReload",
        "setLoop",
        "checkVideoEnded",
        "checkPlayingStatus",
        "toggleCaptions",
        "toggleAudioTrack",
        "hideIframeElement",
        "toggleFullScreen",
        "SucrosePause",
        "SucrosePlay",
        "SucroseVolume",
        "SucroseStretchMode",
    ] {
        assert!(
            html.contains(&format!("function {name}(")) || html.contains(&format!("function {name} (")),
            "{name}"
        );
    }
    for player_var in [
        "autoplay: 1",
        "loop: 1",
        "controls: 0",
        "disablekb: 1",
        "modestbranding: 1",
        "fs: 0",
        "rel: 0",
        "iv_load_policy: 3",
        "playsinline: 0",
        "cc_load_policy: 0",
        "version: 3",
        "showinfo: 0",
        "suggestedQuality: \"highres\"",
        "origin: \"https://sucrose.wallpaper.engine/\"",
        "widget_referrer: \"strict-origin-when-cross-origin\"",
    ] {
        assert!(html.contains(player_var), "{player_var}");
    }
    assert!(html.contains("new URLSearchParams(location.search)"));
    assert!(!html.contains("innerHTML"), "the ids are never written as markup");
    // The only addresses in the page: the public IFrame API script and the virtual origin of the player.
    let mut addresses = Vec::new();
    for scheme in ["https://", "http://"] {
        for (at, _) in html.match_indices(scheme) {
            let end = html[at..]
                .find(['"', '\'', ' ', '<', ')'])
                .map_or(html.len(), |length| at + length);
            addresses.push(&html[at..end]);
        }
    }
    addresses.sort_unstable();
    addresses.dedup();
    assert_eq!(
        addresses,
        [
            "https://sucrose.wallpaper.engine/",
            "https://www.youtube.com/iframe_api"
        ]
    );
    assert!(
        !html.to_lowercase().contains("apikey") && !html.contains("key="),
        "no API key"
    );
}

/// YouTube's embed draws its controls (the title, play/pause, More videos, the logo, the time and the progress bar) in
/// layers next to the player (`#player-controls`, `#bottom-sheet-wrapper`), outside `#movie_player`, so the legacy pass
/// that hid only the inside of `#movie_player` left them on the wallpaper (04-37 D7, debug session
/// youtube-player-ui-visible). The pass also hides every sibling on the way from the player up to the body, runs at once
/// and after every change of the frame's tree, and waits while the frame has no player yet.
#[test]
fn the_youtube_page_hides_the_controls_next_to_the_player_not_only_inside_it() {
    let html = GeneratedPage::YouTube.html();
    let function = |name: &str| {
        let start = html
            .find(&format!("function {name}("))
            .unwrap_or_else(|| panic!("{name} exists"));
        let end = html[start..].find("\n}\n").map_or(html.len(), |length| start + length);
        &html[start..end]
    };

    let hide = function("hideIframeElement");
    assert!(
        hide.contains("SucroseHidePlayerSiblings(e);"),
        "the pass hides what sits next to the player"
    );
    assert!(
        hide.contains("if (!e) { return; }"),
        "no TypeError while the frame has no player yet"
    );
    assert!(hide.contains("new MutationObserver(hide).observe(t, { childList: true, subtree: true });"));
    assert!(
        hide.trim_end().ends_with("hide();"),
        "the pass also runs at once, not only after the next change"
    );
    // The legacy rules for the inside of the player stay: everything but the video is hidden, the captions are kept.
    assert!(hide.contains(r#":scope *:not(.html5-video-container):not(.video-stream.html5-main-video)"#));
    assert!(hide.contains(r#"'.ytp-caption-window-container, .ytp-caption-segment, [role="status"], [aria-live]'"#));

    let siblings = function("SucroseHidePlayerSiblings");
    assert!(
        siblings.contains("node = node.parentElement") && siblings.contains("node !== body"),
        "every level from the player up to the body"
    );
    assert!(siblings.contains("for (const sibling of node.parentElement.children)"));
    assert!(
        siblings.contains("sibling !== node"),
        "the player and what holds it stay"
    );
    assert!(siblings.contains(r#"sibling.style.setProperty("display", "none", "important")"#));
    assert!(siblings.contains("SCRIPT|STYLE"), "scripts and styles are left alone");
    // YouTube resets the inline style of its controls while it starts (two frames of the title on CEF): the hidden
    // elements are also marked, and the Sucrose style sheet hides the mark.
    assert!(siblings.contains(r#"sibling.setAttribute("data-sucrose-hidden", "")"#));
    assert!(hide.contains(r#"e.textContent = "[data-sucrose-hidden]{display:none !important}@keyframes flip{"#));
}

#[test]
fn the_youtube_engine_scripts_are_the_legacy_calls() {
    assert_eq!(loop_script(GeneratedPage::YouTube, true), "setLoop(true);");
    assert_eq!(loop_script(GeneratedPage::YouTube, false), "setLoop(false);");
    assert_eq!(
        shuffle_script(GeneratedPage::YouTube, true).as_deref(),
        Some("setShuffle(true);")
    );
    assert_eq!(
        shuffle_script(GeneratedPage::YouTube, false).as_deref(),
        Some("setShuffle(false);")
    );
    assert_eq!(
        shuffle_script(GeneratedPage::Video, true),
        None,
        "only YouTube has a shuffle"
    );
    assert_eq!(shuffle_script(GeneratedPage::Gif, true), None);
    assert_eq!(volume_script(35), "SucroseVolume(35);");
}

#[test]
fn a_youtube_page_gets_the_set_then_volume_loop_shuffle_and_stretch_and_takes_no_input() {
    let theme = Scratch::new();
    let state = scratch_state();
    state.set_volume(40);
    state.set_looping(true);
    state.set_shuffle(false);
    let kind = PageKind::for_wallpaper(WallpaperType::YouTube, WebEngine::Cef);

    assert_eq!(kind.generated(), Some(GeneratedPage::YouTube));
    assert!(!kind.takes_input(), "legacy: YouTube gets no input");
    let calls = page_calls(kind, &theme.0, 200, &state, &Complaints::default());

    let property_count = SET_KEYS.len();
    let keys: Vec<String> = calls.iter().map(|call| call.key.clone()).collect();
    assert_eq!(calls.len(), property_count + 4, "{keys:?}");
    assert_eq!(
        &keys[property_count..],
        ["volume", "loop", "shuffle", "stretch"],
        "the engine values run once after the set"
    );
    let scripts: Vec<&str> = calls[property_count..]
        .iter()
        .map(|call| call.script.as_str())
        .collect();
    assert_eq!(
        scripts[..3],
        ["SucroseVolume(40);", "setLoop(true);", "setShuffle(false);"]
    );
    assert!(calls[property_count..].iter().all(|call| call.kind == KIND_ENGINE));
}

/// The keys of the legacy CEF and WebView sets in their legacy order (SE/Manage/Internal.cs), with the D-06 volume right
/// after `muted`.
const SET_KEYS: [&str; 16] = [
    "only",
    "scale",
    "saturate",
    "hue-rotate",
    "brightness",
    "contrast",
    "blur",
    "grayscale",
    "sepia",
    "invert",
    "mirror",
    "animation",
    "playbackRate",
    "muted",
    "volume",
    "caption",
];

fn set_keys(set: &SucroseProperties) -> Vec<&str> {
    set.list.iter().map(|(key, _)| key.as_str()).collect()
}

fn scratch_state() -> PageState {
    PageState::new(false, 100, true, true, Stretch::UniformToFill, "Dark")
}

#[test]
fn both_built_in_sets_parse_and_hold_the_legacy_keys_in_the_legacy_order_with_the_volume_after_muted() {
    for (engine, label) in [(WebEngine::Cef, "onlyCef"), (WebEngine::WebView2, "onlyWeb")] {
        let set = builtin_set(engine);

        let mut expected = SET_KEYS.to_vec();
        expected[0] = label;
        assert_eq!(set_keys(set), expected, "{engine:?}");
        assert_eq!(set.listener, "SucrosePropertyListener('{0}', {1});");
        assert!(!set.list.is_empty(), "a built-in set is never empty");
    }
}

#[test]
fn the_volume_is_a_slider_from_0_to_100_in_steps_of_1_that_starts_at_full_volume() {
    for engine in [WebEngine::Cef, WebEngine::WebView2] {
        let (_, volume) = builtin_set(engine)
            .list
            .iter()
            .find(|(key, _)| key == "volume")
            .expect("the volume control");

        let ControlKind::Slider { min, max, step, value } = volume.kind else {
            panic!("the volume is not a slider: {:?}", volume.kind);
        };
        assert_eq!((min, max, step, value), (0.0, 100.0, 1.0, 100.0), "{engine:?}");
    }
}

#[test]
fn the_volume_has_an_english_text_and_help_in_every_culture_of_the_set() {
    for engine in [WebEngine::Cef, WebEngine::WebView2] {
        let set = builtin_set(engine);
        let (_, volume) = set.list.iter().find(|(key, _)| key == "volume").expect("volume");
        let (text, help) = (
            volume.text.as_deref().expect("a text key"),
            volume.help.as_deref().expect("a help key"),
        );

        assert!(!set.localization.is_empty(), "{engine:?}: at least the English table");
        for (culture, texts) in &set.localization {
            for key in [text, help] {
                let found = texts.iter().find(|(name, _)| name == key);
                assert!(
                    found.is_some_and(|(_, shown)| !shown.is_empty()),
                    "{engine:?} {culture}: no text for {key}"
                );
            }
        }
    }
}

#[test]
fn keys_are_compared_byte_for_byte_as_the_legacy_json_spells_them() {
    let keys = set_keys(builtin_set(WebEngine::Cef));

    for exact in ["hue-rotate", "playbackRate", "muted", "volume"] {
        assert!(keys.contains(&exact), "{exact}");
    }
    for wrong in ["hue_rotate", "playbackrate", "Muted", "Volume", "mute"] {
        assert!(!keys.contains(&wrong), "{wrong}");
    }
}

#[test]
fn the_volume_key_collides_with_no_legacy_key_of_the_set() {
    for engine in [WebEngine::Cef, WebEngine::WebView2] {
        let keys = set_keys(builtin_set(engine));
        let mut seen = std::collections::HashSet::new();
        for key in keys {
            assert!(seen.insert(key.to_ascii_lowercase()), "{engine:?}: {key} repeats a key");
        }
    }
}

#[test]
fn the_calls_of_a_set_come_one_per_control_in_the_order_of_its_list_with_its_listener_format() {
    let theme = Scratch::new();

    let calls = js::property_calls_of(builtin_set(WebEngine::Cef), &theme.0);

    let keys: Vec<&str> = calls.iter().map(|call| call.key.as_str()).collect();
    let mut expected = SET_KEYS.to_vec();
    expected[0] = "onlyCef";
    assert_eq!(keys, expected);
    assert!(calls.iter().all(|call| call.kind == KIND_PROPERTY));
    let volume = calls.iter().find(|call| call.key == "volume").expect("volume call");
    assert!(
        volume.script.starts_with("SucrosePropertyListener('volume', {\r\n"),
        "{}",
        volume.script
    );
    assert!(volume.script.contains("\"type\": \"slider\""));
    assert!(volume.script.contains("\"value\": 100.0") || volume.script.contains("\"value\": 100"));
}

fn slider_set(value: f64) -> SucroseProperties {
    parse_properties(&format!(
        r#"{{"PropertyListener":"f('{{0}}', {{1}});","PropertyList":{{"s":{{"type":"slider","min":-5,"max":5,"step":1,"value":{value}}}}}}}"#
    ))
    .expect("a valid set")
}

fn slider_value_of(set: &SucroseProperties) -> f64 {
    let calls = js::property_calls_of(set, Path::new("."));
    let script = &calls.first().expect("one call").script;
    // The script is `f('s', <control>);`: the control is the object from the first brace to the last.
    let json = &script[script.find('{').expect("control")..=script.rfind('}').expect("end")];
    let control: Value = serde_json::from_str(json).expect("control JSON");
    control["value"].as_f64().expect("a number")
}

#[test]
fn a_slider_value_equal_to_its_bounds_is_used_as_it_is_and_one_outside_is_clamped() {
    assert!((slider_value_of(&slider_set(-5.0)) - -5.0).abs() < f64::EPSILON, "min");
    assert!((slider_value_of(&slider_set(5.0)) - 5.0).abs() < f64::EPSILON, "max");
    assert!((slider_value_of(&slider_set(2.5)) - 2.5).abs() < f64::EPSILON, "inside");
    assert!(
        (slider_value_of(&slider_set(5.5)) - 5.0).abs() < f64::EPSILON,
        "above max"
    );
    assert!(
        (slider_value_of(&slider_set(-99.0)) - -5.0).abs() < f64::EPSILON,
        "below min"
    );
}

#[test]
fn a_control_without_a_value_takes_the_default_of_its_type() {
    let set = parse_properties(
        r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{"c":{"type":"checkbox"},"t":{"type":"textbox"},"s":{"type":"slider","min":0,"max":9,"step":1}}}"#,
    )
    .expect("a valid set");

    let calls = js::property_calls_of(&set, Path::new("."));

    assert_eq!(calls.len(), 3);
    assert!(calls[0].script.contains("\"value\": false"), "{}", calls[0].script);
    assert!(calls[1].script.contains("\"value\": \"\""), "{}", calls[1].script);
    assert!(
        calls[2].script.contains("\"value\": 0"),
        "a slider without a value is 0: {}",
        calls[2].script
    );
}

#[test]
fn two_controls_with_equal_values_keep_the_order_of_the_list() {
    let set = parse_properties(
        r#"{"PropertyListener":"f('{0}', {1});","PropertyList":{"z":{"type":"checkbox","value":true},"a":{"type":"checkbox","value":true},"m":{"type":"checkbox","value":true}}}"#,
    )
    .expect("a valid set");

    let keys: Vec<String> = js::property_calls_of(&set, Path::new("."))
        .into_iter()
        .map(|call| call.key)
        .collect();

    assert_eq!(keys, ["z", "a", "m"]);
}

#[test]
fn the_engine_scripts_are_the_legacy_calls() {
    assert_eq!(volume_script(0), "SucroseVolume(0);");
    assert_eq!(volume_script(100), "SucroseVolume(100);");
    assert_eq!(volume_script(250), "SucroseVolume(100);", "the level is capped at 100");
    assert_eq!(loop_script(GeneratedPage::Gif, true), "SucroseLoopMode(true);");
    assert_eq!(loop_script(GeneratedPage::Video, false), "SucroseLoopMode(false);");
    assert_eq!(
        stretch_script(GeneratedPage::Gif, Stretch::UniformToFill),
        "SucroseStretchMode('UniformToFill');"
    );
    assert_eq!(
        stretch_script(GeneratedPage::Video, Stretch::None),
        "SucroseStretchMode('None');"
    );
}

#[test]
fn the_page_volume_is_the_engine_volume_times_the_property_volume() {
    assert!((effective_volume(50, 40.0, false) - 0.2).abs() < 1e-9);
    assert!((effective_volume(100, 100.0, false) - 1.0).abs() < 1e-9);
    assert!(effective_volume(0, 100.0, false).abs() < 1e-9);
    assert!(effective_volume(50, 40.0, true).abs() < 1e-9, "muted gives 0");
    assert!(
        (effective_volume(100, 250.0, false) - 1.0).abs() < 1e-9,
        "the property is capped"
    );
    assert_eq!(engine_volume_level(50, true), 50);
    assert_eq!(
        engine_volume_level(50, false),
        0,
        "a renderer that does not play the sound sends 0"
    );
}

#[test]
fn a_generated_page_gets_the_set_then_the_engine_values_and_nothing_of_the_themes_own_files() {
    let theme = Scratch::new();
    std::fs::write(
        theme.0.join("SucroseProperties.json"),
        r#"{"PropertyListener":"own('{0}', {1});","PropertyList":{"mine":{"type":"checkbox","value":true}}}"#,
    )
    .expect("the theme's own file");
    std::fs::write(
        theme.0.join("SucroseCompatible.json"),
        r#"{"LoopMode":"L('{0}');","VolumeLevel":"V({0});"}"#,
    )
    .expect("the compatible file");
    let state = scratch_state();
    state.set_volume(70);
    state.set_looping(false);
    state.set_stretch(Stretch::Fill);
    let kind = PageKind::Generated(GeneratedPage::Video, builtin_set(WebEngine::Cef));

    let calls = page_calls(kind, &theme.0, 200, &state, &Complaints::default());

    let keys: Vec<String> = calls.iter().map(|call| format!("{}:{}", call.kind, call.key)).collect();
    let property_count = SET_KEYS.len();
    assert_eq!(calls.len(), property_count + 3, "{keys:?}");
    assert!(
        keys.iter()
            .all(|key| !key.contains("mine") && !key.starts_with("compatible")),
        "{keys:?}"
    );
    assert_eq!(
        &keys[property_count..],
        ["engine:volume", "engine:loop", "engine:stretch"],
        "the current values run once, after the set"
    );
    let scripts: Vec<&str> = calls[property_count..]
        .iter()
        .map(|call| call.script.as_str())
        .collect();
    assert_eq!(
        scripts,
        [
            "SucroseVolume(70);",
            "SucroseLoopMode(false);",
            "SucroseStretchMode('Fill');"
        ]
    );
    assert!(calls[property_count..].iter().all(|call| call.kind == KIND_ENGINE));
}

#[test]
fn a_generated_page_that_did_not_load_gets_nothing_and_a_paused_one_ends_with_the_pause() {
    let theme = Scratch::new();
    let state = scratch_state();
    let kind = PageKind::Generated(GeneratedPage::Gif, builtin_set(WebEngine::WebView2));

    assert!(page_calls(kind, &theme.0, 404, &state, &Complaints::default()).is_empty());

    state.set_paused(true);
    let calls = page_calls(kind, &theme.0, 200, &state, &Complaints::default());
    assert_eq!(calls.last().expect("calls").kind, js::KIND_PAUSE);
}

#[test]
fn a_follower_tells_its_generated_page_volume_zero_and_a_master_the_wallpaper_volume() {
    let state = scratch_state();
    state.set_volume(80);

    assert_eq!(
        sucrose_engine_web::page::volume_call(&state).script,
        "SucroseVolume(80);"
    );

    state.set_audio(false);
    assert_eq!(
        sucrose_engine_web::page::volume_call(&state).script,
        "SucroseVolume(0);"
    );

    state.set_audio(true);
    assert_eq!(
        sucrose_engine_web::page::volume_call(&state).script,
        "SucroseVolume(80);"
    );
}

/// The plain rules of the first `<style>` block of `html` as (selectors, declarations); `@` blocks are skipped.
fn style_rules(html: &str) -> Vec<(Vec<String>, String)> {
    let start = html.find("<style>").expect("a style block") + "<style>".len();
    let end = start + html[start..].find("</style>").expect("the end of the style block");
    let mut css = html[start..end].to_owned();
    while let Some(open) = css.find("/*") {
        let close = css[open..].find("*/").map_or(css.len(), |at| open + at + 2);
        css.replace_range(open..close, "");
    }
    let mut rules = Vec::new();
    let mut rest = css.as_str();
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim().to_owned();
        let mut depth = 0;
        let mut close = open;
        for (at, c) in rest[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + at;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !selector.starts_with('@') {
            let selectors = selector.split(',').map(|part| part.trim().to_owned()).collect();
            rules.push((selectors, rest[open + 1..close].replace([' ', '\n'], "")));
        }
        rest = &rest[close + 1..];
    }
    rules
}

#[test]
fn a_page_whose_scale_property_transforms_its_body_keeps_the_body_as_tall_as_the_viewport() {
    // The built-in "scale" property (default 0, sent at every start) sets body.style.transform; a transformed body holds
    // the page's position:fixed media, so a body without a height would leave the media 0 px tall (04-37 D5).
    for page in [GeneratedPage::Video, GeneratedPage::Gif] {
        let html = page.html();
        let compact: String = html.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains("case\"scale\":body.style.transform"),
            "{} no longer scales its body; revisit this test",
            page.file_name()
        );
        let sized = |element: &str| {
            style_rules(html).iter().any(|(selectors, declarations)| {
                selectors.iter().any(|selector| selector == element) && declarations.contains("height:100%")
            })
        };
        assert!(
            sized("html") && sized("body"),
            "{}: html and body must be height:100%",
            page.file_name()
        );
    }
}
