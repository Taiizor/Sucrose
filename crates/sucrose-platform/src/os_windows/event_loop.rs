//! The message loop of the thread that owns the desktop surface, and of the service's main thread.
//!
//! The loop owns a hidden top-level window (class `SucroseServiceWindow`). It is top-level rather than message-only
//! because `TaskbarCreated` and `WM_DISPLAYCHANGE` are only broadcast to top-level windows (later plans listen to them).

use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::c_void,
    marker::PhantomData,
    mem::size_of,
    sync::atomic::{AtomicU32, Ordering},
};

use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            ChangeWindowMessageFilterEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            KillTimer, MSG, MSGFLT_ALLOW, PostMessageW, RegisterClassExW, RegisterWindowMessageW, SPI_SETWORKAREA,
            SetTimer, TranslateMessage, WM_APP, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_SETTINGCHANGE, WM_TIMER,
            WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_POPUP,
        },
    },
    core::{PCWSTR, w},
};

use super::os_error;
use crate::{Flow, LoopEvent, PlatformError};

/// Posted by [`LoopProxy::wake`].
const WM_WAKE: u32 = WM_APP;
/// Posted by [`LoopProxy::quit`].
const WM_QUIT_REQUEST: u32 = WM_APP + 1;

const CLASS_NAME: PCWSTR = w!("SucroseServiceWindow");

/// The id Windows gave the registered `TaskbarCreated` message (0 until a loop exists). The shell broadcasts it to
/// every top-level window after explorer.exe started again.
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// Events the window procedure received on this thread, waiting for `run` to hand them to the handler. A
    /// window procedure can also run inside another modal loop (an open tray menu), so it only queues.
    static PENDING: RefCell<VecDeque<LoopEvent>> = const { RefCell::new(VecDeque::new()) };
}

unsafe extern "system" fn loop_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let taskbar_created = TASKBAR_CREATED.load(Ordering::Relaxed);
    let event = match message {
        WM_WAKE => Some(LoopEvent::Wake),
        WM_QUIT_REQUEST => Some(LoopEvent::Quit),
        WM_DISPLAYCHANGE | WM_DPICHANGED => Some(LoopEvent::DisplayChanged),
        // Only a change of the work area matters here; every other setting goes to the default procedure.
        WM_SETTINGCHANGE if wparam.0 == SPI_SETWORKAREA.0 as usize => Some(LoopEvent::DisplayChanged),
        WM_TIMER => Some(LoopEvent::Timer(wparam.0 as u32)),
        _ if taskbar_created != 0 && message == taskbar_created => Some(LoopEvent::ShellRestarted),
        _ => None,
    };
    if let Some(event) = event {
        PENDING.with(|pending| pending.borrow_mut().push_back(event));
        return LRESULT(0);
    }
    // SAFETY: forwards the arguments the OS gave this window procedure, unchanged.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

fn create_window() -> Result<HWND, PlatformError> {
    // SAFETY: a null module name returns the handle of this executable.
    let module = unsafe { GetModuleHandleW(None) }.map_err(|e| os_error("GetModuleHandleW", &e))?;
    let instance = HINSTANCE(module.0);
    let class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(loop_proc),
        hInstance: instance,
        lpszClassName: CLASS_NAME,
        ..Default::default()
    };
    // SAFETY: `class` is fully initialized and its strings and procedure live for the whole process.
    if unsafe { RegisterClassExW(&class) } == 0 {
        // SAFETY: reads the calling thread's last error.
        let code = unsafe { GetLastError() };
        if code != ERROR_CLASS_ALREADY_EXISTS {
            return Err(PlatformError::Os {
                context: "RegisterClassExW",
                code: code.0 as i32,
            });
        }
    }
    // SAFETY: the class is registered above and the strings are valid; no parent, menu or creation data is passed.
    // The window is never shown: it only receives messages.
    unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            CLASS_NAME,
            w!("Sucrose service"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .map_err(|e| os_error("CreateWindowExW", &e))
}

/// The thread's `GetMessageW` loop. A child window of another process (the desktop surface, an embedded player)
/// and a tray icon only work while the thread that created them pumps messages, so this must run on that thread.
#[derive(Debug)]
pub struct EventLoop {
    hwnd: HWND,
    /// The loop belongs to the thread that created it.
    _thread_bound: PhantomData<*const ()>,
}

/// Registers `TaskbarCreated` and lets it through the message filter of an elevated process, so the loop also hears
/// a shell restart when this process runs with higher rights than explorer.exe.
fn register_shell_messages(hwnd: HWND) {
    // SAFETY: the string is a valid NUL-terminated literal; the call only registers a message name.
    let message = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    if message == 0 {
        tracing::warn!("cannot register the TaskbarCreated message, a shell restart will not be noticed");
        return;
    }
    TASKBAR_CREATED.store(message, Ordering::Relaxed);
    // SAFETY: `hwnd` is the window this thread just created; no CHANGEFILTERSTRUCT is requested back.
    if let Err(error) = unsafe { ChangeWindowMessageFilterEx(hwnd, message, MSGFLT_ALLOW, None) } {
        tracing::debug!(%error, "cannot allow TaskbarCreated through the message filter");
    }
}

impl EventLoop {
    /// Creates the loop's window on the calling thread, which also creates the thread's message queue.
    pub fn new() -> Result<Self, PlatformError> {
        let hwnd = create_window()?;
        register_shell_messages(hwnd);
        Ok(Self {
            hwnd,
            _thread_bound: PhantomData,
        })
    }

    /// Starts (or restarts) the timer `id`: [`LoopEvent::Timer`] arrives every `ms` milliseconds until
    /// [`EventLoop::kill_timer`]. Use [`EventLoop::timers`] to change timers from inside the handler.
    pub fn set_timer(&self, id: u32, ms: u32) {
        self.timers().set(id, ms);
    }

    /// Stops the timer `id`; stopping a timer that is not running does nothing.
    pub fn kill_timer(&self, id: u32) {
        self.timers().kill(id);
    }

    /// A handle for the timers of this loop that the handler can use while [`EventLoop::run`] owns the loop.
    pub fn timers(&self) -> LoopTimers {
        LoopTimers { hwnd: self.hwnd }
    }

    /// A handle that wakes or stops this loop from any thread.
    pub fn proxy(&self) -> LoopProxy {
        LoopProxy {
            hwnd: self.hwnd.0 as isize,
        }
    }

    /// Dispatches messages and hands [`LoopEvent`]s to `handler` until it returns [`Flow::Exit`].
    pub fn run(self, mut handler: impl FnMut(LoopEvent) -> Flow) {
        let mut message = MSG::default();
        loop {
            // SAFETY: `message` is a valid out pointer; a null window handle takes the messages of every window of
            // this thread.
            let result = unsafe { GetMessageW(&mut message, None, 0, 0) };
            match result.0 {
                0 => return,
                -1 => {
                    tracing::error!("GetMessageW failed, the message loop ends");
                    return;
                }
                _ => {
                    // SAFETY: `message` was filled by GetMessageW just above.
                    unsafe {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
            // The queue is popped one event at a time so the handler may cause more events.
            while let Some(event) = PENDING.with(|pending| pending.borrow_mut().pop_front()) {
                if handler(event) == Flow::Exit {
                    return;
                }
            }
        }
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        // SAFETY: the window was created by this thread (the loop is not Send) and is destroyed exactly once, here.
        // A failure leaves nothing to act on at shutdown.
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

/// Starts and stops the timers of an [`EventLoop`]. It belongs to the loop's thread (a window timer can only be
/// changed by the thread that owns the window), which `HWND` not being `Send` already enforces.
#[derive(Debug, Clone, Copy)]
pub struct LoopTimers {
    hwnd: HWND,
}

impl LoopTimers {
    /// Starts (or restarts) the timer `id`; it keeps firing until [`LoopTimers::kill`].
    pub fn set(&self, id: u32, ms: u32) {
        // SAFETY: the window belongs to the calling thread (see the type); a null callback makes the timer post
        // WM_TIMER to the window. Setting an existing id replaces that timer.
        if unsafe { SetTimer(Some(self.hwnd), id as usize, ms, None) } == 0 {
            tracing::warn!(id, ms, "cannot start a loop timer");
        }
    }

    /// Stops the timer `id`; stopping one that is not running does nothing.
    pub fn kill(&self, id: u32) {
        // SAFETY: the window belongs to the calling thread; an unknown timer id only makes the call fail.
        if let Err(error) = unsafe { KillTimer(Some(self.hwnd), id as usize) } {
            tracing::trace!(id, %error, "no such loop timer to stop");
        }
    }
}

/// Wakes or stops an [`EventLoop`] from any thread.
#[derive(Debug, Clone)]
pub struct LoopProxy {
    /// The loop window's handle. A window handle is only an identifier the OS validates on every call, so it may be
    /// used from any thread; it is kept as an integer because `HWND` itself is not `Send`.
    hwnd: isize,
}

impl LoopProxy {
    /// Makes the loop call its handler with [`LoopEvent::Wake`].
    pub fn wake(&self) {
        self.post(WM_WAKE);
    }

    /// Makes the loop call its handler with [`LoopEvent::Quit`].
    pub fn quit(&self) {
        self.post(WM_QUIT_REQUEST);
    }

    fn post(&self, message: u32) {
        let hwnd = HWND(self.hwnd as *mut c_void);
        // SAFETY: plain handle and integers; PostMessageW validates the handle and may be called from any thread. It
        // fails (and is logged) once the loop's window is gone.
        if let Err(error) = unsafe { PostMessageW(Some(hwnd), message, WPARAM(0), LPARAM(0)) } {
            tracing::debug!(%error, "cannot post a message to the loop window");
        }
    }
}
