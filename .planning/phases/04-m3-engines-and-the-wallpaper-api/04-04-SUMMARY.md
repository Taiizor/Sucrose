---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 04
subsystem: engines
tags: [engine-choice, fallback, legacy-engine-names, availability, clock-follower]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-02: the first Web wallpaper in CEF (depends_on); the engine names the settings already carry from Phase 2"
provides:
  - "sucrose_engine::EngineId (7 variants) with from_legacy, to_legacy and supports (D-10)"
  - "sucrose_engine::Availability, Hint, Unsupported: the report shape the engine detection fills (plan 12 feeds the real one)"
  - "sucrose_engine::choose_engine / fallback_order / Choice: the pure per-type choice with the D-11 fallback order"
  - "sucrose_engine::PauseLevel (Light, Heavy; D-22)"
  - "sucrose_engine::sync: the Phase 2 clock follower, moved; sucrose_engine_mpv::sync re-exports it"
affects: [04-05 engine trait with mpv and CEF, 04-12 detection feeds the choice, 04-13, 04-15, 04-20]

actuals:
  tokens: 7500   # chars/4 over the realized diff (30003 bytes, 9 files, of which about 6000 tokens are the new crate and about 1000 the docs)
  tasks: 2
  commits: 3
plan_head_before: fa4637480b1b3992886affd01467a7ba182532ee
plan_head_after: b1a74877364c5c20aa04844e075b8dca7c08a6aa

tech-stack:
  added: []
  patterns:
    - "Pure decision function over an injected availability closure, with sentence-named table tests per wallpaper type (as supervisor.rs)"
    - "Const per-type fallback table; the configured engine is tried first and skipped when it reappears"
    - "Legacy names stay in settings; one place maps them to engines and back"

key-files:
  created:
    - crates/sucrose-engine/Cargo.toml
    - crates/sucrose-engine/src/lib.rs
    - crates/sucrose-engine/src/choice.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-engine/src/sync.rs
    - crates/sucrose-engine-mpv/Cargo.toml
    - crates/sucrose-engine-mpv/src/lib.rs
    - docs/ARCHITECTURE.md

key-decisions:
  - "Fallback order inside the web engines is CEF first, then WebView2, for every type that falls back to a web engine (the research order; D-11 leaves it to discretion)"
  - "A configured engine outside the type's legacy allowed list is replaced by the type's legacy default (CEF for web types, mpv for Gif and Video, the embedder for Application) before the choice, so no fallback is reported for it"
  - "When no engine is available the Choice still carries fell_back_from, so the refusal can name the configured engine and why it is unusable"

requirements-completed: []

coverage:
  - id: D1
    description: "choose_engine picks the configured engine when installed, else the D-11 order per type, and names what it fell back from; no engine when none is available"
    requirement: ENG-07
    verification:
      - kind: unit
        ref: "crates/sucrose-engine/src/choice.rs#choose_* (12 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Legacy engine names map to engines without a settings change (Vexana/Xavier to built-in GIF, Nebula to built-in video, Aurora to the embedder, the rest one to one) and round-trip"
    requirement: ENG-07
    verification:
      - kind: unit
        ref: "crates/sucrose-engine/src/choice.rs#legacy_* (4 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "The clock follower lives in sucrose-engine and every caller compiles and behaves the same through the sucrose_engine_mpv::sync re-export"
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine sync (31 passed); cargo test -p sucrose-engine-mpv; cargo test -p sucrose (275 passed)"
        status: pass
    human_judgment: false
  - id: D4
    description: "ARCHITECTURE.md describes the shipped sucrose-engine, the choice, the fallback order and the name mapping"
    verification: []
    human_judgment: true
    rationale: "Documentation accuracy is a reading judgment; the grep acceptance only proves the section exists"

duration: 10min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 04: The engine choice Summary

**A pure `choose_engine` in the new `sucrose-engine` crate turns the legacy engine names into engines and picks the one that plays with the D-11 fallback order, and the Phase 2 clock follower now lives there too.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-26T01:56:30Z
- **Completed:** 2026-11-26T02:06:00Z
- **Tasks:** 2
- **Files modified:** 9 (3 created, 1 moved, 5 modified)

## Accomplishments

- `EngineId`, `Availability`, `Hint`, `Unsupported`, `Choice` and `PauseLevel` exist with doc comments naming their decisions; the crate has no OS code and no unsafe, so it builds and tests on every CI OS.
- `choose_engine(kind, configured, availability)` is pure and tested per wallpaper type: the configured engine wins when installed; otherwise Web, Url and YouTube try the other web engine, Gif tries built-in GIF, mpv, libVLC, then a web engine, Video tries mpv, built-in video, libVLC, then a web engine, and Application tries the embedder only (T-04-11).
- The `Choice` names the engine it fell back from and its availability, has no engine when none is available, and never touches the setting.
- The clock follower moved with its tests unchanged; `sucrose_engine_mpv::sync` is a re-export, so `apps/sucrose/src/render.rs` was not touched.
- `docs/ARCHITECTURE.md` (still CRLF) lists `sucrose-engine` as shipped and describes the choice, the fallback order and the D-10 name mapping.

## Task Commits

1. **Task 1: A legacy engine name becomes the engine that plays** (tracer, tdd) - `5e92ef43` (test, RED: 11 of 12 `choose` tests failing on their assertions against a stub) and `aee85d17` (feat, GREEN: 16 tests pass)
2. **Task 2: The clock follower moves into sucrose-engine** - `b1a74877` (refactor)

**Plan metadata:** the docs commit that carries this SUMMARY, STATE and ROADMAP.

Tracer gate: interactive run, `end-of-phase` mode, automated-only verify; the verify re-ran green (12 + 4 tests) before the expansion task, so no checkpoint was raised.

## Files Created/Modified

- `crates/sucrose-engine/src/choice.rs` - identities, availability report, `choose_engine`, `fallback_order` and 16 sentence-named tests
- `crates/sucrose-engine/src/lib.rs` - crate docs, `PauseLevel`, re-exports
- `crates/sucrose-engine/src/sync.rs` - the clock follower (moved from sucrose-engine-mpv, unchanged)
- `crates/sucrose-engine/Cargo.toml`, `Cargo.toml`, `Cargo.lock` - the crate and its workspace entry
- `crates/sucrose-engine-mpv/src/lib.rs`, `crates/sucrose-engine-mpv/Cargo.toml` - re-export and dependency
- `docs/ARCHITECTURE.md` - status line, workspace layout line and Engines section

## Decisions Made

- CEF is tried before WebView2 wherever a web engine stands in (research Pattern 1 order; recorded in FEATURES by 04-12).
- An engine the type does not allow is treated as the type's legacy default before the choice (T-04-11), so a hand-edited file neither crashes nor reports a bogus fallback.
- `Choice.fell_back_from` is also set when no engine is available, so the refusal reason can name the configured engine.

## Deviations from Plan

### Auto-fixed Issues

**1. [Minor - dependency list] `thiserror` not added to sucrose-engine**
- **Found during:** Task 1
- **Issue:** The plan lists `thiserror` among the crate's dependencies, but the crate defines no error type; an unused dependency only adds build surface.
- **Fix:** Left it out. The first plan that adds an error type adds it with `{ workspace = true }`.
- **Files modified:** crates/sucrose-engine/Cargo.toml
- **Committed in:** 5e92ef43

---

**Total deviations:** 1 (a dependency the code does not need)
**Impact on plan:** None; no external crate was added at all (T-04-SC).

## Threat Flags

None. The only trust boundary (settings to choice) is covered by T-04-11: tested by `choose_replaces_a_configured_engine_outside_the_allowed_list_with_the_type_default` and `choose_never_returns_an_engine_that_does_not_support_the_type`.

## Issues Encountered

None. `cargo fmt` converts a touched CRLF Rust file to LF, so the CRLF endings of the files this plan touched were restored after formatting (`git ls-files --eol` shows w/crlf; `cargo fmt --check` passes on them).

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-05 can put mpv and CEF behind the engine trait on top of `EngineId`; 04-12 feeds the real availability report into `choose_engine`.
- ENG-07 is marked Complete only by 04-20; the fallback notice, row alerts and the snooze are still to come.

## Self-Check: PASSED

- Created files and the three commits (`5e92ef43`, `aee85d17`, `b1a74877`) exist; the old `sucrose-engine-mpv/src/sync.rs` is gone.
- `cargo xtask ci` (fmt, clippy -D warnings, tests) passes; acceptance criteria: `pub fn choose_engine` present, `sucrose-engine = ` in Cargo.toml, 16 sucrose-engine tests pass, `sucrose_engine::sync` re-export present, `choose_engine` in ARCHITECTURE.md (2), ARCHITECTURE.md still CRLF.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
