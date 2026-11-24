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

// ScreenArea is left out on purpose: its C# ordinals are unclear.
legacy_index!(
    Stretch,
    FullScreenAction,
    PauseKind,
    GpuPreference,
    SortMode,
    SortKind,
    LegacyThemeType,
    LegacyBackdropType
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
}
