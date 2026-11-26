//! The script strings of the web wallpaper JavaScript API, built exactly as the C# app builds them (WAPI-01, WAPI-02).
//!
//! The engine never gives a page a host object or a message channel: every call is a one-way script string that the
//! engine runs in the page (docs/legacy/engines.md section 6). This module only builds the strings; the web engines run
//! them. C# builds a property call with `string.Format(listener, key, JsonConvert.SerializeObject(control, Formatting.Indented))`,
//! so the text here follows .NET composite formatting and Newtonsoft's indented output (two spaces, CRLF).

use std::{
    fs,
    path::{Component, Path},
};

use serde_json::Value;
use sucrose_core::{
    compatible::{COMPATIBLE_FILE, CompatibleError, CompatibleKey, read_compatible},
    properties::{Control, ControlKind, PROPERTIES_FILE, PropertiesError, read_properties},
    settings::Stretch,
};

/// Why a format string could not be applied: the cases where .NET throws a `FormatException`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormatError {
    /// A brace is neither closed nor doubled.
    #[error("a brace is not closed or doubled")]
    Brace,
    /// A placeholder is not `{index[,alignment][:format]}`.
    #[error("a placeholder is not valid")]
    Placeholder,
    /// A placeholder names an argument that was not given.
    #[error("placeholder {0} has no argument")]
    Index(usize),
}

/// Applies a .NET composite format string to text arguments: `{0}`, `{1}` are replaced, `{{` and `}}` stand for one
/// brace, `{0,5}` aligns (negative aligns left) and a `:format` part is ignored, as it is for strings.
///
/// # Errors
/// [`FormatError`] for what makes .NET throw: a lone brace, a malformed placeholder or an index with no argument.
pub fn try_csharp_format(format: &str, args: &[&str]) -> Result<String, FormatError> {
    let mut out = String::with_capacity(format.len() + args.iter().map(|arg| arg.len()).sum::<usize>());
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '}' => return Err(FormatError::Brace),
            '{' => {
                let mut body = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some('{') => return Err(FormatError::Placeholder),
                        Some(other) => body.push(other),
                        None => return Err(FormatError::Brace),
                    }
                }
                let spec = body.split_once(':').map_or(body.as_str(), |(head, _)| head);
                let (index, alignment) = spec.split_once(',').map_or((spec, None), |(i, a)| (i, Some(a)));
                let index: usize = index.trim().parse().map_err(|_| FormatError::Placeholder)?;
                let alignment: i32 = match alignment {
                    Some(text) => text.trim().parse().map_err(|_| FormatError::Placeholder)?,
                    None => 0,
                };
                let arg = args.get(index).ok_or(FormatError::Index(index))?;
                let width = usize::try_from(alignment.unsigned_abs()).unwrap_or(usize::MAX);
                let pad = width.saturating_sub(arg.chars().count());
                if alignment > 0 {
                    out.extend(std::iter::repeat_n(' ', pad));
                }
                out.push_str(arg);
                if alignment < 0 {
                    out.extend(std::iter::repeat_n(' ', pad));
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// [`try_csharp_format`] for callers that skip a failing call: the legacy engine catches the `FormatException` of a
/// bad template and runs nothing, so the result is empty on error, and an empty script is never run.
#[must_use]
pub fn csharp_format(format: &str, args: &[&str]) -> String {
    try_csharp_format(format, args).unwrap_or_default()
}

/// The JSON text Newtonsoft writes for `Formatting.Indented`: two-space indentation, `"key": value` separators, empty
/// arrays and objects as `[]` and `{}`, CRLF line ends and the keys in the order of `value`.
#[must_use]
pub fn indented_json(value: &Value) -> String {
    // Line breaks inside strings are escaped, so every raw LF is structural.
    serde_json::to_string_pretty(value)
        .unwrap_or_default()
        .replace('\n', "\r\n")
}

/// The script of one property call: the theme's listener format with the key and the indented control.
#[must_use]
pub fn property_call(listener: &str, key: &str, control: &Value) -> String {
    csharp_format(listener, &[key, &indented_json(control)])
}

/// The kind of a property call (see [`PageCall::kind`]).
pub const KIND_PROPERTY: &str = "property";
/// The kind of a compatible call.
pub const KIND_COMPATIBLE: &str = "compatible";
/// The kind of the in-page pause and resume calls.
pub const KIND_PAUSE: &str = "pause";

/// One script to run in the main frame of a page, with what it is for (the log shows `kind` and `key`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageCall {
    /// `property`, `compatible` or `pause`.
    pub kind: &'static str,
    /// The property key, the compatible key or `pause`/`resume`.
    pub key: String,
    /// The JavaScript to run.
    pub script: String,
}

/// The names in `folder` of the theme that match `filter`, as the legacy `ListFiles` lists them: files only, sorted
/// ignoring case, `filter` is `|`-separated `*.ext` patterns and `*` or `*.*` means every file. The folder must be
/// inside the theme by canonical path (SEC-02): an absolute folder, a `..` that leaves the theme and a link or junction
/// that leaves it all give an empty list, as does a folder that does not exist.
#[must_use]
pub fn filedropdown_files(theme_dir: &Path, folder: &str, filter: &str) -> Vec<String> {
    list_files(theme_dir, folder, filter).unwrap_or_default()
}

fn list_files(theme_dir: &Path, folder: &str, filter: &str) -> Option<Vec<String>> {
    let relative = Path::new(folder);
    if relative.has_root()
        || relative
            .components()
            .any(|part| matches!(part, Component::Prefix(_) | Component::RootDir))
    {
        return None;
    }
    let root = fs::canonicalize(theme_dir).ok()?;
    let target = fs::canonicalize(root.join(relative)).ok()?;
    if !target.starts_with(&root) {
        return None;
    }
    let patterns: Vec<&str> = filter
        .split('|')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .collect();
    let all = patterns.iter().any(|pattern| matches!(*pattern, "*" | "*.*"));
    let extensions: Vec<String> = patterns
        .iter()
        .map(|pattern| pattern.replace('*', "").to_lowercase())
        .filter(|extension| !extension.is_empty())
        .collect();
    let mut names: Vec<String> = fs::read_dir(&target)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            let lower = name.to_lowercase();
            all || extensions.iter().any(|extension| lower.ends_with(extension.as_str()))
        })
        .collect();
    names.sort_by_key(|name| name.to_uppercase());
    Some(names)
}

/// The JSON of one control as the page receives it. A filedropdown gets its `files` listed from its folder; one whose
/// folder or filter is blank gets `null`, as the legacy listing returns null for it.
#[must_use]
pub fn control_json(theme_dir: &Path, control: &Control) -> Value {
    let mut control = control.clone();
    let mut blank = false;
    if let ControlKind::Filedropdown {
        folder, filter, files, ..
    } = &mut control.kind
    {
        blank = folder.trim().is_empty() || filter.trim().is_empty();
        *files = (!blank).then(|| filedropdown_files(theme_dir, folder, filter));
    }
    let mut value = control.to_json_value();
    if blank && let Value::Object(object) = &mut value {
        object.insert("files".to_owned(), Value::Null);
    }
    value
}

/// The property calls of a theme, one per control in file order, as the engine runs them after the main frame loaded.
/// A theme without a `SucroseProperties.json` or with an empty list has none. A call whose listener cannot be
/// formatted or is empty is left out, as the legacy engine skips it.
///
/// # Errors
/// The read error of a file that exists but cannot be used; the page then simply gets no property calls.
pub fn property_calls(theme_dir: &Path) -> Result<Vec<PageCall>, PropertiesError> {
    let properties = match read_properties(&theme_dir.join(PROPERTIES_FILE)) {
        Ok(properties) => properties,
        Err(PropertiesError::Missing(_)) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(properties
        .list
        .iter()
        .filter_map(|(key, control)| {
            let script = property_call(&properties.listener, key, &control_json(theme_dir, control));
            (!script.is_empty()).then(|| PageCall {
                kind: KIND_PROPERTY,
                key: key.clone(),
                script,
            })
        })
        .collect())
}

/// The value a compatible template is formatted with, written as C# writes it into `string.Format`.
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibleValue {
    /// A C# bool: `True` or `False`, capitalised.
    Bool(bool),
    /// An enum, written by its legacy name (`UniformToFill`, `Dark`).
    Name(String),
    /// A whole number (`VolumeLevel`).
    Int(i64),
    /// A data payload, written as Newtonsoft's indented JSON (the system data of the data hub).
    Json(Value),
}

impl CompatibleValue {
    /// The text the template's `{0}` becomes.
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Self::Bool(true) => "True".to_owned(),
            Self::Bool(false) => "False".to_owned(),
            Self::Name(name) => name.clone(),
            Self::Int(number) => number.to_string(),
            Self::Json(value) => indented_json(value),
        }
    }
}

/// The script of one compatible call: the template with `{0}` replaced by the value; empty when the template cannot
/// be formatted (the legacy engine skips such a call).
#[must_use]
pub fn compatible_call(template: &str, value: &CompatibleValue) -> String {
    csharp_format(template, &[&value.render()])
}

/// The order the legacy engine runs the compatible templates in (`Helper/Compatible.cs` of the legacy engine). It is
/// not the order of the table in docs/legacy/engines.md and of `CompatibleKey::ALL`: the code runs `SystemAudio`
/// before `SystemTheme` and `SystemTheme` before `ShuffleMode`, `StretchMode` and `VolumeLevel`.
pub const LEGACY_RUN_ORDER: [CompatibleKey; 16] = [
    CompatibleKey::LoopMode,
    CompatibleKey::ThemeType,
    CompatibleKey::SystemBios,
    CompatibleKey::SystemDate,
    CompatibleKey::SystemAudio,
    CompatibleKey::SystemTheme,
    CompatibleKey::ShuffleMode,
    CompatibleKey::StretchMode,
    CompatibleKey::VolumeLevel,
    CompatibleKey::SystemMemory,
    CompatibleKey::SystemBattery,
    CompatibleKey::SystemGraphic,
    CompatibleKey::SystemNetwork,
    CompatibleKey::SystemStorage,
    CompatibleKey::SystemProcessor,
    CompatibleKey::SystemMotherboard,
];

/// What the engine itself knows for the non-data compatible keys: the values behind `LoopMode`, `ThemeType`,
/// `SystemTheme`, `ShuffleMode`, `StretchMode` and `VolumeLevel` (the sound rules of Round D adjust the volume later).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibleState {
    /// `Engine.WallpaperLoop`.
    pub looping: bool,
    /// `Engine.WallpaperShuffle`.
    pub shuffle: bool,
    /// `Engine.StretchType`, written by its legacy name.
    pub stretch: Stretch,
    /// The legacy `ThemeType` name (`Dark` or `Light`), the value of both `ThemeType` and `SystemTheme`.
    pub theme_type: String,
    /// The volume, 0..=100.
    pub volume: u8,
}

impl CompatibleState {
    /// The value of `key`; `None` for the system data keys, which the data hub fills.
    #[must_use]
    pub fn value(&self, key: CompatibleKey) -> Option<CompatibleValue> {
        match key {
            CompatibleKey::LoopMode => Some(CompatibleValue::Bool(self.looping)),
            CompatibleKey::ShuffleMode => Some(CompatibleValue::Bool(self.shuffle)),
            CompatibleKey::StretchMode => Some(CompatibleValue::Name(self.stretch.legacy_name().to_owned())),
            CompatibleKey::ThemeType | CompatibleKey::SystemTheme => {
                Some(CompatibleValue::Name(self.theme_type.clone()))
            }
            CompatibleKey::VolumeLevel => Some(CompatibleValue::Int(i64::from(self.volume))),
            CompatibleKey::SystemBios
            | CompatibleKey::SystemDate
            | CompatibleKey::SystemAudio
            | CompatibleKey::SystemMemory
            | CompatibleKey::SystemBattery
            | CompatibleKey::SystemGraphic
            | CompatibleKey::SystemNetwork
            | CompatibleKey::SystemStorage
            | CompatibleKey::SystemProcessor
            | CompatibleKey::SystemMotherboard => None,
        }
    }
}

/// The calls of the non-data templates in [`LEGACY_RUN_ORDER`], whatever the order of the file: `templates` are the
/// non-empty templates of a theme (`SucroseCompatible::templates`), and a template that cannot be formatted is left
/// out. The data keys are not here: they run with every data push.
#[must_use]
pub fn non_data_calls(templates: &[(CompatibleKey, &str)], state: &CompatibleState) -> Vec<PageCall> {
    LEGACY_RUN_ORDER
        .iter()
        .filter_map(|key| {
            let value = state.value(*key)?;
            let (_, template) = templates.iter().find(|(candidate, _)| candidate == key)?;
            let script = compatible_call(template, &value);
            (!script.is_empty()).then(|| PageCall {
                kind: KIND_COMPATIBLE,
                key: key.name().to_owned(),
                script,
            })
        })
        .collect()
}

/// The non-data compatible calls of a theme for `state`. A theme without a `SucroseCompatible.json` has none.
///
/// # Errors
/// The read error of a file that exists but cannot be used; the page then gets no compatible calls.
pub fn compatible_calls(theme_dir: &Path, state: &CompatibleState) -> Result<Vec<PageCall>, CompatibleError> {
    let compatible = match read_compatible(&theme_dir.join(COMPATIBLE_FILE)) {
        Ok(compatible) => compatible,
        Err(PropertiesError::Missing(_)) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(non_data_calls(&compatible.templates(), state))
}

const PAUSE_SCRIPT: &str = r#"(function () {
  try {
    var state = window.__sucroseLight || (window.__sucroseLight = { media: [], hidden: false });
    if (!state.hidden) {
      state.media = [];
      Array.prototype.forEach.call(document.querySelectorAll("video, audio"), function (element) {
        if (!element.paused) {
          state.media.push(element);
          element.pause();
        }
      });
      Object.defineProperty(document, "visibilityState", { configurable: true, get: function () { return "hidden"; } });
      Object.defineProperty(document, "hidden", { configurable: true, get: function () { return true; } });
      state.hidden = true;
      document.dispatchEvent(new Event("visibilitychange"));
    }
    if (typeof SucrosePause === "function") { SucrosePause(); }
  } catch (error) { }
})();"#;

const RESUME_SCRIPT: &str = r#"(function () {
  try {
    var state = window.__sucroseLight || (window.__sucroseLight = { media: [], hidden: false });
    if (state.hidden) {
      delete document.visibilityState;
      delete document.hidden;
      state.hidden = false;
      document.dispatchEvent(new Event("visibilitychange"));
      state.media.forEach(function (element) {
        var started = element.play();
        if (started && started.catch) { started.catch(function () { }); }
      });
      state.media = [];
    }
    if (typeof SucrosePlay === "function") { SucrosePlay(); }
  } catch (error) { }
})();"#;

/// The script of the Light pause inside the page (PLAY-05, D-22): playing media elements are paused and remembered, the
/// page is told it is hidden (`document.visibilityState` and a `visibilitychange` event) and `SucrosePause` runs when
/// the page defines it. `false` undoes exactly that: the remembered media play again, the visibility is restored and
/// `SucrosePlay` runs when defined. No process is suspended and the page gets no host object. Running either twice
/// in a row is harmless.
#[must_use]
pub fn pause_script(paused: bool) -> &'static str {
    if paused { PAUSE_SCRIPT } else { RESUME_SCRIPT }
}

/// The pause or resume of the page as a call to run.
#[must_use]
pub fn pause_call(paused: bool) -> PageCall {
    PageCall {
        kind: KIND_PAUSE,
        key: if paused { "pause" } else { "resume" }.to_owned(),
        script: pause_script(paused).to_owned(),
    }
}
