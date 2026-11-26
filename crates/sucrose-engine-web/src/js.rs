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
use sucrose_core::properties::{Control, ControlKind, PROPERTIES_FILE, PropertiesError, read_properties};

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

/// What a page call is, for the log and the call report of the fixture pages.
pub const KIND_PROPERTY: &str = "property";

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
