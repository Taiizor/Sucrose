//! `SucroseCompatible.json`: the JavaScript templates a web wallpaper registers to receive live data (loop mode,
//! stretch mode, audio, memory ...), read and written compatibly with the C# app (FMT-03).
//!
//! Each of the 16 keys holds a JavaScript format string with `{0}`. A missing key is empty. The order of keys in the
//! file is kept, known and unknown keys alike; unknown keys survive a write. Reading never modifies the file.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use crate::properties::{Ordered, PropertiesError, describe_json_error, read_text};

/// File name of a theme's compatible templates inside its folder.
pub const COMPATIBLE_FILE: &str = "SucroseCompatible.json";

/// Why a compatible file could not be read; the same cases as for the property file.
pub type CompatibleError = PropertiesError;

/// One of the 16 legacy keys, declared in the legacy table order (the order the engine runs the templates in).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibleKey {
    /// Loop mode of the wallpaper.
    LoopMode,
    /// Light or dark theme name.
    ThemeType,
    /// BIOS data.
    SystemBios,
    /// Date and time data.
    SystemDate,
    /// Shuffle mode.
    ShuffleMode,
    /// Stretch mode.
    StretchMode,
    /// Audio data.
    SystemAudio,
    /// Theme type as system theme.
    SystemTheme,
    /// Volume level.
    VolumeLevel,
    /// Memory data.
    SystemMemory,
    /// Battery data.
    SystemBattery,
    /// Graphics data.
    SystemGraphic,
    /// Network data.
    SystemNetwork,
    /// Storage data.
    SystemStorage,
    /// Processor data.
    SystemProcessor,
    /// Motherboard data.
    SystemMotherboard,
}

/// A group of system data the data hub samples (D-24), named like the legacy payload objects. It travels between a
/// renderer and the service (`DataWanted`, `Data`), so it keeps its variant order on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataGroup {
    /// BIOS.
    Bios,
    /// Date and time.
    Date,
    /// Audio (media session and spectrum).
    Audio,
    /// Memory.
    Memory,
    /// Battery.
    Battery,
    /// Graphics.
    Graphic,
    /// Network.
    Network,
    /// Storage.
    Storage,
    /// Processor.
    Processor,
    /// Motherboard.
    Motherboard,
}

impl DataGroup {
    /// All groups in the legacy payload order.
    pub const ALL: [Self; 10] = [
        Self::Bios,
        Self::Date,
        Self::Audio,
        Self::Memory,
        Self::Battery,
        Self::Graphic,
        Self::Network,
        Self::Storage,
        Self::Processor,
        Self::Motherboard,
    ];
}

impl CompatibleKey {
    /// All keys in the legacy table order.
    pub const ALL: [Self; 16] = [
        Self::LoopMode,
        Self::ThemeType,
        Self::SystemBios,
        Self::SystemDate,
        Self::ShuffleMode,
        Self::StretchMode,
        Self::SystemAudio,
        Self::SystemTheme,
        Self::VolumeLevel,
        Self::SystemMemory,
        Self::SystemBattery,
        Self::SystemGraphic,
        Self::SystemNetwork,
        Self::SystemStorage,
        Self::SystemProcessor,
        Self::SystemMotherboard,
    ];

    /// The key as the legacy JSON spells it.
    pub fn name(self) -> &'static str {
        match self {
            Self::LoopMode => "LoopMode",
            Self::ThemeType => "ThemeType",
            Self::SystemBios => "SystemBios",
            Self::SystemDate => "SystemDate",
            Self::ShuffleMode => "ShuffleMode",
            Self::StretchMode => "StretchMode",
            Self::SystemAudio => "SystemAudio",
            Self::SystemTheme => "SystemTheme",
            Self::VolumeLevel => "VolumeLevel",
            Self::SystemMemory => "SystemMemory",
            Self::SystemBattery => "SystemBattery",
            Self::SystemGraphic => "SystemGraphic",
            Self::SystemNetwork => "SystemNetwork",
            Self::SystemStorage => "SystemStorage",
            Self::SystemProcessor => "SystemProcessor",
            Self::SystemMotherboard => "SystemMotherboard",
        }
    }

    /// The key for a name in any ASCII letter case (Newtonsoft matches property names that way).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.name().eq_ignore_ascii_case(name))
    }

    /// The data group this template is fed from; `None` for the keys the engine fills from its own state.
    pub fn data_group(self) -> Option<DataGroup> {
        match self {
            Self::SystemBios => Some(DataGroup::Bios),
            Self::SystemDate => Some(DataGroup::Date),
            Self::SystemAudio => Some(DataGroup::Audio),
            Self::SystemMemory => Some(DataGroup::Memory),
            Self::SystemBattery => Some(DataGroup::Battery),
            Self::SystemGraphic => Some(DataGroup::Graphic),
            Self::SystemNetwork => Some(DataGroup::Network),
            Self::SystemStorage => Some(DataGroup::Storage),
            Self::SystemProcessor => Some(DataGroup::Processor),
            Self::SystemMotherboard => Some(DataGroup::Motherboard),
            Self::LoopMode
            | Self::ThemeType
            | Self::ShuffleMode
            | Self::StretchMode
            | Self::SystemTheme
            | Self::VolumeLevel => None,
        }
    }
}

/// The contents of `SucroseCompatible.json`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SucroseCompatible {
    /// Keys in file order. Known keys hold text (or null, which reads as empty); unknown keys hold any value.
    entries: Vec<(String, Value)>,
}

impl SucroseCompatible {
    /// A file with no templates.
    pub fn new() -> Self {
        Self::default()
    }

    /// The template of `key`; empty when the file has none.
    pub fn get(&self, key: CompatibleKey) -> &str {
        self.entries
            .iter()
            .find(|(name, _)| name == key.name())
            .and_then(|(_, value)| value.as_str())
            .unwrap_or("")
    }

    /// Sets the template of `key`, in place when the file already has it and at the end otherwise.
    pub fn set(&mut self, key: CompatibleKey, template: impl Into<String>) {
        let value = Value::from(template.into());
        match self.entries.iter_mut().find(|(name, _)| name == key.name()) {
            Some(slot) => slot.1 = value,
            None => self.entries.push((key.name().to_owned(), value)),
        }
    }

    /// The non-empty templates in the legacy table order, whatever the order in the file. This is the order the engine
    /// runs them in.
    pub fn templates(&self) -> Vec<(CompatibleKey, &str)> {
        CompatibleKey::ALL
            .into_iter()
            .map(|key| (key, self.get(key)))
            .filter(|(_, template)| !template.is_empty())
            .collect()
    }

    /// The data groups the file needs, in the legacy payload order: those whose template is non-empty.
    pub fn data_groups(&self) -> Vec<DataGroup> {
        self.templates()
            .into_iter()
            .filter_map(|(key, _)| key.data_group())
            .collect()
    }

    /// Keys this build does not know, in file order.
    pub fn unknown_entries(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries
            .iter()
            .filter(|(name, _)| CompatibleKey::from_name(name).is_none())
            .map(|(name, value)| (name.as_str(), value))
    }
}

impl Serialize for SucroseCompatible {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (name, value) in &self.entries {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
}

/// Parses the text of a `SucroseCompatible.json`, tolerating a leading byte order mark. The error names the kind of
/// problem and where, never the offending text. A known key matches in any letter case and is stored under its legacy
/// spelling; its value must be text or null.
pub fn parse_compatible(text: &str) -> Result<SucroseCompatible, String> {
    let raw: Ordered<Value> =
        serde_json::from_str(text.trim_start_matches('\u{FEFF}')).map_err(|error| describe_json_error(&error))?;
    let mut entries: Vec<(String, Value)> = Vec::with_capacity(raw.0.len());
    for (name, value) in raw.0 {
        let name = match CompatibleKey::from_name(&name) {
            Some(key) => {
                if !matches!(value, Value::String(_) | Value::Null) {
                    return Err(format!("`{}` is not text", key.name()));
                }
                key.name().to_owned()
            }
            None => name,
        };
        // Two spellings of one key: the later one wins, at the earlier place.
        match entries.iter_mut().find(|(existing, _)| *existing == name) {
            Some(slot) => slot.1 = value,
            None => entries.push((name, value)),
        }
    }
    Ok(SucroseCompatible { entries })
}

/// Reads a `SucroseCompatible.json`. The file is only opened for reading and is never changed or removed, whatever
/// it contains.
pub fn read_compatible(path: &Path) -> Result<SucroseCompatible, CompatibleError> {
    let text = read_text(path)?;
    parse_compatible(&text).map_err(|reason| PropertiesError::Invalid {
        path: path.to_owned(),
        reason,
    })
}

/// Writes `compatible` the way the C# app does: 2-space indent, CRLF line endings, no byte order mark, no trailing
/// newline. Only the given path is written.
pub fn write_compatible(path: &Path, compatible: &SucroseCompatible) -> io::Result<()> {
    let text = serde_json::to_string_pretty(compatible).map_err(io::Error::other)?;
    // Line breaks inside strings are escaped, so every raw `\n` is structural.
    fs::write(path, text.replace('\n', "\r\n"))
}
