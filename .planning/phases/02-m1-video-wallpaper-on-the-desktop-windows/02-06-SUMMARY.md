---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 06
subsystem: desktop-surface
tags: [windows, progman, workerw, win32, dpi, per-monitor-v2, ipc, fixtures]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 03/04 theme info and legacy settings; plan 05 service and renderer channel (Hello, Close, per-user endpoint)"
provides:
  - "ToRenderer::Assign(Assignment { slot, monitor, rect, source }) sent by the service after a valid Hello"
  - "Service gate for Video themes with a safe relative Source (T-02-13), primary monitor pick, renderer spawn"
  - "sucrose_platform::{DesktopSurface::attach, native_handle, EventLoop, LoopProxy::quit, init_dpi_awareness, Platform::monitors}"
  - "Per-Monitor-V2 awareness in service and renderer, verified and logged"
  - "Pure layout::classify over a ShellSnapshot, fixture-tested against the real 26300 tree and a classic tree"
  - "RectPx::{width, height, to_parent, covers} in geometry.rs (physical pixels only)"
affects: [02-07, 02-08, 02-09, 02-11]

actuals:
  tokens: 16500
  tasks: 2
  commits: 3

plan_head_before: a26e92f1df8b50739344a476e69493160f43b2f9
plan_head_after: ba1cc2044052172af8f048f900a9ae5e12e51b9e

tech-stack:
  added: []
  patterns:
    - "Shell tree captured as plain data (ShellSnapshot) and classified by a pure function; never from the OS build number"
    - "Surface created and pumped on the renderer's main thread; engine embeds with wid in plan 07"
    - "Attach verified after SetParent (root ancestor, visibility, rect) with 10 s backoff retries"

key-files:
  created:
    - crates/sucrose-platform/src/geometry.rs
    - crates/sucrose-platform/src/layout.rs
    - crates/sucrose-platform/src/os_windows/surface.rs
    - crates/sucrose-platform/src/os_windows/display.rs
    - crates/sucrose-platform/src/os_windows/event_loop.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/tests/layout.rs
    - crates/sucrose-platform/tests/fixtures/desktop-26300.json
    - crates/sucrose-platform/tests/fixtures/desktop-classic.json
  modified:
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/main.rs

key-decisions:
  - "Strategy B (layered child of Progman, ordered right under SHELLDLL_DefView, Raised order) is composited on Windows 10.0.26300.9974; no WorkerWChild swap was needed"
  - "Classification lives in layout.rs as a pure function over a recorded tree so macOS and Linux CI run the same tests"
  - "Renderer exit code 4 means the attach failed"

patterns-established:
  - "Fixture-driven shell classification: add a JSON rendering of any new shell tree and a test row"
  - "RectPx::to_parent is the single place a screen rect becomes parent-client coordinates"

requirements-completed: [DESK-01, DESK-06, TYPE-01]

duration: "5 min (this continuation; Task 1 ran in the earlier session)"
completed: 2026-11-22
status: complete

coverage:
  - id: D1
    description: "Renderer surface sits behind the desktop icons on the primary monitor, black, icons visible and clickable above it, no taskbar or Alt-Tab entry"
    requirement: DESK-01
    verification:
      - kind: manual_procedural
        ref: "Task 1 human check on the maintainer's desktop (approved); renderer log 'surface attached layout=Raised strategy=LayeredChild actual=0,0,1920,1080'"
        status: pass
    human_judgment: true
    rationale: "Whether DWM composites the surface behind the icons can only be judged visually on the real shell (RESEARCH A1/A2)"
  - id: D2
    description: "Layout classification (Raised, Classic, Unsupported) and physical-pixel rect mapping are pure and tested against recorded trees"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/tests/layout.rs (9 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Surface covers the whole monitor at a non-100 % display scale with no gap or overhang"
    requirement: DESK-06
    verification: []
    human_judgment: true
    rationale: "The 150 % scale check needs the maintainer to change Windows display scale; recorded as pending for the phase verifier. PMv2 awareness itself is asserted in the renderer log (PerMonitorV2)."
  - id: D4
    description: "Service starts a renderer only for a Video theme with a safe relative Source that exists in the theme folder, then answers Hello with Assign"
    requirement: TYPE-01
    verification:
      - kind: integration
        ref: "automated run: service log 'Hello answered with an Assign slot=0 monitor=DISPLAY1 rect=[0,0,1920,1080]'"
        status: pass
    human_judgment: true
    rationale: "The gate has no dedicated unit tests in this plan; plan 08 adds canonicalized containment and its tests"
---

# Phase 2 Plan 06: Video wallpaper surface on the desktop (Windows) Summary

**A Per-Monitor-V2 renderer attaches a black surface behind the desktop icons of the primary monitor on Windows 11 26300 (layered child of Progman under SHELLDLL_DefView), started by the Rust service for the legacy selection via a new IPC Assign message, with the shell-layout decision isolated in a pure, fixture-tested classifier.**

## Performance

- **Duration:** about 5 min in this continuation (Task 1 in the earlier session)
- **Completed:** 2026-11-22T14:29:25Z
- **Tasks:** 2
- **Files modified:** 20 (since the plan base)

## Accomplishments

- Task 1 (tracer): service gate, Assign, PMv2 renderer and desktop surface, confirmed by the maintainer on the real desktop. Strategy B (LayeredChild, Raised order) worked as written; strategy A (WorkerWChild) remains the fallback and was never needed.
- Task 2: `layout::classify` over `ShellSnapshot`/`WindowNode`, `geometry.rs` with `RectPx::to_parent` and `covers`, two recorded-tree fixtures and 9 tests that run on every OS. `surface.rs` now only captures the live tree and calls `classify`; it contains no OS-build-number comparison.
- Regression run after the refactor: renderer log still reports PerMonitorV2 and `surface attached layout=Raised strategy=LayeredChild actual=(0,0,1920,1080)` equal to the target; the service log shows the Hello answered with Assign for slot 0.
- `cargo xtask ci` (fmt, clippy -D warnings, test) exits 0.

## Task Commits

1. **Task 1: Surface behind the desktop icons** - `0f72874c` (feat)
2. **Task 2 RED: failing fixture tests** - `be0d0668` (test)
3. **Task 2 GREEN: pure classifier and rect math** - `ba1cc204` (feat)

**Plan metadata:** added by the docs commit that carries this file.

## Files Created/Modified

- `crates/sucrose-platform/src/geometry.rs` - RectPx and MonitorInfo, rect mapping into a parent's client space
- `crates/sucrose-platform/src/layout.rs` - ShellSnapshot, WindowNode, DesktopLayout, classify
- `crates/sucrose-platform/src/os_windows/surface.rs` - DesktopSurface: capture, classify, 0x052C once, strategy B/A or classic, verify, place
- `crates/sucrose-platform/src/os_windows/{display,event_loop}.rs` - monitors, DPI awareness, message loop with quit proxy
- `crates/sucrose-platform/tests/layout.rs` and `tests/fixtures/desktop-{26300,classic}.json` - fixture tests
- `crates/sucrose-ipc/src/protocol.rs`, `apps/sucrose/src/{service,render,main}.rs` - Assign, service gate, renderer wiring

## TDD Gate Compliance (Task 2)

- **RED** (`be0d0668`): classify, to_parent and covers were stubs (Unsupported / unchanged / false). `cargo test -p sucrose-platform --test layout`: 5 failed, 4 passed. The failures are on the planned assertions: `the_26300_probe_is_raised` (left Unsupported, right Raised), `the_classic_tree_is_classic`, `the_no_redirection_bitmap_flag_alone_decides_raised`, `to_parent_subtracts_a_negative_parent_origin` (left 100..2020, right 2020..3940), `covers_means_completely_inside`. The 4 passing tests assert behavior a stub satisfies by construction (Unsupported without Progman or icons, zero-origin identity, width kept at 200 %).
- **RED evidence:** libtest's console output is not a report format `gsd_run check tdd-red-evidence` supports, so no machine verdict exists. Semantic assessment: the target tests executed (9 discovered, none skipped), the build succeeded, and each failing test failed on the assertion for the planned behavior, not on a load, parse or fixture fault.
- **GREEN** (`ba1cc204`): all 9 layout tests and the 2 unit tests pass; clippy and xtask ci green.
- **REFACTOR:** none needed.

## Decisions Made

- **Working attach strategy on Windows 10.0.26300.9974:** B, a layered child of Progman placed right under SHELLDLL_DefView (Raised order: LayeredChild first, WorkerWChild second). Plan 07 re-checks it with video (RESEARCH Open Question 1).
- **Input for plan 09 (desktop refresh):** after `taskkill /IM sucrose.exe /F` the black area lingered; the normal wallpaper did not repaint by itself. A forced kill leaves the stale frame because the surface is never destroyed through its Drop. Plan 09 should refresh the desktop (for example SystemParametersInfo SPI_SETDESKWALLPAPER re-apply, or invalidating Progman and the WorkerW) after the renderer ends or is restarted.

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written, with two minor notes:

- `surface.rs` builds the snapshot by walking direct children with `GetWindow(GW_CHILD/GW_HWNDNEXT)` one level deep instead of `EnumChildWindows`; classification only needs direct children and the order (topmost first) is preserved.
- The 26300 probe in the plan lists no Progman window style, so the fixture uses 0x96000000 (popup, visible, clip siblings and children) as a plausible value; classification does not read it.

**Total deviations:** 0 auto-fixed. **Impact:** none.

## Issues Encountered

- Stale black area after a forced kill of sucrose.exe (see Decisions Made); not fixed here by design, tracked for plan 09.

## Pending Human Check

- Task 2 human check (150 % display scale): not run because it needs the maintainer to change Windows display scale. For the phase verifier: set 150 % (or 125 %), start `target/debug/sucrose.exe` with a scratch `SUCROSE_DATA_DIR`, confirm the surface fills the monitor without gap or overhang and the renderer log shows the same physical rect as at 100 %, then restore the scale and stop the app. Recorded in `.planning/WINDOWS.md` as unrun-verify.

## Known Stubs

None. The surface is intentionally an empty black window until plan 07 plays video on it.

## Threat Flags

None beyond the plan's register. T-02-13 (Source gate: relative, no `..`, no `://`, no drive or UNC prefix, must exist as a file) is enforced in the service; T-02-34 (SendMessageTimeoutW 1000 ms, 0x052C sent once only when no wallpaper WorkerW exists, retries stop after 10 s) is enforced in `surface.rs`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07 can embed libmpv into `DesktopSurface::native_handle()` on the renderer's message-pumping main thread and re-check strategy B with video.
- Plan 08 adds canonicalized containment for the Source gate; plan 09 adds the desktop refresh noted above.

## Self-Check: PASSED

All listed files exist and commits 0f72874c, be0d0668 and ba1cc204 are ancestors of HEAD.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
