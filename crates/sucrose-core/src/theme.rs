//! Theme model shared with the C# app: the wallpaper type and `SucroseInfo.json`.
//!
//! Only reading exists so far; the typed `AppVersion` and the compatible writer follow with the theme round trip.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, de};
use serde_json::Value;

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
}

fn deserialize_type<'de, D: Deserializer<'de>>(deserializer: D) -> Result<WallpaperType, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let parsed = match &value {
        Value::Number(number) => number.as_i64().and_then(WallpaperType::from_legacy_int),
        Value::String(name) => WallpaperType::from_legacy_name(name),
        _ => None,
    };
    parsed.ok_or_else(|| de::Error::custom(format!("unknown wallpaper type {value}")))
}

/// The fields of `SucroseInfo.json` this build reads. Unknown fields are ignored for now.
#[derive(Debug, Clone, Deserialize)]
pub struct SucroseInfo {
    /// App version the theme was made with (a string or an object; typed later).
    #[serde(rename = "AppVersion", default)]
    pub app_version: Value,
    /// Display title.
    #[serde(rename = "Title", default)]
    pub title: String,
    /// Kind of wallpaper.
    #[serde(rename = "Type", deserialize_with = "deserialize_type")]
    pub kind: WallpaperType,
    /// File relative to the theme folder, or an address.
    #[serde(rename = "Source")]
    pub source: String,
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
        /// What is wrong with it.
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
    let text = String::from_utf8(bytes).map_err(|error| ThemeError::Invalid {
        path: path.clone(),
        reason: error.to_string(),
    })?;
    parse_info(&text).map_err(|reason| ThemeError::Invalid { path, reason })
}

/// Parses the text of a `SucroseInfo.json`, tolerating a leading byte order mark.
fn parse_info(text: &str) -> Result<SucroseInfo, String> {
    serde_json::from_str(text.trim_start_matches('\u{FEFF}')).map_err(|error| error.to_string())
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
