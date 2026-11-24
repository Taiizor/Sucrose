---
phase: 03-m2-service-ipc-and-portal-connected
plan: 05
subsystem: xtask
tags: [rust, xtask, synthetic-library, image, jpeg, gif, d-25]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Settings model and the atomic store (plan 01), library::scan and app_version (plan 02), Warehouse.hint_tray_icon and Library.delete_corrupt (plan 04)
provides:
  - "cargo xtask synth-library --out <dir> --count <n> [--seed <u64>] [--force]: synthetic library plus prepared Setting/Sucrose.json"
  - "xtask::synth_library::{run, theme_name, Plan}"
  - image 0.25 as a workspace dependency (default features off)
  - target/synth-500: a 500-theme library with real-sized images for the D-25 measurement
affects: [03-08, 03-10, 03-24]
actuals:
  tokens: 7300
  tasks: 2
  commits: 3
plan_head_before: fae3530698104fd9733ce2a6a6d41186343e6beb
plan_head_after: ea64e4fa814cf23f23e54edc01ded962b81cb645
tech-stack:
  added: [image 0.25 (jpeg, png, gif, webp, tiff)]
  patterns: [target-driven image encoding (JPEG quality search, GIF frame count from the first frame's size), marker file that makes a generated folder safe to rewrite, generator self-check with the production scan]
key-files:
  created:
    - xtask/src/synth_library.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - xtask/Cargo.toml
    - xtask/src/main.rs
key-decisions:
  - "--out is refused when it is inside the real Sucrose data folder (paths::legacy_data_dir), even with --force"
  - "The .synth marker is written first as 'incomplete' and rewritten as count and seed only after the library scan self-check passes, so an interrupted run can be repeated without --force and is never reported up to date"
  - "Pool images are generated once per run under <out>/.pool and copied round-robin; JPEG size is hit by a quality search, GIF size by a frame count taken from the first frame's encoded size"
requirements-completed: [UI-07]
coverage:
  - id: D1
    description: "cargo xtask synth-library writes n themes, two corrupt folders, a loose file and a prepared Sucrose.json (nothing selected, tray hidden, tray hint off); the service loads it as Loaded and starts no renderer"
    requirement: UI-07
    verification:
      - kind: integration
        ref: "cargo xtask synth-library --out target/synth-small --count 40 --force, then sucrose.exe with SUCROSE_DATA_DIR: 'settings loaded outcome=Loaded' once, 'renderer spawned' zero times"
        status: pass
    human_judgment: false
  - id: D2
    description: "Thumbnails are JPEG and previews are GIF in the sizes of the maintainer's library (about 100 to 640 KB and about 0.2 to 5 MB)"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_pool_files_span_the_real_sizes"
        status: pass
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_pool_plan_targets_follow_the_real_library"
        status: pass
    human_judgment: false
  - id: D3
    description: "The same --out, --count and --seed give the same names and files, a second run is up to date, a new count rewrites, and a non-empty non-synthetic folder is refused without --force"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_second_run_is_up_to_date_and_a_new_count_rewrites"
        status: pass
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_theme_name_is_25_alphanumerics_and_stable"
        status: pass
    human_judgment: false
duration: 12min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 05: Synthetic library generator Summary

**cargo xtask synth-library: synthetic themes with real-sized JPEG thumbnails and GIF previews, two corrupt folders and a prepared Sucrose.json, self-checked with the production library scan, so every Portal run and the 500-theme D-25 measurement use SUCROSE_DATA_DIR alone**

## Performance

- **Duration:** 12 min
- **Started:** 2026-11-24T04:45:00Z
- **Completed:** 2026-11-24T04:57:00Z
- **Tasks:** 2
- **Files modified:** 5 (one new)

## Accomplishments

- One command writes `<out>/Library` (n valid themes, two corrupt folders, one loose `notes.txt`), `<out>/Setting/Sucrose.json` (written with `write_atomic`; Library.Location points at the library, nothing selected, AppVisible false, HintTrayIcon false, DeleteCorrupt false), a `.synth` marker and a `.pool` folder. Theme folders have 25-character names, Type 3 for three of four themes (else Web and Gif alternately) and every 50th theme needs a newer app (99.1.1.0, for LIB-08).
- The generator scans its own output with `sucrose_core::library::scan` and fails unless it finds exactly n themes and 2 corrupt folders.
- The service started with `SUCROSE_DATA_DIR` at a synthetic folder logged "settings loaded outcome=Loaded" once and "renderer spawned" zero times: no migration, nothing plays (D-25, no test-only app flag).
- The pool matches finding F1: 12 JPEG thumbnails of 1280x720 (97 to 596 KB, median about 190 KB) and 8 GIF previews (0.22 to 5.06 MB, median about 1.6 MB), copied into the theme folders round-robin.
- A second run with the same arguments prints "up to date" and touches nothing; a different count rewrites; a non-empty folder without the marker is refused without `--force`.

## 500-theme library (D-25 scenario 2, used by plan 24)

`cargo xtask synth-library --out target/synth-500 --count 500 --force` (debug xtask, WSL calling the Windows toolchain, folder on /mnt/e):

- 500 themes, 2 corrupt, 1 ignored; 503 entries in `Library`.
- 1220.1 MB in total (21 MB of it is the pool), generated in 23.0 s.
- 124 previews are larger than 4 MiB and 63 are smaller than 300 KiB, so the size spread that exercises the UI-07 caps exists.
- Generated data stays under `target/` (gitignored, `git status --porcelain target` is empty).

## Task Commits

1. **Task 1: A synthetic library the service loads as is** - `7158bd54` (feat)
2. **Task 2: Real-sized pool, determinism, up-to-date run** - RED `d6c5930c` (test), GREEN `ea64e4fa` (feat)

**Plan metadata:** added with the close-out commit (docs: complete plan).

## TDD Gate Compliance

Task 2 (tdd="true"): RED `d6c5930c` then GREEN `ea64e4fa`; no refactor commit. RED evidence is the cargo test console output (a format the `check tdd-red-evidence` adapters do not cover, so the semantic assessment is recorded here): 4 of 7 tests failed on the planned assertions, never on a build or setup fault. `synth_pool_plan_targets_follow_the_real_library` failed on pool sizes (1, 1) against (12, 8); `synth_pool_files_span_the_real_sizes` failed on the thumbnail maximum; `synth_marker_matches_only_a_finished_run_with_the_same_count_and_seed` failed on the stub returning false; `synth_second_run_is_up_to_date_and_a_new_count_rewrites` failed with Written instead of UpToDate. The other 3 (theme_name, frames_for, the real data folder check) passed from the start because Task 1 already held that code.

## Files Created/Modified

- `xtask/src/synth_library.rs` - the generator: argument parsing, name generator (xorshift), target-driven JPEG and GIF encoders, library and settings writer, scan self-check, marker and up-to-date logic, 7 tests
- `xtask/src/main.rs` - `synth-library` task and usage line
- `xtask/Cargo.toml` - `image`, `sucrose-config`, `sucrose-core` (workspace)
- `Cargo.toml` - `image 0.25` in the workspace dependencies (jpeg, png, gif, webp, tiff; default features off)
- `Cargo.lock` - the image tree (3 lines: xtask's new dependencies)

## Decisions Made

- A run refuses an `--out` inside the real Sucrose data folder even with `--force` (see deviation 1).
- The marker is written first as incomplete and finished last, so an interrupted run is repeatable and never counted as up to date.
- Image sizes are hit by construction (JPEG: highest quality that fits the target; GIF: frame count from the first frame's encoded size), not by hoping a fixed setting lands near the target; the tests measure the real encoded sizes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Refuse the real data folder as --out**
- **Found during:** Task 1 (threat T-03-11, prohibition "never writes to %APPDATA%\Sucrose")
- **Issue:** The plan's mitigation (marker or --force) still lets `--force` with a mistyped `--out %APPDATA%\Sucrose` remove the real `Library` and `Setting` folders, which the generator replaces on a rewrite.
- **Fix:** `generate` compares the absolute `--out` (case-insensitive, `.` and `..` resolved textually) with `paths::legacy_data_dir()` and stops before writing; unit test `synth_real_data_folder_is_recognised_even_through_dots_and_case`.
- **Files modified:** xtask/src/synth_library.rs
- **Committed in:** 7158bd54, test in d6c5930c

**2. [Rule 3 - Blocking] clippy -D warnings findings in the first version**
- **Found during:** Task 1 verify
- **Issue:** `unusual_byte_groupings`, `chunks_exact_to_as_chunks` and `type_complexity`.
- **Fix:** a grouped hex constant, `as_chunks::<3>`, and a `PoolFiles` struct.
- **Committed in:** 7158bd54

**3. [Plan shape] The RED commit carries a small refactor and stubs**
- `generate` became `generate_with(plan, pool) -> Generated` so the up-to-date test can run with a tiny pool in a temporary folder; `marker_matches` was a stub returning false in the RED commit. The pool plan stayed the Task 1 single small image until GREEN.

---

**Total deviations:** 3 (1 missing critical, 1 blocking, 1 plan-shape note)
**Impact on plan:** No scope creep; the real-data guard is the stated prohibition made enforceable.

## Issues Encountered

- `cargo test -p xtask synth` takes about 24 s because the picture loop runs unoptimized (only dependencies are optimized in the dev profile); the 500-theme run takes 23 s for the same reason plus about 1.2 GB of copying. Acceptable for a tool run rarely.
- The working tree already had uncommitted changes in `apps/sucrose/src/service.rs` and `crates/sucrose-core/src/lib.rs` (line-ending noise, no diff content); they were left untouched and never staged.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Portal plans (08 to 22) can run against `cargo xtask synth-library --out target/<scratch> --count <n>` with `SUCROSE_DATA_DIR` alone; plan 24 can use `target/synth-500` (re-run with `--force` if it was deleted).
- A synthetic folder the service has run in contains `Log/`; the generator leaves `Log/` and `Cache/` alone on a rewrite and only replaces `Library`, `Setting` and `.pool`.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
