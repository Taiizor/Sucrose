---
phase: 03-m2-service-ipc-and-portal-connected
plan: 04
subsystem: config
tags: [rust, settings, migration, serde, fixtures, theme]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Sucrose.json model, store and the legacy reader (plan 01), SortMode and SortKind enums (plan 02)
provides:
  - General, Portal, Library and Warehouse sections of Sucrose.json as typed fields with legacy defaults, ranges and enum names
  - sucrose_core::settings::{ThemeMode, Backdrop, LegacyThemeType, LegacyBackdropType}
  - LegacyFile::{Portal, Warehouse} and Properties::{get_enum_opt, get_string_list}
  - sucrose_config::migrate::map_theme (pure D-21 mapping) and the four-file migration
  - Sanitized real-file fixtures and tests/migrate.rs with the ignored real_install_migrates
affects: [03-05, 03-06, 03-17, 03-20, 03-21, 03-23, 03-26]
actuals:
  tokens: 14000
  tasks: 2
  commits: 3
plan_head_before: 0bd81e5ad5e2645521ca969d4fbc55c9c54f3747
plan_head_after: 16e06db5bc9bd005db08a3aa3024e3612fc85f1e
tech-stack:
  added: []
  patterns: [one pure mapping function for a legacy value split into two settings, raw strings for legacy DateTime values, get_enum_opt for values that are only mapping inputs]
key-files:
  created:
    - crates/sucrose-config/tests/migrate.rs
    - crates/sucrose-config/tests/fixtures/General.real.json
    - crates/sucrose-config/tests/fixtures/Portal.real.json
    - crates/sucrose-config/tests/fixtures/Library.real.json
    - crates/sucrose-config/tests/fixtures/Warehouse.real.json
  modified:
    - crates/sucrose-core/src/settings.rs
    - crates/sucrose-config/src/legacy.rs
    - crates/sucrose-config/src/model.rs
    - crates/sucrose-config/src/migrate.rs
    - .planning/REQUIREMENTS.md
    - docs/FEATURES.md
key-decisions:
  - "BackdropType Auto always gives theme mode System plus Mica, whatever ThemeType says; nothing stored gives System plus None"
  - "Library.Move is the Rust field move_library (move is a keyword) and keeps the JSON key Move"
  - "General.Culture is None (follow the OS language) when absent or empty, and is not written when None"
  - "Warehouse time strings default to the legacy DateTime.MinValue text 0001-01-01T00:00:00 and are never parsed"
  - "Portal.StoreServerType is not migrated (D-05)"
requirements-completed: [FMT-01, FMT-06, LIB-01]
coverage:
  - id: D1
    description: "Every General, Portal, Library and Warehouse key migrates into a typed field, proven by fixtures from the real files (sanitized)"
    requirement: FMT-01
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#the_real_portal_look_and_library_state_migrate"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#library_and_warehouse_keep_their_values"
        status: pass
    human_judgment: false
  - id: D2
    description: "ThemeType and BackdropType become theme mode and backdrop (D-21); the maintainer's files give Dark + Acrylic"
    requirement: FMT-01
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs#map_theme_follows_d21"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose-config --test migrate -- --ignored real_install_migrates (prints ThemeMode = Dark, Backdrop = Acrylic; legacy files unchanged)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Empty, missing, BOM, wrapper-less and briefly empty legacy files, clamps, exact enum names and non-ASCII strings"
    requirement: FMT-06
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/migrate.rs (empty, bom, retry, clamp, enum-case and missing-folder tests)"
        status: pass
    human_judgment: false
  - id: D4
    description: "LIB-01 and the FEATURES.md Showcase and Window backdrop rows describe the new settings tracking and the theme split"
    requirement: LIB-01
    verification:
      - kind: other
        ref: "grep -c 'tracked in the new settings' .planning/REQUIREMENTS.md docs/FEATURES.md; grep -c -F 'System + Mica' docs/FEATURES.md"
        status: pass
    human_judgment: false
duration: 8min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 04: General, Portal, Library and Warehouse migration Summary

**Every key of the General, Portal, Library and Warehouse legacy files is now a typed field of Sucrose.json, and ThemeType plus BackdropType become the D-21 theme mode and backdrop pair (the maintainer's files migrate to Dark + Acrylic).**

## Performance

- **Duration:** 8 min
- **Started:** 2026-11-24T04:35:00Z
- **Completed:** 2026-11-24T04:43:00Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- The General (7 keys), Portal (15 stored keys plus the new ThemeMode and Backdrop), Library (5) and Warehouse (6) sections have legacy defaults, clamps and exact, case-sensitive enum names; unknown keys survive a round trip.
- `map_theme` is the single pure D-21 mapping: Auto gives System + Mica, the other backdrops map one to one, Dark/Light give the mode, nothing stored gives System + None.
- Fixtures from the real files (paths and the selected theme replaced) plus 18 integration tests cover the behaviors of the plan; the ignored run against the real install printed `ThemeMode = Dark, Backdrop = Acrylic`, 20 showcase names, `HintTrayIcon = false`, and left every legacy file byte- and mtime-identical.
- LIB-01 and two FEATURES.md rows now describe what is built.

## Task Commits

1. **Task 1 (tracer, TDD):**
   - RED `760d90b5` (test): fixtures, tests/migrate.rs and a compiling skeleton; 14 of 18 tests failed on assertions for the planned behavior (for example theme mode System instead of Dark, clamps not applied, showcase empty).
   - GREEN `b064f7a2` (feat): reader helpers, section parsing and the migration.
2. **Task 2: LIB-01 and FEATURES.md rows** - `16e06db5` (docs)

**Plan metadata:** the docs(03-04) commit that follows this summary.

## Files Created/Modified
- `crates/sucrose-core/src/settings.rs` - ThemeMode, Backdrop, LegacyThemeType, LegacyBackdropType and their ordinals
- `crates/sucrose-config/src/legacy.rs` - LegacyFile::{Portal, Warehouse}, get_enum_opt, get_string_list, public section constructors
- `crates/sucrose-config/src/model.rs` - the Portal and Warehouse sections and the extended General and Library
- `crates/sucrose-config/src/migrate.rs` - four-file migration and map_theme
- `crates/sucrose-config/tests/migrate.rs` and `tests/fixtures/*.real.json` - fixture tests
- `.planning/REQUIREMENTS.md`, `docs/FEATURES.md` - LIB-01, Showcase and Window backdrop rows

## Decisions Made
See key-decisions. No legacy file name exists for the token file (D-05); the Objectionable.json name does not appear in legacy.rs.

## Deviations from Plan

None - plan executed exactly as written. Notes: the cargo test output is not a format `gsd_run check tdd-red-evidence` supports and `workflow.tdd_mode` is off, so the RED evidence is the observed assertion failures above (semantic assessment: the target tests ran and failed on the planned behavior, not on build or fixture errors).

## Issues Encountered
- `git status` listed `apps/sucrose/src/service.rs` and `crates/sucrose-core/src/lib.rs` as modified before and after this plan with an empty diff (a CRLF stat artifact, per CLAUDE.md); they were not touched or committed.
- `cargo fmt` writes the two new Rust files with LF in the working copy; the index is LF either way.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Threat Flags

None - T-03-09 holds (no LegacyFile variant or name for the token file, grep count 0) and T-03-10 is covered by typed getters and the pure mapping; no crate was added.

## Next Phase Readiness
- Plans 05, 06, 17, 20, 21, 23 can read `settings.portal`, `settings.general`, `settings.library` and `settings.warehouse`; plan 26 adds the Engine, Backgroundog, Cycling, Hook and Update sections and the D-05 wording of FMT-01.

## Self-Check: PASSED

- Files found: migrate.rs test, four fixtures, SUMMARY target; commits 760d90b5, b064f7a2, 16e06db5 are ancestors of HEAD; `cargo xtask ci` exits 0.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
