---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 24
subsystem: engines
tags: [webview2, cef, page-calls, executescript, shared-module, profile-concurrency]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "sucrose-engine-webview2 host and engine (04-22), the WebView2 range reply and traversal tests (04-23), CEF page calls, data hub and in-page pause (04-09, 04-10, 04-14)"
provides:
  - "sucrose_engine_web::page::{PageState, Complaints, describe, page_calls, wanted_groups, load_templates, push_calls}: the page logic both web engines call"
  - "WebView2 runs the property calls, the non-data compatible calls (legacy run order), DataWanted, data pushes and the in-page pause through ExecuteScript"
  - "concurrent_webview2_instances_never_share_the_persistent_profile and a two-thread serve::plan test"
affects: [04-25, 04-37]

actuals:
  tokens: 23600
  tasks: 2
  commits: 2
plan_head_before: 27e3678b77d93ce2a0bb40763e98df873d7ccc3c
plan_head_after: 568c8c1acd0b168a20dfe3d2e3b659757382e435

tech-stack:
  added: []
  patterns:
    - "engine-neutral page state built from plain values (no IPC type), each engine builds it from its Assignment"
    - "an engine hook returns the page calls for a finished navigation and the host runs them before the loaded hook"

key-files:
  created:
    - crates/sucrose-engine-web/src/page.rs
    - crates/sucrose-engine-web/tests/page.rs
  modified:
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/tests/profile.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - docs/FEATURES.md

key-decisions:
  - "PageState owns the JSON parsing of data snapshots (store_data), so neither engine needs serde_json for it and WebView2 gained no dependency"
  - "CEF keeps its own State only for the coalesced mouse move; everything the page calls read lives in PageState"
  - "WebView2 runs page calls inside the NavigationCompleted handler before the loaded hook, so the first-frame signal and DataWanted come after the page and its calls, as on CEF"

patterns-established:
  - "page-call tests live once in sucrose-engine-web/tests/page.rs; engine crates keep only queue and state tests"

requirements-completed: []

coverage:
  - id: D1
    description: "WebView2 gets the property calls in file order, then the non-data compatible calls in LEGACY_RUN_ORDER, after the main document finished with status 200 (WAPI-01, WAPI-02, FMT-07 on WebView2)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/page.rs#the_calls_fixture_gets_its_property_calls_in_file_order_then_the_non_data_compatible_calls_in_the_legacy_order"
        status: pass
      - kind: e2e
        ref: "calls fixture on WebView2 (SUCROSE_LOG info,sucrose_engine_webview2=debug): 10 __calls/property and 2107 __calls/compatible origin reports in 20 s"
        status: pass
    human_judgment: true
    rationale: "The maintainer's live check of WAPI-01, WAPI-02 and FMT-07 on both engines is 04-37"
  - id: D2
    description: "A WebView2 page with data groups sends DataWanted once after its first successful load and receives the pushes (WAPI-03 Round A groups, WAPI-04 on WebView2)"
    verification:
      - kind: e2e
        ref: "calls fixture on WebView2: 'data wanted groups=[Bios, Date, Memory, Processor, Motherboard]' once, 'data hub started' in the Service log, 191 __calls/compatible/SystemDate reports in 20 s"
        status: pass
    human_judgment: true
    rationale: "WAPI-03 completes in Round D and the live check of WAPI-04 is 04-37"
  - id: D3
    description: "Pause and resume run the in-page pause script, a page that loads while paused is paused again after its calls, Loop and Shuffle changes re-run the compatible calls (PLAY-05, PLAY-01 parts)"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/page.rs#a_page_that_loads_while_paused_is_paused_after_its_calls"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/page.rs#a_new_shuffle_value_reaches_the_next_compatible_push"
        status: pass
    human_judgment: true
    rationale: "The pause drain on the WebView2 surface thread was not observed live; PLAY-01 and PLAY-05 complete in Round E"
  - id: D4
    description: "The page-call logic exists once and CEF calls it with no behavior change"
    verification:
      - kind: e2e
        ref: "calls fixture on CEF (info,sucrose_engine_cef=debug,sucrose_engine_web=debug): 10 __calls/property, 1568 __calls/compatible, 10 property page call lines"
        status: pass
      - kind: unit
        ref: "cargo test -p sucrose-engine-cef (30 passed) and cargo xtask ci (exit 0)"
        status: pass
    human_judgment: false
  - id: D5
    description: "ENG-02 concurrency: two WebView2 instances never share the persistent profile; a killed holder's lock is released; the planner is stateless"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/profile.rs#concurrent_webview2_instances_never_share_the_persistent_profile"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/profile.rs#concurrent_instances_planning_origin_replies_get_their_own_answers"
        status: pass
    human_judgment: false
  - id: D6
    description: "The storage fixture on WebView2 keeps its localStorage counter across a renderer restart on the persistent profile (D-35)"
    verification:
      - kind: e2e
        ref: "storage fixture, two service starts on target/web-store: 'origin request rejected path=/__storage/1' then 'path=/__storage/2', both runs 'web profile kind=Persistent engine=WebView2'"
        status: pass
    human_judgment: false

duration: 14min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 24: The wallpaper JavaScript API on WebView2 Summary

**WebView2 now runs the same property calls, legacy-ordered compatible calls, DataWanted/data pushes and in-page pause as CEF, from one shared `sucrose_engine_web::page` module that CEF also uses.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-26T23:09:00Z
- **Completed:** 2026-11-26T23:23:00Z
- **Tasks:** 2
- **Files modified:** 8 (2 created)

## Accomplishments

- `page.rs` holds PageState (atomics plus mutexes, built from plain values), Complaints, describe, page_calls, wanted_groups, load_templates and push_calls, moved from CEF with the same behavior and log text.
- WebView2: `HostStart.page_calls` runs through ExecuteScript inside NavigationCompleted (status 200) before the loaded hook; the pump drains Pause (in-page pause script), Loop and Shuffle (re-run compatible calls) and Data (push_calls with templates read on the first push); DataWanted is sent once after the first 200 load.
- CEF's private State page logic and its page-call tests are gone; CEF keeps the mouse-move coalescing state and its queue tests. The page tests now exist once in `tests/page.rs` (16 tests including the calls fixture order and its five data groups).
- ENG-02 concurrency: a self-spawning WebView2 lock holder test (second and third instance ephemeral in slots 1 and 2, CEF keeps its own lock, killed holder releases) and a two-thread `serve::plan` test.
- FEATURES rows for PropertyListener, SucroseCompatible, system data (Date and BIOS/motherboard) and the D-33 row record WebView2 on the shared module (status stays partial).

## Live runs

- Calls fixture on WebView2: 10 `__calls/property`, 2107 `__calls/compatible` (191 `SystemDate`) reports in 20 s; `data wanted groups=[Bios, Date, Memory, Processor, Motherboard]` once; `data hub started` in the Service log.
- Calls fixture on CEF after the move: 10 property and 1568 compatible reports (bound: 10 and 6).
- Storage fixture on WebView2, two starts on one scratch folder: `/__storage/1` then `/__storage/2`.

## Task Commits

1. **Task 1: calls fixture on WebView2 through the shared page module** - `855439f5` (feat)
2. **Task 2: CEF on the shared module, storage and concurrency edge on WebView2** - `568c8c1a` (refactor)

**Plan metadata:** the docs commit that follows this SUMMARY.

## Files Created/Modified

- `crates/sucrose-engine-web/src/page.rs` - the shared page state and calls
- `crates/sucrose-engine-web/tests/page.rs` - order, gating and data push tests, calls fixture
- `crates/sucrose-engine-web/src/lib.rs` - exports the module
- `crates/sucrose-engine-web/tests/profile.rs` - WebView2 concurrency and planner tests
- `crates/sucrose-engine-webview2/src/{engine,host}.rs` - page calls, data, pause on WebView2
- `crates/sucrose-engine-cef/src/engine.rs` - uses the shared module
- `docs/FEATURES.md` - rows 67-70 and 78

## Decisions Made

See key-decisions. No new project decision; D-24, D-33 and D-35 were applied to WebView2.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] CEF regression command did not enable the planner's log target**
- **Found during:** Task 2 (CEF calls fixture run)
- **Issue:** the plan's CEF command uses `SUCROSE_LOG="info,sucrose_engine_cef=debug"`, but the "origin request rejected" lines of CEF come from `sucrose_engine_web::serve`, so the first run counted 0 property reports even though 10 `page call kind="property"` lines were logged. (WebView2 logs them under its own target, so Task 1 and the storage run were unaffected.)
- **Fix:** reran with `info,sucrose_engine_cef=debug,sucrose_engine_web=debug`: 10 property and 1568 compatible reports.
- **Files modified:** none
- **Committed in:** n/a

### TDD note

The two tasks are marked `tdd="true"` but the tests and the module were written in the same step, so no separate failing run was captured; the commits are `feat` and `refactor`, not a `test`/`feat` pair. The tests assert the plan's behavior list and pass.

**Total deviations:** 1 auto-fixed (Rule 3), 1 process note. **Impact:** none on the code.

## Line endings

Files that were CRLF in the working copy and restored to CRLF before commit (cargo fmt rewrote some to LF): `crates/sucrose-engine-web/src/lib.rs`, `crates/sucrose-engine-web/tests/profile.rs`, `crates/sucrose-engine-cef/src/engine.rs`, `docs/FEATURES.md` (no final newline kept). `crates/sucrose-engine-webview2/src/{engine,host}.rs` were LF and stay LF; the two new files are LF. No untouched file shows as modified.

## Issues Encountered

None. One `taskkill` of a CEF subprocess reported it had already exited; no `sucrose.exe` was left running.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 04-25. WAPI-03 (Graphic, Network, Storage, Battery groups) stays Round D, PLAY-01 and PLAY-05 Round E; ENG-02, WAPI-01, WAPI-02, WAPI-04 and FMT-07 are marked by 04-37.

## Self-Check: PASSED

- FOUND: crates/sucrose-engine-web/src/page.rs, crates/sucrose-engine-web/tests/page.rs
- FOUND commits: 855439f5, 568c8c1a
- cargo xtask ci exit 0

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
