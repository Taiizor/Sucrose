---
phase: 03-m2-service-ipc-and-portal-connected
plan: 46
subsystem: ui
tags: [gap-closure, gpui, scrollbar, scroll-handle, thumbnails, library-grid]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "In-view thumbnail requests and the library ScrollHandle of 03-42; the maintainer's finding in 03-43 that the grid showed no scrollbar (WR-15, G-03-6)"
provides:
  - "grid::scroll_area: the vertical Scrollbar is an absolute inset_0 sibling of the scroll area inside a non-scrolling relative wrapper, on the same ScrollHandle (WR-15)"
  - "layout::content_height and layout::clamped_scroll_top: the scroll top limited to the content of the rows being requested"
  - "grid::visible_rows: the rows in view from a clamped offset, plus a Window::on_next_frame check that notifies the view when the painted bounds or offset differ from what render read (WR-16)"
affects: [03-47, 03-50, library, portal-ui]

actuals:
  tokens: 8870
  tasks: 3
  commits: 3
plan_head_before: 1b90b6dc5f98181332c2ef7b731f6ce9276d6210
plan_head_after: 41e18030ed7c1c510734b17faef3054d2457c8f8

tech-stack:
  added: []
  patterns:
    - "A scrollbar for a handle the view owns goes in a sibling overlay of the scroll area, never on the scrolled element (GPUI offsets every child of a scroll container)"
    - "Render-time reads of a ScrollHandle lag a frame: clamp them to the content and settle with Window::on_next_frame, comparing the raw reads exactly so the check stops after one extra frame"

key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/library/grid.rs
    - apps/sucrose-portal/src/library/layout.rs
    - apps/sucrose-portal/src/library/mod.rs
    - docs/GPUI.md
    - docs/FEATURES.md

key-decisions:
  - "The scrollbar is mounted as a sibling overlay (the shape of gpui-component's own Scrollable), not through vertical_scrollbar on the scrolled element, which moves away with the cards"
  - "No scrollbar mode is forced and no test-only flag is added: the bar follows Theme.scrollbar_mode as gpui-component syncs it from the OS"
  - "visible_rows compares the raw handle reads with exact Pixels equality (never the clamped value, no tolerance), so a still window draws one extra frame at most"

patterns-established:
  - "GPUI layout tests drive renders only through notifications and Window::simulate_next_frame, never window.draw, so a missing next-frame check cannot be hidden"

requirements-completed: [LIB-04, UI-07]

coverage:
  - id: D1
    description: "The library grid's scrollbar overlay keeps the scroll area's bounds and right edge while the cards scroll, does not block clicks on the cards, and stays put at the page end and on short pages"
    requirement: LIB-04
    verification:
      - kind: automated_ui
        ref: "apps/sucrose-portal/src/library/grid.rs#grid_scrollbar_stays_on_the_right_edge_while_the_cards_scroll"
        status: pass
      - kind: automated_ui
        ref: "apps/sucrose-portal/src/library/grid.rs#grid_scrollbar_overlay_lets_clicks_reach_the_cards"
        status: pass
      - kind: automated_ui
        ref: "apps/sucrose-portal/src/library/grid.rs#grid_scrollbar_page_end_and_short_pages_do_not_move"
        status: pass
    human_judgment: false
  - id: D2
    description: "The scrollbar is visible on the maintainer's desktop in both OS scrollbar modes (hover and always shown)"
    requirement: UI-07
    verification: []
    human_judgment: true
    rationale: "Visibility under the OS auto-hide setting and on the real window is a judgment of the maintainer's eye; it is the check of plan 03-47"
  - id: D3
    description: "After a content shrink or a taller window the cards in view still get their thumbnails without another wheel step (clamped offset, next-frame settle)"
    requirement: LIB-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs#layout_clamped_scroll_top_keeps_the_view_inside_the_content"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs#layout_stale_offset_past_the_content_still_requests_the_last_rows"
        status: pass
      - kind: automated_ui
        ref: "apps/sucrose-portal/src/library/grid.rs#grid_request_settles_after_a_content_shrink"
        status: pass
      - kind: automated_ui
        ref: "apps/sucrose-portal/src/library/grid.rs#grid_request_settles_after_the_window_grows"
        status: pass
    human_judgment: false
  - id: D4
    description: "The release Portal still requests the first screen of the 500-theme library"
    requirement: LIB-04
    verification:
      - kind: other
        ref: "target/grid-smoke.txt and target/grid-smoke-2.txt: 'thumbnails requested first=0', 'page shown page=1 pages=17'"
        status: pass
    human_judgment: false

duration: 12 min
completed: 2026-11-25
status: complete
---

# Phase 3 Plan 46: Library scrollbar mount and settled in-view requests Summary

**Library grid scrollbar mounted as a sibling overlay on the grid's own ScrollHandle (stays on the right edge while the cards scroll), plus a clamped offset and a next-frame settle for the rows in view, all proven in GPUI layout tests**

## Performance

- **Duration:** 12 min
- **Started:** 2026-11-25T16:54:00Z
- **Completed:** 2026-11-25T17:07:00Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- WR-15: `grid::scroll_area` is now a non-scrolling `relative` wrapper holding the `library-scroll` scroll area and an `absolute().inset_0()` `library-scrollbar` overlay (`Scrollbar::new(scroll).axis(ScrollbarAxis::Vertical).viewport_from_layout()`) painted after the cards, bound to the handle that `set_page` resets and `request_visible` reads. In the GPUI test a wheel step of 200 px moves the last row up by exactly 200 px while the overlay's bounds stay exactly equal to the scroll area's (exact `Pixels` equality).
- The overlay does not block the cards (a click left of the track reaches row 0's `on_click`), scrolled past the end the content stops at its last row, and a page shorter than the viewport does not move.
- WR-16: `layout::clamped_scroll_top` limits the offset to the content of the requested rows, so an offset kept after a shrink (row 20's offset on a 12-row page) asks for the last rows (9..12) instead of none; `grid::visible_rows` clamps and, through `Window::on_next_frame`, notifies the view once when the handle's raw bounds or offset differ from what render read. `request_visible` takes its rows from it and no longer reads the handle itself.
- The GPUI tests `grid_request_settles_after_a_content_shrink` (30 rows scrolled to the end, then 12 rows: range 9..12) and `grid_request_settles_after_the_window_grows` (400 to 800 px: 0..3 becomes 0..5 with no input) both end with the settled check, so a still window draws no extra frame.
- docs/GPUI.md says how to mount a scrollbar for a view-owned handle and why render-time handle reads lag a frame; docs/FEATURES.md (Pagination row) says the bar follows the OS scrollbar setting and that its desktop visibility is checked in 03-47.

## Task Commits

1. **Task 1: scrollbar as a sibling overlay (tracer)** - `260765c6` (fix)
2. **Task 2: click-through, page end and short page tests; docs** - `12563795` (test)
3. **Task 3: clamped offset and next-frame settle (WR-16)** - `41e18030` (fix)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Release smoke (500-theme synthetic library, 15 s, `--no-focus`)

After Task 1 (`target/grid-smoke.txt`):

```
2026-11-25T16:57:17.031647Z DEBUG sucrose_portal::library: thumbnails requested first=0 last=11 wanted=12 queued=12
2026-11-25T16:57:17.024521Z DEBUG sucrose_portal::library: page shown page=1 pages=17 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=39 heap_committed_mb=41 gpu_local_mb=33 gpu_non_local_mb=0
```

After Task 3 (`target/grid-smoke-2.txt`):

```
2026-11-25T17:03:01.422749Z DEBUG sucrose_portal::library: thumbnails requested first=0 last=11 wanted=12 queued=12
2026-11-25T17:03:01.422272Z DEBUG sucrose_portal::library: page shown page=1 pages=17 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=39 heap_committed_mb=42 gpu_local_mb=33 gpu_non_local_mb=0
```

Both equal 03-44's smoke ("thumbnails requested first=0 last=11 wanted=12 queued=12"); the measured-bounds request after the next-frame check asked for the same positions, so it logged no second line. `target/synth-500/Log` was removed after each run (03-50 starts clean) and `tasklist.exe` showed no Rust Sucrose process afterwards.

In `grid_request_settles_after_the_window_grows` the range of the render right after the resize was 0..3 (the stale bounds), as expected; the settle brought it to 0..5.

## Files Created/Modified

- `apps/sucrose-portal/src/library/grid.rs` - scroll_area with the sibling overlay, `visible_rows`, five GPUI tests
- `apps/sucrose-portal/src/library/layout.rs` - `content_height`, `clamped_scroll_top`, two layout tests
- `apps/sucrose-portal/src/library/mod.rs` - `request_visible` uses `grid::visible_rows`
- `docs/GPUI.md` - library grid scrolling bullet (scrollbar mount, WR-16 lag and settle)
- `docs/FEATURES.md` - Pagination row (scrollbar per OS setting, visibility pending 03-47)

## Decisions Made

- Sibling overlay mount, as gpui-component's own `Scrollable` does; no scrollbar mode forced, no test-only flag.
- Exact raw-read comparison in the next-frame check (never the clamped value) so no frame loop can form.

## Deviations from Plan

None - plan executed exactly as written. Two housekeeping notes: clippy's `needless_lifetimes` and `print_stdout` lints flagged a helper signature and a debug `println!` in the new tests, both fixed before the task commit; `cargo fmt --all` rewrites grid.rs with LF, so the working copy was converted back to CRLF after each format run (the index stays LF, the commits carry no line-ending change).

## Issues Encountered

None. `cargo xtask ci` (fmt, clippy, test) exits 0.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 03-47 can judge the scrollbar's visibility on the desktop (hover and "Always show scrollbars" modes) and the cards after a resize; 03-50 reruns the memory measurement on this final path.

## Self-Check: PASSED

- grid.rs, layout.rs, mod.rs, docs/GPUI.md, docs/FEATURES.md exist; commits 260765c6, 12563795 and 41e18030 are ancestors of HEAD; the 7 new tests pass; `cargo xtask ci` exit 0.
