//! Card-size thumbnails (UI-07): the decoder, the bounded cache and the queue that limits concurrent decodes.
//!
//! Theme images are untrusted (T-03-25): the header is read first and an image over [`MAX_SIDE`] pixels per side is
//! refused, JPEG is scaled by the DCT before it is fully decoded, and the result is resized to the bucket width before it
//! can enter the cache.

use std::{
    collections::VecDeque,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use anyhow::{Context as _, bail};
use gpui_kit::RenderImage;
use image::{DynamicImage, GrayImage, ImageReader, Limits, RgbImage};

/// The most decoded thumbnails the cache keeps.
pub const CACHE_IMAGES: usize = 64;
/// The most decoded bytes the cache keeps.
pub const CACHE_BYTES: usize = 48 * 1024 * 1024;
/// The most decodes that run at the same time.
pub const DECODE_SLOTS: usize = 3;
/// The longest side, in pixels, an image may declare before it is refused.
pub const MAX_SIDE: u32 = 16_384;
/// The largest thumbnail file that is read.
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
/// The most memory the generic decoder may allocate for one image.
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;

/// The width a thumbnail is decoded at, chosen from the window scale factor so a resize never decodes again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bucket {
    /// For scale factors up to 1.0.
    W360,
    /// For larger scale factors.
    W720,
}

impl Bucket {
    /// The bucket for a window scale factor.
    pub fn for_scale(scale: f32) -> Self {
        if scale <= 1.0 { Self::W360 } else { Self::W720 }
    }

    /// The width in pixels.
    pub const fn width(self) -> u32 {
        match self {
            Self::W360 => 360,
            Self::W720 => 720,
        }
    }
}

/// What identifies one decoded thumbnail: the file, when it was last written and the size bucket.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThumbKey {
    /// The thumbnail file.
    pub path: PathBuf,
    /// Its last write, so a replaced file is decoded again.
    pub modified: Option<SystemTime>,
    /// The width it is decoded at.
    pub bucket: Bucket,
}

/// A cached value that knows how much memory it holds.
pub trait Release {
    /// The bytes the value holds.
    fn bytes(&self) -> usize;
}

/// The bytes of a decoded image: width × height × 4 × frames.
pub fn image_bytes(width: u32, height: u32, frames: usize) -> usize {
    (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4)
        .saturating_mul(frames)
}

impl Release for Arc<RenderImage> {
    fn bytes(&self) -> usize {
        let size = self.size(0);
        image_bytes(
            u32::try_from(size.width.0).unwrap_or(0),
            u32::try_from(size.height.0).unwrap_or(0),
            self.frame_count(),
        )
    }
}

/// The decoded thumbnails that are kept. Inserting returns what had to go, for the caller to release.
pub struct ThumbCache<T: Release> {
    max_images: usize,
    #[allow(dead_code, reason = "the byte cap is enforced by the cache task that follows")]
    max_bytes: usize,
    /// Oldest first.
    entries: VecDeque<(ThumbKey, T)>,
}

impl<T: Release + Clone> ThumbCache<T> {
    /// An empty cache with the two caps.
    pub fn new(max_images: usize, max_bytes: usize) -> Self {
        Self {
            max_images,
            max_bytes,
            entries: VecDeque::new(),
        }
    }

    /// How many images are kept.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The bytes of all kept images.
    pub fn total_bytes(&self) -> usize {
        self.entries.iter().map(|(_, value)| value.bytes()).sum()
    }

    /// Whether `key` is kept.
    pub fn contains(&self, key: &ThumbKey) -> bool {
        self.entries.iter().any(|(kept, _)| kept == key)
    }

    /// The image for `key`.
    pub fn get(&mut self, key: &ThumbKey) -> Option<T> {
        self.entries
            .iter()
            .find(|(kept, _)| kept == key)
            .map(|(_, value)| value.clone())
    }

    /// Keeps `value` and returns the images that were pushed out.
    pub fn insert(&mut self, key: ThumbKey, value: T) -> Vec<T> {
        let mut evicted = Vec::new();
        if let Some(position) = self.entries.iter().position(|(kept, _)| *kept == key) {
            evicted.extend(self.entries.remove(position).map(|(_, old)| old));
        }
        self.entries.push_back((key, value));
        while self.entries.len() > self.max_images {
            evicted.extend(self.entries.pop_front().map(|(_, old)| old));
        }
        evicted
    }
}

/// Which thumbnails wait for a decode and which are being decoded.
pub struct DecodeQueue {
    slots: usize,
    running: usize,
    pending: VecDeque<ThumbKey>,
}

impl DecodeQueue {
    /// An idle queue that lets `slots` decodes run at once.
    pub fn new(slots: usize) -> Self {
        Self {
            slots,
            running: 0,
            pending: VecDeque::new(),
        }
    }

    /// Queues the thumbnails of the page that is shown now.
    pub fn set_wanted(&mut self, keys: impl IntoIterator<Item = ThumbKey>) {
        self.pending.extend(keys);
    }

    /// The next thumbnail to decode when a slot is free.
    pub fn next(&mut self) -> Option<ThumbKey> {
        if self.running >= self.slots {
            return None;
        }
        let key = self.pending.pop_front()?;
        self.running += 1;
        Some(key)
    }

    /// Frees the slot of a finished decode.
    pub fn finish(&mut self, _key: &ThumbKey) {
        self.running = self.running.saturating_sub(1);
    }

    /// Whether nothing waits and nothing runs.
    pub fn is_idle(&self) -> bool {
        self.running == 0 && self.pending.is_empty()
    }
}

/// Reads and decodes the thumbnail at `path` for `bucket`.
pub fn decode_thumbnail(path: &Path, bucket: Bucket) -> anyhow::Result<Arc<RenderImage>> {
    let metadata = std::fs::metadata(path).context("cannot read the thumbnail")?;
    if !metadata.is_file() {
        bail!("the thumbnail is not a file");
    }
    if metadata.len() > MAX_FILE_BYTES {
        bail!(
            "the thumbnail is {} bytes, over the {MAX_FILE_BYTES} byte limit",
            metadata.len()
        );
    }
    let bytes = std::fs::read(path).context("cannot read the thumbnail")?;
    decode_bytes(&bytes, bucket)
}

/// Decodes image `bytes` at the width of `bucket`.
pub fn decode_bytes(bytes: &[u8], bucket: Bucket) -> anyhow::Result<Arc<RenderImage>> {
    let (width, height) = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("the image format is unknown")?
        .into_dimensions()
        .context("the image header cannot be read")?;
    if width > MAX_SIDE || height > MAX_SIDE {
        bail!("the image is {width} x {height} pixels, over the {MAX_SIDE} pixel limit");
    }
    let target = bucket.width();
    let image = match decode_jpeg_scaled(bytes, target) {
        Some(image) => image,
        None => decode_generic(bytes)?,
    };
    Ok(to_render_image(fit(image, target)))
}

/// A JPEG decoded straight at 1/2, 1/4 or 1/8 size, so a 1920 x 1080 cover never exists in memory at full size. `None`
/// when the file is not a JPEG this decoder handles; the generic decoder then takes over.
fn decode_jpeg_scaled(bytes: &[u8], target: u32) -> Option<DynamicImage> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let side = u16::try_from(target).ok()?;
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    let (width, height) = decoder.scale(side, side).ok()?;
    let pixels = decoder.decode().ok()?;
    let (width, height) = (u32::from(width), u32::from(height));
    match decoder.info()?.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => RgbImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgb8),
        jpeg_decoder::PixelFormat::L8 => GrayImage::from_raw(width, height, pixels).map(DynamicImage::ImageLuma8),
        _ => None,
    }
}

/// Any format the image crate reads, with the size and memory limits of T-03-25.
fn decode_generic(bytes: &[u8]) -> anyhow::Result<DynamicImage> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("the image format is unknown")?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(limits);
    reader.decode().context("the image cannot be decoded")
}

/// Shrinks `image` to fit `target` x `target` pixels, keeping its proportions; a smaller image is never enlarged.
fn fit(image: DynamicImage, target: u32) -> DynamicImage {
    if image.width() <= target && image.height() <= target {
        image
    } else {
        image.thumbnail(target, target)
    }
}

/// The BGRA image GPUI draws: alpha premultiplied, red and blue swapped.
fn to_render_image(image: DynamicImage) -> Arc<RenderImage> {
    let mut pixels = image.into_rgba8();
    let raw: &mut [u8] = &mut pixels;
    for pixel in raw.as_chunks_mut::<4>().0 {
        let alpha = u16::from(pixel[3]);
        if alpha != 255 {
            for channel in &mut pixel[..3] {
                *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
            }
        }
        pixel.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![image::Frame::new(pixels)]))
}
