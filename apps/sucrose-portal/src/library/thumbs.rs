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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};

    use super::*;

    /// A cached value with a size, so the cache is tested without GPUI.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Pic(&'static str, usize);

    impl Release for Pic {
        fn bytes(&self) -> usize {
            self.1
        }
    }

    fn key(name: &str) -> ThumbKey {
        ThumbKey {
            path: PathBuf::from(name),
            modified: None,
            bucket: Bucket::W360,
        }
    }

    fn names(images: &[Pic]) -> Vec<&'static str> {
        images.iter().map(|pic| pic.0).collect()
    }

    #[test]
    fn thumbs_cache_evicts_the_least_recently_used_image_past_the_image_cap() {
        let mut cache = ThumbCache::new(CACHE_IMAGES, usize::MAX);
        let names: Vec<String> = (0..=CACHE_IMAGES).map(|n| format!("t{n}")).collect();
        let mut evicted = Vec::new();
        for name in &names {
            evicted.extend(cache.insert(key(name), Pic("p", 1)));
        }
        assert_eq!(evicted.len(), 1);
        assert_eq!(cache.len(), CACHE_IMAGES);
        assert!(!cache.contains(&key("t0")), "the first image is the one that went");
        assert!(cache.contains(&key("t64")));
    }

    #[test]
    fn thumbs_cache_get_moves_an_image_to_the_most_recent_end() {
        let mut cache = ThumbCache::new(2, usize::MAX);
        cache.insert(key("a"), Pic("a", 1));
        cache.insert(key("b"), Pic("b", 1));
        assert_eq!(cache.get(&key("a")), Some(Pic("a", 1)));
        let evicted = cache.insert(key("c"), Pic("c", 1));
        assert_eq!(names(&evicted), ["b"], "a was shown last, so b is the oldest");
        assert!(cache.contains(&key("a")) && cache.contains(&key("c")));
    }

    #[test]
    fn thumbs_cache_evicts_the_oldest_until_the_total_is_under_the_byte_cap() {
        let mut cache = ThumbCache::new(100, 100);
        assert!(cache.insert(key("a"), Pic("a", 40)).is_empty());
        assert!(cache.insert(key("b"), Pic("b", 40)).is_empty());
        let evicted = cache.insert(key("c"), Pic("c", 40));
        assert_eq!(names(&evicted), ["a"]);
        assert_eq!(cache.total_bytes(), 80);
        // One large image can push out several small ones.
        let evicted = cache.insert(key("d"), Pic("d", 90));
        assert_eq!(names(&evicted), ["b", "c"]);
        assert_eq!(cache.total_bytes(), 90);
    }

    #[test]
    fn thumbs_cache_returns_an_image_over_the_byte_cap_at_once_and_keeps_the_rest() {
        let mut cache = ThumbCache::new(100, 100);
        cache.insert(key("a"), Pic("a", 40));
        let evicted = cache.insert(key("big"), Pic("big", 101));
        assert_eq!(names(&evicted), ["big"]);
        assert!(!cache.contains(&key("big")));
        assert!(
            cache.contains(&key("a")),
            "an oversized image does not push the others out"
        );
        assert_eq!(cache.total_bytes(), 40);
    }

    #[test]
    fn thumbs_cache_inserting_a_key_again_returns_the_old_image() {
        let mut cache = ThumbCache::new(5, 100);
        cache.insert(key("a"), Pic("old", 10));
        let evicted = cache.insert(key("a"), Pic("new", 10));
        assert_eq!(names(&evicted), ["old"]);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.total_bytes(), 10);
    }

    #[test]
    fn thumbs_image_bytes_is_width_times_height_times_four_times_frames() {
        assert_eq!(image_bytes(360, 202, 1), 360 * 202 * 4);
        assert_eq!(image_bytes(10, 10, 3), 1200);
        assert_eq!(image_bytes(0, 100, 1), 0);
    }

    #[test]
    fn thumbs_bucket_follows_the_scale_factor() {
        assert_eq!(Bucket::for_scale(0.75), Bucket::W360);
        assert_eq!(Bucket::for_scale(1.0), Bucket::W360);
        assert_eq!(Bucket::for_scale(1.25), Bucket::W720);
        assert_eq!(Bucket::for_scale(2.0), Bucket::W720);
        assert_eq!((Bucket::W360.width(), Bucket::W720.width()), (360, 720));
    }

    #[test]
    fn thumbs_queue_starts_one_decode_for_a_key_requested_twice() {
        let mut queue = DecodeQueue::new(DECODE_SLOTS);
        queue.set_wanted([key("a"), key("a")]);
        assert_eq!(queue.next(), Some(key("a")));
        assert_eq!(
            queue.next(),
            None,
            "the second request of the same key is no new decode"
        );
        // Requested again while it runs: still one decode.
        queue.set_wanted([key("a")]);
        assert_eq!(queue.next(), None);
        queue.finish(&key("a"));
        assert!(queue.is_idle());
    }

    #[test]
    fn thumbs_queue_runs_at_most_three_decodes_at_once() {
        let mut queue = DecodeQueue::new(DECODE_SLOTS);
        queue.set_wanted(["a", "b", "c", "d", "e"].map(key));
        let started: Vec<_> = std::iter::from_fn(|| queue.next()).collect();
        assert_eq!(started, ["a", "b", "c"].map(key));
        queue.finish(&key("b"));
        assert_eq!(queue.next(), Some(key("d")));
        assert_eq!(queue.next(), None);
    }

    #[test]
    fn thumbs_queue_drops_the_requests_of_a_page_that_is_no_longer_shown() {
        let mut queue = DecodeQueue::new(1);
        queue.set_wanted(["a", "b", "c"].map(key));
        assert_eq!(queue.next(), Some(key("a")));
        // The page changed: b and c are no longer wanted, a keeps running.
        queue.set_wanted(["x", "y"].map(key));
        assert_eq!(queue.next(), None, "the slot is still taken by a");
        queue.finish(&key("a"));
        assert_eq!(queue.next(), Some(key("x")));
        queue.finish(&key("x"));
        assert_eq!(queue.next(), Some(key("y")));
        queue.finish(&key("y"));
        assert!(queue.is_idle());
    }

    #[test]
    fn thumbs_queue_does_not_queue_what_a_new_page_shares_with_the_running_decodes() {
        let mut queue = DecodeQueue::new(DECODE_SLOTS);
        queue.set_wanted(["a", "b"].map(key));
        assert_eq!(queue.next(), Some(key("a")));
        queue.set_wanted(["a", "c"].map(key));
        assert_eq!(queue.next(), Some(key("c")), "a already runs");
        assert_eq!(queue.next(), None);
    }

    fn jpeg(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
        let pixels: Vec<u8> = (0..width * height).flat_map(|_| rgb).collect();
        let mut out = Vec::new();
        JpegEncoder::new_with_quality(&mut out, 95)
            .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
            .expect("encode jpeg");
        out
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let pixels = vec![200u8; (width * height * 4) as usize];
        let mut out = Vec::new();
        PngEncoder::new(Cursor::new(&mut out))
            .write_image(&pixels, width, height, image::ExtendedColorType::Rgba8)
            .expect("encode png");
        out
    }

    #[test]
    fn thumbs_decode_scales_a_jpeg_to_the_bucket_and_stores_it_as_bgra() {
        let image = decode_bytes(&jpeg(1920, 1080, [255, 0, 0]), Bucket::W360).expect("decodes");
        let size = image.size(0);
        assert_eq!((size.width.0, size.height.0), (360, 203));
        let pixel = &image.as_bytes(0).expect("one frame")[..4];
        // Red in BGRA: blue first, red third; JPEG is lossy, so the values are approximate.
        assert!(
            pixel[0] < 40 && pixel[1] < 40 && pixel[2] > 215 && pixel[3] == 255,
            "{pixel:?}"
        );
        let large = decode_bytes(&jpeg(1920, 1080, [0, 0, 255]), Bucket::W720).expect("decodes");
        assert_eq!(large.size(0).width.0, 720);
    }

    #[test]
    fn thumbs_decode_never_enlarges_a_small_image() {
        let image = decode_bytes(&png(100, 50), Bucket::W360).expect("decodes");
        let size = image.size(0);
        assert_eq!((size.width.0, size.height.0), (100, 50));
    }

    #[test]
    fn thumbs_decode_refuses_an_image_that_declares_more_than_16384_pixels_a_side() {
        let error = decode_bytes(&png(MAX_SIDE + 1, 1), Bucket::W360).expect_err("too wide");
        assert!(error.to_string().contains("pixel limit"), "{error}");
        assert!(decode_bytes(&png(MAX_SIDE, 1), Bucket::W360).is_ok());
    }

    #[test]
    fn thumbs_decode_reports_a_malformed_image_as_an_error() {
        assert!(decode_bytes(b"this is not an image", Bucket::W360).is_err());
        let mut broken = jpeg(64, 64, [1, 2, 3]);
        broken.truncate(30);
        assert!(decode_bytes(&broken, Bucket::W360).is_err());
    }
}
