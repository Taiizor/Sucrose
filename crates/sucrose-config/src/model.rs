//! The versioned settings model of `Setting/Sucrose.json` (D-01, D-02).
//!
//! One section per legacy setting file, keys named like the legacy keys, values with the legacy defaults, so support
//! and the settings pages map them one to one. Reading is lenient: a missing or invalid value is the legacy default
//! (logged), and keys this build does not know are kept so a write never drops them (T-03-03).

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};
use sucrose_core::settings::{FullScreenAction, GpuPreference, PauseKind, ScreenArea, Stretch};

/// The format version this build writes. A file with a higher version is read but never overwritten.
pub const SETTINGS_VERSION: u32 = 1;

/// Every setting of this build, as stored in `Setting/Sucrose.json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Settings {
    /// Format version (`Version`).
    #[serde(rename = "Version")]
    pub version: u32,
    /// Where themes live and which one is selected (legacy `Library.json`).
    #[serde(rename = "Library")]
    pub library: Library,
    /// Legacy `General.json`.
    #[serde(rename = "General")]
    pub general: General,
    /// Playback and placement (legacy `Engine.json`).
    #[serde(rename = "Engine")]
    pub engine: Engine,
    /// Performance rules (legacy `Backgroundog.json`).
    #[serde(rename = "Backgroundog")]
    pub backgroundog: Backgroundog,
    /// Top-level sections this build does not know, kept as they are.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Library` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Library {
    /// Folder that contains the theme folders; empty means the legacy default folder (`Library.json` `Location`).
    #[serde(rename = "Location")]
    pub location: String,
    /// Folder name of the selected theme, empty when none is selected (`Selected`).
    #[serde(rename = "Selected")]
    pub selected: String,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `General` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct General {
    /// Whether the app window is shown; the tray of the service reads it (`AppVisible`).
    #[serde(rename = "AppVisible")]
    pub app_visible: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Engine` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Engine {
    /// Wallpaper volume, 0..=100 (`WallpaperVolume`).
    #[serde(rename = "WallpaperVolume")]
    pub volume: u8,
    /// Whether the wallpaper loops (`WallpaperLoop`).
    #[serde(rename = "WallpaperLoop")]
    pub looping: bool,
    /// How the wallpaper is fitted (`StretchType`).
    #[serde(rename = "StretchType")]
    pub stretch: Stretch,
    /// Whether the player may decode on the GPU (`HardwareAcceleration`).
    #[serde(rename = "HardwareAcceleration")]
    pub hardware_acceleration: bool,
    /// Which part of the monitor is covered (`ScreenType`).
    #[serde(rename = "ScreenType")]
    pub screen: ScreenArea,
    /// Which GPU Windows is asked to give the renderers (`GraphicPreference`).
    #[serde(rename = "GraphicPreference")]
    pub gpu_preference: GpuPreference,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Backgroundog` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Backgroundog {
    /// What happens while a fullscreen app is in front (`FullScreenPerformance`).
    #[serde(rename = "FullScreenPerformance")]
    pub fullscreen: FullScreenAction,
    /// How a pause is carried out (`PausePerformanceType`).
    #[serde(rename = "PausePerformanceType")]
    pub pause: PauseKind,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            location: String::new(),
            selected: String::new(),
            extra: Map::new(),
        }
    }
}

impl Default for General {
    fn default() -> Self {
        Self {
            app_visible: true,
            extra: Map::new(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            volume: 100,
            looping: true,
            stretch: Stretch::default(),
            hardware_acceleration: true,
            screen: ScreenArea::default(),
            gpu_preference: GpuPreference::default(),
            extra: Map::new(),
        }
    }
}

impl Default for Backgroundog {
    fn default() -> Self {
        Self {
            fullscreen: FullScreenAction::default(),
            pause: PauseKind::default(),
            extra: Map::new(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            library: Library::default(),
            general: General::default(),
            engine: Engine::default(),
            backgroundog: Backgroundog::default(),
            extra: Map::new(),
        }
    }
}

impl Settings {
    /// Parses a settings file. Only text that is not a JSON object is an error (a corrupt file); inside it every
    /// missing or invalid value is the legacy default.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
        match serde_json::from_slice::<Value>(bytes)? {
            Value::Object(root) => Ok(Self::from_map(root)),
            _ => Err(<serde_json::Error as serde::de::Error>::custom(
                "the settings root is not a JSON object",
            )),
        }
    }

    /// The file content: pretty-printed so a person can read it (D-02), ending with a newline.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// The folder that contains the theme folders: the configured one, or `<legacy_root>/Library` when it is empty.
    pub fn library_location(&self, legacy_root: &Path) -> PathBuf {
        if self.library.location.is_empty() {
            legacy_root.join("Library")
        } else {
            PathBuf::from(&self.library.location)
        }
    }

    fn from_map(mut root: Map<String, Value>) -> Self {
        let version = match root.remove("Version") {
            None => SETTINGS_VERSION,
            Some(value) => value
                .as_u64()
                .and_then(|number| u32::try_from(number).ok())
                .unwrap_or_else(|| {
                    tracing::warn!(key = "Version", "setting invalid, using the current version");
                    SETTINGS_VERSION
                }),
        };
        let mut library = Fields::section("Library", root.remove("Library"));
        let mut general = Fields::section("General", root.remove("General"));
        let mut engine = Fields::section("Engine", root.remove("Engine"));
        let mut backgroundog = Fields::section("Backgroundog", root.remove("Backgroundog"));
        Self {
            version,
            library: Library {
                location: library.string("Location", ""),
                selected: library.string("Selected", ""),
                extra: library.rest(),
            },
            general: General {
                app_visible: general.boolean("AppVisible", true),
                extra: general.rest(),
            },
            engine: Engine {
                volume: engine.volume("WallpaperVolume", 100),
                looping: engine.boolean("WallpaperLoop", true),
                stretch: engine.name("StretchType", Stretch::default(), Stretch::from_legacy_name),
                hardware_acceleration: engine.boolean("HardwareAcceleration", true),
                screen: engine.name("ScreenType", ScreenArea::default(), ScreenArea::from_legacy_name),
                gpu_preference: engine.name(
                    "GraphicPreference",
                    GpuPreference::default(),
                    GpuPreference::from_legacy_name,
                ),
                extra: engine.rest(),
            },
            backgroundog: Backgroundog {
                fullscreen: backgroundog.name(
                    "FullScreenPerformance",
                    FullScreenAction::default(),
                    FullScreenAction::from_legacy_name,
                ),
                pause: backgroundog.name(
                    "PausePerformanceType",
                    PauseKind::default(),
                    PauseKind::from_legacy_name,
                ),
                extra: backgroundog.rest(),
            },
            extra: root,
        }
    }
}

/// The keys of one section while it is being read; what is not taken stays as the unknown keys.
struct Fields {
    section: &'static str,
    map: Map<String, Value>,
}

impl Fields {
    /// A section that is missing, or is not an object, has no keys (all defaults).
    fn section(section: &'static str, value: Option<Value>) -> Self {
        let map = match value {
            Some(Value::Object(map)) => map,
            Some(_) => {
                tracing::warn!(section, "settings section invalid, using the defaults");
                Map::new()
            }
            None => Map::new(),
        };
        Self { section, map }
    }

    fn invalid(&self, key: &str) {
        tracing::warn!(section = self.section, key, "setting invalid, using the default");
    }

    fn string(&mut self, key: &str, default: &str) -> String {
        match self.map.remove(key) {
            None => default.to_owned(),
            Some(Value::String(text)) => text,
            Some(_) => {
                self.invalid(key);
                default.to_owned()
            }
        }
    }

    fn boolean(&mut self, key: &str, default: bool) -> bool {
        match self.map.remove(key) {
            None => default,
            Some(Value::Bool(value)) => value,
            Some(_) => {
                self.invalid(key);
                default
            }
        }
    }

    /// A percentage: a number outside 0..=100 is clamped like the legacy reader, anything else is the default.
    fn volume(&mut self, key: &str, default: u8) -> u8 {
        let number = match self.map.remove(key) {
            None => return default,
            Some(Value::Number(number)) => number.as_i64().or_else(|| {
                number
                    .as_f64()
                    .filter(|float| float.is_finite())
                    .map(|float| float.round() as i64)
            }),
            Some(_) => None,
        };
        match number {
            Some(number) => u8::try_from(number.clamp(0, 100)).unwrap_or(default),
            None => {
                self.invalid(key);
                default
            }
        }
    }

    /// An enum stored by its legacy name; an unknown name is the default.
    fn name<T>(&mut self, key: &str, default: T, from_name: fn(&str) -> Option<T>) -> T {
        match self.map.remove(key) {
            None => default,
            Some(Value::String(text)) => from_name(&text).unwrap_or_else(|| {
                self.invalid(key);
                default
            }),
            Some(_) => {
                self.invalid(key);
                default
            }
        }
    }

    fn rest(self) -> Map<String, Value> {
        self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_legacy_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.engine.volume, 100);
        assert!(settings.engine.looping);
        assert!(settings.engine.hardware_acceleration);
        assert!(settings.general.app_visible);
        assert_eq!(settings.engine.stretch, Stretch::UniformToFill);
        assert_eq!(settings.backgroundog.fullscreen, FullScreenAction::Resume);
        assert_eq!(settings.library.selected, "");
    }

    #[test]
    fn serializes_with_the_legacy_section_and_key_names() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["Version"], 1);
        assert_eq!(value["Library"]["Location"], "");
        assert_eq!(value["General"]["AppVisible"], true);
        assert_eq!(value["Engine"]["WallpaperVolume"], 100);
        assert_eq!(value["Engine"]["StretchType"], "UniformToFill");
        assert_eq!(value["Engine"]["ScreenType"], "DisplayBound");
        assert_eq!(value["Engine"]["GraphicPreference"], "HighPerformance");
        assert_eq!(value["Backgroundog"]["FullScreenPerformance"], "Resume");
        assert_eq!(value["Backgroundog"]["PausePerformanceType"], "Light");
    }

    #[test]
    fn round_trips_and_keeps_unknown_keys() {
        let text = br#"{"Version":1,"Future":{"A":1},"Engine":{"WallpaperVolume":7,"Later":"x"},"Library":{"Selected":"abc"}}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.engine.volume, 7);
        assert_eq!(settings.library.selected, "abc");
        let again = Settings::from_slice(&settings.to_bytes().unwrap()).unwrap();
        assert_eq!(again, settings);
        assert_eq!(again.extra["Future"]["A"], 1);
        assert_eq!(again.engine.extra["Later"], "x");
    }

    #[test]
    fn invalid_values_are_the_legacy_defaults() {
        let text = br#"{"Engine":{"WallpaperVolume":"loud","WallpaperLoop":3,"StretchType":"Sideways","ScreenType":7},"General":5}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.engine, Engine::default());
        assert_eq!(settings.general, General::default());
    }

    #[test]
    fn volume_is_clamped() {
        let high = Settings::from_slice(br#"{"Engine":{"WallpaperVolume":250}}"#).unwrap();
        let low = Settings::from_slice(br#"{"Engine":{"WallpaperVolume":-4}}"#).unwrap();
        assert_eq!(high.engine.volume, 100);
        assert_eq!(low.engine.volume, 0);
    }

    #[test]
    fn text_that_is_not_an_object_is_corrupt() {
        assert!(Settings::from_slice(b"").is_err());
        assert!(Settings::from_slice(b"{\"Version\":1,").is_err());
        assert!(Settings::from_slice(b"[1]").is_err());
    }

    #[test]
    fn an_empty_location_is_the_legacy_library_folder() {
        let mut settings = Settings::default();
        assert_eq!(
            settings.library_location(Path::new("root")),
            Path::new("root").join("Library")
        );
        settings.library.location = "elsewhere".to_owned();
        assert_eq!(settings.library_location(Path::new("root")), PathBuf::from("elsewhere"));
    }

    #[test]
    fn a_byte_order_mark_is_ignored() {
        let settings = Settings::from_slice(b"\xEF\xBB\xBF{\"Library\":{\"Selected\":\"x\"}}").unwrap();
        assert_eq!(settings.library.selected, "x");
    }
}
