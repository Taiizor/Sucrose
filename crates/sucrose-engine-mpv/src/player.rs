//! One embedded mpv instance: created, hardened and started on a window handle, then driven until it is dropped.
//!
//! A [Player] owns an mpv handle. It must be opened on the thread that owns the parent window and pumps its
//! messages, because mpv hooks that thread to resize its child window (RESEARCH Pattern 2). The handle is only
//! ever touched through the thread-safe client API, and a dedicated thread is the only caller of
//! mpv_wait_event.

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
}

/// The embedding and hardening options of this slice, in the order they are set.
///
/// wid is not part of the list: [Player::open] sets it from its argument. A theme file never contributes an option
/// name or value, so it cannot turn scripts, configs or URL fetching back on (T-02-14).
#[must_use]
pub fn embedded_options() -> Vec<(&'static str, String)> {
    let mut options: Vec<(&'static str, &str)> = vec![
        ("vo", "gpu"),
        ("hwdec", "auto-safe"),
        ("loop-file", "inf"),
        // The legacy default stretch is UniformToFill.
        ("panscan", "1.0"),
        // Hardening for an embedded, headless instance: no user config, scripts, URL resolver, UI or input.
        ("config", "no"),
        ("load-scripts", "no"),
        ("ytdl", "no"),
        ("osc", "no"),
        ("input-default-bindings", "no"),
        ("input-vo-keyboard", "no"),
        ("input-cursor", "no"),
        ("cursor-autohide", "no"),
        ("terminal", "no"),
        ("idle", "yes"),
        ("stop-screensaver", "no"),
        ("media-controls", "no"),
    ];
    if cfg!(windows) {
        options.insert(1, ("gpu-context", "d3d11"));
    }
    options
        .into_iter()
        .map(|(name, value)| (name, value.to_owned()))
        .collect()
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
    pub fn open(mpv: Arc<Mpv>, wid: u64, file: &Path, options: &[(&'static str, String)]) -> Result<Self, PlayerError> {
        let path = file
            .to_str()
            .filter(|text| file.is_absolute() && !text.contains('\0'))
            .ok_or_else(|| PlayerError::InvalidPath {
                path: file.display().to_string(),
            })?
            .to_owned();

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

        // mpv takes the low 32 bits of the HWND as a decimal number (all Windows handles fit in 32 bits).
        #[allow(clippy::cast_possible_truncation)]
        let wid_value = (wid as u32).to_string();
        player.set_option("wid", &wid_value)?;
        for (name, value) in options {
            player.set_option(name, value)?;
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
        player.load(&path)?;
        Ok(player)
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
            sys::mpv_event_id_MPV_EVENT_FILE_LOADED | sys::mpv_event_id_MPV_EVENT_PLAYBACK_RESTART if !started => {
                started = true;
                tracing::info!("playback started");
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::embedded_options;

    fn value_of(name: &str) -> Option<String> {
        embedded_options()
            .into_iter()
            .find_map(|(option, value)| (option == name).then_some(value))
    }

    #[test]
    fn untrusted_theme_content_cannot_run_scripts_or_fetch_urls() {
        for (name, expected) in [
            ("load-scripts", "no"),
            ("ytdl", "no"),
            ("config", "no"),
            ("input-default-bindings", "no"),
            ("input-vo-keyboard", "no"),
            ("osc", "no"),
        ] {
            assert_eq!(value_of(name).as_deref(), Some(expected), "{name} must be hardened");
        }
    }

    #[test]
    fn no_option_is_named_twice_and_wid_is_left_to_open() {
        let options = embedded_options();
        let names: HashSet<_> = options.iter().map(|(name, _)| *name).collect();
        assert_eq!(names.len(), options.len(), "an option is listed twice");
        assert!(!names.contains("wid"), "Player::open sets wid itself");
    }
}
