---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 15
subsystem: portal
tags: [portal, engines, fallback, toast, snooze, warehouse, theme, wallpaper-page, ipc]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "FallbackView, EngineStateView, HintView and PortalEvent::Engines (04-12), the Wallpaper settings page (04-13, 04-14)"
provides:
  - "PORTAL_PROTOCOL_VERSION 4: FallbackView.notify and PortalRequest::SnoozeFallback { engine } (appended)"
  - "Service side snooze: the quiet time is written into Warehouse.CefSharpTime (CEF) or Warehouse.WebViewTime (WebView2); the report says notify false while it runs; any other engine changes nothing on disk"
  - "toast::Kind::Warning (12 s, close button, snooze action), toast::show_warning, toast::fresh_fallbacks (one toast per engine per Portal run)"
  - "Row alert under a type whose chosen engine is unavailable (persistent, never snoozed) and the Web Engine Settings group with the CEF status line"
  - "theme::warning_colors and WarningColors; the light warning pair in themes/sucrose.json"
affects: [04-16, 04-18, 04-19, 04-20]

actuals:
  tokens: 13400
  tasks: 2
  commits: 4
plan_head_before: b3c1ce0e081152df996b29fa80d9113ff64ab6d0
plan_head_after: 45a99284cf0e9f00e60b5208e7d7dfa8ac192bf8

tech-stack:
  added: []
  patterns:
    - "the Warehouse snooze fields hold the time until which to stay quiet (legacy DateTime.Now.AddDays(1)); the service reads and writes them, the Portal only sees notify"
    - "pure text functions (alert_text, hint_text, status_line, row_alert) over the engine report, tested in English through tr()"
    - "warning colours come from the theme file's editor-style group through one helper; only the icon is a fixed value per mode"

key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/toast.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/settings/row.rs
    - apps/sucrose-portal/src/theme.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/themes/sucrose.json
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md

key-decisions:
  - "The snooze field holds the time until which the toast stays quiet, like the C# apps (DateTime.Now.AddDays(1)), so a migrated value is honoured; notify is false only while that time is ahead and at most a day away, so a changed clock cannot silence a fallback for good"
  - "The time is written as UTC with an explicit +00:00 offset in the legacy text format; the local offset would need OS code outside sucrose-platform"
  - "The row alert names the running fallback when one exists, else the first installed engine the report offers for the type; with none it says no installed engine can play the type (extra key Engine.NoFallback)"

patterns-established:
  - "FallbackView.notify: the service decides whether the Portal raises the toast; the Portal decides only once per run per engine"

requirements-completed: []

coverage:
  - id: D1
    description: "Portal protocol 4: FallbackView.notify and PortalRequest::SnoozeFallback appended with its wire index"
    requirement: "ENG-07"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc: the_portal_protocol_is_at_version_four_since_a_fallback_says_whether_to_notify, portal_request_variants_keep_their_wire_index"
        status: pass
    human_judgment: false
  - id: D2
    description: "notify is true without a snooze or after it passed, false while it runs; the legacy and the written text format parse; a far-ahead or unparsable text is no snooze; only CEF and WebView2 have a field"
    requirement: "ENG-07"
    verification:
      - kind: unit
        ref: "apps/sucrose engines::tests::a_fallback_is_announced_unless_its_engine_is_snoozed_for_now, a_legacy_value_is_honoured_and_a_snooze_writes_the_field_of_its_engine_only, the_view_does_not_notify_for_a_snoozed_engine, the_legacy_date_time_text_reads_and_writes_as_the_same_instant"
        status: pass
    human_judgment: false
  - id: D3
    description: "A fallback is announced, SnoozeFallback writes the Warehouse field, the report says notify false and it stays so after a restart"
    requirement: "ENG-07"
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_snoozed_fallback_is_not_announced_again (ignored by default, run with --include-ignored)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Warning toast: 12 s, one per engine per run, two engines give two, a snoozed one none; the 4th toast still evicts the oldest"
    requirement: "ENG-07"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal toast::tests::*"
        status: pass
      - kind: other
        ref: "target/web-lib/portal-fallback.png (first capture: toast 'WebView2 isn't available.' / 'Playing with CEF instead.' / 'Don't remind me for a day')"
        status: pass
    human_judgment: true
    rationale: "Look and placement of the toast were read from one PrintWindow capture by the executor; the close button shows on hover only and the 12 s timing and the snooze click were not exercised by a person"
  - id: D5
    description: "Row alert text and hints, the CEF status line (checking, ready with version, hint) and the engine hints"
    requirement: "ENG-06"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal settings::wallpaper::tests::wallpaper_alert_*, wallpaper_status_line_*"
        status: pass
      - kind: other
        ref: "target/web-lib/portal-fallback.png (second capture, window enlarged: Web Engine row alert 'WebView2 isn't available, so Web wallpapers play with CEF for now.', CEF status line 'CEF is ready. Version 156.0.31')"
        status: pass
    human_judgment: true
    rationale: "The look of the alert and status line in the dark theme was read from a capture by the executor; the light theme was not captured and the alert's disappearance after an install was not exercised"
  - id: D6
    description: "Warning colours: foreground on the warning fill at least 4.5:1 and icon at least 3:1 in both themes, values as in the UI-SPEC"
    requirement: "ENG-06"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal theme::tests::theme_warning_text_has_at_least_4_5_to_1_on_the_warning_fill_in_both_themes, theme_warning_fill_and_border_are_the_ui_spec_values"
        status: pass
    human_judgment: false

duration: 21min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 15: Fallback toast, snooze and engine alert Summary

**A fallback now raises one 12 s warning toast per engine and run with "Don't remind me for a day" (stored by the service in the migrated CefSharpTime or WebViewTime field), and the Wallpaper page shows a persistent row alert for an unavailable engine plus the CEF status line, over Portal protocol 4 and a light and dark warning colour pair.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-11-26T15:19:00Z
- **Completed:** 2026-11-26T15:41:00Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- The service builds `FallbackView.notify` from the engine's Warehouse field; `SnoozeFallback` writes the quiet time (now plus one day) through the settings host, the single writer, and the report is sent again with notify false. CEF and WebView2 are the only engines with a field; any other engine changes nothing on disk.
- The Portal raises a warning toast for each fallback with notify true, once per configured engine per run, with title, body and the snooze action; the toast counts toward the three-toast cap.
- Types set to an unavailable engine show the Warning alert under their row (Video Player, Web Engine); the "Web Engine Settings" group has the CEF row with its status line (Spinner and "Checking…", CircleCheck and "CEF is ready." with the version, or TriangleAlert and the hint).
- `theme::warning_colors` reads the fill and border of the theme file's warning pair (the light pair is new) and the per-mode icon; the contrast of text and icon is tested for both themes.

## Task Commits

1. **Task 1: warning toast and snooze (tracer)** - `6751de1d` (feat)
2. **Task 2: row alert, CEF status line, warning colours** - `a8309e2c` (feat)
3. **Follow-up: new keys listed in REQUIRED_KEYS** - `6f85a017` (fix)
4. **FEATURES.md row for fallback and snooze** - `45a99284` (docs)

**Plan metadata:** recorded in the docs(04-15) completion commit.

The tracer gate: the end-to-end `<verify>` (toast tests, engines tests, build, integration test a_snoozed_fallback_is_not_announced_again) was run and passed before Task 2.

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs` - protocol 4, `FallbackView.notify`, `SnoozeFallback`
- `apps/sucrose/src/engines.rs` - snooze helpers (legacy date-time text, field mapping), `view` takes the Warehouse and the time
- `apps/sucrose/src/service.rs` - handles `SnoozeFallback`, passes the Warehouse to the report
- `apps/sucrose/tests/portal.rs` - `a_snoozed_fallback_is_not_announced_again`
- `apps/sucrose-portal/src/toast.rs`, `app.rs` - `Kind::Warning`, `show_warning`, `fresh_fallbacks`, raising the toast on a report
- `apps/sucrose-portal/src/settings/wallpaper.rs`, `row.rs` - row alert, status line, Web Engine Settings group, `SettingRow::status`
- `apps/sucrose-portal/src/theme.rs`, `themes/sucrose.json`, `assets.rs` - warning colours, light pair, TriangleAlert and CircleCheck
- `apps/sucrose-portal/locales/sucrose.en.yml`, `src/i18n.rs` - new keys and the REQUIRED_KEYS list
- `docs/FEATURES.md` - the fallback row

## Decisions Made

See key-decisions. The snooze follows the C# semantics (a stored "until" time) instead of the plan's looser "snooze time older than 24 h" wording, because D-16 and the UI-SPEC repeat row say the toast stays quiet "until the stored time passes" and a migrated C# value must be honoured.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Snooze field holds the quiet-until time, not the snooze start**
- **Found during:** Task 1 (reading the legacy writers of WebViewTime and CefSharpTime)
- **Issue:** The C# apps store `DateTime.Now.AddDays(1)` and compare it with `> DateTime.Now`. Writing the current time and treating "within 24 h" as snoozed would silence a migrated value for up to 48 h and read the start of a Sucrose snooze the other way round from the legacy meaning.
- **Fix:** `SnoozeFallback` writes now plus 24 h; `notify` is false while that time is ahead (at most a day plus a minute ahead, so a changed clock cannot silence a fallback for good); a past, empty, minimum-date or unparsable text means notify true.
- **Files modified:** apps/sucrose/src/engines.rs
- **Verification:** engines unit tests and the integration test
- **Committed in:** 6751de1d

**2. [Rule 3 - Blocking] The time is written as UTC with an explicit offset**
- **Found during:** Task 1
- **Issue:** The legacy text carries the local offset, but no crate or safe API outside `sucrose-platform` gives it, and no date crate is in the workspace (T-04-SC forbids adding one).
- **Fix:** Own civil-date conversion; text written as `YYYY-MM-DDTHH:MM:SS.0000000+00:00`, read back with any `Z` or `±HH:MM` offset. A text without an offset is read as UTC.
- **Files modified:** apps/sucrose/src/engines.rs
- **Committed in:** 6751de1d

**3. [Rule 3 - Blocking] Files outside the plan's list**
- **Found during:** Task 2 and CI
- **Issue:** The CEF row needs an icon-and-text line under its title, which `SettingRow` could not draw (`row.rs` gained `status`); `i18n::every_new_key_is_required` demands every new key in `REQUIRED_KEYS` (`i18n.rs`); `docs/FEATURES.md` must stay true (CLAUDE.md).
- **Fix:** Added those edits.
- **Files modified:** apps/sucrose-portal/src/settings/row.rs, apps/sucrose-portal/src/i18n.rs, docs/FEATURES.md
- **Committed in:** a8309e2c, 6f85a017, 45a99284

**4. [Rule 2 - Missing critical] Texts the plan's key list lacks**
- **Found during:** Task 2
- **Issue:** The hints for the WebView2 runtime (HintView has the variants), a sentence for a type no installed engine can play, and the type names for "{type} wallpapers" had no key.
- **Fix:** Added `Engine.Hint.RuntimeMissing`, `Engine.Hint.RuntimeOutdated`, `Engine.NoFallback` and `Type.{Gif,Url,Web,Video,YouTube,Application}`.
- **Committed in:** a8309e2c

---

**Total deviations:** 4 (1 bug, 2 blocking, 1 missing critical). **Impact on plan:** all stay inside the plan's slice; the snooze semantic is the one a verifier may compare with the plan text.

## Issues Encountered

- The integration test first failed because `PortalClient::next_event` (used by `use_theme`) drops engine reports, so the report with the fallback was gone; the test now sends `DetectEngines` after the Use to get the current report.
- The 916x608 Portal window shows only the top of the Wallpaper page, so the row alert and the CEF row were out of the first capture. The capture was repeated after enlarging the Portal window with a one-off inline PowerShell `SetWindowPos` (not in the repository, no input synthesized); the toast is in the first capture, the rows in the second, as the toast has closed by then (12 s).
- TDD: tests and implementation went into the same task commit (no separate RED commit); `TDD_MODE` was not set.
- The alert's fallback when no wallpaper plays is a guess from the report (first installed offered engine in the service's list order), not the service's `choose_engine`; with a wallpaper playing it names the real fallback.

## Line endings

`apps/sucrose/src/service.rs` (w/crlf) and `docs/FEATURES.md` (w/crlf, no final newline) stayed CRLF; every other touched file was w/lf and stayed LF. Nothing needed restoring. The `.planning` files are rewritten by the GSD commands (see STATE below).

## Known Stubs

None.

## Threat Flags

None. `SnoozeFallback` only writes for CEF and WebView2 (T-04-34); the row alert and the tray keep the fallback visible (T-04-35); no crate was added (T-04-SC).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-18 adds the Install or Repair buttons to the toast and the alert; `show_warning` takes one action today and `row_alert` returns text only, so both grow there.
- mpv and libVLC have no Warehouse field; their snooze would last until the Portal restarts (UI-SPEC E3 assumption; no such fallback exists in Round A).

## Self-Check: PASSED

- Commits 6751de1d, a8309e2c, 6f85a017, 45a99284 are on the branch.
- `cargo xtask ci` (fmt, clippy -D warnings, test) passed before this commit; `a_snoozed_fallback_is_not_announced_again` passed with `--include-ignored`; no `sucrose.exe` or `sucrose-portal.exe` was left running.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
