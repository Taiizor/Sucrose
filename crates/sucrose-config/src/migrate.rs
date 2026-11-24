//! One-time migration from the C# app's `Setting` folder into the new settings (D-03).
//!
//! The legacy files are opened for reading only, through the reader of [`crate::legacy`]; nothing here creates,
//! writes, renames or deletes a legacy file.

use std::path::Path;

use sucrose_core::settings::{
    Backdrop, DisplayScreenType, EngineType, FullScreenScope, InputModuleType, InputType, LegacyBackdropType,
    LegacyThemeType, PerformanceType, ScreenModeType, SortKind, SortMode, Stretch, ThemeMode, TransitionCycleType,
    UpdateAutoType, UpdateChannelType, UpdateExtensionType, UpdateModuleType, UpdateServerType,
};

use crate::{
    legacy::{BackgroundogSettings, EngineSettings, GeneralSettings, LegacyFile, LibrarySettings, Properties, read},
    model::{
        Backgroundog, Cycling, DEFAULT_DOWNLOAD_TYPE, DEFAULT_LIMIT_TYPE, DEFAULT_PING_TYPE, DEFAULT_UPLOAD_TYPE,
        DEFAULT_USER_AGENT, Engine, Hook, MAX_STORAGE_VALUE, MIN_DATE_TIME, Settings, Update,
    },
};

/// The legacy setting files [`migrate`] reads, and the only ones it ever opens.
///
/// Left out on purpose (D-05), so none of it is ever read, let alone written to `Sucrose.json`:
/// - `Donate.json`: the donation prompt is gone (D-023).
/// - `Objectionable.json`: a plaintext GitHub token; GitHub as a store server is removed (D-020), so the file is
///   never even opened.
/// - `Aurora.json` `AppProcessName`: runtime state of the launched application (ARCH-04).
/// - `Kernel.json` `Information` and the `System.json` interface lists: write-only system information.
/// - `Portal.json` `StoreServerType`: the store has one server (D-020).
/// - `Backgroundog.json` `TransmissionPort`, `PipeRequired`, `SignalRequired`, `TransmissionRequired`,
///   `AudioRequired`, `PausePerformance`, `ClosePerformance`: runtime state the engines and Backgroundog write
///   (ARCH-04); a stale `ClosePerformance` must never disable the wallpaper. `CommunicationType` goes with them,
///   the new IPC has no choice of channel.
/// - `Backgroundog.json` `GpuUsage`, `CpuUsage`, `CpuPerformance`, `GpuPerformance`: stale names of an older
///   release; the current `GraphicUsage`, `ProcessorUsage`, `ProcessorPerformance` and `GraphicPerformance` keep
///   their own values.
/// - `Cycling.json` `PassingTime`: runtime state of the cycling timer.
pub const MIGRATED_FILES: [LegacyFile; 9] = [
    LegacyFile::General,
    LegacyFile::Portal,
    LegacyFile::Engine,
    LegacyFile::Backgroundog,
    LegacyFile::Cycling,
    LegacyFile::Hook,
    LegacyFile::Library,
    LegacyFile::Update,
    LegacyFile::Warehouse,
];

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

/// A number that a legacy range keeps in `min..=max`; `u32` for the ranges up to 99999999.
fn wide(properties: &Properties, key: &str, default: u32, min: u32, max: u32) -> u32 {
    let value = properties.get_clamped(key, i64::from(default), i64::from(min), i64::from(max));
    u32::try_from(value).unwrap_or(default)
}

/// A number that a legacy range keeps in `min..=max`; `u16` for the port and ping ranges.
fn medium(properties: &Properties, key: &str, default: u16, min: u16, max: u16) -> u16 {
    let value = properties.get_clamped(key, i64::from(default), i64::from(min), i64::from(max));
    u16::try_from(value).unwrap_or(default)
}

/// A string that must not be empty (the value set of the external storage and ping types is unknown).
fn non_empty(properties: &Properties, key: &str, default: &str) -> String {
    match properties.get_str(key) {
        Some(text) if !text.is_empty() => text.to_owned(),
        _ => default.to_owned(),
    }
}

/// The engine of one wallpaper type: a name outside the allowed set of that type is the default of the key.
fn engine_choice(properties: &Properties, key: &str, default: EngineType, allowed: &[EngineType]) -> EngineType {
    properties.get_enum(
        key,
        default,
        |name| EngineType::from_legacy_name(name).filter(|engine| allowed.contains(engine)),
        |index| EngineType::from_legacy_index(index).filter(|engine| allowed.contains(engine)),
    )
}

/// A performance rule of `Backgroundog.json`.
fn rule(properties: &Properties, key: &str, default: PerformanceType) -> PerformanceType {
    properties.get_enum(
        key,
        default,
        PerformanceType::from_legacy_name,
        PerformanceType::from_legacy_index,
    )
}

/// The raw text of a legacy date-time value; the default is the legacy "never set" value.
fn raw_time(properties: &Properties, key: &str) -> String {
    properties.get_str(key).unwrap_or(MIN_DATE_TIME).to_owned()
}

/// Builds the settings from the legacy files in `legacy_setting_dir`. Anything missing or invalid is the legacy
/// default; `legacy_root` is the C# data folder, whose `Library` folder is the default library location.
///
/// Each file of [`MIGRATED_FILES`] is read once and nothing else is opened; every key of those files is a typed field
/// except the documented exclusions, and `ThemeType` and `BackdropType` are mapped to the theme mode and backdrop by
/// [`map_theme`] instead of being stored (D-21).
pub fn migrate(legacy_setting_dir: &Path, legacy_root: &Path) -> Settings {
    let loaded: Vec<(LegacyFile, Properties)> = MIGRATED_FILES
        .into_iter()
        .map(|file| (file, read(legacy_setting_dir, file)))
        .collect();
    let library = loaded_file(&loaded, LegacyFile::Library);
    let general = loaded_file(&loaded, LegacyFile::General);
    let portal = loaded_file(&loaded, LegacyFile::Portal);
    let warehouse = loaded_file(&loaded, LegacyFile::Warehouse);
    let engine_file = loaded_file(&loaded, LegacyFile::Engine);
    let backgroundog_file = loaded_file(&loaded, LegacyFile::Backgroundog);
    let engine = EngineSettings::from_properties(engine_file);
    let backgroundog = BackgroundogSettings::from_properties(backgroundog_file);
    let mut settings = Settings::default();

    // The default location stays empty (it means the legacy default), so the file does not pin a path that was never
    // chosen.
    let location = LibrarySettings::from_properties(library, legacy_root);
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
    settings.general.run_startup = small(general, "RunStartup", 0, 0, 10);
    settings.general.user_agent = general.get_str("UserAgent").unwrap_or(DEFAULT_USER_AGENT).to_owned();
    settings.general.telemetry_data = general.get_bool("TelemetryData", true);
    settings.general.exception_data = general.get_bool("ExceptionData", true);
    settings.general.app_visible = GeneralSettings::from_properties(general).app_visible;
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
    target.background_opacity = small(portal, "BackgroundOpacity", 100, 0, 100);
    target.library_pagination = small(portal, "LibraryPagination", 30, 1, 100);
    target.store_pagination = small(portal, "StorePagination", 30, 1, 100);
    target.adaptive_layout = small(portal, "AdaptiveLayout", 0, 0, 100);
    target.adaptive_margin = small(portal, "AdaptiveMargin", 5, 5, 25);
    target.store_duration = small(portal, "StoreDuration", 3, 1, 24);
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
    settings.warehouse.cef_sharp_time = raw_time(warehouse, "CefSharpTime");
    settings.warehouse.web_view_time = raw_time(warehouse, "WebViewTime");
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
    migrate_engine(engine_file, &mut settings.engine);
    migrate_backgroundog(backgroundog_file, &mut settings.backgroundog);
    migrate_cycling(loaded_file(&loaded, LegacyFile::Cycling), &mut settings.cycling);
    migrate_hook(loaded_file(&loaded, LegacyFile::Hook), &mut settings.hook);
    migrate_update(loaded_file(&loaded, LegacyFile::Update), &mut settings.update);

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

/// The properties of one of the [`MIGRATED_FILES`] after they were all read.
fn loaded_file(loaded: &[(LegacyFile, Properties)], file: LegacyFile) -> &Properties {
    &loaded
        .iter()
        .find(|(candidate, _)| *candidate == file)
        .expect("MIGRATED_FILES lists every file this module reads")
        .1
}

/// The keys of `Engine.json` beyond the playback ones [`EngineSettings`] already read.
fn migrate_engine(properties: &Properties, target: &mut Engine) {
    target.cef_arguments = properties.get_string_map("CefArguments");
    target.web_arguments = properties.get_string_list("WebArguments");
    target.volume_silent_sensitivity = small(properties, "VolumeSilentSensitivity", 5, 1, 10);
    target.developer_port = medium(properties, "DeveloperPort", 0, 0, 65535);
    // The Skylark enums are known by name only; their ordinals are unclear.
    target.duplicate_screen_type = properties.get_enum(
        "DuplicateScreenType",
        ScreenModeType::default(),
        ScreenModeType::from_legacy_name,
        |_| None,
    );
    target.expand_screen_type = properties.get_enum(
        "ExpandScreenType",
        ScreenModeType::default(),
        ScreenModeType::from_legacy_name,
        |_| None,
    );
    target.display_screen_type = properties.get_enum(
        "DisplayScreenType",
        DisplayScreenType::default(),
        DisplayScreenType::from_legacy_name,
        |_| None,
    );
    target.screen_device = properties.get_str("ScreenDevice").unwrap_or_default().to_owned();
    target.input_type = properties.get_enum("InputType", InputType::default(), InputType::from_legacy_name, |_| None);
    target.input_module_type = properties.get_enum(
        "InputModuleType",
        InputModuleType::default(),
        InputModuleType::from_legacy_name,
        InputModuleType::from_legacy_index,
    );
    target.gif = engine_choice(properties, "Gif", EngineType::MpvPlayerLive, EngineType::GIF);
    target.url = engine_choice(properties, "Url", EngineType::CefSharpLive, EngineType::URL);
    target.web = engine_choice(properties, "Web", EngineType::CefSharpLive, EngineType::WEB);
    target.video = engine_choice(properties, "Video", EngineType::MpvPlayerLive, EngineType::VIDEO);
    target.youtube = engine_choice(properties, "YouTube", EngineType::CefSharpLive, EngineType::YOUTUBE);
    target.application = engine_choice(
        properties,
        "Application",
        EngineType::AuroraLive,
        EngineType::APPLICATION,
    );
    target.background_image_path = properties.get_str("BackgroundImagePath").unwrap_or_default().to_owned();
    target.background_image = properties.get_bool("BackgroundImage", false);
    target.disable_slideshow = properties.get_bool("DisableSlideshow", false);
    target.disable_slideshow_live = properties.get_bool("DisableSlideshowLive", false);
    target.wallpaper_shuffle = properties.get_bool("WallpaperShuffle", true);
    target.volume_desktop = properties.get_bool("VolumeDesktop", false);
    target.volume_silent = properties.get_bool("VolumeSilent", false);
    target.input_desktop = properties.get_bool("InputDesktop", false);
    target.developer_mode = properties.get_bool("DeveloperMode", false);
    target.crash_explorer = properties.get_bool("CrashExplorer", false);
    target.library_start = properties.get_bool("LibraryStart", true);
    target.store_start = properties.get_bool("StoreStart", true);
    target.stay_awake = properties.get_bool("StayAwake", false);
}

/// The keys of `Backgroundog.json` beyond the two [`BackgroundogSettings`] already read. The runtime keys and the
/// stale names listed at [`MIGRATED_FILES`] are not named here, so they are never read.
fn migrate_backgroundog(properties: &Properties, target: &mut Backgroundog) {
    // The scope is new: a legacy file never has it, so it stays the default.
    target.full_screen_scope = FullScreenScope::default();
    target.download_value = wide(properties, "DownloadValue", 10, 0, MAX_STORAGE_VALUE);
    target.download_type = non_empty(properties, "DownloadType", DEFAULT_DOWNLOAD_TYPE);
    target.upload_value = wide(properties, "UploadValue", 800, 0, MAX_STORAGE_VALUE);
    target.upload_type = non_empty(properties, "UploadType", DEFAULT_UPLOAD_TYPE);
    target.processor_usage = small(properties, "ProcessorUsage", 70, 0, 100);
    target.graphic_usage = small(properties, "GraphicUsage", 70, 0, 100);
    target.battery_usage = small(properties, "BatteryUsage", 50, 0, 100);
    target.memory_usage = small(properties, "MemoryUsage", 80, 0, 100);
    target.ping_value = medium(properties, "PingValue", 100, 0, 1000);
    target.ping_type = non_empty(properties, "PingType", DEFAULT_PING_TYPE);
    target.network_adapter = properties.get_str("NetworkAdapter").unwrap_or_default().to_owned();
    target.graphic_adapter = properties.get_str("GraphicAdapter").unwrap_or_default().to_owned();
    target.performance_counter = properties.get_bool("PerformanceCounter", true);
    target.battery_saver_performance = rule(properties, "BatterySaverPerformance", PerformanceType::Resume);
    target.screen_saver_performance = rule(properties, "ScreenSaverPerformance", PerformanceType::Pause);
    target.processor_performance = rule(properties, "ProcessorPerformance", PerformanceType::Resume);
    target.virtual_performance = rule(properties, "VirtualPerformance", PerformanceType::Resume);
    target.network_performance = rule(properties, "NetworkPerformance", PerformanceType::Resume);
    target.graphic_performance = rule(properties, "GraphicPerformance", PerformanceType::Resume);
    target.battery_performance = rule(properties, "BatteryPerformance", PerformanceType::Resume);
    target.remote_performance = rule(properties, "RemotePerformance", PerformanceType::Resume);
    target.memory_performance = rule(properties, "MemoryPerformance", PerformanceType::Resume);
    target.focus_performance = rule(properties, "FocusPerformance", PerformanceType::Resume);
    target.session_performance = rule(properties, "SessionPerformance", PerformanceType::Close);
    target.console_performance = rule(properties, "ConsolePerformance", PerformanceType::Close);
    target.sleep_performance = rule(properties, "SleepPerformance", PerformanceType::Close);
    target.lock_performance = rule(properties, "LockPerformance", PerformanceType::Close);
}

/// `Cycling.json` without `PassingTime`, the runtime state of the cycling timer.
fn migrate_cycling(properties: &Properties, target: &mut Cycling) {
    target.transition_time = wide(properties, "TransitionTime", 1800, 10, 99999);
    target.exclusion = properties.get_string_list("Exclusion");
    target.active = properties.get_bool("Active", false);
    target.transition_type = properties.get_enum(
        "TransitionType",
        TransitionCycleType::default(),
        TransitionCycleType::from_legacy_name,
        TransitionCycleType::from_legacy_index,
    );
}

fn migrate_hook(properties: &Properties, target: &mut Hook) {
    target.discord_refresh_delay = wide(properties, "DiscordRefreshDelay", 60, 60, 3600);
    target.discord_refresh_connect = properties.get_bool("DiscordRefreshConnect", true);
    target.discord_connect = properties.get_bool("DiscordConnect", true);
}

/// `Update.json`; `Time` stays the raw legacy date-time text.
fn migrate_update(properties: &Properties, target: &mut Update) {
    target.limit_value = wide(properties, "LimitValue", 500, 0, MAX_STORAGE_VALUE);
    target.limit_type = non_empty(properties, "LimitType", DEFAULT_LIMIT_TYPE);
    target.time = raw_time(properties, "Time");
    target.state = properties.get_bool("State", false);
    target.auto = properties.get_bool("Auto", true);
    target.module_type = properties.get_enum(
        "ModuleType",
        UpdateModuleType::default(),
        UpdateModuleType::from_legacy_name,
        UpdateModuleType::from_legacy_index,
    );
    target.server_type = properties.get_enum(
        "ServerType",
        UpdateServerType::default(),
        UpdateServerType::from_legacy_name,
        UpdateServerType::from_legacy_index,
    );
    target.auto_type = properties.get_enum(
        "AutoType",
        UpdateAutoType::default(),
        UpdateAutoType::from_legacy_name,
        UpdateAutoType::from_legacy_index,
    );
    target.extension_type = properties.get_enum(
        "ExtensionType",
        UpdateExtensionType::default(),
        UpdateExtensionType::from_legacy_name,
        UpdateExtensionType::from_legacy_index,
    );
    target.channel_type = properties.get_enum(
        "ChannelType",
        UpdateChannelType::default(),
        UpdateChannelType::from_legacy_name,
        UpdateChannelType::from_legacy_index,
    );
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
