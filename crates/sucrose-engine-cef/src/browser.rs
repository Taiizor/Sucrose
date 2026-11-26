//! Windowed CEF browser hosted as a child of a renderer's desktop surface (D-013, D-32, D-036).
//!
//! [Findings 1 of the Phase 4 research] CEF runs on the thread that created the surface, with the external message
//! pump: CEF asks to be pumped through `OnScheduleMessagePumpWork` (see [`CefStart::wake`]), the renderer's loop calls
//! [`CefHost::do_work`] on that wake and on its pump timer, and there is no second message loop and no
//! multi-threaded message loop. Every method of [`CefHost`] must be called on that one thread.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

pub use sucrose_engine_web::keys::{KeyAction, key_action};
use sucrose_engine_web::{
    args::{SucroseInputs, Switch},
    js::PageCall,
    profile::ProfileLease,
};

/// The longest the pump timer waits between two pump calls (about 30 Hz): CEF's own sample's safety net for work it
/// forgot to announce.
pub const MAX_PUMP_DELAY_MS: u32 = 33;

/// Why the CEF engine did not start. Every variant is a value for the renderer to report, never a panic (T-04-07).
#[derive(Debug, thiserror::Error)]
pub enum CefError {
    /// This build or this OS has no CEF engine (Windows x86, other systems).
    #[error("CEF is not available on this system")]
    Unsupported,
    /// `libcef.dll` (or a file it needs) is missing or cannot be loaded.
    #[error("the CEF files are missing or cannot be loaded: {0}")]
    Load(String),
    /// CEF refused to initialise.
    #[error("CEF could not be initialised ({0})")]
    Initialize(String),
    /// CEF could not create the browser window.
    #[error("CEF could not create the browser window")]
    CreateBrowser,
    /// A Win32 call on the browser window failed.
    #[error("the browser window could not be moved: {0}")]
    Window(String),
}

/// What a page-load report callback looks like: the address and the HTTP status of the main frame.
pub type LoadedCallback = Arc<dyn Fn(&str, i32) + Send + Sync>;

/// What the page-calls callback looks like: given the address and HTTP status of a main frame that finished loading, the
/// scripts to run in it, in order (WAPI-01, WAPI-02).
pub type PageCallsCallback = Arc<dyn Fn(&str, i32) -> Vec<PageCall> + Send + Sync>;

/// What the renderer hands to [`CefHost::start`].
pub struct CefStart {
    /// The CEF log file of this renderer (`CefRenderer-<slot>.log`).
    pub log_file: PathBuf,
    /// The executable CEF starts for its subprocesses: this one (D-036).
    pub subprocess_path: PathBuf,
    /// Wakes the surface thread so it calls [`CefHost::do_work`]; called from any thread.
    pub wake: Arc<dyn Fn() + Send + Sync>,
    /// Called on the surface thread with the address and HTTP status each time the main frame finishes loading.
    pub on_loaded: LoadedCallback,
    /// Asked on the surface thread each time the main frame finishes loading, before [`CefStart::on_loaded`]: the
    /// scripts it returns run in that frame in order, each logged as a `page call` at debug. The page gets no host
    /// object and no message channel; a call is a one-way script string (legacy parity).
    pub page_calls: PageCallsCallback,
    /// The folders the virtual origin serves, in lookup order: the theme folder, then the generated-content folder.
    pub roots: Vec<PathBuf>,
    /// The browser profile this instance runs on (D-33, D-35); `None` runs on in-memory caches, which keep nothing.
    /// The host keeps the lease until CEF has shut down, so the folder stays this instance's all that time.
    pub profile: Option<Arc<ProfileLease>>,
    /// Whether the user allowed invalid TLS certificates (D-12, `Engine.AllowInvalidCertificates`); off by default.
    pub allow_invalid_certificates: bool,
    /// The user's Chromium switches (D-17); Sucrose's own are computed from [`CefStart::sucrose`] and merged in.
    pub arguments: Vec<Switch>,
    /// What Sucrose's own switches are computed from: hardware acceleration, the debugging port of this renderer and the
    /// Windows build.
    pub sucrose: SucroseInputs,
    /// Whether developer mode is on (D-13): the context menu has Inspect, and F12 and Ctrl+Shift+I open DevTools. Off,
    /// the menu is empty and the DevTools shortcuts are swallowed. DevTools never open by themselves.
    pub developer_mode: bool,
}

impl std::fmt::Debug for CefStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CefStart")
            .field("log_file", &self.log_file)
            .field("subprocess_path", &self.subprocess_path)
            .finish_non_exhaustive()
    }
}

/// The size of the browser window for a surface rectangle (left, top, right, bottom): at least 1x1.
#[must_use]
pub fn browser_size(rect: [i32; 4]) -> (i32, i32) {
    let [left, top, right, bottom] = rect;
    (right.saturating_sub(left).max(1), bottom.saturating_sub(top).max(1))
}

/// The `root_cache_path` and `cache_path` CEF gets for a profile folder: both are that folder, made absolute (CEF
/// requires it), so the persistent profile of the first instance is exactly `<data_dir>/Web/Cef` (D-35). No profile
/// gives two empty paths, which CEF reads as in-memory caches (nothing is kept).
///
/// # Errors
/// `CefError::Initialize` for a folder that cannot be made absolute or is not valid UTF-8.
pub fn profile_paths(profile: Option<&Path>) -> Result<(String, String), CefError> {
    let Some(folder) = profile else {
        return Ok((String::new(), String::new()));
    };
    let absolute = std::path::absolute(folder)
        .map_err(|error| CefError::Initialize(format!("the profile folder path cannot be made absolute: {error}")))?;
    let text = absolute
        .to_str()
        .ok_or_else(|| CefError::Initialize("the profile folder path is not valid UTF-8".to_owned()))?
        .to_owned();
    Ok((text.clone(), text))
}

/// The Chromium switch that makes the browser accept every invalid TLS certificate. CEF no longer has a settings field
/// for it, so the browser process appends this switch itself, and only when the user allowed invalid certificates.
pub const IGNORE_CERTIFICATE_ERRORS_SWITCH: &str = "ignore-certificate-errors";

/// The profile and certificate values of the CEF settings, decided before CEF is touched so they can be tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsValues {
    /// `root_cache_path`, see [`profile_paths`].
    pub root_cache_path: String,
    /// `cache_path`, see [`profile_paths`].
    pub cache_path: String,
    /// Whether invalid TLS certificates are accepted (D-12): only when the user allowed it, never by default.
    pub ignore_certificate_errors: bool,
}

/// The settings values for a profile folder and the user's Allow invalid certificates choice (D-12).
///
/// # Errors
/// See [`profile_paths`].
pub fn settings_values(profile: Option<&Path>, allow_invalid_certificates: bool) -> Result<SettingsValues, CefError> {
    let (root_cache_path, cache_path) = profile_paths(profile)?;
    Ok(SettingsValues {
        root_cache_path,
        cache_path,
        ignore_certificate_errors: allow_invalid_certificates,
    })
}

/// How long the pump timer waits after a pump call, given the delay CEF last asked for: asap becomes 1 ms, nothing
/// asked becomes [`MAX_PUMP_DELAY_MS`], and no delay is longer than that.
#[must_use]
pub fn next_pump_delay_ms(requested: Option<i64>) -> u32 {
    match requested {
        Some(delay) if delay <= 0 => 1,
        Some(delay) => u32::try_from(delay).unwrap_or(MAX_PUMP_DELAY_MS).min(MAX_PUMP_DELAY_MS),
        None => MAX_PUMP_DELAY_MS,
    }
}

/// How long [`CefHost::close`] pumps for the browser to go away before it gives up and shuts CEF down anyway.
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long [`CefHost::close`] waits for CEF to be ready to close the browser (`DoClose`) before it destroys the
/// browser window anyway: a page whose unload handlers hang must not keep the window alive until CEF shuts down.
pub const FORCED_TEAR_DOWN_AFTER: Duration = Duration::from_secs(3);

/// What one turn of the close loop of [`CefHost::close`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseStep {
    /// `OnBeforeClose` came: the browser is gone and CEF can shut down.
    Done,
    /// Destroy the browser window, which completes the close: CEF is ready (`DoClose` came) or never got ready in
    /// time.
    TearDown,
    /// Pump CEF and look again.
    Pump,
    /// The time is up: CEF shuts down with the browser still open.
    GiveUp,
}

/// The next step of the close loop, given whether the browser is closed, whether CEF is ready to close it, whether its
/// window was already destroyed and how long the close has taken.
#[must_use]
pub fn close_step(closed: bool, ready: bool, torn_down: bool, elapsed: Duration) -> CloseStep {
    if closed {
        CloseStep::Done
    } else if elapsed >= CLOSE_TIMEOUT {
        CloseStep::GiveUp
    } else if !torn_down && (ready || elapsed >= FORCED_TEAR_DOWN_AFTER) {
        CloseStep::TearDown
    } else {
        CloseStep::Pump
    }
}

/// What the life span handler answers to CEF's `DoClose` for a browser. For a windowed browser CEF's default (0) posts
/// `WM_CLOSE` to the browser's top-level ancestor; the wallpaper browser is a child of the desktop surface, so that
/// ancestor is the shell's Progman (or its `WorkerW`), and `WM_CLOSE` to Progman opens the Shut Down Windows dialog.
/// So the wallpaper browser answers 1 and Sucrose completes the close itself by destroying the browser window. A popup
/// a page opened (`window.open`) is a top-level window of its own and keeps CEF's default, which closes that popup.
#[must_use]
pub fn do_close_answer(popup: bool) -> i32 {
    i32::from(!popup)
}

/// The command id of the context menu's Inspect item: the first id CEF leaves to applications (`MENU_ID_USER_FIRST`).
pub const INSPECT_COMMAND_ID: i32 = 26500;

/// The pump requests CEF made, kept until the surface thread serves them.
struct Pump {
    requested: Mutex<Option<i64>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Pump {
    /// CEF wants a pump call in `delay_ms` (any thread): remember the soonest and wake the surface thread.
    fn schedule(&self, delay_ms: i64) {
        {
            let mut requested = self.requested.lock().unwrap_or_else(PoisonError::into_inner);
            *requested = Some(requested.map_or(delay_ms, |old| old.min(delay_ms)));
        }
        (self.wake)();
    }

    fn take(&self) -> Option<i64> {
        self.requested.lock().unwrap_or_else(PoisonError::into_inner).take()
    }

    fn peek(&self) -> Option<i64> {
        *self.requested.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod imp {
    use std::{
        cell::Cell,
        path::Path,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        thread::{self, ThreadId},
        time::{Duration, Instant},
    };

    use cef::{args::Args, *};
    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::LibraryLoader::LoadLibraryW,
        UI::WindowsAndMessaging::{
            DestroyWindow, FindWindowExW, PostMessageW, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos,
        },
    };
    use windows::core::{PCWSTR, w};

    use sucrose_engine_web::args::{Switch, plan_cef_command_line};

    use super::{
        CefError, CefStart, CloseStep, IGNORE_CERTIFICATE_ERRORS_SWITCH, INSPECT_COMMAND_ID, KeyAction, LoadedCallback,
        PageCallsCallback, ProfileLease, Pump, browser_size, close_step, do_close_answer, key_action,
        next_pump_delay_ms, settings_values,
    };

    /// `EVENTFLAG_SHIFT_DOWN` and `EVENTFLAG_CONTROL_DOWN` of a CEF key event's modifiers.
    const SHIFT_DOWN: u32 = 1 << 1;
    const CONTROL_DOWN: u32 = 1 << 2;

    /// What the handlers share with the host.
    struct Shared {
        /// `OnBeforeClose` came for the wallpaper browser.
        closed: AtomicBool,
        /// `close` has started: a `DoClose` now answers Sucrose's own `CloseBrowser`.
        closing: AtomicBool,
        /// `DoClose` came while closing: CEF waits for Sucrose to destroy the browser window.
        ready_to_close: AtomicBool,
        on_loaded: LoadedCallback,
        page_calls: PageCallsCallback,
    }

    wrap_browser_process_handler! {
        struct PumpHandler {
            pump: Arc<Pump>,
        }

        impl BrowserProcessHandler {
            fn on_schedule_message_pump_work(&self, delay_ms: i64) {
                self.pump.schedule(delay_ms);
            }
        }
    }

    wrap_app! {
        struct HostApp {
            handler: BrowserProcessHandler,
            ignore_certificate_errors: bool,
            switches: Vec<Switch>,
        }

        impl App {
            fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
                Some(self.handler.clone())
            }

            // The browser process gets the user's switches merged with Sucrose's own (D-17); the subprocesses inherit the
            // browser's command line. CEF's settings have no field for accepting invalid certificates, so the browser
            // process also adds Chromium's switch itself, and only when the user allowed it (D-12): there is no
            // OnCertificateError override, so everything else is rejected. Switches go in one by one through the
            // command line API, never through a shell (T-04-20).
            fn on_before_command_line_processing(
                &self,
                process_type: Option<&CefString>,
                command_line: Option<&mut CommandLine>,
            ) {
                let browser_process = process_type.is_none_or(|kind| kind.to_string().is_empty());
                let Some(command_line) = command_line else { return };
                if !browser_process {
                    return;
                }
                for switch in &self.switches {
                    let name = CefString::from(switch.name.as_str());
                    match &switch.value {
                        Some(value) => {
                            command_line.append_switch_with_value(Some(&name), Some(&CefString::from(value.as_str())));
                        }
                        None => command_line.append_switch(Some(&name)),
                    }
                }
                if self.ignore_certificate_errors {
                    command_line.append_switch(Some(&CefString::from(IGNORE_CERTIFICATE_ERRORS_SWITCH)));
                }
            }
        }
    }

    wrap_life_span_handler! {
        struct SpanHandler {
            shared: Arc<Shared>,
        }

        impl LifeSpanHandler {
            // CEF's default answer would post WM_CLOSE to the browser's top-level ancestor, which for the wallpaper
            // browser is the shell's Progman: the Shut Down Windows dialog (see `do_close_answer`). The wallpaper
            // browser answers 1, so nothing reaches the shell; `CefHost::close` then destroys the browser window, and
            // a window.close() from the page is ignored: a wallpaper is not a browser tab, it stays open.
            fn do_close(&self, browser: Option<&mut Browser>) -> ::std::os::raw::c_int {
                let popup = browser.is_some_and(|browser| browser.is_popup() == 1);
                if !popup {
                    if self.shared.closing.load(Ordering::SeqCst) {
                        self.shared.ready_to_close.store(true, Ordering::SeqCst);
                    } else {
                        tracing::warn!("the page asked to close its window, the wallpaper stays open");
                    }
                }
                do_close_answer(popup)
            }

            // A popup the page opened ends on its own; only the end of the wallpaper browser counts.
            fn on_before_close(&self, browser: Option<&mut Browser>) {
                if browser.is_some_and(|browser| browser.is_popup() == 1) {
                    return;
                }
                self.shared.closed.store(true, Ordering::SeqCst);
            }
        }
    }

    wrap_load_handler! {
        struct PageLoadHandler {
            shared: Arc<Shared>,
        }

        impl LoadHandler {
            fn on_load_end(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, http_status_code: ::std::os::raw::c_int) {
                let Some(frame) = frame else { return };
                if frame.is_main() != 1 {
                    return;
                }
                let url = CefString::from(&frame.url()).to_string();
                tracing::info!(url = ?url, status = http_status_code, "page loaded");
                // The page's own scripts have run by now: the calls go in before the first frame is reported.
                for call in (self.shared.page_calls)(&url, http_status_code) {
                    tracing::debug!(kind = call.kind, key = ?call.key, "page call");
                    frame.execute_java_script(
                        Some(&CefString::from(call.script.as_str())),
                        Some(&CefString::from(url.as_str())),
                        0,
                    );
                }
                (self.shared.on_loaded)(&url, http_status_code);
            }

            fn on_load_error(
                &self,
                _browser: Option<&mut Browser>,
                frame: Option<&mut Frame>,
                error_code: Errorcode,
                error_text: Option<&CefString>,
                failed_url: Option<&CefString>,
            ) {
                if frame.is_some_and(|frame| frame.is_main() != 1) {
                    return;
                }
                tracing::warn!(
                    code = ?error_code,
                    text = %error_text.map(ToString::to_string).unwrap_or_default(),
                    url = ?failed_url.map(ToString::to_string).unwrap_or_default(),
                    "the page failed to load"
                );
            }
        }
    }

    wrap_download_handler! {
        struct NoDownloads;

        impl DownloadHandler {
            // The legacy host cancels downloads: a wallpaper is not a download manager.
            fn can_download(
                &self,
                _browser: Option<&mut Browser>,
                _url: Option<&CefString>,
                _request_method: Option<&CefString>,
            ) -> ::std::os::raw::c_int {
                0
            }
        }
    }

    /// Destroys the browser's own window, a child of the desktop surface. With `DoClose` answered 1 this is what
    /// completes the close: CEF sees the window go, destroys the browser and calls `OnBeforeClose`.
    fn tear_down(browser: &Browser) {
        let Some(host) = browser.host() else { return };
        let window = HWND(host.window_handle().0.cast());
        // SAFETY: the window is this browser's own window, which CEF created on this thread (the thread that runs CEF
        // with the external message pump), so DestroyWindow may destroy it here; it validates the handle and fails for
        // a window that is gone, which only means the close already happened.
        if let Err(error) = unsafe { DestroyWindow(window) } {
            tracing::debug!(%error, "the browser window is already gone");
        }
    }

    /// Shows DevTools for `browser` in a window of its own, on top of nothing (never as a child of the wallpaper), at
    /// the page point `inspect_at` when given. Only ever called for a user action (D-13).
    fn show_devtools(browser: &Browser, inspect_at: Option<&Point>) {
        let Some(host) = browser.host() else { return };
        let window_info = WindowInfo {
            runtime_style: RuntimeStyle::ALLOY,
            ..WindowInfo::default()
        }
        .set_as_popup(cef::sys::HWND(std::ptr::null_mut()), "Sucrose DevTools");
        host.show_dev_tools(Some(&window_info), None, None, inspect_at);
    }

    wrap_context_menu_handler! {
        struct PageMenu {
            developer_mode: bool,
        }

        impl ContextMenuHandler {
            // The legacy host has no context menu, so it is cleared; in developer mode it keeps one item, Inspect
            // (D-13), which is the user's own action.
            fn on_before_context_menu(
                &self,
                _browser: Option<&mut Browser>,
                _frame: Option<&mut Frame>,
                _params: Option<&mut ContextMenuParams>,
                model: Option<&mut MenuModel>,
            ) {
                if let Some(model) = model {
                    model.clear();
                    if self.developer_mode {
                        model.add_item(INSPECT_COMMAND_ID, Some(&CefString::from("Inspect")));
                    }
                }
            }

            fn on_context_menu_command(
                &self,
                browser: Option<&mut Browser>,
                _frame: Option<&mut Frame>,
                params: Option<&mut ContextMenuParams>,
                command_id: ::std::os::raw::c_int,
                _event_flags: EventFlags,
            ) -> ::std::os::raw::c_int {
                if command_id != INSPECT_COMMAND_ID || !self.developer_mode {
                    return 0;
                }
                if let Some(browser) = browser {
                    let at = params.map(|params| Point {
                        x: params.xcoord(),
                        y: params.ycoord(),
                    });
                    show_devtools(browser, at.as_ref());
                }
                1
            }
        }
    }

    wrap_keyboard_handler! {
        struct PageKeys {
            developer_mode: bool,
        }

        impl KeyboardHandler {
            // F12 and Ctrl+Shift+I open DevTools in developer mode; with it off the legacy blocked keys and the DevTools
            // shortcuts are swallowed (D-13). See `key_action`.
            fn on_pre_key_event(
                &self,
                browser: Option<&mut Browser>,
                event: Option<&KeyEvent>,
                _os_event: Option<&mut cef::sys::MSG>,
                _is_keyboard_shortcut: Option<&mut ::std::os::raw::c_int>,
            ) -> ::std::os::raw::c_int {
                let Some(event) = event else { return 0 };
                let ctrl = event.modifiers & CONTROL_DOWN != 0;
                let shift = event.modifiers & SHIFT_DOWN != 0;
                match key_action(self.developer_mode, event.windows_key_code, ctrl, shift) {
                    KeyAction::Pass => 0,
                    KeyAction::Swallow => 1,
                    KeyAction::DevTools => {
                        if event.type_ == KeyEventType::RAWKEYDOWN
                            && let Some(browser) = browser
                        {
                            show_devtools(browser, None);
                        }
                        1
                    }
                }
            }
        }
    }

    wrap_client! {
        struct HostClient {
            life_span: LifeSpanHandler,
            load: LoadHandler,
            download: DownloadHandler,
            menu: ContextMenuHandler,
            keys: KeyboardHandler,
        }

        impl Client {
            fn life_span_handler(&self) -> Option<LifeSpanHandler> {
                Some(self.life_span.clone())
            }

            fn load_handler(&self) -> Option<LoadHandler> {
                Some(self.load.clone())
            }

            fn download_handler(&self) -> Option<DownloadHandler> {
                Some(self.download.clone())
            }

            fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
                Some(self.menu.clone())
            }

            fn keyboard_handler(&self) -> Option<KeyboardHandler> {
                Some(self.keys.clone())
            }
        }
    }

    /// A CEF browser embedded in a window of this thread. Create it, pump it and close it on one thread.
    pub struct CefHost {
        browser: Browser,
        pump: Arc<Pump>,
        shared: Arc<Shared>,
        thread: ThreadId,
        /// Held until CEF has shut down: the profile folder is this instance's for as long as CEF may touch it.
        profile: Option<Arc<ProfileLease>>,
        /// The Chromium widget window that gets the forwarded input messages (0 until found).
        target: Cell<isize>,
        /// Whether the missing widget was already reported.
        target_missing_logged: Cell<bool>,
    }

    impl std::fmt::Debug for CefHost {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("CefHost").finish_non_exhaustive()
        }
    }

    /// Loads `libcef.dll` from beside this executable, so that a missing or broken file is an error value:
    /// the delay-load helper would raise an exception in the first CEF call instead. The handle stays loaded, and the
    /// delay-load finds the module by name.
    fn probe_library() -> Result<(), CefError> {
        let dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .ok_or_else(|| CefError::Load("the folder of this program is unknown".to_owned()))?;
        let library = dir.join("libcef.dll");
        if !library.is_file() {
            return Err(CefError::Load("libcef.dll is not next to this program".to_owned()));
        }
        // SAFETY: loading a DLL by full path from the program's own folder; the pinned CEF dist (hash-verified by
        // xtask) is what is there, and the module is never unloaded.
        unsafe { LoadLibraryW(&windows::core::HSTRING::from(library.as_os_str())) }
            .map(|_| ())
            .map_err(|error| CefError::Load(error.to_string()))
    }

    fn utf8(path: &Path, what: &str) -> Result<String, CefError> {
        path.to_str()
            .map(str::to_owned)
            .ok_or_else(|| CefError::Initialize(format!("the {what} path is not valid UTF-8")))
    }

    impl CefHost {
        /// Initialises CEF on this thread and opens a windowed browser as a child of the window `parent_hwnd`,
        /// covering `rect` (left, top, right, bottom; the child sits at 0,0 of its parent), showing `url`.
        pub fn start(parent_hwnd: u64, rect: [i32; 4], url: &str, start: &CefStart) -> Result<Self, CefError> {
            probe_library()?;
            // CEF requires the API version before any other call.
            let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
            let pump = Arc::new(Pump {
                requested: Mutex::new(None),
                wake: Arc::clone(&start.wake),
            });
            let shared = Arc::new(Shared {
                closed: AtomicBool::new(false),
                closing: AtomicBool::new(false),
                ready_to_close: AtomicBool::new(false),
                on_loaded: Arc::clone(&start.on_loaded),
                page_calls: Arc::clone(&start.page_calls),
            });
            // The persistent profile of the first instance or an ephemeral one (D-33); both paths name the same folder, so
            // CEF's process singleton lock and the profile never meet another running instance's.
            let values = settings_values(
                start.profile.as_deref().map(|lease| lease.path.as_path()),
                start.allow_invalid_certificates,
            )?;
            // The user's switches merged with Sucrose's own: setting keys CEF does not take and names that are not switch
            // names are left out and logged; everything else passes, unknown switches included (D-17).
            let plan = plan_cef_command_line(&start.arguments, &start.sucrose);
            for key in &plan.skipped_settings {
                tracing::info!(key = %key, "cef setting key skipped");
            }
            for (name, error) in &plan.invalid {
                tracing::warn!(name = %name, %error, "a switch name is not valid, the switch is left out");
            }
            tracing::info!(
                switches = %plan.applied.iter().map(Switch::render).collect::<Vec<_>>().join(" "),
                allow_invalid_certificates = values.ignore_certificate_errors,
                "chromium switches"
            );
            let mut app = HostApp::new(
                PumpHandler::new(Arc::clone(&pump)),
                values.ignore_certificate_errors,
                plan.applied,
            );
            let settings = Settings {
                no_sandbox: 1,
                external_message_pump: 1,
                multi_threaded_message_loop: 0,
                windowless_rendering_enabled: 0,
                browser_subprocess_path: CefString::from(utf8(&start.subprocess_path, "subprocess")?.as_str()),
                log_file: CefString::from(utf8(&start.log_file, "log file")?.as_str()),
                locale: CefString::from("en-US"),
                root_cache_path: CefString::from(values.root_cache_path.as_str()),
                cache_path: CefString::from(values.cache_path.as_str()),
                // The legacy host's PersistSessionCookies; it only takes effect with a cache path.
                persist_session_cookies: 1,
                // 0 leaves the debugging port off (D-13); a port is only set from a DeveloperPort above 0.
                remote_debugging_port: start.sucrose.debugging_port.map_or(0, i32::from),
                ..Settings::default()
            };
            let args = Args::new();
            // SAFETY-free API, but it is the call that starts Chromium: it must run on the thread that pumps.
            let initialised = cef::initialize(
                Some(args.as_main_args()),
                Some(&settings),
                Some(&mut app),
                std::ptr::null_mut(),
            );
            if initialised != 1 {
                return Err(CefError::Initialize(format!("initialize returned {initialised}")));
            }
            // The virtual origin must exist before the first navigation, or the page request goes to the network.
            if !crate::scheme::register(start.roots.clone()) {
                cef::shutdown();
                return Err(CefError::Initialize(
                    "the scheme handler of the virtual origin could not be registered".to_owned(),
                ));
            }
            let (width, height) = browser_size(rect);
            // SAFETY-free: the handle is only an identifier CEF passes to Win32.
            let parent = cef::sys::HWND(parent_hwnd as usize as *mut cef::sys::HWND__);
            let window_info = WindowInfo {
                runtime_style: RuntimeStyle::ALLOY,
                ..WindowInfo::default()
            }
            .set_as_child(
                parent,
                &Rect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
            );
            let browser_settings = BrowserSettings {
                // Opaque black until the page paints.
                background_color: 0xFF00_0000,
                ..BrowserSettings::default()
            };
            let mut client = HostClient::new(
                SpanHandler::new(Arc::clone(&shared)),
                PageLoadHandler::new(Arc::clone(&shared)),
                NoDownloads::new(),
                PageMenu::new(start.developer_mode),
                PageKeys::new(start.developer_mode),
            );
            let browser = browser_host_create_browser_sync(
                Some(&window_info),
                Some(&mut client),
                Some(&CefString::from(url)),
                Some(&browser_settings),
                None,
                None,
            );
            let Some(browser) = browser else {
                cef::shutdown();
                return Err(CefError::CreateBrowser);
            };
            Ok(Self {
                browser,
                pump,
                shared,
                thread: thread::current().id(),
                profile: start.profile.clone(),
                target: Cell::new(0),
                target_missing_logged: Cell::new(false),
            })
        }

        /// Runs CEF's pending UI-thread work and returns how many milliseconds the pump timer should wait before the
        /// next call. Call it on the surface thread when it is woken and when the pump timer fires.
        pub fn do_work(&self) -> u32 {
            debug_assert_eq!(
                self.thread,
                thread::current().id(),
                "CEF is pumped on the thread that started it"
            );
            let taken = self.pump.take();
            cef::do_message_loop_work();
            // Work CEF asked for while it ran counts; a request that was due now was just served.
            next_pump_delay_ms(self.pump.peek().or(taken.filter(|delay| *delay > 0)))
        }

        /// Moves the browser window onto the surface rectangle (left, top, right, bottom; physical pixels), to 0,0
        /// of its parent and the size of the rectangle.
        pub fn set_rect(&self, rect: [i32; 4]) -> Result<(), CefError> {
            let host = self
                .browser
                .host()
                .ok_or_else(|| CefError::Window("the browser has no host".to_owned()))?;
            let handle = host.window_handle();
            let (width, height) = browser_size(rect);
            // SAFETY: the window belongs to this thread's browser; SetWindowPos validates the handle and fails with
            // an error for a window that is gone.
            unsafe {
                SetWindowPos(
                    HWND(handle.0.cast()),
                    None,
                    0,
                    0,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
            }
            .map_err(|error| CefError::Window(error.to_string()))
        }

        /// The Chromium widget of this browser: the child `Chrome_WidgetWin_0` of the browser window, else
        /// `Chrome_WidgetWin_1`, as the legacy app finds it. Only this browser's own window is searched (T-04-46).
        fn input_target(&self) -> Option<isize> {
            let cached = self.target.get();
            if cached != 0 {
                return Some(cached);
            }
            let host = self.browser.host()?;
            let browser = HWND(host.window_handle().0.cast());
            for (name, class) in [
                ("Chrome_WidgetWin_0", w!("Chrome_WidgetWin_0")),
                ("Chrome_WidgetWin_1", w!("Chrome_WidgetWin_1")),
            ] {
                // SAFETY: `browser` is the window of this thread's browser and the class is a valid NUL-terminated
                // literal; FindWindowExW only reads the window tree and fails for a window that is gone.
                if let Ok(child) = unsafe { FindWindowExW(Some(browser), None, class, PCWSTR::null()) } {
                    tracing::debug!(class = name, "input target found");
                    self.target.set(child.0 as isize);
                    return Some(child.0 as isize);
                }
            }
            if !self.target_missing_logged.replace(true) {
                tracing::warn!("the browser has no Chromium widget window yet, input is not forwarded until it has");
            }
            None
        }

        /// Posts one legacy input message to the Chromium widget of this browser; `false` when there is no such window
        /// (yet) or the post failed.
        pub fn post_to_widget(&self, message: u32, wparam: usize, lparam: isize) -> bool {
            let Some(target) = self.input_target() else {
                return false;
            };
            // SAFETY: plain integers and a window handle of this browser; PostMessageW validates the handle and fails
            // for a window that is gone, which is handled below.
            let posted = unsafe {
                PostMessageW(
                    Some(HWND(target as *mut core::ffi::c_void)),
                    message,
                    WPARAM(wparam),
                    LPARAM(lparam),
                )
            };
            if posted.is_err() {
                // The widget may have been recreated: look for it again next time.
                self.target.set(0);
            }
            posted.is_ok()
        }

        /// Turns the mouse wheel by `delta` (120 per notch) at `(x, y)` of the page, through the browser host.
        pub fn send_mouse_wheel(&self, x: i32, y: i32, delta: i32) {
            if let Some(host) = self.browser.host() {
                host.send_mouse_wheel_event(Some(&MouseEvent { x, y, modifiers: 0 }), 0, delta);
            }
        }

        /// Shows DevTools for this browser in a window of its own: the Portal's Open DevTools button (D-13).
        pub fn open_devtools(&self) {
            show_devtools(&self.browser, None);
        }

        /// Runs a script in the main frame, as the page calls do (the in-page pause). The page gets no host object and no
        /// message channel: the script string is all it receives.
        pub fn execute_script(&self, script: &str) {
            if let Some(frame) = self.browser.main_frame() {
                frame.execute_java_script(
                    Some(&CefString::from(script)),
                    Some(&CefString::from("sucrose://page-call")),
                    0,
                );
            }
        }

        /// Shows another address in the same browser (the Portal's Use between two Web themes).
        pub fn navigate(&self, url: &str) {
            if let Some(frame) = self.browser.main_frame() {
                frame.load_url(Some(&CefString::from(url)));
            }
        }

        /// Closes the browser, pumps until it is gone and shuts CEF down. Call it before the surface is dropped.
        ///
        /// CEF never gets to send its own close message (see `do_close_answer`): once it is ready (`DoClose`), or
        /// after [`super::FORCED_TEAR_DOWN_AFTER`] without that, the browser window is destroyed here, and CEF ends
        /// the browser with `OnBeforeClose`.
        pub fn close(self) {
            let Self {
                browser,
                shared,
                profile,
                ..
            } = self;
            // Before CloseBrowser: CEF may ask DoClose from inside it.
            shared.closing.store(true, Ordering::SeqCst);
            if let Some(host) = browser.host() {
                host.close_browser(1);
            }
            let started = Instant::now();
            let mut torn_down = false;
            loop {
                let ready = shared.ready_to_close.load(Ordering::SeqCst);
                match close_step(
                    shared.closed.load(Ordering::SeqCst),
                    ready,
                    torn_down,
                    started.elapsed(),
                ) {
                    CloseStep::Done => {
                        tracing::info!(elapsed_ms = started.elapsed().as_millis(), "the CEF browser closed");
                        break;
                    }
                    CloseStep::GiveUp => {
                        tracing::warn!("the CEF browser did not close in time, CEF shuts down anyway");
                        break;
                    }
                    CloseStep::TearDown => {
                        if !ready {
                            tracing::warn!("CEF was not ready to close the browser in time, its window is destroyed");
                        }
                        tear_down(&browser);
                        torn_down = true;
                    }
                    CloseStep::Pump => {
                        cef::do_message_loop_work();
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            // Every CEF reference goes before shutdown.
            drop(browser);
            cef::shutdown();
            // The profile folder is free for the next instance only once CEF no longer writes to it.
            drop(profile);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn shared() -> Arc<Shared> {
            Arc::new(Shared {
                closed: AtomicBool::new(false),
                closing: AtomicBool::new(false),
                ready_to_close: AtomicBool::new(false),
                on_loaded: Arc::new(|_, _| {}),
                page_calls: Arc::new(|_, _| Vec::new()),
            })
        }

        // Called through the handler's C function table, as CEF calls it; no CEF code runs.
        #[test]
        fn the_life_span_handler_never_lets_cef_close_the_desktop_window() {
            let shared = shared();
            let handler = SpanHandler::new(Arc::clone(&shared));

            assert_eq!(
                handler.do_close(None),
                1,
                "a window.close() from the page sends nothing to the shell"
            );
            assert!(
                !shared.ready_to_close.load(Ordering::SeqCst),
                "and the wallpaper stays open"
            );

            shared.closing.store(true, Ordering::SeqCst);
            assert_eq!(
                handler.do_close(None),
                1,
                "Sucrose's own close sends nothing to the shell either"
            );
            assert!(
                shared.ready_to_close.load(Ordering::SeqCst),
                "the close loop may destroy the browser window now"
            );

            handler.on_before_close(None);
            assert!(shared.closed.load(Ordering::SeqCst));
        }
    }
}

/// Targets without a CEF build (Windows x86, other systems): starting reports [`CefError::Unsupported`].
#[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
mod imp {
    use super::{CefError, CefStart, MAX_PUMP_DELAY_MS};

    /// Placeholder: no CEF on this target.
    #[derive(Debug)]
    pub struct CefHost;

    impl CefHost {
        /// Always fails: there is no CEF on this target.
        pub fn start(_parent_hwnd: u64, _rect: [i32; 4], _url: &str, _start: &CefStart) -> Result<Self, CefError> {
            Err(CefError::Unsupported)
        }

        /// Does nothing.
        pub fn do_work(&self) -> u32 {
            MAX_PUMP_DELAY_MS
        }

        /// Does nothing.
        pub fn set_rect(&self, _rect: [i32; 4]) -> Result<(), CefError> {
            Err(CefError::Unsupported)
        }

        /// Does nothing.
        pub fn open_devtools(&self) {}

        /// Does nothing: there is no browser.
        pub fn send_mouse_wheel(&self, _x: i32, _y: i32, _delta: i32) {}

        /// Does nothing: there is no browser window.
        pub fn post_to_widget(&self, _message: u32, _wparam: usize, _lparam: isize) -> bool {
            false
        }

        /// Does nothing.
        pub fn execute_script(&self, _script: &str) {}

        /// Does nothing.
        pub fn navigate(&self, _url: &str) {}

        /// Does nothing.
        pub fn close(self) {}
    }
}

pub use imp::CefHost;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_browser_fills_the_rectangle_and_is_never_empty() {
        assert_eq!(browser_size([-1920, 0, 0, 1080]), (1920, 1080));
        assert_eq!(browser_size([10, 10, 10, 10]), (1, 1));
        assert_eq!(browser_size([5, 5, 0, 0]), (1, 1));
    }

    #[test]
    fn profile_folder_is_both_cache_roots_and_no_profile_is_in_memory() {
        let folder = std::env::temp_dir().join("sucrose-profile-paths");

        let (root, cache) = profile_paths(Some(&folder)).expect("an absolute folder");
        let none = profile_paths(None).expect("no folder");

        assert_eq!(root, cache);
        assert_eq!(Path::new(&root), folder);
        assert_eq!(none, (String::new(), String::new()), "empty paths are in-memory caches");
    }

    #[test]
    fn certificate_errors_are_ignored_only_when_invalid_certificates_are_allowed() {
        let folder = std::env::temp_dir().join("sucrose-certificate-settings");

        let validating = settings_values(Some(&folder), false).expect("validating");
        let allowing = settings_values(Some(&folder), true).expect("allowing");
        let in_memory = settings_values(None, false).expect("in memory");

        assert!(
            !validating.ignore_certificate_errors,
            "certificates are validated by default (D-12)"
        );
        assert!(
            allowing.ignore_certificate_errors,
            "Allow invalid certificates turns the switch on"
        );
        assert!(!in_memory.ignore_certificate_errors);
        assert_eq!(
            allowing.root_cache_path, validating.root_cache_path,
            "the choice does not move the profile"
        );
        assert_eq!(IGNORE_CERTIFICATE_ERRORS_SWITCH, "ignore-certificate-errors");
    }

    #[test]
    fn profile_folder_is_made_absolute_for_cef() {
        let (root, _) = profile_paths(Some(Path::new("Web").join("Cef").as_path())).expect("a relative folder");

        assert!(Path::new(&root).is_absolute(), "{root}");
        assert!(Path::new(&root).ends_with(Path::new("Web").join("Cef")), "{root}");
    }

    #[test]
    fn the_wallpaper_browser_never_lets_cef_send_its_close_message() {
        assert_eq!(
            do_close_answer(false),
            1,
            "CEF's default would post WM_CLOSE to Progman (Shut Down Windows)"
        );
        assert_eq!(
            do_close_answer(true),
            0,
            "a popup of the page is a top-level window of its own and closes the standard way"
        );
    }

    #[test]
    fn the_close_destroys_the_browser_window_once_cef_is_ready() {
        let soon = Duration::from_millis(20);
        assert_eq!(close_step(false, true, false, soon), CloseStep::TearDown);
        assert_eq!(
            close_step(false, true, true, soon),
            CloseStep::Pump,
            "the window goes once"
        );
        assert_eq!(
            close_step(false, false, false, soon),
            CloseStep::Pump,
            "CEF is not ready yet"
        );
        assert_eq!(close_step(true, true, true, soon), CloseStep::Done);
        assert_eq!(
            close_step(true, false, false, Duration::ZERO),
            CloseStep::Done,
            "a browser already gone needs nothing"
        );
    }

    #[test]
    fn the_close_destroys_the_window_anyway_when_cef_never_gets_ready_and_gives_up_at_the_timeout() {
        let just_before = FORCED_TEAR_DOWN_AFTER - Duration::from_millis(1);
        assert_eq!(close_step(false, false, false, just_before), CloseStep::Pump);
        assert_eq!(
            close_step(false, false, false, FORCED_TEAR_DOWN_AFTER),
            CloseStep::TearDown
        );
        assert_eq!(
            close_step(false, false, true, CLOSE_TIMEOUT - Duration::from_millis(1)),
            CloseStep::Pump
        );
        assert_eq!(close_step(false, false, true, CLOSE_TIMEOUT), CloseStep::GiveUp);
        assert_eq!(close_step(false, true, false, CLOSE_TIMEOUT), CloseStep::GiveUp);
        assert_eq!(close_step(true, false, true, CLOSE_TIMEOUT), CloseStep::Done);
    }

    #[test]
    fn the_inspect_item_uses_the_first_application_menu_id() {
        assert_eq!(INSPECT_COMMAND_ID, 26500);
    }

    #[test]
    fn the_pump_delay_follows_what_cef_asked_for_within_the_cap() {
        assert_eq!(next_pump_delay_ms(None), MAX_PUMP_DELAY_MS);
        assert_eq!(next_pump_delay_ms(Some(0)), 1);
        assert_eq!(next_pump_delay_ms(Some(-5)), 1);
        assert_eq!(next_pump_delay_ms(Some(10)), 10);
        assert_eq!(next_pump_delay_ms(Some(5_000)), MAX_PUMP_DELAY_MS);
        assert_eq!(next_pump_delay_ms(Some(i64::MAX)), MAX_PUMP_DELAY_MS);
    }

    #[test]
    fn scheduling_keeps_the_soonest_request_and_wakes_the_loop() {
        let woken = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = Arc::clone(&woken);
        let pump = Pump {
            requested: Mutex::new(None),
            wake: Arc::new(move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        };

        pump.schedule(40);
        pump.schedule(15);
        pump.schedule(30);

        assert_eq!(woken.load(std::sync::atomic::Ordering::SeqCst), 3);
        assert_eq!(pump.peek(), Some(15));
        assert_eq!(pump.take(), Some(15));
        assert_eq!(pump.take(), None);
    }
}
