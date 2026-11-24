//! macOS implementation. Nothing is implemented yet.

use crate::{Capabilities, Platform};

pub use crate::unsupported::{DesktopSurface, EventLoop, LoopProxy, LoopTimers, RendererJob, Tray};
pub(crate) use crate::unsupported::{
    allow_set_foreground, init_dpi_awareness, monotonic_ns, refresh_desktop, spawn_detached,
};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
