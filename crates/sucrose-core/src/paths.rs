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

/// Folder for the persistent web data of the web engines (`Web/Cef`, later `Web/WebView2`; D-35).
///
/// It sits beside [`cache_dir`], not inside it, so cleaning the cache never deletes a wallpaper's localStorage. The C#
/// app's `Cache/CefSharp` and `Cache/WebView2` folders are never read.
pub fn web_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Web"))
}

/// Folder with the themes shipped for first use; each is copied into the library once (D-10).
pub fn showcase_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Showcase"))
}

/// Folder with the per-theme property caches (`Cache\Properties`).
pub fn properties_cache_dir() -> Option<PathBuf> {
    cache_dir().map(|dir| dir.join("Properties"))
}

/// Folder with the Rust app's own working copies of the property files (`<Theme><Type>`, D-20).
///
/// It sits beside [`cache_dir`], not inside it: a working copy holds what the user chose and cleaning the cache must
/// never delete it. The service is its only writer (Pattern 6).
pub fn properties_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("Properties"))
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

/// Folder with the C# app's per-theme property caches (`Cache\Properties`). Read-only, see [`legacy_data_dir`]: D-20 copies
/// its files into [`properties_dir`] once and never writes here.
pub fn legacy_properties_cache_dir() -> Option<PathBuf> {
    legacy_data_dir().map(|dir| dir.join("Cache").join("Properties"))
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
        assert_eq!(showcase_dir().unwrap(), root.join("Showcase"));
        assert_eq!(web_dir().unwrap(), root.join("Web"));
        assert!(
            !web_dir().unwrap().starts_with(cache_dir().unwrap()),
            "web data lives outside Cache (D-35)"
        );
        assert_eq!(properties_cache_dir().unwrap(), root.join("Cache").join("Properties"));
        assert_eq!(properties_dir().unwrap(), root.join("Properties"));
        assert!(
            !properties_dir().unwrap().starts_with(cache_dir().unwrap()),
            "working copies live outside Cache (D-20)"
        );
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
        assert_eq!(
            legacy_properties_cache_dir().unwrap(),
            root.join("Cache").join("Properties")
        );
    }
}
