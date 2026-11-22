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
    /// What to show and where; sent once after a valid [`ToService::Hello`].
    Assign(Assignment),
}

/// One wallpaper job for a renderer: a monitor and the file to play on it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The renderer slot this job is for, as in [`ToService::Hello`].
    pub slot: u8,
    /// The monitor's device name.
    pub monitor: String,
    /// The monitor rectangle in physical pixels: left, top, right, bottom.
    pub rect: [i32; 4],
    /// The absolute path of the file to play.
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_keep_their_wire_index() {
        // Appending is the only allowed change (see the module docs): Close stays 0, Assign is 1.
        let close = postcard::to_stdvec(&ToRenderer::Close).expect("encode Close");
        let assign = postcard::to_stdvec(&ToRenderer::Assign(Assignment {
            slot: 2,
            monitor: "M".to_owned(),
            rect: [0, 0, 10, 20],
            source: "a.mp4".to_owned(),
        }))
        .expect("encode Assign");

        assert_eq!(close, [0]);
        assert_eq!(assign[0], 1);
    }

    #[test]
    fn an_assignment_survives_the_wire() {
        let message = ToRenderer::Assign(Assignment {
            slot: 0,
            monitor: "\\\\.\\DISPLAY1".to_owned(),
            rect: [-1920, 0, 0, 1080],
            source: "C:\\themes\\clip.mp4".to_owned(),
        });

        let bytes = postcard::to_stdvec(&message).expect("encode");

        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
    }
}
