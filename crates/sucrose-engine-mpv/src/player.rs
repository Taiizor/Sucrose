//! One embedded mpv instance: created, hardened and started on a window handle, then driven until it is dropped.
//!
//! A [Player] owns an mpv handle. It must be opened on the thread that owns the parent window and pumps its
//! messages, because mpv hooks that thread to resize its child window (RESEARCH Pattern 2). The handle is only
//! ever touched through the thread-safe client API, and a dedicated thread is the only caller of
//! mpv_wait_event. The option lists come from [crate::config].

use std::ffi::{CStr, CString, c_char, c_int};
use std::path::Path;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use sucrose_core::settings::Stretch;
use sucrose_ipc::protocol::PlaybackSettings;

use crate::loader::Mpv;
use crate::sys;

/// Why a [Player] could not be started.
#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    /// mpv_create returned NULL.
    #[error("libmpv could not create a player instance")]
    CreateFailed,
    /// An option was refused before the player started; a hardening option that does not apply is fatal (T-02-14).
    #[error("libmpv refused the option {name}={value}: {message}")]
    Option {
        /// The option name.
        name: &'static str,
        /// The value that was refused.
        value: String,
        /// mpv's description of the error.
        message: String,
    },
    /// mpv_initialize failed.
    #[error("libmpv could not initialize the player: {message}")]
    Initialize {
        /// mpv's description of the error.
        message: String,
    },
    /// The file path cannot be handed to mpv (relative, not UTF-8 or containing a NUL).
    #[error("the media path {path} cannot be played: it must be absolute, valid UTF-8 and free of NUL characters")]
    InvalidPath {
        /// The rejected path.
        path: String,
    },
    /// The loadfile command was refused.
    #[error("libmpv refused to load {path}: {message}")]
    Load {
        /// The file that was refused.
        path: String,
        /// mpv's description of the error.
        message: String,
    },
    /// The event thread could not be started.
    #[error("cannot start the libmpv event thread: {0}")]
    Thread(#[source] std::io::Error),
    /// A property could not be set on the running player.
    #[error("libmpv refused to set {name} to {value}: {message}")]
    Property {
        /// The property name.
        name: &'static str,
        /// The value that was refused.
        value: String,
        /// mpv's description of the error.
        message: String,
    },
}

/// What the player is configured with, as assembled by the caller (see [crate::config]).
///
/// The order is fixed by [Player::open]: the user's config file or else the generated options first, the embedding
/// options last, so that a user config cannot turn the hardening off (T-02-14).
#[derive(Debug, Clone, Copy)]
pub struct OpenOptions<'a> {
    /// The embedding and hardening options; always applied, and applied last.
    pub embedding: &'a [(&'static str, String)],
    /// The look, performance and audio options; used unless the user config is loaded.
    pub generated: &'a [(&'static str, String)],
    /// The user's own mpv config (D-10); when it is loaded, the generated options are not applied.
    pub user_config: Option<&'a Path>,
}

/// The raw handle, sendable to the event thread.
#[derive(Clone, Copy)]
struct Handle(*mut sys::mpv_handle);

// SAFETY: client.h states that all client API calls on one handle may be made from any thread; this crate calls
// mpv_wait_event from exactly one thread (the event thread) and never frees the handle while that thread runs.
unsafe impl Send for Handle {}
// SAFETY: see the Send impl; the pointer is only passed to thread-safe client API calls.
unsafe impl Sync for Handle {}

/// What the event thread tells whoever drives the player (see [Player::take_events]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEvent {
    /// The first frame was shown: from now on the position is a real clock.
    Started,
    /// Playback restarted after a seek or a loop wrap, so the position jumped.
    Discontinuity,
    /// The player was paused (true) or resumed (false).
    PauseChanged(bool),
    /// The file ended with an error before its first frame (see [`Player::start_failure`]).
    StartFailed,
}

/// What happened to the playback position between two readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Jump {
    /// Ordinary playback.
    None,
    /// The position went back to the start of the file: a loop wrap, or a seek to the start.
    Wrap,
    /// Any other jump, forward or backward.
    Skip,
}

/// Where playback was at the last restart (or pause change), so that the position it should have reached at the next
/// restart can be worked out. Two restarts of a looping file are a whole loop apart, so comparing their positions
/// alone cannot tell a wrap from ordinary playback.
#[derive(Debug, Clone, Copy)]
struct Anchor {
    at: Instant,
    /// The position then, in seconds.
    position: f64,
    /// Seconds of the file that pass per second of time: the playback speed, 0 while paused.
    rate: f64,
}

impl Anchor {
    /// The position playback should have reached at `now`.
    fn expected(&self, now: Instant) -> f64 {
        self.position + now.saturating_duration_since(self.at).as_secs_f64() * self.rate
    }
}

/// Classifies the restart that found playback at `position` (seconds) at `now`, given the `anchor` of the last
/// restart. A restart near where playback should be is ordinary. A restart near the start of the file that follows
/// playback which reached the end (within a margin that also absorbs a follower's small speed changes) is a loop
/// wrap; without a known `duration` any jump back to the start after more than a second counts as one.
fn classify_restart(anchor: Option<Anchor>, now: Instant, position: f64, duration: Option<f64>) -> Jump {
    let Some(anchor) = anchor else {
        return Jump::None;
    };
    let expected = anchor.expected(now);
    if (position - expected).abs() <= 1.0 {
        return Jump::None;
    }
    if position < 1.0 {
        let wrapped = match duration.filter(|duration| duration.is_finite() && *duration > 0.0) {
            Some(duration) => expected >= duration - (1.0 + 0.1 * duration),
            None => expected > position + 1.0,
        };
        if wrapped {
            return Jump::Wrap;
        }
    }
    Jump::Skip
}

/// The properties that follow the playback settings while a file plays: volume, looping and the stretch. They use the
/// values of the generated options of [`crate::config`], so a switch looks like a fresh start with those settings.
fn runtime_properties(settings: &PlaybackSettings) -> Vec<(&'static str, String)> {
    let (panscan, keepaspect, unscaled) = match settings.stretch {
        Stretch::UniformToFill => ("1.0", "yes", "no"),
        Stretch::Fill => ("0.0", "no", "no"),
        Stretch::None => ("0.0", "yes", "yes"),
        Stretch::Uniform => ("0.0", "yes", "no"),
    };
    vec![
        ("volume", settings.volume.min(100).to_string()),
        ("loop-file", if settings.looping { "inf" } else { "no" }.to_owned()),
        ("panscan", panscan.to_owned()),
        ("keepaspect", keepaspect.to_owned()),
        ("video-unscaled", unscaled.to_owned()),
    ]
}

/// State the event thread keeps for the thread that drives the player.
#[derive(Debug, Default)]
struct Shared {
    /// The last value of the observed `pause` property.
    paused: AtomicBool,
    /// How many times the position wrapped back to the start.
    loop_gen: AtomicU32,
    /// Set at the first playback restart: the first frame was shown.
    started: AtomicBool,
    /// Set by [`Player::load_source`] and cleared at the first playback restart of the new file: until then the
    /// position and the first-frame flag belong to the file that was replaced.
    loading: AtomicBool,
    /// mpv's description of the error that ended the file before its first frame, when that happened.
    start_failure: Mutex<Option<String>>,
}

/// What a playback restart means for a source switch (see [`SourceSwitch`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Restart {
    /// A restart of a file that is not the newly requested one (a loop wrap or a finished seek of the old file).
    Old,
    /// The first frame of the file the latest [`Player::load_source`] asked for.
    NewStart,
}

/// Decides, on the event thread, which playback restart is the first frame of a file requested with
/// [`Player::load_source`].
#[derive(Debug, Default)]
struct SourceSwitch {
    requested: u64,
    pending: bool,
}

impl SourceSwitch {
    fn requested(&mut self, generation: u64) {
        if generation > self.requested {
            self.requested = generation;
            self.pending = true;
        }
    }

    fn file_loaded(&mut self, _generation: u64) {}

    fn restart(&mut self) -> Restart {
        if std::mem::take(&mut self.pending) {
            Restart::NewStart
        } else {
            Restart::Old
        }
    }
}

/// A running embedded mpv instance. Dropping it stops the event thread and destroys the instance.
pub struct Player {
    // Declared first so it is dropped last: the library must outlive every call into it.
    mpv: Arc<Mpv>,
    handle: Handle,
    stop: Arc<AtomicBool>,
    events: Option<JoinHandle<()>>,
    shared: Arc<Shared>,
    receiver: Mutex<Option<Receiver<PlayerEvent>>>,
    /// Whether the user's own mpv config was loaded: then the generated look and audio options never applied, and
    /// [`Player::load_source`] leaves those alone too.
    user_config_loaded: bool,
}

impl Player {
    /// [`Player::open`] for any mpv source string and without a window, for the integration tests that play a generated
    /// stream with `vo=null`. Only built with the `test-support` feature, so no other caller can hand theme text to
    /// `loadfile` through it.
    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn open_source_for_tests(
        mpv: Arc<Mpv>,
        wid: Option<u64>,
        source: &str,
        options: &OpenOptions<'_>,
    ) -> Result<Self, PlayerError> {
        Self::open_source(mpv, wid, source, options)
    }
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player").finish_non_exhaustive()
    }
}

impl Player {
    /// Creates an instance embedded in the window wid (the HWND of the surface), applies options and starts file.
    ///
    /// Call this on the thread that owns that window and pumps its messages. file must be absolute.
    pub fn open(mpv: Arc<Mpv>, wid: u64, file: &Path, options: &OpenOptions<'_>) -> Result<Self, PlayerError> {
        let path = file
            .to_str()
            .filter(|text| file.is_absolute() && !text.contains('\0'))
            .ok_or_else(|| PlayerError::InvalidPath {
                path: file.display().to_string(),
            })?
            .to_owned();
        Self::open_source(mpv, Some(wid), &path, options)
    }

    /// Like [Player::open], for a source that is not a file path (an mpv URL such as `av://lavfi:...`) and for an
    /// instance without a window (`wid` of None, for tests with `vo=null`).
    ///
    /// The caller vouches for `source`: only a file the theme gate accepted (see [Player::open]) is passed, never text
    /// taken from a theme (T-02-14). It is private to the crate; the integration tests reach it through
    /// [`Player::open_source_for_tests`].
    fn open_source(
        mpv: Arc<Mpv>,
        wid: Option<u64>,
        source: &str,
        options: &OpenOptions<'_>,
    ) -> Result<Self, PlayerError> {
        // SAFETY: mpv_create has no preconditions; it returns NULL on failure, which is checked right here.
        let raw = unsafe { mpv.api.mpv_create() };
        if raw.is_null() {
            return Err(PlayerError::CreateFailed);
        }
        // From here on the player owns the handle, so every early return below destroys it through Drop.
        let mut player = Self {
            mpv,
            handle: Handle(raw),
            stop: Arc::new(AtomicBool::new(false)),
            events: None,
            shared: Arc::new(Shared::default()),
            receiver: Mutex::new(None),
            user_config_loaded: false,
        };

        // The user's config first, then the generated look, then the embedding on top: see OpenOptions.
        let mut user_config_loaded = false;
        if let Some(config) = options.user_config {
            match player.load_config_file(config) {
                Ok(()) => {
                    tracing::info!(path = %config.display(), "using the user's mpv config");
                    user_config_loaded = true;
                }
                Err(message) => {
                    tracing::warn!(path = %config.display(), %message, "the user's mpv config cannot be used, the generated options apply instead");
                }
            }
        }
        player.user_config_loaded = user_config_loaded;
        if !user_config_loaded {
            for (name, value) in options.generated {
                player.set_option(name, value)?;
            }
        }
        for (name, value) in options.embedding {
            player.set_option(name, value)?;
        }
        if let Some(wid) = wid {
            // mpv takes the low 32 bits of the HWND as a decimal number (all Windows handles fit in 32 bits).
            #[allow(clippy::cast_possible_truncation)]
            let wid_value = (wid as u32).to_string();
            player.set_option("wid", &wid_value)?;
        }

        // SAFETY: the handle is valid and not yet initialized, which is what mpv_initialize requires.
        let status = unsafe { player.mpv.api.mpv_initialize(player.handle.0) };
        if status < 0 {
            return Err(PlayerError::Initialize {
                message: player.error_text(status),
            });
        }
        // The level argument is a NUL-terminated string; a literal outlives the call.
        // SAFETY: the handle is valid and initialized; "warn" is a valid level name.
        let _ = unsafe {
            player
                .mpv
                .api
                .mpv_request_log_messages(player.handle.0, c"warn".as_ptr())
        };

        player.observe_pause();
        player.start_event_thread()?;
        player.load(source)?;
        Ok(player)
    }

    /// Plays `path` instead of the current file on this running player, without restarting it (D-26): `loadfile` with
    /// `replace`. The playback settings that can change while playing (volume, loop, stretch) are applied first, with the
    /// values [`crate::config`] produces for them, unless the user's own mpv config is in charge; hardware decoding keeps
    /// the value the player was opened with. A refused setting is logged and does not stop the switch.
    ///
    /// `path` must be absolute: only a file the theme gate accepted is passed, never text taken from a theme (T-03-24).
    /// Until the first frame of the new file is shown, [`Player::has_started`] is false and the event thread reports the
    /// new file as a new start (a `Started` event), not as a jump of the old one.
    pub fn load_source(&self, path: &Path, settings: &PlaybackSettings) -> Result<(), PlayerError> {
        let text = path
            .to_str()
            .filter(|text| path.is_absolute() && !text.contains('\0'))
            .ok_or_else(|| PlayerError::InvalidPath {
                path: path.display().to_string(),
            })?;
        if !self.user_config_loaded {
            for (name, value) in runtime_properties(settings) {
                if let Err(error) = self.set_property(name, value.as_str()) {
                    tracing::warn!(%error, "cannot apply a playback setting to the new source");
                }
            }
        }
        self.shared.loading.store(true, Ordering::SeqCst);
        if let Err(error) = self.load(text) {
            self.shared.loading.store(false, Ordering::SeqCst);
            return Err(error);
        }
        Ok(())
    }

    /// Pauses (true) or resumes (false) playback.
    pub fn set_paused(&self, paused: bool) -> Result<(), PlayerError> {
        self.set_property("pause", if paused { "yes" } else { "no" })
    }

    /// Sets the playback volume; values above 100 are treated as 100 (volume-max is 100).
    pub fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.set_property("volume", &volume.min(100).to_string())
    }

    /// Sets the playback speed (1.0 is normal); the follower's drift correction uses a speed within 5 % of the master's (see `sync.rs`, D-02).
    pub fn set_speed(&self, speed: f64) -> Result<(), PlayerError> {
        self.set_property("speed", &format!("{speed:.4}"))
    }

    /// Jumps to `position_s` seconds, exactly and not to the nearest keyframe.
    pub fn seek_exact(&self, position_s: f64) -> Result<(), PlayerError> {
        let target = format!("{position_s:.3}");
        let refused = |message: String| PlayerError::Property {
            name: "seek",
            value: target.clone(),
            message,
        };
        let Ok(c_target) = CString::new(target.as_str()) else {
            return Err(refused("the position contains a NUL character".to_owned()));
        };
        let mut args: [*const c_char; 4] = [
            c"seek".as_ptr(),
            c_target.as_ptr(),
            c"absolute+exact".as_ptr(),
            ptr::null(),
        ];
        // SAFETY: the handle is valid; args is a NULL-terminated array of NUL-terminated strings that outlive the
        // call, and mpv_command copies what it needs before returning.
        let status = unsafe { self.mpv.api.mpv_command(self.handle.0, args.as_mut_ptr()) };
        if status < 0 {
            return Err(refused(self.error_text(status)));
        }
        Ok(())
    }

    /// Turns audio decoding on (`aid=auto`) or off (`aid=no`). Only the master clock plays audio (D-02).
    pub fn set_audio(&self, on: bool) -> Result<(), PlayerError> {
        self.set_property("aid", if on { "auto" } else { "no" })
    }

    /// The playback position in seconds, or None before the first frame.
    #[must_use]
    pub fn time_pos(&self) -> Option<f64> {
        self.get_property_seconds("time-pos")
    }

    /// The length of the file in seconds, or None while it is unknown.
    #[must_use]
    pub fn duration(&self) -> Option<f64> {
        self.get_property_seconds("duration").filter(|length| *length > 0.0)
    }

    /// The playback speed, 1.0 when mpv does not say.
    #[must_use]
    pub fn speed(&self) -> f64 {
        self.get_property_seconds("speed")
            .filter(|speed| *speed > 0.0)
            .unwrap_or(1.0)
    }

    /// Whether playback is paused, as the event thread last saw it.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.shared.paused.load(Ordering::SeqCst)
    }

    /// Whether the first frame was shown. Before that the player has no clock worth publishing or following.
    #[must_use]
    pub fn has_started(&self) -> bool {
        self.shared.started.load(Ordering::SeqCst) && !self.shared.loading.load(Ordering::SeqCst)
    }

    /// Why the file could not be played, once mpv ended it with an error before the first frame (the file cannot be
    /// opened or decoded). `loadfile` is asynchronous, so [`Player::open`] succeeding does not mean the file plays.
    #[must_use]
    pub fn start_failure(&self) -> Option<String> {
        self.shared
            .start_failure
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// How many times the position wrapped back to the start (a loop, or a seek to the start).
    #[must_use]
    pub fn loop_gen(&self) -> u32 {
        self.shared.loop_gen.load(Ordering::SeqCst)
    }

    /// The channel of [PlayerEvent]s; there is one receiver, so only the first call gets it.
    #[must_use]
    pub fn take_events(&self) -> Option<Receiver<PlayerEvent>> {
        self.receiver.lock().unwrap_or_else(PoisonError::into_inner).take()
    }

    fn get_property_seconds(&self, name: &str) -> Option<f64> {
        self.get_property_string(name)?
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    }

    /// Asks mpv to report changes of `pause`. A refusal only costs the immediate clock tick on a pause.
    fn observe_pause(&self) {
        // SAFETY: the handle is valid and initialized; the name is a NUL-terminated literal; the userdata is only
        // echoed back in events.
        let status = unsafe {
            self.mpv
                .api
                .mpv_observe_property(self.handle.0, 1, c"pause".as_ptr(), sys::mpv_format_MPV_FORMAT_FLAG)
        };
        if status < 0 {
            tracing::warn!(error = %self.error_text(status), "cannot observe the pause property");
        }
    }

    /// The current value of a property as mpv formats it, or None when it is unavailable.
    #[must_use]
    pub fn get_property_string(&self, name: &str) -> Option<String> {
        property_string(&self.mpv, self.handle, name)
    }

    fn set_property(&self, name: &'static str, value: &str) -> Result<(), PlayerError> {
        let refused = |message: String| PlayerError::Property {
            name,
            value: value.to_owned(),
            message,
        };
        let (Ok(c_name), Ok(c_value)) = (CString::new(name), CString::new(value)) else {
            return Err(refused("the name or value contains a NUL character".to_owned()));
        };
        // SAFETY: the handle is valid for the life of self; both strings are NUL-terminated and outlive the call,
        // and mpv_set_property_string is part of the thread-safe client API.
        let status = unsafe {
            self.mpv
                .api
                .mpv_set_property_string(self.handle.0, c_name.as_ptr(), c_value.as_ptr())
        };
        if status < 0 {
            return Err(refused(self.error_text(status)));
        }
        Ok(())
    }

    /// Loads the user's mpv config into this not yet initialized instance; the error is mpv's description.
    fn load_config_file(&self, config: &Path) -> Result<(), String> {
        let text = config
            .to_str()
            .ok_or_else(|| "the path is not valid UTF-8".to_owned())?;
        let c_path = CString::new(text).map_err(|_| "the path contains a NUL character".to_owned())?;
        // SAFETY: the handle is valid and not yet initialized (mpv_load_config_file may be called before
        // mpv_initialize); the path is NUL-terminated and outlives the call. The file is only read.
        let status = unsafe { self.mpv.api.mpv_load_config_file(self.handle.0, c_path.as_ptr()) };
        if status < 0 {
            return Err(self.error_text(status));
        }
        Ok(())
    }

    fn set_option(&self, name: &'static str, value: &str) -> Result<(), PlayerError> {
        let refused = |message: String| PlayerError::Option {
            name,
            value: value.to_owned(),
            message,
        };
        let (Ok(c_name), Ok(c_value)) = (CString::new(name), CString::new(value)) else {
            return Err(refused("the name or value contains a NUL character".to_owned()));
        };
        // SAFETY: the handle is valid; both strings are NUL-terminated and live until the call returns.
        let status = unsafe {
            self.mpv
                .api
                .mpv_set_option_string(self.handle.0, c_name.as_ptr(), c_value.as_ptr())
        };
        if status < 0 {
            return Err(refused(self.error_text(status)));
        }
        Ok(())
    }

    fn load(&self, path: &str) -> Result<(), PlayerError> {
        let refused = |message: String| PlayerError::Load {
            path: path.to_owned(),
            message,
        };
        let Ok(c_path) = CString::new(path) else {
            return Err(refused("the path contains a NUL character".to_owned()));
        };
        let mut args: [*const c_char; 4] = [c"loadfile".as_ptr(), c_path.as_ptr(), c"replace".as_ptr(), ptr::null()];
        // SAFETY: the handle is valid; args is a NULL-terminated array of NUL-terminated strings that outlive the
        // call, and mpv_command copies what it needs before returning.
        let status = unsafe { self.mpv.api.mpv_command(self.handle.0, args.as_mut_ptr()) };
        if status < 0 {
            return Err(refused(self.error_text(status)));
        }
        Ok(())
    }

    fn error_text(&self, status: c_int) -> String {
        error_text(&self.mpv, status)
    }

    fn start_event_thread(&mut self) -> Result<(), PlayerError> {
        let mpv = Arc::clone(&self.mpv);
        let handle = self.handle;
        let stop = Arc::clone(&self.stop);
        let shared = Arc::clone(&self.shared);
        let (sender, receiver) = mpsc::channel();
        *self.receiver.lock().unwrap_or_else(PoisonError::into_inner) = Some(receiver);
        let thread = thread::Builder::new()
            .name("mpv-events".to_owned())
            .spawn(move || event_loop(&mpv, handle, &stop, &shared, &sender))
            .map_err(PlayerError::Thread)?;
        self.events = Some(thread);
        Ok(())
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(events) = self.events.take() {
            // SAFETY: the handle is valid until mpv_terminate_destroy below; mpv_wakeup is thread-safe and a wakeup
            // that arrives before the next mpv_wait_event is not lost (client.h).
            unsafe { self.mpv.api.mpv_wakeup(self.handle.0) };
            if events.join().is_err() {
                tracing::warn!("the libmpv event thread panicked");
            }
        }
        // SAFETY: the event thread is gone, so nothing else uses the handle; it is never used after this call, and
        // the library stays loaded because self.mpv is dropped after this function returns.
        unsafe { self.mpv.api.mpv_terminate_destroy(self.handle.0) };
    }
}

/// A property as a string (freed with mpv_free), or None when mpv has no value for it.
fn property_string(mpv: &Mpv, handle: Handle, name: &str) -> Option<String> {
    let c_name = CString::new(name).ok()?;
    // SAFETY: the handle is valid for the caller's lifetime; the name is NUL-terminated and outlives the call.
    let raw = unsafe { mpv.api.mpv_get_property_string(handle.0, c_name.as_ptr()) };
    if raw.is_null() {
        return None;
    }
    // SAFETY: a non-NULL result is a NUL-terminated string owned by the caller until mpv_free.
    let text = unsafe { CStr::from_ptr(raw) }.to_string_lossy().into_owned();
    // SAFETY: raw came from mpv_get_property_string and is freed exactly once, here.
    unsafe { mpv.api.mpv_free(raw.cast()) };
    Some(text)
}

/// The text of an mpv error code.
fn error_text(mpv: &Mpv, status: c_int) -> String {
    // SAFETY: mpv_error_string accepts any code and returns a static NUL-terminated string (client.h).
    let text = unsafe { mpv.api.mpv_error_string(status) };
    if text.is_null() {
        return format!("error {status}");
    }
    // SAFETY: the pointer was just checked and points to a static NUL-terminated string.
    unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned()
}

/// Reads events until mpv shuts down or the player is dropped; the only caller of mpv_wait_event.
fn event_loop(mpv: &Mpv, handle: Handle, stop: &AtomicBool, shared: &Shared, events: &Sender<PlayerEvent>) {
    let mut started = false;
    // Where playback was at the last restart or pause change: only this thread reads or writes it.
    let mut anchor: Option<Anchor> = None;
    while !stop.load(Ordering::SeqCst) {
        // SAFETY: the handle stays valid until this thread is joined; the returned event is only read before the
        // next mpv_wait_event call, as client.h requires.
        let event = unsafe { mpv.api.mpv_wait_event(handle.0, -1.0) };
        // SAFETY: a non-NULL pointer from mpv_wait_event points to a valid event (client.h).
        let Some(event) = (unsafe { event.as_ref() }) else {
            continue;
        };
        match event.event_id {
            sys::mpv_event_id_MPV_EVENT_SHUTDOWN => {
                tracing::debug!("libmpv shut down");
                return;
            }
            // The decoder exists once the first frame is about to show, so hwdec-current is only meaningful here.
            // A later restart follows a seek or a loop wrap: the position jumped, so the clock is due.
            sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART if started => {
                if shared.loading.swap(false, Ordering::SeqCst) {
                    // The first frame of a file loaded with `load_source`: a new start. Its position is not a jump
                    // of the old file, so nothing is classified, the anchor starts afresh and the driver is told.
                    anchor = take_anchor(mpv, handle, shared);
                    tracing::info!("playback started on the new source");
                    let _ = events.send(PlayerEvent::Started);
                } else {
                    anchor = note_restart(mpv, handle, shared, events, anchor);
                }
            }
            sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => {
                let was_paused = shared.paused.load(Ordering::SeqCst);
                note_property(event, shared, events);
                // The expected position stops (or starts) advancing here, so it is worked out from this moment on.
                if shared.paused.load(Ordering::SeqCst) != was_paused && anchor.is_some() {
                    anchor = take_anchor(mpv, handle, shared).or(anchor);
                }
            }
            sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART => {
                started = true;
                anchor = take_anchor(mpv, handle, shared);
                let hwdec = property_string(mpv, handle, "hwdec-current")
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "no".to_owned());
                tracing::info!(%hwdec, "playback started");
                // Set before the event is sent, so whoever receives Started also sees has_started.
                shared.started.store(true, Ordering::SeqCst);
                let _ = events.send(PlayerEvent::Started);
            }
            sys::mpv_event_id_MPV_EVENT_END_FILE => {
                if let Some(message) = log_end_file(mpv, event)
                    && (!started || shared.loading.load(Ordering::SeqCst))
                {
                    // The file never reached its first frame: whoever drives the player has to know.
                    *shared.start_failure.lock().unwrap_or_else(PoisonError::into_inner) = Some(message);
                    let _ = events.send(PlayerEvent::StartFailed);
                }
            }
            sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => log_message(event),
            _ => {}
        }
    }
}

/// The anchor of the position playback has now, or `None` when mpv reports none.
fn take_anchor(mpv: &Mpv, handle: Handle, shared: &Shared) -> Option<Anchor> {
    let position = number_property(mpv, handle, "time-pos")?;
    let speed = number_property(mpv, handle, "speed")
        .filter(|speed| *speed > 0.0)
        .unwrap_or(1.0);
    let rate = if shared.paused.load(Ordering::SeqCst) {
        0.0
    } else {
        speed
    };
    Some(Anchor {
        at: Instant::now(),
        position,
        rate,
    })
}

fn number_property(mpv: &Mpv, handle: Handle, name: &str) -> Option<f64> {
    property_string(mpv, handle, name)
        .and_then(|text| text.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
}

/// A restart after the first one: works out whether playback wrapped to the start of a loop, counts it and tells the
/// driver. Returns the anchor for the next restart.
fn note_restart(
    mpv: &Mpv,
    handle: Handle,
    shared: &Shared,
    events: &Sender<PlayerEvent>,
    anchor: Option<Anchor>,
) -> Option<Anchor> {
    let Some(next) = take_anchor(mpv, handle, shared) else {
        return anchor;
    };
    let duration = number_property(mpv, handle, "duration");
    if classify_restart(anchor, next.at, next.position, duration) == Jump::Wrap {
        shared.loop_gen.fetch_add(1, Ordering::SeqCst);
    }
    // A closed channel means nobody listens (a follower, or the player is being dropped): nothing to do.
    let _ = events.send(PlayerEvent::Discontinuity);
    Some(next)
}

/// A change of the observed `pause` property.
fn note_property(event: &sys::mpv_event, shared: &Shared, events: &Sender<PlayerEvent>) {
    // SAFETY: for PROPERTY_CHANGE, data is NULL or points to an mpv_event_property valid until the next wait.
    let Some(property) = (unsafe { event.data.cast::<sys::mpv_event_property>().as_ref() }) else {
        return;
    };
    if property.format != sys::mpv_format_MPV_FORMAT_FLAG || property.data.is_null() {
        return;
    }
    // SAFETY: for MPV_FORMAT_FLAG, data points to a C int that stays valid until the next mpv_wait_event.
    let paused = unsafe { property.data.cast::<c_int>().read() } != 0;
    shared.paused.store(paused, Ordering::SeqCst);
    let _ = events.send(PlayerEvent::PauseChanged(paused));
}

/// Logs the end of a file; the error text when it ended with an error.
fn log_end_file(mpv: &Mpv, event: &sys::mpv_event) -> Option<String> {
    // SAFETY: for END_FILE, data is NULL or points to an mpv_event_end_file valid until the next mpv_wait_event.
    let end = (unsafe { event.data.cast::<sys::mpv_event_end_file>().as_ref() })?;
    if end.reason == sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR {
        let error = error_text(mpv, end.error);
        tracing::warn!(%error, "playback ended with an error");
        Some(error)
    } else {
        tracing::debug!(reason = end.reason, "playback ended");
        None
    }
}

fn log_message(event: &sys::mpv_event) {
    // SAFETY: for LOG_MESSAGE, data is NULL or points to an mpv_event_log_message valid until the next wait.
    let Some(message) = (unsafe { event.data.cast::<sys::mpv_event_log_message>().as_ref() }) else {
        return;
    };
    let text = c_text(message.text);
    let text = text.trim_end();
    let prefix = c_text(message.prefix);
    match message.log_level {
        ..=sys::mpv_log_level_MPV_LOG_LEVEL_ERROR => tracing::error!(target: "libmpv", %prefix, "{text}"),
        sys::mpv_log_level_MPV_LOG_LEVEL_WARN => tracing::warn!(target: "libmpv", %prefix, "{text}"),
        sys::mpv_log_level_MPV_LOG_LEVEL_INFO => tracing::info!(target: "libmpv", %prefix, "{text}"),
        _ => tracing::debug!(target: "libmpv", %prefix, "{text}"),
    }
}

fn c_text(text: *const c_char) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: mpv event strings are NUL-terminated and valid until the next mpv_wait_event call.
    unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// An anchor at `position` taken at `start`, playing at `rate`.
    fn anchor(start: Instant, position: f64, rate: f64) -> Option<Anchor> {
        Some(Anchor {
            at: start,
            position,
            rate,
        })
    }

    fn after(start: Instant, seconds: f64) -> Instant {
        start + Duration::from_secs_f64(seconds)
    }

    fn value_of(properties: &[(&'static str, String)], name: &str) -> Option<String> {
        properties
            .iter()
            .find_map(|(property, value)| (*property == name).then(|| value.clone()))
    }

    #[test]
    fn runtime_properties_carry_volume_and_looping_of_the_settings() {
        let settings = PlaybackSettings {
            volume: 250,
            looping: false,
            ..PlaybackSettings::default()
        };

        let properties = runtime_properties(&settings);

        assert_eq!(value_of(&properties, "volume").as_deref(), Some("100"));
        assert_eq!(value_of(&properties, "loop-file").as_deref(), Some("no"));
        let looping = runtime_properties(&PlaybackSettings::default());
        assert_eq!(value_of(&looping, "loop-file").as_deref(), Some("inf"));
    }

    #[test]
    fn each_stretch_sets_exactly_its_own_property_and_resets_the_others() {
        for (stretch, panscan, keepaspect, unscaled) in [
            (Stretch::UniformToFill, "1.0", "yes", "no"),
            (Stretch::Fill, "0.0", "no", "no"),
            (Stretch::None, "0.0", "yes", "yes"),
            (Stretch::Uniform, "0.0", "yes", "no"),
        ] {
            let properties = runtime_properties(&PlaybackSettings {
                stretch,
                ..PlaybackSettings::default()
            });

            assert_eq!(
                value_of(&properties, "panscan").as_deref(),
                Some(panscan),
                "{stretch:?}"
            );
            assert_eq!(
                value_of(&properties, "keepaspect").as_deref(),
                Some(keepaspect),
                "{stretch:?}"
            );
            assert_eq!(
                value_of(&properties, "video-unscaled").as_deref(),
                Some(unscaled),
                "{stretch:?}"
            );
        }
    }

    #[test]
    fn the_first_restart_is_no_jump() {
        assert_eq!(classify_restart(None, Instant::now(), 5.0, Some(30.0)), Jump::None);
    }

    #[test]
    fn a_restart_where_playback_should_be_is_ordinary() {
        let start = Instant::now();
        let anchor = anchor(start, 5.0, 1.0);
        assert_eq!(classify_restart(anchor, after(start, 0.4), 5.4, Some(30.0)), Jump::None);
        assert_eq!(classify_restart(anchor, after(start, 0.4), 4.6, Some(30.0)), Jump::None);
    }

    #[test]
    fn a_loop_wrap_is_found_one_whole_loop_after_the_start_of_the_last_loop() {
        // The previous restart was the wrap that began this loop, near 0 s, and so is this one, 30 s later: the
        // positions alone differ by almost nothing.
        let start = Instant::now();
        let anchor = anchor(start, 0.05, 1.0);
        assert_eq!(
            classify_restart(anchor, after(start, 29.97), 0.03, Some(30.0)),
            Jump::Wrap
        );
        // A looping file is also a wrap when it is slightly late or early.
        assert_eq!(
            classify_restart(anchor, after(start, 28.5), 0.2, Some(30.0)),
            Jump::Wrap
        );
    }

    #[test]
    fn a_seek_to_the_start_in_the_middle_of_the_file_is_not_a_wrap() {
        let start = Instant::now();
        let anchor = anchor(start, 10.0, 1.0);
        assert_eq!(classify_restart(anchor, after(start, 2.0), 0.0, Some(30.0)), Jump::Skip);
    }

    #[test]
    fn the_speed_moves_the_expected_position() {
        let start = Instant::now();
        // At 1.05x the end of a 30 s file comes after 28.6 s of time.
        let anchor = anchor(start, 0.0, 1.05);
        assert_eq!(
            classify_restart(anchor, after(start, 28.6), 0.0, Some(30.0)),
            Jump::Wrap
        );
    }

    #[test]
    fn a_paused_player_expects_no_progress() {
        let start = Instant::now();
        let anchor = anchor(start, 12.0, 0.0);
        // A restart near 12 s after a long pause is ordinary; one at the start is a seek, not a wrap.
        assert_eq!(
            classify_restart(anchor, after(start, 60.0), 12.2, Some(30.0)),
            Jump::None
        );
        assert_eq!(
            classify_restart(anchor, after(start, 60.0), 0.0, Some(30.0)),
            Jump::Skip
        );
    }

    #[test]
    fn without_a_duration_a_jump_back_to_the_start_after_progress_is_a_wrap() {
        let start = Instant::now();
        let anchor = anchor(start, 0.05, 1.0);
        assert_eq!(classify_restart(anchor, after(start, 29.9), 0.0, None), Jump::Wrap);
        assert_eq!(classify_restart(anchor, after(start, 0.5), 0.0, None), Jump::None);
    }

    #[test]
    fn any_other_jump_is_a_skip() {
        let start = Instant::now();
        let anchor = anchor(start, 5.0, 1.0);
        assert_eq!(
            classify_restart(anchor, after(start, 0.1), 20.0, Some(30.0)),
            Jump::Skip
        );
        let anchor = self::anchor(start, 20.0, 1.0);
        assert_eq!(classify_restart(anchor, after(start, 0.1), 8.0, Some(30.0)), Jump::Skip);
    }

    #[test]
    fn the_expected_position_advances_with_time_and_rate() {
        let start = Instant::now();
        let anchor = Anchor {
            at: start,
            position: 3.0,
            rate: 2.0,
        };
        assert!((anchor.expected(after(start, 1.5)) - 6.0).abs() < 1e-9);
        assert!((anchor.expected(start) - 3.0).abs() < 1e-9);
    }

    #[derive(Clone, Copy)]
    enum Step {
        Requested(u64),
        FileLoaded(u64),
        Restart,
    }

    /// Feeds the steps to a fresh [`SourceSwitch`] and returns what each restart was taken for.
    fn switch_after(steps: &[Step]) -> Vec<Restart> {
        let mut switch = SourceSwitch::default();
        let mut restarts = Vec::new();
        for step in steps {
            match *step {
                Step::Requested(generation) => switch.requested(generation),
                Step::FileLoaded(generation) => switch.file_loaded(generation),
                Step::Restart => restarts.push(switch.restart()),
            }
        }
        restarts
    }

    #[test]
    fn source_switch_ignores_a_restart_queued_before_the_new_file_loaded() {
        // mpv had queued a restart of the old file (a loop wrap) when the load was requested.
        let restarts = switch_after(&[Step::Requested(1), Step::Restart, Step::FileLoaded(1), Step::Restart]);

        assert_eq!(restarts, [Restart::Old, Restart::NewStart]);
    }

    #[test]
    fn source_switch_reports_one_start_per_load_of_the_same_path() {
        let restarts = switch_after(&[
            Step::Requested(1),
            Step::FileLoaded(1),
            Step::Restart,
            Step::Requested(2),
            Step::Restart,
            Step::FileLoaded(2),
            Step::Restart,
            // Nothing is pending any more: a later wrap of the new file is an ordinary restart.
            Step::Restart,
        ]);

        assert_eq!(
            restarts,
            [Restart::NewStart, Restart::Old, Restart::NewStart, Restart::Old]
        );
    }

    #[test]
    fn source_switch_starts_only_the_last_of_two_quick_loads() {
        let restarts = switch_after(&[
            Step::Requested(1),
            Step::Requested(2),
            Step::FileLoaded(1),
            Step::Restart,
            Step::FileLoaded(2),
            Step::Restart,
        ]);

        assert_eq!(restarts, [Restart::Old, Restart::NewStart]);
    }

    #[test]
    fn source_switch_with_no_load_pending_treats_every_restart_as_old() {
        let restarts = switch_after(&[Step::Restart, Step::FileLoaded(0), Step::Restart, Step::Restart]);

        assert_eq!(restarts, [Restart::Old, Restart::Old, Restart::Old]);
    }
}
