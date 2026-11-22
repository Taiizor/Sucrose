---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
verified: 2026-11-22T17:40:00Z
status: human_needed
score: 1/5 roadmap truths fully verified (4 present and wired, behavior not exercised on real hardware)
covered_files:
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-01-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-01-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-02-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-02-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-03-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-03-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-04-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-04-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-05-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-05-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-06-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-06-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-07-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-07-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-08-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-08-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-09-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-09-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-10-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-10-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-11-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-11-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-12-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-12-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-13-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-13-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-14-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-14-SUMMARY.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-15-PLAN.md
  - .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-15-SUMMARY.md
  - apps/sucrose/src/fullscreen.rs
  - apps/sucrose/src/main.rs
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/supervisor.rs
  - apps/sucrose/src/topology.rs
  - crates/sucrose-config/src/legacy.rs
  - crates/sucrose-core/src/theme.rs
  - crates/sucrose-core/src/version.rs
  - crates/sucrose-engine-mpv/src/config.rs
  - crates/sucrose-engine-mpv/src/loader.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/src/sync.rs
  - crates/sucrose-ipc/src/lib.rs
  - crates/sucrose-ipc/src/protocol.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/foreground.rs
  - crates/sucrose-platform/src/os_windows/job.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/os_windows/tray.rs
  - docs/MEASUREMENTS.md
  - xtask/src/fetch_engines.rs
  - xtask/src/measure.rs
covered_digest: "v4:sha256:8f88f44f90932b3e22b552a0f8e0b945e9568c578437b1f2c76d4035be614027"
behavior_unverified: 4
overrides_applied: 0
behavior_unverified_items:
  - truth: "SC1/D-01/D-02: the selected video plays behind the icons on EVERY monitor, the same frame on each, audio only on the master and promoted when the master's monitor goes away"
    test: "On a machine with two displays (or a virtual display), run sucrose.exe with the C# app stopped; watch both monitors for 5 minutes; unplug/disable the primary display"
    expected: "One surface per monitor, same frame (no growing offset), audio from one monitor only; after unplugging the primary the next renderer takes over audio at the configured volume"
    why_human: "Only the one-monitor case ran (maintainer machine has one display). Multi-monitor logic is covered by pure tests (topology diff, elect_master, sync correct) but never exercised against real DWM or a real second monitor"
  - truth: "SC2/DESK-07: the wallpaper survives an explorer.exe restart and monitors being plugged/unplugged"
    test: "taskkill /IM explorer.exe /F then start explorer.exe; plug/unplug or enable/disable a (virtual) display; change display scale while running"
    expected: "Wallpaper comes back on every monitor within seconds without polling (TaskbarCreated -> surface-lost exit code 3 -> respawn); tray icon returns; plug adds a renderer, unplug closes only that monitor's renderer; a scale change moves surfaces without restarting playback"
    why_human: "Kills the maintainer's shell / needs a second display. Executor ran only synthetic substitutes (five posted WM_DISPLAYCHANGE gave exactly one application). decide_exit/shell_restart/topology/debounce logic is unit-tested, the real OS signal path is not"
  - truth: "SC3/PERF-05: while a fullscreen game runs, the configured action applies (Pause: pauses and resumes; Close: stops and restarts)"
    test: "Maintainer sets Backgroundog.json FullScreenPerformance=Pause (then Close), starts a borderless-fullscreen game or browser F11, leaves it, then restores Backgroundog.json"
    expected: "Pause after 5 consecutive 1 s checks, resume after 3 without; Close stops renderers then starts them again; desktop focus never counts"
    why_human: "Needs a real fullscreen app and a maintainer edit of a legacy file (D-08). 02-13 SUMMARY states this check was not run. The predicate, the 5/3 debounce and plan_action are unit-tested (20 fullscreen tests) but the foreground_window probe on a real game was never exercised"
  - truth: "SC5/DESK-06 and TRAY-01: sharp at each monitor's DPI (125/150 % and a scale change while running); tray tooltip, menu and Quit"
    test: "Set display scale to 125 % and 150 %, run, compare sharpness with 100 %; hover the tray icon, open its menu, click Quit"
    expected: "Video fills the monitor in physical pixels and is as sharp as at 100 %; tooltip states what plays or why nothing does; menu has exactly one item Quit; Quit closes every renderer, exits 0 and the normal wallpaper returns"
    why_human: "Visual sharpness and tray clicks cannot be automated (CLAUDE.md: never synthesize input). Only Per-Monitor-V2 awareness (dpi.rs test) and the rect mapping are proven. REQUIREMENTS.md already marks DESK-06 complete although this check never ran"
human_verification:
  - test: "Two displays: every-monitor playback, same frame, single audio source, audio promotion on unplug"
    expected: "See behavior_unverified_items 1"
    why_human: "Machine has one display"
  - test: "explorer.exe restart, monitor plug/unplug, display scale change while running"
    expected: "See behavior_unverified_items 2"
    why_human: "Disturbs the live desktop / needs a second display"
  - test: "Real fullscreen app with FullScreenPerformance Pause and Close"
    expected: "See behavior_unverified_items 3"
    why_human: "Needs a maintainer edit of Backgroundog.json (D-08)"
  - test: "Display scale 125/150 % sharpness; tray tooltip, menu and clicking Quit"
    expected: "See behavior_unverified_items 4"
    why_human: "Visual / click-driven"
  - test: "Service pipe refuses a second Windows account (T-02-11)"
    expected: "A client from another account gets access denied"
    why_human: "Needs a second Windows account; DACL D:P(A;;GA;;;SID) is in code and unit-tested only"
  - test: "Triage the Rust renderer GPU load (93.7 % 3D engine vs 16.5 % for C# MpvPlayer)"
    expected: "Decide whether to fix in Phase 2 (try deband=no / default profile, see REVIEW IN-10) or accept and defer to Phase 4 PERF / Phase 8 sign-off"
    why_human: "Maintainer scope decision; MEAS-01 only requires measured and recorded, which is met"
---

# Phase 2: M1 - Video wallpaper on the desktop (Windows) Verification Report

**Phase Goal:** An existing user's selected video theme plays behind the desktop icons on every monitor on Windows through libmpv, survives shell and monitor changes, and pauses for fullscreen apps.
**Verified:** 2026-11-22T17:40:00Z
**Status:** human_needed
**Re-verification:** No - initial verification

## Goal Achievement

The code that delivers the goal exists, is substantive and is wired end to end (legacy reader -> theme gate -> service -> per-user pipe -> renderer per monitor -> Progman/WorkerW surface -> bundled libmpv; tray, supervisor with job object, topology diff, TaskbarCreated, fullscreen rule, master clock). No stub, TODO/FIXME or hollow path was found. `cargo test --workspace` was re-run by the verifier: 0 failures (sucrose 92 passed + 1 ignored, sucrose-core 24+18, sucrose-config 8+16, engine-mpv 36, ipc 12+1+7, platform 15+1+9, xtask 21). The ignored `loads_the_fetched_libmpv_by_absolute_path` and `real_install_selected_theme_passes_the_gate` tests were also run and pass against the real libmpv-2.dll and the maintainer's real C# install (read-only).

What is NOT proven is everything that needs real OS conditions the maintainer's one-display machine could not provide. Only one monitor, 100 % scale was seen by a human (02-06 and 02-07 human checks, approved). Those open items are routed to human verification, not marked verified.

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | The selected video plays behind the desktop icons on every monitor | PRESENT_BEHAVIOR_UNVERIFIED | Verified by a human for ONE monitor (02-07 approval: video behind icons, icons above, audio 100, wallpaper returns after stop). "Every monitor": `service.rs run()` -> `with_area(platform.monitors())` -> `diff(&[], &monitors)` -> `fleet.apply` spawns one renderer per monitor keyed by device name; 21 topology tests incl. adjacency/order/empty. Never run with two displays (02-12/02-14 SUMMARY) |
| 2 | It survives an explorer.exe restart and monitors being plugged/unplugged | PRESENT_BEHAVIOR_UNVERIFIED | `event_loop.rs` registers TaskbarCreated (line 125) and maps WM_DISPLAYCHANGE/WM_DPICHANGED/WM_SETTINGCHANGE(workarea) to LoopEvent; `service.rs` 419-500 debounces (2000 ms + 500 ms settle) and calls `fleet.shell_restarted`; `surface.rs` WM_DESTROY -> `destroyed_externally` -> exit code 3 (not counted in crash budget); job object kills renderers with the service. 14 supervisor + 16 topology tests green. No real explorer restart or plug/unplug was ever performed (02-12 SUMMARY line 177) |
| 3 | While a fullscreen game runs it applies the configured action (Pause: pauses and resumes) | PRESENT_BEHAVIOR_UNVERIFIED | `fullscreen.rs` predicate + 5/3 debounce + Pause/Close plans (20 tests green), `foreground.rs` probe, timer armed only when FullScreenPerformance != Resume (`enable_fullscreen` tests), `ToRenderer::Pause` reaches `player.set_paused`; ignored libmpv test proves pause/volume/loop on a real stream per 02-11. 02-13 SUMMARY line 155: the real-fullscreen human check was not run |
| 4 | CPU and RAM are measured against the C# MpvPlayer engine + Backgroundog and recorded in docs/MEASUREMENTS.md | VERIFIED | `xtask/src/measure.rs` (1034 lines, 21 xtask tests) and two rows in docs/MEASUREMENTS.md (C# PerDisplay 3 processes vs Rust service + renderer, release builds, same 4K HEVC video, 300 s after 60 s warm-up). Second row ("all monitors") honestly says it repeats the one-monitor numbers. See Warnings for two accuracy problems in the narrative |
| 5 | On a machine with the C# app installed Sucrose plays the theme selected there (Library.json, General.json unchanged), sharp at each monitor's DPI, and the tray icon's Quit stops it | PRESENT_BEHAVIOR_UNVERIFIED | Selected-theme path VERIFIED: `LegacySettings::load` -> `check_selected` -> `Assign`; real-install test passes; human saw the real theme play; legacy files read only (grep of non-test code finds no write to the legacy folder; `write_info` has no non-test caller). DPI: `init_dpi_awareness` PMv2 test passes, rects in physical pixels; sharpness at 125/150 % never viewed. Tray Quit: `Tray::new`, ordered shutdown (`shutdown_closes_all_then_waits_then_kills_only_the_survivors`), `refresh_desktop` are in place and tested at unit level; no human clicked Quit or hovered the tooltip |

**Score:** 1/5 roadmap truths fully verified; 4 present and wired but behavior-unverified (see behavior_unverified_items). No truth FAILED.

### Plan-level must-haves (spot-checked against code, not SUMMARY)

| Plan | Key truths checked | Status |
|------|--------------------|--------|
| 01 | engines.toml pin + SHA256 before extraction (`fetch_engines.rs`), committed bindgen `sys.rs`, loader with NotFound hints, apps/sucrose tracked; ignored load test run | VERIFIED |
| 02 | D-028 in docs/DECISIONS.md, maintainer decision recorded (orchestrator note); not re-read line by line | VERIFIED (docs) |
| 03/04 | Real reader, fixtures, BOM/CRLF/numeric strings, 24 legacy tests; clamp for WallpaperVolume matches C# (`src/Library/Sucrose.Manager/Manage/Engine.cs:18` uses `SHS.Clamp(...,0,100)`), so the 02-04 clamp-vs-default choice is legacy parity, not a regression. Enum/StretchType out of range falls back to the default (docs/legacy/engines.md:130) | VERIFIED |
| 05 | Single instance (`Listener::bind` -> AlreadyRunning), 64 KiB frame cap, per-user DACL, Hello pid check (weaker than claimed, see WR-05) | VERIFIED with caveat |
| 06/07 | Layout classifier from window tree, 0x052C, WS_EX_TOOLWINDOW|NOACTIVATE, PMv2 before windows; Player embedded via wid, load-scripts=no, ytdl=no, config=no, exit code 2 on LoadError; human-approved on one monitor | VERIFIED (one monitor) |
| 08 | SucroseInfo round trip, gate reasons, LegacyVersion ordering, D-029 | VERIFIED (18 theme + 24 core tests) |
| 09 | Tray gated by AppVisible (`start_tray`, 2 tests), Quit ordered shutdown; click path not exercised | VERIFIED (code) / human for click |
| 10 | RendererJob KILL_ON_JOB_CLOSE, RestartBudget 3/60 s, exit classification | VERIFIED (tests) |
| 11 | generated_options + user config + hardening; ignored option tests exist (not re-run: they create real mpv instances) | VERIFIED (per 02-VALIDATION green) |
| 12 | topology diff by device name, debounce, shell restart | code + unit tests only |
| 13 | fullscreen rule | code + unit tests only |
| 14 | sync controller, master election, audio only on master | code + unit tests only |
| 15 | measure task + docs | VERIFIED with warnings |

### Required Artifacts

All artifacts named in the 15 plans exist, are substantive (no file under 40 lines except thin lib.rs/os stubs by design) and are imported and used: `render.rs` uses `DesktopSurface`, `Player`, `loader`, `Follower`; `service.rs` uses `check_selected`, `LegacySettings`, `diff`, `Tray`, `RendererJob` via `Supervisor`, `covered_monitors`; `main.rs` dispatches `--render <slot>`. `target/debug/libmpv-2.dll`, `sucrose.exe` (debug and release) exist.

### Key Link Verification

| From | To | Via | Status |
|------|----|-----|--------|
| service.rs | theme.rs | `check_selected(location, selected, &app_version())` (line 1352) | WIRED |
| service.rs | legacy.rs | `LegacySettings::load(&setting_dir,&root)` (line 1317) | WIRED |
| service.rs | topology.rs | `diff(&[], &monitors)` and `diff(&self.monitors,&monitors)` | WIRED |
| service.rs | event_loop.rs | `LoopEvent::ShellRestarted` -> `fleet.shell_restarted` | WIRED |
| service.rs | fullscreen.rs | `FULLSCREEN_TIMER` set only if `enable_fullscreen` | WIRED |
| service.rs | tray.rs | `start_tray(plan.app_visible, ...)` | WIRED |
| render.rs | surface.rs / player.rs | `surface.native_handle()` -> `Player::open` wid | WIRED |
| render.rs | sync.rs | `Follower::step` in `follow_clock` | WIRED |
| supervisor.rs | job.rs | `RendererJob::spawn` | WIRED |

### Data-Flow Trace (Level 4)

Settings flow from the real legacy files (not constants) into `Assignment.settings` -> `generated_options` -> mpv options; the monitor rect flows from `platform.monitors()` into `Assignment.rect` -> `DesktopSurface::attach`. Status flows renderer -> `ToService::Playing/Failed` -> tooltip (stale-status defect WR-02 below). FLOWING.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Whole workspace | `cargo test --workspace` | 0 failed | PASS |
| libmpv loads by absolute path | `cargo test -p sucrose-engine-mpv --test load -- --ignored` | 1 passed | PASS |
| Selected theme of the real C# install passes the D-13 gate (read-only) | `cargo test -p sucrose -- --ignored real_install` | 1 passed | PASS |
| Running sucrose.exe / GUI | not run (CLAUDE.md: opens a real window on the maintainer's desktop) | - | SKIP |

### Probe Execution

No `probe-*.sh` declared or found (Rust project, automation lives in xtask). SKIPPED.

### Requirements Coverage

All 10 IDs from ROADMAP/REQUIREMENTS for Phase 2 appear in at least one PLAN frontmatter; none orphaned.

| Requirement | Source Plans | Status | Evidence |
|-------------|--------------|--------|----------|
| TYPE-01 | 03,05,06,07,08 | SATISFIED (one monitor, human-seen) | check_selected gate, Assign, real-install test |
| ENG-01 | 01,02,07,11 | SATISFIED | bundled libmpv via xtask pin, loader, Player wid, hwdec/loop/pause/volume |
| DESK-01 | 05,06,07,12,14 | SATISFIED for one monitor; every-monitor NEEDS HUMAN | surface.rs + human approval |
| DESK-06 | 06,07,12 | NEEDS HUMAN | PMv2 proven; sharpness at non-100 % never viewed; REQUIREMENTS.md marks Complete prematurely |
| DESK-07 | 10,12 | NEEDS HUMAN | restart budget + TaskbarCreated + topology logic tested; real events not exercised |
| PERF-05 | 13 | NEEDS HUMAN | logic tested; real fullscreen app never used |
| TRAY-01 | 09 | NEEDS HUMAN | tray built and gated; no click/tooltip check |
| FMT-02 | 08 | SATISFIED | fixture round trip, CRLF/no BOM, unknown fields kept |
| FMT-08 | 03,04 | SATISFIED | read-only reader, fixtures, real-install test |
| MEAS-01 | 15 | SATISFIED | rows recorded; see Warnings on wording and the GPU result |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (all phase files) | - | TODO/FIXME/XXX/TBD/todo!/unimplemented! grep | none | no debt markers, no gate trigger |
| apps/sucrose/src/service.rs | 179-200 | Status never leaves Failed (WR-02) | Warning | tooltip says "no wallpaper" after a recovery (DESK-07 cosmetic) |
| apps/sucrose/src/service.rs | 678-700, 775-792 | Unsynchronised/blocking pipe writes under the Renderers mutex (WR-03/04) | Warning | rare frame interleave; a stuck follower can freeze Quit/hot-plug |
| apps/sucrose/src/service.rs | 846-865 | Hello trusts claimed pid and slot (WR-05) | Warning | same-user impostor can take a slot; contradicts the strength implied by T-02-11 |
| crates/sucrose-engine-mpv/src/config.rs | 67-93 | User uMpvPlayer.config can still enable script=, input-ipc-server= (WR-06) | Warning | hardening claim in plan 07/11 only holds for load-scripts/ytdl/bindings |
| crates/sucrose-engine-mpv/src/loader.rs | 128-142 | libmpv also searched on PATH in release (WR-07) | Warning | DLL planting via user-writable PATH folder |
| crates/sucrose-platform/src/os_windows/surface.rs | 327-347 | up to 10 s sleep on a window-owning thread (WR-08) | Warning | "not responding" risk while explorer restarts |
| crates/sucrose-platform/src/os_windows/event_loop.rs | 172-199 | GetMessageW error ends as clean exit (WR-09) | Warning | wallpaper vanishes silently |
| crates/sucrose-engine-mpv/src/player.rs | 546-559 | loop_gen can never increment (WR-01) | Info in effect | nothing consumes it; follower wrap handling uses duration, so the plan-14 truth holds |
| crates/sucrose-config/src/legacy.rs | 48 | doc comment still says out-of-range gives the default | Info | stale comment; behavior is clamp (matches C#) |

All 9 review warnings and 10 infos are still `open` in 02-REVIEW-DISPOSITION.md; none is Critical and none falsifies a must-have truth by itself.

### Human Verification Required

1. **Every monitor / same frame / audio promotion** - needs a second or virtual display. Expected: one surface per monitor, same frame, audio from one, promotion on unplug.
2. **explorer.exe restart, plug/unplug, scale change while running** - Expected: wallpaper returns on every monitor, tray returns, only the unplugged monitor's renderer closes, no restart of playback on a scale change.
3. **Real fullscreen app, Pause then Close** (maintainer edits and restores Backgroundog.json). Expected: pause after 5 checks, resume after 3; Close stops then restarts.
4. **Scale 125 % / 150 % sharpness; tray hover tooltip, menu (only Quit), clicking Quit.** Expected: as sharp as 100 %; Quit exits 0, no sucrose.exe left, normal wallpaper back. (The black surface lingering after a force-kill was observed in 02-06; the job object plus `refresh_desktop` are meant to address a clean Quit and service kill - confirm both.)
5. **Pipe refuses a second Windows account** (T-02-11).
6. **GPU decision:** Rust renderer 93.7 % GPU 3D vs C# MpvPlayer 16.5 % for the same 4K HEVC video (working set 618 MB vs 599 MB, private 593 MB vs 1220 MB, CPU 0.51 % vs 0.77 %). MEAS-01 asks only to measure and record, so this is not a Phase 2 gap, but CLAUDE.md non-negotiable 6 and Phase 8's "lower RAM/CPU than C#" make it a tracked risk. Review IN-10 names deband=yes plus profile=low-latency as the first suspects. Debug-build A/V desync warnings were logged and never investigated; check them on a release build.

### Gaps Summary

No FAILED truth, MISSING artifact, NOT_WIRED link or debt marker, so there is no gaps_found. The status is human_needed because four of five roadmap criteria rest on real-hardware behavior (second monitor, explorer restart, real fullscreen app, DPI scaling, tray clicks) that was never exercised, and the requirements for them (DESK-06, DESK-07, PERF-05, TRAY-01) are already ticked "Complete" in REQUIREMENTS.md on unit-test evidence alone.

Warnings the maintainer should decide on (none blocks):
- **MEASUREMENTS.md narrative is inaccurate.** It says RAM and CPU are "at or below the C# MpvPlayer engine", but working set is higher for Rust (618 MB vs 599 MB for the three C# processes, 509 MB for MpvPlayer + Backgroundog). Only private bytes (593 vs 1220 MB) and CPU are lower. The "all monitors" row is the one-monitor row repeated.
- **REQUIREMENTS.md** marks DESK-06, DESK-07, PERF-05 and TRAY-01 Complete before their manual checks ran; consider reverting them to pending until the human checks pass.
- **Security hardening gaps** WR-05, WR-06, WR-07 weaken claims in 02-SECURITY.md (threats_open: 0); worth triage before shipping, not before Phase 3.
- Review dispositions are all `open`; triage into fixed/deferred/skipped.

---

_Verified: 2026-11-22T17:40:00Z_
_Verifier: Claude (gsd-verifier)_
