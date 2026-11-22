//! macOS implementation. Nothing is implemented yet.

use crate::{Capabilities, Platform};

pub use crate::unsupported::{DesktopSurface, EventLoop, LoopProxy, LoopTimers, RendererJob, Tray};
pub(crate) use crate::unsupported::{init_dpi_awareness, refresh_desktop};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
