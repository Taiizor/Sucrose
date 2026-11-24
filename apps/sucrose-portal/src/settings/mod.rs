//! The settings mode: the General and Personal pages, the mode the Portal window is in and the shared column layout.

pub mod row;

use sucrose_ipc::portal::PortalPage;

/// Which settings page is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// The General page.
    General,
    /// The Personal page.
    Personal,
}

/// What the Portal window shows: the library, or one of the settings pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The theme library.
    Library,
    /// A settings page.
    Settings(Tab),
}

impl Mode {
    /// The mode that shows `page` (a page asked for on the command line or by Focus).
    pub fn from_page(_page: PortalPage) -> Self {
        Self::Library
    }

    /// The page of the protocol this mode shows.
    pub fn page(self) -> PortalPage {
        PortalPage::Library
    }

    /// Whether a settings page is shown.
    pub fn is_settings(self) -> bool {
        false
    }

    /// The mode the gear leads to: General.
    pub fn enter_settings(self) -> Self {
        self
    }

    /// The mode Back (or Escape) leads to: the library.
    pub fn back(self) -> Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use sucrose_ipc::portal::PortalPage;

    use super::{Mode, Tab};

    #[test]
    fn settings_mode_follows_the_page_that_was_asked_for() {
        assert_eq!(Mode::from_page(PortalPage::Library), Mode::Library);
        assert_eq!(Mode::from_page(PortalPage::General), Mode::Settings(Tab::General));
        assert_eq!(Mode::from_page(PortalPage::Personal), Mode::Settings(Tab::Personal));
    }

    #[test]
    fn settings_mode_names_the_page_it_shows() {
        for page in [PortalPage::Library, PortalPage::General, PortalPage::Personal] {
            assert_eq!(Mode::from_page(page).page(), page);
        }
    }

    #[test]
    fn settings_gear_enters_general_from_the_library_and_keeps_the_page_when_already_in_settings() {
        assert_eq!(Mode::Library.enter_settings(), Mode::Settings(Tab::General));
        assert_eq!(
            Mode::Settings(Tab::Personal).enter_settings(),
            Mode::Settings(Tab::Personal)
        );
    }

    #[test]
    fn settings_back_returns_to_the_library_from_every_page() {
        assert_eq!(Mode::Settings(Tab::General).back(), Mode::Library);
        assert_eq!(Mode::Settings(Tab::Personal).back(), Mode::Library);
        assert_eq!(Mode::Library.back(), Mode::Library);
        assert!(Mode::Settings(Tab::General).is_settings());
        assert!(!Mode::Library.is_settings());
    }
}
