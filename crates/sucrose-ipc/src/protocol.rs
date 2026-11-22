//! The messages the service and its renderers exchange (D-009, D-021).
//!
//! Frames carry postcard-encoded values, and postcard encodes an enum variant by its index. Variants are therefore
//! **only ever appended** at the end of these enums, never inserted, reordered or removed; a renderer and a service
//! of different builds then still agree on every variant they both know. [`PROTOCOL_VERSION`] is bumped only for a
//! change that appending cannot express.

use serde::{Deserialize, Serialize};
use sucrose_core::settings::Stretch;

/// Version of this protocol, sent in [`ToService::Hello`] so the service can refuse a renderer it cannot talk to.
pub const PROTOCOL_VERSION: u16 = 2;

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
    /// The renderer could not start its wallpaper and is about to exit: what went wrong, for the log and the tray
    /// tooltip, and how the user can repair it.
    Failed {
        /// One sentence saying what failed.
        reason: String,
        /// One sentence saying what the user can do about it.
        hint: String,
    },
    /// The wallpaper is playing.
    Playing,
}

/// Messages from the service to a renderer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ToRenderer {
    /// Stop and exit.
    Close,
    /// What to show and where; sent once after a valid [`ToService::Hello`].
    Assign(Assignment),
    /// Pause (`true`) or resume (`false`) playback.
    Pause(bool),
    /// Set the playback volume, 0..=100.
    SetVolume(u8),
    /// Move the surface onto this rectangle (left, top, right, bottom, physical pixels) without restarting playback:
    /// the monitor moved, was resized or changed its DPI.
    SetRect([i32; 4]),
}

/// How the player is configured, taken from the legacy Engine.json (D-09).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackSettings {
    /// Playback volume, 0..=100.
    pub volume: u8,
    /// Whether the video restarts when it ends.
    pub looping: bool,
    /// How the video is fitted to the surface.
    pub stretch: Stretch,
    /// Whether the player may decode on the GPU.
    pub hardware_decoding: bool,
}

impl Default for PlaybackSettings {
    /// The legacy defaults: full volume, looping, UniformToFill, hardware decoding on.
    fn default() -> Self {
        Self {
            volume: 100,
            looping: true,
            stretch: Stretch::default(),
            hardware_decoding: true,
        }
    }
}

/// One wallpaper job for a renderer: a monitor and the file to play on it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The renderer slot this job is for, as in [`ToService::Hello`].
    pub slot: u8,
    /// The monitor's device name.
    pub monitor: String,
    /// The surface rectangle in physical pixels: left, top, right, bottom (the monitor or its work area).
    pub rect: [i32; 4],
    /// The absolute path of the file to play.
    pub source: String,
    /// How to play it.
    pub settings: PlaybackSettings,
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
            settings: PlaybackSettings::default(),
        }))
        .expect("encode Assign");

        assert_eq!(close, [0]);
        assert_eq!(assign[0], 1);
    }

    #[test]
    fn to_service_variants_keep_their_wire_index() {
        let hello = postcard::to_stdvec(&ToService::Hello {
            protocol_version: PROTOCOL_VERSION,
            pid: 1,
            slot: 0,
        })
        .expect("encode Hello");
        let failed = postcard::to_stdvec(&ToService::Failed {
            reason: "r".to_owned(),
            hint: "h".to_owned(),
        })
        .expect("encode Failed");
        let playing = postcard::to_stdvec(&ToService::Playing).expect("encode Playing");

        assert_eq!(hello[0], 0);
        assert_eq!(failed[0], 1);
        assert_eq!(playing, [2]);
    }

    #[test]
    fn pause_and_volume_keep_their_wire_index_and_survive_the_wire() {
        for message in [
            ToRenderer::Pause(true),
            ToRenderer::Pause(false),
            ToRenderer::SetVolume(40),
        ] {
            let bytes = postcard::to_stdvec(&message).expect("encode");
            assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
        }
        // Appended after Close (0) and Assign (1).
        assert_eq!(postcard::to_stdvec(&ToRenderer::Pause(true)).expect("encode")[0], 2);
        assert_eq!(postcard::to_stdvec(&ToRenderer::SetVolume(40)).expect("encode")[0], 3);
    }

    #[test]
    fn set_rect_keeps_its_wire_index_and_survives_the_wire() {
        let message = ToRenderer::SetRect([-1920, 0, 0, 1080]);

        let bytes = postcard::to_stdvec(&message).expect("encode");

        // Appended after Close (0), Assign (1), Pause (2) and SetVolume (3).
        assert_eq!(bytes[0], 4);
        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
    }

    #[test]
    fn a_failure_report_survives_the_wire() {
        let message = ToService::Failed {
            reason: "libmpv could not be found".to_owned(),
            hint: "reinstall".to_owned(),
        };

        let bytes = postcard::to_stdvec(&message).expect("encode");

        assert_eq!(postcard::from_bytes::<ToService>(&bytes).expect("decode"), message);
    }

    #[test]
    fn an_assignment_survives_the_wire() {
        let message = ToRenderer::Assign(Assignment {
            slot: 0,
            monitor: "\\\\.\\DISPLAY1".to_owned(),
            rect: [-1920, 0, 0, 1080],
            source: "C:\\themes\\clip.mp4".to_owned(),
            settings: PlaybackSettings::default(),
        });

        let bytes = postcard::to_stdvec(&message).expect("encode");

        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
    }
}
