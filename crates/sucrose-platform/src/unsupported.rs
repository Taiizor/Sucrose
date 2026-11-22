//! The desktop surface and event loop on OSes that have no wallpaper support yet: every call reports
//! [`PlatformError::Unsupported`] instead of pretending.

use crate::{DpiAwareness, PlatformError, RectPx};

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
    pub fn run(self) {}
}

/// Stops an [`EventLoop`] from any thread.
#[derive(Debug, Clone)]
pub struct LoopProxy;

impl LoopProxy {
    /// Does nothing on this OS.
    pub fn quit(&self) {}
}

pub(crate) fn init_dpi_awareness() -> DpiAwareness {
    DpiAwareness::NotApplicable
}
