---
phase: 03-m2-service-ipc-and-portal-connected
plan: 14
subsystem: ui
tags: [rust, gpui, portal, pagination, search, library, header, empty-states]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: LibraryView, layout::{page_count, page_slice, columns}, library::search and sort, the Portal settings broadcast (plans 02, 10, 12)
provides:
  - "pagination.rs: page_buttons (legacy nine-slot scheme), PageButton, clamp_jump, jump_text_allowed and the bar (Previous, pages, Next, digit-limited numeric field, Jump), composed from Buttons and an Input"
  - "header.rs: Header entity with the Library tab and the search box (320 px shrinking to 200 px, 20 characters, clear button, Escape clears, 500 ms debounce, lowercased); normalize_query; Ctrl+F focus from anywhere in the window"
  - "LibraryView::set_query and set_page over a filtered index (visible), page_after_settings_change (page 1 on a new page size or sort order, page kept on a layout change)"
  - "grid::library_state (LoadError, Empty, SearchEmpty, Grid) and the three state pages with the legacy keys; Try Again rescans"
affects: [03-17, 03-18, 03-20]
actuals:
  tokens: 12735
  tasks: 2
  commits: 5
plan_head_before: 94135d267699cf1ab530b1680f97186bf28a96f9
plan_head_after: 4e1a6aa9389171dc73385701c100d55f60d60c3f
tech-stack:
  added: []
  patterns:
    - "the library keeps one index of visible theme positions (search result order) between the scanned list and the page slice, so search, sort and paging share one path"
    - "a pure function decides what a settings change does to the page (page_after_settings_change) and the view only applies it"
    - "a limit that follows the data (digits of the last page) rides an Rc<Cell> read by the Input validator"
key-files:
  created:
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/library/pagination.rs
  modified:
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/src/library/grid.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
    - docs/GPUI.md
key-decisions:
  - "The pagination bar is composed from Buttons and an Input: gpui-component's Pagination has no jump field and no fixed nine-slot scheme (RESEARCH Pattern 7); the legacy scheme is reproduced slot for slot, including the two adjacent ellipsis slots"
  - "The 20-character limit of the search box is an Input validator (an edit past 20 is refused, a too long paste is refused whole) because InputState has no max length"
  - "An emptied search box (clear button, Escape, whitespace only) applies at once; only typed text waits for the 500 ms debounce"
  - "Ctrl+F is observed with cx.observe_keystrokes on the root entity so it works wherever the focus is; the modifier is secondary (Cmd on macOS)"
requirements-completed: [LIB-03, LIB-04, UI-03]
coverage:
  - id: D1
    description: "The library is paginated with the legacy page-button scheme; Jump clamps to 1..last page, an empty field does nothing, the field takes at most the digits of the last page number; the bar is hidden at one page or fewer"
    requirement: LIB-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/pagination.rs#pagination_scheme_near_the_start, pagination_scheme_in_the_middle, pagination_scheme_near_the_end, pagination_never_shows_more_than_nine_slots_and_always_the_first_and_last_page, pagination_jump_clamps_into_the_page_range, pagination_jump_field_takes_at_most_the_digits_of_the_last_page"
        status: pass
    human_judgment: false
  - id: D2
    description: "A new page size or sort order returns to page 1 and a layout change keeps the page, so the library re-pages and re-lays out at once when the service broadcasts"
    requirement: LIB-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/mod.rs#a_new_page_size_returns_to_page_one_with_the_new_page_count, a_new_sort_order_returns_to_page_one, a_new_layout_keeps_the_page, an_unchanged_page_is_clamped_into_the_range"
        status: pass
    human_judgment: false
  - id: D3
    description: "The bar looks like the legacy one under the grid (Previous disabled, pages, Next, numeric field, Jump) and the header shows the Library tab and the search box"
    requirement: UI-03
    verification:
      - kind: other
        ref: "target/synth-small/portal-pages.png and portal-header.png (40 themes, 30 per page), read by the executor"
        status: pass
    human_judgment: true
    rationale: "Layout is visual; the executor read the captures, the maintainer confirms at the end of the phase"
  - id: D4
    description: "Typing in the search box filters the whole library after 500 ms, lowercased, from page 1; Escape and the clear button restore it; Ctrl+F focuses the box; sort changes reorder at once"
    requirement: LIB-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/header.rs#search_text_is_trimmed_and_lowercased, search_text_is_cut_to_twenty_characters_not_bytes"
        status: pass
    human_judgment: true
    rationale: "Typing, the debounce, Escape and Ctrl+F need a person (no synthesized input); this is the plan's end-of-phase human check"
  - id: D5
    description: "Zero themes shows Your Library is Woven by Spider Webs, no match shows Your Library Enchanted with Spider Webs, an unreadable library folder shows We couldn't read your library with Try Again"
    requirement: LIB-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/grid.rs#a_scan_error_is_a_load_error_whatever_else_is_known, no_themes_is_the_empty_state_even_with_a_query, a_query_without_a_match_is_the_search_empty_state, a_blank_query_shows_the_whole_library, one_theme_is_one_card"
        status: pass
      - kind: other
        ref: "captures of an empty and of a missing library folder on a synthesized data folder, read by the executor"
        status: pass
    human_judgment: false
duration: 23min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 14: Library pagination, header search and states Summary

**Legacy nine-slot pagination bar with a digit-limited jump field, a header with the Library tab and a debounced 20-character search over the whole library, and the empty, search-empty and load-error pages, all following the service's sort, page-size and layout broadcasts.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-11-24T08:07:30Z
- **Completed:** 2026-11-24T08:30:00Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- The library pages like the legacy app: Previous, up to nine page buttons (above six pages the first two, last two and the pages around the current one, with the legacy "..." slots), Next, a numeric field limited to the digits of the last page and Jump. The bar hides at one page; a page change scrolls to the top and new cards show skeleton thumbnails until decoded.
- A new page size or sort order returns to page 1, a layout change (AdaptiveLayout, AdaptiveMargin) re-lays out and keeps the page; the decision is the unit-tested `page_after_settings_change`.
- The header row (48 px) has the Library tab and the search box; `LibraryView::set_query` filters the whole library with `sucrose_core::library::search` and resets to page 1. The query stays in the process: no network call and no log line carries it (T-03-33, D-15).
- Empty, search-empty and load-error pages use the legacy keys; Try Again rescans the folder.

## Task Commits

1. **Task 1 (tracer): the user pages through the library**
   - RED `f634a19e` (test)
   - GREEN `a2c553e2` (feat)
2. **Task 2: header with Library tab and search; empty, search-empty and load-error states**
   - RED `9f15bc0a` (test)
   - GREEN `3c9cd3ff` (feat)

**Docs:** `4e1a6aa9` (FEATURES.md and GPUI.md). **Plan metadata:** the docs(03-14) completion commit that follows.

## Files Created/Modified

- `apps/sucrose-portal/src/library/pagination.rs` - page_buttons, clamp_jump, jump_text_allowed and the bar
- `apps/sucrose-portal/src/header.rs` - Header entity, normalize_query, debounce and Ctrl+F focus target
- `apps/sucrose-portal/src/library/mod.rs` - visible index, set_query, set_page, jump handling, page_after_settings_change, state-aware render
- `apps/sucrose-portal/src/library/grid.rs` - library_state and the three state pages
- `apps/sucrose-portal/src/app.rs` - header between the title bar and the banner, Ctrl+F observer
- `apps/sucrose-portal/src/assets.rs`, `i18n.rs`, `locales/sucrose.en.yml`, `main.rs` - icons, two new keys, module
- `docs/FEATURES.md`, `docs/GPUI.md` - library and header rows, new GPUI lessons

## Decisions Made

See `key-decisions`. The pagination bar uses composed Buttons, not gpui-component's Pagination (the plan asked to record which).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Accessible labels for Previous and Next**
- **Found during:** Task 1
- **Issue:** UI-SPEC requires an accessible label on every icon-only control, but no key existed for the two arrow buttons.
- **Fix:** Added `Portal.Pagination.Previous` and `Portal.Pagination.Next` to `sucrose.en.yml` and `REQUIRED_KEYS`.
- **Files modified:** apps/sucrose-portal/locales/sucrose.en.yml, apps/sucrose-portal/src/i18n.rs
- **Committed in:** a2c553e2

**2. [Rule 1 - Bug] Icons missing from the Portal asset source**
- **Found during:** Task 2 (capture of the empty state: the Library tab and the state icons rendered as nothing)
- **Fix:** `icon_assets!` now also embeds Library, LibraryBig, SearchX and FolderX (the lesson was already in docs/GPUI.md).
- **Files modified:** apps/sucrose-portal/src/assets.rs
- **Committed in:** 3c9cd3ff

### Interpretations of the plan

- **Jump digit limit.** The behavior list says clamp_jump("99", 5) = 5 and also that text longer than the last page's digit count is refused. Both hold: `clamp_jump` clamps any number, and the field itself refuses digits past the limit through `jump_text_allowed` (UI-SPEC E5 long-text row).
- **"page shown" debug log** is written by `LibraryView::request_page`, not by `app.rs`: `PortalApp` already forwards every `Changed` to `LibraryView::update_settings`, which now applies the sort, page-size and layout rules, so `app.rs` only gained the header and the Ctrl+F observer.
- **Change Library Location** is not shown on the Empty and LoadError pages until plan 20 gives it a target (plan text); the key `Portal.LibraryPage.ChangeLocation` stays unused until then.
- **Search max length.** `InputState` has no max length, so the 20-character limit is a validator: a paste that would pass 20 characters is refused whole (WPF would cut it).
- **Tracer gate.** The tracer's human-check names the executor as the one who opens the capture, so the executor read it and continued; no checkpoint was returned. The Task 2 human-check (typing "sea", Escape, sort change) is deferred to the end-of-phase review as the plan says.
- **TDD evidence.** `cargo test` text output is not a report format `gsd_run check tdd-red-evidence` supports, so the classifier was not run. Each RED commit was checked by hand: the stubs returned wrong values and the named tests failed on their assertions (7 of 8 pagination tests, 3 of 4 settings tests, 2 header tests, 3 of 5 library_state tests), not on compile errors.

**Total deviations:** 2 auto-fixed (1 Rule 2, 1 Rule 1), 6 interpretations. **Impact:** no scope change.

## Issues Encountered

- A capture run on a data folder that did not exist (`synth-library --count 0` is refused) started the service without `Setting/Sucrose.json`, so it migrated from the real legacy settings read-only and played the maintainer's selected theme for about 12 s until `taskkill`. Nothing was written to the real data. Later runs used a synthesized folder.
- A shell quoting slip named one capture `synth-empty$1.png`; the file was only under `target/` and was replaced.
- `cargo fmt` and my edit helper rewrote four CRLF working-tree files (docs/FEATURES.md, docs/GPUI.md, grid.rs, i18n.rs) to LF; CRLF was restored before the docs commit (no content diff).

## Known Stubs

None. The missing Change Library Location button is intentional and waits for plan 20.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test -p sucrose-portal pagination`: 8 passed; `cargo test -p sucrose-portal`: 78 passed; `cargo clippy -p sucrose-portal --all-targets -- -D warnings` clean; `cargo xtask ci` exit 0.
- Acceptance greps: page_buttons 1, clamp_jump 1, normalize_query 1, Portal.EmptyLibraryPage 2, Portal.LibraryPage.LoadError 2, telemetry 0.
- Captures (debug build, 40 synthesized themes): bar with Previous disabled, pages 1 and 2, Next, field and Jump under the grid; header with the Library tab and search box; empty and load-error pages with their legacy text.

## Next Phase Readiness

- `Header` is ready for plans 17, 18 and 20 to add the theme toggle, Premium, the more menu and the settings gear.
- `LibraryView::reload` and the two state pages are ready for the Change Library Location button of plan 20.
- Pending human check at the end of the phase: type "sea" in the search box on the real library, clear it with Escape, change Library Sorting.

## Self-Check: PASSED

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
