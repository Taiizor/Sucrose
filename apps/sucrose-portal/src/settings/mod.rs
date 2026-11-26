//! The settings mode: the General and Personal pages, the mode the Portal window is in and the shared column layout.

pub mod arguments;
pub mod general;
pub mod other;
pub mod personal;
pub mod row;
pub mod wallpaper;

use gpui_kit::component::{StyledExt as _, v_flex};
use gpui_kit::*;
use sucrose_ipc::portal::PortalPage;

/// Which settings page is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    /// The General page.
    General,
    /// The Personal page.
    Personal,
    /// The Wallpaper page.
    Wallpaper,
    /// The Other page.
    Other,
}

impl SettingsTab {
    /// Every tab in the order of the header (the legacy order: General, Personal, Wallpaper, Other; Performance follows
    /// in a later plan).
    pub const ALL: [Self; 4] = [Self::General, Self::Personal, Self::Wallpaper, Self::Other];

    /// The position of the tab in the header.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|tab| *tab == self).unwrap_or_default()
    }

    /// The tab at `index` in the header; the first one for an index past the last.
    pub fn at(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(Self::ALL[0])
    }
}

/// What the Portal window shows: the library, or one of the settings pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The theme library.
    Library,
    /// A settings page.
    Settings(SettingsTab),
}

impl Mode {
    /// The mode that shows `page` (a page asked for on the command line or by Focus).
    pub fn from_page(page: PortalPage) -> Self {
        match page {
            PortalPage::Library => Self::Library,
            PortalPage::General => Self::Settings(SettingsTab::General),
            PortalPage::Personal => Self::Settings(SettingsTab::Personal),
            PortalPage::Wallpaper => Self::Settings(SettingsTab::Wallpaper),
            PortalPage::Other => Self::Settings(SettingsTab::Other),
        }
    }

    /// Whether a settings page is shown.
    pub fn is_settings(self) -> bool {
        matches!(self, Self::Settings(_))
    }

    /// The mode the gear leads to: General, or the page already shown when in settings.
    pub fn enter_settings(self) -> Self {
        match self {
            Self::Library => Self::Settings(SettingsTab::General),
            settings @ Self::Settings(_) => settings,
        }
    }

    /// The mode Back (or Escape) leads to: the library.
    pub fn back(self) -> Self {
        Self::Library
    }
}

/// The widest the settings column grows (UI-SPEC: settings content column).
pub const COLUMN_MAX_WIDTH: f32 = 960.0;

/// The name of the place deleted themes go to, for the texts that say so (D-09).
pub fn bin_name() -> &'static str {
    if cfg!(windows) { "Recycle Bin" } else { "Trash" }
}

/// A group of rows under its heading: 8 px between the rows (UI-SPEC: sm).
pub fn group(title: SharedString, rows: Vec<AnyElement>) -> AnyElement {
    v_flex()
        .w_full()
        .gap_2()
        .child(div().text_size(px(16.)).font_semibold().child(title))
        .children(rows)
        .into_any_element()
}

/// The scroll column of a settings page: centered, at most [`COLUMN_MAX_WIDTH`] wide, 16 px padding, 24 px between groups.
pub fn column(id: &'static str, groups: Vec<AnyElement>) -> AnyElement {
    div()
        .id(id)
        .size_full()
        .overflow_y_scroll()
        .child(
            v_flex()
                .w_full()
                .max_w(px(COLUMN_MAX_WIDTH))
                .mx_auto()
                .p_4()
                .gap(px(24.))
                .children(groups),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::PortalPage;

    use super::{Mode, SettingsTab};

    #[test]
    fn settings_tabs_follow_the_legacy_order_with_wallpaper_after_personal_and_other_last() {
        assert_eq!(
            SettingsTab::ALL,
            [
                SettingsTab::General,
                SettingsTab::Personal,
                SettingsTab::Wallpaper,
                SettingsTab::Other
            ]
        );
        for (index, tab) in SettingsTab::ALL.into_iter().enumerate() {
            assert_eq!(tab.index(), index);
            assert_eq!(SettingsTab::at(index), tab);
        }
        assert_eq!(SettingsTab::at(9), SettingsTab::General);
    }

    #[test]
    fn settings_wallpaper_page_is_asked_for_like_the_others() {
        assert_eq!(
            Mode::from_page(PortalPage::Wallpaper),
            Mode::Settings(SettingsTab::Wallpaper)
        );
    }

    #[test]
    fn settings_other_page_is_asked_for_like_the_others() {
        assert_eq!(Mode::from_page(PortalPage::Other), Mode::Settings(SettingsTab::Other));
    }

    #[test]
    fn settings_mode_follows_the_page_that_was_asked_for() {
        assert_eq!(Mode::from_page(PortalPage::Library), Mode::Library);
        assert_eq!(
            Mode::from_page(PortalPage::General),
            Mode::Settings(SettingsTab::General)
        );
        assert_eq!(
            Mode::from_page(PortalPage::Personal),
            Mode::Settings(SettingsTab::Personal)
        );
    }

    #[test]
    fn settings_gear_enters_general_from_the_library_and_keeps_the_page_when_already_in_settings() {
        assert_eq!(Mode::Library.enter_settings(), Mode::Settings(SettingsTab::General));
        assert_eq!(
            Mode::Settings(SettingsTab::Personal).enter_settings(),
            Mode::Settings(SettingsTab::Personal)
        );
    }

    #[test]
    fn settings_column_is_960_wide_at_most() {
        assert!((super::COLUMN_MAX_WIDTH - 960.0).abs() < f32::EPSILON);
    }

    #[test]
    fn settings_bin_is_named_for_the_os() {
        let bin = super::bin_name();
        assert!(bin == "Recycle Bin" || bin == "Trash");
    }

    #[test]
    fn settings_back_returns_to_the_library_from_every_page() {
        assert_eq!(Mode::Settings(SettingsTab::General).back(), Mode::Library);
        assert_eq!(Mode::Settings(SettingsTab::Personal).back(), Mode::Library);
        assert_eq!(Mode::Library.back(), Mode::Library);
        assert!(Mode::Settings(SettingsTab::General).is_settings());
        assert!(!Mode::Library.is_settings());
    }
}
