//! The messages the service and the Portal exchange (D-009, D-033).
//!
//! The Portal speaks over the same endpoint as the renderers, announced by [`ToService::PortalHello`]. Like the
//! renderer protocol these enums are **append-only**: postcard encodes a variant by its index, so a variant is only
//! ever added at the end, never inserted, reordered or removed. [`PORTAL_PROTOCOL_VERSION`] is bumped only for a change
//! appending cannot express. The Portal never reads the settings file: it gets a [`PortalSettings`] view in
//! [`PortalEvent::Welcome`] and the changes after it (D-11).
//!
//! [`ToService::PortalHello`]: crate::protocol::ToService::PortalHello

use serde::{Deserialize, Serialize};
use sucrose_core::{
    settings::{Backdrop, EngineType, InputModuleType, InputType, SortKind, SortMode, Stretch, ThemeMode},
    theme::WallpaperType,
};

/// Version of the Portal protocol, sent in `PortalHello` so the service can refuse a Portal it cannot talk to.
pub const PORTAL_PROTOCOL_VERSION: u16 = 7;

/// The longest path text, in bytes, the service accepts from a Portal and puts into a message it sends. It keeps the
/// largest possible [`PortalEvent::Welcome`] far below [`crate::frame::MAX_FRAME`].
pub const MAX_PATH_TEXT: usize = 4096;

/// A page of the Portal that can be opened directly.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalPage {
    /// The theme library.
    Library,
    /// The general settings.
    General,
    /// The personalization settings.
    Personal,
    /// The wallpaper settings: the engine per wallpaper type and the other wallpaper rows.
    Wallpaper,
    /// The other settings: the Developer group in this phase.
    Other,
}

/// What a launcher asks of the service (the first message of a connection that is neither a renderer nor a Portal).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launch {
    /// Open the Portal, on a page when given.
    OpenPortal {
        /// The page to show.
        page: Option<PortalPage>,
    },
    /// The launcher was given the right to bring a window to the front and passes it on.
    ForegroundGranted {
        /// The page to show.
        page: Option<PortalPage>,
    },
}

/// The service's answer to a [`Launch`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum LaunchReply {
    /// The Portal was started.
    Opened,
    /// A Portal already runs; it was asked to focus.
    PortalRunning {
        /// Process id of that Portal.
        pid: u32,
    },
    /// The request was refused.
    Refused {
        /// One sentence saying why.
        reason: String,
    },
}

/// Messages from the Portal to the service, after its `PortalHello`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PortalRequest {
    /// Make the theme in this library folder the wallpaper.
    Use {
        /// Folder name of the theme inside the library.
        folder: String,
    },
    /// Change one setting.
    Change(SettingChange),
    /// Set how Sucrose starts with the OS.
    SetStartup(StartupMode),
    /// Read the startup state again.
    RefreshStartup,
    /// Run the library maintenance (showcase copy, corrupt themes).
    MaintainLibrary,
    /// Move the library to another folder.
    MoveLibrary {
        /// The absolute path of the new folder.
        destination: String,
    },
    /// Look at the engines again and send the report.
    DetectEngines,
    /// "Don't remind me for a day" on the fallback toast of this engine: the service stores the time until which the
    /// toast stays quiet (D-16). Only the engines with a Warehouse field (CEF, WebView2) store anything.
    SnoozeFallback {
        /// The engine the settings ask for and that is not available.
        engine: EngineType,
    },
    /// Ask for the command line switches of a web engine: the stored ones, the ones Sucrose adds itself and the legacy
    /// defaults. The answer is [`PortalEvent::Arguments`].
    GetArguments {
        /// The web engine whose switches are wanted.
        engine: EngineType,
    },
    /// Restart the running wallpapers that play on one of these engines, so a saved setting that is read when a wallpaper
    /// starts applies now (the "Restart now" of the restart prompt). Wallpapers on other engines are left alone.
    RestartWallpapers {
        /// The engines whose wallpapers restart.
        engines: Vec<EngineType>,
    },
    /// Open the developer tools (DevTools) of the web wallpaper that plays on this monitor: the Portal's Open DevTools
    /// button (D-13). The service routes it to that monitor's renderer only when it plays on a web engine; the answer is
    /// [`PortalEvent::DevToolsResult`]. Nothing else ever opens DevTools.
    OpenDevTools {
        /// The device name of the monitor, as in [`RuntimeState::running_engines`].
        monitor: String,
    },
    /// The user confirmed the install dialog: download the Microsoft installer of this prerequisite, check that it is
    /// signed by Microsoft, run it and look again (D-15). The service reports every stage as
    /// [`PortalEvent::InstallProgress`]. Nothing is downloaded before this request.
    InstallPrerequisite {
        /// The part to install.
        prerequisite: PrerequisiteView,
    },
    /// Stop the running download ("Stop Download"). Once the installer has started it cannot be stopped from here.
    CancelInstall,
}

/// Messages from the service to the Portal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PortalEvent {
    /// Answer to an accepted `PortalHello`: the settings view and the runtime state.
    Welcome {
        /// The settings the Portal shows.
        settings: PortalSettings,
        /// What the service is doing.
        runtime: RuntimeState,
    },
    /// Another Portal is already open; this connection ends.
    AlreadyOpen {
        /// Process id of the Portal that is open.
        existing_pid: u32,
    },
    /// The service cannot talk to this Portal; this connection ends.
    Refused {
        /// One sentence saying why.
        reason: String,
    },
    /// A setting changed (by this Portal, a later plan's other source, or a migration).
    Changed(SettingChange),
    /// A requested change was not applied.
    Rejected {
        /// The change that was refused.
        change: SettingChange,
        /// One sentence saying why.
        reason: String,
    },
    /// The runtime state changed.
    Runtime(RuntimeState),
    /// The outcome of a [`PortalRequest::Use`].
    UseResult {
        /// The folder the request named.
        folder: String,
        /// What became of it.
        outcome: UseOutcome,
    },
    /// The library maintenance finished.
    LibraryMaintained(MaintainReport),
    /// The outcome of a [`PortalRequest::MoveLibrary`].
    MoveResult(MoveOutcome),
    /// Memory use of the Sucrose processes, for the Portal's status page.
    Memory(MemoryReport),
    /// How Sucrose starts with the OS, now.
    Startup(StartupInfo),
    /// Bring the Portal window to the front: another launch asked for it.
    Focus {
        /// The page to show.
        page: Option<PortalPage>,
    },
    /// Close the Portal: the service is stopping.
    Close,
    /// What the service detected about the engines and which fallbacks run.
    Engines(EngineReportView),
    /// The answer to a [`PortalRequest::GetArguments`]: the switches of one web engine.
    Arguments {
        /// The engine the switches are for.
        engine: EngineType,
        /// The user's switches as the engine reads them (an empty stored list reads as the legacy defaults, as in the
        /// classic app).
        user: Vec<SwitchView>,
        /// The switches Sucrose adds from the settings; read-only in the editor, never stored.
        sucrose: Vec<SwitchView>,
        /// What Reset to Defaults puts into the editor: the legacy default switches.
        defaults: Vec<SwitchView>,
    },
    /// A [`PortalRequest::RestartWallpapers`] could not be carried out; the saved setting stays and applies at the next
    /// start of a wallpaper.
    RestartFailed {
        /// One sentence saying why.
        reason: String,
    },
    /// The answer to a [`PortalRequest::OpenDevTools`]: whether the renderer opened the developer tools.
    DevToolsResult {
        /// `false` when no web wallpaper plays on that monitor, the engine has no developer tools or the renderer did not
        /// confirm in time.
        opened: bool,
    },
    /// Where the install the Portal asked for is. Every stage is sent as it starts; `Done`, `Failed` and `Cancelled` are
    /// the last, and a new engine report follows `Done`.
    InstallProgress(InstallStageView),
}

/// A part of the machine the service can install for an engine (D-15).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrerequisiteView {
    /// The Visual C++ Redistributable CEF needs.
    VcRedist,
    /// The WebView2 Runtime (installable from Round B on).
    WebView2Runtime,
}

/// Why an install did not finish, as the Portal words it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallFailureView {
    /// The installer could not be downloaded.
    Download,
    /// The installer is not signed by Microsoft; it was not run and nothing was installed.
    Signature,
    /// The installer did not finish, or the part is still missing afterwards.
    Installer,
}

/// Where an install is, as the Portal shows it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStageView {
    /// The installer is being downloaded.
    Downloading {
        /// Bytes received so far.
        received: u64,
        /// Bytes expected, when the server said.
        total: Option<u64>,
    },
    /// The signature is being checked.
    Verifying,
    /// The installer runs; Windows may ask for permission.
    Installing,
    /// The part is being looked for again.
    Rechecking,
    /// Installed.
    Done,
    /// Stopped with this reason.
    Failed(InstallFailureView),
    /// The user stopped the download.
    Cancelled,
}

/// One command line switch of a web engine as the Portal shows it: a name without leading dashes and, optionally, a
/// value. The rules for names and the storage format are the service's (D-17); the Portal links no engine crate.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SwitchView {
    /// The name without leading dashes, as in `autoplay-policy`.
    pub name: String,
    /// The value after the equals sign; `None` for a switch that stands alone.
    pub value: Option<String>,
}

/// The most switches the service accepts in one list from a Portal. With the two limits below it keeps the largest
/// [`PortalEvent::Arguments`] under [`crate::frame::MAX_FRAME`].
pub const MAX_SWITCHES: usize = 64;

/// The longest switch name, in bytes, the service accepts.
pub const MAX_SWITCH_NAME: usize = 64;

/// The longest switch value, in bytes, the service accepts (the legacy default `disable-features` is the longest known
/// value, at under 300 bytes).
pub const MAX_SWITCH_VALUE: usize = 512;

/// What the user can do about an engine that is not usable (ENG-06), as the Portal shows it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HintView {
    /// The engine's files are not next to the executable: reinstall or repair Sucrose.
    FilesMissing,
    /// The Visual C++ Redistributable is missing or too old; `needed` is the version to install.
    VcRedistMissing {
        /// The version the engine needs.
        needed: String,
    },
    /// A user-installed runtime (WebView2) is not installed.
    RuntimeMissing,
    /// A user-installed runtime is older than the engine needs.
    RuntimeOutdated {
        /// The version found.
        found: String,
        /// The version needed.
        needed: String,
    },
}

/// What the machine can run of one engine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EngineState {
    /// Present and ready.
    Ready {
        /// The version, when the engine reports one without being loaded.
        version: Option<String>,
    },
    /// Not present.
    Missing {
        /// How to get it.
        hint: HintView,
        /// Whether Sucrose can install the missing part itself (a prerequisite) rather than only advise a repair.
        installable: bool,
    },
    /// Present but it cannot be loaded.
    NotLoadable {
        /// The loader's reason.
        reason: String,
        /// How to repair it.
        hint: HintView,
    },
    /// No build exists for this machine or this Sucrose build.
    Unsupported {
        /// `true` when the cause is the processor architecture, `false` when this build does not include the engine.
        architecture: bool,
    },
}

/// One engine and its state, named by the legacy engine name Sucrose writes for it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EngineStateView {
    /// The engine.
    pub engine: EngineType,
    /// What the machine can run of it.
    pub state: EngineState,
}

/// A wallpaper type that plays with another engine than the configured one, for this run only (D-11).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallbackView {
    /// The wallpaper type.
    pub kind: WallpaperType,
    /// The engine the settings ask for.
    pub configured: EngineType,
    /// The engine that plays instead.
    pub playing: EngineType,
    /// Whether the Portal raises its warning toast for this fallback: false while the one-day snooze of the configured
    /// engine runs (D-16). The row alert ignores it, it is never snoozed.
    pub notify: bool,
}

/// The engines the Portal offers for one wallpaper type: the ones whose slice is part of this build, installed or not.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OfferedView {
    /// The wallpaper type.
    pub kind: WallpaperType,
    /// The engines that can play it in this build, in the order of the service's list.
    pub engines: Vec<EngineType>,
}

/// What the service detected about the engines (ENG-06, ENG-07): sent after the Welcome, on request and when a
/// fallback starts or ends.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EngineReportView {
    /// Every engine with its state.
    pub engines: Vec<EngineStateView>,
    /// The wallpaper types this build plays once an engine for them is installed.
    pub supported: Vec<WallpaperType>,
    /// The wallpaper types some available engine can play now (a subset of `supported`).
    pub playable: Vec<WallpaperType>,
    /// Per supported type, the engines the settings may offer (D-26): the Portal never decides which engines exist.
    pub offered: Vec<OfferedView>,
    /// The fallbacks that run now.
    pub fallbacks: Vec<FallbackView>,
}

/// One setting the Portal can change, with its new value.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SettingChange {
    /// Light, dark or follow the system.
    ThemeMode(ThemeMode),
    /// The material behind the window.
    Backdrop(Backdrop),
    /// Path of the window background image, empty for none.
    BackgroundImage(String),
    /// How the background image is fitted.
    BackgroundStretch(Stretch),
    /// Opacity of the window background, 0..=100.
    BackgroundOpacity(u8),
    /// Themes per library page, 1..=100.
    LibraryPagination(u8),
    /// How strongly the library grid adapts to the window width, 0..=100.
    AdaptiveLayout(u8),
    /// Margin of the adaptive grid, 5..=25.
    AdaptiveMargin(u8),
    /// Whether library cards preview the theme.
    LibraryPreview(bool),
    /// Whether library cards hide their details.
    LibraryPreviewHide(bool),
    /// What the library is sorted by.
    LibrarySortMode(SortMode),
    /// Direction of the library sort.
    LibrarySortKind(SortKind),
    /// Whether themes that fail the compatibility check are deleted.
    DeleteCorrupt(bool),
    /// Whether changing the library location moves the themes along.
    LibraryMove(bool),
    /// Whether the tray icon and the app window are shown.
    AppVisible(bool),
    /// Whether closing the Portal exits it instead of hiding it.
    AppExit(bool),
    /// Whether the hint about the notification area icon is still shown.
    HintTrayIcon(bool),
    /// The engine that plays Gif wallpapers.
    GifEngine(EngineType),
    /// The engine that plays Url wallpapers.
    UrlEngine(EngineType),
    /// The engine that plays Web wallpapers.
    WebEngine(EngineType),
    /// The engine that plays Video wallpapers.
    VideoEngine(EngineType),
    /// The engine that plays YouTube wallpapers.
    YouTubeEngine(EngineType),
    /// The engine that plays Application wallpapers.
    ApplicationEngine(EngineType),
    /// Whether wallpapers loop; running wallpapers follow at once (PLAY-01).
    WallpaperLoop(bool),
    /// Whether the `ShuffleMode` call of web wallpapers says shuffle; running pages follow at once (PLAY-01).
    WallpaperShuffle(bool),
    /// Whether the engines may use the GPU; applies the next time a wallpaper starts.
    HardwareAcceleration(bool),
    /// Which input is forwarded to web wallpapers; applies the next time a wallpaper starts.
    InputType(InputType),
    /// How input is read; applies the next time a wallpaper starts.
    InputModuleType(InputModuleType),
    /// Whether input is processed only while the desktop is focused; applies the next time a wallpaper starts.
    InputDesktop(bool),
    /// Whether the web engines accept invalid TLS certificates (D-12); applies the next time a web wallpaper starts.
    AllowInvalidCertificates(bool),
    /// The user's switches of the CEF engine, stored by the service in the legacy dictionary (D-17); applies the next
    /// time a CEF wallpaper starts. Unlike the other changes it has no field in [`PortalSettings`]: the Portal asks
    /// for the list with [`PortalRequest::GetArguments`].
    CefArguments(Vec<SwitchView>),
    /// Whether developer mode is on: the wallpaper's context menu has Inspect and F12 opens the developer tools (D-13);
    /// applies the next time a web wallpaper starts.
    DeveloperMode(bool),
    /// The debugging port of the web engines, 0..=65535, 0 for none; each renderer listens on this port plus its slot.
    /// Applies the next time a web wallpaper starts.
    DeveloperPort(u16),
    /// The user's switches of the WebView2 engine, stored by the service in the legacy list (`"--name"` or
    /// `"--name=value"`, D-17); applies the next time a WebView2 wallpaper starts. Like [`SettingChange::CefArguments`] it
    /// has no field in [`PortalSettings`]: the Portal asks for the list with [`PortalRequest::GetArguments`].
    WebArguments(Vec<SwitchView>),
}

/// The settings the Portal shows: a view of the service's settings, never the file (D-11).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PortalSettings {
    /// Whether the tray icon and the app window are shown.
    pub app_visible: bool,
    /// Whether closing the Portal exits it instead of hiding it.
    pub app_exit: bool,
    /// How Sucrose starts with the OS, 0..=10 (see [`StartupMode::from_run_startup`]).
    pub run_startup: u8,
    /// Light, dark or follow the system.
    pub theme_mode: ThemeMode,
    /// The material behind the window.
    pub backdrop: Backdrop,
    /// Path of the window background image, empty for none.
    pub background_image: String,
    /// How the background image is fitted.
    pub background_stretch: Stretch,
    /// Opacity of the window background, 0..=100.
    pub background_opacity: u8,
    /// Themes per library page.
    pub library_pagination: u8,
    /// How strongly the library grid adapts to the window width.
    pub adaptive_layout: u8,
    /// Margin of the adaptive grid.
    pub adaptive_margin: u8,
    /// Whether library cards preview the theme.
    pub library_preview: bool,
    /// Whether library cards hide their details.
    pub library_preview_hide: bool,
    /// What the library is sorted by.
    pub library_sort_mode: SortMode,
    /// Direction of the library sort.
    pub library_sort_kind: SortKind,
    /// The library folder as configured; empty means the default folder.
    pub library_location: String,
    /// Whether changing the library location moves the themes along.
    pub library_move: bool,
    /// Whether themes that fail the compatibility check are deleted.
    pub delete_corrupt: bool,
    /// Whether the hint about the notification area icon is still shown.
    pub hint_tray_icon: bool,
    /// The engine that plays Gif wallpapers.
    pub gif_engine: EngineType,
    /// The engine that plays Url wallpapers.
    pub url_engine: EngineType,
    /// The engine that plays Web wallpapers.
    pub web_engine: EngineType,
    /// The engine that plays Video wallpapers.
    pub video_engine: EngineType,
    /// The engine that plays YouTube wallpapers.
    pub youtube_engine: EngineType,
    /// The engine that plays Application wallpapers.
    pub application_engine: EngineType,
    /// Whether wallpapers loop.
    pub wallpaper_loop: bool,
    /// Whether the `ShuffleMode` call of web wallpapers says shuffle.
    pub wallpaper_shuffle: bool,
    /// Whether the engines may use the GPU.
    pub hardware_acceleration: bool,
    /// Which input is forwarded to web wallpapers.
    pub input_type: InputType,
    /// How input is read.
    pub input_module_type: InputModuleType,
    /// Whether input is processed only while the desktop is focused.
    pub input_desktop: bool,
    /// Whether the web engines accept invalid TLS certificates.
    pub allow_invalid_certificates: bool,
    /// Whether developer mode is on.
    pub developer_mode: bool,
    /// The debugging port, 0 for none.
    pub developer_port: u16,
}

impl PortalSettings {
    /// Applies `change`: sets exactly the field it names.
    pub fn apply(&mut self, change: &SettingChange) {
        match change {
            SettingChange::ThemeMode(value) => self.theme_mode = *value,
            SettingChange::Backdrop(value) => self.backdrop = *value,
            SettingChange::BackgroundImage(value) => self.background_image.clone_from(value),
            SettingChange::BackgroundStretch(value) => self.background_stretch = *value,
            SettingChange::BackgroundOpacity(value) => self.background_opacity = *value,
            SettingChange::LibraryPagination(value) => self.library_pagination = *value,
            SettingChange::AdaptiveLayout(value) => self.adaptive_layout = *value,
            SettingChange::AdaptiveMargin(value) => self.adaptive_margin = *value,
            SettingChange::LibraryPreview(value) => self.library_preview = *value,
            SettingChange::LibraryPreviewHide(value) => self.library_preview_hide = *value,
            SettingChange::LibrarySortMode(value) => self.library_sort_mode = *value,
            SettingChange::LibrarySortKind(value) => self.library_sort_kind = *value,
            SettingChange::DeleteCorrupt(value) => self.delete_corrupt = *value,
            SettingChange::LibraryMove(value) => self.library_move = *value,
            SettingChange::AppVisible(value) => self.app_visible = *value,
            SettingChange::AppExit(value) => self.app_exit = *value,
            SettingChange::HintTrayIcon(value) => self.hint_tray_icon = *value,
            SettingChange::GifEngine(value) => self.gif_engine = *value,
            SettingChange::UrlEngine(value) => self.url_engine = *value,
            SettingChange::WebEngine(value) => self.web_engine = *value,
            SettingChange::VideoEngine(value) => self.video_engine = *value,
            SettingChange::YouTubeEngine(value) => self.youtube_engine = *value,
            SettingChange::ApplicationEngine(value) => self.application_engine = *value,
            SettingChange::WallpaperLoop(value) => self.wallpaper_loop = *value,
            SettingChange::WallpaperShuffle(value) => self.wallpaper_shuffle = *value,
            SettingChange::HardwareAcceleration(value) => self.hardware_acceleration = *value,
            SettingChange::InputType(value) => self.input_type = *value,
            SettingChange::InputModuleType(value) => self.input_module_type = *value,
            SettingChange::InputDesktop(value) => self.input_desktop = *value,
            SettingChange::AllowInvalidCertificates(value) => self.allow_invalid_certificates = *value,
            // The switches are not part of the view: the Portal asks for them.
            SettingChange::CefArguments(_) | SettingChange::WebArguments(_) => {}
            SettingChange::DeveloperMode(value) => self.developer_mode = *value,
            SettingChange::DeveloperPort(value) => self.developer_port = *value,
        }
    }
}

/// What the service is doing, for the Portal's status.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState {
    /// Folder name of the selected theme, if any.
    pub selected: Option<String>,
    /// Whether a wallpaper renderer runs.
    pub running: bool,
    /// What a performance rule does to the wallpaper now.
    pub rule: RuleState,
    /// Whether the library is being moved.
    pub library_moving: bool,
    /// Whether the tray icon exists.
    pub tray_visible: bool,
    /// The monitor and the engine of each wallpaper that runs now, by monitor in the service's slot order. The Portal
    /// reads it to decide whether a changed setting needs the restart prompt (D-17).
    pub running_engines: Vec<(String, EngineType)>,
}

/// What a performance rule does to the wallpaper.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleState {
    /// No rule acts.
    None,
    /// A rule paused the wallpaper.
    Paused,
    /// A rule closed the wallpaper.
    Closed,
}

/// The outcome of a [`PortalRequest::Use`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum UseOutcome {
    /// The theme is the wallpaper now.
    Applied,
    /// The theme was not applied, for a reason the Portal can explain.
    Refused(UseRefusal),
    /// Applying failed unexpectedly.
    Failed {
        /// One sentence saying what went wrong.
        reason: String,
    },
}

/// Why a theme cannot be used.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseRefusal {
    /// No such theme folder.
    NotFound,
    /// The theme's files are damaged.
    Corrupt,
    /// The theme's type is one this build does not play yet.
    TypeNotPlayable,
    /// The theme needs a newer Sucrose.
    NewerVersion,
    /// A performance rule paused the wallpaper.
    RulePaused,
    /// A performance rule closed the wallpaper.
    RuleClosed,
    /// The library is being moved.
    Moving,
    /// The wallpaper cannot be shown now (no monitor, no player).
    Unavailable,
    /// No installed engine can play the theme's type (D-11).
    NoEngine,
}

/// What the library maintenance did.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaintainReport {
    /// Showcase themes copied into the library.
    pub showcase_copied: u32,
    /// Corrupt themes moved out of the library.
    pub corrupt_moved: u32,
    /// Corrupt themes left where they are.
    pub corrupt_skipped: u32,
}

/// The outcome of a [`PortalRequest::MoveLibrary`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MoveOutcome {
    /// The themes moved to the new folder.
    Moved,
    /// Only the location changed; the themes stayed.
    Changed,
    /// The move was refused.
    Refused(MoveRefusal),
    /// The move failed unexpectedly.
    Failed {
        /// One sentence saying what went wrong.
        reason: String,
    },
}

/// Why the library cannot be moved there.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveRefusal {
    /// The destination holds files already.
    NotEmpty,
    /// The destination cannot be written.
    NotWritable,
    /// The destination lies inside the library or the library inside it.
    Nested,
    /// Another move or a theme change is running.
    Busy,
    /// The destination is the current library.
    Same,
    /// The destination does not exist.
    Missing,
}

/// Memory use of the Sucrose processes, in bytes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MemoryReport {
    /// The service.
    pub service: u64,
    /// Each renderer.
    pub renderers: Vec<u64>,
    /// The Portal itself, as it measured it.
    pub portal: u64,
}

/// How Sucrose starts with the OS, as found now.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupInfo {
    /// The mode the settings ask for.
    pub mode: StartupMode,
    /// Whether the OS entry for that mode exists.
    pub entry_present: bool,
    /// Whether the OS turned the entry off (for example in Task Manager).
    pub disabled_by_os: bool,
    /// Whether an entry of the C# app exists.
    pub legacy_entry: bool,
}

/// How Sucrose starts with the OS.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    /// Not at all.
    None,
    /// A startup entry.
    Normal,
    /// An elevated startup entry.
    Priority,
    /// A scheduler task.
    Scheduler,
}

impl StartupMode {
    /// The mode of a stored `RunStartup` value: 1, 2 and 3 are the entries, 0 and everything above (the legacy range
    /// goes to 10) is off.
    pub fn from_run_startup(value: u8) -> Self {
        match value {
            1 => Self::Normal,
            2 => Self::Priority,
            3 => Self::Scheduler,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::MAX_FRAME;

    /// The wire index of a value: postcard's first byte for an enum with fewer than 128 variants.
    fn index<T: Serialize>(value: &T) -> u8 {
        postcard::to_stdvec(value).expect("encode")[0]
    }

    fn view() -> PortalSettings {
        PortalSettings {
            app_visible: true,
            app_exit: true,
            run_startup: 0,
            theme_mode: ThemeMode::default(),
            backdrop: Backdrop::default(),
            background_image: String::new(),
            background_stretch: Stretch::default(),
            background_opacity: 100,
            library_pagination: 30,
            adaptive_layout: 0,
            adaptive_margin: 5,
            library_preview: false,
            library_preview_hide: false,
            library_sort_mode: SortMode::default(),
            library_sort_kind: SortKind::default(),
            library_location: String::new(),
            library_move: true,
            delete_corrupt: false,
            hint_tray_icon: true,
            gif_engine: EngineType::MpvPlayerLive,
            url_engine: EngineType::CefSharpLive,
            web_engine: EngineType::CefSharpLive,
            video_engine: EngineType::MpvPlayerLive,
            youtube_engine: EngineType::CefSharpLive,
            application_engine: EngineType::AuroraLive,
            wallpaper_loop: true,
            wallpaper_shuffle: true,
            hardware_acceleration: true,
            input_type: InputType::MouseKeyboard,
            input_module_type: InputModuleType::RawInput,
            input_desktop: false,
            allow_invalid_certificates: false,
            developer_mode: false,
            developer_port: 0,
        }
    }

    fn engine_report() -> EngineReportView {
        EngineReportView {
            engines: vec![EngineStateView {
                engine: EngineType::MpvPlayerLive,
                state: EngineState::Ready { version: None },
            }],
            supported: vec![WallpaperType::Video, WallpaperType::Web],
            playable: vec![WallpaperType::Video],
            offered: vec![OfferedView {
                kind: WallpaperType::Video,
                engines: vec![EngineType::MpvPlayerLive],
            }],
            fallbacks: vec![FallbackView {
                kind: WallpaperType::Web,
                configured: EngineType::WebViewLive,
                playing: EngineType::CefSharpLive,
                notify: true,
            }],
        }
    }

    fn runtime() -> RuntimeState {
        RuntimeState {
            selected: None,
            running: false,
            rule: RuleState::None,
            library_moving: false,
            tray_visible: false,
            running_engines: Vec::new(),
        }
    }

    #[test]
    fn the_portal_protocol_is_at_version_seven_since_webview2_has_its_own_argument_change() {
        assert_eq!(PORTAL_PROTOCOL_VERSION, 7);
    }

    #[test]
    fn portal_request_variants_keep_their_wire_index() {
        let folder = String::new();
        let requests = [
            PortalRequest::Use { folder: folder.clone() },
            PortalRequest::Change(SettingChange::AppExit(true)),
            PortalRequest::SetStartup(StartupMode::None),
            PortalRequest::RefreshStartup,
            PortalRequest::MaintainLibrary,
            PortalRequest::MoveLibrary { destination: folder },
            PortalRequest::DetectEngines,
            PortalRequest::SnoozeFallback {
                engine: EngineType::WebViewLive,
            },
            PortalRequest::GetArguments {
                engine: EngineType::CefSharpLive,
            },
            PortalRequest::RestartWallpapers {
                engines: vec![EngineType::CefSharpLive],
            },
            PortalRequest::OpenDevTools { monitor: String::new() },
            PortalRequest::InstallPrerequisite {
                prerequisite: PrerequisiteView::VcRedist,
            },
            PortalRequest::CancelInstall,
        ];
        for (expected, request) in requests.iter().enumerate() {
            assert_eq!(usize::from(index(request)), expected, "{request:?}");
        }
    }

    #[test]
    fn portal_event_variants_keep_their_wire_index() {
        let change = SettingChange::AppExit(true);
        let events = [
            PortalEvent::Welcome {
                settings: view(),
                runtime: runtime(),
            },
            PortalEvent::AlreadyOpen { existing_pid: 1 },
            PortalEvent::Refused { reason: String::new() },
            PortalEvent::Changed(change.clone()),
            PortalEvent::Rejected {
                change,
                reason: String::new(),
            },
            PortalEvent::Runtime(runtime()),
            PortalEvent::UseResult {
                folder: String::new(),
                outcome: UseOutcome::Applied,
            },
            PortalEvent::LibraryMaintained(MaintainReport {
                showcase_copied: 0,
                corrupt_moved: 0,
                corrupt_skipped: 0,
            }),
            PortalEvent::MoveResult(MoveOutcome::Moved),
            PortalEvent::Memory(MemoryReport {
                service: 0,
                renderers: Vec::new(),
                portal: 0,
            }),
            PortalEvent::Startup(StartupInfo {
                mode: StartupMode::None,
                entry_present: false,
                disabled_by_os: false,
                legacy_entry: false,
            }),
            PortalEvent::Focus { page: None },
            PortalEvent::Close,
            PortalEvent::Engines(engine_report()),
            PortalEvent::Arguments {
                engine: EngineType::CefSharpLive,
                user: Vec::new(),
                sucrose: Vec::new(),
                defaults: Vec::new(),
            },
            PortalEvent::RestartFailed { reason: String::new() },
            PortalEvent::DevToolsResult { opened: true },
            PortalEvent::InstallProgress(InstallStageView::Verifying),
        ];
        for (expected, event) in events.iter().enumerate() {
            assert_eq!(usize::from(index(event)), expected, "{event:?}");
        }
    }

    #[test]
    fn engine_states_and_hints_keep_their_wire_index() {
        let hint = HintView::FilesMissing;
        let states = [
            EngineState::Ready { version: None },
            EngineState::Missing {
                hint: hint.clone(),
                installable: false,
            },
            EngineState::NotLoadable {
                reason: String::new(),
                hint,
            },
            EngineState::Unsupported { architecture: true },
        ];
        for (expected, state) in states.iter().enumerate() {
            assert_eq!(usize::from(index(state)), expected, "{state:?}");
        }
        let hints = [
            HintView::FilesMissing,
            HintView::VcRedistMissing { needed: String::new() },
            HintView::RuntimeMissing,
            HintView::RuntimeOutdated {
                found: String::new(),
                needed: String::new(),
            },
        ];
        for (expected, hint) in hints.iter().enumerate() {
            assert_eq!(usize::from(index(hint)), expected, "{hint:?}");
        }
    }

    #[test]
    fn an_engine_report_survives_the_wire() {
        let event = PortalEvent::Engines(EngineReportView {
            engines: vec![EngineStateView {
                engine: EngineType::CefSharpLive,
                state: EngineState::Missing {
                    hint: HintView::VcRedistMissing {
                        needed: "14.40.33816.0".to_owned(),
                    },
                    installable: true,
                },
            }],
            ..engine_report()
        });

        let bytes = postcard::to_stdvec(&event).expect("encode");

        assert_eq!(postcard::from_bytes::<PortalEvent>(&bytes).expect("decode"), event);
    }

    #[test]
    fn setting_change_variants_keep_their_wire_index() {
        let changes = [
            SettingChange::ThemeMode(ThemeMode::default()),
            SettingChange::Backdrop(Backdrop::default()),
            SettingChange::BackgroundImage(String::new()),
            SettingChange::BackgroundStretch(Stretch::default()),
            SettingChange::BackgroundOpacity(0),
            SettingChange::LibraryPagination(0),
            SettingChange::AdaptiveLayout(0),
            SettingChange::AdaptiveMargin(0),
            SettingChange::LibraryPreview(false),
            SettingChange::LibraryPreviewHide(false),
            SettingChange::LibrarySortMode(SortMode::default()),
            SettingChange::LibrarySortKind(SortKind::default()),
            SettingChange::DeleteCorrupt(false),
            SettingChange::LibraryMove(false),
            SettingChange::AppVisible(false),
            SettingChange::AppExit(false),
            SettingChange::HintTrayIcon(false),
            SettingChange::GifEngine(EngineType::default()),
            SettingChange::UrlEngine(EngineType::default()),
            SettingChange::WebEngine(EngineType::default()),
            SettingChange::VideoEngine(EngineType::default()),
            SettingChange::YouTubeEngine(EngineType::default()),
            SettingChange::ApplicationEngine(EngineType::default()),
            SettingChange::WallpaperLoop(false),
            SettingChange::WallpaperShuffle(false),
            SettingChange::HardwareAcceleration(false),
            SettingChange::InputType(InputType::default()),
            SettingChange::InputModuleType(InputModuleType::default()),
            SettingChange::InputDesktop(false),
            SettingChange::AllowInvalidCertificates(false),
            SettingChange::CefArguments(Vec::new()),
            SettingChange::DeveloperMode(false),
            SettingChange::DeveloperPort(0),
            SettingChange::WebArguments(Vec::new()),
        ];
        for (expected, change) in changes.iter().enumerate() {
            assert_eq!(usize::from(index(change)), expected, "{change:?}");
        }
    }

    #[test]
    fn launch_and_its_reply_keep_their_wire_index() {
        assert_eq!(index(&Launch::OpenPortal { page: None }), 0);
        assert_eq!(index(&Launch::ForegroundGranted { page: None }), 1);
        assert_eq!(index(&LaunchReply::Opened), 0);
        assert_eq!(index(&LaunchReply::PortalRunning { pid: 1 }), 1);
        assert_eq!(index(&LaunchReply::Refused { reason: String::new() }), 2);
    }

    #[test]
    fn portal_page_and_rule_state_keep_their_wire_index() {
        assert_eq!(
            [
                PortalPage::Library,
                PortalPage::General,
                PortalPage::Personal,
                PortalPage::Wallpaper,
                PortalPage::Other
            ]
            .map(|page| index(&page)),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            [RuleState::None, RuleState::Paused, RuleState::Closed].map(|state| index(&state)),
            [0, 1, 2]
        );
    }

    #[test]
    fn use_outcomes_and_refusals_keep_their_wire_index() {
        assert_eq!(index(&UseOutcome::Applied), 0);
        assert_eq!(index(&UseOutcome::Refused(UseRefusal::NotFound)), 1);
        assert_eq!(index(&UseOutcome::Failed { reason: String::new() }), 2);
        let refusals = [
            UseRefusal::NotFound,
            UseRefusal::Corrupt,
            UseRefusal::TypeNotPlayable,
            UseRefusal::NewerVersion,
            UseRefusal::RulePaused,
            UseRefusal::RuleClosed,
            UseRefusal::Moving,
            UseRefusal::Unavailable,
            UseRefusal::NoEngine,
        ];
        for (expected, refusal) in refusals.iter().enumerate() {
            assert_eq!(usize::from(index(refusal)), expected, "{refusal:?}");
        }
    }

    #[test]
    fn move_outcomes_and_refusals_keep_their_wire_index() {
        assert_eq!(index(&MoveOutcome::Moved), 0);
        assert_eq!(index(&MoveOutcome::Changed), 1);
        assert_eq!(index(&MoveOutcome::Refused(MoveRefusal::Same)), 2);
        assert_eq!(index(&MoveOutcome::Failed { reason: String::new() }), 3);
        let refusals = [
            MoveRefusal::NotEmpty,
            MoveRefusal::NotWritable,
            MoveRefusal::Nested,
            MoveRefusal::Busy,
            MoveRefusal::Same,
            MoveRefusal::Missing,
        ];
        for (expected, refusal) in refusals.iter().enumerate() {
            assert_eq!(usize::from(index(refusal)), expected, "{refusal:?}");
        }
    }

    #[test]
    fn startup_modes_keep_their_wire_index() {
        assert_eq!(
            [
                StartupMode::None,
                StartupMode::Normal,
                StartupMode::Priority,
                StartupMode::Scheduler
            ]
            .map(|mode| index(&mode)),
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn the_largest_welcome_encodes_far_below_the_frame_limit() {
        // A 3-byte character repeated up to the cap: the worst case for text that is limited in bytes.
        let path = "\u{20ac}".repeat(MAX_PATH_TEXT / 3);
        assert!(path.len() <= MAX_PATH_TEXT);
        let mut settings = view();
        settings.background_image = path.clone();
        settings.library_location = path.clone();
        let mut runtime = runtime();
        runtime.selected = Some(path);
        let welcome = PortalEvent::Welcome { settings, runtime };

        let bytes = postcard::to_stdvec(&welcome).expect("encode");

        let limit = usize::try_from(MAX_FRAME / 4).expect("the limit fits");
        assert!(bytes.len() < limit, "{} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<PortalEvent>(&bytes).expect("decode"), welcome);
    }

    #[test]
    fn apply_sets_exactly_the_field_a_change_names() {
        let base = view();
        let cases: [(SettingChange, PortalSettings); 20] = [
            (
                SettingChange::GifEngine(EngineType::VlcPlayerLive),
                PortalSettings {
                    gif_engine: EngineType::VlcPlayerLive,
                    ..view()
                },
            ),
            (
                SettingChange::UrlEngine(EngineType::WebViewLive),
                PortalSettings {
                    url_engine: EngineType::WebViewLive,
                    ..view()
                },
            ),
            (
                SettingChange::WebEngine(EngineType::WebViewLive),
                PortalSettings {
                    web_engine: EngineType::WebViewLive,
                    ..view()
                },
            ),
            (
                SettingChange::VideoEngine(EngineType::NebulaLive),
                PortalSettings {
                    video_engine: EngineType::NebulaLive,
                    ..view()
                },
            ),
            (
                SettingChange::YouTubeEngine(EngineType::WebViewLive),
                PortalSettings {
                    youtube_engine: EngineType::WebViewLive,
                    ..view()
                },
            ),
            (
                SettingChange::ApplicationEngine(EngineType::AuroraLive),
                PortalSettings {
                    application_engine: EngineType::AuroraLive,
                    ..view()
                },
            ),
            (
                SettingChange::LibraryPagination(17),
                PortalSettings {
                    library_pagination: 17,
                    ..view()
                },
            ),
            (
                SettingChange::BackgroundImage("x.png".to_owned()),
                PortalSettings {
                    background_image: "x.png".to_owned(),
                    ..view()
                },
            ),
            (
                SettingChange::AppVisible(false),
                PortalSettings {
                    app_visible: false,
                    ..view()
                },
            ),
            (
                SettingChange::LibraryMove(false),
                PortalSettings {
                    library_move: false,
                    ..view()
                },
            ),
            (
                SettingChange::HintTrayIcon(false),
                PortalSettings {
                    hint_tray_icon: false,
                    ..view()
                },
            ),
            (
                SettingChange::AdaptiveMargin(25),
                PortalSettings {
                    adaptive_margin: 25,
                    ..view()
                },
            ),
            (
                SettingChange::WallpaperLoop(false),
                PortalSettings {
                    wallpaper_loop: false,
                    ..view()
                },
            ),
            (
                SettingChange::WallpaperShuffle(false),
                PortalSettings {
                    wallpaper_shuffle: false,
                    ..view()
                },
            ),
            (
                SettingChange::HardwareAcceleration(false),
                PortalSettings {
                    hardware_acceleration: false,
                    ..view()
                },
            ),
            (
                SettingChange::InputType(InputType::OnlyMouse),
                PortalSettings {
                    input_type: InputType::OnlyMouse,
                    ..view()
                },
            ),
            (
                SettingChange::InputDesktop(true),
                PortalSettings {
                    input_desktop: true,
                    ..view()
                },
            ),
            (
                SettingChange::AllowInvalidCertificates(true),
                PortalSettings {
                    allow_invalid_certificates: true,
                    ..view()
                },
            ),
            (
                SettingChange::DeveloperMode(true),
                PortalSettings {
                    developer_mode: true,
                    ..view()
                },
            ),
            (
                SettingChange::DeveloperPort(9222),
                PortalSettings {
                    developer_port: 9222,
                    ..view()
                },
            ),
        ];
        for (change, expected) in cases {
            let mut settings = base.clone();
            settings.apply(&change);
            assert_eq!(settings, expected, "{change:?}");
        }
    }

    #[test]
    fn apply_covers_every_change_without_touching_the_startup_or_location() {
        let mut settings = view();
        settings.run_startup = 2;
        settings.library_location = "L".to_owned();

        for change in [
            SettingChange::ThemeMode(ThemeMode::default()),
            SettingChange::BackgroundOpacity(5),
            SettingChange::AdaptiveLayout(9),
            SettingChange::LibraryPreview(true),
            SettingChange::LibraryPreviewHide(true),
            SettingChange::DeleteCorrupt(true),
            SettingChange::AppExit(false),
        ] {
            settings.apply(&change);
        }

        assert_eq!(settings.run_startup, 2);
        assert_eq!(settings.library_location, "L");
        assert_eq!(settings.background_opacity, 5);
        assert_eq!(settings.adaptive_layout, 9);
        assert!(settings.library_preview && settings.library_preview_hide && settings.delete_corrupt);
        assert!(!settings.app_exit);
    }

    #[test]
    fn from_run_startup_maps_the_legacy_range() {
        assert_eq!(StartupMode::from_run_startup(0), StartupMode::None);
        assert_eq!(StartupMode::from_run_startup(1), StartupMode::Normal);
        assert_eq!(StartupMode::from_run_startup(2), StartupMode::Priority);
        assert_eq!(StartupMode::from_run_startup(3), StartupMode::Scheduler);
        for value in 4..=10 {
            assert_eq!(StartupMode::from_run_startup(value), StartupMode::None, "{value}");
        }
    }

    #[test]
    fn the_arguments_of_an_engine_and_a_cef_change_survive_the_wire() {
        let switch = |name: &str, value: Option<&str>| SwitchView {
            name: name.to_owned(),
            value: value.map(str::to_owned),
        };
        let event = PortalEvent::Arguments {
            engine: EngineType::CefSharpLive,
            user: vec![
                switch("disable-gpu-compositing", None),
                switch("autoplay-policy", Some("a")),
            ],
            sucrose: vec![switch("enable-gpu", Some("1"))],
            defaults: vec![switch("no-sandbox", None)],
        };
        let change = PortalRequest::Change(SettingChange::CefArguments(vec![switch("x", Some("y"))]));

        let bytes = postcard::to_stdvec(&event).expect("encode");
        assert_eq!(postcard::from_bytes::<PortalEvent>(&bytes).expect("decode"), event);
        let bytes = postcard::to_stdvec(&change).expect("encode");
        assert_eq!(postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"), change);
    }

    #[test]
    fn a_web_argument_change_leaves_the_settings_view_alone_and_round_trips() {
        let list = vec![SwitchView {
            name: "x".to_owned(),
            value: Some("y".to_owned()),
        }];
        let mut settings = view();
        settings.apply(&SettingChange::WebArguments(list.clone()));
        assert_eq!(settings, view());

        let change = PortalRequest::Change(SettingChange::WebArguments(list));
        let bytes = postcard::to_stdvec(&change).expect("encode");
        assert_eq!(postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"), change);
    }

    #[test]
    fn a_cef_argument_change_leaves_the_settings_view_alone() {
        let mut settings = view();
        settings.apply(&SettingChange::CefArguments(vec![SwitchView {
            name: "x".to_owned(),
            value: None,
        }]));
        assert_eq!(settings, view());
    }

    #[test]
    fn the_largest_arguments_answer_encodes_below_the_frame_limit() {
        let switch = SwitchView {
            name: "n".repeat(MAX_SWITCH_NAME),
            value: Some("v".repeat(MAX_SWITCH_VALUE)),
        };
        let event = PortalEvent::Arguments {
            engine: EngineType::CefSharpLive,
            user: vec![switch.clone(); MAX_SWITCHES],
            sucrose: vec![switch.clone(); 4],
            // The legacy defaults are 35 short switches and one long list.
            defaults: vec![switch; 4],
        };

        let bytes = postcard::to_stdvec(&event).expect("encode");

        // Room is left for the real defaults, about 2 KiB.
        let limit = usize::try_from(MAX_FRAME).expect("the limit fits") - 4096;
        assert!(bytes.len() < limit, "{} bytes", bytes.len());
    }

    #[test]
    fn install_messages_survive_the_wire() {
        let request = PortalRequest::InstallPrerequisite {
            prerequisite: PrerequisiteView::VcRedist,
        };
        let bytes = postcard::to_stdvec(&request).expect("encode");
        assert_eq!(postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"), request);
        let bytes = postcard::to_stdvec(&PortalRequest::CancelInstall).expect("encode");
        assert_eq!(
            postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"),
            PortalRequest::CancelInstall
        );

        for stage in [
            InstallStageView::Downloading {
                received: 1_234_567,
                total: Some(25_000_000),
            },
            InstallStageView::Downloading {
                received: 0,
                total: None,
            },
            InstallStageView::Verifying,
            InstallStageView::Installing,
            InstallStageView::Rechecking,
            InstallStageView::Done,
            InstallStageView::Failed(InstallFailureView::Download),
            InstallStageView::Failed(InstallFailureView::Signature),
            InstallStageView::Failed(InstallFailureView::Installer),
            InstallStageView::Cancelled,
        ] {
            let event = PortalEvent::InstallProgress(stage);
            let bytes = postcard::to_stdvec(&event).expect("encode");
            assert_eq!(postcard::from_bytes::<PortalEvent>(&bytes).expect("decode"), event);
        }
    }

    #[test]
    fn a_portal_request_survives_the_wire() {
        let request = PortalRequest::Change(SettingChange::BackgroundImage("C:\\pics\\a.png".to_owned()));

        let bytes = postcard::to_stdvec(&request).expect("encode");

        assert_eq!(postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"), request);
    }
}
