---
phase: 03-m2-service-ipc-and-portal-connected
plan: 29
subsystem: ui
tags: [image, jpeg, gif, decode-bomb, denial-of-service, gpui]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: thumbnail, background image and preview decoders (03-27)
provides:
  - "thumbs::MAX_PIXELS and thumbs::check_dimensions, one shared pixel-area limit"
  - "thumbs::decode_jpeg_scaled(bytes, (width, height)), the one JPEG fast path"
affects: [03-39, secure-phase]
actuals:
  tokens: 3750
  tasks: 2
  commits: 4
plan_head_before: 9cbb4d07dee8de89a1d851ea80c7e7126e307ed1
plan_head_after: 205ff8dc7299c79fa72118cb43711de77cdb33be
tech-stack:
  added: []
  patterns:
    - "header, limit, decode: every untrusted image header goes through check_dimensions before any decoder allocates"
key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/library/thumbs.rs
    - apps/sucrose-portal/src/background.rs
    - apps/sucrose-portal/src/library/preview.rs
key-decisions:
  - "The area limit is 32 Mi pixels (MAX_PIXELS) next to the per-side MAX_SIDE; about 200 MB for a progressive JPEG instead of 1.6 GB"
  - "A test-only patched JPEG builder lives in thumbs.rs as a cfg(test) pub(crate) function and is shared by the thumbnail and background tests"
requirements-completed: [UI-07, UI-01]
coverage:
  - id: D1
    description: "A 16384 x 16384 JPEG header is refused by the thumbnail path before any decoder allocates"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#a_16384_square_jpeg_header_is_refused_by_the_pixel_area_limit"
        status: pass
    human_judgment: false
  - id: D2
    description: "The background image path and the preview GIF screen use the same limit; the background's own JPEG fast path is gone"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/background.rs#background_refuses_a_16384_square_header_by_the_pixel_area_limit"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/preview.rs#preview_refuses_a_16384_square_screen_by_the_pixel_area_limit"
        status: pass
    human_judgment: false
  - id: D3
    description: "The limit boundary is exact (MAX_PIXELS passes, one row more is refused) and a 1 x 1 image still decodes"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#the_pixel_area_limit_accepts_exactly_max_pixels_and_refuses_one_row_more"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/thumbs.rs#a_one_pixel_image_still_decodes_after_the_pixel_area_limit"
        status: pass
    human_judgment: false
duration: 7min
completed: 2026-11-24
status: complete
---

# Phase 03 Plan 29: Pixel-area limit for untrusted theme images Summary

**One shared 32 Mi pixel area limit (check_dimensions) refuses decode bombs such as a 16384 x 16384 progressive JPEG in the thumbnail, background image and preview GIF paths before any decoder allocates, with one merged JPEG fast path.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-11-24T15:34:00Z
- **Completed:** 2026-11-24T15:41:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `thumbs.rs` has `MAX_PIXELS = 32 * 1024 * 1024` and `check_dimensions(width, height)`, which refuses a side over `MAX_SIDE` or an area over `MAX_PIXELS` with "the image is W x H pixels, over the limit" (CR-01, T-03-66).
- `thumbs::decode_bytes` and `background::decode_bytes` call it right after `into_dimensions`; `preview_size` and `decode_preview` call it on the GIF logical screen (T-03-67). The order is header, limit, decode.
- `decode_jpeg_scaled(bytes, (width, height))` is public in `thumbs.rs`; the thumbnails call it with the bucket width as a square box and the background's private copy is deleted (the decode_jpeg_scaled half of IN-03; the bin_name half stays open and 03-39 records IN-03 as deferred with this partial fix named).
- Five `pixel_area` tests: the patched 16384 x 16384 JPEG header for thumbnails and the background, the patched GIF screen for the preview (it checks `preview_size` and `decode_preview`), the exact boundary (8192 x 4096 passes, 8192 x 4097 and 16385 x 1 are refused) and a 1 x 1 PNG that still decodes. None of them decodes pixels.

## Task Commits

1. **Task 1: thumbnail header limit (tracer)**: RED `6344a0ed` (test), GREEN `1aeb3dfc` (feat)
2. **Task 2: background image and preview GIF, one JPEG fast path**: RED `a6b08232` (test), GREEN `205ff8dc` (feat)

**Plan metadata:** the docs commit that carries this file (docs: complete plan).

## TDD Gate Compliance

- RED 1: with `check_dimensions` a stub returning `Ok`, `the_pixel_area_limit_accepts_exactly_max_pixels_and_refuses_one_row_more` failed on `assertion failed: check_dimensions(8192, 4097).is_err()` (target test, planned assertion, exit non-zero). The two other Task 1 tests were deliberately not run in RED: the patched 16384 x 16384 JPEG would have made the unlimited decoder allocate about 1.6 GB, which is the bug itself; the 1 x 1 test passes before and after by design.
- RED 2: `preview_refuses_a_16384_square_screen_by_the_pixel_area_limit` failed on `assertion failed: preview_size(&bytes, Bucket::W360).is_none()`, its first assertion, so `decode_preview` never ran on the bomb. The background test was not run in RED for the same memory reason.
- Semantic assessment: both RED failures were the planned assertions (the limit missing), not setup, import or fixture faults. GREEN made all five pass.

## Files Created/Modified

- `apps/sucrose-portal/src/library/thumbs.rs` - `MAX_PIXELS`, `check_dimensions`, public `decode_jpeg_scaled`, the cfg(test) `patched_jpeg` helper, module doc names the area limit
- `apps/sucrose-portal/src/background.rs` - area check after `into_dimensions`, shared JPEG fast path, private copy removed, module doc updated
- `apps/sucrose-portal/src/library/preview.rs` - GIF logical screen checked with `check_dimensions` in both functions, module doc updated

## Decisions Made

- The limit is 32 Mi pixels: a progressive JPEG needs about 6 bytes per pixel before scaling, so the worst accepted header costs about 200 MB rather than 1.6 GB, and 8192 x 4096 (an 8K-wide panorama) still passes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] An existing test asserted the old message**
- **Found during:** Task 1
- **Issue:** `thumbs_decode_refuses_an_image_that_declares_more_than_16384_pixels_a_side` expected "pixel limit", which the plan's new message ("over the limit") no longer contains.
- **Fix:** The assertion now expects "over the limit"; the behaviour it tests (a side over MAX_SIDE is refused, MAX_SIDE passes) is unchanged.
- **Files modified:** apps/sucrose-portal/src/library/thumbs.rs
- **Committed in:** 1aeb3dfc

**2. [Plan detail] Preview test GIF patched from a real encoded GIF, and RED partly not run**
- Both are explained under TDD Gate Compliance; no behaviour differs from the plan.

---

**Total deviations:** 1 auto-fixed (1 bug-class test text). **Impact:** none on scope.

## Issues Encountered

None. `cargo xtask ci` (fmt, clippy, tests) passes; `cargo test -p sucrose-portal` reports 194 passed.

## Known Stubs

None.

## Threat Flags

None. No new endpoint, auth path or schema; T-03-66 and T-03-67 are mitigated as planned, and `decode_generic` keeps `Limits::max_alloc`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

CR-01 is closed for thumbnails, background and preview. IN-03 is only half closed here (bin_name remains), which 03-39 records.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
