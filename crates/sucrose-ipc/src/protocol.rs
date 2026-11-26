//! The messages the service and its renderers exchange (D-009, D-021).
//!
//! Frames carry postcard-encoded values, and postcard encodes an enum variant by its index. Variants are therefore
//! **only ever appended** at the end of these enums, never inserted, reordered or removed; a renderer and a service
//! of different builds then still agree on every variant they both know. [`PROTOCOL_VERSION`] is bumped only for a
//! change that appending cannot express.

use serde::{Deserialize, Serialize};
use sucrose_core::{
    settings::{EngineType, PauseKind, Stretch},
    theme::WallpaperType,
};

use crate::portal::{Launch, PortalPage};

/// Version of this protocol, sent in [`ToService::Hello`] so the service can refuse a renderer it cannot talk to.
pub const PROTOCOL_VERSION: u16 = 6;

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
    /// The master's playback clock, about once a second and right after a seek, loop wrap, pause or resume. The
    /// service relays it to the followers and ignores it from any renderer that is not the master (D-02).
    Clock(ClockTick),
    /// First message of a Portal's connection (D-033): which Portal protocol it speaks, who it is and which page it
    /// was opened on. The service answers with a [`crate::portal::PortalEvent`].
    PortalHello {
        /// The Portal's [`crate::portal::PORTAL_PROTOCOL_VERSION`].
        portal_protocol_version: u16,
        /// Process id of the Portal, reported to a second Portal that finds this one open.
        pid: u32,
        /// The page the Portal was asked to open on.
        page: Option<PortalPage>,
    },
    /// First message of a launcher's connection: what it asks of the service.
    Launch(Launch),
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
    /// Whether this renderer is the master clock (plays the audio) or a follower; sent after every [`ToRenderer::Assign`]
    /// and whenever the election changes (D-02).
    SetRole(Role),
    /// The master's clock, for a follower to correct its drift against.
    Clock(ClockTick),
    /// Play this file instead of the current one, in the running renderer and without restarting it: the Portal's
    /// "Use" switched the wallpaper (D-26). `source` is the absolute path the service resolved after the theme gate;
    /// `settings` are the playback settings to apply on the way.
    LoadSource {
        /// The absolute path of the file to play.
        source: String,
        /// How to play it.
        settings: PlaybackSettings,
    },
}

/// What a renderer does when several monitors show the same wallpaper (D-02).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The reference clock: decodes and plays the audio and publishes its position.
    Master,
    /// Follows the master's position and plays no audio.
    Follower,
}

/// One reading of the master's clock.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ClockTick {
    /// Playback position in seconds when the reading was taken.
    pub position_s: f64,
    /// `sucrose_platform::monotonic_ns` at that moment; every process reads the same clock.
    pub stamp_ns: u64,
    /// The master's playback speed, 1.0 normally.
    pub speed: f64,
    /// Whether the master was paused, in which case followers do not chase it.
    pub paused: bool,
    /// How many times the master looped or jumped back to the start so far.
    pub loop_gen: u32,
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
    /// How a pause is carried out (the migrated `PausePerformanceType`): the level every engine's pause receives.
    pub pause: PauseKind,
}

impl Default for PlaybackSettings {
    /// The legacy defaults: full volume, looping, UniformToFill, hardware decoding on.
    fn default() -> Self {
        Self {
            volume: 100,
            looping: true,
            stretch: Stretch::default(),
            hardware_decoding: true,
            pause: PauseKind::default(),
        }
    }
}

/// How the web engines are configured for a renderer, taken from the Engine section (D-12).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WebOptions {
    /// Whether the web engine accepts invalid TLS certificates (`Engine.AllowInvalidCertificates`). Off by default: the
    /// engine validates certificates and rejects the rest.
    pub allow_invalid_certificates: bool,
}

/// One wallpaper job for a renderer: a monitor and the file or page to show on it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The renderer slot this job is for, as in [`ToService::Hello`].
    pub slot: u8,
    /// The monitor's device name.
    pub monitor: String,
    /// The surface rectangle in physical pixels: left, top, right, bottom (the monitor or its work area).
    pub rect: [i32; 4],
    /// The absolute path of the file to play, or of the page of a Web theme.
    pub source: String,
    /// How to play it.
    pub settings: PlaybackSettings,
    /// The wallpaper type the engine plays.
    pub kind: WallpaperType,
    /// The canonical legacy name of the engine that plays this wallpaper, chosen by the service (D-11). The renderer
    /// maps it with `EngineId::from_legacy` and starts that engine, or refuses one its build does not contain.
    pub engine: EngineType,
    /// The absolute path of the theme folder; a Web theme's page and its assets live under it.
    pub theme_dir: String,
    /// How the web engines behave for this job; the other engines ignore it.
    pub web: WebOptions,
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
            kind: WallpaperType::Video,
            engine: EngineType::MpvPlayerLive,
            theme_dir: "C:\\themes\\a".to_owned(),
            web: WebOptions::default(),
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
    fn portal_hello_and_launch_keep_their_wire_index_and_survive_the_wire() {
        let hello = ToService::PortalHello {
            portal_protocol_version: crate::portal::PORTAL_PROTOCOL_VERSION,
            pid: 7,
            page: Some(PortalPage::General),
        };
        let launch = ToService::Launch(Launch::OpenPortal { page: None });

        // Appended after Hello (0), Failed (1), Playing (2) and Clock (3).
        let bytes = postcard::to_stdvec(&hello).expect("encode");
        assert_eq!(bytes[0], 4);
        assert_eq!(postcard::from_bytes::<ToService>(&bytes).expect("decode"), hello);
        let bytes = postcard::to_stdvec(&launch).expect("encode");
        assert_eq!(bytes[0], 5);
        assert_eq!(postcard::from_bytes::<ToService>(&bytes).expect("decode"), launch);
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
    fn clock_and_role_messages_keep_their_wire_index_and_survive_the_wire() {
        let tick = ClockTick {
            position_s: 12.5,
            stamp_ns: 987_654_321,
            speed: 1.0,
            paused: false,
            loop_gen: 3,
        };
        let to_service = ToService::Clock(tick);
        let role = ToRenderer::SetRole(Role::Follower);
        let clock = ToRenderer::Clock(tick);

        // Appended after Hello (0), Failed (1), Playing (2) and after SetRect (4).
        let bytes = postcard::to_stdvec(&to_service).expect("encode");
        assert_eq!(bytes[0], 3);
        assert_eq!(postcard::from_bytes::<ToService>(&bytes).expect("decode"), to_service);
        let bytes = postcard::to_stdvec(&role).expect("encode");
        assert_eq!(bytes[0], 5);
        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), role);
        let bytes = postcard::to_stdvec(&clock).expect("encode");
        assert_eq!(bytes[0], 6);
        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), clock);
    }

    #[test]
    fn load_source_keeps_its_wire_index_and_survives_the_wire() {
        let message = ToRenderer::LoadSource {
            source: "C:\\themes\\other.mp4".to_owned(),
            settings: PlaybackSettings::default(),
        };

        let bytes = postcard::to_stdvec(&message).expect("encode");

        // Appended after Close (0), Assign (1), Pause (2), SetVolume (3), SetRect (4), SetRole (5) and Clock (6).
        assert_eq!(bytes[0], 7);
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
            kind: WallpaperType::Video,
            engine: EngineType::MpvPlayerLive,
            theme_dir: "C:\\themes".to_owned(),
            web: WebOptions::default(),
        });

        let bytes = postcard::to_stdvec(&message).expect("encode");

        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
    }

    #[test]
    fn a_web_assignment_survives_the_wire() {
        let message = ToRenderer::Assign(Assignment {
            slot: 1,
            monitor: "\\\\.\\DISPLAY9".to_owned(),
            rect: [0, 0, 1920, 1080],
            source: "C:\\themes\\hello\\index.html".to_owned(),
            settings: PlaybackSettings::default(),
            kind: WallpaperType::Web,
            engine: EngineType::CefSharpLive,
            theme_dir: "C:\\themes\\hello".to_owned(),
            web: WebOptions {
                allow_invalid_certificates: true,
            },
        });

        let bytes = postcard::to_stdvec(&message).expect("encode");

        assert_eq!(bytes[0], 1);
        assert_eq!(postcard::from_bytes::<ToRenderer>(&bytes).expect("decode"), message);
    }

    #[test]
    fn web_options_validate_certificates_by_default() {
        assert!(
            !WebOptions::default().allow_invalid_certificates,
            "D-12: validation is the default"
        );
    }
}
