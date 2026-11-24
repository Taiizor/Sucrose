//! The General page (UI-SPEC E7, E8, E9): theme mode, notification area icon, window backdrop with its background image,
//! and the library location. Every control sends a SettingChange and shows only what the service broadcast.

use std::path::Path;

use gpui_kit::*;
use sucrose_core::settings::Backdrop;

use crate::background::BackgroundError;

/// Why a backdrop cannot be chosen on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackdropReason {
    /// Windows before 11 (22H2) cannot show Mica, Acrylic or Tabbed.
    RequiresWindows11,
    /// Another operating system has no such material; carries its display name.
    UnavailableOn(String),
}

/// One option of the Window Backdrop select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackdropOption {
    /// The backdrop the option stands for.
    pub value: Backdrop,
    /// Why it cannot be chosen, if it cannot.
    pub reason: Option<BackdropReason>,
}

/// The options of Window Backdrop for a machine that can (`supported`) or cannot show the materials, running `os` (the
/// value of `std::env::consts::OS`).
pub fn backdrop_options(_supported: bool, _os: &str) -> Vec<BackdropOption> {
    Backdrop::ALL
        .iter()
        .map(|value| BackdropOption {
            value: *value,
            reason: None,
        })
        .collect()
}

/// The text that says why `reason` blocks a backdrop.
pub fn reason_text(_reason: &BackdropReason) -> SharedString {
    SharedString::default()
}

/// Whether the file at `path` may be offered as the background image by its extension (the legacy file filter).
pub fn image_choice(_path: &Path) -> Result<(), BackgroundError> {
    Ok(())
}

/// `path` cut to `max_chars` characters, keeping its end (the file name is what tells paths apart).
pub fn truncate_path(path: &str, _max_chars: usize) -> String {
    path.to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sucrose_core::settings::Backdrop;

    use super::{BackdropReason, backdrop_options, image_choice, reason_text, truncate_path};
    use crate::background::BackgroundError;

    fn disabled(options: &[super::BackdropOption]) -> Vec<Backdrop> {
        options.iter().filter(|o| o.reason.is_some()).map(|o| o.value).collect()
    }

    #[test]
    fn settings_backdrop_options_list_the_four_materials_in_order() {
        let values: Vec<Backdrop> = backdrop_options(true, "windows").iter().map(|o| o.value).collect();
        assert_eq!(
            values,
            [Backdrop::None, Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed]
        );
        assert!(disabled(&backdrop_options(true, "windows")).is_empty());
    }

    #[test]
    fn settings_backdrop_options_before_windows_11_disable_everything_but_none() {
        let options = backdrop_options(false, "windows");
        assert_eq!(
            disabled(&options),
            [Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed]
        );
        assert_eq!(options[0].reason, None);
        for option in &options[1..] {
            assert_eq!(option.reason, Some(BackdropReason::RequiresWindows11));
        }
        assert_eq!(&*reason_text(&BackdropReason::RequiresWindows11), "Requires Windows 11");
    }

    #[test]
    fn settings_backdrop_options_on_other_systems_name_the_system() {
        for (os, name) in [("macos", "macOS"), ("linux", "Linux")] {
            let options = backdrop_options(false, os);
            assert_eq!(
                disabled(&options),
                [Backdrop::Mica, Backdrop::Acrylic, Backdrop::Tabbed],
                "{os}"
            );
            let reason = options[1].reason.clone().expect("a reason");
            assert_eq!(reason, BackdropReason::UnavailableOn(name.to_owned()));
            assert_eq!(&*reason_text(&reason), format!("Not available on {name}"));
        }
    }

    #[test]
    fn settings_image_choice_accepts_the_legacy_extensions_in_any_case() {
        for name in ["a.png", "a.JPG", "b.jpeg", "c.TIFF", "d.webp", "e.GiF"] {
            assert_eq!(image_choice(Path::new(name)), Ok(()), "{name}");
        }
    }

    #[test]
    fn settings_image_choice_refuses_other_extensions() {
        for name in ["a.bmp", "a.svg", "a", "a.png.exe", ".png"] {
            assert_eq!(
                image_choice(Path::new(name)),
                Err(BackgroundError::Unsupported),
                "{name}"
            );
        }
    }

    #[test]
    fn settings_path_keeps_the_end_of_a_long_path() {
        let path = "C:\\Users\\Someone\\Pictures\\Wallpapers\\mountains.png";
        let cut = truncate_path(path, 24);
        assert_eq!(cut.chars().count(), 24);
        assert!(cut.starts_with('\u{2026}'));
        assert!(cut.ends_with("Wallpapers\\mountains.png"));
    }

    #[test]
    fn settings_path_that_fits_is_left_alone_and_the_cut_counts_characters() {
        assert_eq!(truncate_path("C:\\a.png", 24), "C:\\a.png");
        let wide = "é".repeat(30);
        let cut = truncate_path(&wide, 10);
        assert_eq!(cut.chars().count(), 10);
    }
}
