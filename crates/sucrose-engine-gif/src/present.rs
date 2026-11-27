//! Showing a decoded frame. [`Presenter`] is what the engine's pump needs. On Windows the frame goes to a child window of
//! the desktop surface through Direct2D ([`D2dPresenter`], research Finding 4, Pitfall 12, A15) and, where Direct2D
//! cannot draw, through GDI `StretchDIBits` ([`GdiPresenter`]). [`Fallback`] chooses between them and carries the logic
//! of the switch, which is pure and tested without a window. Both draw the rectangles of [`crate::fit`].

use sucrose_core::settings::Stretch;

use crate::decode::DecodedFrame;

/// How many times in a row a Direct2D render target may be lost before drawing moves to GDI (T-04-68): the third loss
/// in a row switches, a success in between starts the count again.
pub const LOSSES_BEFORE_FALLBACK: u32 = 3;

/// `D2DERR_RECREATE_TARGET`, named in the log when it causes the switch.
const RECREATE_TARGET_CODE: u32 = 0x8899_000C;

/// Why a frame could not be shown.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PresentError {
    /// The device behind the render target was lost (`D2DERR_RECREATE_TARGET`); the presenter made the target again and
    /// the next frame is tried on it.
    #[error("the render target was lost and made again")]
    DeviceLost,
    /// Anything else, with a sentence saying what; the next frame is tried again.
    #[error("{0}")]
    Failed(String),
}

/// Shows frames on the surface. It belongs to the thread that owns the surface.
pub trait Presenter {
    /// Shows `frame` placed by the stretch mode and keeps it, so a resize or a new mode can draw it again.
    ///
    /// # Errors
    /// [`PresentError`]; the next frame is tried again.
    fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError>;

    /// The surface changed size: the window and the render target follow it and the kept frame is drawn again.
    fn resize(&mut self, width: u32, height: u32);

    /// The stretch mode changed: the kept frame is drawn again with the new rectangles.
    fn set_stretch(&mut self, stretch: Stretch);

    /// A copy of the frame on screen, for the presenter that takes over from this one.
    fn kept_frame(&self) -> Option<DecodedFrame> {
        None
    }

    /// The presenter's name for the log.
    fn name(&self) -> &'static str;
}

/// What a presenter needs to start where another one stopped.
#[derive(Debug, Clone)]
pub struct Handoff {
    /// The surface size in pixels.
    pub size: (u32, u32),
    /// The stretch mode in force.
    pub stretch: Stretch,
    /// The frame on screen, drawn at once.
    pub frame: Option<DecodedFrame>,
}

/// Builds a presenter from a [`Handoff`].
pub type Factory = Box<dyn FnOnce(Handoff) -> Result<Box<dyn Presenter>, String>>;

/// A presenter that starts on a primary one (Direct2D) and moves to a standby one (GDI) when the primary cannot be
/// created or loses its render target [`LOSSES_BEFORE_FALLBACK`] times in a row. It never moves back.
pub struct Fallback {
    active: Box<dyn Presenter>,
    standby: Option<Factory>,
    lost_in_a_row: u32,
    size: (u32, u32),
    stretch: Stretch,
}

impl std::fmt::Debug for Fallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fallback")
            .field("active", &self.active.name())
            .field("lost_in_a_row", &self.lost_in_a_row)
            .finish_non_exhaustive()
    }
}

impl Fallback {
    /// Opens the `primary` presenter, or the `standby` one when the primary cannot be created.
    ///
    /// # Errors
    /// Both reasons, when neither presenter could be created.
    pub fn open(size: (u32, u32), stretch: Stretch, primary: Factory, standby: Factory) -> Result<Self, String> {
        let handoff = || Handoff {
            size,
            stretch,
            frame: None,
        };
        match primary(handoff()) {
            Ok(active) => Ok(Self {
                active,
                standby: Some(standby),
                lost_in_a_row: 0,
                size,
                stretch,
            }),
            Err(reason) => match standby(handoff()) {
                Ok(active) => {
                    tracing::warn!("gif presenter={} reason={reason}", active.name());
                    Ok(Self {
                        active,
                        standby: None,
                        lost_in_a_row: 0,
                        size,
                        stretch,
                    })
                }
                Err(second) => Err(format!("{reason}; the fallback failed too: {second}")),
            },
        }
    }

    /// Moves drawing to the standby presenter, which starts with the frame on screen. False when there is none or it
    /// cannot be created; the active presenter stays then.
    fn switch(&mut self, reason: &str) -> bool {
        let Some(standby) = self.standby.take() else {
            return false;
        };
        let handoff = Handoff {
            size: self.size,
            stretch: self.stretch,
            frame: self.active.kept_frame(),
        };
        match standby(handoff) {
            Ok(next) => {
                tracing::warn!("gif presenter={} reason={reason}", next.name());
                // The old presenter goes after the new one exists, so the picture is never missing.
                self.active = next;
                true
            }
            Err(error) => {
                tracing::error!(%error, "the GDI presenter cannot be created, {} stays", self.active.name());
                false
            }
        }
    }
}

impl Presenter for Fallback {
    fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError> {
        match self.active.present(frame) {
            Ok(()) => {
                self.lost_in_a_row = 0;
                Ok(())
            }
            Err(PresentError::DeviceLost) => {
                self.lost_in_a_row += 1;
                if self.lost_in_a_row >= LOSSES_BEFORE_FALLBACK {
                    let reason = format!(
                        "the render target was lost {} times in a row (D2DERR_RECREATE_TARGET {RECREATE_TARGET_CODE:#010x})",
                        self.lost_in_a_row
                    );
                    if self.switch(&reason) {
                        self.lost_in_a_row = 0;
                        return Ok(());
                    }
                }
                Err(PresentError::DeviceLost)
            }
            Err(other) => Err(other),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        self.active.resize(width, height);
    }

    fn set_stretch(&mut self, stretch: Stretch) {
        self.stretch = stretch;
        self.active.set_stretch(stretch);
    }

    fn kept_frame(&self) -> Option<DecodedFrame> {
        self.active.kept_frame()
    }

    fn name(&self) -> &'static str {
        self.active.name()
    }
}

/// The rectangles (x, y, width, height) of a `surface` that the `dest` rectangle leaves uncovered, which GDI has to
/// paint black itself (Direct2D clears the whole target first). Empty ones are left out.
#[must_use]
pub fn bars(surface: (i32, i32), dest: (i32, i32, i32, i32)) -> Vec<(i32, i32, i32, i32)> {
    let (x, y, width, height) = dest;
    let (right, bottom) = (x + width, y + height);
    [
        (0, 0, surface.0, y),
        (0, bottom, surface.0, surface.1 - bottom),
        (0, y, x, height),
        (right, y, surface.0 - right, height),
    ]
    .into_iter()
    .filter(|(_, _, w, h)| *w > 0 && *h > 0)
    .collect()
}

/// Direct2D with a GDI fallback on a child window of `parent` (the surface, a window of the calling thread).
///
/// # Errors
/// The reasons when neither Direct2D nor GDI could create a window to draw on.
#[cfg(windows)]
pub fn open(parent: u64, width: u32, height: u32, stretch: Stretch) -> Result<Fallback, String> {
    let size = (width.max(1), height.max(1));
    Fallback::open(
        size,
        stretch,
        Box::new(move |handoff| {
            D2dPresenter::new(parent, handoff.size.0, handoff.size.1, handoff.stretch)
                .map(|presenter| Box::new(presenter) as Box<dyn Presenter>)
        }),
        Box::new(move |handoff| {
            GdiPresenter::new(parent, handoff.size.0, handoff.size.1, handoff.stretch, handoff.frame)
                .map(|presenter| Box::new(presenter) as Box<dyn Presenter>)
        }),
    )
}

#[cfg(windows)]
pub use d2d::D2dPresenter;
#[cfg(windows)]
pub use gdi::GdiPresenter;

/// The child window both presenters draw on.
#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    use windows::{
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, RegisterClassExW, WM_NCHITTEST, WNDCLASSEXW, WS_CHILD,
                WS_CLIPSIBLINGS, WS_EX_NOACTIVATE, WS_VISIBLE,
            },
        },
        core::{HRESULT, w},
    };

    /// `ERROR_CLASS_ALREADY_EXISTS`.
    const CLASS_EXISTS: u32 = 1410;

    /// `HTTRANSPARENT`: the child never takes the mouse; the desktop behind it does.
    const HIT_TRANSPARENT: isize = -1;

    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if message == WM_NCHITTEST {
            return LRESULT(HIT_TRANSPARENT);
        }
        // SAFETY: the arguments are the ones the system passed to this procedure, forwarded unchanged.
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }

    /// The surface window for its handle as the IPC carries it.
    pub(super) fn parent_window(parent: u64) -> HWND {
        HWND(parent as usize as *mut c_void)
    }

    /// A visible child window of `parent` at 0,0 that lets the mouse through.
    pub(super) fn create_window(parent: HWND, width: u32, height: u32) -> Result<HWND, String> {
        // SAFETY: a null module name returns the handle of this executable.
        let module = unsafe { GetModuleHandleW(None) }.map_err(|error| format!("GetModuleHandleW: {error}"))?;
        let instance = HINSTANCE(module.0);
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: w!("SucroseGifPresenter"),
            ..Default::default()
        };
        // SAFETY: `class` is fully initialized; its procedure and class name live for the whole process.
        if unsafe { RegisterClassExW(&class) } == 0 {
            let error = windows::core::Error::from_thread();
            if error.code() != HRESULT::from_win32(CLASS_EXISTS) {
                return Err(format!("RegisterClassExW: {error}"));
            }
        }
        let width = i32::try_from(width).unwrap_or(i32::MAX);
        let height = i32::try_from(height).unwrap_or(i32::MAX);
        // SAFETY: the class was registered above and the strings are valid; `parent` is the surface window, owned by
        // this thread, and the child is destroyed before it (see the presenters' `Drop`).
        unsafe {
            CreateWindowExW(
                WS_EX_NOACTIVATE,
                w!("SucroseGifPresenter"),
                w!("Sucrose GIF"),
                WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                0,
                0,
                width,
                height,
                Some(parent),
                None,
                Some(instance),
                None,
            )
        }
        .map_err(|error| format!("CreateWindowExW: {error}"))
    }
}

/// The Direct2D presenter.
#[cfg(windows)]
mod d2d {
    use std::ffi::c_void;

    use sucrose_core::settings::Stretch;
    use windows::{
        Win32::{
            Foundation::HWND,
            Graphics::{
                Direct2D::{
                    Common::{D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES, D2D1_FACTORY_TYPE_SINGLE_THREADED,
                    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
                    D2D1CreateFactory, ID2D1Bitmap, ID2D1Factory, ID2D1HwndRenderTarget,
                },
                Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            },
            UI::WindowsAndMessaging::{DestroyWindow, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos},
        },
        core::HRESULT,
    };

    use super::{PresentError, Presenter, win};
    use crate::{decode::DecodedFrame, fit};

    /// `D2DERR_RECREATE_TARGET`: the device was lost and the render target must be made again.
    const RECREATE_TARGET: HRESULT = HRESULT(0x8899_000C_u32 as i32);

    /// A child window of the surface with a Direct2D render target and one reused bitmap.
    pub struct D2dPresenter {
        window: HWND,
        factory: ID2D1Factory,
        target: ID2D1HwndRenderTarget,
        bitmap: Option<ID2D1Bitmap>,
        /// The size of the frame in `bitmap`.
        frame_size: (u32, u32),
        /// The frame on screen: the source of a redraw after a resize or a lost device.
        last: Option<DecodedFrame>,
        size: (u32, u32),
        stretch: Stretch,
    }

    impl std::fmt::Debug for D2dPresenter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D2dPresenter")
                .field("size", &self.size)
                .field("stretch", &self.stretch)
                .finish_non_exhaustive()
        }
    }

    fn target_size(width: u32, height: u32) -> D2D_SIZE_U {
        D2D_SIZE_U { width, height }
    }

    fn rect(rect: fit::Rect) -> D2D_RECT_F {
        D2D_RECT_F {
            left: rect[0],
            top: rect[1],
            right: rect[2],
            bottom: rect[3],
        }
    }

    fn create_target(factory: &ID2D1Factory, window: HWND, size: (u32, u32)) -> Result<ID2D1HwndRenderTarget, String> {
        // 96 dpi: one device-independent unit is one physical pixel, so the surface rectangle is used as it is.
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            dpiX: 96.0,
            dpiY: 96.0,
            ..Default::default()
        };
        let window_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: window,
            pixelSize: target_size(size.0, size.1),
            presentOptions: D2D1_PRESENT_OPTIONS_NONE,
        };
        // SAFETY: both property structs are valid for the call and `window` is a live window of this thread.
        unsafe { factory.CreateHwndRenderTarget(&properties, &window_properties) }.map_err(|error| {
            format!(
                "Direct2D cannot create the render target ({:#010x}): {error}",
                error.code().0
            )
        })
    }

    impl D2dPresenter {
        /// Creates the child window of `parent` (the surface, a window of the calling thread) at 0,0 with the given
        /// size and a render target on it, and clears it to black.
        ///
        /// # Errors
        /// A sentence naming the call that failed and its HRESULT.
        pub fn new(parent: u64, width: u32, height: u32, stretch: Stretch) -> Result<Self, String> {
            let size = (width.max(1), height.max(1));
            let window = win::create_window(win::parent_window(parent), size.0, size.1)?;
            // SAFETY: plain call; the out interface is returned by value.
            let factory: Result<ID2D1Factory, _> =
                unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) };
            let built = factory
                .map_err(|error| format!("Direct2D cannot create its factory ({:#010x}): {error}", error.code().0))
                .and_then(|factory| create_target(&factory, window, size).map(|target| (factory, target)));
            match built {
                Ok((factory, target)) => {
                    let mut presenter = Self {
                        window,
                        factory,
                        target,
                        bitmap: None,
                        frame_size: (1, 1),
                        last: None,
                        size,
                        stretch,
                    };
                    presenter.draw();
                    Ok(presenter)
                }
                Err(reason) => {
                    // SAFETY: the window was created above by this thread and is destroyed exactly once.
                    let _ = unsafe { DestroyWindow(window) };
                    Err(reason)
                }
            }
        }

        /// One draw of the kept frame over a black clear, with the rectangles of the stretch mode.
        fn draw_once(&mut self) -> windows::core::Result<()> {
            let (dest, src) = fit::fit(self.stretch, self.frame_size, self.size);
            let (dest, src) = (rect(dest), rect(src));
            let black = D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            // SAFETY: the target belongs to this thread; the pointers passed are valid for each call, and the bitmap
            // was created by this target.
            unsafe {
                self.target.BeginDraw();
                self.target.Clear(Some(&black));
                if let Some(bitmap) = &self.bitmap {
                    self.target.DrawBitmap(
                        bitmap,
                        Some(&dest),
                        1.0,
                        D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                        Some(&src),
                    );
                }
                self.target.EndDraw(None, None)
            }
        }

        /// Draws the kept frame; a lost device is recovered by making the target again. True when it was lost.
        fn draw(&mut self) -> bool {
            match self.draw_once() {
                Ok(()) => false,
                Err(error) if error.code() == RECREATE_TARGET => {
                    tracing::warn!("the Direct2D device was lost, the render target is made again");
                    self.recreate();
                    true
                }
                Err(error) => {
                    tracing::warn!(%error, "Direct2D could not finish drawing a frame");
                    false
                }
            }
        }

        /// Makes the render target and the bitmap again from the kept frame.
        fn recreate(&mut self) {
            self.bitmap = None;
            match create_target(&self.factory, self.window, self.size) {
                Ok(target) => {
                    self.target = target;
                    if let Some(frame) = self.last.take() {
                        if let Err(reason) = self.upload(&frame) {
                            tracing::warn!(%reason, "cannot upload the frame to the new render target");
                        }
                        self.last = Some(frame);
                        if let Err(error) = self.draw_once() {
                            tracing::warn!(%error, "Direct2D could not draw after making the render target again");
                        }
                    }
                }
                Err(reason) => tracing::warn!(%reason, "cannot make the render target again"),
            }
        }

        /// Copies `frame` into the reused bitmap, making a new one when the size changed.
        fn upload(&mut self, frame: &DecodedFrame) -> Result<(), String> {
            let pitch = frame.width.saturating_mul(4);
            if frame.pixels.len() < pitch as usize * frame.height as usize {
                return Err("the frame holds fewer bytes than its size says".to_owned());
            }
            let source = frame.pixels.as_ptr().cast::<c_void>();
            if let Some(bitmap) = &self.bitmap {
                // SAFETY: `size` of the bitmap is read from the bitmap itself, and the pixel buffer was checked above
                // to hold at least pitch * height bytes for the frame size that is compared first.
                let same = unsafe { bitmap.GetPixelSize() };
                if same.width == frame.width && same.height == frame.height {
                    // SAFETY: `source` points at `pitch * height` bytes (checked above), the whole bitmap is
                    // written (a null destination rectangle) and the bitmap has the frame's size.
                    return unsafe { bitmap.CopyFromMemory(None, source, pitch) }
                        .map_err(|error| format!("CopyFromMemory: {error}"));
                }
            }
            let properties = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
            };
            // SAFETY: `source` points at `pitch * height` bytes (checked above) that Direct2D copies during the call.
            let bitmap = unsafe {
                self.target
                    .CreateBitmap(target_size(frame.width, frame.height), Some(source), pitch, &properties)
            }
            .map_err(|error| format!("CreateBitmap ({:#010x}): {error}", error.code().0))?;
            self.bitmap = Some(bitmap);
            self.frame_size = (frame.width, frame.height);
            Ok(())
        }
    }

    impl Presenter for D2dPresenter {
        fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError> {
            self.upload(&frame).map_err(PresentError::Failed)?;
            self.last = Some(frame);
            if self.draw() {
                Err(PresentError::DeviceLost)
            } else {
                Ok(())
            }
        }

        fn resize(&mut self, width: u32, height: u32) {
            let size = (width.max(1), height.max(1));
            if size == self.size {
                return;
            }
            self.size = size;
            let (cx, cy) = (
                i32::try_from(size.0).unwrap_or(i32::MAX),
                i32::try_from(size.1).unwrap_or(i32::MAX),
            );
            // SAFETY: the window belongs to this thread; it stays at 0,0 of the surface and only changes size.
            let moved = unsafe { SetWindowPos(self.window, None, 0, 0, cx, cy, SWP_NOZORDER | SWP_NOACTIVATE) };
            if let Err(error) = moved {
                tracing::warn!(%error, "cannot resize the GIF window");
            }
            let pixel_size = target_size(size.0, size.1);
            // SAFETY: `pixel_size` is valid for the call and the target belongs to this thread.
            if let Err(error) = unsafe { self.target.Resize(&pixel_size) } {
                tracing::warn!(%error, "cannot resize the Direct2D render target");
            }
            self.draw();
        }

        fn set_stretch(&mut self, stretch: Stretch) {
            if stretch != self.stretch {
                self.stretch = stretch;
                self.draw();
            }
        }

        fn kept_frame(&self) -> Option<DecodedFrame> {
            self.last.clone()
        }

        fn name(&self) -> &'static str {
            "Direct2D"
        }
    }

    impl Drop for D2dPresenter {
        fn drop(&mut self) {
            // The Direct2D objects go before the window they draw on.
            self.bitmap = None;
            // SAFETY: the window was created by this thread (the presenter is not Send) and is destroyed exactly once;
            // a failure at shutdown leaves nothing to act on.
            let _ = unsafe { DestroyWindow(self.window) };
        }
    }
}

/// The GDI presenter, for a system where Direct2D cannot draw.
#[cfg(windows)]
mod gdi {
    use sucrose_core::settings::Stretch;
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLACKNESS, DIB_RGB_COLORS, GetDC, HALFTONE, PatBlt, ReleaseDC,
            SRCCOPY, SetStretchBltMode, StretchDIBits,
        },
        UI::WindowsAndMessaging::{DestroyWindow, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos},
    };

    use super::{PresentError, Presenter, bars, win};
    use crate::{decode::DecodedFrame, fit};

    /// A child window of the surface drawn with `StretchDIBits` on its device context. The window is drawn on every
    /// frame and after every resize or stretch change; it does not repaint itself.
    pub struct GdiPresenter {
        window: HWND,
        size: (u32, u32),
        stretch: Stretch,
        last: Option<DecodedFrame>,
    }

    impl std::fmt::Debug for GdiPresenter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("GdiPresenter")
                .field("size", &self.size)
                .field("stretch", &self.stretch)
                .finish_non_exhaustive()
        }
    }

    impl GdiPresenter {
        /// Creates the child window of `parent` at 0,0 and draws `frame` (or black) on it.
        ///
        /// # Errors
        /// A sentence naming the call that failed.
        pub fn new(
            parent: u64,
            width: u32,
            height: u32,
            stretch: Stretch,
            frame: Option<DecodedFrame>,
        ) -> Result<Self, String> {
            let size = (width.max(1), height.max(1));
            let window = win::create_window(win::parent_window(parent), size.0, size.1)?;
            let mut presenter = Self {
                window,
                size,
                stretch,
                last: frame,
            };
            presenter.draw();
            Ok(presenter)
        }

        /// Paints the kept frame with the stretch rectangles and the rest black.
        fn draw(&mut self) {
            // SAFETY: the window is alive and belongs to this thread; the device context is released below.
            let hdc = unsafe { GetDC(Some(self.window)) };
            if hdc.is_invalid() {
                tracing::warn!("GDI cannot get the device context of the GIF window");
                return;
            }
            let surface = (
                i32::try_from(self.size.0).unwrap_or(i32::MAX),
                i32::try_from(self.size.1).unwrap_or(i32::MAX),
            );
            let black = |x: i32, y: i32, width: i32, height: i32| {
                // SAFETY: `hdc` is a live device context of this window for the whole draw.
                let _ = unsafe { PatBlt(hdc, x, y, width, height, BLACKNESS) };
            };
            match &self.last {
                None => black(0, 0, surface.0, surface.1),
                Some(frame) if frame.pixels.len() < frame.width as usize * frame.height as usize * 4 => {
                    black(0, 0, surface.0, surface.1);
                }
                Some(frame) => {
                    let (dest, src) = fit::fit(self.stretch, (frame.width, frame.height), self.size);
                    let (dest, src) = (fit::pixels(dest), fit::pixels(src));
                    for (x, y, width, height) in bars(surface, dest) {
                        black(x, y, width, height);
                    }
                    let info = BITMAPINFO {
                        bmiHeader: BITMAPINFOHEADER {
                            biSize: size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: i32::try_from(frame.width).unwrap_or(i32::MAX),
                            // Negative: the rows run top to bottom, as the decoder wrote them.
                            biHeight: -i32::try_from(frame.height).unwrap_or(i32::MAX),
                            biPlanes: 1,
                            biBitCount: 32,
                            biCompression: BI_RGB.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    // SAFETY: `hdc` is live; the pixel buffer holds at least width * height * 4 bytes (checked in the
                    // match arm above) and the header describes exactly that top-down 32 bit image; the source
                    // rectangle lies inside it because `fit` clips it to the frame.
                    unsafe {
                        SetStretchBltMode(hdc, HALFTONE);
                        StretchDIBits(
                            hdc,
                            dest.0,
                            dest.1,
                            dest.2,
                            dest.3,
                            src.0,
                            src.1,
                            src.2,
                            src.3,
                            Some(frame.pixels.as_ptr().cast()),
                            &info,
                            DIB_RGB_COLORS,
                            SRCCOPY,
                        );
                    }
                }
            }
            // SAFETY: `hdc` came from `GetDC` of this window above and is released exactly once.
            unsafe { ReleaseDC(Some(self.window), hdc) };
        }
    }

    impl Presenter for GdiPresenter {
        fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError> {
            if frame.pixels.len() < frame.width as usize * frame.height as usize * 4 {
                return Err(PresentError::Failed(
                    "the frame holds fewer bytes than its size says".to_owned(),
                ));
            }
            self.last = Some(frame);
            self.draw();
            Ok(())
        }

        fn resize(&mut self, width: u32, height: u32) {
            let size = (width.max(1), height.max(1));
            if size == self.size {
                return;
            }
            self.size = size;
            let (cx, cy) = (
                i32::try_from(size.0).unwrap_or(i32::MAX),
                i32::try_from(size.1).unwrap_or(i32::MAX),
            );
            // SAFETY: the window belongs to this thread; it stays at 0,0 of the surface and only changes size.
            let moved = unsafe { SetWindowPos(self.window, None, 0, 0, cx, cy, SWP_NOZORDER | SWP_NOACTIVATE) };
            if let Err(error) = moved {
                tracing::warn!(%error, "cannot resize the GIF window");
            }
            self.draw();
        }

        fn set_stretch(&mut self, stretch: Stretch) {
            if stretch != self.stretch {
                self.stretch = stretch;
                self.draw();
            }
        }

        fn kept_frame(&self) -> Option<DecodedFrame> {
            self.last.clone()
        }

        fn name(&self) -> &'static str {
            "Gdi"
        }
    }

    impl Drop for GdiPresenter {
        fn drop(&mut self) {
            // SAFETY: the window was created by this thread (the presenter is not Send) and is destroyed exactly once;
            // a failure at shutdown leaves nothing to act on.
            let _ = unsafe { DestroyWindow(self.window) };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;

    fn frame() -> DecodedFrame {
        DecodedFrame {
            pixels: vec![0; 4],
            width: 1,
            height: 1,
            delay_ms: 10,
        }
    }

    #[derive(Default)]
    struct Seen {
        /// What each double was asked, as "name: what".
        events: Mutex<Vec<String>>,
        /// The handoff the standby double was built from.
        handoff: Mutex<Option<Handoff>>,
    }

    struct Double {
        name: &'static str,
        script: VecDeque<Result<(), PresentError>>,
        kept: Option<DecodedFrame>,
        seen: Arc<Seen>,
    }

    impl Presenter for Double {
        fn present(&mut self, frame: DecodedFrame) -> Result<(), PresentError> {
            self.kept = Some(frame);
            self.seen
                .events
                .lock()
                .expect("lock")
                .push(format!("{}: present", self.name));
            self.script.pop_front().unwrap_or(Ok(()))
        }

        fn resize(&mut self, width: u32, height: u32) {
            self.seen
                .events
                .lock()
                .expect("lock")
                .push(format!("{}: resize {width}x{height}", self.name));
        }

        fn set_stretch(&mut self, stretch: Stretch) {
            self.seen
                .events
                .lock()
                .expect("lock")
                .push(format!("{}: stretch {stretch:?}", self.name));
        }

        fn kept_frame(&self) -> Option<DecodedFrame> {
            self.kept.clone()
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    fn primary(script: Vec<Result<(), PresentError>>, seen: &Arc<Seen>) -> Factory {
        let seen = Arc::clone(seen);
        Box::new(move |_| {
            Ok(Box::new(Double {
                name: "Direct2D",
                script: script.into(),
                kept: None,
                seen,
            }) as Box<dyn Presenter>)
        })
    }

    fn standby(seen: &Arc<Seen>) -> Factory {
        let seen = Arc::clone(seen);
        Box::new(move |handoff| {
            *seen.handoff.lock().expect("lock") = Some(handoff.clone());
            Ok(Box::new(Double {
                name: "Gdi",
                script: VecDeque::new(),
                kept: handoff.frame,
                seen,
            }) as Box<dyn Presenter>)
        })
    }

    fn lost() -> Result<(), PresentError> {
        Err(PresentError::DeviceLost)
    }

    #[test]
    fn direct2d_is_chosen_when_its_target_can_be_created() {
        let seen = Arc::new(Seen::default());
        let presenter =
            Fallback::open((100, 50), Stretch::Uniform, primary(vec![], &seen), standby(&seen)).expect("open");
        assert_eq!(presenter.name(), "Direct2D");
        assert!(seen.handoff.lock().expect("lock").is_none(), "the standby is not built");
    }

    #[test]
    fn gdi_is_chosen_when_direct2d_cannot_create_its_target() {
        let seen = Arc::new(Seen::default());
        let broken: Factory = Box::new(|_| Err("Direct2D cannot create the render target (0x88990012)".to_owned()));
        let presenter = Fallback::open((100, 50), Stretch::Fill, broken, standby(&seen)).expect("open");
        assert_eq!(presenter.name(), "Gdi");
        let handoff = seen.handoff.lock().expect("lock").clone().expect("built");
        assert_eq!((handoff.size, handoff.stretch), ((100, 50), Stretch::Fill));
    }

    #[test]
    fn opening_fails_with_both_reasons_when_neither_presenter_can_be_created() {
        let first: Factory = Box::new(|_| Err("no Direct2D".to_owned()));
        let second: Factory = Box::new(|_| Err("no window".to_owned()));
        let reason = Fallback::open((1, 1), Stretch::None, first, second).expect_err("both fail");
        assert!(
            reason.contains("no Direct2D") && reason.contains("no window"),
            "{reason}"
        );
    }

    #[test]
    fn the_third_loss_in_a_row_moves_drawing_to_gdi_with_the_frame_and_the_settings() {
        let seen = Arc::new(Seen::default());
        let mut presenter = Fallback::open(
            (640, 360),
            Stretch::UniformToFill,
            primary(vec![lost(), lost(), lost()], &seen),
            standby(&seen),
        )
        .expect("open");
        presenter.resize(800, 450);
        presenter.set_stretch(Stretch::Uniform);

        assert_eq!(presenter.present(frame()), Err(PresentError::DeviceLost));
        assert_eq!(presenter.name(), "Direct2D");
        assert_eq!(presenter.present(frame()), Err(PresentError::DeviceLost));
        assert_eq!(presenter.name(), "Direct2D", "two losses do not switch");
        assert_eq!(
            presenter.present(frame()),
            Ok(()),
            "the frame is shown by the new presenter"
        );
        assert_eq!(presenter.name(), "Gdi");

        let handoff = seen.handoff.lock().expect("lock").clone().expect("built");
        assert_eq!((handoff.size, handoff.stretch), ((800, 450), Stretch::Uniform));
        assert_eq!(handoff.frame, Some(frame()), "the frame on screen goes along");

        // GDI is not asked to switch back, and further frames go to it.
        presenter.resize(1024, 576);
        assert_eq!(presenter.present(frame()), Ok(()));
        let events = seen.events.lock().expect("lock").clone();
        assert!(events.contains(&"Gdi: resize 1024x576".to_owned()));
        assert_eq!(events.last().map(String::as_str), Some("Gdi: present"));
    }

    #[test]
    fn a_success_in_between_starts_the_count_again() {
        let seen = Arc::new(Seen::default());
        let script = vec![lost(), lost(), Ok(()), lost(), lost(), Ok(())];
        let mut presenter =
            Fallback::open((10, 10), Stretch::Fill, primary(script, &seen), standby(&seen)).expect("open");
        for _ in 0..6 {
            let _ = presenter.present(frame());
        }
        assert_eq!(presenter.name(), "Direct2D");
        assert!(seen.handoff.lock().expect("lock").is_none());
    }

    #[test]
    fn another_failure_neither_counts_nor_switches() {
        let seen = Arc::new(Seen::default());
        let failed = || Err(PresentError::Failed("CreateBitmap".to_owned()));
        let script = vec![failed(), failed(), failed(), failed()];
        let mut presenter =
            Fallback::open((10, 10), Stretch::Fill, primary(script, &seen), standby(&seen)).expect("open");
        for _ in 0..4 {
            assert!(matches!(presenter.present(frame()), Err(PresentError::Failed(_))));
        }
        assert_eq!(presenter.name(), "Direct2D");
    }

    #[test]
    fn a_standby_that_cannot_be_created_leaves_direct2d_drawing() {
        let seen = Arc::new(Seen::default());
        let broken: Factory = Box::new(|_| Err("no window".to_owned()));
        let mut presenter = Fallback::open(
            (10, 10),
            Stretch::Fill,
            primary(vec![lost(), lost(), lost(), lost()], &seen),
            broken,
        )
        .expect("open");
        for _ in 0..4 {
            assert_eq!(presenter.present(frame()), Err(PresentError::DeviceLost));
        }
        assert_eq!(presenter.name(), "Direct2D");
    }

    #[test]
    fn the_bars_are_what_the_destination_leaves_of_the_surface() {
        assert_eq!(
            bars((1920, 1080), (420, 0, 1080, 1080)),
            [(0, 0, 420, 1080), (1500, 0, 420, 1080)]
        );
        assert_eq!(bars((1920, 1080), (0, 0, 1920, 1080)), []);
        assert_eq!(
            bars((1920, 1080), (800, 450, 320, 180)),
            [
                (0, 0, 1920, 450),
                (0, 630, 1920, 450),
                (0, 450, 800, 180),
                (1120, 450, 800, 180)
            ]
        );
    }
}
