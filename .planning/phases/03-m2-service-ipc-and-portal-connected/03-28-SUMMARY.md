---
phase: 03-m2-service-ipc-and-portal-connected
plan: 28
subsystem: ui
tags: [gpui, img, element-id, hover-preview, tracing, gap-closure, lib-06]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "atlas::release_image (03-27), preview decode caps and hover state (03-19, 03-25)"
provides:
  - "Card preview img() carries the stable element id (\"library-card-preview\", index), so GPUI keeps its frame index and animates the hover preview"
  - "GPUI tests card_preview_paints_its_second_frame and card_preview_without_an_id_stays_on_frame_zero"
  - "Debug log events: \"hover preview not started\" (reason), \"preview playing\" (frames, reduce_motion), \"preview decoded for a card the pointer left\""
  - "docs/GPUI.md rule on multi-frame img animation; docs/FEATURES.md note on the OS Animation effects switch"
affects: [03-37, 03-38, 03-39, 03-40]

actuals:
  tokens: 5609
  tasks: 3
  commits: 3
plan_head_before: f05f11369df964e6d6c304ef3432466120596c3b
plan_head_after: 1c4d83920693073a070c96d94b93839bbb5c5e67

tech-stack:
  added: []
  patterns: ["a multi-frame RenderImage in img() needs an element id, an active window and App::reduce_motion false"]

key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - docs/GPUI.md
    - docs/FEATURES.md

key-decisions:
  - "Fix the missing element id instead of forcing animation: GPUI follows the OS Animation effects switch, recorded as a behavior change in docs/FEATURES.md"
  - "Hover diagnostics log reasons and counts only, at debug level (T-03-63)"

requirements-completed: [LIB-06]

duration: 42 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 28: Hover preview animates Summary

**The library card preview `img()` now carries the element id ("library-card-preview", index), so GPUI advances the GIF frames: a test proves the second frame is painted, the hover path logs why a preview did not start, and the maintainer confirmed animation, strip hiding and a quiet fast sweep on the release Portal.**

## Performance

- **Duration:** about 42 min wall clock (includes the maintainer's hover run)
- **Started:** 2026-11-24T16:05:00Z
- **Completed:** 2026-11-24T16:47:00Z
- **Tasks:** 3 (one tracer with a RED/GREEN pair, one auto, one human-action checkpoint)
- **Files modified:** 4

## Accomplishments

- Root cause of LIB-06 gap 1: the preview `img()` had no element id. GPUI keeps the frame index as element state keyed by the global id and `request_animation_frame` needs it, so the hovered card stayed on frame 0. Adding `id(("library-card-preview", index))` fixes it.
- Two GPUI tests pin the behavior: a two-frame preview with the id paints both frames in an active window (both frames in the window atlas); the same image without an id stays on frame 0.
- Every early return of `hover_card` and the `on_dwell` branch leave a debug line "hover preview not started" with a reason; a playing preview logs "preview playing" with its frame count and the reduced-motion flag. No folder names, titles or paths are logged.
- docs/GPUI.md states the three conditions for img animation and replaces the 03-25 open-defect note; docs/FEATURES.md records that the preview shows only its first frame when the OS Animation effects switch is off.

## Task Commits

1. **Task 1 RED: failing GPUI test for the card preview's second frame** - `3925f9d3` (test)
2. **Task 1 GREEN: element id on the preview img()** - `870c5d99` (feat)
3. **Task 2: hover path logs why a preview did not start; GPUI.md and FEATURES.md notes** - `1c4d8392` (feat)
4. **Task 3: maintainer hover run** - human-action checkpoint, no code commit

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

- `apps/sucrose-portal/src/library/card.rs` - preview element id, the two frame tests
- `apps/sucrose-portal/src/library/mod.rs` - hover and decode diagnostics at debug level
- `docs/GPUI.md` - img animation rule (CRLF kept)
- `docs/FEATURES.md` - Animation effects behavior note (CRLF and missing final newline kept)

## Hover run (Task 3, LIB-06 confirmation)

The first executor prepared the release build, a scratch data folder (`target/hover-check`, Library Preview and Hide Details on in the scratch `Sucrose.json` only) and a focused release Portal with `SUCROSE_LOG="info,sucrose_portal::library=debug"`. The maintainer hovered cards by hand on DISPLAY1; no input was synthesized.

Maintainer's reply, verbatim:

> animasyon oynuyor. şerit gizleniyor. hızlı geçişte gif oynamıyor

English rendering: "the animation plays. the strip hides. on a fast sweep the gif does not play." That is: animates yes, strip hides yes, fast sweep started nothing yes. Nothing odd reported.

Portal log counts (`target/hover-check/Log/Portal.2026-11-24.log`):

| Event | Count |
|-------|-------|
| `preview playing` | 27 (sampled lines: `frames=16 reduce_motion=false`) |
| `hover preview not started` (any reason) | 0 |
| `preview decoded for a card the pointer left` | 17 |

The 17 stale decodes are previews that finished after the pointer had moved on; they were discarded, as designed. Zero "not started" lines: with Library Preview on, no early return was hit during the run, and the fast sweep produced no preview because the dwell timer never completed (nothing started, as the maintainer saw). `reduce_motion=false` matches the plan-time check that Animation effects is on.

Both apps were stopped with `taskkill.exe` (portal PID 38740, service PID 31912, renderers 52780 and 21648) and `tasklist.exe` showed no Rust Sucrose process afterwards. `target/hover-check` is gitignored scratch data.

**LIB-06 stays Pending in .planning/REQUIREMENTS.md** (as the plan requires); 03-39 marks it Complete, citing this confirmation.

## Decisions Made

- Fixed the cause (missing element id) rather than overriding the OS reduced-motion preference; the behavior difference from the WPF app is documented in docs/FEATURES.md.
- Diagnostics are debug-level, values only, so the D-25 reruns can read why a preview did not play without leaking theme content.

## Deviations from Plan

None - plan executed exactly as written. The checkpoint pause and continuation were the planned human-action gate.

## Issues Encountered

None. `cargo xtask ci` (fmt, clippy, tests) passed at the end of the plan.

## Authentication Gates

None.

## Known Stubs

None.

## Threat Flags

None. T-03-63 (no paths in logs), T-03-64 (frames leave the atlas through 03-27's release helper) and T-03-65 (scratch data folder, real library read-only) were honored.

## Next Phase Readiness

- 03-39 can mark LIB-06 Complete on this SUMMARY's maintainer confirmation after the D-25 reruns.
- 03-37 / 03-38 measurement rounds can use the "preview playing" and "hover preview not started" debug lines.

## Self-Check: PASSED

- Commits 3925f9d3, 870c5d99, 1c4d8392 are ancestors of HEAD; `git rev-list --count f05f1136..HEAD` is 3.
- `library-card-preview` present in card.rs; `hover preview not started` and `preview playing` present in mod.rs; docs/GPUI.md and docs/FEATURES.md still CRLF.
- `cargo xtask ci` passed; no Rust Sucrose process left running.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
