---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 40
subsystem: platform
tags: [windows, queryDisplayConfig, monitors, stable-id, spike, displays]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-39 (Round B end); MonitorInfo, topology, Fleet::apply_snapshot"
provides:
  - "MonitorInfo.device_path, MonitorInfo.edid, MonitorInfo.friendly and MonitorInfo::stable_id"
  - "DisplayTarget, attach_display_ids, os_windows::display::display_targets (QueryDisplayConfig)"
  - "The monitor identified service log line (device, stable_id, edid, friendly)"
  - "Spike verdict for research question 3: the device path survived a DISPLAY9 replug"
affects: [04-41, 04-56, DESK-03]
actuals:
  tokens: 40000
  tasks: 3
  commits: 2
plan_head_before: 6965b5153efcc0036013db2e41e701ed521bc22a
plan_head_after: 97ab8ad56f149b1d1d592a0004fd7fde8761be8d
tech-stack:
  added: ["windows crate feature Win32_Devices_Display (no new crate)"]
  patterns: ["pure join of OS display targets onto monitors by GDI name, never by position; GDI name as the D-08 fallback"]
key-files:
  created: [crates/sucrose-platform/tests/display.rs]
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/geometry.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows/display.rs
    - apps/sucrose/src/topology.rs
    - apps/sucrose/src/fullscreen.rs
    - apps/sucrose/src/service.rs
    - docs/ARCHITECTURE.md
key-decisions:
  - 'device path stable across replug: yes (DISPLAY9 path DISPLAY#MTT1337#1&28a6823a&2&UID256#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}, EDID MTT-1337, read-1 = read-2 = read-3; DISPLAY1 path DISPLAY#BOE0823#4&31aeec77&0&UID8388688#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}, EDID BOE-0823, the same in all three reads; full ids in the body); 04-41 keeps the device path as the id and needs no EDID-derived second key for a replug (D-08 as confirmed)'
  - "Reboot stability is not checked here: spike.txt records LastBootUpTime 28 Kas 2026 Cumartesi 15:26:26 (unchanged at read-3); 04-56 compares its own read with it"
  - "MonitorInfo.id stays the GDI device name, so the slot map, topology diff and renderer paths are unchanged"
requirements-completed: []
coverage:
  - id: D1
    description: "Every monitor of a snapshot has a non-empty stable id (device path from QueryDisplayConfig, GDI name fallback)"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/geometry.rs attach_display_ids tests (join by name in any order, empty path, no target, equal EDID with different paths, empty list)"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose-platform --test display -- --ignored monitors_have_stable_ids (three reads on the maintainer machine)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service logs one monitor identified line per monitor of each applied snapshot"
    requirement: DESK-03
    verification:
      - kind: other
        ref: "target/monitor-ids/lib/Log service log, two lines at startup (DISPLAY1, DISPLAY9)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Spike: DISPLAY9's stable id survives an off/on replug"
    requirement: DESK-03
    verification:
      - kind: manual_procedural
        ref: "read-1.txt vs read-3.txt after the maintainer's replug"
        status: pass
    human_judgment: true
    rationale: "The replug was done by the maintainer by hand; the ids were compared by the executor, but that the display really left the configuration cannot be observed from the reads"
duration: about 20 min (continuation) 
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 40: Monitor stable id spike Summary

**Every monitor now carries a stable id read from QueryDisplayConfig (monitorDevicePath, GDI name fallback) with its EDID key, the service logs it, and DISPLAY9's device path was the same after a replug.**

## Performance

- **Tasks:** 3 (Task 1 and the first half of Task 2 by the first executor, the rest by the continuation)
- **Files modified:** 9 (8 modified, 1 created)
- **Commits:** 2 (measured from the ledger)

## Accomplishments

- `display_targets()` reads every active display path (source GDI name, target device path, friendly name, EDID manufacturer and product code) and `attach_display_ids` joins it onto the monitors by GDI name, never by position. `monitors()` returns the joined list; a failing call or a missing target leaves the GDI name as the stable id.
- The service logs `monitor identified` per monitor at startup and in `Fleet::apply_snapshot` (one helper, `log_monitors_identified`).
- Spike (research question 3): the maintainer replugged DISPLAY9 in two ways. His reply, verbatim: "kapatmak mı? bu sanal ekran o yüzden şuanda sadece bilgisayar ekranını seçtim ve ikinci ekran kapalı gibi duruyor. açıp kapatıp, sonrasında aygıt yöneticisinen devre dışı bırakıp tekrar etkinleştirdim" (translation: "Switch it off? It is a virtual display, so for now I selected 'PC screen only' and the second screen looks switched off. I switched it on and off, and after that I disabled it in Device Manager and enabled it again."). So the replug was a projection switch to "PC screen only" and back, then a Device Manager disable and enable of the virtual display adapter (the stronger replug). Treated as "tamam".
- Reads on the maintainer's machine (all in one boot, LastBootUpTime 28 Kas 2026 Cumartesi 15:26:26):

| Monitor | Stable id (read-1 = read-2 = read-3) | EDID key | Friendly |
|---|---|---|---|
| \\.\DISPLAY1 (primary) | \\?\DISPLAY#BOE0823#4&31aeec77&0&UID8388688#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7} | BOE-0823 | none |
| \\.\DISPLAY9 | \\?\DISPLAY#MTT1337#1&28a6823a&2&UID256#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7} | MTT-1337 | VDD by MTT |

- Verdict line: **device path stable across replug: yes**. The EDID key also did not change (BOE-0823, MTT-1337). The orchestrator ran the ignored test once earlier as a read-only state check (2026-11-28 14:43 UTC) and saw the same ids; the verdict rests on read-3.txt taken by this executor after the reply.
- Caveat: the reads cannot prove that DISPLAY9 left the display configuration during the replug; that rests on the maintainer's report.

## Task Commits

1. **Task 1: A monitor's stable id reaches the Service log** - `b1ad17a7` (feat)
2. **Task 2: DISPLAY9 off and on (checkpoint:human-action)** - no commit (maintainer action, read-3.txt taken after his reply)
3. **Task 3: Spike verdict and architecture note** - `97ab8ad5` (docs)

## Files Created/Modified

- `crates/sucrose-platform/src/geometry.rs` - MonitorInfo fields, stable_id, DisplayTarget, attach_display_ids and its unit tests
- `crates/sucrose-platform/src/os_windows/display.rs` - display_targets (QueryDisplayConfig, SAFETY comments), monitors() join
- `crates/sucrose-platform/tests/display.rs` - ignored real-desktop test `monitors_have_stable_ids`
- `crates/sucrose-platform/Cargo.toml` - Win32_Devices_Display feature
- `apps/sucrose/src/{service,topology,fullscreen}.rs` - the log line and the new MonitorInfo fields in literals
- `docs/ARCHITECTURE.md` - one paragraph in the Platform layer section

## Decisions Made

See key-decisions. Notes from Task 1: MonitorInfo also got `friendly: Option<String>` (the log line needs it); `edid_key` swaps the bytes of the OS manufacturer id before decoding the letters (confirmed by BOE-0823 and MTT-1337 against the device paths).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] friendly name kept on MonitorInfo**
- **Found during:** Task 1
- **Issue:** the plan allowed either keeping the friendly name on MonitorInfo or logging it from the platform; the service log line needs it.
- **Fix:** `MonitorInfo.friendly: Option<String>`, filled by attach_display_ids.
- **Files modified:** geometry.rs and the MonitorInfo literals
- **Committed in:** b1ad17a7

**Total deviations:** 1 (a choice the plan left open). **Impact:** none beyond one extra field.

## Issues Encountered

- `cargo xtask ci` passed once; on a second run the portal test `a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once` failed under build load and passed on rerun (35 passed, 0 failed), a timing flake of portal.rs.
- Line endings: CRLF was restored before b1ad17a7 on fullscreen.rs, service.rs, topology.rs, sucrose-platform Cargo.toml (no final newline kept), geometry.rs, lib.rs and os_windows/display.rs; the new tests/display.rs is LF; docs/ARCHITECTURE.md stayed CRLF (edited at byte level, verified w/crlf). The final `cargo fmt` pass left no working-copy change.

## User Setup Required

None.

## Next Phase Readiness

04-41 can store `Engine.DisplayAssignments` keyed by `MonitorInfo::stable_id()` (device path, GDI fallback) without an EDID second key. Reboot stability is still unchecked; 04-56 compares its read with the boot time above and records "not checked" if no reboot happened.

## Self-Check: PASSED

- Created: crates/sucrose-platform/tests/display.rs exists; commits b1ad17a7 and 97ab8ad5 are on develop-rust; `cargo xtask ci` green (one portal flake, rerun green); no Rust Sucrose process runs.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
