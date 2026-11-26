//! The legacy key table both web engines apply to a wallpaper page (D-13).

use sucrose_engine_web::keys::{KeyAction, key_action};

#[test]
fn developer_mode_opens_devtools_only_from_f12_and_ctrl_shift_i() {
    assert_eq!(key_action(true, 0x7B, false, false), KeyAction::DevTools, "F12");
    assert_eq!(key_action(true, 0x49, true, true), KeyAction::DevTools, "Ctrl+Shift+I");
    assert_eq!(key_action(true, 0x49, false, false), KeyAction::Pass, "a plain I");
    assert_eq!(
        key_action(true, 0x74, false, false),
        KeyAction::Pass,
        "F5 is not blocked in developer mode"
    );
    assert_eq!(
        key_action(true, 0x52, true, false),
        KeyAction::Pass,
        "Ctrl+R is not blocked in developer mode"
    );
}

#[test]
fn without_developer_mode_the_browser_keys_and_the_devtools_shortcuts_are_swallowed() {
    for (code, ctrl, shift, what) in [
        (0x7B, false, false, "F12"),
        (0x74, false, false, "F5"),
        (0x46, true, false, "Ctrl+F"),
        (0x52, true, false, "Ctrl+R"),
        (0x30, true, false, "Ctrl+0"),
        (0x60, true, false, "Ctrl+numpad 0"),
        (0x6B, true, false, "Ctrl+numpad plus"),
        (0x6D, true, false, "Ctrl+numpad minus"),
        (0xBB, true, false, "Ctrl+plus"),
        (0xBD, true, false, "Ctrl+minus"),
        (0x49, true, true, "Ctrl+Shift+I"),
        (0x4A, true, true, "Ctrl+Shift+J"),
        (0x43, true, true, "Ctrl+Shift+C"),
    ] {
        assert_eq!(key_action(false, code, ctrl, shift), KeyAction::Swallow, "{what}");
    }
    assert_eq!(
        key_action(false, 0x41, false, false),
        KeyAction::Pass,
        "a plain A reaches the page"
    );
    assert_eq!(
        key_action(false, 0x46, false, false),
        KeyAction::Pass,
        "a plain F reaches the page"
    );
}

#[test]
fn the_legacy_blocked_keys_are_swallowed_in_both_modes_only_when_developer_mode_is_off() {
    // With developer mode on the same keys reach the page; the table never swallows a plain letter in either mode.
    for developer_mode in [false, true] {
        assert_eq!(
            key_action(developer_mode, 0x41, false, false),
            KeyAction::Pass,
            "a plain A, developer mode {developer_mode}"
        );
    }
    assert_eq!(
        key_action(true, 0x46, true, false),
        KeyAction::Pass,
        "Ctrl+F in developer mode"
    );
}
