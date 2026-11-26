---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 13
subsystem: portal
tags: [portal, settings, engines, wallpaper-page, use-state, localize, ipc]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "PortalEvent::Engines, EngineReport, pick/fallbacks (04-12), the CEF renderer (04-01), the Portal protocol and settings rows (Phase 3)"
provides:
  - "Portal card use_state over the service's engine report: Unavailable until it arrives, UnsupportedType, NoEngine"
  - "Settings > Wallpaper tab (after Personal) with Video Player and Web Engine selects: apps/sucrose-portal/src/settings/wallpaper.rs (WallpaperPage, engine_choices)"
  - "PORTAL_PROTOCOL_VERSION 2: PortalPage::Wallpaper, the six engine SettingChanges and PortalSettings fields, UseRefusal::TypeNotPlayable (renamed NotVideo), EngineReportView.supported and .offered"
  - "settings_host validation of engine choices against the legacy allowed lists; engine change restarts the running wallpaper of that type"
  - "command-line argument WallpaperSetting; localize prefixes Portal.WallpaperSettingPage. and Portal.Area."
affects: [04-14, 04-15, 04-16, 04-18, 04-20]

actuals:
  tokens: 24000
  tasks: 2
  commits: 7
plan_head_before: 4d21886c0c86e9f13b2c4cc3b6f6f5ed93da5d08
plan_head_after: 3749031b5c3c36a7caa4e3243e29bc6aac093962

tech-stack:
  added: []
  patterns:
    - "the Portal decides nothing about engines: supported, playable and offered all come from the service's report"
    - "Choice gains a muted suffix that does not disable the option (Not installed)"
    - "engine change: validate against the legacy list, save, restart only the wallpaper of that type, resend the report"

key-files:
  created:
    - apps/sucrose-portal/src/settings/wallpaper.rs
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/settings/mod.rs
    - apps/sucrose-portal/src/settings/row.rs
    - apps/sucrose-portal/src/settings/general.rs
    - apps/sucrose-portal/locales/legacy.en.yml
    - apps/sucrose-portal/locales/sucrose.en.yml
    - xtask/src/localize.rs
    - docs/FEATURES.md

key-decisions:
  - "EngineReportView gained `supported` (types some slice of this build plays) and `offered` (per type, the engines whose slice landed): without them the card cannot tell UnsupportedType from NoEngine and the page would offer CEF for Video, which CEF cannot play yet; both are the service's word, never the Portal's"
  - "A card waits for the engine report: with none it is Disabled(Unavailable) like before the connection"
  - "A saved engine whose new choice cannot be used now (no engine at all) leaves the running wallpaper as it is and logs a warning; the setting is kept (D-11)"
  - "The Wallpaper page shows the stored engine even when the report does not offer it, with the muted Not installed suffix; an engine outside the type's legacy list is never shown and the service refuses it (T-04-31)"

patterns-established:
  - "Portal pages that need the engine report read PortalApp.engines through set_engines; on_opened asks for DetectEngines again"
  - "Integration tests that play a real CEF wallpaper skip through common::skipped when the report says Web is not playable"

requirements-completed: []

coverage:
  - id: D1
    description: "A Web card's Use is enabled from the service's report; the card reasons are NoEngine and UnsupportedType, not the Phase 3 Video-only rule"
    requirement: "TYPE-03"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/card.rs#a_web_theme_can_be_used_when_the_service_reports_web_as_playable"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#use_plays_a_web_theme"
        status: pass
    human_judgment: false
  - id: D2
    description: "Settings has a Wallpaper tab after Personal with Video Player and Web Engine selects of the engines the service offers, the stored engine marked Not installed when it is missing"
    requirement: "ENG-07"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/wallpaper.rs#wallpaper_the_stored_engine_is_listed_with_the_not_installed_suffix_when_the_build_has_no_such_engine"
        status: pass
      - kind: other
        ref: "target/web-lib/portal-wallpaper.png (capture of the Wallpaper tab, 916x608)"
        status: pass
    human_judgment: true
    rationale: "The look of the page and the muted suffix are a visual judgment; the capture was read by the executor only"
  - id: D3
    description: "An engine change is validated against the legacy allowed list, saved through the service, and restarts only the running wallpaper of that type without a prompt"
    requirement: "ENG-07"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#validate_refuses_an_engine_outside_the_legacy_list_of_the_type"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#an_engine_change_restarts_the_running_wallpaper_of_that_type"
        status: pass
    human_judgment: false
  - id: D4
    description: "ControlState Loading/Disabled, rejected write reverts the select and shows SaveFailed (shared Phase 3 plumbing: on_setting on Rejected, control_state)"
    requirement: "ENG-07"
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#an_engine_choice_is_saved_and_broadcast_and_one_outside_the_list_of_the_type_is_rejected"
        status: pass
    human_judgment: true
    rationale: "The revert of the select and the toast are drawn by GPUI and were not driven by an automated check; the service side (Rejected, nothing written) is tested"
  - id: D5
    description: "The Wallpaper strings come from legacy.en.yml (generated through the new localize prefixes) or sucrose.en.yml, never both"
    verification:
      - kind: other
        ref: "cargo xtask localize --lang EN --check; sucrose-portal i18n tests (every_required_key_resolves_to_text, every_new_key_is_required)"
        status: pass
    human_judgment: false

duration: 29min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 13: Web cards and the Wallpaper settings page Summary

**A Web theme's Use now works from the Portal and plays on CEF, and Settings has a Wallpaper tab with Video Player and Web Engine selects that save through the service (validated against the legacy lists) and restart the running wallpaper of that type on the new engine.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-11-26T05:56:51Z
- **Completed:** 2026-11-26T06:25:12Z
- **Tasks:** 2 (Task 1 tracer, Task 2 auto; both TDD)
- **Files modified:** 21 (one created)

## Accomplishments

- Web cards are usable: `use_state(theme, runtime, engines, connected)` decides by the service's report (Unavailable until it arrives, UnsupportedType, then the new NoEngine reason with the UI-SPEC copy). `use_plays_a_web_theme` drives Use of the `hello` fixture through the real service and checks the renderer logs `engine started` with Cef.
- The Wallpaper tab (third, after Personal) shows Extensions > Video Player and Engines > Web Engine. The choices are what the report offers for the type, in the legacy order, plus the stored engine with a muted "Not installed" suffix. Opening the page sends `DetectEngines`. A capture of the page at `target/web-lib/portal-wallpaper.png` (opened with `WallpaperSetting`) shows the tab bar General / Personal / Wallpaper, "Extensions" with "Video Player" (mpv) and "Engines" with "Web Engine" (CEF).
- Engine changes: `settings_host::validate` accepts an engine only from the legacy list of its type (T-04-31); the service restarts the running wallpaper of that type on the engine now chosen (`restart_for_engine` over `Fleet::restart_with`, only for that type, T-04-32) and sends the engine report again. `an_engine_change_restarts_the_running_wallpaper_of_that_type` sets Web to WebView2 while a CEF wallpaper plays: the report shows the fallback (WebViewLive to CefSharpLive) and the renderer starts a second time.
- Protocol 2: `PortalPage::Wallpaper`, six engine `SettingChange`s and `PortalSettings` fields, `UseRefusal::TypeNotPlayable` (same wire index as `NotVideo`), all with wire-index tests; `WallpaperSetting` is the legacy command-line argument (the service starts the Portal with it for `PortalPage::Wallpaper`).
- `cargo xtask localize` takes `Portal.WallpaperSettingPage.` and `Portal.Area.` (59 new generated keys, 420 in all); new keys `Portal.LibraryCard.NoEngine`, `Portal.SettingsPage.GroupWallpaper`, `Portal.WallpaperSettingPage.Engine.NotInstalled` and `Portal.Enum.EngineType.*` live only in `sucrose.en.yml`.

## Task Commits

1. **Task 1: A Web card's Use plays it on CEF** (tracer)
   - RED `bec97147` (test), GREEN `4d2ea119` (feat)
2. **Task 2: Wallpaper settings page and engine choices**
   - RED `d6b1ca0a` (contract, settings_host, service helpers, page argument), GREEN `3e996298` (validation, restart, argument)
   - RED `11a5082f` (offered engines in the report), GREEN part of `3e996298`
   - RED `8dce6aad` (page choices, tab order, localize prefixes), GREEN `3749031b` (WallpaperPage wired into the app, docs)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

See `key-files`. The new file is `apps/sucrose-portal/src/settings/wallpaper.rs`.

## Decisions Made

See `key-decisions`. In short: the report carries `supported` and `offered` so the Portal never decides what plays or which engines exist; a card waits for the report; a saved engine that cannot be used now leaves the running wallpaper alone.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] EngineReportView.supported**
- **Found during:** Task 1
- **Issue:** the plan wants UnsupportedType for types no slice plays and NoEngine when a supported type has no installed engine, but the report held only `playable`; the two cases cannot be told apart from it, and UI-SPEC says the service reports the supported types.
- **Fix:** `supported` (the types of PLAYABLE) added to the report, filled by `EngineReport::view`; the card reads it.
- **Files modified:** crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/engines.rs, apps/sucrose-portal/src/library/card.rs
- **Commits:** bec97147, 4d2ea119

**2. [Rule 2 - Missing critical] EngineReportView.offered**
- **Found during:** Task 2
- **Issue:** the page must offer the "engines of landed slices", but `engines` states are per engine, not per type: CEF is Ready, yet plays no Video, so Video Player would have offered a choice that silently falls back.
- **Fix:** `offered: Vec<OfferedView>` (per supported type, the engines of the type's legacy list whose slice is in this build, installed or not), with its own test.
- **Files modified:** crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/engines.rs, apps/sucrose-portal/src/settings/wallpaper.rs
- **Commits:** 11a5082f, 3e996298

**3. [Rule 3 - Blocking] Files outside the plan's list**
- **Found during:** Task 1 and Task 2
- **Issue:** `apps/sucrose/src/engines.rs`, `apps/sucrose/src/portal_link.rs`, `apps/sucrose/tests/common/mod.rs`, `apps/sucrose-portal/src/library/mod.rs`, `apps/sucrose-portal/src/i18n.rs`, `apps/sucrose-portal/src/settings/row.rs`, `apps/sucrose-portal/src/settings/general.rs` and `docs/FEATURES.md` had to change: the report view is built in engines.rs; `portal_args` matches `PortalPage` exhaustively; the fixture copy helper is shared by the tests; the library view owns the cards; i18n.rs lists the required keys; `Choice` needed the muted suffix and its two literals in general.rs a new field; FEATURES.md is the parity record (CLAUDE.md "keeping the docs true").
- **Fix:** minimal edits in each.
- **Commits:** bec97147 to 3749031b

**4. [Rule 1 - Bug] clippy print_stderr in the new integration tests**
- **Found during:** `cargo xtask ci`
- **Issue:** the skip messages used `eprintln!`, which the workspace lints deny.
- **Fix:** a `common::skipped` helper with the `allow`, like `service_runs`.
- **Commit:** 3749031b

### Plan notes

- `PORTAL_PROTOCOL_VERSION` is bumped in the first RED commit of Task 2 (the settings view changes shape); `EngineReportView` changed again afterwards, which version 2 covers.
- The plan's `use_plays_a_web_theme` is not `#[ignore]`: it skips itself (stderr line) when the report says Web is not playable, so it runs wherever CEF is next to the executable, and it ran here (not skipped).
- Gif, Url, YouTube and Application rows are not on the page: their slices have not landed, so the report offers no engine for them (UI-SPEC "an engine appears once its slice landed").
- Line endings: `cargo fmt` rewrote CRLF files to LF; restored to CRLF before the commits: `apps/sucrose/src/service.rs`, `apps/sucrose/src/settings_host.rs`, `xtask/src/localize.rs`, and `docs/FEATURES.md` (working copy; the file has no final newline, kept). All files that were LF stayed LF.
- A one-off failure of `os_windows::memory::tests::memory_detail_counts_live_heap_blocks` (sucrose-platform, untouched by this plan) appeared once in `cargo xtask ci` and not on two later runs (alone and in `cargo test --workspace --no-fail-fast`); it looks like a heap-count test that is sensitive to parallel tests, not caused by this plan.

---

**Total deviations:** 4 auto-fixed (2 missing critical, 1 blocking, 1 bug)
**Impact on plan:** all needed for a correct result; no scope creep.

## TDD Gate Compliance

Task 1: RED `bec97147`; five card tests failed on their planned assertions (`a_web_theme_can_be_used_when_the_service_reports_web_as_playable`, `a_supported_type_no_installed_engine_plays_has_its_own_reason`, `the_card_waits_for_the_engine_report_before_it_allows_use`, `the_engine_reasons_come_after_the_newer_theme_and_before_the_selection_and_the_rule`, `the_tooltips_are_the_english_texts_of_the_ui_spec`) against a `use_state` that kept the Video-only rule; the crate compiled. GREEN `4d2ea119`. `use_plays_a_web_theme` passed at RED already (the service side landed in 04-12 and earlier); it is a regression test for the chain.

Task 2: RED `d6b1ca0a` (two settings_host tests, two service tests, `wallpaper_setting_opens_the_wallpaper_page`, `the_portal_argument_comes_from_the_page_enum` failed on their planned assertions), GREEN `3e996298`; RED `11a5082f` (`the_view_offers_each_type_the_engines_whose_slice_landed_for_it_installed_or_not` failed with an empty `offered`), GREEN `3e996298`; RED `8dce6aad` (six `engine_choices` tests failed against a stub that returned nothing), GREEN `3749031b`. `cargo test` output is not a format `check tdd-red-evidence` reads (Rust known gap in tdd.md), so the semantic assessment is recorded here instead of a classifier verdict. The two integration tests for Task 2 (`an_engine_choice_is_saved_and_broadcast...` and `an_engine_change_restarts_the_running_wallpaper...`) were added at RED and failed to compile or run only until the service side landed in `3e996298`; the latter ran against the real service and CEF and passed.

## Issues Encountered

None beyond the deviations. The Portal opened on `WallpaperSetting` with the service started by it (data folder `target/web-lib`, `--no-focus`); both processes were stopped with `taskkill`.

## Known Stubs

None. The Wallpaper page holds only the two rows of the landed slices by design; the Wallpaper group (04-14), the row alert (04-15) and Web Engine Settings (04-16) are later plans.

## Threat Flags

None. The new surface is the engine `SettingChange`s, validated by the service (T-04-31); a restart happens once per committed change, only for the changed type (T-04-32); no crate was added (T-04-SC).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

04-14 adds the Wallpaper group above Extensions, 04-15 the row alert and fallback notice from `PortalApp.engines` (the report already carries states, hints and fallbacks), 04-16 Web Engine Settings. TYPE-03 and ENG-07 stay open until 04-20 (the maintainer checkpoint).

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED
