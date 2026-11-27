//! Deterministic animated GIF, APNG and WebP builders for the animation fixtures (04-32).
//!
//! Nothing here is random and nothing is downloaded: the same call writes the same bytes on every run and every OS, so
//! the fixtures committed under `crates/sucrose-engine-gif/tests/fixtures` can be compared with the generator's output
//! (an xtask test does) and `synth-library --media-fixtures` can add themes of every animated format. The GIF comes
//! from `image`'s own encoder. The APNG container is written here (chunks, CRC-32, a zlib stream of stored deflate
//! blocks with its Adler-32), so no compression or PNG crate is added. The WebP container is written here around the
//! lossless frames `image`'s WebP encoder produces (VP8X, ANIM, one ANMF per frame).

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use image::{
    Delay, ExtendedColorType, Frame, Rgba, RgbaImage,
    codecs::{
        gif::{GifEncoder, Repeat},
        webp::WebPEncoder,
    },
};

/// Result of a builder; the error is the message printed to the user.
type Outcome<T> = Result<T, String>;

/// The folder of the committed animation fixtures, relative to the workspace root.
pub const FIXTURES_DIR: &str = "crates/sucrose-engine-gif/tests/fixtures";

/// The side of the committed fixtures, in pixels (square).
pub const FIXTURE_SIDE: u32 = 64;

/// The delay of each frame of the committed fixtures, in milliseconds. The first is 0 on purpose: the renderer shows
/// such a frame for its default delay.
pub const FIXTURE_DELAYS_MS: [u32; 4] = [0, 50, 50, 120];

/// One frame of an animation: a picture placed at `(x, y)` of the canvas, shown for `delay_ms`. It replaces what is under
/// it (no blending) and is not disposed afterwards.
#[derive(Debug, Clone)]
pub struct Part {
    /// The pixels of the frame.
    pub image: RgbaImage,
    /// Left edge on the canvas (even, so WebP can store it).
    pub x: u32,
    /// Top edge on the canvas (even).
    pub y: u32,
    /// How long the frame is shown, in milliseconds.
    pub delay_ms: u32,
}

impl Part {
    /// A frame that covers the whole canvas.
    #[must_use]
    pub fn full(image: RgbaImage, delay_ms: u32) -> Self {
        Self {
            image,
            x: 0,
            y: 0,
            delay_ms,
        }
    }
}

/// Checks that `parts` make a valid animation on a canvas of `size`: not empty, the first frame covers the canvas and every
/// frame lies inside it.
fn check_parts(size: (u32, u32), parts: &[Part]) -> Outcome<()> {
    let Some(first) = parts.first() else {
        return Err("an animation needs at least one frame".to_owned());
    };
    if size.0 == 0 || size.1 == 0 {
        return Err("the canvas is empty".to_owned());
    }
    if (first.x, first.y, first.image.width(), first.image.height()) != (0, 0, size.0, size.1) {
        return Err("the first frame must cover the whole canvas".to_owned());
    }
    for part in parts {
        let (width, height) = part.image.dimensions();
        if part.x + width > size.0 || part.y + height > size.1 {
            return Err("a frame lies outside the canvas".to_owned());
        }
        if part.x % 2 != 0 || part.y % 2 != 0 {
            return Err("a frame offset must be even".to_owned());
        }
    }
    Ok(())
}

/// An animated GIF that loops forever: frame `n` of `frames` is shown for `delays_ms[n]` (a GIF stores hundredths of a second).
/// The frames are whole canvases.
///
/// # Errors
/// When there is no frame, the delays do not match the frames or the encoder fails.
pub fn animated_gif(frames: &[RgbaImage], delays_ms: &[u32]) -> Outcome<Vec<u8>> {
    if frames.is_empty() || frames.len() != delays_ms.len() {
        return Err("a GIF needs one delay per frame and at least one frame".to_owned());
    }
    let failed = |error: image::ImageError| format!("cannot encode the GIF: {error}");
    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new_with_speed(&mut bytes, 10);
        encoder.set_repeat(Repeat::Infinite).map_err(failed)?;
        for (image, delay) in frames.iter().zip(delays_ms) {
            encoder
                .encode_frame(Frame::from_parts(
                    image.clone(),
                    0,
                    0,
                    Delay::from_numer_denom_ms(*delay, 1),
                ))
                .map_err(failed)?;
        }
    }
    Ok(bytes)
}

/// The CRC-32 table of the PNG specification (polynomial 0xEDB88320).
const CRC_TABLE: [u32; 256] = {
    let mut table = [0_u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 == 1 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
};

/// The CRC-32 of `pieces` taken together.
fn crc32(pieces: &[&[u8]]) -> u32 {
    let mut crc = u32::MAX;
    for piece in pieces {
        for byte in *piece {
            crc = CRC_TABLE[usize::from((crc as u8) ^ byte)] ^ (crc >> 8);
        }
    }
    !crc
}

/// The Adler-32 checksum of `data` (the trailer of a zlib stream).
fn adler32(data: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let (mut a, mut b) = (1_u32, 0_u32);
    for chunk in data.chunks(5_552) {
        for byte in chunk {
            a += u32::from(*byte);
            b += a;
        }
        a %= MODULUS;
        b %= MODULUS;
    }
    (b << 16) | a
}

/// `data` as a zlib stream of stored (uncompressed) deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut blocks = data.chunks(65_535).peekable();
    if blocks.peek().is_none() {
        out.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(block) = blocks.next() {
        let length = u16::try_from(block.len()).unwrap_or(u16::MAX);
        out.push(u8::from(blocks.peek().is_none()));
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// Appends a PNG chunk: length, type, data, CRC-32 over type and data.
fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(u32::MAX).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// The scanlines of `image` as PNG wants them before compression: a 0 filter byte, then RGBA.
fn scanlines(image: &RgbaImage) -> Vec<u8> {
    let row = image.width() as usize * 4;
    let mut data = Vec::with_capacity((row + 1) * image.height() as usize);
    for line in image.as_raw().chunks(row) {
        data.push(0);
        data.extend_from_slice(line);
    }
    data
}

/// An animated PNG that loops forever on a canvas of `size`: IHDR, acTL, then per frame an fcTL and the image data (IDAT
/// for the first frame, fdAT for the others), all RGBA with source blending and no disposal.
///
/// # Errors
/// When `parts` do not make a valid animation or a delay does not fit a PNG frame.
pub fn apng(size: (u32, u32), parts: &[Part]) -> Outcome<Vec<u8>> {
    check_parts(size, parts)?;
    let frames = u32::try_from(parts.len()).map_err(|_| "too many frames".to_owned())?;
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&size.0.to_be_bytes());
    header.extend_from_slice(&size.1.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    png_chunk(&mut out, b"IHDR", &header);
    let mut control = Vec::new();
    control.extend_from_slice(&frames.to_be_bytes());
    control.extend_from_slice(&0_u32.to_be_bytes());
    png_chunk(&mut out, b"acTL", &control);
    let mut sequence = 0_u32;
    for (index, part) in parts.iter().enumerate() {
        let delay = u16::try_from(part.delay_ms).map_err(|_| "a frame delay is above 65535 ms".to_owned())?;
        let mut frame_control = Vec::new();
        frame_control.extend_from_slice(&sequence.to_be_bytes());
        frame_control.extend_from_slice(&part.image.width().to_be_bytes());
        frame_control.extend_from_slice(&part.image.height().to_be_bytes());
        frame_control.extend_from_slice(&part.x.to_be_bytes());
        frame_control.extend_from_slice(&part.y.to_be_bytes());
        frame_control.extend_from_slice(&delay.to_be_bytes());
        frame_control.extend_from_slice(&1000_u16.to_be_bytes());
        frame_control.extend_from_slice(&[0, 0]);
        png_chunk(&mut out, b"fcTL", &frame_control);
        sequence += 1;
        let data = zlib_stored(&scanlines(&part.image));
        if index == 0 {
            png_chunk(&mut out, b"IDAT", &data);
        } else {
            let mut frame_data = sequence.to_be_bytes().to_vec();
            frame_data.extend_from_slice(&data);
            png_chunk(&mut out, b"fdAT", &frame_data);
            sequence += 1;
        }
    }
    png_chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// Appends a RIFF chunk: four-character code, little-endian size, data, a pad byte when the size is odd.
fn riff_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(kind);
    out.extend_from_slice(&u32::try_from(data.len()).unwrap_or(u32::MAX).to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
}

/// The first 24 bits of `value`, little-endian.
fn le24(value: u32) -> [u8; 3] {
    let bytes = value.to_le_bytes();
    [bytes[0], bytes[1], bytes[2]]
}

/// The VP8L chunk (header, data and pad byte) of a lossless encoding of `image`.
fn lossless_chunk(image: &RgbaImage) -> Outcome<Vec<u8>> {
    let mut riff = Vec::new();
    WebPEncoder::new_lossless(&mut riff)
        .encode(image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8)
        .map_err(|error| format!("cannot encode a WebP frame: {error}"))?;
    if riff.len() < 20 || &riff[..4] != b"RIFF" || &riff[8..12] != b"WEBP" || &riff[12..16] != b"VP8L" {
        return Err("the WebP encoder wrote an unexpected container".to_owned());
    }
    Ok(riff[12..].to_vec())
}

/// An animated WebP that loops forever on a canvas of `size`: RIFF, VP8X with the animation and alpha flags, ANIM, and one
/// ANMF per frame holding a lossless VP8L image (source blending, no disposal).
///
/// # Errors
/// When `parts` do not make a valid animation, a delay does not fit 24 bits or the encoder fails.
pub fn animated_webp(size: (u32, u32), parts: &[Part]) -> Outcome<Vec<u8>> {
    check_parts(size, parts)?;
    let mut body = b"WEBP".to_vec();
    let mut extended = vec![0x12, 0, 0, 0];
    extended.extend_from_slice(&le24(size.0 - 1));
    extended.extend_from_slice(&le24(size.1 - 1));
    riff_chunk(&mut body, b"VP8X", &extended);
    // Background colour (transparent) and loop count 0 = forever.
    riff_chunk(&mut body, b"ANIM", &[0, 0, 0, 0, 0, 0]);
    for part in parts {
        if part.delay_ms >= 1 << 24 {
            return Err("a frame delay does not fit 24 bits".to_owned());
        }
        let mut frame = Vec::new();
        frame.extend_from_slice(&le24(part.x / 2));
        frame.extend_from_slice(&le24(part.y / 2));
        frame.extend_from_slice(&le24(part.image.width() - 1));
        frame.extend_from_slice(&le24(part.image.height() - 1));
        frame.extend_from_slice(&le24(part.delay_ms));
        // Bit 1: do not blend; bit 0: do not dispose.
        frame.push(0b10);
        frame.extend_from_slice(&lossless_chunk(&part.image)?);
        riff_chunk(&mut body, b"ANMF", &frame);
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(
        &u32::try_from(body.len())
            .map_err(|_| "the WebP is too large".to_owned())?
            .to_le_bytes(),
    );
    out.extend_from_slice(&body);
    Ok(out)
}

/// The frames of the committed fixtures as parts: two whole frames (red, green), a 32 x 32 blue patch in the middle
/// (smaller than the canvas, so a decoder has to composite it onto the frame before) and a whole yellow frame.
#[must_use]
pub fn fixture_parts() -> Vec<Part> {
    let solid =
        |side: u32, colour: [u8; 3]| RgbaImage::from_pixel(side, side, Rgba([colour[0], colour[1], colour[2], 255]));
    let patch_at = FIXTURE_SIDE / 4;
    vec![
        Part::full(solid(FIXTURE_SIDE, [200, 40, 40]), FIXTURE_DELAYS_MS[0]),
        Part::full(solid(FIXTURE_SIDE, [40, 200, 40]), FIXTURE_DELAYS_MS[1]),
        Part {
            image: solid(FIXTURE_SIDE / 2, [40, 40, 200]),
            x: patch_at,
            y: patch_at,
            delay_ms: FIXTURE_DELAYS_MS[2],
        },
        Part::full(solid(FIXTURE_SIDE, [220, 220, 40]), FIXTURE_DELAYS_MS[3]),
    ]
}

/// What a viewer sees of `parts`: each frame composited onto the one before (source blending, no disposal).
#[must_use]
pub fn composite(size: (u32, u32), parts: &[Part]) -> Vec<RgbaImage> {
    let mut canvas = RgbaImage::new(size.0, size.1);
    parts
        .iter()
        .map(|part| {
            image::imageops::replace(&mut canvas, &part.image, i64::from(part.x), i64::from(part.y));
            canvas.clone()
        })
        .collect()
}

/// The three committed fixtures as `(file name, bytes)`: the same four frames in each format.
///
/// # Errors
/// When a builder fails.
pub fn fixture_files() -> Outcome<Vec<(&'static str, Vec<u8>)>> {
    let size = (FIXTURE_SIDE, FIXTURE_SIDE);
    let parts = fixture_parts();
    let delays: Vec<u32> = parts.iter().map(|part| part.delay_ms).collect();
    Ok(vec![
        ("anim.gif", animated_gif(&composite(size, &parts), &delays)?),
        ("anim.png", apng(size, &parts)?),
        ("anim.webp", animated_webp(size, &parts)?),
    ])
}

/// Writes the three fixtures into `dir`, creating it.
///
/// # Errors
/// When a builder fails or a file cannot be written.
pub fn write_fixtures(dir: &Path) -> Outcome<()> {
    fs::create_dir_all(dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    for (name, bytes) in fixture_files()? {
        let path = dir.join(name);
        fs::write(&path, bytes).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

/// Frame `frame` of the counter animation (`width` x `height`, 480 x 270 in the library): it draws `frame` in binary as five
/// blocks (most significant first) under a bar that moves 20 pixels per frame, on a background whose colour also
/// follows the frame, so every frame differs and a presented frame can be told from its neighbours.
#[must_use]
pub fn counter_frame(frame: u32, width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        let tint = u8::try_from(frame * 8).unwrap_or(u8::MAX);
        let background = [
            30_u8.saturating_add(tint),
            40,
            120_u8.saturating_sub(u8::try_from(frame * 3).unwrap_or(0)),
        ];
        let bar = y < 30 && (frame * 20..frame * 20 + 20).contains(&x);
        let block = (100..170).contains(&y)
            && (0..5_u32).any(|slot| {
                let left = 60 + slot * 80;
                (left..left + 60).contains(&x) && frame >> (4 - slot) & 1 == 1
            });
        let slot_frame =
            (100..170).contains(&y) && (0..5_u32).any(|slot| (60 + slot * 80..120 + slot * 80).contains(&x));
        if bar || block {
            Rgba([255, 255, 255, 255])
        } else if slot_frame {
            Rgba([20, 20, 20, 255])
        } else {
            Rgba([background[0], background[1], background[2], 255])
        }
    })
}

/// The frames of the counter animation: `count` whole frames of `delay_ms` each.
#[must_use]
pub fn counter_parts(width: u32, height: u32, count: u32, delay_ms: u32) -> Vec<Part> {
    (0..count)
        .map(|frame| Part::full(counter_frame(frame, width, height), delay_ms))
        .collect()
}

/// `cargo xtask media-fixtures --out <dir>`: writes `anim.gif`, `anim.png` and `anim.webp` into `<dir>`. Without `--out` it
/// writes the committed fixtures folder.
pub fn run(args: &[String]) -> ExitCode {
    let mut out: Option<PathBuf> = None;
    let mut rest = args.iter();
    while let Some(argument) = rest.next() {
        match (argument.as_str(), rest.next()) {
            ("--out", Some(dir)) => out = Some(PathBuf::from(dir)),
            _ => {
                eprintln!("usage: cargo xtask media-fixtures [--out <dir>]");
                return ExitCode::FAILURE;
            }
        }
    }
    let dir = out.unwrap_or_else(|| crate::workspace_root().join(FIXTURES_DIR));
    match write_fixtures(&dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("media-fixtures: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{
        AnimationDecoder,
        codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
    };

    use super::*;

    /// A decoded frame: its delay in milliseconds and the colour at two points (outside and inside the patch).
    type Seen = (u32, [u8; 4], [u8; 4]);

    fn seen(frames: Vec<image::Frame>) -> Vec<Seen> {
        frames
            .iter()
            .map(|frame| {
                let (numer, denom) = frame.delay().numer_denom_ms();
                let image = frame.buffer();
                assert_eq!(
                    image.dimensions(),
                    (FIXTURE_SIDE, FIXTURE_SIDE),
                    "every frame is a whole canvas"
                );
                (numer / denom, image.get_pixel(2, 2).0, image.get_pixel(32, 32).0)
            })
            .collect()
    }

    fn near(a: [u8; 4], b: [u8; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 16)
    }

    /// What the four fixture frames look like once composited: red, green, green with a blue patch, yellow.
    const EXPECTED: [(u32, [u8; 4], [u8; 4]); 4] = [
        (0, [200, 40, 40, 255], [200, 40, 40, 255]),
        (50, [40, 200, 40, 255], [40, 200, 40, 255]),
        (50, [40, 200, 40, 255], [40, 40, 200, 255]),
        (120, [220, 220, 40, 255], [220, 220, 40, 255]),
    ];

    fn assert_fixture(frames: &[Seen], format: &str) {
        assert_eq!(frames.len(), 4, "{format}: four frames");
        for (index, ((delay, outside, inside), (want_delay, want_outside, want_inside))) in
            frames.iter().zip(EXPECTED).enumerate()
        {
            assert_eq!(*delay, want_delay, "{format}: delay of frame {index}");
            assert!(
                near(*outside, want_outside),
                "{format}: frame {index} outside {outside:?}"
            );
            assert!(near(*inside, want_inside), "{format}: frame {index} inside {inside:?}");
        }
    }

    #[test]
    fn media_crc32_and_adler32_match_the_published_check_values() {
        assert_eq!(crc32(&[b"123456789"]), 0xCBF4_3926);
        assert_eq!(crc32(&[b"1234", b"56789"]), 0xCBF4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn media_apng_decodes_to_the_frames_it_was_built_from() {
        let bytes = apng((FIXTURE_SIDE, FIXTURE_SIDE), &fixture_parts()).unwrap();
        let decoder = PngDecoder::new(Cursor::new(bytes)).unwrap();
        assert!(decoder.is_apng().unwrap());
        let frames = decoder.apng().unwrap().into_frames().collect_frames().unwrap();
        assert_fixture(&seen(frames), "APNG");
    }

    #[test]
    fn media_apng_of_four_frames_of_64_by_64_at_100_ms_keeps_size_delay_and_colours_in_order() {
        let size = (64, 64);
        let parts: Vec<Part> = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]]
            .into_iter()
            .map(|c| Part::full(RgbaImage::from_pixel(64, 64, Rgba([c[0], c[1], c[2], 255])), 100))
            .collect();
        let bytes = apng(size, &parts).unwrap();
        let frames = PngDecoder::new(Cursor::new(bytes))
            .unwrap()
            .apng()
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_eq!(frames.len(), 4);
        for (frame, part) in frames.iter().zip(&parts) {
            assert_eq!(frame.buffer().dimensions(), (64, 64));
            assert_eq!(frame.delay().numer_denom_ms(), (100, 1));
            assert_eq!(frame.buffer().get_pixel(5, 5), part.image.get_pixel(5, 5));
        }
    }

    #[test]
    fn media_webp_decodes_to_the_frames_it_was_built_from() {
        let bytes = animated_webp((FIXTURE_SIDE, FIXTURE_SIDE), &fixture_parts()).unwrap();
        let decoder = WebPDecoder::new(Cursor::new(bytes)).unwrap();
        assert!(decoder.has_animation());
        let frames = decoder.into_frames().collect_frames().unwrap();
        assert_fixture(&seen(frames), "WebP");
    }

    #[test]
    fn media_gif_decodes_to_the_frames_it_was_built_from() {
        let size = (FIXTURE_SIDE, FIXTURE_SIDE);
        let parts = fixture_parts();
        let delays: Vec<u32> = parts.iter().map(|part| part.delay_ms).collect();
        let bytes = animated_gif(&composite(size, &parts), &delays).unwrap();
        let frames = GifDecoder::new(Cursor::new(bytes))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_fixture(&seen(frames), "GIF");
    }

    #[test]
    fn media_builders_write_the_same_bytes_every_time() {
        assert_eq!(fixture_files().unwrap(), fixture_files().unwrap());
    }

    #[test]
    fn media_builders_refuse_what_is_not_an_animation() {
        let whole = RgbaImage::new(8, 8);
        assert!(apng((8, 8), &[]).is_err());
        assert!(animated_webp((8, 8), &[]).is_err());
        assert!(animated_gif(&[], &[]).is_err());
        assert!(animated_gif(std::slice::from_ref(&whole), &[]).is_err());
        let small = Part::full(RgbaImage::new(4, 4), 10);
        assert!(apng((8, 8), &[small]).is_err(), "the first frame must cover the canvas");
        let outside = [
            Part::full(whole.clone(), 10),
            Part {
                image: RgbaImage::new(4, 4),
                x: 6,
                y: 0,
                delay_ms: 10,
            },
        ];
        assert!(animated_webp((8, 8), &outside).is_err());
        let odd = [
            Part::full(whole, 10),
            Part {
                image: RgbaImage::new(2, 2),
                x: 1,
                y: 0,
                delay_ms: 10,
            },
        ];
        assert!(
            apng((8, 8), &odd).is_err(),
            "WebP cannot store an odd offset, so no format takes one"
        );
    }

    #[test]
    fn media_counter_frames_all_differ() {
        let parts = counter_parts(480, 270, 24, 50);
        assert_eq!(parts.len(), 24);
        assert_ne!(parts[0].image, parts[1].image);
        assert_ne!(parts[22].image, parts[23].image);
    }
}
