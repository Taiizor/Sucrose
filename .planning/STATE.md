---
gsd_state_version: "1.0"
current_phase: 04
current_phase_name: M3 — Engines and the wallpaper API
status: executing
stopped_at: Completed 04-44-PLAN.md
last_updated: "2026-11-28T16:48:26.195Z"
last_activity: 2026-11-28
last_activity_desc: Phase 04 execution started
state_head: 7321484b4a5bd91bc7c2ec47036d588822a755bd
progress:
  total_phases: 8
  completed_phases: 3
  total_plans: 135
  completed_plans: 123
  percent: 38
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-11-25)

**Core value:** An existing user upgrades and loses nothing (themes, store, web wallpapers and their JavaScript API, settings migrated automatically) while the new app uses less memory and CPU than the C# one.
**Current focus:** Phase 04 — M3 — Engines and the wallpaper API

## Current Position

Phase: 04 (M3 — Engines and the wallpaper API) — EXECUTING
Plan: 45 of 56
Status: Ready to execute
Last activity: 2026-11-28 — Phase 04 execution started
Next step: Round B is executed and all its requirements are Complete (04-39 done 2026-11-28; Farm Weather Cycles addendum 04:37; Addendum 2 marked TYPE-05 and ENG-02 on the machine proofs of the debug fixes 62ec2f49 and f5d57b98, maintainer decision 2026-11-28). TYPE-03, TYPE-04, TYPE-05, WAPI-05, WAPI-06, WAPI-08, ENG-02 and ENG-03 are Complete. Next: execute Round C (/gsd-execute-phase 4 runs 04-40..04-56; 04-40 has a short maintainer DISPLAY9 off/on step, 04-56 is the final Round C checkpoint with DISPLAY9 attached and the unattended measurement); phase verification runs only after Round E. Open findings: D13 (occasional duplicate mousedown), the WebView2 software decode of the 4K VP9 clip (Round D), BROWSER_EXIT_WAIT 1.5 s is still exceeded on a fully saturated CPU (16 busy threads; recorded as expected degraded behaviour), S3 expired.badssl.com did not load within 13 s in one of five runs (badssl.com slow, not blocking).

Progress: [████░░░░░░] 38%

## Performance Metrics

**Velocity:**
- Total plans completed: 79
- Average duration: -
- Total execution time: -

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1 | 0 | - | - |
| 02 | 29 | - | - |
| 03 | 50 | - | - |

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
| Phase 03 P30 | 6 min | 2 tasks | 3 files |
| Phase 03 P31 | 7 min | 3 tasks | 1 files |
| Phase 03 P34 | 7 min | 2 tasks | 2 files |
| Phase 03 P35 | 4 min | 2 tasks | 2 files |
| Phase 03 P29 | 7 min | 2 tasks | 3 files |
| Phase 03 P32 | 4 min | 2 tasks | 1 files |
| Phase 03 P36 | 13 min | 2 tasks | 3 files |
| Phase 03 P28 | 42 min | 3 tasks | 4 files |
| Phase 03 P33 | 13 min | 2 tasks | 7 files |
| Phase 03 P40 | 14 min | 2 tasks | 7 files |
| Phase 03 P37 | 60min | 3 tasks | 0 files |
| Phase 03 P38 | 25 min | 3 tasks | 0 files |
| Phase 03 P39 | 7 min | 3 tasks | 5 files |
| Phase 03 P41 | 12 min | 2 tasks | 3 files |
| Phase 03 P42 | 13 min | 2 tasks | 7 files |
| Phase 03 P43 | 40 min | 2 tasks | 0 files |
| Phase 03 P44 | 15 min | 1 tasks | 0 files |
| Phase 03 P45 | 11 min | 2 tasks | 9 files |
| Phase 03 P46 | 12 min | 3 tasks | 5 files |
| Phase 03 P47 | 25 min | 3 tasks | 2 files |
| Phase 03 P48 | 2 min | 2 tasks | 0 files |
| Phase 03 P49 | 2 min | 2 tasks | 0 files |
| Phase 03 P50 | 10 min | 1 tasks | 0 files |
| Phase 04 P01 | 32 min | 3 tasks | 18 files |
| Phase 04 P03 | 11 min | 2 tasks | 43 files |
| Phase 04 P02 | 21 min | 2 tasks | 14 files |
| Phase 04 P04 | 10 min | 2 tasks | 9 files |
| Phase 04 P05 | 19 min | 2 tasks | 17 files |
| Phase 04 P06 | 24 min | 2 tasks | 18 files |
| Phase 04 P07 | 18 min | 3 tasks | 15 files |
| Phase 04 P08 | 16 min | 2 tasks | 17 files |
| Phase 04 P09 | 22 min | 2 tasks | 18 files |
| Phase 04 P10 | 19 min | 2 tasks | 22 files |
| Phase 04 P11 | 31 min | 2 tasks | 12 files |
| Phase 04 P12 | 27 min | 3 tasks | 14 files |
| Phase 04 P13 | 29 min | 2 tasks | 21 files |
| Phase 04 P14 | 25 min | 2 tasks | 11 files |
| Phase 04 P15 | 21 min | 2 tasks | 14 files |
| Phase 04 P16 | 27 min | 2 tasks | 18 files |
| Phase 04 P17 | 17 min | 2 tasks | 20 files |
| Phase 04 P18 | 28 min | 3 tasks | 20 files |
| Phase 04 P19 | 20 min | 2 tasks | 20 files |
| Phase 04 P20 | 30 min | 3 tasks | 2 files |
| Phase 04 P21 | 30 min | 3 tasks | 2 files |
| Phase 04 P22 | 30 min | 2 tasks | 18 files |
| Phase 04 P23 | 14 min | 2 tasks | 8 files |
| Phase 04 P24 | 14 min | 2 tasks | 8 files |
| Phase 04 P25 | 15 min | 3 tasks | 11 files |
| Phase 04 P26 | 15 min | 2 tasks | 8 files |
| Phase 04 P27 | 9 min | 2 tasks | 5 files |
| Phase 04 P28 | 22 min | 2 tasks | 12 files |
| Phase 04 P29 | 26 min | 2 tasks | 13 files |
| Phase 04 P30 | 25 min | 2 tasks | 15 files |
| Phase 04 P31 | 11 min | 2 tasks | 7 files |
| Phase 04 P32 | 22 min | 2 tasks | 10 files |
| Phase 04 P33 | 21 min | 3 tasks | 14 files |
| Phase 04 P34 | 20 min | 2 tasks | 14 files |
| Phase 04 P35 | 19 min | 2 tasks | 17 files |
| Phase 04 P36 | 21 min | 2 tasks | 6 files |
| Phase 04 P37 | 35min | 3 tasks | 1 files |
| Phase 04 P38 | 75 min | 3 tasks | 3 files |
| Phase 04 P39 | 26 min | 3 tasks | 0 files |
| Phase 04 P40 | about 20 min (continuation) | 3 tasks | 9 files |
| Phase 04 P41 | 25 min | 3 tasks | 9 files |
| Phase 04 P42 | 17 min | 2 tasks | 10 files |
| Phase 04 P43 | 24 min | 3 tasks | 14 files |
| Phase 04 P44 | 18 min | 3 tasks | 13 files |

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
- [Phase 03]: 03-30: an unreadable Sucrose.json or backup makes Store::open fail with StoreError::Io (host runs in no-save mode); only Missing or Corrupt files lead to a restore or migration
- [Phase 03]: 03-31: a settled monitor snapshot during a library move is kept (latest wins) and applied by Fleet::finish_move after the job names the new library; respawn_closed waits for the move's end
- [Phase 03]: 03-34: a load_source's new start is the first playback restart after the FILE_LOADED of its own request (SourceSwitch, generation read at START_FILE), not the next restart
- [Phase 03]: Plan 03-35: after a failed startup enable/disable call set_startup settles RunStartup by the OS-reported entry state; the Task Manager flag removal is best effort
- [Phase 03]: 03-29: one shared pixel-area limit (MAX_PIXELS = 32 Mi pixels) beside MAX_SIDE, checked by thumbs::check_dimensions right after into_dimensions in the thumbnail, background and preview paths
- [Phase 03]: 03-32: the Quit path exits through the common end-of-pass check (quit_done plus may_exit), so one rule holds every quitting exit while a library move runs; a move that ends while quitting starts no renderer
- [Phase 03]: 03-36: a page-log round starts when the Portal leaves page 1 and ends at the first page-1 event after the last page; at least 20 s of idle after the last round end are required
- [Phase 03]: 03-36: the sampler treats a closed Portal as a normal end (ended_by portal-exit); only the page-log summary accepts such a series
- [Phase 03]: 03-28: fix the missing element id on the card preview img() instead of overriding the OS reduced-motion preference; behavior difference recorded in docs/FEATURES.md
- [Phase 03]: 03-33: Connection::disconnect ends a connection for every handle (DisconnectNamedPipe in sucrose-platform); PortalRegistry::send ends the link it drops; every ended Portal connection re-checks the exit rule (WR-04)
- [Phase 03]: [03-40] Attribution verdict order: not recorded first, then within the cap (48 MiB), then the part that is at least half of the private growth, else unattributed
- [Phase 03]: 03-39: both D-25 reruns pass the rule as written; atlas release (03-27) confirmed as the source of the 500-theme growth; real-library round-3 attribution 'unattributed' recorded as a caveat
- [Phase 03]: 03-41: the Root fill is transparent only while a backdrop shows; theme tokens stay opaque, one deferral applies it after open_window mounts the Root
- [Phase 03]: 03-42: thumbnails follow the scroll position (rows in view plus one row ahead); the cache never evicts a wanted thumbnail and keeps its 64 / 48 MiB caps otherwise
- [Phase 03]: 03-43: maintainer check found Mica/Tabbed backdrop and the library scrollbar still failing; recorded as leads, UI-01/UI-02/LIB-04/LIB-06 left open for gap closure
- [Phase 03]: 03-44: the maintainer skipped the D-25 scenario 2 rerun (UAT test 10 stays open); no measurement taken, UI-07 not marked Complete
- [Phase 03]: 03-45: Sucrose sets the DWM system backdrop type itself through Platform::set_window_backdrop and applies Opaque before every changed GPUI appearance (theme::appearance_steps), because gpui-pre-windows never clears Acrylic or the DWM type (WR-14)
- [Phase 03]: 03-46: library scrollbar is a sibling overlay of the scroll area on the grid's own ScrollHandle (vertical_scrollbar on the scrolled element moves with the cards); visible_rows clamps the offset and settles on the painted geometry through Window::on_next_frame with exact raw-read comparison
- [Phase 03]: 03-47: frame_extension not-needed, backdrop_next closed, backdrop_confirmed yes - Mica, Tabbed, Acrylic and None confirmed by eye in Dark and Light; grid scrollbar and WR-16 confirmed in both OS modes
- [Phase 03]: 03-50: maintainer skipped the D-25 scenario 2 rerun on the final thumbnail and scrollbar path (reply: skip); no measurement, UI-07 stays open
- [Phase 03]: UAT tests 10/11 (verify-work, 2026-11-25): D-25 scenario 2 on the final thumbnail and scrollbar path passes (working set final 163 against 168 MB, round 3 end 177 against 184; private 155 against 161, 170 against 175); UI-07 Complete, phase 03 closed
- [Phase 04]: Planned in five rounds along the D-31 slices (maintainer decision 2026-11-26, after the planner returned PHASE SPLIT RECOMMENDED; no decimal phases). Round A = slice 1 = plans 04-01..04-21. Coverage gate override, approved with that choice: the 29 requirements and 20 CONTEXT decisions of slices 2-8 are deferred to Rounds B-E, not dropped; every Round A requirement and decision is covered. Phase verification runs only after Round E has executed.
- [Phase 04]: Research open questions resolved with the recommended options (2026-11-26): D-32 CEF delay-load with a sucrose-cef.exe fallback, D-33 first web instance per engine keeps the persistent profile, D-34 sucrose-sysinfo unsafe-allowed, D-35 web data in <data_dir>/Web, D-36 process-event spike with a bounded 2 s ToolHelp fallback
- [Phase 04]: D-32 holds: libcef.dll is delay-loaded into sucrose.exe, the service and mpv renderers run without it; no sucrose-cef.exe fallback
- [Phase 04]: CEF on Windows x86 is left out of the cef cfg: the i686 Rust target was not installed, so it could not be built (A3); 04-12 reports CEF unsupported on x86
- [Phase 04]: 04-03: ordered JSON objects are read through a map visitor into Vec pairs; unknown control types keep their raw object, unknown keys survive a write
- [Phase 04]: 04-03: dropdown valuetext is computed from items and value as in the legacy model; Control::to_json_value gives the legacy page form (empty text, null help)
- [Phase 04]: D-036: CEF is built against the pinned dist and delay-loaded; CEF subprocesses are sucrose.exe, no sucrose-cef.exe fallback
- [Phase 04]: A CEF renderer answers LoadSource by navigating its browser; Use between a Video and a Web theme restarts the renderers (UseDecision::Restart)
- [Phase 04]: 04-04: choose_engine tries CEF before WebView2 wherever a web engine stands in; a configured engine outside the type's allowed list counts as the type's default; Choice.fell_back_from is also set when no engine is available
- [Phase 04]: 04-05: EngineClock and Engine carry what the renderer really calls (wait_event, start_failure, loop_gen, set_audio, name); the pause level travels in PlaybackSettings.pause
- [Phase 04]: 04-05: the service offers CEF through its static availability report only for Web themes, so a video configured for CEF keeps mpv until generated pages exist
- [Phase 04]: 04-05: CefEngine.load_source is Unsupported; a Use that involves a Web theme starts a new renderer
- [Phase 04]: 04-06: Reject gains TooLong and BadUtf8 and ResolvedFile carries its root index; a link out of one root is Outside unless a later root serves the name from inside itself
- [Phase 04]: 04-06: the CEF scheme handler is registered in CefHost::start between initialize and browser creation, so the first navigation already goes through the virtual origin
- [Phase 04]: CEF 156 has no ignore_certificate_errors settings field; the browser process appends Chromium ignore-certificate-errors only when Allow invalid certificates is on (D-037)
- [Phase 04]: A failed profile acquisition leaves a CEF renderer on in-memory caches with a warning instead of failing the wallpaper (D-038)
- [Phase 04]: 04-08: sucrose-ipc depends on sucrose-engine-web for the Switch type; an empty CefArguments/WebArguments means the legacy defaults at job creation; Sucrose's switch wins over the user's same-name switch; debugging port sums below 1024 are refused
- [Phase 04]: Plan 04-09: serde_json preserve_order is enabled workspace-wide so control objects keep the legacy key order (indexmap already locked)
- [Phase 04]: Plan 04-09: non-data compatible calls run in the legacy code order (js::LEGACY_RUN_ORDER), not the engines.md table order of CompatibleKey::ALL; plans 10 and 11 should push data keys in the same order
- [Phase 04]: Plan 04-09: Portal.ThemeMode System is sent as the legacy ThemeType name Dark until the OS app theme is read
- [Phase 04]: Plan 04-09: a CEF page that finishes loading while paused is paused again after its calls
- [Phase 04]: D-039: sucrose-sysinfo is an unsafe-allowed provider crate (D-34)
- [Phase 04]: The data hub exists only while a running page needs data and never reads PerformanceCounter (D-24 behaviour change)
- [Phase 04]: Bios and Motherboard are asked for every 10th tick (sampler caches the WMI read) so a page that starts later gets them within a second
- [Phase 04]: CoresMin and CoresMax keep the legacy per-core-loop quirk; the legacy behaviour is kept and recorded in FEATURES
- [Phase 04]: Processor State is true only when the WMI static read worked, as in the legacy code; a failing source gives the legacy default with State false and one log line, retried after 60 s
- [Phase 04]: 04-12: EngineReport.availability_for adds the build rule that CEF plays Web themes only until its generated GIF/video pages exist
- [Phase 04]: 04-12: a type no engine can play is refused with UseRefusal::NoEngine (appended) and the configured engine's repair hint
- [Phase 04]: EngineReportView carries supported and offered so the Portal never decides which types play or which engines exist (04-13)
- [Phase 04]: A Portal card waits for the engine report: with none it is Disabled(Unavailable) (04-13)
- [Phase 04]: A saved engine that cannot be used now leaves the running wallpaper as it is; the setting is kept (D-11, 04-13)
- [Phase 04]: 04-14: Loop and Shuffle Mode apply live through appended SetLoop/SetShuffle renderer messages and also update the job and shared assignments; Hardware Acceleration and interaction settings are read at the next wallpaper start
- [Phase 04]: 04-15: the Warehouse snooze fields hold the quiet-until time (C# DateTime.Now.AddDays(1)); SnoozeFallback writes now plus 24 h as UTC with an offset and notify is false only while it is ahead
- [Phase 04]: 04-15: the row alert names the running fallback, else the first installed engine the report offers for the type
- [Phase 04]: Plan 04-16: a restart rebuilds the job from the saved settings and hands the engine and web options to every slot's assignment, so restarted renderers read the new certificate switch, switches and hardware acceleration
- [Phase 04]: Plan 04-16: the service owns the CEF switch rules and storage; GetArguments answers an empty stored dictionary with the legacy defaults, so an empty list cannot be stored as no switches; limits 64 switches, 64-byte names, 512-byte values keep the answer inside one IPC frame
- [Phase 04]: 04-17: the renderer confirms Open DevTools with an appended ToService::DevToolsOpened(bool); the service answers the Portal on it or after 5 s as failed
- [Phase 04]: 04-17: the Portal enables Open DevTools by web engine; the service refuses wallpapers that are not Web, Url or YouTube (a Video on a web engine ends in the error toast)
- [Phase 04]: 04-18: the Microsoft root policy runs with the application-root flag; revocation soft-fails only when no source answers; a Cancelled stage ends a stopped install
- [Phase 04]: Keys follow the per-monitor cursor rule like the mouse: only the renderer whose rectangle holds the cursor forwards keystrokes
- [Phase 04]: The legacy key lParam is kept exactly, extended-key bit always set; the OnlyKeyboard focus click is an InputEvent::FocusClick made by the renderer
- [Phase 04]: Forwarded positions are relative to the renderer rectangle; InputEvent Debug never shows which key it was (T-04-45)
- [Phase 04]: 04-20: Complete rule case 2: step 1 failed and steps 5, 6, 8 were skipped, so requirements-completed is empty and no requirement is marked Complete; the Shut Down Windows dialog defect is a blocker for /gsd-debug
- [Phase 04]: 04-21: M3 CEF row records Rust alone because --row refused the pair (no Rust GPU adapter, G-02-6); C# CefSharp numbers in the paragraph; Rust RAM lower and CPU higher recorded as found, not tuned
- [Phase 04]: Round B planned (2026-11-26): 04-22..04-38 for D-31 slices 2 and 3 (WebView2; built-in GIF/APNG/WebP, generated pages, URL, YouTube), threat IDs T-04-50..T-04-81. Maintainer decisions: add plans and leave Round A unchanged; the final checkpoint 04-37 reruns the failed or skipped 04-20 steps on CEF and WebView2 and can mark TYPE-03, ENG-07, WAPI-01, WAPI-02, WAPI-04 and FMT-07; the open 04-20 defects go to /gsd-debug and are its precondition; step B6 (two monitors with DISPLAY9) is a required ENG-02 proof.
- [Phase 04]: Round B coverage gate override (approved deferral, 04-CONTEXT D-31): the 21 requirements of Rounds C–E (TYPE-06, ENG-04, ENG-05, DESK-02..04, PLAY-02..04, PERF-01..04, PERF-06..09, PROP-01..04) and their 16 CONTEXT decisions (D-01, D-03, D-05, D-07, D-08, D-18..D-21, D-23, D-25, D-27..D-30, D-36) are deferred to their round, not dropped; every Round B requirement is covered.
- [Phase 04]: Round B re-check planned (2026-11-28): 04-39 (wave 38, after 04-38) re-runs the 04-37 steps that failed or were not checked after the 04-37 debug batch and quick batch 261127-uor, plus the D-040 storage eye check and fast WebView2 switching (maintainer choices); threat IDs T-04-82..T-04-87. It can mark TYPE-03, TYPE-04, TYPE-05, ENG-02, ENG-03, WAPI-05, WAPI-06 and WAPI-08; its "Requirement parts per engine" table names the proof of every requirement part on CEF and WebView2. Coverage gate override as for Round B (approved deferral, 04-CONTEXT D-31): the same 21 Round C–E requirements and 16 decisions stay deferred to their round. Plan checker: 9 passes (the haiku checker found a new item at each full pass; the maintainer chose "fix and check" each time); the final pass had 0 blockers and 0 warnings.
- [Phase 04]: Round C planned (2026-11-28): 04-40..04-56 (waves 39..55) for D-31 slices 4 and 5 (display modes and per-monitor wallpapers with Display Preferences; the properties editor), threat IDs T-04-88..T-04-124. Maintainer decisions: add plans and leave 04-01..04-39 unchanged; D-08 confirmed (`Engine.DisplayAssignments` of `{MonitorId, DeviceName, Theme}`, monitorDevicePath with the GDI name as fallback, an EDID second key only if the 04-40 spike needs it); measurement unattended inside the final checkpoint 04-56 (no separate maintainer measurement plan, no C# comparison). 04-56 can mark DESK-03, DESK-04 and PROP-01..04; DESK-02, DESK-05 and PROP-05 are partial in Round C and complete in Round E. Planned decision entries: D-042 (per-monitor assignments) and D-043 (web pages and the built-in GIF follow a shared time base kept by the service; mpv keeps D-030). Plan checker: 1 pass, 0 blockers, 0 warnings, 3 info (the haiku checker said it checked structure and coverage, not every task action line by line).
- [Phase 04]: Round C coverage gate override (approved deferral, 04-CONTEXT D-31): the 14 requirements of Rounds D and E (TYPE-06, ENG-04, ENG-05, PLAY-02..04, PERF-01..04, PERF-06..09) and their 7 CONTEXT decisions (D-23, D-25, D-27..D-30, D-36) are deferred to their round, not dropped; every Round C requirement is covered.
- [Phase 04]: 04-22: WebView2 answers the virtual origin through sucrose_engine_web::serve::plan (shared with CEF); a Range request gets the whole file and 200 until 04-23
- [Phase 04]: 04-22: the WebView2 page gets no host object and no message channel; environment creation retries once after 2 s with a 30 s capped message pump
- [Phase 04]: 04-23: a WebView2 range body is a memory stream of at most 4 MiB (serve::MAX_RANGE_BODY) whose Content-Range names the bytes sent; whole files stream from disk
- [Phase 04]: 04-24: PageState owns data snapshot JSON parsing (store_data); CEF keeps its own State only for coalesced mouse moves
- [Phase 04]: 04-24: WebView2 runs page calls inside NavigationCompleted before the loaded hook, so DataWanted follows the page and its calls as on CEF
- [Phase 04]: 04-25: the WebView2 browser arguments are planned in sucrose-engine-web (plan_webview2_command_line, browser_arguments) and the legacy key table is shared as sucrose_engine_web::keys; DevTools stay enabled but closed unless asked (D-13)
- [Phase 04]: 04-26: WebArguments reuses the CEF switch validation and is stored in the legacy Engine.WebArguments list; one ArgumentsEditor per web engine routed by engine
- [Phase 04]: 04-27: installable() is architecture-aware: runtime and VC++ hints are installable only where the prerequisite has a download address on this processor
- [Phase 04]: 04-28: default render device change noticed by polling the device id once a second, not IMMNotificationClient (needs a direct windows-core dependency, T-04-SC)
- [Phase 04]: 04-28: the audio capture feeds silence by elapsed time so bars fall to 0 on a device that plays nothing; a running silent capture publishes 128 zeros
- [Phase 04]: 04-29: GSMTC change handlers only signal the media worker; the worker reads and updates the record (WinRT reads block, so they stay off thread-pool callbacks)
- [Phase 04]: 04-29: a chunk set that did not fit a renderer's queue is offered again as a new blob id; art over the 2 MiB BLOB_MAX is replaced by an empty ThumbnailString (04-37 to decide whether to raise the cap)
- [Phase 04]: 04-30: the built-in GIF renderer opens its Animation on the decode thread (image's frame iterator is not Send); start waits for the first frame so a damaged file is a start failure
- [Phase 04]: 04-30: decide_use loads in place only between two mpv wallpapers; every other engine change restarts the renderers (UseInput.running_engine replaces running_kind)
- [Phase 04]: 04-31: the built-in GIF decoder is stretch-aware (Within, Cover, Crop) so None shows the centre at 1:1 and UniformToFill does not upscale a fitted frame; a mode change needing another preparation re-opens the decoder at the shown frame
- [Phase 04]: 04-31: the third D2DERR_RECREATE_TARGET in a row switches the built-in GIF renderer to a GDI StretchDIBits presenter (a success in between resets the count); the GDI path is only unit-tested (assumption A15)
- [Phase 04]: 04-32: the frame delay rule is the legacy one for GIF, APNG and WebP (0 ms plays at 100 ms, other delays as given); a one-frame file is sent as Message::Still, shown once with no frame timer
- [Phase 04]: 04-33: property_calls_of clamps slider values to their bounds while a theme's own file is passed as is; the generated pages' volume control uses Property.Localization.Volume keys with an English PropertyLocalization table
- [Phase 04]: 04-33: GifContent.html keeps the legacy gifuct-js bundle (MIT, Matt Way) with its notice added; the legacy page shipped it without one, maintainer to confirm
- [Phase 04]: 04-34: the theme source is a typed ThemeSource (file or address); Url and YouTube Sources pass an http/https-only address gate
- [Phase 04]: 04-34: only consecutive mouse moves are coalesced (MoveSlot); web pages take input by page kind (Web and Url yes, generated Gif and Video no)
- [Phase 04]: 04-35: The YouTube page's SucroseStretchMode is a no-op (the player fills the surface; legacy YouTube is not stretched), so the engines' stretch path stays uniform for every generated page
- [Phase 04]: 04-35: referer_for fills only a missing Referer for https youtube.com and youtube-nocookie.com hosts (subdomains included) with the virtual origin and never replaces one; CEF returns a request handler only for those hosts
- [Phase 04]: 04-35: the YouTube page keeps the engine volume at 0 until the first SucroseVolume call and applies the product with the property volume once the player is ready (a pause before ready is applied on ready too)
- [Phase 04]: 04-36: Gif select order follows the legacy allowed list (Built-in, WebView2, CEF, mpv, libVLC), as the Video row does, not the plan prose order
- [Phase 04]: 04-36: a stored VexanaLive shows as the single Built-in choice (XavierLive); VexanaLive is never a second list entry
- [Phase 04]: 04-37: Complete rule applied per requirement; TYPE-02, ENG-07, WAPI-01, WAPI-02, WAPI-04, FMT-07, WAPI-09 and WAPI-10 marked, TYPE-03, TYPE-04, TYPE-05, ENG-02, ENG-03, WAPI-05, WAPI-06 and WAPI-08 stay open (defects D1-D8 recorded)
- [Phase 04]: 04-37: D9 (D-33) design finding, with two monitors the first renderer to start takes the persistent web profile so the primary monitor can run ephemeral; proposal to prefer the primary monitor awaits the maintainer
- [Phase 04]: 04-37 D9 decided by the maintainer (2026-11-27): with several monitors the persistent web profile goes to the primary monitor (amends 04-CONTEXT D-33); build it together with the D10 fix (slot-1 profile folder still locked after a quick switch, os error 32)
- [Phase 04]: 04-29 album art over the 2 MiB blob cap: re-encode (downscale) the art so it fits under BLOB_MAX, do not raise the cap (maintainer left the choice to the orchestrator; low resource use, the page only shows a thumbnail)
- [Phase 04]: 04-33 GIF decoder bundle in GifContent.html (gifuct-js + jsBinarySchemaParser, MIT upstream, notice added): license reading confirmed by the maintainer (2026-11-27)
- [Phase 04]: 04-38: M3 WebView2 row is Rust-only because --row refused the pair (GPU adapter sets differ, G-02-6); C# numbers are in the paragraph; the tool was not changed
- [Phase 04]: 04-38: built-in GIF renderer uses less RAM than Rust/C# mpv but more CPU and GPU time; recorded as found, flagged for /gsd-verify-work, not tuned
- [Phase 04]: 04-39: Complete rule case 1 for TYPE-04, WAPI-05, WAPI-06, WAPI-08; case 2 for TYPE-03 and ENG-03 (Farm Weather Cycles never Used on CEF) and TYPE-05 and ENG-02 (P1a-WV2 run 1 measured a FAIL: start index 17 with Shuffle Mode off); reruns passing does not undo the FAIL. Addendum 2026-11-28 04:37: Farm Weather Cycles Used on CEF, maintainer "ok", page loaded 200 on both renderers, GPU exits 0, so TYPE-03 and ENG-03 are Complete as well (rule case 1). Addendum 2 (2026-11-28): after the debug fixes 62ec2f49 (playlist first start waits for setShuffle; P1a-WV2 10/10 at index 0) and f5d57b98 (BROWSER_EXIT_WAIT 1.5 s; 72 closes, no WARN), the maintainer chose to mark TYPE-05 and ENG-02 Complete on the machine proofs without a new eye check (rule case 1). C2/F3 (a Web theme appears on both monitors 0.7 to 2.4 s after a switch on WebView2) accepted by the maintainer 2026-11-28 ("C2 kabul"), no fix planned
- [Phase 04]: 04-40: device path stable across replug: yes (DISPLAY9 MTT1337 id unchanged after a projection switch and a Device Manager disable/enable); 04-41 keeps monitorDevicePath as the monitor id, no EDID second key needed
- [Phase 04]: D-042: Engine.DisplayAssignments remembers per-monitor wallpapers by the stable monitor id; renderers are grouped by theme (one clock and one sound per theme)
- [Phase 04]: 04-42: RuntimeState.selected is the theme on the target monitor in Per display; Remove Wallpaper(None) in Span and Duplicate clears Library.Selected; display view texts are cut at 128 bytes
- [Phase 04]: D-043: renderers of one theme on several monitors share a service-kept epoch (Assignment.time_base, ToRenderer::TimeBase); the built-in GIF renderer follows it at loop wraps, mpv keeps the D-030 clock
- [Phase 04]: Span is one pseudo-monitor span:all over the union of display bounds or work areas; a Display Mode change diffs target lists and loads or restarts only the renderers whose theme changes
- [Phase 04]: 04-44: the CEF time base shim is registered through DevTools Page.addScriptToEvaluateOnNewDocument (in-process browser host, browser made on about:blank first); WebView2 uses AddScriptToExecuteOnDocumentCreated
- [Phase 04]: 04-44: only endless CSS animations are aligned to the epoch after load, and Date is never shimmed

### Pending Todos

- [Phase 4]: Small tasks after the 04-37 debug batch (asked by the maintainer on 2026-11-27):
  - DONE 2026-11-27 (quick 261127-uos, bd84d9af..93e3f099; D-040): (a) D9 + D10: give the persistent web profile to the primary monitor (the D9 decision, amends 04-CONTEXT D-33) and fix the slot-1 WebView2 profile folder that is still locked after a quick switch (os error 32)
  - DONE 2026-11-27 (quick 261127-uot, 5de6cf69..e54c0523; maintainer saw the FLAC cover, 3145928 -> 167404 chars, 768x768): (b) 04-29 album art over the 2 MiB blob cap: re-encode (downscale) the art so it fits under BLOB_MAX instead of replacing it with an empty ThumbnailString; the cap stays. Seen in a real run on 2026-11-27 (audio-bars-silent maintainer check, both engines): while a local FLAC track played, the cover did not show; the service logged "the album art is too large for the renderer, the page gets none size=3145928" at each page start (10 times, 5 on WebView2 and 5 on CEF), i.e. 3.0 MiB of base64 for about 2.25 MiB of art, and no thumbnail read error
  - DONE 2026-11-27 (quick 261127-uou, 72e4b592..7b0f4abd; cargo test --workspace passes with DISPLAY9 attached): (c) apps/sucrose/tests/portal.rs desktop tests that assume one monitor: count the `engine started` lines per renderer slot, so they pass with several monitors
  - DONE 2026-11-27 (quick 261127-uov, faf94eb8..a4048b8f; PROTOCOL_VERSION 11): (d) General.UserAgent applied on both web engines as the legacy app does (CefSettings.UserAgent, CoreWebView2.Settings.UserAgent; default `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit`); the docs/FEATURES.md YouTube row says Not yet
  - DONE 2026-11-28 (quick 261128-0du, cd43de7e..9e5c67e6; Engine.WebViewSmartScreen, PROTOCOL_VERSION 12, PORTAL_PROTOCOL_VERSION 8; maintainer saw the switch and the WebView2-only prompt): (e) WebView2 SmartScreen setting per D-041 (the maintainer's decision on 2026-11-27: the user chooses, off by default; webview2-browser-exit-slow): planned key `Engine.WebViewSmartScreen` (boolean, default false; a migrated, missing or non-boolean value reads false), a toggle on the Wallpaper page under the WebView2 engine row, the restart prompt as for Allow invalid certificates (only WebView2 wallpapers restart); on, Sucrose's own WebView2 switches leave `msSmartScreenProtection` out; BROWSER_EXIT_WAIT stays 1 s (D-041 says why); when it lands, rename the test args_webview2_always_runs_without_smartscreen_and_cef_never_gets_the_switch and update the docs/FEATURES.md Web and web-data rows

### Blockers/Concerns

- [Phase 2]: G-02-7 is an accepted limitation (D-031), not a fix: 02-UAT.md keeps it failed; the cause of the one ~90 s UAT episode is not established
- [Phase 2]: 7 info-level review findings stay open (IN-19 … IN-23, IN-26, IN-27 in 02-REVIEW-DISPOSITION.md)
- [Phase 3]: 16 info-level review findings stay open (03-REVIEW-DISPOSITION.md); touches a success criterion
- [Phase 3]: the Portal logs six "invalid window handle" ERROR lines (0x80070578 / 0x80040102) when its window closes (first seen in 03-37); not triaged, measurements unaffected
- [Phase 5]: DESK-10 (lock screen) needs a `docs/DECISIONS.md` entry on the mechanism before it is built (D-026)
- [Phase 5]: STOR-01 and FMT-05 depend on D-022 (deferred); use the existing v8 endpoints until the API rewrite
- [Phase 5]: Update signing key scheme still open (Ed25519 proposed, D-019)
- [Phase 6]: No Apple Developer ID yet (D-025); macOS packages stay unsigned, manual tests need a real Mac
- [Phase 7]: Linux package formats are decided in Phase 7
- [All phases]: Measure RAM/CPU of user-visible changes in release builds; record notable numbers in `docs/MEASUREMENTS.md`
- [Phase 4]: RESOLVED 2026-11-26 (/gsd-debug, `.planning/debug/resolved/cef-close-shutdown-dialog.md`): the Windows "Shut Down Windows" dialog after every CEF theme switch (04-20 run). Cause: our CEF life span handler kept the default DoClose, so CEF posted WM_CLOSE to the browser's top-level ancestor, which under the desktop surface is Progman; the close then timed out after 5 s and the renderer crashed in cef::shutdown (0xffff7003). Fix: DoClose answers 1 for the wallpaper browser and the renderer destroys the browser window itself. Maintainer run 2026-11-26: no dialog, 14 of 14 CEF closes in 4-28 ms, every CEF renderer exit code 0. The 04-20 slice-1 check still needs its gap-closure re-run.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/cef-web-themes-stuck-loading.md`): the CEF GPU process crashing 3 times at every start (concern since the 04-20 checkpoint), the 04-20 CORS-like error of Test Sucrose API, 04-37 D1 (CEF Web themes stuck on loading), D4 (CEF seek fixture video), the CEF half of D6 (url-input black on CEF) and Music Tunnel misbehaving on CEF. Causes: sucrose.exe had no Windows 10 compatibility manifest (CEF's GPU process is the same executable and crashed, so no page got WebGL); Chrome dropped --disable-web-security because --user-data-dir was not on the CEF command line; the CEF scheme handler did not implement Skip, so every Range starting past byte 0 failed. Fix a0e22d9a: build.rs has the MSVC linker embed the manifest (/MANIFEST:EMBED, no build dependency), the CEF browser process gets --user-data-dir set to its profile folder, the scheme handler implements Skip. Maintainer run 2026-11-27: themes open, the seek video reaches the middle of the clip, url-input and Music Tunnel render on CEF, "GPU process exited" 0 times. Still open: D2 (white border), D8 (audio bars), the WebView2 mouse half of D6. 04-38 measurements must use a build with this fix (a0e22d9a or later): CEF now renders on the GPU, and the 04-21 M3 CEF row was taken in software rendering after the GPU crashes.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/webview2-certs-video-input.md`): 04-37 D3 (WebView2 certs frame after Allow invalid certificates), D5 (WebView2 video-clip page black) and the WebView2 half of D6 (url-input no mouse reaction). Causes: WebView2 accepted bad certificates only through ServerCertificateErrorDetected/ALWAYS_ALLOW, which holds each allowed request about 10 s (the frame appeared 10-21 s after a start); VideoContent.html was ported with a doctype (standards mode), so the built-in scale call's body transform collapsed the position:fixed video to 0 px (on CEF too); input was posted to Chrome_WidgetWin_0, WebView2's host window in our own process, instead of its Chrome_WidgetWin_1 child (and url-input only draws while dragging). Fixes: cf8f7d6a --ignore-certificate-errors when the setting is on (as CEF), 44758204 html,body{height:100%} in VideoContent.html, 747bbf92 input goes to the Chrome_WidgetWin_1 child. Maintainer run 2026-11-27 (release 747bbf92): D3, D5 and D6 confirmed.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/web-loading-white-border.md`): 04-37 D2 (white border at the screen edges while a Web theme loads). Cause, CEF only (every CEF wallpaper: Web, URL, YouTube and the generated pages): the CEF browser window was created visible around Chromium's widget window Chrome_WidgetWin_1, which has WS_THICKFRAME; Windows painted the classic 7 px sizing frame of that child into the layered desktop surface (at creation and again on later non-client messages), and nothing painted over it until the GPU process presented the page's first frame, 1-3 s later. WebView2's windows have no sizing frame and showed none in the probe; the WebView2 half of the report is explained by the CEF YouTube wallpapers that ran between the WebView2 Web themes in the 04-37 run (YouTube stayed on CEF). Fix 6c6d9a3c: the browser window is created hidden, WS_THICKFRAME is taken off the widget (SWP_FRAMECHANGED), then the window is shown with SW_SHOWNOACTIVATE. Desktop probe: 0 framed captures in 12 CEF starts after the fix, framed captures again on revert. Maintainer run 2026-11-27 (release build of the fix, Web, URL and YouTube on CEF, then WebView2): no frame on CEF and none on WebView2.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/youtube-player-ui-visible.md`): 04-37 D7 (YouTube player UI, title, play/pause and bottom UI, visible on both engines). Cause: YouTube's embed now draws its controls (title and channel, previous/play-pause/next, share, More videos, logo, time, progress bar, end screen) in a layer next to the player (#player-controls, #bottom-sheet-wrapper, outside #movie_player) and shows it for 5-8 s at every start, on resume after a pause and around the end of a video; the legacy hideIframeElement, ported verbatim, only hides the inside of #movie_player; YouTube also resets that layer's inline style while it starts. Not the user agent and not frame access (the frame is reachable on both engines). Fix adfb0922: the page also hides every element beside the path from #movie_player up to the frame's body (inline display:none !important plus a data-sucrose-hidden mark hidden by the injected style sheet), runs the pass at once and after every change, and skips it while the frame has no player yet. Maintainer run 2026-11-27 18:55 (release adfb0922, Showcase YouTube themes, CEF then WebView2): "hiç bir şey görünmedi ... webview2 de sorunsuz" (nothing showed on CEF, no problems on WebView2); captions and pause were not tested by the maintainer and were machine-verified on both engines (per-animation-frame monitor in the player frame: 0 frames with UI during the Light pause and resume, a player-API pause and play, the end of the video and the loop restart, while a positive control showed YouTube's layer on resume and at the end; captions on through the caption property stay visible over the video inside #movie_player, with 0 frames of other UI). Follow-up not part of D7: General.UserAgent is not applied by the Rust web engines yet (Pending Todos).
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/keyboard-input-doubled.md`): 04-37 D11 (one key press reacts twice, both web engines). Cause: the desktop surface is a child window of Progman, so Windows gives the renderer's thread, explorer's desktop thread and (WebView2) the browser's widget thread one shared input state; once Chromium's widget holds its keyboard focus (CEF takes it at navigation, a forwarded click can give it on either engine), Windows delivers the real keystroke to the page while the desktop is in front, and the renderer posted its raw-input copy on top. Fix 6fb2c103: a key is posted only while the keyboard focus of the foreground window's thread is not inside the renderer's surface (GetGUIThreadInfo + IsChild); mouse forwarding unchanged. Maintainer run 2026-11-27 on the release build of the fix: one step per press on CEF and on WebView2, and a read-only page probe counted exactly one keydown and one keyup per press on both. Follow-ups D12 and D13 below.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/cef-start-steals-focus.md`): 04-37 D12 (CEF takes the foreground at wallpaper start). Cause: CEF focused and activated its own widget twice while the wallpaper browser was made: CreateHostWindow shows Chromium's widget with Widget::Show, which without WS_EX_NOACTIVATE in the window info uses SW_SHOWNORMAL, SetForegroundWindow and SetFocus, and LoadMainFrameURL asks OnSetFocus(FOCUS_SOURCE_NAVIGATION), which calls ::SetFocus when no focus handler cancels it; the surface shares explorer's desktop input state, so that took the desktop's keyboard focus and activated Progman, and when the new renderer was allowed to set the foreground the user's window lost it. The port had dropped the two defaults of the legacy CefSharp.Wpf.HwndHost control. Fix d860631f: the window info has WS_EX_NOACTIVATE and a focus handler cancels the navigation focus of the wallpaper browser (popups and System requests keep CEF's default); the page gets the focus from the first forwarded click. Probe: the desktop focus moved at a start on the pre-fix build and with the focus handler alone, never with the full fix. Maintainer run 2026-11-27 21:24 +03 (release build of the fix, Notepad in front): Notepad kept the foreground and its focus through the start, and after one click on the desktop a read-only page probe counted exactly one keydown and one keyup per press (D11 still holds). Docs: 06421720 corrects the FEATURES.md Web row (the legacy CEF engine was the windowed CefSharp.Wpf.HwndHost control, not off-screen).
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/audio-bars-silent.md`): 04-37 D8 (audio bars silent in Music Tunnel and the audio fixture on both engines, now playing unconfirmed). Not a code defect: the 04-37 B5 step named Music Tunnel as a bars theme, and the maintainer tried Music Tunnel and Neomorphic Music Player, which draw no spectrum at all (they only read the media-session fields, in the C# app too); the audio fixture, which draws Data, was never opened in B5. The spectrum and now-playing path works on both engines (CDP probe on the release build aceee813: SucroseAudioData about 10 times a second, Data/DataLeft/DataRight 128 values in [0,1] that follow the music and fall to 0 after a pause; title, artist and a data-URL cover in the fixture, Neomorphic and Music Tunnel; Ray Music Visualizer animates). Maintainer run 2026-11-27 21:49 +03 (Portal, WebView2 then CEF, own music): the bars move with the music, now playing is visible, CEF the same as WebView2. The 04-37 B5 step mapping must use Ray Music Visualizer or the audio fixture for the bars and Music Tunnel or Neomorphic Music Player for now playing (Music Tunnel has no bars by design); WAPI-05 and WAPI-06 can be re-proven with that mapping in the B5 re-run. The fixture's peak (max of Data) reaching 1.000 is the clamp at full scale, as in the legacy visualizer, not a parity gap; the missing cover of a local FLAC track in that run was the blob cap of Pending Todos (b). No code change.
- [Phase 4]: RESOLVED 2026-11-27 (/gsd-debug, `.planning/debug/resolved/webview2-browser-exit-slow.md`): the 04-37 D10 follow-up (quick 261127-uos: a closed WebView2 renderer's browser process sometimes took about 6.2 s to exit, so with the wait of c1ac7f05 a Web theme switch on WebView2 took about 7 s and the exit wait needed an 8 s bound instead of the planned 1 s). Cause: Edge SmartScreen, on by default in the WebView2 browser: on a profile folder that was never used (every ephemeral folder at every start, the persistent one on its first run) the first navigation sends a SmartScreen URL check, and a controller close while that check is in flight holds the browser's tear-down about 6.3 s (probe: 4 of 4 such closes slow with SmartScreen on, 0 of 26 with it off). Fix e747c7be: Sucrose's own WebView2 switches add `disable-features=msSmartScreenProtection` (D-041, Accepted 2026-11-27 by the maintainer: off by default, the user may turn it on), BROWSER_EXIT_WAIT back to 1 s; closes exit in 0.21-0.25 s, a switch on two monitors takes 0.82-0.94 s (release build), new desktop test a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once (fails with the switch removed). Resolved on machine evidence; the visual check (fast Web theme switching with DISPLAY9 attached, about 1 s per switch on both monitors) is pending in the combined maintainer session at the end of the quick batch. The setting that turns SmartScreen on is Pending Todos (e).
- [Phase 4]: 04-37 maintainer run (2026-11-27) found D1-D9, for /gsd-debug (details in 04-37-SUMMARY.md): D1 CEF Web themes stuck on loading; D2 white border at the screen edges on some Web themes (both engines); D3 WebView2 certs frame still fails after Allow invalid certificates + Restart now; D4 CEF seek fixture video never visible; D5 WebView2 video-clip page black; D6 url-input black on CEF and no mouse reaction on WebView2; D7 YouTube player UI (title, play/pause, bottom UI) visible on both engines; D8 audio bars silent in Music Tunnel and audio on both engines, now playing unconfirmed; D9 (D-33 design) the first renderer to start takes the persistent web profile, so the primary monitor can run ephemeral (decided 2026-11-27: the primary monitor gets it; to build with D10, the slot-1 WebView2 profile folder still locked after a quick switch, os error 32); D11 keyboard input reacts twice to one key press, engine not yet known (reported 2026-11-27 18:2x +03 during the D2 check, after the run). TYPE-03, TYPE-04, TYPE-05, ENG-02, ENG-03, WAPI-05, WAPI-06 and WAPI-08 stay open for /gsd-verify-work. Status 2026-11-27: D1, D4 and the CEF half of D6 resolved by cef-web-themes-stuck-loading, D3, D5 and the WebView2 half of D6 by webview2-certs-video-input, D2 by web-loading-white-border, D7 by youtube-player-ui-visible, D11 by keyboard-input-doubled (6fb2c103), D12 (CEF takes the foreground at wallpaper start) by cef-start-steals-focus (d860631f), D8 by audio-bars-silent (not a code defect: the B5 step named themes without bars) (see above); D9, D10 and D13 (rare duplicate mousedowns) stay open.
- [Phase 4]: Open finding D13 (04-37, seen 2026-11-27 in the keyboard-input-doubled probe on the fixed build): a left click sometimes reaches a Web wallpaper as two mousedowns at the same or adjacent millisecond: 3 pairs among 37 mousedowns over both rounds (CEF 2 of 15: t=43431 twice and t=44659 twice in page time, the first after 13.6 s without a click and opening a burst of fast clicks, the second inside it; WebView2 1 of 22: t=31094 and t=31095, after 8.5 s without a click). Every mousedown was button 0, isTrusted true, with the document focused; the probe recorded no mouseup, event.detail, event.timeStamp, buttons, coordinates or pointerType, the focus watch has no capture column, and the renderer logs no input. Ruled out by reading the code: one raw record cannot give two downs (one LEFT_DOWN bit per record; down and up in one record give down then up), and the input queue, the render loop and both engines' post_input forward and post each event once. The keyboard cause does not carry over as such: real mouse messages are routed by hit test (the shell's icon window covers the desktop) or by capture, not by the keyboard focus, and a steady second channel would double every click. Open candidates: Chromium takes mouse capture when it handles our posted WM_LBUTTONDOWN, so the real WM_LBUTTONDOWN, taken from the shared input queue after that, reaches the captured widget too (a race); a bouncing mouse switch; something else. Next session: a read-only probe with mouseup order, detail, timeStamp and coordinates, an hwndCapture watch and raw record counts (counts only), and the same clicks in a normal browser window to rule out the mouse itself. Not fixed by 6fb2c103 (mouse forwarding unchanged).
- [Phase 4]: Open observation (04-37 D5 re-check, 2026-11-27): the 4K60 VP9 video-clip fixture decodes in software on WebView2 (VpxVideoDecoder, no platform decoder) and drops about 340 of 715 frames in 3 s; the maintainer sees a slight heaviness. Not part of D5; for the Round D performance work and the 04-38 measurement.
- 04-35: the CEF part is fixed (cef-web-themes-stuck-loading, 2026-11-27: Chrome dropped --disable-web-security without --user-data-dir; the player frame is reached on CEF now); the player UI that still showed on both engines was 04-37 D7, resolved by youtube-player-ui-visible (adfb0922)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 261122-723 | Target architectures (D-027): Windows x86/x64/ARM64, macOS universal, Linux x86_64/aarch64; CI matrix, docs, FND-02 | 2026-11-22 | 971149f4 | [261122-723-add-target-architecture-support-d-027-wi](./quick/261122-723-add-target-architecture-support-d-027-wi/) |
| 2 | Shorten the COVERAGE.md no-integration reason to under 200 chars so the phase 03 api-coverage verify:pre gate passes | 2026-11-25 | d33531f2 | — |
| 261127-uos | 04-37 D9+D10: persistent web profile on the primary monitor; WebView2 profile folder lock after a quick switch | 2026-11-27 | 93e3f099 | [261127-uos-04-37-d9-d10-give-the-persistent-web-profile-to-the-primary](./quick/261127-uos-04-37-d9-d10-give-the-persistent-web-profile-to-the-primary/) |
| 261127-uot | 04-29: album art over the 2 MiB blob cap is downscaled to fit instead of dropped | 2026-11-27 | e54c0523 | [261127-uot-04-29-album-art-over-the-2-mib-blob-max-is-re-encoded-or-dow](./quick/261127-uot-04-29-album-art-over-the-2-mib-blob-max-is-re-encoded-or-dow/) |
| 261127-uou | portal.rs desktop tests count engine starts per renderer slot (pass with DISPLAY9 attached) | 2026-11-27 | 7b0f4abd | [261127-uou-apps-sucrose-tests-portal-rs-real-desktop-tests-that-assume](./quick/261127-uou-apps-sucrose-tests-portal-rs-real-desktop-tests-that-assume/) |
| 261127-uov | General.UserAgent applied on CEF and WebView2 like the legacy app | 2026-11-27 | a4048b8f | [261127-uov-apply-the-general-useragent-setting-on-both-web-engines-cef](./quick/261127-uov-apply-the-general-useragent-setting-on-both-web-engines-cef/) |
| 261128-0du | (e) WebView2 SmartScreen setting per D-041: user-selectable, off by default, toggle under the WebView2 row | 2026-11-28 | 9e5c67e6 | [261128-0du-e-webview2-smartscreen-setting-per-d-041](./quick/261128-0du-e-webview2-smartscreen-setting-per-d-041/) |

## Deferred Items

Items acknowledged and deferred at milestone close, most recent first:

| Category | Item | Status | Deferred At | Milestone |
|----------|------|--------|-------------|-----------|
| *(none)* | | | | |

## Session Continuity

Last session: 2026-11-28T16:48:02.766Z
Stopped at: Completed 04-44-PLAN.md
Resume file: None
