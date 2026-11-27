//! Theme model shared with the C# app: the wallpaper type, `SucroseInfo.json` (read, written and checked) and the gate
//! that decides whether the selected theme may start (D-13).

use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::{Map, Value};

use crate::version::LegacyVersion;

/// File name of a theme's metadata inside its folder.
pub const INFO_FILE: &str = "SucroseInfo.json";

/// Kind of wallpaper a theme contains, with the legacy ordinals (0 Gif .. 5 Application).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WallpaperType {
    /// Animated GIF.
    Gif,
    /// Web page by address.
    Url,
    /// Local web wallpaper.
    Web,
    /// Video file.
    Video,
    /// YouTube video.
    YouTube,
    /// External application.
    Application,
}

impl WallpaperType {
    /// Type for a legacy ordinal, or `None` outside 0..=5.
    pub fn from_legacy_int(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Gif),
            1 => Some(Self::Url),
            2 => Some(Self::Web),
            3 => Some(Self::Video),
            4 => Some(Self::YouTube),
            5 => Some(Self::Application),
            _ => None,
        }
    }

    /// Type for a legacy enum name (case-sensitive, as `Enum.Parse`) or a numeric string.
    pub fn from_legacy_name(name: &str) -> Option<Self> {
        match name {
            "Gif" => Some(Self::Gif),
            "Url" => Some(Self::Url),
            "Web" => Some(Self::Web),
            "Video" => Some(Self::Video),
            "YouTube" => Some(Self::YouTube),
            "Application" => Some(Self::Application),
            other => other.parse().ok().and_then(Self::from_legacy_int),
        }
    }

    /// Type for a name in any letter case (or a numeric string). Newtonsoft's enum reader, which parses
    /// `SucroseInfo.json`, ignores case, unlike `Enum.Parse` as used for the settings.
    pub fn from_name_ignore_case(name: &str) -> Option<Self> {
        const NAMES: [(&str, WallpaperType); 6] = [
            ("Gif", WallpaperType::Gif),
            ("Url", WallpaperType::Url),
            ("Web", WallpaperType::Web),
            ("Video", WallpaperType::Video),
            ("YouTube", WallpaperType::YouTube),
            ("Application", WallpaperType::Application),
        ];
        NAMES
            .iter()
            .find(|(known, _)| known.eq_ignore_ascii_case(name))
            .map(|(_, kind)| *kind)
            .or_else(|| Self::from_legacy_name(name))
    }

    /// The legacy ordinal, which is how the type is written.
    pub const fn legacy_int(self) -> i64 {
        match self {
            Self::Gif => 0,
            Self::Url => 1,
            Self::Web => 2,
            Self::Video => 3,
            Self::YouTube => 4,
            Self::Application => 5,
        }
    }
}

fn deserialize_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<WallpaperType, D::Error> {
    let parsed = match Value::deserialize(deserializer)? {
        Value::Number(number) => number.as_i64().and_then(WallpaperType::from_legacy_int),
        Value::String(name) => WallpaperType::from_name_ignore_case(&name),
        _ => None,
    };
    // The value is not echoed: it is untrusted text and this message reaches the log.
    parsed.ok_or_else(|| de::Error::custom("unknown wallpaper type"))
}

fn serialize_type<S: Serializer>(kind: &WallpaperType, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(kind.legacy_int())
}

fn default_version() -> LegacyVersion {
    LegacyVersion::new(1, 0, 0, 0)
}

/// Title and description of a theme in one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedText {
    /// Title in that language.
    #[serde(rename = "Title")]
    pub title: String,
    /// Description in that language.
    #[serde(rename = "Description")]
    pub description: String,
}

/// The contents of `SucroseInfo.json` in the legacy field order. Fields this build does not know survive in
/// `extra` (D-007 allows new optional fields) and are written back after the known ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SucroseInfo {
    /// App version the theme was made with.
    #[serde(rename = "AppVersion", default)]
    pub app_version: LegacyVersion,
    /// Version of the theme itself.
    #[serde(rename = "Version", default = "default_version")]
    pub version: LegacyVersion,
    /// Display title.
    #[serde(rename = "Title", default)]
    pub title: String,
    /// Thumbnail file, relative to the theme folder.
    #[serde(rename = "Thumbnail", default)]
    pub thumbnail: String,
    /// Preview file, relative to the theme folder.
    #[serde(rename = "Preview", default)]
    pub preview: String,
    /// Description.
    #[serde(rename = "Description", default)]
    pub description: String,
    /// Author.
    #[serde(rename = "Author", default)]
    pub author: String,
    /// License; written as an explicit null when absent.
    #[serde(rename = "License", default)]
    pub license: Option<String>,
    /// Address or e-mail of the author.
    #[serde(rename = "Contact", default)]
    pub contact: String,
    /// Kind of wallpaper; written as its ordinal.
    #[serde(
        rename = "Type",
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    pub kind: WallpaperType,
    /// File relative to the theme folder, or an address.
    #[serde(rename = "Source")]
    pub source: String,
    /// Tags; written as an explicit null when absent.
    #[serde(rename = "Tags", default)]
    pub tags: Option<Vec<String>>,
    /// Arguments for the wallpaper; written as an explicit null when absent.
    #[serde(rename = "Arguments", default)]
    pub arguments: Option<String>,
    /// Texts per language; the C# writer emits an explicit null when there are none.
    #[serde(rename = "Localization", default)]
    pub localization: Option<BTreeMap<String, LocalizedText>>,
    /// Fields this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Why a theme's `SucroseInfo.json` could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// The theme folder has no `SucroseInfo.json`.
    #[error("{} does not exist", .0.display())]
    Missing(PathBuf),
    /// The file exists but could not be read.
    #[error("cannot read {}: {source}", .path.display())]
    Unreadable {
        /// File that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// The file is not a valid `SucroseInfo.json`.
    #[error("{} is not a valid SucroseInfo.json: {reason}", .path.display())]
    Invalid {
        /// File that failed.
        path: PathBuf,
        /// What is wrong with it, without quoting the file.
        reason: String,
    },
}

/// Reads `SucroseInfo.json` from a theme folder. The file is only opened for reading.
pub fn read_info(theme_dir: &Path) -> Result<SucroseInfo, ThemeError> {
    let path = theme_dir.join(INFO_FILE);
    let bytes = fs::read(&path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            ThemeError::Missing(path.clone())
        } else {
            ThemeError::Unreadable {
                path: path.clone(),
                source,
            }
        }
    })?;
    let text = String::from_utf8(bytes).map_err(|_| ThemeError::Invalid {
        path: path.clone(),
        reason: "the file is not UTF-8 text".to_owned(),
    })?;
    parse_info(&text).map_err(|reason| ThemeError::Invalid { path, reason })
}

/// Parses the text of a `SucroseInfo.json`, tolerating a leading byte order mark. The error names the kind of problem
/// and where, never the offending text (it ends up in logs).
pub fn parse_info(text: &str) -> Result<SucroseInfo, String> {
    serde_json::from_str(text.trim_start_matches('\u{FEFF}')).map_err(|error| {
        let (line, column) = (error.line(), error.column());
        match error.classify() {
            serde_json::error::Category::Syntax => format!("invalid JSON at line {line} column {column}"),
            serde_json::error::Category::Eof => "the JSON ends too early".to_owned(),
            serde_json::error::Category::Io => "the JSON could not be read".to_owned(),
            serde_json::error::Category::Data => {
                format!("a field is missing or has the wrong type (line {line} column {column})")
            }
        }
    })
}

/// Writes `info` to `path` the way the C# app does: 2-space indent, CRLF line endings, no byte order mark, no
/// trailing newline. Only the given path is written, never a theme folder of the user's library on its own (D-08).
pub fn write_info(path: &Path, info: &SucroseInfo) -> io::Result<()> {
    let text = serde_json::to_string_pretty(info).map_err(io::Error::other)?;
    // Line breaks inside strings are escaped, so every raw `\n` is structural.
    fs::write(path, text.replace('\n', "\r\n"))
}

/// Why the selected theme must not start. The text is one English sentence for the log and the tray tooltip; it names
/// the theme folder and the problem and never quotes file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeRejection {
    /// `Library.json` selects no theme.
    NothingSelected,
    /// The selected theme has no folder in the library.
    FolderMissing {
        /// Folder name as selected.
        folder: String,
    },
    /// `SucroseInfo.json` is missing, unreadable or invalid.
    InfoUnreadable {
        /// Folder name as selected.
        folder: String,
        /// The problem.
        reason: String,
    },
    /// The theme's type is one this build does not play yet.
    TypeNotPlayable {
        /// Folder name as selected.
        folder: String,
        /// The type it has.
        found: WallpaperType,
    },
    /// `Source` is not a regular file inside the theme folder, or (Url and YouTube themes) not an acceptable address.
    SourceInvalid {
        /// Folder name as selected.
        folder: String,
        /// The problem.
        reason: String,
    },
    /// The theme needs a newer app than this one.
    NeedsNewerApp {
        /// Folder name as selected.
        folder: String,
        /// The `AppVersion` of the theme.
        required: LegacyVersion,
        /// The version of this app.
        current: LegacyVersion,
    },
}

impl fmt::Display for ThemeRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingSelected => write!(f, "No theme is selected in the library."),
            Self::FolderMissing { folder } => write!(f, "The folder of the selected theme {folder} does not exist."),
            Self::InfoUnreadable { folder, reason } => {
                write!(
                    f,
                    "The selected theme {folder} has no usable SucroseInfo.json: {reason}."
                )
            }
            Self::TypeNotPlayable { folder, found } => {
                write!(
                    f,
                    "The selected theme {folder} is a {found:?} wallpaper, which this build does not play yet."
                )
            }
            Self::SourceInvalid { folder, reason } => {
                write!(f, "The selected theme {folder} cannot play: {reason}.")
            }
            Self::NeedsNewerApp {
                folder,
                required,
                current,
            } => write!(
                f,
                "The selected theme {folder} needs Sucrose {required} or newer; this is {current}."
            ),
        }
    }
}

impl std::error::Error for ThemeRejection {}

/// What a theme plays, decided by its type: a file inside the theme folder, or an address on the web. The two never
/// travel as one untyped string, so a later type cannot slip through the wrong gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeSource {
    /// Absolute path of the video file, image or web page, verified to be a regular file inside the theme folder.
    File(PathBuf),
    /// An absolute http or https address with a host (a Url theme, or a YouTube theme with a video or list id).
    Address(String),
}

/// The video and playlist ids of a YouTube address, as the legacy `Various.cs` reads them from the `v` and `list` query
/// parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YouTubeIds {
    /// The `v` parameter.
    pub video: Option<String>,
    /// The `list` parameter.
    pub list: Option<String>,
}

/// The ids of a YouTube `address`: `None` when it carries neither a `v` nor a `list` parameter, or when one it carries
/// is not an id (each of `[A-Za-z0-9_-]`, 1 to 64 characters; a parameter given twice is not one).
pub fn youtube_ids(address: &str) -> Option<YouTubeIds> {
    checked_youtube_ids(address).ok()
}

/// The longest address a Url or YouTube theme may carry.
const MAX_ADDRESS_BYTES: usize = 2048;

/// The longest YouTube video or playlist id.
const MAX_YOUTUBE_ID: usize = 64;

/// The value of the query parameter `key`: `None` when absent, `Some(None)` when it is given more than once (the
/// legacy reader joins the values with a comma, which is not an id), `Some(Some(value))` otherwise.
fn query_parameter<'a>(query: &'a str, key: &str) -> Option<Option<&'a str>> {
    let mut values = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .filter(|(name, _)| *name == key)
        .map(|(_, value)| value);
    let first = values.next()?;
    Some(if values.next().is_some() { None } else { Some(first) })
}

/// Whether `id` is a YouTube id the page can embed: 1 to 64 characters of `[A-Za-z0-9_-]`.
fn is_youtube_id(id: &str) -> bool {
    (1..=MAX_YOUTUBE_ID).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

/// [`youtube_ids`] with the reason it found none.
fn checked_youtube_ids(address: &str) -> Result<YouTubeIds, &'static str> {
    let query = address
        .split_once('?')
        .map_or("", |(_, rest)| rest.split('#').next().unwrap_or(""));
    let read = |key: &str| -> Result<Option<String>, &'static str> {
        match query_parameter(query, key) {
            None => Ok(None),
            Some(Some(value)) if is_youtube_id(value) => Ok(Some(value.to_owned())),
            Some(_) => Err("a YouTube id in the Source is not a valid id"),
        }
    };
    let (video, list) = (read("v")?, read("list")?);
    if video.is_none() && list.is_none() {
        return Err("the YouTube address has neither a v nor a list parameter");
    }
    Ok(YouTubeIds { video, list })
}

/// Checks the `Source` of a Url or YouTube theme and returns it unchanged: an absolute http or https address with a
/// host, at most 2048 bytes, without whitespace or control characters. Every other scheme (`file:`, `javascript:`,
/// `data:`, `about:`), a relative text and an address without a host are refused, so a theme can never make a web
/// engine open a local file or run script through its Source (T-04-72).
fn verified_address(source: &str) -> Result<String, &'static str> {
    if source.is_empty() {
        return Err("the theme has no Source");
    }
    if source.len() > MAX_ADDRESS_BYTES {
        return Err("the Source address is longer than 2048 bytes");
    }
    if source.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("the Source address contains whitespace or control characters");
    }
    let Some((scheme, rest)) = source.split_once(':') else {
        return Err("the Source is not an absolute address");
    };
    let mut scheme_chars = scheme.chars();
    let scheme_syntax = scheme_chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && scheme_chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if !scheme_syntax {
        return Err("the Source is not an absolute address");
    }
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err("the Source address is not an http or https address");
    }
    let Some(after_slashes) = rest.strip_prefix("//") else {
        return Err("the Source address has no host");
    };
    let authority = after_slashes.split(['/', '?', '#', '\\']).next().unwrap_or("");
    let host_and_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let host = match host_and_port.strip_prefix('[') {
        Some(bracketed) => bracketed.split(']').next().unwrap_or(""),
        None => host_and_port.split(':').next().unwrap_or(""),
    };
    if host.is_empty() {
        return Err("the Source address has no host");
    }
    Ok(source.to_owned())
}

/// A selected theme that passed the gate.
#[derive(Debug, Clone)]
pub struct SelectedTheme {
    /// The theme folder.
    pub dir: PathBuf,
    /// Its `SucroseInfo.json`.
    pub info: SucroseInfo,
    /// What it plays: the verified file inside `dir`, or the verified address.
    pub source: ThemeSource,
}

/// Why a `Source` cannot be played, judged on its text alone.
fn source_text_problem(source: &str) -> Option<&'static str> {
    if source.is_empty() {
        Some("the theme has no Source")
    } else if source.contains("://") {
        Some("the Source is an address, not a file")
    } else if source.starts_with(['/', '\\']) {
        // Also covers UNC (`\\server\share`) and device (`\\?\`, `\\.\`) paths.
        Some("the Source is an absolute, UNC or device path")
    } else if source.contains(':') {
        Some("the Source has a drive prefix")
    } else if source.split(['/', '\\']).any(|part| part == "..") {
        Some("the Source leaves the theme folder")
    } else {
        None
    }
}

/// Checks the `Source` against the file system and returns its absolute path: it must exist, be a regular file and
/// stay inside the theme folder once links are resolved (T-02-16).
fn verified_source(theme_dir: &Path, source: &str) -> Result<PathBuf, &'static str> {
    if let Some(problem) = source_text_problem(source) {
        return Err(problem);
    }
    let path = theme_dir.join(source);
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err("the Source file does not exist"),
        Err(_) => return Err("the Source file cannot be accessed"),
    };
    if !metadata.is_file() {
        return Err("the Source is not a regular file");
    }
    let (Ok(root), Ok(real)) = (fs::canonicalize(theme_dir), fs::canonicalize(&path)) else {
        return Err("the Source file cannot be resolved");
    };
    if !real.starts_with(&root) {
        return Err("the Source resolves outside the theme folder");
    }
    std::path::absolute(&path).map_err(|_| "the Source path cannot be made absolute")
}

/// Decides whether the theme selected in the library may start: a folder that exists, a readable `SucroseInfo.json`, a
/// type listed in `playable` and a `Source` that is a regular file inside the folder (a video file, or the page of a
/// Web theme) or, for a Url or YouTube theme, an http or https address. Every failure is its own [`ThemeRejection`]; nothing is started or written.
///
/// A theme whose `AppVersion` is newer than `app` is refused too (needs a newer app); the same version is accepted.
pub fn check_selected(
    location: &Path,
    selected: &str,
    app: &LegacyVersion,
    playable: &[WallpaperType],
) -> Result<SelectedTheme, ThemeRejection> {
    if selected.is_empty() {
        return Err(ThemeRejection::NothingSelected);
    }
    let folder = selected.to_owned();
    // The selection is a folder name; anything with a separator or a `..` would leave the library.
    let mut components = Path::new(selected).components();
    let single_name = matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    ) && !selected.contains(['/', '\\']);
    let dir = location.join(selected);
    if !single_name || !dir.is_dir() {
        return Err(ThemeRejection::FolderMissing { folder });
    }
    let info = read_info(&dir).map_err(|error| ThemeRejection::InfoUnreadable {
        folder: folder.clone(),
        reason: match error {
            ThemeError::Missing(_) => "the file is missing".to_owned(),
            ThemeError::Unreadable { .. } => "the file cannot be read".to_owned(),
            ThemeError::Invalid { reason, .. } => reason,
        },
    })?;
    if !playable.contains(&info.kind) {
        return Err(ThemeRejection::TypeNotPlayable {
            folder,
            found: info.kind,
        });
    }
    if info.app_version > *app {
        return Err(ThemeRejection::NeedsNewerApp {
            folder,
            required: info.app_version,
            current: *app,
        });
    }
    let source = match info.kind {
        WallpaperType::Url => verified_address(&info.source).map(ThemeSource::Address),
        WallpaperType::YouTube => verified_address(&info.source).and_then(|address| {
            checked_youtube_ids(&address)?;
            Ok(ThemeSource::Address(address))
        }),
        _ => verified_source(&dir, &info.source).map(ThemeSource::File),
    }
    .map_err(|reason| ThemeRejection::SourceInvalid {
        folder: folder.clone(),
        reason: reason.to_owned(),
    })?;
    Ok(SelectedTheme { dir, info, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIDEO: &str = r#"{"AppVersion":"26.10.19.0","Title":"Sample","Type":3,"Source":"video.mp4","License":null}"#;

    #[test]
    fn type_accepts_ordinals_and_names() {
        assert_eq!(WallpaperType::from_legacy_int(3), Some(WallpaperType::Video));
        assert_eq!(WallpaperType::from_legacy_int(6), None);
        assert_eq!(WallpaperType::from_legacy_name("YouTube"), Some(WallpaperType::YouTube));
        assert_eq!(WallpaperType::from_legacy_name("video"), None);
        assert_eq!(WallpaperType::from_legacy_name("2"), Some(WallpaperType::Web));
    }

    #[test]
    fn parses_a_video_theme() {
        let info = parse_info(VIDEO).unwrap();
        assert_eq!(info.kind, WallpaperType::Video);
        assert_eq!(info.source, "video.mp4");
        assert_eq!(info.title, "Sample");
        assert_eq!(info.app_version, LegacyVersion::new(26, 10, 19, 0));
    }

    #[test]
    fn a_type_name_in_any_case_parses_in_info_but_not_in_settings_names() {
        for name in ["video", "VIDEO", "Video", "vIdEo"] {
            let info = parse_info(&format!("{{\"Type\":\"{name}\",\"Source\":\"a.mp4\"}}")).unwrap();
            assert_eq!(info.kind, WallpaperType::Video, "{name}");
        }
        assert_eq!(
            WallpaperType::from_name_ignore_case("youtube"),
            Some(WallpaperType::YouTube)
        );
        assert_eq!(WallpaperType::from_name_ignore_case("3"), Some(WallpaperType::Video));
        assert_eq!(WallpaperType::from_name_ignore_case("film"), None);
        assert_eq!(WallpaperType::from_legacy_name("video"), None);
    }

    #[test]
    fn parses_a_named_type_and_a_bom() {
        let info = parse_info("\u{FEFF}{\"Type\":\"Video\",\"Source\":\"a.mp4\"}").unwrap();
        assert_eq!(info.kind, WallpaperType::Video);
    }

    #[test]
    fn rejects_unknown_type_and_garbage() {
        assert!(parse_info(r#"{"Type":9,"Source":"a"}"#).is_err());
        assert!(parse_info(r#"{"Type":3}"#).is_err());
        assert!(parse_info("{").is_err());
    }

    #[test]
    fn parse_errors_do_not_quote_the_file() {
        let reason = parse_info(r#"{"Type":3,"Source":"secret-value-123","Title":["zzz-quoted"]}"#).unwrap_err();
        assert!(
            !reason.contains("secret-value-123") && !reason.contains('7'),
            "{reason}"
        );
    }

    #[test]
    fn read_info_reports_a_missing_file() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no_such_theme");
        assert!(matches!(read_info(&dir), Err(ThemeError::Missing(_))));
    }

    #[test]
    fn read_info_reads_a_fixture() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video_theme");
        let info = read_info(&dir).unwrap();
        assert_eq!(info.kind, WallpaperType::Video);
        assert_eq!(info.source, "video.mp4");
    }

    #[test]
    fn read_info_reports_an_invalid_file() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/broken_theme");
        assert!(matches!(read_info(&dir), Err(ThemeError::Invalid { .. })));
    }
}
