---
phase: quick-261127-uot
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/sucrose-sysinfo/src/media.rs
  - crates/sucrose-sysinfo/src/lib.rs
  - apps/sucrose/src/data_hub.rs
  - docs/FEATURES.md
autonomous: false

must_haves:
  truths:
    - "A track whose album art gives a base64 text over 2 MiB (the 2026-11-27 local FLAC case: 3,145,928 characters) reaches the page as a non-empty ThumbnailString of at most 2,097,152 characters that decodes to the same picture, re-encoded at its own size first and scaled down only when that is not enough, aspect ratio kept (maintainer decision 2026-11-27)"
    - "Art whose text already fits goes out exactly as before: the legacy Thumbnail.cs base64, with the 233 x 233 crop at (34, 1) when the corner is transparent below build 22000"
    - "Opaque art that has to be made smaller becomes a JPEG of quality 90; art with a transparent pixel stays a PNG and keeps its alpha"
    - "Art that cannot be decoded within the limits (8192 px a side, 256 MiB), or does not fit above 64 px on the longer side within 8 tries, gives an empty ThumbnailString and one WARN 'the album art cannot be made small enough for the renderer, the page gets none' with sizes only"
    - "THUMBNAIL_TEXT_MAX of sucrose-sysinfo is tied to sucrose_ipc::blob::BLOB_MAX by a compile-time assert in the data hub; the cap stays 2 MiB; the data hub's over-cap guard stays and media-session art never reaches it"
    - "No new crate or crate feature: Cargo.toml, Cargo.lock and the member manifests are unchanged (T-04-SC)"
    - "docs/FEATURES.md, Now playing row, describes the fit-to-blob behavior and records it as a behavior change against the legacy app; every edited file keeps its line endings and end-of-file state"
    - statement: "The re-encoded cover of the maintainer's own local FLAC track shows on the desktop page and looks like the original cover (not stretched, not garbled)"
      verification: backstop
  artifacts:
    - path: crates/sucrose-sysinfo/src/media.rs
      provides: "THUMBNAIL_TEXT_MAX, the bounded art decoder, thumbnail_text_within and the fit ladder (JPEG for opaque art, PNG for transparent art)"
      contains: "pub const THUMBNAIL_TEXT_MAX: usize = 2 * 1024 * 1024;"
    - path: crates/sucrose-sysinfo/src/lib.rs
      provides: "THUMBNAIL_TEXT_MAX re-exported next to MediaSession"
      contains: "THUMBNAIL_TEXT_MAX"
    - path: apps/sucrose/src/data_hub.rs
      provides: "compile-time link of the media session limit to BLOB_MAX and a boundary test"
      contains: "const _: () = assert!(sucrose_sysinfo::THUMBNAIL_TEXT_MAX <= BLOB_MAX"
    - path: docs/FEATURES.md
      provides: "Now playing row: art over the blob cap is re-encoded to fit (behavior change recorded)"
      contains: "re-encoded to fit one IPC blob"
  key_links:
    - from: crates/sucrose-sysinfo/src/media.rs
      to: apps/sucrose/src/data_hub.rs
      via: "THUMBNAIL_TEXT_MAX checked against BLOB_MAX at compile time; the hub takes the text through MediaSession::thumbnail_if_changed"
      pattern: "THUMBNAIL_TEXT_MAX <= BLOB_MAX"
    - from: apps/sucrose/src/data_hub.rs
      to: crates/sucrose-ipc/src/blob.rs
      via: "serve_thumbnails hands the text to the blob deliverer, which cuts it with chunks (empty result over BLOB_MAX)"
      pattern: "BLOB_MAX"
    - from: crates/sucrose-sysinfo/src/media.rs
      to: crates/sucrose-sysinfo/src/media.rs
      via: "platform::refresh calls thumbnail_text before taking the Shared lock; thumbnail_text delegates to thumbnail_text_within with THUMBNAIL_TEXT_MAX"
      pattern: "thumbnail_text_within"
    - from: docs/FEATURES.md
      to: crates/sucrose-sysinfo/src/media.rs
      via: "the row's numbers (2 MiB, quality 90, 8192 px, 256 MiB, 8 tries, 64 px) are the constants of media.rs"
      pattern: "ART_JPEG_QUALITY"
---

<objective>
Album art whose base64 text is over the 2 MiB IPC blob cap (BLOB_MAX) is made to fit, instead of being replaced by an empty ThumbnailString. The cap stays (maintainer decision 2026-11-27, STATE.md Pending Todos (b)). The media-session worker in sucrose-sysinfo re-encodes the art. It tries the art's own size first and scales it down only when that is not enough, as a JPEG for opaque art and a PNG that keeps transparency otherwise. It does this once per track change on its own thread. Art that already fits is unchanged.

Purpose: a now-playing page (audio fixture, Music Tunnel, Neomorphic Music Player) shows the cover of tracks with large embedded art. The evidence is a local FLAC track with 3,145,928 characters of art text, whose cover did not show on CEF or WebView2 on 2026-11-27 (the service logged its over-cap WARN 10 times). The scope adds no new crate.
Output: the fit path and its tests in crates/sucrose-sysinfo/src/media.rs, the THUMBNAIL_TEXT_MAX re-export, a compile-time link and a boundary test in apps/sucrose/src/data_hub.rs, the corrected Now playing row in docs/FEATURES.md, and the maintainer's check with the real track.
</objective>

<execution_context>
@~/.claude/gsd-core/workflows/execute-plan.md
@~/.claude/gsd-core/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.claude/rules/rust.md
@.claude/rules/cargo.md
@.claude/rules/docs.md
@.planning/STATE.md
@.planning/debug/resolved/audio-bars-silent.md
@crates/sucrose-sysinfo/src/media.rs
@crates/sucrose-sysinfo/src/lib.rs
@apps/sucrose/src/data_hub.rs
@crates/sucrose-ipc/src/blob.rs

## Maintainer decision (locked, do not revisit)

- 2026-11-27, STATE.md Pending Todos (b): "04-29 album art over the 2 MiB blob cap: re-encode (downscale) the art so it fits under BLOB_MAX instead of replacing it with an empty ThumbnailString; the cap stays." BLOB_MAX (crates/sucrose-ipc/src/blob.rs) and BLOB_CHUNK are not changed, and neither are the renderer's Reassembler or the 64 KiB frame cap.
- No new crate without the maintainer's approval (T-04-SC). None is needed. sucrose-sysinfo already depends on the workspace image 0.25.10, which has the jpeg and png features: image::ImageReader, image::Limits (public fields, #[non_exhaustive], so start from Limits::default()), image::codecs::jpeg::JpegEncoder::new_with_quality, image::codecs::png::PngEncoder::new_with_quality with CompressionType::Best and the png FilterType::Adaptive, and DynamicImage::resize_exact with image::imageops::FilterType::CatmullRom. These were checked in the registry source during planning. Tests must not add a dev-dependency either: noise comes from a fixed-seed LCG written in the test module.

## Choices made by the planner (Claude's discretion, reversible)

- Where: in sucrose-sysinfo's media worker, not in the data hub. The art is read there once per track change, off the hub tick and outside the Shared lock (platform::refresh computes the thumbnail before it locks). The hub keeps only the bytes.
- The limit lives in sucrose-sysinfo as pub const THUMBNAIL_TEXT_MAX (2 MiB, the length of the base64 text, the payload the hub sends). The data hub asserts at compile time that it is at most BLOB_MAX, so sucrose-sysinfo gets no dependency on sucrose-ipc.
- Ladder: try once at the decoded size. While the text is over the limit, the scale becomes scale × clamp(sqrt(limit ÷ last text length) × 0.9, 0.25, 0.75), and both sides are resized from the original decoded image (never from a previous attempt), so the aspect ratio holds within one pixel of rounding. The ladder stops at ART_ATTEMPTS = 8 encodes, or when the longer side would drop below ART_MIN_SIDE = 64. Opaque art (no alpha channel, or every alpha 255) is a JPEG of ART_JPEG_QUALITY = 90. Art with any transparent pixel is a PNG with CompressionType::Best. The decode is bounded by ART_DECODE_SIDE = 8192 and ART_DECODE_ALLOC = 256 MiB. The crop branch of thumbnail_text uses the same bounded decoder, so all untrusted art goes through one limited reader.
- Pages build a data URL from the text with an image/png prefix (the audio fixture does this). Chromium sniffs the image bytes, and the legacy app already passed players' JPEG art the same way, so JPEG output needs no page change.
- docs/DECISIONS.md is not changed. No D-entry covers the blob cap; the 04-29 choice and this amendment are recorded in STATE.md and, as a parity behavior change, in the docs/FEATURES.md row. docs/ARCHITECTURE.md does not mention the blob path and is not changed either. STATE.md Pending Todos (b) is closed by the quick-batch orchestrator, not by this plan.

## Exact old text of the FEATURES.md sentence (Task 2 replaces it; quoted here, not in an action)

"Art whose base64 text is over 2 MiB is not sent (the page gets an empty `ThumbnailString` and the service logs a warning)."

## Line endings (checked during planning with git ls-files --eol)

- crates/sucrose-sysinfo/src/media.rs: w/lf. crates/sucrose-sysinfo/src/lib.rs, apps/sucrose/src/data_hub.rs and docs/FEATURES.md: w/crlf. docs/FEATURES.md has no final newline (it ends with "0."). Keep both properties.
- The REPL edit() helper and the Edit tool keep a CRLF file CRLF and keep its end-of-file state when given LF text. file().text() returns LF-normalized text, so never rebuild a whole existing file from it with write() or the Write tool. Never run a whole-file sed over docs/FEATURES.md, because it would add a CR to the last line.
- cargo fmt through the Windows wrapper can rewrite CRLF to LF. Run git ls-files --eol on the touched files after it, and give any file that was w/crlf before and is w/lf now its CRLF back with sed -i 's/\r\?$/\r/'. Only do that for files that end with a newline (data_hub.rs and lib.rs do; check with tail -c 2 | od -c first).

## Source coverage audit

| Source | Item | Covered by |
|---|---|---|
| GOAL (item text) | art over BLOB_MAX re-encoded or downscaled so it fits, not dropped | Tasks 1-2 (code), Task 3 (real track) |
| CONTEXT (maintainer 2026-11-27) | the cap stays | Task 1 (BLOB_MAX untouched, compile-time link) |
| CONTEXT (item text) | no new crate without approval | Task 1 and Task 2 manifest gates, T-uot-SC |
| Evidence | local FLAC track, 3,145,928-character art text | Task 1 test at production limit (900 x 900 noise ≈ 3.2 M chars), Task 3 real run |
| Project rule | docs true in the same change | Task 2 (FEATURES row) |
| Project rule | fmt, clippy, workspace tests at the end | Task 2 verify |
| REQ | none: a quick item; it serves the now-playing path of WAPI-06, which stays open for its 04-37 B5 re-run and is not marked here | n/a |
</context>

<tasks>

<task type="tracer" tdd="true">
  <name>Task 1: Oversized opaque album art fits one IPC blob end to end (media session text, data hub, one whole blob)</name>
  <files>crates/sucrose-sysinfo/src/media.rs, crates/sucrose-sysinfo/src/lib.rs, apps/sucrose/src/data_hub.rs</files>
  <read_first>crates/sucrose-sysinfo/src/media.rs lines 24-40 (constants and their cfg_attr pattern), 190-250 (base64, crop_png, thumbnail_text), 560-645 (read_art, MAX_ART, refresh: thumbnail_text runs before the Shared lock), 855-950 (test helpers png_of, picture, decode and the crop tests); crates/sucrose-sysinfo/src/lib.rs line 23 (pub use media::...); apps/sucrose/src/data_hub.rs line 16 (BLOB_MAX import), 176-230 (serve_thumbnails and its over-cap guard), 839-870 (blob_log writes "blob slot id size", art_source), 1014-1036 (art_over_the_blob_cap_is_replaced_by_an_empty_one); crates/sucrose-ipc/src/blob.rs lines 12-26 (BLOB_CHUNK, BLOB_MAX, chunks)</read_first>
  <behavior>
    - media.rs, a_3_mib_art_text_fits_the_media_session_limit_as_a_smaller_jpeg: an opaque 900 x 900 noise PNG (fixed-seed LCG, alpha 255). First assert that base64 of the PNG is longer than THUMBNAIL_TEXT_MAX, so the test really is over the limit. thumbnail_text(&png, true) is non-empty, at most THUMBNAIL_TEXT_MAX long, decodes, image::guess_format says Jpeg, width equals height, and the side is at most 900. This is the maintainer's case on Windows 11 (cobalt, no crop).
    - media.rs, oversized_opaque_art_is_reencoded_at_its_own_size_when_that_fits: a 512 x 512 opaque photo-like PNG (a smooth gradient plus ±8 LCG noise) with a budget of half its PNG text length. thumbnail_text_within(&png, true, budget) is a JPEG of exactly 512 x 512 and at most budget long.
    - media.rs, art_that_still_does_not_fit_is_downscaled_keeping_its_aspect_ratio: a 600 x 400 opaque noise PNG with a budget of 40_000. The result is at most budget long and decodes. Its width is below 600, |width × 400 − height × 600| ≤ 600, and its longer side is at least ART_MIN_SIDE.
    - media.rs, art_that_fits_goes_out_unchanged_up_to_the_last_character: for any PNG with text length L, thumbnail_text_within(&png, true, L) equals base64(&png) (the limit is inclusive), and with L − 1 the result differs from base64(&png).
    - media.rs, art_that_cannot_be_decoded_and_does_not_fit_gives_an_empty_text: b"not an image" repeated 100 times with a budget of 64 gives "".
    - media.rs, art_that_cannot_fit_above_the_smallest_side_gives_an_empty_text: a 300 x 300 opaque noise PNG with a budget of 16 gives "".
    - The existing media.rs tests stay green unchanged: art_with_a_transparent_corner_is_cropped..., art_is_left_whole_on_cobalt..., art_that_cannot_be_decoded_goes_out_as_it_is, a_crop_of_a_small_picture..., base64_matches_the_standard_alphabet_with_padding.
    - data_hub.rs, art_at_the_media_session_limit_goes_out_as_one_whole_blob: a TakeThumbnail that gives "A" repeated sucrose_sysinfo::THUMBNAIL_TEXT_MAX times. After one pass with an Audio slot, the first log line is "blob 0 1 2097152" (built with format! from THUMBNAIL_TEXT_MAX). The existing art_over_the_blob_cap_is_replaced_by_an_empty_one stays green unchanged.
  </behavior>
  <action>Implements the maintainer decision of 2026-11-27 (STATE.md Pending Todos (b)): art over the cap is made to fit, and the cap stays. Write the tests of the behavior block first and run them to see them fail (RED), then implement (GREEN).
1. crates/sucrose-sysinfo/src/media.rs, constants beside MAX_ART:
   - pub const THUMBNAIL_TEXT_MAX: usize = 2 * 1024 * 1024; Its doc comment says what it is for: the longest ThumbnailString the media session gives, because the service sends the art to a renderer as one IPC blob, the data hub checks at compile time that it is not above that blob cap, and bigger art is made smaller to fit. It needs no cfg_attr, because apps/sucrose uses it on every OS.
   - Private: const ART_JPEG_QUALITY: u8 = 90; const ART_MIN_SIDE: u32 = 64; const ART_ATTEMPTS: u32 = 8; const ART_DECODE_SIDE: u32 = 8192; const ART_DECODE_ALLOC: u64 = 256 * 1024 * 1024; each with a one-line doc. Give new private items the same #[cfg_attr(not(windows), allow(dead_code))] annotation as the code that reaches them, so builds that are not Windows stay warning-free, as they are at HEAD.
2. A bounded decoder, decode_art(bytes) returning Option of image::DynamicImage. It uses image::ImageReader::new over a std::io::Cursor with with_guessed_format. It sets limits from image::Limits::default() with max_image_width and max_image_height Some(ART_DECODE_SIDE) and max_alloc Some(ART_DECODE_ALLOC); reader.limits takes &mut self. Then it decodes; any error gives None. Use it in the crop branch of the current thumbnail_text instead of image::load_from_memory, so one bounded reader decodes all untrusted art (T-uot-01).
3. pub(crate) fn thumbnail_text_within(bytes, cobalt, max_text) -> String. It first builds the legacy payload exactly as thumbnail_text does today: the cropped PNG when not cobalt and pixel (0, 0) is transparent, otherwise the bytes as they are. When the base64 length of that payload, payload.len().div_ceil(3) * 4, is at most max_text, it returns base64 of the payload, unchanged from today. Otherwise it calls fit_art(&payload, max_text). thumbnail_text(bytes, cobalt) becomes a call of thumbnail_text_within(bytes, cobalt, THUMBNAIL_TEXT_MAX), so both callers in platform::refresh stay as they are and keep running before the Shared lock. Do not move them under the lock.
4. fit_art(payload, max_text) returns Option of the encoded bytes together with width, height and attempts for the log. It decodes with decode_art and gives None when that fails. It judges opacity: an image without an alpha channel is opaque, and otherwise it checks every pixel's alpha through image::GenericImageView::pixels, so no second full copy is made. In this task, art with any transparent pixel gives None (Task 2 of this plan adds its PNG path). Opaque art: encode the image at its decoded size as a JPEG through JpegEncoder::new_with_quality(ART_JPEG_QUALITY) on to_rgb8(). While its base64 length is over max_text, set scale to scale × clamp(sqrt(max_text ÷ last base64 length) × 0.9, 0.25, 0.75) in f64. The new sides are each the original side × scale, rounded, at least 1. Resize the original decoded image with resize_exact and image::imageops::FilterType::CatmullRom, then encode again. Stop with None when the longer new side is below ART_MIN_SIDE or ART_ATTEMPTS encodes have run. Write the image::imageops::FilterType path out in full: the png module has a FilterType too.
5. Logging, in thumbnail_text_within (T-04-64: sizes only, never a title or bytes). When the art was made smaller: tracing::debug! with text = the original base64 length, fitted = the new base64 length, width, height, attempts and elapsed_ms (std::time::Instant), message "the album art was made smaller to fit the renderer". On None: tracing::warn! with size = the original base64 length, message "the album art cannot be made small enough for the renderer, the page gets none", and the function returns String::new().
6. Tests: write the behavior-block tests in the existing tests module. Split the base64 reverse of the current decode helper into an unbase64 helper that returns the bytes, so tests can call image::guess_format on them; decode keeps working for PNG and JPEG. Add a small fixed-seed LCG helper for noise. Add THUMBNAIL_TEXT_MAX, ART_MIN_SIDE and thumbnail_text_within to the tests' use super:: list. Add no crate and no dev-dependency.
7. crates/sucrose-sysinfo/src/lib.rs: add THUMBNAIL_TEXT_MAX to the existing pub use media::{MediaSession, media_unavailable} line (cargo fmt orders it).
8. apps/sucrose/src/data_hub.rs: at module level beside the use of BLOB_MAX, add const _: () = assert!(sucrose_sysinfo::THUMBNAIL_TEXT_MAX <= BLOB_MAX, "the album art of the media session must fit one IPC blob"); Above the guard in serve_thumbnails, add a short comment: the media session's art already fits (THUMBNAIL_TEXT_MAX), and this guard only keeps a payload that chunks would refuse from going out as nothing. The guard, its WARN text and its existing test stay unchanged. Add the boundary test art_at_the_media_session_limit_goes_out_as_one_whole_blob, using blob_log and a once-only TakeThumbnail as the over-cap test does.
Keep line endings: media.rs LF; lib.rs and data_hub.rs CRLF (see context). Commit (English): "fix(media): fit album art over the blob cap instead of dropping it".</action>
  <verify>
    <automated>
set -e
cargo test -p sucrose-sysinfo media # [VALIDATED: ran at HEAD d1d76bd7 during planning: 12 passed, 1 ignored; after this task at least 18 passed]
cargo test -p sucrose data_hub # [VALIDATED: ran at HEAD d1d76bd7 during planning: 28 passed; after this task 29 passed]
test "$(grep -c 'pub const THUMBNAIL_TEXT_MAX: usize [=] 2 [*] 1024 [*] 1024;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'const ART_JPEG_QUALITY: u8 [=] 90;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'const ART_MIN_SIDE: u32 [=] 64;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'const ART_ATTEMPTS: u32 [=] 8;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'const ART_DECODE_SIDE: u32 [=] 8192;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'const ART_DECODE_ALLOC: u64 [=] 256 [*] 1024 [*] 1024;' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c 'ImageReader' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'image::load_from_memory(bytes)' crates/sucrose-sysinfo/src/media.rs || true)" -eq 0 # [HEAD=1 EXPECT==0] the crop branch uses decode_art; the tests' load_from_memory(&bytes) is a different literal
test "$(grep -c -F 'thumbnail_text_within(' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1] the definition, the call in thumbnail_text and the tests
test "$(grep -c -F 'cannot be made small enough for the renderer' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'THUMBNAIL_TEXT_MAX' crates/sucrose-sysinfo/src/lib.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'const _: () = assert!(sucrose_sysinfo::THUMBNAIL_TEXT_MAX <= BLOB_MAX' apps/sucrose/src/data_hub.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'too large for the renderer, the page gets none' apps/sucrose/src/data_hub.rs || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: deleting the data hub's over-cap guard instead of keeping it as the backstop]
test "$(grep -c 'pub const BLOB_MAX: usize [=] 2 [*] 1024 [*] 1024;' crates/sucrose-ipc/src/blob.rs || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: raising the blob cap instead of fitting the art]
MANIFESTS=$(git diff --name-only HEAD -- Cargo.toml Cargo.lock crates/sucrose-sysinfo/Cargo.toml apps/sucrose/Cargo.toml)
test "$(printf '%s' "$MANIFESTS" | grep -c . || true)" -eq 0 # [HEAD=0 EXPECT==0 @d1d76bd7 GUARD MUTATION: adding a crate, a feature or a dev-dependency to a manifest] run before the commit
EOL=$(git ls-files --eol crates/sucrose-sysinfo/src/media.rs crates/sucrose-sysinfo/src/lib.rs apps/sucrose/src/data_hub.rs)
test "$(printf '%s\n' "$EOL" | grep 'media.rs' | grep -c 'w/lf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: saving media.rs with CRLF line endings]
test "$(printf '%s\n' "$EOL" | grep 'lib.rs' | grep -c 'w/crlf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: cargo fmt or a whole-file write turning lib.rs into LF]
test "$(printf '%s\n' "$EOL" | grep 'data_hub.rs' | grep -c 'w/crlf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: cargo fmt or a whole-file write turning data_hub.rs into LF]
    </automated>
  </verify>
  <done>The new tests failed before the change and pass after it. A 3.2 M-character opaque art text comes out as a JPEG of at most 2,097,152 characters with its aspect ratio kept, and art that fits is unchanged to the byte. The data hub builds only while THUMBNAIL_TEXT_MAX ≤ BLOB_MAX and sends a limit-sized text as one whole blob. BLOB_MAX and every manifest are unchanged, line endings are kept, and the task is committed.</done>
  <reversibility rating="reversible">The quality, ladder and limit values are constants in one module; the maintainer's decision (fit, keep the cap) is already settled.</reversibility>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Transparent album art over the cap keeps its alpha as a PNG, and the parity row tells the truth</name>
  <files>crates/sucrose-sysinfo/src/media.rs, docs/FEATURES.md</files>
  <read_first>crates/sucrose-sysinfo/src/media.rs (fit_art, decode_art and the tests from Task 1); docs/FEATURES.md row "| Now playing (media session, thumbnail) |" (line 72 at HEAD; search for the row title, since the line number may move)</read_first>
  <behavior>
    - media.rs, transparent_art_stays_a_png_with_its_alpha_when_it_is_made_smaller: a 300 x 300 RGBA noise picture whose alpha varies across the image and whose pixel (0, 0) is opaque, encoded as PNG. The budget is half its PNG text length; cobalt is true, so no crop. thumbnail_text_within is non-empty and at most budget long, image::guess_format says Png, and the decoded picture has at least one pixel with alpha below 255.
    - All Task 1 tests stay green, the opaque ones still give JPEG.
  </behavior>
  <action>Extends Task 1 per the same maintainer decision (2026-11-27): transparent covers fit too, without being flattened.
1. crates/sucrose-sysinfo/src/media.rs, fit_art: art with any transparent pixel no longer gives None. It runs the same ladder (same scale rule, same ART_ATTEMPTS and ART_MIN_SIDE stop), encoding each attempt as a PNG of to_rgba8() through image::codecs::png::PngEncoder::new_with_quality with image::codecs::png::CompressionType::Best and image::codecs::png::FilterType::Adaptive (DynamicImage::write_with_encoder or the encoder's ImageEncoder::write_image). Opaque art keeps the JPEG path. Write the test of the behavior block first and see it fail (RED: it gives an empty text before this change), then implement.
2. docs/FEATURES.md, Now playing row only: replace the sentence that begins "Art whose base64 text is over 2 MiB" (quoted in full in this plan's context) with the text below, word for word. It sits on the same table row: no line break, and the rest of the row and its trailing " | 🟨 |" stay as they are. Use the REPL edit() helper or the Edit tool with the old sentence as anchor, so the CRLF and the missing final newline are kept. Text: Art whose base64 text would be over 2 MiB (the IPC blob cap) is made to fit instead of dropped (maintainer decision 2026-11-27; a local FLAC track had 3,145,928 characters of art text): the media worker decodes it (at most 8192 px a side and 256 MiB), re-encodes it at its own size first, as a JPEG of quality 90 when every pixel is opaque and as a PNG that keeps its transparency otherwise, then scales both sides down together, at most 8 tries and never below 64 px on the longer side, until the text fits; art that cannot be decoded or made to fit gives an empty `ThumbnailString` and one warning, and art that fits is sent exactly as before. Behavior change against the legacy app, which sent art of any size: art over the cap reaches the page re-encoded to fit one IPC blob, so a page can get a JPEG or a smaller picture where the C# app passed the player's original bytes.
3. Final checks for the whole plan: cargo fmt --all, then restore CRLF where the wrapper turned a w/crlf file into w/lf (see context; files that end with a newline only). Then cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, and cargo test --workspace. The workspace tests include the real-desktop tests of apps/sucrose/tests/portal.rs, which open windows without focus. With the virtual monitor DISPLAY9 attached, two of them fail until quick item 261127-uou lands. If that happens, record which two tests failed and that DISPLAY9 was attached, and do not change portal.rs here. Stop any sucrose.exe you started before rebuilding.
Commit (English): "fix(media): keep transparent album art as a PNG when it is made to fit" (code), then "docs(features): album art over the blob cap is re-encoded to fit" (FEATURES.md), or one commit with both lines in its message.</action>
  <verify>
    <automated>
set -e
cargo test -p sucrose-sysinfo media # [VALIDATED: ran at HEAD d1d76bd7 during planning: 12 passed, 1 ignored; after this task at least 19 passed]
test "$(grep -c -F 'CompressionType::Best' crates/sucrose-sysinfo/src/media.rs || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'is not sent (the page gets an empty' docs/FEATURES.md || true)" -eq 0 # [HEAD=1 EXPECT==0]
test "$(grep -c -F 're-encoded to fit one IPC blob' docs/FEATURES.md || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F 'at most 8 tries and never below 64 px on the longer side' docs/FEATURES.md || true)" -ge 1 # [HEAD=0 EXPECT>=1]
test "$(grep -c -F '| Now playing (media session, thumbnail) |' docs/FEATURES.md || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: splitting the row with a line break or duplicating it]
FEOL=$(git ls-files --eol docs/FEATURES.md)
test "$(printf '%s\n' "$FEOL" | grep -c 'w/crlf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: saving docs/FEATURES.md with LF line endings]
test "$(tail -c 1 docs/FEATURES.md | grep -c '^\.$' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: adding a final newline or a trailing CR to docs/FEATURES.md]
NUMSTAT=$(git diff --numstat HEAD -- docs/FEATURES.md)
test "$(printf '%s' "$NUMSTAT" | cut -f1)" -eq 1 # [HEAD=absent EXPECT==1] one line added, run before the commit
test "$(printf '%s' "$NUMSTAT" | cut -f2)" -eq 1 # [HEAD=absent EXPECT==1] one line deleted, run before the commit
MANIFESTS=$(git diff --name-only HEAD -- Cargo.toml Cargo.lock crates/sucrose-sysinfo/Cargo.toml apps/sucrose/Cargo.toml)
test "$(printf '%s' "$MANIFESTS" | grep -c . || true)" -eq 0 # [HEAD=0 EXPECT==0 @d1d76bd7 GUARD MUTATION: adding a crate, a feature or a dev-dependency to a manifest] run before the commit
cargo fmt --all -- --check # [VALIDATED: project command from CLAUDE.md; cargo xtask ci runs the same check]
cargo clippy --workspace --all-targets -- -D warnings # [VALIDATED: project command from CLAUDE.md, clean at 04-37 (target/ci-0437.log)]
cargo test --workspace # [VALIDATED: project command from CLAUDE.md; two portal.rs desktop tests fail only while DISPLAY9 is attached (261127-uou)]
EOL=$(git ls-files --eol crates/sucrose-sysinfo/src/media.rs crates/sucrose-sysinfo/src/lib.rs apps/sucrose/src/data_hub.rs)
test "$(printf '%s\n' "$EOL" | grep 'media.rs' | grep -c 'w/lf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: cargo fmt or a whole-file write changing media.rs to CRLF]
test "$(printf '%s\n' "$EOL" | grep 'lib.rs' | grep -c 'w/crlf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: cargo fmt through the wrapper turning lib.rs into LF]
test "$(printf '%s\n' "$EOL" | grep 'data_hub.rs' | grep -c 'w/crlf' || true)" -ge 1 # [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: cargo fmt through the wrapper turning data_hub.rs into LF]
    </automated>
  </verify>
  <done>Transparent art over the cap fits as a PNG with its alpha and opaque art still becomes JPEG. The FEATURES Now playing row states the fit behavior and the behavior change on one row, with CRLF and no final newline kept. fmt, clippy and the workspace tests pass, or only the two DISPLAY9-dependent portal.rs tests fail and this is recorded. Everything is committed.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: The maintainer's local FLAC track shows its cover on the desktop page</name>
  <files>none (scratch folder target/art-fit only)</files>
  <action>Executor, before presenting:
1. tasklist.exe shows no sucrose.exe or sucrose-portal.exe. If one runs that you did not start, ask; do not kill it.
2. cargo build --release -p sucrose (stop the app before rebuilding: Windows locks running executables).
3. rm -rf target/art-fit, then cargo xtask synth-library --out target/art-fit --count 2 --web-fixtures --select audio --engine Web=WebViewLive --force [VALIDATED during planning: it writes Library/audio and Setting/Sucrose.json with "Selected": "audio" and "Web": "WebViewLive"]. One engine is enough: the change is in the service, and the renderer and page path are unchanged.
4. Present the instructions below to the maintainer in Turkish and wait. Do not start anything and do not play sound; playing music is his action. A reply that declines ("atla" or "skip") ends the plan with a halted SUMMARY that records the check as skipped.
5. After he replies "başla" or "start": start target/release/sucrose.exe --no-portal in the background (never in the foreground) with SUCROSE_DATA_DIR="$(wslpath -w target/art-fit)", SUCROSE_LOG=debug and WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG. Wait until the renderer log shows a page loaded line and an album art received line, or 60 s. Then run the automated log checks below and tell him in Turkish what they show (sizes only) before asking for his verdict.
6. After his verdict: taskkill.exe /IM sucrose.exe /F (there is no Portal in this run; also run taskkill.exe /IM sucrose-portal.exe /F if tasklist shows one). Never kill msedgewebview2.exe by name. tasklist.exe must show no Sucrose process at the end. Keep target/art-fit/Log for the SUMMARY; nothing under target/ is committed.
Instructions (English source, present in Turkish): "The fix for the missing cover of your local FLAC track is ready. Please close the C# Sucrose app. Start playing in your usual player the local FLAC track whose cover did not show on 27 November (the one with the very large embedded cover), and reply 'start'. I will then start Sucrose with a scratch copy of the settings; the audio test page appears on the desktop with the title, artist, state and cover of what is playing. Please tell me whether the cover shows and whether it looks like the real cover (not stretched, not garbled). Reply 'skip' to not run it now."</action>
  <what-built>Album art over the 2 MiB IPC blob cap is re-encoded at its own size, and scaled down only when needed, until it fits. The audio fixture page gets a non-empty ThumbnailString for the track whose cover was missing on 2026-11-27.</what-built>
  <how-to-verify>Machine evidence first (automated block, against target/art-fit/Log). The service log has no line of the old over-cap WARN and none of the new one. It has a "the album art was made smaller to fit the renderer" line whose text= is above 2097152 (3145928 is expected for that track). Its "album art taken" size is between 1 and 2097152. The renderer log has an "album art received" line of the same size. Then the maintainer looks at the desktop: the cover of the playing FLAC track shows on the audio test page and looks right. If the "made smaller" line is missing, the track that played was not the over-cap one: say so and ask him to play that track; do not count the run.</how-to-verify>
<!-- planner-discipline-allow: cannot be made small enough for the renderer -->
  <verify>
    <automated>
set -e
cat target/art-fit/Log/Service* > /dev/null # [UNVALIDATABLE: the log exists only after the maintainer-started run; file prefixes Service and Renderer-0 as in the 2026-11-27 debug logs, sucrose-log daily files named after the app]
test "$(cat target/art-fit/Log/Service* | grep -c 'the album art is too large for the renderer' || true)" -eq 0 # [UNVALIDATABLE: run-time log; the same check counted 10 on 2026-11-27 before the fix; EXPECT==0]
test "$(cat target/art-fit/Log/Service* | grep -c 'cannot be made small enough for the renderer' || true)" -eq 0 # [UNVALIDATABLE: run-time log; EXPECT==0]
test "$(cat target/art-fit/Log/Service* | grep 'made smaller to fit the renderer' | grep -o 'text=[0-9]*' | cut -d= -f2 | sort -n | tail -1)" -gt 2097152 # [UNVALIDATABLE: run-time log; EXPECT>2097152, 3145928 for the 2026-11-27 track]
SIZE=$(cat target/art-fit/Log/Service* | grep 'album art taken' | grep -o 'size=[0-9]*' | cut -d= -f2 | sort -n | tail -1); test "$SIZE" -gt 0 && test "$SIZE" -le 2097152 # [UNVALIDATABLE: run-time log; data_hub.rs logs "album art taken" with size]
cat target/art-fit/Log/Renderer-0* | grep 'album art received' | grep -Eq "size=$SIZE([^0-9]|$)" # [UNVALIDATABLE: run-time log; render.rs logs "album art received" with id and size]
    </automated>
  </verify>
  <resume-signal>The maintainer's verdict in his own words (for example "kapak görünüyor", the cover shows), a description of what is wrong, or "atla"/"skip". Quote it verbatim in the SUMMARY with an English translation.</resume-signal>
  <done>The machine checks pass for the over-cap track. The maintainer confirmed the cover shows and looks right, and his words are quoted. No Sucrose process is left running. A skip gives a halted SUMMARY instead.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Other app → media session worker | The album art bytes come from whichever player owns the current GSMTC session: untrusted input, decoded in the service |
| Service → renderer (IPC) | The art text travels as one ToRenderer::Blob set, bounded by BLOB_MAX on both sides |
| Maintainer's data → scratch run | Task 3 runs on target/art-fit; the maintainer's settings, library and music stay untouched |

## STRIDE Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation Plan |
|-----------|----------|-----------|----------|-------------|-----------------|
| T-uot-01 | Denial of Service | media.rs decode_art and fit_art (decompression bomb or huge art from a misbehaving player) | medium | mitigate | Input stays capped by MAX_ART (8 MiB, unchanged); decode through image::ImageReader with Limits (8192 px a side, 256 MiB), also for the crop branch; at most ART_ATTEMPTS (8) encodes; runs on the media-session worker once per track change, before the Shared lock, never on the hub tick or a UI thread |
| T-uot-02 | Information Disclosure | the new debug and warn lines | low | mitigate | Only lengths, dimensions, attempts and elapsed ms are logged; never title, artist, source app or bytes (T-04-64); the SUMMARY names no track |
| T-uot-03 | Tampering | IPC blob contract between service and renderer | low | mitigate | Compile-time assert THUMBNAIL_TEXT_MAX ≤ BLOB_MAX in data_hub.rs; BLOB_MAX, BLOB_CHUNK, the hub's over-cap guard and the renderer's Reassembler caps are unchanged (Task 1 gates) |
| T-uot-04 | Tampering | maintainer's real data during the Task 3 run | medium | mitigate | Scratch folder target/art-fit written by synth-library; SUCROSE_DATA_DIR with WSLENV; no write to %APPDATA%\Sucrose or H:\Sucrose; the music file is never opened, copied or named; only Rust binaries are stopped by name |
| T-uot-05 | Denial of Service | page-side decode of the re-encoded art | low | accept | The page gets a standard JPEG or PNG below 2 MiB of text, smaller than what the legacy app passed; the existing data-URL path is unchanged |
| T-uot-SC | Tampering | cargo dependencies (T-04-SC) | high | mitigate | No new crate, feature or dev-dependency: image 0.25.10 with jpeg and png is already a sucrose-sysinfo dependency; gates check that Cargo.toml, Cargo.lock and both member manifests are unchanged; noise in tests comes from an in-test LCG |
</threat_model>

<verification>
Run the Task 1 and Task 2 automated blocks again after the last commit; all must exit 0, except the manifest and numstat lines marked "run before the commit", which are re-checked with git diff --name-only d1d76bd7 -- Cargo.toml Cargo.lock crates/sucrose-sysinfo/Cargo.toml apps/sucrose/Cargo.toml limited to this plan's commits. The Task 3 automated block passes on the maintainer run's logs. git --no-pager diff --stat over this plan's commits lists exactly crates/sucrose-sysinfo/src/media.rs, crates/sucrose-sysinfo/src/lib.rs, apps/sucrose/src/data_hub.rs and docs/FEATURES.md. Nothing under src/ (the legacy C# app) changes, and nothing is pushed.
</verification>

<success_criteria>
- Album art over the 2 MiB blob cap reaches the page as a fitting, non-empty ThumbnailString (JPEG quality 90 for opaque art, PNG with alpha for transparent art, own size first, aspect ratio kept). Art that fits is unchanged.
- BLOB_MAX stays 2 MiB and is tied to THUMBNAIL_TEXT_MAX at compile time. No new crate.
- The FEATURES Now playing row records the behavior and the behavior change. Line endings are kept.
- fmt, clippy and the workspace tests pass (with only the known DISPLAY9 exception, if it is attached).
- The maintainer saw the cover of the over-cap FLAC track on the desktop, or the SUMMARY is halted with the skip recorded.
</success_criteria>

<output>
Create .planning/quick/261127-uot-04-29-album-art-over-the-2-mib-blob-max-is-re-encoded-or-dow/261127-uot-SUMMARY.md when done (status complete, or halted when the maintainer skipped Task 3). Include: the test counts of cargo test -p sucrose-sysinfo media and -p sucrose data_hub; from the real run, the text= and fitted= lengths, width, height, attempts and elapsed_ms of the "made smaller" line (sizes only, no track name); the maintainer's verdict quoted verbatim with an English translation; whether DISPLAY9 was attached during cargo test --workspace and which tests failed because of it; the git ls-files --eol lines of the four edited files; and a note for the orchestrator that STATE.md Pending Todos (b) (04-29 album art over the 2 MiB blob cap) can be marked done.
</output>
