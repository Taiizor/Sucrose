---
phase: 03-m2-service-ipc-and-portal-connected
plan: 19
subsystem: ui
tags: [gpui, gif, hover-preview, image, render-image, decode-slots]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "Plan 10 thumbnail decode slots, BGRA conversion and drop_image usage; plans 04/06 LibraryPreview and LibraryPreviewHide settings"
provides:
  - "library/preview.rs: capped card-size GIF preview decode (decode_preview, preview_size, load_preview) and the single-live-preview PreviewState"
  - "Hover wiring on library cards: 200 ms dwell, background decode through a shared decode slot, animated image above the thumbnail, release on leave"
  - "Live settings behavior: Library Preview off stops a playing preview, Hide Details hides the strip while it plays"
affects: [03-20, 03-24, library, portal]

actuals:
  tokens: 10800
  tasks: 2
  commits: 3
plan_head_before: 861f0bf7b84476439fccd0d579e08c255658b2c0
plan_head_after: da10e1065ebca0012ba385f7d0e1fbcdc91138a7

tech-stack:
  added: []
  patterns:
    - "Pure generic state machine (PreviewState<I>) for async UI lifecycles, tested without GPUI; the view only applies its returned release obligations"
    - "Non-thumbnail decodes take a shared decode slot through DecodeQueue::reserve/unreserve"

key-files:
  created:
    - apps/sucrose-portal/src/library/preview.rs
  modified:
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/thumbs.rs
    - docs/FEATURES.md
    - docs/GPUI.md

key-decisions:
  - "A capped GIF loops the frames decoded so far (unlike the background image, which falls back to its first frame), as the plan's truths require"
  - "Zero or tiny GIF frame delays play at 100 ms (browser convention) so a preview cannot spin the UI"
  - "Preview decode size keeps the GIF's own proportions at the bucket width (never enlarged) and the card cover-fits it"
  - "A preview that failed to decode is remembered per file until the next rescan, so it is neither retried nor logged again on every hover"

patterns-established:
  - "State-machine results (Finish::Stale) hand images back to be released, so no path can leak atlas memory"

requirements-completed: [LIB-06, UI-07]

duration: 11 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 19: Hover preview Summary

**Hovered library cards play their Preview GIF after a 200 ms dwell, decoded at card size on a background thread within 120 frames and 24 MiB, one preview at a time, released from the GPUI atlas when the pointer leaves.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-24T09:50:42Z
- **Completed:** 2026-11-24T10:01:30Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `decode_preview` produces card-size BGRA frames with the source delays, stops at `MAX_PREVIEW_FRAMES` (120) or `MAX_PREVIEW_BYTES` (24 MiB) before pulling the next frame, keeps what a damaged tail left, refuses GIF logical screens above 16384 px per side and returns `None` for garbage (T-03-44).
- `PreviewState` keeps at most one card waiting, decoding or playing; hovering elsewhere releases the playing image first, and a late result for a card the pointer left is dropped and released.
- Cards show the preview above the thumbnail (which stays underneath); `LibraryPreviewHide` hides the strip while it plays. A GIF that cannot be decoded never starts, shows no message and logs once (warn without the path, debug with it).
- The preview decode takes one of the shared decode slots (`DecodeQueue::reserve`) and is served before queued thumbnails.
- Turning Library Preview off, a page change, a search, a rescan or a hidden window releases the live preview; preview frames never enter `ThumbCache`.

## Task Commits

1. **Task 1: hovered card plays its preview** (tracer, TDD)
   - RED `4fd488e7` (test): stubs plus 14 failing preview tests (11 failed on assertions, 3 trivially passing)
   - GREEN `9b460ac0` (feat): implementation, card hover wiring, decode-slot reservation
2. **Task 2: previews follow the settings live and stay out of the thumbnail cache** - `da10e106` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `apps/sucrose-portal/src/library/preview.rs` - capped GIF decode, `PreviewState`, `Finish`, constants, 14 tests
- `apps/sucrose-portal/src/library/card.rs` - `on_hover`, preview layer, `strip_visible`
- `apps/sucrose-portal/src/library/mod.rs` - single `PreviewState`, dwell timer, preview job and pump priority, stop on page/search/rescan/visibility/setting off
- `apps/sucrose-portal/src/library/thumbs.rs` - `DecodeQueue::reserve`/`unreserve`
- `docs/FEATURES.md` - hover preview row 🟨 with the behavior
- `docs/GPUI.md` - hover preview gotcha

## Decisions Made

See key-decisions. `MAX_PREVIEW_*` and the 200 ms dwell remain the A8 defaults pending plan 24's measurement. `background.rs` keeps its own `GIF_MAX_*` copies of the same numbers (not in this plan's file list); they should be pointed at `preview.rs` constants if A8 changes them.

UI-07 is declared by plans 03-24 and 03-25 as well, which have no SUMMARY yet; per the shared-ID gate it stays Pending in REQUIREMENTS.md until they finish. LIB-06 is marked complete.

## Deviations from Plan

None - plan executed exactly as written. (Small additions inside the plan's intent: `preview_size`/`load_preview` helpers, `DecodeQueue::reserve` to realize "through one of the decode slots", a failed-preview set, and the FEATURES/GPUI doc updates required by CLAUDE.md.)

## Issues Encountered

None. The first clippy run flagged a derivable `Default`, a collapsible `if` and a test-only method; fixed before the GREEN commit.

## Verification

- `cargo test -p sucrose-portal preview`: 17 passed (acceptance: at least 6, then 8); `cargo test -p sucrose-portal thumbs`: 17 passed
- `cargo clippy -p sucrose-portal --all-targets -- -D warnings`: clean
- `cargo xtask ci`: exit 0
- Acceptance greps: `MAX_PREVIEW_FRAMES` 6 hits in preview.rs, `drop_image` in mod.rs, `LibraryPreview` in mod.rs

## Human check (end of phase, plan 20 rows)

Hovering needs real pointer movement, so the visual and memory behavior was not exercised: hover several cards slowly, sweep fast across a row and watch status bar memory in the release Portal with Library Preview and Hide Details on.

## Known Stubs

None.

## Threat Flags

None - the only new surface is the preview GIF decode, already covered by T-03-44 and T-03-45 (only the contained Preview path from the scan is opened).

## Next Phase Readiness

Plan 20 can exercise hover previews on a scratch copy; plan 24 measures the memory effect and may change the caps or dwell.

## TDD Gate Compliance

RED `test(03-19)` `4fd488e7` precedes GREEN `feat(03-19)` `9b460ac0`. RED evidence: `cargo test -p sucrose-portal preview` ran 14 tests, 11 failed on the planned assertions (stub returned None / no-op), semantic assessment: targets executed and failed for the intended reason; no INVALID_RED conditions (the crate compiled). No REFACTOR commit.

## Self-Check: PASSED

- preview.rs, card.rs, mod.rs, thumbs.rs exist; commits 4fd488e7, 9b460ac0, da10e106 are ancestors of HEAD.
