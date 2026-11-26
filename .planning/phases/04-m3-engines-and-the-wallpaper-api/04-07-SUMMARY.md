---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 07
subsystem: engines
tags: [cef, web-profile, file-lock, localstorage, tls, certificates, ipc, config]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-06 virtual origin served by the CEF scheme handler; 04-02/05 CefHost::start with in-memory caches"
provides:
  - "sucrose_engine_web::profile::{acquire, ProfileLease, ProfileKind}: lock-file profile allocator (persistent <data_dir>/Web/<engine> or an emptied ephemeral slot folder)"
  - "paths::web_dir() = <data_dir>/Web, outside Cache (D-35)"
  - "CEF renderers run on the leased profile as root_cache_path and cache_path; renderer log line 'web profile' (kind, engine, path, slot)"
  - "Engine.AllowInvalidCertificates (default false) and Assignment.web: WebOptions (protocol v6)"
  - "CEF validates TLS certificates by default; Chromium's ignore-certificate-errors is appended only when the option is on"
  - "fixture Web theme 'storage' (localStorage counter plus /__storage/<n> request)"
  - "docs/DECISIONS.md D-037 and D-038; FEATURES rows for the TLS default and web data persistence"
affects: [04-08, 04-16, 04-20, round-b-webview2]
actuals:
  tokens: 13400
  tasks: 3
  commits: 5
plan_head_before: 8f0a6433276b51fbc0eb79d495ff62b9a4a74b71
plan_head_after: 369f926a8e6d4ac15f8bd4ac8412a6a45c0ab26d
tech-stack:
  added: []
  patterns:
    - "std File::try_lock as the cross-process profile lock; the OS releases it with the process"
    - "pure settings-value builder (settings_values) tested without CEF; the CEF call site only applies it"
key-files:
  created:
    - crates/sucrose-engine-web/src/profile.rs
    - crates/sucrose-engine-web/tests/profile.rs
    - apps/sucrose/tests/fixtures/web-themes/storage/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/storage/index.html
  modified:
    - crates/sucrose-core/src/paths.rs
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-config/src/model.rs
    - crates/sucrose-config/tests/migrate.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - docs/DECISIONS.md
    - docs/FEATURES.md
key-decisions:
  - "CEF 156 has no ignore_certificate_errors settings field, so the browser process appends Chromium's ignore-certificate-errors switch from on_before_command_line_processing, only when Allow invalid certificates is on"
  - "A failure to get any profile folder leaves the renderer on in-memory caches with a warning instead of failing the wallpaper"
  - "Ephemeral folders are emptied when acquired, not when dropped, as the plan states"
requirements-completed: []
coverage:
  - id: D1
    description: "Profile allocator: first acquire persistent at <web>/Cef under an OS lock, concurrent ones ephemeral and emptied, persistent again after the lease or the holding process is gone"
    requirement: ENG-03
    verification:
      - kind: integration
        ref: "cargo test -p sucrose-engine-web profile (8 tests incl. a second process holding the lock)"
        status: pass
    human_judgment: false
  - id: D2
    description: "A CEF wallpaper keeps localStorage across a service restart on the persistent profile; a second concurrent instance runs ephemeral"
    requirement: ENG-03
    verification:
      - kind: e2e
        ref: "storage fixture, two runs: Renderer log shows kind=Persistent twice and /__storage/1 then /__storage/2 (one monitor)"
        status: pass
    human_judgment: true
    rationale: "The Ephemeral half on a second monitor needs the maintainer's two-monitor setup; this run saw one monitor and so only the Persistent line"
  - id: D3
    description: "CEF validates TLS certificates unless Engine.AllowInvalidCertificates is on"
    requirement: TYPE-03
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine-cef certificate; cargo test -p sucrose-config; cargo test -p sucrose web_jobs_validate"
        status: pass
    human_judgment: true
    rationale: "A live bad-certificate page is network-dependent and is the human check of the plan 04-20 checkpoint"
  - id: D4
    description: "D-037 and D-038 recorded in DECISIONS; FEATURES carries the behavior changes"
    verification:
      - kind: other
        ref: "grep '^## D-037' and '^## D-038' docs/DECISIONS.md; git ls-files --eol; FEATURES last byte is not a newline"
        status: pass
    human_judgment: false
duration: 18 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 07: Web data that persists safely Summary

**A lock-file profile allocator gives the first CEF wallpaper a persistent <data_dir>/Web/Cef (localStorage survives a restart) and every concurrent one an emptied ephemeral profile, while TLS certificates are validated unless the new Engine.AllowInvalidCertificates key is on.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-11-26T03:03:22Z
- **Completed:** 2026-11-26T03:21:40Z
- **Tasks:** 3
- **Files modified:** 15 (4 created, 11 modified)

## Accomplishments

- `profile::acquire` (std only, `File::try_lock`) takes `<web_dir>/<engine>` when no running instance holds `.sucrose-profile.lock`, otherwise `<cache>/WebEphemeral/<engine>/<slot>` emptied at acquire. A test binary that spawns itself as a lock holder proves the cross-process case and that a killed holder's lock is released.
- `CefStart.profile` carries the lease; `CefHost::start` passes its folder as `root_cache_path` and `cache_path`, keeps `persist_session_cookies` on and holds the lease until after `cef::shutdown()`. The renderer logs `web profile kind=... engine="Cef" path=... slot=...`.
- Live proof (storage fixture, two service runs on a scratch data dir): both Renderer logs carry `web profile kind=Persistent`, `Web/Cef` and its lock file exist, and the origin saw `/__storage/1` then `/__storage/2`, so the counter persisted across the restart.
- `Engine.AllowInvalidCertificates` (default false, non-boolean reads false, migration writes false), protocol v6 `Assignment.web: WebOptions`, service `Job.web`, and a tested `settings_values` builder. A run with the key on starts normally and loads the page.
- D-037 and D-038 added to DECISIONS (LF kept); FEATURES Web row, two new API-table rows and the Security bullet updated (CRLF kept, no final newline).

## Task Commits

1. **Task 1: CEF profile rule (tracer)** - `c2fa36bb` (test, RED), `c953bc96` (feat, GREEN)
2. **Task 2: certificates validated by default** - `800cb23e` (test, RED), `d61c3dec` (feat, GREEN)
3. **Task 3: DECISIONS and FEATURES** - `369f926a` (docs)

**Plan metadata:** committed after this file (docs: complete plan).

## TDD Gate Compliance

Both TDD tasks have a `test(04-07)` commit followed by a `feat(04-07)` commit.
- Task 1 RED: seven of the eight `profile` tests failed on their planned assertions against a stub `acquire` (Ephemeral at the wrong path); the holder child is a helper that passes without its env variable. Semantic assessment: the target tests ran and failed on the planned behavior, not on build, setup or fixture faults.
- Task 2 RED: `allow_invalid_certificates_defaults_to_off_and_survives_a_round_trip` (true read back as false) and the CEF `certificate_errors_are_ignored_only_when_invalid_certificates_are_allowed` failed on assertions; the non-boolean test passed against the stub, as expected for a default-false key. Protocol types have no RED: they are plain data, covered by wire round-trip tests that pass on landing.

## Files Created/Modified

- `crates/sucrose-engine-web/src/profile.rs` - lock-based profile allocator
- `crates/sucrose-engine-web/tests/profile.rs` - eight tests including the two-process lock case
- `crates/sucrose-core/src/paths.rs` - `web_dir()` outside Cache
- `crates/sucrose-engine-cef/src/browser.rs` - lease held by `CefHost`, `profile_paths`, `settings_values`, certificate switch in the app handler
- `crates/sucrose-engine-cef/src/engine.rs` - `acquire_profile`, assignment option into `CefStart`
- `crates/sucrose-config/src/model.rs`, `tests/migrate.rs` - `AllowInvalidCertificates`
- `crates/sucrose-ipc/src/protocol.rs`, `apps/sucrose/src/service.rs`, `render.rs` - protocol v6, `WebOptions`, `Job.web`
- `apps/sucrose/tests/fixtures/web-themes/storage/` - the persistence fixture
- `docs/DECISIONS.md`, `docs/FEATURES.md` - D-037, D-038, feature rows

## Decisions Made

- Certificate switch via Chromium's `ignore-certificate-errors` command-line switch (see deviation 1).
- Profile acquisition failure degrades to in-memory caches (wallpaper still plays, keeps nothing) rather than a start failure.
- No new crate: the allocator uses std's file locking (Rust 1.99 toolchain).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Research premise wrong: CEF 156 has no `ignore_certificate_errors` settings field**
- **Found during:** Task 2
- **Issue:** 04-RESEARCH and the plan say the CEF settings struct has an ignore flag; the pinned bindings (`cef-dll-sys` 156.7.0) only mention it in a doc comment and `Settings` has no such field.
- **Fix:** the browser process appends Chromium's `ignore-certificate-errors` switch in `on_before_command_line_processing` (browser process only), and only when the option is on. The builder field is still named `ignore_certificate_errors`, so the plan's acceptance grep holds. No `OnCertificateError` override, as planned.
- **Files modified:** crates/sucrose-engine-cef/src/browser.rs
- **Verification:** unit test on the builder; a live run with the key on starts and loads the page. Effect on a real bad-certificate page is left to the 04-20 checkpoint.
- **Committed in:** d61c3dec

---

**Total deviations:** 1 auto-fixed (1 bug in the plan's premise)
**Impact on plan:** none on scope; D-037 records the mechanism.

## Issues Encountered

- The service started one renderer (one monitor, `DISPLAY1`; the virtual display was not active), so the second-run logs show only `Persistent`, as the acceptance criterion allows. The Ephemeral path is covered by the two-process test, not by a live two-monitor run; the maintainer's two-monitor check stays open for 04-20.
- `git status` listed `crates/sucrose-engine-cef/src/engine.rs` as modified at the start with an empty diff (line-ending artifact); it was not committed on its own.
- `rustfmt` rewrote edited CRLF files with LF; each was converted back to CRLF before committing.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- 04-08 can extend `WebOptions` (it already travels in the assignment) and 04-16 can add the Portal switch for `AllowInvalidCertificates`.
- Round B's WebView2 engine reuses `profile::acquire` with engine name `WebView2`.
- Open for the 04-20 checkpoint: two-monitor Persistent plus Ephemeral lines, and a live bad-certificate page with the switch off and on.
- ENG-03 stays partial and TYPE-03 is marked only by 04-20, so `requirements-completed` is empty.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED

Files, the five commits and the acceptance greps (web_dir 1, try_lock 1, AllowInvalidCertificates 9, ignore_certificate_errors 8, FEATURES last byte not 0a) were checked; `cargo xtask ci` exited 0.
