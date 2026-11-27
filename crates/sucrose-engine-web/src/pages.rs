//! The generated pages of the web engines (WAPI-08, D-14): the legacy `VideoContent.html` and `GifContent.html` that
//! play a Video or a Gif theme inside a web engine. The pages are embedded in the binary, written into
//! `<cache_dir>/Content` when they are missing or differ (the folder the virtual origin serves after the theme folder),
//! and loaded as `https://sucrose.wallpaper.engine/<Name>Content.html?src=<media path relative to the theme folder>`. The
//! page reads the path from `location.search` and requests the media from the same origin, so a video seeks through
//! `Range` like any other theme file. No OS code and no unsafe: both web engines call these functions.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sucrose_core::{
    properties::{SucroseProperties, parse_properties},
    settings::Stretch,
    theme::WallpaperType,
};

use crate::{args::WebEngine, origin::ORIGIN};

/// Everything but the unreserved characters is escaped, so the `/` of a relative path travels as `%2F` inside the query.
const QUERY: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

/// A page Sucrose generates for a wallpaper type that has no page of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedPage {
    /// `VideoContent.html`: one `<video>` that fills the surface.
    Video,
    /// `GifContent.html`: the legacy canvas player with its embedded GIF decoder.
    Gif,
}

impl GeneratedPage {
    /// Every generated page, in the order they are written.
    pub const ALL: &'static [Self] = &[Self::Video, Self::Gif];

    /// The page that plays a wallpaper of `kind`; `None` for a type that is not a generated page.
    #[must_use]
    pub fn for_kind(kind: WallpaperType) -> Option<Self> {
        match kind {
            WallpaperType::Video => Some(Self::Video),
            WallpaperType::Gif => Some(Self::Gif),
            _ => None,
        }
    }

    /// The file name in the content folder and in the page address.
    #[must_use]
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Video => "VideoContent.html",
            Self::Gif => "GifContent.html",
        }
    }

    /// The page text embedded in this binary.
    #[must_use]
    pub const fn html(self) -> &'static str {
        match self {
            Self::Video => include_str!("pages/VideoContent.html"),
            Self::Gif => include_str!("pages/GifContent.html"),
        }
    }
}

/// Why a generated page could not be prepared.
#[derive(Debug, thiserror::Error)]
pub enum PageError {
    /// The source names no file.
    #[error("the media source is empty")]
    Empty,
    /// The source is not inside the theme folder.
    #[error("the media source is not inside the theme folder")]
    OutsideTheme,
    /// There is no cache folder to write the pages into.
    #[error("there is no cache folder for the generated pages")]
    NoCacheFolder,
    /// A page file could not be written.
    #[error("cannot write {file} into the content folder: {source}")]
    Write {
        /// The page file name.
        file: &'static str,
        /// The reason.
        source: io::Error,
    },
}

/// A path with forward slashes and without the verbatim prefix, for comparing and encoding.
fn slashes(path: &str) -> String {
    let path = path.replace('\\', "/");
    path.strip_prefix("//?/").map_or(path.clone(), str::to_owned)
}

/// Whether a slash-separated path is rooted or names a drive.
fn is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with('/') || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

/// The path of `source` relative to `theme_dir`, with forward slashes. A source that is already relative is taken as it
/// is; an absolute one must be inside the theme folder. A `..` segment is refused in both.
fn relative_source(theme_dir: &Path, source: &str) -> Result<String, PageError> {
    let source = slashes(source);
    let theme = slashes(&theme_dir.to_string_lossy());
    let theme = theme.trim_end_matches('/');
    let relative = if is_absolute(&source) {
        match source.get(..theme.len()) {
            Some(head)
                if !theme.is_empty() && head.eq_ignore_ascii_case(theme) && source[theme.len()..].starts_with('/') =>
            {
                &source[theme.len()..]
            }
            _ => return Err(PageError::OutsideTheme),
        }
    } else {
        source.as_str()
    };
    let mut segments = Vec::new();
    for segment in relative
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
    {
        if segment == ".." {
            return Err(PageError::OutsideTheme);
        }
        segments.push(segment);
    }
    if segments.is_empty() {
        return Err(PageError::Empty);
    }
    Ok(segments.join("/"))
}

/// The address a web engine loads for `page` to play `source`: the origin, the page and the media path relative to the
/// theme folder, percent-encoded as one query value.
///
/// # Errors
/// [`PageError::OutsideTheme`] for a source outside `theme_dir` (or with a `..` segment) and [`PageError::Empty`] for
/// a source with no file in it.
pub fn generated_url(page: GeneratedPage, theme_dir: &Path, source: &str) -> Result<String, PageError> {
    let relative = relative_source(theme_dir, source)?;
    let mut url = format!("{ORIGIN}{}?src=", page.file_name());
    url.extend(utf8_percent_encode(&relative, QUERY));
    Ok(url)
}

/// Makes sure `path` holds exactly `bytes`, with `rename` as the way a finished temporary file replaces it. An identical
/// file is not touched (`Ok(false)`). A different or missing one is written to a temporary file next to it and renamed
/// over it (`Ok(true)`), so a reader never sees half a page. When the rename is refused (another renderer has the
/// page open) the temporary file is removed, and if the file now holds the same bytes that is success too: two renderers
/// writing the same page at once is normal.
///
/// # Errors
/// The I/O error of the folder, the temporary file or a rename that left the file different.
pub fn ensure_file_with(
    path: &Path,
    bytes: &[u8],
    rename: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<bool> {
    if fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(false);
    }
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    fs::write(&temporary, bytes)?;
    match rename(&temporary, path) {
        Ok(()) => Ok(true),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            if fs::read(path).is_ok_and(|current| current == bytes) {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

/// [`ensure_file_with`] with `std::fs::rename`.
///
/// # Errors
/// See [`ensure_file_with`].
pub fn ensure_file(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    ensure_file_with(path, bytes, &|from, to| fs::rename(from, to))
}

/// Writes every generated page into `content_dir` that is missing or differs from the embedded text, and returns the
/// pages it wrote (empty when the folder was up to date). Only the embedded bytes are ever written (T-04-71).
///
/// # Errors
/// [`PageError::Write`] for the first page that could not be written.
pub fn ensure_written(content_dir: &Path) -> Result<Vec<GeneratedPage>, PageError> {
    let mut written = Vec::new();
    for page in GeneratedPage::ALL {
        let file = page.file_name();
        match ensure_file(&content_dir.join(file), page.html().as_bytes()) {
            Ok(true) => written.push(*page),
            Ok(false) => {}
            Err(source) => return Err(PageError::Write { file, source }),
        }
    }
    Ok(written)
}

/// What a web engine loads for a wallpaper of `kind` playing `source`, with the pages written into `content_dir` first:
/// the address of the generated page, or `None` when `kind` is not a generated page (the engine then loads the
/// theme's own page).
///
/// # Errors
/// A page that cannot be written or a source that is not usable.
pub fn prepare_in(
    content_dir: &Path,
    kind: WallpaperType,
    theme_dir: &Path,
    source: &str,
) -> Result<Option<String>, PageError> {
    let Some(page) = GeneratedPage::for_kind(kind) else {
        return Ok(None);
    };
    let url = generated_url(page, theme_dir, source)?;
    ensure_written(content_dir)?;
    Ok(Some(url))
}

/// [`prepare_in`] with the content folder of this installation (`<cache_dir>/Content`, the folder the engines serve).
///
/// # Errors
/// [`PageError::NoCacheFolder`] when there is no cache folder, otherwise see [`prepare_in`].
pub fn prepare(kind: WallpaperType, theme_dir: &Path, source: &str) -> Result<Option<String>, PageError> {
    if GeneratedPage::for_kind(kind).is_none() {
        return Ok(None);
    }
    let content = sucrose_core::paths::cache_dir()
        .ok_or(PageError::NoCacheFolder)?
        .join("Content");
    prepare_in(&content, kind, theme_dir, source)
}

/// The built-in property set the generated pages of `engine` use (PROP-05): the legacy CEF or WebView set decoded from
/// `SE/Manage/Internal.cs`, with the D-06 `volume` slider right after `muted`. A Gif or Video theme on a web engine
/// always gets this set and never its own `SucroseProperties.json`, as in the legacy app.
///
/// # Panics
/// Never for the files in this build: a test parses both, so an invalid set cannot ship.
#[must_use]
pub fn builtin_set(engine: WebEngine) -> &'static SucroseProperties {
    static CEF: OnceLock<SucroseProperties> = OnceLock::new();
    static WEBVIEW: OnceLock<SucroseProperties> = OnceLock::new();
    let (slot, text) = match engine {
        WebEngine::Cef => (&CEF, include_str!("pages/properties-cef.json")),
        WebEngine::WebView2 => (&WEBVIEW, include_str!("pages/properties-webview.json")),
    };
    slot.get_or_init(|| {
        parse_properties(text).expect("the embedded built-in property set is valid JSON (a test parses it)")
    })
}

/// The level `SucroseVolume` is told: the wallpaper volume (0..=100) while this renderer plays the sound, 0 when it does
/// not (a follower, D-02, or any renderer told `set_audio(false)`).
#[must_use]
pub fn engine_volume_level(volume: u8, audio: bool) -> u8 {
    if audio { volume.min(100) } else { 0 }
}

/// The script that hands a generated page the engine's volume level (0..=100).
#[must_use]
pub fn volume_script(level: u8) -> String {
    format!("SucroseVolume({});", level.min(100))
}

/// The script that sets whether the media loops. Both pages answer `SucroseLoopMode(<bool>)`, as the legacy GIF page does.
#[must_use]
pub fn loop_script(_page: GeneratedPage, on: bool) -> String {
    format!("SucroseLoopMode({on});")
}

/// The script that fits the media to the surface by the legacy stretch name. Both pages answer
/// `SucroseStretchMode('<Name>')`, as the legacy GIF page does.
#[must_use]
pub fn stretch_script(_page: GeneratedPage, stretch: Stretch) -> String {
    format!("SucroseStretchMode('{}');", stretch.legacy_name())
}

/// The volume a generated page plays at, as a fraction of full scale (D-06): the engine's level (0..=100) times the
/// property's (0..=100) over 10 000, and 0 when the property `muted` is on. The pages compute the same product in
/// `SucroseApplyVolume`; this is the rule written once where it can be tested.
#[must_use]
pub fn effective_volume(engine_level: u8, property: f64, muted: bool) -> f64 {
    if muted || !property.is_finite() {
        return 0.0;
    }
    (f64::from(engine_level.min(100)) / 100.0) * (property.clamp(0.0, 100.0) / 100.0)
}
