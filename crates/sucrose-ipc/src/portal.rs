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
use sucrose_core::settings::{Backdrop, SortKind, SortMode, Stretch, ThemeMode};

/// Version of the Portal protocol, sent in `PortalHello` so the service can refuse a Portal it cannot talk to.
pub const PORTAL_PROTOCOL_VERSION: u16 = 1;

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
    /// The theme is not a video, and this build plays only videos.
    NotVideo,
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
        }
    }

    fn runtime() -> RuntimeState {
        RuntimeState {
            selected: None,
            running: false,
            rule: RuleState::None,
            library_moving: false,
            tray_visible: false,
        }
    }

    #[test]
    fn the_portal_protocol_starts_at_version_one() {
        assert_eq!(PORTAL_PROTOCOL_VERSION, 1);
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
        ];
        for (expected, event) in events.iter().enumerate() {
            assert_eq!(usize::from(index(event)), expected, "{event:?}");
        }
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
            [PortalPage::Library, PortalPage::General, PortalPage::Personal].map(|page| index(&page)),
            [0, 1, 2]
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
            UseRefusal::NotVideo,
            UseRefusal::NewerVersion,
            UseRefusal::RulePaused,
            UseRefusal::RuleClosed,
            UseRefusal::Moving,
            UseRefusal::Unavailable,
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
        let cases: [(SettingChange, PortalSettings); 6] = [
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
    fn a_portal_request_survives_the_wire() {
        let request = PortalRequest::Change(SettingChange::BackgroundImage("C:\\pics\\a.png".to_owned()));

        let bytes = postcard::to_stdvec(&request).expect("encode");

        assert_eq!(postcard::from_bytes::<PortalRequest>(&bytes).expect("decode"), request);
    }
}
