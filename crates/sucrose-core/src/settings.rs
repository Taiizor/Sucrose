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
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
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
legacy_index!(Stretch, FullScreenAction, PauseKind);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_legacy_defaults() {
        assert_eq!(Stretch::default(), Stretch::UniformToFill);
        assert_eq!(ScreenArea::default(), ScreenArea::DisplayBound);
        assert_eq!(FullScreenAction::default(), FullScreenAction::Resume);
        assert_eq!(PauseKind::default(), PauseKind::Light);
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
