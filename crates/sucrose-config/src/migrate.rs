//! One-time migration from the C# app's `Setting` folder into the new settings (D-03).
//!
//! The legacy files are opened for reading only, through the reader of [`crate::legacy`]; nothing here creates,
//! writes, renames or deletes a legacy file.

use std::path::Path;

use sucrose_core::settings::{Backdrop, LegacyBackdropType, LegacyThemeType, SortKind, SortMode, Stretch, ThemeMode};

use crate::{
    legacy::{BackgroundogSettings, EngineSettings, GeneralSettings, LegacyFile, LibrarySettings, Properties, read},
    model::{DEFAULT_USER_AGENT, MIN_DATE_TIME, Settings},
};

/// Maps the legacy `ThemeType` and `BackdropType` to the theme mode and backdrop (D-21).
pub fn map_theme(theme_type: Option<LegacyThemeType>, backdrop: Option<LegacyBackdropType>) -> (ThemeMode, Backdrop) {
    let backdrop_mode = match backdrop {
        Some(LegacyBackdropType::Auto) => Backdrop::Mica,
        Some(LegacyBackdropType::Mica) => Backdrop::Mica,
        Some(LegacyBackdropType::Acrylic) => Backdrop::Acrylic,
        Some(LegacyBackdropType::Tabbed) => Backdrop::Tabbed,
        Some(LegacyBackdropType::None) | None => Backdrop::None,
    };
    let mode = match (backdrop, theme_type) {
        (Some(LegacyBackdropType::Auto), _) | (_, None) => ThemeMode::System,
        (_, Some(LegacyThemeType::Dark)) => ThemeMode::Dark,
        (_, Some(LegacyThemeType::Light)) => ThemeMode::Light,
    };
    (mode, backdrop_mode)
}

/// A number that a legacy range keeps in `min..=max`; `u8` because every range of these files is at most 0..=100.
fn small(properties: &Properties, key: &str, default: u8, min: u8, max: u8) -> u8 {
    let value = properties.get_clamped(key, i64::from(default), i64::from(min), i64::from(max));
    u8::try_from(value).unwrap_or(default)
}

/// The raw text of a legacy date-time value; the default is the legacy "never set" value.
fn raw_time(properties: &Properties, key: &str) -> String {
    properties.get_str(key).unwrap_or(MIN_DATE_TIME).to_owned()
}

/// Builds the settings from the legacy files in `legacy_setting_dir`. Anything missing or invalid is the legacy
/// default; `legacy_root` is the C# data folder, whose `Library` folder is the default library location.
///
/// Each file is read once; the General, Portal, Library and Warehouse keys are all typed fields, and `ThemeType` and
/// `BackdropType` are mapped to the theme mode and backdrop by [`map_theme`] instead of being stored (D-21).
pub fn migrate(legacy_setting_dir: &Path, legacy_root: &Path) -> Settings {
    let dir = legacy_setting_dir;
    let library = read(dir, LegacyFile::Library);
    let general = read(dir, LegacyFile::General);
    let portal = read(dir, LegacyFile::Portal);
    let warehouse = read(dir, LegacyFile::Warehouse);
    let engine = EngineSettings::from_properties(&read(dir, LegacyFile::Engine));
    let backgroundog = BackgroundogSettings::from_properties(&read(dir, LegacyFile::Backgroundog));
    let mut settings = Settings::default();

    // The default location stays empty (it means the legacy default), so the file does not pin a path that was never
    // chosen.
    let location = LibrarySettings::from_properties(&library, legacy_root);
    if location.location != legacy_root.join("Library") {
        settings.library.location = location.location.to_string_lossy().into_owned();
    }
    settings.library.selected = location.selected;
    settings.library.delete_corrupt = library.get_bool("DeleteCorrupt", false);
    settings.library.delete_confirm = library.get_bool("DeleteConfirm", true);
    settings.library.move_library = library.get_bool("Move", true);

    // `Culture` stays `None` (the OS language) unless the legacy file names one.
    settings.general.culture = general
        .get_str("Culture")
        .filter(|culture| !culture.is_empty())
        .map(str::to_owned);
    settings.general.run_startup = small(&general, "RunStartup", 0, 0, 10);
    settings.general.user_agent = general.get_str("UserAgent").unwrap_or(DEFAULT_USER_AGENT).to_owned();
    settings.general.telemetry_data = general.get_bool("TelemetryData", true);
    settings.general.exception_data = general.get_bool("ExceptionData", true);
    settings.general.app_visible = GeneralSettings::from_properties(&general).app_visible;
    settings.general.app_exit = general.get_bool("AppExit", true);

    let (theme_mode, backdrop) = map_theme(
        general.get_enum_opt(
            "ThemeType",
            LegacyThemeType::from_legacy_name,
            LegacyThemeType::from_legacy_index,
        ),
        portal.get_enum_opt(
            "BackdropType",
            LegacyBackdropType::from_legacy_name,
            LegacyBackdropType::from_legacy_index,
        ),
    );
    let target = &mut settings.portal;
    target.theme_mode = theme_mode;
    target.backdrop = backdrop;
    target.background_opacity = small(&portal, "BackgroundOpacity", 100, 0, 100);
    target.library_pagination = small(&portal, "LibraryPagination", 30, 1, 100);
    target.store_pagination = small(&portal, "StorePagination", 30, 1, 100);
    target.adaptive_layout = small(&portal, "AdaptiveLayout", 0, 0, 100);
    target.adaptive_margin = small(&portal, "AdaptiveMargin", 5, 5, 25);
    target.store_duration = small(&portal, "StoreDuration", 3, 1, 24);
    target.background_image = portal.get_str("BackgroundImage").unwrap_or_default().to_owned();
    target.library_preview_hide = portal.get_bool("LibraryPreviewHide", false);
    target.store_preview_hide = portal.get_bool("StorePreviewHide", false);
    target.library_preview = portal.get_bool("LibraryPreview", false);
    target.store_preview = portal.get_bool("StorePreview", false);
    target.store_adult = portal.get_bool("StoreAdult", true);
    target.background_stretch = portal.get_enum(
        "BackgroundStretch",
        Stretch::default(),
        Stretch::from_legacy_name,
        Stretch::from_legacy_index,
    );
    target.library_sort_kind = portal.get_enum(
        "LibrarySortKind",
        SortKind::default(),
        SortKind::from_legacy_name,
        SortKind::from_legacy_index,
    );
    target.library_sort_mode = portal.get_enum(
        "LibrarySortMode",
        SortMode::default(),
        SortMode::from_legacy_name,
        SortMode::from_legacy_index,
    );

    // The time strings stay raw: the C# values carry seven fractional digits and an offset.
    settings.warehouse.showcase = warehouse.get_string_list("Showcase");
    settings.warehouse.cef_sharp_time = raw_time(&warehouse, "CefSharpTime");
    settings.warehouse.web_view_time = raw_time(&warehouse, "WebViewTime");
    settings.warehouse.cefsharp_continue = warehouse.get_bool("CefsharpContinue", false);
    settings.warehouse.web_view_continue = warehouse.get_bool("WebViewContinue", false);
    settings.warehouse.hint_tray_icon = warehouse.get_bool("HintTrayIcon", true);

    settings.engine.volume = engine.volume;
    settings.engine.looping = engine.looping;
    settings.engine.stretch = engine.stretch;
    settings.engine.hardware_acceleration = engine.hardware_acceleration;
    settings.engine.screen = engine.screen;
    settings.engine.gpu_preference = engine.gpu_preference;
    settings.backgroundog.fullscreen = backgroundog.fullscreen;
    settings.backgroundog.pause = backgroundog.pause;

    // Values only: no paths, no theme names and no showcase names (T-02-40).
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
        theme_mode = ?settings.portal.theme_mode,
        backdrop = ?settings.portal.backdrop,
        showcase = settings.warehouse.showcase.len(),
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
