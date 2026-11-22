---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 02
subsystem: docs
tags: [engines, libmpv, decisions, licensing, gpl]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 01 fetch-engines pins and the libmpv loader (vulkan-1.dll import finding)"
provides:
  - "D-028: engine binaries ship inside the Sucrose package (supersedes D-005)"
  - "Maintainer decisions recorded: upstream-now pin source, GPL build signed off"
  - "Every doc D-16 lists (and two it missed) no longer say engines are never bundled"
affects: [phase-02, phase-04, phase-05-packaging]
actuals:
  tokens: 6500
  tasks: 3
  commits: 2
commits: 2
plan_head_before: bf25ff2d95d5b19e0bb5e79ab874f98514b93d6d
plan_head_after: 1356acae8111db2b6f90fce74dbfd64a1ca7e315
tech-stack:
  added: []
  patterns: ["Decision recorded with the docs in the same change (CLAUDE.md Keeping the docs true)"]
key-files:
  created: []
  modified:
    - docs/DECISIONS.md
    - CLAUDE.md
    - .claude/rules/platform-ffi.md
    - .claude/rules/cargo.md
    - docs/ARCHITECTURE.md
    - docs/FEATURES.md
    - .planning/PROJECT.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
key-decisions:
  - "D-028: engine binaries ship in the package; fetched and SHA256-verified by cargo xtask fetch-engines, loaded at runtime (D-013)"
  - "Pin source stays on the upstream shinchiro release for now; mirroring before Phase 5 packaging is an open question"
  - "The GPL-2.0-or-later libmpv build may ship inside the GPL-3.0-only package; license texts and a source offer are added in Phase 5"
  - "The loader PATH step searches only absolute PATH entries (narrows D-18)"
requirements-completed: [ENG-01]
coverage:
  - id: D1
    description: "D-028 entry with pin facts, GPL sign-off, mirror answer and vulkan-1.dll import note; D-005 superseded; D-006 caveat updated"
    requirement: "ENG-01"
    verification:
      - kind: other
        ref: "grep -c '^## D-028' docs/DECISIONS.md == 1; grep -c 'Superseded by D-028' docs/DECISIONS.md == 1"
        status: pass
    human_judgment: false
  - id: D2
    description: "Agent-facing rules (CLAUDE.md, platform-ffi, cargo) and planning/architecture docs consistently cite D-028"
    requirement: "ENG-01"
    verification:
      - kind: other
        ref: "task acceptance greps for Tasks 2 and 3; cargo xtask ci exit 0"
        status: pass
    human_judgment: true
    rationale: "Wording accuracy across documents is a judgment call; grep proves old phrases are gone, not that new text is right"
duration: 4min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 02: Engine delivery decision (D-028) Summary

**D-028 records that libmpv and the other engine binaries ship inside the Sucrose package from SHA256-pinned upstream builds, with the GPL-2.0-or-later sign-off, and every doc that said "never bundled" now follows it.**

## Performance

- **Duration:** about 4 min for this continuation (Task 1 checkpoint ran in the previous executor)
- **Completed:** 2026-11-22
- **Tasks:** 3 (1 checkpoint decision, 2 auto)
- **Files modified:** 9

## Accomplishments
- Maintainer answered the checkpoint: pin source "upstream-now", "GPL: yes"; recorded in D-028 (engines.toml URLs and sha256/size pins unchanged, verified by empty `git diff -G` checks).
- D-028 added after D-027 with decision, pin facts (shinchiro 20261111, mpv v0.42.0-38-gbcde6b93c, client API 2.6, sizes, GPL, source offer coordinates, no LGPL i686 alternative), the vulkan-1.dll normal-import consequence (`LoadError::LoadFailed` without a Vulkan loader), and the narrowed PATH search; D-005 is Superseded, the D-006 caveat points at the bundled mpv; an Open questions bullet tracks mirroring before Phase 5.
- CLAUDE.md non-negotiable 3, `.claude/rules/platform-ffi.md`, `.claude/rules/cargo.md`, PROJECT.md, REQUIREMENTS.md (ENG-01, ENG-06, ENG-07, out-of-scope row), ROADMAP.md notes, ARCHITECTURE.md (status line adds `sucrose-engine-mpv`) and FEATURES.md (libmpv row) updated.
- `cargo xtask fetch-engines --arch all` re-verified all three pins; `cargo xtask ci` passes.

## Task Commits

1. **Task 1: pin source and GPL sign-off** - no commit (checkpoint:decision; answers recorded in Task 2)
2. **Task 2: D-028, rules** - `65d41429` (docs)
3. **Task 3: planning and architecture docs** - `1356acae` (docs)

## Files Created/Modified
- `docs/DECISIONS.md` - D-028, D-005 superseded, D-006 caveat, open question
- `CLAUDE.md`, `.claude/rules/platform-ffi.md`, `.claude/rules/cargo.md` - agent-facing rules
- `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `docs/ARCHITECTURE.md`, `docs/FEATURES.md` - consistent wording

## Decisions Made
See key-decisions. `xtask/engines.toml` needed no edit under upstream-now.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Additional docs still said engines are never bundled**
- **Found during:** Task 3
- **Issue:** The must-have "no project doc still says Sucrose never bundles engines" was not met by the listed edits alone: REQUIREMENTS.md "Out of Scope" table row, the ENG-07 citation of D-005, the Phase 4 notes in ROADMAP.md ("never bundled or installed (D-005)") and the ARCHITECTURE.md Goals line ("Engines are optional and user-installed").
- **Fix:** Reworded each to cite D-028 (ENG-07 now cites D-006, D-028).
- **Files modified:** .planning/REQUIREMENTS.md, .planning/ROADMAP.md, docs/ARCHITECTURE.md
- **Committed in:** 1356acae

**Total deviations:** 1 auto-fixed (Rule 2). **Impact:** wording only, no scope creep.

## Issues Encountered
- The REPL file reader returns LF text for CRLF files; edits were applied with a byte-level helper that preserves CRLF, and the diffs are minimal (1-10 lines per file).

## User Setup Required
None. The maintainer still has to decide on a mirror before Phase 5 (Open question in DECISIONS.md).

## Known Stubs
None.

## Next Phase Readiness
- Decision and docs are in place; remaining Phase 2 plans can rely on bundled libmpv.
- Open: mirror the pinned archives before upstream prunes release 20261111 (about a month of retention); `--from` with the local archive works offline meanwhile.

## Self-Check: PASSED
- Commits 65d41429 and 1356acae are ancestors of HEAD; modified files exist; acceptance greps and `cargo xtask ci` pass.
