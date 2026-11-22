//! Theme model shared with the C# app: the wallpaper type, \`SucroseInfo.json\` (read, written and checked) and the gate
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        Value::String(name) => WallpaperType::from_legacy_name(&name),
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
    /// The theme is not a video wallpaper.
    NotVideo {
        /// Folder name as selected.
        folder: String,
        /// The type it has.
        found: WallpaperType,
    },
    /// `Source` is not a regular file inside the theme folder.
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
            Self::NotVideo { folder, found } => {
                write!(
                    f,
                    "The selected theme {folder} is a {found:?} wallpaper; only Video plays in this build."
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

/// A selected theme that passed the gate.
#[derive(Debug, Clone)]
pub struct SelectedTheme {
    /// The theme folder.
    pub dir: PathBuf,
    /// Its `SucroseInfo.json`.
    pub info: SucroseInfo,
    /// Absolute path of the video file, verified to be a regular file inside `dir`.
    pub source_path: PathBuf,
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
/// Video type and a `Source` that is a regular file inside the folder. Every failure is its own
/// [`ThemeRejection`]; nothing is started or written.
///
/// A theme whose `AppVersion` is newer than `app` is refused too (needs a newer app); the same version is accepted.
pub fn check_selected(location: &Path, selected: &str, app: &LegacyVersion) -> Result<SelectedTheme, ThemeRejection> {
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
    if info.kind != WallpaperType::Video {
        return Err(ThemeRejection::NotVideo {
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
    let source_path = verified_source(&dir, &info.source).map_err(|reason| ThemeRejection::SourceInvalid {
        folder: folder.clone(),
        reason: reason.to_owned(),
    })?;
    Ok(SelectedTheme { dir, info, source_path })
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
