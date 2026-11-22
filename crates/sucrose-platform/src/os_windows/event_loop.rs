//! The message loop of the thread that owns the desktop surface.

use std::marker::PhantomData;

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, TranslateMessage, WM_QUIT,
        WM_USER,
    },
};

use crate::PlatformError;

/// The thread's `GetMessageW` loop. A child window of another process (the desktop surface, an embedded player)
/// only works while the thread that created it pumps messages, so this must run on that thread.
#[derive(Debug)]
pub struct EventLoop {
    thread_id: u32,
    /// The loop belongs to the thread that created it.
    _thread_bound: PhantomData<*const ()>,
}

impl EventLoop {
    /// Prepares the calling thread's message queue.
    pub fn new() -> Result<Self, PlatformError> {
        let mut message = MSG::default();
        // SAFETY: `message` is a valid out pointer. Peeking a message that never exists is the documented way to
        // make the OS create this thread's queue, so a later PostThreadMessageW from another thread cannot fail.
        let _ = unsafe { PeekMessageW(&mut message, None, WM_USER, WM_USER, PM_NOREMOVE) };
        Ok(Self {
            // SAFETY: no arguments; returns the id of the calling thread.
            thread_id: unsafe { GetCurrentThreadId() },
            _thread_bound: PhantomData,
        })
    }

    /// A handle that stops this loop from any thread.
    pub fn proxy(&self) -> LoopProxy {
        LoopProxy {
            thread_id: self.thread_id,
        }
    }

    /// Dispatches messages until [`LoopProxy::quit`] is called.
    pub fn run(self) {
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
        }
    }
}

/// Stops an [`EventLoop`] from any thread.
#[derive(Debug, Clone)]
pub struct LoopProxy {
    thread_id: u32,
}

impl LoopProxy {
    /// Asks the loop to return from `run`.
    pub fn quit(&self) {
        // SAFETY: plain integer arguments; the target thread's queue exists since EventLoop::new.
        if let Err(error) = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) } {
            tracing::warn!(%error, "cannot post the quit message to the loop thread");
        }
    }
}
