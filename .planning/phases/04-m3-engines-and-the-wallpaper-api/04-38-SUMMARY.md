---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 38
subsystem: measurement
tags: [webview2, builtin-gif, mpv, measure, xtask, memory, cpu, maintainer-checkpoint, round-b]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-37: the release build, the scratch Round B run folder (target/round-b) and the WebView2 engine, the built-in GIF renderer"
provides:
  - "cargo xtask measure counts a WebView2 wallpaper's whole process tree on both sides (Rust role webview2; C# roles webview and webview-subprocess)"
  - "The recorded M3 WebView2 row (Rust only, the pairing was refused) and the M3 built-in GIF renderer row, with reading paragraphs, in docs/MEASUREMENTS.md"
affects: [verify-work, round-d, gap-closure]
plan_head_before: 82420ad45e499a8b6107d1a8a3f13173a5c242dc
plan_head_after: f51b4285453e8a7beb8af2807ea7eea2c1d7de33
actuals:
  tokens: 5700
  tasks: 3
  commits: 2
tech-stack:
  added: []
  patterns:
    - "A WebView2 process belongs to Sucrose by the --webview-exe-name marker (or the Sucrose WebView2 user data folder when the marker is absent); another host name in the marker is decisive"
key-files:
  created: []
  modified:
    - xtask/src/measure.rs
    - xtask/src/main.rs
    - docs/MEASUREMENTS.md
key-decisions:
  - "The M3 WebView2 row holds the Rust numbers alone because --row refused the pair (the C# side lists the NVIDIA adapter on the host process and Microsoft Basic Render Driver on its GPU-using WebView2 process; the Rust side only Basic Render Driver, G-02-6); the C# numbers are in the paragraph, as 04-21 did; measure.rs was not changed to make the pair pass"
  - "Outcome rule applied: WebView2 Rust RAM lower and CPU equal, nothing flagged; the built-in GIF renderer's CPU and GPU time are above both mpv readings, recorded as found and flagged for /gsd-verify-work, nothing tuned"
requirements-completed: []
coverage:
  - id: D1
    description: "cargo xtask measure counts the WebView2 process trees (Rust webview2 role, C# webview and webview-subprocess roles) without counting other applications' WebView2 processes, and prints per-role totals"
    requirement: ENG-02
    verification:
      - kind: unit
        ref: "cargo test -p xtask measure; cargo xtask ci (1726 tests passed, 0 failed)"
        status: pass
      - kind: integration
        ref: "target/measure/webview2-smoke.json and the 300 s Rust WebView2 run list the renderer and 7 webview2 processes"
        status: pass
    human_judgment: false
  - id: D2
    description: "M3 WebView2 and M3 built-in GIF renderer rows and reading paragraphs recorded in docs/MEASUREMENTS.md from release-build runs against the C# engines (and Rust mpv, D-10)"
    requirement: TYPE-02
    verification:
      - kind: other
        ref: "grep -c 'M3 Web wallpaper on WebView2' docs/MEASUREMENTS.md = 2; grep -c 'M3 GIF wallpaper on the built-in renderer' docs/MEASUREMENTS.md = 2"
        status: pass
    human_judgment: true
    rationale: "The built-in renderer's higher CPU and GPU time and the Basic Render Driver attribution of WebView2 GPU work are findings a human must judge in /gsd-verify-work"
duration: ~1h 15m wall-clock (continuation only; the maintainer's runs dominate)
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 38: WebView2 and built-in GIF renderer measurements Summary

**cargo xtask measure sees a WebView2 wallpaper's whole process tree on both sides, and release-build rows put Rust WebView2 (931 MB, 17.5 % CPU) against C# WebView (1168 MB, 17.9 %) and the built-in GIF renderer (149 MB, 2.6 % CPU) against Rust mpv (221 MB, 0.84 %) and C# mpv (478 MB, 1.21 %).**

## Performance

- **Duration:** about 1h 15m wall-clock for this continuation (Task 1 commit 02:42 local, final commit 03:35 local; the five samples ran 02:47-03:27)
- **Tasks:** 3 (Task 1 code, Task 2 ready gate, Task 3 five samples and the rows)
- **Files modified:** 3 (xtask/src/measure.rs, xtask/src/main.rs, docs/MEASUREMENTS.md)

## Accomplishments

- measure.rs: Rust role `webview2` (msedgewebview2.exe whose `--webview-exe-name` names sucrose.exe, or, without the marker, whose user data folder is a Sucrose Web/WebView2 or WebEphemeral/WebView2 folder; another host in the marker is decisive, so other applications' WebView2 is never counted); C# roles `webview` (Sucrose.Live.WebView) and `webview-subprocess` (its msedgewebview2.exe); per-role totals in the row; the `--row` adapter check judges a WebView2 run on its GPU-using processes. The real command-line marker was `--webview-exe-name=sucrose.exe`; the 300 s run attributed 7 msedgewebview2 processes to the renderer's tree and the machine's other WebView2 applications were not counted.
- Five 300 s samples (60 s warm-up) on a release build of 82420ad4 (xtask at 65e5e203), one display, one app at a time, run by the orchestrator.
- Two rows and two reading paragraphs in docs/MEASUREMENTS.md.

### Rows

WebView2 (Rust only; `--row` refused the pair):

> service + renderer + 7 WebView2 processes: RAM 931 MB (peak 956), private 516 MB (peak 534), CPU 17.50% (peak 27.0%), GPU 3.6% (one WebView2 process, attributed to Microsoft Basic Render Driver), GPU memory 0 MB dedicated / 6 MB shared; the WebView2 processes alone 881 MB RAM, 501 MB private, 17.44% CPU; the service alone 23.9 MB RAM, 6.9 MB private; the renderer alone 26.8 MB RAM, 8.2 MB private

C# WebView, same theme (in the paragraph): 3 non-WebView processes plus 7 WebView2 processes: RAM 1168 MB (peak 1182), private 679 MB (peak 691), CPU 17.85% (peak 26.1%), GPU 3.7%; WebView + Backgroundog alone 1060 MB RAM, 641 MB private, 17.84% CPU. Refusal text: "the two sides ran on different GPUs (C#: Microsoft Basic Render Driver / NVIDIA GeForce RTX 3050 Laptop GPU, Rust: Microsoft Basic Render Driver); the comparison is not valid".

GIF (built-in against C# mpv, accepted, NVIDIA GeForce RTX 3050 Laptop GPU on both sides):

> C#: 3 processes: RAM 478 MB (peak 481), private 460 MB, CPU 1.21% (peak 2.7%), GPU 14.5%, GPU memory 108 MB dedicated / 82 MB shared; MpvPlayer + Backgroundog: RAM 388 MB (peak 390), private 428 MB, CPU 1.20% (peak 2.7%) | Rust built-in: service + 1 renderer: RAM 149 MB (peak 157), private 165 MB, CPU 2.60% (peak 5.1%), GPU 21.3%, GPU memory 24 MB dedicated / 46 MB shared

Rust mpv on the same GIF (paragraph, D-10): service + 1 renderer: RAM 221 MB (peak 226), private 295 MB (peak 300), CPU 0.84% (peak 3.9%), GPU 14.3%, GPU memory 83 MB dedicated / 74 MB shared.

### Findings (flag for /gsd-verify-work, nothing tuned)

- The built-in GIF renderer uses less RAM than Rust mpv (0.68 x working set, 0.56 x private) and less video memory (24 against 83 MB), but more CPU (2.60 % against 0.84 %, about 3.1 x) and more GPU time (21.3 % against 14.3 %). Against C# mpv: CPU about 2.1 x, GPU 21.3 % against 14.5 %.
- Rust mpv is below C# mpv in RAM (0.46 x working set) and CPU (0.84 % against 1.21 %), equal in GPU time.
- WebView2: Rust RAM is lower (0.80 x working set, 0.76 x private against the C# total), CPU equal. The two WebView2 process trees are about the same size (Rust 881 MB, C# 824 MB working set); the saving is the host processes (51 MB against 344 MB working set).
- The WebView2 GPU-using process is attributed to Microsoft Basic Render Driver on both sides (possibly a software or basic adapter; not Rust-specific). Not investigated here.

## T0 of every run (local, +03:00)

| Run | Scenario | T0 | Result file |
| --- | --- | --- | --- |
| 1 | Rust WebView2 | 02:49:12 | target/measure/webview2-rust.json |
| 2 | C# WebView | 02:59:19 | target/measure/webview-csharp.json |
| 3 | Rust built-in GIF | 03:06:51 | target/measure/gif-builtin-rust.json |
| 4 | Rust mpv GIF | 03:13:29 | target/measure/gif-mpv-rust.json |
| 5 | C# mpv GIF | 03:21:23 | target/measure/gif-mpv-csharp.json |

Machine state: DISPLAY9 off (one display, \\.\DISPLAY1 1920x1080), Windows 10.0.26300.9974, scratch data folder target/round-b; the C# app was closed during every Rust run and no Rust Sucrose process ran during the C# runs. Settings (Engine.Web, Engine.Gif, Library.Selected) and the scratch SucroseProperties.json were restored and cmp-checked after the runs; no property value appears in any repository file.

## Maintainer replies (verbatim, in order)

"start A" (A = the full plan as written: 60 s warm-up, 300 s sample each, no cut), "C# hazır" (after switching his C# Web engine to WebView and selecting Farm Weather Cycles), "C# kapandı" (after restoring his C# Web engine and exiting the C# app from the tray), "C# hazır" (after selecting the Asus ROG GIF theme in C#, his C# Gif engine is mpv), "tamam" (after selecting his own theme again; at the orchestrator's request he then exited the C# app again and turned DISPLAY9 back on for plan 04-39).

## Task Commits

1. **Task 1: WebView2 process-tree roles in the measure tool** - `65e5e203` (feat)
2. **Task 2: Ready gate** - checkpoint, no commit (the maintainer replied "start A")
3. **Task 3: Five samples and the rows** - `f51b4285` (docs)

**Plan metadata:** the SUMMARY/STATE/ROADMAP commit follows this file.

## Files Created/Modified

- `xtask/src/measure.rs` - webview2, webview and webview-subprocess roles, row totals, adapter check for WebView2 runs, tests
- `xtask/src/main.rs` - USAGE and command list name the new roles
- `docs/MEASUREMENTS.md` - the two rows and two reading paragraphs (CRLF kept; w/crlf checked before and after)

## Decisions Made

- The WebView2 pair is recorded Rust-only, C# in the paragraph, because `--row` refused it; the tool was not changed to pass it.
- No finding was tuned; the outcome rule says record and flag.

## Deviations from Plan

### Process deviations

**1. [Process - TDD] Tests and implementation written together in Task 1**
- **Found during:** Task 1
- **Issue:** The plan asked for tests first; the tests and the implementation were written before the first test run, so there is no separate RED run or RED commit.
- **Impact:** one `feat` commit holds both; the tests pass (`cargo test -p xtask measure`, and `cargo xtask ci`: 1726 passed, 0 failed, no flaky test needed a rerun).

**2. [Process - Orchestration] The five samples were run by the orchestrator, not a continuation executor**
- **Found during:** Task 3
- **Issue:** The plan says the continuation that receives "start" runs the samples; per the timed-run memory rule the orchestrator started the service and the sampler itself, and this continuation only produced the rows.
- **Impact:** none on the numbers; the continuation started no application.

**3. [Observation] The Rust service kept running about 2 minutes after Run 1's sample ended**
- The orchestrator's wait loop delayed the taskkill; the idle time is outside every sample.

### Auto-fixed Issues

None.

**Total deviations:** 3 process notes. **Impact on plan:** none on the recorded numbers.

## Issues Encountered

- `--row` refused the WebView2 pair (different GPU adapter sets); recorded as 04-21 did.
- `.planning`/CRLF: docs/MEASUREMENTS.md was w/crlf and stayed w/crlf; measure.rs and main.rs stayed w/lf; no file needed a CRLF restore.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Plan 04-39 runs next with DISPLAY9 attached; the C# app is closed. Findings above (built-in GIF CPU/GPU, WebView2 Basic Render Driver attribution) are for /gsd-verify-work.

## Self-Check: PASSED

- docs/MEASUREMENTS.md, xtask/src/measure.rs, xtask/src/main.rs and the five result files exist; commits 65e5e203 and f51b4285 are on HEAD's history; `cargo xtask ci` exit 0.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
