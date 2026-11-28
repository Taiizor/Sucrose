//! Lazy, bounded animation decoding of GIF, APNG and animated WebP. Pure code, the same on every OS.
//!
//! The format is told by the first bytes of the file, never by its extension. An [`Animation`] decodes one frame at a time from its source, scales a frame down to fit the bound it was opened
//! with (never up) and converts it to premultiplied BGRA once, so the presenter copies it as it is. Nothing here
//! keeps a frame after it was handed out: the memory of a playing animation is a few frames, whatever its length
//! (T-04-66). A file that is not a supported animation, or is damaged, is a [`DecodeError`] with a reason, never a
//! panic (T-04-67).

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
};

use image::{
    AnimationDecoder, Delay, DynamicImage, Frame, Frames, ImageDecoder, Limits, RgbaImage,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
    imageops::{self, FilterType},
};

/// The delay of a frame whose own delay is 0, as the legacy XamlAnimatedGif library has it (milliseconds).
pub const DEFAULT_DELAY_MS: u32 = 100;

/// The largest animation canvas side the decoder accepts. A hostile header cannot make it allocate more (T-04-66).
const MAX_SIDE: u32 = 16_384;

/// The most memory the decoder may allocate for one canvas (T-04-66).
const MAX_ALLOC: u64 = 256 * 1024 * 1024;

/// The most pixels a source canvas may have (8192 x 8192). A larger source is refused from its header, before any frame
/// is decoded (T-04-69).
pub const MAX_SOURCE_PIXELS: u64 = 8192 * 8192;

/// Why an animation could not be decoded. One reason per value, shown in the log and the tray.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    /// The file could not be read.
    #[error("the animation file cannot be read: {0}")]
    Io(String),
    /// The file is not an animation format this renderer knows.
    #[error("the file is not a GIF, APNG or WebP animation")]
    NotAnimation,
    /// The picture is larger than [`MAX_SOURCE_PIXELS`] allows.
    #[error("the picture is {width} x {height} pixels, above the limit of 8192 x 8192")]
    TooLarge {
        /// Width of the canvas.
        width: u32,
        /// Height of the canvas.
        height: u32,
    },
    /// The file is damaged or too large for the decoder's limits.
    #[error("the animation is damaged or too large: {0}")]
    Malformed(String),
    /// The file holds no frame.
    #[error("the animation has no frames")]
    NoFrames,
}

/// A reader the decoder can take: buffered and seekable.
trait ReadSeek: BufRead + Seek {}
impl<T: BufRead + Seek> ReadSeek for T {}

/// Where an animation is read from. Cloneable, so the decoder can reopen it to loop.
#[derive(Debug, Clone)]
pub enum Source {
    /// A file on disk.
    Path(PathBuf),
    /// Bytes in memory (tests).
    #[cfg(test)]
    Memory(std::sync::Arc<[u8]>),
}

impl Source {
    fn open(&self) -> Result<Box<dyn ReadSeek>, DecodeError> {
        match self {
            Self::Path(path) => {
                let file = File::open(path).map_err(|error| DecodeError::Io(error.to_string()))?;
                Ok(Box::new(BufReader::new(file)))
            }
            #[cfg(test)]
            Self::Memory(bytes) => Ok(Box::new(std::io::Cursor::new(std::sync::Arc::clone(bytes)))),
        }
    }
}

impl From<&Path> for Source {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_owned())
    }
}

/// One decoded frame, ready to present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFrame {
    /// Premultiplied BGRA, `width * height * 4` bytes, rows without padding.
    pub pixels: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// How long the frame stays on screen, in milliseconds (never 0).
    pub delay_ms: u32,
}

/// The format a file turned out to be, decided by its bytes (never by the extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A GIF.
    Gif,
    /// A PNG with an `acTL` chunk (animated).
    Apng,
    /// A PNG without one: a single picture.
    Png,
    /// A WebP, animated (`ANIM`) or a single picture.
    WebP,
}

/// What the first bytes of a file say it is, before the decoder looks closer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sniffed {
    Gif,
    Png,
    WebP,
    Unknown,
}

fn sniff(head: &[u8]) -> Sniffed {
    if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Sniffed::Gif
    } else if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Sniffed::Png
    } else if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Sniffed::WebP
    } else {
        Sniffed::Unknown
    }
}

/// How many bytes at the start of a file [`declared_size`] reads.
const HEAD_LEN: usize = 32;

/// The canvas size a file declares in its first bytes, when they hold it: the GIF logical screen, the PNG header, the
/// WebP extended header or the first bitstream header. `None` when the head is too short or says nothing about it.
fn declared_size(format: Sniffed, head: &[u8]) -> Option<(u32, u32)> {
    let le16 = |at: usize| {
        head.get(at..at + 2)
            .map(|bytes| u32::from(u16::from_le_bytes([bytes[0], bytes[1]])))
    };
    let le24 = |at: usize| {
        head.get(at..at + 3)
            .map(|b| u32::from(b[0]) | u32::from(b[1]) << 8 | u32::from(b[2]) << 16)
    };
    match format {
        Sniffed::Gif => Some((le16(6)?, le16(8)?)),
        Sniffed::Png if head.get(12..16) == Some(b"IHDR") => {
            let be32 = |at: usize| {
                head.get(at..at + 4)
                    .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            };
            Some((be32(16)?, be32(20)?))
        }
        Sniffed::WebP => match head.get(12..16)? {
            b"VP8X" => Some((le24(24)? + 1, le24(27)? + 1)),
            b"VP8L" if head.get(20) == Some(&0x2f) => {
                let bits = head.get(21..25).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))?;
                Some(((bits & 0x3FFF) + 1, (bits >> 14 & 0x3FFF) + 1))
            }
            b"VP8 " => Some((le16(26)? & 0x3FFF, le16(28)? & 0x3FFF)),
            _ => None,
        },
        _ => None,
    }
}

/// Refuses a canvas of `width` x `height` above [`MAX_SOURCE_PIXELS`].
fn check_size(width: u32, height: u32) -> Result<(), DecodeError> {
    if u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS {
        return Err(DecodeError::TooLarge { width, height });
    }
    Ok(())
}

/// The decoder limits every format gets.
fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

/// A single picture as a one-frame animation, decoded when the frame is asked for. Its delay is 0, so it gets the
/// default one like any frame without a delay.
fn still(decoder: impl ImageDecoder + 'static) -> Frames<'static> {
    Frames::new(Box::new(std::iter::once_with(move || {
        DynamicImage::from_decoder(decoder)
            .map(|picture| Frame::from_parts(picture.into_rgba8(), 0, 0, Delay::from_numer_denom_ms(0, 1)))
    })))
}

/// The size a `width` by `height` frame is scaled to so it fits `max`: the same size when it already fits, never
/// larger, aspect ratio kept, never below one pixel.
#[must_use]
pub fn fit(width: u32, height: u32, max: (u32, u32)) -> (u32, u32) {
    let (max_w, max_h) = (max.0.max(1), max.1.max(1));
    if width <= max_w && height <= max_h {
        return (width.max(1), height.max(1));
    }
    let scale = (f64::from(max_w) / f64::from(width)).min(f64::from(max_h) / f64::from(height));
    let scaled = |side: u32| ((f64::from(side) * scale).round() as u32).max(1);
    (scaled(width).min(max_w), scaled(height).min(max_h))
}

/// How a decoded frame is prepared for the surface it is bound to (chosen from the stretch mode, see `fit::scaling`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scaling {
    /// Scaled down, aspect kept, until it fits the bound; never up.
    #[default]
    Within,
    /// Scaled down, aspect kept, only until it still covers the bound; never up.
    Cover,
    /// Never scaled: only the centre that fits the bound is kept.
    Crop,
}

/// The size a `width` by `height` frame is scaled to for `max` under `scaling`. A cropped frame keeps its scale: the
/// size returned is the size of the centre that stays.
#[must_use]
pub fn scaled_size(width: u32, height: u32, max: (u32, u32), scaling: Scaling) -> (u32, u32) {
    let (max_w, max_h) = (max.0.max(1), max.1.max(1));
    match scaling {
        Scaling::Within => fit(width, height, max),
        Scaling::Crop => (width.clamp(1, max_w), height.clamp(1, max_h)),
        Scaling::Cover => {
            let scale = (f64::from(max_w) / f64::from(width.max(1))).max(f64::from(max_h) / f64::from(height.max(1)));
            if scale >= 1.0 {
                return (width.max(1), height.max(1));
            }
            let scaled = |side: u32, floor: u32| ((f64::from(side) * scale).round() as u32).max(floor.min(side)).max(1);
            (scaled(width, max_w), scaled(height, max_h))
        }
    }
}

/// A frame before scaling: the composed canvas and its delay.
struct RawFrame {
    image: RgbaImage,
    delay_ms: u32,
}

impl RawFrame {
    fn new(image: RgbaImage, delay_ms: u32) -> Self {
        #[cfg(test)]
        live::born();
        Self { image, delay_ms }
    }
}

#[cfg(test)]
impl Drop for RawFrame {
    fn drop(&mut self) {
        live::died();
    }
}

/// Counts the decoded canvases alive on this thread, so a test can see that a long animation is never in memory as a
/// whole (T-04-66).
#[cfg(test)]
pub(crate) mod live {
    use std::cell::Cell;

    thread_local! {
        static NOW: Cell<usize> = const { Cell::new(0) };
        static PEAK: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) fn born() {
        NOW.with(|now| {
            now.set(now.get() + 1);
            PEAK.with(|peak| peak.set(peak.get().max(now.get())));
        });
    }

    pub(super) fn died() {
        NOW.with(|now| now.set(now.get().saturating_sub(1)));
    }

    /// Starts counting from nothing.
    pub(crate) fn reset() {
        NOW.with(|now| now.set(0));
        PEAK.with(|peak| peak.set(0));
    }

    /// The most canvases that were alive together since [`reset`].
    pub(crate) fn peak() -> usize {
        PEAK.with(Cell::get)
    }
}

/// An animation being decoded one frame at a time. It is not `Send` (the frame iterator is not): the thread that
/// decodes opens it.
pub struct Animation {
    source: Source,
    max: (u32, u32),
    scaling: Scaling,
    frames: image::Frames<'static>,
    format: Format,
    /// The size of the canvas, before any scaling.
    canvas: (u32, u32),
    /// The first frame, decoded by `open` so that a damaged file fails there and not mid-wallpaper.
    pending: Option<RawFrame>,
    /// What came after the first frame, looked at by `open` to know whether the file is a single picture.
    ahead: Option<Result<Option<RawFrame>, DecodeError>>,
    /// The file holds exactly one frame: it is shown once and needs no frame timer.
    single: bool,
}

impl std::fmt::Debug for Animation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animation")
            .field("source", &self.source)
            .field("max", &self.max)
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
}

impl Animation {
    /// Opens the animation file `path` for a surface of at most `max` pixels and decodes its first frame.
    ///
    /// # Errors
    /// A [`DecodeError`] when the file cannot be read, is not a supported animation, is damaged or holds no frame.
    pub fn open(path: &Path, max: (u32, u32)) -> Result<Self, DecodeError> {
        Self::open_source(Source::from(path), max)
    }

    /// [`Animation::open`] for any [`Source`].
    ///
    /// # Errors
    /// See [`Animation::open`].
    pub fn open_source(source: Source, max: (u32, u32)) -> Result<Self, DecodeError> {
        Self::open_scaled(source, max, Scaling::Within)
    }

    /// [`Animation::open_source`] with the frames prepared under `scaling` instead of fitted.
    ///
    /// # Errors
    /// See [`Animation::open`].
    pub fn open_scaled(source: Source, max: (u32, u32), scaling: Scaling) -> Result<Self, DecodeError> {
        let mut reader = source.open()?;
        let mut head = [0_u8; HEAD_LEN];
        let mut read = 0;
        while read < head.len() {
            match reader.read(&mut head[read..]) {
                Ok(0) => break,
                Ok(count) => read += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(DecodeError::Io(error.to_string())),
            }
        }
        let sniffed = sniff(&head[..read]);
        if sniffed == Sniffed::Unknown {
            return Err(DecodeError::NotAnimation);
        }
        if let Some((width, height)) = declared_size(sniffed, &head[..read]) {
            // The size is in the header: a huge canvas is refused before the decoder is built.
            check_size(width, height)?;
        }
        reader.rewind().map_err(|error| DecodeError::Io(error.to_string()))?;
        let malformed = |error: image::ImageError| DecodeError::Malformed(error.to_string());
        let (format, canvas, mut frames) = match sniffed {
            Sniffed::Gif => {
                let mut decoder = GifDecoder::new(reader).map_err(malformed)?;
                let canvas = decoder.dimensions();
                check_size(canvas.0, canvas.1)?;
                decoder.set_limits(limits()).map_err(malformed)?;
                (Format::Gif, canvas, decoder.into_frames())
            }
            Sniffed::Png => {
                let decoder = PngDecoder::with_limits(reader, limits()).map_err(malformed)?;
                let canvas = decoder.dimensions();
                check_size(canvas.0, canvas.1)?;
                if decoder.is_apng().map_err(malformed)? {
                    (Format::Apng, canvas, decoder.apng().map_err(malformed)?.into_frames())
                } else {
                    (Format::Png, canvas, still(decoder))
                }
            }
            Sniffed::WebP => {
                let mut decoder = WebPDecoder::new(reader).map_err(malformed)?;
                let canvas = decoder.dimensions();
                check_size(canvas.0, canvas.1)?;
                decoder.set_limits(limits()).map_err(malformed)?;
                if decoder.has_animation() {
                    (Format::WebP, canvas, decoder.into_frames())
                } else {
                    (Format::WebP, canvas, still(decoder))
                }
            }
            Sniffed::Unknown => return Err(DecodeError::NotAnimation),
        };
        let first = match frames.next() {
            None => return Err(DecodeError::NoFrames),
            Some(Err(error)) => return Err(malformed(error)),
            Some(Ok(frame)) => raw(frame),
        };
        let ahead = match frames.next() {
            None => Ok(None),
            Some(Err(error)) => Err(malformed(error)),
            Some(Ok(frame)) => Ok(Some(raw(frame))),
        };
        let single = matches!(ahead, Ok(None));
        Ok(Self {
            source,
            max: (max.0.max(1), max.1.max(1)),
            scaling,
            frames,
            format,
            canvas,
            pending: Some(first),
            ahead: Some(ahead),
            single,
        })
    }

    /// The format the file turned out to be.
    #[must_use]
    pub fn format(&self) -> Format {
        self.format
    }

    /// The width and height of the canvas in the file, before any scaling.
    #[must_use]
    pub fn canvas(&self) -> (u32, u32) {
        self.canvas
    }

    /// Whether the file holds exactly one frame (a still picture or a one-frame animation): it is shown once and no
    /// frame timer is needed.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.single
    }

    fn pull(&mut self) -> Result<Option<RawFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
        }
        if let Some(ahead) = self.ahead.take() {
            return ahead;
        }
        match self.frames.next() {
            None => Ok(None),
            Some(Err(error)) => Err(DecodeError::Malformed(error.to_string())),
            Some(Ok(frame)) => Ok(Some(raw(frame))),
        }
    }

    /// The next frame, scaled to the bound and ready to present; `None` after the last one.
    ///
    /// # Errors
    /// A [`DecodeError`] when the file turns out damaged after the first frame.
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, DecodeError> {
        Ok(self.pull()?.map(|frame| finish(frame, self.max, self.scaling)))
    }

    /// Steps over up to `count` frames without scaling them; returns how many were skipped (fewer when the animation
    /// ends first).
    ///
    /// # Errors
    /// A [`DecodeError`] when the file turns out damaged.
    pub fn skip(&mut self, count: usize) -> Result<usize, DecodeError> {
        for skipped in 0..count {
            if self.pull()?.is_none() {
                return Ok(skipped);
            }
        }
        Ok(count)
    }

    /// Starts again at frame 0 by reopening the source.
    ///
    /// # Errors
    /// A [`DecodeError`] when the source cannot be opened again.
    pub fn rewind(&mut self) -> Result<(), DecodeError> {
        *self = Self::open_scaled(self.source.clone(), self.max, self.scaling)?;
        Ok(())
    }
}

fn raw(frame: image::Frame) -> RawFrame {
    let (numer, denom) = frame.delay().numer_denom_ms();
    let delay_ms = numer.checked_div(denom).unwrap_or(0);
    RawFrame::new(
        frame.into_buffer(),
        if delay_ms == 0 { DEFAULT_DELAY_MS } else { delay_ms },
    )
}

fn finish(frame: RawFrame, max: (u32, u32), scaling: Scaling) -> DecodedFrame {
    let (source_w, source_h) = (frame.image.width(), frame.image.height());
    let (width, height) = scaled_size(source_w, source_h, max, scaling);
    let prepared;
    let image = if (width, height) == (source_w, source_h) {
        &frame.image
    } else {
        prepared = if scaling == Scaling::Crop {
            imageops::crop_imm(
                &frame.image,
                (source_w - width) / 2,
                (source_h - height) / 2,
                width,
                height,
            )
            .to_image()
        } else {
            imageops::resize(&frame.image, width, height, FilterType::Triangle)
        };
        &prepared
    };
    let mut pixels = Vec::with_capacity(image.as_raw().len());
    for pixel in image.as_raw().as_chunks::<4>().0 {
        let alpha = u32::from(pixel[3]);
        let premultiply = |channel: u8| ((u32::from(channel) * alpha + 127) / 255) as u8;
        pixels.extend_from_slice(&[
            premultiply(pixel[2]),
            premultiply(pixel[1]),
            premultiply(pixel[0]),
            pixel[3],
        ]);
    }
    DecodedFrame {
        pixels,
        width,
        height,
        delay_ms: frame.delay_ms,
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use image::{
        Delay, Frame, Rgba, RgbaImage,
        codecs::gif::{GifEncoder, Repeat},
    };

    /// A GIF of solid frames: frame `i` has red `(i % 4) * 70` and green `20 + (i / 4) * 30`, so up to 32 frames have a
    /// number of their own. `delays_ms` gives one delay per frame.
    pub(crate) fn gif_bytes(width: u32, height: u32, delays_ms: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new_with_speed(&mut bytes, 10);
            encoder.set_repeat(Repeat::Infinite).expect("repeat");
            for (index, delay) in delays_ms.iter().enumerate() {
                let red = u8::try_from(index % 4 * 70).expect("below 256");
                let green = u8::try_from(20 + index / 4 * 30).expect("below 256");
                let image = RgbaImage::from_pixel(width, height, Rgba([red, green, 160, 255]));
                encoder
                    .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(*delay, 1)))
                    .expect("frame");
            }
        }
        bytes
    }

    /// The frame number a decoded frame shows, read from its red and green channels (BGRA: bytes 2 and 1).
    pub(crate) fn frame_number(pixels: &[u8]) -> u32 {
        (u32::from(pixels[2]) + 35) / 70 + 4 * ((u32::from(pixels[1]) + 15).saturating_sub(20) / 30)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{fixtures::*, *};

    fn memory(bytes: Vec<u8>) -> Source {
        Source::Memory(Arc::from(bytes))
    }

    #[test]
    fn four_frames_come_in_file_order_with_their_delays_then_none() {
        let source = memory(gif_bytes(16, 9, &[0, 30, 50, 120]));
        let mut animation = Animation::open_source(source, (16, 9)).expect("open");
        let mut seen = Vec::new();
        while let Some(frame) = animation.next_frame().expect("frame") {
            seen.push((frame_number(&frame.pixels), frame.delay_ms));
        }
        assert_eq!(seen, [(0, DEFAULT_DELAY_MS), (1, 30), (2, 50), (3, 120)]);
        assert_eq!(animation.next_frame().expect("end"), None);
    }

    #[test]
    fn rewinding_starts_at_frame_zero_again() {
        let mut animation = Animation::open_source(memory(gif_bytes(8, 8, &[10, 10, 10])), (8, 8)).expect("open");
        while animation.next_frame().expect("frame").is_some() {}
        animation.rewind().expect("rewind");
        let first = animation.next_frame().expect("frame").expect("a frame");
        assert_eq!(frame_number(&first.pixels), 0);
    }

    #[test]
    fn skipping_steps_over_frames_without_losing_the_order() {
        let mut animation = Animation::open_source(memory(gif_bytes(8, 8, &[10; 4])), (8, 8)).expect("open");
        assert_eq!(animation.skip(2).expect("skip"), 2);
        let frame = animation.next_frame().expect("frame").expect("a frame");
        assert_eq!(frame_number(&frame.pixels), 2);
        assert_eq!(animation.skip(10).expect("skip"), 1, "only frame 3 was left");
    }

    #[test]
    fn a_large_animation_is_scaled_down_to_the_surface_and_a_small_one_stays() {
        let big = Animation::open_source(memory(gif_bytes(192, 108, &[10])), (96, 54)).expect("open");
        let frame = big.into_first();
        assert_eq!((frame.width, frame.height), (96, 54));
        assert_eq!(frame.pixels.len(), 96 * 54 * 4);

        let small = Animation::open_source(memory(gif_bytes(32, 18, &[10])), (96, 54)).expect("open");
        let frame = small.into_first();
        assert_eq!((frame.width, frame.height), (32, 18), "never scaled up");
    }

    #[test]
    fn fit_keeps_the_aspect_ratio_and_never_goes_below_one_pixel() {
        assert_eq!(fit(1920, 1080, (960, 540)), (960, 540));
        assert_eq!(fit(1000, 1000, (960, 540)), (540, 540));
        assert_eq!(fit(320, 180, (960, 540)), (320, 180));
        assert_eq!(fit(10_000, 10, (100, 100)), (100, 1));
    }

    #[test]
    fn cover_keeps_the_frame_as_large_as_the_surface_needs_and_crop_keeps_only_the_centre() {
        assert_eq!(scaled_size(1000, 1000, (960, 540), Scaling::Cover), (960, 960));
        assert_eq!(scaled_size(4000, 3000, (1920, 1080), Scaling::Cover), (1920, 1440));
        assert_eq!(
            scaled_size(320, 180, (960, 540), Scaling::Cover),
            (320, 180),
            "never scaled up"
        );
        assert_eq!(scaled_size(3840, 2160, (1920, 1080), Scaling::Crop), (1920, 1080));
        assert_eq!(scaled_size(320, 180, (1920, 1080), Scaling::Crop), (320, 180));
        assert_eq!(scaled_size(1000, 1000, (1920, 1080), Scaling::Within), (1000, 1000));
    }

    #[test]
    fn a_cropped_frame_is_the_centre_of_the_original() {
        let wide = Animation::open_scaled(memory(gif_bytes(40, 20, &[10])), (20, 20), Scaling::Crop).expect("open");
        let frame = wide.into_first();
        assert_eq!((frame.width, frame.height), (20, 20));
    }

    #[test]
    fn pixels_are_premultiplied_bgra() {
        let frame = finish(
            RawFrame::new(RgbaImage::from_pixel(1, 1, image::Rgba([200, 100, 50, 128])), 10),
            (1, 1),
            Scaling::Within,
        );
        assert_eq!(frame.pixels, [25, 50, 100, 128]);
    }

    #[test]
    fn text_and_unknown_files_are_errors_with_a_reason() {
        for bytes in [b"hello, this is not a picture".to_vec(), Vec::new(), vec![0; 64]] {
            let error = Animation::open_source(memory(bytes), (10, 10)).expect_err("not an animation");
            assert_eq!(error, DecodeError::NotAnimation);
            assert!(!error.to_string().is_empty());
        }
    }

    /// A plain PNG of one colour, from image's encoder (no acTL chunk).
    fn still_png(width: u32, height: u32) -> Vec<u8> {
        use image::ImageEncoder;
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(
                RgbaImage::from_pixel(width, height, image::Rgba([70, 90, 160, 255])).as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .expect("png");
        bytes
    }

    /// A still lossless WebP of one colour, from image's encoder (no ANIM chunk).
    fn still_webp(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut bytes)
            .encode(
                RgbaImage::from_pixel(width, height, image::Rgba([70, 90, 160, 255])).as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .expect("webp");
        bytes
    }

    #[test]
    fn the_bytes_decide_the_format_not_the_extension() {
        let dir = std::env::temp_dir().join(format!("sucrose-gif-sniff-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let gif_named_png = dir.join("really-a-gif.png");
        std::fs::write(&gif_named_png, gif_bytes(8, 8, &[10, 10])).expect("write");
        let animation = Animation::open(&gif_named_png, (8, 8)).expect("open");
        assert_eq!(animation.format(), Format::Gif);
        assert!(!animation.is_single());
        let png_named_gif = dir.join("really-a-png.gif");
        std::fs::write(&png_named_gif, still_png(8, 8)).expect("write");
        let animation = Animation::open(&png_named_gif, (8, 8)).expect("open");
        assert_eq!(animation.format(), Format::Png);
        std::fs::remove_dir_all(&dir).expect("clean");
    }

    #[test]
    fn a_plain_png_and_a_still_webp_open_as_one_frame_that_gets_the_default_delay() {
        for (bytes, format) in [(still_png(12, 6), Format::Png), (still_webp(12, 6), Format::WebP)] {
            let mut animation = Animation::open_source(memory(bytes), (12, 6)).expect("open");
            assert_eq!(animation.format(), format);
            assert!(animation.is_single(), "{format:?}");
            assert_eq!(animation.canvas(), (12, 6));
            let frame = animation.next_frame().expect("frame").expect("a frame");
            assert_eq!((frame.width, frame.height, frame.delay_ms), (12, 6, DEFAULT_DELAY_MS));
            assert_eq!(animation.next_frame().expect("end"), None, "{format:?}");
        }
    }

    #[test]
    fn a_one_frame_gif_is_single_and_a_two_frame_gif_is_not() {
        let one = Animation::open_source(memory(gif_bytes(8, 8, &[10])), (8, 8)).expect("open");
        assert!(one.is_single());
        let two = Animation::open_source(memory(gif_bytes(8, 8, &[10, 10])), (8, 8)).expect("open");
        assert!(!two.is_single());
    }

    #[test]
    fn a_png_or_webp_that_is_cut_short_or_empty_is_an_error_with_a_reason() {
        for bytes in [still_png(16, 16), still_webp(16, 16)] {
            for cut in [8, 12, 20, 33, bytes.len() / 2, bytes.len() - 1] {
                let result = Animation::open_source(memory(bytes[..cut].to_vec()), (16, 16))
                    .and_then(|mut animation| animation.next_frame());
                if let Err(error) = result {
                    assert!(!error.to_string().is_empty(), "cut at {cut}");
                } else {
                    // Only a cut that loses nothing the decoder needs may succeed; it never panics either way.
                }
            }
        }
        assert!(Animation::open_source(memory(b"RIFF\0\0\0\0WEBP".to_vec()), (8, 8)).is_err());
        assert!(Animation::open_source(memory(b"\x89PNG\r\n\x1a\n".to_vec()), (8, 8)).is_err());
    }

    #[test]
    fn a_long_animation_is_never_in_memory_as_a_whole_and_its_first_frame_does_not_wait_for_the_rest() {
        use std::time::Instant;

        // 120 frames of 1920 x 1080: the whole decoded animation would be about a gigabyte. One frame is encoded and its
        // block repeated, so building the file is cheap; the decoder still reads 120 frames.
        let one = gif_bytes(1920, 1080, &[10]);
        let first_block = one
            .windows(3)
            .position(|bytes| bytes == [0x21, 0xF9, 0x04])
            .expect("a graphic control block");
        let (head, rest) = one.split_at(first_block);
        let (block, trailer) = rest.split_at(rest.len() - 1);
        let mut bytes = head.to_vec();
        for _ in 0..120 {
            bytes.extend_from_slice(block);
        }
        bytes.extend_from_slice(trailer);
        live::reset();
        let started = Instant::now();
        let mut animation = Animation::open_source(memory(bytes), (480, 270)).expect("open");
        let first = animation.next_frame().expect("frame").expect("a frame");
        let first_after = started.elapsed();
        assert_eq!((first.width, first.height), (480, 270));
        let mut frames = 1;
        while let Some(frame) = animation.next_frame().expect("frame") {
            assert_eq!((frame.width, frame.height), (480, 270));
            frames += 1;
        }
        let whole = started.elapsed();
        assert_eq!(frames, 120);
        assert!(
            live::peak() <= 2,
            "{} decoded canvases were alive together, at most 2 may be",
            live::peak()
        );
        assert!(
            first_after * 4 < whole,
            "the first frame took {first_after:?} of {whole:?}: it must not wait for the other frames"
        );
    }

    #[test]
    fn a_gif_header_above_the_pixel_limit_is_refused_before_any_decoding() {
        // "GIF89a", a 9000 x 9000 logical screen, no colour table, then the trailer.
        let mut header = b"GIF89a".to_vec();
        header.extend_from_slice(&9000_u16.to_le_bytes());
        header.extend_from_slice(&9000_u16.to_le_bytes());
        header.extend_from_slice(&[0, 0, 0, 0x3B]);
        let error = Animation::open_source(memory(header), (100, 100)).expect_err("too large");
        assert_eq!(
            error,
            DecodeError::TooLarge {
                width: 9000,
                height: 9000
            }
        );
        assert!(error.to_string().contains("9000"), "{error}");
    }

    #[test]
    fn a_truncated_gif_is_an_error_never_a_panic() {
        let bytes = gif_bytes(32, 18, &[10; 4]);
        // Cut inside the header, inside the first frame and inside a later frame.
        for cut in [4, 13, 30, bytes.len() / 2, bytes.len() - 3] {
            let source = memory(bytes[..cut].to_vec());
            match Animation::open_source(source, (32, 18)) {
                Err(error) => assert!(!error.to_string().is_empty(), "cut at {cut}"),
                Ok(mut animation) => {
                    // Whatever happens next is a value: frames, an end or an error.
                    for _ in 0..10 {
                        match animation.next_frame() {
                            Ok(Some(_)) => {}
                            Ok(None) | Err(_) => break,
                        }
                    }
                }
            }
        }
        let early = Animation::open_source(memory(bytes[..13].to_vec()), (32, 18));
        assert!(early.is_err(), "a GIF cut inside its first frame does not open");
    }

    #[test]
    fn a_missing_file_is_an_io_error() {
        let error = Animation::open(Path::new("this/file/does/not/exist.gif"), (10, 10)).expect_err("missing");
        assert!(matches!(error, DecodeError::Io(_)), "{error:?}");
    }

    impl Animation {
        fn into_first(mut self) -> DecodedFrame {
            self.next_frame().expect("frame").expect("a frame")
        }
    }
}
