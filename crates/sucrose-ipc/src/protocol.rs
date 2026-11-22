//! The messages the service and its renderers exchange (D-009, D-021).
//!
//! Frames carry postcard-encoded values, and postcard encodes an enum variant by its index. Variants are therefore
//! **only ever appended** at the end of these enums, never inserted, reordered or removed; a renderer and a service
//! of different builds then still agree on every variant they both know. [`PROTOCOL_VERSION`] is bumped only for a
//! change that appending cannot express.

use serde::{Deserialize, Serialize};

/// Version of this protocol, sent in [`ToService::Hello`] so the service can refuse a renderer it cannot talk to.
pub const PROTOCOL_VERSION: u16 = 1;

/// Messages from a renderer to the service.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ToService {
    /// First message of every connection: who is calling and which protocol it speaks.
    Hello {
        /// The renderer's [`PROTOCOL_VERSION`].
        protocol_version: u16,
        /// Process id of the renderer; the service only accepts the processes it started itself.
        pid: u32,
        /// Renderer slot the service chose on the command line (`sucrose --render <slot>`).
        slot: u8,
    },
}

/// Messages from the service to a renderer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ToRenderer {
    /// Stop and exit.
    Close,
}
