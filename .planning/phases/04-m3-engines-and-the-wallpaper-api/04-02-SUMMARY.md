---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 02
subsystem: engines
tags: [cef, web-wallpaper, external-message-pump, protocol-v4, theme-gate, delay-load]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: pinned CEF dist, delay-loaded libcef.dll, CEF subprocess routing, fixture Web library (04-01)
provides:
  - "check_selected(location, selected, app, playable) and ThemeRejection::TypeNotPlayable; the service plays Video and Web (PLAYABLE)"
  - "Protocol v4: Assignment.kind and Assignment.theme_dir"
  - "UseDecision::Restart: Use between Video and Web restarts the renderers on the other engine"
  - "sucrose_engine_cef::browser::{CefHost, CefStart, CefError}: windowed child browser on the surface thread, external message pump"
  - "Renderer CEF path in render.rs (engine started engine=Cef, page loaded, Playing after the first main-frame load)"
  - "D-036 and the cargo, FFI, FEATURES and ARCHITECTURE updates"
affects: [04-05, 04-06, 04-07, 04-08, 04-09, 04-12, 04-13, 04-20, phase-04-round-B]

actuals:
  tokens: 12100
  tasks: 2
  commits: 2
plan_head_before: 5b742f22821b403ea253eff44b74ee10da721d38
plan_head_after: b155688904b369266b305577904c5b92969172b4
commits: 2

tech-stack:
  added: []
  patterns:
    - "CEF pumped on the surface thread: OnScheduleMessagePumpWork records the soonest delay and wakes the loop; do_work runs do_message_loop_work and returns the next timer delay (1 ms .. 33 ms)"
    - "libcef.dll is probed with LoadLibraryW before the first CEF call so a missing file is ENGINE_MISSING, not a delay-load exception"
    - "The renderer branches on Assignment.kind after the surface is attached; the mpv path is unchanged"

key-files:
  created:
    - crates/sucrose-engine-cef/src/browser.rs
  modified:
    - crates/sucrose-core/src/theme.rs
    - crates/sucrose-core/tests/theme.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/tests/portal.rs
    - crates/sucrose-engine-cef/src/lib.rs
    - crates/sucrose-engine-cef/Cargo.toml
    - docs/DECISIONS.md
    - .claude/rules/cargo.md
    - .claude/rules/platform-ffi.md
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md

key-decisions:
  - "D-036: CEF is built against the pinned dist and delay-loaded; no sucrose-cef.exe fallback"
  - "A Use between two Web themes keeps the LoadSource path; the CEF renderer answers it by navigating its browser to the new page"
  - "CEF's browser child uses RuntimeStyle::ALLOY, no sandbox and an in-memory profile until plan 04-07"

patterns-established:
  - "Pure helpers (browser_size, next_pump_delay_ms, file_url) carry the unit tests for code whose real behavior needs CEF"

requirements-completed: []

coverage:
  - id: D1
    description: "A Web theme selected in a scratch library plays in a windowed CEF browser that is a child of the renderer's desktop surface, and the service process maps no libcef.dll"
    requirement: ENG-03
    verification:
      - kind: integration
        ref: "plan 04-02 Task 1 verify run: tasklist /m libcef.dll lists no service pid, lists the renderer and CEF subprocesses; Renderer log has engine started engine=Cef and page loaded status=200 for hello"
        status: pass
    human_judgment: true
    rationale: "That the page is visible inside the layered Progman surface on DISPLAY1 and DISPLAY9 needs eyes (A1/A15); the plan 04-20 checkpoint covers it"
  - id: D2
    description: "The gate accepts a Web theme and refuses an empty, missing, folder, outside or address Source; Gif, Url, YouTube and Application are refused as not playable yet"
    requirement: TYPE-03
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#the_gate_accepts_a_web_theme_and_returns_its_page, #a_web_theme_whose_source_is_empty_missing_a_folder_outside_or_an_address_is_refused, #other_wallpaper_types_are_refused"
        status: pass
    human_judgment: false
  - id: D3
    description: "Use between Video and Web restarts the renderers; between two themes of one engine it keeps LoadSource; Use of the playing theme changes nothing"
    requirement: TYPE-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#decide_use::decide_use_restarts_when_the_engine_changes, #keeps_the_load_source_path_between_themes_of_the_same_engine, #of_the_playing_web_theme_changes_nothing"
        status: pass
    human_judgment: false
  - id: D4
    description: "Protocol v4 carries the wallpaper kind and theme folder in the assignment, with the wire indexes unchanged"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc/src/protocol.rs#a_web_assignment_survives_the_wire, #variants_keep_their_wire_index"
        status: pass
    human_judgment: false
  - id: D5
    description: "D-036, the cargo and FFI rule exceptions, FEATURES and ARCHITECTURE record the D-32 outcome"
    verification:
      - kind: other
        ref: "grep -c '^## D-036' docs/DECISIONS.md and the two rule files; git ls-files --eol shows DECISIONS w/lf and the other four w/crlf; FEATURES keeps no final newline"
        status: pass
    human_judgment: false

duration: 21min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 02: The First Web Wallpaper in CEF Summary

**A Web theme plays in a windowed CEF browser embedded in the renderer's desktop surface, pumped on the surface thread through CEF's external message pump; the theme gate, protocol v4 and the Use decision know Web, and D-036 records the delay-load outcome**

## Performance

- **Duration:** 21 min
- **Started:** 2026-11-26T01:30:00Z
- **Completed:** 2026-11-26T01:51:00Z
- **Tasks:** 2
- **Files modified:** 14 (1 created)

## Accomplishments

- `check_selected` takes the playable types and returns `TypeNotPlayable`; the service's `PLAYABLE` is Video and Web, so a Web theme with a page inside its folder passes the gate and an address Source is refused with its reason.
- `PROTOCOL_VERSION` 4: `Assignment` carries `kind` and `theme_dir`; `UseDecision::Restart` closes and restarts the renderers when a Use switches between mpv and CEF.
- `CefHost` initialises CEF with the external pump and no multi-threaded loop, creates a windowed child browser (Alloy style, black background), cancels downloads and clears the context menu (legacy), reports each main-frame load end, moves the child with the surface and shuts down cleanly.
- The renderer starts CEF for Web assignments on the surface thread: Wake and the `CEF_PUMP_TIMER` call `do_work`, `SetRect` moves the surface then the browser, `Close` closes the browser before the surface is dropped, and Playing is sent after the first main-frame load.
- D-32 gate verdict recorded in D-036: with a Web theme playing, the service maps no `libcef.dll`; the renderer and its CEF subprocesses do.

## Task Commits

1. **Task 1: A Web theme plays in a windowed CEF browser on the desktop surface (tracer)** - `4e965ee3` (feat)
2. **Task 2: The D-32 outcome is written down** - `b1556889` (docs)

**Plan metadata:** the docs(04-02) commit that adds this file.

_Tracer: the plan's end-to-end verify (tests, build, scratch library, service run, tasklist checks, page-loaded log grep) passed after Task 1, so expansion into the docs task went ahead._

## Files Created/Modified

- `crates/sucrose-engine-cef/src/browser.rs` - CefHost, CefStart, CefError, the pump, the handlers
- `crates/sucrose-core/src/theme.rs`, `crates/sucrose-core/tests/theme.rs` - playable-type gate and Web cases
- `crates/sucrose-ipc/src/protocol.rs` - protocol v4, Assignment kind and theme_dir
- `apps/sucrose/src/service.rs` - PLAYABLE, Job kind and theme_dir, UseDecision::Restart, tests
- `apps/sucrose/src/render.rs` - CEF path, `file_url`, `move_surface`, engine started logs
- `apps/sucrose/tests/portal.rs` - the refusal test uses a Gif theme
- `docs/DECISIONS.md`, `.claude/rules/cargo.md`, `.claude/rules/platform-ffi.md`, `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - D-036 and its consequences

## Decisions Made

- D-036 (see DECISIONS.md): the `cef` crate stays, built against the pinned dist; `libcef.dll` is delay-loaded; CEF subprocesses are `sucrose.exe`; x86 is reported unsupported.
- A CEF renderer answers `LoadSource` by navigating to the new page (below).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] A CEF renderer must not ignore LoadSource**
- **Found during:** Task 1 (renderer message handling)
- **Issue:** the plan keeps the LoadSource path for Use between two themes of one engine, and says LoadSource is logged and ignored for a CEF renderer. A Use between two Web themes would then store the new selection and show nothing new.
- **Fix:** the CEF reader hands the new page to the surface thread, which calls `CefHost::navigate` with the `file://` URL of the new source; Pause, SetVolume, SetRole and Clock stay ignored until 04-05 and 04-09.
- **Files modified:** apps/sucrose/src/render.rs, crates/sucrose-engine-cef/src/browser.rs
- **Verification:** logic only (decision tests prove Web to Web takes Load); the navigation itself is for the 04-20 checkpoint.
- **Committed in:** 4e965ee3

**2. [Rule 3 - Blocking] sucrose-engine-cef needs the windows crate**
- **Found during:** Task 1 (moving the child window, probing libcef.dll)
- **Issue:** `SetWindowPos` and `LoadLibraryW` are not in the `cef` crate's public API.
- **Fix:** `windows` (already a workspace dependency, no new crate; features Win32_Foundation, Win32_System_LibraryLoader, Win32_UI_WindowsAndMessaging) added to the CEF target dependencies of the crate.
- **Files modified:** crates/sucrose-engine-cef/Cargo.toml, Cargo.lock
- **Committed in:** 4e965ee3

**3. [Rule 2 - Missing critical] libcef.dll is probed before the first CEF call**
- **Found during:** Task 1 (ENGINE_MISSING requirement, T-04-07)
- **Issue:** with a delay-loaded DLL a missing or unloadable file raises an exception in the first CEF call, which no Rust code can turn into the ENGINE_MISSING exit.
- **Fix:** `CefHost::start` loads `libcef.dll` by full path from the program's folder first and returns `CefError::Load` when that fails; the renderer maps it to ENGINE_MISSING with the plan's hint.
- **Files modified:** crates/sucrose-engine-cef/src/browser.rs, apps/sucrose/src/render.rs
- **Committed in:** 4e965ee3

---

**Total deviations:** 3 auto-fixed (1 bug, 1 missing critical, 1 blocking). **Impact:** none on scope; all three keep the plan's behavior honest.

## Issues Encountered

- **CEF's GPU process crashes at the start of a Web wallpaper on the maintainer's machine.** CefRenderer-0.log shows the GPU process exiting three times with code 0x80000003 right after the page loaded (and one `Failed to create shared context for virtualization`); Chromium then composites in software and the page keeps rendering (frame-latency messages continue). The cause is not found (candidates: the virtual display adapter, the layered desktop surface); no diagnostic flag was added because the maintainer wants no test-only app flags. It is recorded in D-036 and belongs to the plan 04-20 checkpoint (visual check on DISPLAY1 and DISPLAY9). No screenshot was taken: capture.ps1 captures a process's main window and the wallpaper surface has none.
- Only one renderer started in the verify run: the service saw one monitor (DISPLAY1), so `engine started engine="Cef"` appears once, which is one per monitor.
- My file-edit helper wrote LF over CRLF files; every touched file was converted back and checked with `git ls-files --eol` before each commit.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None that block the plan's goal. A CEF renderer logs and ignores Pause, SetVolume, SetRole and Clock until plans 04-05 and 04-09 wire them (renderer behavior chosen by the plan, not a data stub).

## Threat Flags

None beyond the plan's threat model: T-04-05 (Source verified inside the theme folder, file:// replaced in 04-06), T-04-06 (no sandbox, no host objects, recorded in D-036) and T-04-07 (every CEF start failure is a value mapped to ENGINE_MISSING or START_FAILED) hold; no crate was added (T-04-SC).

## Next Phase Readiness

- Plan 04-03 and the later plans can build on `CefHost`; 04-06 swaps `file_url` for the virtual origin, 04-05 and 04-09 wire pause, audio and role.
- ENG-03 stays partial and TYPE-03 stays open (04-20), so `requirements-completed` is empty.
- Open: the GPU process crash above.

## Self-Check: PASSED

Created files exist, commits 4e965ee3 and b1556889 are ancestors of HEAD, `commits: 2` is measured from the ledger, the acceptance greps (PROTOCOL_VERSION 4, no "only Video plays in this build", TypeNotPlayable x3, CEF_PUMP_TIMER x5, D-036 in DECISIONS and both rule files) and `cargo xtask ci` (1031 tests) pass.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
