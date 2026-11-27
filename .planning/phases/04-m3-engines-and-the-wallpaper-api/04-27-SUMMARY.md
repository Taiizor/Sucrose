---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 27
subsystem: engines
tags: [webview2, installer, authenticode, ureq, prerequisites]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-18 verified install worker (VC++), 04-22 WebView2 runtime detection"
provides:
  - "Prerequisite::WebView2Runtime download address, silent arguments and re-detection in the verified install worker"
  - "installable() that follows the download address of the prerequisite a hint names"
  - "Ignored real-download test: Evergreen Bootstrapper accepted by the Microsoft signature check, flipped-byte copy refused"
  - "Portal tests for the WebView2 install title, Install/Repair actions and stage copies"
affects: [04-37, ENG-06]

actuals:
  tokens: 7285
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "One fwlink address for every architecture; the Bootstrapper picks the runtime for the device"
    - "Re-detection after an install reuses the report rule (webview2_is_fine over webview2_state)"

key-files:
  created: []
  modified:
    - apps/sucrose/src/install.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose-portal/src/dialogs/install.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - docs/FEATURES.md

key-decisions:
  - "installable() is architecture-aware: true only when the prerequisite named by the hint has a download address on this processor"
  - "The outdated-runtime button keeps the existing Repair label (existing keys only, no new English keys)"

requirements-completed: [ENG-06]

plan_head_before: 384b846f6651633da2ac285bb23ad76a142aa36e
plan_head_after: 735f7a5a8238dbaf56a570d5bcf054e82ccb26ff

coverage:
  - id: D1
    description: "The WebView2 Bootstrapper address, silent arguments and recorded run through run_install, Done only when the runtime is re-detected"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/install.rs#the_webview2_bootstrapper_has_one_https_address_for_every_architecture_and_silent_arguments"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/install.rs#a_webview2_install_runs_the_bootstrapper_silently_and_is_done_when_the_runtime_is_there"
        status: pass
    human_judgment: false
  - id: D2
    description: "The real Bootstrapper downloaded over HTTPS passes the Microsoft signature check and a copy with one byte changed is refused"
    requirement: ENG-06
    verification:
      - kind: integration
        ref: "cargo test -p sucrose -- --ignored webview2_bootstrapper_downloads_and_passes_the_signature_check"
        status: pass
    human_judgment: false
  - id: D3
    description: "A missing or outdated WebView2 Runtime is installable and the Portal dialog and row offer Install or Repair"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/engines.rs#a_missing_or_outdated_webview2_runtime_is_installable_where_a_download_exists"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/wallpaper.rs#wallpaper_install_a_missing_or_outdated_webview2_runtime_gets_install_or_repair_and_a_ready_one_nothing"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/install.rs#install_the_webview2_runtime_has_its_own_title_and_keeps_the_body_and_every_stage_copy"
        status: pass
    human_judgment: false
  - id: D4
    description: "A live install on a machine without the WebView2 Runtime"
    requirement: ENG-06
    verification: []
    human_judgment: true
    rationale: "The maintainer's machine has the runtime (158.0.4370.33), so the real Bootstrapper run (UAC, silent install) is not exercised; the proof is the real download plus signature check through the same worker"

duration: 9 min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 27: Installing the WebView2 Runtime Summary

**The verified install worker now installs the WebView2 Runtime: the Evergreen Bootstrapper comes from the legacy fwlink over HTTPS, passes the Microsoft Authenticode check (measured on the real file), runs as `/silent /install`, and the runtime is re-detected with the report's own rule.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-11-27T00:09:00Z
- **Completed:** 2026-11-27T00:18:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `Prerequisite::WebView2Runtime`: `https://go.microsoft.com/fwlink/p/?LinkId=2124703` for X86, X64 and Arm64 (None for other processors), arguments `/silent /install`, file `webview2_setup.exe`; `installed_now` asks `runtime_version()` through the new `engines::webview2_is_fine` (the same rule as `webview2_state`: newer than the floor).
- `installable` is now architecture-aware through `installable_on(hint, arch)`: runtime hints are installable where the download address exists, engine files never.
- The ignored test `webview2_bootstrapper_downloads_and_passes_the_signature_check` downloads the real Bootstrapper through `SystemSteps`, reaches the recorded run, and refuses a one-byte-changed copy. Measured result: signer reported by the check `Some("Microsoft Corporation")`; the changed copy gave `Err("the signature does not match the file")`. Both private folders are asserted gone (a `MustBeGone` guard, plus an empty `Install` folder).
- Portal: the dialog title reads "Install WebView2 Runtime", the body and the stage and failure copies are the VC++ ones, a missing runtime gets Install, an outdated one Repair, a ready one nothing. FEATURES row for prerequisite detection extended.

## Task Commits

1. **Task 1 (tracer, tdd) RED:** `74962d46` (test) - three tests failed on their assertions (address None, run never reached because of "no download")
2. **Task 1 GREEN:** `e3e9656c` (feat)
3. **Task 2:** `735f7a5a` (test)

**Plan metadata:** the docs commit that follows this file.

## Files Created/Modified

- `apps/sucrose/src/install.rs` - WebView2 address, arguments, re-detection, the ignored real-download test
- `apps/sucrose/src/engines.rs` - `installable_on`, `webview2_is_fine`, tests
- `apps/sucrose-portal/src/dialogs/install.rs`, `apps/sucrose-portal/src/settings/wallpaper.rs` - WebView2 install tests
- `docs/FEATURES.md` - Round B (04-27) sentence in the prerequisite detection row

## Decisions Made

- `installable` takes the processor into account (the plan's key link) instead of only excluding FilesMissing.
- Outdated runtime keeps the Repair label (no new English keys; the hint text says "Update it").
- No new locale key was needed, so `sucrose.en.yml` is unchanged.

## Deviations from Plan

None - plan executed exactly as written, with these notes:

- **Task 2 had no RED phase.** The Portal install title, keys and `install_action` arms for WebView2 were already in place from earlier rounds (04-18, 04-22), so the new tests pin existing behavior and passed on first run; only the test and FEATURES text changed. `sucrose.en.yml` and non-test Portal code did not change.
- The ureq agent already used `https_only(true)`, which also refuses a redirect to plain HTTP, so the redirecting fwlink needed no download change.
- Task 1 RED evidence is the cargo test output (three assertion failures), not a `tdd-red-evidence` record: cargo's libtest text is not one of the formats that classifier reads.

## Issues Encountered

None. `cargo xtask ci` passed (exit 0). The two `crates/sucrose-engine-web` files show as modified (line-ending artifact from 04-25) and were not touched.

## Line endings

Every file this plan edited except `docs/FEATURES.md` is LF in the working copy and stayed LF; `docs/FEATURES.md` stayed CRLF with no final newline. `cargo fmt` changed no CRLF files, so nothing needed restoring.

## Requirement status

ENG-06: Complete rule case 1 - the run finished, the real download and signature test, the detection, installable and dialog tests all passed. A live install on a machine without the runtime was not run (the maintainer has WebView2 Runtime 158.0.4370.33); that is recorded as D4 for /gsd-verify-work.

## Known Stubs

None.

## Threat Flags

None. T-04-60 (HTTPS only, size cap, private folder, signature before run, flipped-byte refusal) and T-04-61 (only after the user's click, ShellExecuteEx default verb, no self-elevation) hold; no crate was added.

## Next Phase Readiness

Ready for 04-28.

## Self-Check: PASSED

- apps/sucrose/src/install.rs, apps/sucrose/src/engines.rs, the two Portal files and docs/FEATURES.md exist and are committed.
- Commits 74962d46, e3e9656c and 735f7a5a are ancestors of HEAD.
