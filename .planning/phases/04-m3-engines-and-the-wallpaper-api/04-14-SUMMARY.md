---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 14
subsystem: portal
tags: [portal, settings, wallpaper-page, ipc, loop, shuffle, cef, mpv]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "The Wallpaper page, PORTAL_PROTOCOL_VERSION 2 and the engine SettingChanges (04-13), the CEF renderer and its compatible calls (04-01, 04-09), the mpv engine"
provides:
  - "PORTAL_PROTOCOL_VERSION 3: SettingChange::{WallpaperLoop, WallpaperShuffle, HardwareAcceleration, InputType, InputModuleType, InputDesktop} and the matching PortalSettings fields"
  - "ToRenderer::{SetLoop, SetShuffle} (appended) and Engine::set_shuffle (default no-op); renderer log lines 'loop changed' and 'shuffle changed'"
  - "Fleet::apply_live: a saved Loop or Shuffle change reaches every connected renderer and the job and shared assignments, with no restart"
  - "The Wallpaper group on the Wallpaper settings page: Interaction Method (module and desktop-focus in its footer), Loop Mode, Shuffle Mode, Hardware Acceleration"
affects: [04-15, 04-16, 04-19, 04-20]

actuals:
  tokens: 14000
  tasks: 2
  commits: 5
plan_head_before: 034182fdca40966b5d15d2fc7ed9d44891a10f27
plan_head_after: 3ee004108c4d8300f14e1bfb8a70c9c953929cd5

tech-stack:
  added: []
  patterns:
    - "live settings: validate, save, then send one appended ToRenderer message to every connected renderer and fold the value into the job and assignments so a restarted renderer keeps it"
    - "switches and the checkbox are drawn from the broadcast settings, so a rejected change reverts them by re-rendering; selects use show_choice"

key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - docs/FEATURES.md

key-decisions:
  - "Loop Mode and Shuffle Mode apply live through appended ToRenderer variants (SetLoop, SetShuffle); Hardware Acceleration and the interaction settings are only saved and are read by the next renderer that starts (04-16 adds the restart prompt, 04-19 the input forwarding)"
  - "A live change also updates Fleet.job and the shared assignments, so a renderer that restarts later keeps the new value; it is no Use, so the restart budgets stay"
  - "CEF re-runs the compatible calls when loop or shuffle change, so a running page gets the new LoopMode and ShuffleMode value at once instead of at the next data push"

patterns-established:
  - "Engine::set_shuffle: a defaulted trait method for a setting only one engine has a call for"

requirements-completed: []

coverage:
  - id: D1
    description: "The six Wallpaper group SettingChanges, their PortalSettings fields and apply, PORTAL_PROTOCOL_VERSION 3, SetLoop and SetShuffle with wire indexes 10 and 11"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc: setting_change_variants_keep_their_wire_index, set_loop_and_set_shuffle_keep_their_wire_index_and_survive_the_wire, apply_sets_exactly_the_field_a_change_names"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service stores the six changes in the Engine section and shows them in the view; a failed save is rejected and keeps the value"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "apps/sucrose settings_host::tests::a_wallpaper_group_change_is_written_to_the_engine_section_and_shown"
        status: pass
    human_judgment: false
  - id: D3
    description: "Turning Loop Mode off reaches the playing wallpaper: Changed comes back and the renderer logs 'loop changed' with looping=false"
    requirement: "PLAY-01"
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_loop_change_reaches_the_running_renderer (ignored by default, run with --include-ignored)"
        status: pass
      - kind: unit
        ref: "apps/sucrose service::tests::a_loop_change_reaches_every_connected_renderer_and_the_assignments_without_a_restart"
        status: pass
    human_judgment: false
  - id: D4
    description: "CEF pages get the new ShuffleMode and LoopMode value with the next compatible push, which the change triggers"
    requirement: "PLAY-01"
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-cef engine::tests::a_page_gets_the_new_shuffle_value_with_the_next_compatible_push"
        status: pass
    human_judgment: false
  - id: D5
    description: "The Wallpaper group rows look right and follow ControlState on the Wallpaper page"
    requirement: "WAPI-07"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal settings::wallpaper::tests::wallpaper_group_*"
        status: pass
    human_judgment: true
    rationale: "Layout and wording of the new rows were checked on one PrintWindow capture of the debug build by the executor; a person has not looked at them"
  - id: D6
    description: "docs/FEATURES.md says the Stay awake setting returns in Phase 5 (DESK-11) and notes the Round A Loop and Shuffle behavior"
    verification:
      - kind: other
        ref: "grep -c 'returns with the setting in Phase 3/4' docs/FEATURES.md prints 0; the file stays w/crlf with no final newline"
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 14: Wallpaper group, live loop and shuffle Summary

**Loop Mode and Shuffle Mode switch live in running mpv and CEF wallpapers through new SetLoop/SetShuffle renderer messages, and the Wallpaper page gains the Interaction Method, Loop, Shuffle and Hardware Acceleration rows over Portal protocol 3.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-11-26T06:32:05Z
- **Completed:** 2026-11-26T06:57:00Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Six new SettingChanges (WallpaperLoop, WallpaperShuffle, HardwareAcceleration, InputType, InputModuleType, InputDesktop) are validated, written to the Engine section and shown in the Portal view; PORTAL_PROTOCOL_VERSION is 3.
- A saved Loop or Shuffle change is sent to every connected renderer (ToRenderer::SetLoop and SetShuffle, appended after Data), and the job and shared assignments take the value so a renderer that restarts keeps it. Nothing restarts.
- The renderer applies SetLoop to its engine and the follower's wrap flag and logs "loop changed"; SetShuffle goes through the new Engine::set_shuffle (no-op except in CEF, which keeps it in an atomic and re-runs the compatible calls).
- The Wallpaper page shows the Wallpaper group above Extensions in the legacy order, with ControlState (skeleton while loading, disabled while the service is unreachable) and the existing Rejected revert plus the Portal.Setting.SaveFailed toast.
- FEATURES: the Stay awake row says the setting returns in Phase 5 (DESK-11); the Loop, shuffle row records the Round A UI and live apply.

## Task Commits

1. **Task 1 (tracer): Loop Mode off reaches the playing wallpaper** - `e88147eb` (test, RED), `fe579bda` (feat, GREEN)
2. **Task 2: Wallpaper group rows, shuffle in CEF, FEATURES fix** - `c493adf1` (feat)

The range `plan_head_before..plan_head_after` also holds two maintainer commits ("Update config.toml", `221396c1` and `3ee00410`, .cargo/config.toml only) made while the plan ran; `commits: 5` is the measured count and includes them. This plan made 3 commits.

**Plan metadata:** added by the docs commit that follows this file.

## RED evidence (Task 1)

The target tests failed on their planned assertions against stubs (settings_host write, Fleet::apply_live, live_message and the renderer dispatch were no-ops): `a_wallpaper_group_change_is_written_to_the_engine_section_and_shown` (`!view.wallpaper_loop && ...`), `live_message_maps_loop_and_shuffle_and_nothing_else` (left None, right Some(SetLoop(false))), `a_loop_change_reaches_every_connected_renderer_and_the_assignments_without_a_restart` (left [], right [0, 1]), the shuffle and no-connection variants. The protocol tests passed already because the wire types are not stubbed. Tracer verify re-run end to end after GREEN: sucrose-ipc, settings_host, build, and the integration test against the maintainer's migrated settings (read-only, scratch folder) all pass.

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs` - version 3, six SettingChanges, PortalSettings fields, apply
- `crates/sucrose-ipc/src/protocol.rs` - ToRenderer::SetLoop and SetShuffle
- `crates/sucrose-engine/src/engine.rs` - Engine::set_shuffle (default no-op)
- `crates/sucrose-engine-cef/src/engine.rs` - shuffle atomic, Command::Shuffle, push on loop/shuffle
- `apps/sucrose/src/settings_host.rs` - store and view of the six changes
- `apps/sucrose/src/service.rs` - live_message, put_live, Fleet::apply_live and its call after a stored change
- `apps/sucrose/src/render.rs` - SetLoop and SetShuffle handling ("loop changed", "shuffle changed")
- `apps/sucrose/tests/portal.rs`, `apps/sucrose/tests/common/mod.rs` - ignored integration test and TestService::start_migrated
- `apps/sucrose-portal/src/settings/wallpaper.rs` - the Wallpaper group (Flag, input choices, switches, checkbox)
- `docs/FEATURES.md` - Stay awake and Loop, shuffle rows

## Decisions Made

See key-decisions above. `apps/sucrose-portal/locales/sucrose.en.yml` needed no change: `Portal.SettingsPage.GroupWallpaper` already existed from 04-13 and every other label is a legacy key.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Engine::set_shuffle added in Task 1, not Task 2**
- **Found during:** Task 1 (renderer dispatch of SetShuffle)
- **Issue:** the renderer cannot route SetShuffle without the trait method, which the plan listed under Task 2
- **Fix:** added the defaulted trait method in the Task 1 GREEN commit; the CEF implementation stayed in Task 2
- **Files modified:** crates/sucrose-engine/src/engine.rs
- **Committed in:** fe579bda

**2. [Rule 2 - Missing critical] A live change also updates the job and the shared assignments**
- **Found during:** Task 1 (service side)
- **Issue:** sending SetLoop only to connected renderers would leave a renderer that restarts (crash budget) or connects late on the old value
- **Fix:** Fleet::apply_live folds the value into Fleet.job and every shared assignment, with unit tests for connected, shuffle and unconnected renderers
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** e88147eb (tests), fe579bda

**3. [Rule 2 - Missing critical] CEF re-runs the compatible calls on a loop or shuffle change**
- **Found during:** Task 2
- **Issue:** the state alone would only reach a page at the next data push, which may never come for a page without data keys
- **Fix:** Command::Loop and Command::Shuffle mark the drain as a push, the same path a data snapshot uses
- **Files modified:** crates/sucrose-engine-cef/src/engine.rs
- **Committed in:** c493adf1

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 missing critical). **Impact on plan:** all needed for the stated truth "reaches running wallpapers at once"; no scope creep.

## Issues Encountered

- The integration test plays the maintainer's selected Video theme for about a second on the real desktop (scratch data folder, legacy settings read-only), so it is `#[ignore]` and the verify command uses `--include-ignored`. The service is killed when the test ends; `taskkill.exe /IM sucrose.exe /F` found nothing left.
- The first `cargo xtask ci` after the capture run ended with "Erisim engellendi (os error 5)" in the cef-dll-sys build script (a transient file lock after the apps were stopped); the rerun passed (exit 0).

## Line endings

Files that were w/crlf before the edits and were restored to CRLF before their commits: crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/settings_host.rs, apps/sucrose/src/service.rs, apps/sucrose/src/render.rs, crates/sucrose-engine/src/engine.rs, crates/sucrose-engine-cef/src/engine.rs, docs/FEATURES.md (its last line has no newline and none was added). The LF files (portal.rs, tests, wallpaper.rs) stayed LF.

## Known Stubs

None. Hardware Acceleration and the interaction settings are saved and shown, and are read by later plans by design (04-16 restart prompt, 04-19 input forwarding); Screen Layout and Stretch Mode wait for Round C.

## Threat Flags

None. The new changes are typed enums and booleans; InputModuleType has the single value RawInput, so T-04-33 holds by construction.

## Next Phase Readiness

- 04-16 can offer Restart now after a Hardware Acceleration change; 04-19 reads InputType, InputModuleType and InputDesktop when it forwards input.
- PLAY-01 and WAPI-07 stay partial in Round A (`requirements-completed: []`).

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*

## Self-Check: PASSED

All key files exist and the task commits e88147eb, fe579bda and c493adf1 are ancestors of HEAD.
