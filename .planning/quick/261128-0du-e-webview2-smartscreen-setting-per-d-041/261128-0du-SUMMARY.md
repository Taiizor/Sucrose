---
phase: quick-261128-0du
plan: 01
subsystem: web-engines
tags: [webview2, smartscreen, d-041, portal, restart-prompt, ipc]
status: complete
requires:
  - D-041 (Accepted 2026-11-27, WebView2 runs without SmartScreen by default; the user chooses)
provides:
  - "Engine.WebViewSmartScreen (boolean, default false) in Setting/Sucrose.json; missing, non-boolean and migrated values read false"
  - "WebView2 renderers leave msSmartScreenProtection out of Sucrose's own disable-features switch while the setting is on; CEF is unchanged"
  - "A WebView2 SmartScreen switch under the WebView2 row of the Wallpaper page's Web Engine Settings group, with the restart prompt for the WebView2 wallpapers only"
affects: [sucrose-config, sucrose-engine-web, sucrose-engine-webview2, sucrose-engine-cef, sucrose-ipc, sucrose service, sucrose-portal]
requirements: [ENG-02]
key-files:
  modified:
    - crates/sucrose-config/src/model.rs
    - crates/sucrose-config/tests/migrate.rs
    - crates/sucrose-engine-web/src/args.rs
    - crates/sucrose-engine-web/tests/args.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/settings/other.rs
    - apps/sucrose-portal/src/dialogs/restart.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
    - docs/DECISIONS.md
    - .planning/phases/04-m3-engines-and-the-wallpaper-api/04-UI-SPEC.md
decisions:
  - "D-041 amended (2026-11-28): the planned setting is built as planned; one rule in SucroseInputs.webview_smartscreen; a user disable-features that names the feature keeps it; PROTOCOL_VERSION 12, PORTAL_PROTOCOL_VERSION 8; SETTINGS_VERSION stays 1"
metrics:
  completed: 2026-11-28
commits: 2
plan_head_before: 99742e7698c008d2e06cccdac6011a04c984fa42
plan_head_after: 9e5c67e6
actuals:
  tokens: 24000
  tasks: 3
  commits: 2
---

# Phase quick-261128-0du Plan 01: WebView2 SmartScreen setting (D-041) Summary

`Engine.WebViewSmartScreen` now travels from Sucrose.json through the service and the renderer pipe to the WebView2 command line, and a switch under the WebView2 row of the Wallpaper page sets it, with a restart prompt that names only the WebView2 wallpapers; the maintainer's eye check passed on a two-monitor desktop run.

## What was built

- **Task 1 (cd43de7e): the setting reaches the command line.** `Engine.webview_smartscreen` (serde name `WebViewSmartScreen`, default false; `engine.boolean(..., false)` makes any non-boolean read false) in `sucrose-config`. `SucroseInputs.webview_smartscreen` is the single rule read by `sucrose_switches`: WebView2 gets `disable-features=msSmartScreenProtection` only while it is false; CEF never gets it. `WebOptions.webview_smartscreen` is appended last (`PROTOCOL_VERSION` 12); `web_options` in the service copies it, the WebView2 `browser_arguments` and the "chromium switches" log line carry it, the CEF inputs and the switch editor's `arguments_answer` follow the same rule. The SmartScreen test of `crates/sucrose-engine-web/tests/args.rs` has its new name. `BROWSER_EXIT_WAIT` stays 1 s (only its doc changed).
- **Task 2 (9e5c67e6): the Portal switch and the prompt.** `SettingChange::WebViewSmartScreen` is appended after `WebArguments` and `PortalSettings.webview_smartscreen` last (`PORTAL_PROTOCOL_VERSION` 8). The service validates, stores and broadcasts the change (`settings_host.rs` write and view_of). The Wallpaper page has a `web-smartscreen` switch row directly under the WebView2 row with an info hint; `RestartSetting::SmartScreen` is read by WebView2 only, so the "Restart wallpapers?" prompt names only WebView2; the switch editor asks for Sucrose's own switches again after the change. Locale and `REQUIRED_KEYS` entries added. docs/FEATURES.md (Web and web-data rows) and the 04-UI-SPEC describe the built setting; D-041 got an amendment line (added lines only).
- **Tests added** (names as committed): config `webview_smartscreen_defaults_to_off_and_survives_a_round_trip`, `a_non_boolean_webview_smartscreen_reads_off` and the migrate test's key; args `args_webview2_runs_without_smartscreen_unless_the_setting_turns_it_on_and_cef_never_gets_the_switch`, `smartscreen_on_leaves_the_smartscreen_feature_out_of_the_browser_arguments` (WebView2 engine); protocol `web_options_run_webview2_without_smartscreen_by_default`; service `web_jobs_carry_the_webview2_smartscreen_setting`; portal protocol `the_portal_protocol_is_at_version_eight_since_the_settings_view_has_the_smartscreen_setting` plus the extended wire-index and apply tests; settings host `a_webview2_smartscreen_change_is_stored_and_shown`, `the_webview2_arguments_answer_lists_the_smartscreen_switch_only_while_smartscreen_is_off`; restart dialog `restart_the_smartscreen_setting_names_only_the_webview2_wallpapers`; Portal page `smartscreen_row`, `restart_trigger_of_the_smartscreen_switch_names_it_and_both_values`, `own_switches_are_asked_again_after_hardware_acceleration_or_smartscreen_changes`; desktop tests (apps/sucrose/tests/portal.rs) `the_webview2_smartscreen_setting_leaves_the_smartscreen_switch_out_of_the_renderer` and `webview2_smartscreen_saved_from_the_portal_restarts_only_the_webview2_wallpapers`.
- **Task 3 (checkpoint:human-verify, run by the coordinator).** See below.

## Task 3: maintainer eye check (approved)

Run on 2026-11-28 at 00:43 +03 from the current release build, scratch data dir `target/0du-check`: `sucrose.exe --no-portal`, then `sucrose-portal.exe WallpaperSetting` (with focus), two monitors (DISPLAY1 primary, DISPLAY9). The maintainer turned the WebView2 SmartScreen switch on, saw the restart prompt that names only WebView2, pressed Restart now, and the wallpaper came back. Verdict verbatim: "tamam" (ok: the switch shows under the WebView2 row, the prompt names only WebView2, the wallpaper came back).

Machine evidence:

- `target/0du-check/Setting/Sucrose.json`: `Engine.WebViewSmartScreen` = true.
- `Renderer-0` and `Renderer-1` logs: the first "chromium switches" line (21:42:52Z) has `msSmartScreenProtection` and `webview_smartscreen=false`; the last one (21:44:46Z) has no `msSmartScreenProtection` and `webview_smartscreen=true`.
- `panicked` count 0 in all four logs.
- Apps stopped with taskkill (sucrose-portal.exe, sucrose.exe; never msedgewebview2.exe by name); `tasklist.exe` showed no Sucrose process.

## Decisions

- **D-041 amended** (docs/DECISIONS.md, 2026-11-28): the planned setting is built as planned; the discretion choices of the plan are recorded there (one rule in `SucroseInputs.webview_smartscreen`; a user `disable-features` that names the feature keeps it; `WebOptions` and `PortalSettings` fields appended last; protocol versions 12 and 8; `SETTINGS_VERSION` stays 1; the switch stays enabled on every OS like the certificate switch).

## Deviations from Plan

1. **[Process] No separate RED run for Task 1 (tdd="true").** The tests were written together with the implementation, so no failing-test run was recorded before the code. The tests assert the behavior block of the plan and pass on the committed code; the maintainer's run shows the switch change in the real renderer logs (false before, true after).
2. **[Evidence] Screenshot note.** `target/0du-check/wallpaper-page.png` did not reach the Web Engine Settings row (the capture shows the top of the page), so it is not evidence of the row. The row and the prompt are covered by the maintainer's verdict above and by the Portal page unit tests.
3. **[Docs] No `docs/MEASUREMENTS.md` row.** The change is a switch that is off by default and changes no resource use of the default path, so nothing was measured for it.

## Left to the orchestrator

- STATE.md Pending Todos (e) (the WebView2 SmartScreen setting) is not edited here; the orchestrator closes it. This SUMMARY is not committed by this step.
- ENG-02 is contributed to, not closed (as the plan says); REQUIREMENTS.md is untouched.

## Line-ending check

`git ls-files --eol` after both commits: index i/lf everywhere; CRLF files stay w/crlf (protocol.rs, service.rs, settings_host.rs, args.rs, model.rs, docs/FEATURES.md and the other CRLF files of the plan), LF files stay w/lf (portal.rs, restart.rs, wallpaper.rs, engine.rs of WebView2, DECISIONS.md). This SUMMARY is a new file written with LF like the other quick summaries of .planning.

## Known Stubs

None.

## Threat Flags

None. The surface is the one the plan's threat model names (T-0du-01 to T-0du-06): off by default with non-boolean values reading false, the new fields and the SettingChange variant are appended last with the protocol versions bumped, and the hint says that on sends the wallpaper's address to Microsoft. No new endpoint, auth path or file access was added.

## Self-Check: PASSED

- Commits cd43de7e and 9e5c67e6 exist on HEAD (`git merge-base --is-ancestor`); `git rev-list --count` from plan_head_before gives 2.
- Every file the two commits touched exists; the working tree is clean apart from this SUMMARY (not committed, as instructed).
- `tasklist.exe` lists no Sucrose process.
