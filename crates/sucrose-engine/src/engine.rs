//! The seam every engine plugs into: the renderer starts one engine through these traits and never names mpv or CEF
//! again (ENG-03). An engine is a thread-safe handle ([`Engine`]) the IPC and clock threads call, plus, when the engine
//! must be driven from the thread that owns the desktop surface, a [`SurfacePump`] that thread calls.

use std::{sync::Arc, time::Duration};

use sucrose_core::{compatible::DataGroup, settings::Stretch};
use sucrose_ipc::protocol::PlaybackSettings;

use crate::PauseLevel;

/// Why an engine did not start or did not carry out a command. Every variant is a value with one reason and one repair
/// hint, never a panic (D-20); the host maps it to an exit code.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The engine's files are missing or cannot be loaded.
    #[error("{reason}")]
    Missing {
        /// One sentence saying what is missing.
        reason: String,
        /// One sentence saying what the user can do about it.
        hint: String,
    },
    /// The engine could not start, or could not carry out a command.
    #[error("{reason}")]
    Start {
        /// One sentence saying what failed.
        reason: String,
    },
    /// The engine cannot do what was asked of it (or the renderer does not contain the engine the service named).
    #[error("{what} is not supported by this engine")]
    Unsupported {
        /// What was asked for.
        what: String,
    },
}

/// What the clock thread and the follower read from an engine that plays on a timeline (a video). Engines without a
/// timeline return no clock from [`Engine::clock`].
pub trait EngineClock: Send + Sync {
    /// The playback position in seconds, or `None` before the first frame.
    fn position_s(&self) -> Option<f64>;
    /// The length of the file in seconds, or `None` while it is unknown.
    fn duration_s(&self) -> Option<f64>;
    /// One frame of the playing video in seconds and whether the value is final (it came from the container, so it
    /// cannot change); an estimate or a default is read again on the next reading.
    fn frame_period(&self) -> (f64, bool);
    /// The playback speed, 1.0 normally.
    fn speed(&self) -> f64;
    /// Sets the playback speed; the follower's drift correction uses it.
    fn set_speed(&self, speed: f64) -> Result<(), EngineError>;
    /// Jumps to `position_s` seconds, exactly.
    fn seek_exact(&self, position_s: f64) -> Result<(), EngineError>;
    /// Whether playback is paused.
    fn is_paused(&self) -> bool;
    /// Whether the first frame of the current source was shown; before that there is no clock worth publishing.
    fn has_started(&self) -> bool;
    /// Why the source could not be played, once the engine gave up on it before its first frame.
    fn start_failure(&self) -> Option<String>;
    /// How many times the position wrapped back to the start so far.
    fn loop_gen(&self) -> u32;
    /// Waits up to `timeout` for the engine to report something the clock cares about (first frame, seek, loop wrap,
    /// pause change, start failure). `true` when something happened. Only the clock thread calls it.
    fn wait_event(&self, timeout: Duration) -> bool;
}

/// A wallpaper engine, driven from the IPC reader and the clock threads.
///
/// Every method is callable from any thread. An engine whose library is bound to one thread (CEF) queues the command
/// for the thread that owns the surface and returns at once.
pub trait Engine: Send + Sync {
    /// The engine's identity, for the log.
    fn name(&self) -> &'static str;
    /// Pauses or resumes. `level` is the migrated `PausePerformanceType` (D-22): Light resumes instantly, Heavy also
    /// frees resources; until Round D every engine treats Heavy like Light, as Phase 2 did. No engine suspends a
    /// process or attaches a debugger to pause (PLAY-05).
    fn set_paused(&self, paused: bool, level: PauseLevel) -> Result<(), EngineError>;
    /// Sets the playback volume, 0..=100.
    fn set_volume(&self, volume: u8) -> Result<(), EngineError>;
    /// Sets whether the wallpaper loops (PLAY-01).
    fn set_loop(&self, looping: bool) -> Result<(), EngineError>;
    /// Sets the value of the `ShuffleMode` call of a web page (PLAY-01). Only a web engine has such a call; every other
    /// engine ignores it.
    fn set_shuffle(&self, _shuffle: bool) {}
    /// Sets how the wallpaper is fitted to the surface (DESK-05).
    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError>;
    /// Turns audio output on or off; only the master clock plays audio (D-02). Engines without audio ignore it.
    fn set_audio(&self, _on: bool) -> Result<(), EngineError> {
        Ok(())
    }
    /// Plays `source` instead of the current file, without restarting the engine (D-26). `source` is an absolute path
    /// the service resolved after the theme gate. An engine that cannot returns [`EngineError::Unsupported`] and the
    /// service restarts the renderer instead.
    fn load_source(&self, source: &std::path::Path, settings: &PlaybackSettings) -> Result<(), EngineError>;
    /// The engine's clock, when it plays on a timeline.
    fn clock(&self) -> Option<&dyn EngineClock>;
    /// Shows the developer tools of the page this engine plays (D-13). Only a web engine with developer tools does;
    /// every other engine answers [`EngineError::Unsupported`], which the renderer logs.
    fn open_devtools(&self) -> Result<(), EngineError> {
        Err(EngineError::Unsupported {
            what: "Developer tools".to_owned(),
        })
    }
    /// Hands the engine a snapshot of the system data its page asked for, each group with its JSON text (D-24). Only a
    /// web engine has a page to give it to; every other engine ignores it.
    fn push_data(&self, _data: Vec<(DataGroup, String)>) {}
}

/// The part of an engine that must run on the thread that owns the desktop surface (for CEF the thread that pumps its
/// message loop). The renderer's message loop calls it; it is deliberately not `Send`.
pub trait SurfacePump {
    /// The loop was woken: apply what other threads queued, then do the engine's own work.
    fn on_wake(&mut self);
    /// A loop timer fired. `true` when the pump owns that timer id.
    fn on_timer(&mut self, id: u32) -> bool;
    /// The surface moved onto `rect` (left, top, right, bottom, physical pixels): move the engine's own window too.
    fn set_rect(&mut self, rect: [i32; 4]);
    /// Closes the engine's window and releases its resources; called before the surface is dropped.
    fn close(self: Box<Self>);
}

/// A started engine: the thread-safe handle and, when the engine needs one, its surface-thread pump.
pub struct Started {
    /// The handle the IPC reader and the clock thread use.
    pub engine: Arc<dyn Engine>,
    /// The pump the message loop drives, for engines bound to the surface thread.
    pub pump: Option<Box<dyn SurfacePump>>,
}

impl std::fmt::Debug for Started {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Started")
            .field("engine", &self.engine.name())
            .field("pump", &self.pump.is_some())
            .finish()
    }
}
