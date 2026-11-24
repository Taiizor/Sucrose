//! The tray hint dialog (UI-05, D-11): on the first close of the window, while the hint is on and the tray icon is
//! visible, the Portal says it keeps running in the system tray. Close is disabled for a 5 second countdown; it turns the
//! hint off through the service and then closes the window.

/// Seconds the Close button stays disabled.
pub const COUNTDOWN_START: u32 = 5;

/// Whether closing the window shows the hint: the setting is on, the tray icon is visible and the hint was not shown
/// during this window's life yet.
pub fn should_hint(hint_tray_icon: bool, tray_visible: bool, already_shown: bool) -> bool {
    let _ = (hint_tray_icon, tray_visible, already_shown);
    false
}

/// The Close label with the seconds that remain; plain "Close" once the countdown is over.
pub fn countdown_label(remaining: u32) -> String {
    let _ = remaining;
    String::new()
}

/// Whether Close can be pressed.
pub fn close_enabled(remaining: u32) -> bool {
    let _ = remaining;
    true
}

/// The seconds that remain after one more second.
pub fn next_remaining(remaining: u32) -> u32 {
    remaining
}

#[cfg(test)]
mod tests {
    use super::{COUNTDOWN_START, close_enabled, countdown_label, next_remaining, should_hint};

    #[test]
    fn the_hint_shows_only_when_everything_allows_it() {
        assert!(should_hint(true, true, false));
        assert!(!should_hint(false, true, false), "the user turned the hint off");
        assert!(
            !should_hint(true, false, false),
            "without a tray icon there is nothing to point at"
        );
        assert!(!should_hint(true, true, true), "once per window");
        assert!(!should_hint(false, false, true));
    }

    #[test]
    fn the_countdown_counts_from_five_to_a_plain_close() {
        let labels: Vec<String> = (0..=COUNTDOWN_START).rev().map(countdown_label).collect();
        assert_eq!(
            labels,
            ["Close (5)", "Close (4)", "Close (3)", "Close (2)", "Close (1)", "Close"]
        );
    }

    #[test]
    fn close_works_only_after_the_countdown() {
        for remaining in 1..=COUNTDOWN_START {
            assert!(!close_enabled(remaining), "{remaining}");
        }
        assert!(close_enabled(0));
    }

    #[test]
    fn the_countdown_stops_at_zero() {
        assert_eq!(next_remaining(5), 4);
        assert_eq!(next_remaining(1), 0);
        assert_eq!(next_remaining(0), 0);
    }
}
