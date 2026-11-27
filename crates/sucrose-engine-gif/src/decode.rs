//! Lazy, bounded animation decoding. Pure code, the same on every OS.
//!
//! An [`Animation`] decodes one frame at a time from its source, scales a frame down to fit the bound it was opened
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
    AnimationDecoder, ImageDecoder, Limits, RgbaImage,
    codecs::gif::GifDecoder,
    imageops::{self, FilterType},
};

/// The delay of a frame whose own delay is 0, as the legacy XamlAnimatedGif library has it (milliseconds).
pub const DEFAULT_DELAY_MS: u32 = 100;

/// The largest animation canvas side the decoder accepts. A hostile header cannot make it allocate more (T-04-66).
const MAX_SIDE: u32 = 16_384;

/// The most memory the decoder may allocate for one canvas (T-04-66); 04-32 adds the source size cap on top.
const MAX_ALLOC: u64 = 256 * 1024 * 1024;

/// Why an animation could not be decoded. One reason per value, shown in the log and the tray.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    /// The file could not be read.
    #[error("the animation file cannot be read: {0}")]
    Io(String),
    /// The file is not an animation format this renderer knows.
    #[error("the file is not a GIF animation")]
    NotAnimation,
    /// The format is known but not played yet.
    #[error("{0} animations are not supported yet")]
    Unsupported(&'static str),
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

/// The format of a file, by its first bytes (never by the extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Gif,
    Png,
    WebP,
    Unknown,
}

fn sniff(head: &[u8]) -> Format {
    if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Format::Gif
    } else if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Format::Png
    } else if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Format::WebP
    } else {
        Format::Unknown
    }
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

/// An animation being decoded one frame at a time. It is not `Send` (the frame iterator is not): the thread that
/// decodes opens it.
pub struct Animation {
    source: Source,
    max: (u32, u32),
    scaling: Scaling,
    frames: image::Frames<'static>,
    /// The first frame, decoded by `open` so that a damaged file fails there and not mid-wallpaper.
    pending: Option<RawFrame>,
}

impl std::fmt::Debug for Animation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animation")
            .field("source", &self.source)
            .field("max", &self.max)
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
        let mut head = [0_u8; 12];
        let mut read = 0;
        while read < head.len() {
            match reader.read(&mut head[read..]) {
                Ok(0) => break,
                Ok(count) => read += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(DecodeError::Io(error.to_string())),
            }
        }
        match sniff(&head[..read]) {
            Format::Gif => {}
            Format::Png => return Err(DecodeError::Unsupported("APNG")),
            Format::WebP => return Err(DecodeError::Unsupported("WebP")),
            Format::Unknown => return Err(DecodeError::NotAnimation),
        }
        reader.rewind().map_err(|error| DecodeError::Io(error.to_string()))?;
        let mut decoder = GifDecoder::new(reader).map_err(|error| DecodeError::Malformed(error.to_string()))?;
        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_SIDE);
        limits.max_image_height = Some(MAX_SIDE);
        limits.max_alloc = Some(MAX_ALLOC);
        decoder
            .set_limits(limits)
            .map_err(|error| DecodeError::Malformed(error.to_string()))?;
        let mut frames = decoder.into_frames();
        let first = match frames.next() {
            None => return Err(DecodeError::NoFrames),
            Some(Err(error)) => return Err(DecodeError::Malformed(error.to_string())),
            Some(Ok(frame)) => raw(frame),
        };
        Ok(Self {
            source,
            max: (max.0.max(1), max.1.max(1)),
            scaling,
            frames,
            pending: Some(first),
        })
    }

    fn pull(&mut self) -> Result<Option<RawFrame>, DecodeError> {
        if let Some(frame) = self.pending.take() {
            return Ok(Some(frame));
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
    RawFrame {
        image: frame.into_buffer(),
        delay_ms: if delay_ms == 0 { DEFAULT_DELAY_MS } else { delay_ms },
    }
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

    /// A GIF of solid frames: frame `i` has red `i * 70`. `delays_ms` gives one delay per frame.
    pub(crate) fn gif_bytes(width: u32, height: u32, delays_ms: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new_with_speed(&mut bytes, 10);
            encoder.set_repeat(Repeat::Infinite).expect("repeat");
            for (index, delay) in delays_ms.iter().enumerate() {
                let red = u8::try_from(index * 70).expect("few frames");
                let image = RgbaImage::from_pixel(width, height, Rgba([red, 90, 160, 255]));
                encoder
                    .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(*delay, 1)))
                    .expect("frame");
            }
        }
        bytes
    }

    /// The frame number a decoded frame shows, read from its red channel (BGRA: byte 2).
    pub(crate) fn frame_number(pixels: &[u8]) -> u32 {
        (u32::from(pixels[2]) + 35) / 70
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
            RawFrame {
                image: RgbaImage::from_pixel(1, 1, image::Rgba([200, 100, 50, 128])),
                delay_ms: 10,
            },
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

    #[test]
    fn png_and_webp_are_not_supported_yet() {
        let png = b"\x89PNG\r\n\x1a\n....".to_vec();
        assert_eq!(
            Animation::open_source(memory(png), (10, 10)).expect_err("png"),
            DecodeError::Unsupported("APNG")
        );
        let webp = b"RIFF\0\0\0\0WEBPVP8 ".to_vec();
        assert_eq!(
            Animation::open_source(memory(webp), (10, 10)).expect_err("webp"),
            DecodeError::Unsupported("WebP")
        );
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
