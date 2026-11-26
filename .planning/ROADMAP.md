# Roadmap: Sucrose (Rust rewrite)

## Overview

Sucrose is rewritten from scratch in Rust on `develop-rust`, Windows first, then macOS and Linux (D-002). The phases follow the maintainer's milestones M0–M7 (formerly kept in `docs/ROADMAP.md`), one phase per milestone: a green cross-platform foundation, one video wallpaper on the Windows desktop, the service and the Portal connected over IPC, every engine with the compatible web wallpaper API, the remaining product features with an own installer and updater, then macOS, Linux and the parity sign-off. Each phase is a usable vertical slice; existing users keep their themes, store, web wallpapers and settings, and the new app uses less memory and CPU than the C# one.

## Phases

**Phase Numbering:**
- Integer phases (1–8): Phase N is milestone M(N-1)
- No decimal phases are planned

- [x] **Phase 1: M0 — Foundation** - CI is green on Windows, macOS and Linux with stub platform implementations (completed 2026-11-22)
- [x] **Phase 2: M1 — Video wallpaper on the desktop (Windows)** - The selected video theme plays behind the desktop icons on every monitor on Windows through libmpv (completed 2026-11-23)
- [x] **Phase 3: M2 — Service, IPC and Portal connected** - The Portal shows the library and applies wallpapers through a single-instance service; settings migrate and persist (completed 2026-11-25)
- [ ] **Phase 4: M3 — Engines and the wallpaper API** - Every wallpaper type and engine on Windows, the compatible web wallpaper API, display modes, property editing and all performance rules
- [ ] **Phase 5: M4 — Product features** - Store, theme authoring, cycling, lock screen, 25 languages, integrations and an own signed installer and updater
- [ ] **Phase 6: M5 — macOS** - Every feature macOS allows; the rest reported through platform capabilities, never faked
- [ ] **Phase 7: M6 — Linux (X11, then Wayland compositors)** - Every feature Linux allows, X11 first and then Wayland compositors, under the same rule
- [ ] **Phase 8: M7 — Parity sign-off** - Every FEATURES.md row ported or covered by a decision, lower RAM/CPU than the C# app, C# code removed

## Phase Details

### Phase 1: M0 — Foundation
**Goal**: The workspace builds, lints and tests in CI on Windows, macOS and Linux, so every later phase stays cross-platform from its first commit.
**Mode:** mvp
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02
**Success Criteria** (what must be TRUE):
  1. CI is green on all three OSes with stub platform implementations for macOS and Linux.
  2. A push to `develop-rust` runs `.github/workflows/rust.yml`, and fmt, clippy `-D warnings` and the tests pass on Windows, Ubuntu and macOS, and cargo-deny passes.
  3. `cargo xtask ci` on the maintainer's Windows machine runs the same checks as CI and passes.
  4. CI builds and tests natively on Windows ARM64 (`windows-11-arm`) and Linux ARM64 (`ubuntu-24.04-arm`), the Windows x64 job runs clippy and the tests for `i686-pc-windows-msvc`, and the macOS job checks the `x86_64-apple-darwin` target.
**Plans**: TBD
**Notes**: Closed 2026-11-22 without plans: every other M0 item was already done (PROJECT.md → Validated), and CI runs 44968506512 (FND-01) and 44978158638 (FND-02) are the evidence (`phases/01-m0-foundation/01-VERIFICATION.md`).

### Phase 2: M1 — Video wallpaper on the desktop (Windows)
**Goal**: An existing user's selected video theme plays behind the desktop icons on every monitor on Windows through libmpv, survives shell and monitor changes, and pauses for fullscreen apps.
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: TYPE-01, ENG-01, DESK-01, DESK-06, DESK-07, PERF-05, TRAY-01, FMT-02, FMT-08, MEAS-01, DESK-12
**Success Criteria** (what must be TRUE):
  1. The selected video plays behind the desktop icons on every monitor.
  2. It survives an explorer.exe restart and monitors being plugged/unplugged.
  3. While a fullscreen game runs it applies the configured action (Pause: pauses and resumes).
  4. CPU and RAM are measured against the C# MpvPlayer engine + Backgroundog and recorded in `docs/MEASUREMENTS.md`.
  5. On a machine with the C# app installed, Sucrose plays the theme selected there (read from the legacy `Library.json` and `General.json`, which stay unchanged), sharp at each monitor's DPI, and the tray icon's Quit stops it.
**Plans**: 29/29 plans complete (02-16 to 02-26: gap closure from UAT; 02-27 to 02-29: gap closure from verification)
Plans:
**Wave 1**
- [x] 02-01-PLAN.md — libmpv delivered: `cargo xtask fetch-engines` (pinned, SHA256-verified), committed bindgen bindings, D-18 loader; apps/sucrose tracked by git

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 02-02-PLAN.md — D-028 recorded across the docs after the maintainer decides the libmpv pin source and signs off the GPL build

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 02-03-PLAN.md — tracer: the service names the theme selected in the C# install (`sucrose-config` legacy reader core, read-only)

**Wave 4** *(blocked on Wave 3 completion)*
- [x] 02-04-PLAN.md — all D-09 legacy settings with legacy defaults and clamps, fixtures from the real files

**Wave 5** *(blocked on Wave 4 completion)*
- [x] 02-05-PLAN.md — `sucrose-ipc`: single instance and the service ↔ renderer channel, per-user pipe ACL

**Wave 6** *(blocked on Wave 5 completion)*
- [x] 02-06-PLAN.md — tracer: the renderer surface sits behind the icons on the primary monitor (24H2-aware attach, PMv2, layout classifier)

**Wave 7** *(blocked on Wave 6 completion)*
- [x] 02-07-PLAN.md — tracer: the selected video plays on that surface (bundled libmpv via wid, hardening, D-20 exit path)

**Wave 8** *(blocked on Wave 7 completion)*
- [x] 02-08-PLAN.md — SucroseInfo.json round-trip, D-13 gate, date-based version scheme (maintainer decision, D-029)

**Wave 9** *(blocked on Wave 8 completion)*
- [x] 02-09-PLAN.md — tray icon with Quit, failure reasons in the tooltip, AppVisible honored

**Wave 10** *(blocked on Wave 9 completion)*
- [x] 02-10-PLAN.md — renderer supervision: job object, bounded crash restarts (D-22)

**Wave 11** *(blocked on Wave 10 completion)*
- [x] 02-11-PLAN.md — Engine.json settings to mpv (volume, loop, stretch, hwdec, screen area), user mpv config, pause/volume controls

**Wave 12** *(blocked on Wave 11 completion)*
- [x] 02-12-PLAN.md — one renderer per monitor; hot-plug/DPI diffs; explorer.exe restart recovery via TaskbarCreated

**Wave 13** *(blocked on Wave 12 completion)*
- [x] 02-13-PLAN.md — fullscreen rule with the configured action (Pause/Close/Resume), both scopes

**Wave 14** *(blocked on Wave 13 completion)*
- [x] 02-14-PLAN.md — cross-monitor sync: master clock, follower drift correction, audio only on the master

**Wave 15** *(blocked on Wave 14 completion)*
- [x] 02-15-PLAN.md — `cargo xtask measure`, MEAS-01 rows vs the C# app, M1 docs close-out, verification scope

**Gap closure from UAT** *(gap_closure plans; 02-16 to 02-22 in their own waves 1–7, 02-23 to 02-26 in waves 1–4, and the plan for the behavior chosen at the 02-26 checkpoint is written after that decision; run with `/gsd-execute-phase 2 --gaps-only`)*
- [x] 02-16-PLAN.md — gap wave 1: sent display and shell messages reach the service at once; WorkerW-aware attach and an order guard after explorer.exe restarts (G-02-2d, G-02-2a)
- [x] 02-17-PLAN.md — gap wave 2: GPU preference, platform part (DESK-12): GpuPreference setting, UserGpuPreferences write with the legacy merge rules, DXGI adapter list, gpu_preference capability (G-02-2b)
- [x] 02-18-PLAN.md — gap wave 3: GPU preference wired (DESK-12): Engine.json GraphicPreference applied to sucrose.exe before the renderers start, adapter logged, docs (G-02-2b)
- [x] 02-19-PLAN.md — gap wave 4: renderers count once their first frame shows; master policy chosen by the maintainer and recorded in DECISIONS.md (G-02-2c)
- [x] 02-20-PLAN.md — gap wave 5: follower controller converges to the same frame, drift summary (G-02-2c)
- [x] 02-21-PLAN.md — gap wave 6: measure records each process's GPU adapter and refuses cross-GPU rows; C# re-measured (G-02-6)
- [x] 02-22-PLAN.md — gap wave 7: Rust re-measured on the same GPU, MEAS-01 rows replaced, mpv options kept unless measured otherwise (DECISIONS.md entry if changed) (G-02-6)
- [x] 02-23-PLAN.md — gap round 2, wave 1: every surface placement (move and attach) is checked against its target and the shell's desktop window and logged with a verdict (G-02-7)
- [x] 02-24-PLAN.md — gap round 2, wave 2: the surface follows the shell's desktop window when explorer moves it (WinEvent location watch, no polling); FEATURES note (G-02-7)
- [x] 02-25-PLAN.md — gap round 2, wave 3: every surface is re-placed after every applied display change, as the legacy app does (G-02-7)
- [x] 02-26-PLAN.md — gap round 2, wave 4: maintainer runs the instrumented build through UAT 7(c) and an origin-only change; decision on what a monitor shows while explorer leaves it outside its desktop window (G-02-7 stays open until the plan written after this decision)
**Gap closure from verification** *(gap_closure plans 02-27 to 02-29 in waves 1–2; 02-29 is maintainer-run; run with `/gsd-execute-phase 2 --gaps-only`)*
- [x] 02-27-PLAN.md — verification round, wave 1: a late report of a removed monitor's renderer or of a renderer the fullscreen rule closed no longer changes the tray status (WR-19); FEATURES Video row and service doc comments follow (IN-25, IN-28)
- [x] 02-28-PLAN.md — verification round, wave 2: the maintainer's 'wait' on G-02-7 recorded as an accepted limitation in docs/FEATURES.md and docs/DECISIONS.md D-031 (WR-20); G-02-7 stays open
- [x] 02-29-PLAN.md — verification round, wave 2: RAM/CPU of the final M1 build (checked placement, location watch, re-place-all) measured on a release build and recorded in docs/MEASUREMENTS.md (MEAS-01)
**Notes**: Criteria 1–4 are the "Done when" list of M1 from the former `docs/ROADMAP.md`. The desktop surface must handle the Windows 11 24H2 Progman/WorkerW hierarchy. libmpv ships inside the Sucrose package — `cargo xtask fetch-engines` places the pinned, hash-verified build next to the exe — and is loaded at runtime (D-028, D-013); detection with hints for the WebView2 Runtime and VC++ Redist is ENG-06 in Phase 4. `SucroseInfo.json` fixtures come from real themes with personal data removed. Measure in a release build. DESK-12 (GPU preference) was pulled in from Phase 5 by the UAT decision (test 6): on hybrid-GPU laptops the video engine needs it.

### Phase 3: M2 — Service, IPC and Portal connected
**Goal**: The service and the Portal work as one app: the user picks a theme in the Portal's library, "Use" applies it through the service, settings migrate from the C# install and persist, and Sucrose can start with the OS.
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: ARCH-01, ARCH-02, ARCH-03, ARCH-04, ARCH-05, FMT-01, FMT-06, TRAY-03, TRAY-05, LIB-01, LIB-02, LIB-03, LIB-04, LIB-06, LIB-08, LIB-09, UI-01, UI-02, UI-03, UI-05, UI-07
**Success Criteria** (what must be TRUE):
  1. On first run the Portal shows the library with the showcase themes copied in and corrupt or incompatible themes handled as in the legacy app; the user can sort, search and page through it, see hover previews and move it to another folder, and "Use" on a video theme makes it play on the desktop through the service.
  2. With an existing C# install, every legacy `Setting/*.json` value except `Donate.json` and the documented exclusions (runtime state, write-only system information, the removed GitHub token and store server; listed in `docs/FEATURES.md`) is in the new settings after first run, the data folder keeps its legacy layout, a setting changed in the Portal survives a restart, and killing Sucrose during a write never leaves a corrupt settings file. Legacy `Setting/*.json` files are only read during migration; they are never modified or deleted, so the C# app keeps working side by side.
  3. Starting the service or the Portal a second time reuses the running instance, Exit closes every Sucrose process, and a renderer that keeps crashing is restarted a bounded number of times and then stopped, never in a loop.
  4. An optional module the user turns off starts no thread and does no polling, and with "start with the OS" on, Sucrose starts after sign-in and resumes the wallpaper.
  5. The Portal follows light, dark or the OS theme with its Mica/Acrylic backdrop and optional background image, has header actions, search, a status bar and Help/About, and scrolling through a large library keeps its memory bounded (release build, recorded in `docs/MEASUREMENTS.md`).
**Plans**: 50/50 plans complete
Plans:
**Wave 1**
- [x] 03-01-PLAN.md — Settings tracer: Setting/Sucrose.json migrated once, atomic store with .bak and kill test, service reads it (D-032)
- [x] 03-02-PLAN.md — Library scan core: legacy validity rules, path containment, case-insensitive Type, sort, search, URL check
- [x] 03-03-PLAN.md — Localization: cargo xtask localize, legacy-keyed English strings, new keys, rust-i18n in the Portal, deny.toml

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 03-04-PLAN.md — General, Portal, Library and Warehouse keys typed and migrated, D-21 theme/backdrop split, LIB-01 and FEATURES wording
- [x] 03-05-PLAN.md — cargo xtask synth-library: synthetic libraries with prepared settings for Portal runs and D-25

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 03-06-PLAN.md — Portal protocol on the one endpoint: PortalHello routing, registry, AlreadyOpen, endpoint per data folder, test harness (D-033)
- [x] 03-26-PLAN.md — Engine, Backgroundog, Cycling, Hook and Update keys typed and migrated, D-05 exclusions and stale names, ROADMAP/FMT-01/FEATURES wording (split from 03-04)

**Wave 4** *(blocked on Wave 3 completion)*
- [x] 03-07-PLAN.md — Settings host: validated, atomic, broadcast changes that persist across restarts

**Wave 5** *(blocked on Wave 4 completion)*
- [x] 03-08-PLAN.md — Portal connects: starts the service with --no-portal, banners, reconnect, single instance, strict arguments

**Wave 6** *(blocked on Wave 5 completion)*
- [x] 03-09-PLAN.md — Use through the service: LoadSource into running renderers, refusals, budget reset, runtime state
- [x] 03-10-PLAN.md — Library grid with card-size thumbnails and the bounded, released thumbnail cache

**Wave 7** *(blocked on Wave 6 completion)*
- [x] 03-11-PLAN.md — Checkpoint: maintainer verifies the no-black-frame Use transition (D-26)

**Wave 8** *(blocked on Wave 7 completion)*
- [x] 03-12-PLAN.md — Use from a card: spinner, Selected pill, disabled reasons, keyboard, toasts
- [x] 03-13-PLAN.md — Optional modules start nothing when off; live tray toggle; memory report while the Portal is connected

**Wave 9** *(blocked on Wave 8 completion)*
- [x] 03-14-PLAN.md — Header search, pagination with the legacy scheme, sort and layout live, empty and error states

**Wave 10** *(blocked on Wave 9 completion)*
- [x] 03-15-PLAN.md — Lifecycle: second launch opens or focuses the Portal, legacy Launcher start rule, D-13 exit, foreground handoff

**Wave 11** *(blocked on Wave 10 completion)*
- [x] 03-16-PLAN.md — Tray menu Open Sucrose and Close Applications/Exit; Close Applications ends every Sucrose process

**Wave 12** *(blocked on Wave 11 completion)*
- [x] 03-17-PLAN.md — Theme mode with toggle and OS following, backdrop by capability, background image layer

**Wave 13** *(blocked on Wave 12 completion)*
- [x] 03-18-PLAN.md — Status bar with memory total and quotes, Premium, Help and About, tray hint dialog

**Wave 14** *(blocked on Wave 13 completion)*
- [x] 03-19-PLAN.md — Hover previews: capped card-size GIF, one at a time, released on leave

**Wave 15** *(blocked on Wave 14 completion)*
- [x] 03-20-PLAN.md — Settings mode: General and Personal pages with the Phase 3 keys

**Wave 16** *(blocked on Wave 15 completion)*
- [x] 03-21-PLAN.md — Showcase copy tracked in the new settings; corrupt themes to the recycle bin

**Wave 17** *(blocked on Wave 16 completion)*
- [x] 03-22-PLAN.md — Library move: verified rename or copy, rollback, nested refusal, wallpaper restart

**Wave 18** *(blocked on Wave 17 completion)*
- [x] 03-23-PLAN.md — Start with the OS: own Run value, LaunchAgent, XDG autostart; legacy entry detection (D-034)

**Wave 19** *(blocked on Wave 18 completion)*
- [x] 03-24-PLAN.md — Checkpoint: Portal memory sampled while the maintainer pages the real and the 500-theme library (D-25)

**Wave 20** *(blocked on Wave 19 completion)*
- [x] 03-25-PLAN.md — UI-07 judged and recorded in MEASUREMENTS.md; ARCHITECTURE, FEATURES and GPUI updated for M2

**Cross-cutting constraints:** <!-- gsd:generated -->
- The clock starts only after the maintainer replied 'start' to the ready checkpoint; the continuation starts the Portal and the sampler about 30 s later at most and returns the exact local start time at once, so no time runs while a checkpoint is in transit
- Rounds, first page and final value come from the Portal's own page log (03-36 --portal-log), not from fixed windows; the maintainer pages at their own pace and ends the run by closing the Portal after at least 30 s idle on page 1
- Hover previews animate during the run (the Portal log has 'preview playing' lines; the maintainer confirms)
- The 'page shown' lines carry 03-40's heap_allocated_mb, heap_committed_mb, gpu_local_mb and gpu_non_local_mb, and the summary's attribution line (growth of video memory, held heap and allocator retention against the private growth, with its verdict) goes into the SUMMARY with the numbers, so 03-39 can confirm the atlas cause or pick a named follow-up
- The run uses a scratch data folder; the real settings and library are only read; the preview switch is edited only in the scratch Sucrose.json
- Afterwards no sucrose.exe or sucrose-portal.exe is left running (taskkill, confirmed with tasklist)
- The result is judged by the D-25 rule as recorded for rows 16-17 of docs/MEASUREMENTS.md (final at most first page + 48 MiB and round 3 end at most round 1 end x 1.05, on working set and private bytes) and recorded there as a new row with a reading paragraph, pass or FAIL; no cap or code is changed after the run
- The maintainer reports whether thumbnails appeared without a visible stall while paging and scrolling, and whether the previews animated ('preview playing' count in the SUMMARY and the row)

**Gap closure (2026-11-24, from 03-VERIFICATION.md gaps and 03-REVIEW.md CR-01, CR-02, WR-01..WR-08)**

**Gap wave 1**
- [x] 03-27-PLAN.md — Images released inside a window update leave GPUI's atlas (paging-memory leading cause, atlas::release_image, GPUI tests)
- [x] 03-30-PLAN.md — CR-02: an unreadable Sucrose.json is retried and never restored or migrated over
- [x] 03-31-PLAN.md — WR-01, WR-03, WR-07: library move exclusive with maintenance, quiet fleet, printable status text
- [x] 03-34-PLAN.md — WR-08: the new source starts only after its own file loaded
- [x] 03-35-PLAN.md — WR-05: turning startup off never leaves setting and Run value disagreeing

**Gap wave 2** *(blocked on gap wave 1)*
- [x] 03-28-PLAN.md — Hover previews animate (card element id, frame test, diagnostics, maintainer hover run)
- [x] 03-29-PLAN.md — CR-01: one pixel-area limit for thumbnails, background image and preview GIF
- [x] 03-32-PLAN.md — WR-02: no Portal request acts while quitting; exit waits for a library move
- [x] 03-36-PLAN.md — D-25 rounds from the Portal's page log; WR-06 canonical synth-library guard

**Gap wave 3** *(blocked on gap wave 2)*
- [x] 03-33-PLAN.md — WR-04: a Portal dropped for a full queue is disconnected; every Portal end re-checks the exit rule

**Gap wave 4** *(blocked on gap wave 3)*
- [x] 03-40-PLAN.md — SC5 growth-source attribution: heap and video memory on the page log, attribution verdict in the D-25 summary

**Gap wave 5** *(blocked on gap wave 4)*
- [x] 03-37-PLAN.md — Checkpoint: D-25 scenario 1 rerun on the real library (ready gate, page-log rounds, attribution)

**Gap wave 6** *(blocked on gap wave 5)*
- [x] 03-38-PLAN.md — Checkpoint: D-25 scenario 2 rerun on the 500-theme library (attribution)

**Gap wave 7** *(blocked on gap wave 6)*
- [x] 03-39-PLAN.md — Record and judge the reruns and their attribution (cause confirmed or named follow-up), close the review ledger, mark LIB-06 and UI-07 on evidence

**Gap closure round 2 (2026-11-25, from 03-UAT.md gaps G-03-5, G-03-6, G-03-8 and the blocked test 10)**

**Gap round 2, wave 1**
- [x] 03-41-PLAN.md — G-03-5: clear gpui-component's opaque base Root fill while a backdrop is active, deferred until the Root exists; real-screen None/Acrylic check; GPUI.md lesson

**Gap round 2, wave 2** *(blocked on wave 1)*
- [x] 03-42-PLAN.md — G-03-6, G-03-8: thumbnails for the rows in view plus one row ahead (no first-64 cap), wanted thumbnails kept in the bounded cache, grid scrollbar, static placeholder for cards that are not loading

**Gap round 2, wave 3** *(blocked on wave 2)*
- [x] 03-43-PLAN.md — Checkpoint: the maintainer checks the backdrop (Acrylic, Mica, Tabbed, None; Light and Dark), the 100-per-page library while scrolling, and hover previews with Library Preview on

**Gap round 2, wave 4** *(blocked on wave 3)*
- [x] 03-44-PLAN.md — Checkpoint: D-25 scenario 2 rerun on the in-view thumbnail path (UAT test 10, ready gate, page-log rounds), recorded in MEASUREMENTS.md

**Gap closure round 3 (2026-11-25, from 03-VERIFICATION.md gaps "SC5 backdrop" and "grid scrollbar", the behavior-unverified SC5 memory item, and 03-REVIEW.md WR-14, WR-15, WR-16)**

**Gap round 3, wave 1**
- [x] 03-45-PLAN.md — WR-14: Opaque before a changed backdrop, DWM system backdrop type set and reset through sucrose-platform (Platform::set_window_backdrop) with the HRESULT logged as dwm_result

**Gap round 3, wave 2** *(blocked on wave 1)*
- [x] 03-46-PLAN.md — WR-15: grid scrollbar mounted beside the scroll area (GPUI layout tests), docs say it is unconfirmed until 03-47; WR-16: rows in view from an offset clamped to the content, settled on the next frame after a shrink or resize (layout and GPUI tests), before the D-25 rerun

**Gap round 3, wave 3** *(blocked on wave 2)*
- [x] 03-47-PLAN.md — Checkpoint: the maintainer judges Mica, Tabbed, Acrylic and None (both directions, Dark and Light, active window), the grid scrollbar with "Always show scrollbars" off and on, and the thumbnails after a resize (WR-16); frame-extension verdict by rule with a next step for every verdict (backdrop_next: closed, 03-48 or replan); docs corrected

**Gap round 3, wave 4** *(blocked on wave 3)*
- [x] 03-48-PLAN.md — Only when 03-47 records frame_extension: needed: extend the DWM frame into the client area for Mica and Tabbed

**Gap round 3, wave 5** *(blocked on wave 4)*
- [x] 03-49-PLAN.md — Checkpoint, only after a frame extension: the maintainer judges the four backdrops again

**Gap round 3, wave 6** *(blocked on wave 5)*
- [x] 03-50-PLAN.md — Checkpoint: D-25 scenario 2 rerun on the final thumbnail and scrollbar path (UAT test 10, ready gate, page-log rounds), recorded in MEASUREMENTS.md
**UI hint**: yes
**Notes**: `apps/sucrose-portal` is built from scratch; `exp/GPUI/Sucrose.Portal` is not its base. Runtime state (pause/close flags, ports) travels over IPC and is never stored in settings. Only Video themes can play until Phase 4 adds the other engines. The bounded thumbnail cache built here (UI-07) is reused by the store in Phase 5. Normal startup only; the priority and scheduled-task modes are TRAY-04 in Phase 5 (the M2 checklist said "Task Scheduler option later").

### Phase 4: M3 — Engines and the wallpaper API
**Goal**: Every wallpaper type plays on Windows on every engine that supports it, existing web wallpapers work unchanged with the C#-compatible JavaScript API, and display modes, property editing and the performance rules match or improve on the C# app.
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: TYPE-02, TYPE-03, TYPE-04, TYPE-05, TYPE-06, ENG-02, ENG-03, ENG-04, ENG-05, ENG-06, ENG-07, DESK-02, DESK-03, DESK-04, DESK-05, PLAY-01, PLAY-02, PLAY-03, PLAY-04, PLAY-05, PERF-01, PERF-02, PERF-03, PERF-04, PERF-06, PERF-07, PERF-08, PERF-09, WAPI-01, WAPI-02, WAPI-03, WAPI-04, WAPI-05, WAPI-06, WAPI-07, WAPI-08, WAPI-09, WAPI-10, PROP-01, PROP-02, PROP-03, PROP-04, PROP-05, FMT-03, FMT-07, SEC-02
**Success Criteria** (what must be TRUE):
  1. User can set GIF (also APNG and WebP), Web, URL, YouTube and Application themes besides Video; with no engine installed, GIF and Video still play through the built-in renderers; installed WebView2, CEF, libmpv and libVLC are detected, missing ones show install hints, and the user picks the engine per wallpaper type with a reported fallback.
  2. Existing web wallpapers and the fixture pages work unchanged on WebView2 and CEF: they receive property calls, `SucroseCompatible.json` templates, system, BIOS and motherboard data, the audio spectrum, now-playing data and forwarded mouse and keyboard input with the legacy payloads, and a page cannot read files outside its theme folder.
  3. User can show a wallpaper per display, spanned or duplicated, put a different wallpaper on each monitor, and choose the working area or the full display bounds and a stretch mode, with every engine.
  4. Changing a value in the properties editor (all ten control types, texts in the user's language) changes the running wallpaper immediately; the user can restore defaults, turn on developer mode with a remote debugging port, and add, remove or reset web engine arguments.
  5. Each performance condition (lock, sleep, user switch, screen saver, focus, fullscreen, CPU/memory/GPU, network, battery, remote desktop/VM) runs its configured Close, Pause or Resume with the legacy timings; pausing goes through the engine and never suspends processes; volume, loop, shuffle and the two sound rules apply to every engine.
**Plans**: 3/21 plans executed (Round A = D-31 slice 1; Rounds B–E are planned after Round A has executed, 04-CONTEXT D-31 "Planning rounds")
Plans:
**Wave 1**
- [x] 04-01-PLAN.md — D-32 CEF delay-load spike, part 1: pinned CEF via fetch-engines and CEF_PATH, delay-loaded libcef.dll, CEF subprocess routing, fixture Web library
- [x] 04-03-PLAN.md — SucroseProperties.json and SucroseCompatible.json models with real and showcase fixtures (FMT-03)

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 04-02-PLAN.md — D-32 gate verdict: a Web theme plays in a windowed CEF browser on the desktop surface; D-036 and the rule exceptions

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 04-04-PLAN.md — sucrose-engine: engine identities (D-10), availability, choose_engine with the D-11 fallback order, shared clock follower

**Wave 4** *(blocked on Wave 3 completion)*
- [ ] 04-05-PLAN.md — Engine trait: mpv and CEF behind one renderer host, Assignment names the engine

**Wave 5** *(blocked on Wave 4 completion)*
- [ ] 04-06-PLAN.md — Virtual origin https://sucrose.wallpaper.engine/ with one tested resolver, Range, traversal refused (D-14, SEC-02)

**Wave 6** *(blocked on Wave 5 completion)*
- [ ] 04-07-PLAN.md — Web profiles in <data_dir>/Web/Cef (D-33, D-35), TLS validation by default (D-12), D-037 and D-038

**Wave 7** *(blocked on Wave 6 completion)*
- [ ] 04-08-PLAN.md — Chromium switches (CefArguments model, Sucrose switches, merge) and developer mode/port/OpenDevTools on CEF (D-17, D-13)

**Wave 8** *(blocked on Wave 7 completion)*
- [ ] 04-09-PLAN.md — JS API on CEF: PropertyListener calls, filedropdown, non-data compatible keys, Light pause inside the page

**Wave 9** *(blocked on Wave 8 completion)*
- [ ] 04-10-PLAN.md — On-demand data hub and sucrose-sysinfo (Date), D-24 decoupling, D-039 (D-34)

**Wave 10** *(blocked on Wave 9 completion)*
- [ ] 04-11-PLAN.md — Memory, Processor, Bios and Motherboard data groups with legacy payloads

**Wave 11** *(blocked on Wave 10 completion)*
- [ ] 04-12-PLAN.md — Engine detection (files, VC++ Redist), run-only fallback with log, tray and Portal report (ENG-06, ENG-07)

**Wave 12** *(blocked on Wave 11 completion)*
- [ ] 04-13-PLAN.md — Portal: Web cards usable from the service report; Wallpaper page with engine choice

**Wave 13** *(blocked on Wave 12 completion)*
- [ ] 04-14-PLAN.md — Portal: Wallpaper group rows (interaction, loop, shuffle, hardware acceleration), live loop/shuffle

**Wave 14** *(blocked on Wave 13 completion)*
- [ ] 04-15-PLAN.md — Portal: fallback toast with the one-day snooze, row alert, CEF status line, warning colours (D-11, D-16)

**Wave 15** *(blocked on Wave 14 completion)*
- [ ] 04-16-PLAN.md — Portal: certificate switch, CEF arguments editor with service-owned rules, shared Restart prompt (D-12, D-17)

**Wave 16** *(blocked on Wave 15 completion)*
- [ ] 04-17-PLAN.md — Portal: Other page Developer group, Open DevTools, developer switch and port (D-13)

**Wave 17** *(blocked on Wave 16 completion)*
- [ ] 04-18-PLAN.md — Verified prerequisite install: Authenticode, install worker, install dialog (D-15)

**Wave 18** *(blocked on Wave 17 completion)*
- [ ] 04-19-PLAN.md — Input forwarding to CEF: raw input, legacy messages, cursor and InputType/InputDesktop filters (WAPI-07)

**Wave 19** *(blocked on Wave 18 completion)*
- [ ] 04-20-PLAN.md — Maintainer checkpoint: 8 Web themes and fixture pages on a release build; marks the Round A completions

**Wave 20** *(blocked on Wave 19 completion)*
- [ ] 04-21-PLAN.md — Maintainer measurement: CEF against C# CefSharp RAM/CPU into MEASUREMENTS.md

**UI hint**: yes
**Notes**: Engine binaries ship inside the package (D-028); own `bindgen` bindings loaded with `libloading` (D-013). Legacy bugs are fixed, not ported: CEF path traversal (SEC-02), ping checked first even at threshold 0 (PERF-07), Application wallpapers with fake handles or one process for all screens (TYPE-06), debugger-attach and `NtSuspendProcess` pausing (PLAY-05). Record per-engine RAM/CPU from release builds in `docs/MEASUREMENTS.md`.

### Phase 5: M4 — Product features
**Goal**: A C# user can switch to the Rust app for daily use: store, theme authoring, cycling, lock screen, 25 languages, integrations and an own signed installer and updater that migrates the C# install.
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: STOR-01, STOR-02, STOR-03, STOR-04, STOR-05, THEM-01, THEM-02, THEM-03, THEM-04, THEM-05, LIB-05, LIB-07, SVC-01, SVC-02, SVC-03, SVC-04, SVC-05, SVC-06, SVC-07, SVC-08, DESK-08, DESK-09, DESK-10, DESK-11, UI-04, UI-06, TRAY-02, TRAY-04, FMT-04, FMT-05, SEC-01
**Success Criteria** (what must be TRUE):
  1. User can browse store categories with the adult filter, search, page, preview and install store themes from the Soferity server, sees offline, broken and unknown states from the legacy-compatible cache, and can report a theme through the Soferity API.
  2. User can create themes of all six types, edit them with per-language title and description, review, delete, import a `.zip` validated with the legacy `CompatibilityType` codes, export and publish them, drop files, `.url` files, archives and URLs onto the library, and use every theme-card action.
  3. User can cycle wallpapers on an interval in random or sequential order with exclusions, set a Video or GIF theme as the live lock and sign-in screen wallpaper, hide the desktop icons, set the OS wallpaper from the theme thumbnail and keep the computer awake.
  4. The Portal is available in all 25 languages with RTL for Arabic and Hebrew and a live switch, its settings pages cover every key in `portal.md` §4, the full tray menu works with a grayscale icon while paused, Discord Rich Presence shows the wallpaper, and the user can bundle logs, back up, restore or reset settings and clear caches.
  5. A signed installer installs over an existing C# install and migrates it, updates are verified before they install, every connection validates TLS, opting out of crash reports and telemetry means nothing is written or sent, the user can pick the priority or scheduled-task startup mode, and a quiet uninstall removes every startup entry and GPU-preference value.
**Plans**: TBD
**UI hint**: yes
**Notes**: DESK-10 (lock screen) needs a `docs/DECISIONS.md` entry on the mechanism before it is built (D-026; reference: `docs/legacy/lock-screen.md`, branch `lock-screen-native` 112de37f). STOR-01 and FMT-05 use the existing v8 endpoints until the API rewrite (D-022, deferred). The update signing key scheme (Ed25519 proposed, D-019) is settled before the updater ships. signtool reads the certificate location from environment variables (D-025). Legacy defaults stay (`StoreAdult`, `TelemetryData`, `ExceptionData` = true, D-024).

### Phase 6: M5 — macOS
**Goal**: macOS users get every feature macOS allows, and the Portal shows clearly what macOS cannot do instead of faking it.
**Mode:** mvp
**Depends on**: Phase 5
**Requirements**: MAC-01, MAC-02, MAC-03, MAC-04, MAC-05, MAC-06
**Success Criteria** (what must be TRUE):
  1. On macOS, Video, GIF, Web, URL and YouTube themes play at desktop level on every monitor, sharp on Retina displays, and the performance rules react to fullscreen apps and power events.
  2. WKWebView, CEF, libmpv, libVLC and the built-in AVFoundation engine are detected and play wallpapers; web wallpapers receive system data, the audio spectrum once the user grants permission, and the media session data macOS allows.
  3. The tray menu works, Sucrose starts at login through a LaunchAgent, and its data lives under the macOS config directory.
  4. An unsigned package installs Sucrose and later updates it.
  5. Every `?` in the macOS column of `docs/FEATURES.md` (including the lock screen) is investigated and set, and features macOS cannot do, such as Application wallpapers, are hidden or explained in the Portal through platform capabilities.
**Plans**: TBD
**UI hint**: yes
**Notes**: No Apple Developer ID exists yet, so packages stay unsigned until one does (D-025). macOS builds run in CI; manual checks need a real Mac.

### Phase 7: M6 — Linux (X11, then Wayland compositors)
**Goal**: Linux users get every feature their session allows, X11 first and then Wayland compositors, with what a session cannot do reported through platform capabilities, never faked.
**Mode:** mvp
**Depends on**: Phase 6
**Requirements**: LNX-01, LNX-02, LNX-03, LNX-04, LNX-05, LNX-06, LNX-07
**Success Criteria** (what must be TRUE):
  1. On X11, themes (including Application wallpapers) play on a desktop-type surface on every monitor at the right DPI, and the performance rules react to fullscreen apps and power events.
  2. On wlr-layer-shell compositors, and on KDE and GNOME through a plugin or extension where needed, wallpapers play on a background surface, and features a compositor cannot support (such as input forwarding or Application wallpapers) are reported per compositor.
  3. WebKitGTK, CEF, libmpv, libVLC and the built-in GStreamer engine play wallpapers; web wallpapers receive system data, the PipeWire/PulseAudio audio spectrum and MPRIS now-playing data.
  4. The tray works (AppIndicator on GNOME), XDG autostart starts Sucrose at login, data lives under the platform config directory, and the chosen package formats install and update Sucrose.
  5. Every `?` in the Linux column of `docs/FEATURES.md` (including the lock screen) is investigated and set.
**Plans**: TBD
**UI hint**: yes
**Notes**: Package formats are decided at the start of this phase and recorded in `docs/DECISIONS.md`. Linux builds run in Docker (official `rust` image) or CI; no Linux Rust toolchain is installed in WSL.

### Phase 8: M7 — Parity sign-off
**Goal**: Parity is proven and signed off: every legacy feature is ported or covered by an accepted decision, the Rust app uses less RAM and CPU than the C# one, and the C# code is gone.
**Mode:** mvp
**Depends on**: Phase 7
**Requirements**: SIGN-01, SIGN-02, MEAS-02
**Success Criteria** (what must be TRUE):
  1. Every row in `docs/FEATURES.md` is ported (✅) or covered by an accepted decision (➖), with correct Windows, macOS and Linux cells, and none of the "Legacy problems not to carry over" remains.
  2. In every scenario recorded in `docs/MEASUREMENTS.md` the release build uses less RAM and CPU than the C# app, and the always-on service is close to idle cost.
  3. `src/` is removed, and `CLAUDE.md` and `docs/` describe the finished project.
**Plans**: TBD
**Notes**: Removing `src/` ends its read-only reference role (D-016); do it only after SIGN-01 and MEAS-02 hold.

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. M0 — Foundation | 0/0 | Complete    | 2026-11-22 |
| 2. M1 — Video wallpaper on the desktop (Windows) | 29/29 | Complete    | 2026-11-23 |
| 3. M2 — Service, IPC and Portal connected | 50/50 | Complete    | 2026-11-25 |
| 4. M3 — Engines and the wallpaper API | 3/21 | In Progress | - |
| 5. M4 — Product features | 0/TBD | Not started | - |
| 6. M5 — macOS | 0/TBD | Not started | - |
| 7. M6 — Linux (X11, then Wayland compositors) | 0/TBD | Not started | - |
| 8. M7 — Parity sign-off | 0/TBD | Not started | - |

**Coverage:** 127/127 v1 requirements mapped; traceability in `.planning/REQUIREMENTS.md`.
