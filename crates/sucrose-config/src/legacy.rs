//! Read-only reader of the C# app's `Setting/<File>.json` files (FMT-08).
//!
//! Files are opened for reading only; nothing in this crate creates, writes, renames or deletes a legacy file (D-008).
//! Like the legacy reader, any failure yields defaults, but each failure is logged.

use std::{
    fs, io,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde_json::{Map, Value};

/// How often a read is attempted when the C# app is writing the file at that moment.
const ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(50);

/// The legacy setting files read by this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyFile {
    /// `Library.json`: library location and selected theme.
    Library,
    /// `General.json`.
    General,
    /// `Engine.json`.
    Engine,
    /// `Backgroundog.json`.
    Backgroundog,
}

impl LegacyFile {
    /// File name inside the `Setting` folder.
    pub fn file_name(self) -> &'static str {
        match self {
            Self::Library => "Library.json",
            Self::General => "General.json",
            Self::Engine => "Engine.json",
            Self::Backgroundog => "Backgroundog.json",
        }
    }
}

/// The `Properties` object of a legacy setting file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Properties(Map<String, Value>);

impl Properties {
    /// String value of a key, or `None` when the key is absent or not a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(Value::as_str)
    }
}

/// Why one attempt to read a file failed.
#[derive(Debug, thiserror::Error)]
enum ReadError {
    #[error("cannot read the file: {0}")]
    Io(#[from] io::Error),
    #[error("the file is empty")]
    Empty,
    #[error("not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("no Properties object")]
    NoProperties,
}

impl ReadError {
    /// Whether a retry may succeed: the writer truncates in place, so a read can see an empty file or a sharing violation.
    fn is_transient(&self) -> bool {
        match self {
            Self::Io(error) => error.kind() != io::ErrorKind::NotFound,
            Self::Empty => true,
            Self::Json(_) | Self::NoProperties => false,
        }
    }
}

fn parse(bytes: &[u8]) -> Result<Properties, ReadError> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(ReadError::Empty);
    }
    match serde_json::from_slice::<Value>(bytes)? {
        Value::Object(mut root) => match root.remove("Properties") {
            Some(Value::Object(properties)) => Ok(Properties(properties)),
            _ => Err(ReadError::NoProperties),
        },
        _ => Err(ReadError::NoProperties),
    }
}

/// Reads a legacy setting file from `dir`. A missing, locked, empty or unparsable file gives empty properties and a warning.
pub fn read(dir: &Path, file: LegacyFile) -> Properties {
    let path = dir.join(file.file_name());
    let mut attempt = 1;
    loop {
        match fs::read(&path).map_err(ReadError::from).and_then(|bytes| parse(&bytes)) {
            Ok(properties) => return properties,
            Err(error) if error.is_transient() && attempt < ATTEMPTS => {
                attempt += 1;
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "legacy setting file read as defaults");
                return Properties::default();
            }
        }
    }
}

/// The library settings: where themes live and which one is selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySettings {
    /// Folder that contains the theme folders.
    pub location: PathBuf,
    /// Folder name of the selected theme, empty when none is selected.
    pub selected: String,
}

impl LibrarySettings {
    /// Builds the settings from `Library.json`, with the legacy defaults (`<legacy_root>/Library`, nothing selected).
    pub fn from_properties(properties: &Properties, legacy_root: &Path) -> Self {
        let location = match properties.get_str("Location") {
            Some(location) if !location.is_empty() => PathBuf::from(location),
            _ => legacy_root.join("Library"),
        };
        Self {
            location,
            selected: properties.get_str("Selected").unwrap_or_default().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn reads_the_library_fixture() {
        let properties = read(&fixtures(), LegacyFile::Library);
        let library = LibrarySettings::from_properties(&properties, Path::new("root"));
        assert_eq!(library.location, PathBuf::from("H:\\Library"));
        assert_eq!(library.selected, "AAAAAAAAAAAAAAAAAAAAAAAAA");
    }

    #[test]
    fn missing_file_gives_the_legacy_defaults() {
        let properties = read(&fixtures().join("nothing"), LegacyFile::Library);
        let library = LibrarySettings::from_properties(&properties, Path::new("root"));
        assert_eq!(library.location, Path::new("root").join("Library"));
        assert_eq!(library.selected, "");
    }

    #[test]
    fn invalid_file_gives_defaults() {
        assert_eq!(read(&fixtures(), LegacyFile::General), Properties::default());
    }

    #[test]
    fn parse_strips_a_byte_order_mark() {
        let properties = parse(b"\xEF\xBB\xBF{\"Properties\":{\"Selected\":\"x\"}}").unwrap();
        assert_eq!(properties.get_str("Selected"), Some("x"));
    }

    #[test]
    fn parse_rejects_empty_and_wrapperless_files() {
        assert!(matches!(parse(b"  \r\n"), Err(ReadError::Empty)));
        assert!(matches!(parse(b"{}"), Err(ReadError::NoProperties)));
        assert!(matches!(parse(b"[1]"), Err(ReadError::NoProperties)));
        assert!(matches!(parse(b"{"), Err(ReadError::Json(_))));
    }

    #[test]
    fn non_string_values_are_not_strings() {
        let properties = parse(br#"{"Properties":{"Selected":5}}"#).unwrap();
        assert_eq!(properties.get_str("Selected"), None);
    }
}
