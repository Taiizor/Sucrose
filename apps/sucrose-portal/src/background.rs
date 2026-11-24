//! The background image layer (UI-01, UI-07): a full-window image below the page tint, fitted by the stored stretch and
//! drawn at the stored opacity.
//!
//! The file is untrusted (T-03-40): the header is read first and an image over [`MAX_SIDE`] pixels per side is refused
//! before any decoding, the decode runs on the background executor, and the result is no larger than the window needs
//! (long edge at most [`MAX_LONG_EDGE`]). An animated GIF keeps its frames only inside the hover-preview caps
//! ([`GIF_MAX_FRAMES`], [`GIF_MAX_BYTES`]); beyond them only the first frame is shown. A replaced image is released from
//! GPUI's atlas.

use std::{
    io::{self, Cursor},
    path::Path,
    sync::Arc,
};

use gpui_kit::*;
use image::{
    AnimationDecoder as _, DynamicImage, Frame, ImageDecoder as _, ImageFormat, ImageReader, Limits,
    codecs::gif::GifDecoder, imageops::FilterType,
};
use sucrose_core::settings::Stretch;

use crate::atlas;
use crate::library::thumbs::{MAX_SIDE, decode_generic, image_bytes, to_bgra};

/// The longest edge, in pixels, a decoded background image may have.
pub const MAX_LONG_EDGE: u32 = 2560;
/// The most frames an animated GIF keeps (the hover preview's cap).
pub const GIF_MAX_FRAMES: usize = 120;
/// The most decoded bytes an animated GIF keeps (the hover preview's cap).
pub const GIF_MAX_BYTES: usize = 24 * 1024 * 1024;
/// The largest image file that is read.
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
/// The most memory the generic decoder may allocate for one image.
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;
/// The extensions of the legacy file filter (the formats the image crate is built with).
const EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "tiff", "webp", "gif"];

/// Why the background image is not shown; the settings row turns it into the legacy text (UI-SPEC E9 error row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundError {
    /// The file is not there.
    Missing,
    /// Another process has the file open exclusively.
    Locked,
    /// The user may not read the file.
    Denied,
    /// The file is not an image this build reads, or it is damaged or too large.
    Unsupported,
}

impl BackgroundError {
    /// The locale key of the text the settings row shows for this error.
    pub fn text_key(self) -> &'static str {
        match self {
            Self::Missing | Self::Unsupported => "Portal.GeneralSettingPage.WindowBackdrop.ImageMissing",
            Self::Locked => "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Lock",
            Self::Denied => "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Access",
        }
    }
}

/// Which error an I/O failure on the image file is.
pub fn classify_error(error: &io::Error) -> BackgroundError {
    match error.kind() {
        io::ErrorKind::NotFound => BackgroundError::Missing,
        io::ErrorKind::PermissionDenied => BackgroundError::Denied,
        // ERROR_SHARING_VIOLATION and ERROR_LOCK_VIOLATION: another process holds the file.
        _ if cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33)) => BackgroundError::Locked,
        _ => BackgroundError::Unsupported,
    }
}

/// Whether the file name carries an extension of the legacy file filter.
pub fn supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.iter().any(|known| known.eq_ignore_ascii_case(extension)))
}

/// The pixel size to decode an `image` for a `window`, both in pixels: the smallest size that still covers the window,
/// never larger than the original, and never above [`MAX_LONG_EDGE`] on its long edge.
pub fn decode_size(image: (u32, u32), window: (u32, u32)) -> (u32, u32) {
    let (width, height) = (f64::from(image.0.max(1)), f64::from(image.1.max(1)));
    let cover = (f64::from(window.0) / width).max(f64::from(window.1) / height);
    let mut scale = cover.min(1.0);
    let long = width.max(height) * scale;
    if long > f64::from(MAX_LONG_EDGE) {
        scale *= f64::from(MAX_LONG_EDGE) / long;
    }
    let side = |value: f64| (value * scale).round().clamp(1.0, f64::from(MAX_LONG_EDGE)) as u32;
    (side(width), side(height))
}

/// Where the image is drawn inside the window area, in logical pixels from the area's top left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// The placement of an `image` (width, height) in an `area` for `stretch`: None keeps the image size centered, Fill
/// stretches to the area, Uniform contains the image and UniformToFill covers the area (the overflow is clipped).
pub fn fit(stretch: Stretch, image: (f32, f32), area: (f32, f32)) -> Placement {
    let whole = Placement {
        x: 0.0,
        y: 0.0,
        width: area.0,
        height: area.1,
    };
    if image.0 <= 0.0 || image.1 <= 0.0 {
        return whole;
    }
    let scale = match stretch {
        Stretch::Fill => return whole,
        Stretch::None => 1.0,
        Stretch::Uniform => (area.0 / image.0).min(area.1 / image.1),
        Stretch::UniformToFill => (area.0 / image.0).max(area.1 / image.1),
    };
    let (width, height) = (image.0 * scale, image.1 * scale);
    Placement {
        x: (area.0 - width) / 2.0,
        y: (area.1 - height) / 2.0,
        width,
        height,
    }
}

/// The alpha of the image layer for a stored opacity of 0 to 100, or `None` when the layer is not drawn at all.
pub fn layer_alpha(opacity: u8) -> Option<f32> {
    match opacity {
        0 => None,
        100.. => Some(1.0),
        percent => Some(f32::from(percent) / 100.0),
    }
}

/// A decoded background image, ready for GPUI.
pub struct Decoded {
    /// What is drawn: every frame of an animated GIF, otherwise the one frame.
    pub shown: Arc<RenderImage>,
    /// The first frame alone, kept only for an animated image so it can stand still while the window is not visible.
    pub still: Option<Arc<RenderImage>>,
}

/// Reads and decodes the image file at `path` for a window of `window` pixels.
pub fn decode_file(path: &Path, window: (u32, u32)) -> Result<Decoded, BackgroundError> {
    if !supported_extension(path) {
        return Err(BackgroundError::Unsupported);
    }
    let metadata = std::fs::metadata(path).map_err(|error| classify_error(&error))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(BackgroundError::Unsupported);
    }
    let bytes = std::fs::read(path).map_err(|error| classify_error(&error))?;
    decode_bytes(&bytes, window)
}

/// Decodes image `bytes` for a window of `window` pixels.
pub fn decode_bytes(bytes: &[u8], window: (u32, u32)) -> Result<Decoded, BackgroundError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| BackgroundError::Unsupported)?;
    let format = reader.format();
    let (width, height) = reader.into_dimensions().map_err(|_| BackgroundError::Unsupported)?;
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(BackgroundError::Unsupported);
    }
    let target = decode_size((width, height), window);
    if format == Some(ImageFormat::Gif) {
        return decode_gif(bytes, target);
    }
    let image = match decode_jpeg_scaled(bytes, target) {
        Some(image) => image,
        None => decode_generic(bytes).map_err(|_| BackgroundError::Unsupported)?,
    };
    let image = if (image.width(), image.height()) == target {
        image
    } else {
        image.resize_exact(target.0, target.1, FilterType::Triangle)
    };
    let frame = Frame::new(to_bgra(image.into_rgba8()));
    Ok(Decoded {
        shown: Arc::new(RenderImage::new(vec![frame])),
        still: None,
    })
}

/// A JPEG decoded straight at 1/2, 1/4 or 1/8 size, so a large photo never exists in memory at full size. `None` when it
/// is not a JPEG this decoder handles; the generic decoder then takes over.
fn decode_jpeg_scaled(bytes: &[u8], target: (u32, u32)) -> Option<DynamicImage> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    let (width, height) = decoder
        .scale(u16::try_from(target.0).ok()?, u16::try_from(target.1).ok()?)
        .ok()?;
    let pixels = decoder.decode().ok()?;
    let (width, height) = (u32::from(width), u32::from(height));
    match decoder.info()?.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            image::RgbImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgb8)
        }
        jpeg_decoder::PixelFormat::L8 => {
            image::GrayImage::from_raw(width, height, pixels).map(DynamicImage::ImageLuma8)
        }
        _ => None,
    }
}

/// A GIF at `target` pixels: every frame while they fit the caps, otherwise the first frame only.
fn decode_gif(bytes: &[u8], target: (u32, u32)) -> Result<Decoded, BackgroundError> {
    let mut decoder = GifDecoder::new(Cursor::new(bytes)).map_err(|_| BackgroundError::Unsupported)?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    decoder.set_limits(limits).map_err(|_| BackgroundError::Unsupported)?;
    let resize = |frame: Frame| {
        let delay = frame.delay();
        let buffer = image::imageops::resize(frame.buffer(), target.0, target.1, FilterType::Triangle);
        Frame::from_parts(to_bgra(buffer), 0, 0, delay)
    };
    let mut frames = decoder.into_frames();
    let first = frames
        .next()
        .and_then(Result::ok)
        .map(resize)
        .ok_or(BackgroundError::Unsupported)?;
    let frame_bytes = image_bytes(target.0, target.1, 1);
    let mut kept = vec![first.clone()];
    for next in frames {
        // A damaged tail keeps the frames before it.
        let Ok(frame) = next else { break };
        if kept.len() >= GIF_MAX_FRAMES || frame_bytes.saturating_mul(kept.len() + 1) > GIF_MAX_BYTES {
            // Over the caps: the first frame alone is shown.
            kept.truncate(1);
            break;
        }
        kept.push(resize(frame));
    }
    if kept.len() > 1 {
        Ok(Decoded {
            shown: Arc::new(RenderImage::new(kept)),
            still: Some(Arc::new(RenderImage::new(vec![first]))),
        })
    } else {
        Ok(Decoded {
            shown: Arc::new(RenderImage::new(kept)),
            still: None,
        })
    }
}

/// The background image layer: holds the decoded image, the stored stretch and opacity, and the last error.
pub struct BackgroundImage {
    path: String,
    stretch: Stretch,
    opacity: u8,
    loaded: Option<Decoded>,
    error: Option<BackgroundError>,
    /// Counts the decodes started; a result of an older one is dropped.
    generation: u64,
    /// Whether a decode is running.
    decoding: bool,
    /// Whether frames drawn for the window are shown: a GIF stands still while they are not.
    visible: bool,
    _decode: Option<Task<()>>,
    _visibility: Subscription,
}

impl BackgroundImage {
    /// A layer that draws nothing until an image is set.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let this = cx.weak_entity();
        let visibility = window.observe_window_visibility(move |visibility, _, cx| {
            this.update(cx, |this, cx| {
                this.visible = visibility.is_visible();
                cx.notify();
            })
            .ok();
        });
        Self {
            path: String::new(),
            stretch: Stretch::default(),
            opacity: 100,
            loaded: None,
            error: None,
            generation: 0,
            decoding: false,
            visible: window.is_visible(),
            _decode: None,
            _visibility: visibility,
        }
    }

    /// Shows the image at `path` (empty: none) with `stretch` and `opacity`. The decode runs on the background
    /// executor and the layer stays empty until it finishes; the previous image is released when the path changes.
    pub fn set(&mut self, path: &str, stretch: Stretch, opacity: u8, window: &mut Window, cx: &mut Context<Self>) {
        self.stretch = stretch;
        self.opacity = opacity;
        let unchanged = self.path == path && (self.loaded.is_some() || self.decoding);
        if !unchanged {
            path.clone_into(&mut self.path);
            self.generation += 1;
            self.decoding = false;
            self._decode = None;
            self.error = None;
            self.release(cx);
            if !path.is_empty() {
                self.start_decode(window, cx);
            }
        }
        cx.notify();
    }

    /// Changes only the fit.
    pub fn set_stretch(&mut self, stretch: Stretch, cx: &mut Context<Self>) {
        self.stretch = stretch;
        cx.notify();
    }

    /// Changes only the layer opacity (0 to 100).
    pub fn set_opacity(&mut self, opacity: u8, cx: &mut Context<Self>) {
        self.opacity = opacity;
        cx.notify();
    }

    /// Why the image is not shown, for the settings row; `None` while it is fine or not set.
    pub fn error(&self) -> Option<BackgroundError> {
        self.error
    }

    /// Whether an image is on screen (so the page tint must let it show through).
    pub fn is_active(&self) -> bool {
        self.loaded.is_some() && layer_alpha(self.opacity).is_some()
    }

    fn start_decode(&mut self, window: &Window, cx: &mut Context<Self>) {
        let size = window.viewport_size();
        let scale = window.scale_factor();
        // Whole device pixels of the window; the truncation is the intent.
        let pixels = |logical: Pixels| (f32::from(logical) * scale).round().max(1.0) as u32;
        let target = (pixels(size.width), pixels(size.height));
        let path = std::path::PathBuf::from(&self.path);
        let generation = self.generation;
        self.decoding = true;
        let work = cx
            .background_executor()
            .spawn(async move { decode_file(&path, target) });
        self._decode = Some(cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| this.finish(generation, result, cx)).ok();
        }));
    }

    fn finish(&mut self, generation: u64, result: Result<Decoded, BackgroundError>, cx: &mut Context<Self>) {
        if generation != self.generation {
            // The path changed while this decode ran: nothing of it was drawn.
            if let Ok(decoded) = result {
                Self::drop_decoded(decoded, cx);
            }
            return;
        }
        self.decoding = false;
        match result {
            Ok(decoded) => {
                tracing::debug!(frames = decoded.shown.frame_count(), "background image decoded");
                self.loaded = Some(decoded);
            }
            Err(error) => {
                tracing::warn!(?error, "the background image cannot be shown, the setting is kept");
                self.error = Some(error);
            }
        }
        cx.notify();
    }

    /// Drops the shown image and frees it from GPUI's atlas.
    fn release(&mut self, cx: &mut App) {
        if let Some(decoded) = self.loaded.take() {
            Self::drop_decoded(decoded, cx);
        }
    }

    fn drop_decoded(decoded: Decoded, cx: &mut App) {
        atlas::release_images(cx, std::iter::once(decoded.shown).chain(decoded.still));
    }
}

impl Render for BackgroundImage {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let layer = div().absolute().inset_0().overflow_hidden();
        let (Some(alpha), Some(loaded)) = (layer_alpha(self.opacity), &self.loaded) else {
            return layer;
        };
        // A GIF stands still on its first frame while no frame is shown.
        let source = if self.visible {
            Arc::clone(&loaded.shown)
        } else {
            Arc::clone(loaded.still.as_ref().unwrap_or(&loaded.shown))
        };
        let scale = window.scale_factor();
        let size = source.size(0);
        let image = (size.width.0 as f32 / scale, size.height.0 as f32 / scale);
        let area = window.viewport_size();
        let placement = fit(self.stretch, image, (f32::from(area.width), f32::from(area.height)));
        layer.opacity(alpha).child(
            img(source)
                .absolute()
                .left(px(placement.x))
                .top(px(placement.y))
                .w(px(placement.width))
                .h(px(placement.height))
                .object_fit(ObjectFit::Fill),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{io, io::Cursor, path::Path};

    use gpui_kit::{
        ImageSource, IntoElement as _, ParentElement as _, RenderImage, Styled as _, TestAppContext, div, img, point,
        px, size,
    };
    use image::{
        Delay, Frame, ImageEncoder as _, Rgba, RgbaImage,
        codecs::{gif::GifEncoder, jpeg::JpegEncoder, png::PngEncoder},
    };
    use std::sync::Arc;
    use sucrose_core::settings::Stretch;

    // Not `use super::*`: the module glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{
        BackgroundError, BackgroundImage, Decoded, GIF_MAX_FRAMES, MAX_LONG_EDGE, Placement, classify_error,
        decode_bytes, decode_file, decode_size, fit, layer_alpha, supported_extension,
    };

    const AREA: (f32, f32) = (1000.0, 500.0);

    #[test]
    fn background_fit_none_keeps_the_size_centered() {
        let placement = fit(Stretch::None, (200.0, 100.0), AREA);
        assert_eq!(
            placement,
            Placement {
                x: 400.0,
                y: 200.0,
                width: 200.0,
                height: 100.0
            }
        );
    }

    #[test]
    fn background_fit_fill_stretches_to_the_area() {
        let placement = fit(Stretch::Fill, (200.0, 100.0), AREA);
        assert_eq!(
            (placement.x, placement.y, placement.width, placement.height),
            (0.0, 0.0, 1000.0, 500.0)
        );
    }

    #[test]
    fn background_fit_uniform_contains_the_image() {
        // 4:3 inside 2:1: the height limits, the sides are empty.
        let placement = fit(Stretch::Uniform, (400.0, 300.0), AREA);
        assert!((placement.width - 666.666_7).abs() < 0.01 && placement.height == 500.0);
        assert!((placement.x - 166.666_7).abs() < 0.01 && placement.y == 0.0);
    }

    #[test]
    fn background_fit_uniform_to_fill_covers_the_area() {
        let placement = fit(Stretch::UniformToFill, (400.0, 300.0), AREA);
        assert_eq!((placement.width, placement.height), (1000.0, 750.0));
        assert_eq!((placement.x, placement.y), (0.0, -125.0));
    }

    #[test]
    fn background_fit_of_an_empty_image_is_the_area() {
        for stretch in Stretch::ALL {
            let placement = fit(*stretch, (0.0, 0.0), AREA);
            assert_eq!((placement.width, placement.height), AREA, "{stretch:?}");
        }
    }

    #[test]
    fn background_decode_size_stays_inside_the_window_and_the_long_edge_cap() {
        let (width, height) = decode_size((6000, 4000), (1800, 1200));
        assert!(width <= 1800 && height <= 1200, "{width} x {height}");
        assert_eq!((width, height), (1800, 1200));
        // A 4K window is bounded by the 2560 cap.
        let (width, height) = decode_size((8000, 4000), (7680, 4320));
        assert!(width.max(height) <= MAX_LONG_EDGE, "{width} x {height}");
        assert_eq!((width, height), (2560, 1280));
    }

    #[test]
    fn background_decode_size_never_enlarges() {
        assert_eq!(decode_size((800, 600), (1920, 1080)), (800, 600));
        assert_eq!(decode_size((0, 0), (1920, 1080)), (1, 1));
    }

    #[test]
    fn background_opacity_maps_to_the_layer_alpha() {
        assert_eq!(layer_alpha(0), None);
        assert_eq!(layer_alpha(100), Some(1.0));
        assert_eq!(layer_alpha(55), Some(0.55));
        assert_eq!(layer_alpha(200), Some(1.0));
    }

    #[test]
    fn background_errors_follow_the_io_failure() {
        assert_eq!(
            classify_error(&io::Error::from(io::ErrorKind::NotFound)),
            BackgroundError::Missing
        );
        assert_eq!(
            classify_error(&io::Error::from(io::ErrorKind::PermissionDenied)),
            BackgroundError::Denied
        );
        assert_eq!(
            classify_error(&io::Error::from(io::ErrorKind::InvalidData)),
            BackgroundError::Unsupported
        );
    }

    #[cfg(windows)]
    #[test]
    fn background_a_sharing_violation_is_locked() {
        assert_eq!(
            classify_error(&io::Error::from_raw_os_error(32)),
            BackgroundError::Locked
        );
        assert_eq!(
            classify_error(&io::Error::from_raw_os_error(33)),
            BackgroundError::Locked
        );
    }

    #[test]
    fn background_an_unknown_extension_is_unsupported_and_a_missing_file_is_missing() {
        let unknown = decode_file(Path::new("picture.xyz"), (100, 100));
        assert_eq!(unknown.err(), Some(BackgroundError::Unsupported));
        let missing = decode_file(&std::env::temp_dir().join("sucrose-no-such-background.png"), (100, 100));
        assert_eq!(missing.err(), Some(BackgroundError::Missing));
        assert!(supported_extension(Path::new("A.JPG")) && supported_extension(Path::new("b.webp")));
        assert!(!supported_extension(Path::new("noextension")));
    }

    #[test]
    fn background_error_texts_are_localized_keys_of_the_settings_row() {
        assert!(BackgroundError::Denied.text_key().ends_with("BackgroundImage.Access"));
        assert!(BackgroundError::Locked.text_key().ends_with("BackgroundImage.Lock"));
        assert!(BackgroundError::Missing.text_key().ends_with("ImageMissing"));
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = RgbaImage::from_pixel(width, height, Rgba([200, 30, 30, 255]));
        let mut bytes = Vec::new();
        PngEncoder::new(Cursor::new(&mut bytes))
            .write_image(&image, width, height, image::ExtendedColorType::Rgba8)
            .expect("encode png");
        bytes
    }

    fn gif(frames: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut bytes);
            for step in 0..frames {
                let shade = u8::try_from(step % 250).unwrap_or(0);
                let image = RgbaImage::from_pixel(8, 8, Rgba([shade, 0, 0, 255]));
                encoder
                    .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(40, 1)))
                    .expect("encode frame");
            }
        }
        bytes
    }

    #[test]
    fn background_a_large_png_is_decoded_no_larger_than_the_window() {
        let decoded = decode_bytes(&png(3000, 2000), (900, 600)).expect("decodes");
        let size = decoded.shown.size(0);
        assert_eq!((size.width.0, size.height.0), (900, 600));
        assert_eq!(decoded.shown.frame_count(), 1);
        assert!(decoded.still.is_none());
    }

    #[test]
    fn background_a_jpeg_is_decoded_at_the_window_size() {
        let image = RgbaImage::from_pixel(1200, 800, Rgba([10, 120, 200, 255]));
        let rgb = image::DynamicImage::ImageRgba8(image).into_rgb8();
        let mut bytes = Vec::new();
        JpegEncoder::new(&mut bytes)
            .write_image(&rgb, 1200, 800, image::ExtendedColorType::Rgb8)
            .expect("encode jpeg");
        let decoded = decode_bytes(&bytes, (300, 200)).expect("decodes");
        let size = decoded.shown.size(0);
        assert_eq!((size.width.0, size.height.0), (300, 200));
    }

    #[test]
    fn background_a_short_gif_keeps_its_frames_and_a_still() {
        let decoded = decode_bytes(&gif(3), (100, 100)).expect("decodes");
        assert_eq!(decoded.shown.frame_count(), 3);
        assert_eq!(decoded.still.expect("a still frame").frame_count(), 1);
    }

    #[test]
    fn background_a_gif_over_the_frame_cap_shows_only_its_first_frame() {
        let decoded = decode_bytes(&gif(GIF_MAX_FRAMES + 1), (100, 100)).expect("decodes");
        assert_eq!(decoded.shown.frame_count(), 1);
        assert!(decoded.still.is_none());
        let at_cap = decode_bytes(&gif(GIF_MAX_FRAMES), (100, 100)).expect("decodes");
        assert_eq!(at_cap.shown.frame_count(), GIF_MAX_FRAMES);
    }

    #[test]
    fn background_something_that_is_not_an_image_is_unsupported() {
        assert_eq!(
            decode_bytes(b"not an image at all", (100, 100)).err(),
            Some(BackgroundError::Unsupported)
        );
    }

    #[gpui_kit::test]
    fn background_release_inside_a_window_update_empties_the_atlas(cx: &mut TestAppContext) {
        let frame = || Frame::new(RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255])));
        let shown = Arc::new(RenderImage::new(vec![frame(), frame()]));
        let still = Arc::new(RenderImage::new(vec![frame()]));
        let decoded = Decoded {
            shown: shown.clone(),
            still: Some(still.clone()),
        };
        let window = cx.add_empty_window();
        window.draw(point(px(0.), px(0.)), size(px(80.), px(40.)), |_, _| {
            div()
                .child(img(ImageSource::Render(shown.clone())).size(px(40.)))
                .child(img(ImageSource::Render(still.clone())).size(px(40.)))
                .into_any_element()
        });
        window.update(|window, _| {
            assert!(window.has_image_atlas_entry(&still), "the still image was drawn");
        });
        window.update(|_, cx| BackgroundImage::drop_decoded(decoded, cx));
        window.run_until_parked();
        window.update(|window, _| {
            assert!(!window.has_image_atlas_entry(&shown), "the shown image left the atlas");
            assert!(!window.has_image_atlas_entry(&still), "the still image left the atlas");
        });
    }
}
