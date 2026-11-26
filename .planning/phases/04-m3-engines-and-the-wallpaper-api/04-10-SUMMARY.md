---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 10
subsystem: system-data
tags: [sysinfo, data-hub, cef, javascript-api, sucrose-compatible, date, ipc, unsafe-provider-crate]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-03 DataGroup and SucroseCompatible::data_groups; 04-09 js module, CEF non-data compatible calls, LEGACY_RUN_ORDER; modules.rs start_module"
provides:
  - "crates/sucrose-sysinfo: date_snapshot() -> Result<serde_json::Value, SysInfoError>, SysInfoError::{Unsupported, Os}; unsafe-allowed provider crate (D-039)"
  - "ToService::DataWanted(Vec<DataGroup>) and ToRenderer::Data(Vec<(DataGroup, String)>), appended, protocol version stays 8"
  - "apps/sucrose/src/data_hub.rs: DataHub (100 ms worker), DataModule (wanted groups per renderer, hub exists only while any is wanted), per-group cadence, system_sampler"
  - "sucrose_engine_web::js::push_calls and Engine::push_data: a CEF page runs every template in the legacy order with the newest snapshot of each group on every push"
  - "log lines: renderer 'data wanted', service 'data hub started' / 'data hub stopped'"
  - "DECISIONS D-039; CLAUDE.md non-negotiable 5, platform-ffi.md paths and ARCHITECTURE list sucrose-sysinfo; FEATURES records the D-24 behaviour change"
affects: [04-11, 04-20, round-d-system-data]
plan_head_before: 40940eacce10984e07e71bd4c6a5ed2ab82f0e40
plan_head_after: c3e3a99a4c8b2a0f902729c8b7b3e3ca8113f8a5
actuals:
  tokens: 20584
  tasks: 2
  commits: 2
tech-stack:
  added: []
  patterns:
    - "the hub's pass is a plain function over (tick, wanted slots, sampler, deliver), so cadence, sharing and dropping are unit-tested without threads or a service"
    - "a snapshot is stored in the engine state and only a marker is queued, so a full command queue loses nothing"
    - "the per-OS provider returns Unsupported where there is none (non-Windows), never a faked value"
key-files:
  created:
    - crates/sucrose-sysinfo/Cargo.toml
    - crates/sucrose-sysinfo/src/lib.rs
    - crates/sucrose-sysinfo/src/date.rs
    - apps/sucrose/src/data_hub.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - crates/sucrose-core/src/compatible.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-cef/Cargo.toml
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-web/src/js.rs
    - crates/sucrose-engine-web/tests/js.rs
    - docs/DECISIONS.md
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
    - CLAUDE.md
    - .claude/rules/platform-ffi.md
key-decisions:
  - "D-039 (maintainer's answer to research open question 3 = 04-CONTEXT D-34): sucrose-sysinfo is an unsafe-allowed provider crate"
  - "The hub never reads the PerformanceCounter setting: web data no longer stops when it is off (D-24 behaviour change, recorded in FEATURES)"
  - "Every push runs all templates in LEGACY_RUN_ORDER (non-data keys again, each data key whose group has a snapshot), as the legacy engine does for every message it receives"
  - "A full per-renderer queue drops the data snapshot (Link::offer treats Data like Clock); the next tick sends a fresh one, so the hub never waits for a slow renderer"
requirements-completed: []
coverage:
  - id: D1
    description: "Date group with the legacy keys and value types, from the new provider crate"
    requirement: WAPI-03
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-sysinfo (5 tests: key order, value types, leap day, day names, live read on Windows)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The data hub exists only while a running page needs data, reads each wanted group once, drops a snapshot for a full queue, and stops with the last wanting renderer"
    requirement: WAPI-03
    verification:
      - kind: unit
        ref: "cargo test -p sucrose data_hub (12 tests)"
        status: pass
      - kind: integration
        ref: "calls fixture on CEF: 'data hub started' logged and 130 SystemDate reports from renderer 0 in about 12 s; hello fixture logs no 'data hub started'"
        status: pass
    human_judgment: false
  - id: D3
    description: "DataWanted and Data are appended wire variants and a CEF page runs the templates with the newest snapshots"
    requirement: WAPI-03
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-ipc, -p sucrose-engine-web --test js, -p sucrose-engine-cef (wire index, push_calls, snapshot storage, full queue)"
        status: pass
    human_judgment: false
  - id: D4
    description: "D-039 and the D-24 change are written where the unsafe-allowed crates and the system data rows are listed"
    requirement: WAPI-03
    verification:
      - kind: other
        ref: "grep counts of D-039, sucrose-sysinfo (CLAUDE.md, platform-ffi.md) and D-24 (FEATURES.md); eol listing unchanged per file"
        status: pass
    human_judgment: false
duration: 19min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 10: System data hub and the Date group Summary

**Live Date data flows from the new unsafe-allowed `sucrose-sysinfo` crate through an on-demand 100 ms data hub to a CEF page's `SystemDate` template, and D-039 records the crate in every place the unsafe rule is listed.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-26T04:19:20Z
- **Completed:** 2026-11-26T04:38:00Z
- **Tasks:** 2
- **Files modified:** 22 (4 created)

## Accomplishments

- `sucrose-sysinfo` exists with `date_snapshot()`. The keys and value types follow `src/Project/Sucrose.Backgroundog/Extension/Data.cs` `GetDateInfo` (lines 26-39) and the .NET `DateTime.Now` of `Helper/Specification.cs` (lines 70-83): `Day`, `Hour`, `Year`, `Month`, `Minute`, `Second`, `DayOfYear`, `Millisecond` as whole numbers (`int?` in `Struct/Data/Date.cs`), `State` true, `Kind` as `$"{Kind}"` which is `Local` for `DateTime.Now`, `DayOfWeek` as `$"{DayOfWeek}"` (the English day name), in that key order. Windows reads it with `GetLocalTime`; other systems answer `Unsupported`.
- The data hub (`DataModule`) starts through `start_module("data hub", ...)` only when a renderer's `DataWanted` names a group, ticks every 100 ms on one worker thread, reads each wanted group once however many renderers want it, and ends (thread joined, `data hub stopped`) when the last wanting renderer ends or reconnects. It has a per-group cadence table (Date every tick, performance groups every 1 s, Battery 2 s, Bios and Motherboard rarely) ready for plan 04-11.
- A CEF renderer sends `DataWanted` once after its first page loaded (status 200) with the groups its `SucroseCompatible.json` defines; an empty list sends nothing. Each `Data` message is stored per group and one push runs every template in the legacy order (the non-data keys again, each data key with a snapshot, `{0}` = indented JSON).
- Live proof with the `calls` fixture: `data hub started` in the service log, `data wanted groups=[Bios, Date, Memory, Processor, Motherboard]` in the renderer log, 130 `__calls/compatible/SystemDate` reports in about 12 s (the other six compatible keys about 131 each, as they run with every push). The `hello` fixture (no `SucroseCompatible.json`) logs no `data hub started`.
- D-039, CLAUDE.md non-negotiable 5, `.claude/rules/platform-ffi.md`, `docs/ARCHITECTURE.md` and the FEATURES System data row (including the D-24 behaviour change) are updated; CLAUDE.md stays at 80 lines.

## Task Commits

1. **Task 1: Live Date data reaches the compatible fixture on CEF** - `03b9d3ad` (feat)
2. **Task 2: D-34 written down, D-24 change in FEATURES** - `c3e3a99a` (docs)

**Plan metadata:** the commit that carries this SUMMARY, STATE.md and ROADMAP.md.

## Files Created/Modified

- `crates/sucrose-sysinfo/{Cargo.toml,src/lib.rs,src/date.rs}` - the provider crate (`allow(unsafe_code)`, `undocumented_unsafe_blocks` warning, one SAFETY-commented block)
- `apps/sucrose/src/data_hub.rs` - `DataHub`, `DataModule`, `Delivery`, `every_ticks`, `system_sampler`
- `apps/sucrose/src/service.rs` - DataWanted handling (only the renderer holding the slot counts), release on Hello and on Exited, `deliver_data`, `Link::offer` drops a full-queue `Data`
- `apps/sucrose/src/render.rs` - the `on_data_wanted` hook, `ToRenderer::Data` handed to the engine
- `crates/sucrose-ipc/src/protocol.rs` - the two appended variants with wire-index tests
- `crates/sucrose-engine-cef/src/engine.rs`, `crates/sucrose-engine/src/engine.rs`, `crates/sucrose-engine-web/src/js.rs` - `push_data`, `push_calls`, the pump's data push
- docs and rules as listed above

## Decisions Made

See `key-decisions`. Plan 04-11 adds Memory, Processor, Bios and Motherboard providers to the same crate and wires them into `system_sampler`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Files outside the plan's list had to change**
- **Found during:** Task 1
- **Issue:** The renderer reaches an engine only through the `Engine` trait (`Arc<dyn Engine>`) and sends messages through the renderer's own connection, and the CEF pump needs a JSON dependency, none of which the `files` list names.
- **Fix:** `Engine::push_data` (default no-op) in `crates/sucrose-engine/src/engine.rs`; `js::push_calls` plus tests in `crates/sucrose-engine-web`; the `on_data_wanted` hook and `ToRenderer::Data` handling in `apps/sucrose/src/render.rs`; `serde_json` (an existing workspace dependency, no new crate) for `sucrose-engine-cef`; `serde_json` moved from dev to normal dependencies of `sucrose`; `Status::after` and `Link::offer` in `service.rs`.
- **Files modified:** as listed
- **Verification:** `cargo xtask ci` passes
- **Committed in:** `03b9d3ad`

**2. [Rule 1 - Bug] Working-copy line endings restored**
- **Found during:** Task 2 verification (`git ls-files --eol`)
- **Issue:** The edit helper rewrote CRLF files with LF endings in the working tree.
- **Fix:** Converted every touched file back to its original ending (CRLF except `Cargo.lock`, `crates/sucrose-core/src/compatible.rs` and `docs/DECISIONS.md`, which were LF). The committed (index) content is LF either way, so no commit contains the change.
- **Verification:** the eol listing shows the original endings; `docs/FEATURES.md` still has no newline after its last line.

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both necessary; no scope change.

## Issues Encountered

- `data hub stopped` is not in the live log because `taskkill` ends the service before its main loop sees the renderer exit; the stop path is covered by `the_hub_runs_while_a_renderer_wants_data_and_stops_with_the_last_one` (no sampling after the last release).
- `git status` lists several CRLF working-copy files as modified although `git diff` is empty (the line-ending artifact CLAUDE.md describes), and `crates/sucrose-engine-web/src/args.rs` and `tests/args.rs` were already modified before this plan; none of them were touched or committed here.

## Known Stubs

- Groups other than Date have no provider yet: `system_sampler` answers `None` for them, the hub logs one debug line per group and sends nothing. Intentional: Memory, Processor, Bios and Motherboard arrive in plan 04-11, Graphic, Network, Storage, Battery and Audio in Round D. The calls fixture's other data functions therefore do not report yet.

## Threat Flags

None: the new surface (a renderer-to-service message and a service-to-renderer snapshot over the existing authenticated local connection, one Win32 time call) is covered by T-04-24 to T-04-26.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04-11 can add providers to `sucrose-sysinfo` and one match arm in `system_sampler`; the hub, cadence table, messages and CEF push need no change. WAPI-03 stays open (Round A part: Date now; Processor and Memory next).

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED
