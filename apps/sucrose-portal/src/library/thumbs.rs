//! Card-size thumbnails (UI-07): the decoder, the bounded cache and the queue that limits concurrent decodes.
//!
//! Theme images are untrusted (T-03-25, CR-01): the header is read first and an image over [`MAX_SIDE`] pixels per side
//! or over [`MAX_PIXELS`] pixels in area is refused by [`check_dimensions`] before any decoder allocates, JPEG is scaled
//! by the DCT before it is fully decoded, and the result is resized to the bucket width before it can enter the cache.

use std::{
    collections::{HashSet, VecDeque},
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex, PoisonError},
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
/// The most pixels (width x height) an image may declare before it is refused. A progressive JPEG needs about 6 bytes per
/// pixel before it is scaled, so a header of 16384 x 16384 would cost about 1.6 GB (CR-01); 32 Mi pixels cost about 200 MB.
pub const MAX_PIXELS: u64 = 32 * 1024 * 1024;
/// The largest thumbnail file that is read.
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
/// The most memory the generic decoder may allocate for one image; also the most the JPEG decoder may buffer for one.
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;
/// What a JPEG decode is estimated to hold per declared pixel before it is scaled down (a progressive JPEG keeps its
/// coefficients, about 6 bytes per pixel).
const JPEG_BYTES_PER_PIXEL: u64 = 6;
/// The most memory, by that estimate, the JPEG decodes of the whole Portal (thumbnails and the background image) hold
/// together (WR-12). A decode that would go over waits for the running ones; one that is alone always runs, so a file
/// within [`MAX_PIXELS`] is never refused for its size alone. Ordinary photos cost a few MB and never wait.
const JPEG_BUDGET: u64 = 256 * 1024 * 1024;

/// A shared allowance of estimated decode memory: [`DecodeBudget::acquire`] blocks while the running decodes hold too
/// much, and the returned permit gives its share back when dropped.
struct DecodeBudget {
    used: Mutex<u64>,
    freed: Condvar,
}

/// The share of a [`DecodeBudget`] one running decode holds.
struct DecodePermit<'a> {
    budget: &'a DecodeBudget,
    cost: u64,
}

impl DecodeBudget {
    const fn new() -> Self {
        Self {
            used: Mutex::new(0),
            freed: Condvar::new(),
        }
    }

    /// Waits until `cost` fits under `limit` next to the running decodes, or until none runs, and takes that share.
    fn acquire(&self, cost: u64, limit: u64) -> DecodePermit<'_> {
        let mut used = self.used.lock().unwrap_or_else(PoisonError::into_inner);
        while *used > 0 && used.saturating_add(cost) > limit {
            used = self.freed.wait(used).unwrap_or_else(PoisonError::into_inner);
        }
        *used = used.saturating_add(cost);
        DecodePermit { budget: self, cost }
    }
}

impl Drop for DecodePermit<'_> {
    fn drop(&mut self) {
        let mut used = self.budget.used.lock().unwrap_or_else(PoisonError::into_inner);
        *used = used.saturating_sub(self.cost);
        drop(used);
        self.budget.freed.notify_all();
    }
}

/// The JPEG decodes in progress in this process.
static JPEG_DECODES: DecodeBudget = DecodeBudget::new();

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

/// The decoded thumbnails that are kept, at most `max_images` and `max_bytes`; the one shown longest ago goes first.
/// Inserting returns what had to go, for the caller to release.
pub struct ThumbCache<T: Release> {
    max_images: usize,
    max_bytes: usize,
    /// The bytes of all entries.
    bytes: usize,
    /// Least recently shown first.
    entries: VecDeque<(ThumbKey, T)>,
}

impl<T: Release + Clone> ThumbCache<T> {
    /// An empty cache with the two caps.
    pub fn new(max_images: usize, max_bytes: usize) -> Self {
        Self {
            max_images,
            max_bytes,
            bytes: 0,
            entries: VecDeque::new(),
        }
    }

    /// How many images are kept.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The bytes of all kept images.
    pub fn total_bytes(&self) -> usize {
        self.bytes
    }

    /// Whether `key` is kept.
    pub fn contains(&self, key: &ThumbKey) -> bool {
        self.entries.iter().any(|(kept, _)| kept == key)
    }

    /// The image for `key`; it counts as shown now.
    pub fn get(&mut self, key: &ThumbKey) -> Option<T> {
        let position = self.entries.iter().position(|(kept, _)| kept == key)?;
        let entry = self.entries.remove(position)?;
        let value = entry.1.clone();
        self.entries.push_back(entry);
        Some(value)
    }

    /// Keeps `value` as the most recently shown image and returns the images that were pushed out. An image that is over
    /// the byte cap on its own is returned at once and not kept; the others stay.
    pub fn insert(&mut self, key: ThumbKey, value: T) -> Vec<T> {
        let mut evicted = Vec::new();
        if let Some(position) = self.entries.iter().position(|(kept, _)| *kept == key)
            && let Some((_, old)) = self.entries.remove(position)
        {
            self.bytes -= old.bytes();
            evicted.push(old);
        }
        let size = value.bytes();
        if size > self.max_bytes {
            evicted.push(value);
            return evicted;
        }
        self.bytes += size;
        self.entries.push_back((key, value));
        while self.entries.len() > self.max_images || self.bytes > self.max_bytes {
            let Some((_, oldest)) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= oldest.bytes();
            evicted.push(oldest);
        }
        evicted
    }
}

/// Which thumbnails wait for a decode and which are being decoded: a key is decoded once, and only `slots` decodes run at
/// the same time.
pub struct DecodeQueue {
    slots: usize,
    /// Slots held by decodes that are not thumbnails (the hover preview).
    reserved: usize,
    running: HashSet<ThumbKey>,
    pending: VecDeque<ThumbKey>,
}

impl DecodeQueue {
    /// An idle queue that lets `slots` decodes run at once.
    pub fn new(slots: usize) -> Self {
        Self {
            slots,
            reserved: 0,
            running: HashSet::new(),
            pending: VecDeque::new(),
        }
    }

    /// Takes a free slot for a decode that is not a thumbnail; `false` when every slot is taken.
    pub fn reserve(&mut self) -> bool {
        if self.running.len() + self.reserved >= self.slots {
            return false;
        }
        self.reserved += 1;
        true
    }

    /// Gives back a slot taken by [`Self::reserve`].
    pub fn unreserve(&mut self) {
        self.reserved = self.reserved.saturating_sub(1);
    }

    /// Replaces the waiting thumbnails with those of the page that is shown now, first card first: what the page left is
    /// dropped from the queue, what already runs or repeats is not queued again. Decodes that run keep running.
    pub fn set_wanted(&mut self, keys: impl IntoIterator<Item = ThumbKey>) {
        self.pending.clear();
        for key in keys {
            if !self.running.contains(&key) && !self.pending.contains(&key) {
                self.pending.push_back(key);
            }
        }
    }

    /// The next thumbnail to decode when a slot is free.
    pub fn next(&mut self) -> Option<ThumbKey> {
        if self.running.len() + self.reserved >= self.slots {
            return None;
        }
        let key = self.pending.pop_front()?;
        self.running.insert(key.clone());
        Some(key)
    }

    /// Frees the slot of a finished decode.
    pub fn finish(&mut self, key: &ThumbKey) {
        self.running.remove(key);
    }

    /// Whether nothing waits and nothing runs.
    pub fn is_idle(&self) -> bool {
        self.running.is_empty() && self.pending.is_empty() && self.reserved == 0
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

/// Refuses an image that declares a side over [`MAX_SIDE`] or an area over [`MAX_PIXELS`]. Called with the size from the
/// header, before any decoder allocates.
pub fn check_dimensions(width: u32, height: u32) -> anyhow::Result<()> {
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        bail!("the image is {width} x {height} pixels, over the limit");
    }
    Ok(())
}

/// Decodes image `bytes` at the width of `bucket`.
pub fn decode_bytes(bytes: &[u8], bucket: Bucket) -> anyhow::Result<Arc<RenderImage>> {
    let (width, height) = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("the image format is unknown")?
        .into_dimensions()
        .context("the image header cannot be read")?;
    check_dimensions(width, height)?;
    let target = bucket.width();
    let image = match decode_jpeg_scaled(bytes, (target, target)) {
        Some(image) => image,
        None => decode_generic(bytes)?,
    };
    Ok(to_render_image(fit(image, target)))
}

/// A JPEG decoded straight at 1/2, 1/4 or 1/8 size of which the result still covers `target` (width, height), so a large
/// photo never exists in memory at full size. Shared by the thumbnails and the background image; the caller has checked
/// the declared size with [`check_dimensions`] first. `None` when the file is not a JPEG this decoder handles; the generic
/// decoder then takes over.
pub fn decode_jpeg_scaled(bytes: &[u8], target: (u32, u32)) -> Option<DynamicImage> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    // The decoder refuses before it allocates more than this for the coefficients of one image (WR-12).
    decoder.set_max_decoding_buffer_size(usize::try_from(MAX_DECODE_ALLOC).ok()?);
    // The decoder's own reading of the header, which is what it allocates by: it is checked again and it prices the
    // decode against the budget that all the decodes of the Portal share.
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    check_dimensions(u32::from(info.width), u32::from(info.height)).ok()?;
    let cost = u64::from(info.width) * u64::from(info.height) * JPEG_BYTES_PER_PIXEL;
    let _permit = JPEG_DECODES.acquire(cost, JPEG_BUDGET);
    let (width, height) = decoder
        .scale(u16::try_from(target.0).ok()?, u16::try_from(target.1).ok()?)
        .ok()?;
    let pixels = decoder.decode().ok()?;
    let (width, height) = (u32::from(width), u32::from(height));
    match decoder.info()?.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => RgbImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgb8),
        jpeg_decoder::PixelFormat::L8 => GrayImage::from_raw(width, height, pixels).map(DynamicImage::ImageLuma8),
        _ => None,
    }
}

/// Any format the image crate reads, with the size and memory limits of T-03-25.
pub fn decode_generic(bytes: &[u8]) -> anyhow::Result<DynamicImage> {
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
    Arc::new(RenderImage::new(vec![image::Frame::new(to_bgra(image.into_rgba8()))]))
}

/// The pixels of `pixels` in the form GPUI draws: alpha premultiplied, red and blue swapped. Shared with the background
/// image, whose animated frames are converted one by one.
pub fn to_bgra(mut pixels: image::RgbaImage) -> image::RgbaImage {
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
    pixels
}

/// A small real JPEG whose frame header (SOF0) declares `width` x `height`, so the header says one thing and the data
/// another: for tests of limits that must refuse before any pixel is decoded.
#[cfg(test)]
pub(crate) fn patched_jpeg(width: u16, height: u16) -> Vec<u8> {
    use image::{ImageEncoder as _, codecs::jpeg::JpegEncoder};

    let mut out = Vec::new();
    JpegEncoder::new(&mut out)
        .write_image(&[128u8; 16 * 16 * 3], 16, 16, image::ExtendedColorType::Rgb8)
        .expect("encode jpeg");
    let sof = out
        .windows(2)
        .position(|pair| pair == [0xFF, 0xC0])
        .expect("a baseline JPEG has a SOF0 marker");
    out[sof + 5..sof + 7].copy_from_slice(&height.to_be_bytes());
    out[sof + 7..sof + 9].copy_from_slice(&width.to_be_bytes());
    out
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
    fn thumbs_cache_default_caps_are_64_images_and_48_mib() {
        assert_eq!((CACHE_IMAGES, CACHE_BYTES), (64, 48 * 1024 * 1024));
        let mut cache = ThumbCache::new(CACHE_IMAGES, CACHE_BYTES);
        let twelve_mib = 12 * 1024 * 1024;
        for name in ["a", "b", "c", "d"] {
            assert!(cache.insert(key(name), Pic("p", twelve_mib)).is_empty());
        }
        let evicted = cache.insert(key("e"), Pic("e", twelve_mib));
        assert_eq!(
            evicted.len(),
            1,
            "four images fill the byte cap, the fifth pushes one out"
        );
        assert!(!cache.contains(&key("a")));
        assert_eq!(cache.total_bytes(), CACHE_BYTES);
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
    fn thumbs_queue_shares_its_slots_with_a_reserved_preview_decode() {
        let mut queue = DecodeQueue::new(2);
        assert!(queue.reserve());
        queue.set_wanted(["a", "b"].map(key));
        assert_eq!(queue.next(), Some(key("a")));
        assert_eq!(queue.next(), None, "the preview holds the second slot");
        assert!(!queue.reserve(), "no slot is left for another preview");
        queue.unreserve();
        assert_eq!(queue.next(), Some(key("b")));
        queue.finish(&key("a"));
        queue.finish(&key("b"));
        assert!(queue.is_idle());
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
        assert!(error.to_string().contains("over the limit"), "{error}");
        assert!(decode_bytes(&png(MAX_SIDE, 1), Bucket::W360).is_ok());
    }

    #[test]
    fn a_16384_square_jpeg_header_is_refused_by_the_pixel_area_limit() {
        let bytes = patched_jpeg(16_384, 16_384);
        let error = decode_bytes(&bytes, Bucket::W360).expect_err("over the area limit");
        assert!(error.to_string().contains("over the limit"), "{error}");
    }

    #[test]
    fn the_pixel_area_limit_accepts_exactly_max_pixels_and_refuses_one_row_more() {
        assert_eq!(MAX_PIXELS, 8192 * 4096);
        assert!(check_dimensions(8192, 4096).is_ok());
        assert!(check_dimensions(8192, 4097).is_err());
        assert!(check_dimensions(16_385, 1).is_err());
    }

    #[test]
    fn a_jpeg_whose_own_header_is_over_the_limit_is_not_decoded_by_the_scaled_decoder() {
        assert!(decode_jpeg_scaled(&patched_jpeg(16_384, 16_384), (360, 360)).is_none());
    }

    #[test]
    fn the_decode_budget_admits_what_fits_and_gives_it_back_on_drop() {
        let budget = DecodeBudget::new();
        let first = budget.acquire(60, 100);
        let second = budget.acquire(40, 100);
        assert_eq!(*budget.used.lock().unwrap(), 100);
        drop(first);
        drop(second);
        assert_eq!(*budget.used.lock().unwrap(), 0);
    }

    #[test]
    fn a_decode_over_the_whole_budget_still_runs_when_it_is_alone() {
        let budget = DecodeBudget::new();
        let permit = budget.acquire(500, 100);
        assert_eq!(*budget.used.lock().unwrap(), 500);
        drop(permit);
        assert_eq!(*budget.used.lock().unwrap(), 0);
    }

    #[test]
    fn a_decode_that_does_not_fit_waits_for_the_running_one() {
        use std::{sync::mpsc, thread, time::Duration};

        let budget = Arc::new(DecodeBudget::new());
        let running = budget.acquire(80, 100);
        let (sender, receiver) = mpsc::channel();
        let waiting = {
            let budget = Arc::clone(&budget);
            thread::spawn(move || {
                let _permit = budget.acquire(80, 100);
                sender.send(()).unwrap();
            })
        };

        assert!(
            receiver.recv_timeout(Duration::from_millis(200)).is_err(),
            "the second decode must wait while the first holds the budget"
        );
        drop(running);
        assert!(
            receiver.recv_timeout(Duration::from_secs(5)).is_ok(),
            "the second decode runs once the first is done"
        );
        waiting.join().unwrap();
        assert_eq!(*budget.used.lock().unwrap(), 0);
    }

    #[test]
    fn a_one_pixel_image_still_decodes_after_the_pixel_area_limit() {
        let image = decode_bytes(&png(1, 1), Bucket::W360).expect("decodes");
        let size = image.size(0);
        assert_eq!((size.width.0, size.height.0), (1, 1));
    }

    #[test]
    fn thumbs_decode_reports_a_malformed_image_as_an_error() {
        assert!(decode_bytes(b"this is not an image", Bucket::W360).is_err());
        let mut broken = jpeg(64, 64, [1, 2, 3]);
        broken.truncate(30);
        assert!(decode_bytes(&broken, Bucket::W360).is_err());
    }
}
