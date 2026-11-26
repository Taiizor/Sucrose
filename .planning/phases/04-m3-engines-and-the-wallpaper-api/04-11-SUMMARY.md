---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 11
subsystem: system-data
tags: [sysinfo, data-hub, pdh, wmi, memory, processor, bios, motherboard, javascript-api, sucrose-compatible]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-10 sucrose-sysinfo crate with date_snapshot, DataHub/DataModule with the 100 ms tick and per-renderer groups, ToRenderer::Data, the CEF push of every template"
provides:
  - "sucrose_sysinfo::{memory_snapshot, ProcessorSampler, bios_snapshot, motherboard_snapshot} and the legacy default payloads memory_unavailable, processor_unavailable, bios_unavailable, motherboard_unavailable"
  - "data hub sampler (system_sampling) that holds the PDH counters and the WMI texts only while a page wants the group, plus the Release hook of DataModule"
  - "workspace dependency wmi 0.18.4 (Windows only, behind sucrose-sysinfo)"
  - "calls fixture page that shows each system group it receives as a key/value table"
affects: [04-20, round-d-system-data]
plan_head_before: 71d79351c47cabb09c6aa03eb440e1b05f129d09
plan_head_after: f91255c0bcaa9ba5cba8e9cfa779f075b388ef8a
actuals:
  tokens: 15400
  tasks: 2
  commits: 4
tech-stack:
  added: ["wmi 0.18.4 (WMI queries, Windows only)", "windows crate feature Win32_System_Performance (PDH)"]
  patterns:
    - "the pure payload builders (Tracker, snapshot, bios_payload, motherboard_payload) are separate from the OS reads, so key order and value kinds are unit-tested on every OS"
    - "a failing source gives the group's legacy default (State false, empty values) and one log line; the failed read is retried after 60 s"
    - "the sampler's state sits behind one lock and a Release hook lets the service thread close counters the hub thread opened"
key-files:
  created:
    - crates/sucrose-sysinfo/src/memory.rs
    - crates/sucrose-sysinfo/src/processor.rs
    - crates/sucrose-sysinfo/src/wmi_info.rs
    - crates/sucrose-sysinfo/tests/payload.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-sysinfo/Cargo.toml
    - crates/sucrose-sysinfo/src/lib.rs
    - apps/sucrose/src/data_hub.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/fixtures/web-themes/calls/index.html
    - docs/FEATURES.md
key-decisions:
  - "Bios and Motherboard are asked for every 10th tick like the other groups (they were 600) because the sampler caches the WMI read; a page that starts later would otherwise wait up to a minute"
  - "CoresMin and CoresMax keep the legacy quirk of being updated inside the per-core loop (CoresMin follows the first cores, not the lowest core); the legacy behaviour is kept, not improved silently, and recorded in FEATURES"
  - "The figures are f32 written with the shortest round-trip digits, as Newtonsoft writes the C# float values"
  - "The Processor State is true only when the WMI static read worked, as in the legacy code; the PDH figures flow either way"
requirements-completed: []
coverage:
  - id: D1
    description: "Memory data (GlobalMemoryStatusEx, once a second) reaches compatible pages with the legacy keys, GiB figures and percentages"
    requirement: "WAPI-03"
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/payload.rs#the_memory_group_has_the_legacy_keys_and_gib_values"
        status: pass
      - kind: integration
        ref: "cargo build -p sucrose + synth-library calls fixture + sucrose.exe --no-portal: renderer log counts __calls/compatible/SystemMemory (165)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Processor data (WMI static fields, PDH total and per-core load once a second, running Min/Max, Cores array) reaches compatible pages with the legacy keys and value kinds"
    requirement: "WAPI-03"
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/payload.rs#the_second_processor_sample_has_the_legacy_keys_and_consistent_figures"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/payload.rs#a_processor_sampler_can_be_created_twice_in_one_process"
        status: pass
      - kind: integration
        ref: "calls fixture run: renderer log counts __calls/compatible/SystemProcessor (165)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Bios and Motherboard data (WMI, read once per hub lifetime) reach compatible pages with the legacy keys"
    requirement: "WAPI-04"
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/payload.rs#the_bios_group_has_the_legacy_keys_and_texts"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/payload.rs#the_motherboard_group_has_the_legacy_keys_and_the_joined_name"
        status: pass
      - kind: integration
        ref: "calls fixture run: renderer log counts __calls/compatible/SystemBios and SystemMotherboard (165 each)"
        status: pass
    human_judgment: false
  - id: D4
    description: "A group is collected only while a running page names it; a failing source gives the legacy default and one log line (D-24, D-002)"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/data_hub.rs#a_pass_reads_only_the_groups_somebody_wants, #a_group_is_released_when_its_last_renderer_stops_wanting_it_while_the_hub_goes_on, #every_group_is_released_when_the_hub_stops, #a_failed_one_time_read_gives_the_legacy_default_and_is_not_retried_at_once"
        status: pass
    human_judgment: false
  - id: D5
    description: "The calls fixture page shows each system group as a table and reads right to the maintainer's eye"
    verification: []
    human_judgment: true
    rationale: "A visual check of the table layout is planned at 04-20 with the maintainer; no test asserts how the page looks"
duration: 31min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 11: Memory, Processor, BIOS and Motherboard data Summary

**Memory (GlobalMemoryStatusEx), Processor (PDH per-core load plus WMI static fields), Bios and Motherboard (WMI, read once) reach compatible CEF pages with the legacy keys and value kinds, sampled only while a running page names the group**

## Performance

- **Duration:** 31 min
- **Started:** 2026-11-26T04:43:34Z
- **Completed:** 2026-11-26T05:14:00Z
- **Tasks:** 2
- **Files modified:** 12 (4 created)

## Accomplishments

- `sucrose-sysinfo` has the Memory, Processor, Bios and Motherboard providers. Each returns the group as a JSON object with the keys of the C# `Data.cs` in the same order; payload tests compare keys, order and value kinds, and the pure builders are tested on every OS.
- `ProcessorSampler` opens one PDH query (English counter paths, so display language does not matter) with `_Total` and one counter per logical processor, keeps the running Min/Max like the legacy code and closes the query on drop. A sampler created twice in one process works.
- The data hub holds the counters and the WMI texts only while a page wants the group: a `Release` hook of `DataModule` lets go of a group when its last page stops wanting it and of every group when the hub stops. A page that needs only the date opens no PDH query and no WMI connection (D-24).
- A failing source (WMI unavailable, a counter missing) sends the group's legacy default (`State` false, empty values) with one log line; the failed read is tried again after 60 s.
- End to end: with the `calls` fixture, the renderer log shows 165 calls each of SystemProcessor, SystemMemory, SystemBios, SystemMotherboard and SystemDate in a 13 s run. The fixture page now shows each group as a key/value table.

## Task Commits

1. **Task 1: Memory and Processor data reach the page** - RED `378ec3fc` (test), GREEN `bcd26133` (feat)
2. **Task 2: BIOS and motherboard data through WMI, read once** - RED `98360250` (test), GREEN `f91255c0` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY, STATE.md and ROADMAP.md.

## Files Created/Modified

- `crates/sucrose-sysinfo/src/memory.rs` - Memory group (f32 figures, GiB, legacy load formula, zero total guarded)
- `crates/sucrose-sysinfo/src/processor.rs` - Tracker (running figures, Cores entries), PDH safe layer, ProcessorSampler
- `crates/sucrose-sysinfo/src/wmi_info.rs` - WMI first-instance reads for Processor static fields, Bios and Motherboard
- `crates/sucrose-sysinfo/tests/payload.rs` - legacy-shaped payload tests
- `crates/sucrose-sysinfo/src/lib.rs`, `crates/sucrose-sysinfo/Cargo.toml`, `Cargo.toml`, `Cargo.lock` - exports, `float` helper, features and the wmi dependency
- `apps/sucrose/src/data_hub.rs` - SystemState (Held, once, retry), Release hook, cadence, tests
- `apps/sucrose/src/service.rs` - builds the module with the release hook
- `apps/sucrose/tests/fixtures/web-themes/calls/index.html` - system group tables
- `docs/FEATURES.md` - the two System data rows

## Decisions Made

See the frontmatter. None of them reopens an Accepted entry in docs/DECISIONS.md.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] wmi dependency and wmi_info.rs arrive with Task 1**
- **Found during:** Task 1
- **Issue:** the Processor group's static fields (Name, Core, Thread) come from WMI, so Task 1 needed the wmi crate and a WMI read; the plan lists both under Task 2.
- **Fix:** Task 1 adds the workspace dependency, `wmi_info.rs` with the first-instance helper and `processor_info`; Task 2 adds Bios and Motherboard to the same module.
- **Files modified:** Cargo.toml, Cargo.lock, crates/sucrose-sysinfo/Cargo.toml, crates/sucrose-sysinfo/src/wmi_info.rs
- **Committed in:** bcd26133

**2. [Rule 3 - Blocking] apps/sucrose/src/service.rs changed (two lines)**
- **Found during:** Task 1
- **Issue:** a sampler that holds open PDH counters needs a way to be told that no page wants a group any more; the existing `Sampler` closure could not be told, and a counter opened on the hub thread must be closable from the service thread. service.rs builds the module and is not in the plan's file list.
- **Fix:** `DataModule::with_release` plus `system_sampling()` returning the sampler and its release; service.rs passes both.
- **Files modified:** apps/sucrose/src/data_hub.rs, apps/sucrose/src/service.rs
- **Committed in:** bcd26133

**3. [Rule 1 - Bug] Bios and Motherboard cadence 600 ticks to 10 ticks**
- **Found during:** Task 2
- **Issue:** the 04-10 cadence asked for these groups once a minute at most, so a page that starts after the hub would wait up to a minute for BIOS data (a hub tick is read when `tick % 600 == 0`). The legacy payload carries them in every message.
- **Fix:** every 10th tick; the sampler returns the cached text, so each ask costs a clone, and the WMI query still runs once per hub lifetime.
- **Files modified:** apps/sucrose/src/data_hub.rs
- **Committed in:** f91255c0

**4. [Rule 2 - Missing critical] failed one-time reads are retried after 60 s, not every tick**
- **Found during:** Task 2
- **Issue:** at a 1 s cadence a machine without WMI would be queried every second.
- **Fix:** `Held::Failed(Instant)` keeps the failure for 60 s and the group reports its legacy default meanwhile.
- **Files modified:** apps/sucrose/src/data_hub.rs
- **Committed in:** f91255c0 (and bcd26133 for the Processor counters)

**5. Fixture page lists a repeating system group once**
- The page used to append a list item for every call; with groups arriving ten times a second the list would grow without end, so each system group updates its table in place and is listed once. The fetch for each call (which the verify greps count) is unchanged.

---

**Total deviations:** 4 auto-fixed (2 Rule 3, 1 Rule 1, 1 Rule 2) and 1 page-only change
**Impact on plan:** all needed for correctness or to build the planned behaviour; no scope growth.

## TDD Gate Compliance

Both tasks followed RED then GREEN (`378ec3fc` then `bcd26133`; `98360250` then `f91255c0`). `cargo test` output is not a report format that `gsd_run check tdd-red-evidence` reads (Rust is a known gap of the format list), so no machine record was written. Semantic assessment of RED: the stubs return `Unsupported`, the three (Task 1) and two (Task 2) Windows tests failed on `.expect(...)` of exactly that provider call, at the planned target tests; the unavailable-default tests that do not need a provider passed in RED by design. No REFACTOR commit.

## Issues Encountered

- `cargo fmt` rewrote some touched files to LF; line endings were restored to CRLF before each commit (`git ls-files --eol` shows w/crlf, FEATURES.md and Cargo.toml keep their missing final newline).
- The WMI queries of a first read run on the hub thread, so a slow WMI start delays that one tick's Date snapshot (the legacy code used fire-and-forget tasks). Only the first read of a group per hub lifetime pays it.

## Known Stubs

None.

## Threat Flags

None. T-04-27 (serial number to pages that ask for SystemBios) and T-04-28 (queries exist only while wanted, released on drop, failures logged once) are implemented as planned; wmi is the audited crate of T-04-SC.

## Next Phase Readiness

- Round A data groups are in. 04-20 does the maintainer's visual check of the calls page and the 8 Web themes; WAPI-04 stays open until then.
- Graphic, Network, Storage, Battery follow in Round D and Audio in Round B; the `Held`/`once`/Release pattern in data_hub.rs is the place to hang them.

## Self-Check: PASSED

All created files exist and the four task commits (378ec3fc, bcd26133, 98360250, f91255c0) are on the branch; `cargo xtask ci` (fmt, clippy -D warnings, tests) passes.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
