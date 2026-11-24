---
phase: 03-m2-service-ipc-and-portal-connected
plan: 02
subsystem: library
tags: [rust, library, scan, sort, search, serde_json, fixtures]
requires:
  - phase: 02-m1-service-and-renderer
    provides: SucroseInfo parser (sucrose_core::theme), LegacyVersion and app_version()
provides:
  - pure read-only library scan with the legacy validity rules (sucrose_core::library::scan)
  - needs-newer-app compatibility, so incompatible themes stay visible
  - path containment for Preview, Thumbnail and Source
  - stable sort, search index and search ordered by matched-word count
  - legacy URL check as code (is_url) and display_title
  - SortMode and SortKind with legacy names and defaults
  - showcase_dir() and properties_cache_dir()
affects: [03-09, 03-10, 03-14, 03-21, 03-22]
actuals:
  tokens: 11000
  tasks: 2
  commits: 6
plan_head_before: afc08a50985511dc02f180f987b7485e80852b40
plan_head_after: be5f464b58c00f0382f46e908bda6572bc69478d
tech-stack:
  added: []
  patterns: [scan returns classified entries and never writes, fixture libraries of empty files plus hand-written SucroseInfo.json, ignored test that reads the real library read-only, explicit folder-name tie-break instead of relying on sort stability]
key-files:
  created:
    - crates/sucrose-core/src/library.rs
    - crates/sucrose-core/tests/library.rs
    - crates/sucrose-core/tests/fixtures/library/
    - crates/sucrose-core/tests/fixtures/library-real/
  modified:
    - crates/sucrose-core/src/lib.rs
    - crates/sucrose-core/src/theme.rs
    - crates/sucrose-core/src/settings.rs
    - crates/sucrose-core/src/paths.rs
    - docs/FEATURES.md
key-decisions:
  - "A folder that cannot be read is Unreadable and outranks every other reason, so a transient error never reads as corrupt"
  - "A PathEscapes reason outranks a plain missing file when several of Preview, Thumbnail and Source are bad"
  - "WallpaperType::from_name_ignore_case is used for SucroseInfo.json only; settings names stay case-sensitive like Enum.Parse"
  - "LibraryTheme.index is built during the scan from title, description, complete localizations and tags"
requirements-completed: [LIB-02, LIB-03, LIB-08]
coverage:
  - id: D1
    description: "The library is scanned read-only with the legacy validity rules; loose files are ignored, other folders are corrupt with a reason"
    requirement: LIB-02
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#fixture_library_has_six_themes_four_corrupt_and_one_ignored"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#corrupt_folders_get_their_reason"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose-core --test library -- --ignored real_library (90 themes, 16 corrupt, 2 ignored)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Preview, Thumbnail and Source that leave the theme folder make the folder corrupt"
    requirement: LIB-02
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#paths_that_leave_the_theme_folder_make_it_corrupt"
        status: pass
    human_judgment: false
  - id: D3
    description: "A theme made for a newer app stays in the list as needing a newer app"
    requirement: LIB-08
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#a_newer_app_version_is_a_theme_not_a_corrupt_folder"
        status: pass
    human_judgment: false
  - id: D4
    description: "Sort by name, creation or modification is stable; search matches index tokens and orders by matched words"
    requirement: LIB-03
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#equal_keys_keep_folder_name_order_in_both_directions"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#themes_matching_more_words_come_first_and_ties_keep_the_order"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/library.rs#the_index_holds_title_description_and_tags_lowercased_without_punctuation"
        status: pass
    human_judgment: false
duration: 10min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 02: Library scan, sort and search Summary

**Pure, read-only library scan in sucrose-core with the legacy validity rules, theme-folder containment, needs-newer-app classification, stable sort and a search index, proven on fixtures and on the real 90-theme library**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-24T04:09:00Z
- **Completed:** 2026-11-24T04:19:00Z
- **Tasks:** 2
- **Files modified:** 42 (including fixtures)

## Accomplishments

- `sucrose_core::library::scan` classifies every folder of the library location as a theme, a theme needing a newer app, or a corrupt folder with a reason; loose files are counted as ignored and nothing is ever written.
- On the maintainer's real library the ignored `real_library` test reports 90 themes, 16 corrupt folders and 2 ignored files, matching finding F1.
- Preview, Thumbnail and Source are refused when absolute, drive or UNC prefixed, containing `..` or resolving outside the canonical theme folder (T-03-04); an unreadable folder is `Unreadable`, never corrupt (T-03-05).
- `SucroseInfo.json` Type names now parse in any letter case like Newtonsoft; ordinals and missing optional keys behave as before.
- Sort (name, creation, modification, both directions, folder-name tie-break), the search index, search ordered by matched words, `display_title`, `is_url`, `SortMode`, `SortKind`, `showcase_dir()` and `properties_cache_dir()` exist for plans 09, 10, 14, 21 and 22.

## Task Commits

1. **Task 1: Scan the real library read-only** - RED `6536fdde` (test), GREEN `3e8ce0e5` (feat)
2. **Task 2: Sort, search index, URL check, SortMode/SortKind, paths** - `17299bc7` (feat: enums and paths), RED `994b0282` (test), GREEN `a351795b` (feat), `be5f464b` (docs: FEATURES.md notes)

**Plan metadata:** added by the following docs commit.

## Files Created/Modified

- `crates/sucrose-core/src/library.rs` - scan, classification, containment, sort, search, index, `is_url`, `display_title`
- `crates/sucrose-core/tests/library.rs` - fixture tests and the ignored `real_library` scan
- `crates/sucrose-core/tests/fixtures/library/` - hand-written fixture library (10 folders, 1 loose file)
- `crates/sucrose-core/tests/fixtures/library-real/` - three sanitized real SucroseInfo.json files (Video, Web, Gif)
- `crates/sucrose-core/src/theme.rs` - `WallpaperType::from_name_ignore_case` used for the Type field
- `crates/sucrose-core/src/settings.rs`, `paths.rs`, `lib.rs` - `SortMode`, `SortKind`, `showcase_dir`, `properties_cache_dir`, module export
- `docs/FEATURES.md` - notes on the two Library rows

## Decisions Made

- Unreadable outranks PathEscapes, which outranks a missing file, when several of the three paths are bad, so the safest reason always wins.
- `LibraryTheme.index` is built during the scan, as the plan's search description says.
- Punctuation removal approximates .NET `\p{P}` with ASCII punctuation minus symbols plus the common Unicode blocks (the standard library has no general category table); documented in the code.
- `is_url` does not accept a trailing line feed, unlike .NET's `$`; a theme with such a Source is stricter than legacy (corrupt, not deleted unless the user turns the option on).

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written. One ordering choice: the `SortMode`/`SortKind` enums and the two path functions were committed first (`17299bc7`) because the Task 2 sort tests need them to compile; the Task 2 RED commit then covers the library functions only. The Task 1 RED commit carries final types and a placeholder `scan` so the tests fail on their assertions instead of on a build error.

## Issues Encountered

- `cargo fmt` rewrote the CRLF working copies of the files it touched to LF; the endings were restored to CRLF after each run (index content is LF either way).
- The measured scan on the real library is 90 themes, 16 corrupt, 2 ignored, as expected from finding F1; no difference to report.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Plans 09, 10, 14, 21 and 22 can call `scan`, `sort`, `search` and `display_title` directly.
- The scan classifies but does not delete or move; moving corrupt folders to the bin stays with the service plans.
- Pre-existing local modifications in `apps/sucrose/src/service.rs` and `crates/sucrose-config/src/model.rs` were left untouched and uncommitted.

## Self-Check: PASSED

- Files found: library.rs, tests/library.rs, fixtures/library, fixtures/library-real.
- Commits found: 6536fdde, 3e8ce0e5, 17299bc7, 994b0282, a351795b, be5f464b.
- `cargo xtask ci` passes (fmt, clippy -D warnings, tests).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
