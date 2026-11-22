//! The only crate for OS-specific code (D-002). Each OS implements [`Platform`]; anything an OS cannot do is
//! reported through [`Capabilities`] instead of being faked. This crate may opt out of the workspace `unsafe_code`
//! lint with `#![allow(unsafe_code)]` once it needs FFI (D-011).

// D-011: the Windows module calls Win32 APIs; each unsafe block carries a SAFETY comment, which clippy enforces.
#![allow(unsafe_code)]
#![warn(clippy::undocumented_unsafe_blocks)]

#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "macos")]
mod os_macos;
#[cfg(target_os = "windows")]
mod os_windows;
#[cfg(not(target_os = "windows"))]
mod unsupported;

#[cfg(target_os = "linux")]
use os_linux as imp;
#[cfg(target_os = "macos")]
use os_macos as imp;
#[cfg(target_os = "windows")]
use os_windows as imp;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Sucrose supports Windows, macOS and Linux only");

pub mod geometry;
pub mod icon;
pub mod layout;

pub use geometry::{MonitorInfo, RectPx};
pub use icon::truncate_tooltip;
pub use imp::{DesktopSurface, EventLoop, LoopProxy, LoopTimers, RendererJob, Tray};

/// What an [`EventLoop`] tells its handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopEvent {
    /// Another thread called [`LoopProxy::wake`]: drain whatever it queued.
    Wake,
    /// Another thread (or the tray menu) called [`LoopProxy::quit`].
    Quit,
    /// A monitor was plugged in or removed, its resolution or DPI changed, or the work area changed. Bursts are
    /// normal: the receiver debounces.
    DisplayChanged,
    /// The shell (explorer.exe) started again, so everything it owned, including the desktop surface, is gone.
    ShellRestarted,
    /// The timer with this id (see [`EventLoop::set_timer`]) fired.
    Timer(u32),
}

/// The window the user is working in, as the OS reports it. Only what the fullscreen rule needs; the title is never
/// read (T-02-28).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundWindow {
    /// The window rectangle in physical pixels, frame included.
    pub rect: RectPx,
    /// The window class name (`Progman`, `Chrome_WidgetWin_1`, ...).
    pub class_name: String,
    /// The process that owns the window.
    pub process_id: u32,
    /// Whether the window is visible.
    pub visible: bool,
    /// Whether the window is minimized.
    pub minimized: bool,
    /// Whether the shell hides the window although it is "visible" (another virtual desktop, a suspended UWP app).
    pub cloaked: bool,
}

/// What the handler of an [`EventLoop`] wants next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Keep running.
    Continue,
    /// Return from [`EventLoop::run`].
    Exit,
}

/// What the current OS and session support. The UI hides or explains anything that is `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// A surface behind the desktop icons can host a wallpaper.
    pub desktop_wallpaper: bool,
    /// Another application's window can be embedded (Application wallpapers).
    pub embed_application: bool,
    /// Fullscreen applications can be detected.
    pub fullscreen_detection: bool,
}

/// Why a platform call failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlatformError {
    /// This OS does not provide the feature (reported through capabilities where the UI needs to know).
    #[error("{0} is not supported on this OS")]
    Unsupported(&'static str),
    /// An OS call failed.
    #[error("{context} failed with OS error {code}")]
    Os {
        /// What was being done.
        context: &'static str,
        /// The OS error code (the HRESULT on Windows).
        code: i32,
    },
    /// A library that wraps the OS failed.
    #[error("{context} failed: {message}")]
    Failed {
        /// What was being done.
        context: &'static str,
        /// What the library reported.
        message: String,
    },
}

/// The DPI awareness a thread of this process runs with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiAwareness {
    /// Per-Monitor-V2: coordinates are physical pixels and windows get DPI changes.
    PerMonitorV2,
    /// Per-Monitor (V1).
    PerMonitor,
    /// One DPI for the whole session.
    System,
    /// DPI virtualized by the OS.
    Unaware,
    /// The OS reported something else.
    Unknown,
    /// The OS has no such concept.
    NotApplicable,
}

/// Makes this process Per-Monitor-V2 aware and reports the awareness that is in effect afterwards. Call it before
/// any window or engine thread exists: threads inherit the process default.
pub fn init_dpi_awareness() -> DpiAwareness {
    imp::init_dpi_awareness()
}

/// Repaints the desktop after the last wallpaper surface is gone, so no frame of the wallpaper is left behind.
pub fn refresh_desktop() {
    imp::refresh_desktop();
}

/// OS integration used by the service.
pub trait Platform: Send + Sync {
    /// Short OS name for logs, the same value as `std::env::consts::OS`.
    fn name(&self) -> &'static str;

    /// Features this implementation provides.
    fn capabilities(&self) -> Capabilities;

    /// A string that identifies the current user on this machine (the account SID on Windows), used to name the
    /// per-user service endpoint and to restrict it to that user.
    fn user_key(&self) -> Result<String, PlatformError> {
        Err(PlatformError::Unsupported("user identity"))
    }

    /// The connected displays, primary first. Empty where the OS cannot enumerate them.
    fn monitors(&self) -> Vec<MonitorInfo> {
        Vec::new()
    }

    /// The window the user is working in, or `None` where the OS cannot tell or there is none.
    fn foreground_window(&self) -> Option<ForegroundWindow> {
        None
    }
}

/// The implementation for the OS this binary was built for.
pub fn current() -> Box<dyn Platform> {
    Box::new(imp::Current)
}

#[cfg(test)]
mod tests {
    #[test]
    fn current_matches_the_build_target() {
        assert_eq!(super::current().name(), std::env::consts::OS);
    }
}
