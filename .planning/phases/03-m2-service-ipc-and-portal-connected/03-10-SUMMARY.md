---
phase: 03-m2-service-ipc-and-portal-connected
plan: 10
subsystem: ui
tags: [rust, gpui, library, thumbnails, jpeg-decoder, lru-cache, adaptive-grid]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: library scan, sort and display_title in sucrose-core (plan 02), Portal link, settings view and i18n (plan 08), synth-library (plan 05)
provides:
  - "apps/sucrose-portal/src/library: LibraryView (background scan, sort, page state), layout (columns, card_width, row_width, page_count, page_slice), grid, card, thumbs"
  - "thumbs: ThumbCache (64 images / 48 MiB, least recently shown evicted, oversized image returned at once), DecodeQueue (one decode per key, 3 at a time, stale page dropped), decode_thumbnail (DCT scaling, header limit 16384 px, BGRA premultiplied), Bucket, ThumbKey, Release"
  - "LibraryView::{load, update_settings, reload, set_page}; PortalApp owns the entity and feeds it Welcome and Changed"
  - "assets.rs: PortalAssets, the default component icons plus icon_assets! for ImageOff and PackageX"
  - "sucrose_core::library::display_description (localized description, then base, else none)"
  - "log line library loaded (themes, corrupt, ignored); debug line thumbnail cache (images, bytes)"
affects: [03-11, 03-12, 03-14, 03-19, 03-21, 03-24]
actuals:
  tokens: 13400
  tasks: 2
  commits: 4
plan_head_before: 7284540cc705daf42fddbc46a97e248824f28a6a
plan_head_after: b04144cdf5de42f31485efed577ad2ab3344b506
tech-stack:
  added: [jpeg-decoder 0.3.2 (default features off)]
  patterns: [pure grid math and a pure decode queue and cache tested without GPUI, scan and decode on the background executor with a generation counter so a stale scan is ignored, cache keys use the thumbnail mtime read during the background scan so the GPUI thread never stats a file, evicted images are released from the atlas with cx.drop_image]
key-files:
  created:
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/layout.rs
    - apps/sucrose-portal/src/library/grid.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/thumbs.rs
    - apps/sucrose-portal/src/assets.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose-portal/Cargo.toml
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/main.rs
    - crates/sucrose-core/src/library.rs
    - crates/sucrose-core/tests/library.rs
    - docs/FEATURES.md
    - docs/GPUI.md
key-decisions:
  - "The thumbnail is fitted into a bucket-wide square box (360 or 720 px) with its proportions kept and is never enlarged; JPEG is DCT-scaled first, other formats and any JPEG the scaled path cannot take go through the image crate with the same size and allocation limits"
  - "The bucket (W360 or W720) comes from the window scale factor, never from the window width, so a resize never decodes again"
  - "Only the first 64 cards of a page are queued: a page larger than the cache cap would otherwise evict visible cards while decoding the rest and loop"
  - "Cards read the cache in render (get refreshes recency); a miss with no queued decode shows the skeleton, a decode failure shows ImageOff"
  - "Card widths are rounded down to 1/64 px and re-checked so a row never exceeds the content width; a width within 0.01 px below a column boundary counts as reaching it"
requirements-completed: [LIB-04, UI-07]
coverage:
  - id: D1
    description: "Grid math: columns from the legacy formula with the AdaptiveLayout cap, card width capped at 400 px and rows never wider than the content, page count and slices"
    requirement: LIB-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs#layout_ (10 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Thumbnail cache bounded by 64 images and 48 MiB with least-recently-shown eviction, and a decode queue that decodes a key once, three at a time, and drops a left page"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_cache_ and thumbs_queue_ (12 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Card-size decode: JPEG DCT scaling, BGRA output, no enlargement, header limit of 16384 px, malformed input is an error"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_decode_ (4 tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Background scan on the real window shows 40 synthetic themes as a 3-column grid of cards with thumbnails, a gradient title strip, and the red PackageX pill on a newer-version theme"
    requirement: LIB-04
    verification:
      - kind: manual_procedural
        ref: "cargo xtask synth-library --count 40, sucrose-portal.exe --no-focus, capture.ps1: log library loaded themes=40, PNG read"
        status: pass
    human_judgment: true
    rationale: "Thumbnail sharpness and the card look are visual; the capture was inspected by the executor, not by the maintainer"
  - id: D5
    description: "The skeleton grid after 150 ms without a scan result and the skeleton thumbnail while a decode runs"
    requirement: LIB-04
    verification: []
    human_judgment: true
    rationale: "The synthetic scan finishes in milliseconds, so the 150 ms skeleton path was not observed; it needs a slow or large library"
duration: 19 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 10: Library page Summary

**Portal library page: background scan of the library folder, legacy sort, an adaptive card grid, and DCT-scaled card-size thumbnails in a 64-image / 48 MiB cache that releases evicted images from the GPUI atlas**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-24T06:20:10Z
- **Completed:** 2026-11-24T06:39:00Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- `LibraryView` scans `Library.Location` on the background executor, sorts with `LibrarySortMode` and `LibrarySortKind`, logs `library loaded themes=40 corrupt=2 ignored=1` (no names) and shows page 1 (default 30 cards). A scan generation counter drops a stale result; the skeleton grid (columns x 3) is armed with a 150 ms timer.
- The grid follows the legacy layout: 3 columns at the 900 px minimum, up to the `AdaptiveLayout` cap and 400 px cards, a capped row centered, gap = `AdaptiveMargin`, vertical scroll with 16 px padding.
- The card is 160 px high with cover-fit thumbnail, a 56 px black-gradient strip with the title (display_title) over the description (localized, white 82 %), the red PackageX pill with the legacy tooltip on themes for a newer Sucrose, a skeleton while decoding and ImageOff on a failed decode.
- Thumbnails decode off the UI thread, at most three at a time, DCT-scaled for JPEG, limited to 16384 px per side and 64 MiB of file, fitted to the bucket and stored as premultiplied BGRA. The cache evicts the least recently shown by count and bytes and every evicted image goes through `cx.drop_image`. A decode that finishes after its page changed still enters the cache under the caps.
- Release-free numbers of the run: 30 images and 6.9 MB cached for the first page of the synthetic library (debug build); no warnings in the Portal log.

## Task Commits

1. **Task 1: library page tracer (settings view to scan to sort to grid to decode to card)** - `e5d7d870` (feat)
2. **Task 2: bounded cache, queue, layout math** - `69183596` (test, RED) and `4a92add1` (feat, GREEN)
3. **Docs: FEATURES and GPUI gotchas** - `b04144cd` (docs)

**Plan metadata:** added by the close-out commits that follow this file.

## Files Created/Modified

- `apps/sucrose-portal/src/library/mod.rs` - LibraryView entity: settings, background scan, pages, thumbnail requests and results
- `apps/sucrose-portal/src/library/layout.rs` - columns, card_width, row_width, page_count, page_slice and their tests
- `apps/sucrose-portal/src/library/grid.rs` - rows of cards, skeleton grid, scroll area
- `apps/sucrose-portal/src/library/card.rs` - the card, its thumbnail states, strip and pill
- `apps/sucrose-portal/src/library/thumbs.rs` - decoder, ThumbCache, DecodeQueue, Bucket, ThumbKey and tests
- `apps/sucrose-portal/src/assets.rs` - PortalAssets with the ImageOff and PackageX icons
- `apps/sucrose-portal/src/app.rs`, `main.rs` - own the entity, feed it Welcome and Changed, register the assets
- `crates/sucrose-core/src/library.rs` and its test - `display_description`
- `Cargo.toml`, `Cargo.lock`, `apps/sucrose-portal/Cargo.toml` - jpeg-decoder 0.3.2 (default features off), image
- `docs/FEATURES.md`, `docs/GPUI.md` - library rows and the new gotchas

## Decisions Made

See `key-decisions`. Nothing reopens an Accepted entry of `docs/DECISIONS.md`.

## TDD Gate Compliance

Task 2 followed RED then GREEN: `69183596` (test) then `4a92add1` (feat). RED semantic assessment: the 6 thumbs tests and 4 layout tests that failed ran the named targets and failed on the planned assertions (cache did not refresh recency on get, did not enforce the byte cap, kept an oversized image; the queue queued a key twice and kept a left page; columns dropped a column on float noise, a row was wider than the content, card width went negative, page 0 underflowed). Tests that passed at RED (the count-cap eviction, boundary columns, decode) covered behavior the tracer already had. `gsd_run check tdd-red-evidence` has no adapter for libtest output (the documented Rust gap), so the evidence is the cargo test output inspected by hand, not a classifier verdict.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Card icons rendered as nothing**
- **Found during:** Task 1 (capture of the newer-version pill)
- **Issue:** `gpui_kit::assets::Assets` embeds only the default component icons, so PackageX and ImageOff were missing and the pill was an empty red oval.
- **Fix:** `assets.rs` composes `Assets` with `icon_assets!(PageIcons, [ImageOff, PackageX])` and main.rs registers it; the pill then showed its icon in the capture. Recorded in docs/GPUI.md.
- **Files modified:** apps/sucrose-portal/src/assets.rs, apps/sucrose-portal/src/main.rs
- **Committed in:** e5d7d870

**2. [Rule 2 - Missing Critical] Localized description of a theme**
- **Found during:** Task 1 (card strip)
- **Issue:** The plan asks for the description on the card but core only had `display_title`; the base description would ignore a theme's localized text.
- **Fix:** `sucrose_core::library::display_description` (localized, then base, else none) with a test in crates/sucrose-core/tests/library.rs. Both files are outside the plan's file list.
- **Committed in:** e5d7d870

**3. [Rule 2 - Missing Critical] A page larger than the cache cap would evict its own visible cards**
- **Found during:** Task 2 design (the legacy page size goes up to 100, the cap is 64 images)
- **Issue:** decoding all cards of a 100-card page evicts the first ones while the last decode, and the cards of an evicted image fall back to the skeleton.
- **Fix:** only the first 64 cards of a page are queued, so no decode work is wasted and nothing loops; cards 65 to 100 keep the skeleton thumbnail. See Known Stubs.
- **Committed in:** e5d7d870

---

**Total deviations:** 3 auto-fixed (1 bug, 2 missing critical)
**Impact on plan:** none on the contract; the extra files are one new module and one small core function.

## Issues Encountered

- The synthetic library has a newer-version theme only at every 50th index, none in a 40-theme library. The badge was checked on a scratch copy (`target/synth-badge`, library location rewritten to the copy, one theme set to AppVersion 99.1.1.0); the real library and the synth-small output were not touched.
- `cargo fmt` and `git` leave some working-tree files LF and others CRLF in this checkout; the new Rust files follow whatever rustfmt wrote.

## Known Stubs

- The empty and the error state of the library page (zero themes, unreadable folder) render an empty area; their copy and buttons belong to the search and paging plan (03-14). `LibraryView` already reports the `Failed` phase and logs the reason.
- A page with more than 64 cards shows the skeleton thumbnail on cards 65 and up (cache cap, see deviation 3).
- `set_page` and `reload` have no caller yet (pagination bar, retry button); they are marked `allow(dead_code)` with the reason.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 12 (Use from a card) can wrap `card::render_card` and read `LibraryView::themes`; plan 14 adds the page bar over `set_page` and the empty and error states; plan 19 (previews) reuses `ThumbKey`, `Bucket` and `drop_image` for frames; plan 24 measures memory against the 64 / 48 MiB cache.
- Not measured here: the release-build RAM and CPU of the library page (D-25 backstop in plan 24).

## Self-Check: PASSED

All created files exist and the commits e5d7d870, 69183596, 4a92add1 and b04144cd are ancestors of HEAD. `cargo test -p sucrose-portal` (42 tests), `cargo clippy --workspace --all-targets -- -D warnings` and `cargo xtask ci` pass.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
