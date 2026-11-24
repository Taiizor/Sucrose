//! The hover preview (LIB-06, D-24, UI-07): a card's Preview GIF is decoded at card size, off the UI thread, only after the
//! pointer rested on the card for [`HOVER_DWELL`], within hard frame and byte caps. One preview lives at a time and it is
//! released when the pointer leaves, so a sweep across the grid decodes nothing and memory returns to where it was.
//!
//! The GIF is untrusted (T-03-44): a logical screen over [`MAX_SIDE`] pixels per side is refused, the decoder allocates at
//! most [`MAX_DECODE_ALLOC`] bytes, and every frame is shrunk to card size before it is kept.

use std::{io::Cursor, path::Path, sync::Arc, time::Duration};

use anyhow::{Context as _, bail};
use gpui_kit::RenderImage;
use image::{AnimationDecoder as _, Delay, Frame, ImageDecoder as _, Limits, codecs::gif::GifDecoder, imageops};

use super::thumbs::{Bucket, MAX_SIDE, image_bytes, to_bgra};

/// The most frames a preview keeps; a longer GIF loops what was decoded.
pub const MAX_PREVIEW_FRAMES: usize = 120;
/// The most decoded bytes a preview keeps.
pub const MAX_PREVIEW_BYTES: usize = 24 * 1024 * 1024;
/// How long the pointer must rest on a card before its preview is decoded (UI-SPEC).
pub const HOVER_DWELL: Duration = Duration::from_millis(200);
/// The largest preview file that is read.
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
/// The most memory the GIF decoder may allocate for one frame.
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;
/// A frame that declares less than this plays at [`DEFAULT_DELAY_MS`] instead, as browsers do, so no GIF spins the UI.
const MIN_DELAY_MS: u32 = 20;
const DEFAULT_DELAY_MS: u32 = 100;

/// The size a preview of `bytes` is decoded at for `bucket`: the GIF's own proportions, never wider than the bucket and
/// never enlarged. `None` when the file is not a GIF or its logical screen is over [`MAX_SIDE`].
pub fn preview_size(_bytes: &[u8], _bucket: Bucket) -> Option<(u32, u32)> {
    None
}

/// The frames of the GIF in `bytes` at `width` x `height` (BGRA, alpha premultiplied, source delays), at most
/// [`MAX_PREVIEW_FRAMES`] and [`MAX_PREVIEW_BYTES`]. A damaged tail keeps the frames before it; garbage gives `None`.
pub fn decode_preview(_bytes: &[u8], _width: u32, _height: u32) -> Option<Vec<Frame>> {
    None
}

/// Reads and decodes the preview GIF at `path` for `bucket`.
pub fn load_preview(_path: &Path, _bucket: Bucket) -> anyhow::Result<Arc<RenderImage>> {
    bail!("not implemented")
}

/// What the finished decode of a folder means for the single live preview.
#[derive(Debug, PartialEq, Eq)]
pub enum Finish<I> {
    /// The preview plays.
    Playing,
    /// The decode failed; the card keeps its thumbnail.
    Failed,
    /// The pointer moved on: the image (when there is one) is of no use and must be released.
    Stale(Option<I>),
}

/// The single live preview.
#[derive(Debug, PartialEq, Eq)]
pub enum PreviewState<I> {
    /// No card is hovered with a preview in mind.
    Idle,
    /// The pointer rests on `folder`; the dwell has not passed.
    Waiting { folder: String },
    /// The dwell passed; the decode of `folder` runs.
    Decoding { folder: String },
    /// `image` plays on the card of `folder`.
    Playing { folder: String, image: I },
}

impl<I> PreviewState<I> {
    /// The pointer entered the card of `folder`. Returns the image of the card that played before, to release.
    pub fn hover(&mut self, _folder: &str) -> Option<I> {
        None
    }

    /// The pointer left the card of `folder`. Returns its image, to release.
    pub fn leave(&mut self, _folder: &str) -> Option<I> {
        None
    }

    /// The dwell of `folder` ended: whether its decode should start now.
    pub fn dwell_elapsed(&mut self, _folder: &str) -> bool {
        false
    }

    /// The decode of `folder` ended with `image` (`None` when it failed).
    pub fn finished(&mut self, _folder: &str, image: Option<I>) -> Finish<I> {
        Finish::Stale(image)
    }

    /// Stops whatever runs and returns the playing image, to release (a setting turned off, a page change, a search).
    pub fn stop(&mut self) -> Option<I> {
        None
    }

    /// The folder whose preview plays, when one does.
    pub fn playing(&self) -> Option<&str> {
        None
    }
}

impl<I> Default for PreviewState<I> {
    fn default() -> Self {
        Self::Idle
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use image::{
        Delay, Frame, ImageEncoder as _, Rgba, RgbaImage,
        codecs::{gif::GifEncoder, png::PngEncoder},
    };

    use super::*;

    /// A GIF of `frames` frames, `side` x `side` pixels (16:9 when `wide`), each 40 ms.
    fn gif(frames: usize, width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut bytes);
            for step in 0..frames {
                let green = u8::try_from(step % 250).unwrap_or(0);
                let image = RgbaImage::from_pixel(width, height, Rgba([255, green, 0, 255]));
                encoder
                    .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(40, 1)))
                    .expect("encode frame");
            }
        }
        bytes
    }

    fn delay_ms(frame: &Frame) -> u32 {
        let (numer, denom) = frame.delay().numer_denom_ms();
        numer / denom
    }

    /// A GIF header whose logical screen is `width` x `height`, with no frames.
    fn screen_only(width: u16, height: u16) -> Vec<u8> {
        let mut bytes = b"GIF89a".to_vec();
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0x3B]);
        bytes
    }

    #[test]
    fn preview_decodes_every_frame_at_card_size_as_bgra_with_the_source_delays() {
        let frames = decode_preview(&gif(10, 64, 36), 360, 203).expect("decodes");
        assert_eq!(frames.len(), 10);
        for frame in &frames {
            assert_eq!(frame.buffer().dimensions(), (360, 203));
            assert_eq!(delay_ms(frame), 40);
        }
        // Orange (255, 0, 0 + green step) is BGRA: blue 0 first, red 255 third, opaque.
        let pixel = &frames[0].buffer().as_raw()[..4];
        assert!(pixel[0] < 8 && pixel[2] > 247 && pixel[3] == 255, "{pixel:?}");
    }

    #[test]
    fn preview_stops_at_the_frame_cap_and_keeps_what_it_decoded() {
        let frames = decode_preview(&gif(MAX_PREVIEW_FRAMES + 10, 8, 8), 8, 8).expect("decodes");
        assert_eq!(frames.len(), MAX_PREVIEW_FRAMES);
    }

    #[test]
    fn preview_stops_before_the_frame_cap_when_the_byte_cap_comes_first() {
        let frames = decode_preview(&gif(20, 8, 8), 1024, 1024).expect("decodes");
        assert_eq!(frames.len(), MAX_PREVIEW_BYTES / (1024 * 1024 * 4));
        assert!(frames.len() < MAX_PREVIEW_FRAMES);
    }

    #[test]
    fn preview_of_garbage_is_none() {
        assert!(decode_preview(b"this is not a gif", 360, 203).is_none());
        assert!(decode_preview(&[], 360, 203).is_none());
    }

    #[test]
    fn preview_refuses_a_logical_screen_over_the_side_limit() {
        let wide = screen_only(20_000, 1);
        assert!(preview_size(&wide, Bucket::W360).is_none());
        assert!(decode_preview(&wide, 360, 203).is_none());
    }

    #[test]
    fn preview_size_keeps_the_proportions_and_never_enlarges() {
        let bytes = gif(1, 64, 36);
        assert_eq!(preview_size(&bytes, Bucket::W360), Some((64, 36)));
        let large = gif(1, 800, 450);
        assert_eq!(preview_size(&large, Bucket::W360), Some((360, 203)));
        assert_eq!(preview_size(&large, Bucket::W720), Some((720, 405)));
        assert!(preview_size(b"nope", Bucket::W360).is_none());
        // A PNG is not a preview.
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&[0u8; 16], 2, 2, image::ExtendedColorType::Rgba8)
            .expect("encode png");
        assert!(preview_size(&png, Bucket::W360).is_none());
    }

    #[test]
    fn preview_a_zero_delay_plays_at_the_browser_default() {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut bytes);
            let image = RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255]));
            encoder
                .encode_frame(Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(0, 1)))
                .expect("encode frame");
        }
        let frames = decode_preview(&bytes, 4, 4).expect("decodes");
        assert_eq!(delay_ms(&frames[0]), 100);
    }

    #[test]
    fn preview_hover_then_leave_before_the_dwell_decodes_nothing() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        assert_eq!(state.hover("A"), None);
        assert_eq!(state, PreviewState::Waiting { folder: "A".into() });
        assert_eq!(state.leave("A"), None);
        assert_eq!(state, PreviewState::Idle);
        assert!(!state.dwell_elapsed("A"), "the timer of a left card starts no decode");
    }

    #[test]
    fn preview_hover_elsewhere_releases_the_playing_preview_before_the_next_starts() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        state.hover("A");
        assert!(state.dwell_elapsed("A"));
        assert_eq!(state.finished("A", Some(7)), Finish::Playing);
        assert_eq!(state.playing(), Some("A"));
        assert_eq!(state.hover("B"), Some(7), "A's image is handed back to be released");
        assert_eq!(state, PreviewState::Waiting { folder: "B".into() });
        assert_eq!(state.playing(), None);
    }

    #[test]
    fn preview_a_result_for_a_card_the_pointer_left_is_dropped_and_released() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        state.hover("A");
        assert!(state.dwell_elapsed("A"));
        state.hover("B");
        assert_eq!(state.finished("A", Some(3)), Finish::Stale(Some(3)));
        assert_eq!(state, PreviewState::Waiting { folder: "B".into() });
        assert!(state.dwell_elapsed("B"));
        assert_eq!(state.finished("B", None), Finish::Failed);
        assert_eq!(state, PreviewState::Idle);
    }

    #[test]
    fn preview_turned_off_while_playing_stops_and_releases_at_once() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        state.hover("A");
        state.dwell_elapsed("A");
        state.finished("A", Some(9));
        assert_eq!(state.stop(), Some(9));
        assert_eq!(state, PreviewState::Idle);
        assert_eq!(state.stop(), None);
    }

    #[test]
    fn preview_page_changed_while_decoding_drops_the_late_result() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        state.hover("A");
        assert!(state.dwell_elapsed("A"));
        assert_eq!(state.stop(), None, "nothing plays yet");
        assert_eq!(state.finished("A", Some(5)), Finish::Stale(Some(5)));
        assert_eq!(state, PreviewState::Idle);
    }

    #[test]
    fn preview_leaving_a_playing_card_releases_it_and_hovering_it_again_waits_again() {
        let mut state: PreviewState<u8> = PreviewState::Idle;
        state.hover("A");
        state.dwell_elapsed("A");
        state.finished("A", Some(1));
        assert_eq!(state.hover("A"), None, "the same card keeps playing");
        assert_eq!(state.playing(), Some("A"));
        assert_eq!(state.leave("B"), None, "another card's leave changes nothing");
        assert_eq!(state.leave("A"), Some(1));
        assert_eq!(state.hover("A"), None);
        assert_eq!(state, PreviewState::Waiting { folder: "A".into() });
    }

    #[test]
    fn preview_dwell_is_200_ms() {
        assert_eq!(HOVER_DWELL, Duration::from_millis(200));
    }
}
