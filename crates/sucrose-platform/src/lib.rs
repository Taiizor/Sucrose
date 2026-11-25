//! The only crate for OS-specific code (D-002). Each OS implements [`Platform`]; anything an OS cannot do is
//! reported through [`Capabilities`] instead of being faked. This crate may opt out of the workspace `unsafe_code`
//! lint with `#![allow(unsafe_code)]` once it needs FFI (D-011).

// D-011: the Windows module calls Win32 APIs; each unsafe block carries a SAFETY comment, which clippy enforces.
#![allow(unsafe_code)]
#![warn(clippy::undocumented_unsafe_blocks)]

use std::{
    io,
    path::Path,
    process::{Child, Command},
};

#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "macos")]
mod os_macos;
#[cfg(target_os = "windows")]
mod os_windows;
mod trash;
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

pub mod autostart;
pub mod geometry;
pub mod icon;
pub mod layout;

pub use autostart::{AutostartState, autostart_state, disable_autostart, enable_autostart, legacy_entry};
pub use geometry::{MonitorInfo, Placement, PlacementVerdict, RectPx};
pub use icon::truncate_tooltip;
pub use imp::{DesktopSurface, EventLoop, LoopProxy, LoopTimers, RendererJob, Tray};
pub use sucrose_core::settings::GpuPreference;
pub use trash::move_to_trash;

/// What the user chose on the tray icon (D-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// Open Sucrose: from the menu item or a left double-click on the icon.
    Open,
    /// The exit item: Close Applications or Exit, by what its label says.
    Exit,
}

/// The texts of the tray menu items. The service owns them, so the platform crate carries no language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayLabels {
    /// The Open item.
    pub open: String,
    /// The exit item.
    pub exit: String,
}

/// A clock in nanoseconds that every process on this machine reads the same way, for comparing moments across
/// the service and its renderers (D-02). On Windows it is QueryPerformanceCounter scaled to nanoseconds, which is
/// monotonic and system-wide. On the OSes without an implementation yet it is the wall clock since the Unix epoch,
/// which is **not** monotonic.
#[must_use]
pub fn monotonic_ns() -> u64 {
    imp::monotonic_ns()
}

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
    /// The timer with this id (see [`LoopTimers::set`]) fired.
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

/// Why [`EventLoop::run`] returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopEnd {
    /// The handler asked to exit, or the thread was told to quit: the normal end.
    Quit,
    /// The OS could not hand out messages any more: the owner must not treat this as a quit.
    Failed,
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
    /// The GPU an executable's Direct3D devices use can be requested.
    pub gpu_preference: bool,
    /// Sucrose can start with the OS: a Run value (Windows), a LaunchAgent (macOS) or an XDG autostart file (Linux).
    pub autostart: bool,
    /// The window backdrop materials (Mica, Acrylic, Tabbed) can be shown behind a window (Windows 11 22H2 and later).
    /// Where this is `false` a stored backdrop is applied as none without changing the setting (D-22).
    pub window_backdrop: bool,
    /// Folders can be moved to the recycle bin (Windows) or trash (macOS, Linux) instead of being deleted (D-09). The
    /// `trash` crate supports a bin on all three OSes; on a Linux system without a trash folder the move itself fails
    /// and the caller keeps the item.
    pub recycle_bin: bool,
}

/// A GPU adapter of this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuAdapter {
    /// The adapter's description (for example the GPU model).
    pub name: String,
    /// The adapter's LUID in the form typeperf prints in GPU instance names (`0x00000000_0x0000A1B2`: high part,
    /// then low part), so logs and `cargo xtask measure` name an adapter the same way.
    pub luid: String,
    /// Dedicated video memory in MiB.
    pub dedicated_video_memory_mb: u64,
}

/// What [`Platform::set_gpu_preference`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPreferenceChange {
    /// The OS setting was written.
    Written,
    /// The OS setting already had the wanted preference, so nothing was written.
    Unchanged,
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

/// Starts a sibling executable that has to outlive the caller, without a console window where the OS would open one.
/// The Portal uses it to start the service when none runs. The child gets no handle back to the caller: the caller
/// builds `cmd` from an argument vector, never a shell string.
pub fn spawn_detached(cmd: &mut Command) -> io::Result<Child> {
    imp::spawn_detached(cmd)
}

/// Lets the process `pid` bring a window to the front once. A process the user just started holds the foreground right
/// and uses this to pass it to the Portal that is already open, so the existing window comes to the front instead of
/// only flashing in the taskbar. Fails with [`PlatformError::Unsupported`] where the OS has no such right, and with
/// [`PlatformError::Os`] when the caller does not hold it; callers log that and go on.
pub fn allow_set_foreground(pid: u32) -> Result<(), PlatformError> {
    imp::allow_set_foreground(pid)
}

/// Disconnects the server end of a named pipe, which ends the connection for every handle of it (the IPC layer uses it
/// to drop a peer whose reader thread is blocked on another handle). Windows only: other OSes have no equivalent call
/// and the IPC layer reports that itself.
#[cfg(target_os = "windows")]
pub fn disconnect_pipe(handle: std::os::windows::io::BorrowedHandle<'_>) -> Result<(), PlatformError> {
    imp::disconnect_pipe(handle)
}

/// The parts of a process's own memory, in bytes, that a measurement needs to tell apart (the heap Rust keeps alive, the
/// heap's committed pages, and the video memory the process uses). A field is `None` where the OS gives no reading:
/// it is reported, never guessed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryDetail {
    /// Bytes the process heap has handed out and not got back (what Rust code keeps alive).
    pub heap_allocated: Option<u64>,
    /// Bytes of the process heap the OS has committed, which stay committed after frees until the heap gives them back.
    pub heap_committed: Option<u64>,
    /// Video memory this process uses in the adapters' local segment group (dedicated memory), over all adapters.
    pub gpu_local: Option<u64>,
    /// Video memory this process uses in the adapters' non-local segment group (shared system memory), over all adapters.
    pub gpu_non_local: Option<u64>,
}

/// Reads [`MemoryDetail`] for this process. It is a measurement aid: callers read it only while they log it.
pub fn memory_detail() -> MemoryDetail {
    #[cfg(target_os = "windows")]
    {
        imp::memory_detail()
    }
    #[cfg(not(target_os = "windows"))]
    {
        MemoryDetail::default()
    }
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

    /// Asks the OS to give the Direct3D devices of the executable `exe` the GPU `preference` names (the per-app
    /// graphics setting on Windows). Other settings the OS keeps for that executable are preserved.
    fn set_gpu_preference(&self, exe: &Path, preference: GpuPreference) -> Result<GpuPreferenceChange, PlatformError> {
        let _ = (exe, preference);
        Err(PlatformError::Unsupported("GPU preference"))
    }

    /// Tells the OS which system backdrop material to draw behind `window`: on Windows 11 22H2 and later the DWM system
    /// backdrop type (Mica: main window, Tabbed: tabbed window; Acrylic and None clear it, because the Portal draws
    /// Acrylic through GPUI's accent policy). Fails with the OS error (the HRESULT on Windows) when the OS refuses, and
    /// with [`PlatformError::Unsupported`] where [`Capabilities::window_backdrop`] is false; nothing is changed then.
    fn set_window_backdrop(
        &self,
        window: &dyn raw_window_handle::HasWindowHandle,
        backdrop: sucrose_core::settings::Backdrop,
    ) -> Result<(), PlatformError> {
        let _ = (window, backdrop);
        Err(PlatformError::Unsupported("window backdrop"))
    }

    /// Every GPU adapter, to name a LUID. The order is the OS's enumeration order and is not evidence of which
    /// adapter a device of this process uses (see [`Platform::default_gpu_adapter`]). Empty where the OS has none to
    /// report.
    fn gpu_adapters(&self) -> Vec<GpuAdapter> {
        Vec::new()
    }

    /// The adapter a graphics device of this process gets when it names no adapter, read back from such a device, so
    /// it reflects the OS GPU preference. While no `d3d11-adapter` is set this is the adapter libmpv's video device
    /// uses. `None` where it cannot be determined.
    fn default_gpu_adapter(&self) -> Option<GpuAdapter> {
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

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn memory_detail_is_empty_where_the_os_has_no_reading() {
        assert_eq!(super::memory_detail(), super::MemoryDetail::default());
    }
}
