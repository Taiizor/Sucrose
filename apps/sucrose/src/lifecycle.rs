//! When the service starts a Portal and when it ends by itself (D-12, D-13). Both rules are pure functions of facts the
//! service already holds, so they are tested without a process.

use std::path::Path;

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

/// What Close Applications still waits for (TRAY-05): the renderers to end and the Portal to be gone. Pure, so the order
/// of the two is tested without a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closing {
    /// Every renderer has ended (or was killed after the grace).
    pub renderers_done: bool,
    /// The Portal is gone: it closed, was ended, or was never there.
    pub portal_done: bool,
}

impl Closing {
    /// Starts closing `renderers` running renderers and, when `portal` is true, one registered Portal.
    pub fn new(renderers: usize, portal: bool) -> Self {
        Self {
            renderers_done: renderers == 0,
            portal_done: !portal,
        }
    }

    /// The shutdown worker reported every renderer ended.
    pub fn renderers_ended(&mut self) {
        self.renderers_done = true;
    }

    /// The Portal is gone, whether it closed on the Close it was sent, closed before that, or was ended.
    pub fn portal_gone(&mut self) {
        self.portal_done = true;
    }

    /// Whether the service may exit.
    pub fn done(&self) -> bool {
        self.renderers_done && self.portal_done
    }
}

/// Whether the process that holds the Portal's pid is the Portal executable next to this one (T-03-37). `running` is
/// the executable path the OS reports for it, `sibling` the path the service starts the Portal from. Anything else
/// is not ours to end.
pub fn is_sibling_portal(running: Option<&Path>, sibling: &Path) -> bool {
    running.is_some_and(|running| same_path(running, sibling))
}

/// Whether two paths name the same file by their text: component by component, and on Windows without regard to
/// ASCII case (its file names are case-insensitive). A difference in non-ASCII case reads as another file, which only
/// ever leaves a process alone.
fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        let mut a = a.components();
        let mut b = b.components();
        loop {
            match (a.next(), b.next()) {
                (None, None) => return true,
                (Some(x), Some(y)) => {
                    if !x
                        .as_os_str()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&y.as_os_str().to_string_lossy())
                    {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    } else {
        a == b
    }
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

    #[test]
    fn closing_with_nothing_to_wait_for_is_done_at_once() {
        assert!(Closing::new(0, false).done());
    }

    #[test]
    fn closing_with_a_portal_is_done_only_after_the_portal_is_gone() {
        let mut closing = Closing::new(0, true);
        assert!(!closing.done());
        closing.portal_gone();
        assert!(closing.done());
    }

    #[test]
    fn a_portal_gone_before_close_was_sent_counts() {
        // The Portal left on its own between the choice and the Close: its disconnect is all that is awaited.
        let mut closing = Closing::new(2, true);
        closing.portal_gone();
        assert!(!closing.done(), "the renderers are still closing");
        closing.renderers_ended();
        assert!(closing.done());
    }

    #[test]
    fn closing_with_renderers_and_a_portal_needs_both_in_either_order() {
        let mut renderers_first = Closing::new(2, true);
        renderers_first.renderers_ended();
        assert!(!renderers_first.done());
        renderers_first.portal_gone();
        assert!(renderers_first.done());

        let mut portal_first = Closing::new(2, true);
        portal_first.portal_gone();
        assert!(!portal_first.done());
        portal_first.renderers_ended();
        assert!(portal_first.done());
    }

    #[test]
    fn closing_with_renderers_and_no_portal_waits_for_the_renderers() {
        let mut closing = Closing::new(1, false);
        assert!(!closing.done());
        closing.renderers_ended();
        assert!(closing.done());
    }

    #[test]
    fn only_the_sibling_portal_executable_may_be_ended() {
        use std::path::Path;
        let sibling = Path::new("C:/Sucrose/sucrose-portal.exe");
        assert!(is_sibling_portal(
            Some(Path::new("C:/Sucrose/sucrose-portal.exe")),
            sibling
        ));
        assert!(!is_sibling_portal(
            Some(Path::new("C:/Other/sucrose-portal.exe")),
            sibling
        ));
        assert!(!is_sibling_portal(Some(Path::new("C:/Sucrose/sucrose.exe")), sibling));
        assert!(!is_sibling_portal(
            Some(Path::new("C:/Sucrose/Sucrose.Portal.exe")),
            sibling
        ));
        assert!(
            !is_sibling_portal(None, sibling),
            "an executable the OS did not report is not trusted"
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_sibling_check_ignores_case_on_windows() {
        use std::path::Path;
        assert!(is_sibling_portal(
            Some(Path::new("c:\\sucrose\\SUCROSE-PORTAL.EXE")),
            Path::new("C:\\Sucrose\\sucrose-portal.exe")
        ));
    }
}
