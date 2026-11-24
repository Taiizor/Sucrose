---
phase: 03-m2-service-ipc-and-portal-connected
plan: 26
subsystem: config
tags: [rust, settings, migration, serde, fixtures, tdd]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: General, Portal, Library and Warehouse sections, MIGRATED-style migration and the real-file fixture convention (plan 04)
provides:
  - Engine (every key), Backgroundog (every non-runtime key plus FullScreenScope), Cycling, Hook and Update sections of Sucrose.json as typed fields with legacy defaults, ranges and enum names
  - sucrose_core::settings::{DisplayScreenType, ScreenModeType, InputType, InputModuleType, EngineType (with GIF/URL/WEB/VIDEO/YOUTUBE/APPLICATION allowed sets), TransitionCycleType, UpdateModuleType, UpdateServerType, UpdateAutoType, UpdateExtensionType, UpdateChannelType, FullScreenScope, PerformanceType}
  - LegacyFile::{Cycling, Hook, Update}, Properties::get_string_map and sucrose_config::migrate::MIGRATED_FILES (the nine files)
  - Real-file fixtures for the five files and Objectionable and Donate sentinels; the D-05 exclusion list in ROADMAP criterion 2, FMT-01 and the FEATURES.md Formats row
affects: [03-07, 03-17, 03-20, 03-21, 03-23]
actuals:
  tokens: 21000
  tasks: 2
  commits: 3
plan_head_before: c56625f3efc47c8501a96d817219015327af3d26
plan_head_after: 9d3c8a168ac30b5c9eff8a0ff08a01c6797221b7
tech-stack:
  added: []
  patterns: [allowed-set validation of an enum per key (EngineType subsets), runtime keys and stale names excluded by never naming them in the per-file key list, JSON-shaped RED tests that fail on assertions before the typed API exists]
key-files:
  created:
    - crates/sucrose-config/tests/fixtures/Engine.real.json
    - crates/sucrose-config/tests/fixtures/Backgroundog.real.json
    - crates/sucrose-config/tests/fixtures/Cycling.real.json
    - crates/sucrose-config/tests/fixtures/Hook.real.json
    - crates/sucrose-config/tests/fixtures/Update.real.json
    - crates/sucrose-config/tests/fixtures/Objectionable.sentinel.json
    - crates/sucrose-config/tests/fixtures/Donate.sentinel.json
  modified:
    - crates/sucrose-core/src/settings.rs
    - crates/sucrose-config/src/legacy.rs
    - crates/sucrose-config/src/model.rs
    - crates/sucrose-config/src/migrate.rs
    - crates/sucrose-config/tests/migrate.rs
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
    - docs/FEATURES.md
key-decisions:
  - "Engine choices (Gif, Url, Web, Video, YouTube, Application) are validated against the allowed engine set of their wallpaper type; anything else is the key's own default, both on migration and when Sucrose.json is read"
  - "DownloadType, UploadType, LimitType and PingType stay validated non-empty strings because the Skylark StorageType value set is unknown"
  - "Skylark enums (DisplayScreenType, ScreenModeType, InputType) are read by name only; the C# enums with known declaration order also accept numeric ordinals"
  - "Backgroundog.FullScreenScope is new and never read from a legacy file; Cycling.PassingTime and the Backgroundog runtime keys are not fields"
requirements-completed: [FMT-01]
coverage:
  - id: D1
    description: "Every key of the Engine, Backgroundog, Cycling, Hook and Update legacy files except the D-05 exclusions is a typed field, proven from real-file fixtures"
    requirement: FMT-01
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#the_real_engine_file_migrates_every_key"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#the_real_backgroundog_file_keeps_its_current_names_and_drops_runtime_and_stale_ones"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#the_real_cycling_hook_and_update_files_migrate"
        status: pass
    human_judgment: false
  - id: D2
    description: "Runtime state, stale names, the GitHub token and Donate never reach Sucrose.json; MIGRATED_FILES lists exactly the nine files; legacy files stay byte-identical on the real install"
    requirement: FMT-01
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#a_token_file_and_a_donate_file_never_reach_the_new_settings"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#migrated_files_lists_exactly_the_nine_files"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose-config --test migrate -- --ignored real_install_migrates (no excluded name in the written Sucrose.json, Backdrop Acrylic, legacy files unchanged)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Allowed engine sets, clamps, exact enum names and stable serialization"
    requirement: FMT-01
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs (engine_choices_*, engine_numbers_*, backgroundog_numbers_*, cycling_hook_and_update_*, the_nine_real_files_serialize_to_stable_text_*)"
        status: pass
    human_judgment: false
  - id: D4
    description: "ROADMAP criterion 2, FMT-01 and the FEATURES.md Formats row list the exclusions"
    requirement: FMT-01
    verification:
      - kind: other
        ref: "grep -c 'documented exclusions' .planning/ROADMAP.md .planning/REQUIREMENTS.md; grep -c TransmissionRequired docs/FEATURES.md"
        status: pass
    human_judgment: false
duration: 9min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 26: Engine, Backgroundog, Cycling, Hook and Update migration Summary

**All nine legacy settings files now migrate key by key into typed Sucrose.json fields (engine choices checked against their wallpaper type's allowed set, new Backgroundog.FullScreenScope), while runtime state, stale Backgroundog names, PassingTime, the GitHub token and Donate never reach the new file.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-11-24T05:16:00Z
- **Completed:** 2026-11-24T05:25:30Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments
- Engine has every key of the legacy table (CefArguments as a sorted string map, WebArguments as a list, six per-type engines, all flags); Backgroundog has the usage limits, the sixteen performance rules with their legacy defaults and FullScreenScope; new Cycling, Hook and Update sections.
- `MIGRATED_FILES` names the nine files and `migrate()` opens only those; the exclusion list with reasons sits next to it in `migrate.rs`.
- Fixtures from the maintainer's real files (personal paths and adapter names replaced) plus token and Donate sentinels; the ignored `real_install_migrates` run now also checks the written Sucrose.json for excluded names.
- ROADMAP criterion 2, FMT-01 and the FEATURES.md Formats row (now in progress) say what is migrated and what is not.

## Task Commits

1. **Task 1 (tracer, TDD): RED** - `a2b4a1d4` (test): fixtures and nine failing tests
2. **Task 1: GREEN** - `d253e1ab` (feat): enums, sections, migration and MIGRATED_FILES
3. **Task 2: exclusion wording** - `9d3c8a16` (docs)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## TDD Gate Compliance

- **RED:** `a2b4a1d4`; nine new tests failed on assertions (a missing key read as JSON Null against the expected value, or a missing section), 19 existing tests passed; the tests build against the old API because they inspect the serialized JSON, so the failure is never a build error. semanticAssessment: the target tests executed and failed on the planned assertions for the intended reason; `cargo test` output has no supported report format for `gsd_run check tdd-red-evidence`, so the verdict is the inspected console output, not the classifier. The sentinel test passed already (it guards a property that holds before the change).
- **GREEN:** `d253e1ab`; 29 migrate tests pass, 22 model/legacy unit tests pass, `cargo xtask ci` passes.
- **REFACTOR:** none.

## Files Created/Modified
- `crates/sucrose-core/src/settings.rs` - the remaining enums, EngineType allowed sets, `PerformanceType` alias
- `crates/sucrose-config/src/model.rs` - Engine and Backgroundog fields, Cycling, Hook and Update sections, lenient readers
- `crates/sucrose-config/src/migrate.rs` - `MIGRATED_FILES`, the five migrations and the exclusion comment
- `crates/sucrose-config/src/legacy.rs` - `LegacyFile::{Cycling, Hook, Update}`, `get_string_map`
- `crates/sucrose-config/tests/migrate.rs` and fixtures - real-file and sentinel tests
- `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `docs/FEATURES.md` - D-05 wording

## Decisions Made
See key-decisions. Field names are Rust snake_case with the legacy key as the serde name; `VirtualPerformance` is `virtual_performance` (`virtual` is a keyword).

## Deviations from Plan

None - plan executed exactly as written. (The Objectionable and Donate sentinels are committed as `*.sentinel.json` fixtures that the test copies under the legacy names, so no token-shaped file sits under a legacy name in the repository.)

## Issues Encountered
None. Working-tree line endings: a helper briefly normalized CRLF files to LF; `legacy.rs`, `model.rs`, `settings.rs` and `docs/FEATURES.md` were restored to CRLF (`git ls-files --eol` checked), the `.planning` files and the two migrate files stay LF. The pre-existing uncommitted change in `crates/sucrose-core/src/lib.rs` was left alone.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 07 can read `Backgroundog.full_screen_scope`; later phases read the Engine, Cycling, Hook and Update fields.
- No blockers.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
