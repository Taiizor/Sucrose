use std::{
    io::Cursor,
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
};

use gpui_kit::RenderImage;
use image::{DynamicImage, GrayImage, RgbImage};
use smallvec::SmallVec;

/// Thumbnails are decoded at card size, not at their original resolution
/// (many are 1920x1080), so a page of cards stays small in memory.
const THUMB_WIDTH: u32 = 480;
const THUMB_HEIGHT: u32 = 270;

/// At most this many images are decoded at once, which caps the temporary
/// memory a page of thumbnails needs while it loads.
const DECODE_SLOTS: usize = 3;

pub enum ThumbSource {
    File(PathBuf),
    Store { cache: PathBuf, url: String },
}

struct Slots {
    used: Mutex<usize>,
    freed: Condvar,
}

static SLOTS: Slots = Slots { used: Mutex::new(0), freed: Condvar::new() };

struct SlotGuard;

impl SlotGuard {
    fn acquire() -> Self {
        let mut used = SLOTS.used.lock().unwrap();
        while *used >= DECODE_SLOTS {
            used = SLOTS.freed.wait(used).unwrap();
        }
        *used += 1;
        SlotGuard
    }
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        *SLOTS.used.lock().unwrap() -= 1;
        SLOTS.freed.notify_one();
    }
}

pub fn load(source: ThumbSource) -> Option<Arc<RenderImage>> {
    let bytes = match source {
        ThumbSource::File(path) => std::fs::read(path).ok()?,
        ThumbSource::Store { cache, url } => std::fs::read(cache).ok().or_else(|| crate::data::fetch(&url))?,
    };

    let _slot = SlotGuard::acquire();
    let image = decode_jpeg_scaled(&bytes).or_else(|| image::load_from_memory(&bytes).ok())?;
    let mut pixels = image.thumbnail(THUMB_WIDTH, THUMB_HEIGHT).into_rgba8();

    // GPUI expects BGRA.
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    Some(Arc::new(RenderImage::new(SmallVec::from_elem(image::Frame::new(pixels), 1))))
}

/// JPEG can be decoded straight at 1/2, 1/4 or 1/8 size (DCT scaling), so a
/// 1920x1080 cover never exists in memory at full resolution.
fn decode_jpeg_scaled(bytes: &[u8]) -> Option<DynamicImage> {
    if !bytes.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
    let (width, height) = decoder.scale(THUMB_WIDTH as u16, THUMB_HEIGHT as u16).ok()?;
    let pixels = decoder.decode().ok()?;
    let (width, height) = (u32::from(width), u32::from(height));

    match decoder.info()?.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => RgbImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgb8),
        jpeg_decoder::PixelFormat::L8 => GrayImage::from_raw(width, height, pixels).map(DynamicImage::ImageLuma8),
        _ => None,
    }
}
