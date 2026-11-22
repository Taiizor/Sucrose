//! Pixel geometry shared by every OS implementation. Everything here is physical pixels of the virtual desktop;
//! nothing is ever scaled by DPI.

/// A rectangle in physical pixels of the virtual desktop (never scaled by DPI), edges as in the Win32 `RECT`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RectPx {
    /// Left edge.
    pub left: i32,
    /// Top edge.
    pub top: i32,
    /// Right edge (exclusive).
    pub right: i32,
    /// Bottom edge (exclusive).
    pub bottom: i32,
}

impl RectPx {
    /// Width in pixels.
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// Height in pixels.
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    /// This screen rectangle expressed in the client space of a parent window whose client origin sits at
    /// `parent_screen_origin` on the screen. A parent is not guaranteed to start at the virtual-desktop origin.
    pub fn to_parent(&self, parent_screen_origin: (i32, i32)) -> Self {
        let (origin_x, origin_y) = parent_screen_origin;
        Self {
            left: self.left - origin_x,
            top: self.top - origin_y,
            right: self.right - origin_x,
            bottom: self.bottom - origin_y,
        }
    }

    /// Whether `other` lies completely inside this rectangle.
    pub fn covers(&self, other: &Self) -> bool {
        self.left <= other.left && self.top <= other.top && self.right >= other.right && self.bottom >= other.bottom
    }
}

/// One display as the OS reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorInfo {
    /// Stable device name (`\\.\DISPLAY1` on Windows).
    pub id: String,
    /// Full monitor rectangle in physical pixels.
    pub rect: RectPx,
    /// Work area (without the taskbar) in physical pixels.
    pub work: RectPx,
    /// Effective DPI of the monitor (96 is 100 %).
    pub dpi: u32,
    /// Whether this is the primary monitor.
    pub primary: bool,
}
