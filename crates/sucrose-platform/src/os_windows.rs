//! Windows implementation. Nothing is implemented yet (desktop surfaces come in M1).

use crate::{Capabilities, Platform};

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }
}
