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
use std::sync::atomic::{AtomicBool, Ordering};
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

/// A running embedded mpv instance. Dropping it stops the event thread and destroys the instance.
pub struct Player {
    // Declared first so it is dropped last: the library must outlive every call into it.
    mpv: Arc<Mpv>,
    handle: Handle,
    stop: Arc<AtomicBool>,
    events: Option<JoinHandle<()>>,
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
        let thread = thread::Builder::new()
            .name("mpv-events".to_owned())
            .spawn(move || event_loop(&mpv, handle, &stop))
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
fn event_loop(mpv: &Mpv, handle: Handle, stop: &AtomicBool) {
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
            sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART if !started => {
                started = true;
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
