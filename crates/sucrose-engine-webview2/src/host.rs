//! WebView2 on the surface thread: the environment, the controller and the WebResourceRequested adapter of the virtual
//! origin. Every COM object here is created on the thread that owns the desktop surface and is only used there (the
//! WebView2 threading model: callbacks arrive through that thread's message loop), so nothing in this module is `Send`.
//!
//! The pure parts (bounds, scale, the header block of a reply, the stream path) are plain functions with tests; the
//! COM calls are in [`Host`], which is Windows only. Other systems get a stub that refuses to start (D-002).

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use sucrose_engine_web::{js::PageCall, serve::Reply};

/// Called on the surface thread after every completed main-frame navigation, with its URL and HTTP status (0 when the
/// navigation failed).
pub type LoadedHook = Arc<dyn Fn(&str, i32) + Send + Sync>;

/// Called on the surface thread at the end of every main-frame navigation, before [`LoadedHook`], with the URL and the
/// HTTP status (0 when the navigation failed). The scripts it returns run in that page in order (WAPI-01, WAPI-02).
pub type PageCallsHook = Arc<dyn Fn(&str, i32) -> Vec<PageCall> + Send + Sync>;

/// What the host needs to start, besides the surface and the page.
pub struct HostStart {
    /// The folders the virtual origin serves from, in order: the theme folder, then the generated content (D-14).
    pub roots: Vec<PathBuf>,
    /// The WebView2 user data folder: the leased profile (D-33, D-35).
    pub user_data_folder: PathBuf,
    /// The text of `AdditionalBrowserArguments`: the user's switches merged with Sucrose's own (D-17, WAPI-10). The
    /// environment is created with it, so it cannot change while the renderer runs.
    pub browser_arguments: String,
    /// `Engine.DeveloperMode`: the page gets its context menu and the F12 / Ctrl+Shift+I shortcuts (D-13).
    pub developer_mode: bool,
    /// `Engine.AllowInvalidCertificates`: certificate errors are allowed instead of refused (D-12). Off by default.
    pub allow_invalid_certificates: bool,
    /// `General.UserAgent` as the service decided it (legacy `CoreWebView2.Settings.UserAgent`); empty keeps
    /// WebView2's own.
    pub user_agent: String,
    /// The scripts a page gets after every completed main-frame navigation: they run before `on_loaded`.
    pub page_calls: PageCallsHook,
    /// Called after every completed main-frame navigation.
    pub on_loaded: LoadedHook,
    /// A script that runs in every new document before the page's own scripts: the time base shim of a Web or Url page
    /// that is part of a theme group (D-04). `None` for a single wallpaper and for a generated page.
    pub document_start_script: Option<String>,
}

/// Why WebView2 did not start.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    /// This build or system has no WebView2.
    #[error("WebView2 is not available on this system")]
    Unsupported,
    /// No WebView2 Runtime is installed.
    #[error("the WebView2 Runtime is not installed")]
    RuntimeMissing,
    /// The thread was already set up for another COM threading model.
    #[error("the surface thread cannot host WebView2: {0}")]
    ComMode(String),
    /// The WebView2 environment could not be created.
    #[error("cannot create the WebView2 environment: {0}")]
    Environment(String),
    /// The WebView2 controller could not be created.
    #[error("cannot create the WebView2 controller: {0}")]
    Controller(String),
    /// The controller or the page could not be set up.
    #[error("cannot set up the WebView2 page: {0}")]
    Setup(String),
    /// WebView2 did not answer in time.
    #[error("WebView2 did not finish {0} in time")]
    TimedOut(&'static str),
    /// The message loop was asked to quit while WebView2 was starting.
    #[error("the message loop ended while WebView2 was starting")]
    Cancelled,
}

/// The width and height of the controller for a surface rectangle (left, top, right, bottom): never below one pixel.
#[must_use]
pub fn bounds_size(rect: [i32; 4]) -> (i32, i32) {
    (
        rect[2].saturating_sub(rect[0]).max(1),
        rect[3].saturating_sub(rect[1]).max(1),
    )
}

/// The rasterization scale for a monitor DPI: 96 DPI is 1.0. An unknown DPI (0) is 1.0.
#[must_use]
pub fn scale_for_dpi(dpi: u32) -> f64 {
    if dpi == 0 { 1.0 } else { f64::from(dpi) / 96.0 }
}

/// The header block WebView2 takes for a reply: the type and every planned header, one `Name: value` per line.
#[must_use]
pub fn header_block(reply: &Reply) -> String {
    let mut block = format!("Content-Type: {}\r\n", reply.mime);
    for (name, value) in &reply.headers {
        block.push_str(name);
        block.push_str(": ");
        block.push_str(value);
        block.push_str("\r\n");
    }
    block
}

/// `reply` with its `Content-Range` replaced by `content_range`: the reply of a range cut to the in-memory cap names the
/// bytes it carries, not the range the planner first decided.
#[must_use]
pub fn ranged_reply(reply: &Reply, content_range: &str) -> Reply {
    let mut ranged = reply.clone();
    ranged.headers.retain(|(name, _)| *name != "Content-Range");
    ranged.headers.push(("Content-Range", content_range.to_owned()));
    ranged
}

/// The path to open a served file with: the canonical form Windows gives (`\\?\C:\...`) without its verbatim prefix,
/// which the shell stream API does not take. A UNC path keeps its prefix.
#[must_use]
pub fn stream_path(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC\\") => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

/// What the host sets on the page's settings and which handlers it registers, from the two settings that decide them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewSettings {
    /// The developer tools exist in the page. Always true: the Portal's Open DevTools button needs them, and nothing opens
    /// them by itself (D-13); the keys and the menu are what keep them closed.
    pub devtools_enabled: bool,
    /// The default context menu (with Inspect) shows only in developer mode.
    pub context_menus: bool,
    /// The browser accelerator keys stay on so the key table sees them (it swallows the blocked ones).
    pub accelerator_keys: bool,
    /// A certificate error handler is registered that allows every error: only with Allow invalid certificates (D-12).
    /// Without it nothing is registered, so Chromium rejects an invalid certificate.
    pub allow_invalid_certificates: bool,
}

/// The view settings for developer mode and the certificate switch (D-12, D-13).
#[must_use]
pub fn view_settings(developer_mode: bool, allow_invalid_certificates: bool) -> ViewSettings {
    ViewSettings {
        devtools_enabled: true,
        context_menus: developer_mode,
        accelerator_keys: true,
        allow_invalid_certificates,
    }
}

/// The user agent to set on the page's settings: `None` for an empty value, so the COM call is skipped instead of relying
/// on `put_UserAgent`'s empty-string rule, otherwise the value as given (the service already refused blank and
/// control-character values).
#[must_use]
pub fn user_agent_override(user_agent: &str) -> Option<&str> {
    (!user_agent.is_empty()).then_some(user_agent)
}

/// A number as JSON text with no exponent and at most three decimals: the invariant form the DevTools protocol takes (the
/// legacy host formatted with the invariant culture too).
fn json_number(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    // -0 prints as "-0"; both zeros are plain 0.
    if rounded == 0.0 {
        "0".to_owned()
    } else {
        rounded.to_string()
    }
}

/// The parameters of the `Input.dispatchMouseEvent` call that turns the wheel: `x` and `y` are relative to the page's
/// surface in physical pixels, `delta` is the raw wheel amount (120 per notch, positive away from the user). The
/// protocol takes CSS pixels, so everything is divided by the monitor `scale` (120 at 150 % is 80); the page zoom stays
/// 1. The vertical amount is negated, as the legacy engine does (engines.md section 5).
#[must_use]
pub fn wheel_params(delta: i32, x: i32, y: i32, scale: f64) -> String {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    format!(
        r#"{{"type":"mouseWheel","x":{},"y":{},"deltaX":0,"deltaY":{},"modifiers":0,"pointerType":"mouse"}}"#,
        json_number(f64::from(x) / scale),
        json_number(f64::from(y) / scale),
        json_number(-f64::from(delta) / scale),
    )
}

/// The host name of `uri` for a log line: what follows `://` up to the first `/`, `?`, `#` or `:` (a user name before
/// an `@` is dropped). Never the path or the query, which can carry secrets.
#[must_use]
pub fn uri_host(uri: &str) -> &str {
    let rest = uri.split_once("://").map_or(uri, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    if let Some(bracketed) = host.strip_prefix('[') {
        return bracketed.split_once(']').map_or(bracketed, |(inside, _)| inside);
    }
    host.split(':').next().unwrap_or_default()
}

/// The class of WebView2's host window: created in this process under the surface, it holds the browser's widget and does
/// not hand posted input on to the page.
pub const HOST_WIDGET_CLASS: &str = "Chrome_WidgetWin_0";

/// The class of the browser process's widget that takes the posted input messages and feeds them to the page.
pub const INPUT_WIDGET_CLASS: &str = "Chrome_WidgetWin_1";

/// The window the legacy input messages go to, in the window tree under `surface` (the window that holds the controller),
/// given each window's direct children and class: the [`INPUT_WIDGET_CLASS`] child of the first [`HOST_WIDGET_CLASS`]
/// window, as the legacy app walks WebView2's window chain (`Helper/Handle.cs`), else the first [`INPUT_WIDGET_CLASS`]
/// window. The host window itself is never the target; while it holds no widget yet there is none, and the next event
/// looks again. Each search takes the direct children first, then goes down to two levels deeper; only the surface's own
/// tree is searched, never the desktop (T-04-46).
#[must_use]
pub fn input_widget<W: Copy>(surface: W, children: impl Fn(W) -> Vec<W>, class: impl Fn(W) -> String) -> Option<W> {
    fn search<W: Copy>(
        parent: W,
        wanted: &str,
        depth: u32,
        children: &impl Fn(W) -> Vec<W>,
        class: &impl Fn(W) -> String,
    ) -> Option<W> {
        let below = children(parent);
        if let Some(found) = below.iter().copied().find(|child| class(*child) == wanted) {
            return Some(found);
        }
        if depth == 0 {
            return None;
        }
        below
            .into_iter()
            .find_map(|child| search(child, wanted, depth - 1, children, class))
    }
    match search(surface, HOST_WIDGET_CLASS, 2, &children, &class) {
        Some(host) => children(host)
            .into_iter()
            .find(|child| class(*child) == INPUT_WIDGET_CLASS),
        None => search(surface, INPUT_WIDGET_CLASS, 2, &children, &class),
    }
}

#[cfg(windows)]
mod imp {
    use std::{
        cell::Cell,
        path::{Path, PathBuf},
        rc::Rc,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };

    use sucrose_engine_web::{
        keys::{KeyAction, key_action},
        origin,
        serve::{self, Content},
        youtube::{referer_for, request_filters},
    };
    use webview2_com::{
        AcceleratorKeyPressedEventHandler, AddScriptToExecuteOnDocumentCreatedCompletedHandler,
        BrowserProcessExitedEventHandler, CallDevToolsProtocolMethodCompletedHandler, CoreWebView2EnvironmentOptions,
        CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
        DownloadStartingEventHandler, ExecuteScriptCompletedHandler,
        Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_BOUNDS_MODE_USE_RAW_PIXELS, COREWEBVIEW2_COLOR, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
            COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN, COREWEBVIEW2_SERVER_CERTIFICATE_ERROR_ACTION_ALWAYS_ALLOW,
            COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL, CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2,
            ICoreWebView2_4, ICoreWebView2_14, ICoreWebView2Controller, ICoreWebView2Controller2,
            ICoreWebView2Controller3, ICoreWebView2Environment, ICoreWebView2Environment5,
            ICoreWebView2EnvironmentOptions, ICoreWebView2NavigationCompletedEventArgs,
            ICoreWebView2NavigationCompletedEventArgs2, ICoreWebView2Settings2, ICoreWebView2Settings3,
            ICoreWebView2WebResourceRequest, ICoreWebView2WebResourceRequestedEventArgs,
        },
        NavigationCompletedEventHandler, ServerCertificateErrorDetectedEventHandler, WebResourceRequestedEventHandler,
        take_pwstr,
    };
    use windows::{
        Win32::{
            Foundation::{E_POINTER, HWND, LPARAM, RECT, RPC_E_CHANGED_MODE, WPARAM},
            Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromRect},
            System::Com::{
                COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize, IStream, STGM_READ, STGM_SHARE_DENY_NONE,
            },
            UI::{
                HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
                Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_SHIFT},
                Shell::{SHCreateMemStream, SHCreateStreamOnFileEx},
                WindowsAndMessaging::{
                    DispatchMessageW, FindWindowExW, GetClassNameW, MSG, MsgWaitForMultipleObjects, PM_REMOVE,
                    PeekMessageW, PostMessageW, PostQuitMessage, QS_ALLINPUT, TranslateMessage, WM_QUIT,
                },
            },
        },
        core::{BOOL, Interface, PCWSTR, PWSTR},
    };

    use super::{
        HostError, HostStart, INPUT_WIDGET_CLASS, bounds_size, header_block, input_widget, ranged_reply, scale_for_dpi,
        stream_path, uri_host, user_agent_override, view_settings, wheel_params,
    };

    /// How long the environment and the controller get to come up.
    const CREATE_TIMEOUT: Duration = Duration::from_secs(30);

    /// How long a closing host waits for WebView2's browser process to exit before it lets go of the profile folder
    /// (04-37 D10, D-040). The browser exits about 0.2-0.35 s after the close. Now and then the Runtime takes a slower
    /// path of its own shutdown: every child process has ended by about 0.3 s, and the browser process then waits,
    /// idle, until it exits 0.77-1.12 s after the close on an otherwise idle machine. An audio output stream on a
    /// profile folder's first start sends it there in about 1 close in 10, and a close in the first second of a start
    /// while the parallel test suite runs nearly always (0.82-1.25 s), mostly for both monitors' browsers at once;
    /// nothing in Sucrose's close sequence or the page's work does (debug session webview2-browser-exit-can-exce,
    /// WebView2 Runtime 158). The bound of 1.5 s waits that path out (1251 ms the longest seen) and stays under the
    /// Quit grace (2 s) and the library move grace (4 s), which end a renderer that is still waiting. Two cases outlast
    /// any bound under those graces: a CPU saturated by other work (more than 2.5 s with 16 busy threads on 16 logical
    /// CPUs; 8 change nothing), and Edge SmartScreen (about 6.3 s): a folder that was never used sends a SmartScreen
    /// check with its first navigation, and a close while it is in flight holds the browser's tear-down (debug session
    /// webview2-browser-exit-slow); Sucrose's own switches turn SmartScreen off by default (D-041;
    /// `Engine.WebViewSmartScreen` turns it on). A bound that covered them would only make the switch wait for them. A
    /// later exit is logged and the next renderer retries the folder.
    const BROWSER_EXIT_WAIT: Duration = Duration::from_millis(1500);

    /// How long to wait before the one retry of the environment (a previous renderer's browser process may still hold
    /// the same user data folder).
    const RETRY_DELAY: Duration = Duration::from_secs(2);

    /// The HRESULT of ERROR_FILE_NOT_FOUND: what the loader answers when no WebView2 Runtime is installed.
    const RUNTIME_NOT_FOUND: i32 = 0x8007_0002_u32 as i32;

    /// A NUL-terminated UTF-16 copy of `text` for a `PCWSTR` argument.
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// The same for a path.
    fn wide_path(path: &Path) -> Vec<u16> {
        wide(&path.to_string_lossy())
    }

    /// Keeps a COM error as the host error of the step that failed.
    fn classify(step: fn(String) -> HostError, error: &windows::core::Error) -> HostError {
        if error.code().0 == RUNTIME_NOT_FOUND {
            HostError::RuntimeMissing
        } else {
            step(error.to_string())
        }
    }

    /// Pumps this thread's messages until `rx` has a value, up to [`CREATE_TIMEOUT`]. WebView2 delivers its completion
    /// callbacks through the message loop of the thread that started the operation, so waiting without pumping would
    /// never finish.
    fn wait_for<T>(rx: &mpsc::Receiver<T>, what: &'static str) -> Result<T, HostError> {
        pump_until(rx, what, CREATE_TIMEOUT)
    }

    /// [`wait_for`] with the time it may take.
    fn pump_until<T>(rx: &mpsc::Receiver<T>, what: &'static str, timeout: Duration) -> Result<T, HostError> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(value) = rx.try_recv() {
                return Ok(value);
            }
            if Instant::now() >= deadline {
                return Err(HostError::TimedOut(what));
            }
            // SAFETY: no handles are passed; the call only sleeps until a message arrives or 50 ms pass.
            let _ = unsafe { MsgWaitForMultipleObjects(None, false, 50, QS_ALLINPUT) };
            let mut message = MSG::default();
            // SAFETY: `message` is filled by PeekMessageW and handed unchanged to the two calls that follow.
            while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                if message.message == WM_QUIT {
                    // The loop that owns this thread is ending: let it see the request again.
                    // SAFETY: plain call; the exit code is the one the request carried.
                    unsafe { PostQuitMessage(i32::try_from(message.wParam.0).unwrap_or(0)) };
                    return Err(HostError::Cancelled);
                }
                // SAFETY: see above.
                unsafe {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }

    /// A registration for the exit of the WebView2 browser process of an environment (04-37 D10).
    struct ExitWatch {
        environment: ICoreWebView2Environment5,
        token: i64,
        exited: mpsc::Receiver<()>,
    }

    /// Registers for the exit of the browser process of `environment`; `None` when the runtime is too old to say.
    fn watch_browser_exit(environment: &ICoreWebView2Environment) -> Option<ExitWatch> {
        let environment = match environment.cast::<ICoreWebView2Environment5>() {
            Ok(environment) => environment,
            Err(error) => {
                tracing::debug!(%error, "this WebView2 runtime cannot report its browser process exit");
                return None;
            }
        };
        let (sender, exited) = mpsc::channel();
        let handler = BrowserProcessExitedEventHandler::create(Box::new(move |_environment, _args| {
            // The receiver is gone after a timeout; nobody waits for the event then.
            let _ = sender.send(());
            Ok(())
        }));
        let mut token = 0_i64;
        // SAFETY: the handler is a COM object the call keeps its own reference to, and `token` outlives the call.
        match unsafe { environment.add_BrowserProcessExited(&handler, &mut token) } {
            Ok(()) => Some(ExitWatch {
                environment,
                token,
                exited,
            }),
            Err(error) => {
                tracing::debug!(%error, "cannot watch the WebView2 browser process");
                None
            }
        }
    }

    /// Pumps this thread's messages until the browser process of `watch` exited or [`BROWSER_EXIT_WAIT`] passed, logs
    /// which, and removes the registration.
    fn await_browser_exit(watch: ExitWatch) {
        let started = Instant::now();
        let outcome = pump_until(&watch.exited, "the WebView2 browser process to exit", BROWSER_EXIT_WAIT);
        let elapsed_ms = started.elapsed().as_millis();
        match outcome {
            Ok(()) => tracing::info!(elapsed_ms, "webview2 browser process exited"),
            Err(HostError::TimedOut(_)) => tracing::warn!(
                elapsed_ms,
                "the WebView2 browser process did not exit in time; its profile folder may still be in use"
            ),
            Err(_) => tracing::debug!(
                elapsed_ms,
                "the message loop is ending, not waiting for the WebView2 browser process"
            ),
        }
        // SAFETY: the token came from the registration on this environment; a failure only means it is gone.
        let _ = unsafe { watch.environment.remove_BrowserProcessExited(watch.token) };
    }

    /// One attempt at the environment on `folder`.
    fn try_environment(folder: &Path, arguments: &str) -> Result<ICoreWebView2Environment, HostError> {
        let (sender, receiver) = mpsc::channel();
        let options = CoreWebView2EnvironmentOptions::default();
        // SAFETY: the options object is not shared with COM yet, so nothing else reads these fields. Language stays
        // empty (the OS default), extensions and custom crash reporting stay off, and the browser arguments are the
        // user's switches merged with Sucrose's own (D-17).
        unsafe {
            options.set_are_browser_extensions_enabled(false);
            options.set_is_custom_crash_reporting_enabled(false);
            options.set_additional_browser_arguments(arguments.to_owned());
        }
        let options: ICoreWebView2EnvironmentOptions = options.into();
        let handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |code, environment| {
            let result = code.and_then(|()| environment.ok_or_else(|| windows::core::Error::from(E_POINTER)));
            // The receiver is gone after a timeout; nobody wants the environment then.
            let _ = sender.send(result);
            Ok(())
        }));
        let folder = wide_path(folder);
        // SAFETY: `folder` is NUL-terminated and outlives the call; the options and the handler are COM objects the
        // loader keeps its own references to.
        unsafe {
            CreateCoreWebView2EnvironmentWithOptions(PCWSTR::null(), PCWSTR(folder.as_ptr()), &options, &handler)
        }
        .map_err(|error| classify(HostError::Environment, &error))?;
        wait_for(&receiver, "creating the environment")?.map_err(|error| classify(HostError::Environment, &error))
    }

    /// The environment, with one retry after [`RETRY_DELAY`] when the first attempt failed (not when it timed out).
    fn create_environment(folder: &Path, arguments: &str) -> Result<ICoreWebView2Environment, HostError> {
        match try_environment(folder, arguments) {
            Err(HostError::Environment(first)) => {
                tracing::warn!(error = %first, "the WebView2 environment failed, trying once more");
                std::thread::sleep(RETRY_DELAY);
                try_environment(folder, arguments)
            }
            other => other,
        }
    }

    /// The controller of `environment` as a child of `parent`.
    fn create_controller(
        environment: &ICoreWebView2Environment,
        parent: HWND,
    ) -> Result<ICoreWebView2Controller, HostError> {
        let (sender, receiver) = mpsc::channel();
        let handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |code, controller| {
            let result = code.and_then(|()| controller.ok_or_else(|| windows::core::Error::from(E_POINTER)));
            let _ = sender.send(result);
            Ok(())
        }));
        // SAFETY: `parent` is the surface window of this thread; the handler is a COM object the call keeps a
        // reference to.
        unsafe { environment.CreateCoreWebView2Controller(parent, &handler) }
            .map_err(|error| classify(HostError::Controller, &error))?;
        wait_for(&receiver, "creating the controller")?.map_err(|error| classify(HostError::Controller, &error))
    }

    /// The scale of the monitor that holds `rect` (physical pixels): its effective DPI divided by 96, 1.0 when
    /// Windows does not say.
    fn monitor_scale(rect: [i32; 4]) -> f64 {
        let area = RECT {
            left: rect[0],
            top: rect[1],
            right: rect[2],
            bottom: rect[3],
        };
        // SAFETY: `area` is a valid RECT for the duration of the call; the nearest monitor always exists.
        let monitor = unsafe { MonitorFromRect(&area, MONITOR_DEFAULTTONEAREST) };
        let (mut dpi_x, mut dpi_y) = (0_u32, 0_u32);
        // SAFETY: both out pointers are valid for the call.
        match unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) } {
            Ok(()) => scale_for_dpi(dpi_x),
            Err(_) => 1.0,
        }
    }

    /// Opens a served file as a read-only stream that other readers may share.
    fn open_stream(path: &Path) -> windows::core::Result<IStream> {
        let wide = wide_path(&stream_path(path));
        // SAFETY: `wide` is NUL-terminated and outlives the call; the mode only reads and shares.
        unsafe {
            SHCreateStreamOnFileEx(
                PCWSTR(wide.as_ptr()),
                (STGM_READ | STGM_SHARE_DENY_NONE).0,
                0,
                false,
                None,
            )
        }
    }

    /// A memory stream holding a copy of `bytes` (at most [`serve::MAX_RANGE_BODY`] of them, so it fits the length the
    /// call takes).
    fn memory_stream(bytes: &[u8]) -> Option<IStream> {
        // SAFETY: the slice is valid for the call, which copies it into the stream.
        unsafe { SHCreateMemStream(Some(bytes)) }
    }

    /// The value of the request header `name`, when the request has it.
    fn request_header(request: &ICoreWebView2WebResourceRequest, name: &str) -> Option<String> {
        let name = wide(name);
        // SAFETY: the request and its headers belong to this event; `name` is NUL-terminated and outlives the calls;
        // the returned string is freed by `take_pwstr`.
        unsafe {
            let headers = request.Headers().ok()?;
            let mut present = BOOL(0);
            headers.Contains(PCWSTR(name.as_ptr()), &mut present).ok()?;
            if !present.as_bool() {
                return None;
            }
            let mut value = PWSTR::null();
            headers.GetHeader(PCWSTR(name.as_ptr()), &mut value).ok()?;
            Some(take_pwstr(value))
        }
    }

    /// The address of the request of `args`; `None` when WebView2 cannot say.
    fn request_uri(args: &ICoreWebView2WebResourceRequestedEventArgs) -> Option<String> {
        // SAFETY: the request belongs to this event; the returned string is freed by `take_pwstr`.
        unsafe {
            let request = args.Request().ok()?;
            let mut uri = PWSTR::null();
            request.Uri(&mut uri).ok()?;
            Some(take_pwstr(uri))
        }
    }

    /// Whether `uri` is an address of the virtual origin (what the planner answers).
    fn is_origin_uri(uri: &str) -> bool {
        uri.get(..origin::ORIGIN.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(origin::ORIGIN))
    }

    /// The Referer safety net of the YouTube page (D-14): a request to a YouTube host that carries no Referer gets the
    /// virtual origin. A Referer that is there is never replaced and the response is left to the network.
    fn fill_referer(args: &ICoreWebView2WebResourceRequestedEventArgs, uri: &str) -> windows::core::Result<()> {
        // SAFETY: the request and its headers belong to this event.
        let request = unsafe { args.Request()? };
        let existing = request_header(&request, "Referer");
        let Some(referer) = referer_for(uri, existing.as_deref()) else {
            return Ok(());
        };
        let name = wide("Referer");
        let value = wide(&referer);
        // SAFETY: the headers belong to this event; both strings are NUL-terminated and outlive the call.
        unsafe {
            request
                .Headers()?
                .SetHeader(PCWSTR(name.as_ptr()), PCWSTR(value.as_ptr()))?;
        }
        tracing::debug!(host = %uri_host(uri), "referer filled");
        Ok(())
    }

    /// The reply to send and its body for a planned `reply`: a range as a memory stream of at most
    /// [`serve::MAX_RANGE_BODY`] bytes whose `Content-Range` names exactly what it carries, a whole file as a stream
    /// from disk, a refusal as it is. A `HEAD` request gets the headers and no body.
    fn prepare(reply: serve::Reply, head: bool) -> (serve::Reply, Option<IStream>) {
        let Content::File { path, .. } = &reply.content else {
            return (reply, None);
        };
        let path = path.clone();
        if reply.status == 206 {
            let Some(body) = serve::range_body(&reply) else {
                tracing::info!("origin request rejected: the planned range cannot be cut");
                return (serve::Reply::not_found(), None);
            };
            if head {
                let content_range = body.content_range.clone();
                return (ranged_reply(&reply, &content_range), None);
            }
            return match serve::read_range(&path, body.offset, body.len) {
                Ok(bytes) if !bytes.is_empty() => {
                    // The file may have shrunk since the plan: the header names the bytes actually read.
                    let sent = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
                    let content_range = serve::content_range_text(body.offset, sent, body.total);
                    let planned = match &reply.content {
                        Content::File { len, .. } => *len,
                        Content::Text(_) => sent,
                    };
                    if sent < planned {
                        tracing::debug!(content_range = %content_range, "origin range cut to what fits in memory");
                    }
                    match memory_stream(&bytes) {
                        Some(stream) => (ranged_reply(&reply, &content_range), Some(stream)),
                        None => {
                            tracing::info!("origin request rejected: no memory stream for the range");
                            (serve::Reply::not_found(), None)
                        }
                    }
                }
                Ok(_) => {
                    tracing::info!("origin request rejected: the range is empty now");
                    (serve::Reply::not_found(), None)
                }
                Err(error) => {
                    tracing::info!(%error, "origin request rejected: the file cannot be read");
                    (serve::Reply::not_found(), None)
                }
            };
        }
        if head {
            return (reply, None);
        }
        match open_stream(&path) {
            Ok(stream) => (reply, Some(stream)),
            Err(error) => {
                tracing::info!(%error, "origin request rejected: the file cannot be opened");
                (serve::Reply::not_found(), None)
            }
        }
    }

    /// Logs what is sent for a request (this adapter's own lines: the planner logs under another target, so a run that
    /// enables only this engine's target still sees every request the handler got): a served file with its status and,
    /// for a range, the `Content-Range` of the bytes actually sent, a refusal at debug.
    fn log_served(uri: &str, reply: &serve::Reply) {
        let path: String = origin::url_path(uri).unwrap_or_default().chars().take(200).collect();
        if reply.status == 404 {
            tracing::debug!(%path, status = 404, "origin request rejected");
            return;
        }
        if !(200..300).contains(&reply.status) {
            return;
        }
        match reply.headers.iter().find(|(name, _)| *name == "Content-Range") {
            Some((_, content_range)) => {
                tracing::debug!(%path, status = reply.status, content_range = %content_range, "origin request served");
            }
            None => tracing::debug!(%path, status = reply.status, "origin request served"),
        }
    }

    /// Answers one request of the virtual origin from `serve::plan`: a range as a capped memory stream, a whole file
    /// as a stream from disk, an empty body for every refusal.
    fn answer(
        environment: &ICoreWebView2Environment,
        roots: &[PathBuf],
        args: &ICoreWebView2WebResourceRequestedEventArgs,
    ) -> windows::core::Result<()> {
        // SAFETY: the request, its URI and its method belong to this event; each returned string is freed by
        // `take_pwstr` (a failed call leaves it null, which reads empty).
        let (uri, range, head) = unsafe {
            let request = args.Request()?;
            let mut uri = PWSTR::null();
            request.Uri(&mut uri)?;
            let uri = take_pwstr(uri);
            let mut method = PWSTR::null();
            let _ = request.Method(&mut method);
            let head = take_pwstr(method).eq_ignore_ascii_case("HEAD");
            (uri, request_header(&request, "Range"), head)
        };
        let roots: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
        let (reply, stream) = prepare(serve::plan(&uri, range.as_deref(), &roots), head);
        log_served(&uri, &reply);
        respond(environment, args, &reply, stream.as_ref())
    }

    /// Hands `reply` to WebView2 for the request of `args`.
    fn respond(
        environment: &ICoreWebView2Environment,
        args: &ICoreWebView2WebResourceRequestedEventArgs,
        reply: &serve::Reply,
        stream: Option<&IStream>,
    ) -> windows::core::Result<()> {
        let reason = wide(reply.status_text);
        let headers = wide(&header_block(reply));
        // SAFETY: both strings are NUL-terminated and outlive the call; the stream is a COM object the response keeps
        // its own reference to.
        unsafe {
            let response = environment.CreateWebResourceResponse(
                stream,
                i32::from(reply.status),
                PCWSTR(reason.as_ptr()),
                PCWSTR(headers.as_ptr()),
            )?;
            args.SetResponse(&response)
        }
    }

    /// The direct children of `parent`, in Z order.
    fn child_windows(parent: HWND) -> Vec<HWND> {
        let mut children = Vec::new();
        let mut previous: Option<HWND> = None;
        // SAFETY: `parent` is a window of this thread's surface tree; FindWindowExW only reads the window tree and fails
        // for a window that is gone; the previous child is a handle it returned a moment ago.
        while let Ok(child) = unsafe { FindWindowExW(Some(parent), previous, PCWSTR::null(), PCWSTR::null()) } {
            children.push(child);
            previous = Some(child);
        }
        children
    }

    /// The class name of `window`, empty when it is gone.
    fn window_class(window: HWND) -> String {
        let mut name = [0_u16; 256];
        // SAFETY: the buffer is valid for its whole length, which is the count passed; a window that is gone gives 0.
        let copied = unsafe { GetClassNameW(window, &mut name) };
        String::from_utf16_lossy(&name[..usize::try_from(copied).unwrap_or(0).min(name.len())])
    }

    /// The Chromium widget that takes the posted input messages, chosen by [`input_widget`] in the window tree under
    /// `parent` (the surface that holds the controller), with the class it has for the log.
    fn find_widget(parent: HWND) -> Option<(isize, &'static str)> {
        input_widget(parent, child_windows, window_class).map(|found| (found.0 as isize, INPUT_WIDGET_CLASS))
    }

    /// The WebView2 pieces of one renderer; closing it (or dropping it) releases them in order.
    pub struct Host {
        environment: Option<ICoreWebView2Environment>,
        controller: Option<ICoreWebView2Controller>,
        controller3: Option<ICoreWebView2Controller3>,
        webview: Option<ICoreWebView2>,
        tokens: Tokens,
        scale: Cell<f64>,
        /// The surface window that holds the controller: the root of the search for the input widget.
        parent: HWND,
        /// The Chromium widget input is posted to, 0 until found (after the first navigation, and again when a post
        /// fails). Shared with the navigation handler, which looks it up once the page exists.
        target: Rc<Cell<isize>>,
        /// Whether this host set COM up on its thread and so owes the matching `CoUninitialize`.
        com: bool,
    }

    /// The event registrations to remove before the controller closes.
    #[derive(Default)]
    struct Tokens {
        resource: Option<i64>,
        completed: Option<i64>,
        download: Option<i64>,
        certificate: Option<i64>,
        accelerator: Option<i64>,
    }

    impl Host {
        /// Starts WebView2 on the calling thread, which must own the surface `parent` and run its message loop, as a
        /// child of `parent` covering `rect` (the child sits at 0,0 of its parent), and navigates to `url`.
        pub fn start(parent: u64, rect: [i32; 4], url: &str, start: &HostStart) -> Result<Self, HostError> {
            // SAFETY: plain call; the matching CoUninitialize runs in `close_inner` for a successful call.
            let com = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            if com == RPC_E_CHANGED_MODE {
                return Err(HostError::ComMode(
                    "the thread was set up for another COM threading model".to_owned(),
                ));
            }
            if com.is_err() {
                return Err(HostError::ComMode(windows::core::Error::from(com).to_string()));
            }
            let mut host = Self {
                environment: None,
                controller: None,
                controller3: None,
                webview: None,
                tokens: Tokens::default(),
                scale: Cell::new(1.0),
                parent: HWND(parent as usize as *mut std::ffi::c_void),
                target: Rc::new(Cell::new(0)),
                com: true,
            };
            host.open(parent, rect, url, start)?;
            Ok(host)
        }

        /// Creates the environment and the controller and shows the page. A failure leaves the pieces made so far in
        /// `self`, which the caller drops.
        fn open(&mut self, parent: u64, rect: [i32; 4], url: &str, start: &HostStart) -> Result<(), HostError> {
            let setup = |error: windows::core::Error| HostError::Setup(error.to_string());
            let environment = create_environment(&start.user_data_folder, &start.browser_arguments)?;
            self.environment = Some(environment.clone());
            let window = HWND(parent as usize as *mut std::ffi::c_void);
            let controller = create_controller(&environment, window)?;
            self.controller = Some(controller.clone());

            // SAFETY: the controller and the webview belong to this thread; every call below only passes plain
            // values, strings that outlive it (the UTF-16 buffer of the user agent lives until the call returns), or
            // COM objects whose references the callee takes.
            unsafe {
                let webview = controller.CoreWebView2().map_err(setup)?;
                self.webview = Some(webview.clone());

                // Opaque black until the page paints (the CEF default as well).
                if let Ok(second) = controller.cast::<ICoreWebView2Controller2>() {
                    second
                        .SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                            A: 255,
                            R: 0,
                            G: 0,
                            B: 0,
                        })
                        .map_err(setup)?;
                }
                // Physical pixels, scaled by the monitor's DPI: a child of WorkerW gets no WM_DPICHANGED, so WebView2
                // must not wait for one (Pitfall 5).
                let third: ICoreWebView2Controller3 = controller.cast().map_err(setup)?;
                third
                    .SetBoundsMode(COREWEBVIEW2_BOUNDS_MODE_USE_RAW_PIXELS)
                    .map_err(setup)?;
                third.SetShouldDetectMonitorScaleChanges(false).map_err(setup)?;
                let scale = monitor_scale(rect);
                third.SetRasterizationScale(scale).map_err(setup)?;
                self.scale.set(scale);
                self.controller3 = Some(third);
                let (width, height) = bounds_size(rect);
                controller
                    .SetBounds(RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: height,
                    })
                    .map_err(setup)?;
                controller.SetIsVisible(true).map_err(setup)?;

                // The page gets no host object and no message channel (T-04-51). The developer tools exist but never
                // open by themselves: the context menu shows only in developer mode and the key table below swallows the
                // shortcuts unless developer mode is on; the Portal's button opens them (D-13).
                let view = view_settings(start.developer_mode, start.allow_invalid_certificates);
                let settings = webview.Settings().map_err(setup)?;
                settings.SetIsWebMessageEnabled(false).map_err(setup)?;
                settings.SetAreHostObjectsAllowed(false).map_err(setup)?;
                settings.SetIsStatusBarEnabled(false).map_err(setup)?;
                settings.SetAreDevToolsEnabled(view.devtools_enabled).map_err(setup)?;
                settings
                    .SetAreDefaultContextMenusEnabled(view.context_menus)
                    .map_err(setup)?;
                if let Ok(third) = settings.cast::<ICoreWebView2Settings3>() {
                    third
                        .SetAreBrowserAcceleratorKeysEnabled(view.accelerator_keys)
                        .map_err(setup)?;
                }
                // The legacy hosts set CoreWebView2.Settings.UserAgent in every page type's initialization handler; this
                // host is shared by all of them, so one call before the first navigation covers them. A failure leaves
                // WebView2's own user agent: the wallpaper still plays.
                if let Some(agent) = user_agent_override(&start.user_agent) {
                    let text = wide(agent);
                    let applied = settings
                        .cast::<ICoreWebView2Settings2>()
                        .and_then(|second| second.SetUserAgent(PCWSTR(text.as_ptr())));
                    match applied {
                        Ok(()) => tracing::info!(user_agent = ?agent, "user agent"),
                        Err(error) => {
                            tracing::warn!(%error, "cannot set the user agent, the page gets WebView2's own");
                        }
                    }
                }

                // The legacy blocked keys and the DevTools shortcuts go through the table CEF uses (D-13).
                let keys_webview = webview.clone();
                let developer_mode = start.developer_mode;
                let handler = AcceleratorKeyPressedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    let mut code = 0_u32;
                    args.VirtualKey(&mut code)?;
                    let mut kind = COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN;
                    args.KeyEventKind(&mut kind)?;
                    // The high bit of GetKeyState is "down". The keys are never logged (T-04-45).
                    let ctrl = GetKeyState(i32::from(VK_CONTROL.0)) < 0;
                    let shift = GetKeyState(i32::from(VK_SHIFT.0)) < 0;
                    match key_action(developer_mode, i32::try_from(code).unwrap_or(0), ctrl, shift) {
                        KeyAction::Pass => {}
                        KeyAction::Swallow => args.SetHandled(true)?,
                        KeyAction::DevTools => {
                            // The user pressed the shortcut: that is asking for the tools (D-13).
                            if kind == COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                                || kind == COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
                            {
                                let _ = keys_webview.OpenDevToolsWindow();
                            }
                            args.SetHandled(true)?;
                        }
                    }
                    Ok(())
                }));
                let mut token = 0;
                controller
                    .add_AcceleratorKeyPressed(&handler, &mut token)
                    .map_err(setup)?;
                self.tokens.accelerator = Some(token);

                // Chromium refuses an invalid certificate unless the user allowed them (D-12): nothing is registered
                // without the setting. With it the engine also passes ignore-certificate-errors, so this legacy handler
                // only sees an error the switch did not cover.
                if view.allow_invalid_certificates {
                    let fourteenth: ICoreWebView2_14 = webview.cast().map_err(setup)?;
                    let handler = ServerCertificateErrorDetectedEventHandler::create(Box::new(|_sender, args| {
                        if let Some(args) = args {
                            let mut uri = PWSTR::null();
                            // The string is freed by `take_pwstr`; a failed call leaves it null, which reads empty.
                            let _ = args.RequestUri(&mut uri);
                            let uri = take_pwstr(uri);
                            args.SetAction(COREWEBVIEW2_SERVER_CERTIFICATE_ERROR_ACTION_ALWAYS_ALLOW)?;
                            tracing::info!(host = %uri_host(&uri), "certificate error allowed");
                        }
                        Ok(())
                    }));
                    let mut token = 0;
                    fourteenth
                        .add_ServerCertificateErrorDetected(&handler, &mut token)
                        .map_err(setup)?;
                    self.tokens.certificate = Some(token);
                }

                // Every request of the virtual origin goes through the shared planner.
                let filter = wide(&format!("{}*", origin::ORIGIN));
                webview
                    .AddWebResourceRequestedFilter(PCWSTR(filter.as_ptr()), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)
                    .map_err(setup)?;
                // The YouTube hosts are only looked at, to fill a missing Referer; the network answers them.
                for filter in request_filters() {
                    let filter = wide(&filter);
                    webview
                        .AddWebResourceRequestedFilter(PCWSTR(filter.as_ptr()), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)
                        .map_err(setup)?;
                }
                let roots = start.roots.clone();
                let served = environment.clone();
                let handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
                    match request_uri(&args) {
                        Some(uri) if !is_origin_uri(&uri) => {
                            if let Err(error) = fill_referer(&args, &uri) {
                                tracing::warn!(%error, host = %uri_host(&uri), "cannot fill the referer");
                            }
                            return Ok(());
                        }
                        _ => {}
                    }
                    if let Err(error) = answer(&served, &roots, &args) {
                        tracing::warn!(%error, "origin request failed, answering 404");
                        return respond(&served, &args, &serve::Reply::not_found(), None);
                    }
                    Ok(())
                }));
                let mut token = 0;
                webview.add_WebResourceRequested(&handler, &mut token).map_err(setup)?;
                self.tokens.resource = Some(token);

                let on_loaded = Arc::clone(&start.on_loaded);
                let page_calls = Arc::clone(&start.page_calls);
                let target = Rc::clone(&self.target);
                let surface = self.parent;
                let handler = NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                    let url = sender.as_ref().map_or_else(String::new, |webview| {
                        let mut source = PWSTR::null();
                        // The string is freed by `take_pwstr`; a failed call leaves it null, which reads empty.
                        let _ = webview.Source(&mut source);
                        take_pwstr(source)
                    });
                    let status = args.map_or(0, |args| completed_status(&args));
                    tracing::info!(url = ?url, status, "page loaded");
                    // The Chromium widget exists once the first page is up: find it now, not on the first input.
                    if target.get() == 0 {
                        match find_widget(surface) {
                            Some((widget, class)) => {
                                tracing::debug!(class, "input target found");
                                target.set(widget);
                            }
                            None => tracing::debug!("the page has no Chromium widget window yet"),
                        }
                    }
                    // The page calls run in the order the planner gave them: ExecuteScript keeps the order it is
                    // called in on this thread.
                    if let Some(webview) = &sender {
                        for call in page_calls(&url, status) {
                            tracing::debug!(kind = call.kind, key = ?call.key, "page call");
                            run_script(webview, &call.script, call.kind);
                        }
                    }
                    on_loaded(&url, status);
                    Ok(())
                }));
                let mut token = 0;
                webview.add_NavigationCompleted(&handler, &mut token).map_err(setup)?;
                self.tokens.completed = Some(token);

                // The legacy host cancels every download.
                let fourth: ICoreWebView2_4 = webview.cast().map_err(setup)?;
                let handler = DownloadStartingEventHandler::create(Box::new(|_sender, args| {
                    if let Some(args) = args {
                        // The event arguments belong to this callback.
                        args.SetCancel(true)?;
                        args.SetHandled(true)?;
                    }
                    Ok(())
                }));
                let mut token = 0;
                fourth.add_DownloadStarting(&handler, &mut token).map_err(setup)?;
                self.tokens.download = Some(token);

                // The time base shim goes in before the first navigation, so it runs before the page's own scripts. A shim
                // that cannot be registered leaves the page unsynced, never unshown (best effort).
                if let Some(script) = &start.document_start_script
                    && let Err(error) = add_document_script(&webview, script)
                {
                    tracing::warn!(%error, "the time base script could not be registered, the page is not synced");
                }

                let target = wide(url);
                webview.Navigate(PCWSTR(target.as_ptr())).map_err(setup)?;
            }
            Ok(())
        }

        /// Moves the controller onto the surface rectangle (left, top, right, bottom; physical pixels), to 0,0 of its
        /// parent and the size of the rectangle, and follows the DPI of the monitor it is on now.
        pub fn set_rect(&self, rect: [i32; 4]) -> Result<(), HostError> {
            let Some(controller) = &self.controller else {
                return Err(HostError::Setup("the controller is closed".to_owned()));
            };
            let (width, height) = bounds_size(rect);
            let scale = monitor_scale(rect);
            // SAFETY: the controller belongs to this thread.
            unsafe {
                if let Some(third) = &self.controller3
                    && (scale - self.scale.get()).abs() > f64::EPSILON
                {
                    third
                        .SetRasterizationScale(scale)
                        .map_err(|error| HostError::Setup(error.to_string()))?;
                    self.scale.set(scale);
                }
                controller
                    .SetBounds(RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: height,
                    })
                    .map_err(|error| HostError::Setup(error.to_string()))
            }
        }

        /// Posts one legacy input message to the Chromium widget of this page; `false` when there is no such window (yet)
        /// or the post failed, after which the widget is looked up again next time.
        pub fn post_to_widget(&self, message: u32, wparam: usize, lparam: isize) -> bool {
            if self.target.get() == 0 {
                match find_widget(self.parent) {
                    Some((widget, class)) => {
                        tracing::debug!(class, "input target found");
                        self.target.set(widget);
                    }
                    None => return false,
                }
            }
            // SAFETY: plain integers and a window handle of this page; PostMessageW validates the handle and fails for a
            // window that is gone, which is handled below.
            let posted = unsafe {
                PostMessageW(
                    Some(HWND(self.target.get() as *mut std::ffi::c_void)),
                    message,
                    WPARAM(wparam),
                    LPARAM(lparam),
                )
            };
            if posted.is_err() {
                self.target.set(0);
            }
            posted.is_ok()
        }

        /// Turns the mouse wheel by `delta` (120 per notch) at `(x, y)` of the page through the DevTools protocol, as the
        /// legacy WebView engine does. The position is relative to the surface, in physical pixels.
        pub fn send_mouse_wheel(&self, x: i32, y: i32, delta: i32) {
            let Some(webview) = &self.webview else {
                return;
            };
            let params = wide(&wheel_params(delta, x, y, self.scale.get()));
            let method = wide("Input.dispatchMouseEvent");
            let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|code, _result| {
                if let Err(error) = code {
                    tracing::debug!(%error, "the wheel call failed");
                }
                Ok(())
            }));
            // SAFETY: both strings are NUL-terminated and outlive the call, which copies them; the handler is a COM
            // object the call keeps its own reference to; the webview belongs to this thread.
            if let Err(error) = unsafe {
                webview.CallDevToolsProtocolMethod(PCWSTR(method.as_ptr()), PCWSTR(params.as_ptr()), &handler)
            } {
                tracing::debug!(%error, "the wheel call could not be started");
            }
        }

        /// Shows the developer tools in a window of their own: the Portal's Open DevTools button (D-13). Nothing else
        /// calls it but the developer-mode shortcuts.
        pub fn open_devtools(&self) -> Result<(), HostError> {
            let Some(webview) = &self.webview else {
                return Err(HostError::Setup("the page is closed".to_owned()));
            };
            // SAFETY: the webview belongs to this thread.
            unsafe { webview.OpenDevToolsWindow() }.map_err(|error| HostError::Setup(error.to_string()))
        }

        /// Runs a script in the main frame, as the page calls do (the in-page pause). The page gets no host object and no
        /// message channel; the script's result is not read.
        pub fn execute_script(&self, script: &str) {
            if let Some(webview) = &self.webview {
                run_script(webview, script, "script");
            }
        }

        /// Closes the controller, releases the webview and the controller, waits (bounded) for WebView2's browser process
        /// to exit, then releases the environment: the profile folder is free for the next renderer only after that
        /// (04-37 D10). Call it before the surface is dropped. Nothing is posted to any window: the controller is a child of the desktop
        /// surface, and a close message to its top-level ancestor would reach the shell (the Round A CEF lesson).
        pub fn close(mut self) {
            self.close_inner();
        }

        fn close_inner(&mut self) {
            let opened = self.controller.is_some();
            if let Some(webview) = &self.webview {
                // SAFETY: the tokens came from the registrations of this webview; a failure only means it is gone.
                unsafe {
                    if let Some(token) = self.tokens.resource.take() {
                        let _ = webview.remove_WebResourceRequested(token);
                    }
                    if let Some(token) = self.tokens.completed.take() {
                        let _ = webview.remove_NavigationCompleted(token);
                    }
                    if let (Some(token), Ok(fourth)) = (self.tokens.download.take(), webview.cast::<ICoreWebView2_4>())
                    {
                        let _ = fourth.remove_DownloadStarting(token);
                    }
                    if let (Some(token), Ok(fourteenth)) =
                        (self.tokens.certificate.take(), webview.cast::<ICoreWebView2_14>())
                    {
                        let _ = fourteenth.remove_ServerCertificateErrorDetected(token);
                    }
                }
            }
            if let (Some(controller), Some(token)) = (&self.controller, self.tokens.accelerator.take()) {
                // SAFETY: the token came from this controller's registration; a failure only means it is gone.
                let _ = unsafe { controller.remove_AcceleratorKeyPressed(token) };
            }
            // Registered before the close, so an exit that follows at once is not missed.
            let watch = if opened {
                self.environment.as_ref().and_then(watch_browser_exit)
            } else {
                None
            };
            if let Some(controller) = &self.controller {
                // SAFETY: the controller belongs to this thread; closing twice is harmless, the second answer is an
                // error that is logged.
                if let Err(error) = unsafe { controller.Close() } {
                    tracing::warn!(%error, "closing the WebView2 controller failed");
                }
            }
            self.webview = None;
            self.controller3 = None;
            self.controller = None;
            if let Some(watch) = watch {
                await_browser_exit(watch);
            }
            self.environment = None;
            if opened {
                tracing::info!("webview2 closed");
            }
            if self.com {
                self.com = false;
                // SAFETY: balances the successful CoInitializeEx of `start`, on the same thread.
                unsafe { CoUninitialize() };
            }
        }
    }

    impl Drop for Host {
        fn drop(&mut self) {
            self.close_inner();
        }
    }

    /// Registers `script` to run in every document the webview creates from now on, before the page's own scripts, and waits
    /// (pumping messages) until WebView2 has taken it, so the navigation that follows is certain to run it. The page gets
    /// no way back: the script has no host object and no message channel.
    fn add_document_script(webview: &ICoreWebView2, script: &str) -> Result<(), HostError> {
        let source = wide(script);
        let (sender, receiver) = mpsc::channel();
        let handler = AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(move |code, _id| {
            let _ = sender.send(code);
            Ok(())
        }));
        // SAFETY: `source` is NUL-terminated and outlives the call, which copies it; the handler is a COM object the
        // call keeps its own reference to; the webview belongs to this thread.
        unsafe { webview.AddScriptToExecuteOnDocumentCreated(PCWSTR(source.as_ptr()), &handler) }
            .map_err(|error| HostError::Setup(error.to_string()))?;
        wait_for(&receiver, "registering the time base script")?.map_err(|error| HostError::Setup(error.to_string()))
    }

    /// Starts `script` in the main frame of `webview`. WebView2 runs scripts in the order they were started on this
    /// thread; a failure to start or to run one is logged at debug with what the script was for, never its text.
    fn run_script(webview: &ICoreWebView2, script: &str, what: &'static str) {
        let source = wide(script);
        let handler = ExecuteScriptCompletedHandler::create(Box::new(move |code, _result| {
            if let Err(error) = code {
                tracing::debug!(%error, what, "a page script failed");
            }
            Ok(())
        }));
        // SAFETY: `source` is NUL-terminated and outlives the call, which copies it; the handler is a COM object the
        // call keeps its own reference to; the webview belongs to this thread.
        if let Err(error) = unsafe { webview.ExecuteScript(PCWSTR(source.as_ptr()), &handler) } {
            tracing::warn!(%error, what, "a page script could not be started");
        }
    }

    /// The HTTP status of a completed navigation: 0 when it failed or when WebView2 does not say.
    fn completed_status(args: &ICoreWebView2NavigationCompletedEventArgs) -> i32 {
        let mut success = BOOL(0);
        // SAFETY: the out pointer is valid for the call; the arguments belong to the event being handled.
        if unsafe { args.IsSuccess(&mut success) }.is_err() || !success.as_bool() {
            return 0;
        }
        let Ok(second) = args.cast::<ICoreWebView2NavigationCompletedEventArgs2>() else {
            return 0;
        };
        let mut status = 0;
        // SAFETY: as above.
        match unsafe { second.HttpStatusCode(&mut status) } {
            Ok(()) => status,
            Err(_) => 0,
        }
    }
}

#[cfg(windows)]
pub use imp::Host;

/// Systems without WebView2: nothing starts.
#[cfg(not(windows))]
mod stub {
    use super::{HostError, HostStart};

    /// Never created: [`Host::start`] refuses.
    pub struct Host(());

    impl Host {
        /// WebView2 exists only on Windows.
        pub fn start(_parent: u64, _rect: [i32; 4], _url: &str, _start: &HostStart) -> Result<Self, HostError> {
            Err(HostError::Unsupported)
        }

        /// Nothing to move.
        pub fn set_rect(&self, _rect: [i32; 4]) -> Result<(), HostError> {
            Ok(())
        }

        /// No page to post to.
        pub fn post_to_widget(&self, _message: u32, _wparam: usize, _lparam: isize) -> bool {
            false
        }

        /// No page to turn.
        pub fn send_mouse_wheel(&self, _x: i32, _y: i32, _delta: i32) {}

        /// Nothing to show the tools of.
        pub fn open_devtools(&self) -> Result<(), HostError> {
            Err(HostError::Unsupported)
        }

        /// Nothing to run a script in.
        pub fn execute_script(&self, _script: &str) {}

        /// Nothing to close.
        pub fn close(self) {}
    }
}

#[cfg(not(windows))]
pub use stub::Host;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_user_agent_keeps_webview2s_own_and_a_value_is_set_as_given() {
        assert_eq!(user_agent_override(""), None);
        assert_eq!(
            user_agent_override("SucroseProbe/1.0 (uov check)"),
            Some("SucroseProbe/1.0 (uov check)")
        );
        assert_eq!(user_agent_override(" padded "), Some(" padded "), "never trimmed");
    }

    #[test]
    fn the_bounds_are_the_rectangle_size_and_never_below_one_pixel() {
        assert_eq!(bounds_size([0, 0, 1920, 1080]), (1920, 1080));
        assert_eq!(bounds_size([-1920, 0, 0, 1080]), (1920, 1080));
        assert_eq!(bounds_size([5, 5, 5, 3]), (1, 1));
        assert_eq!(bounds_size([i32::MIN, 0, i32::MAX, 0]), (i32::MAX, 1));
    }

    #[test]
    fn the_scale_is_the_dpi_over_96() {
        assert!((scale_for_dpi(96) - 1.0).abs() < f64::EPSILON);
        assert!((scale_for_dpi(144) - 1.5).abs() < f64::EPSILON);
        assert!((scale_for_dpi(0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_header_block_leads_with_the_type_and_carries_every_planned_header() {
        let reply = Reply {
            status: 200,
            status_text: "OK",
            mime: "text/html",
            headers: vec![("Accept-Ranges", "bytes".to_owned())],
            content: sucrose_engine_web::serve::Content::Text(""),
        };

        assert_eq!(
            header_block(&reply),
            "Content-Type: text/html\r\nAccept-Ranges: bytes\r\n"
        );
    }

    #[test]
    fn a_ranged_reply_carries_the_content_range_of_the_cut_and_nothing_else_changes() {
        let reply = Reply {
            status: 206,
            status_text: "Partial Content",
            mime: "video/mp4",
            headers: vec![
                ("Accept-Ranges", "bytes".to_owned()),
                ("Content-Range", "bytes 0-10485759/10485760".to_owned()),
            ],
            content: sucrose_engine_web::serve::Content::Text(""),
        };

        let ranged = ranged_reply(&reply, "bytes 0-4194303/10485760");

        assert_eq!(ranged.status, 206);
        assert_eq!(
            ranged
                .headers
                .iter()
                .filter(|(name, _)| *name == "Content-Range")
                .count(),
            1
        );
        assert!(
            ranged
                .headers
                .contains(&("Content-Range", "bytes 0-4194303/10485760".to_owned()))
        );
        assert!(ranged.headers.contains(&("Accept-Ranges", "bytes".to_owned())));
    }

    #[test]
    fn the_developer_tools_exist_in_both_modes_but_the_menu_and_the_certificate_handler_follow_the_settings() {
        let closed = view_settings(false, false);
        let developer = view_settings(true, false);

        assert!(
            closed.devtools_enabled && developer.devtools_enabled,
            "Open DevTools needs them"
        );
        assert!(!closed.context_menus, "no context menu without developer mode");
        assert!(developer.context_menus, "Inspect in developer mode");
        assert!(closed.accelerator_keys && developer.accelerator_keys);
        assert!(!closed.allow_invalid_certificates, "D-12: validation is the default");
        assert!(
            !developer.allow_invalid_certificates,
            "developer mode does not allow bad certificates"
        );
        assert!(view_settings(false, true).allow_invalid_certificates);
    }

    #[test]
    fn the_wheel_delta_is_divided_by_the_monitor_scale_and_the_vertical_amount_is_negated() {
        assert_eq!(
            wheel_params(120, 300, 200, 1.5),
            r#"{"type":"mouseWheel","x":200,"y":133.333,"deltaX":0,"deltaY":-80,"modifiers":0,"pointerType":"mouse"}"#
        );
        assert_eq!(
            wheel_params(-120, 0, 0, 1.0),
            r#"{"type":"mouseWheel","x":0,"y":0,"deltaX":0,"deltaY":120,"modifiers":0,"pointerType":"mouse"}"#
        );
        assert!(
            wheel_params(240, 10, 10, 0.0).contains(r#""deltaY":-240"#),
            "an unknown scale is 1"
        );
    }

    #[test]
    fn the_log_host_of_a_uri_has_no_path_query_port_or_user() {
        assert_eq!(uri_host("https://expired.badssl.com/"), "expired.badssl.com");
        assert_eq!(
            uri_host("https://user:pw@example.com:8443/a/b?token=1#x"),
            "example.com"
        );
        assert_eq!(uri_host("https://[::1]:9000/"), "::1");
        assert_eq!(uri_host("example.org/path"), "example.org");
        assert_eq!(uri_host(""), "");
    }

    /// A window tree as `(class, children)` by window number; window 1 is the surface.
    fn chosen(tree: &[(u32, &str, &[u32])]) -> Option<u32> {
        let node = |window: u32| tree.iter().find(|(id, _, _)| *id == window).copied();
        input_widget(
            1,
            |window| node(window).map_or_else(Vec::new, |(_, _, children)| children.to_vec()),
            |window| node(window).map_or_else(String::new, |(_, class, _)| (*class).to_owned()),
        )
    }

    #[test]
    fn input_goes_to_the_browser_widget_inside_webview2s_host_window() {
        // The tree a WebView2 renderer has (04-37 debug run): the host window lives in this process, the widget inside it
        // in the browser process, and only that one hands posted input to the page.
        let tree: &[(u32, &str, &[u32])] = &[
            (1, "SucroseDesktopSurface", &[2]),
            (2, "Chrome_WidgetWin_0", &[3]),
            (3, "Chrome_WidgetWin_1", &[4, 5]),
            (4, "Chrome_RenderWidgetHostHWND", &[]),
            (5, "Intermediate D3D Window", &[]),
        ];

        assert_eq!(chosen(tree), Some(3), "never the host window itself");
    }

    #[test]
    fn a_host_window_without_its_browser_widget_yet_is_no_target_and_the_widget_alone_is_one() {
        let waiting: &[(u32, &str, &[u32])] = &[
            (1, "SucroseDesktopSurface", &[2, 3]),
            (2, "Chrome_WidgetWin_0", &[]),
            (3, "Chrome_WidgetWin_1", &[]),
        ];
        let widget_only: &[(u32, &str, &[u32])] = &[
            (1, "SucroseDesktopSurface", &[6]),
            (6, "Other", &[3]),
            (3, "Chrome_WidgetWin_1", &[]),
        ];
        let nothing: &[(u32, &str, &[u32])] = &[(1, "SucroseDesktopSurface", &[])];

        assert_eq!(chosen(waiting), None, "looked up again at the next event");
        assert_eq!(chosen(widget_only), Some(3));
        assert_eq!(chosen(nothing), None);
    }

    #[test]
    fn a_verbatim_drive_path_loses_its_prefix_and_a_unc_path_keeps_it() {
        assert_eq!(
            stream_path(Path::new(r"\\?\C:\themes\a\index.html")),
            Path::new(r"C:\themes\a\index.html")
        );
        assert_eq!(
            stream_path(Path::new(r"\\?\UNC\server\share\a.html")),
            Path::new(r"\\?\UNC\server\share\a.html")
        );
        assert_eq!(stream_path(Path::new("rel/a.html")), Path::new("rel/a.html"));
    }
}
