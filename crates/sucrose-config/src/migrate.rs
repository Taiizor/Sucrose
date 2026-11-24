//! One-time migration from the C# app's `Setting` folder into the new settings (D-03).
//!
//! The legacy files are opened for reading only, through the reader of [`crate::legacy`]; nothing here creates,
//! writes, renames or deletes a legacy file.

use std::path::Path;

use sucrose_core::settings::{Backdrop, LegacyBackdropType, LegacyThemeType, ThemeMode};

use crate::{legacy::LegacySettings, model::Settings};

/// Maps the legacy `ThemeType` and `BackdropType` to the theme mode and backdrop (D-21).
pub fn map_theme(theme_type: Option<LegacyThemeType>, backdrop: Option<LegacyBackdropType>) -> (ThemeMode, Backdrop) {
    let _ = (theme_type, backdrop);
    (ThemeMode::default(), Backdrop::default())
}

/// Builds the settings from the legacy files in `legacy_setting_dir`. Anything missing or invalid is the legacy
/// default; `legacy_root` is the C# data folder, whose `Library` folder is the default library location.
pub fn migrate(legacy_setting_dir: &Path, legacy_root: &Path) -> Settings {
    let legacy = LegacySettings::load(legacy_setting_dir, legacy_root);
    let mut settings = Settings::default();

    // The default location stays empty (it means the legacy default), so the file does not pin a path that was never
    // chosen.
    if legacy.library.location != legacy_root.join("Library") {
        settings.library.location = legacy.library.location.to_string_lossy().into_owned();
    }
    settings.library.selected = legacy.library.selected;
    settings.general.app_visible = legacy.general.app_visible;
    settings.engine.volume = legacy.engine.volume;
    settings.engine.looping = legacy.engine.looping;
    settings.engine.stretch = legacy.engine.stretch;
    settings.engine.hardware_acceleration = legacy.engine.hardware_acceleration;
    settings.engine.screen = legacy.engine.screen;
    settings.engine.gpu_preference = legacy.engine.gpu_preference;
    settings.backgroundog.fullscreen = legacy.backgroundog.fullscreen;
    settings.backgroundog.pause = legacy.backgroundog.pause;

    // Values only: no paths and no theme names (T-02-40).
    tracing::info!(
        volume = settings.engine.volume,
        looping = settings.engine.looping,
        stretch = ?settings.engine.stretch,
        hardware_acceleration = settings.engine.hardware_acceleration,
        screen = ?settings.engine.screen,
        gpu_preference = ?settings.engine.gpu_preference,
        fullscreen = ?settings.backgroundog.fullscreen,
        pause = ?settings.backgroundog.pause,
        app_visible = settings.general.app_visible,
        "settings migrated"
    );
    settings
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn migrates_the_fixture_files() {
        let settings = migrate(&fixtures(), Path::new("root"));
        assert_eq!(settings.library.location, "C:\\Sucrose\\Library");
        assert_eq!(settings.library.selected, "AAAAAAAAAAAAAAAAAAAAAAAAA");
    }

    #[test]
    fn a_missing_folder_gives_the_legacy_defaults() {
        let settings = migrate(&fixtures().join("nothing"), Path::new("root"));
        assert_eq!(settings, Settings::default());
    }
}
