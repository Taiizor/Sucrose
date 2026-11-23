//! The surface a wallpaper is drawn on: a window parented into the desktop shell so it sits behind the icons.
//!
//! Two shell layouts exist. The classic one (Windows 7 to 11 23H2) turns into a top-level `WorkerW` window after the
//! `0x052C` message to Progman, and the surface becomes a child of that `WorkerW`. The raised one (Windows 11 24H2
//! and later) keeps `SHELLDLL_DefView` and a `WorkerW` as children of Progman, and a plain child of Progman is not
//! composited there. The layout is read from the window tree, never from the OS build number.

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem::size_of,
    time::{Duration, Instant},
};

use windows::{
    Win32::{
        Foundation::{
            COLORREF, ERROR_CLASS_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
        },
        Graphics::Gdi::{BLACK_BRUSH, ClientToScreen, GetStockObject, HBRUSH},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, EVENT_OBJECT_CREATE, EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_OBJECT_REORDER, EVENT_OBJECT_SHOW, EnumWindows, FindWindowExW, FindWindowW, GA_PARENT, GA_ROOT,
                GW_CHILD, GW_HWNDNEXT, GWL_EXSTYLE, GWL_STYLE, GetAncestor, GetClassNameW, GetClientRect,
                GetSystemMetrics, GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId,
                IsWindowVisible, KillTimer, LWA_ALPHA, MA_NOACTIVATE, OBJID_CLIENT, OBJID_WINDOW, PostQuitMessage,
                RegisterClassExW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
                SMTO_NORMAL, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SendMessageTimeoutW,
                SetLayeredWindowAttributes, SetParent, SetTimer, SetWindowLongPtrW, SetWindowPos,
                WINDOW_LONG_PTR_INDEX, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_DESTROY, WM_MOUSEACTIVATE,
                WNDCLASSEXW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_LAYERED, WS_EX_NOACTIVATE,
                WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
            },
        },
    },
    core::{BOOL, HRESULT, PCWSTR, w},
};

use super::{event_loop::wait_pumping, os_error};
use crate::{
    Placement, PlatformError, RectPx,
    geometry::{PlacementVerdict, placement_verdict},
    layout::{self, DEFVIEW_CLASS, DesktopLayout, RaisedOrder, ShellSnapshot, SiblingKind, WORKERW_CLASS, WindowNode},
};

const CLASS_NAME: PCWSTR = w!("SucroseDesktopSurface");
/// How long attaching keeps retrying: explorer can create its WorkerW late, or be restarting.
const ATTACH_TIMEOUT: Duration = Duration::from_secs(10);
/// The undocumented Progman message that makes the shell create the wallpaper `WorkerW`.
const SPAWN_WORKERW: u32 = 0x052C;

thread_local! {
    /// Surfaces that `Drop` is destroying on purpose, by window handle.
    static DROPPING: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
    /// Surfaces somebody else destroyed (the shell destroys its windows, and with them every child, when explorer.exe
    /// ends), by window handle.
    static LOST: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

/// Restores the order guard may do per surface within [`GUARD_WINDOW`] before it holds back (T-02-36).
const GUARD_MAX_RESTORES: usize = 10;
/// The span over which [`GUARD_MAX_RESTORES`] is counted.
const GUARD_WINDOW: Duration = Duration::from_secs(60);

/// A little after the oldest restore leaves [`GUARD_WINDOW`], so that the budget is open when the re-check looks.
const GUARD_RECHECK_MARGIN: Duration = Duration::from_millis(100);

/// What the order guard of one surface needs inside its WinEvent callback.
struct Guarded {
    hook: HWINEVENTHOOK,
    surface: HWND,
    progman: HWND,
    budget: RestoreBudget,
}

/// The restore budget of one guard (T-02-36) and the bookkeeping of its re-check.
#[derive(Debug, Default)]
struct RestoreBudget {
    /// When the guard last moved the surface, oldest first, only those within [`GUARD_WINDOW`].
    restores: VecDeque<Instant>,
    /// Whether the guard is holding back because the budget is used up; set to warn once per episode.
    throttled: bool,
    /// The thread timer that looks at the order again once the budget opens, when one is armed. The guard only
    /// reacts to shell events, so without it the last event of a burst that used the budget up would leave the
    /// surface under the shell's wallpaper until some unrelated event came (WR-14).
    recheck: Option<usize>,
}

impl RestoreBudget {
    /// What the budget says at `now` about a restore an event asks for. Nothing is counted here: a restore counts
    /// once it worked ([`RestoreBudget::record`]).
    fn consult(&mut self, now: Instant) -> Budget {
        while self
            .restores
            .front()
            .is_some_and(|at| now.duration_since(*at) > GUARD_WINDOW)
        {
            self.restores.pop_front();
        }
        if self.restores.len() >= GUARD_MAX_RESTORES
            && let Some(oldest) = self.restores.front()
        {
            let first = !self.throttled;
            self.throttled = true;
            // One re-check per episode of holding back: the timer is armed once and fires when the budget opens.
            let arm_in = self
                .recheck
                .is_none()
                .then(|| (*oldest + GUARD_WINDOW).saturating_duration_since(now) + GUARD_RECHECK_MARGIN);
            return Budget::Exhausted { first, arm_in };
        }
        self.throttled = false;
        Budget::Allowed
    }

    /// Counts a restore that moved the surface.
    fn record(&mut self, now: Instant) {
        self.restores.push_back(now);
    }
}

/// What the restore budget of a guard says about the restore an event asks for.
#[derive(Debug, PartialEq, Eq)]
enum Budget {
    /// Restore now.
    Allowed,
    /// The budget is used up; `first` is true for the first refusal of this episode. `arm_in` is the time to the
    /// re-check that has to be armed, `None` when one is armed already.
    Exhausted { first: bool, arm_in: Option<Duration> },
    /// No guard can be consulted (the registry is borrowed or the guard is gone); do nothing.
    Unavailable,
}

/// What the location watch of one surface needs inside its WinEvent callback.
struct Watched {
    hook: HWINEVENTHOOK,
    surface: HWND,
    /// The window the surface is a child of, right after the attach.
    parent: HWND,
    /// The top-level window of `parent` (Progman for the raised layout, the `WorkerW` itself for the classic one).
    root: HWND,
    /// Where the surface belongs on the screen, in physical pixels.
    target: RectPx,
    /// The parent's client area on the screen at the last placement; `None` when it could not be read.
    parent_rect: Option<RectPx>,
    /// Whether the surface lies partly or wholly outside the parent, as of the last placement (an "outside episode").
    outside: bool,
}

/// A change in whether the surface lies outside its desktop window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Episode {
    /// The surface was inside (or its verdict was not known yet) and now lies outside: warn once.
    Started,
    /// The surface lay outside and is shown again: report the recovery once.
    Ended,
}

/// The watch's registry could not be used (it is borrowed right now, or the watch is gone).
struct Busy;

/// What a placement with `verdict` changes in an outside episode that `was_outside`. PartlyHidden and Hidden count
/// as outside; Misplaced says nothing about the parent, so it changes nothing (its caller logs it on its own).
fn outside_episode(was_outside: bool, verdict: PlacementVerdict) -> Option<Episode> {
    let outside = matches!(
        verdict,
        PlacementVerdict::PartlyHidden { .. } | PlacementVerdict::Hidden
    );
    match (was_outside, verdict) {
        (false, _) if outside => Some(Episode::Started),
        (true, PlacementVerdict::Shown) => Some(Episode::Ended),
        _ => None,
    }
}

/// Stores the target and the parent's client area of a placement in the watch `entry`, so that a later parent move
/// re-places at the newest target, and moves the outside episode on. Returns what changed in the episode.
fn record_placement(entry: &mut Watched, target: RectPx, placement: &Placement) -> Option<Episode> {
    entry.target = target;
    entry.parent_rect = Some(placement.parent);
    let change = outside_episode(entry.outside, placement.verdict);
    match change {
        Some(Episode::Started) => entry.outside = true,
        Some(Episode::Ended) => entry.outside = false,
        None => {}
    }
    change
}

/// Records a placement of the surface of the watch `hook` (see [`record_placement`]); the registry is only borrowed
/// for the update, never while a window moves.
fn note_placement(hook: HWINEVENTHOOK, target: RectPx, placement: &Placement) -> Result<Option<Episode>, Busy> {
    WATCHED.with(|watched| {
        let mut watched = watched.try_borrow_mut().map_err(|_| Busy)?;
        let entry = watched.iter_mut().find(|entry| entry.hook == hook).ok_or(Busy)?;
        Ok(record_placement(entry, target, placement))
    })
}

/// Logs the start of an outside episode (a warning) or its end.
fn log_episode(change: Option<Episode>, placement: &Placement) {
    match change {
        Some(Episode::Started) => tracing::warn!(
            target = ?placement.target,
            parent = ?placement.parent,
            virtual_screen = ?placement.virtual_screen,
            "the surface lies outside the desktop window"
        ),
        Some(Episode::Ended) => tracing::info!(
            target = ?placement.target,
            parent = ?placement.parent,
            "the surface is inside the desktop window again"
        ),
        None => {}
    }
}

thread_local! {
    /// The surfaces of this thread that have a location watch, found by hook handle like [`GUARDS`].
    static WATCHED: RefCell<Vec<Watched>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    /// The surfaces of this thread that have an order guard. The WinEvent callback has no user data, so it finds its
    /// surface here by the hook handle it was called for.
    static GUARDS: RefCell<Vec<Guarded>> = const { RefCell::new(Vec::new()) };
}

/// A window behind the desktop icons of one monitor. It lives on the thread that created it, which must pump
/// messages (see [`crate::EventLoop`]), so it is not `Send`.
#[derive(Debug)]
pub struct DesktopSurface {
    hwnd: HWND,
    /// The WinEvent hook that keeps the surface above the shell's wallpaper `WorkerW` (see [`order_guard_proc`]).
    guard: Cell<Option<HWINEVENTHOOK>>,
    /// The WinEvent hook that puts the surface back on its target when explorer moves its parent window (see
    /// [`location_watch_proc`]).
    watch: Cell<Option<HWINEVENTHOOK>>,
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
    if message == WM_DESTROY {
        let handle = hwnd.0 as isize;
        if !DROPPING.with(|dropping| dropping.borrow().contains(&handle)) {
            LOST.with(|lost| lost.borrow_mut().push(handle));
            // SAFETY: only asks this thread's message loop to end; there is nothing to validate.
            unsafe { PostQuitMessage(0) };
        }
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

/// The client area of `hwnd` on the screen (physical pixels): where a child of it can show.
fn client_rect_on_screen(hwnd: HWND) -> Option<RectPx> {
    let mut client = RECT::default();
    // SAFETY: `client` is a valid out pointer.
    unsafe { GetClientRect(hwnd, &mut client) }.ok()?;
    let mut origin = POINT::default();
    // SAFETY: `origin` is a valid in/out point; it turns the client origin into screen coordinates.
    if !unsafe { ClientToScreen(hwnd, &mut origin) }.as_bool() {
        return None;
    }
    Some(RectPx {
        left: origin.x,
        top: origin.y,
        right: origin.x + (client.right - client.left),
        bottom: origin.y + (client.bottom - client.top),
    })
}

/// The whole virtual desktop (every monitor) in physical pixels.
fn virtual_screen() -> RectPx {
    // SAFETY: plain metric queries without arguments to validate.
    let (left, top, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };
    RectPx {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

/// Reads where `surface` is relative to its `parent` and judges it against `target`, without moving anything.
fn read_placement(surface: HWND, parent: HWND, target: RectPx) -> Result<Placement, String> {
    let parent_area =
        client_rect_on_screen(parent).ok_or_else(|| "the parent's client area is unreadable".to_owned())?;
    let actual = window_rect(surface).ok_or_else(|| "GetWindowRect failed".to_owned())?;
    Ok(Placement {
        target,
        local: target.to_parent((parent_area.left, parent_area.top)),
        parent: parent_area,
        actual,
        virtual_screen: virtual_screen(),
        verdict: placement_verdict(target, actual, parent_area),
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
                code: HRESULT::from_win32(code.0).0,
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

/// Makes `surface` a child of `parent`. SetParent leaves the WS_CHILD and WS_POPUP bits to the caller, and its
/// documentation asks for them to be changed before the call when a top-level window becomes a child, so they are.
fn reparent(surface: HWND, parent: HWND) -> Result<(), String> {
    let style = window_long(surface, GWL_STYLE);
    set_window_long(surface, GWL_STYLE, (style & !WS_POPUP.0) | WS_CHILD.0 | WS_VISIBLE.0);
    // SAFETY: both handles are windows; the surface belongs to this thread.
    if let Err(error) = unsafe { SetParent(surface, Some(parent)) } {
        // The window stays what it was, so that a later attempt starts from the same state.
        set_window_long(surface, GWL_STYLE, style);
        return Err(format!("SetParent: {error}"));
    }
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

/// Moves `surface` like [`place`], then reads back where it is and judges it against its parent.
fn place_checked(surface: HWND, parent: HWND, target: RectPx) -> Result<Placement, String> {
    place(surface, parent, target)?;
    read_placement(surface, parent, target)
}

/// Progman's children, topmost first.
fn progman_children(progman: HWND) -> Vec<HWND> {
    let mut children = Vec::new();
    // SAFETY: read-only walk over the child list of a window handle; an invalid handle ends the walk at once.
    let mut next = unsafe { GetWindow(progman, GW_CHILD) }.ok();
    while let Some(child) = next {
        children.push(child);
        // SAFETY: as above.
        next = unsafe { GetWindow(child, GW_HWNDNEXT) }.ok();
    }
    children
}

/// What each child of Progman is, topmost first, for [`layout::raised_order`]; `surface` is the window being checked.
fn progman_order(progman: HWND, surface: HWND) -> Vec<SiblingKind> {
    progman_children(progman)
        .into_iter()
        .map(|child| {
            if child == surface {
                return SiblingKind::Surface;
            }
            match class_name(child).as_str() {
                DEFVIEW_CLASS => SiblingKind::Icons,
                WORKERW_CLASS => SiblingKind::Wallpaper,
                _ => SiblingKind::Other,
            }
        })
        .collect()
}

/// The classes of Progman's children, topmost first, joined with " > " (for example
/// "SHELLDLL_DefView > SucroseDesktopSurface > WorkerW"), for logs and errors.
fn order_text(progman: HWND) -> String {
    progman_children(progman)
        .into_iter()
        .map(class_name)
        .collect::<Vec<_>>()
        .join(" > ")
}

fn verify(surface: HWND, expected_root: HWND, target: RectPx, strategy: Strategy) -> Result<RectPx, String> {
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
    // Only here the surface is a sibling of the shell's own windows; in the other strategies it is a child of the
    // WorkerW and the shell's window order does not matter.
    if strategy == Strategy::LayeredChild {
        let order = layout::raised_order(&progman_order(expected_root, surface));
        if order != RaisedOrder::Correct {
            return Err(format!(
                "the Progman child order is {} ({order:?}), expected SHELLDLL_DefView, the surface, then every WorkerW",
                order_text(expected_root)
            ));
        }
    }
    Ok(actual)
}

impl DesktopSurface {
    /// Creates the surface and attaches it behind the desktop icons, covering `rect` (physical pixels of the virtual
    /// desktop). Retries for up to ten seconds while the shell settles, dispatching this thread's messages between
    /// the attempts.
    pub fn attach(rect: RectPx) -> Result<Self, PlatformError> {
        let surface = Self {
            hwnd: create_window()?,
            guard: Cell::new(None),
            watch: Cell::new(None),
        };
        let deadline = Instant::now() + ATTACH_TIMEOUT;
        let mut delay = Duration::from_millis(100);
        loop {
            match surface.try_attach(rect, false) {
                Ok(()) => return Ok(surface),
                Err(reason) if Instant::now() >= deadline => {
                    // Last resort: a shell that never created its wallpaper WorkerW still gets the surface, under
                    // the icons, and the order guard keeps it above the WorkerW if one shows up later.
                    tracing::debug!(%reason, "attach attempt failed at the deadline, trying once without the WorkerW");
                    return match surface.try_attach(rect, true) {
                        Ok(()) => Ok(surface),
                        Err(reason) => {
                            tracing::error!(%reason, "the desktop surface could not be attached");
                            Err(PlatformError::Failed {
                                context: "attach the desktop surface",
                                message: reason,
                            })
                        }
                    };
                }
                Err(reason) => tracing::debug!(%reason, "attach attempt failed, retrying"),
            }
            // The thread owns two windows: waiting must not stop their messages, or the shell and the service see
            // an unresponsive process for as long as the shell takes to settle (WR-08).
            if wait_pumping(delay) {
                tracing::warn!("the message loop was asked to quit while the surface was attaching");
                return Err(PlatformError::Failed {
                    context: "attach the desktop surface",
                    message: "the message loop was asked to quit".to_owned(),
                });
            }
            delay = (delay * 2).min(Duration::from_secs(1));
        }
    }

    /// Moves the surface onto `rect` (physical pixels of the virtual desktop) without recreating it, so what is
    /// embedded in it keeps playing; the embedded window follows the surface's size by itself.
    ///
    /// Returns where the surface really is relative to its parent; a surface the parent clips away is not an error,
    /// the caller reports it.
    pub fn set_rect(&self, rect: RectPx) -> Result<Placement, PlatformError> {
        // SAFETY: read-only query on a window handle owned by this thread.
        let parent = unsafe { GetAncestor(self.hwnd, GA_PARENT) };
        if parent.is_invalid() {
            return Err(PlatformError::Failed {
                context: "move the desktop surface",
                message: "the surface has no parent window".to_owned(),
            });
        }
        let placement = place_checked(self.hwnd, parent, rect).map_err(|message| PlatformError::Failed {
            context: "move the desktop surface",
            message,
        })?;
        // The watch re-places at the newest target from now on (the window is not moving here, so the registry is
        // only borrowed briefly).
        if let Some(hook) = self.watch.get() {
            match note_placement(hook, rect, &placement) {
                Ok(change) => log_episode(change, &placement),
                Err(Busy) => tracing::debug!("the location watch is busy, it keeps its previous target"),
            }
        }
        Ok(placement)
    }

    /// Whether the surface was destroyed by someone else (Windows destroyed the window; a killed explorer.exe was
    /// observed not to do this). A renderer whose surface is gone has to be replaced, not repaired.
    pub fn destroyed_externally(&self) -> bool {
        let handle = self.hwnd.0 as isize;
        LOST.with(|lost| lost.borrow().contains(&handle))
    }

    /// The window handle as a number, for engines that embed into it (mpv `wid`).
    pub fn native_handle(&self) -> u64 {
        self.hwnd.0 as usize as u64
    }

    /// One attach attempt. `last_resort` is only set for the attempt after the deadline: it attaches in the raised
    /// layout even though the shell has no wallpaper `WorkerW` yet.
    fn try_attach(&self, target: RectPx, last_resort: bool) -> Result<(), String> {
        let mut shell = Shell::capture();
        let progman = shell.progman.ok_or("Progman is not available")?;
        if !shell.has_wallpaper_workerw() {
            // Sent again by every attempt that finds no WorkerW (the legacy app retries it too): after an explorer
            // restart Progman's desktop thread can still be starting and miss the first one.
            if !send_spawn_message(progman) {
                return Err("Progman did not answer the WorkerW message within a second".to_owned());
            }
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
        let without_workerw = layout == DesktopLayout::Raised && shell.workerw_in_progman.is_none();
        if without_workerw && !last_resort {
            return Err("Progman has not created its wallpaper WorkerW yet".to_owned());
        }
        let mut failures = Vec::new();
        for &strategy in strategies {
            match self.attach_with(strategy, &shell, progman, target) {
                Ok(actual) => {
                    if without_workerw {
                        tracing::warn!(
                            "attached before the shell created its wallpaper WorkerW; the order guard keeps the surface above it"
                        );
                    }
                    if strategy == Strategy::LayeredChild {
                        self.install_order_guard(progman);
                    }
                    let order = (strategy == Strategy::LayeredChild).then(|| order_text(progman));
                    // SAFETY: read-only query on a window handle owned by this thread.
                    let parent = unsafe { GetAncestor(self.hwnd, GA_PARENT) };
                    // Judged, never refused: a renderer that would not attach spends its restart budget while the
                    // shell is stale, and a surface outside the desktop window shows up once the shell moves it.
                    let placement = read_placement(self.hwnd, parent, target);
                    self.install_location_watch(
                        parent,
                        root_of(parent),
                        target,
                        placement.as_ref().ok().map(|p| p.parent),
                    );
                    if let Ok(placement) = &placement {
                        // The first placement of the watch: an outside attach starts an episode (warned once).
                        let first = outside_episode(false, placement.verdict);
                        let change = match self.watch.get() {
                            Some(hook) => note_placement(hook, target, placement).unwrap_or(first),
                            None => first,
                        };
                        log_episode(change, placement);
                    }
                    tracing::info!(
                        ?layout,
                        ?strategy,
                        target = ?target,
                        actual = ?actual,
                        parent = ?placement.as_ref().ok().map(|p| p.parent),
                        verdict = ?placement.as_ref().ok().map(|p| p.verdict),
                        order = order.as_deref(),
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

    /// Starts the order guard (see [`order_guard_proc`]) for this surface; a failure is logged and the surface
    /// works without the guard.
    fn install_order_guard(&self, progman: HWND) {
        let mut process = 0_u32;
        // SAFETY: read-only query with a valid out pointer.
        let thread = unsafe { GetWindowThreadProcessId(progman, Some(&raw mut process)) };
        if thread == 0 || process == 0 {
            tracing::warn!("cannot find the process of Progman, the order guard is not installed");
            return;
        }
        // SAFETY: the callback is a plain function that lives for the whole process and only touches this thread's
        // state; out-of-context hooks run it on this thread, inside its message loop. The hook is removed in `Drop`.
        let hook = unsafe {
            SetWinEventHook(
                EVENT_OBJECT_CREATE,
                EVENT_OBJECT_REORDER,
                None,
                Some(order_guard_proc),
                process,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.is_invalid() {
            tracing::warn!("cannot install the order guard, the shell may put its wallpaper above the surface");
            return;
        }
        GUARDS.with(|guards| {
            guards.borrow_mut().push(Guarded {
                hook,
                surface: self.hwnd,
                progman,
                budget: RestoreBudget::default(),
            });
        });
        self.guard.set(Some(hook));
    }

    /// Starts the location watch (see [`location_watch_proc`]) for this surface, once; a failure is logged and the
    /// surface works without the watch.
    ///
    /// The hook is limited to the process and thread that own `root`: explorer's desktop thread owns Progman and its
    /// `WorkerW` and moves and resizes them itself, so the location events of the taskbar, File Explorer windows and
    /// the cursor (other threads) never wake the renderer. There is no polling (D-11). Hook delivery cannot be tested
    /// in-process; it was checked on a real desktop (UAT G-02-7, D-031).
    fn install_location_watch(&self, parent: HWND, root: HWND, target: RectPx, parent_rect: Option<RectPx>) {
        if self.watch.get().is_some() || parent.is_invalid() || root.is_invalid() {
            return;
        }
        let mut process = 0_u32;
        // SAFETY: read-only query with a valid out pointer.
        let thread = unsafe { GetWindowThreadProcessId(root, Some(&raw mut process)) };
        if thread == 0 || process == 0 {
            tracing::warn!("cannot find the thread of the desktop window, the location watch is not installed");
            return;
        }
        // SAFETY: the callback is a plain function that lives for the whole process and only touches this thread's
        // state; out-of-context hooks run it on this thread, inside its message loop. The hook is removed in `Drop`.
        let hook = unsafe {
            SetWinEventHook(
                EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_OBJECT_LOCATIONCHANGE,
                None,
                Some(location_watch_proc),
                process,
                thread,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.is_invalid() {
            tracing::warn!("cannot install the location watch, the surface does not follow its desktop window");
            return;
        }
        WATCHED.with(|watched| {
            watched.borrow_mut().push(Watched {
                hook,
                surface: self.hwnd,
                parent,
                root,
                target,
                parent_rect,
                outside: false,
            });
        });
        self.watch.set(Some(hook));
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
                return verify(self.hwnd, progman, target, strategy);
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
        verify(self.hwnd, root, target, strategy)
    }
}

/// Asks Progman to create its wallpaper `WorkerW`; `false` when it did not answer within a second.
fn send_spawn_message(progman: HWND) -> bool {
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
    sent.0 != 0
}

/// The order guard: puts the surface back between the icons and the shell's wallpaper `WorkerW` whenever the shell
/// creates, shows or reorders windows in Progman. It is a WinEvent hook (no polling, D-11) limited to the process
/// that owns Progman and to five event ids, so a callback costs O(1) work and most return after two comparisons.
///
/// It lives as long as the surface, not only after a shell restart: explorer can recreate its `WorkerW` at any time
/// (a wallpaper change, a slideshow step). It only ever moves its own surface; it never moves, resizes or restyles a
/// shell window. Past [`GUARD_MAX_RESTORES`] restores in [`GUARD_WINDOW`] it holds back (the hook stays) until the
/// oldest restore leaves the window, so a shell that keeps reordering cannot keep the renderer busy (T-02-36), and
/// when it holds back it arms a one-shot timer that looks at the order again at that moment, so the end of a burst
/// of events cannot leave the surface hidden until an unrelated event comes. Only a restore that worked is counted.
unsafe extern "system" fn order_guard_proc(
    hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if hwnd.is_invalid() {
        return;
    }
    // A short borrow: the registry is not held while windows are queried or moved.
    let Some((surface, progman)) = GUARDS.with(|guards| {
        guards
            .try_borrow()
            .ok()?
            .iter()
            .find(|guarded| guarded.hook == hook)
            .map(|guarded| (guarded.surface, guarded.progman))
    }) else {
        return;
    };
    let relevant = match event {
        // Progman's children were reordered. The event names Progman itself (the object id is the window or its
        // client area, depending on who raised it).
        EVENT_OBJECT_REORDER => hwnd == progman && (id_object == OBJID_WINDOW.0 || id_object == OBJID_CLIENT.0),
        EVENT_OBJECT_CREATE | EVENT_OBJECT_SHOW => {
            id_object == OBJID_WINDOW.0 && {
                // SAFETY: read-only query on a window handle the event gave this callback.
                let parent = unsafe { GetAncestor(hwnd, GA_PARENT) };
                parent == progman && class_name(hwnd) == WORKERW_CLASS
            }
        }
        _ => false,
    };
    if relevant {
        restore_order(hook, surface, progman);
    }
}

/// Moves `surface` back under the icons when the order of Progman's children is wrong, within the restore budget.
fn restore_order(hook: HWINEVENTHOOK, surface: HWND, progman: HWND) {
    let order = layout::raised_order(&progman_order(progman, surface));
    if !matches!(
        order,
        RaisedOrder::WallpaperAboveSurface | RaisedOrder::SurfaceAboveIcons
    ) {
        return;
    }
    let budget = GUARDS.with(|guards| {
        let Ok(mut guards) = guards.try_borrow_mut() else {
            return Budget::Unavailable;
        };
        let Some(guarded) = guards.iter_mut().find(|guarded| guarded.hook == hook) else {
            return Budget::Unavailable;
        };
        guarded.budget.consult(Instant::now())
    });
    match budget {
        Budget::Allowed => {}
        Budget::Exhausted { first, arm_in } => {
            if first {
                tracing::warn!("the shell keeps reordering the desktop, the order guard holds back for a while");
            }
            if let Some(arm_in) = arm_in {
                arm_recheck(hook, arm_in);
            }
            return;
        }
        Budget::Unavailable => return,
    }
    let Some(defview) = find_child(Some(progman), None, w!("SHELLDLL_DefView")) else {
        return;
    };
    let before = order_text(progman);
    // SAFETY: both handles are windows; only the z-order of this renderer's own surface changes, no move, size or
    // activation is requested.
    let moved = unsafe {
        SetWindowPos(
            surface,
            Some(defview),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    };
    match moved {
        Ok(()) => {
            GUARDS.with(|guards| {
                if let Ok(mut guards) = guards.try_borrow_mut()
                    && let Some(guarded) = guards.iter_mut().find(|guarded| guarded.hook == hook)
                {
                    guarded.budget.record(Instant::now());
                }
            });
            tracing::info!(
                before = %before,
                after = %order_text(progman),
                "the shell put its wallpaper above the surface, order restored"
            );
        }
        Err(error) => tracing::warn!(%error, before = %before, "cannot restore the surface order"),
    }
}

/// Arms the one-shot re-check of the guard of `hook` in `delay`. The timer belongs to this thread's message queue (no
/// window), which the surface's thread pumps; [`recheck_proc`] ends it.
fn arm_recheck(hook: HWINEVENTHOOK, delay: Duration) {
    let millis = u32::try_from(delay.as_millis()).unwrap_or(u32::MAX).max(1);
    // SAFETY: a thread timer without a window; the callback is a plain function that lives for the whole process.
    let id = unsafe { SetTimer(None, 0, millis, Some(recheck_proc)) };
    if id == 0 {
        tracing::warn!("cannot arm the order guard's re-check, it only acts on the next shell event");
        return;
    }
    let stored = GUARDS.with(|guards| {
        let Ok(mut guards) = guards.try_borrow_mut() else {
            return false;
        };
        match guards.iter_mut().find(|guarded| guarded.hook == hook) {
            Some(guarded) => {
                guarded.budget.recheck = Some(id);
                true
            }
            None => false,
        }
    });
    if !stored {
        // The guard is gone (or the registry is busy): the timer would find nobody, so it is removed again.
        // SAFETY: `id` is the timer just created on this thread.
        let _ = unsafe { KillTimer(None, id) };
    }
}

/// Who a re-check timer belongs to.
enum Recheck {
    /// The registry is borrowed right now; the timer stays and fires again after its period.
    Busy,
    /// No guard owns the timer (the guard was removed): it only has to end.
    Nobody,
    /// The guard `hook` of `surface` asked for it.
    Guard {
        hook: HWINEVENTHOOK,
        surface: HWND,
        progman: HWND,
    },
}

/// The re-check of an order guard: the budget has had time to open, so the order is looked at again.
unsafe extern "system" fn recheck_proc(_hwnd: HWND, _message: u32, id: usize, _time: u32) {
    let owner = GUARDS.with(|guards| {
        let Ok(mut guards) = guards.try_borrow_mut() else {
            return Recheck::Busy;
        };
        match guards.iter_mut().find(|guarded| guarded.budget.recheck == Some(id)) {
            Some(guarded) => {
                guarded.budget.recheck = None;
                Recheck::Guard {
                    hook: guarded.hook,
                    surface: guarded.surface,
                    progman: guarded.progman,
                }
            }
            None => Recheck::Nobody,
        }
    });
    if matches!(owner, Recheck::Busy) {
        return;
    }
    // SAFETY: `id` is a thread timer of this thread. It is one-shot by design and removed before the order is looked
    // at, because that may arm a new one.
    let _ = unsafe { KillTimer(None, id) };
    if let Recheck::Guard { hook, surface, progman } = owner {
        restore_order(hook, surface, progman);
    }
}

/// The top-level window of `window` (itself when it has no parent).
fn root_of(window: HWND) -> HWND {
    // SAFETY: read-only query on a window handle.
    unsafe { GetAncestor(window, GA_ROOT) }
}

/// Whether a location event asks for a re-place: the event is about a window's own location (not its caret, cursor
/// or client area), the window is the watched parent or its root, and the parent's client area now differs from the
/// one recorded at the last placement. `current` is only asked for once the cheap comparisons agree, and a parent that
/// cannot be read changes nothing.
fn location_reaction(
    window: HWND,
    id_object: i32,
    parent: HWND,
    root: HWND,
    recorded: Option<RectPx>,
    current: impl FnOnce() -> Option<RectPx>,
) -> bool {
    if id_object != OBJID_WINDOW.0 || (window != parent && window != root) {
        return false;
    }
    current().is_some_and(|now| recorded != Some(now))
}

/// The location watch: explorer moved or resized a window, so if it is the one the surface lives in the surface is put
/// back at its stored target at once.
///
/// A surface's screen position is its parent's client origin plus the offset fixed at its last placement, so a parent
/// that moves drags the surface along (off its monitor), and a parent that moves back over a target that lay outside
/// it shows the surface again only if somebody re-places it. Only the renderer's own surface is ever moved here.
unsafe extern "system" fn location_watch_proc(
    hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if event != EVENT_OBJECT_LOCATIONCHANGE || hwnd.is_invalid() {
        return;
    }
    // A short borrow: the registry is not held while windows are queried or moved.
    let Some((surface, parent, root, target, recorded)) = WATCHED.with(|watched| {
        watched
            .try_borrow()
            .ok()?
            .iter()
            .find(|entry| entry.hook == hook)
            .map(|entry| (entry.surface, entry.parent, entry.root, entry.target, entry.parent_rect))
    }) else {
        return;
    };
    if location_reaction(hwnd, id_object, parent, root, recorded, || {
        client_rect_on_screen(parent)
    }) {
        follow_parent(hook, surface, parent, target, recorded);
    }
}

/// Puts `surface` back at `target` inside its `parent` after the parent moved, remembers the parent's new client
/// area and logs the move.
fn follow_parent(hook: HWINEVENTHOOK, surface: HWND, parent: HWND, target: RectPx, previous: Option<RectPx>) {
    match place_checked(surface, parent, target) {
        Ok(placement) => {
            match note_placement(hook, target, &placement) {
                Ok(change) => log_episode(change, &placement),
                Err(Busy) => tracing::debug!("the location watch is busy, its record of the parent stays as it was"),
            }
            tracing::info!(
                previous = ?previous,
                parent = ?placement.parent,
                target = ?placement.target,
                local = ?placement.local,
                verdict = ?placement.verdict,
                "the desktop window moved, surface re-placed"
            );
        }
        Err(error) => tracing::warn!(%error, "cannot re-place the surface after the desktop window moved"),
    }
}

/// Removes the watch of `hook`: forgets it and removes the hook; `false` when it was already removed.
fn remove_watch(hook: HWINEVENTHOOK) -> bool {
    let known = WATCHED.with(|watched| {
        let mut watched = watched.borrow_mut();
        let before = watched.len();
        watched.retain(|entry| entry.hook != hook);
        watched.len() != before
    });
    if known {
        // SAFETY: the hook was installed by this thread and is removed exactly once, because the registry entry
        // that proves it is still installed is gone now.
        let _ = unsafe { UnhookWinEvent(hook) };
    }
    known
}

/// Forgets the guard of `hook` and removes the hook; `false` when it was already removed.
fn remove_guard(hook: HWINEVENTHOOK) -> bool {
    let (known, recheck) = GUARDS.with(|guards| {
        let mut guards = guards.borrow_mut();
        let before = guards.len();
        let recheck = guards
            .iter()
            .find(|guarded| guarded.hook == hook)
            .and_then(|guarded| guarded.budget.recheck);
        guards.retain(|guarded| guarded.hook != hook);
        (guards.len() != before, recheck)
    });
    if let Some(id) = recheck {
        // SAFETY: the timer was armed by this thread for the guard that is gone now.
        let _ = unsafe { KillTimer(None, id) };
    }
    if known {
        // SAFETY: the hook was installed by this thread and is removed exactly once, because the registry entry
        // that proves it is still installed is gone now.
        let _ = unsafe { UnhookWinEvent(hook) };
    }
    known
}

impl Drop for DesktopSurface {
    fn drop(&mut self) {
        if let Some(hook) = self.guard.take() {
            remove_guard(hook);
        }
        if let Some(hook) = self.watch.take() {
            remove_watch(hook);
        }
        let handle = self.hwnd.0 as isize;
        // A surface the shell already destroyed has no window left to destroy, and its handle value may be reused.
        if !self.destroyed_externally() {
            DROPPING.with(|dropping| dropping.borrow_mut().push(handle));
            // SAFETY: the window was created by this thread and is destroyed exactly once, here. A failure leaves
            // nothing to act on at shutdown.
            let _ = unsafe { DestroyWindow(self.hwnd) };
            DROPPING.with(|dropping| dropping.borrow_mut().retain(|h| *h != handle));
        }
        LOST.with(|lost| lost.borrow_mut().retain(|h| *h != handle));
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use windows::Win32::UI::WindowsAndMessaging::{OBJID_CARET, OBJID_CURSOR};

    use super::*;

    fn spent(now: Instant) -> RestoreBudget {
        let mut budget = RestoreBudget::default();
        for _ in 0..GUARD_MAX_RESTORES {
            budget.record(now);
        }
        budget
    }

    #[test]
    fn a_budget_with_room_allows_and_counts_nothing_until_a_restore_worked() {
        let now = Instant::now();
        let mut budget = RestoreBudget::default();

        // Asking again and again does not use the budget up: only recorded restores count.
        for _ in 0..(GUARD_MAX_RESTORES * 3) {
            assert_eq!(budget.consult(now), Budget::Allowed);
        }
        assert!(budget.restores.is_empty());
    }

    #[test]
    fn a_spent_budget_asks_for_one_recheck_when_the_oldest_restore_leaves_the_window() {
        let start = Instant::now();
        let mut budget = spent(start);
        let later = start + Duration::from_secs(20);

        let Budget::Exhausted { first, arm_in } = budget.consult(later) else {
            panic!("the budget is spent");
        };

        assert!(first, "the first refusal of the episode");
        assert_eq!(arm_in, Some(Duration::from_secs(40) + GUARD_RECHECK_MARGIN));
        // Once the timer is armed no second one is asked for, and the episode is not announced again.
        budget.recheck = Some(1);
        let Budget::Exhausted { first, arm_in } = budget.consult(later) else {
            panic!("still spent");
        };
        assert!(!first);
        assert_eq!(arm_in, None);
    }

    #[test]
    fn the_budget_opens_again_after_the_window() {
        let start = Instant::now();
        let mut budget = spent(start);

        let after = start + GUARD_WINDOW + GUARD_RECHECK_MARGIN;

        assert_eq!(
            budget.consult(after),
            Budget::Allowed,
            "the recheck finds the budget open"
        );
        assert!(budget.restores.is_empty());
        assert!(!budget.throttled, "a new episode is announced again");
    }

    #[test]
    fn placement_check_against_a_real_parent_window() {
        fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
            RectPx {
                left,
                top,
                right,
                bottom,
            }
        }
        // Both windows stay hidden: the parent is never shown, and a child of a hidden window shows nothing.
        let parent = create_window().expect("the parent window");
        let child = create_window().expect("the child window");
        let parent_rect = rect(-200, 0, 200, 100);
        // SAFETY: plain handle and integers; SWP_NOZORDER makes the null insert-after harmless and nothing is shown.
        unsafe {
            SetWindowPos(
                parent,
                None,
                parent_rect.left,
                parent_rect.top,
                parent_rect.width(),
                parent_rect.height(),
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
        .expect("move the parent");
        reparent(child, parent).expect("reparent the child");

        let cases = [
            (rect(200, 0, 400, 100), rect(400, 0, 600, 100), PlacementVerdict::Hidden),
            (rect(0, 0, 200, 100), rect(200, 0, 400, 100), PlacementVerdict::Shown),
            (
                rect(100, 0, 300, 100),
                rect(300, 0, 500, 100),
                PlacementVerdict::PartlyHidden {
                    visible: rect(100, 0, 200, 100),
                },
            ),
        ];
        let mut seen = Vec::new();
        for (target, local, verdict) in cases {
            seen.push((place_checked(child, parent, target), local, verdict, target));
        }

        // Destroyed the way `Drop` does, so that the window procedure posts no quit to this thread.
        for window in [child, parent] {
            DROPPING.with(|dropping| dropping.borrow_mut().push(window.0 as isize));
            // SAFETY: a window created above by this thread.
            let _ = unsafe { DestroyWindow(window) };
            DROPPING.with(|dropping| dropping.borrow_mut().retain(|&handle| handle != window.0 as isize));
        }

        for (placement, local, verdict, target) in seen {
            let placement = placement.expect("the placement");
            assert_eq!(placement.verdict, verdict, "verdict for {target:?}");
            assert_eq!(placement.local, local, "local rectangle for {target:?}");
            assert_eq!(placement.parent, parent_rect, "the parent's client area");
            assert_eq!(placement.actual, target, "the surface lands on its target");
        }
    }

    fn test_rect(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
        RectPx {
            left,
            top,
            right,
            bottom,
        }
    }

    fn handle(value: usize) -> HWND {
        HWND(value as *mut c_void)
    }

    #[test]
    fn the_watch_reacts_to_a_move_of_the_parent() {
        let (parent, root) = (handle(0x10), handle(0x20));
        let before = test_rect(-200, 0, 200, 100);
        let after = test_rect(0, 0, 400, 100);
        assert!(location_reaction(
            parent,
            OBJID_WINDOW.0,
            parent,
            root,
            Some(before),
            || Some(after)
        ));
        assert!(
            location_reaction(parent, OBJID_WINDOW.0, parent, root, None, || Some(after)),
            "a parent whose last rectangle is unknown is re-placed"
        );
    }

    #[test]
    fn the_watch_reacts_to_a_move_of_the_root() {
        // WorkerWChild: the surface lives in the WorkerW, the event names Progman, its root.
        let (parent, root) = (handle(0x10), handle(0x20));
        let before = test_rect(-200, 0, 200, 100);
        let after = test_rect(0, 0, 400, 100);
        assert!(location_reaction(
            root,
            OBJID_WINDOW.0,
            parent,
            root,
            Some(before),
            || Some(after)
        ));
    }

    #[test]
    fn the_watch_ignores_a_move_that_changed_nothing() {
        let (parent, root) = (handle(0x10), handle(0x20));
        let same = test_rect(-200, 0, 200, 100);
        assert!(!location_reaction(
            parent,
            OBJID_WINDOW.0,
            parent,
            root,
            Some(same),
            || Some(same)
        ));
    }

    #[test]
    fn the_watch_ignores_another_window() {
        let (parent, root) = (handle(0x10), handle(0x20));
        let asked = Cell::new(false);
        let reacted = location_reaction(handle(0x30), OBJID_WINDOW.0, parent, root, None, || {
            asked.set(true);
            Some(test_rect(0, 0, 400, 100))
        });
        assert!(!reacted);
        assert!(
            !asked.get(),
            "no window is queried for an event that is not about the parent"
        );
    }

    #[test]
    fn the_watch_ignores_other_objects_of_the_window() {
        let (parent, root) = (handle(0x10), handle(0x20));
        let after = test_rect(0, 0, 400, 100);
        for object in [OBJID_CLIENT.0, OBJID_CARET.0, OBJID_CURSOR.0] {
            assert!(
                !location_reaction(parent, object, parent, root, None, || Some(after)),
                "object id {object}"
            );
        }
    }

    #[test]
    fn the_watch_ignores_a_parent_that_cannot_be_read() {
        let (parent, root) = (handle(0x10), handle(0x20));
        assert!(!location_reaction(parent, OBJID_WINDOW.0, parent, root, None, || None));
    }

    #[test]
    fn the_watch_re_places_after_its_parent_moved() {
        // Both windows stay hidden: the parent is never shown, and a child of a hidden window shows nothing.
        let parent = create_window().expect("the parent window");
        let child = create_window().expect("the child window");
        let move_parent = |rect: RectPx| {
            // SAFETY: plain handle and integers; SWP_NOZORDER makes the null insert-after harmless, nothing is shown.
            unsafe {
                SetWindowPos(
                    parent,
                    None,
                    rect.left,
                    rect.top,
                    rect.width(),
                    rect.height(),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
            }
            .expect("move the parent");
        };
        let target = test_rect(0, 0, 200, 100);
        let hook = HWINEVENTHOOK(0x2000 as *mut c_void);
        move_parent(test_rect(-200, 0, 200, 100));
        reparent(child, parent).expect("reparent the child");
        let first = place_checked(child, parent, target);
        WATCHED.with(|watched| {
            watched.borrow_mut().push(Watched {
                hook,
                surface: child,
                parent,
                root: parent,
                target,
                parent_rect: first.as_ref().ok().map(|p| p.parent),
                outside: false,
            });
        });

        // The shell moves the parent, which drags the child off its target.
        move_parent(test_rect(0, 0, 400, 100));
        let dragged = window_rect(child);
        // An event about another window does nothing; the parent's event puts the child back.
        // SAFETY: the callback only reads this thread's registry and the windows named above.
        unsafe { location_watch_proc(hook, EVENT_OBJECT_LOCATIONCHANGE, child, OBJID_WINDOW.0, 0, 0, 0) };
        let ignored = window_rect(child);
        // SAFETY: as above.
        unsafe { location_watch_proc(hook, EVENT_OBJECT_LOCATIONCHANGE, parent, OBJID_WINDOW.0, 0, 0, 0) };
        let after = read_placement(child, parent, target);
        let recorded = WATCHED.with(|watched| {
            let mut watched = watched.borrow_mut();
            let recorded = watched.iter().find(|e| e.hook == hook).and_then(|e| e.parent_rect);
            watched.retain(|e| e.hook != hook);
            recorded
        });
        // Destroyed the way `Drop` does, so that the window procedure posts no quit to this thread.
        for window in [child, parent] {
            DROPPING.with(|dropping| dropping.borrow_mut().push(window.0 as isize));
            // SAFETY: a window created above by this thread.
            let _ = unsafe { DestroyWindow(window) };
            DROPPING.with(|dropping| dropping.borrow_mut().retain(|&handle| handle != window.0 as isize));
        }

        assert_eq!(first.expect("the first placement").verdict, PlacementVerdict::Shown);
        assert_eq!(
            dragged,
            Some(test_rect(200, 0, 400, 100)),
            "the parent's move drags the child along"
        );
        assert_eq!(ignored, dragged, "an event about another window re-places nothing");
        let after = after.expect("the placement after the event");
        assert_eq!(after.actual, target, "the child is back on its target");
        assert_eq!(after.verdict, PlacementVerdict::Shown);
        assert_eq!(
            recorded,
            Some(test_rect(0, 0, 400, 100)),
            "the new parent rectangle is recorded"
        );
    }

    fn test_placement(parent: RectPx, verdict: PlacementVerdict) -> Placement {
        Placement {
            target: parent,
            local: parent,
            parent,
            actual: parent,
            virtual_screen: parent,
            verdict,
        }
    }

    fn test_entry(target: RectPx, parent_rect: RectPx) -> Watched {
        Watched {
            hook: HWINEVENTHOOK(0x3000 as *mut c_void),
            surface: handle(0x40),
            parent: handle(0x10),
            root: handle(0x20),
            target,
            parent_rect: Some(parent_rect),
            outside: false,
        }
    }

    #[test]
    fn the_watch_episode_starts_on_hidden() {
        let partly = PlacementVerdict::PartlyHidden {
            visible: test_rect(0, 0, 10, 10),
        };
        assert_eq!(outside_episode(false, PlacementVerdict::Hidden), Some(Episode::Started));
        assert_eq!(outside_episode(false, partly), Some(Episode::Started));
    }

    #[test]
    fn the_watch_episode_stays_quiet_while_outside() {
        let partly = PlacementVerdict::PartlyHidden {
            visible: test_rect(0, 0, 10, 10),
        };
        assert_eq!(outside_episode(true, PlacementVerdict::Hidden), None);
        assert_eq!(outside_episode(true, partly), None);
        assert_eq!(
            outside_episode(true, PlacementVerdict::Misplaced),
            None,
            "Misplaced changes nothing"
        );
    }

    #[test]
    fn the_watch_episode_ends_when_shown_again() {
        assert_eq!(outside_episode(true, PlacementVerdict::Shown), Some(Episode::Ended));
    }

    #[test]
    fn the_watch_episode_is_silent_while_inside() {
        assert_eq!(outside_episode(false, PlacementVerdict::Shown), None);
        assert_eq!(outside_episode(false, PlacementVerdict::Misplaced), None);
    }

    #[test]
    fn the_watch_records_the_newest_target_and_parent() {
        let (t1, p1) = (test_rect(0, 0, 200, 100), test_rect(-200, 0, 200, 100));
        let (t2, p2) = (test_rect(200, 0, 400, 100), test_rect(0, 0, 400, 100));
        let mut entry = test_entry(t1, p1);

        let started = record_placement(&mut entry, t2, &test_placement(p2, PlacementVerdict::Hidden));
        assert_eq!((entry.target, entry.parent_rect, entry.outside), (t2, Some(p2), true));
        assert_eq!(started, Some(Episode::Started));

        let quiet = record_placement(&mut entry, t2, &test_placement(p2, PlacementVerdict::Hidden));
        assert_eq!(quiet, None, "one warning per episode");

        let ended = record_placement(&mut entry, t1, &test_placement(p1, PlacementVerdict::Shown));
        assert_eq!((entry.target, entry.parent_rect, entry.outside), (t1, Some(p1), false));
        assert_eq!(ended, Some(Episode::Ended));
    }

    #[test]
    fn a_recheck_timer_fires_on_the_thread_and_clears_its_mark() {
        // A guard with handles no window has: the re-check finds the order unreadable and does nothing, which is all
        // this test needs. What it proves is that the timer fires through this thread's messages exactly once.
        let hook = HWINEVENTHOOK(0x1000 as *mut c_void);
        GUARDS.with(|guards| {
            guards.borrow_mut().push(Guarded {
                hook,
                surface: HWND(std::ptr::null_mut()),
                progman: HWND(std::ptr::null_mut()),
                budget: RestoreBudget::default(),
            });
        });

        arm_recheck(hook, Duration::from_millis(30));
        let armed = GUARDS.with(|guards| {
            guards
                .borrow()
                .iter()
                .find(|g| g.hook == hook)
                .and_then(|g| g.budget.recheck)
        });
        assert!(armed.is_some(), "the timer id is remembered");

        wait_pumping(Duration::from_millis(400));

        let cleared = GUARDS.with(|guards| {
            let mut guards = guards.borrow_mut();
            let mark = guards.iter().find(|g| g.hook == hook).map(|g| g.budget.recheck);
            guards.retain(|g| g.hook != hook);
            mark
        });
        assert_eq!(cleared, Some(None), "the re-check ran and ended its timer");
    }
}
