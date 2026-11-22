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

/// `WS_EX_NOREDIRECTIONBITMAP`: Progman carries it once the shell composes the desktop itself.
const WS_EX_NOREDIRECTIONBITMAP: u32 = 0x0020_0000;
/// The window class that holds the desktop icons.
pub const DEFVIEW_CLASS: &str = "SHELLDLL_DefView";

/// Classifies a snapshot of the shell's window tree. The raised layout is recognized by Progman's own style or by
/// the icons living inside it; the classic one by a top-level window other than Progman holding the icons.
pub fn classify(snapshot: &ShellSnapshot) -> DesktopLayout {
    let Some(progman) = &snapshot.progman else {
        return DesktopLayout::Unsupported;
    };
    if progman.ex_style & WS_EX_NOREDIRECTIONBITMAP != 0 || progman.children.iter().any(|c| c.class == DEFVIEW_CLASS) {
        DesktopLayout::Raised
    } else if !snapshot.top_level_with_defview.is_empty() {
        DesktopLayout::Classic
    } else {
        DesktopLayout::Unsupported
    }
}

/// The window class of the shell's wallpaper window (a child of Progman in the raised layout).
pub const WORKERW_CLASS: &str = "WorkerW";

/// What one child of Progman is, for [`raised_order`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiblingKind {
    /// `SHELLDLL_DefView`, the desktop icons.
    Icons,
    /// The surface that is being checked.
    Surface,
    /// A `WorkerW`, the shell's own wallpaper.
    Wallpaper,
    /// Anything else, including the surfaces of other renderers.
    Other,
}

/// How the surface sits among Progman's children in the raised layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaisedOrder {
    /// Icons above the surface, the surface above every wallpaper window: the video is visible behind the icons.
    Correct,
    /// The surface covers the icons.
    SurfaceAboveIcons,
    /// A wallpaper window is above the surface and hides the video.
    WallpaperAboveSurface,
    /// The surface is not among Progman's children.
    SurfaceMissing,
    /// The icons window is not among Progman's children.
    IconsMissing,
}

/// Checks Progman's children (topmost first) for the raised layout: the icons first, then the surface, then every
/// wallpaper window. Other windows may sit anywhere.
pub fn raised_order(siblings: &[SiblingKind]) -> RaisedOrder {
    let Some(surface) = siblings.iter().position(|kind| *kind == SiblingKind::Surface) else {
        return RaisedOrder::SurfaceMissing;
    };
    let Some(icons) = siblings.iter().position(|kind| *kind == SiblingKind::Icons) else {
        return RaisedOrder::IconsMissing;
    };
    if surface < icons {
        return RaisedOrder::SurfaceAboveIcons;
    }
    if siblings[..surface].contains(&SiblingKind::Wallpaper) {
        return RaisedOrder::WallpaperAboveSurface;
    }
    RaisedOrder::Correct
}
