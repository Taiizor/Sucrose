//! Which desktop layout the shell is in, decided from the window tree only (never from the OS build number).
//!
//! The functions here are pure so they run, and are tested against recorded trees, on every OS.

use serde::{Deserialize, Serialize};

/// One window of the shell's tree as the attach code saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowNode {
    /// The window class name.
    pub class: String,
    /// The extended window style bits.
    pub ex_style: u32,
    /// The window style bits.
    pub style: u32,
    /// Whether the window is visible.
    pub visible: bool,
    /// The screen rectangle as `[left, top, right, bottom]`.
    pub rect: [i32; 4],
    /// Child windows, topmost first.
    #[serde(default)]
    pub children: Vec<WindowNode>,
}

/// What the shell looks like right now.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellSnapshot {
    /// The `Progman` window, if the shell has one.
    pub progman: Option<WindowNode>,
    /// Top-level windows other than Progman that hold a `SHELLDLL_DefView` (the icons).
    #[serde(default)]
    pub top_level_with_defview: Vec<WindowNode>,
}

/// How the shell arranges the desktop windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopLayout {
    /// The icons and the wallpaper `WorkerW` are children of Progman (Windows 11 24H2 and later).
    Raised,
    /// The icons moved into a top-level `WorkerW` after the `0x052C` message (Windows 7 to 11 23H2).
    Classic,
    /// The tree matches no known layout.
    Unsupported,
}

/// Classifies a snapshot of the shell's window tree.
pub fn classify(snapshot: &ShellSnapshot) -> DesktopLayout {
    let _ = snapshot;
    DesktopLayout::Unsupported
}
