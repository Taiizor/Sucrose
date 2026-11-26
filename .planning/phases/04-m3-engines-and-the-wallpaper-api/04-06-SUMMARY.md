---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 06
subsystem: engines
tags: [virtual-origin, path-traversal, cef, scheme-handler, range-requests, mime]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-05: CefEngine/CefPump and sucrose_engine_cef::engine::start with the Assignment's theme_dir and source"
provides:
  - "sucrose_engine_web::origin::{ORIGIN, HOST, resolve, Reject, ResolvedFile, byte_range, ByteRange, page_url, url_path}: the one request-path resolver (D-14, SEC-02)"
  - "sucrose_engine_web::mime::mime_for: the legacy MIME table plus .mjs and .wasm"
  - "sucrose_engine_cef::scheme::{plan, register}: CEF scheme handler for https://sucrose.wallpaper.engine/ only, with Range (206/416)"
  - "Fixture themes traversal and seek under apps/sucrose/tests/fixtures/web-themes"
  - "Renderer log lines 'origin request served' (debug) and 'origin request rejected' (info for traversal-class reasons, debug for a missing file)"
affects: [04-07, 04-09, 04-12, 04-20]

actuals:
  tokens: 14600   # chars/4 over the added lines of the realized diff (58475 bytes, 18 files)
  tasks: 2
  commits: 4
plan_head_before: 780f7446f27efc656b0cf40c071e5dee00e3078f
plan_head_after: 9d04f185eeeb824479035c526369c9c20c6d8f2b

tech-stack:
  added: ["percent-encoding 2.3.2 (workspace dependency, audit verdict OK)"]
  patterns:
    - "One pure resolver, thin engine adapters: the CEF scheme handler only streams what scheme::plan decides, so WebView2 can reuse resolve unchanged"
    - "Per-request handler with its own open file: the resolver holds no state and opens no file, concurrent requests share nothing"

key-files:
  created:
    - crates/sucrose-engine-web/Cargo.toml
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/src/origin.rs
    - crates/sucrose-engine-web/src/mime.rs
    - crates/sucrose-engine-web/tests/origin.rs
    - crates/sucrose-engine-cef/src/scheme.rs
    - apps/sucrose/tests/fixtures/web-themes/traversal/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/traversal/index.html
    - apps/sucrose/tests/fixtures/web-themes/seek/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/seek/index.html
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-engine-cef/Cargo.toml
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/lib.rs
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md

key-decisions:
  - "Reject grew by TooLong (path over 2048 bytes) and BadUtf8 (decodes to non-UTF-8), and ResolvedFile carries the index of the root it came from: the log line names the root and the 5000-character vector needs its own reason; Reject::is_traversal separates the info-level refusals from a plain missing file"
  - "A lookup that finds a link leading out of one root reports Outside unless a later root serves the name from inside itself; the file the link points to is never served"
  - "Segments made only of dots and spaces (for example ' .. ') are refused as Dots, because Windows normalises them to a parent segment"
  - "CefStart carries the roots and CefHost::start registers the scheme handler between initialize and browser creation, so the first navigation already goes through the origin"

patterns-established:
  - "Engine adapters decide nothing: scheme::plan (pure, tested on every OS) builds the status, headers and body descriptor, the CEF handler streams it"

requirements-completed: [SEC-02]

coverage:
  - id: D1
    description: "Web theme pages load from https://sucrose.wallpaper.engine/ on CEF; the file:// transport is gone and no port is opened"
    requirement: TYPE-03
    verification:
      - kind: integration
        ref: "scratch run on target/web-lib (hello fixture): Renderer log 'page loaded url=https://sucrose.wallpaper.engine/index.html status=200'; grep file:/// over engine.rs and browser.rs prints 0"
        status: pass
    human_judgment: false
  - id: D2
    description: "One resolver refuses every traversal spelling (dots, encoded and double-encoded dots, backslash, drive, colon, UNC, NUL, too long, folders, links that lead out) and serves theme folder before content folder"
    requirement: SEC-02
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/origin.rs#origin_refuses_every_traversal_vector_with_its_reason"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/origin.rs#origin_refuses_a_link_inside_the_theme_that_leads_out (junction created, not skipped)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/origin.rs#origin_resolves_parallel_requests_independently"
        status: pass
    human_judgment: false
  - id: D3
    description: "On CEF the traversal fixture page requests every vector and gets 404; each refusal is logged and nothing outside the theme folder is served"
    requirement: SEC-02
    verification:
      - kind: integration
        ref: "scratch run, theme traversal with a junction outside -> target/web-lib: 17 'origin request rejected' lines (11 at info), 0 'origin request served' lines naming secret"
        status: pass
    human_judgment: false
  - id: D4
    description: "Media requests honour a single Range (206 with Content-Range and Accept-Ranges, 416 past the end) so a local video seeks"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/src/origin.rs#origin_byte_range_* and crates/sucrose-engine-cef/src/scheme.rs#a_single_range_is_a_206_with_content_range"
        status: pass
      - kind: integration
        ref: "scratch run, theme seek with a VP9 clip: CEF console 'range: 206 bytes 0-9/12701 accept-ranges=bytes' and 'seek: reached 2.00 of 4.00'"
        status: pass
    human_judgment: false
  - id: D5
    description: "Only the host sucrose.wallpaper.engine is handled by Sucrose; every other https request keeps CEF's own networking"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/src/origin.rs#origin_url_path_takes_only_the_virtual_host (domain-scoped registration is not exercised against a real foreign https page)"
        status: pass
    human_judgment: true
    rationale: "The registration passes the domain to CEF, but no run requests a foreign https address to see CEF's network stack answer it; the phase verifier judges it with a real web theme that loads a remote asset"

# Metrics
duration: 24min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 06: The virtual origin Summary

**Every CEF web wallpaper is served from https://sucrose.wallpaper.engine/ through one pure, vector-tested path resolver with Range support, and path traversal is refused on the resolver and on the running CEF build**

## Performance

- **Duration:** 24 min
- **Started:** 2026-11-26T02:34:00Z
- **Completed:** 2026-11-26T02:58:30Z
- **Tasks:** 2
- **Files modified:** 18 (10 created)

## Accomplishments

- New pure crate `sucrose-engine-web`: `resolve` takes the URL path only, decodes it once, refuses NUL, backslash, colon, leading `//`, dot segments and double encoding, then looks in the theme folder before the generated-content folder with canonical containment, so a junction or link that leads out is refused and a folder is not a file. It holds no state and opens no file. `byte_range`, `page_url`, `url_path` and the legacy MIME table (plus `.mjs`, `.wasm`) live beside it.
- The CEF engine loads its page from the virtual origin: a scheme handler registered for `https` on the one host (between `initialize` and the first navigation) streams the file of its own request, answers every refusal with 404, a single Range with 206 or 416, and keeps the legacy CORS headers. The `file://` transport and its helper are gone.
- Proven on the Windows build: the traversal fixture page requests 16 vectors (one behind a junction to a folder holding a secret file) and the log shows 17 refusals and no served file outside the theme; the seek fixture gets `206 bytes 0-9/12701` and a VP9 clip seeks to its middle. The vector table, the junction cases and a parallel-resolve test run as unit tests (16 tests match the plan's `origin` filter).
- FEATURES records the origin, the CORS parity, the MIME additions and the fixed traversal hole; ARCHITECTURE lists `sucrose-engine-web` as shipped and describes the origin.

## Task Commits

1. **Task 1: A Web theme page loads from https://sucrose.wallpaper.engine/ on CEF** - `09d8ee9a` (test, RED), `10f90bbd` (feat)
2. **Task 2: Traversal vectors are refused on the resolver and on CEF; Range seeks a local video** - `d9086736` (test), `9d04f185` (feat)

**Plan metadata:** the docs commit after this summary.

## Files Created/Modified

- `crates/sucrose-engine-web/src/origin.rs` - ORIGIN, HOST, resolve, Reject, ResolvedFile, byte_range, page_url, url_path
- `crates/sucrose-engine-web/src/mime.rs` - the fixed MIME table
- `crates/sucrose-engine-web/tests/origin.rs` - the traversal vector table, link and parallel tests
- `crates/sucrose-engine-cef/src/scheme.rs` - `plan` (pure decisions and log lines) and the CEF scheme handler factory and resource handler
- `crates/sucrose-engine-cef/src/browser.rs` - `CefStart.roots`, registration before the first navigation
- `crates/sucrose-engine-cef/src/engine.rs` - page URL from `page_url`, `file_url` removed
- `apps/sucrose/tests/fixtures/web-themes/{traversal,seek}` - the two fixture pages
- `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - the origin and the shipped crate

## Decisions Made

See key-decisions. D-14 was used exactly as confirmed (one shared origin); no new decision and no Accepted entry was reopened.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] browser.rs and the Reject contract needed small additions**
- **Found during:** Task 1
- **Issue:** The scheme handler must be registered after `cef::initialize` and before the browser navigates, which happens inside `CefHost::start` (browser.rs, not in the plan's file list), and the plan's `Reject` list had no reason for a 5000-character path or for bytes that are not UTF-8.
- **Fix:** `CefStart` gained `roots` and `CefHost::start` registers the factory; `Reject` gained `TooLong` and `BadUtf8`, `ResolvedFile` gained `root`, and `Reject::is_traversal` drives the log level.
- **Files modified:** crates/sucrose-engine-cef/src/browser.rs, crates/sucrose-engine-web/src/origin.rs
- **Verification:** cargo test, clippy -D warnings, scratch runs
- **Committed in:** 10f90bbd

**2. [Rule 3 - Blocking] rustfmt rewrote touched Rust files and Cargo.lock to LF**
- **Found during:** Tasks 1 and 2 (cargo fmt, cargo)
- **Issue:** The working tree keeps these files CRLF (autocrlf); the formatter and cargo wrote LF.
- **Fix:** Converted the touched files back to CRLF before the commits and at the end; docs/FEATURES.md keeps its missing final newline.
- **Committed in:** 10f90bbd, 9d04f185

### Notes

- Chromium's URL parser removes `..` and `%2e%2e` segments before a request leaves the page, so on CEF the vectors `/../x`, `/%2e%2e/x`, `/%2E%2E/x` and `/a/../../x` arrive as `/x` and are refused as a missing file (debug level). The spellings that stay intact (double encoding, `%5c`, `//`, drive letters, NUL, the long path, the junction) are refused with their own reason at info level. The unit vector table covers the dot spellings the browser never sends.
- The Task 2 RED commit holds the vector table, which already passed because the resolver landed with Task 1 (a deliberate pin of the security behavior); its RED part is the four `byte_range` tests. `gsd_run check tdd-red-evidence` has no cargo test report adapter, so the RED evidence is the failing run recorded in the commit history, not a classifier verdict.
- The repository has no playable video (the committed test videos are empty files) and CEF ships without H.264, so the seek run used a 4 s VP9-in-MP4 clip made with ffmpeg in the scratch library only (nothing committed). The page logs its results to the console, which CEF writes to `CefRenderer-0.log`; a seek on a large real video stays with the 04-20 checkpoint as the plan says.

---

**Total deviations:** 2 auto-fixed (both blocking)
**Impact on plan:** No scope change; the contract only grew.

## Issues Encountered

None. The scratch junction was removed with `rmdir` (link only) after the traversal run, and no sucrose process was left running.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- WebView2 (Round B) puts its WebResourceRequested adapter on `resolve` and `byte_range`; generated pages for GIF and video go to `<cache_dir>/Content`, which the resolver already looks in second.
- ENG-03 (Round B) and TYPE-03 (04-20) stay open; SEC-02 is complete for CEF, and WebView2 must call the same resolver without reopening it.

## Self-Check: PASSED

Created files exist (crates/sucrose-engine-web, crates/sucrose-engine-cef/src/scheme.rs, both fixture folders), the four commits are ancestors of HEAD, `cargo xtask ci` exited 0, `grep file:///` over engine.rs and browser.rs prints 0, and no sucrose process is running.
