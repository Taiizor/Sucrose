---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 03
subsystem: config
tags: [rust, serde_json, legacy-settings, theme, read-only, tracing]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: libmpv loader and engine delivery decision (plans 01-02, no code dependency used here)
provides:
  - "sucrose-config crate with the read-only legacy Setting/*.json reader (LegacyFile, Properties, read, LibrarySettings)"
  - "sucrose_core::paths::{legacy_data_dir, legacy_setting_dir, legacy_mpv_cache_dir} that ignore SUCROSE_DATA_DIR"
  - "sucrose_core::theme::{WallpaperType, SucroseInfo, ThemeError, read_info}"
  - "Service start logs the theme selected in the C# install"
affects: [02-04 legacy settings snapshot, 02-06 selected-theme gate, 02-08 theme round trip, 02-11 mpv cache, Phase 3 migration FMT-01]

plan_head_before: 15d8eafe8a69dff446183a35465c22d9eac35779
plan_head_after: ce539de7bf611400a32b5feda7874cb23131395a
actuals:
  tokens: 6202
  tasks: 1
  commits: 1

tech-stack:
  added: [serde_json 1.0.154 (workspace), serde (sucrose-core), thiserror (sucrose-core, sucrose-config)]
  patterns:
    - "Legacy files are only opened with std::fs::read; no write API exists in sucrose-config"
    - "Any legacy read failure returns empty Properties plus a tracing warning; locked or empty files are retried 3 times"
    - "Fixtures under crates/*/tests/fixtures, resolved with CARGO_MANIFEST_DIR"

key-files:
  created:
    - crates/sucrose-config/Cargo.toml
    - crates/sucrose-config/src/lib.rs
    - crates/sucrose-config/src/legacy.rs
    - crates/sucrose-config/tests/fixtures/Library.json
    - crates/sucrose-config/tests/fixtures/General.json
    - crates/sucrose-core/src/theme.rs
    - crates/sucrose-core/tests/fixtures/video_theme/SucroseInfo.json
    - crates/sucrose-core/tests/fixtures/broken_theme/SucroseInfo.json
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-core/Cargo.toml
    - crates/sucrose-core/src/lib.rs
    - crates/sucrose-core/src/paths.rs
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/main.rs

key-decisions:
  - "Locked or empty legacy files are retried 3 times at 50 ms before falling back to defaults, because the C# writer truncates in place (RESEARCH Pattern 7); a missing file is not retried"
  - "SucroseInfo keeps AppVersion as serde_json::Value and ignores unknown fields until plan 08 types and round-trips it"

requirements-completed: [FMT-08, TYPE-01]

coverage:
  - id: D1
    description: "Service start reads the real Library.json and the theme's SucroseInfo.json and logs 'selected theme' with kind=Video and the source file"
    requirement: TYPE-01
    verification:
      - kind: e2e
        ref: "cargo build -p sucrose && target/debug/sucrose.exe with SUCROSE_DATA_DIR scratch; log line 'selected theme theme=PEDoEdU... kind=Video source=s_cyber-samurai-princess-live-wallpaper.mp4'"
        status: pass
    human_judgment: false
  - id: D2
    description: "Legacy setting files are only read: Library.json size, mtime and SHA-256 are identical before and after a run; no write API in sucrose-config"
    requirement: FMT-08
    verification:
      - kind: e2e
        ref: "stat and sha256sum of %APPDATA%/Sucrose/Setting/Library.json before and after the run"
        status: pass
      - kind: other
        ref: "grep for OpenOptions/File::create/fs::write/remove_/rename in crates/sucrose-config/src finds only a doc comment"
        status: pass
    human_judgment: false
  - id: D3
    description: "Missing selection, missing or invalid files and bad SucroseInfo.json do not panic and produce defaults, a warning or a typed ThemeError"
    requirement: FMT-08
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-config -p sucrose-core (legacy::tests, theme::tests)"
        status: pass
    human_judgment: false

duration: 5 min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 03: Selected theme from the C# install Summary

**Read-only legacy Library.json reader (new sucrose-config crate) plus a SucroseInfo.json reader in sucrose-core, wired so the service logs the maintainer's selected Video theme at start**

## Performance

- **Duration:** 5 min
- **Started:** 2026-11-22T13:32:20Z
- **Completed:** 2026-11-22T13:36:54Z
- **Tasks:** 1 (tracer)
- **Files modified:** 15 (8 created, 7 modified)

## Accomplishments

- `sucrose-config` exists with `LegacyFile`, `Properties::get_str`, `read` and `LibrarySettings::from_properties`; it tolerates a BOM, retries a locked or empty file and returns defaults with a warning on any failure.
- `sucrose-core` gained `legacy_data_dir`, `legacy_setting_dir` and `legacy_mpv_cache_dir` (they ignore `SUCROSE_DATA_DIR`, D-08) and the `theme` module (`WallpaperType`, `SucroseInfo`, `ThemeError`, `read_info`) with int-or-name `Type` parsing.
- Real run on the maintainer's machine logged `selected theme theme=PEDoEdUHdEKlmxJFYH2Q1zk7T kind=Video source=s_cyber-samurai-princess-live-wallpaper.mp4`; the real `Library.json` (size, mtime 1795301179, SHA-256) was unchanged.
- `cargo xtask ci` (fmt, clippy -D warnings, tests) exits 0; clippy for the three touched packages is clean.

## Task Commits

1. **Task 1: Service start logs the theme selected in the C# install** - `ce539de7` (feat)

**Plan metadata:** recorded in the docs commit that follows this summary.

_Note: an unrelated maintainer commit (`8a6b0751`, planning config only) landed on the branch after the task commit; `commits` and `plan_head_after` are measured up to the task commit `ce539de7`._

## Files Created/Modified

- `crates/sucrose-config/src/legacy.rs` - read-only legacy reader and `LibrarySettings`
- `crates/sucrose-core/src/theme.rs` - wallpaper type and `SucroseInfo.json` reader
- `crates/sucrose-core/src/paths.rs` - `legacy_*` folders
- `apps/sucrose/src/main.rs` - `log_selected_theme` at service start
- `Cargo.toml`, `Cargo.lock`, crate manifests - `serde_json` and `sucrose-config` wiring
- fixtures under `crates/sucrose-config/tests/fixtures` and `crates/sucrose-core/tests/fixtures` (synthetic, no personal data)

## Decisions Made

- Retry locked or empty legacy files 3 times (50 ms apart) before defaults; the plan text only said "warn and return empty", RESEARCH Pattern 7 asked for the retry because the C# writer is not atomic.
- `selected` is joined to `Location` as is and only logged; the `..`/absolute-path gate is plan 06's job (D-13).

## Deviations from Plan

None - plan executed exactly as written.

Note (not a deviation): `cargo fmt` on Windows rewrote `lib.rs`, `paths.rs` and `legacy.rs` to LF; they were restored to CRLF to match the working tree. The git index stores LF either way.

## Issues Encountered

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 04 can add the typed getters on `Properties`, `sucrose_core::settings` and `LegacySettings::load` on this reader.
- Plan 06 can gate playback on `read_info` (Type, Source containment, AppVersion).
- FMT-08 and TYPE-01 stay open: the full D-09 key set (plan 04) and the remaining theme work (plans 06, 08) are not done yet.

## Self-Check: PASSED

- FOUND: crates/sucrose-config/src/legacy.rs, crates/sucrose-core/src/theme.rs, apps/sucrose/src/main.rs
- FOUND: commit ce539de7 (ancestor of HEAD)

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
