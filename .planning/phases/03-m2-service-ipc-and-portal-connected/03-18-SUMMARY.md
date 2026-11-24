---
phase: 03-m2-service-ipc-and-portal-connected
plan: 18
subsystem: ui
tags: [gpui, status-bar, dialogs, tray-hint, memory-report, clipboard]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "PortalEvent::Memory (plan 06), RuntimeState.tray_visible and SettingChange::HintTrayIcon (plans 04/06/13), the Header entity and service link (plan 14), the theme and backdrop (plan 17)"
provides:
  - "Status bar: build, version, architecture, total memory of all Sucrose processes with a per-process tooltip, a random quote that copies on click, copyright with responsive hiding"
  - "Header Premium button and a more menu with Help and About (D-15, nothing of later phases)"
  - "Help, About and tray hint dialogs; every link an https constant of the build"
  - "First-close tray hint with a 5 s countdown that turns HintTrayIcon off through the service and then closes the window"
affects: [03-20, 03-26, settings pages, UI-06 RTL check]

actuals:
  tokens: 11700
  tasks: 3
  commits: 6
plan_head_before: 085a40be30e55ab1be95ba19e6b798cdc064da09
plan_head_after: e6dc187b6b0ec26ce10ad90f621fa9f7ab755b72

tech-stack:
  added: []
  patterns:
    - "Dialogs through WindowExt::open_dialog with an entity as the footer for state that changes while open (the countdown)"
    - "Clipboard copy confirmed by reading the text back, because write_to_clipboard returns nothing"
    - "Window close interception: on_window_should_close reaches the root entity through a weak handle"

key-files:
  created:
    - apps/sucrose-portal/src/status_bar.rs
    - apps/sucrose-portal/src/dialogs/mod.rs
    - apps/sucrose-portal/src/dialogs/help.rs
    - apps/sucrose-portal/src/dialogs/about.rs
    - apps/sucrose-portal/src/dialogs/tray_hint.rs
  modified:
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/assets.rs
    - docs/FEATURES.md
    - docs/GPUI.md

key-decisions:
  - "The status bar and About show the version as the legacy app writes it (sucrose_core::version::app_version(), four components) instead of the raw Cargo text, so both read the same"
  - "The quote hides below 900 px and the copyright below 1000 px; the window never opens narrower than 900 px, so at the default size the copyright is already hidden"
  - "The tray hint dialog turns Escape and backdrop presses off for its whole life: the dialog layer fixes those settings when it opens"
  - "The Premium address lives in header.rs, the other links in dialogs/mod.rs; all are compile-time https constants"

patterns-established:
  - "StatusBar entity: set_memory(None) while disconnected shows a dash; the service's report replaces it once a second"
  - "TrayHint entity owns the countdown and the Close action; PortalApp only arms the 1 s close fallback"

requirements-completed: [UI-03, UI-05]

duration: 13 min
completed: 2026-11-24
status: complete

coverage:
  - id: D1
    description: "The status bar shows the build, the total memory of all Sucrose processes from the service (dash until the first report or while disconnected) with a per-process tooltip, a quote that copies on click, and the copyright with responsive hiding"
    requirement: UI-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/status_bar.rs#the_total_shows_in_whole_megabytes"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/status_bar.rs#the_tooltip_lists_every_process_and_the_total"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/status_bar.rs#narrow_windows_hide_the_copyright_first_and_then_the_quote"
        status: pass
      - kind: e2e
        ref: "debug Portal on target/synth-small: capture target/synth-small/portal-status.png shows icon, Pre-Release, Rust, x64, v26.11.22.1, 134 MB and a quote on one line"
        status: pass
    human_judgment: false
  - id: D2
    description: "The header has exactly the Phase 3 actions: Library tab, search, light/dark toggle, Premium and a more menu with Help and About"
    requirement: UI-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/header.rs#premium_is_a_page_of_the_official_site_over_https"
        status: pass
      - kind: e2e
        ref: "capture target/synth-small/portal-header.png shows toggle, crown and the three-dot button"
        status: pass
    human_judgment: true
    rationale: "Opening the menu, clicking Premium and the dialog links needs real input and a browser; the maintainer's check is the end-of-phase row"
  - id: D3
    description: "Help shows five link rows and About shows the blurb, name, version and five links, all opening in the OS browser through https constants, with the body scrolling under a fixed title and Close"
    requirement: UI-05
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/mod.rs#every_link_is_a_plain_https_address"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/help.rs#help_has_the_five_legacy_rows_with_distinct_targets"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/about.rs#about_has_the_five_links_and_no_update_or_partner_row"
        status: pass
    human_judgment: true
    rationale: "Dialog layout, scrolling, focus return and the browser launch were not exercised: this run synthesizes no input"
  - id: D4
    description: "The first window close with HintTrayIcon on and the tray visible shows the tray hint with a 5 s countdown; Close turns the hint off through the service and closes the window"
    requirement: UI-05
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/tray_hint.rs#the_hint_shows_only_when_everything_allows_it"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/tray_hint.rs#the_countdown_counts_from_five_to_a_plain_close"
        status: pass
    human_judgment: true
    rationale: "Closing the window needs real input; the wiring (on_window_should_close, the change request, the close fallback) is not covered by a test"
  - id: D5
    description: "Header, status bar and dialogs order their items by flex order so the layout can mirror for Arabic and Hebrew"
    requirement: UI-03
    verification: []
    human_judgment: true
    rationale: "Backstop: a mirrored screenshot is due with UI-06 (Phase 5), UI-SPEC E4 RTL row"
---

# Phase 3 Plan 18: Service, IPC and Portal connected, Portal shell Summary

**Status bar with the service-reported total memory of all Sucrose processes and a copyable quote, Premium and a Help/About menu in the header, Help, About and tray hint dialogs, and a first-close hint that turns itself off through the service**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-24T09:32:47Z
- **Completed:** 2026-11-24T09:45:36Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Status bar (32 px): apps icon, Pre-Release, Rust, architecture, version, memory total with a Service/Renderer n/Portal/Total tooltip, a one-line truncated quote with the full text as tooltip that copies on click ("Copied" for 2 s, or the failure text after a read-back check), and the copyright. The copyright hides below 1000 px and the quote below 900 px. The memory text is a dash until the first report and again while the service is unreachable.
- Header: the light/dark toggle, Premium (opens the premium store) and a three-dot menu with Help and About, grouped with an 8 px gap. Nothing of later phases (no Create Wallpaper, Cycling or Display Preferences).
- Help: five link rows (Web Page, Documentation, Source Code, Discussions, Report Issue). About: blurb, "Sucrose Wallpaper Engine" in display size, "Version: v..." and the Release Notes, Contributors, Terms of Service, Privacy Policy and Code of Conduct links, without update status, partners or "Last Checked". Widths 480 and 560, a body capped at window height minus 64 px that scrolls under a fixed title and Close.
- Tray hint: closing the window while HintTrayIcon is on and the tray icon is visible opens the System Tray dialog (static tray illustration, the legacy message, Close disabled during a 5 to 0 countdown). Close sends HintTrayIcon off to the service and the window closes when the service answers (or refuses) or after 1 s.
- FEATURES rows updated (D-19 behavior change recorded); GPUI notes added.

## Task Commits

1. **Task 1: status bar (tracer, TDD)** - RED `3f7ed47c` (test), GREEN `582ff17f` (feat)
2. **Task 2: Premium, the more menu, Help and About** - `7f804c76` (feat)
3. **Task 3: tray hint dialog (TDD)** - RED `eba6a991` (test), GREEN `d7639af8` (feat)

**Docs follow-up:** `e6dc187b` (corrects a wrong claim about the legacy Escape behavior in FEATURES and adds the GPUI notes)

**Plan metadata:** the final docs commit (this SUMMARY, STATE, ROADMAP, REQUIREMENTS).

## Files Created/Modified

- `apps/sucrose-portal/src/status_bar.rs` - StatusBar entity and the pure memory, visibility, architecture, quote and copy helpers
- `apps/sucrose-portal/src/dialogs/mod.rs` - link constants, body height cap, link row, close footer, open_link
- `apps/sucrose-portal/src/dialogs/help.rs`, `about.rs`, `tray_hint.rs` - the three dialogs
- `apps/sucrose-portal/src/app.rs` - status bar placement, Memory event, window close interception, hint close fallback
- `apps/sucrose-portal/src/header.rs` - Premium and the more menu
- `apps/sucrose-portal/src/main.rs` - module registration and `version_text()`
- `apps/sucrose-portal/src/assets.rs` - the Lucide icons the new UI uses
- `docs/FEATURES.md`, `docs/GPUI.md` - parity rows and gotchas

## Decisions Made

See key-decisions. The version text (four components, as the legacy app shows it) is used in both the status bar and About; the plan text said the raw Cargo version for the status bar.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Escape and the legacy tray hint behavior mis-stated in FEATURES**
- **Found during:** Task 3 follow-up
- **Issue:** The first FEATURES row claimed the legacy dialog had no Escape. The legacy code blocks Enter and Escape only during the countdown.
- **Fix:** Reworded the row: the Rust dialog turns Escape and backdrop presses off for its whole life, because the dialog layer fixes those settings when it opens.
- **Files modified:** docs/FEATURES.md
- **Committed in:** `e6dc187b`

### Departures that were not bugs

- **Escape after the countdown (UI-SPEC Dialogs):** the spec lets Escape close the tray hint once the countdown is over; the Rust dialog never does (see decisions). Close works, so the dialog is never stuck. Recorded in FEATURES.
- **Opening a link cannot fail visibly:** GPUI's open_url returns nothing, so the "error is logged" requirement is met by logging every attempt (an info line with the address); an OS failure itself cannot be detected. The clipboard write is checked by reading the text back.
- **Quote hiding threshold:** the spec says the quote hides after the copyright without a number; 900 px was chosen, which equals the minimum window width, so at the default size the quote stays and the copyright is hidden. The plan's capture check expected the copyright in the screenshot; at 900 px it is hidden by the spec rule.
- **Apps icon:** the Lucide LayoutGrid glyph stands for the legacy "apps" symbol; the tray illustration is drawn from the same glyph instead of an image of the app icon.
- **Focus on open and return:** the dialog layer's own focus handling is used; focus movement to the first action and back to ⋮ was not verified (no input is synthesized).

---

**Total deviations:** 1 auto-fixed (1 documentation bug), 5 departures noted above
**Impact on plan:** No scope creep; all three tasks delivered. The unverified items are the maintainer's end-of-phase checks.

## Issues Encountered

- The plan's acceptance greps expect `open_url` in help.rs, the Premium address in header.rs and the HintTrayIcon change in tray_hint.rs. The structure was adjusted to match (Premium constant in header.rs, the tray hint entity sends the change itself, help.rs documents the open_url route through open_link) rather than the greps being bent.
- A first RED run of each TDD task failed on assertions of the planned behavior (8 and 4 failing tests), not on build errors.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Threat Flags

None. T-03-42 holds: every opened address is an https constant of the build (a unit test checks the scheme, no credentials, query or fragment). T-03-43 accepted: only the clicked quote is copied. No crate was added (T-03-SC).

## Next Phase Readiness

- The Portal shell is complete for Phase 3: the settings pages (plan 19 onward) can use the dialog helpers (`dialogs::body`, `close_footer`, `link_row`) and the toast module.
- The maintainer's end-of-phase checks listed in the plan remain: open ⋮ > Help and About and click a link in each, click Premium, close the window with HintTrayIcon on and the tray visible.
- The mirrored (RTL) screenshot is due with UI-06.

## Self-Check: PASSED

All created files exist, all six commits are ancestors of HEAD, the acceptance greps hold, cargo xtask ci passes.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
