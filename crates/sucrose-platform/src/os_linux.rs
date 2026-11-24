//! Linux implementation. Nothing is implemented yet.

use crate::{Capabilities, Platform};

pub use crate::unsupported::{DesktopSurface, EventLoop, LoopProxy, LoopTimers, RendererJob, Tray};
pub(crate) use crate::unsupported::{
    allow_set_foreground, autostart_state, disable_autostart, enable_autostart, init_dpi_awareness, legacy_entry,
    monotonic_ns, refresh_desktop, spawn_detached,
};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            autostart: true,
            recycle_bin: true,
            ..Capabilities::default()
        }
    }
}
