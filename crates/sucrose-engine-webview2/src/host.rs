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

use sucrose_engine_web::serve::Reply;

/// Called on the surface thread after every completed main-frame navigation, with its URL and HTTP status (0 when the
/// navigation failed).
pub type LoadedHook = Arc<dyn Fn(&str, i32) + Send + Sync>;

/// What the host needs to start, besides the surface and the page.
pub struct HostStart {
    /// The folders the virtual origin serves from, in order: the theme folder, then the generated content (D-14).
    pub roots: Vec<PathBuf>,
    /// The WebView2 user data folder: the leased profile (D-33, D-35).
    pub user_data_folder: PathBuf,
    /// `Engine.DeveloperMode`: the page gets its context menu and the developer tools (D-13).
    pub developer_mode: bool,
    /// Called after every completed main-frame navigation.
    pub on_loaded: LoadedHook,
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

#[cfg(windows)]
mod imp {
    use std::{
        cell::Cell,
        path::{Path, PathBuf},
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };

    use sucrose_engine_web::{
        origin,
        serve::{self, Content},
    };
    use webview2_com::{
        CoreWebView2EnvironmentOptions, CreateCoreWebView2ControllerCompletedHandler,
        CreateCoreWebView2EnvironmentCompletedHandler, DownloadStartingEventHandler,
        Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_BOUNDS_MODE_USE_RAW_PIXELS, COREWEBVIEW2_COLOR, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2, ICoreWebView2_4, ICoreWebView2Controller,
            ICoreWebView2Controller2, ICoreWebView2Controller3, ICoreWebView2Environment,
            ICoreWebView2EnvironmentOptions, ICoreWebView2NavigationCompletedEventArgs,
            ICoreWebView2NavigationCompletedEventArgs2, ICoreWebView2WebResourceRequestedEventArgs,
        },
        NavigationCompletedEventHandler, WebResourceRequestedEventHandler, take_pwstr,
    };
    use windows::{
        Win32::{
            Foundation::{E_POINTER, HWND, RECT, RPC_E_CHANGED_MODE},
            Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromRect},
            System::Com::{
                COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize, IStream, STGM_READ, STGM_SHARE_DENY_NONE,
            },
            UI::{
                HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
                Shell::SHCreateStreamOnFileEx,
                WindowsAndMessaging::{
                    DispatchMessageW, MSG, MsgWaitForMultipleObjects, PM_REMOVE, PeekMessageW, PostQuitMessage,
                    QS_ALLINPUT, TranslateMessage, WM_QUIT,
                },
            },
        },
        core::{BOOL, Interface, PCWSTR, PWSTR},
    };

    use super::{HostError, HostStart, bounds_size, header_block, scale_for_dpi, stream_path};

    /// How long the environment and the controller get to come up.
    const CREATE_TIMEOUT: Duration = Duration::from_secs(30);

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
        let deadline = Instant::now() + CREATE_TIMEOUT;
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

    /// One attempt at the environment on `folder`.
    fn try_environment(folder: &Path) -> Result<ICoreWebView2Environment, HostError> {
        let (sender, receiver) = mpsc::channel();
        let options = CoreWebView2EnvironmentOptions::default();
        // SAFETY: the options object is not shared with COM yet, so nothing else reads these fields. Language stays
        // empty (the OS default), extensions and custom crash reporting stay off, and Sucrose's switches arrive with
        // plan 04-25.
        unsafe {
            options.set_are_browser_extensions_enabled(false);
            options.set_is_custom_crash_reporting_enabled(false);
            options.set_additional_browser_arguments(String::new());
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
    fn create_environment(folder: &Path) -> Result<ICoreWebView2Environment, HostError> {
        match try_environment(folder) {
            Err(HostError::Environment(first)) => {
                tracing::warn!(error = %first, "the WebView2 environment failed, trying once more");
                std::thread::sleep(RETRY_DELAY);
                try_environment(folder)
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

    /// Answers one request of the virtual origin from `serve::plan`: a whole file as a stream, an empty body for every
    /// refusal. A Range request gets the whole file with 200 until plan 04-23 adds partial replies, which HTTP allows.
    fn answer(
        environment: &ICoreWebView2Environment,
        roots: &[PathBuf],
        args: &ICoreWebView2WebResourceRequestedEventArgs,
    ) -> windows::core::Result<()> {
        // SAFETY: the request and its URI belong to this event; the returned string is freed by `take_pwstr`.
        let uri = unsafe {
            let request = args.Request()?;
            let mut uri = PWSTR::null();
            request.Uri(&mut uri)?;
            take_pwstr(uri)
        };
        let roots: Vec<&Path> = roots.iter().map(PathBuf::as_path).collect();
        let mut reply = serve::plan(&uri, None, &roots);
        let stream = match &reply.content {
            Content::File { path, .. } => match open_stream(path) {
                Ok(stream) => Some(stream),
                Err(error) => {
                    tracing::info!(%error, "origin request rejected: the file cannot be opened");
                    reply = serve::Reply::not_found();
                    None
                }
            },
            Content::Text(_) => None,
        };
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

    /// The WebView2 pieces of one renderer; closing it (or dropping it) releases them in order.
    pub struct Host {
        environment: Option<ICoreWebView2Environment>,
        controller: Option<ICoreWebView2Controller>,
        controller3: Option<ICoreWebView2Controller3>,
        webview: Option<ICoreWebView2>,
        tokens: Tokens,
        scale: Cell<f64>,
        /// Whether this host set COM up on its thread and so owes the matching `CoUninitialize`.
        com: bool,
    }

    /// The event registrations to remove before the controller closes.
    #[derive(Default)]
    struct Tokens {
        resource: Option<i64>,
        completed: Option<i64>,
        download: Option<i64>,
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
                com: true,
            };
            host.open(parent, rect, url, start)?;
            Ok(host)
        }

        /// Creates the environment and the controller and shows the page. A failure leaves the pieces made so far in
        /// `self`, which the caller drops.
        fn open(&mut self, parent: u64, rect: [i32; 4], url: &str, start: &HostStart) -> Result<(), HostError> {
            let setup = |error: windows::core::Error| HostError::Setup(error.to_string());
            let environment = create_environment(&start.user_data_folder)?;
            self.environment = Some(environment.clone());
            let window = HWND(parent as usize as *mut std::ffi::c_void);
            let controller = create_controller(&environment, window)?;
            self.controller = Some(controller.clone());

            // SAFETY: the controller and the webview belong to this thread; every call below only passes plain
            // values, strings that outlive it, or COM objects whose references the callee takes.
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

                // The page gets no host object and no message channel (T-04-51); menus and developer tools only in
                // developer mode (D-13).
                let settings = webview.Settings().map_err(setup)?;
                settings.SetIsWebMessageEnabled(false).map_err(setup)?;
                settings.SetAreHostObjectsAllowed(false).map_err(setup)?;
                settings.SetIsStatusBarEnabled(false).map_err(setup)?;
                settings.SetAreDevToolsEnabled(start.developer_mode).map_err(setup)?;
                settings
                    .SetAreDefaultContextMenusEnabled(start.developer_mode)
                    .map_err(setup)?;

                // Every request of the virtual origin goes through the shared planner.
                let filter = wide(&format!("{}*", origin::ORIGIN));
                webview
                    .AddWebResourceRequestedFilter(PCWSTR(filter.as_ptr()), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)
                    .map_err(setup)?;
                let roots = start.roots.clone();
                let served = environment.clone();
                let handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args) = args else {
                        return Ok(());
                    };
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
                let handler = NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
                    let url = sender.map_or_else(String::new, |webview| {
                        let mut source = PWSTR::null();
                        // The string is freed by `take_pwstr`; a failed call leaves it null, which reads empty.
                        let _ = webview.Source(&mut source);
                        take_pwstr(source)
                    });
                    let status = args.map_or(0, |args| completed_status(&args));
                    tracing::info!(url = ?url, status, "page loaded");
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

        /// Closes the controller, then releases the webview, the controller and the environment in that order. Call it
        /// before the surface is dropped. Nothing is posted to any window: the controller is a child of the desktop
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
                }
            }
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
