//! Linux implementation. Nothing is implemented yet.

use crate::{Capabilities, Platform};

pub(crate) use crate::unsupported::init_dpi_awareness;
pub use crate::unsupported::{DesktopSurface, EventLoop, LoopProxy, Tray};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
