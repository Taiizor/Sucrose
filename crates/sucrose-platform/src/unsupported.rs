//! The desktop surface and event loop on OSes that have no wallpaper support yet: every call reports
//! [`PlatformError::Unsupported`] instead of pretending.

use crate::{DpiAwareness, Flow, LoopEvent, PlatformError, RectPx};

/// A surface behind the desktop icons. Not available on this OS.
#[derive(Debug)]
pub struct DesktopSurface;

impl DesktopSurface {
    /// Always fails on this OS.
    pub fn attach(_rect: RectPx) -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported("desktop surface"))
    }

    /// The native window handle; there is none here.
    pub fn native_handle(&self) -> u64 {
        0
    }
}

/// The thread's message loop. Not available on this OS.
#[derive(Debug)]
pub struct EventLoop;

impl EventLoop {
    /// Always fails on this OS.
    pub fn new() -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported("event loop"))
    }

    /// A handle that can stop the loop from another thread.
    pub fn proxy(&self) -> LoopProxy {
        LoopProxy
    }

    /// Returns at once: there is nothing to run.
    pub fn run(self, _handler: impl FnMut(LoopEvent) -> Flow) {}
}

/// Wakes or stops an [`EventLoop`] from any thread.
#[derive(Debug, Clone)]
pub struct LoopProxy;

impl LoopProxy {
    /// Does nothing on this OS.
    pub fn wake(&self) {}

    /// Does nothing on this OS.
    pub fn quit(&self) {}
}

/// The notification-area icon. Not available on this OS.
#[derive(Debug)]
pub struct Tray;

impl Tray {
    /// Always fails on this OS.
    pub fn new(_ico: &[u8], _tooltip: &str, _proxy: LoopProxy) -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported("tray icon"))
    }

    /// Does nothing on this OS.
    pub fn set_tooltip(&self, _tooltip: &str) {}
}

pub(crate) fn init_dpi_awareness() -> DpiAwareness {
    DpiAwareness::NotApplicable
}
