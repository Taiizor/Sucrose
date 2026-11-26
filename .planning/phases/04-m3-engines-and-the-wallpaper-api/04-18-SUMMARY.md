---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 18
subsystem: engines
tags: [authenticode, wintrust, installer, ureq, ipc, gpui, prerequisites]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "EngineState::Missing { hint, installable } and HintView::VcRedistMissing (04-12), the row alert and the warning toast (04-15)"
provides:
  - "Platform::verify_microsoft_signature (WinVerifyTrust, Microsoft Corporation leaf, Microsoft application root) and Platform::run_installer (ShellExecuteEx, UAC by Windows)"
  - "apps/sucrose/src/install.rs: Prerequisite, InstallStage, InstallFailure, Steps, InstallWorker, SystemSteps"
  - "PortalRequest::{InstallPrerequisite, CancelInstall} and PortalEvent::InstallProgress (appended)"
  - "dialogs::install::open_install_dialog with the pure dialog_state, Install/Repair on the engine row alert and the fallback toast"
affects: [04-19, 04-20, 04-21, round-b-webview2, phase-verifier]
plan_head_before: bdd73edd7acce571ba3357784b261c234ef0d6e3
plan_head_after: 9ec3e6a081134366a07a6d2e6198bbbb224b8219
actuals:
  tokens: 22400
  tasks: 3
  commits: 4
tech-stack:
  added: ["ureq (already a workspace dependency) in apps/sucrose", "windows features Win32_Security_Cryptography(+_Catalog,_Sip) and Win32_Security_WinTrust in sucrose-platform"]
  patterns:
    - "Injectable Steps trait (download, verify, run, installed) so the stage order and cleanup are tested with fakes"
    - "Dialog content rebuilt on every render from a shared stage; the Portal event handler routes service reports through a gpui Global"
key-files:
  created:
    - crates/sucrose-platform/src/os_windows/trust.rs
    - crates/sucrose-platform/src/os_windows/installer.rs
    - crates/sucrose-platform/tests/trust.rs
    - apps/sucrose/src/install.rs
    - apps/sucrose-portal/src/dialogs/install.rs
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/engines.rs
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/dialogs/mod.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/toast.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
key-decisions:
  - "The Microsoft root policy is run with MICROSOFT_ROOT_CERT_CHAIN_POLICY_CHECK_APPLICATION_ROOT_FLAG: with default flags it refused the root of the real Visual C++ bootstrapper (CERT_E_UNTRUSTEDROOT at the root element)"
  - "Revocation is checked online; when no revocation source answers the verification is repeated without it (signature, digest, signer and Microsoft root still checked), while a revoked certificate is always refused"
  - "A Cancelled stage ends a stopped install, so the Portal never waits for a result that does not come; a stop that arrives after the download still keeps the installer from running"
  - "The installer runs with /install /passive /norestart and exit codes 0, 1638 and 3010 count, the registry check afterwards decides"
requirements-completed: []
duration: 28 min
completed: 2026-11-26
status: complete
coverage:
  - id: D1
    description: "A Microsoft-signed installer passes the signature check; a tampered copy, an unsigned file and a missing file are refused"
    requirement: ENG-06
    verification:
      - kind: integration
        ref: "crates/sucrose-platform/tests/trust.rs (4 tests, real vc_redist.x64.exe via SUCROSE_TRUST_SAMPLE)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service downloads over HTTPS, verifies, runs, deletes and re-detects, and can be cancelled; a refused signature deletes the file and never runs it"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/install.rs tests (fake steps: stages, signature refusal, download failure, installer failure, cancel, late stop, URLs per architecture)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-ipc/src/portal.rs install_messages_survive_the_wire and the wire index tests"
        status: pass
    human_judgment: false
  - id: D3
    description: "The install dialog states, Install or Repair only for installable hints, FilesMissing without a button"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose-portal dialogs::install::tests and settings::wallpaper::tests::wallpaper_install_*"
        status: pass
    human_judgment: false
  - id: D4
    description: "How the 480 px dialog, its progress bar and the Install button in the alert and toast look and behave on screen, and the real end-to-end install (UAC, Microsoft installer)"
    verification: []
    human_judgment: true
    rationale: "No window capture was made and the maintainer's machine already has the Redistributable, so the real install and the visual result are unproven here; the phase verifier flags it for a machine without the prerequisite"
---

# Phase 4 Plan 18: Verified prerequisite install Summary

**A missing VC++ Redistributable can be installed from the Portal: the Microsoft bootstrapper is downloaded over HTTPS only after the user confirms, accepted only when WinVerifyTrust and the Microsoft Corporation signer and root checks pass, run through the shell so Windows shows UAC, and the engine is re-detected.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-11-26T16:39:00Z
- **Completed:** 2026-11-26T17:07:00Z
- **Tasks:** 3 (tracer, auto, auto)
- **Files modified:** 20 (plus Cargo.lock)

## Accomplishments

- `Platform::verify_microsoft_signature` on Windows: WinVerifyTrust (no UI, whole-chain revocation), leaf common name exactly `Microsoft Corporation`, Microsoft application root policy; the real `vc_redist.x64.exe` is accepted, a copy with one byte flipped is refused, an unsigned file is `NotSigned`, a missing file is `Io`.
- `Platform::run_installer`: ShellExecuteEx with the default verb (Windows decides on UAC), waits up to 30 minutes, returns the exit code. Sucrose never elevates itself.
- Install worker in the service: HTTPS-only ureq download (15 s connect, 30 s per read, 30 min total, 128 MiB cap) into a private `Cache/Install/<unique>` folder removed on every path, progress every 250 ms, one install at a time, Stop Download, a stop before the installer starts still wins, a panic reports a failed installer. After Done the service re-detects and the Portal gets a new engine report.
- Portal: install dialog (confirm, download with bar and Stop Download, signature, installing, checking again, three failure copies with Try Again and Close; Escape only in the first and failure states), Install/Repair buttons on the row alert and the fallback toast only when `installable`.
- FEATURES prerequisite row records the install path.

## Task Commits

1. **Task 1 (tracer): signature check** - `fdf447f5` (test, RED: 4 tests failing with Unsupported), `a7d75a10` (feat, GREEN)
2. **Task 2: install worker, wire messages, service** - `2bc105cd` (feat)
3. **Task 3: dialog and buttons** - `9ec3e6a0` (feat)

**Plan metadata:** the docs commit that carries this file.

Tracer gate (auto mode off, HUMAN_VERIFY_MODE end-of-phase, automated-only verify): the trust tests were re-run end to end with the sample set (4 passed) before expansion.

## Files Created/Modified

See the frontmatter. Line endings: `crates/sucrose-platform/Cargo.toml` became LF through a tool write and was restored to CRLF before its commit; `apps/sucrose/{Cargo.toml,main.rs,service.rs}` and `docs/FEATURES.md` (no newline after the last line, kept) stayed CRLF; the platform sources `lib.rs`, `os_windows.rs`, `engines.rs`, `portal.rs` and all Portal files were LF and stayed LF. `cargo fmt` changed no line endings.

## Decisions Made

See key-decisions. The plan's wording "chain checked with the Microsoft-root chain policy" needed the application-root flag; revocation soft-fail is documented in `trust.rs` and FEATURES.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Microsoft root policy refused the real bootstrapper's root**
- **Found during:** Task 1
- **Issue:** `CertVerifyCertificateChainPolicy(CERT_CHAIN_POLICY_MICROSOFT_ROOT)` with default flags returned CERT_E_UNTRUSTEDROOT for the root element of the Visual C++ bootstrapper.
- **Fix:** pass `MICROSOFT_ROOT_CERT_CHAIN_POLICY_CHECK_APPLICATION_ROOT_FLAG` (0x20000); the genuine file then passes and the tampered one still fails at the digest.
- **Files modified:** crates/sucrose-platform/src/os_windows/trust.rs
- **Committed in:** a7d75a10

**2. [Rule 2 - Missing critical] A `Cancelled` stage**
- **Found during:** Task 2
- **Issue:** the plan's stage list had no end for a stopped download, so the Portal would wait for an event that never comes, and a stop arriving after the download would still run the installer.
- **Fix:** `InstallStage::Cancelled` / `InstallStageView::Cancelled`; the worker checks the stop flag again before it starts the installer.
- **Committed in:** 2bc105cd

**3. [Rule 3 - Blocking] Supporting changes outside files_modified**
- `apps/sucrose-portal/src/i18n.rs`: the new keys must be listed in `REQUIRED_KEYS` (a test fails otherwise). `apps/sucrose-portal/src/toast.rs`: `show_warning` takes a list of actions so the toast can hold Install next to the snooze. `apps/sucrose/src/engines.rs`: `vc_is_fine` exposes the existing rule for the re-check. `crates/sucrose-platform/src/os_windows/installer.rs` is a new file for the shell launch; `unsupported.rs` needed no change because the two new trait methods default to Unsupported. `Cargo.lock` gained the ureq edge for `sucrose`.
- **Committed in:** 2bc105cd, 9ec3e6a0

**4. [Process] TDD order for Tasks 2 and 3**
- Task 1 has separate RED and GREEN commits. For Tasks 2 and 3 the tests were written together with the implementation and committed once; no separate failing commit exists.

---

**Total deviations:** 4 (1 bug, 1 missing critical, 1 blocking, 1 process)
**Impact on plan:** none on behavior beyond the Cancelled stage.

## Authentication Gates

None.

## Issues Encountered

None. No known flaky test failed (portal.rs timing tests, memory_detail, cef-dll-sys os error 5 did not appear). `cargo xtask ci` passed (exit 0).

## Known Stubs

None. `Prerequisite::WebView2Runtime` has no download address on purpose until Round B (the worker answers a download failure); no button offers it in Round A.

## Threat Flags

None beyond the plan's register: the service now has an HTTP client (T-04-42, HTTPS-only with a size cap, the signature check is the authority) and starts a shell process (T-04-43, only after the user confirms). The service does not call out until the user presses Download and Install.

## Flagged for the verifier

- The real end-to-end install (UAC, Microsoft installer, re-detection) was not run: the maintainer's machine has the Redistributable. The dialog was not captured on screen. Both are proven only by tests and fakes plus the real signature check.
- ENG-06 stays partial (WebView2 Runtime in Round B), so `requirements-completed` is empty.

## Next Phase Readiness

Round B adds the WebView2 Runtime to the same path: a download address in `Prerequisite::url`, its detection in `installed_now`, and `installable` in the engine report.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED

All created files exist and the four task commits are ancestors of HEAD.
