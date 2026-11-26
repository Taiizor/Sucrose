---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 16
subsystem: portal
tags: [portal, cef, arguments, certificates, restart-prompt, ipc, settings, gpui]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "sucrose_engine_web::args (04-08), Engine.allow_invalid_certificates (04-07), the Web Engine Settings group with the CEF row (04-15)"
provides:
  - "PORTAL_PROTOCOL_VERSION 5: SettingChange::{AllowInvalidCertificates, CefArguments}, SwitchView, PortalRequest::{GetArguments, RestartWallpapers}, PortalEvent::{Arguments, RestartFailed}, RuntimeState.running_engines, PortalSettings.allow_invalid_certificates (all appended)"
  - "Service side: certificate switch stored, CEF switch list validated (names, duplicates, limits) and stored in the legacy CefArguments dictionary, GetArguments answer (stored, Sucrose own, legacy defaults), RestartWallpapers restarting only the renderers of the listed engines with the settings as saved"
  - "dialogs::restart::{RestartSetting, affected_engines, should_prompt, engines_text, prompt_text, open_restart_prompt}: the shared Restart wallpapers prompt"
  - "settings::arguments::{ArgumentsDraft, ArgumentsEditor}: the Arguments editor of the CEF row"
  - "The Allow invalid certificates switch as the first row of the Web Engine Settings group"
affects: [04-17, 04-18, 04-19, 04-20]

actuals:
  tokens: 29950
  tasks: 2
  commits: 2
plan_head_before: f66c0b51ae44fcb8f9c1dfdf938623412871da95
plan_head_after: 2f249af30abd0d1ceaf612142736a5e3847c5c7b

tech-stack:
  added: []
  patterns:
    - "the service owns the rules and the storage format of the switches; the Portal gets ready-to-show lists over IPC and links no engine crate"
    - "a saved setting that a renderer reads only at start raises one prompt after the service broadcast (previous and committed value compared on the broadcast), and nothing restarts without Restart now"
    - "a pure draft model (rows, dirty, errors) apart from the view; per-row InputState pairs rebuilt after structural changes"

key-files:
  created:
    - apps/sucrose-portal/src/dialogs/restart.rs
    - apps/sucrose-portal/src/settings/arguments.rs
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/settings/mod.rs
    - apps/sucrose-portal/src/dialogs/mod.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
    - docs/GPUI.md

key-decisions:
  - "A restart rebuilds the job from the settings as saved now and hands the engine and the web options to every slot's assignment (Fleet::adopt_job did not carry them before), so a restarted renderer reads the new certificate switch, arguments and hardware acceleration"
  - "RestartWallpapers restarts the wallpaper only when the job's engine is listed (one job plays on every monitor); refused with RestartFailed while the library moves"
  - "GetArguments answers an empty stored dictionary with the legacy defaults (the engine reads them that way, as in the legacy app), so saving an empty list cannot store 'no switches at all'"
  - "List limits keep the Arguments answer inside one 64 KiB IPC frame: 64 switches, names 64 bytes, values 512 bytes"
  - "RestartSetting::Arguments carries the engine, so the CEF arguments restart only CEF wallpapers"

patterns-established:
  - "Restart prompt trigger: restart_trigger(change, previous, now) gives the setting and both values, should_prompt compares them and the running engines; a refused change leaves equal values and never prompts"

requirements-completed: []

coverage:
  - id: D1
    description: "Portal protocol 5 appends the certificate and CEF argument changes, GetArguments, RestartWallpapers, Arguments, RestartFailed and RuntimeState.running_engines, each with its wire index"
    requirement: "WAPI-10"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc portal::tests (protocol version, request, event and setting change wire indexes, the Arguments answer under the frame limit)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service stores the certificate switch, validates and stores CEF switches in the legacy dictionary (unknown switches stored and logged) and answers GetArguments"
    requirement: "WAPI-10"
    verification:
      - kind: unit
        ref: "apps/sucrose settings_host::tests (certificate switch, cleaned list stored as dictionary, bad or repeated names refused, limits, arguments answer, defaults for an empty list)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Restart now restarts only the wallpapers on the affected engines and the new renderer reads the saved certificate switch"
    requirement: "ENG-03"
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#restart_now_restarts_only_the_affected_wallpapers (ran with --include-ignored on the desktop from a scratch folder)"
        status: pass
      - kind: unit
        ref: "apps/sucrose service::tests (engines_running, restart_affects, a restart hands the engine and web options to the assignments)"
        status: pass
    human_judgment: false
  - id: D4
    description: "The restart prompt text, the engines it names, and when it is raised (value changed and an affected wallpaper runs)"
    requirement: "WAPI-10"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal dialogs::restart::tests, settings::wallpaper::tests::restart_trigger_*"
        status: pass
    human_judgment: true
    rationale: "The dialog itself (480 px, Restart now focused, Escape and outside click mean Later, focus returns to the control) was not opened in an automated run: opening it needs a click on the switch, which automation must not synthesize"
  - id: D5
    description: "The Arguments editor draft model: add, remove, reset, dirty, invalid names, duplicates, limits, saved list"
    requirement: "WAPI-10"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal settings::arguments::tests"
        status: pass
    human_judgment: false
  - id: D6
    description: "The Web Engine Settings group shows the certificate switch with its info hint and the CEF row with the Arguments editor (read-only Sucrose rows, user rows, buttons)"
    requirement: "WAPI-10"
    verification:
      - kind: other
        ref: "PrintWindow capture target/web-lib/portal-arguments.png (debug build, scratch data folder, group moved to the top for the capture)"
        status: pass
    human_judgment: true
    rationale: "Visual fit, the empty and error states and the scrolling past 240 px need the maintainer's eye; automated captures cannot click Add argument or type"

duration: 27 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 16: Certificates, CEF arguments and the restart prompt Summary

**The Web Engine Settings group gets the Allow invalid certificates switch and a CEF Arguments editor whose rules the service owns, and a shared Restart wallpapers prompt applies a saved certificate, arguments or hardware acceleration change to the running wallpapers only when the user chooses Restart now.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-11-26T15:45:00Z
- **Completed:** 2026-11-26T16:12:00Z
- **Tasks:** 2
- **Files modified:** 18 (2 created)

## Accomplishments

- Portal protocol 5 with the certificate switch, the CEF switch list, the argument and restart requests, and the engines of the running wallpapers in the runtime state.
- Service side: the switch is stored, a switch list is validated (letters, digits and dashes, no repeats, bounded), stored in the legacy `CefArguments` dictionary and answered back with Sucrose's own switches and the legacy defaults; `RestartWallpapers` restarts only the renderers on the listed engines from the settings as saved.
- Portal: the certificate switch with the info hint, the shared restart prompt (also for Hardware Acceleration), and the Arguments editor in the CEF row (read-only rows, user rows, Add, Reset, Save, loading, empty and error states, 240 px scrolling list).
- The integration test ran on the desktop: with a Web wallpaper on CEF the switch is saved without a restart, a restart for mpv leaves the renderer alone, a restart for CEF starts a new renderer whose log shows `allow_invalid_certificates=true`.

## Task Commits

1. **Task 1: certificate switch, restart prompt, RestartWallpapers (tracer)** - `6947e46a` (feat)
2. **Task 2: the CEF arguments editor** - `2f249af3` (feat)

**Plan metadata:** the docs commit that carries this file, STATE.md and ROADMAP.md.

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs` - protocol 5: changes, requests, events, `SwitchView`, limits, `running_engines`
- `apps/sucrose/src/settings_host.rs` - certificate switch, switch list validation and storage, `arguments_answer`
- `apps/sucrose/src/service.rs` - `RestartWallpapers`, `GetArguments`, `engines_running`, assignments carry engine and web options on a restart
- `apps/sucrose/src/portal_link.rs` - `runtime_state` takes the running engines
- `apps/sucrose/tests/portal.rs` - `restart_now_restarts_only_the_affected_wallpapers`
- `crates/sucrose-engine-cef/src/browser.rs` - the chromium switches log line says whether invalid certificates are allowed
- `apps/sucrose-portal/src/dialogs/restart.rs` - the shared prompt
- `apps/sucrose-portal/src/settings/arguments.rs` - the draft model and the editor
- `apps/sucrose-portal/src/settings/wallpaper.rs`, `app.rs`, `mod.rs`, `dialogs/mod.rs`, `assets.rs`, `i18n.rs`, `library/card.rs`, `locales/sucrose.en.yml` - wiring, icons (Lock, Plus, X), strings
- `docs/FEATURES.md`, `docs/GPUI.md` - the two rows and the new lessons

## Decisions Made

See key-decisions. The empty-list limit is recorded in the FEATURES row.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] A restart did not hand the new engine and web options to the assignments**
- **Found during:** Task 1
- **Issue:** `Fleet::adopt_job` updated source, settings, kind and theme_dir of the shared assignments but not `engine` and `web`, so a restarted renderer would have read the old certificate switch, switches and hardware acceleration.
- **Fix:** `adopt_job` also sets `assignment.engine` and `assignment.web`; a unit test covers it.
- **Files modified:** apps/sucrose/src/service.rs
- **Commit:** 6947e46a

**2. [Rule 2 - Missing critical] Bounds for the switch list and its answer**
- **Found during:** Task 1 (the IPC frame is 64 KiB)
- **Issue:** the plan sets no limits; an unbounded list could not be sent back in one frame.
- **Fix:** at most 64 switches, names 64 bytes, values 512 bytes (`MAX_SWITCHES`, `MAX_SWITCH_NAME`, `MAX_SWITCH_VALUE`); the service refuses a larger list and cuts a stored legacy list in its answer. Tests cover both.
- **Files modified:** crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/settings_host.rs
- **Commit:** 6947e46a

**3. [Rule 3 - Blocking] Contract additions the plan did not list**
- **Issue:** the Portal needs the stored certificate value (`PortalSettings.allow_invalid_certificates`), a way to be told a restart failed (`PortalEvent::RestartFailed`, shown as the Use-failed style toast with a new `Portal.RestartPrompt.Failed` string), and `app.rs`/`card.rs`/`portal_link.rs`/`browser.rs` edits to route events, fix fixtures and log the certificate option for the integration test.
- **Commit:** 6947e46a, 2f249af3

**4. Task split.** The service-side argument rules (`check_switches`, `arguments_answer`, `GetArguments`) landed in the Task 1 commit together with the protocol they belong to; Task 2 is the Portal editor, its strings, icons and docs.

---

**Total deviations:** 3 auto-fixed (1 bug, 1 missing critical, 1 blocking) and 1 commit-split note.
**Impact on plan:** none on scope; every item is needed for the requirement to work.

## Issues Encountered

- A tall settings page cannot be captured whole: Windows clamps the window to the monitor height. The capture was taken with the Web Engine Settings group moved to the top in a temporary local edit, reverted before the commit (recorded in docs/GPUI.md).
- `cargo fmt` through the Windows wrapper rewrote the CRLF working copies to LF; restored CRLF before each commit for `apps/sucrose/src/service.rs`, `apps/sucrose/src/settings_host.rs` and `crates/sucrose-engine-cef/src/browser.rs` (`docs/FEATURES.md` and `docs/GPUI.md` stayed CRLF). The other touched files were LF and stay LF. No known flaky test failed in `cargo xtask ci` (1322 passed).
- The restart dialog was not opened visually (it needs a click on the switch); its pure parts and the service path are tested and the dialog follows the pattern of the Move Library dialog.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model: T-04-36 (service-side name validation, duplicates refused, unknown names logged, values never logged), T-04-37 (off by default, the switch has the less-secure hint, applies only after a restart), T-04-38 (only listed engines restart, once per request, budgets reset like a Use).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

04-17 (Developer group, Open DevTools) can add `RestartSetting::DeveloperMode` and `DeveloperPort` to the same prompt and reuse `restart_trigger`. WebView2's editor row (Round B) reuses `ArgumentsEditor` with `EngineType::WebViewLive`; `GetArguments` already answers it from the WebArguments list, but a `SettingChange` for its list does not exist yet.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED

- apps/sucrose-portal/src/dialogs/restart.rs and apps/sucrose-portal/src/settings/arguments.rs exist; commits 6947e46a and 2f249af3 are ancestors of HEAD; `cargo xtask ci` passes; no sucrose-engine dependency in apps/sucrose-portal/Cargo.toml.
