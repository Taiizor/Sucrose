//! Decoding, ordering, delay, empty and bound tests of the three animation formats (GIF, APNG, animated WebP), on the
//! fixtures `cargo xtask media-fixtures` generates (committed under `tests/fixtures`) and on small files built here.
//! They go through the public decoder only, the way the renderer's decode thread does.

use std::{
    fs,
    path::{Path, PathBuf},
};

use image::{
    ExtendedColorType, ImageEncoder, Rgba, RgbaImage,
    codecs::{gif::GifEncoder, png::PngEncoder, webp::WebPEncoder},
};
use sucrose_engine_gif::decode::{Animation, DEFAULT_DELAY_MS, DecodeError, Format, MAX_SOURCE_PIXELS};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// What a viewer sees of the fixture frames: delay in ms, the colour away from the patch and the colour in the middle
/// (red, green, green with a blue patch, yellow). The first delay is the 0 of the file turned into the default.
const EXPECTED: [(u32, [u8; 3], [u8; 3]); 4] = [
    (DEFAULT_DELAY_MS, [200, 40, 40], [200, 40, 40]),
    (50, [40, 200, 40], [40, 200, 40]),
    (50, [40, 200, 40], [40, 40, 200]),
    (120, [220, 220, 40], [220, 220, 40]),
];

/// A folder of this test run for files built by a test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sucrose-gif-formats-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).expect("scratch folder");
    dir
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("write");
    path
}

fn fixture(name: &str) -> PathBuf {
    Path::new(FIXTURES).join(name)
}

/// The red, green and blue of a presented (premultiplied BGRA, opaque) pixel at (x, y).
fn rgb_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 3] {
    let at = ((y * width + x) * 4) as usize;
    [pixels[at + 2], pixels[at + 1], pixels[at]]
}

fn near(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 16)
}

/// Plays a file to its end: (delay, colour outside the patch, colour in the middle) per frame.
fn play(animation: &mut Animation) -> Vec<(u32, [u8; 3], [u8; 3])> {
    let mut seen = Vec::new();
    while let Some(frame) = animation.next_frame().expect("frame") {
        assert_eq!((frame.width, frame.height), (64, 64));
        seen.push((
            frame.delay_ms,
            rgb_at(&frame.pixels, frame.width, 2, 2),
            rgb_at(&frame.pixels, frame.width, 32, 32),
        ));
    }
    seen
}

fn assert_expected(seen: &[(u32, [u8; 3], [u8; 3])], format: &str) {
    assert_eq!(seen.len(), 4, "{format}: four frames in file order");
    for (index, (got, want)) in seen.iter().zip(EXPECTED).enumerate() {
        assert_eq!(got.0, want.0, "{format}: delay of frame {index}");
        assert!(
            near(got.1, want.1),
            "{format}: frame {index} outside the patch is {:?}",
            got.1
        );
        assert!(
            near(got.2, want.2),
            "{format}: frame {index} in the middle is {:?}",
            got.2
        );
    }
}

#[test]
fn each_format_plays_its_frames_in_file_order_with_their_own_delays() {
    for (file, format) in [
        ("anim.gif", Format::Gif),
        ("anim.png", Format::Apng),
        ("anim.webp", Format::WebP),
    ] {
        let mut animation = Animation::open(&fixture(file), (64, 64)).expect(file);
        assert_eq!(animation.format(), format, "{file}");
        assert_eq!(animation.canvas(), (64, 64), "{file}");
        assert!(!animation.is_single(), "{file}");
        assert_expected(&play(&mut animation), file);
        assert_eq!(
            animation.next_frame().expect("end"),
            None,
            "{file}: nothing after the last frame"
        );
    }
}

#[test]
fn equal_inputs_time_equally_in_every_format() {
    let delays: Vec<Vec<u32>> = ["anim.gif", "anim.png", "anim.webp"]
        .iter()
        .map(|file| {
            let mut animation = Animation::open(&fixture(file), (64, 64)).expect(file);
            play(&mut animation).iter().map(|frame| frame.0).collect()
        })
        .collect();
    assert_eq!(
        delays[0],
        [DEFAULT_DELAY_MS, 50, 50, 120],
        "a delay of 0 plays for the default"
    );
    assert_eq!(delays[0], delays[1], "GIF and APNG");
    assert_eq!(delays[0], delays[2], "GIF and WebP");
}

#[test]
fn after_the_last_frame_a_rewound_animation_starts_again_at_frame_zero() {
    for file in ["anim.gif", "anim.png", "anim.webp"] {
        let mut animation = Animation::open(&fixture(file), (64, 64)).expect(file);
        let first_pass = play(&mut animation);
        animation.rewind().expect("rewind");
        let second_pass = play(&mut animation);
        assert_eq!(first_pass, second_pass, "{file}: the second pass is the first again");
    }
}

#[test]
fn a_frame_smaller_than_the_canvas_is_composited_onto_the_frame_before_it() {
    // Frame 2 of the APNG and the WebP is a 32 x 32 patch; the viewer sees the green canvas with the patch on it.
    for file in ["anim.png", "anim.webp"] {
        let mut animation = Animation::open(&fixture(file), (64, 64)).expect(file);
        let seen = play(&mut animation);
        assert!(
            near(seen[2].1, [40, 200, 40]),
            "{file}: the canvas around the patch is kept"
        );
        assert!(near(seen[2].2, [40, 40, 200]), "{file}: the patch is drawn");
    }
}

#[test]
fn a_larger_animation_is_scaled_to_the_surface_in_every_format() {
    for file in ["anim.gif", "anim.png", "anim.webp"] {
        let mut animation = Animation::open(&fixture(file), (32, 32)).expect(file);
        let frame = animation.next_frame().expect("frame").expect("a frame");
        assert_eq!((frame.width, frame.height), (32, 32), "{file}");
        assert_eq!(frame.pixels.len(), 32 * 32 * 4);
    }
}

/// A CRC-32 as PNG chunks carry it (bitwise: only for the few chunks tests build).
fn crc32(parts: &[&[u8]]) -> u32 {
    let mut crc = u32::MAX;
    for byte in parts.iter().flat_map(|part| part.iter()) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
    chunk.extend_from_slice(kind);
    chunk.extend_from_slice(data);
    chunk.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
    chunk
}

fn solid(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba([70, 90, 160, 255]))
}

fn still_png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(solid(width, height).as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("png");
    bytes
}

/// A PNG made animated by hand: an acTL chunk of one frame after IHDR and an fcTL before the first IDAT, so its only
/// image is the first frame of the animation.
fn one_frame_apng(width: u32, height: u32) -> Vec<u8> {
    let png = still_png(width, height);
    let mut out = png[..8].to_vec();
    let mut at = 8;
    while at < png.len() {
        let length = u32::from_be_bytes([png[at], png[at + 1], png[at + 2], png[at + 3]]) as usize;
        let kind: [u8; 4] = png[at + 4..at + 8].try_into().expect("four bytes");
        let end = at + 12 + length;
        if &kind == b"IDAT" && !out.windows(4).any(|window| window == b"fcTL") {
            let mut control = 0_u32.to_be_bytes().to_vec();
            control.extend_from_slice(&width.to_be_bytes());
            control.extend_from_slice(&height.to_be_bytes());
            control.extend_from_slice(&[0; 8]);
            control.extend_from_slice(&50_u16.to_be_bytes());
            control.extend_from_slice(&1000_u16.to_be_bytes());
            control.extend_from_slice(&[0, 0]);
            out.extend_from_slice(&png_chunk(b"fcTL", &control));
        }
        out.extend_from_slice(&png[at..end]);
        if &kind == b"IHDR" {
            let mut animation = 1_u32.to_be_bytes().to_vec();
            animation.extend_from_slice(&0_u32.to_be_bytes());
            out.extend_from_slice(&png_chunk(b"acTL", &animation));
        }
        at = end;
    }
    out
}

fn one_frame_gif(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new_with_speed(&mut bytes, 10);
        encoder
            .encode_frame(image::Frame::from_parts(
                solid(width, height),
                0,
                0,
                image::Delay::from_numer_denom_ms(40, 1),
            ))
            .expect("gif");
    }
    bytes
}

fn still_webp(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    WebPEncoder::new_lossless(&mut bytes)
        .encode(solid(width, height).as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("webp");
    bytes
}

#[test]
fn a_file_of_one_frame_is_a_single_frame_in_every_format() {
    let dir = scratch("single");
    for (name, bytes, format) in [
        ("one.gif", one_frame_gif(24, 12), Format::Gif),
        ("one.png", still_png(24, 12), Format::Png),
        ("one-animated.png", one_frame_apng(24, 12), Format::Apng),
        ("one.webp", still_webp(24, 12), Format::WebP),
    ] {
        let path = write(&dir, name, &bytes);
        let mut animation = Animation::open(&path, (24, 12)).expect(name);
        assert_eq!(animation.format(), format, "{name}");
        assert!(
            animation.is_single(),
            "{name}: the renderer shows it once and starts no frame timer"
        );
        let frame = animation.next_frame().expect("frame").expect("one frame");
        assert_eq!((frame.width, frame.height), (24, 12), "{name}");
        assert!(frame.delay_ms >= 1, "{name}: never a zero delay");
        assert_eq!(animation.next_frame().expect("end"), None, "{name}: and no second one");
        animation.rewind().expect("rewind");
        assert!(animation.is_single(), "{name}: still one after a rewind");
    }
    fs::remove_dir_all(&dir).expect("clean");
}

#[test]
fn a_missing_empty_or_frameless_file_is_an_error_with_a_reason_never_a_blank_animation() {
    let dir = scratch("empty");
    let missing = Animation::open(&dir.join("not-there.gif"), (10, 10)).expect_err("missing");
    assert!(matches!(missing, DecodeError::Io(_)), "{missing:?}");

    let empty = write(&dir, "empty.gif", b"");
    assert_eq!(
        Animation::open(&empty, (10, 10)).expect_err("empty"),
        DecodeError::NotAnimation
    );

    // A GIF with a header and a trailer and no frame between them.
    let mut frameless = b"GIF89a".to_vec();
    frameless.extend_from_slice(&[1, 0, 1, 0, 0, 0, 0, 0x3B]);
    let path = write(&dir, "frameless.gif", &frameless);
    let error = Animation::open(&path, (10, 10)).expect_err("no frame");
    assert!(
        matches!(error, DecodeError::NoFrames | DecodeError::Malformed(_)),
        "{error:?}"
    );
    for error in [missing, error] {
        assert!(!error.to_string().is_empty());
    }
    fs::remove_dir_all(&dir).expect("clean");
}

#[test]
fn a_file_cut_after_its_header_is_an_error_in_every_format() {
    let dir = scratch("cut");
    // The header of each fixture ends after: the GIF logical screen (13 bytes), the PNG signature and IHDR (33), the
    // WebP RIFF header and the VP8X chunk (30).
    for (file, header) in [("anim.gif", 13), ("anim.png", 33), ("anim.webp", 30)] {
        let bytes = fs::read(fixture(file)).expect("fixture");
        let path = write(&dir, file, &bytes[..header]);
        let error = Animation::open(&path, (64, 64)).expect_err(file);
        assert!(!error.to_string().is_empty(), "{file}: {error:?}");
    }
    // Cut inside the animation: it either does not open or ends with an error value, and nothing panics.
    for file in ["anim.gif", "anim.png", "anim.webp"] {
        let bytes = fs::read(fixture(file)).expect("fixture");
        for cut in [bytes.len() / 3, bytes.len() / 2, bytes.len() - 1] {
            let path = write(&dir, file, &bytes[..cut]);
            if let Ok(mut animation) = Animation::open(&path, (64, 64)) {
                for _ in 0..8 {
                    match animation.next_frame() {
                        Ok(Some(_)) => {}
                        Ok(None) | Err(_) => break,
                    }
                }
            }
        }
    }
    fs::remove_dir_all(&dir).expect("clean");
}

#[test]
fn a_source_above_8192_by_8192_is_refused_from_its_header_in_every_format() {
    let dir = scratch("huge");
    assert_eq!(MAX_SOURCE_PIXELS, 8192 * 8192);

    // GIF: the logical screen of 9000 x 9000, then the trailer.
    let mut gif = b"GIF89a".to_vec();
    gif.extend_from_slice(&9000_u16.to_le_bytes());
    gif.extend_from_slice(&9000_u16.to_le_bytes());
    gif.extend_from_slice(&[0, 0, 0, 0x3B]);

    // PNG: a signature and an IHDR of 9000 x 9000, and nothing else.
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = 9000_u32.to_be_bytes().to_vec();
    header.extend_from_slice(&9000_u32.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    png.extend_from_slice(&png_chunk(b"IHDR", &header));

    // WebP: a VP8X chunk with a canvas of 9000 x 9000, and nothing else.
    let mut webp = b"RIFF".to_vec();
    webp.extend_from_slice(&22_u32.to_le_bytes());
    webp.extend_from_slice(b"WEBPVP8X");
    webp.extend_from_slice(&10_u32.to_le_bytes());
    webp.extend_from_slice(&[0x12, 0, 0, 0]);
    webp.extend_from_slice(&8999_u32.to_le_bytes()[..3]);
    webp.extend_from_slice(&8999_u32.to_le_bytes()[..3]);

    for (name, bytes) in [("huge.gif", gif), ("huge.png", png), ("huge.webp", webp)] {
        let path = write(&dir, name, &bytes);
        let error = Animation::open(&path, (100, 100)).expect_err(name);
        assert_eq!(
            error,
            DecodeError::TooLarge {
                width: 9000,
                height: 9000
            },
            "{name}"
        );
        assert!(error.to_string().contains("8192"), "{name}: the reason names the limit");
    }
    fs::remove_dir_all(&dir).expect("clean");
}
