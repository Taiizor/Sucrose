//! The message loop of the thread that owns the desktop surface, and of the service's main thread.
//!
//! The loop owns a hidden top-level window (class `SucroseServiceWindow`). It is top-level rather than message-only
//! because `TaskbarCreated` and `WM_DISPLAYCHANGE` are only broadcast to top-level windows (later plans listen to them).
//!
//! `WM_DISPLAYCHANGE`, `WM_DPICHANGED`, `WM_SETTINGCHANGE` and the broadcast `TaskbarCreated` are *sent* messages:
//! `GetMessageW` runs the window procedure for them and keeps waiting, so without another message nothing would hand
//! the queued event to the handler. The window procedure therefore posts a private `WM_DRAIN` after queuing one, which
//! makes `GetMessageW` return.

use std::{
    cell::{Cell, RefCell},
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
/// Posted by the window procedure itself after it queued an event from a sent message, so `GetMessageW` returns and
/// `run` drains `PENDING`. It carries no event.
const WM_DRAIN: u32 = WM_APP + 2;

const CLASS_NAME: PCWSTR = w!("SucroseServiceWindow");

/// The id Windows gave the registered `TaskbarCreated` message (0 until a loop exists). The shell broadcasts it to
/// every top-level window after explorer.exe started again.
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

#[cfg(test)]
thread_local! {
    /// How many drain posts this thread made, so a test can see that a burst posts one.
    static DRAIN_POSTS: Cell<u32> = const { Cell::new(0) };
}

thread_local! {
    /// Whether a `WM_DRAIN` is already posted and not yet dispatched on this thread, so a burst of sent messages
    /// posts one, not one per message (T-02-35).
    static DRAIN_POSTED: Cell<bool> = const { Cell::new(false) };
}

thread_local! {
    /// Events the window procedure received on this thread, waiting for `run` to hand them to the handler. A
    /// window procedure can also run inside another modal loop (an open tray menu), so it only queues. Display and
    /// shell messages are sent, not posted: `GetMessageW` does not return for them, which is why `WM_DRAIN` exists.
    static PENDING: RefCell<VecDeque<LoopEvent>> = const { RefCell::new(VecDeque::new()) };
}

/// Makes `GetMessageW` return so `run` hands the queued events to the handler, unless a drain is already on its way.
fn request_drain(hwnd: HWND) {
    if DRAIN_POSTED.get() {
        return;
    }
    DRAIN_POSTED.set(true);
    // SAFETY: plain handle and integers; PostMessageW validates the handle. It is called from the thread that owns
    // the window, from inside its window procedure, which is allowed.
    match unsafe { PostMessageW(Some(hwnd), WM_DRAIN, WPARAM(0), LPARAM(0)) } {
        Ok(()) => {
            #[cfg(test)]
            DRAIN_POSTS.with(|posts| posts.set(posts.get() + 1));
        }
        Err(error) => {
            // The next event tries again.
            DRAIN_POSTED.set(false);
            tracing::debug!(%error, "cannot post the drain message to the loop window");
        }
    }
}

unsafe extern "system" fn loop_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let taskbar_created = TASKBAR_CREATED.load(Ordering::Relaxed);
    // The second field says whether the message was sent (not posted) and so needs a drain post to be noticed.
    let event = match message {
        WM_WAKE => Some((LoopEvent::Wake, false)),
        WM_QUIT_REQUEST => Some((LoopEvent::Quit, false)),
        WM_DISPLAYCHANGE | WM_DPICHANGED => Some((LoopEvent::DisplayChanged, true)),
        // Only a change of the work area matters here; every other setting goes to the default procedure.
        WM_SETTINGCHANGE if wparam.0 == SPI_SETWORKAREA.0 as usize => Some((LoopEvent::DisplayChanged, true)),
        WM_TIMER => Some((LoopEvent::Timer(wparam.0 as u32), false)),
        _ if taskbar_created != 0 && message == taskbar_created => Some((LoopEvent::ShellRestarted, true)),
        _ => None,
    };
    if let Some((event, sent)) = event {
        PENDING.with(|pending| pending.borrow_mut().push_back(event));
        if sent {
            request_drain(hwnd);
        }
        return LRESULT(0);
    }
    if message == WM_DRAIN {
        // No event of its own: `GetMessageW` returning for it is the point. `run` drains right after dispatching it,
        // also when a modal loop (the tray menu) dispatched it, because the drain happens once that dispatch returns.
        DRAIN_POSTED.set(false);
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

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant},
    };

    use windows::Win32::UI::WindowsAndMessaging::{SMTO_NORMAL, SendMessageTimeoutW, SendMessageW, SendNotifyMessageW};

    use super::*;

    /// How long the sender waits so the loop is already inside GetMessageW with an empty queue.
    const SEND_AFTER: Duration = Duration::from_millis(100);
    /// A broken loop only ends through this quit, so a regression fails instead of hanging the run.
    const WATCHDOG: Duration = Duration::from_secs(3);
    /// A sent message must reach the handler well before the watchdog.
    const LIMIT: Duration = Duration::from_millis(500);

    /// Runs a loop on this thread, lets another thread call `send` with the loop window's handle after
    /// [`SEND_AFTER`], and returns when the send started and every event the handler saw (with its arrival time) up
    /// to the first one that satisfies `done`. Sent messages are never posted: a posted message wakes GetMessageW
    /// and hides the bug these tests guard against.
    fn run_after_send(
        done: impl Fn(&LoopEvent) -> bool,
        send: impl FnOnce(isize) + Send + 'static,
    ) -> (Instant, Vec<(Instant, LoopEvent)>) {
        let event_loop = EventLoop::new().expect("the loop window can be created");
        let proxy = event_loop.proxy();
        let hwnd = proxy.hwnd;
        let sent_at = Arc::new(Mutex::new(None));
        let sender_clock = Arc::clone(&sent_at);
        let sender = thread::spawn(move || {
            thread::sleep(SEND_AFTER);
            *sender_clock.lock().expect("the clock is not poisoned") = Some(Instant::now());
            send(hwnd);
        });
        let watchdog = thread::spawn(move || {
            thread::sleep(WATCHDOG);
            proxy.quit();
        });
        let mut seen = Vec::new();
        event_loop.run(|event| {
            seen.push((Instant::now(), event));
            if done(&event) || event == LoopEvent::Quit {
                Flow::Exit
            } else {
                Flow::Continue
            }
        });
        sender.join().expect("the sender thread ends");
        drop(watchdog);
        let started = sent_at
            .lock()
            .expect("the clock is not poisoned")
            .expect("the sender recorded its start");
        (started, seen)
    }

    fn assert_arrived_at_once(expected: LoopEvent, started: Instant, seen: &[(Instant, LoopEvent)]) {
        let (arrived, first) = seen.first().expect("the handler saw an event");
        assert_eq!(
            *first, expected,
            "the first event is the sent one, not the watchdog quit"
        );
        let delay = arrived.duration_since(started);
        assert!(delay < LIMIT, "the event took {delay:?} to reach the handler");
    }

    #[test]
    fn sent_display_change_reaches_the_handler_without_a_post() {
        let (started, seen) = run_after_send(
            |event| *event == LoopEvent::DisplayChanged,
            |hwnd| {
                // SAFETY: a plain handle and integers; the call validates the handle and does not wait for the loop.
                let _ =
                    unsafe { SendNotifyMessageW(HWND(hwnd as *mut c_void), WM_DISPLAYCHANGE, WPARAM(0), LPARAM(0)) };
            },
        );
        assert_arrived_at_once(LoopEvent::DisplayChanged, started, &seen);
    }

    #[test]
    fn sent_work_area_change_reaches_the_handler_without_a_post() {
        let (started, seen) = run_after_send(
            |event| *event == LoopEvent::DisplayChanged,
            |hwnd| {
                // SAFETY: a plain handle and integers; the 1 s timeout keeps a broken loop from blocking this thread.
                let _ = unsafe {
                    SendMessageTimeoutW(
                        HWND(hwnd as *mut c_void),
                        WM_SETTINGCHANGE,
                        WPARAM(SPI_SETWORKAREA.0 as usize),
                        LPARAM(0),
                        SMTO_NORMAL,
                        1000,
                        None,
                    )
                };
            },
        );
        assert_arrived_at_once(LoopEvent::DisplayChanged, started, &seen);
    }

    #[test]
    fn sent_taskbar_created_reaches_the_handler_without_a_post() {
        let (started, seen) = run_after_send(
            |event| *event == LoopEvent::ShellRestarted,
            |hwnd| {
                let message = TASKBAR_CREATED.load(Ordering::Relaxed);
                // SAFETY: a plain handle and integers; the call validates the handle and does not wait for the loop.
                let _ = unsafe { SendNotifyMessageW(HWND(hwnd as *mut c_void), message, WPARAM(0), LPARAM(0)) };
            },
        );
        assert_arrived_at_once(LoopEvent::ShellRestarted, started, &seen);
    }

    #[test]
    fn sent_burst_is_delivered_in_full_with_one_drain_post() {
        let event_loop = EventLoop::new().expect("the loop window can be created");
        let hwnd = event_loop.hwnd;
        DRAIN_POSTS.with(|posts| posts.set(0));
        for _ in 0..20 {
            // SAFETY: the window belongs to this thread, so the message goes straight to its procedure.
            unsafe { SendMessageW(hwnd, WM_DISPLAYCHANGE, Some(WPARAM(0)), Some(LPARAM(0))) };
        }
        assert_eq!(
            DRAIN_POSTS.with(Cell::get),
            1,
            "a burst posts one drain, not one per message"
        );
        let proxy = event_loop.proxy();
        let watchdog = thread::spawn(move || {
            thread::sleep(WATCHDOG);
            proxy.quit();
        });
        let mut changes = 0;
        event_loop.run(|event| {
            if event == LoopEvent::DisplayChanged {
                changes += 1;
            }
            if changes == 20 || event == LoopEvent::Quit {
                Flow::Exit
            } else {
                Flow::Continue
            }
        });
        drop(watchdog);
        assert_eq!(changes, 20, "every sent message becomes one event");
    }
}
