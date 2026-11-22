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

/// Data folder of the installed C# app, or `None` when the OS reports no config directory.
///
/// Read-only: Sucrose never writes here (D-008). Unlike [`data_dir`] it ignores `SUCROSE_DATA_DIR`, because that
/// override only redirects what the Rust app writes; the C# install is always read from its real folder.
pub fn legacy_data_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_NAME))
}

/// Folder with the C# app's `Setting/*.json` files. Read-only, see [`legacy_data_dir`].
pub fn legacy_setting_dir() -> Option<PathBuf> {
    legacy_data_dir().map(|dir| dir.join("Setting"))
}

/// Folder where the C# app's mpv player keeps its cache (`Cache\MpvPlayer`). Read-only, see [`legacy_data_dir`].
pub fn legacy_mpv_cache_dir() -> Option<PathBuf> {
    legacy_data_dir().map(|dir| dir.join("Cache").join("MpvPlayer"))
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

    #[test]
    fn legacy_folders_ignore_the_data_dir_override() {
        let Some(config) = dirs::config_dir() else {
            return;
        };
        let root = config.join(APP_NAME);
        assert_eq!(legacy_data_dir().unwrap(), root);
        assert_eq!(legacy_setting_dir().unwrap(), root.join("Setting"));
        assert_eq!(legacy_mpv_cache_dir().unwrap(), root.join("Cache").join("MpvPlayer"));
    }
}
