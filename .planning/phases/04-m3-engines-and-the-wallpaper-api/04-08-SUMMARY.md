---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 08
subsystem: engines
tags: [cef, chromium-switches, developer-mode, devtools, remote-debugging, ipc]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-07 WebOptions and the CEF browser process app (on_before_command_line_processing); 04-02/05 CefHost"
provides:
  - "sucrose_engine_web::args: Switch, CefArguments and WebArguments converters, legacy defaults, SucroseInputs/sucrose_switches, merge, effective_switches, plan_cef_command_line, validate_name, debugging_port, CEF_SETTING_KEYS"
  - "protocol v7: WebOptions { allow_invalid_certificates, arguments, hardware_acceleration, developer_mode, developer_port }; ToRenderer::OpenDevTools (wire index 8)"
  - "CEF browser process appends the merged switches; renderer log lines 'chromium switches', 'cef setting key skipped', 'remote debugging'"
  - "Developer mode on CEF: Inspect context menu item, F12 and Ctrl+Shift+I; remote_debugging_port = DeveloperPort + slot; DevTools never open by themselves (D-13)"
  - "Engine::open_devtools (default Unsupported) and sucrose_platform::os_build"
affects: [04-16, 04-17, round-b-webview2]
plan_head_before: fa581ee8b70fefb8164d4153d1827b3cf86e8331
plan_head_after: 7ad94c647eef998df3a5ca6edf4891ac95826189
actuals:
  tokens: 18100
  tasks: 2
  commits: 3
tech-stack:
  added: []
  patterns:
    - "pure plan (plan_cef_command_line) decides what reaches the CEF command line; the CEF callback only applies it"
    - "pure key_action table for the wallpaper keyboard rules, tested without CEF"
key-files:
  created:
    - crates/sucrose-engine-web/src/args.rs
    - crates/sucrose-engine-web/tests/args.rs
  modified:
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/Cargo.toml
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/Cargo.toml
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/Cargo.toml
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
    - Cargo.lock
key-decisions:
  - "sucrose-ipc depends on sucrose-engine-web so that WebOptions can carry the Switch type; WebOptions is no longer Copy"
  - "An empty CefArguments or WebArguments means the legacy defaults at job creation, as the legacy app did, without writing them back (the Portal editor and Reset are 04-16)"
  - "Where the user and Sucrose set the same switch, Sucrose's value wins and is logged (plan D-17); legacy kept the user's"
  - "A debugging port sum below 1024 is refused like one above 65535, because CEF ignores ports below 1024"
requirements-completed: []
coverage:
  - id: D1
    description: "Arguments model: CEF dictionary and WebView2 list round-trip, Reset gives the legacy defaults (checked against the C# source), unknown switches pass through, bad names are left out"
    requirement: WAPI-10
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine-web args (13 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The user's legacy switches and Sucrose's own reach the CEF browser process; the two CEF setting keys are skipped and logged"
    requirement: WAPI-10
    verification:
      - kind: e2e
        ref: "fixture 'hello' run on a scratch data dir: Renderer log 'chromium switches' has autoplay-policy=no-user-gesture-required, enable-gpu=1, no multi-threaded-message-loop; two 'cef setting key skipped' lines; page loaded status=200"
        status: pass
    human_judgment: false
  - id: D3
    description: "DeveloperPort above 0 makes each CEF renderer listen on the port plus its slot; with 0 no port and no log line"
    requirement: WAPI-09
    verification:
      - kind: e2e
        ref: "DeveloperPort 9222: 'remote debugging port=9222 slot=0' logged and curl /json/version answered Chrome/156; DeveloperPort 0: no remote debugging line"
        status: pass
    human_judgment: false
  - id: D4
    description: "Inspect, F12 and Ctrl+Shift+I open DevTools in developer mode only, and ToRenderer::OpenDevTools shows them; nothing opens by itself"
    requirement: WAPI-09
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine-cef key_action tests; cargo test -p sucrose-ipc open_dev_tools_keeps_its_wire_index; open_devtools_queues_a_command"
        status: pass
    human_judgment: true
    rationale: "A real DevTools window needs a keypress, a right click or the Portal button (04-17); automated runs never synthesize input, so the window itself was not seen"
duration: 16 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 08: Switches and developer tools on CEF Summary

**A CefArguments/WebArguments switch model with the legacy defaults, Sucrose's computed GPU and debugging switches merged without repeated feature lists, fed to the CEF browser process, plus developer mode (Inspect, F12, per-slot debugging port, OpenDevTools) that never opens DevTools by itself.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-11-26T03:27:21Z
- **Completed:** 2026-11-26T03:43:30Z
- **Tasks:** 2
- **Files modified:** 17 (2 created, 15 modified)

## Accomplishments

- `args.rs`: `Switch`, dictionary and list converters, `legacy_defaults_cef/web` (one table, a test parses the C# `Internal.cs` and compares both lists), `sucrose_switches` (GPU on/off by engine and Windows build, debugging origin rule), `merge` (feature lists combined, Sucrose's value wins and is logged), `plan_cef_command_line` (skips `CEF_SETTING_KEYS`, drops invalid names), `debugging_port` (base plus slot, checked).
- The service builds `WebOptions` per chosen engine (empty list means legacy defaults); protocol v7 carries it; the CEF browser process appends the plan and logs `chromium switches` once.
- Developer mode: context menu Inspect, F12 and Ctrl+Shift+I open DevTools in a popup of their own; off, the menu is empty and the legacy blocked keys plus the DevTools shortcuts are swallowed. `remote_debugging_port` is `DeveloperPort + slot`. `ToRenderer::OpenDevTools` (wire index 8) reaches `Engine::open_devtools`; other engines answer Unsupported and the renderer logs it.
- Live proof on the fixture library: the legacy switches reach CEF, the page still loads, and with DeveloperPort 9222 the endpoint answers while DeveloperPort 0 leaves no port.

## Task Commits

1. **Task 1: switches reach the CEF browser process (tracer)** - `f53ce3d2` (model and tests), `17266c76` (service, protocol, CEF wiring, shared with Task 2)
2. **Task 2: developer mode, per-slot port, OpenDevTools** - `17266c76` (code), `7ad94c64` (FEATURES and ARCHITECTURE docs)

**Plan metadata:** committed after this file (docs: complete plan).

## Files Created/Modified

- `crates/sucrose-engine-web/src/args.rs`, `tests/args.rs` - the switches model and its 13 tests
- `crates/sucrose-ipc/src/protocol.rs` - protocol v7, `WebOptions` fields, `OpenDevTools`
- `crates/sucrose-engine-cef/src/browser.rs`, `engine.rs` - command line plan, Inspect/keyboard handlers, port, open_devtools
- `crates/sucrose-engine/src/engine.rs` - `Engine::open_devtools`
- `apps/sucrose/src/service.rs`, `render.rs` - web options per engine, OpenDevTools routing
- `crates/sucrose-platform/src/lib.rs`, `os_windows.rs` - `os_build()`
- `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - rows and note

## Decisions Made

See key-decisions above. The legacy `IsGermanium` removal of the user's `disable-gpu-compositing` (build 26000 and later, CEF only) was kept as `dropped_user_switches`.

## Deviations from Plan

**1. [Rule 2 - Missing critical] Kept the legacy removal of disable-gpu-compositing on build 26000+**
- **Found during:** Task 1 (reading the legacy CefSharp host for the GPU rule)
- **Issue:** Parity: the legacy host drops that user switch on Windows 11 24H2; the plan did not list it.
- **Fix:** `dropped_user_switches` and `effective_switches` in args.rs, tested; recorded in FEATURES.
- **Commit:** f53ce3d2

**2. [Rule 1 - Bug] Debugging ports below 1024**
- **Found during:** Task 2
- **Issue:** CEF ignores a remote debugging port below 1024, so logging it as active would be wrong.
- **Fix:** `args::debugging_port` refuses a sum below 1024 as well as above 65535, with a log line.
- **Commit:** f53ce3d2

**3. [Rule 3 - Blocking] ipc depends on engine-web; WebOptions is no longer Copy**
- **Found during:** Task 1
- **Issue:** The contract puts `Switch` in `sucrose_engine_web::args` and `WebOptions` in the ipc crate.
- **Fix:** `sucrose-ipc` gains the `sucrose-engine-web` path dependency (no new external crate); `Job.web` is cloned.
- **Commit:** 17266c76

**4. [Process] TDD order and commit split**
- The tests and implementation of the model were written together and committed as one feat commit, because a test target for a module that does not exist cannot compile (INVALID_RED); there is no separate `test(04-08)` commit.
- The service, protocol and CEF edits of Task 1 and Task 2 touch the same files, so they share `17266c76` instead of two commits.

**5. [Rule 1 - Bug] clippy print_stderr in the new test**
- **Fix:** removed the `eprintln!` of the skip path in `args_legacy_defaults_equal_the_lists_of_the_legacy_source`; the test returns silently when the legacy tree is gone (M7).

**6. [Process] Line endings**
- `cargo fmt` rewrote several CRLF files to LF; they were converted back to CRLF before committing. `Cargo.lock` stays as cargo writes it (LF).

---

**Total deviations:** 6 (1 missing critical, 2 bugs, 1 blocking, 2 process)
**Impact on plan:** No scope creep beyond legacy parity; WAPI-09 and WAPI-10 stay partial (CEF only).

## Issues Encountered

None beyond the deviations above.

## Known Stubs

None. The DevTools window itself was not seen (see D4 coverage): it needs a keypress, a right click or the Portal's Open DevTools button (04-17).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-16 can build the arguments editor on `from_cef_map`, `to_cef_map`, `from_web_list`, `to_web_list`, `legacy_defaults_*`, `validate_name` and `repeated_names`; 04-17 only has to send `ToRenderer::OpenDevTools`.
- Round B (WebView2) reuses `sucrose_switches` with `WebEngine::WebView2` and `debugging_port`.

## Self-Check: PASSED
