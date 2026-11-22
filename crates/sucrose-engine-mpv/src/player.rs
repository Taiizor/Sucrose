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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, PoisonError};
use std::thread::{self, JoinHandle};

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
    /// Playback restarted after a seek or a loop wrap, so the position jumped.
    Discontinuity,
    /// The player was paused (true) or resumed (false).
    PauseChanged(bool),
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

/// Classifies the move from the position `previous` to `now` (seconds). Playback advances by well under a second
/// between two readings that follow a restart, so a larger move is a jump.
fn classify_jump(previous: Option<f64>, now: f64) -> Jump {
    let Some(previous) = previous else {
        return Jump::None;
    };
    let delta = now - previous;
    if delta.abs() <= 1.0 {
        Jump::None
    } else if delta < 0.0 && now < 1.0 {
        Jump::Wrap
    } else {
        Jump::Skip
    }
}

/// State the event thread keeps for the thread that drives the player.
#[derive(Debug, Default)]
struct Shared {
    /// The last value of the observed `pause` property.
    paused: AtomicBool,
    /// How many times the position wrapped back to the start.
    loop_gen: AtomicU32,
    /// The last position the event thread saw, as the bits of an f64; NaN when unknown.
    last_position: AtomicU64,
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
    /// The caller vouches for `source`: only a file the theme gate accepted (see [Player::open]) or a fixed test
    /// source may be passed, never text taken from a theme (T-02-14).
    #[doc(hidden)]
    pub fn open_source(
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
            shared: Arc::new(Shared {
                last_position: AtomicU64::new(f64::NAN.to_bits()),
                ..Shared::default()
            }),
            receiver: Mutex::new(None),
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

    /// Pauses (true) or resumes (false) playback.
    pub fn set_paused(&self, paused: bool) -> Result<(), PlayerError> {
        self.set_property("pause", if paused { "yes" } else { "no" })
    }

    /// Sets the playback volume; values above 100 are treated as 100 (volume-max is 100).
    pub fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.set_property("volume", &volume.min(100).to_string())
    }

    /// Sets the playback speed (1.0 is normal); the follower's drift correction uses 0.98 and 1.02 (D-02).
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
                note_restart(mpv, handle, shared, events);
            }
            sys::mpv_event_id_MPV_EVENT_PROPERTY_CHANGE => note_property(event, shared, events),
            sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART => {
                started = true;
                if let Some(position) =
                    property_string(mpv, handle, "time-pos").and_then(|text| text.trim().parse().ok())
                {
                    shared.last_position.store(f64::to_bits(position), Ordering::SeqCst);
                }
                let hwdec = property_string(mpv, handle, "hwdec-current")
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "no".to_owned());
                tracing::info!(%hwdec, "playback started");
            }
            sys::mpv_event_id_MPV_EVENT_END_FILE => log_end_file(mpv, event),
            sys::mpv_event_id_MPV_EVENT_LOG_MESSAGE => log_message(event),
            _ => {}
        }
    }
}

/// A restart after the first one: compares the position with the last one seen, counts a wrap and tells the driver.
fn note_restart(mpv: &Mpv, handle: Handle, shared: &Shared, events: &Sender<PlayerEvent>) {
    let Some(position) = property_string(mpv, handle, "time-pos").and_then(|text| text.trim().parse::<f64>().ok())
    else {
        return;
    };
    let previous = f64::from_bits(shared.last_position.swap(position.to_bits(), Ordering::SeqCst));
    let previous = (!previous.is_nan()).then_some(previous);
    if classify_jump(previous, position) == Jump::Wrap {
        shared.loop_gen.fetch_add(1, Ordering::SeqCst);
    }
    // A closed channel means nobody listens (a follower, or the player is being dropped): nothing to do.
    let _ = events.send(PlayerEvent::Discontinuity);
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

fn log_end_file(mpv: &Mpv, event: &sys::mpv_event) {
    // SAFETY: for END_FILE, data is NULL or points to an mpv_event_end_file valid until the next mpv_wait_event.
    let Some(end) = (unsafe { event.data.cast::<sys::mpv_event_end_file>().as_ref() }) else {
        return;
    };
    if end.reason == sys::mpv_end_file_reason_MPV_END_FILE_REASON_ERROR {
        tracing::warn!(error = %error_text(mpv, end.error), "playback ended with an error");
    } else {
        tracing::debug!(reason = end.reason, "playback ended");
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
    use super::*;

    #[test]
    fn ordinary_playback_is_no_jump() {
        assert_eq!(classify_jump(None, 5.0), Jump::None);
        assert_eq!(classify_jump(Some(5.0), 5.4), Jump::None);
        assert_eq!(classify_jump(Some(5.0), 4.2), Jump::None);
    }

    #[test]
    fn a_jump_back_to_the_start_is_a_wrap() {
        assert_eq!(classify_jump(Some(29.8), 0.05), Jump::Wrap);
    }

    #[test]
    fn any_other_jump_is_a_skip() {
        assert_eq!(classify_jump(Some(5.0), 20.0), Jump::Skip);
        assert_eq!(classify_jump(Some(20.0), 8.0), Jump::Skip);
    }
}
