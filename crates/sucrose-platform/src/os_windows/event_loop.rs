//! The message loop of the thread that owns the desktop surface, and of the service's main thread.
//!
//! The loop owns a hidden top-level window (class `SucroseServiceWindow`). It is top-level rather than message-only
//! because `TaskbarCreated` and `WM_DISPLAYCHANGE` are only broadcast to top-level windows (later plans listen to them).

use std::{cell::RefCell, collections::VecDeque, ffi::c_void, marker::PhantomData, mem::size_of};

use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, MSG, PostMessageW,
            RegisterClassExW, TranslateMessage, WM_APP, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_POPUP,
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

thread_local! {
    /// Events the window procedure received on this thread, waiting for `run` to hand them to the handler. A
    /// window procedure can also run inside another modal loop (an open tray menu), so it only queues.
    static PENDING: RefCell<VecDeque<LoopEvent>> = const { RefCell::new(VecDeque::new()) };
}

unsafe extern "system" fn loop_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let event = match message {
        WM_WAKE => Some(LoopEvent::Wake),
        WM_QUIT_REQUEST => Some(LoopEvent::Quit),
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

impl EventLoop {
    /// Creates the loop's window on the calling thread, which also creates the thread's message queue.
    pub fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            hwnd: create_window()?,
            _thread_bound: PhantomData,
        })
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
