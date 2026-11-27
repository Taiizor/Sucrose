---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 32
subsystem: engines
tags: [apng, webp, gif, image-crate, decoder, fixtures, xtask, built-in-renderer]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-30/04-31 built-in GIF renderer (Animation decoder, GifPump, stretch, Direct2D/GDI presenters) and synth-library --media-fixtures with gif-anim"
provides:
  - "decode::Animation plays GIF, APNG and animated WebP by sniffed bytes: Format {Gif, Apng, Png, WebP}, format(), canvas(), is_single(), MAX_SOURCE_PIXELS, DecodeError::TooLarge"
  - "Message::Still in the engine: a one-frame file is shown once, no frame timer, loop restarts nothing; log line gif opened format=... width=... height=..."
  - "xtask media.rs: animated_gif, apng, animated_webp, fixture builders and cargo xtask media-fixtures"
  - "Committed fixtures anim.gif, anim.png, anim.webp (generated, compared with the generator by an xtask test)"
  - "synth-library --media-fixtures themes apng-anim, webp-anim and video-clip next to gif-anim"
affects: [04-33, 04-37]
actuals:
  tokens: 17000
  tasks: 2
  commits: 2
plan_head_before: 53552894ed6bf1b37ad174f98a2c3d70dae6ba03
plan_head_after: fb25bca4746e4cbe6b5ba82294b344ef55fbe8e0
tech-stack:
  added: []
  patterns:
    - "Format sniffing from bytes, then a per-format image-crate decoder behind one lazy Frames iterator; a still picture is a one-item lazy iterator"
    - "The decoder peeks one frame ahead at open to know whether the file is a single picture; the pump gets Message::Still and arms no timer"
    - "Source size limit read from the GIF, PNG and WebP headers before any decoder is built"
key-files:
  created:
    - xtask/src/media.rs
    - crates/sucrose-engine-gif/tests/formats.rs
    - crates/sucrose-engine-gif/tests/fixtures/anim.gif
    - crates/sucrose-engine-gif/tests/fixtures/anim.png
    - crates/sucrose-engine-gif/tests/fixtures/anim.webp
  modified:
    - xtask/src/main.rs
    - xtask/src/synth_library.rs
    - crates/sucrose-engine-gif/src/decode.rs
    - crates/sucrose-engine-gif/src/engine.rs
    - docs/FEATURES.md
key-decisions:
  - "The frame delay rule is the legacy one for every format: 0 ms plays at 100 ms, any other delay is used as it is; the legacy library has no other threshold"
  - "No new crate: GIF from image's encoder, APNG container written in media.rs with stored deflate blocks, WebP container written around image's lossless VP8L frames"
  - "A single-frame file is detected by a one-frame lookahead at open and sent as Message::Still; no frame timer is armed for it whatever loop says"
requirements-completed: []
duration: 22 min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 32: APNG and WebP on the built-in renderer Summary

**The built-in renderer plays animated PNG and animated WebP as well as GIF through one lazy decoder (bytes decide the format), with the same delay rule, a header-level 8192 x 8192 limit, still pictures shown once without a timer, and generated deterministic fixtures in all three formats.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-11-27T02:31:25Z
- **Completed:** 2026-11-27T02:53:00Z
- **Tasks:** 2
- **Files modified:** 10 (5 created, 5 modified)

## Accomplishments

- Task 1 (tracer): xtask builds an APNG, a synthetic theme carries it, the decoder sniffs it and the renderer loops it. A real run of the service on a scratch library logged `gif opened format=Apng width=480 height=270` and six completed loops in 12 s.
- Task 2: the committed anim.gif, anim.png and anim.webp (64 x 64, four frames, delays 0, 50, 50, 120 ms, the third frame a smaller patch) decode in file order with equal effective delays (the 0 becomes 100 ms) in every format and loop back to frame 0. A one-frame GIF, PNG, APNG and WebP is a single frame. Missing, empty, frameless, header-only and truncated files are errors with reasons; a 9000 x 9000 GIF, PNG or WebP header is `DecodeError::TooLarge`.
- A 120-frame 1920 x 1080 GIF keeps at most two decoded canvases alive (a test-only thread-local counter) and its first frame arrives in under a quarter of the time of the whole pass.
- FEATURES row 14 records the delay rule and the limits.

## Task Commits

1. **Task 1: An APNG theme loops on the built-in renderer** - `ab30de5c` (feat)
2. **Task 2: WebP, the TYPE-02 edges and the memory bound, on committed generated fixtures** - `fb25bca4` (feat)

**Plan metadata:** recorded by the closing docs commit (docs: complete plan)

## Files Created/Modified

- `xtask/src/media.rs` - deterministic GIF, APNG and WebP builders, fixture parts, counter frames, `media-fixtures` command, its tests
- `xtask/src/main.rs` - USAGE line and dispatch of `media-fixtures`
- `xtask/src/synth_library.rs` - `--media-fixtures` writes gif-anim, apng-anim, webp-anim and video-clip from the builders
- `crates/sucrose-engine-gif/src/decode.rs` - format sniffing and per-format decoders, header size check, lookahead, live-canvas counter and tests
- `crates/sucrose-engine-gif/src/engine.rs` - `Message::Still`, `show_still`, the `gif opened` log line, single-frame tests
- `crates/sucrose-engine-gif/tests/formats.rs` and `tests/fixtures/anim.{gif,png,webp}` - format tests and committed generated fixtures
- `docs/FEATURES.md` - row 14 (GIF)

## Decisions Made

- The delay rule is 0 plays at 100 ms; other delays are used as they are (the plan said "or one below the legacy threshold", but XamlAnimatedGif has no such threshold, only the 0 check).
- The decoder peeks one raw frame ahead at open, so a single-frame file is known before its first frame is shown and no timer is armed for it. At most two raw canvases are alive at any time (counted in a test).
- The size limit is read from the file header for all three formats (GIF logical screen, PNG IHDR, WebP VP8X/VP8L/VP8) before any decoder is built, so a header-only hostile file never allocates.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `animated_gif` signature**
- **Found during:** Task 1
- **Issue:** The plan's `animated_gif(frames, size, delay_ms)` cannot express per-frame delays (0, 50, 50, 120) or APNG/WebP sub-frames.
- **Fix:** `animated_gif(frames: &[RgbaImage], delays_ms: &[u32])`, `apng(size, &[Part])`, `animated_webp(size, &[Part])` with `Part { image, x, y, delay_ms }`.
- **Files modified:** xtask/src/media.rs
- **Committed in:** ab30de5c

**2. [Rule 1 - Bug] Test GIF builder overflowed after four frames**
- **Found during:** Task 1 (the 120-frame bound test)
- **Issue:** `fixtures::gif_bytes` took the red channel as `index * 70` in a u8 and panicked past frame 3.
- **Fix:** `index * 70 % 256` (unchanged for the first four frames).
- **Files modified:** crates/sucrose-engine-gif/src/decode.rs
- **Committed in:** ab30de5c

**3. [Rule 2 - Missing critical] Size limit from PNG and WebP headers**
- **Found during:** Task 2
- **Issue:** A header-only 9000 x 9000 PNG or WebP failed with a different error before the size check (a decoder cannot be built from it), so "refused from the header" held only for GIF.
- **Fix:** `declared_size` reads the GIF, PNG and WebP headers from the first 32 bytes before any decoder is built.
- **Files modified:** crates/sucrose-engine-gif/src/decode.rs
- **Committed in:** fb25bca4

**4. [Rule 2 - Missing critical] Single-frame handling in the pump**
- **Found during:** Task 1
- **Issue:** A one-frame file armed the frame timer and looped, redrawing frame 0 forever, against the plan's "shown once and starts no frame timer".
- **Fix:** `Message::Still` plus `GifPump::still`: shown once, no timer, loop on or off restarts nothing.
- **Files modified:** crates/sucrose-engine-gif/src/engine.rs, decode.rs
- **Committed in:** ab30de5c

---

**Total deviations:** 4 auto-fixed (1 bug, 2 missing critical, 1 blocking)
**Impact on plan:** All needed for the plan's stated behavior; no scope creep. `image` decodes GIF lazily (the A6 assumption holds), so no `gif` dependency and no T-04-SC checkpoint was needed.

## Issues Encountered

- `apps/sucrose` test `service::tests::a_restart_of_another_engine_closes_nothing_and_one_during_a_library_move_is_refused` failed in one `cargo xtask ci` run and in one of three isolated runs, and passed on the rerun of `cargo xtask ci` (exit 0). service.rs is untouched by this plan; logged in deferred-items.md.
- The 120-frame 1920 x 1080 bound test takes about 18 s in a debug build (frame decoding dominates); the file is built cheaply by repeating one encoded frame block.

## Line endings

All touched Rust files and the new files are LF in the working copy and were left as they were (`cargo fmt` changed no endings). `docs/FEATURES.md` stayed `w/crlf` with no newline after its last line (checked with `git ls-files --eol` before the commit). No file needed CRLF restored.

## Known Stubs

None. The `video-clip` fixture theme holds placeholder video bytes on purpose (04-33 plays it as a generated page; a checkpoint copies a real clip over it).

## Threat Flags

None. T-04-69 is mitigated as planned: header size cap before decoding, two canvases alive, truncated files are errors, all pinned by tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-33 can use the `video-clip` theme. TYPE-02 completes at 04-37; `requirements-completed` stays empty here as the plan says.

## Self-Check: PASSED

All created files exist and both task commits (`ab30de5c`, `fb25bca4`) are ancestors of HEAD. `cargo xtask ci` exit 0 on the final tree.
