---
phase: 03-m2-service-ipc-and-portal-connected
plan: 20
subsystem: ui
tags: [gpui, settings, select, slider, backdrop, background-image, ipc]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "SettingChange/PortalSettings and Changed/Rejected/Focus events (plans 04/06), Header and library entities (plans 14/19), Capabilities.window_backdrop, BackgroundImage layer and BackgroundError texts (plan 17)"
provides:
  - "settings/row.rs: SettingRow expander row, commit_number, ControlState, Choice select helpers (disabled options with tooltip) and NumberField"
  - "settings/personal.rs: Personal page with Corrupt Themes, Theme Preview (+ Hide Details), Adaptive Layout, Library Sorting and Library Pagination"
  - "settings/general.rs: General page with Application Theme, Notification Area Icon (+ AppExit), Window Backdrop with image picker, Stretch and Opacity, Library Location; backdrop_options, image_choice, truncate_path"
  - "Settings mode of PortalApp (gear, Back, General/Personal tabs, Escape), command-line and Focus page handling, Change Library Location buttons that open General"
affects: [03-21, 03-22, 03-23, 03-24, settings pages, library]

actuals:
  tokens: 18200
  tasks: 3
  commits: 5
plan_head_before: 1235adaa8b9c04e838dc4d8c73eefba121633049
plan_head_after: f91e7b30cd2557ba7a6b66265875661e8bf54aa6

tech-stack:
  added: []
  patterns:
    - "Controls show only broadcast values: a change is sent, Changed updates the control, Rejected re-shows the broadcast value (set_selected_value/set_value emit no event)"
    - "Generic Choice<T> select item with an optional reason that disables the option and becomes its tooltip"
    - "Header and Library report navigation through EventEmitter events that the root PortalApp subscribes to"

key-files:
  created:
    - apps/sucrose-portal/src/settings/mod.rs
    - apps/sucrose-portal/src/settings/row.rs
    - apps/sucrose-portal/src/settings/personal.rs
    - apps/sucrose-portal/src/settings/general.rs
  modified:
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/src/background.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/grid.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/GPUI.md

key-decisions:
  - "The settings tab enum is SettingsTab (not Tab) because gpui-component's Tab is imported in the header"
  - "Escape returns from settings through observe_keystrokes only when no dialog is open and no field or select of the shown page holds the keyboard"
  - "The library entity is kept alive across modes and only stops its hover preview on suspend, so page, scroll offset and search text survive without being saved"
  - "Sliders send on release or after a 150 ms rest; a value equal to the broadcast one is not sent"

requirements-completed: [ARCH-05, LIB-04, UI-01, UI-02]

coverage:
  - id: D1
    description: "Settings mode: gear enters General, Back and Escape return to the library with page, scroll and search intact; GeneralSetting, SystemSetting and Focus open General"
    requirement: UI-02
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/mod.rs#settings_gear_enters_general_from_the_library_and_keeps_the_page_when_already_in_settings"
        status: pass
      - kind: e2e
        ref: "debug Portal started with GeneralSetting on a synth-library scratch folder: capture target/synth-small/portal-settings.png and portal-general.png show Back, General (active) and Personal"
        status: pass
    human_judgment: true
    rationale: "Scroll position surviving Back and the Escape behavior need real input; the maintainer's end-of-phase check covers them"
  - id: D2
    description: "Personal page shows only the Phase 3 keys and sends them through the service; numeric fields clamp on commit and revert when emptied"
    requirement: LIB-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/row.rs#settings_number_above_the_range_is_cut_to_the_maximum"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/personal.rs#settings_number_field_changes_name_their_setting"
        status: pass
    human_judgment: true
    rationale: "Typing 250 into Library Pagination and the persisted values after a restart need real input (plan end-of-phase check)"
  - id: D3
    description: "General page: theme mode, tray icon with AppExit, backdrop options disabled by capability, background image picker with inline errors, stretch, opacity and library location"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/general.rs#settings_backdrop_options_before_windows_11_disable_everything_but_none"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/general.rs#settings_image_choice_refuses_other_extensions"
        status: pass
    human_judgment: true
    rationale: "The file picker, the disabled-option tooltips and the live backdrop need real input and a Windows 10 machine"
  - id: D4
    description: "Every control sends a SettingChange, shows only the broadcast value and reverts on Rejected; controls are skeletons before the first Welcome and disabled while disconnected"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/row.rs#settings_controls_wait_for_the_first_settings_then_follow_the_connection"
        status: pass
    human_judgment: true
    rationale: "A rejected write against the real service needs a failing write to provoke"

duration: 17min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 20: Settings mode and the General and Personal pages Summary

**Settings mode with Back and General/Personal tabs and the Phase 3 rows in their legacy places, where every control sends a SettingChange through the service, shows only the broadcast value and reverts on a rejected write**

## Performance

- **Duration:** 17 min
- **Started:** 2026-11-24T10:05:00Z
- **Completed:** 2026-11-24T10:22:00Z
- **Tasks:** 3
- **Files modified:** 14 (4 created)

## Accomplishments

- Settings mode: gear (after a divider) enters General, Back and Escape return, the library entity is not rebuilt; `GeneralSetting`, `SystemSetting` and `Focus { page }` open the matching page, and the empty and unreadable library states have a Change Library Location button that opens General.
- Personal page: Corrupt Themes (Recycle Bin or Trash wording), Theme Preview with a Hide Details switch disabled while the preview is off, Spacing Distance 5-25, Layout Order 0-100, Sort Mode, Sort Type and Library Pagination 1-100.
- General page: Application Theme, Notification Area Icon with the AppExit checkbox, Window Backdrop with options disabled by `Capabilities.window_backdrop` ("Requires Windows 11" or "Not available on {OS}" as tooltip), background image picker (validated off the UI thread), Remove, truncated path with tooltip, Stretch, Opacity slider and the Library Location row with its open-folder button and Move checkbox.
- Shared behavior: skeletons before the first Welcome, disabled controls while disconnected, `commit_number` clamping, one scroll column of max width 960.

## Task Commits

1. **Task 1: settings mode, tracer (gear, Personal, Pagination, Changed, Back)** - `62a78b89` (test, RED), `bf947ce9` (feat)
2. **Task 2: remaining Personal rows** - `80b67c7d` (feat)
3. **Task 3: General rows and page targets** - `a32b0eac` (test, RED), `f91e7b30` (feat)

## Files Created/Modified

- `apps/sucrose-portal/src/settings/mod.rs` - Mode, SettingsTab, column and group layout, bin_name
- `apps/sucrose-portal/src/settings/row.rs` - SettingRow, commit_number, ControlState, Choice select helpers, NumberField
- `apps/sucrose-portal/src/settings/personal.rs` - PersonalPage and the Number field enum
- `apps/sucrose-portal/src/settings/general.rs` - GeneralPage, backdrop_options, image_choice, truncate_path
- `apps/sucrose-portal/src/app.rs`, `header.rs` - mode state, header events, Escape, page wiring
- `apps/sucrose-portal/src/library/{mod,grid}.rs` - suspend, location_of, ChangeLocation event and buttons
- `apps/sucrose-portal/src/{assets,i18n,background,main}.rs`, `locales/sucrose.en.yml`, `docs/GPUI.md` - icons, six new locale keys, text-key allow removed, module, GPUI notes

## Decisions Made

See key-decisions. Startup (plan 23) and the folder picker with move (plan 22) are left for their plans; the Library group of General holds the path, open-folder button and Move checkbox they extend.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Commit messages carried literal backslash-n**
- **Found during:** final self-check
- **Issue:** the first five commit messages were written with literal `\n` sequences
- **Fix:** rewrote the five unpushed commits of this plan with `git filter-branch --msg-filter` so the messages have real line breaks; hashes in this summary are the rewritten ones
- **Files modified:** none (history only)
- **Verification:** `git log` shows subject and body lines

**2. [Rule 3 - Blocking] Header Tab type clashed with the settings tab enum**
- **Found during:** Task 1
- **Fix:** named the settings enum `SettingsTab`
- **Committed in:** bf947ce9

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** none on behavior.

## Issues Encountered

- A first draft of the truncate_path test expected one character too many; fixed in the GREEN commit.
- Personal could not be reached from the command line, so its capture used a local, uncommitted temporary mode override that was reverted before committing.

## Known Stubs

None. The Library Location row is intentionally partial (no folder picker or move): plan 22 adds them.

## Threat Flags

None. T-03-46 (clamping in the Portal, service stays authoritative) and T-03-47 (image checked on a background task) are applied as planned.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 22 can add the folder picker and move dialog to the Library group of `GeneralPage`; plan 23 can add the startup row to its Appearance & Behavior group. The end-of-phase human check (drive each row, restart, type 250) is still open.

## Self-Check: PASSED

- FOUND: apps/sucrose-portal/src/settings/mod.rs
- FOUND: apps/sucrose-portal/src/settings/row.rs
- FOUND: apps/sucrose-portal/src/settings/personal.rs
- FOUND: apps/sucrose-portal/src/settings/general.rs
- FOUND: 62a78b89
- FOUND: bf947ce9
- FOUND: 80b67c7d
- FOUND: a32b0eac
- FOUND: f91e7b30

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
