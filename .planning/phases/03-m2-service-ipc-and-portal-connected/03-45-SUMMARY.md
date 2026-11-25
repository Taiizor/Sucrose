---
phase: 03-m2-service-ipc-and-portal-connected
plan: 45
subsystem: ui
tags: [gap-closure, backdrop, dwm, mica, tabbed, acrylic, gpui, raw-window-handle]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "Root fill of 03-41 and the maintainer's finding in 03-43 that Mica and Tabbed changed nothing (WR-14, G-03-5)"
provides:
  - "Platform::set_window_backdrop: the DWM system backdrop type (none, main window, tabbed window) set through sucrose-platform, HRESULT returned"
  - "theme::appearance_steps: Opaque before a changed GPUI appearance, so an earlier Acrylic accent policy is cleared"
  - "apply_backdrop logs steps and dwm_result on the 'backdrop applied' line"
affects: [03-47, backdrop, sucrose-platform, sucrose-portal]

actuals:
  tokens: 5980
  tasks: 2
  commits: 2
plan_head_before: 5f09582a8378b1b56e3d52278e66f0ec0a9e02f1
plan_head_after: 22e7c33662cdf57988b1da5709fe37e6df39af80

tech-stack:
  added: [raw-window-handle 0.6.2 (workspace dependency, already locked by GPUI)]
  patterns:
    - "Every OS feature behind the Platform trait: a defaulted Unsupported method, the Windows override holds the single unsafe call"
    - "The view remembers the last GPUI appearance it set, because Window has no public getter"

key-files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/theme.rs
    - docs/GPUI.md
    - docs/ARCHITECTURE.md

key-decisions:
  - "The DWM system backdrop type is set by Sucrose through Platform::set_window_backdrop on every apply, because gpui-pre-windows 0.3.12 sets it only for Mica and Tabbed, discards the HRESULT and never resets it (WR-14)"
  - "Every changed GPUI appearance goes through Opaque first (theme::appearance_steps); an unchanged one sets nothing, an unknown previous counts as changed"
  - "raw-window-handle 0.6.2 added to the root workspace dependencies; Cargo.lock only gains it in the dependency list of sucrose-platform (no new package)"

patterns-established:
  - "dwm_result in the Portal log: 0x00000000 accepted, HRESULT hex when refused, 'skipped' where the capability is false"

requirements-completed: [UI-01, UI-02]

duration: 11 min
completed: 2026-11-25
status: complete

coverage:
  - id: D1
    description: "apply_backdrop resets the window to Opaque before a changed appearance (Acrylic cleared before Mica or Tabbed)"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_appearance_steps_resets_before_a_changed_target"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_appearance_steps_is_empty_when_nothing_changes"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_appearance_steps_treats_an_unknown_previous_as_changed"
        status: pass
    human_judgment: false
  - id: D2
    description: "The DWM system backdrop type is set through Platform::set_window_backdrop and its result is logged"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows.rs#backdrop_type_maps_every_setting"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows.rs#set_window_backdrop_refuses_a_window_without_a_handle"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/app.rs#app_dwm_result_text_names_each_outcome"
        status: pass
      - kind: integration
        ref: "release Portal on target/backdrop-smoke with Backdrop Mica logs steps=[Opaque, MicaBackdrop] dwm_result=0x00000000"
        status: pass
    human_judgment: false
  - id: D3
    description: "Mica and Tabbed are visible on the active window (the visual result of this change)"
    requirement: UI-02
    verification: []
    human_judgment: true
    rationale: "A backdrop cannot be captured (PrintWindow returns no content under a backdrop); the maintainer judges it by eye in plan 03-47"
---

# Phase 3 Plan 45: Backdrop reaches DWM Summary

**Mica and Tabbed now reach the window: Opaque is applied before every changed GPUI appearance, the DWM system backdrop type is set and reset through a new safe Platform::set_window_backdrop (DwmSetWindowAttribute), and the HRESULT is logged as dwm_result.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-25T16:37:00Z
- **Completed:** 2026-11-25T16:48:00Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- `Platform::set_window_backdrop(&dyn HasWindowHandle, Backdrop)` on the trait (default `Unsupported`); the Windows override takes the HWND from raw-window-handle, refuses non-Win32 handles and builds below 22621, maps None and Acrylic to DWMSBT_NONE, Mica to DWMSBT_MAINWINDOW, Tabbed to DWMSBT_TABBEDWINDOW (no wildcard arm) and returns the HRESULT as `PlatformError::Os`. One unsafe block with a SAFETY comment; the Portal stays free of unsafe.
- `theme::appearance_steps(previous, target)`: Opaque before a changed target, nothing when unchanged, unknown previous counts as changed. `PortalApp::applied_appearance` remembers the last appearance set.
- `apply_backdrop` order: Opaque steps, target appearance, DWM type (only where `window_backdrop` is true), Root fill. The "backdrop applied" line carries `steps` and `dwm_result`; a refusal also logs a warning.
- docs/GPUI.md (Windows backdrops bullet) and docs/ARCHITECTURE.md (sucrose-platform list) describe the mechanism; both keep CRLF.

## Release smoke (scratch library target/backdrop-smoke, Backdrop Mica, Portal --no-focus, 15 s)

Task 1 (before appearance_steps):

```
INFO sucrose_portal::app: backdrop applied backdrop=Mica supported=true appearance=MicaBackdrop root_transparent=true dwm_result=0x00000000
```

Task 2 (final):

```
INFO sucrose_portal::app: backdrop applied backdrop=Mica supported=true appearance=MicaBackdrop steps=[Opaque, MicaBackdrop] root_transparent=true dwm_result=0x00000000
```

Both apps were stopped with taskkill afterwards; `tasklist.exe` showed no Rust Sucrose process. DWM accepted the type on the maintainer's Windows 11 build. Whether Mica and Tabbed are visible is the maintainer's eye check in 03-47.

## Task Commits

1. **Task 1: Mica reaches DWM and the result is logged** - `938a28f8` (feat)
2. **Task 2: Opaque before a changed target, docs** - `22e7c336` (fix)

RED was observed for each task as a compile failure on the missing items (`system_backdrop_type`, `dwm_result_text`, `appearance_steps`) before implementing, in the same commit as the code (no separate test commit).

## Files Created/Modified

- `Cargo.toml`, `Cargo.lock`, `crates/sucrose-platform/Cargo.toml` - raw-window-handle 0.6.2; the lock only gains it in sucrose-platform's dependency list
- `crates/sucrose-platform/src/lib.rs` - trait method `set_window_backdrop`
- `crates/sucrose-platform/src/os_windows.rs` - Windows implementation, `system_backdrop_type`, tests
- `apps/sucrose-portal/src/theme.rs` - `appearance_steps` and tests
- `apps/sucrose-portal/src/app.rs` - `applied_appearance`, `dwm_result_text`, `apply_backdrop` order and log fields
- `docs/GPUI.md`, `docs/ARCHITECTURE.md` - the mechanism

## Decisions Made

See key-decisions. The tracer feedback gate (Task 1) ran with an automated-only verify in end-of-phase mode: the verify passed end to end, so expansion (Task 2) continued without a checkpoint.

## Deviations from Plan

None - plan executed exactly as written.

Notes: `cargo fmt --all` rewrote `lib.rs` and `os_windows.rs` from CRLF to LF in the working tree; both were converted back to CRLF before the commit (`git ls-files --eol` shows w/crlf, `cargo fmt --check` passes). `.planning/STATE.md` was already modified before this plan started and is committed with the state update.

## Issues Encountered

None. `cargo xtask ci` exits 0 (fmt, clippy -D warnings, all tests).

## Known Stubs

None.

## Threat Flags

None. T-03-98 (one unsafe call, HWND from the borrowed live window, exact-size local value), T-03-99 (raw-window-handle already locked, no new package), T-03-100 (only enum values, steps and HRESULT logged) are implemented as planned.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 03-47 can ask the maintainer to judge Mica, Tabbed, Acrylic and None on the active window; the log line gives the OS answer and the steps for each switch.
- If Mica or Tabbed still do not show, the frame-extension question is decided by the rule of 03-47 (this plan does not extend the DWM frame or change the page tint).

## Self-Check: PASSED

- Files modified exist; commits `938a28f8` and `22e7c336` are ancestors of HEAD; `git rev-list --count` from the ledger base gives 2.
- All acceptance criteria re-run: `fn set_window_backdrop` in lib.rs, `DWMWA_SYSTEMBACKDROP_TYPE` in os_windows.rs, `raw-window-handle` in Cargo.toml, no new `[[package]]` in Cargo.lock, `dwm_result` at least 2 in app.rs, `fn appearance_steps`, `applied_appearance` at least 3, GPUI.md and ARCHITECTURE.md mention the mechanism and stay CRLF; smoke lines quoted above.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-25*
