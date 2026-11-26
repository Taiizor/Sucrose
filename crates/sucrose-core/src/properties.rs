//! `SucroseProperties.json`: the property list a web wallpaper (and mpv/VLC wallpapers) offers to the user, read and
//! written compatibly with the C# app (FMT-03).
//!
//! Reading is lenient (numbers may be integers or doubles, missing fields take their zero value, keys match in any
//! letter case as in Newtonsoft) but lossless: the order of the property list and of the localization tables is kept,
//! unknown top-level keys, unknown control fields and unknown control types survive a write. Reading never modifies,
//! moves or deletes the file; the legacy app deleted unparsable copies, Sucrose does not (D-017).

use std::{
    collections::HashMap,
    fmt, fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use serde::{
    Serialize, Serializer,
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::SerializeMap,
};
use serde_json::{Map, Value};

/// File name of a theme's property list inside its folder.
pub const PROPERTIES_FILE: &str = "SucroseProperties.json";

/// Why a theme's property or compatible file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum PropertiesError {
    /// The file does not exist.
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
    /// The file is not valid.
    #[error("{} is not a valid theme file: {reason}", .path.display())]
    Invalid {
        /// File that failed.
        path: PathBuf,
        /// What is wrong with it, without quoting the file.
        reason: String,
    },
}

/// The control of one property: what the user sees (name, text, help) and the type-specific fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    /// Key of the control in the property list; it is the name the page receives. Never written as a field.
    pub name: String,
    /// Label text or localization key; `None` when the file has none (labels often omit it).
    pub text: Option<String>,
    /// Help text or localization key.
    pub help: Option<String>,
    /// The type and its fields.
    pub kind: ControlKind,
    /// Fields of a known control type this build does not know; written back after the known ones.
    pub extra: Map<String, Value>,
}

/// The ten legacy control types, plus any other type kept as it was read.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlKind {
    /// Static text.
    Label {
        /// Text or localization key.
        value: String,
    },
    /// Button; the page gets the key when it is pressed.
    Button {
        /// Caption or localization key.
        value: String,
    },
    /// Number between two bounds.
    Slider {
        /// Lowest value.
        min: f64,
        /// Highest value.
        max: f64,
        /// Step between values.
        step: f64,
        /// Current value.
        value: f64,
    },
    /// Single-line text.
    Textbox {
        /// Current text.
        value: String,
    },
    /// On/off switch.
    Checkbox {
        /// Current state.
        value: bool,
    },
    /// Choice of one item.
    Dropdown {
        /// Index of the chosen item.
        value: i64,
        /// The items.
        items: Vec<String>,
        /// Text of the chosen item as of the read; writing and page calls recompute it from `items` and `value`.
        valuetext: String,
    },
    /// Number with a fixed count of decimal places.
    Numberbox {
        /// Lowest value.
        min: f64,
        /// Highest value.
        max: f64,
        /// Decimal places.
        places: i64,
        /// Current value.
        value: f64,
    },
    /// Single-line text shown masked.
    Passwordbox {
        /// Current text.
        value: String,
    },
    /// Choice of one file of a theme folder.
    Filedropdown {
        /// Description of the file kind.
        desc: String,
        /// Chosen file name.
        value: String,
        /// Title of the chooser.
        title: String,
        /// Folder inside the theme to list.
        folder: String,
        /// `|`-separated patterns such as `*.png|*.jpg`.
        filter: String,
        /// Listed files; filled at run time, usually absent in the file.
        files: Option<Vec<String>>,
    },
    /// Colour as `#AARRGGBB` text.
    Colorpicker {
        /// Current colour.
        value: String,
    },
    /// A type this build does not know. `raw` is the whole control object as read and is what gets written.
    Unknown {
        /// The type as spelled in the file.
        type_name: String,
        /// The control object with all its fields.
        raw: Map<String, Value>,
    },
}

impl ControlKind {
    /// The lower-case type name the legacy files use.
    pub fn type_name(&self) -> &str {
        match self {
            Self::Label { .. } => "label",
            Self::Button { .. } => "button",
            Self::Slider { .. } => "slider",
            Self::Textbox { .. } => "textbox",
            Self::Checkbox { .. } => "checkbox",
            Self::Dropdown { .. } => "dropdown",
            Self::Numberbox { .. } => "numberbox",
            Self::Passwordbox { .. } => "passwordbox",
            Self::Filedropdown { .. } => "filedropdown",
            Self::Colorpicker { .. } => "colorpicker",
            Self::Unknown { type_name, .. } => type_name,
        }
    }

    /// A dropdown with its `valuetext` computed from the items, as the legacy model does.
    pub fn dropdown(value: i64, items: Vec<String>) -> Self {
        let valuetext = dropdown_text(value, &items);
        Self::Dropdown {
            value,
            items,
            valuetext,
        }
    }

    /// The type-specific fields in the order the legacy models declare them. Empty for `Unknown`.
    fn fields(&self) -> Vec<(&'static str, Value)> {
        match self {
            Self::Label { value }
            | Self::Button { value }
            | Self::Textbox { value }
            | Self::Passwordbox { value }
            | Self::Colorpicker { value } => vec![("value", Value::from(value.as_str()))],
            Self::Slider { min, max, step, value } => vec![
                ("min", Value::from(*min)),
                ("max", Value::from(*max)),
                ("step", Value::from(*step)),
                ("value", Value::from(*value)),
            ],
            Self::Checkbox { value } => vec![("value", Value::from(*value))],
            Self::Dropdown { value, items, .. } => vec![
                ("value", Value::from(*value)),
                ("items", Value::from(items.clone())),
                ("valuetext", Value::from(dropdown_text(*value, items))),
            ],
            Self::Numberbox {
                min,
                max,
                places,
                value,
            } => vec![
                ("min", Value::from(*min)),
                ("max", Value::from(*max)),
                ("places", Value::from(*places)),
                ("value", Value::from(*value)),
            ],
            Self::Filedropdown {
                desc,
                value,
                title,
                folder,
                filter,
                files,
            } => {
                let mut fields = vec![
                    ("desc", Value::from(desc.as_str())),
                    ("value", Value::from(value.as_str())),
                    ("title", Value::from(title.as_str())),
                    ("folder", Value::from(folder.as_str())),
                    ("filter", Value::from(filter.as_str())),
                ];
                if let Some(files) = files {
                    fields.push(("files", Value::from(files.clone())));
                }
                fields
            }
            Self::Unknown { .. } => Vec::new(),
        }
    }
}

/// Text of the item a dropdown value points at; empty outside the list (`ElementAtOrDefault` in the legacy model).
fn dropdown_text(value: i64, items: &[String]) -> String {
    usize::try_from(value)
        .ok()
        .and_then(|index| items.get(index))
        .cloned()
        .unwrap_or_default()
}

impl Control {
    /// The control as the legacy app serializes it for a page call (`JsonConvert.SerializeObject(control)`): `type`,
    /// `text` (empty when absent), `help` (null when absent) and the type's fields; the name and unknown fields of
    /// known types are not part of it. An unknown type is returned as it was read.
    pub fn to_json_value(&self) -> Value {
        if let ControlKind::Unknown { raw, .. } = &self.kind {
            return Value::Object(raw.clone());
        }
        let mut object = Map::new();
        object.insert("type".to_owned(), Value::from(self.kind.type_name()));
        object.insert("text".to_owned(), Value::from(self.text.clone().unwrap_or_default()));
        object.insert("help".to_owned(), self.help.clone().map_or(Value::Null, Value::from));
        for (key, value) in self.kind.fields() {
            object.insert(key.to_owned(), value);
        }
        Value::Object(object)
    }
}

impl Serialize for Control {
    /// The file form: `type`, `text` and `help` when present, the type's fields, then unknown fields.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let ControlKind::Unknown { raw, .. } = &self.kind {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", self.kind.type_name())?;
        if let Some(text) = &self.text {
            map.serialize_entry("text", text)?;
        }
        if let Some(help) = &self.help {
            map.serialize_entry("help", help)?;
        }
        for (key, value) in self.kind.fields() {
            map.serialize_entry(key, &value)?;
        }
        for (key, value) in &self.extra {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

/// The contents of `SucroseProperties.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct SucroseProperties {
    /// JavaScript format string with `{0}` for the property name and `{1}` for the control.
    pub listener: String,
    /// The controls in file order, keyed by property name.
    pub list: Vec<(String, Control)>,
    /// Texts per culture in file order; empty when the file has no `PropertyLocalization`.
    pub localization: Vec<(String, Vec<(String, String)>)>,
    /// Top-level keys this build does not know; written back after the known ones.
    pub extra: Map<String, Value>,
}

/// A slice of pairs written as a JSON object in that order.
struct Entries<'a, T>(&'a [(String, T)]);

impl<T: Serialize> Serialize for Entries<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl Serialize for SucroseProperties {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("PropertyListener", &self.listener)?;
        map.serialize_entry("PropertyList", &Entries(&self.list))?;
        if !self.localization.is_empty() {
            let tables: Vec<(String, Entries<'_, String>)> = self
                .localization
                .iter()
                .map(|(culture, texts)| (culture.clone(), Entries(texts)))
                .collect();
            map.serialize_entry("PropertyLocalization", &Entries(&tables))?;
        }
        for (key, value) in &self.extra {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

/// A JSON object read with its keys in file order. A repeated key keeps its first position and its last value, like
/// Newtonsoft's `JObject`.
pub(crate) struct Ordered<T>(pub(crate) Vec<(String, T)>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ordered<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for OrderedVisitor<T> {
            type Value = Ordered<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries: Vec<(String, T)> = Vec::new();
                let mut positions: HashMap<String, usize> = HashMap::new();
                while let Some((key, value)) = map.next_entry::<String, T>()? {
                    match positions.get(&key) {
                        Some(&position) => entries[position].1 = value,
                        None => {
                            positions.insert(key.clone(), entries.len());
                            entries.push((key, value));
                        }
                    }
                }
                Ok(Ordered(entries))
            }
        }

        deserializer.deserialize_map(OrderedVisitor(PhantomData))
    }
}

/// Reads the text of a theme file. The file is only opened for reading.
pub(crate) fn read_text(path: &Path) -> Result<String, PropertiesError> {
    let bytes = fs::read(path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            PropertiesError::Missing(path.to_owned())
        } else {
            PropertiesError::Unreadable {
                path: path.to_owned(),
                source,
            }
        }
    })?;
    String::from_utf8(bytes).map_err(|_| PropertiesError::Invalid {
        path: path.to_owned(),
        reason: "the file is not UTF-8 text".to_owned(),
    })
}

/// Names the kind of JSON problem and where, never the offending text (it ends up in logs).
pub(crate) fn describe_json_error(error: &serde_json::Error) -> String {
    let (line, column) = (error.line(), error.column());
    match error.classify() {
        serde_json::error::Category::Syntax => format!("invalid JSON at line {line} column {column}"),
        serde_json::error::Category::Eof => "the JSON ends too early".to_owned(),
        serde_json::error::Category::Io => "the JSON could not be read".to_owned(),
        serde_json::error::Category::Data => {
            format!("a section has the wrong type (line {line} column {column})")
        }
    }
}

/// Removes and returns the field `name`, matching the key exactly first and in any ASCII letter case otherwise.
pub(crate) fn take_field(object: &mut Map<String, Value>, name: &str) -> Option<Value> {
    if let Some(value) = object.remove(name) {
        return Some(value);
    }
    let key = object.keys().find(|key| key.eq_ignore_ascii_case(name))?.clone();
    object.remove(&key)
}

/// Shortens a key for an error text.
fn short(name: &str) -> String {
    match name.char_indices().nth(40) {
        Some((end, _)) => format!("{}...", &name[..end]),
        None => name.to_owned(),
    }
}

/// A string field; absent or null is empty, numbers and booleans become text as Newtonsoft converts them.
fn string_field(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<String, String> {
    match take_field(object, field) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(text)) => Ok(text),
        Some(Value::Number(number)) => Ok(number.to_string()),
        Some(Value::Bool(flag)) => Ok(if flag { "True" } else { "False" }.to_owned()),
        Some(_) => Err(format!("control `{}`: `{field}` is not text", short(name))),
    }
}

/// A number field; absent or null is zero, a number in a string is accepted.
fn number_field(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<f64, String> {
    let wrong = || format!("control `{}`: `{field}` is not a number", short(name));
    match take_field(object, field) {
        None | Some(Value::Null) => Ok(0.0),
        Some(Value::Number(number)) => number.as_f64().ok_or_else(wrong),
        Some(Value::String(text)) => text.trim().parse::<f64>().map_err(|_| wrong()),
        Some(_) => Err(wrong()),
    }
}

/// A whole-number field; a double with no fraction is accepted.
fn integer_field(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<i64, String> {
    let wrong = || format!("control `{}`: `{field}` is not a whole number", short(name));
    let number = number_field(object, field, name)?;
    // The range check makes the cast exact.
    if number.fract() == 0.0 && (-9.0e15..=9.0e15).contains(&number) {
        Ok(number as i64)
    } else {
        Err(wrong())
    }
}

/// A boolean field; absent or null is false, `"true"`/`"false"` text and numbers are accepted.
fn bool_field(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<bool, String> {
    let wrong = || format!("control `{}`: `{field}` is not true or false", short(name));
    match take_field(object, field) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(flag)) => Ok(flag),
        Some(Value::String(text)) if text.eq_ignore_ascii_case("true") => Ok(true),
        Some(Value::String(text)) if text.eq_ignore_ascii_case("false") => Ok(false),
        Some(Value::Number(number)) => Ok(number.as_f64().is_some_and(|value| value != 0.0)),
        Some(_) => Err(wrong()),
    }
}

/// A list of strings; absent or null is `None`.
fn list_field(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<Option<Vec<String>>, String> {
    let wrong = || format!("control `{}`: `{field}` is not a list of text", short(name));
    match take_field(object, field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .into_iter()
            .map(|item| match item {
                Value::String(text) => Ok(text),
                Value::Number(number) => Ok(number.to_string()),
                _ => Err(wrong()),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(wrong()),
    }
}

/// An optional text field (`text`, `help`).
fn optional_text(object: &mut Map<String, Value>, field: &str, name: &str) -> Result<Option<String>, String> {
    match take_field(object, field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => Err(format!("control `{}`: `{field}` is not text", short(name))),
    }
}

impl Control {
    /// Builds a control from its JSON object. `name` is the key in the property list.
    fn from_json(name: &str, value: Value) -> Result<Self, String> {
        let Value::Object(mut object) = value else {
            return Err(format!("control `{}` is not an object", short(name)));
        };
        let type_name = match take_field(&mut object, "type") {
            Some(Value::String(text)) => text,
            Some(_) => return Err(format!("control `{}`: `type` is not text", short(name))),
            None => return Err(format!("control `{}` has no type", short(name))),
        };
        // Put the type back so an unknown control keeps it; known types write their own.
        let original = {
            let mut copy = object.clone();
            copy.insert("type".to_owned(), Value::from(type_name.as_str()));
            copy
        };
        let text = optional_text(&mut object, "text", name)?;
        let help = optional_text(&mut object, "help", name)?;
        let kind = match type_name.to_ascii_lowercase().as_str() {
            "label" => ControlKind::Label {
                value: string_field(&mut object, "value", name)?,
            },
            "button" => ControlKind::Button {
                value: string_field(&mut object, "value", name)?,
            },
            "slider" => ControlKind::Slider {
                min: number_field(&mut object, "min", name)?,
                max: number_field(&mut object, "max", name)?,
                step: number_field(&mut object, "step", name)?,
                value: number_field(&mut object, "value", name)?,
            },
            "textbox" => ControlKind::Textbox {
                value: string_field(&mut object, "value", name)?,
            },
            "checkbox" => ControlKind::Checkbox {
                value: bool_field(&mut object, "value", name)?,
            },
            "dropdown" => {
                let value = integer_field(&mut object, "value", name)?;
                let items = list_field(&mut object, "items", name)?.unwrap_or_default();
                // The legacy model computes it and ignores what the file says.
                object.remove("valuetext");
                ControlKind::dropdown(value, items)
            }
            "numberbox" => ControlKind::Numberbox {
                min: number_field(&mut object, "min", name)?,
                max: number_field(&mut object, "max", name)?,
                places: integer_field(&mut object, "places", name)?,
                value: number_field(&mut object, "value", name)?,
            },
            "passwordbox" => ControlKind::Passwordbox {
                value: string_field(&mut object, "value", name)?,
            },
            "filedropdown" => ControlKind::Filedropdown {
                desc: string_field(&mut object, "desc", name)?,
                value: string_field(&mut object, "value", name)?,
                title: string_field(&mut object, "title", name)?,
                folder: string_field(&mut object, "folder", name)?,
                filter: string_field(&mut object, "filter", name)?,
                files: list_field(&mut object, "files", name)?,
            },
            "colorpicker" => ControlKind::Colorpicker {
                value: string_field(&mut object, "value", name)?,
            },
            _ => {
                return Ok(Self {
                    name: name.to_owned(),
                    text,
                    help,
                    kind: ControlKind::Unknown {
                        type_name,
                        raw: original,
                    },
                    extra: Map::new(),
                });
            }
        };
        Ok(Self {
            name: name.to_owned(),
            text,
            help,
            kind,
            extra: object,
        })
    }
}

/// The top level as read, before the controls are typed.
struct RawProperties {
    listener: Option<Value>,
    list: Option<Ordered<Value>>,
    localization: Option<Ordered<Ordered<Value>>>,
    extra: Map<String, Value>,
}

impl<'de> Deserialize<'de> for RawProperties {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RawVisitor;

        impl<'de> Visitor<'de> for RawVisitor {
            type Value = RawProperties;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut raw = RawProperties {
                    listener: None,
                    list: None,
                    localization: None,
                    extra: Map::new(),
                };
                while let Some(key) = map.next_key::<String>()? {
                    if key.eq_ignore_ascii_case("PropertyListener") {
                        raw.listener = map.next_value::<Option<Value>>()?;
                    } else if key.eq_ignore_ascii_case("PropertyList") {
                        raw.list = map.next_value::<Option<Ordered<Value>>>()?;
                    } else if key.eq_ignore_ascii_case("PropertyLocalization") {
                        raw.localization = map.next_value::<Option<Ordered<Ordered<Value>>>>()?;
                    } else {
                        raw.extra.insert(key, map.next_value::<Value>()?);
                    }
                }
                Ok(raw)
            }
        }

        deserializer.deserialize_map(RawVisitor)
    }
}

/// Parses the text of a `SucroseProperties.json`, tolerating a leading byte order mark. The error names the kind of
/// problem and where, never the offending text.
pub fn parse_properties(text: &str) -> Result<SucroseProperties, String> {
    let raw: RawProperties =
        serde_json::from_str(text.trim_start_matches('\u{FEFF}')).map_err(|error| describe_json_error(&error))?;
    let listener = match raw.listener {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text,
        Some(_) => return Err("`PropertyListener` is not text".to_owned()),
    };
    let list = raw
        .list
        .map(|list| list.0)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, value)| Control::from_json(&name, value).map(|control| (name, control)))
        .collect::<Result<Vec<_>, _>>()?;
    let localization = raw
        .localization
        .map(|tables| tables.0)
        .unwrap_or_default()
        .into_iter()
        .map(|(culture, texts)| {
            texts
                .0
                .into_iter()
                .map(|(key, value)| match value {
                    Value::String(text) => Ok((key, text)),
                    _ => Err(format!("a localization text of `{}` is not text", short(&culture))),
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|texts| (culture, texts))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SucroseProperties {
        listener,
        list,
        localization,
        extra: raw.extra,
    })
}

/// Reads a `SucroseProperties.json`. The file is only opened for reading and is never changed or removed, whatever
/// it contains.
pub fn read_properties(path: &Path) -> Result<SucroseProperties, PropertiesError> {
    let text = read_text(path)?;
    parse_properties(&text).map_err(|reason| PropertiesError::Invalid {
        path: path.to_owned(),
        reason,
    })
}

/// Writes `properties` the way the C# app does: 2-space indent, CRLF line endings, no byte order mark, no trailing
/// newline. Only the given path is written.
pub fn write_properties(path: &Path, properties: &SucroseProperties) -> io::Result<()> {
    let text = serde_json::to_string_pretty(properties).map_err(io::Error::other)?;
    // Line breaks inside strings are escaped, so every raw `\n` is structural.
    fs::write(path, text.replace('\n', "\r\n"))
}
