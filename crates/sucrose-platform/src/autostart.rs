//! Starting Sucrose with the OS (TRAY-03, D-08, D-034).

use std::path::Path;

use crate::PlatformError;

/// The Windows Run value Sucrose's own start entry uses (D-034).
pub const RUN_VALUE_NAME: &str = "Sucrose.Service";
/// The label of the macOS LaunchAgent (D-034).
pub const LAUNCH_AGENT_LABEL: &str = "dev.sucrose.Sucrose";
/// The file name of the XDG autostart entry (D-034).
pub const DESKTOP_FILE_NAME: &str = "sucrose.desktop";

/// What the OS says about Sucrose's start entry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutostartState {
    /// Sucrose's own entry for this executable exists.
    pub entry_present: bool,
    /// The OS turned the entry off (Task Manager's Startup apps).
    pub disabled_by_os: bool,
    /// An entry of the C# app exists.
    pub legacy_entry: bool,
}

/// Where an entry lives as a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A macOS LaunchAgent plist.
    LaunchAgent,
    /// A Linux XDG autostart file.
    Desktop,
}

/// The LaunchAgent plist text.
pub fn launch_agent_plist(_label: &str, _exe: &str) -> String {
    String::new()
}

/// The XDG autostart file text.
pub fn desktop_entry(_name: &str, _exe: &str) -> String {
    String::new()
}

/// The data of the Run value.
pub fn run_value_data(_exe: &Path) -> Result<String, PlatformError> {
    Ok(String::new())
}

/// Whether the StartupApproved bytes say the entry is turned off.
pub fn disabled_by_os(_bytes: &[u8]) -> bool {
    false
}

/// The autostart folder's parent.
pub fn config_dir_from(_xdg: Option<&std::ffi::OsStr>, _home: Option<&std::ffi::OsStr>) -> Option<std::path::PathBuf> {
    None
}

/// The file text of an entry.
pub fn entry_text(_kind: EntryKind, _exe: &Path) -> Result<String, PlatformError> {
    Ok(String::new())
}

/// The path of an entry under `base`.
pub fn entry_path(_kind: EntryKind, base: &Path) -> std::path::PathBuf {
    base.to_path_buf()
}

/// Writes an entry.
pub fn write_entry(_kind: EntryKind, _base: &Path, _exe: &Path) -> Result<(), PlatformError> {
    Ok(())
}

/// Removes an entry.
pub fn remove_entry(_kind: EntryKind, _base: &Path) -> Result<(), PlatformError> {
    Ok(())
}

/// The state of an entry file.
pub fn entry_state(_kind: EntryKind, _base: &Path, _exe: &Path) -> AutostartState {
    AutostartState::default()
}

/// Turns Sucrose's own start entry on.
pub fn enable_autostart(_exe: &Path) -> Result<(), PlatformError> {
    Ok(())
}

/// Turns Sucrose's own start entry off.
pub fn disable_autostart() -> Result<(), PlatformError> {
    Ok(())
}

/// The state of Sucrose's own start entry.
pub fn autostart_state(_exe: &Path) -> AutostartState {
    AutostartState::default()
}

/// Whether the C# app has a start entry.
pub fn legacy_entry() -> bool {
    false
}
