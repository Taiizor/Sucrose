//! When the service starts a Portal and when it ends by itself (D-12, D-13). Both rules are pure functions of facts the
//! service already holds, so they are tested without a process.

/// What the service does at start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartActions {
    /// The selected theme exists, so the wallpaper starts (when its gate accepts it).
    pub start_wallpaper: bool,
    /// The service opens the Portal.
    pub open_portal: bool,
}

/// The legacy Launcher rule (D-12): a selected theme that exists starts the wallpaper, and also opens the Portal when
/// the tray is hidden (the user would have no other way in); with nothing selected the Portal opens. A service started
/// with `--no-portal` (`open_portal_flag` false) never opens one.
pub fn start_actions(selected_exists: bool, tray_visible: bool, open_portal_flag: bool) -> StartActions {
    StartActions {
        start_wallpaper: selected_exists,
        open_portal: open_portal_flag && (!selected_exists || !tray_visible),
    }
}

/// The service exit rule (D-13): with the tray hidden, no wallpaper wanted and no Portal connected nothing needs the
/// service; with the tray visible it stays until Exit.
pub fn should_exit(tray_visible: bool, wallpaper_wanted: bool, portal_connected: bool) -> bool {
    !tray_visible && !wallpaper_wanted && !portal_connected
}

/// The Open item of the tray menu. English until the service has a language of its own (UI-06, Phase 5).
pub const OPEN_LABEL: &str = "Open Sucrose";

/// The exit item with AppExit on: it closes every Sucrose application, not only the tray.
pub const CLOSE_APPLICATIONS_LABEL: &str = "Close Applications";

/// The exit item with AppExit off: it removes only the tray.
pub const EXIT_LABEL: &str = "Exit";

/// What the exit item of the tray menu says for the current AppExit value (D-14).
pub fn exit_label(app_exit: bool) -> &'static str {
    if app_exit { CLOSE_APPLICATIONS_LABEL } else { EXIT_LABEL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exit_item_is_named_by_app_exit() {
        assert_eq!(exit_label(true), "Close Applications");
        assert_eq!(exit_label(false), "Exit");
    }

    fn actions(selected: bool, tray: bool, flag: bool) -> (bool, bool) {
        let actions = start_actions(selected, tray, flag);
        (actions.start_wallpaper, actions.open_portal)
    }

    #[test]
    fn a_selected_theme_with_the_tray_visible_starts_the_wallpaper_only() {
        assert_eq!(actions(true, true, true), (true, false));
    }

    #[test]
    fn a_selected_theme_with_the_tray_hidden_starts_the_wallpaper_and_the_portal() {
        assert_eq!(actions(true, false, true), (true, true));
    }

    #[test]
    fn nothing_selected_opens_the_portal_whatever_the_tray_does() {
        assert_eq!(actions(false, true, true), (false, true));
        assert_eq!(actions(false, false, true), (false, true));
    }

    #[test]
    fn without_the_flag_no_portal_opens_in_any_case() {
        for selected in [false, true] {
            for tray in [false, true] {
                assert!(!start_actions(selected, tray, false).open_portal, "{selected} {tray}");
            }
        }
        assert!(start_actions(true, true, false).start_wallpaper);
    }

    #[test]
    fn the_service_exits_when_nothing_needs_it() {
        assert!(should_exit(false, false, false));
    }

    #[test]
    fn any_one_reason_keeps_the_service_running() {
        assert!(!should_exit(true, false, false), "the tray is visible");
        assert!(!should_exit(false, true, false), "a wallpaper is wanted");
        assert!(!should_exit(false, false, true), "a Portal is connected");
        assert!(!should_exit(true, true, true));
    }
}
