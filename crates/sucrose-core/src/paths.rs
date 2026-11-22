//! Locations of Sucrose's data folders.
//!
//! On Windows the root is `%APPDATA%\Sucrose`, the same folder the C# app uses, so both can share data during the
//! rewrite (D-007). Elsewhere it is the platform config directory. Set `SUCROSE_DATA_DIR` to use another folder, for
//! example during development or in tests, so the real data is not touched.

use std::path::PathBuf;

use crate::APP_NAME;

/// Environment variable that overrides the data folder.
pub const DATA_DIR_ENV: &str = "SUCROSE_DATA_DIR";

/// Root data folder, or `None` when the OS reports no config directory.
pub fn data_dir() -> Option<PathBuf> {
    match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => dirs::config_dir().map(|dir| dir.join(APP_NAME)),
    }
}

/// Folder for daily log files.
pub fn log_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Log"))
}

/// Folder for settings.
pub fn setting_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Setting"))
}

/// Folder for caches (store, engines, properties).
pub fn cache_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Cache"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_live_under_the_data_dir() {
        let Some(root) = data_dir() else {
            return;
        };
        assert_eq!(log_dir().unwrap(), root.join("Log"));
        assert_eq!(setting_dir().unwrap(), root.join("Setting"));
        assert_eq!(cache_dir().unwrap(), root.join("Cache"));
    }
}
