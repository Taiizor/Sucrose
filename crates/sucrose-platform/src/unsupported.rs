//! The desktop surface and event loop on OSes that have no wallpaper support yet: every call reports
//! [`PlatformError::Unsupported`] instead of pretending.

use std::{
    io,
    process::{Child, Command},
};

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

    /// Always fails on this OS.
    pub fn set_rect(&self, _rect: RectPx) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported("desktop surface"))
    }

    /// Never true: no surface exists on this OS.
    pub fn destroyed_externally(&self) -> bool {
        false
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

    /// Does nothing on this OS.
    pub fn set_timer(&self, _id: u32, _ms: u32) {}

    /// Does nothing on this OS.
    pub fn kill_timer(&self, _id: u32) {}

    /// A handle for the loop's timers, which do nothing on this OS.
    pub fn timers(&self) -> LoopTimers {
        LoopTimers
    }
}

/// Starts and stops the timers of an [`EventLoop`]. Not available on this OS.
#[derive(Debug, Clone, Copy)]
pub struct LoopTimers;

impl LoopTimers {
    /// Does nothing on this OS.
    pub fn set(&self, _id: u32, _ms: u32) {}

    /// Does nothing on this OS.
    pub fn kill(&self, _id: u32) {}
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

/// Renderer processes started by the service. No wallpaper renderer exists on this OS yet, so this only spawns.
#[derive(Debug)]
pub struct RendererJob;

impl RendererJob {
    /// Always succeeds: there is nothing to create.
    pub fn new() -> Result<Self, PlatformError> {
        Ok(Self)
    }

    /// Starts `cmd` as a plain child process.
    pub fn spawn(&self, cmd: &mut Command) -> io::Result<Child> {
        cmd.spawn()
    }

    /// Does nothing: this OS has no job to end.
    pub fn terminate_all(&self) -> Result<(), PlatformError> {
        Ok(())
    }
}

pub(crate) fn init_dpi_awareness() -> DpiAwareness {
    DpiAwareness::NotApplicable
}

/// Nothing is painted on this OS, so there is nothing to refresh.
pub(crate) fn refresh_desktop() {}

/// The wall clock since the Unix epoch in nanoseconds: not monotonic, which is why [crate::monotonic_ns] documents it
/// as a fallback until these OSes get a real implementation.
pub(crate) fn monotonic_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))
}
