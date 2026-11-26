---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 25
subsystem: engines
tags: [webview2, chromium-switches, devtools, certificates, input-forwarding, cdp]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "the switches model (04-07), CEF developer mode, certificates and input forwarding (04-08, 04-17, 04-19), the WebView2 host and engine (04-22), the shared page module (04-24)"
provides:
  - "sucrose_engine_web::args::{plan_webview2_command_line, browser_arguments, WebView2Plan}: the AdditionalBrowserArguments text from effective_switches with WebEngine::WebView2"
  - "sucrose_engine_web::keys::{KeyAction, key_action}: the legacy key table both web engines apply (moved from the CEF crate)"
  - "WebView2 with the user's and Sucrose's switches, the debugging port, developer mode, Open DevTools, the certificate rule and input forwarding (mouse and keys posted to Chrome_WidgetWin_0, wheel through Input.dispatchMouseEvent)"
affects: [04-26, 04-27, 04-37]

actuals:
  tokens: 18900
  tasks: 3
  commits: 3
plan_head_before: f71e5d69b2f104b31803ba0fca3cc020345c8462
plan_head_after: 17bd94e2dc468ee08a8c22baae0d43cd28425d93

tech-stack:
  added: []
  patterns:
    - "pure helpers (browser_arguments, view_settings, uri_host, wheel_params, the key table) are tested on every OS; the COM calls stay in the Windows-only Host"
    - "input is coalesced like CEF's: one queued marker per burst of mouse moves, the newest move kept in a shared slot"

key-files:
  created:
    - crates/sucrose-engine-web/src/keys.rs
    - crates/sucrose-engine-web/tests/keys.rs
  modified:
    - crates/sucrose-engine-web/src/args.rs
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/tests/args.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-webview2/Cargo.toml
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - apps/sucrose/tests/portal.rs
    - docs/FEATURES.md

key-decisions:
  - "The browser arguments are planned in sucrose-engine-web (plan_webview2_command_line + browser_arguments), so the quoting and the invalid-name rule are tested on every OS; a value with a blank or a quote is wrapped in quotes with Windows command line escaping"
  - "The developer tools stay enabled in both modes (Open DevTools needs them); the context menu and the AcceleratorKeyPressed key table keep them closed unless developer mode is on"
  - "The CEF portal test body became a shared open_devtools_roundtrip helper that the WebView2 test also calls, skipping when another engine started"
  - "The input widget is looked up once after the first completed navigation (and again after a failed post), searching the surface's own window tree two levels deep for Chrome_WidgetWin_0, else _1"

patterns-established:
  - "an engine crate owns only the thin adapter; the shared rule tables (switches, keys, page calls, serve planner) live in sucrose-engine-web"

requirements-completed: []

coverage:
  - id: D1
    description: "WebView2 starts with the user's WebArguments (legacy defaults when empty) merged with Sucrose's GPU and debugging switches, logs chromium switches once, and DeveloperPort + slot opens the debugging port (ENG-02, WAPI-10 engine side, WAPI-09 on WebView2)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/args.rs#args_the_webview2_text_merges_the_feature_lists_into_one_switch_with_a_port"
        status: pass
      - kind: e2e
        ref: "hello fixture on WebView2 with DeveloperPort 9222: curl http://127.0.0.1:9222/json/version answered Edg/158.0.4370.45; Renderer log: remote debugging port=9222 slot=0 and the chromium switches line with autoplay-policy"
        status: pass
    human_judgment: true
    rationale: "ENG-02, WAPI-09 and WAPI-10 complete only at the final checkpoint 04-37"
  - id: D2
    description: "Developer tools never open by themselves on WebView2; the key table swallows the blocked keys and, without developer mode, the DevTools shortcuts; Open DevTools opens them (D-13)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/keys.rs#without_developer_mode_the_browser_keys_and_the_devtools_shortcuts_are_swallowed"
        status: pass
      - kind: e2e
        ref: "apps/sucrose/tests/portal.rs#open_devtools_reaches_the_webview2_renderer (engine started engine=\"WebView2\", unknown monitor false, real monitor true, devtools opened)"
        status: pass
    human_judgment: true
    rationale: "That F12, Ctrl+Shift+I and the context menu behave as described needs a real keyboard and mouse; no input is synthesized here (the maintainer's check is 04-37)"
  - id: D3
    description: "A certificate error handler (AlwaysAllow, host-only log) is registered only with Allow invalid certificates; without it Chromium validates (D-12)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-webview2/src/host.rs#the_developer_tools_exist_in_both_modes_but_the_menu_and_the_certificate_handler_follow_the_settings"
        status: pass
    human_judgment: true
    rationale: "The badssl check needs network access and is the maintainer's check in 04-37"
  - id: D4
    description: "A Web wallpaper on WebView2 takes input: raw input is registered, legacy mouse and key messages go to Chrome_WidgetWin_0, the wheel goes through Input.dispatchMouseEvent divided by the monitor scale (WAPI-07 on WebView2)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-webview2/src/host.rs#the_wheel_delta_is_divided_by_the_monitor_scale_and_the_vertical_amount_is_negated"
        status: pass
      - kind: e2e
        ref: "input fixture on WebView2: 'raw input registered mouse=true keyboard=true', 'engine started engine=WebView2', 'input target found class=Chrome_WidgetWin_0' after the first load"
        status: pass
    human_judgment: true
    rationale: "WAPI-07 is partial in Round B and completes in Round E; the real forwarding with a mouse and keyboard is checked by the maintainer in 04-37"

duration: 15min
completed: 2026-11-27
status: complete
---

# Phase 04 Plan 25: WebView2 honours the web engine settings Summary

**WebView2 now gets the same switch model, developer mode, certificate rule and input forwarding as CEF: the user's switches merged with Sucrose's own go into AdditionalBrowserArguments, DeveloperPort + slot opens a real debugging port, the shared key table keeps DevTools closed until asked, and mouse, keys and wheel reach the page (wheel through the DevTools protocol).**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-26T23:28:00Z
- **Completed:** 2026-11-26T23:43:00Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Task 1 (tracer): `args::plan_webview2_command_line` and `args::browser_arguments` build the environment's AdditionalBrowserArguments from `effective_switches` with `WebEngine::WebView2`; the renderer logs `chromium switches` once and `remote debugging port=...` when set. Verified live: with DeveloperPort 9222 the slot 0 renderer answers `/json/version` (Edg/158.0.4370.45); a base of 0 opens none and an out-of-range sum is refused by the existing `debugging_port`.
- Task 2: the legacy key table moved to `sucrose_engine_web::keys` (CEF re-exports it, behaviour unchanged). On WebView2 the tools are enabled but closed: the context menu shows only in developer mode, `AcceleratorKeyPressed` runs the table (blocked keys and, without developer mode, the DevTools shortcuts are swallowed; with it on F12 and Ctrl+Shift+I call `OpenDevToolsWindow`). The certificate handler is registered only with Allow invalid certificates and logs the host only. `open_devtools` queues a command and the pump calls `OpenDevToolsWindow`; `open_devtools_reaches_the_webview2_renderer` passes (and `open_devtools_reaches_the_web_renderer` still passes on the shared helper).
- Task 3: `WebView2Engine::takes_input`/`forward_input` with CEF's move coalescing; the pump posts the legacy messages to the first `Chrome_WidgetWin_0` under the surface, else `_1`, found after the first navigation (log: `input target found class="Chrome_WidgetWin_0"`); the wheel runs `Input.dispatchMouseEvent` with delta and position divided by the monitor scale. FEATURES row 73 updated.
- `cargo xtask ci` (fmt, clippy -D warnings, tests) passes.

## Task Commits

1. **Task 1: switches reach WebView2 and open its debugging port** - `1bf3a47a` (feat)
2. **Task 2: developer mode, Open DevTools and the certificate rule** - `ae3669ed` (feat)
3. **Task 3: input reaches a Web wallpaper on WebView2** - `17bd94e2` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

- `crates/sucrose-engine-web/src/keys.rs`, `tests/keys.rs` - the key table (moved) and its tests
- `crates/sucrose-engine-web/src/args.rs`, `tests/args.rs` - WebView2 plan, argument text and quoting, tests
- `crates/sucrose-engine-web/src/lib.rs` - `pub mod keys`
- `crates/sucrose-engine-cef/src/browser.rs` - re-exports the moved table, its tests moved
- `crates/sucrose-engine-webview2/src/host.rs` - view settings, key handler, certificate handler, `open_devtools`, widget lookup, `post_to_widget`, `send_mouse_wheel`, `wheel_params`, `uri_host`
- `crates/sucrose-engine-webview2/src/engine.rs` - arguments and port, `open_devtools`, `takes_input`, `forward_input`, input drain
- `crates/sucrose-engine-webview2/Cargo.toml` - the `Win32_UI_Input_KeyboardAndMouse` feature of the already pinned `windows` crate (GetKeyState for the key chords)
- `apps/sucrose/tests/portal.rs` - `open_devtools_reaches_the_webview2_renderer` and the shared `open_devtools_roundtrip`
- `docs/FEATURES.md` - row 73

## Decisions Made

See the `key-decisions` front matter. No entry in `docs/DECISIONS.md` was needed: D-12, D-13 and D-17 are applied to WebView2 as written.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added a windows crate feature for the key chord state**
- **Found during:** Task 2
- **Issue:** `AcceleratorKeyPressed` gives the virtual key but not Ctrl and Shift; the table needs them. `GetKeyState` needs `Win32_UI_Input_KeyboardAndMouse`.
- **Fix:** enabled that feature on the existing workspace `windows` dependency in the WebView2 crate; no new crate.
- **Files modified:** crates/sucrose-engine-webview2/Cargo.toml
- **Commit:** `ae3669ed`

**2. [Rule 1 - Bug] IPv6 host in the certificate log line**
- **Found during:** Task 2 (own unit test)
- **Issue:** `uri_host` returned `[` for `https://[::1]:9000/`.
- **Fix:** strip the brackets before splitting; test added.
- **Files modified:** crates/sucrose-engine-webview2/src/host.rs
- **Commit:** `ae3669ed`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact:** none on scope.

## Line endings

Restored CRLF (they were `w/crlf` before this plan) on `crates/sucrose-engine-web/src/args.rs`, `src/lib.rs`, `tests/args.rs`, `crates/sucrose-engine-cef/src/browser.rs` and `docs/FEATURES.md` (the last keeps no newline after its final line). The WebView2 sources, `apps/sucrose/tests/portal.rs`, the crate's Cargo.toml and the two new files are LF. `git status` keeps showing `args.rs` and `tests/args.rs` as modified with an empty diff: a line-ending stat artifact, not touched further.

## Issues Encountered

None. No flaky test needed a rerun.

## Authentication Gates

None.

## Known Stubs

None.

## Threat Flags

None. T-04-55 (no certificate handler by default), T-04-56 (port only with DeveloperPort > 0, logged) and T-04-57 (input filtered by the renderer, never logged) are applied as planned.

## Not checked here

The real mouse, keyboard and wheel forwarding on WebView2, F12 and the context menu with a real keyboard, and the badssl certificate check were not run: no input is synthesized and the network check belongs to the maintainer's checkpoint 04-37. ENG-02, WAPI-09, WAPI-10 and WAPI-07 are therefore not marked complete (`requirements-completed: []`).

## Next Phase Readiness

Ready for 04-26 (the Portal half of WAPI-10).

## Self-Check: PASSED

Files created exist (keys.rs, tests/keys.rs); commits 1bf3a47a, ae3669ed and 17bd94e2 are ancestors of HEAD; the acceptance greps pass (effective_switches/plan_webview2_command_line, WebEngine::WebView2, fn forward_input, Input.dispatchMouseEvent, ServerCertificateErrorDetected, pub fn key_action, the portal test name, FEATURES.md still w/crlf); no Rust Sucrose process is left running.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
