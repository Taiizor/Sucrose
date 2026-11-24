//! Starting Sucrose with the OS (TRAY-03, D-08, D-034).
//!
//! Sucrose writes its own start entry only when the user turns Normal startup on in the Portal: a Run value on Windows,
//! a LaunchAgent on macOS and an XDG autostart file on Linux. The text of those entries is built by pure functions that
//! run (and are tested) on every OS; the code that touches the registry or the folders sits behind
//! [`enable_autostart`], [`disable_autostart`] and [`autostart_state`]. Entries of the C# app are never written,
//! renamed or removed (D-08).

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::PlatformError;

/// The Windows Run value Sucrose's own start entry uses (D-034). The C# app uses "Sucrose", so both can exist.
pub const RUN_VALUE_NAME: &str = "Sucrose.Service";
/// The label of the macOS LaunchAgent (D-034).
pub const LAUNCH_AGENT_LABEL: &str = "dev.sucrose.Sucrose";
/// The file name of the XDG autostart entry (D-034).
pub const DESKTOP_FILE_NAME: &str = "sucrose.desktop";
/// The name the desktop entry shows.
const DESKTOP_ENTRY_NAME: &str = "Sucrose";

/// What the OS says about Sucrose's start entry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutostartState {
    /// Sucrose's own entry for this executable exists.
    pub entry_present: bool,
    /// The OS turned the entry off (Task Manager's Startup apps).
    pub disabled_by_os: bool,
    /// An entry of the C# app exists. [`autostart_state`] never looks for it; see [`legacy_entry`].
    pub legacy_entry: bool,
}

/// Where an entry lives as a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A macOS LaunchAgent plist under `~/Library/LaunchAgents`.
    LaunchAgent,
    /// A Linux XDG autostart file under `$XDG_CONFIG_HOME/autostart`.
    Desktop,
}

fn unusable(message: &str) -> PlatformError {
    PlatformError::Failed {
        context: "startup entry",
        message: message.to_owned(),
    }
}

/// The text of `exe`, or why it cannot go into an entry: it must be valid Unicode, not empty and free of control
/// characters (a newline would start another line of a file entry).
fn exe_text(exe: &Path) -> Result<&str, PlatformError> {
    let text = exe
        .to_str()
        .ok_or_else(|| unusable("the executable path is not valid Unicode"))?;
    if text.is_empty() {
        return Err(unusable("the executable path is empty"));
    }
    if text.chars().any(char::is_control) {
        return Err(unusable("the executable path has a control character"));
    }
    Ok(text)
}

/// `text` without the verbatim prefix (two backslashes, a question mark and a backslash) that Windows puts before a
/// drive path.
fn without_verbatim_prefix(text: &str) -> &str {
    match text.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest,
        _ => text,
    }
}

/// The data of the Run value for `exe`: the path in double quotes and nothing after it. A path that holds a double
/// quote is refused, because the shell would read the rest as arguments.
pub fn run_value_data(exe: &Path) -> Result<String, PlatformError> {
    let text = without_verbatim_prefix(exe_text(exe)?);
    if text.contains('"') {
        return Err(unusable("the executable path has a double quote"));
    }
    Ok(format!("\"{text}\""))
}

/// Whether the bytes of a StartupApproved value say Task Manager turned the entry off. The first byte tells: 0x02 and
/// 0x06 are enabled, 0x03 and 0x07 disabled; a missing value (no bytes) is enabled.
pub fn disabled_by_os(bytes: &[u8]) -> bool {
    matches!(bytes.first(), Some(0x03 | 0x07))
}

/// `text` with the characters XML reserves in text replaced.
fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

/// The LaunchAgent that starts `exe` with no arguments when the user logs in.
pub fn launch_agent_plist(label: &str, exe: &str) -> String {
    let (label, exe) = (xml_escape(label), xml_escape(exe));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
\t<key>Label</key>\n\
\t<string>{label}</string>\n\
\t<key>ProgramArguments</key>\n\
\t<array>\n\
\t\t<string>{exe}</string>\n\
\t</array>\n\
\t<key>RunAtLoad</key>\n\
\t<true/>\n\
</dict>\n\
</plist>\n"
    )
}

/// The characters that make a Desktop Entry `Exec` argument need double quotes.
const EXEC_RESERVED: &str = " \t\n\"'\\><~|&;$*?#()\u{60}";

/// `text` as one `Exec` argument of a Desktop Entry: `%` doubled (it starts a field code), the argument quoted when
/// it holds a reserved character with quote, backtick, dollar sign and backslash escaped inside the quotes, and then the string
/// rule of the file format applied on top (every backslash doubled, line breaks and tabs written as escapes), so no
/// character of a path can start another line of the file.
fn exec_argument(text: &str) -> String {
    let text = text.replace('%', "%%");
    let quoted = if text.chars().any(|c| EXEC_RESERVED.contains(c)) {
        let mut out = String::with_capacity(text.len() + 2);
        out.push('"');
        for c in text.chars() {
            if matches!(c, '"' | '\u{60}' | '$' | '\\') {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
        out
    } else {
        text
    };
    string_value(&quoted)
}

/// `text` as a value of type string in a Desktop Entry file.
fn string_value(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

/// The XDG autostart entry that starts `exe` with no arguments when the user logs in.
pub fn desktop_entry(name: &str, exe: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        string_value(name),
        exec_argument(exe)
    )
}

/// The folder that holds the `autostart` folder: `xdg` (`XDG_CONFIG_HOME`) when it is an absolute path, else
/// `.config` under `home`. The XDG rules say a relative value is invalid and must be ignored.
pub fn config_dir_from(xdg: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(xdg) = xdg.map(Path::new).filter(|path| path.is_absolute()) {
        return Some(xdg.to_path_buf());
    }
    home.filter(|home| !home.is_empty())
        .map(|home| Path::new(home).join(".config"))
}

/// The text of an entry file for `exe`.
pub fn entry_text(kind: EntryKind, exe: &Path) -> Result<String, PlatformError> {
    let text = exe_text(exe)?;
    Ok(match kind {
        EntryKind::LaunchAgent => launch_agent_plist(LAUNCH_AGENT_LABEL, text),
        EntryKind::Desktop => desktop_entry(DESKTOP_ENTRY_NAME, text),
    })
}

/// The path of an entry file: `base` is the user's home folder for a LaunchAgent and the config folder (see
/// [`config_dir_from`]) for a desktop entry.
pub fn entry_path(kind: EntryKind, base: &Path) -> PathBuf {
    match kind {
        EntryKind::LaunchAgent => base
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{LAUNCH_AGENT_LABEL}.plist")),
        EntryKind::Desktop => base.join("autostart").join(DESKTOP_FILE_NAME),
    }
}

fn io_failure(context: &'static str, error: &std::io::Error) -> PlatformError {
    // The text names no path: callers log it.
    PlatformError::Failed {
        context,
        message: error.kind().to_string(),
    }
}

/// Writes Sucrose's own entry file for `exe` (the folders are created), through a temporary file in the same folder
/// so a reader never sees half of it.
pub fn write_entry(kind: EntryKind, base: &Path, exe: &Path) -> Result<(), PlatformError> {
    let text = entry_text(kind, exe)?;
    let path = entry_path(kind, base);
    let folder = path.parent().ok_or_else(|| unusable("the entry has no folder"))?;
    std::fs::create_dir_all(folder).map_err(|error| io_failure("creating the startup folder", &error))?;
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, text).map_err(|error| io_failure("writing the startup entry", &error))?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        io_failure("writing the startup entry", &error)
    })
}

/// Removes Sucrose's own entry file and nothing else; a missing file is success.
pub fn remove_entry(kind: EntryKind, base: &Path) -> Result<(), PlatformError> {
    match std::fs::remove_file(entry_path(kind, base)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure("removing the startup entry", &error)),
    }
}

/// What an entry file says: present when it exists and starts `exe`. Files carry no "turned off by the OS" flag.
pub fn entry_state(kind: EntryKind, base: &Path, exe: &Path) -> AutostartState {
    let Ok(expected) = entry_text(kind, exe) else {
        return AutostartState::default();
    };
    let entry_present = std::fs::read_to_string(entry_path(kind, base)).is_ok_and(|found| found == expected);
    AutostartState {
        entry_present,
        ..AutostartState::default()
    }
}

/// Turns Sucrose's own start entry on for `exe` (the running service's own path, never text from a client).
pub fn enable_autostart(exe: &Path) -> Result<(), PlatformError> {
    crate::imp::enable_autostart(exe)
}

/// Turns Sucrose's own start entry off. Only Sucrose's own entry is removed; a missing one is success.
pub fn disable_autostart() -> Result<(), PlatformError> {
    crate::imp::disable_autostart()
}

/// The state of Sucrose's own start entry for `exe`. Cheap: it never looks for the C# entry, so
/// [`AutostartState::legacy_entry`] is false; ask [`legacy_entry`] for that.
pub fn autostart_state(exe: &Path) -> AutostartState {
    crate::imp::autostart_state(exe)
}

/// Whether the C# app has a start entry (the Run value "Sucrose" or its scheduled task) on Windows; false elsewhere.
/// Read-only, and it may start a short-lived process, so the caller runs it off the UI thread and only when asked.
pub fn legacy_entry() -> bool {
    crate::imp::legacy_entry()
}
