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
    let _ = (selected_exists, tray_visible, open_portal_flag);
    StartActions {
        start_wallpaper: false,
        open_portal: false,
    }
}

/// The service exit rule (D-13): with the tray hidden, no wallpaper wanted and no Portal connected nothing needs the
/// service; with the tray visible it stays until Exit.
pub fn should_exit(tray_visible: bool, wallpaper_wanted: bool, portal_connected: bool) -> bool {
    let _ = (tray_visible, wallpaper_wanted, portal_connected);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

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
