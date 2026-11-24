---
phase: 03-m2-service-ipc-and-portal-connected
plan: 40
subsystem: tooling
tags: [rust, windows, heap, dxgi, xtask, measurement, d-25, gap-closure, tdd]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the log-split D-25 summary and the "page shown" debug event (03-36), the atlas release fix (03-27), the 03-28 library page edits
provides:
  - "sucrose_platform::MemoryDetail and memory_detail(): process heap allocated and committed bytes and this process's video memory in use (local and non-local, all DXGI adapters) on Windows, None elsewhere"
  - "heap_allocated_mb, heap_committed_mb, gpu_local_mb and gpu_non_local_mb on the Portal's page shown event, read only when the debug filter is on"
  - "portal_attribution, Verdict, Part, Growth, Attribution, PageMemory in xtask/src/measure.rs; the attribution line of --portal-summary --portal-log and the verdict in each --portal-row cell"
affects: [03-37, 03-38, 03-39]
actuals:
  tokens: 8818
  tasks: 2
  commits: 4
plan_head_before: 5c5574138ce7cb43e1ed1cb3dc57940b1264be5a
plan_head_after: 7e731173131d7a1a5abcee97c36f9d691ace2295
tech-stack:
  added: []
  patterns:
    - "A measurement reading is taken only inside tracing::enabled!(DEBUG), so a normal run pays nothing"
    - "A growth is attributed mechanically: nothing within the cap, else the part that is at least half of the private growth, else unattributed with the largest part named"
key-files:
  created:
    - crates/sucrose-platform/src/os_windows/memory.rs
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - apps/sucrose-portal/src/library/mod.rs
    - xtask/src/measure.rs
    - xtask/src/main.rs
key-decisions:
  - "Verdict order: 'not recorded' is decided first (a log without the four fields cannot be attributed, whatever the growth), then 'within the cap' (private growth at most 48 MiB, in the MiB the series values use), then the largest of gpu, held heap and allocator retention when it is at least half of the private growth, else 'unattributed' naming the largest part and its growth"
  - "PageEvent carries the figures in a PageMemory struct, so a line without them (an older Portal) parses with every field None"
requirements-completed: [UI-07]
coverage:
  - id: D1
    description: "memory_detail() reports the process heap's allocated and committed bytes and this process's video memory on the local and non-local segment groups on Windows; None elsewhere"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows/memory.rs#memory_detail_counts_live_heap_blocks"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows/memory.rs#memory_detail_reports_video_memory_when_an_adapter_exists"
        status: pass
    human_judgment: false
  - id: D2
    description: "The page shown event carries the four memory fields when the debug filter is on and keeps its message and the fields page, pages, thumbs, thumb_bytes and preview_playing"
    requirement: UI-07
    verification:
      - kind: integration
        ref: "smoke: target/release/sucrose-portal.exe --no-focus with SUCROSE_LOG=info,sucrose_portal::library=debug, grep 'heap_allocated_mb=' | grep -c 'gpu_local_mb=' prints 1"
        status: pass
    human_judgment: false
  - id: D3
    description: "The log-split summary attributes the private growth to gpu atlas, held heap or allocator retention with the mechanical verdict, prints the attribution line and adds the verdict to the --portal-row cells; it is idempotent and older logs still summarize"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs#portal_attribution_names_the_gpu_when_video_memory_grows_most"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_attribution_is_unattributed_when_no_part_reaches_half"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_attribution_is_not_recorded_for_a_log_without_the_fields"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_attribution_is_the_same_twice"
        status: pass
    human_judgment: false
duration: 14min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 40: Growth-source attribution for the Portal's paging memory Summary

**The Portal's page log now carries its process heap (allocated, committed) and its video memory in use, and the log-split D-25 summary attributes the private-bytes growth mechanically to GPU atlas, held heap or allocator retention, so the 03-37/03-38 reruns tell the three candidate causes apart.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-24T17:14:47Z
- **Completed:** 2026-11-24T17:28:24Z
- **Tasks:** 2
- **Files modified:** 7 (1 created)

## Accomplishments

- `sucrose_platform::memory_detail()` reads `HeapSummary` on `GetProcessHeap` and `IDXGIAdapter3::QueryVideoMemoryInfo` (`CurrentUsage`, local and non-local, summed over every adapter); every unsafe block has a SAFETY comment, a failed call gives None, other OSes give an all-None `MemoryDetail`.
- The "page shown" debug event adds `heap_allocated_mb`, `heap_committed_mb`, `gpu_local_mb` and `gpu_non_local_mb`, read only when the debug filter is on; message and the older fields are unchanged.
- `portal_attribution` compares the private growth (03-36's first-page value to the last round's end value) with the growth of GPU memory, held heap, allocator retention and the thumbnail bytes between the first page-leaving event and the closing page-1 event of the last round; the verdict rules are exactly those of the plan.
- `--portal-summary --portal-log` prints an `attribution:` line after the working-set and private lines; `--portal-row --portal-logs` appends `attribution <verdict>` to each scenario's cell; the USAGE text describes both.

## Evidence for 03-39

- **Smoke (unattended, 40 synthetic themes, release build, Portal started with `--no-focus`, 10 s, then `taskkill.exe`)**, the first page line of `target/synth-small/Log/Portal.2026-11-24.log`:

  `2026-11-24T17:21:19.287101Z DEBUG sucrose_portal::library: page shown page=1 pages=2 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=38 heap_committed_mb=40 gpu_local_mb=28 gpu_non_local_mb=0`

  The verify pipeline's count of lines with both `heap_allocated_mb=` and `gpu_local_mb=` was 1. `tasklist.exe` listed no Sucrose process afterwards.
- **One live 64 MiB `Vec` (single block, taken inside `memory_detail_counts_live_heap_blocks`)** raised `heap_allocated` by 64 to 65 MiB over repeated runs (an earlier run beside a parallel test read 86 to 95 MiB, which is why the observation lives in the one test). So large decode-sized buffers do show in `heap_allocated_mb`: a growth in held decode buffers would be named "held heap".
- The Portal itself already shows 28 MB of video memory at first page on the smallest library, so the "gpu atlas" part is measurable from the first line.

## Task Commits

1. **Task 1: page log carries heap and video memory (tracer)**
   - RED `53dd1a3d` (test): MemoryDetail, memory_detail() and a platform stub; the live-heap and video-memory tests fail on the missing reading
   - GREEN `a5e84eac` (feat): the Windows reading and the Portal page event fields
2. **Task 2: attribution in the log-split summary**
   - RED `ef1d261a` (test): parser fields, types and a stub; eight attribution tests fail
   - GREEN `7e731173` (feat): portal_attribution, the CLI line, the row cell, USAGE

**Plan metadata:** the docs commit that follows this file.

## TDD Gate Compliance

Both tasks have a `test(03-40)` commit followed by a `feat(03-40)` commit. The RED evidence is cargo test output, a format `gsd_run check tdd-red-evidence` does not read, so it was inspected by hand: Task 1's RED failed in `memory_detail_counts_live_heap_blocks` (`expect` on a missing heap reading, the planned behavior absent) and `memory_detail_reports_video_memory_when_an_adapter_exists` (adapters present, fields None); Task 2's RED failed on the verdict assertions of the attribution tests (`left: NotRecorded right: Named(GpuAtlas)` and the like) with the stub in place. Neither failed on a compile or setup fault.

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows/memory.rs` - heap and per-process video memory readings (CRLF), with the tests
- `crates/sucrose-platform/src/lib.rs`, `os_windows.rs`, `Cargo.toml` - `MemoryDetail`, `memory_detail()`, the module and the `Win32_System_Memory` feature
- `apps/sucrose-portal/src/library/mod.rs` - the four fields on the page shown event
- `xtask/src/measure.rs`, `xtask/src/main.rs` - `PageMemory`, `portal_attribution`, `log_attribution`, `attribution_text`, the summary line, the row cell, USAGE

## Decisions Made

- 'not recorded' is decided before 'within the cap': a log without the four fields cannot be attributed whatever the growth, and the cap check would otherwise hide the fact that nothing was recorded.
- The cap is 48.0 in the series' MiB units (the plan's "48 MiB (50.3 MB)"; the series values divide by 1024*1024, so the figures compare directly).
- `PageEvent` gained a `memory: PageMemory` field; the two existing test literals in 03-36's tests were updated for it (`PageMemory::default()`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The 64 MiB observation test raced the live-heap test**
- **Found during:** Task 1 (full `cargo test -p sucrose-platform`)
- **Issue:** the planned "print the rise for one 64 MiB Vec" as its own test ran in parallel with `memory_detail_counts_live_heap_blocks`, whose 56 MiB margin it broke (flaky failure, observed rise 86 to 95 MiB)
- **Fix:** the single-block observation now runs first inside `memory_detail_counts_live_heap_blocks`, so no parallel test disturbs either reading; five repeated runs passed
- **Files modified:** crates/sucrose-platform/src/os_windows/memory.rs
- **Committed in:** a5e84eac

**2. [Rule 1 - Test data] A series-level attribution test used the wrong private scale**
- **Found during:** Task 2
- **Issue:** the test series' private bytes are twice its working-set values, so the expected growth was 160 MB, not 80
- **Fix:** scaled the test series so the private growth is 80 MB and the video memory growth of 60 MB is at least half of it
- **Files modified:** xtask/src/measure.rs
- **Committed in:** 7e731173

---

**Total deviations:** 2 auto-fixed (both in tests). **Impact:** none on the product code. Two attribution tests beyond the seven planned were added (memory fields parsed from a line, attribution taken from a series' log).

## Issues Encountered

None.

## Known Stubs

None.

## Threat Flags

None. The new FFI surface (HeapSummary, QueryVideoMemoryInfo) is the plan's T-03-86; the page event carries MiB figures only (T-03-85), read only when the debug filter is on (T-03-87).

## Next Phase Readiness

- 03-37 and 03-38 can run the D-25 reruns with the Portal started under `SUCROSE_LOG=info,sucrose_portal::library=debug`; the summary then prints the attribution line 03-39 reads.
- The real-library reruns were not run here (they need the maintainer's hand).

## Self-Check: PASSED

- Files: `crates/sucrose-platform/src/os_windows/memory.rs` found; platform, Portal and xtask edits in the commits below.
- Commits found in HEAD ancestry: 53dd1a3d, a5e84eac, ef1d261a, 7e731173.
- `cargo xtask ci` exit 0 (fmt, clippy -D warnings, tests); acceptance greps: Win32_System_Memory 1, pub fn memory_detail 1, SAFETY 5, heap_allocated_mb 1, portal_attribution 1, allocator retention 4; line endings kept (CRLF platform files and memory.rs 163/163, LF Portal and xtask files).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
