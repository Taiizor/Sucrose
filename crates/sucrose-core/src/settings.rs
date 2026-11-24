//! Domain enums of the legacy settings, with the names and ordinals the C# app uses (D-007).
//!
//! The C# app stores enums as name strings (`Enum.Parse` also accepts numeric strings). Names are exact and
//! case-sensitive. Ordinals are only provided where `docs/legacy/formats.md` documents the declaration order.

macro_rules! legacy_enum {
    (
        $(#[$meta:meta])*
        $name:ident, default $default:ident {
            $($(#[$vmeta:meta])* $variant:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, ::serde::Serialize, ::serde::Deserialize)]
        pub enum $name {
            $($(#[$vmeta])* $variant),+
        }

        impl $name {
            /// Every value, in the legacy declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// The value the C# app stores for this one.
            pub fn legacy_name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),+
                }
            }

            /// Value for a legacy name: exact and case-sensitive, like `Enum.Parse`.
            pub fn from_legacy_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|value| value.legacy_name() == name)
            }
        }
    };
}

legacy_enum! {
    /// How a wallpaper is fitted to the screen (`StretchType`); the legacy default is `UniformToFill`.
    Stretch, default UniformToFill {
        /// Original size, no scaling.
        None,
        /// Fill the screen, ignoring the aspect ratio.
        Fill,
        /// Fit inside the screen, keeping the aspect ratio.
        Uniform,
        /// Cover the screen, keeping the aspect ratio and cropping the overflow.
        #[default]
        UniformToFill,
    }
}

legacy_enum! {
    /// Which part of a monitor the wallpaper covers (`ScreenType`); the legacy default is `DisplayBound`.
    ///
    /// The C# enum comes from an external library and its ordinals are unclear, so only names are read.
    ScreenArea, default DisplayBound {
        /// The whole monitor, including the taskbar area.
        #[default]
        DisplayBound,
        /// The monitor minus the taskbar.
        WorkingArea,
    }
}

legacy_enum! {
    /// What a wallpaper does while a rule applies (`PerformanceType`); the legacy default for a fullscreen app is
    /// `Resume`.
    FullScreenAction, default Resume {
        /// Stop the wallpaper.
        Close,
        /// Pause the wallpaper.
        Pause,
        /// Keep playing.
        #[default]
        Resume,
    }
}

legacy_enum! {
    /// How a paused wallpaper is paused (`PausePerformanceType`); the legacy default is `Light`.
    PauseKind, default Light {
        /// Free the engine's resources while paused.
        Heavy,
        /// Keep the engine loaded and only stop playback.
        #[default]
        Light,
    }
}

legacy_enum! {
    /// Which GPU Windows gives Sucrose's Direct3D devices (`HighPerformanceType`, Engine.json `GraphicPreference`);
    /// the legacy default is `HighPerformance`.
    GpuPreference, default HighPerformance {
        /// Let Windows decide.
        Default,
        /// The power saving GPU (usually the integrated one).
        PowerSaving,
        /// The high performance GPU (usually the discrete one).
        #[default]
        HighPerformance,
    }
}

impl GpuPreference {
    /// The number the C# app writes as `GpuPreference=<n>` in the per-app graphics settings of Windows: the enum
    /// ordinal (Default 0, PowerSaving 1, HighPerformance 2).
    #[must_use]
    pub fn legacy_ordinal(self) -> u32 {
        match self {
            Self::Default => 0,
            Self::PowerSaving => 1,
            Self::HighPerformance => 2,
        }
    }
}

legacy_enum! {
    /// What the library is sorted by (`SortModeType`, `Portal.json:LibrarySortMode`); the legacy default is `Creation`.
    SortMode, default Creation {
        /// The localized title.
        Name,
        /// Creation time of the theme folder.
        #[default]
        Creation,
        /// Last write of `SucroseInfo.json`.
        Modification,
    }
}

legacy_enum! {
    /// Direction of the library sort (`SortKindType`, `Portal.json:LibrarySortKind`); the legacy default is
    /// `Descending`.
    SortKind, default Descending {
        /// Smallest first.
        Ascending,
        /// Largest first.
        #[default]
        Descending,
    }
}

legacy_enum! {
    /// The portal's light or dark look (`Portal.ThemeMode`, D-21); a fresh install follows the system.
    ///
    /// The C# app stored only Dark or Light (`ThemeType`); `System` is new.
    ThemeMode, default System {
        /// Always light.
        Light,
        /// Always dark.
        Dark,
        /// Follow the operating system.
        #[default]
        System,
    }
}

legacy_enum! {
    /// The material behind the portal window (`Portal.Backdrop`, D-21); the legacy default is no backdrop.
    ///
    /// Not the C# `WindowBackdropType`: its `Auto` is split into theme mode `System` and backdrop `Mica`.
    Backdrop, default None {
        /// A plain window background.
        #[default]
        None,
        /// The Windows 11 Mica material.
        Mica,
        /// The Windows acrylic blur.
        Acrylic,
        /// The Windows 11 tabbed (Mica Alt) material.
        Tabbed,
    }
}

legacy_enum! {
    /// The C# `WindowsThemeType` stored as `General.json:ThemeType`; only read to migrate to [`ThemeMode`].
    LegacyThemeType, default Dark {
        /// A dark OS theme.
        #[default]
        Dark,
        /// A light OS theme.
        Light,
    }
}

legacy_enum! {
    /// The WPF-UI `WindowBackdropType` stored as `Portal.json:BackdropType`; only read to migrate to [`Backdrop`]
    /// (and, for `Auto`, [`ThemeMode`]).
    LegacyBackdropType, default None {
        /// No backdrop (the legacy default).
        #[default]
        None,
        /// Follow the system: becomes theme mode `System` with the `Mica` backdrop.
        Auto,
        /// Mica.
        Mica,
        /// Acrylic.
        Acrylic,
        /// Tabbed (Mica Alt).
        Tabbed,
    }
}

legacy_enum! {
    /// How wallpapers are spread over several monitors (Skylark `DisplayScreenType`, Engine.json
    /// `DisplayScreenType`); the legacy default is `PerDisplay`.
    ///
    /// The C# enum comes from an external library and its ordinals are unclear, so only names are read.
    DisplayScreenType, default PerDisplay {
        /// Each monitor plays its own wallpaper.
        #[default]
        PerDisplay,
        /// Every monitor plays the same wallpaper.
        SameDuplicate,
        /// One wallpaper is stretched across all monitors.
        SpanAcross,
    }
}

legacy_enum! {
    /// How a duplicated or expanded screen layout is handled (Skylark screen mode, Engine.json
    /// `DuplicateScreenType` and `ExpandScreenType`); the legacy default is `Default`.
    ///
    /// The C# enum comes from an external library and its ordinals are unclear, so only names are read.
    ScreenModeType, default Default {
        /// The standard handling.
        #[default]
        Default,
        /// The advanced handling.
        Advanced,
    }
}

legacy_enum! {
    /// Which input is forwarded to a web wallpaper (Skylark `InputType`, Engine.json `InputType`); the legacy
    /// default is `MouseKeyboard`.
    ///
    /// The C# enum comes from an external library and its ordinals are unclear, so only names are read.
    InputType, default MouseKeyboard {
        /// No input.
        Close,
        /// Mouse input only.
        OnlyMouse,
        /// Keyboard input only.
        OnlyKeyboard,
        /// Mouse and keyboard input.
        #[default]
        MouseKeyboard,
    }
}

legacy_enum! {
    /// How input is read (`InputModuleType`, Engine.json `InputModuleType`); the only value is `RawInput`.
    InputModuleType, default RawInput {
        /// Windows raw input.
        #[default]
        RawInput,
    }
}

legacy_enum! {
    /// The engine that plays a wallpaper (`EngineType`, Engine.json `Gif`, `Url`, `Web`, `Video`, `YouTube`
    /// and `Application`).
    ///
    /// Every wallpaper type has its own default and its own allowed subset (see [`EngineType::GIF`] and the
    /// other constants); the value here is only the enum default.
    EngineType, default MpvPlayerLive {
        /// The application engine.
        AuroraLive,
        /// The video engine of the legacy app.
        NebulaLive,
        /// A gif engine.
        VexanaLive,
        /// A gif engine.
        XavierLive,
        /// The WebView2 engine.
        WebViewLive,
        /// The CefSharp engine.
        CefSharpLive,
        /// The mpv player engine.
        #[default]
        MpvPlayerLive,
        /// The VLC player engine.
        VlcPlayerLive,
    }
}

impl EngineType {
    /// The engines a gif wallpaper may use (`GifEngineType`).
    pub const GIF: &'static [Self] = &[
        Self::VexanaLive,
        Self::XavierLive,
        Self::WebViewLive,
        Self::CefSharpLive,
        Self::MpvPlayerLive,
        Self::VlcPlayerLive,
    ];
    /// The engines a url wallpaper may use (`UrlEngineType`).
    pub const URL: &'static [Self] = &[Self::WebViewLive, Self::CefSharpLive];
    /// The engines a web wallpaper may use (`WebEngineType`).
    pub const WEB: &'static [Self] = &[Self::WebViewLive, Self::CefSharpLive];
    /// The engines a video wallpaper may use (`VideoEngineType`).
    pub const VIDEO: &'static [Self] = &[
        Self::NebulaLive,
        Self::WebViewLive,
        Self::CefSharpLive,
        Self::MpvPlayerLive,
        Self::VlcPlayerLive,
    ];
    /// The engines a YouTube wallpaper may use (`YouTubeEngineType`).
    pub const YOUTUBE: &'static [Self] = &[Self::WebViewLive, Self::CefSharpLive];
    /// The engines an application wallpaper may use (`ApplicationEngineType`).
    pub const APPLICATION: &'static [Self] = &[Self::AuroraLive];
}

legacy_enum! {
    /// The order in which cycling picks the next wallpaper (`TransitionCycleType`); the legacy default is `Random`.
    TransitionCycleType, default Random {
        /// A random theme.
        #[default]
        Random,
        /// The next theme in library order.
        Sequential,
    }
}

legacy_enum! {
    /// How an update is downloaded (`UpdateModuleType`); the legacy default is `Downloader`.
    UpdateModuleType, default Downloader {
        /// The built-in downloader of the legacy app.
        Native,
        /// The separate downloader.
        #[default]
        Downloader,
    }
}

legacy_enum! {
    /// Where updates come from (`UpdateServerType`); the legacy default is `Soferity`.
    UpdateServerType, default Soferity {
        /// GitHub releases.
        GitHub,
        /// The Soferity server.
        #[default]
        Soferity,
    }
}

legacy_enum! {
    /// How visible an automatic update is (`UpdateAutoType`); the legacy default is `SemiSilent`.
    UpdateAutoType, default SemiSilent {
        /// Fully visible.
        Visible,
        /// Visible only to confirm.
        #[default]
        SemiSilent,
        /// Silent update, visible result.
        UpdateSilent,
        /// Silent from start to end.
        CompleteSilent,
    }
}

legacy_enum! {
    /// The package kind an update downloads (`UpdateExtensionType`); the legacy default is `Executable`.
    UpdateExtensionType, default Executable {
        /// A compressed archive.
        Compressed,
        /// An installer executable.
        #[default]
        Executable,
    }
}

legacy_enum! {
    /// The release channel updates come from (`UpdateChannelType`); the legacy default is `Release`.
    UpdateChannelType, default Release {
        /// Stable releases.
        #[default]
        Release,
        /// Pre-releases too.
        PreRelease,
    }
}

legacy_enum! {
    /// Which monitors a fullscreen app affects (new, `Backgroundog.FullScreenScope`, Phase 2 D-05); the default is
    /// `AllMonitors`.
    FullScreenScope, default AllMonitors {
        /// A fullscreen app on any monitor applies the rule to every wallpaper.
        #[default]
        AllMonitors,
        /// Only the wallpaper on the monitor of the fullscreen app is affected.
        AffectedMonitor,
    }
}

/// What a wallpaper does while a performance rule applies: the legacy `PerformanceType` of every `*Performance`
/// key, the same enum as [`FullScreenAction`].
pub type PerformanceType = FullScreenAction;

macro_rules! legacy_index {
    ($($name:ident),+) => {
        $(
            impl $name {
                /// Value for a legacy ordinal (0-based declaration order), or `None` when out of range.
                pub fn from_legacy_index(index: i64) -> Option<Self> {
                    usize::try_from(index).ok().and_then(|index| Self::ALL.get(index).copied())
                }
            }
        )+
    };
}

// ScreenArea, DisplayScreenType, ScreenModeType and InputType are left out on purpose: their C# ordinals are unclear.
// FullScreenScope is new and has no legacy ordinal.
legacy_index!(
    Stretch,
    FullScreenAction,
    PauseKind,
    GpuPreference,
    SortMode,
    SortKind,
    LegacyThemeType,
    LegacyBackdropType,
    InputModuleType,
    EngineType,
    TransitionCycleType,
    UpdateModuleType,
    UpdateServerType,
    UpdateAutoType,
    UpdateExtensionType,
    UpdateChannelType
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_legacy_defaults() {
        assert_eq!(Stretch::default(), Stretch::UniformToFill);
        assert_eq!(ScreenArea::default(), ScreenArea::DisplayBound);
        assert_eq!(FullScreenAction::default(), FullScreenAction::Resume);
        assert_eq!(PauseKind::default(), PauseKind::Light);
        assert_eq!(GpuPreference::default(), GpuPreference::HighPerformance);
        assert_eq!(SortMode::default(), SortMode::Creation);
        assert_eq!(SortKind::default(), SortKind::Descending);
        assert_eq!(ThemeMode::default(), ThemeMode::System);
        assert_eq!(Backdrop::default(), Backdrop::None);
    }

    #[test]
    fn theme_enums_use_the_legacy_names_and_declaration_order() {
        assert_eq!(ThemeMode::from_legacy_name("Dark"), Some(ThemeMode::Dark));
        assert_eq!(ThemeMode::from_legacy_name("dark"), None);
        assert_eq!(Backdrop::from_legacy_name("Tabbed"), Some(Backdrop::Tabbed));
        assert_eq!(Backdrop::from_legacy_name("Auto"), None);
        assert_eq!(LegacyThemeType::from_legacy_index(1), Some(LegacyThemeType::Light));
        assert_eq!(LegacyThemeType::from_legacy_index(2), None);
        assert_eq!(
            LegacyBackdropType::from_legacy_name("Auto"),
            Some(LegacyBackdropType::Auto)
        );
        assert_eq!(LegacyBackdropType::from_legacy_name("auto"), None);
        assert_eq!(
            LegacyBackdropType::from_legacy_index(4),
            Some(LegacyBackdropType::Tabbed)
        );
        assert_eq!(LegacyBackdropType::from_legacy_index(5), None);
        assert_eq!(ThemeMode::ALL, &[ThemeMode::Light, ThemeMode::Dark, ThemeMode::System]);
    }

    #[test]
    fn sort_enums_round_trip_their_legacy_names_and_follow_the_declaration_order() {
        for value in SortMode::ALL {
            assert_eq!(SortMode::from_legacy_name(value.legacy_name()), Some(*value));
        }
        for value in SortKind::ALL {
            assert_eq!(SortKind::from_legacy_name(value.legacy_name()), Some(*value));
        }
        assert_eq!(SortMode::from_legacy_name("name"), None);
        assert_eq!(SortMode::from_legacy_index(2), Some(SortMode::Modification));
        assert_eq!(SortMode::from_legacy_index(3), None);
        assert_eq!(SortKind::from_legacy_index(0), Some(SortKind::Ascending));
        assert_eq!(SortKind::from_legacy_index(2), None);
    }

    #[test]
    fn gpu_preference_names_indexes_and_ordinals_match_the_legacy_enum() {
        assert_eq!(
            GpuPreference::from_legacy_name("PowerSaving"),
            Some(GpuPreference::PowerSaving)
        );
        assert_eq!(GpuPreference::from_legacy_name("powersaving"), None);
        assert_eq!(
            GpuPreference::from_legacy_index(2),
            Some(GpuPreference::HighPerformance)
        );
        assert_eq!(GpuPreference::from_legacy_index(3), None);
        assert_eq!(GpuPreference::Default.legacy_ordinal(), 0);
        assert_eq!(GpuPreference::PowerSaving.legacy_ordinal(), 1);
        assert_eq!(GpuPreference::HighPerformance.legacy_ordinal(), 2);
        for (index, value) in GpuPreference::ALL.iter().enumerate() {
            assert_eq!(u32::try_from(index).ok(), Some(value.legacy_ordinal()));
        }
    }

    #[test]
    fn names_are_exact_and_case_sensitive() {
        assert_eq!(Stretch::from_legacy_name("UniformToFill"), Some(Stretch::UniformToFill));
        assert_eq!(Stretch::from_legacy_name("uniformtofill"), None);
        assert_eq!(
            ScreenArea::from_legacy_name("WorkingArea"),
            Some(ScreenArea::WorkingArea)
        );
        assert_eq!(
            FullScreenAction::from_legacy_name("Pause"),
            Some(FullScreenAction::Pause)
        );
        assert_eq!(FullScreenAction::from_legacy_name("pause"), None);
        assert_eq!(PauseKind::from_legacy_name("Heavy"), Some(PauseKind::Heavy));
        assert_eq!(PauseKind::from_legacy_name(""), None);
    }

    #[test]
    fn every_value_round_trips_through_its_name() {
        for value in Stretch::ALL {
            assert_eq!(Stretch::from_legacy_name(value.legacy_name()), Some(*value));
        }
        for value in ScreenArea::ALL {
            assert_eq!(ScreenArea::from_legacy_name(value.legacy_name()), Some(*value));
        }
        for value in FullScreenAction::ALL {
            assert_eq!(FullScreenAction::from_legacy_name(value.legacy_name()), Some(*value));
        }
        for value in PauseKind::ALL {
            assert_eq!(PauseKind::from_legacy_name(value.legacy_name()), Some(*value));
        }
    }

    #[test]
    fn ordinals_follow_the_documented_declaration_order() {
        assert_eq!(Stretch::from_legacy_index(0), Some(Stretch::None));
        assert_eq!(Stretch::from_legacy_index(3), Some(Stretch::UniformToFill));
        assert_eq!(FullScreenAction::from_legacy_index(0), Some(FullScreenAction::Close));
        assert_eq!(FullScreenAction::from_legacy_index(1), Some(FullScreenAction::Pause));
        assert_eq!(FullScreenAction::from_legacy_index(2), Some(FullScreenAction::Resume));
        assert_eq!(PauseKind::from_legacy_index(0), Some(PauseKind::Heavy));
        assert_eq!(PauseKind::from_legacy_index(1), Some(PauseKind::Light));
    }

    #[test]
    fn ordinals_outside_the_range_are_rejected() {
        assert_eq!(Stretch::from_legacy_index(4), None);
        assert_eq!(Stretch::from_legacy_index(-1), None);
        assert_eq!(PauseKind::from_legacy_index(2), None);
    }

    #[test]
    fn the_engine_and_update_enums_follow_the_declaration_order_of_the_legacy_enums() {
        assert_eq!(EngineType::from_legacy_index(0), Some(EngineType::AuroraLive));
        assert_eq!(EngineType::from_legacy_index(7), Some(EngineType::VlcPlayerLive));
        assert_eq!(EngineType::from_legacy_index(8), None);
        assert_eq!(EngineType::from_legacy_name("cefsharplive"), None);
        assert_eq!(
            EngineType::from_legacy_name("CefSharpLive"),
            Some(EngineType::CefSharpLive)
        );
        assert_eq!(
            TransitionCycleType::from_legacy_index(1),
            Some(TransitionCycleType::Sequential)
        );
        assert_eq!(UpdateModuleType::from_legacy_index(0), Some(UpdateModuleType::Native));
        assert_eq!(
            UpdateServerType::from_legacy_name("GitHub"),
            Some(UpdateServerType::GitHub)
        );
        assert_eq!(
            UpdateAutoType::from_legacy_index(3),
            Some(UpdateAutoType::CompleteSilent)
        );
        assert_eq!(UpdateAutoType::from_legacy_index(4), None);
        assert_eq!(
            UpdateExtensionType::from_legacy_name("Compressed"),
            Some(UpdateExtensionType::Compressed)
        );
        assert_eq!(
            UpdateChannelType::from_legacy_name("PreRelease"),
            Some(UpdateChannelType::PreRelease)
        );
        assert_eq!(InputModuleType::from_legacy_index(0), Some(InputModuleType::RawInput));
        assert_eq!(InputModuleType::from_legacy_index(1), None);
    }

    #[test]
    fn the_skylark_enums_are_read_by_name_only() {
        assert_eq!(
            InputType::from_legacy_name("OnlyKeyboard"),
            Some(InputType::OnlyKeyboard)
        );
        assert_eq!(InputType::from_legacy_name("onlykeyboard"), None);
        assert_eq!(
            DisplayScreenType::from_legacy_name("SpanAcross"),
            Some(DisplayScreenType::SpanAcross)
        );
        assert_eq!(
            ScreenModeType::from_legacy_name("Advanced"),
            Some(ScreenModeType::Advanced)
        );
        assert_eq!(InputType::default(), InputType::MouseKeyboard);
        assert_eq!(DisplayScreenType::default(), DisplayScreenType::PerDisplay);
        assert_eq!(ScreenModeType::default(), ScreenModeType::Default);
        assert_eq!(FullScreenScope::default(), FullScreenScope::AllMonitors);
    }

    #[test]
    fn every_wallpaper_type_allows_only_its_engines() {
        assert!(EngineType::VIDEO.contains(&EngineType::NebulaLive));
        assert!(!EngineType::VIDEO.contains(&EngineType::AuroraLive));
        assert!(!EngineType::GIF.contains(&EngineType::NebulaLive));
        assert_eq!(EngineType::URL, EngineType::WEB);
        assert_eq!(EngineType::URL, EngineType::YOUTUBE);
        assert_eq!(EngineType::APPLICATION, &[EngineType::AuroraLive]);
        let performance: PerformanceType = PerformanceType::Pause;
        assert_eq!(performance, FullScreenAction::Pause);
    }
}
