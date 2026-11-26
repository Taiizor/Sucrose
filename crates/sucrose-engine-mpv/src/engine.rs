//! The mpv player behind the [`Engine`] trait: what the renderer used to do with a [`Player`] directly (load the
//! library, assemble the options, start the player), plus the clock reads the follower needs.

use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        mpsc::{Receiver, RecvTimeoutError},
    },
    thread,
    time::Duration,
};

use sucrose_core::{
    paths::{cache_dir, legacy_mpv_cache_dir},
    settings::Stretch,
};
use sucrose_engine::{
    PauseLevel,
    engine::{Engine, EngineClock, EngineError, Started},
    sync::frame_period,
};
use sucrose_ipc::protocol::{Assignment, PlaybackSettings};

use crate::{
    config::{embedding_options, generated_options, shader_cache_options, user_config_path},
    loader::{self, LoadError},
    player::{OpenOptions, Player, PlayerError, PlayerEvent},
};

/// The engine's name in the log.
const NAME: &str = "Mpv";

/// An mpv player embedded in the desktop surface, playing a video or GIF.
#[derive(Debug)]
pub struct MpvEngine {
    player: Player,
    /// The player's event channel; only the clock thread waits on it.
    events: Mutex<Option<Receiver<PlayerEvent>>>,
}

/// Loads the bundled libmpv (only the renderer does, D-17) and starts the player embedded in the window `wid`, with
/// the options the assignment's settings lead to, or the user's own mpv config when there is one (D-10). mpv follows
/// its parent window by itself, so no [`SurfacePump`](sucrose_engine::SurfacePump) is returned. Must be called on the
/// thread that pumps the surface's messages: mpv hooks that thread to resize its child.
pub fn start(wid: u64, assignment: &Assignment) -> Result<Started, EngineError> {
    let mpv = Arc::new(loader::load_default().map_err(load_failure)?);
    let shader_cache = shader_cache_dir();
    // The shader cache is always redirected, so that a user config cannot send it into the C# app's MpvPlayer folder.
    let mut embedding = embedding_options();
    embedding.extend(shader_cache_options(&shader_cache));
    // No audio is decoded until the service names this renderer the master clock (D-02): followers never play it.
    embedding.push(("aid", "no".to_owned()));
    let generated = generated_options(&assignment.settings, &shader_cache);
    // Only the path is taken from the C# app's folder; mpv reads the file and nothing is written there (D-08).
    let user_config = legacy_mpv_cache_dir().and_then(|dir| user_config_path(&dir));
    let options = OpenOptions {
        embedding: &embedding,
        generated: &generated,
        user_config: user_config.as_deref(),
    };
    let player = Player::open(mpv, wid, Path::new(&assignment.source), &options).map_err(player_failure)?;
    let events = Mutex::new(player.take_events());
    Ok(Started {
        engine: Arc::new(MpvEngine { player, events }),
        pump: None,
    })
}

/// The failure of loading libmpv, as the reason and repair hint the service shows (D-20). The full search list is in
/// the log; the user only needs to know the file is missing.
fn load_failure(error: LoadError) -> EngineError {
    tracing::error!(%error, hint = error.hint(), "libmpv cannot be loaded");
    let reason = match &error {
        LoadError::NotFound { .. } => "libmpv was not found.".to_owned(),
        other => other.to_string(),
    };
    EngineError::Missing {
        reason,
        hint: error.hint().to_owned(),
    }
}

/// The failure of creating the player or starting the file.
fn player_failure(error: PlayerError) -> EngineError {
    tracing::error!(%error, "cannot start the player");
    EngineError::Start {
        reason: error.to_string(),
    }
}

/// A command the player refused: the host logs it and goes on.
fn refused(error: PlayerError) -> EngineError {
    EngineError::Start {
        reason: error.to_string(),
    }
}

/// The folder for mpv's shader cache, inside Sucrose's own data (SUCROSE_DATA_DIR when set), created when missing.
fn shader_cache_dir() -> PathBuf {
    let dir = cache_dir().unwrap_or_else(std::env::temp_dir).join("mpv-shaders");
    if let Err(error) = std::fs::create_dir_all(&dir) {
        tracing::warn!(%error, path = %dir.display(), "cannot create the shader cache folder");
    }
    dir
}

impl Engine for MpvEngine {
    fn name(&self) -> &'static str {
        NAME
    }

    fn set_paused(&self, paused: bool, _level: PauseLevel) -> Result<(), EngineError> {
        // Heavy is handled like Light until Round D: mpv keeps its decoder and only stops advancing.
        self.player.set_paused(paused).map_err(refused)
    }

    fn set_volume(&self, volume: u8) -> Result<(), EngineError> {
        self.player.set_volume(volume).map_err(refused)
    }

    fn set_loop(&self, looping: bool) -> Result<(), EngineError> {
        self.player.set_looping(looping).map_err(refused)
    }

    fn set_stretch(&self, stretch: Stretch) -> Result<(), EngineError> {
        self.player.set_stretch(stretch).map_err(refused)
    }

    fn set_audio(&self, on: bool) -> Result<(), EngineError> {
        self.player.set_audio(on).map_err(refused)
    }

    fn load_source(&self, source: &Path, settings: &PlaybackSettings) -> Result<(), EngineError> {
        self.player.load_source(source, settings).map_err(refused)
    }

    fn clock(&self) -> Option<&dyn EngineClock> {
        Some(self)
    }
}

impl EngineClock for MpvEngine {
    fn position_s(&self) -> Option<f64> {
        self.player.time_pos()
    }

    fn duration_s(&self) -> Option<f64> {
        self.player.duration()
    }

    /// From the container's frame rate, else mpv's estimate, else a default; only the container's value is final.
    fn frame_period(&self) -> (f64, bool) {
        let read = |name: &str| {
            self.player
                .get_property_string(name)
                .and_then(|value| value.trim().parse::<f64>().ok())
                .filter(|fps| fps.is_finite() && *fps > 0.0)
        };
        let container = read("container-fps");
        let fps = container.or_else(|| read("estimated-vf-fps"));
        (frame_period(fps), container.is_some())
    }

    fn speed(&self) -> f64 {
        self.player.speed()
    }

    fn set_speed(&self, speed: f64) -> Result<(), EngineError> {
        self.player.set_speed(speed).map_err(refused)
    }

    fn seek_exact(&self, position_s: f64) -> Result<(), EngineError> {
        self.player.seek_exact(position_s).map_err(refused)
    }

    fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    fn has_started(&self) -> bool {
        self.player.has_started()
    }

    fn start_failure(&self) -> Option<String> {
        self.player.start_failure()
    }

    fn loop_gen(&self) -> u32 {
        self.player.loop_gen()
    }

    fn wait_event(&self, timeout: Duration) -> bool {
        let guard = self.events.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(events) = guard.as_ref() else {
            drop(guard);
            thread::sleep(timeout);
            return false;
        };
        match events.recv_timeout(timeout) {
            Ok(_) => {
                // One wake covers a burst of events.
                while events.try_recv().is_ok() {}
                true
            }
            Err(RecvTimeoutError::Timeout) => false,
            Err(RecvTimeoutError::Disconnected) => {
                drop(guard);
                thread::sleep(timeout);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_libmpv_is_reported_as_missing_with_its_repair_hint() {
        let error = load_failure(LoadError::NotFound { searched: Vec::new() });

        let EngineError::Missing { reason, hint } = error else {
            panic!("a missing library is not Missing: {error:?}");
        };
        assert_eq!(reason, "libmpv was not found.");
        assert!(hint.contains("fetch-engines") || hint.contains("reinstall"), "{hint}");
    }

    #[test]
    fn a_player_failure_is_a_start_failure() {
        let error = player_failure(PlayerError::CreateFailed);

        assert!(
            matches!(error, EngineError::Start { ref reason } if !reason.is_empty()),
            "{error:?}"
        );
    }
}
