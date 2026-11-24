---
gsd_state_version: "1.0"
current_phase: 03
current_phase_name: M2 — Service, IPC and Portal connected
status: executing
stopped_at: Completed 03-27-PLAN.md
last_updated: "2026-11-24T14:56:08.141Z"
last_activity: 2026-11-24
last_activity_desc: Phase 03 execution started
state_head: 05ac3117905115a69080928c47549ce7dd411279
progress:
  total_phases: 8
  completed_phases: 2
  total_plans: 69
  completed_plans: 56
  percent: 25
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-23)

**Core value:** An existing user upgrades and loses nothing (themes, store, web wallpapers and their JavaScript API, settings migrated automatically) while the new app uses less memory and CPU than the C# one.
**Current focus:** Phase 03 — M2 — Service, IPC and Portal connected

## Current Position

Phase: 03 (M2 — Service, IPC and Portal connected) — EXECUTING
Plan: 28 of 40
Status: Ready to execute
Last activity: 2026-11-24 — Phase 03 execution started
Next step: Phase 3 (M2 — Service, IPC and Portal connected): `/gsd-discuss-phase 3`, or `/gsd-plan-phase 3` to plan directly.

Progress: [███░░░░░░░] 25%

## Performance Metrics

**Velocity:**
- Total plans completed: 29
- Average duration: -
- Total execution time: -

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 0 | - | - |
| 02 | 29 | - | - |

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 02 P01 | 32 min | 3 tasks | 15 files |
| Phase 02 P02 | 4 min | 3 tasks | 9 files |
| Phase 02 P03 | 5 min | 1 tasks | 15 files |
| Phase 02 P04 | 7 min | 2 tasks | 13 files |
| Phase 02 P05 | 26 min | 3 tasks | 15 files |
| Phase 02 P06 | 5 min | 2 tasks | 20 files |
| Phase 02 P07 | 13 min | 2 tasks | 6 files |
| Phase 02 P08 | 25 min | 3 tasks | 12 files |
| Phase 02 P09 | 21 min | 2 tasks | 18 files |
| Phase 02 P10 | 14 min | 2 tasks | 12 files |
| Phase 02 P11 | 8 min | 2 tasks | 11 files |
| Phase 02 P12 | 14 min | 3 tasks | 14 files |
| Phase 02 P13 | 5 min | 2 tasks | 8 files |
| Phase 02 P14 | 9 min | 2 tasks | 13 files |
| Phase 02 P15 | 40 min | 3 tasks | 8 files |
| Phase 02 P16 | 15 min | 3 tasks | 7 files |
| Phase 02 P17 | 8 min | 2 tasks | 6 files |
| Phase 02 P18 | 6 min | 2 tasks | 8 files |
| Phase 02 P19 | 4 min | 3 tasks | 5 files |
| Phase 02 P20 | 8 min | 2 tasks | 3 files |
| Phase 02 P21 | 40 min | 3 tasks | 3 files |
| Phase 02 P22 | 12 min | 3 tasks | 2 files |
| Phase 02 P23 | 5 min | 2 tasks | 5 files |
| Phase 02 P24 | 7 min | 2 tasks | 2 files |
| Phase 02 P25 | 5 min | 2 tasks | 3 files |
| Phase 02 P26 | interactive spike | 3 tasks | 1 files |
| Phase 02 P27 | 4 min | 2 tasks | 2 files |
| Phase 02 P28 | 3 min | 2 tasks | 3 files |
| Phase 02 P29 | 10 min | 3 tasks | 1 files |
| Phase 03 P01 | 11 min | 2 tasks | 9 files |
| Phase 03 P02 | 10 min | 2 tasks | 42 files |
| Phase 03 P03 | 11 min | 2 tasks | 11 files |
| Phase 03 P04 | 8 min | 2 tasks | 11 files |
| Phase 03 P05 | 12 min | 2 tasks | 5 files |
| Phase 03 P06 | 11 min | 2 tasks | 11 files |
| Phase 03 P26 | 9 min | 2 tasks | 15 files |
| Phase 03 P07 | 9 min | 2 tasks | 7 files |
| Phase 03 P08 | 17 min | 2 tasks | 13 files |
| Phase 03 P09 | 15 min | 2 tasks | 7 files |
| Phase 03 P10 | 19 min | 2 tasks | 15 files |
| Phase 03 P11 | 60 min | 3 tasks | 2 files |
| Phase 03 P12 | 27 min | 2 tasks | 7 files |
| Phase 03 P13 | 17 min | 2 tasks | 7 files |
| Phase 03 P14 | 23 min | 2 tasks | 11 files |
| Phase 03 P15 | 13 min | 3 tasks | 13 files |
| Phase 03 P16 | 15 min | 2 tasks | 7 files |
| Phase 03 P17 | 19 min | 3 tasks | 15 files |
| Phase 03 P18 | 13 min | 3 tasks | 11 files |
| Phase 03 P19 | 11 min | 2 tasks | 6 files |
| Phase 03 P20 | 17 min | 3 tasks | 14 files |
| Phase 03 P21 | 24 min | 2 tasks | 17 files |
| Phase 03 P22 | 33 min | 2 tasks | 12 files |
| Phase 03 P23 | 21 min | 2 tasks | 19 files |
| Phase 03 P24 | 90 min | 3 tasks | 2 files |
| Phase 03 P25 | 6 min | 2 tasks | 4 files |
| Phase 03 P27 | 14 min | 2 tasks | 8 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions and, authoritatively, in `docs/DECISIONS.md` (append-only, D-001 … D-031).
Recent decisions affecting current work:

- [Roadmap]: Exactly 8 phases, Phase N = milestone M(N-1); no decimal phases (maintainer decision)
- [Roadmap]: Requirements not placed by the milestone checklists were assigned so each phase is a usable slice: library browsing and the Portal shell in Phase 3; engine detection and choice, playback and sound rules, web API fixtures and the theme-folder path check in Phase 4; theme-card actions, drag and drop, full settings pages and tray, startup modes, desktop options, logs and backup, TLS, archives and store format in Phase 5
- [Phase 02]: 02-01: libmpv loader searches the exe folder, then absolute PATH entries only (D-018 narrowed against DLL planting; plan 02 records it in D-028)
- [Phase 02]: 02-01: sys.rs bindings are generated once in the rust Docker image (clang + libclang-dev), committed, and the xtask bindgen feature keeps libclang off CI
- [Phase 02]: 02-01: vulkan-1.dll is a normal (not delay-load) import of libmpv-2.dll; a PC without the Vulkan loader gets LoadError::LoadFailed
- [Phase 02]: D-028: engine binaries ship inside the Sucrose package (supersedes D-005); libmpv pins stay on the upstream release for now, GPL-2.0-or-later build signed off
- [Phase 02]: 02-03: locked or empty legacy setting files are retried 3 times (50 ms) before defaults, since the C# writer truncates in place
- [Phase 02]: 02-03: SucroseInfo keeps AppVersion as raw JSON and ignores unknown fields until plan 08 types and round-trips it
- [Phase 02]: 02-04: legacy numbers outside a range are clamped (volume 250 gives 100, -5 gives 0); wrong types and unknown enum names give the legacy default; ScreenArea reads names only
- [Phase 02]: 02-05: a second pipe on the same name fails with PermissionDenied on Windows (A7); bind maps it and AddrInUse to AlreadyRunning, and the owner SID is validated before it enters the SDDL text
- [Phase 02]: Surface attach strategy B (layered child of Progman under SHELLDLL_DefView) is composited on Windows 10.0.26300.9974; layout is classified from the window tree only
- [Phase 02]: Plan 09 must refresh the desktop after the renderer ends: a force-killed renderer left the black surface area unrepainted
- [Phase 02]: Plan 07: Raised/LayeredChild surface composites a libmpv d3d11 video child as-is on Windows 10.0.26300; renderer exit code 2 = libmpv missing, 4 = other engine start failure
- [Phase 02]: 02-08: D-029 version scheme YY.M.D+R, first value 26.11.22+1 (maintainer choice); CARGO_PKG_VERSION keeps the +R suffix (A16 tested)
- [Phase 02]: 02-08: an unmappable Cargo version makes app_version() the lowest version so themes are refused loudly
- [Phase 02]: 02-09: ToService::Playing appended next to Failed so the tooltip says playing only once the renderer plays
- [Phase 02]: 02-09: Quit waits 2 s in total on a worker thread, then kills survivors; refresh_desktop re-applies the wallpaper after the renderers exit
- [Phase 02]: 02-10: renderer restarts are bounded by RestartBudget (3 crashes per 60 s per monitor, 0.5/1/2 s backoff, surface-lost cap 5 per 60 s); shutdown kills survivors through the job object
- [Phase 02]: 02-11: embedding and hardening mpv options are applied after the user's uMpvPlayer.config, and the shader cache is always redirected to Cache/mpv-shaders, so a user config cannot weaken hardening or write into the C# folder
- [Phase 02]: 02-12: timers are changed through the LoopTimers handle (EventLoop::timers) because EventLoop::run consumes the loop; plans 13 and 14 use it inside handlers
- [Phase 02]: 02-12: TaskbarCreated restarts are acted on at most 5 per 60 s; a lost surface inside the 30 s window is not a crash, a closed renderer is respawned once per slot
- [Phase 02]: 02-13: the fullscreen predicate compares against full monitor rectangles (not surface areas), recovery undoes the action on the monitors it was done to, and a closed slot is respawned only after its old renderer's exit event
- [Phase 02]: 02-14: ToService::Clock appended after Playing (wire index 3); position jumps come from playback-restart events, not an observed time-pos, to keep the event thread idle
- [Phase 02]: 02-14: followers correct through Follower (nudge once, return to master speed inside a frame, seeks 2 s apart); the master restores speed 1.0 when promoted
- [Phase 02]: 02-15: GPU memory also reads GPU Process Memory Shared Usage because Dedicated Usage is 0 on the integrated GPU; Rust renderer GPU 3D engine 93.7% vs C# MpvPlayer 16.5% flagged for Phase 2 verification
- [Phase 02]: 02-16: the 500 ms restart delay after TaskbarCreated stays; the attach waits up to 10 s for the wallpaper WorkerW and a WinEvent order guard keeps the surface above it
- [Phase 02]: 02-17: default_gpu_adapter reads the adapter back from a default D3D11 device (IDXGIDevice::GetAdapter); the EnumAdapters1 order is never taken as the adapter in use
- [Phase 02]: 02-18: Engine.json GraphicPreference is read as a legacy enum (names and 0..2); the service writes UserGpuPreferences for sucrose.exe before the first renderer, only when a wallpaper job exists
- [Phase 02]: D-030: sticky master clock; a renderer counts only after its first frame; departs from the 02-CONTEXT primary-monitor rule
- [Phase 02]: 02-20: follower integral gain 0.1 with a closing-drift guard (planner's 0.03 overshoots the one-frame band in simulation)
- [Phase 02]: 02-21: GPU adapter evidence for measure is the per-pid typeperf LUID with the largest summed utilization; --row refuses a cross-adapter or adapter-less pair
- [Phase 02]: 02-22: mpv deband=yes and profile=low-latency kept for parity; same-GPU R = 1.02 (Rust 16.4 % vs C# 16.1 % on RTX 3050)
- [Phase 02]: 02-23: a PartlyHidden or Hidden placement verdict never fails an attach or a move; the renderer attaches, keeps playing and logs
- [Phase 02]: 02-24: the location watch is a second WinEvent hook filtered to the thread of the parent's root; the 02-16 order guard is unchanged
- [Phase 02]: 02-25: replace_all re-places every surviving surface after each applied display change; diff stays for the initial start; the master election runs only when monitors changed
- [Phase 02]: G-02-7 after 02-26: wait for explorer and accept the gap; sign-off 'wait' (maintainer's verbatim reply); class (c) not seen, no watch fix
- [Phase 02]: 02-27: a report of an unassigned slot (monitor removed or renderer closed by the fullscreen rule) is dropped even while the renderer is connected, through SlotState.assigned from Fleet::slot_assigned (WR-19)
- [Phase 02]: D-031: wait for explorer after a display change, no fallback window; G-02-7 stays open as an accepted limitation
- [Phase 02]: 02-29: final M1 build row is Rust-only because --row refused the C# pair after a reboot (LUIDs differ); same-GPU C# reference stays the first 2026-11-23 row
- [Phase 03]: Settings live in Setting/Sucrose.json (D-01, D-032): versioned, atomic temp+sync_all+rename, .bak outside the *.json glob, service is the only writer
- [Phase 03]: Default library location is stored as an empty string meaning the legacy default folder; invalid values read as legacy defaults section by section
- [Phase 03]: 03-02: a folder that cannot be read is Unreadable and outranks every other corrupt reason; PathEscapes outranks a missing file, so a transient error never reads as corrupt
- [Phase 03]: 03-02: SucroseInfo Type names parse in any letter case (Newtonsoft), settings enum names stay case-sensitive
- [Phase 03]: 03-03: legacy keys reworded for the bin are left out of legacy.en.yml (REPLACED_KEYS) and defined in sucrose.en.yml, so no key lives in two locale files
- [Phase 03]: 03-03: a locale file with only comments makes every rust-i18n lookup return the key; tr_count_with fills extra placeholders such as the bin name
- [Phase 03]: BackdropType Auto migrates to theme mode System plus Mica whatever ThemeType says; nothing stored gives System plus None (D-21)
- [Phase 03]: synth-library refuses an --out inside the real Sucrose data folder even with --force
- [Phase 03]: synth-library marker is written first as incomplete and finished only after the scan self-check, so an interrupted run is repeatable and never up to date
- [Phase 03]: Synthetic pool sizes are hit by construction (JPEG quality search, GIF frame count) and copied round-robin into theme folders
- [Phase 03]: D-033: one endpoint for service, renderers and Portal, routed by the first message; one registered Portal; endpoint name carries a hash of SUCROSE_DATA_DIR
- [Phase 03]: A Portal connection is identified by a service-assigned connection id, never by the pid it claims
- [Phase 03]: [03-26] Engine choices are validated against the allowed engine set of their wallpaper type; storage and ping type names stay non-empty strings; Backgroundog runtime keys, stale names and Cycling.PassingTime are never fields
- [Phase 03]: 03-07: SettingsHost without a store keeps legacy settings in memory and refuses every change with the not-saved text
- [Phase 03]: 03-08: the Portal says PortalHello before opening its window and waits up to 2 s for the first event, so a second Portal exits with 0 on AlreadyOpen without a window; a connection that broke before Welcome does not reset the 3 reconnect attempts
- [Phase 03]: Use checks a library move, then the fullscreen rule, then the gate, then idempotence: a refused Use never changes Library.Selected
- [Phase 03]: The Use transition is LoadSource (loadfile replace in the running renderers); USE_TRANSITION switches to Restart, already built on the fullscreen rule's close-and-want-back path, if plan 11 sees a black frame
- [Phase 03]: Library thumbnails: bucket-wide square box, never enlarged, DCT-scaled JPEG; bucket from the scale factor; only the first 64 cards of a page are queued
- [Phase 03]: Portal asset source composes the default component icons with icon_assets! for the icons pages add
- [Phase 03]: D-26 confirmed: Use between Video themes keeps UseTransition::LoadSource (no restart, no hybrid memory-threshold restart); maintainer checked ten switches on 2026-11-24
- [Phase 03]: 03-12: every non-Applied UseOutcome shows the Use-failed toast; keyboard Use rides on GPUI click-on-Enter/Space; error toasts self-time 8 s
- [Phase 03]: 03-13: modules go through start_module; MemoryModule remembers the last Portal connection so a failed sampler start is logged once per Portal
- [Phase 03]: Plan 03-14: the legacy pagination bar is composed from Buttons and an Input (gpui-component Pagination has no jump field or nine-slot scheme); the search box limit of 20 characters is an Input validator; an emptied search box applies at once, typed text after 500 ms
- [Phase 03]: 03-15: the registered Portal is focused only after a second sucrose or Portal passed on the foreground right; a just-spawned Portal counts as connected for the exit rule for 10 s
- [Phase 03]: 03-16: tray-icon and muda event handlers are set-once per process, so the tray installs them once and routes through a static wiring (a rebuilt tray works)
- [Phase 03]: 03-16: after the 2 s grace the Portal counts as gone whatever the outcome; only the sibling sucrose-portal executable is ever ended
- [Phase 03]: 03-17: the background image is decoded at the smallest size that covers the window (never enlarged, long edge 2560), so UniformToFill is not upscaled
- [Phase 03]: 03-17: a stored backdrop the OS cannot show is applied as Opaque and never rewritten; Capabilities.window_backdrop is read once with RtlGetVersion (build 22621)
- [Phase 03]: Plan 18: status bar and About show the legacy four-component version text; the tray hint dialog has Escape and backdrop presses off for its whole life (the dialog layer fixes them at open)
- [Phase 03]: Plan 18: quote hides below 900 px and copyright below 1000 px; Premium address lives in header.rs, other links in dialogs/mod.rs as https constants
- [Phase 03]: 03-19: a capped preview GIF loops the frames decoded so far; zero or tiny delays play at 100 ms; failed previews are remembered until the next rescan
- [Phase 03]: Settings pages show only broadcast values; Escape leaves settings only when no dialog is open and no field holds the keyboard
- [Phase 03]: Library maintenance: an existing showcase folder is recorded without being written to; names are recorded only after a successful copy
- [Phase 03]: Corrupt themes go to the OS bin through sucrose_platform::move_to_trash (trash 5.4.0, path-free errors); the service re-scans and trusts no list from the Portal
- [Phase 03]: 03-22: With Move off the destination may hold files (legacy rule); the empty-folder check applies only when the themes move along
- [Phase 03]: 03-22: Library.Location is written only after the worker verified the content; the old copy is removed after the write, and a failed write rolls the content back
- [Phase 03]: 03-22: The MoveLibrary answer is sent when the whole move is over (wallpaper restarted), so the old folder is gone when the Portal hears Moved
- [Phase 03]: 03-22: Renderers close through RuleClosed and a 4 s timer ends survivors, so library_moving cannot stay true
- [Phase 03]: 03-22: The Portal applies the new library location from the destination it sent; no SettingChange exists for it
- [Phase 03]: D-034: Sucrose's own start entry is Sucrose.Service (Run value), dev.sucrose.Sucrose (LaunchAgent), sucrose.desktop (XDG); the C# entry is only detected
- [Phase 03]: 03-23: a startup setting that cannot be saved takes the entry change back; the C# entry lookup runs on a worker thread only when the General page opens
- [Phase 03]: 03-24: both D-25 Portal scenarios must be repeated after the hover-preview defect (card.rs:282-283, mod.rs:541-562) is fixed; scenario 2 round windows are skewed, only first-page, peak and final are valid
- [Phase 03]: 03-25: UI-07 is not marked complete; the synthetic 500-theme library fails the D-25 rule (working set 131 MB first page, 269 MB final against 181 MB) and is reported as a gap
- [Phase 03]: 03-27: every Portal image release goes through atlas::release_image(s), deferred with App::defer, because App::drop_image with no window misses the window being updated
- [Phase 03]: 03-27: GPUI test support comes from a gpui-pre dev-dependency (workspace key gpui), since gpui-kit test-support cannot resolve against the locked wgpu

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 2]: G-02-7 is an accepted limitation (D-031), not a fix: 02-UAT.md keeps it failed; the cause of the one ~90 s UAT episode is not established
- [Phase 2]: 7 info-level review findings stay open (IN-19 … IN-23, IN-26, IN-27 in 02-REVIEW-DISPOSITION.md)
- [Phase 5]: DESK-10 (lock screen) needs a `docs/DECISIONS.md` entry on the mechanism before it is built (D-026)
- [Phase 5]: STOR-01 and FMT-05 depend on D-022 (deferred); use the existing v8 endpoints until the API rewrite
- [Phase 5]: Update signing key scheme still open (Ed25519 proposed, D-019)
- [Phase 6]: No Apple Developer ID yet (D-025); macOS packages stay unsigned, manual tests need a real Mac
- [Phase 7]: Linux package formats are decided in Phase 7
- [All phases]: Measure RAM/CPU of user-visible changes in release builds; record notable numbers in `docs/MEASUREMENTS.md`

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 261122-723 | Target architectures (D-027): Windows x86/x64/ARM64, macOS universal, Linux x86_64/aarch64; CI matrix, docs, FND-02 | 2026-11-22 | 971149f4 | [261122-723-add-target-architecture-support-d-027-wi](./quick/261122-723-add-target-architecture-support-d-027-wi/) |

## Deferred Items

Items acknowledged and deferred at milestone close, most recent first:

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| *(none)* | | | | |

## Session Continuity

Last session: 2026-11-24T14:55:54.873Z
Stopped at: Completed 03-27-PLAN.md
Resume file: None
