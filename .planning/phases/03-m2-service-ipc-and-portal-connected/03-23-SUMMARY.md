---
phase: 03-m2-service-ipc-and-portal-connected
plan: 23
subsystem: platform
tags: [startup, autostart, registry, launchagent, xdg, ipc, portal, tray-03]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "PortalRequest::SetStartup/RefreshStartup and PortalEvent::Startup(StartupInfo) (plan 06); SettingsHost::update (plan 07); spawn_detached (plan 08); the General page groups and keys (plans 03, 20)"
provides:
  - "crates/sucrose-platform/src/autostart.rs: RUN_VALUE_NAME, LAUNCH_AGENT_LABEL, DESKTOP_FILE_NAME, launch_agent_plist, desktop_entry, run_value_data, disabled_by_os, config_dir_from, EntryKind file entries, enable_autostart, disable_autostart, autostart_state, legacy_entry, AutostartState, Capabilities.autostart"
  - "crates/sucrose-platform/src/os_windows/autostart.rs: HKCU Run value, StartupApproved reset, read-only C# entry detection"
  - "apps/sucrose/src/startup.rs and the service: SetStartup (Normal/None work, Priority/Scheduler refused), RefreshStartup with a worker-thread C# lookup, Startup answers, log lines 'startup entry written' and 'startup entry removed'"
  - "Portal: Startup at Application Launch row with startup_options, startup_note, refresh on open"
  - "docs/DECISIONS.md D-034, docs/FEATURES.md startup row"
affects: [03-24, 03-25, 03-26, phase-05-installer, phase-05-updater, TRAY-04]

actuals:
  tokens: 22900
  tasks: 2
  commits: 6
plan_head_before: 020f1962f7183818a836687e88199e5c2c0b059b
plan_head_after: 977c6510bb405a7da3b85bafc590fb8e96fb22dc

tech-stack:
  added: []
  patterns:
    - "OS start entries as pure text builders that run and are tested on every OS, plus thin per-OS writers (registry on Windows, files elsewhere) behind enable/disable/state"
    - "The service's decisions sit behind a StartupOs trait so rollback rules are unit-tested with a fake instead of the registry"
    - "A lookup that starts a process (schtasks /Query) runs on its own worker thread and comes back as an Event; it runs only when the Portal asks, never polled"

key-files:
  created:
    - crates/sucrose-platform/src/autostart.rs
    - crates/sucrose-platform/src/os_windows/autostart.rs
    - crates/sucrose-platform/tests/autostart.rs
    - apps/sucrose/src/startup.rs
  modified:
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/src/unsupported.rs
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/general.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/DECISIONS.md
    - docs/FEATURES.md

key-decisions:
  - "D-034: the entry names Sucrose.Service, dev.sucrose.Sucrose and sucrose.desktop (settled by the maintainer before execution, used exactly)"
  - "entry_present means the entry starts this executable (Run value data equal to the quoted current_exe, or the entry file equal to what would be written), so an entry left behind by a moved install reads as missing"
  - "A setting that cannot be saved takes the entry change back (a new entry is removed, a removed one is written again), so the entry and RunStartup never disagree"
  - "The C# entry lookup runs on a startup-legacy thread and its last answer is reused by the other replies; the main thread never waits for schtasks"
  - "Desktop Entry text applies the quoting rule and then the string rule (and writes line breaks as escapes), so no character of a path can start another key"

patterns-established:
  - "Pattern: the ignored real-HKCU tests are serialized, leave a pre-existing Sucrose.Service or Sucrose value alone and clean up after themselves"

requirements-completed: [TRAY-03]

coverage:
  - id: D1
    description: "Choosing Normal makes the service write Sucrose's own start entry (Windows Run value Sucrose.Service with the quoted exe, no arguments) and store RunStartup 1; None removes only that entry"
    requirement: "TRAY-03"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#normal_writes_the_entry_and_stores_one"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#none_removes_the_entry_and_stores_zero"
        status: pass
      - kind: integration
        ref: "crates/sucrose-platform/tests/autostart.rs#registry::autostart_registry_enable_disable_leaves_the_csharp_entry_alone (ignored, run with --ignored)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The LaunchAgent plist and the .desktop text escape the executable path and cannot be split by it, tested on every OS"
    requirement: "TRAY-03"
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/tests/autostart.rs#autostart_plist_carries_the_label_the_escaped_path_and_run_at_load"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/tests/autostart.rs#autostart_desktop_entry_escapes_quote_backslash_backtick_and_dollar"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/tests/autostart.rs#autostart_desktop_entry_doubles_percent_and_cannot_be_split_by_a_newline"
        status: pass
    human_judgment: false
  - id: D3
    description: "Priority and Scheduler create no entry and change nothing; migration creates no entry; the two options are disabled with the later tip"
    requirement: "TRAY-03"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#priority_and_scheduler_are_refused_and_change_nothing"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#set_startup_priority_and_scheduler_are_refused_and_change_nothing"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/general.rs#settings_startup_options_disable_priority_and_scheduler_with_the_later_tip"
        status: pass
    human_judgment: false
  - id: D4
    description: "The C# Run value and scheduled task are only detected (registry read, schtasks /Query), never written, changed or removed"
    requirement: "TRAY-03"
    verification:
      - kind: other
        ref: "grep -c -E 'schtasks[^\\n]*(/Delete|/Change|/Create)' crates/sucrose-platform/src/os_windows/autostart.rs prints 0"
        status: pass
      - kind: integration
        ref: "crates/sucrose-platform/tests/autostart.rs#registry::autostart_registry_enable_disable_leaves_the_csharp_entry_alone (ignored, run with --ignored)"
        status: pass
    human_judgment: false
  - id: D5
    description: "Enabling clears Sucrose's own Task Manager flag; the row tells when Task Manager turned the entry off"
    requirement: "TRAY-03"
    verification:
      - kind: integration
        ref: "crates/sucrose-platform/tests/autostart.rs#registry::autostart_registry_enable_clears_the_task_manager_flag (ignored, run with --ignored)"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/general.rs#settings_startup_an_entry_task_manager_turned_off_says_so"
        status: pass
    human_judgment: false
  - id: D6
    description: "After sign-in with the entry on, sucrose starts with no argument and the selected wallpaper resumes; the row looks right in the window (requested value at once, one-line labels, wrapping notes)"
    requirement: "TRAY-03"
    verification: []
    human_judgment: true
    rationale: "Sign-in behavior needs a real logon and the row needs a look at the window; the plan's human-check runs at the end of the phase with the maintainer's own settings"

duration: 21min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 23: Start with the OS Summary

**Normal startup through the service on all three OSes: a Run value "Sucrose.Service" (plus a Task Manager flag reset) on Windows, a LaunchAgent and an XDG autostart file with tested escaping elsewhere, a read-only look at the C# entry, refused Priority and Scheduler, the Portal's Startup row with its notes, and D-034 fixing the names for Phase 5**

## Performance

- **Duration:** 21 min
- **Started:** 2026-11-24T11:30:57Z
- **Completed:** 2026-11-24T11:51:30Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments

- `sucrose-platform` writes and removes Sucrose's own start entry: HKCU Run value `Sucrose.Service` with the quoted `current_exe` and no arguments, a LaunchAgent `dev.sucrose.Sucrose`, and `~/.config/autostart/sucrose.desktop`. The text builders are pure and tested on every OS; a quote in a Windows path is refused, and no character of a path can add a key to a desktop file.
- The service answers `SetStartup` and `RefreshStartup` with a `Startup` event: Normal and None work, Priority and Scheduler change nothing, a failed settings save takes the entry change back, and the C# lookup (a registry read and `schtasks /Query`) runs on its own thread only when the General page opens.
- The General page has the Startup select with Priority and Scheduler disabled ("Available in a later version") and notes in the row description: not available yet, no entry, turned off in Task Manager, and "both apps will start together". The page asks `RefreshStartup` each time it opens.
- D-034 and the FEATURES.md startup row record the names and behavior the Phase 5 updater and uninstaller must find.

## Task Commits

1. **Task 1: Normal startup writes Sucrose's own entry through the service** - `e2eb75ac` (test, RED), `785344cd` (feat, platform), `bb9fd843` (test, RED), `762e9c21` (feat, service)
2. **Task 2: Legacy entry detection, the Startup row with its notes, D-034 and FEATURES** - `af19c340` (test, RED), `977c6510` (feat). The Windows legacy detection itself landed with `785344cd` because it shares the registry helpers.

**Plan metadata:** the docs commit that follows this file (docs: complete plan)

## Files Created/Modified

- `crates/sucrose-platform/src/autostart.rs` - constants, pure builders, file entries, the public autostart API
- `crates/sucrose-platform/src/os_windows/autostart.rs` - Run value, StartupApproved, read-only C# detection
- `crates/sucrose-platform/src/unsupported.rs`, `os_linux.rs`, `os_macos.rs` - the file-based entry for macOS and Linux and `Capabilities.autostart`
- `crates/sucrose-platform/tests/autostart.rs` - builders, file entries, ignored real-HKCU tests
- `apps/sucrose/src/startup.rs` - SetStartup decisions with rollback and the Startup info, behind a StartupOs trait
- `apps/sucrose/src/service.rs`, `main.rs`, `portal_link.rs` - request handling, the `StartupLegacy` event, module wiring
- `apps/sucrose/tests/portal.rs` - RefreshStartup and refused SetStartup against the real binary
- `apps/sucrose-portal/src/settings/general.rs`, `app.rs`, `i18n.rs`, `locales/sucrose.en.yml` - the row, notes, refresh on open, two new keys
- `docs/DECISIONS.md`, `docs/FEATURES.md` - D-034 and the startup row

## Decisions Made

- The names are the ones the maintainer settled (D-034); nothing else was used.
- `entry_present` compares against this executable, so a stale entry of a moved install is reported as missing and the row says so.
- Rollback keeps the entry and RunStartup consistent when the settings cannot be saved.
- The Task Manager flag that cannot be reset on enable only logs a warning: the entry is written and the row then says Task Manager turned it off.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Service logic in a new module `apps/sucrose/src/startup.rs`**
- **Found during:** Task 1
- **Issue:** service.rs is already about 5,000 lines; the SetStartup decisions, the OS seam and their tests belong in a module like `library_ops.rs` and `lifecycle.rs`.
- **Fix:** `startup.rs` holds `set_startup`, `startup_info`, the `StartupOs` trait and 10 unit tests; service.rs only wires the requests and the `StartupLegacy` event.
- **Files modified:** apps/sucrose/src/startup.rs, apps/sucrose/src/main.rs, apps/sucrose/src/service.rs
- **Committed in:** 762e9c21

**2. [Rule 2 - Missing Critical] A note for "Normal without an entry" and a text for "turned off in Task Manager"**
- **Found during:** Task 2
- **Issue:** the UI-SPEC startup notes cover the C# entry and Priority/Scheduler only. Migration copies RunStartup 1 without creating an entry, so the select would read Normal while Sucrose does not start; the plan's own "Task Manager disabled" note had no string.
- **Fix:** two new keys, `ApplicationStartup.NoEntry` and `ApplicationStartup.DisabledByOs`, in `sucrose.en.yml` and `REQUIRED_KEYS`, used by `startup_note`.
- **Files modified:** apps/sucrose-portal/locales/sucrose.en.yml, apps/sucrose-portal/src/i18n.rs, apps/sucrose-portal/src/settings/general.rs
- **Committed in:** 977c6510

**2a. [Rule 3 - Blocking] Files outside the plan's list**
- **Found during:** Tasks 1 and 2
- **Issue:** the Portal must send `RefreshStartup` when the page opens and show the `Startup` event (app.rs); `Capabilities.autostart` and the macOS/Linux entry need os_linux.rs, os_macos.rs and lib.rs; the service has no `request_kind` caller left once every request has an arm, so the helper and its import were removed, and the Portal's catch-all event arm became unreachable and was removed.
- **Files modified:** apps/sucrose-portal/src/app.rs, crates/sucrose-platform/src/os_linux.rs, os_macos.rs, lib.rs, apps/sucrose/src/portal_link.rs
- **Committed in:** 785344cd, 762e9c21, 977c6510

---

**Total deviations:** 3 auto-fixed (1 missing critical, 2 blocking)
**Impact on plan:** No scope change; every deviation supports the planned behavior.

## Issues Encountered

- The ignored registry tests failed when run in parallel because they share the same registry values; they now take a lock. Verified three runs in a row, and `Sucrose.Service` is absent from the Run and StartupApproved keys afterwards.
- Working-copy line endings: the platform crate and `docs/FEATURES.md` are CRLF, the rest LF; `file().text()` normalizes endings, so edits were written back with each file's original endings. `git status` still lists three platform files as modified with an empty diff (a line-ending stat artifact, not a change).

## Known Stubs

None.

## Threat Flags

None. The only process started is `schtasks.exe /Query` from the system folder (T-03-55), already in the plan's threat model.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 24 to 26 can rely on `SetStartup`/`Startup` and D-034.
- Open for the end-of-phase human check: sign out and in with Normal on (the selected wallpaper resumes, the C# entry is unchanged), then None removes the `Sucrose.Service` value. The Startup row was not captured in a window in this run; macOS and Linux entries are verified as text and file locations only (assumption A12).

## Self-Check: PASSED

- Created files exist: autostart.rs (both), tests/autostart.rs, startup.rs.
- Commits found on HEAD: e2eb75ac, 785344cd, bb9fd843, 762e9c21, af19c340, 977c6510.
- `cargo xtask ci` exit 0; ignored HKCU tests pass three times and leave nothing behind.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
