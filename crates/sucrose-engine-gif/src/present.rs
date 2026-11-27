//! Showing a decoded frame. [`Presenter`] is what the engine's pump needs; [`d2d`] is the Windows implementation: a
//! Direct2D render target on a child window of the desktop surface (research Finding 4, Pitfall 12, A15). 04-31 adds the
//! stretch rectangles and a GDI fallback presenter behind the same trait.

use crate::decode::DecodedFrame;

/// Shows frames on the surface. It belongs to the thread that owns the surface.
pub trait Presenter {
    /// Shows `frame` filling the surface and keeps it, so a resize can draw it again.
    ///
    /// # Errors
    /// A sentence saying why the frame could not be shown; the next frame is tried again.
    fn present(&mut self, frame: DecodedFrame) -> Result<(), String>;

    /// The surface changed size: the window and the render target follow it and the kept frame is drawn again.
    fn resize(&mut self, width: u32, height: u32);

    /// The presenter's name for the log.
    fn name(&self) -> &'static str;
}

#[cfg(windows)]
pub use d2d::D2dPresenter;

/// The Direct2D presenter.
#[cfg(windows)]
mod d2d {
    use std::ffi::c_void;

    use windows::{
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
            Graphics::{
                Direct2D::{
                    Common::{D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES, D2D1_FACTORY_TYPE_SINGLE_THREADED,
                    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
                    D2D1CreateFactory, ID2D1Bitmap, ID2D1Factory, ID2D1HwndRenderTarget,
                },
                Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            },
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, SWP_NOACTIVATE, SWP_NOZORDER,
                SetWindowPos, WM_NCHITTEST, WNDCLASSEXW, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE, WS_VISIBLE,
            },
        },
        core::{HRESULT, w},
    };

    use super::Presenter;
    use crate::decode::DecodedFrame;

    /// `D2DERR_RECREATE_TARGET`: the device was lost and the render target must be made again.
    const RECREATE_TARGET: HRESULT = HRESULT(0x8899_000C_u32 as i32);

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

    /// A child window of the surface with a Direct2D render target and one reused bitmap.
    pub struct D2dPresenter {
        window: HWND,
        factory: ID2D1Factory,
        target: ID2D1HwndRenderTarget,
        bitmap: Option<ID2D1Bitmap>,
        /// The frame on screen: the source of a redraw after a resize or a lost device.
        last: Option<DecodedFrame>,
        size: (u32, u32),
    }

    impl std::fmt::Debug for D2dPresenter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("D2dPresenter")
                .field("size", &self.size)
                .finish_non_exhaustive()
        }
    }

    fn target_size(width: u32, height: u32) -> D2D_SIZE_U {
        D2D_SIZE_U { width, height }
    }

    fn create_window(parent: HWND, width: u32, height: u32) -> Result<HWND, String> {
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
        // this thread, and the child is destroyed before it (see `Drop`).
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
        pub fn new(parent: u64, width: u32, height: u32) -> Result<Self, String> {
            let parent = HWND(parent as usize as *mut c_void);
            let size = (width.max(1), height.max(1));
            let window = create_window(parent, size.0, size.1)?;
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
                        last: None,
                        size,
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

        /// One draw of the kept frame over a black clear.
        fn draw_once(&mut self) -> windows::core::Result<()> {
            let rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: self.size.0 as f32,
                bottom: self.size.1 as f32,
            };
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
                    self.target
                        .DrawBitmap(bitmap, Some(&rect), 1.0, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, None);
                }
                self.target.EndDraw(None, None)
            }
        }

        /// Draws the kept frame; a lost device is recovered once.
        fn draw(&mut self) {
            if let Err(error) = self.draw_once() {
                if error.code() == RECREATE_TARGET {
                    tracing::warn!("the Direct2D device was lost, the render target is made again");
                    self.recreate();
                } else {
                    tracing::warn!(%error, "Direct2D could not finish drawing a frame");
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
            Ok(())
        }
    }

    impl Presenter for D2dPresenter {
        fn present(&mut self, frame: DecodedFrame) -> Result<(), String> {
            self.upload(&frame)?;
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
            let pixel_size = target_size(size.0, size.1);
            // SAFETY: `pixel_size` is valid for the call and the target belongs to this thread.
            if let Err(error) = unsafe { self.target.Resize(&pixel_size) } {
                tracing::warn!(%error, "cannot resize the Direct2D render target");
            }
            self.draw();
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
