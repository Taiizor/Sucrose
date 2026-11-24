---
phase: 03-m2-service-ipc-and-portal-connected
plan: 25
subsystem: testing
tags: [measurement, documentation, d-25, ui-07, architecture, parity]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "Portal measure mode and the two D-25 sample files (plan 24), the M2 code and plan SUMMARYs (plans 01-23, 26)"
provides:
  - "docs/MEASUREMENTS.md rows and a reading paragraph for the two D-25 scenarios, each with an explicit rule verdict (real library pass, synthetic 500 themes FAIL)"
  - "docs/ARCHITECTURE.md describing the M2 state (settings file and migration, Portal protocol and endpoint, platform additions, Portal on the one endpoint with --no-portal, D-13 service lifetime)"
  - "docs/FEATURES.md final Phase 3 status for the rows the phase touched, docs/GPUI.md lessons of the phase"
  - "The Verification scope of Phase 3 for /gsd-verify-work (below)"
affects: [verify-work, ui-07, meas-02, phase-03-gap-closure]

actuals:
  tokens: 7700
  tasks: 2
  commits: 2
plan_head_before: ef28010a4789f4a07466ed35e84fcb3a3d8b7463
plan_head_after: d20cc810eaea9769712796e7ab27f1f47d69245c

tech-stack:
  added: []
  patterns:
    - "A failing measurement is recorded as failing with its caveats, not re-run or tuned in the recording plan"

key-files:
  created: []
  modified:
    - docs/MEASUREMENTS.md
    - docs/ARCHITECTURE.md
    - docs/FEATURES.md
    - docs/GPUI.md

key-decisions:
  - "UI-07 is not marked complete: the synthetic 500-theme scenario fails the D-25 rule, so the requirement stays open as a gap for /gsd-verify-work"
  - "The rows are recorded as found although plan 24 said both scenarios must be repeated after the hover-preview fix; the plan's prohibition forbids re-running or tuning here"

requirements-completed: []  # UI-07 is declared by this plan but deliberately NOT marked complete, see Gaps

coverage:
  - id: D1
    description: "MEASUREMENTS.md carries the D-25 rows for the real library and the 500-theme synthetic library with first page, peak and final values, caps and scenario, each with a pass or fail verdict"
    requirement: "UI-07"
    verification:
      - kind: other
        ref: "cargo xtask measure --portal-summary on target/measure/portal-real.json and portal-synth.json (exit 0); grep -c 'Portal library' docs/MEASUREMENTS.md = 3"
        status: pass
    human_judgment: false
  - id: D2
    description: "Paging through the library for three rounds does not make the Portal's memory grow beyond the first page plus the cache cap"
    requirement: "UI-07"
    verification:
      - kind: other
        ref: "target/measure/portal-synth.json: working set first page 130.8 MB, final 269.4 MB (limit 130.8 + 48 MiB = 181.1 MB)"
        status: fail
    human_judgment: true
    rationale: "The rule fails on the synthetic library; the runs also had non-animating previews and skewed round windows, so a repeat after the preview fix and a decision on the cause are the maintainer's"
  - id: D3
    description: "ARCHITECTURE.md, FEATURES.md and GPUI.md describe M2 as built, with the files' line endings kept"
    verification:
      - kind: other
        ref: "grep -c 'Sucrose.json' docs/ARCHITECTURE.md = 2; grep -c 'no-portal' docs/ARCHITECTURE.md = 1; git ls-files --eol shows w/crlf for the three files; FEATURES.md has no final newline"
        status: pass
    human_judgment: true
    rationale: "Whether the prose matches what the maintainer saw of M2 is a reading judgment; the status choices (🟨 where macOS and Linux are unverified) follow the plan's rule"

duration: 5 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 25: Service, IPC and Portal connected, measurement and docs Summary

**Both D-25 Portal scenarios are judged and recorded in MEASUREMENTS.md (real 90-theme library passes the UI-07 rule, the synthetic 500-theme library fails it: working set 131 MB on the first page, 269 MB at the end against a 181 MB limit), and ARCHITECTURE, FEATURES and GPUI now describe M2 as built.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-11-24T12:34Z
- **Completed:** 2026-11-24T12:40Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- No Rust Sucrose process was left running (the service had exited after the Portal closed, D-13); `taskkill` confirmed it. The summaries of both sample files were re-run and gave the numbers of plan 24 unchanged.
- Pass rule (final at most first page + 48 MiB, round 3 end at most round 1 end x 1.05), MB, release build, Portal process only:
  - Real library (90 themes, 480 samples): working set first page 172.8, final 205.5 (limit 172.8 + 50.3 = 223.1), round ends 214 / 206 / 206 (round 3 within 1.05 x 214): **pass**; private first page 163.1, final 196.0 (limit 163.1 + 50.3 = 213.4), round ends 204 / 196 / 196: **pass**.
  - Synthetic 500 themes (800 samples): working set first page 130.8, final 269.4 (limit 130.8 + 50.3 = 181.1), round ends 206 / 206 / 270 (limit 216): **FAIL** on both conditions; private first page 123.5, final 261.8, round ends 199 / 199 / 262: **FAIL** on both. The first-page against final condition fails on its own, independent of the skewed round windows.
- MEASUREMENTS.md has the two rows (counts and memory figures only, T-03-58) and a reading paragraph saying what was paged, the rule and the result.
- ARCHITECTURE.md: M2 status line, the Portal as a peer on the one endpoint (D-033), `--no-portal`, the D-13 service lifetime, `Setting/Sucrose.json` with its `.bak` (D-032).
- FEATURES.md: sorting, light/dark, settings pages and single instance moved from not started to in progress with Phase 3 notes (all 🟨: macOS and Linux unverified by hand); the hover preview row now says the memory effect was not measured; the fullscreen scope note names the persisted key.
- GPUI.md: the service-link bridge, the `test` macro shadowing by the crate-root glob, the foreground handoff, the open hover-preview defect and the open paging-memory finding.
- `cargo xtask ci` passes (fmt, clippy, tests).

## Task Commits

1. **Task 1: Judge and record the UI-07 numbers (tracer)** - `e6776eeb` (docs)
2. **Task 2: ARCHITECTURE, FEATURES and GPUI notes for M2** - `d20cc810` (docs)

**Plan metadata:** the docs(03-25) completion commit that follows.

## Files Created/Modified

- `docs/MEASUREMENTS.md` - two Portal library rows and the reading paragraph (CRLF kept)
- `docs/ARCHITECTURE.md` - M2 status, Portal on the one endpoint, D-13, settings file
- `docs/FEATURES.md` - final Phase 3 statuses and notes
- `docs/GPUI.md` - Phase 3 lessons and two open findings

## Decisions Made

- UI-07 is not marked complete in REQUIREMENTS.md; the requirement stays open until the memory finding is resolved (see Gaps).
- The rows record the numbers of plan 24 as found. Plan 24 advised repeating both scenarios after the hover-preview fix; this plan's prohibition (no re-run with other settings, no cap changes) and its executor cannot drive the Portal by hand, so the repeat is left to the gap closure.

## Deviations from Plan

### Auto-fixed Issues

None - plan executed exactly as written. Two notes that are not deviations: the plan's `tasklist.exe | grep -ci 'sucrose.exe'` criterion was already 0 at the start (scenario 2's apps were not running when this plan began), and `cargo xtask ci` ran with three unrelated uncommitted files in the working tree (`crates/sucrose-platform/src/autostart.rs`, `os_windows/autostart.rs`, `unsupported.rs`) that this plan did not touch and did not stage.

**Total deviations:** 0. **Impact:** none.

## Gaps for /gsd-verify-work

1. **UI-07 fails on the synthetic 500-theme library (D-25 scenario 2).** Working set first page 130.8 MB, peak 286.6, final 269.4 against a limit of 181.1 MB; private 123.5, 279.3, 261.8. The real 90-theme library passes. The memory rose in two steps (to about 221 MB in round 1, flat 206-207 MB while idle on page 1, up to 287 MB in rounds 2 and 3), so something outside the 64 image / 48 MiB thumbnail cache grows with the number of pages visited; the cause (atlas pages, decode buffers, allocator retention) is not established (see `docs/GPUI.md`).
2. **Hover previews do not animate in the release Portal** (plan 24 finding; leads at `apps/sucrose-portal/src/library/card.rs:282-283` and `library/mod.rs:541-562`). Neither run exercised the preview cost (caps 120 frames / 24 MiB), so both scenarios must be repeated after the fix with a tighter round cue (plan 24 deviation 2).
3. Success criterion 5 ("scrolling through a large library keeps its memory bounded") is therefore not met on the large library; this is reported, not tuned away.

## Verification scope

Success criteria of Phase 3 (ROADMAP) against what covers them. "Human" means an end-of-phase check by the maintainer (no synthesized input is allowed, so the plans harvested these as `<human-check>` blocks).

**1. Library, hover previews, move, Use through the service.**
- Automated: library scan on fixtures and the real 90-theme library (`crates/sucrose-core/tests/library.rs`, 03-02); Use decision, refusals and LoadSource (`apps/sucrose/src/service.rs` tests and `apps/sucrose/tests/portal.rs`, 03-09); showcase copy and recycle bin (`apps/sucrose/src/library_ops.rs`, `tests/portal.rs`, `crates/sucrose-platform/src/trash.rs`, 03-21); library move with rollback (`library_ops.rs`, `tests/portal.rs#a_library_moves_to_an_empty_folder_and_the_location_follows`, 03-22); pagination, search and sort pure logic (03-10, 03-14); preview state machine (03-19).
- Runs and captures by the executors on synthetic folders (grid, pages, selected pill, settings, status bar); Use transition watched by the maintainer on two monitors (03-11, D-26).
- Human at the end of the phase: card click Use and keyboard Use (03-12), search typing, Escape, Ctrl+F and sort (03-14), hover previews and their memory (03-19), Recycle Bin contents and restore (03-21), folder picker, destructive confirmation and progress text (03-22).
- Not met or unverified: hover previews do not animate (Gap 2); macOS and Linux rendering; the 150 ms skeleton path on a slow library.

**2. Settings migration, persistence, atomic writes.**
- Automated: `crates/sucrose-config/tests/store.rs` (migration runs once and never writes a legacy file, kill during write never leaves a corrupt file, stale temporary files); `crates/sucrose-config/tests/migrate.rs` (the real files of all nine migrated legacy files, exclusions, Donate and token files never reach the new file); `apps/sucrose/tests/portal.rs#a_change_survives_a_restart`; `sha256sum` of the 14 real legacy files unchanged before and after runs (03-01).
- Human at the end of the phase: every General and Personal row shows the migrated value and survives a Portal restart; 250 in Library Pagination clamps (03-20).
- Unverified: a C# install other than the maintainer's.

**3. Second start reuses the instance, Exit closes everything, bounded renderer restarts.**
- Automated: `apps/sucrose/tests/portal.rs` (a launch with a Portal registered gets PortalRunning; second Portal gets AlreadyOpen; service exit rule), `apps/sucrose/src/lifecycle.rs` tests (start and exit rules, closing state, sibling-executable check); the bounded restart budget comes from Phase 2 (supervisor tests) and was not changed here.
- Human at the end of the phase: the open Portal comes to the front on a second launch (03-15); tray Close Applications ends the renderers, the Portal and the service and Open Sucrose is greyed while the Portal is open (03-16).
- Unverified: the foreground behavior on macOS and Linux.

**4. Optional modules start nothing when off; start with the OS.**
- Automated: `apps/sucrose/src/modules.rs` tests (no tray with AppVisible off, memory module only while a Portal is connected); a service run on a synthetic folder logs `module off` for tray, memory and fullscreen and no `module started` line before a Portal connects (03-13); `apps/sucrose/src/startup.rs` tests and `crates/sucrose-platform/tests/autostart.rs` (registry entry, LaunchAgent plist and XDG desktop entry text, the C# entry left alone).
- Human at the end of the phase: after a real sign-in with the entry on, Sucrose starts and the selected wallpaper resumes (03-23), on the maintainer's own settings.
- Unverified: the macOS LaunchAgent and the Linux autostart on real machines; the Windows Run value is the only one exercised.

**5. Theme, backdrop, background image, header, status bar, Help/About, bounded memory.**
- Automated: theme and backdrop mapping tests, `Capabilities.window_backdrop` (03-17); background decode limits (03-17); status bar and dialogs unit tests (03-18); thumbnail cache caps and decode queue (03-10).
- Runs: the memory rows of this plan (real library pass, synthetic 500 themes FAIL) in `docs/MEASUREMENTS.md`.
- Human at the end of the phase: light/dark toggle and OS following (03-17), Acrylic and background image with memory returning after Remove (03-17, 03-20), Help, About and Premium links (03-18), the tray hint after the first window close (03-18).
- Not met: bounded memory on the large library (Gap 1). Unverified: macOS and Linux theme, backdrop and background rendering, second-monitor cases beyond the 03-11 check, right-to-left mirroring (UI-06, Phase 5), cargo-deny license entries (CI job only, 03-03).

## Issues Encountered

- `$(cat file)` in the shell helper dropped the final newline of `docs/MEASUREMENTS.md` on the first write; caught in `git diff` (a changed last line) and restored before the commit.

## Known Stubs

None in the code of this plan (documentation only).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 3 plans are all executed; phase verification should read the Verification scope and Gaps above. A gap-closure plan is needed for the hover-preview defect and the paging memory growth, followed by a repeat of both D-25 scenarios.
- Phase 4 (engines) can start from the documented M2 state, but UI-07 stays open until the repeat passes.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
