//! The surface a wallpaper is drawn on: a window parented into the desktop shell so it sits behind the icons.
//!
//! Two shell layouts exist. The classic one (Windows 7 to 11 23H2) turns into a top-level `WorkerW` window after the
//! `0x052C` message to Progman, and the surface becomes a child of that `WorkerW`. The raised one (Windows 11 24H2
//! and later) keeps `SHELLDLL_DefView` and a `WorkerW` as children of Progman, and a plain child of Progman is not
//! composited there. The layout is read from the window tree, never from the OS build number.

use std::{
    mem::size_of,
    thread,
    time::{Duration, Instant},
};

use windows::{
    Win32::{
        Foundation::{
            COLORREF, ERROR_CLASS_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
        },
        Graphics::Gdi::{BLACK_BRUSH, ClientToScreen, GetStockObject, HBRUSH},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, EnumWindows, FindWindowExW, FindWindowW, GA_ROOT, GW_CHILD,
            GW_HWNDNEXT, GWL_EXSTYLE, GWL_STYLE, GetAncestor, GetClassNameW, GetWindow, GetWindowLongPtrW,
            GetWindowRect, IsWindowVisible, LWA_ALPHA, MA_NOACTIVATE, RegisterClassExW, SMTO_NORMAL, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SendMessageTimeoutW, SetLayeredWindowAttributes,
            SetParent, SetWindowLongPtrW, SetWindowPos, WINDOW_LONG_PTR_INDEX, WM_MOUSEACTIVATE, WNDCLASSEXW, WS_CHILD,
            WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        },
    },
    core::{BOOL, PCWSTR, w},
};

use super::os_error;
use crate::{
    PlatformError, RectPx,
    layout::{self, DesktopLayout, ShellSnapshot, WindowNode},
};

const CLASS_NAME: PCWSTR = w!("SucroseDesktopSurface");
/// How long attaching keeps retrying: explorer can create its WorkerW late, or be restarting.
const ATTACH_TIMEOUT: Duration = Duration::from_secs(10);
/// The undocumented Progman message that makes the shell create the wallpaper `WorkerW`.
const SPAWN_WORKERW: u32 = 0x052C;

/// A window behind the desktop icons of one monitor. It lives on the thread that created it, which must pump
/// messages (see [`crate::EventLoop`]), so it is not `Send`.
#[derive(Debug)]
pub struct DesktopSurface {
    hwnd: HWND,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    /// A layered child of Progman, ordered right under `SHELLDLL_DefView`.
    LayeredChild,
    /// A child of the `WorkerW` that is a child of Progman.
    WorkerWChild,
    /// A child of the top-level `WorkerW` behind the icons.
    ClassicWorkerW,
}

/// The shell windows one attach attempt works with.
struct Shell {
    progman: Option<HWND>,
    /// `SHELLDLL_DefView` inside Progman.
    defview_in_progman: Option<HWND>,
    /// A `WorkerW` inside Progman.
    workerw_in_progman: Option<HWND>,
    /// Top-level windows other than Progman that hold a `SHELLDLL_DefView`.
    defview_holders: Vec<HWND>,
}

impl Shell {
    fn capture() -> Self {
        let progman = find_window(w!("Progman"));
        let (defview_in_progman, workerw_in_progman) = match progman {
            Some(progman) => (
                find_child(Some(progman), None, w!("SHELLDLL_DefView")),
                find_child(Some(progman), None, w!("WorkerW")),
            ),
            None => (None, None),
        };
        let mut holders: Vec<HWND> = Vec::new();
        // SAFETY: the callback only touches the Vec behind the pointer passed here, which outlives the synchronous call.
        let _ = unsafe { EnumWindows(Some(collect_defview_holders), LPARAM(&raw mut holders as isize)) };
        Self {
            progman,
            defview_in_progman,
            workerw_in_progman,
            defview_holders: holders,
        }
    }

    /// The window tree as plain data; the layout is decided from it by [`layout::classify`] alone.
    fn snapshot(&self) -> ShellSnapshot {
        ShellSnapshot {
            progman: self.progman.map(|progman| node_of(progman, 1)),
            top_level_with_defview: self.defview_holders.iter().map(|h| node_of(*h, 1)).collect(),
        }
    }

    /// The top-level `WorkerW` that follows the window holding the icons (classic layout).
    fn classic_workerw(&self) -> Option<HWND> {
        self.defview_holders
            .iter()
            .find_map(|holder| find_child(None, Some(*holder), w!("WorkerW")))
    }

    fn has_wallpaper_workerw(&self) -> bool {
        self.workerw_in_progman.is_some() || self.classic_workerw().is_some()
    }
}

unsafe extern "system" fn collect_defview_holders(hwnd: HWND, data: LPARAM) -> BOOL {
    // SAFETY: `data` is the address of the Vec that `Shell::capture` passed to EnumWindows and keeps alive for the
    // whole synchronous enumeration.
    let list = unsafe { &mut *(data.0 as *mut Vec<HWND>) };
    if class_name(hwnd) != "Progman" && find_child(Some(hwnd), None, w!("SHELLDLL_DefView")).is_some() {
        list.push(hwnd);
    }
    true.into()
}

unsafe extern "system" fn surface_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    // SAFETY: forwards the arguments the OS gave this window procedure, unchanged.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// Describes `hwnd` and, `depth` levels down, its children (topmost first).
fn node_of(hwnd: HWND, depth: u32) -> WindowNode {
    let mut children = Vec::new();
    if depth > 0 {
        // SAFETY: read-only walk over the child list of a window handle; an invalid handle ends the walk at once.
        let mut next = unsafe { GetWindow(hwnd, GW_CHILD) }.ok();
        while let Some(child) = next {
            children.push(node_of(child, depth - 1));
            // SAFETY: as above.
            next = unsafe { GetWindow(child, GW_HWNDNEXT) }.ok();
        }
    }
    let rect = window_rect(hwnd).unwrap_or_default();
    WindowNode {
        class: class_name(hwnd),
        ex_style: window_long(hwnd, GWL_EXSTYLE),
        style: window_long(hwnd, GWL_STYLE),
        // SAFETY: read-only query.
        visible: unsafe { IsWindowVisible(hwnd) }.as_bool(),
        rect: [rect.left, rect.top, rect.right, rect.bottom],
        children,
    }
}

fn find_window(class: PCWSTR) -> Option<HWND> {
    // SAFETY: `class` is a valid NUL-terminated string and a null title matches any.
    unsafe { FindWindowW(class, PCWSTR::null()) }
        .ok()
        .filter(|h| !h.is_invalid())
}

fn find_child(parent: Option<HWND>, after: Option<HWND>, class: PCWSTR) -> Option<HWND> {
    // SAFETY: `class` is a valid NUL-terminated string, the handles come from the OS, and a null title matches any.
    unsafe { FindWindowExW(parent, after, class, PCWSTR::null()) }
        .ok()
        .filter(|h| !h.is_invalid())
}

fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0_u16; 256];
    // SAFETY: the buffer is valid for writes of its whole length.
    let length = unsafe { GetClassNameW(hwnd, &mut buffer) };
    String::from_utf16_lossy(&buffer[..usize::try_from(length).unwrap_or(0)])
}

fn window_long(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> u32 {
    // SAFETY: reading a style of a window handle; an invalid handle only makes the call return 0.
    (unsafe { GetWindowLongPtrW(hwnd, index) }) as u32
}

fn set_window_long(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: u32) {
    // SAFETY: changes a style of a window this process owns. The return value is the old value, which is not needed.
    let _ = unsafe { SetWindowLongPtrW(hwnd, index, value as isize) };
}

fn window_rect(hwnd: HWND) -> Option<RectPx> {
    let mut rect = RECT::default();
    // SAFETY: `rect` is a valid out pointer.
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    Some(RectPx {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

fn create_window() -> Result<HWND, PlatformError> {
    // SAFETY: a null module name returns the handle of this executable.
    let module = unsafe { GetModuleHandleW(None) }.map_err(|e| os_error("GetModuleHandleW", &e))?;
    let instance = HINSTANCE(module.0);
    let class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(surface_proc),
        hInstance: instance,
        // SAFETY: a stock brush is owned by the OS and must not be deleted.
        hbrBackground: HBRUSH(unsafe { GetStockObject(BLACK_BRUSH) }.0),
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
    // A hidden popup first: a child needs a parent at creation, and the parent is only chosen while attaching.
    // SAFETY: the class is registered above and the strings are valid; no parent, menu or creation data is passed.
    unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            CLASS_NAME,
            w!("Sucrose desktop surface"),
            WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .map_err(|e| os_error("CreateWindowExW", &e))
}

fn set_layered(hwnd: HWND, layered: bool) -> Result<(), String> {
    let ex_style = window_long(hwnd, GWL_EXSTYLE);
    if layered {
        set_window_long(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0);
        // SAFETY: the window is layered just above; fully opaque, so the black fill shows as is.
        unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA) }
            .map_err(|e| format!("SetLayeredWindowAttributes: {e}"))
    } else {
        set_window_long(hwnd, GWL_EXSTYLE, ex_style & !WS_EX_LAYERED.0);
        Ok(())
    }
}

/// Makes `surface` a child of `parent` (SetParent leaves the WS_CHILD and WS_POPUP bits to the caller).
fn reparent(surface: HWND, parent: HWND) -> Result<(), String> {
    // SAFETY: both handles are windows; the surface belongs to this thread.
    unsafe { SetParent(surface, Some(parent)) }.map_err(|e| format!("SetParent: {e}"))?;
    let style = window_long(surface, GWL_STYLE);
    set_window_long(surface, GWL_STYLE, (style & !WS_POPUP.0) | WS_CHILD.0 | WS_VISIBLE.0);
    Ok(())
}

/// Moves `surface` onto the screen rectangle `target` (physical pixels), whatever the parent's own origin is.
fn place(surface: HWND, parent: HWND, target: RectPx) -> Result<(), String> {
    let mut origin = POINT::default();
    // SAFETY: `origin` is a valid in/out point; it turns the parent's client origin into screen coordinates.
    if !unsafe { ClientToScreen(parent, &mut origin) }.as_bool() {
        return Err("ClientToScreen failed".to_owned());
    }
    let local = target.to_parent((origin.x, origin.y));
    // SAFETY: plain window handle and integers; a null insert-after is ignored because of SWP_NOZORDER.
    unsafe {
        SetWindowPos(
            surface,
            None,
            local.left,
            local.top,
            local.width(),
            local.height(),
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
    }
    .map_err(|e| format!("SetWindowPos: {e}"))
}

fn verify(surface: HWND, expected_root: HWND, target: RectPx) -> Result<RectPx, String> {
    // SAFETY: read-only queries on window handles.
    let root = unsafe { GetAncestor(surface, GA_ROOT) };
    if root != expected_root {
        return Err(format!("the surface root is {root:?}, expected {expected_root:?}"));
    }
    // SAFETY: read-only query.
    if !unsafe { IsWindowVisible(surface) }.as_bool() {
        return Err("the surface is not visible".to_owned());
    }
    let actual = window_rect(surface).ok_or_else(|| "GetWindowRect failed".to_owned())?;
    if actual != target {
        return Err(format!("the surface rect is {actual:?}, expected {target:?}"));
    }
    Ok(actual)
}

impl DesktopSurface {
    /// Creates the surface and attaches it behind the desktop icons, covering `rect` (physical pixels of the virtual
    /// desktop). Retries for up to ten seconds while the shell settles.
    pub fn attach(rect: RectPx) -> Result<Self, PlatformError> {
        let surface = Self { hwnd: create_window()? };
        let deadline = Instant::now() + ATTACH_TIMEOUT;
        let mut delay = Duration::from_millis(100);
        let mut spawn_sent = false;
        loop {
            match surface.try_attach(rect, &mut spawn_sent) {
                Ok(()) => return Ok(surface),
                Err(reason) if Instant::now() >= deadline => {
                    tracing::error!(%reason, "the desktop surface could not be attached");
                    return Err(PlatformError::Os {
                        context: "attach the desktop surface",
                        code: 0,
                    });
                }
                Err(reason) => tracing::debug!(%reason, "attach attempt failed, retrying"),
            }
            thread::sleep(delay);
            delay = (delay * 2).min(Duration::from_secs(1));
        }
    }

    /// The window handle as a number, for engines that embed into it (mpv `wid`).
    pub fn native_handle(&self) -> u64 {
        self.hwnd.0 as usize as u64
    }

    fn try_attach(&self, target: RectPx, spawn_sent: &mut bool) -> Result<(), String> {
        let mut shell = Shell::capture();
        let progman = shell.progman.ok_or("Progman is not available")?;
        if !*spawn_sent && !shell.has_wallpaper_workerw() {
            *spawn_sent = true;
            send_spawn_message(progman);
            shell = Shell::capture();
        }
        let snapshot = shell.snapshot();
        tracing::debug!(snapshot = %serde_json::to_string(&snapshot).unwrap_or_default(), "shell window tree");
        let layout = layout::classify(&snapshot);
        let strategies: &[Strategy] = match layout {
            DesktopLayout::Raised => &[Strategy::LayeredChild, Strategy::WorkerWChild],
            DesktopLayout::Classic => &[Strategy::ClassicWorkerW],
            DesktopLayout::Unsupported => return Err("the desktop window tree matches no known layout".to_owned()),
        };
        let mut failures = Vec::new();
        for &strategy in strategies {
            match self.attach_with(strategy, &shell, progman, target) {
                Ok(actual) => {
                    tracing::info!(
                        ?layout,
                        ?strategy,
                        target = ?target,
                        actual = ?actual,
                        style = format_args!("{:#x}", window_long(self.hwnd, GWL_STYLE)),
                        ex_style = format_args!("{:#x}", window_long(self.hwnd, GWL_EXSTYLE)),
                        "surface attached"
                    );
                    return Ok(());
                }
                Err(reason) => failures.push(format!("{strategy:?}: {reason}")),
            }
        }
        Err(failures.join("; "))
    }

    fn attach_with(&self, strategy: Strategy, shell: &Shell, progman: HWND, target: RectPx) -> Result<RectPx, String> {
        let (parent, root) = match strategy {
            Strategy::LayeredChild => {
                let defview = shell
                    .defview_in_progman
                    .ok_or("Progman has no SHELLDLL_DefView child")?;
                set_layered(self.hwnd, true)?;
                reparent(self.hwnd, progman)?;
                place(self.hwnd, progman, target)?;
                // Right under the icons, above the system's own WorkerW.
                // SAFETY: both handles are windows; no move, size or activation is requested.
                unsafe {
                    SetWindowPos(
                        self.hwnd,
                        Some(defview),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    )
                }
                .map_err(|e| format!("SetWindowPos (z-order): {e}"))?;
                return verify(self.hwnd, progman, target);
            }
            Strategy::WorkerWChild => {
                let workerw = shell.workerw_in_progman.ok_or("Progman has no WorkerW child")?;
                (workerw, progman)
            }
            Strategy::ClassicWorkerW => {
                let workerw = shell.classic_workerw().ok_or("no WorkerW behind the icons")?;
                (workerw, workerw)
            }
        };
        set_layered(self.hwnd, false)?;
        reparent(self.hwnd, parent)?;
        place(self.hwnd, parent, target)?;
        verify(self.hwnd, root, target)
    }
}

fn send_spawn_message(progman: HWND) {
    let mut result = 0_usize;
    // SAFETY: `result` is a valid out pointer. The timeout keeps a hung explorer from blocking this thread (T-02-34).
    let sent = unsafe {
        SendMessageTimeoutW(
            progman,
            SPAWN_WORKERW,
            WPARAM(0xD),
            LPARAM(1),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        )
    };
    if sent.0 == 0 {
        tracing::warn!("Progman did not answer the WorkerW message within a second");
    }
}

impl Drop for DesktopSurface {
    fn drop(&mut self) {
        // SAFETY: the window was created by this thread and is destroyed exactly once, here. A failure leaves nothing
        // to act on at shutdown.
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}
