//! macOS implementation. Nothing is implemented yet.

use crate::{Capabilities, Platform};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
