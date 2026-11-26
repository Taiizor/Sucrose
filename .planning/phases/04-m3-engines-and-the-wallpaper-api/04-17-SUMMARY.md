---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 17
subsystem: portal
tags: [portal, devtools, cef, ipc, settings, restart-prompt, gpui]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "ToRenderer::OpenDevTools and Engine::open_devtools (04-08), RuntimeState.running_engines and the shared Restart prompt (04-16)"
provides:
  - "PORTAL_PROTOCOL_VERSION 6: PortalPage::Other, PortalRequest::OpenDevTools { monitor }, PortalEvent::DevToolsResult { opened }, SettingChange::{DeveloperMode, DeveloperPort}, PortalSettings.{developer_mode, developer_port} (all appended)"
  - "ToService::DevToolsOpened(bool) (appended): the renderer confirms to the service that its engine showed the developer tools"
  - "Service routing: OpenDevTools reaches only the renderer of that monitor, only for a Web, Url or YouTube wallpaper on a web engine; answered failed when the renderer does not confirm within 5 s (loop timer 7)"
  - "settings::other::{OtherPage, DevToolsState, open_devtools_state}: the Other tab with the Developer group (Default Developer Tools switch, Debugging Port u16 field, legacy hint, note, Screen select, Open DevTools button)"
  - "row::{PortField, commit_port}: the u16 number field; RestartSetting::{DeveloperMode, DeveloperPort}"
  - "Portal argument OtherSetting (portal_args maps PortalPage::Other to it)"
affects: [04-18, 04-19, 04-20, 04-21]

actuals:
  tokens: 19850
  tasks: 2
  commits: 3
plan_head_before: 3fd03bf99365b6d20a901310a29df70ba3635b9c
plan_head_after: a199962b41d6f29ab362124e68755b11cabe9e89

tech-stack:
  added: []
  patterns:
    - "an action that needs the renderer's answer goes Portal -> service -> renderer -> service -> Portal, with one pending slot and a loop timer so the Portal is never left waiting"
    - "pure decision functions (open_devtools_state in the Portal, devtools_slot in the service) apart from the views and the event loop"

key-files:
  created:
    - apps/sucrose-portal/src/settings/other.rs
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/src/dialogs/restart.rs
    - apps/sucrose-portal/src/settings/mod.rs
    - apps/sucrose-portal/src/settings/row.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - apps/sucrose-portal/locales/legacy.en.yml
    - xtask/src/localize.rs
    - docs/FEATURES.md

key-decisions:
  - "The renderer confirms Open DevTools with an appended ToService::DevToolsOpened(bool); the service answers the Portal on that message or after 5 s as failed, so the Portal never waits for ever"
  - "A second Open DevTools click while the first is still being answered waits for the same answer instead of sending the renderer a second request"
  - "The Portal enables Open DevTools by engine (a wallpaper plays on a web engine); the service additionally refuses wallpapers that are not Web, Url or YouTube, so a Video on a web engine ends in the error toast"
  - "The legacy page argument for the Other tab is OtherSetting (docs/legacy/portal.md)"

patterns-established:
  - "Developer settings are read by both web engines at wallpaper start, so a committed change raises the shared Restart prompt only while WebView2 or CEF plays"

requirements-completed: []

coverage:
  - id: D1
    description: "Settings has an Other tab after Wallpaper with the Developer group: Developer Tools row, Default Developer Tools switch, Debugging Port field, legacy hint, note and the Open DevTools button"
    requirement: WAPI-09
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/mod.rs#settings_tabs_follow_the_legacy_order_with_wallpaper_after_personal_and_other_last"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/main.rs#other_setting_opens_the_other_page"
        status: pass
    human_judgment: true
    rationale: "Layout and copy of the page were checked on one capture (916x608, dark theme); a maintainer eye check of spacing and wording is still open"
  - id: D2
    description: "Open DevTools goes Portal, service, the renderer of that monitor, CEF ShowDevTools, and answers the Portal; nothing opens DevTools by itself"
    requirement: WAPI-09
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#open_devtools_reaches_the_web_renderer"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#open_devtools_goes_only_to_a_running_web_wallpaper_on_a_web_engine"
        status: pass
    human_judgment: false
  - id: D3
    description: "Open DevTools button state: disabled with no web wallpaper, direct target with one, Screen select with several"
    requirement: WAPI-09
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/other.rs#other_open_devtools_asks_for_the_screen_when_several_web_wallpapers_run"
        status: pass
    human_judgment: false
  - id: D4
    description: "Default Developer Tools and Debugging Port are saved by the service (port 0-65535, empty field reverts) and raise the shared Restart prompt once per committed change while a web wallpaper runs"
    requirement: WAPI-09
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#settings_host_developer_mode_and_port_are_written_to_the_engine_section_and_shown"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/restart.rs#restart_the_developer_settings_prompt_only_while_a_web_engine_wallpaper_runs"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/row.rs#settings_port_above_the_range_is_cut_to_65535"
        status: pass
    human_judgment: true
    rationale: "The prompt opening on a real commit (Enter or blur in the port field) was not driven; no input may be synthesized on the maintainer's desktop"

duration: 17 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 17: Open DevTools and the Developer settings Summary

**A new Other tab with the Developer group: Open DevTools reaches the running web wallpaper's renderer through the service and CEF ShowDevTools on purpose, and the developer switch and debugging port are saved by the service with the shared Restart prompt.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-11-26T16:16:38Z
- **Completed:** 2026-11-26T16:33:30Z
- **Tasks:** 2 (the first a tracer, verified end to end)
- **Files modified:** 20

## Accomplishments

- Open DevTools: button (Other tab) -> PortalRequest::OpenDevTools -> the service picks the slot of that monitor (only a Web, Url or YouTube wallpaper on WebView2 or CEF) -> ToRenderer::OpenDevTools -> the renderer calls the engine, logs `devtools opened` and answers ToService::DevToolsOpened -> PortalEvent::DevToolsResult. A failure or a missing answer within 5 s shows the error toast "DevTools couldn't be opened."
- The tracer gate passed end to end: `open_devtools_reaches_the_web_renderer` plays the `hello` fixture on CEF, asks for an unknown monitor (answer false, nothing opens) and for the real one (answer true, renderer log `devtools opened`); no process was left running.
- The Other page shows the Developer Tools row with the Default Developer Tools switch, the Debugging Port number field (u16; text above 65535 clamps to 65535, an empty field reverts, nothing is sent when the value equals the stored one), the legacy hint with the port in its address, the note "Developer tools no longer open by themselves...", the Screen select (only with several web wallpapers) and the Open DevTools button with its disabled tooltip.
- DeveloperMode and DeveloperPort are validated, written to the Engine section and shown by the settings host; both raise the shared Restart prompt through `should_prompt` only while a web-engine wallpaper runs.
- Nothing in the code calls DevTools at start (D-13); FEATURES records the Portal button and the Other tab.

## Task Commits

1. **Task 1 (tracer): Open DevTools opens the inspector of the running Web wallpaper** - `be6a2b5a` (feat)
2. **Task 2: Default Developer Tools and Debugging Port saved through the service with the restart prompt; strings and FEATURES** - `b96d7795` (feat)
3. **Fix: array length of the settings-apply test table** - `a199962b` (fix)

**Plan metadata:** committed after this SUMMARY (docs: complete plan).

_TDD note: the tests were written with the code in each task commit; a separate RED commit was not made (Rust unit tests share the files with the code they test)._

## Files Created/Modified

- `apps/sucrose-portal/src/settings/other.rs` - the Other page: Developer group, `open_devtools_state`, `restart_trigger`, `hint_text`
- `crates/sucrose-ipc/src/portal.rs` - protocol version 6 and the appended variants and fields
- `crates/sucrose-ipc/src/protocol.rs` - `ToService::DevToolsOpened`
- `apps/sucrose/src/service.rs` - OpenDevTools routing, pending slot and 5 s timer, `devtools_slot`
- `apps/sucrose/src/render.rs` - the renderer confirms and logs `devtools opened`
- `apps/sucrose/src/settings_host.rs` - validate, write and view for the developer settings
- `apps/sucrose/src/portal_link.rs` - `OtherSetting` argument for `PortalPage::Other`
- `apps/sucrose-portal/src/{main,header,app,i18n}.rs`, `settings/{mod,row,wallpaper}.rs`, `dialogs/restart.rs` - the tab, the argument, the wiring, the u16 field, the prompt settings
- `apps/sucrose-portal/locales/*.yml`, `xtask/src/localize.rs` - the `Portal.OtherSettingPage.Developer` prefix and the new keys
- `docs/FEATURES.md` - the developer row

## Decisions Made

See key-decisions above. The Video-on-a-web-engine case is the one known gap between the Portal (enables by engine) and the service (refuses by wallpaper type): the Portal cannot tell the type from `running_engines`; the result is the error toast, not a wrong DevTools window.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The renderer needed a way to confirm, and three files outside files_modified changed**
- **Found during:** Task 1
- **Issue:** The plan has the service answer "when the renderer confirms", but the renderer protocol had no message for it, and the Portal page enum needed a launcher mapping.
- **Fix:** Appended `ToService::DevToolsOpened(bool)` (crates/sucrose-ipc/src/protocol.rs, with a wire-index test), the renderer sends it (apps/sucrose/src/render.rs, which also logs `devtools opened` instead of `developer tools requested`), and `portal_args` maps `PortalPage::Other` (apps/sucrose/src/portal_link.rs). Appending keeps PROTOCOL_VERSION 8.
- **Files modified:** crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/render.rs, apps/sucrose/src/portal_link.rs
- **Committed in:** be6a2b5a

**2. [Rule 1 - Bug] i18n required-keys test failed after the new keys**
- **Found during:** Task 2 (full Portal test run)
- **Issue:** `every_new_key_is_required` lists each key of sucrose.en.yml; Task 1 added keys without listing them.
- **Fix:** Listed the new keys and the legacy Developer keys in apps/sucrose-portal/src/i18n.rs.
- **Committed in:** b96d7795

**3. [Rule 1 - Bug] sucrose-ipc tests did not compile**
- **Found during:** final `cargo xtask ci`
- **Issue:** The settings-apply test table had a declared length of 18 and grew to 20.
- **Fix:** Length set to 20.
- **Committed in:** a199962b

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 bugs)
**Impact on plan:** All needed for the plan's behavior or its tests; no scope creep.

## Issues Encountered

None beyond the deviations. No known flaky test failed; `cargo xtask ci` passed (exit 0).

## Line endings

Before the first edit `git ls-files --eol` was recorded; the CRLF files touched are apps/sucrose-portal/src/main.rs, apps/sucrose/src/{service,render,settings_host}.rs, crates/sucrose-ipc/src/protocol.rs, xtask/src/localize.rs and docs/FEATURES.md. `cargo fmt` turned service.rs to LF once; it was restored to CRLF before the first commit and every file matched its original ending at each commit. FEATURES still has no newline after its last line. The other touched files were and are LF.

## Visual check

A debug build was started without focus on a scratch data folder under target/ with the `OtherSetting` page and captured with capture.ps1 (916x608, dark): the Developer group, switch, port field, hint, note and the enabled Open DevTools button render in the layout of the other pages. Every process was stopped afterwards (`tasklist.exe` shows none).

## Known Stubs

None.

## Threat Flags

None. T-04-39: the request names only a monitor, the service routes it to its own renderer and only for web engines. T-04-40: the port is a developer setting with the legacy hint and 0 (off) by default.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

WAPI-09 stays partial (CEF only; WebView2 in Round B), so `requirements-completed` is empty. The Other tab is ready for the rest of the legacy Other page (UI-04) to be added as further groups.

## Self-Check: PASSED

- `apps/sucrose-portal/src/settings/other.rs` exists; commits `be6a2b5a`, `b96d7795` and `a199962b` are on the branch.
- `cargo xtask ci` exits 0; `open_devtools_reaches_the_web_renderer` reports 1 passed (renderer log `devtools opened`); `cargo xtask localize --lang EN --check` is current.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
