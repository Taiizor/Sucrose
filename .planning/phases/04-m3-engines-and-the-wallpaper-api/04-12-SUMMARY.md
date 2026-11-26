---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 12
subsystem: engines
tags: [engine-detection, fallback, vc-redist, registry, portal-ipc, tray]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: choose_engine, Availability, Hint, EngineId (04-04/05), the CEF renderer (04-01), the Portal protocol (Phase 3)
provides:
  - "apps/sucrose/src/engines.rs: EngineReport, detect, pick, Fallbacks, FallbackNote (availability report from files and the registry, no engine loaded)"
  - "sucrose_engine::prereq: VC_REDIST_MIN, WEBVIEW2_FLOOR, version_at_least, version_newer_than"
  - "Platform::vc_redist(Arch) -> Option<VcRedist> (Windows registry, 64-bit view; None elsewhere)"
  - "PortalEvent::Engines, PortalRequest::DetectEngines, UseRefusal::NoEngine and the engine view types"
  - "log line 'engine fallback'; tray tooltip 'Playing with {fallback} ({engine} isn't available)'"
affects: [04-13, 04-15, 04-18, 04-20]

actuals:
  tokens: 22400
  tasks: 3
  commits: 4

tech-stack:
  added: []
  patterns:
    - "the service decides from files and registry values only (D-028); renderers alone load engines"
    - "a fallback is per run: Fleet holds the report and the fallbacks, nothing is written to Sucrose.json (D-11)"
    - "append-only Portal messages with wire-index tests (D-033)"

key-files:
  created:
    - apps/sucrose/src/engines.rs
    - crates/sucrose-engine/src/prereq.rs
    - crates/sucrose-platform/src/os_windows/prereq.rs
  modified:
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/main.rs
    - crates/sucrose-engine/src/lib.rs
    - crates/sucrose-engine-cef/src/lib.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
    - docs/FEATURES.md

key-decisions:
  - "The report is engine-wide; EngineReport::availability_for(kind, engine) adds the build rule that CEF plays Web themes only until its generated GIF and video pages exist, so a Video job configured for CEF still falls back to mpv as before"
  - "A type no engine can play is refused with UseRefusal::NoEngine (appended variant) and a sentence that carries the configured engine's repair hint, instead of the former 'Corrupt' mapping"
  - "Fallbacks and the report live in Fleet, so every place that builds a job (start, Use, library move) sees the same state and adopt_job records the fallback in one place"
  - "VC++ detection reports Some(installed: false) on Windows when the key is missing and None on other OSes, so only Windows applies the prerequisite"

patterns-established:
  - "Status::Starting and Status::Playing carry the fallback note; transitions take a Current { title, fallback } so a restart keeps it"

requirements-completed: []

coverage:
  - id: D1
    description: "The service builds an availability report without loading any engine (libmpv, CEF files and VC++ per architecture; other engines not in this build)"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/engines.rs#engines::tests (cargo test -p sucrose engines)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows/prereq.rs#the_registry_answers_for_this_machines_architecture_without_failing"
        status: pass
    human_judgment: false
  - id: D2
    description: "Legacy version thresholds compared component by component (VC++ 14.40.33816.0, WebView2 newer than 131.0.2903.70)"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "crates/sucrose-engine/src/prereq.rs#tests (cargo test -p sucrose-engine prereq)"
        status: pass
    human_judgment: false
  - id: D3
    description: "A fallback plays for this run only, is logged, shown in the tray tooltip and keeps the configured engine in Sucrose.json"
    requirement: ENG-07
    verification:
      - kind: e2e
        ref: "service run on a scratch folder with Web set to WebViewLive: log 'engine fallback' names WebView, renderer started Cef, setting still WebViewLive"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#the_tooltip_of_a_fallback_names_the_engine_that_plays_and_the_one_that_is_missing"
        status: pass
    human_judgment: true
    rationale: "The tooltip text in the real Windows notification area (127-character cut, wording) was not looked at by eye; the string and its truncation are unit-tested"
  - id: D4
    description: "The Portal receives the engine report after Welcome and on request through appended messages"
    requirement: ENG-06
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#the_engine_report_follows_the_welcome_and_comes_again_on_request"
        status: pass
      - kind: unit
        ref: "crates/sucrose-ipc/src/portal.rs#engine_states_and_hints_keep_their_wire_index"
        status: pass
    human_judgment: false
  - id: D5
    description: "FEATURES records the D-10 engine name mapping, the D-11 fallback order and the prerequisite detection"
    verification: []
    human_judgment: true
    rationale: "Documentation wording; reviewed by the maintainer with the phase"

duration: 27min
completed: 2026-11-26
status: complete
commits: 4
plan_head_before: 4c88854198f35b74652d12f1272395aa501d85ca
plan_head_after: 39f69654d84da48b117710e0ac91f64db24f2efc
---

# Phase 04 Plan 12: Engine detection, fallback and the Portal report Summary

**The service now detects libmpv, CEF and the VC++ Redistributable from files and the registry without loading anything, picks each job's engine from that report with a run-only fallback that is logged, shown in the tray and sent to the Portal.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-11-26T05:21:17Z
- **Completed:** 2026-11-26T05:48:00Z
- **Tasks:** 3
- **Files modified:** 14 (3 created)

## Accomplishments

- `engines::detect` reports mpv (next to the exe or an absolute PATH entry), CEF (its five files and `locales/` next to the exe plus the VC++ rule) and every other engine as not in this build; CEF on x86 Windows reports no build for the architecture.
- `Platform::vc_redist` reads `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\{x86,x64,ARM64}` through the 64-bit view; the legacy thresholds live in `sucrose_engine::prereq` and are shared with Round B (WebView2).
- `job_for` chooses with `engines::pick`; a fallback is logged as `engine fallback`, recorded in `Fleet.fallbacks`, shown as "Playing with CEF (WebView2 isn't available)" and never written to the settings.
- The Portal gets `PortalEvent::Engines` right after Welcome, on `DetectEngines`, and whenever the view changes; a type with no engine refuses Use with the new `UseRefusal::NoEngine`.
- docs/FEATURES.md records the engine name mapping, the fallback order and the detection rules.

## Task Commits

1. **Task 1: WebViewLive plays on CEF and the tray says so (tracer)** - `44e58b14` (feat)
2. **Task 2: VC++ detection with the legacy threshold (TDD)** - RED `b2700096` (test), GREEN `5cbc0e34` (feat)
3. **Task 3: Portal report and FEATURES** - `39f69654` (feat)

**Plan metadata:** the docs commit that carries this file.

## Tracer verification (Task 1)

Scratch folder `target/web-lib` with `"Web": "WebViewLive"`, service started with `--no-portal`, 15 s, then `taskkill`:

- `engine fallback type=Web configured=WebViewLive playing=CefSharpLive reason=this build of Sucrose does not play this wallpaper with WebView2`
- renderer log: `engine started engine="Cef"`
- `"Web": "WebViewLive"` still in `Setting/Sucrose.json` (count 1).

Detected on the maintainer's machine (service start log): `Visual C++ Redistributable installed=true version="v14.52.37025.00" arch=X64` and `engines detected mpv=Installed { version: None } cef=Installed { version: Some("156.0.31") }`.

## TDD Gate Compliance

Task 2: RED commit `b2700096`, GREEN commit `5cbc0e34`, no refactor commit. RED evidence is the `cargo test -p sucrose-engine prereq` run against stubs that return false: four target tests failed on their planned assertions (`at_least_accepts_the_threshold_itself_and_anything_newer`, `missing_components_count_as_zero`, `newer_than_excludes_the_floor_itself`, `the_registry_spelling_with_a_leading_v_and_padded_zeros_counts`) and three passed (the cases the stub satisfies by returning false); the crate compiled, so no build or fixture failure. `cargo test` output is not a format `check tdd-red-evidence` reads, so the semantic assessment is recorded here instead of a classifier verdict (Rust known gap in tdd.md).

## Decisions Made

See `key-decisions` above. The one behavior change to know about: with libmpv missing, a Video theme is now refused at the service ("No installed engine can play this wallpaper. The files of mpv are missing. Reinstall or repair Sucrose.") instead of starting a renderer that reports the D-20 hint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Portal app match on PortalEvent**
- **Found during:** Task 3
- **Issue:** `apps/sucrose-portal/src/app.rs` matches `PortalEvent` exhaustively, so the appended `Engines` variant broke the Portal build; the file is not in the plan's list.
- **Fix:** an arm that logs the counts at debug level; plans 13 and 15 replace it with the UI.
- **Files modified:** apps/sucrose-portal/src/app.rs
- **Commit:** 39f69654

**2. [Rule 1 - Bug] Existing Portal integration tests read the next event after Welcome**
- **Found during:** Task 3 (`cargo xtask ci`, 7 tests failed)
- **Issue:** the Engines event now follows Welcome, so tests that expected their own event next saw it.
- **Fix:** `PortalClient::next_event` skips `Engines`; `next_event_any` returns every event and the new test uses it.
- **Files modified:** apps/sucrose/tests/common/mod.rs, apps/sucrose/tests/portal.rs
- **Commit:** 39f69654

**3. [Rule 2 - Missing critical] UseRefusal::NoEngine**
- **Found during:** Task 1
- **Issue:** the refusal of a type with no engine mapped to Corrupt ("damaged theme"), which is wrong and cannot show UI-SPEC reason 4.
- **Fix:** appended `UseRefusal::NoEngine` (wire-index test updated); `refusal_of` maps the no-engine sentence to it.
- **Files modified:** crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/service.rs
- **Commit:** 44e58b14

**4. [Rule 2 - Convention] cfg kept out of apps/sucrose**
- **Found during:** Task 1
- **Issue:** distinguishing "no build for this architecture" from "not included yet" needed `cfg!(windows)` in the app crate, which `.claude/rules/rust.md` forbids.
- **Fix:** `sucrose_engine_cef::NO_BUILD_FOR_ARCHITECTURE`.
- **Files modified:** crates/sucrose-engine-cef/src/lib.rs
- **Commit:** 44e58b14

### Plan notes

- `crates/sucrose-platform/src/unsupported.rs` is untouched: `Platform::vc_redist` has a default method that returns `None`, which is what the other OSes need.
- `cargo fmt --all` rewrote the working copies of the Rust files it touched to LF. The index already holds LF and `git diff` shows only the intended changes; `docs/FEATURES.md` is CRLF in the working copy with no final newline (checked).

---

**Total deviations:** 4 auto-fixed (1 bug, 2 missing critical, 1 blocking)
**Impact on plan:** all needed for a compiling, correct result; no scope creep.

## Issues Encountered

None beyond the deviations above. The tooltip was not looked at in the real notification area (D3 carries `human_judgment: true`).

## Known Stubs

None. WebView2, libVLC, the built-in players and the embedder report "not in this build" by design until their rounds; the VC++ check is real.

## Threat Flags

None. Detection only reads files and one registry key family and never loads a library (T-04-29); a fallback is chosen once per job and a type with no engine is refused instead of restarting renderers (T-04-30); no crate was added (T-04-SC).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

04-13 and 04-15 can show `PortalEvent::Engines` (states, hints, installable flag, fallbacks); 04-18 implements the install behind `installable`; Round B adds the WebView2 check with `WEBVIEW2_FLOOR`.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
## Self-Check: PASSED
