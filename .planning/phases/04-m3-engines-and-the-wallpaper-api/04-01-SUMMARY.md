---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 01
subsystem: engines
tags: [cef, cef-dll-sys, delay-load, fetch-engines, tar-bz2, synth-library, web-fixtures]

requires:
  - phase: 03-m2-ui-and-library
    provides: libmpv fetch-engines pins, apps/sucrose service and renderer, synth-library
provides:
  - "CEF 156.0.31 pinned (sha256 + upstream sha1) for x86_64, i686 and aarch64 and extracted by cargo xtask fetch-engines --engine cef into target/engines/cef/<ver>/<os_arch>/"
  - "CEF_PATH in .cargo/config.toml: every build reads the pinned dist, no build script downloads CEF"
  - "sucrose-engine-cef crate (is_cef_subprocess, run_if_subprocess, CEF_VERSION); libcef.dll delay-loaded by sucrose.exe"
  - "D-32 gate first half proven: the service and an mpv renderer run with no libcef.dll mapped"
  - "cargo xtask synth-library --web-fixtures --select <folder> and the fixture Web theme hello"
affects: [04-02, 04-05, 04-12, 04-13, phase-04-round-B]

actuals:
  tokens: 16700
  tasks: 3
  commits: 3
plan_head_before: d54b6dfa1847dbb30704ef9ad01d4805e8c5788a
plan_head_after: 95247e34c20c5c181ef7f95e0886737a268337cc
commits: 3

tech-stack:
  added: [cef 156.7.0 (cef-dll-sys, download-cef as its build dependency), tar 0.4.46, bzip2 0.6.1]
  patterns:
    - "Link args for delay-loading come from each package's own build.rs with cargo::rustc-link-arg (applies to its test executables too)"
    - "Archive extraction goes to a staging folder renamed into place only after the whole archive passed the safety checks"

key-files:
  created:
    - crates/sucrose-engine-cef/Cargo.toml
    - crates/sucrose-engine-cef/build.rs
    - crates/sucrose-engine-cef/src/lib.rs
    - apps/sucrose/build.rs
    - apps/sucrose/tests/fixtures/web-themes/hello/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/hello/index.html
  modified:
    - xtask/engines.toml
    - xtask/src/fetch_engines.rs
    - xtask/src/main.rs
    - xtask/src/synth_library.rs
    - xtask/Cargo.toml
    - Cargo.toml
    - Cargo.lock
    - .cargo/config.toml
    - .github/workflows/rust.yml
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/main.rs
    - docs/ARCHITECTURE.md

key-decisions:
  - "D-32 holds: delay-loading libcef.dll works, so the sucrose-cef.exe fallback is not needed"
  - "Layout of the CEF dist is rebuilt like download-cef does (Release and Resources merged into the root) so cef-dll-sys accepts it; the .verified marker sits beside the dist folder, not inside it"
  - "x86 (i686) is left out of the cef cfg: the i686 Rust target is not installed, so it could not be built (A3); 04-12 reports CEF unsupported on x86"

patterns-established:
  - "Pins carry per-engine fields; parse_manifest validates per engine name and one row per (name, arch)"
  - "Anything refusable (select, fixtures, guard) is refused before the first write in synth-library"

requirements-completed: []

coverage:
  - id: D1
    description: "cargo xtask fetch-engines --engine cef downloads the pinned CEF, refuses it unless the SHA256 matches, and extracts the dist where cef-dll-sys reads it (safe tar.bz2 extraction, archive.json)"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "xtask/src/fetch_engines.rs#cef_extraction_builds_the_layout_download_cef_makes"
        status: pass
      - kind: unit
        ref: "xtask/src/fetch_engines.rs#cef_extraction_refuses_unsafe_entries_and_writes_nothing"
        status: pass
      - kind: integration
        ref: "cargo xtask fetch-engines --engine cef && cargo build -p sucrose-engine-cef (no .tar.bz2 in any cef-dll-sys build folder)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every build reads CEF_PATH from .cargo/config.toml and cargo xtask ci refuses to run before the CEF dist is fetched"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "xtask/src/fetch_engines.rs#ci_knows_whether_the_cef_dist_of_the_host_is_fetched"
        status: pass
      - kind: integration
        ref: "cargo xtask ci (fmt, clippy, test) passed with the dist fetched"
        status: pass
    human_judgment: false
  - id: D3
    description: "sucrose.exe delay-loads libcef.dll: the service and an mpv renderer of the same build run with no libcef.dll mapped"
    requirement: ENG-03
    verification:
      - kind: integration
        ref: "tasklist /m libcef.dll /fi 'imagename eq sucrose.exe' lists none (service 25676, renderer 24580 hold libmpv-2.dll only)"
        status: pass
    human_judgment: false
  - id: D4
    description: "A command line with CEF's --type= switch goes to CEF's subprocess entry before any Sucrose code; every other command line still goes through the strict parser"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/main.rs#a_cef_subprocess_command_line_never_reaches_the_sucrose_parser"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-cef/src/lib.rs#sucrose_command_lines_are_not_cef_subprocesses"
        status: pass
      - kind: integration
        ref: "sucrose.exe --type=gpu-process --lang=en-US exits (code 3) and creates no data folder or log"
        status: pass
    human_judgment: false
  - id: D5
    description: "cargo xtask synth-library --web-fixtures --select <folder> writes a scratch library with the fixture Web themes and selects one"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_web_fixtures_are_copied_next_to_the_synthetic_themes_and_selected"
        status: pass
      - kind: integration
        ref: "cargo xtask synth-library --out target/web-lib --count 2 --web-fixtures --select hello --force; Selected is hello"
        status: pass
    human_judgment: false
  - id: D6
    description: "The CI workflow fetches CEF and checks CMake and Ninja on the Windows runners"
    requirement: ENG-03
    verification: []
    human_judgment: true
    rationale: "A workflow can only be seen running after the maintainer pushes; nothing here executed it"

duration: 32min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 01: Pinned CEF and the Delay-Load Gate Summary

**CEF 156.0.31 is pinned, hash-checked and extracted by xtask into the dist cef-dll-sys reads through CEF_PATH; sucrose.exe delay-loads libcef.dll and hands --type= command lines to CEF first, and the service and mpv renderer run without the DLL mapped (D-32 holds)**

## Performance

- **Duration:** 32 min
- **Started:** 2026-11-26T00:36:00Z
- **Completed:** 2026-11-26T01:08:00Z
- **Tasks:** 3
- **Files modified:** 18 (6 created)

## Accomplishments

- `cargo xtask fetch-engines [--engine libmpv|cef|all] [--arch ...] [--from <archive>] [--print-sha256]`: the Pin type holds libmpv and cef rows (validated per engine, one row per (name, arch)); CEF tar.bz2 is checked by SHA256 before it is opened, extracted into a staging folder (absolute, `..`, drive, link, device and outside-top-folder entries refuse the archive and leave nothing) and renamed to `target/engines/cef/156.0.31/cef_windows_<arch>/` with the `archive.json` download-cef writes.
- `.cargo/config.toml` sets `CEF_PATH`; `cargo build -p sucrose-engine-cef` compiled the CEF C++ wrapper with CMake and Ninja and downloaded nothing (no `.tar.bz2` in any cef-dll-sys build folder).
- D-32 gate first half passed: service (pid 25676) and mpv renderer (pid 24580) of the debug build show no libcef.dll in `tasklist /m`; the renderer holds libmpv-2.dll.
- `run_if_subprocess()` is the first statement of `main`; `sucrose.exe --type=gpu-process --lang=en-US` run by hand exits with code 3 and creates no data folder or log, so no Sucrose code ran (and libcef.dll did load through the delay-load when called).
- `synth-library --web-fixtures --select` and the fixture theme `hello`.

## Required evidence (task 1 acceptance)

**sha1sum of the three cached archives, equal to the upstream SHA1 of the research table (compared before the SHA256 was written into engines.toml):**

```
cba6c57226f4d72dddbdf8fab18440da69573214  cef_binary_156.0.31+ge603d97+chromium-156.0.8078.100_windows32_minimal.tar.bz2
d179fb251141d6c30992a1775db80a6a879852ba  cef_binary_156.0.31+ge603d97+chromium-156.0.8078.100_windows64_minimal.tar.bz2
0d1a346ac91fef15ad32e81db68a19a0c547e2b0  cef_binary_156.0.31+ge603d97+chromium-156.0.8078.100_windowsarm64_minimal.tar.bz2
```

Pinned SHA256 (from `--print-sha256`): x86_64 `c691636862b5f6ba...e63fd`, i686 `44842ff664dacf71...ffac64`, aarch64 `2aa778651c3720e9...b51bc` (full values in `xtask/engines.toml`).

**cef-dll-sys with a CEF_PATH that holds no matching version (read in `build.rs`):** it does not fail, it downloads. If `CEF_PATH` does not exist it downloads into it; if `CEF_PATH/<cef_version>` exists but `<os_arch>` does not, `resolve_cef_dir` downloads and extracts the archive from the CEF CDN; if `CEF_PATH` exists without the version folder it checks `archive.json` and downloads when that fails. That is why `cargo xtask ci` refuses to run before `archive.json` is there (and why `CEF_PATH` has a real dist behind it in this checkout).

**i686:** the `i686-pc-windows-msvc` Rust target is not installed (`rustup target list --installed` shows only x86_64), so the i686 build could not be tried. x86 is left out of the cfg (`x86_64` and `aarch64` only). The i686 archive is pinned and its hashes verified, so a later plan can widen the cfg. 04-12 reports CEF unsupported on Windows x86 (D-027, D-002, D-32). aarch64 was not built either (CI only, windows-11-arm).

**Who copies the runtime files:** `cef-dll-sys` itself (`copy_cef_runtime_files`): every file of the dist root and `locales/` is copied next to the built executables in `target/<profile>` when its build script runs (libcef.dll, chrome_elf.dll, the paks, icudtl.dat, v8_context_snapshot.bin, locales, plus CMakeLists.txt, archive.json and the .lib files). xtask does nothing more for CEF.

## Task Commits

1. **Task 1: pinned CEF reaches the build (tracer)** - `d4415d1f` (feat)
2. **Task 2: delay-load libcef.dll, route CEF subprocess command lines first** - `b4a61667` (feat)
3. **Task 3: Web fixture library for engine runs** - `95247e34` (feat)

**Plan metadata:** the docs(04-01) commit that adds this file.

_Tests were written with the implementation inside each task commit; this plan is `type: execute`, so no separate RED commits._

## Files Created/Modified

- `xtask/engines.toml`, `xtask/src/fetch_engines.rs` - cef pins; Pin generalised; --engine, --print-sha256; tar.bz2 tree extraction; `cef_fetched` for the ci guard
- `.cargo/config.toml`, `Cargo.toml`, `Cargo.lock`, `xtask/Cargo.toml` - CEF_PATH, `cef`, `tar`, `bzip2`, `sucrose-engine-cef`
- `crates/sucrose-engine-cef/*` - subprocess routing without loading libcef.dll; own build.rs for its test executable
- `apps/sucrose/build.rs`, `apps/sucrose/Cargo.toml`, `apps/sucrose/src/main.rs` - delay-load link args, first-statement routing, `pid` on the service start line, routing test
- `xtask/src/main.rs`, `.github/workflows/rust.yml` - ci guard and usage; CI steps (cmake/ninja check, cache, fetch) on the Windows runners
- `xtask/src/synth_library.rs`, `apps/sucrose/tests/fixtures/web-themes/hello/*`, `docs/ARCHITECTURE.md` - fixture library and the status line

## Decisions Made

- The `cef` requirement is `=156.7.0`, not `=156.7.0+156.0.31`: cargo warns that build metadata in a requirement is ignored. The comment in Cargo.toml says the crate carries CEF 156.0.31 as build metadata.
- `App` given to `execute_process` is a no-method `wrap_app!` object; the renderer-side script bridge comes with the web wallpaper host (04-02 and later).
- CEF archives are cached in `target/engines/cef/downloads/` (outside every version folder, so cef-dll-sys never sees them) and the `.verified` marker is `target/engines/cef/<ver>/<os_arch>.verified` beside the dist, because cef-dll-sys copies every dist file into target/<profile>.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] sucrose-engine-cef needs its own build.rs**
- **Found during:** Task 2
- **Issue:** a link argument from `apps/sucrose/build.rs` does not reach the test executable of `sucrose-engine-cef`; without the delay-load there, that test exe imports libcef.dll and cannot start from `target/debug/deps` (the plan's verify runs `cargo test -p sucrose-engine-cef`). The app's build.rs uses `rustc-link-arg` (not `-bins`) for the same reason, so `cargo test -p sucrose` works.
- **Fix:** `crates/sucrose-engine-cef/build.rs` prints the same `/DELAYLOAD:libcef.dll` and `delayimp`; every future package that links the crate and builds executables needs the same file.
- **Files modified:** crates/sucrose-engine-cef/build.rs, apps/sucrose/build.rs
- **Committed in:** b4a61667

**2. [Rule 3 - Blocking] Fixture theme needs a Thumbnail and a Preview**
- **Found during:** Task 3
- **Issue:** the library scan marks a folder without its declared Thumbnail and Preview as corrupt, and the plan lists only SucroseInfo.json and index.html for `hello`.
- **Fix:** the generator writes the declared Thumbnail and Preview from the pool when a fixture lacks them (plain file names only), so the repository holds no binary fixture.
- **Files modified:** xtask/src/synth_library.rs
- **Committed in:** 95247e34

**3. [Rule 1 - Bug] cef version requirement with build metadata**
- **Found during:** Task 1 (first cargo build)
- **Issue:** `=156.7.0+156.0.31` makes cargo warn the metadata is ignored.
- **Fix:** `=156.7.0`, with the metadata named in the comment.
- **Committed in:** d4415d1f

**4. [Rule 2 - Missing critical] `--from` and the marker needed the engine dimension**
- **Found during:** Task 1
- **Issue:** with every engine selected by default, `--from` would be ambiguous and a marker inside the dist would be copied into target/<profile>.
- **Fix:** `--from` needs one engine and one architecture; the marker sits beside the dist folder.
- **Committed in:** d4415d1f

---

**Total deviations:** 4 auto-fixed (1 bug, 1 missing critical, 2 blocking). **Impact:** none on scope; x86 CEF left out is a plan-allowed outcome (A3).

## Issues Encountered

- `cargo fmt` through the Windows wrapper rewrites CRLF files as LF. The CRLF files this plan touched (`fetch_engines.rs`, `apps/sucrose/src/main.rs`, `sucrose-engine-cef` sources) were converted back before each commit; `cargo fmt --all --check` accepts CRLF, so `cargo xtask ci` is unaffected. `.cargo/config.toml`, `Cargo.toml`, `xtask/engines.toml`, `xtask/Cargo.toml`, `rust.yml` and `ARCHITECTURE.md` keep `w/crlf` (checked with `git ls-files --eol`).
- The CI result can only be seen after the maintainer pushes: the workflow steps (CMake/Ninja check, cache, fetch) were not executed here (assumption A5), and the aarch64 and x86 jobs were not built locally.

## User Setup Required

Ninja on the Windows PATH (04-01 `user_setup`) was already done by the maintainer before this run; `cmd.exe /c "where cmake && where ninja"` found both.

## Known Stubs

None. (`run_if_subprocess` returns `None` on targets without a CEF build by design.)

## Threat Flags

None beyond the plan's threat model (T-04-01 to T-04-04 and T-04-SC are mitigated: SHA256 before open, staged extraction with refusals, `--type=` as the only route to CEF, delay-load verified, only cef/tar/bzip2 added).

## Next Phase Readiness

- Plan 02 can host a Web theme in a CEF browser and record D-036: the gate's first half passed, no fallback package is needed.
- `cargo xtask synth-library --out target/web-lib --count 2 --web-fixtures --select hello --force` gives engine runs a library; CEF subprocesses are `sucrose.exe` itself, so `taskkill.exe /IM sucrose.exe /F` ends them.
- ENG-03 stays partial (not marked Complete); it completes in Round B.

## Self-Check: PASSED

All created files exist, commits d4415d1f, b4a61667 and 95247e34 are ancestors of HEAD, `commits: 3` is measured from the ledger, and the acceptance greps (`name = "cef"` x3, `cef = `, `CEF_PATH`, `DELAYLOAD:libcef.dll`, `run_if_subprocess`, `fetch-engines --engine cef`, `CEF is not fetched`, `web-fixtures`, `sucrose-engine-cef` in ARCHITECTURE.md) and EOL checks pass.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
