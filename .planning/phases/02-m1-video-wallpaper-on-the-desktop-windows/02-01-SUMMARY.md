---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 01
subsystem: engine
tags: [libmpv, bindgen, libloading, xtask, sha256, 7z, ffi]

requires:
  - phase: 01-m0-workspace-skeleton
    provides: Cargo workspace, xtask runner, unsafe-isolation lint policy
provides:
  - "cargo xtask fetch-engines: pinned, SHA256-verified libmpv archives placed next to the dev exe"
  - "cargo xtask bindgen-mpv and the committed dynamic-loading bindings (sys::Mpv) for client API 2.6"
  - "sucrose-engine-mpv crate with loader::{LIBRARY_NAMES, candidates, locate, load, load_default, Mpv, LoadError}"
  - "apps/sucrose tracked by git (gitignore re-include)"
affects: [02-02, 02-07, 02-09, 02-11, 02-14]

plan_head_before: 8cc3075f3abba6254948e95f4c46936f00ee5f8c
plan_head_after: 855e989dda065c20cb51886c26291efbfd6f4b8f

actuals:
  tokens: 18400
  tasks: 3
  commits: 4

tech-stack:
  added: [sevenz-rust2 0.25.0, sha2 0.11.1, ureq 3.5.2, toml 1.2.2, serde 1.0.229, bindgen 0.73.2 (optional xtask feature), libloading 0.9.1]
  patterns:
    - "Pins in xtask/engines.toml (compiled in with include_str), verified before the archive is opened"
    - "Generated FFI bindings are committed; bindgen sits behind an off-by-default xtask feature so CI never needs libclang"
    - "Loader failures are typed LoadError values with a user-facing hint(), never panics"

key-files:
  created:
    - xtask/engines.toml
    - xtask/src/fetch_engines.rs
    - xtask/src/bindgen_mpv.rs
    - crates/sucrose-engine-mpv/Cargo.toml
    - crates/sucrose-engine-mpv/src/lib.rs
    - crates/sucrose-engine-mpv/src/sys.rs
    - crates/sucrose-engine-mpv/src/loader.rs
    - crates/sucrose-engine-mpv/tests/load.rs
  modified:
    - .gitignore
    - Cargo.toml
    - Cargo.lock
    - xtask/Cargo.toml
    - xtask/src/main.rs

key-decisions:
  - "Loader searches the exe folder, then absolute PATH entries only (D-018 narrowed against DLL planting; plan 02 records it in D-028)"
  - "sys.rs is generated once in the rust Docker image (clang + libclang-dev) with wrap_unsafe_ops and without C doc comments, and committed"
  - "vulkan-1.dll is a normal (not delay-load) import of libmpv-2.dll, so a PC without the Vulkan loader gets LoadError::LoadFailed"

patterns-established:
  - "xtask::target_dir()/workspace_root() shared helpers honor CARGO_TARGET_DIR"
  - "Ignored integration tests load the real fetched DLL from target/engines/<arch>"

requirements-completed: [ENG-01]

coverage:
  - id: D1
    description: "cargo xtask fetch-engines downloads the pinned shinchiro 20261111 x86_64 archive, verifies SHA256, extracts libmpv-2.dll and client.h, and places the DLL in target/debug"
    requirement: ENG-01
    verification:
      - kind: integration
        ref: "cargo xtask fetch-engines && test -f target/engines/x86_64/libmpv-2.dll && test -f target/debug/libmpv-2.dll"
        status: pass
      - kind: unit
        ref: "xtask/src/fetch_engines.rs#extraction_writes_only_the_two_named_entries_to_fixed_paths"
        status: pass
    human_judgment: false
  - id: D2
    description: "An archive whose SHA256 does not match the pin is deleted and nothing is extracted or placed"
    requirement: ENG-01
    verification:
      - kind: unit
        ref: "xtask/src/fetch_engines.rs#a_mismatching_archive_is_deleted_and_never_extracted"
        status: pass
      - kind: integration
        ref: "xtask.exe fetch-engines --from <junk file> with an empty CARGO_TARGET_DIR: exit 1, cache folder left empty"
        status: pass
    human_judgment: false
  - id: D3
    description: "Committed bindgen bindings for the pinned client.h, regenerable with one documented command"
    requirement: ENG-01
    verification:
      - kind: other
        ref: "cargo clippy -p sucrose-engine-mpv --all-targets -- -D warnings (Windows and Linux image)"
        status: pass
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/load.rs#loads_the_fetched_libmpv_by_absolute_path"
        status: pass
    human_judgment: false
  - id: D4
    description: "Loader follows the D-018 search order, loads by absolute path, checks API major 2, and reports every failure as a LoadError with a hint"
    requirement: ENG-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/loader.rs tests (10 tests)"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose-engine-mpv -- --ignored"
        status: pass
    human_judgment: false
  - id: D5
    description: "apps/sucrose (the service binary) is tracked by git so CI builds it"
    verification:
      - kind: other
        ref: "git ls-files --error-unmatch apps/sucrose/Cargo.toml apps/sucrose/src/main.rs"
        status: pass
    human_judgment: false

duration: 32min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 01: libmpv delivery and loader Summary

**Pinned, SHA256-verified libmpv-2.dll fetched by cargo xtask fetch-engines, committed bindgen bindings for client API 2.6, and a sucrose-engine-mpv loader that searches only the exe folder and absolute PATH entries and reports failures as typed errors**

## Performance

- **Duration:** 32 min
- **Started:** 2026-11-22T12:42:05Z
- **Completed:** 2026-11-22T13:14:00Z
- **Tasks:** 3 (task 3 as a TDD pair, so 4 commits)
- **Files modified:** 15 (8 created, 7 modified incl. Cargo.lock)

## Accomplishments

- Tracer verified end to end: `cargo xtask fetch-engines` downloaded the 30 MiB x86_64 archive, matched the pinned hash 273c079b..., extracted `libmpv-2.dll` and `client.h` to fixed paths and placed the DLL in `target/debug`; a second run reports "up to date" from the `.verified` marker.
- A tampered archive is refused before it is opened as 7z: the bad file is deleted, nothing is extracted or placed (also exercised end to end with a junk `--from` file in a scratch target folder).
- `sucrose-engine-mpv` loads the fetched DLL by absolute path through our own bindings and sees client API 2.6 (ignored test passes).
- Loader tests (10) cover search order, PATH filtering (relative, empty, duplicate), `NotAbsolute`, a non-library file and the `NotFound` listing and hint. The crate also passes clippy and tests inside the Linux `rust:1.99` image, so the non-Windows `open` path compiles.
- `apps/sucrose` (ignored by the `Sucrose/` rule on a case-insensitive repo) is now tracked.

## Task Commits

1. **Task 1: fetch-engines (tracer)** - `053e93e3` (feat)
2. **Task 2: bindgen bindings and sucrose-engine-mpv crate** - `1acf1df1` (feat)
3. **Task 3: loader, RED** - `1465f2f9` (test); **GREEN** - `855e989d` (feat)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `xtask/engines.toml` - pins for x86_64, i686, aarch64 (url, sha256, size, provenance)
- `xtask/src/fetch_engines.rs` - download, verify, extract, place; 11 unit tests
- `xtask/src/bindgen_mpv.rs` - regenerates sys.rs (feature `bindgen`)
- `xtask/src/main.rs` - task dispatch plus shared workspace_root/target_dir helpers
- `crates/sucrose-engine-mpv/src/{lib,sys,loader}.rs`, `tests/load.rs` - the engine crate
- `.gitignore`, `Cargo.toml`, `Cargo.lock`, `xtask/Cargo.toml` - re-include rule and dependencies

## Decisions Made

- Loader search is exe folder first, then absolute PATH entries; relative, empty and duplicate entries and the current folder are never searched (T-02-02). Plan 02 writes this narrowing of D-018 into D-028.
- If the first existing candidate fails to load, its error is returned instead of falling through to the next candidate (an install problem should be visible, not masked).
- `vulkan-1.dll` is a regular import of `libmpv-2.dll` (objdump: it is in the normal import table and the Delay Import Directory is empty). A PC without a Vulkan loader therefore gets `LoadError::LoadFailed` instead of a working libmpv; this answers RESEARCH Open Question 5 and is for plan 02 to put into D-028.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] clang package added to the bindgen Docker run**
- **Found during:** Task 2
- **Issue:** with only `libclang-dev` bindgen failed with "stddef.h file not found" (the compiler resource headers come with `clang`).
- **Fix:** the container installs `libclang-dev clang`. The container used the Linux docker CLI, so `$PWD` was mounted directly (no wslpath).
- **Files modified:** none (command only)
- **Committed in:** 1acf1df1

**2. [Rule 3 - Blocking] bindgen options wrap_unsafe_ops and generate_comments(false)**
- **Found during:** Task 2 and Task 3
- **Issue:** edition 2024 denies `unsafe_op_in_unsafe_fn` (38 errors in generated wrappers), and the indented C examples in the generated doc comments ran as Rust doctests and failed `cargo test`.
- **Fix:** `.wrap_unsafe_ops(true)` and `.generate_comments(false)`; sys.rs regenerated (318 lines).
- **Files modified:** xtask/src/bindgen_mpv.rs, crates/sucrose-engine-mpv/src/sys.rs
- **Committed in:** 1acf1df1 (wrap_unsafe_ops), 855e989d (comments off, regenerated)

**2b. [Rule 2 - Missing critical] Loader reads the version before building the bindings struct**
- **Found during:** Task 3
- **Issue:** the plan maps a missing symbol in `from_library` to `ApiTooOld`, which loses the real cause and the reported version.
- **Fix:** `mpv_client_api_version` is looked up first (missing, or major other than 2, gives `ApiTooOld`); a later `from_library` failure is `LoadFailed` whose message names the missing symbol.
- **Files modified:** crates/sucrose-engine-mpv/src/loader.rs
- **Committed in:** 855e989d

**3. [Rule 3 - Blocking] Line endings kept as CRLF**
- The plan asked for LF in .gitignore, but the working tree files are CRLF (git normalizes with text=auto, core.autocrlf=true); the existing endings were preserved and new files written as CRLF to match.

---

**Total deviations:** 3 auto-fixed (all Rule 3 or 2, none architectural)
**Impact on plan:** No scope change; every acceptance criterion holds.

## TDD Notes (Task 3)

- RED (`1465f2f9`): stubs for candidates/locate/resolve/load; 7 of 10 loader tests failed on their planned assertions (empty candidate lists, `None` from locate, NotAbsolute instead of LoadFailed, empty NotFound listing); the 3 that passed (relative path rejected, locate with nothing, hints) are true for the stubs by design. Semantic assessment: every failure is on the behavior under test, none from compile, discovery or fixture errors.
- The RED evidence classifier (`gsd_run check tdd-red-evidence`) has no adapter for cargo's libtest output, so RED evidence is the captured test output above, not a classifier verdict.
- GREEN (`855e989d`): all 10 pass, plus the ignored load test; no refactor commit.

## Issues Encountered

- i686 build check was not run locally (`rustup target list --installed` lists only x86_64-pc-windows-msvc); CI covers it. Flagged assumption A13 (MinGW i686 struct alignment) is unchanged: the code reads mpv structs only through pointers libmpv returns, and nothing reads them yet.
- Windows locks a running `sucrose.exe`'s copy of the DLL; `fetch-engines` reports that case in its error text.

## Known Stubs

None.

## Threat Flags

None. The new network surface (HTTPS download in xtask) is the planned T-02-01 boundary, mitigated by the pre-extraction hash check.

## User Setup Required

None - no external service configuration required. Docker Desktop is only needed to regenerate the bindings (`cargo xtask bindgen-mpv`), not to build.

## Next Phase Readiness

- Plan 02 can record the D-028 narrowing, the objdump finding and the architecture status line; plans 07, 09, 11 and 14 can call `loader::load_default`.
- Plan 02 may point `url` at a mirror (the upstream release disappears after a few weeks; the hash stays).

## Self-Check: PASSED

All created files exist and the four commits (053e93e3, 1acf1df1, 1465f2f9, 855e989d) are ancestors of HEAD.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
