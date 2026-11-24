//! The desktop surface and event loop on OSes that have no wallpaper support yet: every call reports
//! [`PlatformError::Unsupported`] instead of pretending.

use std::{
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use crate::{
    DpiAwareness, Flow, LoopEnd, LoopEvent, Placement, PlatformError, RectPx, TrayAction, TrayLabels,
    autostart::{self, AutostartState, EntryKind},
};

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
    pub fn set_rect(&self, _rect: RectPx) -> Result<Placement, PlatformError> {
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
    pub fn run(self, _handler: impl FnMut(LoopEvent) -> Flow) -> LoopEnd {
        LoopEnd::Quit
    }

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
    pub fn new(
        _ico: &[u8],
        _tooltip: &str,
        _labels: &TrayLabels,
        _on_action: impl Fn(TrayAction) + Send + Sync + 'static,
    ) -> Result<Self, PlatformError> {
        Err(PlatformError::Unsupported("tray icon"))
    }

    /// Does nothing on this OS.
    pub fn set_tooltip(&self, _tooltip: &str) {}

    /// Does nothing on this OS.
    pub fn set_open_enabled(&self, _enabled: bool) {}

    /// Does nothing on this OS.
    pub fn set_exit_label(&self, _label: &str) {}
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

/// A plain spawn: no OS here opens a console window for a child of a GUI process.
pub(crate) fn spawn_detached(cmd: &mut Command) -> io::Result<Child> {
    cmd.spawn()
}

pub(crate) fn init_dpi_awareness() -> DpiAwareness {
    DpiAwareness::NotApplicable
}

/// The kind of start entry file this OS uses and the folder it lives under: the home folder for a macOS LaunchAgent,
/// the config folder for a Linux XDG autostart file.
fn entry_target() -> Result<(EntryKind, PathBuf), PlatformError> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").filter(|home| Path::new(home).is_absolute());
        home.map(|home| (EntryKind::LaunchAgent, PathBuf::from(home)))
            .ok_or(PlatformError::Unsupported("the home folder"))
    }
    #[cfg(target_os = "linux")]
    {
        autostart::config_dir_from(
            std::env::var_os("XDG_CONFIG_HOME").as_deref(),
            std::env::var_os("HOME").as_deref(),
        )
        .map(|dir| (EntryKind::Desktop, dir))
        .ok_or(PlatformError::Unsupported("the config folder"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(PlatformError::Unsupported("start with the OS"))
    }
}

/// Writes Sucrose's own LaunchAgent or XDG autostart file for `exe`.
pub(crate) fn enable_autostart(exe: &Path) -> Result<(), PlatformError> {
    if !exe.is_absolute() {
        return Err(PlatformError::Failed {
            context: "startup entry",
            message: "the executable path is not absolute".to_owned(),
        });
    }
    let (kind, base) = entry_target()?;
    autostart::write_entry(kind, &base, exe)
}

/// Removes Sucrose's own entry file.
pub(crate) fn disable_autostart() -> Result<(), PlatformError> {
    let (kind, base) = entry_target()?;
    autostart::remove_entry(kind, &base)
}

/// Reads Sucrose's own entry file.
pub(crate) fn autostart_state(exe: &Path) -> AutostartState {
    entry_target().map_or_else(
        |_| AutostartState::default(),
        |(kind, base)| autostart::entry_state(kind, &base, exe),
    )
}

/// The C# app has no start entry on these OSes.
pub(crate) fn legacy_entry() -> bool {
    false
}

/// Nothing is painted on this OS, so there is nothing to refresh.
pub(crate) fn refresh_desktop() {}

/// Always fails on this OS: there is no foreground right to hand over.
pub(crate) fn allow_set_foreground(_pid: u32) -> Result<(), PlatformError> {
    Err(PlatformError::Unsupported("foreground handoff"))
}

/// The wall clock since the Unix epoch in nanoseconds: not monotonic, which is why [crate::monotonic_ns] documents it
/// as a fallback until these OSes get a real implementation.
pub(crate) fn monotonic_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))
}
