---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 46
subsystem: portal-ui
tags: [display-preferences, monitor-canvas, placement-mode, remove-wallpaper, gpui, desk-02, desk-03]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-42/04-43/04-45: RuntimeState.displays, SettingChange::{ScreenDevice, DisplayScreenType}, PortalRequest::RemoveWallpaper, portal protocol 11"
provides:
  - "dialogs/display.rs: DisplayModel (tiles, rows, not-connected rows), canvas_layout, ModeState, DisplayPreferences, open_display_preferences"
  - "The Display Preferences icon button in the library header (Lucide Monitor, between Premium and the more menu)"
  - "LibraryView::theme_brief (title and thumbnail file of a theme) and ServiceLink::recording (tests only)"
  - "The Portal.DisplayPreferences.* strings (legacy import prefix and 14 new keys)"
affects: [04-54, 04-56, DESK-02, DESK-03]
requirements-completed: []
actuals:
  tokens: 22600
  tasks: 3
  commits: 3
plan_head_before: 62f0cda49bd05379221ed82d43a62408508adf24
plan_head_after: a70e95c97dfc0f88f9f420d76b0c076259da14fd
tech-stack:
  added: []
  patterns: ["the dialog is a pure model (DisplayModel::build from RuntimeState) drawn by one entity, so every rule is tested without a window", "state that must end with a dialog is held by a guard entity captured by the dialog builder and released with cx.on_release", "a focused element takes Enter through on_action(Confirm) so the dialog does not confirm and close"]
key-files:
  created:
    - apps/sucrose-portal/src/dialogs/display.rs
  modified:
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/dialogs/mod.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/ipc.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/legacy.en.yml
    - apps/sucrose-portal/locales/sucrose.en.yml
    - xtask/src/localize.rs
    - docs/GPUI.md
    - docs/FEATURES.md
key-decisions:
  - "The target and the mode shown are only what the service broadcast; a radio click is the one exception, it shows the asked mode with Applying until the broadcast (or reverts on Rejected)"
  - "A row or tile that already is the target sends nothing; Remove sits in its own click area so it never also selects the row"
  - "Tile thumbnails reuse library::thumbs::decode_thumbnail at the window's bucket (360 or 720 px wide, at least the tile width), so thumbs.rs needed no change"
  - "A refused DisplayScreenType or ScreenDevice change raises the placement toast instead of the generic save-failed toast"
status: complete
coverage:
  - id: D1
    description: "The header Display Preferences button opens the dialog; the canvas draws every connected monitor from its rectangle with one scale factor, centred; a tile or row click sends SettingChange::ScreenDevice once and the target outline follows only the broadcast"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal dialogs::display::tests display_canvas_* (4), display_model_* (9)"
        status: pass
      - kind: integration
        ref: "dialogs::display::tests::display_header_button_opens_the_dialog_and_a_tile_click_sets_the_target (GPUI Root window, recording link)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Placement Mode: a radio sends SettingChange::DisplayScreenType once, shows Applying until the broadcast, a Rejected change reverts the radios and asks for the placement toast; the radios are disabled while loading"
    requirement: DESK-02
    verification:
      - kind: unit
        ref: "display_mode_* (2), display_placement_a_radio_sends_one_change_and_waits_for_the_broadcast, display_placement_a_rejected_change_reverts_the_radios_and_asks_for_the_toast, display_placement_the_radios_are_disabled_until_the_service_has_reported"
        status: pass
    human_judgment: false
  - id: D3
    description: "Remove Wallpaper per monitor, the All screens row of Span and Duplicate (RemoveWallpaper None), the Not connected section, empty tiles and Remove disabled without a wallpaper"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "display_list_remove_sends_the_monitor_and_does_not_also_select_the_row, display_list_remove_is_disabled_on_a_monitor_without_a_wallpaper, display_list_span_has_one_all_screens_row_that_removes_the_single_wallpaper, display_not_connected_assignments_have_their_own_section_with_a_remove_button"
        status: pass
    human_judgment: false
  - id: D4
    description: "Tile thumbnails decode on the background executor and are released with atlas::release_images when the dialog closes; an undecodable file gives the ImageOff fallback"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "display_thumbnails_are_decoded_in_the_background_drawn_and_released_when_the_dialog_closes, display_thumbnails_a_file_that_is_no_image_gives_the_tile_the_fallback"
        status: pass
    human_judgment: false
  - id: D5
    description: "Keyboard: focus lands in the radio group on open, left and right move between tiles, Enter selects without closing the dialog, Escape closes it"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "display_keyboard_arrows_move_between_tiles_and_enter_selects, display_keyboard_focus_lands_on_the_radios_and_escape_closes_the_dialog"
        status: pass
    human_judgment: false
  - id: D6
    description: "UI Considerations E5 visual backstop: with DISPLAY1 and DISPLAY9 the canvas shows two readable tiles in the right proportions"
    requirement: DESK-03
    verification: []
    human_judgment: true
    rationale: "Proportions and readability are an eye check; the maintainer's glance in 04-56 decides, nothing automated does (the plan names it a backstop)"
duration: 22min
completed: 2026-11-28
---

# Phase 4 Plan 46: Display Preferences dialog Summary

**The Display Preferences dialog: a header button opens a 728 px dialog with the scaled monitor canvas, the Placement Mode radios (Applying indicator, revert on refusal), the monitor list with target selection and Remove Wallpaper, the All screens row of Span and Duplicate, and a Not connected section, all driven by RuntimeState.displays and sending only requests.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-11-28T17:20:00Z
- **Completed:** 2026-11-28T17:42:00Z
- **Tasks:** 3
- **Files modified:** 13 (one created)

## Accomplishments

- `DisplayModel::build` turns a `RuntimeState`, the stored target device and the shown mode into tiles, rows and not-connected rows; `canvas_layout` scales the union of the monitor rectangles by one factor into the 680 x 232 canvas (centred, 48 x 28 minimum tile). An empty or unknown target is the first monitor (D-02).
- The dialog never changes anything itself (D-11): tile and row clicks send `ScreenDevice`, a radio sends `DisplayScreenType` and shows Applying until the broadcast, Remove sends `RemoveWallpaper` for one monitor (or `None` for the single wallpaper of Span and Duplicate).
- Thumbnails decode at the window's bucket on the background executor and leave the atlas when the dialog is dropped, however it closed.
- The refused-placement toast replaces the generic save-failed toast for `DisplayScreenType` and `ScreenDevice` rejections.

## Task Commits

1. **Task 1: header button, dialog with the canvas, target selection (tracer)** - `363c2590` (feat)
2. **Task 2: Placement Mode, Remove Wallpaper, All screens, Not connected, states** - `929e8f2c` (feat)
3. **Task 3: strings and GPUI notes** - `a70e95c9` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

- `apps/sucrose-portal/src/dialogs/display.rs` - model, layout, dialog view, 27 tests
- `apps/sucrose-portal/src/header.rs`, `app.rs`, `assets.rs`, `dialogs/mod.rs` - the button, the persistent model entity fed by Welcome, Changed, Rejected and Runtime, icons Monitor, MonitorOff, Pause, Power
- `apps/sucrose-portal/src/library/mod.rs` - `ThemeBrief` and `LibraryView::theme_brief`
- `apps/sucrose-portal/src/ipc.rs` - `ServiceLink::recording` (tests only)
- `xtask/src/localize.rs`, `locales/legacy.en.yml`, `locales/sucrose.en.yml`, `i18n.rs` - strings
- `docs/GPUI.md`, `docs/FEATURES.md` - lessons and the per-display row note

## Decisions Made

See key-decisions. Placement toast, no redundant sends for the current target, and reuse of the shared thumbnail decoder instead of a new decode entry.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] A recording ServiceLink for the view tests**
- **Found during:** Task 1
- **Issue:** The plan names "a recording ServiceLink stand-in as the existing view tests do", but no such stand-in exists and `ServiceLink` only talks to the IPC threads.
- **Fix:** `#[cfg(test)] ServiceLink::recording()` returns a link whose requests stay in a queue.
- **Files modified:** apps/sucrose-portal/src/ipc.rs (not in the plan's file list)
- **Committed in:** 363c2590

**2. [Rule 1 - Bug] Enter on a focused tile would have confirmed and closed the dialog**
- **Found during:** Task 2 (keyboard test)
- **Issue:** gpui-base binds Enter to the dialog's Confirm action, which is handled before the element's click, so Enter on a tile or row closed the dialog instead of selecting.
- **Fix:** tiles and rows handle the `Confirm` action themselves; the test asserts the dialog stays open. Recorded in docs/GPUI.md.
- **Committed in:** 929e8f2c

**3. [Rule 2 - Missing critical] The refused-placement toast**
- **Found during:** Task 2
- **Issue:** the Rejected handler raised the generic save-failed toast for every refusal, which would have shown wrong text (and no revert) for a refused mode change.
- **Fix:** `DisplayPreferences::on_rejected` reverts the radios and reports whether the change was a placement change; app.rs picks the placement toast then.
- **Committed in:** 929e8f2c

### Plan notes

- `library/thumbs.rs` needed no change: `decode_thumbnail` at the window's bucket already decodes off the UI thread with the shared size limits (T-04-101). `cargo test -p sucrose-portal thumbs` still passes.
- `docs/FEATURES.md` (per-display row) got a note, as CLAUDE.md asks; status columns stay for 04-56.
- The legacy key `Portal.DisplayPreferences.Screen` (Selected Screen Only) is imported and unused.
- TDD: tests and code were committed per task, not as separate RED and GREEN commits (the plan is `type: execute`).

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 missing critical). **Impact:** none on scope.

## Issues Encountered

- `cargo xtask ci` failed once on the known flaky `a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once`; the rerun passed fully.
- Line endings: all edited Portal files and locale files are LF (untouched by fmt); `xtask/src/localize.rs`, `docs/GPUI.md` and `docs/FEATURES.md` are CRLF and kept CRLF (neither has a final newline, as before). No file needed a CRLF restore after `cargo fmt`.

## Known Stubs

None. The Customize buttons of this dialog come with 04-54 by plan.

## Threat Flags

None. No new endpoint, auth path or file access: thumbnails go through the existing bounded decoder (T-04-101); monitor names are shown as single-line text (T-04-102).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The dialog is ready for the Customize buttons (04-54) and the Round C checkpoint (04-56), which carries the two-monitor eye check (E5 visual backstop). No GUI capture was made in this plan.

## Self-Check: PASSED

Created file and the three commits exist (checked with `[ -f ]` and `git merge-base --is-ancestor`); `cargo xtask ci` passes (one known flaky rerun).
