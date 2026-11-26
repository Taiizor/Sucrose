//! The legacy key rules both web engines apply to a wallpaper page (D-13): which keys a wallpaper swallows, and which
//! open the developer tools in developer mode. Pure values keyed on Windows virtual-key codes, so the table is tested on
//! every OS and CEF's keyboard handler and WebView2's accelerator key handler share one source.

/// What a key press in the wallpaper does (D-13, legacy keyboard handler).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// The page gets the key as usual.
    Pass,
    /// The key is swallowed: a wallpaper is not a browser, so refresh, find, zoom and the DevTools shortcuts do nothing.
    Swallow,
    /// The key opens DevTools (developer mode only).
    DevTools,
}

/// The Windows virtual-key codes the keyboard rules use.
mod vk {
    pub const ZERO: i32 = 0x30;
    pub const C: i32 = 0x43;
    pub const F: i32 = 0x46;
    pub const I: i32 = 0x49;
    pub const J: i32 = 0x4A;
    pub const R: i32 = 0x52;
    pub const NUMPAD0: i32 = 0x60;
    pub const ADD: i32 = 0x6B;
    pub const SUBTRACT: i32 = 0x6D;
    pub const F5: i32 = 0x74;
    pub const F12: i32 = 0x7B;
    pub const OEM_PLUS: i32 = 0xBB;
    pub const OEM_MINUS: i32 = 0xBD;
}

/// What the key with the Windows virtual-key code `code` does, given the modifiers. With developer mode on, F12 and
/// Ctrl+Shift+I open DevTools and everything else passes. With it off, the legacy blocked keys (Ctrl+F, Ctrl+R, F5,
/// Ctrl with plus, minus or zero, F12) and the DevTools shortcuts (Ctrl+Shift+I, J and C) are swallowed, so no
/// DevTools exist (D-13).
#[must_use]
pub fn key_action(developer_mode: bool, code: i32, ctrl: bool, shift: bool) -> KeyAction {
    let devtools_chord = ctrl && shift && code == vk::I;
    if developer_mode {
        return if code == vk::F12 || devtools_chord {
            KeyAction::DevTools
        } else {
            KeyAction::Pass
        };
    }
    let page_chord = ctrl
        && matches!(
            code,
            vk::F | vk::R | vk::ZERO | vk::NUMPAD0 | vk::ADD | vk::SUBTRACT | vk::OEM_PLUS | vk::OEM_MINUS
        );
    let devtools_chord = ctrl && shift && matches!(code, vk::I | vk::J | vk::C);
    if code == vk::F5 || code == vk::F12 || page_chord || devtools_chord {
        KeyAction::Swallow
    } else {
        KeyAction::Pass
    }
}
