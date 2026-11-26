---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 23
subsystem: engines
tags: [webview2, range-requests, reply-planner, traversal, synth-library]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "sucrose-engine-webview2, serve::plan and the WebResourceRequested handler (04-22), origin::resolve and byte_range (04-06)"
provides:
  - "serve::{MAX_RANGE_BODY, RangeBody, range_body, content_range_text, read_range}: the in-memory range cut under a 4 MiB cap"
  - "WebView2 answers a single Range with 206 (memory stream, Content-Range of the bytes sent), 416, 200; HEAD gets headers only"
  - "cargo xtask synth-library --engine <Type>=<EngineType> (repeatable)"
  - "crates/sucrose-engine-web/tests/serve.rs: the origin vector table and the Range cases against serve::plan"
affects: [04-24, 04-25, 04-30, 04-37]

actuals:
  tokens: 15000
  tasks: 2
  commits: 3
plan_head_before: c7baf452ccc03d2c86efec2150a8c79a456d3a73
plan_head_after: 200619f71d0a3c2554a444d4d9f4257ec127ad68

tech-stack:
  added: []
  patterns:
    - "an adapter that holds a body in memory cuts it with a pure helper under a cap and names the cut in Content-Range"
    - "an adapter logs what it sent under its own tracing target, so a run that enables one engine's target sees every request"

key-files:
  created:
    - crates/sucrose-engine-web/tests/serve.rs
  modified:
    - crates/sucrose-engine-web/src/serve.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - xtask/src/synth_library.rs
    - xtask/src/main.rs
    - docs/ARCHITECTURE.md
    - docs/FEATURES.md

key-decisions:
  - "A range body is read into a memory stream of at most MAX_RANGE_BODY (4 MiB); when the file shrank between plan and read, Content-Range names the bytes actually read"
  - "The synth-library marker records the --engine choices, so a run with another engine rewrites instead of reporting up to date"
  - "The WebView2 host logs 'origin request served' (status, content_range) and, at debug, 'origin request rejected' for each 404 under its own target"

patterns-established:
  - "range_body/read_range live in the engine-neutral crate and are unit tested; the COM code only builds the stream"

requirements-completed: []

coverage:
  - id: D1
    description: "A single Range request on WebView2 gets the planner's reply: 206 with Content-Range and Accept-Ranges, the body a memory stream of at most 4 MiB whose Content-Range names the bytes it carries (ENG-02 part)"
    requirement: ENG-02
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/src/serve.rs#an_open_range_of_a_long_file_is_cut_to_the_cap_with_a_content_range_that_names_the_cut and 4 more helper tests"
        status: pass
      - kind: e2e
        ref: "seek fixture on WebView2: 'origin request served path=/clip.mp4 status=206 content_range=bytes 0-9/16' (2 lines with 206); with a 10 MiB clip 'status=206 content_range=bytes 0-4194303/10485776'"
        status: pass
    human_judgment: true
    rationale: "That a video really seeks on WebView2 with these replies is checked with a real clip at the final maintainer checkpoint (04-37); here only the reply status was checked, on a placeholder file"
  - id: D2
    description: "The traversal fixture on WebView2 refuses every vector that reaches the handler and serves no file outside the theme folder, including the secret behind a junction (SEC-02 adapter test)"
    requirement: SEC-02
    verification:
      - kind: integration
        ref: "crates/sucrose-engine-web/tests/serve.rs (3 tests: vector table with and without Range, link out, 200/206/416)"
        status: pass
      - kind: e2e
        ref: "traversal fixture with a junction on WebView2: 28 'origin request rejected' lines (11 at info from the planner, 17 at debug from the host; CEF measured 17), 0 served lines naming secret"
        status: pass
    human_judgment: false
  - id: D3
    description: "synth-library --engine sets the engine of a type in the prepared settings; an engine outside the type's legacy list or an unknown type is an error and writes nothing"
    verification:
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_parse_reads_repeatable_engine_options, #synth_engine_outside_the_legacy_list_or_an_unknown_name_is_an_error, #synth_engine_is_written_to_the_settings_and_a_bad_one_writes_nothing"
        status: pass
    human_judgment: false

duration: 14min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 23: WebView2 serves the virtual origin like CEF Summary

**WebView2 now answers a Range request with the planner's 206/416/200 (a range cut to a 4 MiB memory stream whose Content-Range names the bytes sent), the traversal fixture refuses every vector on WebView2 as it does on CEF, and synth-library takes --engine <Type>=<EngineType>.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-26T22:49:00Z
- **Completed:** 2026-11-26T23:03:00Z
- **Tasks:** 2 (Task 1 tracer, Task 2 auto)
- **Files modified:** 8 (1 created)

## Accomplishments

- `serve.rs` gained `MAX_RANGE_BODY`, `range_body`, `content_range_text` and `read_range`; the planner's debug line now carries `content_range`.
- The WebView2 host reads the `Range` header and the method; a range is a memory stream (`SHCreateMemStream`), a whole file streams from disk, `HEAD` gets the headers only, and the reply names the bytes actually read.
- `tests/serve.rs` runs the whole origin vector table (30 vectors, with and without a Range), a link that leads out, and the 200/206/416 cases against `serve::plan`.
- `cargo xtask synth-library --engine Web=WebViewLive` (repeatable, legacy allowed lists, recorded in the marker).

## Task Commits

1. **Task 1: Range 206 on WebView2, --engine** - `a7dc72f9` (feat); `bd509292` (fix: the host's own log lines, see deviations)
2. **Task 2: planner vector tests and docs** - `200619f7` (test)

## TDD evidence (Task 1)

RED, serve helpers (stubbed API, assertions failing): `cargo test -p sucrose-engine-web --lib serve` gave 7 passed, 4 failed (`a_range_under_the_cap_keeps_its_bytes`, `content_range_text_names_the_inclusive_end`, `an_open_range_of_a_long_file_is_cut_to_the_cap...`, `read_range_reads_exactly_the_bytes_asked...`), then 11 passed after the implementation. The xtask tests were written with the implementation (compile-level RED only). No separate RED commit: tests and code are in one commit per task.

## Verification

- `cargo xtask ci`: exit 0 (fmt, clippy -D warnings, tests).
- Task 1 chain: `cargo test -p sucrose-engine-web serve`, `cargo test -p xtask synth` (19 passed), `cargo build -p sucrose`; seek fixture on WebView2: `origin request served path=/clip.mp4 status=206 content_range=bytes 0-9/16` and `status=206 content_range=bytes 0-15/16`, grep count of 206 lines for clip.mp4: **2**. With a 10 MiB clip the open range was cut: `status=206 content_range=bytes 0-4194303/10485776`.
- Task 2 chain: `cargo test -p sucrose-engine-web serve` (3 integration tests) and `origin` pass; traversal fixture with a junction behind `outside` on WebView2: **28** `origin request rejected` lines (11 INFO from the planner, 17 DEBUG from the host; the CEF run of 04-22 had 17), the plan's check (at least 14 and no served line naming the secret) exit 0. The junction was removed with `rmdir` afterwards.
- Acceptance: `MAX_RANGE_BODY` in serve.rs (many), `"--engine"` in synth_library.rs (1), `sucrose-engine-webview2` in ARCHITECTURE.md (2), `docs/ARCHITECTURE.md` and `docs/FEATURES.md` still w/crlf, FEATURES.md still without a final newline.
- Known flakes: none seen.

## Files and line endings

Touched files that were `w/lf` (no restore needed): serve.rs, host.rs, synth_library.rs, main.rs. `docs/ARCHITECTURE.md` and `docs/FEATURES.md` were edited with in-line replacements and stayed CRLF. `tests/serve.rs` is new (LF). `cargo fmt` changed no CRLF file, so nothing was restored.

## Decisions Made

See key-decisions; all follow the plan (cap and helper behind the planner, reversible).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The plan's verify filter cannot show the served and rejected lines**
- **Found during:** Task 2 (and Task 1)
- **Issue:** The verify commands run with `SUCROSE_LOG=info,sucrose_engine_webview2=debug`, but the served (debug) and not-found (debug) lines are logged by the planner under `sucrose_engine_web`. Run as written the 206 count would be 0 and the refusals only the 11 info lines (below the bound 14).
- **Fix:** The host now logs `origin request served` (status, and `content_range` of the bytes actually sent) and, at debug, `origin request rejected` for each 404 under its own target, which the plan's literal commands enable. The planner's own lines stay (a run that enables both sees duplicates at debug).
- **Files modified:** crates/sucrose-engine-webview2/src/host.rs
- **Commit:** a7dc72f9 (served line), bd509292 (refusal lines)

**2. [Rule 3 - Blocking] xtask usage text**
- **Found during:** Task 1
- **Issue:** The plan says USAGE mentions `--engine`; the command list is in `xtask/src/main.rs`, which the plan's file list omits.
- **Fix:** Updated the synth-library line there (also names --web-fixtures and --select).
- **Files modified:** xtask/src/main.rs
- **Commit:** a7dc72f9

**Total deviations:** 2 auto-fixed (1 bug in the plan's verify filter, 1 blocking). **Impact:** none on scope.

## Issues Encountered

- A transient `.git/index.lock` blocked one commit; it was gone on the check (no git process of mine running) and the retry succeeded.

## Known Stubs

None.

## Threat Flags

None. T-04-52 is mitigated: a range body is at most 4 MiB in memory (`MAX_RANGE_BODY`), whole files stream from disk.

## Next Phase Readiness

- 04-24 onward can use `--engine <Type>=<EngineType>` for engine runs. The video seek itself (a real clip on WebView2) is left to the checkpoint steps; only reply statuses were checked here.
- ENG-02 completes at 04-37; SEC-02 stays Complete and is not re-marked.

## Self-Check: PASSED

Created and modified files exist, commits `a7dc72f9`, `bd509292`, `200619f7` are ancestors of HEAD.
