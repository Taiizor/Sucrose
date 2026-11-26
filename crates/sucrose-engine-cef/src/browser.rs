//! Windowed CEF browser hosted as a child of a renderer's desktop surface (D-013, D-32, D-036).
//!
//! [Findings 1 of the Phase 4 research] CEF runs on the thread that created the surface, with the external message
//! pump: CEF asks to be pumped through `OnScheduleMessagePumpWork` (see [`CefStart::wake`]), the renderer's loop calls
//! [`CefHost::do_work`] on that wake and on its pump timer, and there is no second message loop and no
//! multi-threaded message loop. Every method of [`CefHost`] must be called on that one thread.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use sucrose_engine_web::profile::ProfileLease;

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
    /// The folders the virtual origin serves, in lookup order: the theme folder, then the generated-content folder.
    pub roots: Vec<PathBuf>,
    /// The browser profile this instance runs on (D-33, D-35); `None` runs on in-memory caches, which keep nothing.
    /// The host keeps the lease until CEF has shut down, so the folder stays this instance's all that time.
    pub profile: Option<Arc<ProfileLease>>,
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
        Foundation::HWND,
        System::LibraryLoader::LoadLibraryW,
        UI::WindowsAndMessaging::{SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos},
    };

    use super::{
        CefError, CefStart, LoadedCallback, ProfileLease, Pump, browser_size, next_pump_delay_ms, profile_paths,
    };

    /// How long `close` pumps for the browser to go away before it gives up and shuts CEF down anyway.
    const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

    /// What the handlers share with the host.
    struct Shared {
        closed: AtomicBool,
        on_loaded: LoadedCallback,
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
        }

        impl App {
            fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
                Some(self.handler.clone())
            }
        }
    }

    wrap_life_span_handler! {
        struct SpanHandler {
            shared: Arc<Shared>,
        }

        impl LifeSpanHandler {
            fn on_before_close(&self, _browser: Option<&mut Browser>) {
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

    wrap_context_menu_handler! {
        struct NoMenu;

        impl ContextMenuHandler {
            // The legacy host has no context menu; until the menu work of a later plan it stays empty.
            fn on_before_context_menu(
                &self,
                _browser: Option<&mut Browser>,
                _frame: Option<&mut Frame>,
                _params: Option<&mut ContextMenuParams>,
                model: Option<&mut MenuModel>,
            ) {
                if let Some(model) = model {
                    model.clear();
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
                on_loaded: Arc::clone(&start.on_loaded),
            });
            let mut app = HostApp::new(PumpHandler::new(Arc::clone(&pump)));
            // The persistent profile of the first instance or an ephemeral one (D-33); both paths name the same folder, so
            // CEF's process singleton lock and the profile never meet another running instance's.
            let (root_cache_path, cache_path) =
                profile_paths(start.profile.as_deref().map(|lease| lease.path.as_path()))?;
            let settings = Settings {
                no_sandbox: 1,
                external_message_pump: 1,
                multi_threaded_message_loop: 0,
                windowless_rendering_enabled: 0,
                browser_subprocess_path: CefString::from(utf8(&start.subprocess_path, "subprocess")?.as_str()),
                log_file: CefString::from(utf8(&start.log_file, "log file")?.as_str()),
                locale: CefString::from("en-US"),
                root_cache_path: CefString::from(root_cache_path.as_str()),
                cache_path: CefString::from(cache_path.as_str()),
                // The legacy host's PersistSessionCookies; it only takes effect with a cache path.
                persist_session_cookies: 1,
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
                NoMenu::new(),
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

        /// Shows another address in the same browser (the Portal's Use between two Web themes).
        pub fn navigate(&self, url: &str) {
            if let Some(frame) = self.browser.main_frame() {
                frame.load_url(Some(&CefString::from(url)));
            }
        }

        /// Closes the browser, pumps until it is gone and shuts CEF down. Call it before the surface is dropped.
        pub fn close(self) {
            let Self {
                browser,
                shared,
                profile,
                ..
            } = self;
            if let Some(host) = browser.host() {
                host.close_browser(1);
            }
            let deadline = Instant::now() + CLOSE_TIMEOUT;
            while !shared.closed.load(Ordering::SeqCst) && Instant::now() < deadline {
                cef::do_message_loop_work();
                thread::sleep(Duration::from_millis(10));
            }
            if !shared.closed.load(Ordering::SeqCst) {
                tracing::warn!("the CEF browser did not close in time, CEF shuts down anyway");
            }
            // Every CEF reference goes before shutdown.
            drop(browser);
            cef::shutdown();
            // The profile folder is free for the next instance only once CEF no longer writes to it.
            drop(profile);
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
    fn profile_folder_is_made_absolute_for_cef() {
        let (root, _) = profile_paths(Some(Path::new("Web").join("Cef").as_path())).expect("a relative folder");

        assert!(Path::new(&root).is_absolute(), "{root}");
        assert!(Path::new(&root).ends_with(Path::new("Web").join("Cef")), "{root}");
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
