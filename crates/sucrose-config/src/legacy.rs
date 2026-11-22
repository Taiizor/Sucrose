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
use sucrose_core::settings::{FullScreenAction, GpuPreference, PauseKind, ScreenArea, Stretch};

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
///
/// The typed getters never fail: a missing key is normal and gives the default quietly, a key with the wrong type, an
/// out-of-range number or an unknown enum value gives the default and logs a warning naming the file and the key.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Properties {
    /// The file these properties came from, for log lines.
    file: Option<LegacyFile>,
    values: Map<String, Value>,
}

impl Properties {
    fn from_values(file: LegacyFile, values: Map<String, Value>) -> Self {
        Self {
            file: Some(file),
            values,
        }
    }

    fn file_name(&self) -> &'static str {
        self.file.map_or("", LegacyFile::file_name)
    }

    /// The raw value of a key; a missing key is logged quietly.
    fn value(&self, key: &str) -> Option<&Value> {
        let value = self.values.get(key);
        if value.is_none() {
            tracing::debug!(
                file = self.file_name(),
                key,
                "legacy setting missing, using the default"
            );
        }
        value
    }

    fn warn_invalid(&self, key: &str) {
        tracing::warn!(
            file = self.file_name(),
            key,
            "legacy setting invalid, using the default"
        );
    }

    /// String value of a key, or `None` when the key is absent or not a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(Value::as_str)
    }

    /// Boolean value of a key, or `default` when it is missing or not a JSON boolean.
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.value(key) {
            None => default,
            Some(Value::Bool(value)) => *value,
            Some(_) => {
                self.warn_invalid(key);
                default
            }
        }
    }

    /// Integer value of a key: a JSON number or a numeric string, like the legacy reader. `None` when it is missing or
    /// not a number.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        let value = self.value(key)?;
        let number = integer(value);
        if number.is_none() {
            self.warn_invalid(key);
        }
        number
    }

    /// Integer value of a key limited to `min..=max`: a number outside the range is clamped, a missing or non-numeric
    /// value gives `default`.
    pub fn get_clamped(&self, key: &str, default: i64, min: i64, max: i64) -> i64 {
        match self.get_i64(key) {
            None => default,
            Some(value) if (min..=max).contains(&value) => value,
            Some(value) => {
                self.warn_invalid(key);
                value.clamp(min, max)
            }
        }
    }

    /// Enum value of a key: a name (exact, case-sensitive) or a numeric string or number for `from_index`. Anything
    /// else, or a missing key, gives `default`.
    pub fn get_enum<T>(
        &self,
        key: &str,
        default: T,
        from_name: impl Fn(&str) -> Option<T>,
        from_index: impl Fn(i64) -> Option<T>,
    ) -> T {
        let Some(value) = self.value(key) else {
            return default;
        };
        let parsed = match value {
            Value::String(text) => from_name(text).or_else(|| text.trim().parse().ok().and_then(&from_index)),
            Value::Number(_) => integer(value).and_then(&from_index),
            _ => None,
        };
        parsed.unwrap_or_else(|| {
            self.warn_invalid(key);
            default
        })
    }
}

/// An integer from a JSON number (a fraction is rounded) or a numeric string.
fn integer(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|float| float.is_finite())
                .map(|float| float.round() as i64)
        }),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
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
            Some(Value::Object(values)) => Ok(Properties { file: None, values }),
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
            Ok(properties) => return Properties::from_values(file, properties.values),
            Err(error) if error.is_transient() && attempt < ATTEMPTS => {
                attempt += 1;
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "legacy setting file read as defaults");
                return Properties::from_values(file, Map::new());
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

/// Settings of `General.json` that M1 uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralSettings {
    /// Whether the app window is shown (`AppVisible`); the tray gate of the service reads it.
    pub app_visible: bool,
}

impl GeneralSettings {
    fn from_properties(properties: &Properties) -> Self {
        Self {
            app_visible: properties.get_bool("AppVisible", true),
        }
    }
}

/// Settings of `Engine.json` that M1 uses.
///
/// Not read on purpose: `DisplayScreenType` and `ScreenDevice` (D-01), `CrashExplorer` (D-11), the per-type engine
/// choice (ENG-07) and `StayAwake`, none of which is in D-09.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineSettings {
    /// Wallpaper volume, 0..=100 (`WallpaperVolume`).
    pub volume: u8,
    /// Whether the wallpaper loops (`WallpaperLoop`).
    pub looping: bool,
    /// How the wallpaper is fitted (`StretchType`).
    pub stretch: Stretch,
    /// Whether the player may decode on the GPU (`HardwareAcceleration`).
    pub hardware_acceleration: bool,
    /// Which part of the monitor is covered (`ScreenType`).
    pub screen: ScreenArea,
    /// Which GPU Windows is asked to give the renderers (`GraphicPreference`, default high performance, DESK-12).
    pub gpu_preference: GpuPreference,
}

impl EngineSettings {
    fn from_properties(properties: &Properties) -> Self {
        let volume = properties.get_clamped("WallpaperVolume", 100, 0, 100);
        Self {
            volume: u8::try_from(volume).unwrap_or(100),
            looping: properties.get_bool("WallpaperLoop", true),
            stretch: properties.get_enum(
                "StretchType",
                Stretch::default(),
                Stretch::from_legacy_name,
                Stretch::from_legacy_index,
            ),
            hardware_acceleration: properties.get_bool("HardwareAcceleration", true),
            // The ordinals of this external enum are unclear, so a number falls back to the default.
            screen: properties.get_enum(
                "ScreenType",
                ScreenArea::default(),
                ScreenArea::from_legacy_name,
                |_| None,
            ),
            // Only the three enum names or ordinals 0..=2 count; anything else is the legacy default (T-02-39).
            gpu_preference: properties.get_enum(
                "GraphicPreference",
                GpuPreference::default(),
                GpuPreference::from_legacy_name,
                GpuPreference::from_legacy_index,
            ),
        }
    }
}

/// Settings of `Backgroundog.json` that M1 uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundogSettings {
    /// What happens while a fullscreen app is in front (`FullScreenPerformance`).
    pub fullscreen: FullScreenAction,
    /// How a pause is carried out (`PausePerformanceType`).
    pub pause: PauseKind,
}

impl BackgroundogSettings {
    fn from_properties(properties: &Properties) -> Self {
        Self {
            fullscreen: properties.get_enum(
                "FullScreenPerformance",
                FullScreenAction::default(),
                FullScreenAction::from_legacy_name,
                FullScreenAction::from_legacy_index,
            ),
            pause: properties.get_enum(
                "PausePerformanceType",
                PauseKind::default(),
                PauseKind::from_legacy_name,
                PauseKind::from_legacy_index,
            ),
        }
    }
}

/// Every legacy setting M1 uses, read once from the C# install's `Setting` folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacySettings {
    /// `Library.json`.
    pub library: LibrarySettings,
    /// `General.json`.
    pub general: GeneralSettings,
    /// `Engine.json`.
    pub engine: EngineSettings,
    /// `Backgroundog.json`.
    pub backgroundog: BackgroundogSettings,
}

impl LegacySettings {
    /// Reads the four files from `setting_dir` (read-only). Anything missing or invalid is the legacy default;
    /// `legacy_root` is the C# data folder, used for the default library location.
    pub fn load(setting_dir: &Path, legacy_root: &Path) -> Self {
        Self {
            library: LibrarySettings::from_properties(&read(setting_dir, LegacyFile::Library), legacy_root),
            general: GeneralSettings::from_properties(&read(setting_dir, LegacyFile::General)),
            engine: EngineSettings::from_properties(&read(setting_dir, LegacyFile::Engine)),
            backgroundog: BackgroundogSettings::from_properties(&read(setting_dir, LegacyFile::Backgroundog)),
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
        assert_eq!(library.location, PathBuf::from("C:\\Sucrose\\Library"));
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
        assert!(read(&fixtures().join("nothing"), LegacyFile::General).values.is_empty());
        assert!(matches!(
            parse(b"{ \"Properties\": { \"AppVisible\": tru"),
            Err(ReadError::Json(_))
        ));
    }

    #[test]
    fn sharing_violations_and_empty_files_are_retried() {
        assert!(ReadError::Io(io::Error::from_raw_os_error(32)).is_transient());
        assert!(ReadError::Empty.is_transient());
        assert!(!ReadError::Io(io::Error::from(io::ErrorKind::NotFound)).is_transient());
        assert!(!ReadError::NoProperties.is_transient());
    }

    #[test]
    fn getters_fall_back_for_wrong_types() {
        let properties = parse(br#"{"Properties":{"A":"yes","B":"7","C":7.6,"D":[1],"E":"Fill"}}"#).unwrap();
        assert!(properties.get_bool("A", true));
        assert!(!properties.get_bool("A", false));
        assert_eq!(properties.get_i64("B"), Some(7));
        assert_eq!(properties.get_i64("C"), Some(8));
        assert_eq!(properties.get_i64("D"), None);
        assert_eq!(properties.get_i64("missing"), None);
        assert_eq!(properties.get_clamped("B", 1, 0, 5), 5);
        assert_eq!(properties.get_clamped("E", 1, 0, 5), 1);
        assert_eq!(
            properties.get_enum("E", Stretch::None, Stretch::from_legacy_name, |_| None),
            Stretch::Fill
        );
        assert_eq!(
            properties.get_enum("D", Stretch::None, Stretch::from_legacy_name, |_| None),
            Stretch::None
        );
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
