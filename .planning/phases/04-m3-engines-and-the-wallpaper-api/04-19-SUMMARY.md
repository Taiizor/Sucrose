---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 19
subsystem: engines
tags: [rawinput, input-forwarding, cef, win32, ipc]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "The Engine trait and CefEngine/CefPump command queue (04-05), the renderer event loop, the Engine section settings read when a job is made (04-14)"
provides:
  - "sucrose_platform::input: InputEvent, MouseButton, InputFilter, mouse_message, key_message, key_lparam, should_forward, cursor_inside, devices, is_desktop_class, key_event, mouse_events, focus_click_messages"
  - "LoopEvent::Input, EventLoop::register_raw_input (RIDEV_INPUTSINK on the loop window), sucrose_platform::cursor_position"
  - "Engine::takes_input and Engine::forward_input; CefEngine queues input for the surface thread and CefHost posts the legacy messages to the Chromium widget"
  - "Assignment.input: InputSettings { input_type, desktop_only } (PROTOCOL_VERSION 9); renderer log line raw input registered"
  - "Fixture web theme input and the FEATURES row for mouse and keyboard forwarding"
affects: [04-20, 04-21, round-b-url-input, round-e-application-input, phase-verifier]
plan_head_before: 4812ef33198f70e3a5e4d12e4f82529ec6e38036
plan_head_after: 353649b46548461b54e04cce5a2528298eac7b32
actuals:
  tokens: 18800
  tasks: 2
  commits: 4
tech-stack:
  added: ["windows features Win32_UI_Input and Win32_UI_Input_KeyboardAndMouse in sucrose-platform", "sucrose-engine now depends on sucrose-platform (path crate, for InputEvent)"]
  patterns:
    - "Pure input model and legacy message mapping on every OS; the OS glue only produces InputEvent values"
    - "Mouse moves coalesced in a state slot with one queue marker per burst, so a fast mouse never fills the command queue"
    - "InputEvent Debug redacts which key it was: nothing the user types reaches a log or a panic message"
key-files:
  created:
    - crates/sucrose-platform/src/input.rs
    - crates/sucrose-platform/src/os_windows/raw_input.rs
    - apps/sucrose/tests/fixtures/web-themes/input/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/input/index.html
  modified:
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_windows/event_loop.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine/Cargo.toml
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md
    - Cargo.lock
key-decisions:
  - "Keys follow the per-monitor cursor rule like the mouse: only the renderer whose rectangle holds the cursor forwards keystrokes (new with one renderer per monitor; the plan's truth says a renderer forwards only while the cursor is inside its rectangle)"
  - "The legacy key lParam is kept exactly, including the extended-key bit that is always set, so key_lparam takes only the scan code and the direction; InputEvent::Key still records the E0 flag for later use"
  - "The OnlyKeyboard focus click is an InputEvent::FocusClick variant made by the renderer before its first forwarded key, so the engine does not need to know the InputType"
  - "Forwarded positions are relative to the renderer's own rectangle (the surface origin the pump learns from the assignment and from SetRect), which is what the page's client coordinates need"
requirements-completed: []
duration: 20 min
completed: 2026-11-26
status: complete
coverage:
  - id: D1
    description: "A renderer playing a Web theme on CEF registers raw mouse and keyboard input on its own hidden window and forwards the legacy mouse messages to the Chromium widget, relative to its monitor, only while the cursor is inside its rectangle"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-platform input (mouse_message, cursor_inside, should_forward, mouse_events)"
        status: pass
      - kind: integration
        ref: "renderer of the input fixture logs 'raw input registered mouse=true keyboard=true' (cargo xtask synth-library --select input, target/debug/sucrose.exe --no-portal)"
        status: pass
    human_judgment: true
    rationale: "Raw input comes from the real mouse and the project never synthesizes input; the dot following the pointer on the monitor under it is checked by the maintainer at 04-20"
  - id: D2
    description: "Keyboard, wheel and the legacy filters (InputType, InputDesktop, OnlyKeyboard focus click, dead keys, legacy lParam) apply to forwarded events"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-platform input (17 tests: filters, key_lparam, dead keys, focus click, wheel delta, desktop class, redacted Debug)"
        status: pass
      - kind: unit
        ref: "apps/sucrose service tests (the_input_settings_of_a_job_follow_the_engine_section) and sucrose-ipc protocol tests (an_assignment_survives_the_wire)"
        status: pass
    human_judgment: true
    rationale: "Keystrokes and the wheel come from real devices; the page showing the typed keys and Mouse Only dropping them is checked by the maintainer at 04-20"
  - id: D3
    description: "The input settings reach the renderer in its Assignment and apply at the next start; mpv renderers register no raw input"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc/src/protocol.rs#an_assignment_survives_the_wire, apps/sucrose/src/service.rs#the_input_settings_of_a_job_follow_the_engine_section"
        status: pass
    human_judgment: false
---

# Phase 04 Plan 19: Raw input forwarding to CEF pages Summary

**RawInput (RIDEV_INPUTSINK) on the renderer's hidden window, the legacy mouse, key and wheel messages posted to the CEF Chromium widget, the per-monitor cursor rule and the InputType / InputDesktop filters from the Assignment.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-11-26T17:15:15Z
- **Completed:** 2026-11-26T17:35:00Z
- **Tasks:** 2
- **Files modified:** 20 (4 created)

## Accomplishments

- A Web wallpaper on CEF registers raw mouse and keyboard input on its own loop window (log line `raw input registered`) and hands each event to `Engine::forward_input`; the CEF engine coalesces moves and posts the legacy messages (`WM_MOUSEMOVE` 0x20, button messages 0x1/0x2/0x10, `WM_KEYDOWN`/`WM_KEYUP` with the legacy lParam) to `Chrome_WidgetWin_0`, else `_1`, under its own browser window; the wheel goes through the browser host.
- A renderer forwards only while the cursor is inside its own rectangle; Close forwards nothing, OnlyMouse only mouse and wheel, OnlyKeyboard only keys (after the legacy one-time click at 99999, 99999), and with InputDesktop on nothing is forwarded unless the foreground window is Progman, WorkerW or SHELLDLL_DefView (looked up at most once a second).
- `Assignment.input` carries the Engine section's InputType and InputDesktop (PROTOCOL_VERSION 9); mpv renderers and Closed register nothing.
- The `input` fixture theme (dot, clicked button, last key, wheel) is ready for the maintainer's check at 04-20; FEATURES row 73 is 🟨.

## Task Commits

1. **Task 1: Mouse movement reaches the CEF page (tracer)** - RED `0d1bd68d` (test), GREEN `d5b8f261` (feat)
2. **Task 2: Keyboard, wheel and the legacy filters** - RED `adb9e36c` (test), GREEN `353649b4` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## TDD Gate Compliance

Both tasks followed RED then GREEN with stub bodies (`unimplemented!`) so the target tests failed on the planned function (3 of 4 failing in Task 1 RED, 9 of 16 in Task 2 RED, each panicking in the stubbed function or failing the filter assertion), then passed after the implementation. `workflow.tdd_mode` is off, so the `gsd_run check tdd-red-evidence` classifier was not run (cargo test output is not one of its supported report formats).

## Files Created/Modified

- `crates/sucrose-platform/src/input.rs` - the pure input model, legacy mapping and filters
- `crates/sucrose-platform/src/os_windows/raw_input.rs` - RIDEV_INPUTSINK registration, WM_INPUT decoding, GetCursorPos
- `crates/sucrose-platform/src/os_windows/event_loop.rs` - WM_INPUT to `LoopEvent::Input`, `register_raw_input`
- `crates/sucrose-engine/src/engine.rs` - `takes_input`, `forward_input`
- `crates/sucrose-engine-cef/src/engine.rs`, `browser.rs` - input queue with move coalescing, widget lookup and `PostMessageW`, wheel through the host
- `crates/sucrose-ipc/src/protocol.rs`, `apps/sucrose/src/service.rs`, `apps/sucrose/src/render.rs` - `InputSettings`, the job and the renderer's filter, desktop focus cache, focus click
- `apps/sucrose/tests/fixtures/web-themes/input/` - the input fixture
- `docs/FEATURES.md` - the forwarding row

## Decisions Made

See `key-decisions` above. None needed a DECISIONS.md entry: they follow D-007 (legacy compatibility) and the plan's truths.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] browser.rs and Cargo manifests outside the plan's file lists**
- **Found during:** Task 1
- **Issue:** Posting to the Chromium widget and the wheel need the browser's window handle and `BrowserHost`, which only `CefHost` in `crates/sucrose-engine-cef/src/browser.rs` holds; `Engine::forward_input` names `InputEvent`, so `sucrose-engine` needs `sucrose-platform`; the raw input calls need two `windows` crate features. Also `unsupported.rs`, `os_linux.rs` and `os_macos.rs` needed the non-Windows `register_raw_input` and `cursor_position`.
- **Fix:** `CefHost::post_to_widget` and `send_mouse_wheel` (stubs on targets without CEF), the path dependency (no new crate; `Cargo.lock` gained only the edge), the features, and the non-Windows stubs.
- **Files modified:** crates/sucrose-engine-cef/src/browser.rs, crates/sucrose-engine/Cargo.toml, crates/sucrose-platform/Cargo.toml, crates/sucrose-platform/src/{unsupported,os_linux,os_macos}.rs, Cargo.lock
- **Committed in:** d5b8f261

**2. [Rule 2 - Missing critical] Keystrokes must never reach a log (T-04-45)**
- **Found during:** Task 1 and 2
- **Issue:** The CEF command queue logs commands with `{:?}` (debug and queue-full warnings), which would print a key.
- **Fix:** Input commands are handled before the debug log and a full queue drops them without content; `InputEvent`'s `Debug` shows only press or release for keys (tested).
- **Files modified:** crates/sucrose-engine-cef/src/engine.rs, crates/sucrose-platform/src/input.rs
- **Committed in:** d5b8f261, 353649b4

**3. [Rule 2 - Missing critical] A fast mouse must not fill the command queue**
- **Found during:** Task 1
- **Issue:** One command per raw mouse record (up to 1000 per second) would fill the 64-entry queue and drop clicks.
- **Fix:** Moves are coalesced in `State.pending_move` with one marker per burst (two tests).
- **Committed in:** d5b8f261

**4. [Rule 2 - Added variant] `InputEvent::FocusClick` and `key_message`**
- **Issue:** The contract lists four `InputEvent` variants; the OnlyKeyboard focus click and the key message mapping needed a place that does not require the engine to know the InputType.
- **Fix:** One extra variant made by the renderer, and a pure `key_message`. `mouse_events` also takes the raw wheel data as a parameter.
- **Committed in:** 353649b4

---

**Total deviations:** 4 auto-fixed (1 blocking, 3 missing critical)
**Impact on plan:** None on scope; all four were needed for correctness, privacy or to compile.

## Issues Encountered

- `cargo fmt` through the Windows wrapper turned CRLF working copies into LF again. Restored CRLF on: apps/sucrose/src/render.rs, crates/sucrose-engine-cef/src/engine.rs, crates/sucrose-platform/src/os_windows/event_loop.rs, crates/sucrose-platform/src/os_linux.rs, crates/sucrose-platform/src/os_macos.rs (render.rs again before the second task commit). `docs/FEATURES.md` stays CRLF with no newline after its last line. New files (input.rs, raw_input.rs, the fixture) are LF.
- No flaky test failed in `cargo xtask ci` (exit 0, fmt, clippy -D warnings, all tests).
- The tracer's `<verify>` has a `<human-check>`; the plan itself anchors it at the 04-20 maintainer checkpoint, so execution continued to Task 2 after the automated verify passed, and the human check stays open for 04-20.
- Not verified here: that the Chromium widget really sits as a direct child named `Chrome_WidgetWin_0` or `_1` of the CEF 156 browser window, and that CEF accepts the posted messages. That needs a real mouse (the project never synthesizes input); the widget lookup logs a debug line `input target found` and a warning once when it finds nothing, which the maintainer's run at 04-20 will show.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model (T-04-45 and T-04-46 are mitigated: forwarding follows InputType and InputDesktop, keys are never logged, targets are found only under the renderer's own CEF browser HWND). New surface: the renderer now receives all system-wide raw input while it forwards (inherent to the plan).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 04-20 (the maintainer's checkpoint with the input fixture on DISPLAY1) and 04-21.
- WAPI-07 stays open: URL themes (Round B) and Application themes (Round E) forward input later.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED

- Created files found: input.rs, raw_input.rs, fixture SucroseInfo.json and index.html.
- Commits found in HEAD: 0d1bd68d, d5b8f261, adb9e36c, 353649b4.
- Acceptance criteria: RIDEV_INPUTSINK count 4, `Input(` in lib.rs 1, SAFETY comments 8, `pub fn key_lparam` 1, 17 input tests passed, last byte of docs/FEATURES.md is 0x2e (not a newline).
