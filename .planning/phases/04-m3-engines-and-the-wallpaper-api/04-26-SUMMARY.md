---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 26
subsystem: portal
tags: [webview2, arguments-editor, ipc, settings, restart-prompt]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "the CEF arguments editor and the restart prompt (04-16), the WebView2 engine and its switches (04-22, 04-25), the GetArguments answer for both engines"
provides:
  - "SettingChange::WebArguments (appended after DeveloperPort) and PORTAL_PROTOCOL_VERSION 7"
  - "settings_host validation of the WebView2 list with the CEF rules, stored in the legacy Engine.WebArguments list (--name or --name=value)"
  - "The WebView2 row on the Wallpaper page with its status line and its own Arguments editor, beside the CEF row"
affects: [04-27, 04-37]

actuals:
  tokens: 10800
  tasks: 2
  commits: 3
plan_head_before: d04cd8ff7078cbcbda45e759f9378008268cf0ba
plan_head_after: e5877f70491d8d76a22169e763931e70f204cc54

tech-stack:
  added: []
  patterns:
    - "one ArgumentsEditor per web engine; save_change and saved_list map an engine to its SettingChange and back, so the page and the app route Arguments, Changed and Rejected by engine"

key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/settings/arguments.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/dialogs/restart.rs
    - docs/FEATURES.md

key-decisions:
  - "WebArguments reuses check_switches, so CEF and WebView2 share one set of rules and limits; the stored form is args::to_web_list, the legacy list text"
  - "apps/sucrose/src/service.rs needed no change: web_options and RestartWallpapers already read Engine.WebArguments and route by engine (04-16, 04-22)"
  - "No new locale keys: the status line, hints, Arguments copy and restart prompt text are all engine-parameterised already"

patterns-established:
  - "an engine-specific setting that is not part of PortalSettings is a SettingChange variant the view ignores; the Portal asks for it with GetArguments"

requirements-completed: []

coverage:
  - id: D1
    description: "Save Arguments for WebView2 sends WebArguments; the service validates, stores it in the legacy list and a restarted WebView2 renderer starts with the saved switch (WAPI-10, D-17)"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#a_webview2_argument_list_is_cleaned_and_stored_as_the_legacy_list"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#a_webview2_argument_list_that_is_bad_repeated_or_too_long_is_refused_and_nothing_is_stored"
        status: pass
      - kind: e2e
        ref: "apps/sucrose/tests/portal.rs#saved_webview2_arguments_reach_the_restarted_renderer"
        status: pass
    human_judgment: true
    rationale: "WAPI-10 completes at the final checkpoint 04-37 where the maintainer sees the WebView2 editor"
  - id: D2
    description: "The Wallpaper page shows the certificate row, the WebView2 row and the CEF row, each with its status line and its own Arguments editor (UI-SPEC E2, E4)"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/wallpaper.rs#wallpaper_webview2_status_line_names_the_runtime_version_or_the_hint"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/arguments.rs#arguments_the_editor_of_each_engine_saves_its_own_change"
        status: pass
    human_judgment: true
    rationale: "Visual fit and the editor states need the maintainer's eye (04-37); the captures only show the top of each editor"
  - id: D3
    description: "The shared Restart prompt names WebView2 for its arguments and raises only while a WebView2 wallpaper runs (UI-SPEC E12)"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/restart.rs#restart_webview2_arguments_name_only_the_webview2_wallpapers"
        status: pass
    human_judgment: true
    rationale: "The dialog itself is checked by the maintainer in 04-37"

duration: 15min
completed: 2026-11-27
status: complete
---

# Phase 04 Plan 26: WebView2 in the Portal Summary

**WebView2 switches are saved from the Portal through a new SettingChange::WebArguments (protocol 7), validated with the CEF rules and stored in the legacy Engine.WebArguments list, and the Wallpaper page gets a WebView2 row with its own status line and Arguments editor beside CEF's.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-26T23:50:30Z
- **Completed:** 2026-11-27T00:05:30Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Task 1 (tracer): `SettingChange::WebArguments` is appended after `DeveloperPort` (wire index test updated, version 7, the settings view ignores it). `validate` runs the CEF list rules (letters, digits and dashes, no repeats, 64 switches, 64-byte names, 512-byte values) and `write` stores `args::to_web_list`, so `--disable-web-security` and `--autoplay-policy=no-user-gesture-required` are what the file holds. `GetArguments` for WebViewLive already answered from that list (legacy defaults when empty). The live test `saved_webview2_arguments_reach_the_restarted_renderer` passed: it takes the defaults from `GetArguments`, adds `Translate` to `disable-features`, saves, checks the stored legacy text and that a save alone starts nothing, sends `RestartWallpapers [WebViewLive]` and finds the saved value in the second renderer's `chromium switches` line. No Rust Sucrose process stayed running afterwards.
- Task 2: `web_engines_group` draws Allow invalid certificates, the WebView2 row and the CEF row, each with its status line and its own `ArgumentsEditor`. `save_change` and `saved_list` (arguments.rs) map an engine to `WebArguments` or `CefArguments` and back; `on_arguments`, `on_arguments_saved` and `on_rejected` of the page and the Changed routing in app.rs go by engine, so an Arguments event for WebViewLive reaches only the WebView2 editor and a rejected WebView2 list keeps its draft and shows the existing save-failed toast. Restart prompt: `Arguments(WebViewLive)` already read by WebView2 wallpapers only; tests pin "WebView2", "CEF", "WebView2 and CEF" and the no-prompt case. FEATURES row 76 records it.
- Capture `target/web-lib/portal-webview2.png` (debug build, scratch folder, fixture library with Engine.Web = WebViewLive, group moved to the top in a temporary local edit that was reverted before the commit, the same method as 04-16): Web Engine Settings with the certificate row (off, info hint), then the WebView2 row with a green check and "WebView2 is ready." followed by "Version 158.0.4370.45", the Arguments heading and description for WebView2, two grey read-only rows (`enable-gpu`, `enable-gpu-vsync`) with locks, then user rows (`disable-gpu-compositing`, `disable-direct-write`, ...) with remove buttons. A second capture with the rows swapped, `target/web-lib/portal-cef-row.png`, shows the CEF row with "CEF is ready." "Version 156.0.31" and its own list (`allow-file-access`, ...), so the two editors hold their own lists. The window clamps to the monitor, so the CEF row below the WebView2 one is not in one frame.
- `cargo xtask ci` passed on the first run. A second run failed once on the known flake `memory_detail_counts_live_heap_blocks` (sucrose-platform, unrelated); rerun alone and a full `cargo test --workspace` both passed.

## Task Commits

1. **Task 1: WebView2 switches saved through the service reach the restarted renderer** - `14d414ef` (feat)
2. **Task 2: the WebView2 row, status line and Arguments editor** - `e5877f70` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs` - the variant, version 7, wire and view tests
- `apps/sucrose/src/settings_host.rs` - validation and storage of the WebView2 list, three tests
- `apps/sucrose/tests/portal.rs` - `saved_webview2_arguments_reach_the_restarted_renderer`
- `apps/sucrose-portal/src/settings/wallpaper.rs` - two editors, `web_engine_row`, routing by engine, WebView2 status line test
- `apps/sucrose-portal/src/settings/arguments.rs` - `save_change`, `saved_list`, the Save button sends its own engine's change
- `apps/sucrose-portal/src/app.rs` - Changed routing through `saved_list`
- `apps/sucrose-portal/src/dialogs/restart.rs` - WebView2 argument prompt test
- `docs/FEATURES.md` - row 76

## Decisions Made

See key-decisions: one validator for both engines, the legacy list as the stored form, no service.rs change, no new locale keys.

## Deviations from Plan

None - plan executed exactly as written. (`apps/sucrose/src/service.rs` and `apps/sucrose-portal/locales/sucrose.en.yml` are in the plan's file list but needed no change; the plan allowed "none if the copy already exists".)

## Line endings

Before editing, `settings_host.rs`, `service.rs` and `FEATURES.md` were w/crlf; the first two were converted to LF for editing and restored to CRLF before the commit (service.rs ends unchanged, no diff), FEATURES.md was edited in place with CRLF kept and no final newline. `portal.rs`, `tests/portal.rs` and the Portal sources stayed LF. The two `crates/sucrose-engine-web` files that git lists as modified are the 04-25 line-ending artifact and were left alone.

## Issues Encountered

None beyond the unrelated flake above.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-27 (WebView2 Runtime detection and install path) can build on the WebView2 row's status line and hints.
- WAPI-10 and ENG-02 stay open for 04-37; no requirement is marked here.

## Self-Check: PASSED

- Commits `14d414ef` and `e5877f70` are in the history; the plan's grep criteria hold (WebArguments(Vec<SwitchView>) in portal.rs, PORTAL_PROTOCOL_VERSION 7, the test name in tests/portal.rs, SettingChange::WebArguments in arguments.rs).
- `git ls-files --eol`: wallpaper.rs w/lf, FEATURES.md w/crlf.
