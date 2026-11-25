---
phase: 03-m2-service-ipc-and-portal-connected
plan: 42
subsystem: ui
tags: [gpui, gpui-kit, library, thumbnails, scroll, scrollbar, cache]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "ThumbCache, DecodeQueue and the 64-image / 48 MiB caps (03-10, D-25); atlas::release_images (03-27); the page shown debug line (03-36, 03-40)"
provides:
  - "layout::rows_in_view and layout::cards_to_request: scroll offset to the card positions to request, pure arithmetic"
  - "ThumbCache::peek, touch, insert_keeping and trim: reads without reordering and eviction that never drops a wanted thumbnail"
  - "LibraryView::request_visible and request_in_view: thumbnails requested from render for the rows in view plus one row ahead"
  - "A vertical scrollbar on the library grid and Thumb::Idle, a static placeholder for cards that are not loading"
affects: [03-43, 03-44]

actuals:
  tokens: 10915
  tasks: 2
  commits: 4
plan_head_before: c447c0e5da3065879529dcbccb9d190f21b7d43e
plan_head_after: 53407a5b71a727f78dca87e4b1d373a5e4cc0221

tech-stack:
  added: []
  patterns:
    - "A GPUI view that owns its ScrollHandle reads offset and bounds in render and requests work only when the wanted positions change; no scroll listener (an overflow-scroll div notifies its view on every wheel step)"
    - "A bounded cache that exceeds its caps only for the entries a caller keeps, and trims back when they are no longer wanted"
    - "overflow_y_scroll().track_scroll(&handle).vertical_scrollbar(&handle) when the view sets the offset itself"

key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/library/layout.rs
    - apps/sucrose-portal/src/library/thumbs.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/grid.rs
    - docs/GPUI.md
    - docs/FEATURES.md

key-decisions:
  - "Thumbnails follow the scroll position (rows in view plus LOOKAHEAD_ROWS = 1) instead of the first 64 cards of the page; CACHE_IMAGES and CACHE_BYTES stay 64 and 48 MiB"
  - "The first request uses the window height while the scroll bounds are still zero, so the first frame asks for the top cards and a later frame corrects it"
  - "ThumbCache::get and insert are kept for the existing tests and compiled for tests only; the page uses peek, touch and insert_keeping"

patterns-established:
  - "The 'page shown' debug line stays one per page change (request_page logs and forgets); the new 'thumbnails requested' line carries numbers only"

requirements-completed: [LIB-04, LIB-06, UI-07]

coverage:
  - id: D1
    description: "Thumbnails are requested for the cards in view plus one row ahead from the scroll offset and bounds, at any page size, so cards past the 64th of a 100-card page get their thumbnails"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs#layout_rows_in_view_scrolled_to_the_screenshot"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs#layout_cards_to_request_puts_the_visible_rows_first"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_scrolling_a_100_card_page_keeps_every_wanted_card"
        status: pass
    human_judgment: true
    rationale: "Scrolling a 100-card page by wheel or scrollbar was simulated, not driven (no input synthesis); the maintainer confirms it by hand in 03-43"
  - id: D2
    description: "The cache never evicts a wanted thumbnail, holds more than its caps only while more cards are on screen, trims back when they scroll away, and cards() reads without reordering"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_cache_insert_keeping_never_evicts_a_kept_key"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_wanted_cards_over_the_byte_cap_stay_until_they_scroll_away"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#thumbs_cache_peek_does_not_count_as_shown"
        status: pass
    human_judgment: true
    rationale: "The D-25 memory rule on this path is measured in 03-44"
  - id: D3
    description: "Cards nothing was requested for show a static placeholder (Thumb::Idle) and only wanted cards waiting for a decode animate"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/mod.rs#library_thumb_for_picks_ready_failed_loading_or_idle"
        status: pass
    human_judgment: false
  - id: D4
    description: "The library grid shows a vertical scrollbar bound to the ScrollHandle that set_page resets, following the OS scrollbar mode"
    requirement: LIB-04
    verification:
      - kind: other
        ref: "grep -c vertical_scrollbar apps/sucrose-portal/src/library/grid.rs"
        status: pass
    human_judgment: true
    rationale: "The scrollbar is hidden until hover in the default Windows mode, so a capture without a pointer cannot show it; the maintainer checks it in 03-43"
  - id: D5
    description: "With Library Preview on, hovering a card past the 64th plays its preview (the hover path has no page-size dependency)"
    requirement: LIB-06
    verification: []
    human_judgment: true
    rationale: "The reported missing preview was the maintainer's LibraryPreview=false setting; 03-43 re-runs it with the preview on"

duration: 13 min
completed: 2026-11-25
status: complete
---

# Phase 3 Plan 42: Service, IPC and Portal connected (library thumbnails follow the scroll) Summary

**Library thumbnails are requested for the rows in view plus one row ahead from the grid's ScrollHandle (layout::rows_in_view, cards_to_request), the cache keeps every wanted thumbnail (insert_keeping, trim, peek) under unchanged 64 / 48 MiB caps, the grid has a vertical scrollbar, and cards that are not loading show a static placeholder.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-25T13:41:44Z
- **Completed:** 2026-11-25T13:55:13Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- G-03-6 and G-03-8: `request_page` no longer queues the first 64 cards; the next render computes the wanted positions from the scroll offset, the scroll bounds (the window height before the first paint), the fixed 160 px card height and the margin, and requests only when they change.
- `ThumbCache` gained `peek`, `touch`, `insert_keeping` and `trim`; a wanted thumbnail is never evicted, the cache exceeds its caps only while more wanted cards than that are on screen, and `trim` brings it back. `insert` delegates with a keep that keeps nothing.
- `cards()` reads with `peek`, so drawing a page no longer reorders the cache.
- `library-scroll` gets `.vertical_scrollbar(scroll)` on the same handle `set_page` resets; `Thumb::Idle` (static `theme.skeleton` fill) replaces the endless Skeleton for cards that are not wanted.
- GPUI.md records the in-view request rules and the scrollbar handle rule; FEATURES.md notes the scrollbar and scrolling thumbnails.

## Release first-screen evidence (synthetic library, 120 themes, LibraryPagination 100, 900 px window, Backdrop None)

From `target/thumbs-check/portal-first-screen.log` (untracked, gitignored):

```
DEBUG sucrose_portal::library: page shown page=1 pages=2 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=38 heap_committed_mb=40 gpu_local_mb=28 gpu_non_local_mb=0
DEBUG sucrose_portal::library: thumbnails requested first=0 last=14 wanted=15 queued=15
DEBUG sucrose_portal::library: thumbnails requested first=0 last=11 wanted=12 queued=6
DEBUG sucrose_portal::library: thumbnail cache images=12 bytes=3507840
```

The first request (15) uses the window height; the second (12) uses the measured scroll bounds. The cache holds the 12 decoded first-screen thumbnails, not the 100 of the page. The second run (after Task 2, Idle placeholder and scrollbar) logged the same requests (wanted 15, then 12) and `thumbnail cache images=12`. The capture `target/thumbs-check/first-screen.png` (916x608) shows the first two rows of cards with their thumbnails and the third row partly; the scrollbar is not visible without a hover, which is the OS mode (D4). `tasklist.exe` showed no Rust Sucrose process afterwards.

## Task Commits

1. **Task 1 (tracer): thumbnails requested for the rows in view, cache keeps wanted ones**
   - `cf375476` (test): layout and ThumbCache stubs with failing tests (assertion failures, 10 layout and 8 thumbs tests red)
   - `8577cf28` (feat): rows_in_view, cards_to_request, peek/touch/insert_keeping/trim, request_visible/request_in_view in LibraryView
2. **Task 2: scrollbar, static placeholder, docs**
   - `8faec395` (test): Thumb::Idle and a thumb_for stub with the failing library_thumb_for_picks_ready_failed_loading_or_idle
   - `53407a5b` (feat): thumb_for picks Idle, vertical_scrollbar in scroll_area, GPUI.md and FEATURES.md

**Plan metadata:** the docs(03-42) commit that holds this SUMMARY, STATE.md, ROADMAP.md and REQUIREMENTS.md.

## Files Created/Modified

- `apps/sucrose-portal/src/library/layout.rs` - rows_in_view and cards_to_request with tests
- `apps/sucrose-portal/src/library/thumbs.rs` - peek, touch, insert_keeping, trim; insert delegates; simulation tests (CRLF kept)
- `apps/sucrose-portal/src/library/mod.rs` - wanted/requested fields, LOOKAHEAD_ROWS, request_visible, request_in_view, thumb_for, insert_keeping in on_decoded, peek in cards
- `apps/sucrose-portal/src/library/card.rs` - Thumb::Idle layer
- `apps/sucrose-portal/src/library/grid.rs` - vertical_scrollbar on library-scroll (CRLF kept)
- `docs/GPUI.md`, `docs/FEATURES.md` - rules and feature row (CRLF kept, FEATURES.md still has no final newline)

## Decisions Made

See key-decisions above. The caps were not raised (prohibition kept): only thumbnails on screen may hold the cache above them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ThumbCache::get and insert are test-only**
- **Found during:** Task 1 (clippy -D warnings)
- **Issue:** The page now reads with peek/touch and inserts with insert_keeping, so `get` and `insert` became dead code outside tests and `cargo clippy -D warnings` fails. The plan says to keep them so the existing thumbs_cache_* tests stay as they are.
- **Fix:** Marked both `#[cfg(test)]` with a doc line pointing at the replacements.
- **Files modified:** apps/sucrose-portal/src/library/thumbs.rs
- **Committed in:** 8577cf28

**2. [Rule 1 - Bug] rows_in_view doc comment tripped clippy::doc_lazy_continuation**
- **Found during:** Task 1
- **Fix:** Rewrote the doc paragraph (a line starting with `+` read as a list item).
- **Committed in:** 8577cf28

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact:** none on behavior or scope.

## Issues Encountered

- A first edit through the helper wrote `thumbs.rs` with LF endings (the file reader normalizes CRLF); noticed before any commit, converted back to CRLF, and every later edit and `cargo fmt` run was followed by an `git ls-files --eol` check. The RED commit diff for thumbs.rs is insertions only.
- Not driven: a real wheel or scrollbar scroll of a 100-card page and a hover preview past the 64th card (no input synthesis). The scroll path is covered by the simulation test; the maintainer re-checks both in 03-43. The D-25 measurement of this path is 03-44.

## TDD Gate Compliance

Task-level TDD (plan type execute). Task 1 RED `cf375476`: the stubs compile and the 10 layout_ and 8 thumbs_ tests fail on their assertions (for example `left: 0..0 right: 20..23`, not on compile errors); GREEN `8577cf28` passes all 46. Task 2 RED `8faec395`: `assertion failed: matches!(thumb_for(None, false, false), Thumb::Idle)`; GREEN `53407a5b` passes. `cargo xtask ci` exits 0 (269 + 26 + 25 tests).

## Known Stubs

None.

## Threat Flags

None. T-03-91..94 mitigations are in place: wanted-only cache growth with `trim`, static Idle fill, numbers-only log line, synthetic data with DeleteCorrupt false and every started app stopped.

## Next Phase Readiness

03-43 re-runs the maintainer checks (scroll a 100-card page, scrollbar, hover preview with Library Preview on); 03-44 measures the D-25 rule on the in-view path.

## Self-Check: PASSED

All created or modified files exist, commits cf375476, 8577cf28, 8faec395 and 53407a5b are ancestors of HEAD, `commits: 4` was measured from the plan ledger, and no Rust Sucrose process is left running.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-25*
