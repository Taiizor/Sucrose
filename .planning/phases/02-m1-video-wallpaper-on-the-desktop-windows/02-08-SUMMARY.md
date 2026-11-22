---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 08
subsystem: theme-format
tags: [sucroseinfo, serde, version, semver, d-13-gate, d-029]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 03 theme model (WallpaperType, read_info); plan 06/07 service that spawns the renderer for the selected theme"
provides:
  - "sucrose_core::version::{LegacyVersion, app_version}: .NET Version semantics (string or object serde, -1 for missing, Ord) and the date-based app version"
  - "sucrose_core::theme::{SucroseInfo (all legacy fields plus unknown-field passthrough), LocalizedText, write_info, check_selected, SelectedTheme, ThemeRejection}"
  - "D-13 gate in the service: a renderer starts only for a theme that passed, every failure has its own logged reason"
  - "Workspace version 26.11.22+1 (D-029) and the AppVersion rule"
affects: [02-09, 02-15]

actuals:
  tokens: 13500
  tasks: 3
  commits: 3

plan_head_before: ecfc4f0086874c2a57ce2da572f9e82676f44e72
plan_head_after: 5d76dd07fc8bf92552913fbcdd87fefcc726610b

tech-stack:
  added: []
  patterns:
    - "Gate reasons are typed values (ThemeRejection) whose Display is one English sentence naming the folder and the problem, never file content"
    - "Cargo version `YY.M.D+R` maps onto four legacy components; build metadata is the revision"

key-files:
  created:
    - crates/sucrose-core/tests/theme.rs
    - crates/sucrose-core/tests/fixtures/SucroseInfo.video.json
    - crates/sucrose-core/tests/fixtures/SucroseInfo.full.json
    - crates/sucrose-core/tests/fixtures/SucroseInfo.object-version.json
    - crates/sucrose-core/src/version.rs
  modified:
    - crates/sucrose-core/src/theme.rs
    - crates/sucrose-core/src/lib.rs
    - apps/sucrose/src/service.rs
    - Cargo.toml
    - Cargo.lock
    - docs/DECISIONS.md
    - docs/FEATURES.md

key-decisions:
  - "D-029: version scheme YY.M.D+R, first value 26.11.22+1 (maintainer choice), maps to legacy 26.11.22.1, same-day rebuilds +2, +3"
  - "A16 tested: CARGO_PKG_VERSION keeps the +R suffix, so app_version() parses it directly; no Cargo.toml read at build time"
  - "An unmappable Cargo version makes app_version() the lowest version, so themes are refused loudly rather than accepted"

requirements-completed: [FMT-02, TYPE-01]

duration: "~25 min (Task 1 and the checkpoint in earlier sessions; this continuation ~4 min)"
completed: 2026-11-22
status: complete

coverage:
  - id: D1
    description: "SucroseInfo.json fixtures parse, serialize and parse back equal; the writer reproduces the real theme file byte for byte (2-space indent, CRLF, no BOM, Type as int, Version as string, explicit nulls); unknown fields survive"
    requirement: FMT-02
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#every_fixture_round_trips_to_an_equal_value"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#write_info_reproduces_the_real_theme_file_byte_for_byte"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#unknown_fields_survive_a_round_trip"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every unplayable-theme case (nothing selected, missing folder, bad info, non-Video type, unsafe Source, newer AppVersion) is its own ThemeRejection and no renderer starts"
    requirement: TYPE-01
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#a_source_that_is_not_a_relative_file_inside_the_theme_is_refused"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#one_more_revision_needs_a_newer_app"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#a_theme_for_exactly_this_app_version_is_accepted"
        status: pass
    human_judgment: false
  - id: D3
    description: "The workspace version is 26.11.22+1, maps to 26.11.22.1 (greater than the last C# release v26.11.22.0) and the maintainer's real selected theme passes the gate"
    requirement: TYPE-01
    verification:
      - kind: unit
        ref: "crates/sucrose-core/src/version.rs#the_app_is_newer_than_the_last_csharp_release"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/src/version.rs#cargo_version_text_keeps_the_build_metadata"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose -- --ignored real_install"
        status: pass
    human_judgment: false
  - id: D4
    description: "D-029 records the one-way version scheme"
    verification: []
    human_judgment: true
    rationale: "Whether the recorded wording matches the maintainer's intent is a reading judgment"
---

# Phase 2 Plan 08: SucroseInfo contract, D-13 gate and date-based version Summary

**SucroseInfo.json is read and written byte-compatibly with fixture proof, the service starts a renderer only for a theme that passes a gate with a distinct reason per failure, and the app now reports the maintainer-chosen date version 26.11.22+1 so themes needing a newer app are refused.**

## Performance

- **Duration:** about 25 min in total across the sessions (this continuation about 4 min)
- **Completed:** 2026-11-22
- **Tasks:** 3 (Task 1 tracer, Task 2 decision checkpoint, Task 3 TDD)
- **Files modified:** 12 (excluding Cargo.lock)

## Accomplishments

- `SucroseInfo` carries every legacy field in order, typed `LegacyVersion`s, `Type` as int or name, explicit nulls and unknown-field passthrough; `write_info` writes 2-space indented CRLF without BOM to the explicit path only.
- `check_selected` returns NothingSelected, FolderMissing, InfoUnreadable, NotVideo, SourceInvalid or NeedsNewerApp; the service logs the reason and waits without a renderer.
- Maintainer chose build-meta `26.11.22+1`; `from_cargo` maps `+R` to the fourth component, `app_version()` reads `CARGO_PKG_VERSION` (A16: the suffix is kept). The real install's selected theme (AppVersion 26.10.19.0) passes.
- D-029 appended to docs/DECISIONS.md; docs/FEATURES.md marks the SucroseInfo row partial.

## Task Commits

1. **Task 1: SucroseInfo round trip and the D-13 gate** - `df8b0b8f` (feat, earlier session)
2. **Task 2: version form decision** - checkpoint, no commit (answer: build-meta 26.11.22+1)
3. **Task 3 RED: version mapping and AppVersion rule tests** - `c691d249` (test; stubs compile, 3 lib tests and 1 gate test failed on their assertions)
4. **Task 3 GREEN: date-based version and AppVersion rule** - `5d76dd07` (feat)

**Plan metadata:** the docs(02-08) commit that adds this file.

## Files Created/Modified

- `crates/sucrose-core/src/version.rs` - LegacyVersion, from_cargo, app_version
- `crates/sucrose-core/src/theme.rs` - SucroseInfo, write_info, check_selected, ThemeRejection
- `crates/sucrose-core/tests/theme.rs` and `tests/fixtures/SucroseInfo.{video,full,object-version}.json` - round-trip and gate tests
- `apps/sucrose/src/service.rs` - gate wired in, `app_version()` passed and logged at start
- `Cargo.toml`, `Cargo.lock` - workspace version 26.11.22+1
- `docs/DECISIONS.md`, `docs/FEATURES.md` - D-029, Formats row

## Decisions Made

- D-029 (see key-decisions); recorded in docs/DECISIONS.md.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Localization written as null when absent (Task 1)**
- Byte-for-byte round trip of the real file requires the key as the C# writer emits it; committed in `df8b0b8f`.

**2. [Rule 2 - Missing Critical] Rejection variants carry the folder name; parse errors use fixed phrases with line and column (Task 1)**
- Reasons must name the theme and must never quote file content (T-02-18); committed in `df8b0b8f`.

**3. [Rule 2 - Missing Critical] A Selected with a separator or `..` is reported as FolderMissing (Task 1)**
- Stops the selection from leaving the library folder (T-02-16); committed in `df8b0b8f`.

**4. [Rule 1 - Cleanup] Old service.rs video_source and safe_relative_source checks and their tests removed (Task 1)**
- Replaced by `check_selected` and tests/theme.rs so the checks have one owner; committed in `df8b0b8f`.

**5. [Rule 2 - Missing Critical] from_cargo refuses pre-release suffixes, leading zeros and non-digit metadata (Task 3)**
- Cargo-valid but unmappable versions must not silently become a wrong legacy version; `5d76dd07`.

**6. [Rule 3 - Blocking] Gate tests use `app_version()` through a helper instead of the fixed 0.1.0 constant (Task 3)**
- The constant would have broken at every version bump; `c691d249`.

---

**Total deviations:** 6 auto-fixed (2 bugs/cleanup, 3 missing critical, 1 blocking). **Impact on plan:** all within the plan's slice; no scope creep.

## Issues Encountered

- `cargo fmt` rewrites touched files as LF; CRLF restored for every edited file. The listed modified files main.rs, player.rs, protocol.rs, os_*.rs and display.rs are pre-existing CRLF/stat artifacts and were not touched or committed.

## Known Stubs

None. (The RED-commit stubs in version.rs were replaced in the GREEN commit.)

## Threat Flags

None. T-02-16 mitigated in `check_selected` (relative regular file, canonical path inside the theme folder); T-02-17 holds because no caller passes a library path to `write_info` and tests write only under temp folders.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 09 and 15 can use `check_selected`, `ThemeRejection` (tooltip text) and `app_version()`.
- The first published version is a one-way door; Phase 5 release tooling must bump `+R` / the date per D-029.

## Self-Check: PASSED

- Files present: version.rs, theme.rs, tests/theme.rs, three fixtures, D-029 (1 match), no `version = "0.1.0"` left in Cargo.toml.
- Commits present: df8b0b8f, c691d249, 5d76dd07.
- `cargo xtask ci` exit 0; `cargo test -p sucrose -- --ignored real_install` passes.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
