//! The versioned settings model of `Setting/Sucrose.json` (D-01, D-02).
//!
//! One section per legacy setting file, keys named like the legacy keys, values with the legacy defaults, so support
//! and the settings pages map them one to one. Reading is lenient: a missing or invalid value is the legacy default
//! (logged), and keys this build does not know are kept so a write never drops them (T-03-03).

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::{Map, Value};
use sucrose_core::settings::{
    Backdrop, DisplayScreenType, EngineType, FullScreenScope, GpuPreference, InputModuleType, InputType, PauseKind,
    PerformanceType, ScreenArea, ScreenModeType, SortKind, SortMode, Stretch, ThemeMode, TransitionCycleType,
    UpdateAutoType, UpdateChannelType, UpdateExtensionType, UpdateModuleType, UpdateServerType,
};

/// The `General.UserAgent` default of the C# app.
pub const DEFAULT_USER_AGENT: &str = "Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit";

/// What the C# app stores for a `DateTime` that was never set (`DateTime.MinValue`); kept as the raw string.
pub const MIN_DATE_TIME: &str = "0001-01-01T00:00:00";

/// The legacy `StorageType` name of the default `Backgroundog.DownloadType`; the value set of that external enum is
/// unknown, so a stored value only has to be a non-empty name.
pub const DEFAULT_DOWNLOAD_TYPE: &str = "Megabyte";

/// The legacy `StorageType` name of the default `Backgroundog.UploadType`.
pub const DEFAULT_UPLOAD_TYPE: &str = "Kilobyte";

/// The legacy `StorageType` name of the default `Update.LimitType`.
pub const DEFAULT_LIMIT_TYPE: &str = "Megabyte";

/// The default `Backgroundog.PingType` host key.
pub const DEFAULT_PING_TYPE: &str = "Bing";

/// The largest legacy value of a storage limit (`DownloadValue`, `UploadValue`, `LimitValue`).
pub const MAX_STORAGE_VALUE: u32 = 99_999_999;

/// The format version this build writes. A file with a higher version is read but never overwritten.
pub const SETTINGS_VERSION: u32 = 1;

/// Every setting of this build, as stored in `Setting/Sucrose.json`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Settings {
    /// Format version (`Version`).
    #[serde(rename = "Version")]
    pub version: u32,
    /// Where themes live and which one is selected (legacy `Library.json`).
    #[serde(rename = "Library")]
    pub library: Library,
    /// Legacy `General.json`.
    #[serde(rename = "General")]
    pub general: General,
    /// The portal look, library view and store view (legacy `Portal.json`, plus the D-21 theme mode and backdrop).
    #[serde(rename = "Portal")]
    pub portal: Portal,
    /// Playback and placement (legacy `Engine.json`).
    #[serde(rename = "Engine")]
    pub engine: Engine,
    /// Performance rules (legacy `Backgroundog.json`).
    #[serde(rename = "Backgroundog")]
    pub backgroundog: Backgroundog,
    /// Wallpaper cycling (legacy `Cycling.json`).
    #[serde(rename = "Cycling")]
    pub cycling: Cycling,
    /// The Discord hook (legacy `Hook.json`).
    #[serde(rename = "Hook")]
    pub hook: Hook,
    /// Update rules (legacy `Update.json`).
    #[serde(rename = "Update")]
    pub update: Update,
    /// Showcase copy state and hints (legacy `Warehouse.json`).
    #[serde(rename = "Warehouse")]
    pub warehouse: Warehouse,
    /// Top-level sections this build does not know, kept as they are.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Library` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Library {
    /// Folder that contains the theme folders; empty means the legacy default folder (`Library.json` `Location`).
    #[serde(rename = "Location")]
    pub location: String,
    /// Folder name of the selected theme, empty when none is selected (`Selected`).
    #[serde(rename = "Selected")]
    pub selected: String,
    /// Whether themes that fail the compatibility check are deleted from the library (`DeleteCorrupt`).
    #[serde(rename = "DeleteCorrupt")]
    pub delete_corrupt: bool,
    /// Whether deleting a theme asks first (`DeleteConfirm`).
    #[serde(rename = "DeleteConfirm")]
    pub delete_confirm: bool,
    /// Whether changing the library location moves the themes along (`Move`).
    #[serde(rename = "Move")]
    pub move_library: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `General` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct General {
    /// UI language as a two-letter code; `None` follows the OS language, applied in Phase 5 (`Culture`).
    #[serde(rename = "Culture", skip_serializing_if = "Option::is_none")]
    pub culture: Option<String>,
    /// How Sucrose starts with the OS, 0..=10: 0 off, 1 startup entry, 2 elevated entry, 3 scheduler task
    /// (`RunStartup`).
    #[serde(rename = "RunStartup")]
    pub run_startup: u8,
    /// The user agent web wallpapers and the store use (`UserAgent`).
    #[serde(rename = "UserAgent")]
    pub user_agent: String,
    /// Whether telemetry is sent (`TelemetryData`).
    #[serde(rename = "TelemetryData")]
    pub telemetry_data: bool,
    /// Whether exception reports are sent (`ExceptionData`).
    #[serde(rename = "ExceptionData")]
    pub exception_data: bool,
    /// Whether the app window is shown; the tray of the service reads it (`AppVisible`).
    #[serde(rename = "AppVisible")]
    pub app_visible: bool,
    /// Whether closing the portal exits it instead of hiding it (`AppExit`).
    #[serde(rename = "AppExit")]
    pub app_exit: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Portal` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Portal {
    /// Light, dark or follow the system (new, D-21; migrated from `General.json:ThemeType`).
    #[serde(rename = "ThemeMode")]
    pub theme_mode: ThemeMode,
    /// The material behind the window (new, D-21; migrated from `BackdropType`).
    #[serde(rename = "Backdrop")]
    pub backdrop: Backdrop,
    /// Opacity of the window background, 0..=100 (`BackgroundOpacity`).
    #[serde(rename = "BackgroundOpacity")]
    pub background_opacity: u8,
    /// Themes per library page, 1..=100 (`LibraryPagination`).
    #[serde(rename = "LibraryPagination")]
    pub library_pagination: u8,
    /// Items per store page, 1..=100 (`StorePagination`).
    #[serde(rename = "StorePagination")]
    pub store_pagination: u8,
    /// How strongly the library grid adapts to the window width, 0..=100 (`AdaptiveLayout`).
    #[serde(rename = "AdaptiveLayout")]
    pub adaptive_layout: u8,
    /// Margin of the adaptive grid, 5..=25 (`AdaptiveMargin`).
    #[serde(rename = "AdaptiveMargin")]
    pub adaptive_margin: u8,
    /// Hours the store listing stays cached, 1..=24 (`StoreDuration`).
    #[serde(rename = "StoreDuration")]
    pub store_duration: u8,
    /// Path of the window background image, empty for none (`BackgroundImage`).
    #[serde(rename = "BackgroundImage")]
    pub background_image: String,
    /// Whether library cards hide their details (`LibraryPreviewHide`).
    #[serde(rename = "LibraryPreviewHide")]
    pub library_preview_hide: bool,
    /// Whether store cards hide their details (`StorePreviewHide`).
    #[serde(rename = "StorePreviewHide")]
    pub store_preview_hide: bool,
    /// Whether library cards preview the theme (`LibraryPreview`).
    #[serde(rename = "LibraryPreview")]
    pub library_preview: bool,
    /// Whether store cards preview the theme (`StorePreview`).
    #[serde(rename = "StorePreview")]
    pub store_preview: bool,
    /// Whether adult items are shown in the store (`StoreAdult`).
    #[serde(rename = "StoreAdult")]
    pub store_adult: bool,
    /// How the background image is fitted (`BackgroundStretch`).
    #[serde(rename = "BackgroundStretch")]
    pub background_stretch: Stretch,
    /// Direction of the library sort (`LibrarySortKind`).
    #[serde(rename = "LibrarySortKind")]
    pub library_sort_kind: SortKind,
    /// What the library is sorted by (`LibrarySortMode`).
    #[serde(rename = "LibrarySortMode")]
    pub library_sort_mode: SortMode,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Warehouse` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Warehouse {
    /// Folder names of the showcase themes already copied into the library (`Showcase`).
    #[serde(rename = "Showcase")]
    pub showcase: Vec<String>,
    /// When the CefSharp runtime was last checked, as the raw legacy date-time text (`CefSharpTime`).
    #[serde(rename = "CefSharpTime")]
    pub cef_sharp_time: String,
    /// When the WebView runtime was last checked, as the raw legacy date-time text (`WebViewTime`).
    #[serde(rename = "WebViewTime")]
    pub web_view_time: String,
    /// Whether to continue without the CefSharp runtime (`CefsharpContinue`, lowercase "s" like the legacy key).
    #[serde(rename = "CefsharpContinue")]
    pub cefsharp_continue: bool,
    /// Whether to continue without the WebView runtime (`WebViewContinue`).
    #[serde(rename = "WebViewContinue")]
    pub web_view_continue: bool,
    /// Whether the hint about the notification area icon is still shown (`HintTrayIcon`).
    #[serde(rename = "HintTrayIcon")]
    pub hint_tray_icon: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Engine` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Engine {
    /// Wallpaper volume, 0..=100 (`WallpaperVolume`).
    #[serde(rename = "WallpaperVolume")]
    pub volume: u8,
    /// Whether the wallpaper loops (`WallpaperLoop`).
    #[serde(rename = "WallpaperLoop")]
    pub looping: bool,
    /// How the wallpaper is fitted (`StretchType`).
    #[serde(rename = "StretchType")]
    pub stretch: Stretch,
    /// Whether the player may decode on the GPU (`HardwareAcceleration`).
    #[serde(rename = "HardwareAcceleration")]
    pub hardware_acceleration: bool,
    /// Which part of the monitor is covered (`ScreenType`).
    #[serde(rename = "ScreenType")]
    pub screen: ScreenArea,
    /// Which GPU Windows is asked to give the renderers (`GraphicPreference`).
    #[serde(rename = "GraphicPreference")]
    pub gpu_preference: GpuPreference,
    /// Command line switches handed to the CefSharp engine, by name (`CefArguments`).
    #[serde(rename = "CefArguments")]
    pub cef_arguments: BTreeMap<String, String>,
    /// Command line switches handed to the WebView engine, in order (`WebArguments`).
    #[serde(rename = "WebArguments")]
    pub web_arguments: Vec<String>,
    /// How sensitive the silence detection is, 1..=10 (`VolumeSilentSensitivity`).
    #[serde(rename = "VolumeSilentSensitivity")]
    pub volume_silent_sensitivity: u8,
    /// The port of the developer tools, 0..=65535, 0 for none (`DeveloperPort`).
    #[serde(rename = "DeveloperPort")]
    pub developer_port: u16,
    /// How a duplicated screen layout is handled (`DuplicateScreenType`).
    #[serde(rename = "DuplicateScreenType")]
    pub duplicate_screen_type: ScreenModeType,
    /// How an expanded screen layout is handled (`ExpandScreenType`).
    #[serde(rename = "ExpandScreenType")]
    pub expand_screen_type: ScreenModeType,
    /// How wallpapers are spread over several monitors (`DisplayScreenType`).
    #[serde(rename = "DisplayScreenType")]
    pub display_screen_type: DisplayScreenType,
    /// The monitor a single wallpaper plays on, empty for the primary one (`ScreenDevice`).
    #[serde(rename = "ScreenDevice")]
    pub screen_device: String,
    /// Which input is forwarded to web wallpapers (`InputType`).
    #[serde(rename = "InputType")]
    pub input_type: InputType,
    /// How input is read (`InputModuleType`).
    #[serde(rename = "InputModuleType")]
    pub input_module_type: InputModuleType,
    /// The engine of gif wallpapers (`Gif`).
    #[serde(rename = "Gif")]
    pub gif: EngineType,
    /// The engine of url wallpapers (`Url`).
    #[serde(rename = "Url")]
    pub url: EngineType,
    /// The engine of web wallpapers (`Web`).
    #[serde(rename = "Web")]
    pub web: EngineType,
    /// The engine of video wallpapers (`Video`).
    #[serde(rename = "Video")]
    pub video: EngineType,
    /// The engine of YouTube wallpapers (`YouTube`).
    #[serde(rename = "YouTube")]
    pub youtube: EngineType,
    /// The engine of application wallpapers (`Application`).
    #[serde(rename = "Application")]
    pub application: EngineType,
    /// Path of the wallpaper behind a transparent theme, empty for none (`BackgroundImagePath`).
    #[serde(rename = "BackgroundImagePath")]
    pub background_image_path: String,
    /// Whether that background image is used (`BackgroundImage`).
    #[serde(rename = "BackgroundImage")]
    pub background_image: bool,
    /// Whether the slideshow is disabled (`DisableSlideshow`).
    #[serde(rename = "DisableSlideshow")]
    pub disable_slideshow: bool,
    /// Whether the live slideshow is disabled (`DisableSlideshowLive`).
    #[serde(rename = "DisableSlideshowLive")]
    pub disable_slideshow_live: bool,
    /// Whether the wallpaper is picked at random when none is selected (`WallpaperShuffle`).
    #[serde(rename = "WallpaperShuffle")]
    pub wallpaper_shuffle: bool,
    /// Whether the volume follows the desktop (`VolumeDesktop`).
    #[serde(rename = "VolumeDesktop")]
    pub volume_desktop: bool,
    /// Whether silence detection is on (`VolumeSilent`).
    #[serde(rename = "VolumeSilent")]
    pub volume_silent: bool,
    /// Whether input is only forwarded while the desktop is in front (`InputDesktop`).
    #[serde(rename = "InputDesktop")]
    pub input_desktop: bool,
    /// Whether developer mode is on (`DeveloperMode`).
    #[serde(rename = "DeveloperMode")]
    pub developer_mode: bool,
    /// Whether Explorer is restarted when Sucrose crashes (`CrashExplorer`).
    #[serde(rename = "CrashExplorer")]
    pub crash_explorer: bool,
    /// Whether the library is started with the engine (`LibraryStart`).
    #[serde(rename = "LibraryStart")]
    pub library_start: bool,
    /// Whether the store is started with the engine (`StoreStart`).
    #[serde(rename = "StoreStart")]
    pub store_start: bool,
    /// Whether the system is kept awake while a wallpaper plays (`StayAwake`).
    #[serde(rename = "StayAwake")]
    pub stay_awake: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Backgroundog` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Backgroundog {
    /// What happens while a fullscreen app is in front (`FullScreenPerformance`).
    #[serde(rename = "FullScreenPerformance")]
    pub fullscreen: PerformanceType,
    /// How a pause is carried out (`PausePerformanceType`).
    #[serde(rename = "PausePerformanceType")]
    pub pause: PauseKind,
    /// Which monitors a fullscreen app affects (new, `FullScreenScope`).
    #[serde(rename = "FullScreenScope")]
    pub full_screen_scope: FullScreenScope,
    /// Download speed that counts as busy network, 0..=99999999 in `download_type` units (`DownloadValue`).
    #[serde(rename = "DownloadValue")]
    pub download_value: u32,
    /// The unit of `DownloadValue` as the legacy storage type name, never empty (`DownloadType`).
    #[serde(rename = "DownloadType")]
    pub download_type: String,
    /// Upload speed that counts as busy network, 0..=99999999 in `upload_type` units (`UploadValue`).
    #[serde(rename = "UploadValue")]
    pub upload_value: u32,
    /// The unit of `UploadValue` as the legacy storage type name, never empty (`UploadType`).
    #[serde(rename = "UploadType")]
    pub upload_type: String,
    /// Processor load percentage that triggers the processor rule, 0..=100 (`ProcessorUsage`).
    #[serde(rename = "ProcessorUsage")]
    pub processor_usage: u8,
    /// Graphics load percentage that triggers the graphics rule, 0..=100 (`GraphicUsage`).
    #[serde(rename = "GraphicUsage")]
    pub graphic_usage: u8,
    /// Battery percentage that triggers the battery rule, 0..=100 (`BatteryUsage`).
    #[serde(rename = "BatteryUsage")]
    pub battery_usage: u8,
    /// Memory load percentage that triggers the memory rule, 0..=100 (`MemoryUsage`).
    #[serde(rename = "MemoryUsage")]
    pub memory_usage: u8,
    /// Ping in milliseconds that triggers the network rule, 0..=1000 (`PingValue`).
    #[serde(rename = "PingValue")]
    pub ping_value: u16,
    /// The host the ping goes to, as a legacy host key such as `Bing` (`PingType`).
    #[serde(rename = "PingType")]
    pub ping_type: String,
    /// The network adapter that is measured, empty for the default (`NetworkAdapter`).
    #[serde(rename = "NetworkAdapter")]
    pub network_adapter: String,
    /// The graphics adapter that is measured, empty for the default (`GraphicAdapter`).
    #[serde(rename = "GraphicAdapter")]
    pub graphic_adapter: String,
    /// Whether the performance counters run at all (`PerformanceCounter`).
    #[serde(rename = "PerformanceCounter")]
    pub performance_counter: bool,
    /// What happens on battery saver (`BatterySaverPerformance`).
    #[serde(rename = "BatterySaverPerformance")]
    pub battery_saver_performance: PerformanceType,
    /// What happens while the screen saver runs (`ScreenSaverPerformance`).
    #[serde(rename = "ScreenSaverPerformance")]
    pub screen_saver_performance: PerformanceType,
    /// What happens under high processor load (`ProcessorPerformance`).
    #[serde(rename = "ProcessorPerformance")]
    pub processor_performance: PerformanceType,
    /// What happens inside a virtual machine (`VirtualPerformance`).
    #[serde(rename = "VirtualPerformance")]
    pub virtual_performance: PerformanceType,
    /// What happens on a slow network (`NetworkPerformance`).
    #[serde(rename = "NetworkPerformance")]
    pub network_performance: PerformanceType,
    /// What happens under high graphics load (`GraphicPerformance`).
    #[serde(rename = "GraphicPerformance")]
    pub graphic_performance: PerformanceType,
    /// What happens on a low battery (`BatteryPerformance`).
    #[serde(rename = "BatteryPerformance")]
    pub battery_performance: PerformanceType,
    /// What happens in a remote session (`RemotePerformance`).
    #[serde(rename = "RemotePerformance")]
    pub remote_performance: PerformanceType,
    /// What happens under high memory load (`MemoryPerformance`).
    #[serde(rename = "MemoryPerformance")]
    pub memory_performance: PerformanceType,
    /// What happens while another window has focus (`FocusPerformance`).
    #[serde(rename = "FocusPerformance")]
    pub focus_performance: PerformanceType,
    /// What happens when the session is switched away (`SessionPerformance`).
    #[serde(rename = "SessionPerformance")]
    pub session_performance: PerformanceType,
    /// What happens while a console app is in front (`ConsolePerformance`).
    #[serde(rename = "ConsolePerformance")]
    pub console_performance: PerformanceType,
    /// What happens when the PC sleeps (`SleepPerformance`).
    #[serde(rename = "SleepPerformance")]
    pub sleep_performance: PerformanceType,
    /// What happens when the PC is locked (`LockPerformance`).
    #[serde(rename = "LockPerformance")]
    pub lock_performance: PerformanceType,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Cycling` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cycling {
    /// Seconds between two wallpapers, 10..=99999 (`TransitionTime`).
    #[serde(rename = "TransitionTime")]
    pub transition_time: u32,
    /// Folder names of themes that cycling skips (`Exclusion`).
    #[serde(rename = "Exclusion")]
    pub exclusion: Vec<String>,
    /// Whether cycling is on (`Active`).
    #[serde(rename = "Active")]
    pub active: bool,
    /// The order in which cycling picks the next theme (`TransitionType`).
    #[serde(rename = "TransitionType")]
    pub transition_type: TransitionCycleType,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Hook` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hook {
    /// Seconds between two Discord reconnect attempts, 60..=3600 (`DiscordRefreshDelay`).
    #[serde(rename = "DiscordRefreshDelay")]
    pub discord_refresh_delay: u32,
    /// Whether the Discord connection is refreshed (`DiscordRefreshConnect`).
    #[serde(rename = "DiscordRefreshConnect")]
    pub discord_refresh_connect: bool,
    /// Whether Discord is connected at all (`DiscordConnect`).
    #[serde(rename = "DiscordConnect")]
    pub discord_connect: bool,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The `Update` section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Update {
    /// Download limit, 0..=99999999 in `limit_type` units (`LimitValue`).
    #[serde(rename = "LimitValue")]
    pub limit_value: u32,
    /// The unit of `LimitValue` as the legacy storage type name, never empty (`LimitType`).
    #[serde(rename = "LimitType")]
    pub limit_type: String,
    /// When updates were last checked, as the raw legacy date-time text (`Time`).
    #[serde(rename = "Time")]
    pub time: String,
    /// Whether an update is pending (`State`).
    #[serde(rename = "State")]
    pub state: bool,
    /// Whether updates are checked automatically (`Auto`).
    #[serde(rename = "Auto")]
    pub auto: bool,
    /// How an update is downloaded (`ModuleType`).
    #[serde(rename = "ModuleType")]
    pub module_type: UpdateModuleType,
    /// Where updates come from (`ServerType`).
    #[serde(rename = "ServerType")]
    pub server_type: UpdateServerType,
    /// How visible an automatic update is (`AutoType`).
    #[serde(rename = "AutoType")]
    pub auto_type: UpdateAutoType,
    /// The package kind an update downloads (`ExtensionType`).
    #[serde(rename = "ExtensionType")]
    pub extension_type: UpdateExtensionType,
    /// The release channel updates come from (`ChannelType`).
    #[serde(rename = "ChannelType")]
    pub channel_type: UpdateChannelType,
    /// Keys of the section this build does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            location: String::new(),
            selected: String::new(),
            delete_corrupt: false,
            delete_confirm: true,
            move_library: true,
            extra: Map::new(),
        }
    }
}

impl Default for General {
    fn default() -> Self {
        Self {
            culture: None,
            run_startup: 0,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            telemetry_data: true,
            exception_data: true,
            app_visible: true,
            app_exit: true,
            extra: Map::new(),
        }
    }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::default(),
            backdrop: Backdrop::default(),
            background_opacity: 100,
            library_pagination: 30,
            store_pagination: 30,
            adaptive_layout: 0,
            adaptive_margin: 5,
            store_duration: 3,
            background_image: String::new(),
            library_preview_hide: false,
            store_preview_hide: false,
            library_preview: false,
            store_preview: false,
            store_adult: true,
            background_stretch: Stretch::default(),
            library_sort_kind: SortKind::default(),
            library_sort_mode: SortMode::default(),
            extra: Map::new(),
        }
    }
}

impl Default for Warehouse {
    fn default() -> Self {
        Self {
            showcase: Vec::new(),
            cef_sharp_time: MIN_DATE_TIME.to_owned(),
            web_view_time: MIN_DATE_TIME.to_owned(),
            cefsharp_continue: false,
            web_view_continue: false,
            hint_tray_icon: true,
            extra: Map::new(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            volume: 100,
            looping: true,
            stretch: Stretch::default(),
            hardware_acceleration: true,
            screen: ScreenArea::default(),
            gpu_preference: GpuPreference::default(),
            cef_arguments: BTreeMap::new(),
            web_arguments: Vec::new(),
            volume_silent_sensitivity: 5,
            developer_port: 0,
            duplicate_screen_type: ScreenModeType::default(),
            expand_screen_type: ScreenModeType::default(),
            display_screen_type: DisplayScreenType::default(),
            screen_device: String::new(),
            input_type: InputType::default(),
            input_module_type: InputModuleType::default(),
            gif: EngineType::MpvPlayerLive,
            url: EngineType::CefSharpLive,
            web: EngineType::CefSharpLive,
            video: EngineType::MpvPlayerLive,
            youtube: EngineType::CefSharpLive,
            application: EngineType::AuroraLive,
            background_image_path: String::new(),
            background_image: false,
            disable_slideshow: false,
            disable_slideshow_live: false,
            wallpaper_shuffle: true,
            volume_desktop: false,
            volume_silent: false,
            input_desktop: false,
            developer_mode: false,
            crash_explorer: false,
            library_start: true,
            store_start: true,
            stay_awake: false,
            extra: Map::new(),
        }
    }
}

impl Default for Backgroundog {
    fn default() -> Self {
        Self {
            fullscreen: PerformanceType::Resume,
            pause: PauseKind::default(),
            full_screen_scope: FullScreenScope::default(),
            download_value: 10,
            download_type: DEFAULT_DOWNLOAD_TYPE.to_owned(),
            upload_value: 800,
            upload_type: DEFAULT_UPLOAD_TYPE.to_owned(),
            processor_usage: 70,
            graphic_usage: 70,
            battery_usage: 50,
            memory_usage: 80,
            ping_value: 100,
            ping_type: DEFAULT_PING_TYPE.to_owned(),
            network_adapter: String::new(),
            graphic_adapter: String::new(),
            performance_counter: true,
            battery_saver_performance: PerformanceType::Resume,
            screen_saver_performance: PerformanceType::Pause,
            processor_performance: PerformanceType::Resume,
            virtual_performance: PerformanceType::Resume,
            network_performance: PerformanceType::Resume,
            graphic_performance: PerformanceType::Resume,
            battery_performance: PerformanceType::Resume,
            remote_performance: PerformanceType::Resume,
            memory_performance: PerformanceType::Resume,
            focus_performance: PerformanceType::Resume,
            session_performance: PerformanceType::Close,
            console_performance: PerformanceType::Close,
            sleep_performance: PerformanceType::Close,
            lock_performance: PerformanceType::Close,
            extra: Map::new(),
        }
    }
}

impl Default for Cycling {
    fn default() -> Self {
        Self {
            transition_time: 1800,
            exclusion: Vec::new(),
            active: false,
            transition_type: TransitionCycleType::default(),
            extra: Map::new(),
        }
    }
}

impl Default for Hook {
    fn default() -> Self {
        Self {
            discord_refresh_delay: 60,
            discord_refresh_connect: true,
            discord_connect: true,
            extra: Map::new(),
        }
    }
}

impl Default for Update {
    fn default() -> Self {
        Self {
            limit_value: 500,
            limit_type: DEFAULT_LIMIT_TYPE.to_owned(),
            time: MIN_DATE_TIME.to_owned(),
            state: false,
            auto: true,
            module_type: UpdateModuleType::default(),
            server_type: UpdateServerType::default(),
            auto_type: UpdateAutoType::default(),
            extension_type: UpdateExtensionType::default(),
            channel_type: UpdateChannelType::default(),
            extra: Map::new(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            library: Library::default(),
            general: General::default(),
            portal: Portal::default(),
            engine: Engine::default(),
            backgroundog: Backgroundog::default(),
            cycling: Cycling::default(),
            hook: Hook::default(),
            update: Update::default(),
            warehouse: Warehouse::default(),
            extra: Map::new(),
        }
    }
}

impl Settings {
    /// Parses a settings file. Only text that is not a JSON object is an error (a corrupt file); inside it every
    /// missing or invalid value is the legacy default.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
        match serde_json::from_slice::<Value>(bytes)? {
            Value::Object(root) => Ok(Self::from_map(root)),
            _ => Err(<serde_json::Error as serde::de::Error>::custom(
                "the settings root is not a JSON object",
            )),
        }
    }

    /// The file content: pretty-printed so a person can read it (D-02), ending with a newline.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// The folder that contains the theme folders: the configured one, or `<legacy_root>/Library` when it is empty.
    pub fn library_location(&self, legacy_root: &Path) -> PathBuf {
        if self.library.location.is_empty() {
            legacy_root.join("Library")
        } else {
            PathBuf::from(&self.library.location)
        }
    }

    fn from_map(mut root: Map<String, Value>) -> Self {
        let version = match root.remove("Version") {
            None => SETTINGS_VERSION,
            Some(value) => value
                .as_u64()
                .and_then(|number| u32::try_from(number).ok())
                .unwrap_or_else(|| {
                    tracing::warn!(key = "Version", "setting invalid, using the current version");
                    SETTINGS_VERSION
                }),
        };
        let mut library = Fields::section("Library", root.remove("Library"));
        let mut general = Fields::section("General", root.remove("General"));
        let mut portal = Fields::section("Portal", root.remove("Portal"));
        let mut engine = Fields::section("Engine", root.remove("Engine"));
        let mut backgroundog = Fields::section("Backgroundog", root.remove("Backgroundog"));
        let mut cycling = Fields::section("Cycling", root.remove("Cycling"));
        let mut hook = Fields::section("Hook", root.remove("Hook"));
        let mut update = Fields::section("Update", root.remove("Update"));
        let mut warehouse = Fields::section("Warehouse", root.remove("Warehouse"));
        Self {
            version,
            library: Library {
                location: library.string("Location", ""),
                selected: library.string("Selected", ""),
                delete_corrupt: library.boolean("DeleteCorrupt", false),
                delete_confirm: library.boolean("DeleteConfirm", true),
                move_library: library.boolean("Move", true),
                extra: library.rest(),
            },
            general: General {
                culture: general.optional_string("Culture"),
                run_startup: general.clamped("RunStartup", 0, 0, 10),
                user_agent: general.string("UserAgent", DEFAULT_USER_AGENT),
                telemetry_data: general.boolean("TelemetryData", true),
                exception_data: general.boolean("ExceptionData", true),
                app_visible: general.boolean("AppVisible", true),
                app_exit: general.boolean("AppExit", true),
                extra: general.rest(),
            },
            portal: Portal {
                theme_mode: portal.name("ThemeMode", ThemeMode::default(), ThemeMode::from_legacy_name),
                backdrop: portal.name("Backdrop", Backdrop::default(), Backdrop::from_legacy_name),
                background_opacity: portal.clamped("BackgroundOpacity", 100, 0, 100),
                library_pagination: portal.clamped("LibraryPagination", 30, 1, 100),
                store_pagination: portal.clamped("StorePagination", 30, 1, 100),
                adaptive_layout: portal.clamped("AdaptiveLayout", 0, 0, 100),
                adaptive_margin: portal.clamped("AdaptiveMargin", 5, 5, 25),
                store_duration: portal.clamped("StoreDuration", 3, 1, 24),
                background_image: portal.string("BackgroundImage", ""),
                library_preview_hide: portal.boolean("LibraryPreviewHide", false),
                store_preview_hide: portal.boolean("StorePreviewHide", false),
                library_preview: portal.boolean("LibraryPreview", false),
                store_preview: portal.boolean("StorePreview", false),
                store_adult: portal.boolean("StoreAdult", true),
                background_stretch: portal.name("BackgroundStretch", Stretch::default(), Stretch::from_legacy_name),
                library_sort_kind: portal.name("LibrarySortKind", SortKind::default(), SortKind::from_legacy_name),
                library_sort_mode: portal.name("LibrarySortMode", SortMode::default(), SortMode::from_legacy_name),
                extra: portal.rest(),
            },
            engine: Engine {
                volume: engine.volume("WallpaperVolume", 100),
                looping: engine.boolean("WallpaperLoop", true),
                stretch: engine.name("StretchType", Stretch::default(), Stretch::from_legacy_name),
                hardware_acceleration: engine.boolean("HardwareAcceleration", true),
                screen: engine.name("ScreenType", ScreenArea::default(), ScreenArea::from_legacy_name),
                gpu_preference: engine.name(
                    "GraphicPreference",
                    GpuPreference::default(),
                    GpuPreference::from_legacy_name,
                ),
                cef_arguments: engine.string_map("CefArguments"),
                web_arguments: engine.strings("WebArguments"),
                volume_silent_sensitivity: engine.clamped("VolumeSilentSensitivity", 5, 1, 10),
                developer_port: engine.clamped_u16("DeveloperPort", 0, 0, 65535),
                duplicate_screen_type: engine.name(
                    "DuplicateScreenType",
                    ScreenModeType::default(),
                    ScreenModeType::from_legacy_name,
                ),
                expand_screen_type: engine.name(
                    "ExpandScreenType",
                    ScreenModeType::default(),
                    ScreenModeType::from_legacy_name,
                ),
                display_screen_type: engine.name(
                    "DisplayScreenType",
                    DisplayScreenType::default(),
                    DisplayScreenType::from_legacy_name,
                ),
                screen_device: engine.string("ScreenDevice", ""),
                input_type: engine.name("InputType", InputType::default(), InputType::from_legacy_name),
                input_module_type: engine.name(
                    "InputModuleType",
                    InputModuleType::default(),
                    InputModuleType::from_legacy_name,
                ),
                gif: engine.engine("Gif", EngineType::MpvPlayerLive, EngineType::GIF),
                url: engine.engine("Url", EngineType::CefSharpLive, EngineType::URL),
                web: engine.engine("Web", EngineType::CefSharpLive, EngineType::WEB),
                video: engine.engine("Video", EngineType::MpvPlayerLive, EngineType::VIDEO),
                youtube: engine.engine("YouTube", EngineType::CefSharpLive, EngineType::YOUTUBE),
                application: engine.engine("Application", EngineType::AuroraLive, EngineType::APPLICATION),
                background_image_path: engine.string("BackgroundImagePath", ""),
                background_image: engine.boolean("BackgroundImage", false),
                disable_slideshow: engine.boolean("DisableSlideshow", false),
                disable_slideshow_live: engine.boolean("DisableSlideshowLive", false),
                wallpaper_shuffle: engine.boolean("WallpaperShuffle", true),
                volume_desktop: engine.boolean("VolumeDesktop", false),
                volume_silent: engine.boolean("VolumeSilent", false),
                input_desktop: engine.boolean("InputDesktop", false),
                developer_mode: engine.boolean("DeveloperMode", false),
                crash_explorer: engine.boolean("CrashExplorer", false),
                library_start: engine.boolean("LibraryStart", true),
                store_start: engine.boolean("StoreStart", true),
                stay_awake: engine.boolean("StayAwake", false),
                extra: engine.rest(),
            },
            backgroundog: Backgroundog {
                fullscreen: backgroundog.rule("FullScreenPerformance", PerformanceType::Resume),
                pause: backgroundog.name(
                    "PausePerformanceType",
                    PauseKind::default(),
                    PauseKind::from_legacy_name,
                ),
                full_screen_scope: backgroundog.name(
                    "FullScreenScope",
                    FullScreenScope::default(),
                    FullScreenScope::from_legacy_name,
                ),
                download_value: backgroundog.clamped_u32("DownloadValue", 10, 0, MAX_STORAGE_VALUE),
                download_type: backgroundog.non_empty_string("DownloadType", DEFAULT_DOWNLOAD_TYPE),
                upload_value: backgroundog.clamped_u32("UploadValue", 800, 0, MAX_STORAGE_VALUE),
                upload_type: backgroundog.non_empty_string("UploadType", DEFAULT_UPLOAD_TYPE),
                processor_usage: backgroundog.volume("ProcessorUsage", 70),
                graphic_usage: backgroundog.volume("GraphicUsage", 70),
                battery_usage: backgroundog.volume("BatteryUsage", 50),
                memory_usage: backgroundog.volume("MemoryUsage", 80),
                ping_value: backgroundog.clamped_u16("PingValue", 100, 0, 1000),
                ping_type: backgroundog.non_empty_string("PingType", DEFAULT_PING_TYPE),
                network_adapter: backgroundog.string("NetworkAdapter", ""),
                graphic_adapter: backgroundog.string("GraphicAdapter", ""),
                performance_counter: backgroundog.boolean("PerformanceCounter", true),
                battery_saver_performance: backgroundog.rule("BatterySaverPerformance", PerformanceType::Resume),
                screen_saver_performance: backgroundog.rule("ScreenSaverPerformance", PerformanceType::Pause),
                processor_performance: backgroundog.rule("ProcessorPerformance", PerformanceType::Resume),
                virtual_performance: backgroundog.rule("VirtualPerformance", PerformanceType::Resume),
                network_performance: backgroundog.rule("NetworkPerformance", PerformanceType::Resume),
                graphic_performance: backgroundog.rule("GraphicPerformance", PerformanceType::Resume),
                battery_performance: backgroundog.rule("BatteryPerformance", PerformanceType::Resume),
                remote_performance: backgroundog.rule("RemotePerformance", PerformanceType::Resume),
                memory_performance: backgroundog.rule("MemoryPerformance", PerformanceType::Resume),
                focus_performance: backgroundog.rule("FocusPerformance", PerformanceType::Resume),
                session_performance: backgroundog.rule("SessionPerformance", PerformanceType::Close),
                console_performance: backgroundog.rule("ConsolePerformance", PerformanceType::Close),
                sleep_performance: backgroundog.rule("SleepPerformance", PerformanceType::Close),
                lock_performance: backgroundog.rule("LockPerformance", PerformanceType::Close),
                extra: backgroundog.rest(),
            },
            cycling: Cycling {
                transition_time: cycling.clamped_u32("TransitionTime", 1800, 10, 99999),
                exclusion: cycling.strings("Exclusion"),
                active: cycling.boolean("Active", false),
                transition_type: cycling.name(
                    "TransitionType",
                    TransitionCycleType::default(),
                    TransitionCycleType::from_legacy_name,
                ),
                extra: cycling.rest(),
            },
            hook: Hook {
                discord_refresh_delay: hook.clamped_u32("DiscordRefreshDelay", 60, 60, 3600),
                discord_refresh_connect: hook.boolean("DiscordRefreshConnect", true),
                discord_connect: hook.boolean("DiscordConnect", true),
                extra: hook.rest(),
            },
            update: Update {
                limit_value: update.clamped_u32("LimitValue", 500, 0, MAX_STORAGE_VALUE),
                limit_type: update.non_empty_string("LimitType", DEFAULT_LIMIT_TYPE),
                time: update.string("Time", MIN_DATE_TIME),
                state: update.boolean("State", false),
                auto: update.boolean("Auto", true),
                module_type: update.name(
                    "ModuleType",
                    UpdateModuleType::default(),
                    UpdateModuleType::from_legacy_name,
                ),
                server_type: update.name(
                    "ServerType",
                    UpdateServerType::default(),
                    UpdateServerType::from_legacy_name,
                ),
                auto_type: update.name("AutoType", UpdateAutoType::default(), UpdateAutoType::from_legacy_name),
                extension_type: update.name(
                    "ExtensionType",
                    UpdateExtensionType::default(),
                    UpdateExtensionType::from_legacy_name,
                ),
                channel_type: update.name(
                    "ChannelType",
                    UpdateChannelType::default(),
                    UpdateChannelType::from_legacy_name,
                ),
                extra: update.rest(),
            },
            warehouse: Warehouse {
                showcase: warehouse.strings("Showcase"),
                cef_sharp_time: warehouse.string("CefSharpTime", MIN_DATE_TIME),
                web_view_time: warehouse.string("WebViewTime", MIN_DATE_TIME),
                cefsharp_continue: warehouse.boolean("CefsharpContinue", false),
                web_view_continue: warehouse.boolean("WebViewContinue", false),
                hint_tray_icon: warehouse.boolean("HintTrayIcon", true),
                extra: warehouse.rest(),
            },
            extra: root,
        }
    }
}

/// The keys of one section while it is being read; what is not taken stays as the unknown keys.
struct Fields {
    section: &'static str,
    map: Map<String, Value>,
}

impl Fields {
    /// A section that is missing, or is not an object, has no keys (all defaults).
    fn section(section: &'static str, value: Option<Value>) -> Self {
        let map = match value {
            Some(Value::Object(map)) => map,
            Some(_) => {
                tracing::warn!(section, "settings section invalid, using the defaults");
                Map::new()
            }
            None => Map::new(),
        };
        Self { section, map }
    }

    fn invalid(&self, key: &str) {
        tracing::warn!(section = self.section, key, "setting invalid, using the default");
    }

    fn string(&mut self, key: &str, default: &str) -> String {
        match self.map.remove(key) {
            None => default.to_owned(),
            Some(Value::String(text)) => text,
            Some(_) => {
                self.invalid(key);
                default.to_owned()
            }
        }
    }

    fn boolean(&mut self, key: &str, default: bool) -> bool {
        match self.map.remove(key) {
            None => default,
            Some(Value::Bool(value)) => value,
            Some(_) => {
                self.invalid(key);
                default
            }
        }
    }

    /// A string that may be absent: a missing, empty or non-string value is `None`.
    fn optional_string(&mut self, key: &str) -> Option<String> {
        match self.map.remove(key) {
            None => None,
            Some(Value::String(text)) if !text.is_empty() => Some(text),
            Some(Value::String(_)) => None,
            Some(_) => {
                self.invalid(key);
                None
            }
        }
    }

    /// A list of strings: a missing or invalid list is empty, a non-string item is skipped.
    fn strings(&mut self, key: &str) -> Vec<String> {
        match self.map.remove(key) {
            None => Vec::new(),
            Some(Value::Array(items)) => items
                .into_iter()
                .filter_map(|item| match item {
                    Value::String(text) => Some(text),
                    _ => {
                        self.invalid(key);
                        None
                    }
                })
                .collect(),
            Some(_) => {
                self.invalid(key);
                Vec::new()
            }
        }
    }

    /// A percentage: a number outside 0..=100 is clamped like the legacy reader, anything else is the default.
    fn volume(&mut self, key: &str, default: u8) -> u8 {
        self.clamped(key, default, 0, 100)
    }

    /// A small number limited to `min..=max`: a number outside the range is clamped like the legacy reader, anything
    /// else is the default.
    fn clamped(&mut self, key: &str, default: u8, min: u8, max: u8) -> u8 {
        let value = self.clamped_i64(key, i64::from(default), i64::from(min), i64::from(max));
        u8::try_from(value).unwrap_or(default)
    }

    /// Like [`Self::clamped`] for a number up to `u16::MAX`.
    fn clamped_u16(&mut self, key: &str, default: u16, min: u16, max: u16) -> u16 {
        let value = self.clamped_i64(key, i64::from(default), i64::from(min), i64::from(max));
        u16::try_from(value).unwrap_or(default)
    }

    /// Like [`Self::clamped`] for a number up to `u32::MAX`.
    fn clamped_u32(&mut self, key: &str, default: u32, min: u32, max: u32) -> u32 {
        let value = self.clamped_i64(key, i64::from(default), i64::from(min), i64::from(max));
        u32::try_from(value).unwrap_or(default)
    }

    fn clamped_i64(&mut self, key: &str, default: i64, min: i64, max: i64) -> i64 {
        let number = match self.map.remove(key) {
            None => return default,
            Some(Value::Number(number)) => number.as_i64().or_else(|| {
                number
                    .as_f64()
                    .filter(|float| float.is_finite())
                    .map(|float| float.round() as i64)
            }),
            Some(_) => None,
        };
        match number {
            Some(number) => number.clamp(min, max),
            None => {
                self.invalid(key);
                default
            }
        }
    }

    /// A string that must not be empty: a missing, empty or non-string value is the default.
    fn non_empty_string(&mut self, key: &str, default: &str) -> String {
        match self.map.remove(key) {
            None => default.to_owned(),
            Some(Value::String(text)) if !text.is_empty() => text,
            Some(_) => {
                self.invalid(key);
                default.to_owned()
            }
        }
    }

    /// An object of string values: a missing or invalid object is empty, an entry with another value is skipped.
    fn string_map(&mut self, key: &str) -> BTreeMap<String, String> {
        match self.map.remove(key) {
            None => BTreeMap::new(),
            Some(Value::Object(entries)) => entries
                .into_iter()
                .filter_map(|(name, value)| match value {
                    Value::String(text) => Some((name, text)),
                    _ => {
                        self.invalid(key);
                        None
                    }
                })
                .collect(),
            Some(_) => {
                self.invalid(key);
                BTreeMap::new()
            }
        }
    }

    /// An engine stored by its legacy name that must be one of `allowed`: any other name is the default.
    fn engine(&mut self, key: &str, default: EngineType, allowed: &[EngineType]) -> EngineType {
        match self.map.remove(key) {
            None => default,
            Some(Value::String(text)) => EngineType::from_legacy_name(&text)
                .filter(|engine| allowed.contains(engine))
                .unwrap_or_else(|| {
                    self.invalid(key);
                    default
                }),
            Some(_) => {
                self.invalid(key);
                default
            }
        }
    }

    /// A performance rule stored by its legacy name.
    fn rule(&mut self, key: &str, default: PerformanceType) -> PerformanceType {
        self.name(key, default, PerformanceType::from_legacy_name)
    }

    /// An enum stored by its legacy name; an unknown name is the default.
    fn name<T>(&mut self, key: &str, default: T, from_name: fn(&str) -> Option<T>) -> T {
        match self.map.remove(key) {
            None => default,
            Some(Value::String(text)) => from_name(&text).unwrap_or_else(|| {
                self.invalid(key);
                default
            }),
            Some(_) => {
                self.invalid(key);
                default
            }
        }
    }

    fn rest(self) -> Map<String, Value> {
        self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_legacy_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.engine.volume, 100);
        assert!(settings.engine.looping);
        assert!(settings.engine.hardware_acceleration);
        assert!(settings.general.app_visible);
        assert_eq!(settings.engine.stretch, Stretch::UniformToFill);
        assert_eq!(settings.backgroundog.fullscreen, PerformanceType::Resume);
        assert_eq!(settings.library.selected, "");
    }

    #[test]
    fn serializes_with_the_legacy_section_and_key_names() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["Version"], 1);
        assert_eq!(value["Library"]["Location"], "");
        assert_eq!(value["General"]["AppVisible"], true);
        assert_eq!(value["Engine"]["WallpaperVolume"], 100);
        assert_eq!(value["Engine"]["StretchType"], "UniformToFill");
        assert_eq!(value["Engine"]["ScreenType"], "DisplayBound");
        assert_eq!(value["Engine"]["GraphicPreference"], "HighPerformance");
        assert_eq!(value["Backgroundog"]["FullScreenPerformance"], "Resume");
        assert_eq!(value["Backgroundog"]["PausePerformanceType"], "Light");
    }

    #[test]
    fn round_trips_and_keeps_unknown_keys() {
        let text = br#"{"Version":1,"Future":{"A":1},"Engine":{"WallpaperVolume":7,"Later":"x"},"Library":{"Selected":"abc"}}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.engine.volume, 7);
        assert_eq!(settings.library.selected, "abc");
        let again = Settings::from_slice(&settings.to_bytes().unwrap()).unwrap();
        assert_eq!(again, settings);
        assert_eq!(again.extra["Future"]["A"], 1);
        assert_eq!(again.engine.extra["Later"], "x");
    }

    #[test]
    fn invalid_values_are_the_legacy_defaults() {
        let text = br#"{"Engine":{"WallpaperVolume":"loud","WallpaperLoop":3,"StretchType":"Sideways","ScreenType":7},"General":5}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.engine, Engine::default());
        assert_eq!(settings.general, General::default());
    }

    #[test]
    fn the_portal_and_warehouse_sections_use_the_legacy_key_names() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["Portal"]["ThemeMode"], "System");
        assert_eq!(value["Portal"]["Backdrop"], "None");
        assert_eq!(value["Portal"]["LibraryPagination"], 30);
        assert_eq!(value["Portal"]["StoreAdult"], true);
        assert_eq!(value["Portal"]["BackgroundStretch"], "UniformToFill");
        assert_eq!(value["Library"]["Move"], true);
        assert_eq!(value["General"]["RunStartup"], 0);
        assert!(value["General"].get("Culture").is_none());
        assert_eq!(value["Warehouse"]["CefsharpContinue"], false);
        assert_eq!(value["Warehouse"]["HintTrayIcon"], true);
        assert_eq!(value["Warehouse"]["CefSharpTime"], MIN_DATE_TIME);
    }

    #[test]
    fn the_new_sections_clamp_and_keep_their_values_on_read() {
        let text = br#"{"Portal":{"ThemeMode":"Dark","Backdrop":"Tabbed","LibraryPagination":0,"AdaptiveMargin":99},
            "General":{"Culture":"TR","RunStartup":50},"Warehouse":{"Showcase":["a",1,"b"],"HintTrayIcon":false}}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.portal.theme_mode, ThemeMode::Dark);
        assert_eq!(settings.portal.backdrop, Backdrop::Tabbed);
        assert_eq!(settings.portal.library_pagination, 1);
        assert_eq!(settings.portal.adaptive_margin, 25);
        assert_eq!(settings.general.culture.as_deref(), Some("TR"));
        assert_eq!(settings.general.run_startup, 10);
        assert_eq!(settings.warehouse.showcase, ["a", "b"]);
        assert!(!settings.warehouse.hint_tray_icon);
    }

    #[test]
    fn the_engine_cycling_hook_and_update_sections_use_the_legacy_names_and_defaults() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["Engine"]["Gif"], "MpvPlayerLive");
        assert_eq!(value["Engine"]["Application"], "AuroraLive");
        assert_eq!(value["Engine"]["InputType"], "MouseKeyboard");
        assert_eq!(value["Engine"]["VolumeSilentSensitivity"], 5);
        assert_eq!(value["Backgroundog"]["FullScreenScope"], "AllMonitors");
        assert_eq!(value["Backgroundog"]["ScreenSaverPerformance"], "Pause");
        assert_eq!(value["Backgroundog"]["LockPerformance"], "Close");
        assert_eq!(value["Backgroundog"]["DownloadType"], "Megabyte");
        assert_eq!(value["Cycling"]["TransitionTime"], 1800);
        assert_eq!(value["Hook"]["DiscordRefreshDelay"], 60);
        assert_eq!(value["Update"]["AutoType"], "SemiSilent");
        assert_eq!(value["Update"]["Time"], MIN_DATE_TIME);
        assert!(value["Cycling"].get("PassingTime").is_none());
    }

    #[test]
    fn a_stored_engine_outside_its_allowed_set_is_the_default() {
        let text = br#"{"Engine":{"Video":"AuroraLive","Application":"VlcPlayerLive","Gif":"NebulaLive","Url":"WebViewLive"}}"#;
        let settings = Settings::from_slice(text).unwrap();
        assert_eq!(settings.engine.video, EngineType::MpvPlayerLive);
        assert_eq!(settings.engine.application, EngineType::AuroraLive);
        assert_eq!(settings.engine.gif, EngineType::MpvPlayerLive);
        assert_eq!(settings.engine.url, EngineType::WebViewLive);
    }

    #[test]
    fn volume_is_clamped() {
        let high = Settings::from_slice(br#"{"Engine":{"WallpaperVolume":250}}"#).unwrap();
        let low = Settings::from_slice(br#"{"Engine":{"WallpaperVolume":-4}}"#).unwrap();
        assert_eq!(high.engine.volume, 100);
        assert_eq!(low.engine.volume, 0);
    }

    #[test]
    fn text_that_is_not_an_object_is_corrupt() {
        assert!(Settings::from_slice(b"").is_err());
        assert!(Settings::from_slice(b"{\"Version\":1,").is_err());
        assert!(Settings::from_slice(b"[1]").is_err());
    }

    #[test]
    fn an_empty_location_is_the_legacy_library_folder() {
        let mut settings = Settings::default();
        assert_eq!(
            settings.library_location(Path::new("root")),
            Path::new("root").join("Library")
        );
        settings.library.location = "elsewhere".to_owned();
        assert_eq!(settings.library_location(Path::new("root")), PathBuf::from("elsewhere"));
    }

    #[test]
    fn a_byte_order_mark_is_ignored() {
        let settings = Settings::from_slice(b"\xEF\xBB\xBF{\"Library\":{\"Selected\":\"x\"}}").unwrap();
        assert_eq!(settings.library.selected, "x");
    }
}
