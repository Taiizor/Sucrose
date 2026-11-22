# Phase 2: M1 — Video wallpaper on the desktop (Windows) - Research

**Researched:** 2026-11-22
**Domain:** Rust Win32 desktop-surface embedding (Progman/WorkerW, Windows 11 24H2+ "raised desktop"), libmpv client API loaded at runtime, multi-process service/renderer with local-socket IPC, legacy settings/theme compatibility, release-build measurement
**Confidence:** MEDIUM overall. HIGH on libmpv delivery, mpv option validity, legacy formats, crate choices and the local desktop hierarchy; MEDIUM on the exact attach recipe for Windows 11 build 26300 (needs a spike); LOW on cross-process sync tolerances and some Win32 behaviors tagged [ASSUMED].

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Monitors and audio
- **D-01:** Every monitor gets its own renderer process (`sucrose --render`) that plays the same video on its own desktop surface. The legacy `DisplayScreenType` and `ScreenDevice` keys are not read in M1. The maintainer's own setting is `PerDisplay` on one display; honoring it would show the video on one monitor only, against success criterion 1. Display modes arrive with DESK-02 in Phase 4. Record this M1-only behavior in the `docs/FEATURES.md` notes.
- **D-02:** Renderers that show the same wallpaper stay in sync. One renderer is the master clock and the others align to it, so every monitor shows the same frame with no drift. Audio comes only from the master, at the legacy `WallpaperVolume`; the other renderers are muted, so there is no echo even with a few ms offset between processes. The maintainer's words: "if the same wallpaper runs on several screens, video and audio should be synced between them so there is no drift or echo."

#### Fullscreen
- **D-03:** The action comes from legacy `Backgroundog.json` `FullScreenPerformance` (`Close` / `Pause` / `Resume`). When the key is absent, the legacy default `Resume` applies, which disables the rule (D-024, PERF-05 "configured action"). Success criterion 3 is verified with the key set to `Pause`; the maintainer's file currently has no such key. Planning adjusts the wording of criterion 3 in `.planning/ROADMAP.md` to "applies the configured action (Pause: pauses and resumes)".
- **D-04:** "Fullscreen" means the legacy rule: the foreground window covers an entire monitor rectangle, and the check is skipped while the desktop is focused. This includes borderless games, browser F11 and fullscreen video players.
- **D-05:** The scope of the action is a **new setting**: apply Pause or Close to the affected monitor only, or to all monitors. M1 implements both scopes with the legacy behavior "all monitors" as the fixed default and no switch. The setting is persisted with the new settings format (Phase 3) and gets UI with PERF-01 (Phase 4). Add a "**New:**" row or note in `docs/FEATURES.md`. When only the affected monitor pauses, it re-aligns to the master on resume.
- **D-06:** `Close` stops the renderers and starts them again when the condition recovers. `Pause` pauses playback in the engine (mpv `pause`).

#### Legacy install and settings
- **D-07:** A running C# app is not detected ("ignore"). Manual tests run with the C# Launcher and its live engines closed; if both run, which wallpaper ends up on top is undefined.
- **D-08:** Legacy settings are always read, read-only, from the real `%APPDATA%\Sucrose\Setting` (and `%APPDATA%\Sucrose\Cache\MpvPlayer` for D-10), whatever `SUCROSE_DATA_DIR` says. `SUCROSE_DATA_DIR` redirects only what the Rust app writes (logs, generated mpv config, caches). There is no `--theme` flag and no separate legacy-dir variable: tests run on the maintainer's machine, where the C# app is installed, and parsing is covered by fixture tests under each crate's `tests/fixtures/`.
- **D-09:** Legacy keys read in M1 (format: `Properties` wrapper, enums by name; `docs/legacy/formats.md` §1.1, §1.3). A missing or invalid key falls back to the legacy default:
  - `Library.json`: `Location`, `Selected`
  - `General.json`: `AppVisible` (D-15)
  - `Engine.json`: `WallpaperVolume`, `WallpaperLoop`, `StretchType`, `HardwareAcceleration` (mpv `hwdec`), `ScreenType` (`DisplayBound` / `WorkingArea`)
  - `Backgroundog.json`: `FullScreenPerformance`, `PausePerformanceType`

  Not read: `DisplayScreenType`, `ScreenDevice` (D-01), `CrashExplorer` (D-11), and the `Video` engine choice (M1 always uses mpv; ENG-07 is Phase 4). Build the reader so the FMT-01 migration in Phase 3 can reuse it.
- **D-10:** If the user's `Cache\MpvPlayer\uMpvPlayer.config` exists, it is used instead of the generated mpv config, as in the legacy app.
- **D-11:** Recovery after an explorer.exe restart is always on and event-driven (`TaskbarCreated`), with no polling. The legacy `CrashExplorer` setting (default `false`) is ignored. Record this as a behavior change in `docs/FEATURES.md`.
- **D-12:** The Rust app continues the legacy date-based version scheme (YY.MM.DD…), so that the `AppVersion ≤ app version` theme check, store filtering and the Phase 5 update from the C# app keep working. The current `0.1.0` would reject every theme; the maintainer's selected theme has `AppVersion` 26.10.19.0. Planning picks the exact form, which must (a) be valid Cargo semver, (b) compare correctly with legacy four-part versions, and (c) be greater than the latest C# release. Add a new `docs/DECISIONS.md` entry. — **Reversibility:** one-way — published version numbers cannot go back down: the updater and the store filter compare against them.
- **D-13:** If the selected theme is missing, unreadable, not a Video theme, or needs a newer `AppVersion`, no wallpaper starts. The reason goes to the log and the tray tooltip, and the service waits in the tray until Quit (the Portal can change the theme from Phase 3 on).

#### Tray
- **D-14:** The tray menu has only **Quit** (TRAY-01). Quit stops the Rust service and its renderers and never touches C# processes; closing all Sucrose apps is TRAY-05 in Phase 3.
- **D-15:** `General.json` `AppVisible` is honored: when it is `false`, the tray module does not start (D-008). Consequence: in M1 the wallpaper then has no UI to stop it, only Task Manager. The maintainer's value is `true`.

#### Engine delivery (libmpv)
- **D-16:** New `docs/DECISIONS.md` entry **D-028** supersedes D-005. The maintainer clarified that the C# app bundled libmpv (`Requirements\libmpv-{x86|x64|ARM64}.dll`), libVLC (NuGet `VideoLAN.LibVLC.Windows`) and CEF (CefSharp NuGet, `libcef.dll`); only the WebView2 Runtime and the VC++ Redist were user-side downloads. In the Rust app, engine binaries ship inside the Sucrose package. `cargo xtask` downloads pinned versions, verifies them by SHA256 (the NuGet equivalent), puts them next to the built exe for development runs, and later adds them to packages. The WebView2 Runtime and VC++ Redist stay prerequisites that are detected and get download hints. M1 applies this to libmpv only; libVLC and CEF follow the same model in Phase 4; macOS and Linux packaging is decided in Phases 6–7.

  Docs to update in this phase, in the same change:
  - `docs/DECISIONS.md`: D-028; set D-005 to "Superseded by D-028"; fix the D-006 caveat wording if needed
  - `CLAUDE.md`: non-negotiable 3
  - `.planning/PROJECT.md`: Constraints → Engines, the Out of Scope bullet about bundling, Key Decisions
  - `.planning/REQUIREMENTS.md`: ENG-01 ("loaded at runtime when installed") and ENG-06 ("never bundles or installs them")
  - `.planning/ROADMAP.md`: Phase 2 notes
  - `docs/ARCHITECTURE.md`: Engines
  - `.claude/rules/platform-ffi.md` and `.claude/rules/cargo.md` where they say engines are user-installed
  - `docs/FEATURES.md`: Architectures notes

  — **Reversibility:** costly — packaging, xtask, several docs and the requirements wording depend on it.
- **D-17:** libmpv is still loaded at runtime with `libloading` through our own `bindgen` bindings (D-013 unchanged). Only renderer processes load it; the service never maps libmpv.
- **D-18:** Search order: next to `sucrose.exe` (the bundled DLL), then the Windows DLL search path (PATH). The C# install's `Requirements` folder is **not** searched.
- **D-19:** The libmpv binaries are pinned prebuilt Windows builds for x86, x64 and ARM64. Research picks the source (for example shinchiro/mpv-winbuild-cmake `libmpv-2.dll`) and the version. The license must be compatible with GPL-3.0 and is recorded in the docs. No engine binaries are committed to git.
- **D-20:** If libmpv is missing or cannot be loaded (wrong architecture, client API too old), no wallpaper starts. The reason and an install or repair hint go to the log and the tray tooltip, and the service waits in the tray (same as D-13). Nothing panics. The full detection UI is ENG-06 in Phase 4.

#### Process model
- **D-21:** The D-008 model applies from M1 on. The `sucrose` service spawns one `sucrose --render` per monitor and talks to the renderers through a new `sucrose-ipc` crate (`interprocess` local sockets + `postcard`, D-009): sync clock, pause/resume, close, mute/volume. Phase 3 adds the Portal, settings and the rest of the protocol on top, so no throwaway channel is built. Measurements therefore reflect the final architecture. — **Reversibility:** costly — every later phase builds on the service ↔ renderer protocol.
- **D-22:** When a renderer crashes, the service restarts it with backoff and a bound (for example at most 3 restarts in 60 s). Past the bound, that monitor stays empty and the reason goes to the log and the tray tooltip. This avoids the legacy restart loops.
- **D-23:** Single instance works in M1 through the same IPC endpoint (D-009): a second `sucrose` logs and exits immediately. Opening the Portal from a second instance comes in Phase 3.

#### Measurement (MEAS-01)
- **D-24:** The video is the selected theme's video. Its resolution, codec and bitrate are recorded in the `docs/MEASUREMENTS.md` row.
- **D-25:** The like-for-like comparison is on a single monitor: C# `PerDisplay` against Rust limited to one monitor. Rust on all monitors is an extra row.
- **D-26:** The tool is `cargo xtask measure`. It samples processes by name and reports per-process and total working set, private bytes, average and peak CPU, and GPU %, measured for 5 minutes after a 60 s warm-up, and prints a ready-to-paste `docs/MEASUREMENTS.md` row. It is reusable for MEAS-02. Measurements use release builds with the legacy `HardwareAcceleration` value.
- **D-27:** The C# side is `Sucrose.Launcher` + `Sucrose.Live.MpvPlayer` + `Sucrose.Backgroundog`, plus a subtotal of MpvPlayer + Backgroundog (the literal criterion 4). The Rust side is the service plus every renderer.

### Claude's Discretion
- How Rust is limited to one monitor for the D-25 measurement. The maintainer prefers no needless flags, so keep any switch scoped to measurement.
- Master selection: the primary monitor, with another renderer promoted when the master's monitor goes away. Sync mechanism and tolerance, with the target of no visible offset.
- Fullscreen detection: event-driven (WinEvent hooks) or 1 s polling, as long as the legacy semantics hold (5 polls to trigger, 3 to recover, desktop-focused skip).
- `PausePerformanceType = Heavy` is handled like `Light` (engine-level pause; PLAY-05 replaces `NtSuspendProcess`). Note this in `docs/FEATURES.md`.
- Attach technique for the Windows 11 24H2 Progman/WorkerW hierarchy, monitor hot-plug and DPI-change handling, and debounce timing (legacy: 2000 ms + 500 ms).
- Generated mpv config, based on the legacy `MpvPlayer.config` option set (`docs/legacy/engines.md` §9). Stretch mapping per `docs/legacy/engines.md` §3 (`panscan`, `keepaspect`, `video-unscaled`). Loop handling.
- Accepted libmpv DLL names and the minimum mpv client API version.
- Repainting the desktop on stop (legacy `RefreshDesktop`) so the last frame does not remain.
- Tray icon asset (reuse the legacy icon), tooltip text, and English-only strings in M1 (i18n is Phase 5).
- Crate placement within `docs/ARCHITECTURE.md`: legacy reader in `sucrose-config`, theme types in `sucrose-core`, engine trait in `sucrose-engine`, mpv in `sucrose-engine-mpv`.

### Deferred Ideas (OUT OF SCOPE)
- Phase 4 (DESK-03): when different wallpapers run on different monitors, each may output its own audio. Same-content sync (D-02) also applies to other video engines and YouTube.
- Phase 3/4 (PERF-01): persist and expose the action-scope setting from D-05 (affected monitor or all, for Pause and Close).
- Phase 4: libVLC and CEF delivered as pinned, xtask-fetched bundled binaries (D-028). ENG-06 then covers WebView2 Runtime and VC++ Redist detection and hints.
- Phase 5: per-architecture packaging of the bundled engines in the installer, and migration from the C# install (SVC-04).
- Phases 6–7: how libmpv ships on macOS and Linux.
- Phase 3: Quit closes all Sucrose apps (TRAY-05), `AppVisible` together with the Portal, and a second instance opening the Portal.
- Possible later improvement (raised by Claude, not requested): decode once and share frames across monitors, instead of one decode per renderer, if the measurements show the N× decode cost matters.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TYPE-01 | User can set a Video theme as the desktop wallpaper | Legacy `Library.json` → theme folder → `SucroseInfo.json` (Type 3) → mpv; theme gate (D-13); Source path containment (Security) |
| ENG-01 | libmpv engine loaded at runtime, embeds into the desktop surface, hardware decode, loop, pause, volume | libmpv-2.dll pinned builds (Standard Stack), `wid` embedding verified in mpv source, option set verified against mpv v0.42.0-38, loader order (D-18) |
| DESK-01 | Surface behind the desktop icons incl. 24H2 hierarchy | Local probe of build 26300.9974 hierarchy, Lively/Intonaco/Microsoft guidance, two attach strategies + classifier (Architecture Pattern 1) |
| DESK-06 | Sharp on every monitor (per-monitor DPI) | PMv2 process awareness, physical-pixel rect math, `WM_DPICHANGED` not delivered to WorkerW children (legacy §2) |
| DESK-07 | Recover from monitor changes and shell restarts | `TaskbarCreated`, `WM_DISPLAYCHANGE` + debounce, verified mpv behavior when the parent window is destroyed, supervisor/backoff |
| PERF-05 | Fullscreen app triggers the configured action | Legacy rule (services.md §1.1), 1 s timer only while rule is active, pure predicate + debounce state machine |
| TRAY-01 | Tray icon with Quit | `tray-icon` 0.26.1 (re-registers on `TaskbarCreated`, verified in source), target-gated dependency |
| FMT-02 | `SucroseInfo.json` read/written compatibly, fixtures | Field table, `Version` semantics, CRLF/indent, null-allowed fields, round-trip with unknown-field preservation |
| FMT-08 | Selected theme and library location read from legacy `Library.json` / `General.json` | Reader design, BOM/CRLF/lenient-default behavior, verified shapes on the maintainer's real files |
| MEAS-01 | RAM/CPU measured vs C# MpvPlayer + Backgroundog | `cargo xtask measure` design (sysinfo + PDH GPU counters verified present), theme video properties |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

Directives that apply to this phase (CLAUDE.md files at the repo root and in `src/`, plus `.claude/rules/`), treated as locked:

- `src/` (C#) is a **read-only reference**: never edit, build or reformat it.
- **Parity (1)** and **Compatibility (2)**: record behavior changes in `docs/FEATURES.md`; prove compatibility with fixture tests.
- **No web runtime in the core (3)**, **cross-platform from day one (4)**: OS code only in `sucrose-platform` and engine crates; macOS/Linux keep compiling (stubs); CI builds Windows x64/x86/ARM64, Linux, macOS.
- **unsafe isolated (5, D-011)**: only `sucrose-platform` and engine/FFI crates, each block with a `// SAFETY:` comment; `#![allow(unsafe_code)]` opt-in per crate.
- **Low resource use is a feature (6)**: optional modules must not start when disabled; measure release builds and record in `docs/MEASUREMENTS.md`.
- **Decisions are written down (7)**: new entries for D-028 (engine delivery) and the version scheme; do not reopen Accepted entries.
- Rust edition 2024, toolchain pinned to 1.99.0, `rustfmt` + `clippy -D warnings`; `thiserror` in crates, `anyhow` in apps; `tracing` only (no `println!` outside `xtask`/tests); no `unwrap()/expect()` on I/O, parsing, IPC or FFI results; no async runtime in the service (D-012); `crossbeam-channel` + plain threads.
- Automation lives in `xtask`, no new shell/PowerShell scripts (D-014); nothing committed may use `/mnt/...` paths, symlinks or WSL-only tooling.
- Workspace dependency rules: versions only in the root `[workspace.dependencies]`; dependencies must be maintained and GPL-3.0 compatible (`deny.toml`); never add `vlc-rs`, `libmpv-sys`, `single-instance`, `libmpv2`.
- Environment: `cargo` in WSL is the Windows toolchain; GUI runs open real windows on the maintainer's desktop: start without focus, never synthesize input, close what you start; set `SUCROSE_DATA_DIR` (+ `WSLENV=SUCROSE_DATA_DIR` from WSL) to a scratch folder; the maintainer's real settings/library are read-only; stop the app before rebuilding (Windows locks running exes).
- Line endings: `.planning/` files use LF; `docs/DECISIONS.md` uses CRLF; preserve each file's endings. User memory: tests run on the maintainer's PC with the C# app installed; **no test-only app flags**.
- Project skill `port-feature` (`.claude/skills/port-feature/SKILL.md`): FEATURES/DECISIONS/MEASUREMENTS must be updated in the same change; the capture script uses `PrintWindow` without stealing focus.

## Summary

The phase is feasible with the decisions in CONTEXT.md, and the riskiest part is not libmpv but the **desktop surface on Windows 11 24H2+/26H2**. On the maintainer's machine (Windows 10.0.26300.9974, one 1920x1080 display) the "raised desktop" hierarchy is present: `Progman` has `WS_EX_NOREDIRECTIONBITMAP`, its topmost child is the `WS_EX_LAYERED` `SHELLDLL_DefView` and below it is a visible full-screen `WorkerW` child, and there is no top-level `WorkerW` holding the DefView. Public reports (Lively issue #2074, Intonaco PR #5, both quoting Microsoft guidance) say a plain child of that tree may never be composited, and recommend a `WS_EX_LAYERED` child of `Progman` z-ordered directly under DefView with `SetLayeredWindowAttributes(255, LWA_ALPHA)`. mpv's own `--wid=0` path and the legacy C# fallback instead parent into the `WorkerW` child of `Progman`. Plan the surface as a **classify-then-attach** component with both strategies behind one trait method, and make a headless-verifiable spike the first plan so later plans do not build on an unproven attach.

libmpv delivery is solved and verified: the shinchiro `mpv-winbuild-cmake` release 20261111 ships `mpv-dev-{x86_64,i686,aarch64}` archives containing `libmpv-2.dll` (mpv v0.42.0-38, client API 2.6, GPL build). I downloaded all three, matched their SHA256 to the GitHub release digests, and confirmed the PE machine types (x64, ARM64, i386). Two things the plan must handle: upstream prunes releases (~30 kept), so pins must be **mirrored** to a Sucrose-controlled location, and the archives contain no license file. The mpv `wid` embedding path is verified in mpv's source: mpv creates its own child window, resizes it with the parent via a hook, and shuts down when the parent window is destroyed, so an explorer.exe restart (which destroys `Progman` and every child) means **respawn the renderer**, not "re-parent".

The legacy mpv config in the C# app contains options that **no longer exist** in this mpv (`v-sync`, `volume-min`, `audio-pass-through` were rejected or not found when tested against the real build), so the generated config must be re-derived and validated against a real libmpv in a test rather than copied. Everything else (service/renderer split, `interprocess` + `postcard` IPC with a current-user-only pipe ACL, `tray-icon` target-gated to Windows, 1 s fullscreen polling only while the rule is active, `cargo xtask measure` with `sysinfo` + PDH GPU counters) uses mainstream, verified crates.

**Primary recommendation:** Build in this order: (1) workspace deps, version scheme, D-028 docs and `cargo xtask fetch-engines`; (2) a desktop-surface spike proving attach on build 26300 with both strategies and a `libmpv` child window; (3) pure-logic crates with fixtures (theme/legacy reader, IPC protocol, fullscreen state machine, sync controller, mpv config); (4) service/renderer integration; (5) `xtask measure` and docs. Keep every OS call behind `sucrose-platform` and every libmpv call behind `sucrose-engine-mpv`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Read legacy settings and theme (FMT-02, FMT-08) | Library crates (`sucrose-config`, `sucrose-core`) | Service process | Pure parsing, no OS code, reused by the Phase 3 migration; service calls it once at start |
| Single instance, supervising renderers, tray, fullscreen rule | Service process (`sucrose`) | Platform layer | D-008: service owns policy; OS facts (foreground window, monitors) come from `sucrose-platform` |
| Desktop surface (Progman/WorkerW attach, DPI, hot-plug facts, TaskbarCreated) | Platform layer (`sucrose-platform`) | Renderer process | Only crate allowed OS code (D-002/D-011); the renderer owns the surface HWND for its monitor |
| Video decode and presentation (libmpv, hwdec, loop, volume, pause, stretch) | Renderer process via `sucrose-engine-mpv` | Engine library (libmpv-2.dll) | Crash isolation (D-008); only renderers map libmpv (D-17) |
| Cross-monitor sync clock and audio ownership | Service (policy: who is master) | Renderers (apply drift correction) | Master/follower are renderer roles; the service assigns and re-assigns them over IPC |
| IPC transport, framing, single-instance endpoint | `sucrose-ipc` crate | Service and renderers | D-009; one endpoint serves both IPC and single-instance |
| Engine binary fetch, pin verification, placement | `xtask` | — | D-014/D-016: never at runtime, never committed to git |
| Process measurement (RAM/CPU/GPU) | `xtask measure` | OS counters (PDH, process APIs) | D-26; reusable for MEAS-02 |
| Last-frame cleanup on stop (`RefreshDesktop`) | Platform layer | Service | Legacy behavior; OS call (`SystemParametersInfo`-style refresh) |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| libmpv (shinchiro build 20261111, mpv v0.42.0-38-gbcde6b93c, client API 2.6) | pinned | Video engine, loaded at runtime | Same engine as the legacy app; x86, x64 and ARM64 builds exist in one release [VERIFIED: GitHub API + local SHA256 + PE header check] |
| `windows` | 0.62.2 (cfg(windows) only) | Win32 FFI in `sucrose-platform` | Mandated by `.claude/rules/platform-ffi.md`; Microsoft-maintained [VERIFIED: crates.io, gsd package-legitimacy OK] |
| `libloading` | 0.9.1 | Runtime loading of libmpv | D-013 [VERIFIED: crates.io] |
| `bindgen` | 0.73.2 (xtask-only, dev time) | Generate dynamic-loading bindings for `client.h` | D-013 [VERIFIED: crates.io]; needs libclang, see Pitfall 9 |
| `interprocess` | 2.4.9 | Local-socket IPC + single instance | D-009; `ListenerOptionsExt::security_descriptor` and first-pipe-instance flag verified in source [VERIFIED: github.com/kotauskas/interprocess src/os/windows/local_socket.rs, create_instance.rs] |
| `postcard` | 1.2.0 | Binary message encoding | D-009 [VERIFIED: crates.io] |
| `serde` / `serde_json` | 1.0.229 / 1.0.154 | Derive + legacy JSON parsing | De-facto standard [VERIFIED: crates.io] |
| `crossbeam-channel` | 0.5.17 | Thread communication | D-012 [VERIFIED: crates.io] |
| `tray-icon` | 0.26.1 (cfg(windows), `default-features = false`) | Tray icon + Quit menu | Maintained by Tauri; re-registers on `TaskbarCreated` [VERIFIED: source tray-icon-v0.26.1 src/platform_impl/windows/mod.rs L51-54, L159-166] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sysinfo` | 0.40.4 (xtask) | Per-process CPU and memory sampling | `cargo xtask measure` [VERIFIED: crates.io; MSRV 1.95 ≤ toolchain 1.99] |
| `windows` (feature `Win32_System_Performance`) | 0.62.2 (xtask, cfg(windows)) | PDH GPU counters | GPU % and GPU memory per PID in `xtask measure` |
| `ureq` | 3.5.2 (xtask) | HTTPS download of pinned archives | `cargo xtask fetch-engines` [VERIFIED: crates.io] |
| `sha2` | 0.11.1 (xtask) | SHA256 verification | D-16 pin check [VERIFIED: crates.io] |
| `sevenz-rust2` | 0.25.0 (xtask) | Extract `.7z` without external tools | Cross-platform D-014 (Apache-2.0, allowed by `deny.toml`) [VERIFIED: crates.io] |
| `toml` | 1.2.2 (xtask) | Read the pin manifest `xtask/engines.toml` | Keep pins out of code [VERIFIED: crates.io] |
| `embed-manifest` | 1.5.1 (build-dependency of `apps/sucrose`, optional) | Embed a PerMonitorV2 + UTF-8 + supportedOS manifest | Alternative to the runtime DPI call (see Pattern 4) [VERIFIED: crates.io, MIT] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `wid` embedding | mpv **D3D11 render API** (`render_d3d11.h`, in mpv v0.42.0 but **not shipped** in the shinchiro dev archive) | We would own the swapchain: full control of DPI, sizing, presentation and a path to "decode once, show on N monitors". More code (D3D11/DXGI swapchain, frame pacing) and a header that must be fetched from the mpv tag. Not for M1; record as the Phase 4+ option [VERIFIED: mpv v0.42.0 include/mpv/render.h L52-54 and `render_d3d11.h`] |
| shinchiro build | zhongfly/mpv-winbuild (has `lgpl` variants) | LGPL variants exist only for x86_64 and aarch64 (no i686); useful if the GPL question becomes a problem [VERIFIED: GitHub API asset list] |
| `tray-icon` | Hand-rolled `Shell_NotifyIconW` in `sucrose-platform` | Fewer deps and RAM, but we must re-add the icon on `TaskbarCreated` ourselves; only worth it if measurements show the crate matters |
| `sevenz-rust2` | System `7z` / `tar` | Zero crates but breaks "same on every host" (D-014) |

**Installation (workspace `Cargo.toml`, versions central):**
```toml
# [workspace.dependencies] additions (all verified on crates.io 2026-11-22)
windows = "0.62.2"
libloading = "0.9.1"
interprocess = "2.4.9"
postcard = { version = "1.2.0", features = ["use-std"] }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.154"
crossbeam-channel = "0.5.17"
tray-icon = { version = "0.26.1", default-features = false }
# xtask only
bindgen = "0.73.2"
sysinfo = "0.40.4"
ureq = "3.5.2"
sha2 = "0.11.1"
sevenz-rust2 = "0.25.0"
toml = "1.2.2"
```
The `postcard` feature name `use-std` is [ASSUMED] (check `cargo add postcard --features` output); `tray-icon`'s default feature is `libappindicator` (Linux GTK), so it **must** stay `default-features = false` and be a `[target.'cfg(windows)'.dependencies]` entry, otherwise the Linux CI jobs (which do not install GTK) fail [VERIFIED: tray-icon-v0.26.1 Cargo.toml `default = ["libappindicator"]`].

**Version verification:** every version above was read from the crates.io API on 2026-11-22 (`max_stable_version`, license, MSRV); `windows` 0.62.2 was published 2025-10-06 (no newer `windows` release; `windows-core` has a 0.100.0 on 2026-09-03 that is not paired with a `windows` release yet, so stay on 0.62.2).

### libmpv pins (verified)

| Arch | Asset (tag `20261111`, repo `shinchiro/mpv-winbuild-cmake`) | Size | SHA256 (equals GitHub digest and local `sha256sum`) | `libmpv-2.dll` |
|---|---|---|---|---|
| x64 | `mpv-dev-x86_64-20261111-git-bcde6b93cc.7z` | 31,776,374 | `273c079b11ea5575822f972756f4c9a71149a5c10130353abeb8586738938452` | 121,789,952 B, PE32+ x86-64 |
| x86 | `mpv-dev-i686-20261111-git-bcde6b93cc.7z` | 29,634,506 | `f648b25096e5db9ee8e6b3a82bfa9ab57ec5634ab4820cf58c9ec71b5b033d98` | 99,536,384 B, PE32 i386 |
| ARM64 | `mpv-dev-aarch64-20261111-git-bcde6b93cc.7z` | 26,329,311 | `a42c1cebf7366a957c7c321a20df414b80d4a55bbc65ded032d78b749c8d6445` | 84,073,984 B, PE32+ ARM64 |

- Do **not** use the `x86_64-v3` archives (require AVX2-class CPUs) [ASSUMED: v3 = x86-64-v3 microarchitecture level].
- Each archive contains `include/mpv/{client.h,render.h,render_gl.h,stream_cb.h}`, `libmpv-2.dll` and `libmpv.dll.a`; **no license file** [VERIFIED: `7z x` listing]. `render_d3d11.h` is referenced by `render.h` but absent.
- `libmpv-2.dll` reports `mpv v0.42.0-38-gbcde6b93c` and a configuration with `-Dlibmpv=true` and the feature list including `gpl`, `d3d11`, `d3d-hwaccel`, `wasapi`, `luajit`, `javascript`, `rubberband-3`, `vulkan` [VERIFIED: `strings` on the DLL and `mpv.com --version` of the matching full archive]. Header `MPV_CLIENT_API_VERSION` is `MPV_MAKE_VERSION(2, 6)` [VERIFIED: client.h L250-251].
- Import table (x64/i686): only system DLLs (`KERNEL32`, `USER32`, `d2d1`, `dwmapi`, `vulkan-1.dll`, ...); ffmpeg and friends are statically linked. `vulkan-1.dll` appears in the import list; whether it is delay-loaded (works without a Vulkan runtime) is **not verified**, see Open Question 5.
- **Upstream retention:** the release list holds 30 releases (oldest 2026-10-03, newest 20261111 on 2026-11-11, none since) [VERIFIED: GitHub API]. A pinned upstream URL will 404 in about a month. **Mirror the three pinned archives to a Sucrose-controlled release asset** (not git) and pin that URL plus the SHA256; keep the upstream tag/commit in the manifest for provenance.
- The legacy docs claim 75–101 MB per architecture; the pinned x64 DLL is 121.8 MB. Update D-028's size note.

## Package Legitimacy Audit

Run via `gsd_run query package-legitimacy check --ecosystem crates ...` on 2026-11-22: every package returned **OK** (output was cut after `embed-manifest` in this session's log, but all verdict lines shown were `OK` and the remaining names are the same mainstream crates; the planner should re-run the check once in Wave 0 and keep the output with the plan).

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| windows | crates.io | 7 yrs | ~9.2M/wk | github.com/microsoft/windows-rs | OK | Approved |
| libloading | crates.io | 10 yrs | ~13.2M/wk | github.com/nagisa/rust_libloading | OK | Approved |
| bindgen | crates.io | 11 yrs | ~8.6M/wk | github.com/rust-lang/rust-bindgen | OK | Approved (xtask only) |
| interprocess | crates.io | 6 yrs | ~0.4M/wk | github.com/kotauskas/interprocess | OK | Approved (D-009) |
| postcard | crates.io | 7 yrs | ~3.5M/wk | github.com/jamesmunns/postcard | OK | Approved (D-009) |
| serde / serde_json | crates.io | 11 yrs | ~35M/wk | github.com/serde-rs | OK | Approved |
| crossbeam-channel | crates.io | 8 yrs | ~13.0M/wk | github.com/crossbeam-rs/crossbeam | OK | Approved |
| tray-icon | crates.io | 4 yrs | ~1.6M/wk | github.com/tauri-apps/tray-icon | OK | Approved |
| sysinfo | crates.io | 11 yrs | ~5.1M/wk | github.com/GuillaumeGomez/sysinfo | OK | Approved (xtask) |
| embed-manifest | crates.io | 5 yrs | ~75K/wk | codeberg.org/carey/embed-manifest | OK | Approved (optional) |
| ureq, sha2, sevenz-rust2, toml | crates.io | see Standard Stack | high | upstream repos | not individually re-read | Approved pending the Wave 0 re-run (all mainstream; `sevenz-rust2` is the youngest, 2025-02, ~1.6M recent downloads) |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none
No `postinstall` equivalent exists for crates; `build.rs` scripts of `windows`/`tray-icon` are standard. `cargo deny check` must pass in CI (new transitive licenses from `ureq` → `rustls` → `ring`/`webpki-roots` are expected to be covered by the allow-list: `ISC`, `Apache-2.0`, `CDLA-Permissive-2.0`; verify).

## Architecture Patterns

### System Architecture Diagram

```
 C# install (read-only)                                   %APPDATA%\Sucrose\Setting\*.json, Cache\MpvPlayer\uMpvPlayer.config
        │ (D-08: always the real folder)                         │
        ▼                                                        ▼
 ┌──────────────────────────── sucrose.exe  (service, main thread = Win32 message loop) ───────────────────────────┐
 │ start: single-instance listener (named pipe, first instance, current-user ACL)  ── second instance logs + exits │
 │        read Library/General/Engine/Backgroundog (sucrose-config) ─► resolve theme (sucrose-core) ─► gate D-13   │
 │        locate libmpv path (probe only; service never loads it, D-17)                                           │
 │  ┌─ tray (tray-icon, only if AppVisible)  Quit ─► stop renderers ─► exit                                       │
 │  ├─ hidden top-level window: WM_DISPLAYCHANGE / WM_DPICHANGED / TaskbarCreated ─► debounce ─► topology diff     │
 │  ├─ fullscreen timer (1 s, only when FullScreenPerformance != Resume): foreground rect vs monitor rects ─► 5/3  │
 │  └─ supervisor: spawn/respawn renderers (job object KILL_ON_CLOSE), backoff 3 per 60 s, master election        │
 └───────────────┬───────────────────────────────────────────────────────────────────────────────────────────────┘
                 │ local socket (postcard frames): Hello, Assign{role,monitor,theme,engine cfg}, Clock, Pause, Close, Volume
        ┌────────┴──────────┬───────────────────┐
        ▼                   ▼                   ▼
  sucrose --render      sucrose --render     sucrose --render        (one per monitor, D-01)
  monitor A (master)    monitor B (follower) monitor C (follower)
  PMv2 DPI aware        … same …             … same …
  1. classify desktop layout, attach surface HWND to Progman/WorkerW, size to monitor rect (physical px)
  2. libloading libmpv-2.dll ─► mpv_create ─► set wid=<surface HWND> + options ─► mpv_initialize ─► loadfile
  3. event thread: mpv_wait_event ─► time-pos observation ─► (master) Clock{pos, qpc} to service ─► followers
  4. on parent destroyed (explorer restart) or mpv SHUTDOWN: exit non-zero ─► service respawns after TaskbarCreated
```

Primary use case trace: user theme selected in the C# Portal → service starts → reads `Library.json` → resolves `<Location>\<Selected>\SucroseInfo.json` (Type 3, `AppVersion ≤ app version`) → spawns one renderer per monitor → each renderer attaches a surface and plays the same file through its own libmpv → master emits clock ticks, followers correct drift → fullscreen timer pauses/closes per `FullScreenPerformance` → Quit in the tray stops everything.

### Recommended Project Structure

```
crates/
├── sucrose-core/            # + theme.rs (SucroseInfo, WallpaperType, LegacyVersion), legacy_paths.rs (legacy data dir that ignores SUCROSE_DATA_DIR)
├── sucrose-config/          # NEW: legacy.rs (Properties-wrapper reader, lenient typed getters), later the FMT-01 migration
├── sucrose-ipc/             # NEW: protocol.rs (messages), frame.rs (length-prefixed postcard), endpoint.rs (name, listener, ACL), single_instance
├── sucrose-engine/          # NEW: Engine trait, EngineError, EngineSettings, Stretch, engine discovery result types
├── sucrose-engine-mpv/      # NEW: sys.rs (generated bindings, committed), loader.rs, player.rs, config.rs, sync.rs (pure controller)
├── sucrose-platform/        # + windows/{surface,layout,monitors,events,foreground,job,dpi}.rs; macOS/Linux stay stubs
└── sucrose-log/             # unchanged ("Service", "Renderer-<monitor-index>")
apps/sucrose/                # main.rs: mode switch (service vs --render), service/, render/, tray.rs, supervisor.rs, fullscreen.rs
xtask/                       # + engines.toml (pins), fetch_engines.rs, bindgen_mpv.rs, measure.rs
```
Dependency rules to respect: `sucrose-engine-mpv` → `sucrose-engine`, `sucrose-platform` (surface handle type), `sucrose-core`; `sucrose-ipc` and `sucrose-config` have no `unsafe` and no OS-specific code except `cfg` to pick the named-pipe ACL helper (place the ACL construction in `sucrose-platform` if `interprocess`'s SDDL helper needs `unsafe`); `sucrose-core` stays OS-free (the legacy data directory is `dirs::config_dir()/Sucrose` on every OS, ignoring `SUCROSE_DATA_DIR`, so it can live in core).

### Pattern 1: Classify the desktop layout, then attach (DESK-01)

**What:** One platform function inspects the window tree and returns a layout; the attach strategy is chosen from the layout, not from the OS build number.

Layout facts on the maintainer's machine, Windows 10.0.26300.9974, one 1920x1080 display, probed read-only with a PMv2-aware script on this session's host [VERIFIED: local probe]:
```
Progman  ex=0x200080 (WS_EX_NOREDIRECTIONBITMAP|WS_EX_TOOLWINDOW) rect=(0,0,1920,1080)
  SHELLDLL_DefView ex=0x80000 (WS_EX_LAYERED) style=0x56010000 rect=(0,0,1920,1080)   <- topmost child
    SysListView32 ...
  WorkerW          ex=0x0 style=0x58000000 visible rect=(0,0,1920,1080)               <- below DefView
(no top-level WorkerW that owns a SHELLDLL_DefView)
```
Known layouts and strategies:

| Layout (how to detect) | Strategy |
|---|---|
| **Raised** (24H2+/26xxx): `Progman` has `WS_EX_NOREDIRECTIONBITMAP` **or** `SHELLDLL_DefView` is a child of `Progman` | **A** (legacy fallback, mpv's own `--wid=0` path): `SetParent(surface, FindWindowExW(Progman, 0, "WorkerW"))`. **B** (Microsoft guidance quoted in Lively #2074): make the surface a `WS_EX_LAYERED` child of `Progman`, `SetLayeredWindowAttributes(surface, 0, 255, LWA_ALPHA)` **before** `SetParent`, then `SetWindowPos(surface, DefView, 0,0,0,0, SWP_NOMOVE\|SWP_NOSIZE\|SWP_NOACTIVATE)` so it sits directly under DefView and above the system `WorkerW`. Try A, verify visibility/coverage, fall back to B (or the reverse if the spike shows B is the one DWM composites). |
| **Classic** (Win10/early Win11): a top-level window has a `SHELLDLL_DefView` child and a sibling `WorkerW` | `SendMessageTimeout(Progman, 0x052C, 0xD, 1, SMTO_NORMAL, 1000)`, `EnumWindows` → `FindWindowExW(NULL, thatTop, "WorkerW", NULL)` → `SetParent` (legacy sequence, up to 8 attempts, retry on empty) |

Evidence: legacy sequence [CITED: docs/legacy/engines.md §2 "Attach sequence", L70-76]; mpv's `get_workerw_hwnd()` does `FindWindowExW(progman, NULL, L"WorkerW", NULL)` first ("24H2 or later") then `EnumWindows`, and `set_raised_desktop()` sends `0x052c, 0xd, 1` [VERIFIED: mpv v0.42.0 video/out/w32_common.c, `get_workerw_hwnd`, `set_raised_desktop`, `gui_thread`]; the layered-child guidance, "DefView renders transparently only with HDR on or during a slideshow animation" on some builds, and "0x052C sent once only when the tree has no wallpaper layer, because extra variants can undo the new layer" [CITED: github.com/lively-community/lively/issues/2074, github.com/77worldd/Intonaco/pull/5]; Intonaco reports "a plain child window ... never composited" in that tree on 26H2 build 26300.9711 and uses strategy B [CITED: Intonaco PR #5].

**Rules for the implementation:**
- Send `0x052C` once, and only when no wallpaper `WorkerW` exists yet (the local probe shows one already exists) [CITED: Intonaco PR #5].
- Verify after attach (window visible, rect equals target, ancestor chain ends at `Progman`) instead of trusting `SetParent`'s return value (the legacy code can call `SetParent(h, NULL)`, engines.md L76).
- Retry with backoff up to ~10 s: the `WorkerW` may appear late after shell start [CITED: AutoHotkey forum thread referenced by the web search; treat as MEDIUM].
- Keep a periodic self-check **only if** the spike shows the shell reorders the surface (Intonaco re-asserts z-order every 5 s). Prefer event-driven repair (`TaskbarCreated`, display change) per D-11; add a timer only with evidence.
- Place the surface by **screen rectangle**: compute the target rect in physical screen pixels (monitor `rcMonitor` or `rcWork` per `ScreenType`), then convert into the parent's client space with `MapWindowPoints(HWND_DESKTOP, parent, ...)`. Do not assume the parent origin equals the virtual-desktop origin (the legacy code offsets by the combined virtual-desktop rectangle, engines.md L78-80).
- Hide from taskbar/Alt-Tab: create the surface as `WS_EX_TOOLWINDOW|WS_EX_NOACTIVATE`, no `WS_EX_APPWINDOW` [ASSUMED]; legacy toggled `ShowInTaskbar` to dodge a Task View crash (engines.md L66-68).

**Spike first (Wave 0 plan):** it changes the maintainer's real desktop, so it is a normal plan (not a throw-away binary) whose verification is a `human-verify` checkpoint: run `sucrose --render` (no focus), confirm the video is visible behind the icons, capture with `PrintWindow`, then quit. Ask the maintainer whether the C# app currently shows its wallpaper on this machine: if it does, strategy A (what the C# app uses on this layout) is proven on build 26300.

### Pattern 2: Renderer = Win32 thread + mpv event thread

- **Main thread:** owns the surface HWND and runs `GetMessage`. The `wid` parent window **must** be created on a thread that pumps messages: mpv installs a `WH_CALLWNDPROC` hook on the parent's thread (same process) and resizes its child on `WM_WINDOWPOSCHANGED` [VERIFIED: w32_common.c `install_parent_hook`, `parent_win_hook`, `resize_child_win`]. This replaces the legacy `Surface.Correct()` hack.
- **mpv thread:** `mpv_wait_event(handle, timeout)` loop (or `mpv_set_wakeup_callback` posting a thread message). `mpv_set_wakeup_callback`'s callback must not call back into mpv [ASSUMED from client.h L1773 contract].
- **IPC thread:** blocking read loop on the socket, forwards decoded messages through a `crossbeam-channel` and wakes the main thread with `PostMessage(WM_APP)`.
- Commands are mpv properties set from the thread that owns the handle's logic; `pause`, `volume`, `mute`, `speed`, `panscan`, `keepaspect`, `video-unscaled`, `loop-file` are plain properties.

**Handle setup order (verified against client.h):** `mpv_create` → options (`wid`, `vo`, `gpu-context`, `hwdec`, ...) → optionally `mpv_load_config_file(<user uMpvPlayer.config>)` → `mpv_initialize` → `mpv_observe_property(time-pos, pause)` → `mpv_command(["loadfile", path])`. libmpv defaults are already "terminal off, no config files, idle mode on, reduced input handling" [VERIFIED: client.h L438-461]; the legacy app set `wid` via a property after creation, which also worked, but options before `mpv_initialize` is the documented route. The `wid` value is "a value cast to `uint32_t` (all Windows handles are 32-bit)" [VERIFIED: mpv v0.42.0 DOCS/man/options.rst `--wid`]; pass the HWND as a decimal string through `mpv_set_option_string("wid", ...)` using the low 32 bits, never as a negative number. `mpv_load_config_file` ignores errors for unknown options (it returns an error only for a fatal parse error), so D-10's user config cannot break startup but may silently not apply [VERIFIED: client.h L585-600].

**Generated option set (re-derived and checked against `mpv.com --list-options` of v0.42.0-38; each name below exists unless marked):**
```
# identity/embedding
wid=<hwnd as u32>            vo=gpu                gpu-context=d3d11
# decode and sync
hwdec=auto-safe | no         (HardwareAcceleration; "auto-safe" was accepted by this build on the command line)
profile=low-latency          framedrop=decoder     video-sync=audio     cache=yes
demuxer-max-bytes=64M        demuxer-seekable-cache=yes
# audio (master only; followers set aid=no, see Pattern 3)
volume=<WallpaperVolume>     volume-max=100        audio-channels=stereo   audio-samplerate=48000   audio-buffer=0.1
# looping and end
loop-file=inf | no           keep-open=no          idle=yes
# video look (legacy parity)
deband=yes                   interpolation=no      scale-antiring=0        temporal-dither=no     gpu-shader-cache=yes
# system hygiene
stop-screensaver=no          media-controls=no
# hardening for an embedded, headless instance [ASSUMED names; validate with the option test]
osc=no  input-default-bindings=no  input-vo-keyboard=no  input-cursor=no  cursor-autohide=no  border=no
load-scripts=no  ytdl=no  terminal=no  config=no
# cache locations under SUCROSE_DATA_DIR (writes only; never the real legacy folder)
gpu-shader-cache-dir=<Cache\MpvPlayer\shaders>
```
**Legacy keys that do not exist in this mpv and must be dropped from the generated config:** `v-sync` (command-line load printed `Error parsing option v-sync (option not found)`), `volume-min` and `audio-pass-through` (not in `--list-options`), and the flag-style `no-idle` (`idle` is now a `no|once|yes` choice). `fullscreen=yes` and `unregister=yes` are accepted names but meaningless for an embedded instance; omit `fullscreen` (embedded mode ignores fullscreen, `w32->parent` checks in w32_common.c) and omit `unregister` [VERIFIED: option listing + source; legacy list at src/Shared/Engine/Sucrose.Shared.Engine/Manage/Internal.cs L100-149]. Also `hwdec-codecs=all` exists but changes the default whitelist; keep only if hardware decode of the maintainer's 4K HEVC needs it (measure with and without).
**Stretch mapping** (legacy, unchanged): `panscan=1.0` only for UniformToFill; `keepaspect=no` only for Fill; `video-unscaled=yes` only for None (engines.md §3 L129-140; legacy code `Helper/Video.cs` `SetStretch`). `StretchType` is applied by the legacy app every second; in Rust apply it once at open and when the service sends a settings change.
**Volume:** legacy sets `Volume = WallpaperVolume` (0..100) [VERIFIED: legacy `View/Video.xaml.cs` initializer `Volume = SSEHD.GetVolume()` and Mpv.NET `MpvPlayer.Volume` setter → `volume` property]. `VolumeSilent`/`VolumeDesktop` are outside the D-09 key list and stay out of M1.
**Hard-vs-soft loop end:** with `loop-file=no` and `keep-open=no` the file ends and the desktop shows nothing; legacy had the same behavior. Record the choice; consider `keep-open=yes` (hold last frame) as a documented improvement only after maintainer sign-off.

### Pattern 3: Cross-monitor sync (D-02), designed as a pure controller

Verified facts: with `video-sync=audio` and "If audio is disabled, this uses the system clock" [VERIFIED: mpv v0.42.0 options.rst `--video-sync`]; `seek <target> absolute+exact` is available, and `time-pos` / `playback-time` are readable in seconds [VERIFIED: mpv v0.42.0 input.rst]. The rest is design [ASSUMED, tune in the spike].

- **Roles:** the service picks the master (primary monitor's renderer, D-02 discretion) and sends `Assign{role}`. Followers start with `aid=no` (no audio decode at all: cheaper and no echo). Promotion on master loss: set `aid=auto` and `volume` on the promoted renderer, set `aid=no` on the others.
- **Clock message:** master observes `time-pos` and sends `Clock{pos_s, qpc_ticks, speed, paused, loop_gen}` about once per second and immediately on seek, loop wrap, pause, resume. `QueryPerformanceCounter` values are comparable across processes on one machine (system-wide counter) [ASSUMED]; `QueryPerformanceFrequency` is constant after boot.
- **Follower correction:** expected = pos + (now_qpc − qpc)/freq × speed, wrapped modulo `duration` when looping (use circular distance near the wrap). Drift thresholds (starting values): ≤ 1 frame (≈33 ms at 30 fps): do nothing; ≤ 500 ms: set `speed` to 1.02 or 0.98 until inside tolerance, then `speed=1`; > 500 ms: `seek absolute+exact`. Avoid seeks while decoding 4K HEVC unless needed (a seek costs a keyframe-to-target decode burst).
- **Pause scope (D-05):** when only one monitor pauses, it re-aligns on resume through the same correction path; all-monitor Pause pauses every renderer and the master's next Clock tick (paused=true) keeps followers from "catching up".
- **Pure code:** implement the controller as a pure function `fn correct(state, tick, now_qpc) -> Action` in `sucrose-engine-mpv/src/sync.rs` so it is unit-testable without libmpv or a desktop.
- Different monitor refresh rates mean frames are shown on different vsync grids; a residual offset below one frame is physically unavoidable. State this as the achievable bound in FEATURES.md.

### Pattern 4: DPI (DESK-06)

- The renderer process must be Per-Monitor-V2 aware **before** creating any window or loading libmpv, so window and mpv-child sizes are physical pixels and Windows does not bitmap-scale the surface (blurriness). Two ways: call `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` first thing in `main` (simple, no build step; threads mpv creates inherit the process default) [ASSUMED], or embed a manifest with `embed-manifest` (also gives `supportedOS` Windows 10 and UTF-8 code page). Legacy engines declared `PerMonitorV2` in `App.manifest` (engines.md §3 L151-153). Recommend the manifest for the final exe and a runtime assertion (`GetThreadDpiAwarenessContext` equals PMv2, logged) to catch regressions.
- A window parented into `WorkerW`/`Progman` stops receiving `WM_DPICHANGED` (legacy note, engines.md L80-82). So the service owns topology and DPI: it re-enumerates monitors (rect, `GetDpiForMonitor`) on `WM_DISPLAYCHANGE` / `WM_DPICHANGED` / device-arrival, debounces, diffs against the last snapshot, and tells affected renderers `SetRect{rect_px}`. mpv scales to the surface; there is no mpv-level DPI setting to touch.
- Mixed-DPI parenting (child PMv2 inside a shell parent with a different awareness context) is a known Windows area with a hosting-behavior setting; the spike must test a two-DPI setup, or at least a non-100% scale on the single display [ASSUMED, flagged in Open Questions].

### Pattern 5: Fullscreen rule (PERF-05)

- **Evaluate only while the rule is active:** a `SetTimer` 1000 ms on the service's message loop that exists only when `FullScreenPerformance` is `Pause` or `Close` (D-008: nothing runs when the rule is `Resume`, which is the maintainer's current state). This keeps the legacy semantics exactly (5 consecutive true polls to trigger, 3 consecutive false polls to recover [CITED: docs/legacy/services.md §1.1 table row FullScreen, L99-100]) and is simpler to test than WinEvent hooks. Re-read the setting when it changes (the Portal is out of scope; re-read on service start only in M1 and document it).
- **Predicate (pure function over a snapshot):** `foreground_hwnd` exists, is visible, not minimized, not cloaked (`DWMWA_CLOAKED`), not one of our windows, not the desktop shell (`Progman`, `WorkerW`, `SHELLDLL_DefView`, the taskbar), and its `GetWindowRect` covers any monitor's `rcMonitor` completely. **Skip entirely while the desktop is focused** (foreground class `WorkerW` or `SHELLDLL_DefView`, or `Progman`): [CITED: services.md L99-100; engines.md volume logic L185-190 for the desktop-focus classes].
- **State machine:** `enum Fs { Idle, Triggering(n), Active, Recovering(n) }` with the 5/3 counters; inject a clock/snapshot source so tests drive it without a desktop.
- **Action:** `Pause` → `Pause` message to all renderers (D-05 default scope "all monitors"; the per-monitor scope must be implemented but unreachable); `Close` → stop renderers and respawn on recovery (D-06). `PausePerformanceType = Heavy` is treated as `Light` (note in FEATURES.md).
- Success criterion 3 requires `FullScreenPerformance` to be `Pause`, which means editing the maintainer's **real** `Backgroundog.json` (read-only per CLAUDE.md). Make that a `checkpoint:human-action` in the plan: the maintainer sets the key, runs the check, and restores it.

### Pattern 6: Supervisor, shell restart and monitor changes (DESK-07, D-22)

- **Explorer restart:** the service's hidden top-level window receives the registered `TaskbarCreated` message (`RegisterWindowMessageW("TaskbarCreated")`) [CITED: tray-icon source L51-54 documents it as broadcast to all top-level windows; `ChangeWindowMessageFilterEx` is used so elevated processes still get it, L159]. When explorer dies, `Progman` and all its children, including the renderer surfaces **and mpv's own child window, are destroyed**; mpv posts a close key on `WM_NCDESTROY`/`WM_DESTROY` when it did not initiate it, "This can also happen in --wid mode when the parent window is destroyed" [VERIFIED: w32_common.c L1489-1494]. So the recovery action is **respawn the renderer** (or at least recreate the mpv handle), after waiting for `TaskbarCreated` plus a retry loop for `Progman`/`WorkerW`. Renderer-side: treat parent destruction as "exit with a distinct code"; service-side: such exits during a pending shell restart do not count toward the 3-in-60-s budget.
- **Monitor hot-plug (legacy timing 2000 ms debounce then 500 ms, engines.md L95-96):** on a debounced topology change the service diffs snapshots by monitor identity (device name such as `\\.\DISPLAY1` plus `HMONITOR` as a hint): new monitor → spawn; removed → close that renderer and, if it was master, promote another; moved/resized/DPI changed → `SetRect`. Do not respawn for pure geometry changes (keeps playback position).
- **Backoff:** a small struct `RestartBudget{ window: 60 s, max: 3 }` with a pure `allow(now) -> bool` plus exponential delay (0.5 s, 1 s, 2 s). Past the budget the monitor stays empty; log and tooltip text name the reason (D-22).
- **Process lifetime tie:** put every renderer into a **job object** with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` so a crashed or killed service cannot leave renderers painting the desktop (Quit and Task Manager both end cleanly) [ASSUMED from Win32 docs; verify].
- **Stop cleanup:** legacy `RefreshDesktop` repaints the desktop so the last frame does not remain (engines.md L101-102). Implementation (e.g. `SystemParametersInfoW(SPI_SETDESKWALLPAPER, ...)` with the current path, or `InvalidateRect(NULL,...)`) is [ASSUMED]; destroying the surface normally exposes the system `WorkerW` again, so test it before adding code.

### Pattern 7: Legacy reader (FMT-08) and theme (FMT-02)

- **Reader contract:** read `%APPDATA%\Sucrose\Setting\<File>.json` (use `dirs::config_dir()` → `Roaming` on Windows), ignore `SUCROSE_DATA_DIR` (D-08). Parse into `serde_json::Value`, take `Properties` as a map, and expose **typed getters with defaults and clamps** (`get_bool("AppVisible", true)`, `get_i32_clamped("WallpaperVolume", 100, 0..=100)`, `get_enum::<Stretch>("StretchType", UniformToFill)`). Any failure returns the default, exactly like the legacy `GetSetting<T>` ("Any exception returns the default silently", formats.md §1.1) but with a `tracing::warn!` so the Rust side is observable.
- **Format facts [VERIFIED on the maintainer's real files and docs/legacy/formats.md §1.1, L9-36]:** wrapper `{"Properties": {...}}`, no BOM (first bytes `7b0d0a` = `{` CR LF), CRLF line endings, enums as **name strings** (case-sensitive; numeric strings also accepted by legacy `Enum.Parse`), ints as JSON numbers, UTF-8. `serde_json` rejects a UTF-8 BOM, so strip a leading `\u{FEFF}` defensively (legacy `StreamReader` tolerates it).
- **Concurrent writer:** the C# app writes with `FileShare.None` and a truncating write, **not atomic** (formats.md L28-30). A read can fail with a sharing violation or see an empty/truncated file; retry 3× with short sleeps before falling back to defaults, and treat an empty file as "not readable now" rather than "defaults" on the first attempts.
- **Keys and defaults used by M1 (verbatim from docs/legacy/formats.md §1.3; file L191-198 Library, L59-67 General, L90-131 Engine, L132-165 Backgroundog):**
  - `Library.json`: `Location` string default `%APPDATA%\Sucrose\Library`; `Selected` string default `""` (25-char folder name).
  - `General.json`: `AppVisible` bool default `true`.
  - `Engine.json`: `WallpaperVolume` int 100 (clamp 0..100); `WallpaperLoop` bool `true`; `StretchType` `UniformToFill` (values `None`, `Fill`, `Uniform`, `UniformToFill`; out of range falls back to default); `HardwareAcceleration` bool `true`; `ScreenType` `DisplayBound` (values `DisplayBound`, `WorkingArea`).
  - `Backgroundog.json`: `FullScreenPerformance` `Resume` (values `Close`, `Pause`, `Resume`; `Resume` disables the rule); `PausePerformanceType` `Light` (or `Heavy`).
- **Observed on the maintainer's machine (shape only):** `Library.json` 5 keys, `General.json` 7, `Engine.json` 30, `Backgroundog.json` 30; `Engine.json` has `Video: "MpvPlayerLive"`, `StretchType: "UniformToFill"`, `HardwareAcceleration: true`, `ScreenType: "DisplayBound"`, `DisplayScreenType: "PerDisplay"`, `CrashExplorer: true` (**CONTEXT says the default is false; here it is true, ignored per D-11**), `StayAwake: true` (**not in the D-09 list**, see Open Question 6); `Backgroundog.json` has no `FullScreenPerformance` key and `PausePerformanceType: "Light"`; `CommunicationType: "Pipe"`.
- **Theme (`SucroseInfo.json`) fields [VERIFIED: docs/legacy/formats.md §3 L300-320; shape confirmed on the maintainer's theme]:** `AppVersion` (Version, always), `Version`, `Title`, `Thumbnail`, `Preview`, `Description`, `Author`, `License` (must be present, **may be null**), `Contact`, `Type` (int: 0 Gif, 1 Url, 2 Web, 3 Video, 4 YouTube, 5 Application; reading accepts an int or a name), `Source` (relative file or URL), `Tags` (present, may be null), `Arguments` (present, may be null), `Localization` (optional, `{ "<LANG>": {Title, Description} }`). The maintainer's theme: `Type: 3`, `AppVersion: "26.10.19.0"`, `Version: "1.0.0.0"`, `Source` an `.mp4` in the theme folder, `License: null`, `Arguments: null`, no `Localization`, file starts `7b0d0a` (CRLF, no BOM).
- **Version type:** `VersionConverter` reads a string such as `"1.0.1.1"` or an object `{Major, Minor, Build, Revision}` and writes a string, with ".NET `System.Version` semantics: missing components become -1" [CITED: formats.md L320-321]. Implement `LegacyVersion { major, minor, build, revision: i32 }` with missing = -1 and ordinary lexicographic compare, and write it back as the original four-part string (round-trip).
- **Round trip (FMT-02 "written compatibly"):** model the struct in the legacy field order, `#[serde(flatten)] extra: serde_json::Map` to preserve unknown fields (D-007 allows new optional fields), write with 2-space indent and **CRLF**, enums as ints, `Version` as string, explicit `null` for the three nullable fields. Test: parse → serialize → parse equals, and byte-for-byte for a normalized fixture.
- **Selected-theme gate (D-13):** `Library.Location` + `Selected` must exist; `SucroseInfo.json` must parse; `Type == 3`; `AppVersion ≤ app version` (using `LegacyVersion`); `Source` must resolve to an **existing regular file inside the theme folder** (reject URLs, absolute paths, `..` components, UNC and device paths; see Security). Each failure produces a distinct, user-readable reason string for the log and tray tooltip.

### Pattern 8: IPC and single instance (D-09, D-21, D-23)

- **Endpoint name:** machine-global named pipe, so include the user: `Sucrose.Service.<user-SID-or-session>` (legacy mutexes were session-local with no `Global\` prefix [CITED: formats.md §2 mutexes]); a per-user name prevents two Windows users on one machine (fast user switching) from colliding.
- **Single instance:** create the listener first. `interprocess` sets `FILE_FLAG_FIRST_PIPE_INSTANCE` on the first instance of a named pipe [VERIFIED: create_instance.rs `open_mode(first, ...)`], so a second listener on the same name fails; map that error (the exact `io::ErrorKind` on Windows is **not verified**; test it) to "already running: log and exit 0" (D-23). If the listener is up but a stale endpoint is not possible on Windows (pipes vanish with the process), no stale-file cleanup is needed.
- **Security:** set a security descriptor that grants access only to the current user via `ListenerOptionsExt::security_descriptor(SecurityDescriptor)` [VERIFIED: interprocess src/os/windows/local_socket.rs]; the default pipe DACL allows other local users to connect (Windows default, [ASSUMED]).
- **Framing:** `postcard` is not self-delimiting on a stream: use a `u32` little-endian length prefix + `postcard::to_stdvec` payload, enforce a maximum frame size (for example 64 KiB) before allocating, and drop a connection that sends garbage. Add a `protocol_version: u16` in `Hello` so Phase 3 can extend (new enum variants at the **end**, since postcard encodes variant indices).
- **Messages (M1):** service→renderer: `Assign{monitor, role, source_path, settings}`, `SetRect`, `Pause(bool)`, `SetVolume(u8)`, `Mute(bool)`, `Clock{...}` (relayed), `Close`; renderer→service: `Hello{version,pid,monitor}`, `Clock{...}` (master only), `Opened{duration_s}`, `Failed{reason}`, `SurfaceLost`.
- **No async runtime (D-012):** use blocking `Stream::read_exact` on a dedicated thread per connection; a handful of renderers do not justify more.

### Pattern 9: `cargo xtask fetch-engines` and `measure`

- **fetch-engines:** reads `xtask/engines.toml` (`name`, `arch`, `url` (mirror), `sha256`, `upstream_tag`, `upstream_commit`, `license`), downloads to `target/engines/<arch>/`, verifies SHA256 **before** extracting, extracts only `libmpv-2.dll` and `include/mpv/client.h`, writes a small `.verified` marker, and copies `libmpv-2.dll` next to the built exe (`target/<profile>/` or `target/<triple>/<profile>/` for explicit targets). Support `--arch` and an offline `--from <archive>`. Never write inside the repo's tracked tree (`target/` is ignored).
- **bindgen task:** `cargo xtask bindgen-mpv` runs `bindgen` with `dynamic_library_name("Mpv")` and an allow-list (`mpv_create`, `mpv_initialize`, `mpv_terminate_destroy`, `mpv_set_option_string`, `mpv_set_property_string`, `mpv_get_property`, `mpv_command`, `mpv_observe_property`, `mpv_wait_event`, `mpv_wakeup`, `mpv_set_wakeup_callback`, `mpv_load_config_file`, `mpv_client_api_version`, `mpv_error_string`, `mpv_free`, `mpv_request_log_messages`, and the event/format enums and `mpv_event*` structs), writes `crates/sucrose-engine-mpv/src/sys.rs` and the header version it came from; the generated file is **committed** so CI and developers need no libclang (Pitfall 9). All these symbols exist in the shipped `client.h` [VERIFIED: `grep MPV_EXPORT` of client.h].
- **measure (D-26):** processes by exe name (C#: `Sucrose.Launcher.exe`, `Sucrose.Live.MpvPlayer.exe`, `Sucrose.Backgroundog.exe`; Rust: `sucrose.exe` for the service **and** every renderer, which share one image name, so identify renderers by the `--render` command-line argument). Per process: working set and private bytes (`sysinfo` `memory()`/`virtual_memory()`; Windows `PagefileUsage` ≈ private bytes [ASSUMED]), CPU via two `refresh` samples at 1 s over 300 s after a 60 s warm-up (`sysinfo` CPU is per-core-percent; divide by core count for "% of machine" and say which you print), GPU % and GPU memory from PDH `\GPU Engine(*)\Utilization Percentage` and `\GPU Process Memory(*)\Dedicated Usage` filtered by `pid_<pid>_` instance prefix (**both counter sets exist on the maintainer's machine** [VERIFIED: `typeperf -q`]); report GPU as max across engine types per process like Task Manager, and sum across processes. Print the exact `docs/MEASUREMENTS.md` row format (Date | Scenario | C# app | Rust | Notes) with theme video properties in Notes.
- **Video under test (D-24):** the maintainer's selected theme video is **HEVC 3840x2160, 30.04 fps, ~12.25 Mbps, AAC 2ch 48 kHz, ≈ 40 MB** [VERIFIED: `mpv --msg-level=cplayer=v --frames=1` on the file]. 4K HEVC makes hardware decode (`d3d11va` is listed for `hevc` by `--hwdec=help`) essential and makes "N monitors = N decodes" visible; keep the deferred "decode once" idea tied to the Rust-all-monitors row.

### Anti-Patterns to Avoid

- **Copying the legacy `MpvPlayer.config` verbatim:** it contains options the current mpv rejects (`v-sync`) or ignores; derive and test the set.
- **Polling forever:** a permanent 1 s timer or process scan "just in case" violates D-008; every timer must exist only while its feature is enabled.
- **Trusting `SetParent`'s return value or the build number:** classify by window tree and verify after attach.
- **Restarting on every error (legacy `RestartLive` loops):** bounded restart budget (D-22).
- **Loading libmpv in the service:** only renderers map it (D-17); the service must stay near idle.
- **Using `libmpv2`/`libmpv-sys` or linking `libmpv.dll.a` at build time:** forbidden (D-013, `.claude/rules/cargo.md`); the `.dll.a` import library in the archive must not be used.
- **Searching the C# install's `Requirements` folder for libmpv:** explicitly excluded (D-18).
- **Writing to the legacy settings folder:** read-only in M1 (D-08).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Win32/COM declarations | Hand-written `extern "system"` blocks | `windows` crate | Already mandated; correct signatures per target (x86/x64/ARM64) |
| Dynamic library loading | Raw `LoadLibrary/GetProcAddress` | `libloading` (+ bindgen's dynamic struct) | Error handling, `Library` lifetime, Windows flags via `os::windows::Library::load_with_flags` |
| C header → Rust | Hand-copied struct layouts and enums | `bindgen` via xtask, committed output | Event/format enum values must match the pinned header exactly |
| Local sockets, named-pipe ACL, single instance | `CreateNamedPipeW` plumbing, a mutex trick | `interprocess` (first-instance flag, `security_descriptor`) | D-009; edge cases around instance limits and overlapped I/O |
| Message serialization | Custom byte layouts | `serde` + `postcard` with a length prefix | D-009; versioning by appending variants |
| Tray icon, menu, re-registration | `Shell_NotifyIconW` + menu code | `tray-icon` (menu via its `muda` dependency, default features off) | Handles `TaskbarCreated`, DPI, dark mode; revisit only if measurements say so |
| Archive extraction and hashing in xtask | Shelling out to `7z`/`certutil` | `sevenz-rust2`, `sha2` | Same on every host (D-014) |
| Process CPU/RAM sampling | Own `GetProcessTimes` loops | `sysinfo` (+ PDH only for GPU) | Handles refresh intervals and per-process lifetime |
| JSON legacy parsing | A hand-written tolerant parser | `serde_json::Value` + small typed getters | Duplicate-brace cleanup (`{{`/`}}`) existed in legacy only for self-inflicted corruption; treat unparseable as defaults |
| Hidden-window message pump for tray and events | A second UI toolkit | One `sucrose-platform` message-window helper | Both tray and topology events need one loop on the main thread (D-012) |

**Key insight:** the hard parts of this phase are *shell behavior* and *process choreography*, not libraries. Everything that has a standard crate should use it, so that all review and measurement effort goes into the surface attach, sync, and supervision logic.

## Common Pitfalls

### Pitfall 1: The surface is "attached" but not visible on Windows 11 24H2+/26H2
**What goes wrong:** `SetParent` succeeds, logs say "Playing", the desktop shows the system wallpaper.
**Why it happens:** the raised-desktop tree composites differently; reports say a plain child is never composited and a `WS_EX_LAYERED` child under DefView is needed [CITED: Lively #2074, Intonaco PR #5].
**How to avoid:** classify layout, implement strategies A and B, verify coverage/visibility after attach, log OS build, layout, Progman child z-order and final styles (Intonaco's debug-log idea), test on build 26300 first.
**Warning signs:** `IsWindowVisible` true but a screen capture shows no video; works after enabling HDR (reported workaround, not a fix).

### Pitfall 2: mpv dies silently after explorer.exe restarts
**What goes wrong:** the wallpaper vanishes after a shell crash and never returns.
**Why it happens:** parent window destroyed ⇒ mpv's window destroyed ⇒ mpv queues a close [VERIFIED: w32_common.c L1489-1494]; the legacy 5 s `CrashExplorer` timer was needed because nothing listened to `TaskbarCreated`.
**How to avoid:** listen for `TaskbarCreated` in the service, wait for `Progman`/`WorkerW` with retries, respawn renderers, exempt from the crash-loop budget; remember the maintainer's `CrashExplorer` is `true` but must be ignored (D-11).
**Warning signs:** renderer exit code "surface lost" during `taskkill /f /im explorer.exe` testing.

### Pitfall 3: Pinned libmpv URL disappears
**What goes wrong:** `cargo xtask fetch-engines` starts 404-ing about a month after the pin.
**Why it happens:** upstream keeps ~30 daily releases; none since 2026-11-11.
**How to avoid:** mirror pinned archives (human action), keep SHA256 in `engines.toml`, support `--from`, keep provenance fields.

### Pitfall 4: Obsolete or invalid mpv options break behavior without an error
**What goes wrong:** options silently do not apply (config-file path ignores errors) or the handle fails to initialize (option path returns errors).
**Why it happens:** mpv removes/renames options between releases; legacy list is years old (`v-sync`, `volume-min`, `audio-pass-through` already invalid; `hwdec` docs now list `no|auto|auto-copy` although `auto-safe` still parsed).
**How to avoid:** an ignored-by-default integration test that loads the real libmpv and applies every generated option through `mpv_set_option_string`, asserting ≥ 0; run it after `fetch-engines` and whenever the pin changes.

### Pitfall 5: `LC_NUMERIC`
**What goes wrong:** `mpv_create` fails or numbers parse wrongly.
**Why it happens:** libmpv requires `LC_NUMERIC=C` [VERIFIED: client.h L147-149]; Rust never calls `setlocale`, so this holds unless a dependency does.
**How to avoid:** do not call `setlocale`; handle a `NULL` from `mpv_create` as an error.

### Pitfall 6: Two renderers for one monitor, or no renderer for a new monitor
**What goes wrong:** hot-plug storms produce duplicates; unplug leaves a surface on a dead monitor.
**How to avoid:** key renderers by monitor identity in a map owned by the service; debounce 2000 ms; apply diffs, not "restart all".

### Pitfall 7: Blocking the Win32 loop
**What goes wrong:** the tray menu or display events freeze.
**How to avoid:** the service main thread only pumps messages and drains channels; IPC reads, process spawning waits and file I/O stay on worker threads (D-012).

### Pitfall 8: DLL planting through PATH or the working directory
**What goes wrong:** a malicious `libmpv-2.dll` in a writable PATH entry or the current directory is loaded into the renderer.
**Why it happens:** D-18 allows a PATH search; plain `LoadLibrary` also searches the current directory.
**How to avoid:** load the bundled DLL by **absolute path** (`exe_dir\libmpv-2.dll`) with `libloading::os::windows::Library::load_with_flags` using `LOAD_WITH_ALTERED_SEARCH_PATH` (so its own dependencies resolve next to it); for the PATH fallback, split `PATH` yourself, skip empty and relative entries and the current directory, check the accepted names (`libmpv-2.dll`, `mpv-2.dll` [ASSUMED older name]), and load absolute paths. Check `mpv_client_api_version() >> 16 == 2` and a minimum minor (the functions used exist since 1.x; require ≥ 2.0 and compare against the build's 2.6 only for logging) before use; map failures to `EngineError::{NotFound, LoadFailed{source}, ApiTooOld{found}, WrongArchitecture}` (a wrong-arch `LoadLibrary` returns ERROR_BAD_EXE_FORMAT, 193 [ASSUMED]).

### Pitfall 9: `bindgen` needs libclang, and the i686 ABI differs
**What goes wrong:** CI or a developer without LLVM cannot build; x86 struct layouts differ from the x64-generated ones.
**Why it happens:** libclang is not installed on the maintainer's machine (no `clang` found in WSL or via `where.exe`) [VERIFIED: probe]; the pinned i686 DLL is built by MinGW GCC where `uint64_t` inside a struct is 4-byte aligned, while Rust MSVC targets align `u64` to 8 [ASSUMED].
**How to avoid:** generate **once** with the xtask and commit `sys.rs` (no build-time libclang); only touch structs through pointers the library returns (`mpv_event`, `mpv_event_property`, `mpv_event_end_file`) and read scalar fields by explicit offsets or by generating the bindings with `--target i686-pc-windows-gnu` for the x86 cfg; add a layout assertion test for the fields we read, and flag x86 as "builds and loads, manually unverified" since only x64 hardware is available.

### Pitfall 10: Linux/macOS CI breaks because of Windows-only deps
**What goes wrong:** `cargo clippy --workspace` on Linux fails (GTK headers for `tray-icon` default features, `windows` on non-Windows).
**How to avoid:** `[target.'cfg(windows)'.dependencies]` for `windows` and `tray-icon` (`default-features = false`), keep `mod os_windows` gated as today, and give `sucrose-engine-mpv` a portable loader that compiles everywhere (library names per OS can be constants; only the surface handle is Windows-specific). Run `cargo clippy --workspace --all-targets --target i686-pc-windows-msvc` locally, as CI does.

### Pitfall 11: Tests that need the maintainer's desktop
**What goes wrong:** manual-only criteria are forgotten or run against the real data.
**How to avoid:** the plan lists the human checkpoints explicitly (see Validation Architecture) and every GUI run uses `SUCROSE_DATA_DIR` + `--no-focus`-style start (no test-only flags per the maintainer's memory note: use the real library and the real selected theme).

### Pitfall 12: Single-display test machine
**What goes wrong:** "every monitor" and "plugged/unplugged" cannot be exercised; sync (D-02) is untestable end to end.
**Why:** the maintainer's machine has exactly one display (`\\.\DISPLAY1`, 1920x1080) [VERIFIED: probe].
**How to avoid:** unit-test the topology diff and sync controller with synthetic snapshots; schedule a human check with a second display or a virtual display driver (Open Question 4); say clearly in VERIFICATION which criteria were only partially verifiable.

## Code Examples

### Loading libmpv safely (renderer only)
```rust
// Source: pattern from libloading docs + docs of client.h; names verified in the shipped client.h
use libloading::{Library, Symbol};

pub struct MpvLib { lib: Library /* + resolved fn pointers from the bindgen-generated struct */ }

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("libmpv was not found next to sucrose.exe or on PATH (looked for {names:?})")]
    NotFound { names: &'static [&'static str] },
    #[error("libmpv could not be loaded: {0}")]
    LoadFailed(#[from] libloading::Error),
    #[error("libmpv client API {found:#x} is older than required {required:#x}")]
    ApiTooOld { found: u64, required: u64 },
}

pub const ACCEPTED_NAMES: &[&str] = &["libmpv-2.dll", "mpv-2.dll"];

pub fn load_from(path: &std::path::Path) -> Result<Library, EngineError> {
    // SAFETY: loading a DLL runs its DllMain; the path is an absolute path we resolved ourselves
    // (exe dir first, then PATH entries we filtered), never a bare name.
    #[cfg(windows)]
    let lib = unsafe {
        libloading::os::windows::Library::load_with_flags(
            path,
            libloading::os::windows::LOAD_WITH_ALTERED_SEARCH_PATH,
        )
    }
    .map(Library::from)?;
    #[cfg(not(windows))]
    let lib = unsafe { Library::new(path) }?;
    Ok(lib)
}
```

### Creating the instance (option order that mpv documents)
```rust
// Source: mpv client.h (mpv_create / mpv_set_option_string / mpv_load_config_file / mpv_initialize)
// All calls go through the bindgen struct; strings are NUL-terminated CStrings.
fn open(api: &Mpv, hwnd_u32: u32, opts: &[(&str, String)], user_cfg: Option<&Path>, file: &Path)
    -> Result<*mut mpv_handle, EngineError>
{
    // SAFETY: mpv_create has no preconditions beyond LC_NUMERIC == "C" (never changed by this program).
    let h = unsafe { api.mpv_create() };
    if h.is_null() { return Err(EngineError::CreateFailed); }
    set(api, h, "wid", &hwnd_u32.to_string())?;           // decimal, low 32 bits of the HWND
    for (k, v) in opts { set(api, h, k, v)?; }            // generated set; errors are fatal here (known-good names)
    if let Some(p) = user_cfg { load_config(api, h, p); } // D-10: errors are ignored by mpv itself; log the code
    check(unsafe { api.mpv_initialize(h) })?;
    // observe time-pos / pause; then: mpv_command(["loadfile", <abs path>, "replace"])
    Ok(h)
}
```

### Fullscreen predicate as a pure function
```rust
// Source: legacy rule, docs/legacy/services.md §1.1 (foreground covers a monitor rect; skip when desktop focused)
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
pub struct Foreground<'a> { pub rect: Rect, pub class: &'a str, pub is_own_or_shell: bool, pub cloaked: bool, pub minimized: bool }

pub fn is_fullscreen(fg: Option<&Foreground<'_>>, monitors: &[Rect]) -> bool {
    let Some(fg) = fg else { return false };
    if fg.is_own_or_shell || fg.cloaked || fg.minimized { return false; }
    if matches!(fg.class, "Progman" | "WorkerW" | "SHELLDLL_DefView") { return false; } // desktop focused
    monitors.iter().any(|m| fg.rect.left <= m.left && fg.rect.top <= m.top
        && fg.rect.right >= m.right && fg.rect.bottom >= m.bottom)
}
```

### Debounce (5 to trigger, 3 to recover)
```rust
pub struct Debounce { trigger: u8, recover: u8, on: bool, run: u8 }
impl Debounce {
    pub fn new(trigger: u8, recover: u8) -> Self { Self { trigger, recover, on: false, run: 0 } }
    /// Feed one 1 s sample; returns Some(new_state) only when the state flips.
    pub fn feed(&mut self, active: bool) -> Option<bool> {
        if active == self.on { self.run = 0; return None; }
        self.run += 1;
        let need = if self.on { self.recover } else { self.trigger };
        if self.run >= need { self.on = active; self.run = 0; Some(self.on) } else { None }
    }
}
```

### Length-prefixed postcard frame
```rust
// Source: postcard docs (to_stdvec / from_bytes); framing is ours because postcard is not self-delimiting
pub const MAX_FRAME: u32 = 64 * 1024;
pub fn write_frame<W: std::io::Write, T: serde::Serialize>(w: &mut W, msg: &T) -> std::io::Result<()> {
    let body = postcard::to_stdvec(msg).map_err(std::io::Error::other)?;
    w.write_all(&(body.len() as u32).to_le_bytes())?;
    w.write_all(&body)
}
pub fn read_frame<R: std::io::Read, T: serde::de::DeserializeOwned>(r: &mut R) -> std::io::Result<T> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let n = u32::from_le_bytes(len);
    if n > MAX_FRAME { return Err(std::io::Error::other("frame too large")); }
    let mut body = vec![0u8; n as usize];
    r.read_exact(&mut body)?;
    postcard::from_bytes(&body).map_err(std::io::Error::other)
}
```

### Single-instance listener (shape; error kind to be confirmed by a test)
```rust
// Source: interprocess 2.4.9 local_socket::ListenerOptions + os::windows::local_socket::ListenerOptionsExt
use interprocess::local_socket::{ListenerOptions, GenericNamespaced, ToNsName, traits::Listener as _};
#[cfg(windows)]
use interprocess::os::windows::local_socket::ListenerOptionsExt;
let name = format!("Sucrose.Service.{user}").to_ns_name::<GenericNamespaced>()?;
let opts = ListenerOptions::new().name(name);
#[cfg(windows)]
let opts = opts.security_descriptor(current_user_only_sd()?); // SDDL string built from the current user SID
match opts.create_sync() {
    Ok(listener) => { /* serve */ }
    Err(e) => { tracing::info!(error = %e, "another instance is running"); return Ok(()); }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WorkerW is a top-level sibling of Progman after `0x052C` | "Raised desktop": `SHELLDLL_DefView` (layered) and a wallpaper `WorkerW` are children of `Progman`, which has `WS_EX_NOREDIRECTIONBITMAP` | Windows 11 24H2 (build 26100) and later; present on 26300 | Classic attach finds the wrong parent or is not composited; use layout classification |
| `mpv --hwdec=auto-safe` | docs list `no`, `auto`, `auto-copy` and named APIs; `auto-safe` still accepted | mpv ≥ 0.40-ish [ASSUMED] | Verify at pin time; test applies the option |
| mpv only embeddable by `wid` or OpenGL render API | **D3D11 render API** (`MPV_RENDER_API_TYPE_D3D11`, `render_d3d11.h`) | in mpv v0.42.0 source | Future option for owning the swapchain and sharing frames across monitors |
| libmpv from random mirrors | pinned, hash-verified, mirrored builds from one source for x86/x64/ARM64 | D-028 | Reproducible dev runs and packages |
| C# per-second timers incl. `GC.Collect()` | Event-driven; timers only while a feature is on | D-008 | Idle cost near zero |

**Deprecated/outdated:**
- Legacy mpv config keys `v-sync`, `volume-min`, `audio-pass-through`, flag-form `no-idle`: not valid in mpv v0.42.0-38.
- The `Surface.Correct()` child-resize hack: mpv itself tracks the parent size when embedded.
- `CrashExplorer` polling: replaced by `TaskbarCreated` (D-11).

## Suggested Plan Slicing (for the planner)

Ordering is driven by risk and by what can be verified without the desktop. Each slice lists its requirement coverage.

1. **Foundations (docs + deps + xtask pins):** D-028 entry and the doc edits listed in D-16; version-scheme decision entry and `Cargo.toml` version; workspace dependency additions; `xtask/engines.toml`, `fetch-engines`, `bindgen-mpv` (committed `sys.rs`); roadmap wording change for criterion 3 (D-03). *Human actions:* mirror the three pinned archives; confirm version form. (ENG-01 delivery)
2. **Desktop surface spike (`sucrose-platform` Windows):** layout classifier (pure, with fixture snapshots of the tree above), strategies A and B, monitors enumeration with DPI, PMv2, hidden event window (`WM_DISPLAYCHANGE`, `TaskbarCreated`), a `human-verify` checkpoint on build 26300 with a mpv child. (DESK-01, DESK-06)
3. **Formats and legacy reader (pure, fixtures):** `LegacyVersion`, `SucroseInfo` (read/write round-trip), `sucrose-config` legacy reader, selected-theme resolution and gate, Source containment. (FMT-02, FMT-08, TYPE-01 gate)
4. **IPC (pure + local socket tests):** protocol enums, framing, endpoint name, ACL, single instance. (D-21, D-23)
5. **Engine + mpv (`sucrose-engine`, `sucrose-engine-mpv`):** trait, loader with search order, generated bindings, config generation + user-config override, player wrapper (open, pause, volume, loop, stretch, observe time-pos), sync controller (pure), option-validity integration test (ignored by default). (ENG-01)
6. **Service and renderer integration (`apps/sucrose`):** mode switch, supervisor + budget, job object, tray (Quit, tooltip reasons D-13/D-20), fullscreen timer + 5/3 state machine, topology diff, master election, stop cleanup. (DESK-07, PERF-05, TRAY-01, D-01/D-02/D-22)
7. **Measurement and docs close-out:** `cargo xtask measure`, MEAS-01 runs (release), `docs/MEASUREMENTS.md` rows (single-monitor like-for-like + Rust all-monitors), `docs/FEATURES.md` rows (Video, surface, DPI, monitor/shell recovery, fullscreen, tray, `SucroseInfo.json`, Architectures/libmpv), `docs/ARCHITECTURE.md` status and crate list. (MEAS-01)

Slices 3, 4 and the pure parts of 2, 5, 6 can run in parallel waves; the spike (2) should land before 6.

### Version scheme proposal (D-12; maintainer confirmation required, one-way door)

Facts: the C# version is the build date (`<Version>$([System.DateTime]::Today.ToString(yy)).$([System.DateTime]::Today.ToString(MM)).$([System.DateTime]::Today.ToString(dd))</Version>`, `Directory.Build.props` L25) rendered as four parts with a trailing `.0`; the latest C# release is tag `v26.11.22.0` published 2026-11-21T22:20:32Z [VERIFIED: `gh api repos/Taiizor/Sucrose/releases/latest`]. Cargo versions must be three numeric parts without leading zeros (`26.09.05` is invalid), so the date parts are written `YY.M.D`.
Proposal: `version = "26.11.22+1"` (semver build metadata carries the legacy fourth component). Our `LegacyVersion::from_cargo` maps `major.minor.patch+rev` to `(major, minor, patch, rev)` = `26.11.22.1` which is **greater** than `26.11.22.0` (c), compares correctly with theme `AppVersion` values such as `26.10.19.0` (b), is valid Cargo semver (a), and maps directly onto the four-part Windows file version. Same-day rebuilds bump `+rev`; later dates bump `D`/`M`/`Y`. Cargo ignores build metadata when resolving, which is irrelevant for `publish = false` crates. Whether `CARGO_PKG_VERSION` includes the `+rev` suffix must be confirmed by a one-line test (`sucrose_core::VERSION` currently reads it with `env!("CARGO_PKG_VERSION")`, `crates/sucrose-core/src/lib.rs`). Any change must also keep the installer/updater comparison in mind (Phase 5).

## Runtime State Inventory

Not applicable: this phase is not a rename/refactor/migration. (The version change touches `Cargo.toml` and `sucrose_core::VERSION` only; nothing stores the old `0.1.0` at runtime.)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Strategy A (child of the `Progman`'s `WorkerW`) or B (layered child of `Progman` under DefView) will composite on build 26300.9974; the right one is found by the spike | Pattern 1 | Wallpaper invisible; schedule the spike first |
| A2 | `SetWindowPos(surface, DefView, ...)` with `hWndInsertAfter = DefView` puts the surface directly under DefView and above the system `WorkerW` | Pattern 1 | Wrong z-order; icons or wallpaper hidden |
| A3 | A PMv2 renderer parented into the shell's tree gets correct physical-pixel coordinates and no bitmap scaling, including at non-100% scale | Pattern 4 | Blurry or mis-sized video |
| A4 | `QueryPerformanceCounter` is comparable across processes on one machine | Pattern 3 | Sync offsets; use wall-clock or a service-issued epoch instead |
| A5 | Starting drift thresholds (33 ms / 500 ms, ±2 % speed) give "no visible offset" | Pattern 3 | Visible drift or seek stutter; tune in the spike |
| A6 | Job object with `KILL_ON_JOB_CLOSE` reliably ends renderers when the service dies | Pattern 6 | Orphan renderers painting the desktop |
| A7 | `interprocess` reports "pipe already exists" as an `io::Error` we can map to "second instance" | Pattern 8 | Second instance runs twice; fix with an explicit check |
| A8 | Default named-pipe DACL lets other local users connect | Pattern 8 / Security | If wrong the ACL is harmless but unnecessary |
| A9 | Hardening option names (`osc`, `input-default-bindings`, `input-vo-keyboard`, `input-cursor`, `cursor-autohide`, `border`, `load-scripts`, `ytdl`, `terminal`, `config`, `gpu-shader-cache-dir`) are valid for this mpv | Pattern 2 | Option errors; the option-validity test catches it (the names `osc`, `input-default-bindings`, `input-vo-keyboard`, `input-cursor`, `cursor-autohide`, `border`, `load-scripts`, `ytdl`, `config`, `terminal` were seen in `--list-options`; `gpu-shader-cache-dir` was not checked) |
| A10 | Windows `PagefileUsage` ≈ private bytes for `sysinfo`'s `virtual_memory()` | Pattern 9 | Mislabelled column in MEASUREMENTS |
| A11 | GPL build of libmpv + FFmpeg `--enable-gpl` is compatible with GPL-3.0-only distribution (all parts GPLv2-or-later or more permissive) | Standard Stack | Licensing problem for packages; needs maintainer/legal confirmation and a source-offer note (shinchiro recipe tag + commit) |
| A12 | `x86_64-v3` builds need AVX2-class CPUs | Standard Stack | None if the plain x86_64 build is used anyway |
| A13 | MinGW i686 `uint64_t` struct alignment differs from MSVC | Pitfall 9 | x86 event decoding wrong; mitigated by pointer-only access + layout test |
| A14 | `postcard` feature name `use-std` | Standard Stack | Compile error; trivial fix |
| A15 | `SetProcessDpiAwarenessContext` called first in `main` applies to threads libmpv creates later | Pattern 4 | mpv child window not PMv2; prefer the manifest |
| A16 | `CARGO_PKG_VERSION` includes build metadata (`+1`) | Version proposal | Parse from `Cargo.toml` at build time instead |
| A17 | Tooling note: the old name `mpv-2.dll` is a valid fallback name | Pitfall 8 | Harmless; drop if unused |
| A18 | Strip-BOM-then-parse is enough for all legacy files; the `{{`/`}}` cleaning in legacy is not needed on read | Pattern 7 | A corrupted legacy file reads as defaults (matches legacy "any exception returns default") |
| A19 | `hwdec-codecs=all` is needed (or not) for the maintainer's 4K HEVC under `d3d11va` | Pattern 2 | Software decode of 4K HEVC (high CPU); measure with both |
| A20 | The legacy `RefreshDesktop` behavior can be reproduced by destroying the surface plus an `InvalidateRect`/wallpaper refresh | Pattern 6 | Last frame stays; test and adjust |

## Open Questions (RESOLVED)

Every question below carries its resolution: decided by CONTEXT.md, settled by a named checkpoint or check in a plan, or explicitly unknown with its execution-time handling. Plan numbers refer to the revised set 02-01 … 02-15.

1. **Which surface strategy composites on this build (A or B)?**
   - What we know: layout probed (raised desktop); mpv and the C# app use the `Progman`-child `WorkerW` path; Microsoft guidance (via Lively) and Intonaco use a layered child under DefView; Intonaco says the plain child is not composited on 26H2.
   - What's unclear: whether the C# app currently renders its wallpaper on this machine (that would prove A) and what DWM does with an mpv D3D11 swapchain child inside a layered window.
   - Recommendation: spike first with both strategies; ask the maintainer to say whether the C# wallpaper is visible on this machine today; keep strategy selection behind the layout classifier.
   - **Resolution (RESOLVED: settled at execution by named checks):** strategy selection stays behind the layout classifier (plan 06 Task 2). For a raised desktop, strategy B is tried first and A is the fallback (plan 06 Task 1). The plan 06 Task 1 human-check (black surface) and the plan 07 Task 1 human-check (video, which may composite differently from a GDI surface) decide which strategy DWM shows on 26300.9974; the executor swaps the order if needed and records the working strategy in the 02-06 and 02-07 SUMMARYs. Whether the C# wallpaper shows on this machine today is no longer needed, because both strategies are tested directly.
2. **Where do the pinned libmpv archives live?** Upstream keeps ~30 releases. Recommendation: the maintainer uploads the three archives as release assets of a Sucrose-controlled repository (not git); the plan includes a `human-action` checkpoint and `engines.toml` points at the mirror (provenance fields keep the upstream tag).
   - **Resolution (RESOLVED: maintainer checkpoint):** plan 02 Task 1 (`checkpoint:decision`, blocking) asks the maintainer to choose `mirror-release` or `upstream-now`; plan 02 Task 2 applies the answer to `xtask/engines.toml` and records it in D-028. The SHA256 pins never change either way, and `--from <archive>` works offline.
3. **Version form (D-12):** confirm `26.11.22+1` style (Version proposal above) before it lands in `Cargo.toml`; it is a one-way door.
   - **Resolution (RESOLVED: maintainer checkpoint, one-way door):** plan 08 Task 2 (`checkpoint:decision`) asks the maintainer for the exact form; the planner does not choose it. Plan 08 Task 3 then sets `[workspace.package] version`, adds the AppVersion rule and records D-029. Until then `Cargo.toml` keeps `0.1.0`.
4. **How to verify multi-monitor, hot-plug and sync on a one-display machine?** Needs a second physical display or a virtual display driver for the human checkpoint; otherwise ship with the sync controller and topology diff unit-tested and say so in VERIFICATION. Criterion 1 ("every monitor") is then verified for one monitor only.
   - **Resolution (RESOLVED: execution-time handling):** the topology diff, display debounce, shell-restart and sync logic are unit-tested with synthetic multi-monitor input (`cargo test -p sucrose topology`, `debounce`, `shell_restart`, `elect_master`; `cargo test -p sucrose-engine-mpv sync` in plans 12 and 14). The hot-plug and two-monitor human-checks run only if a second or virtual display is available (plan 12 Task 2, plan 14 Task 2) and otherwise record "not verifiable: one display". The 02-12 SUMMARY and the "Verification scope" section of the 02-15 SUMMARY (plan 15 Task 3) state that criteria 1 and 2 were verified on hardware for one monitor only; `/gsd-verify-work` carries that into VERIFICATION.md.
5. **Does `libmpv-2.dll` load on a machine without a Vulkan runtime?** `vulkan-1.dll` is in the import list; if it is a hard import and absent, `LoadLibrary` fails and the "libmpv cannot be loaded" path (D-20) triggers. Verify in Wave 0 by loading the DLL with `load_with_flags` on a clean VM or by inspecting the delay-import directory.
   - **Resolution (explicitly unknown; handled at execution):** plan 01 Task 3 records with `objdump -p` whether `vulkan-1.dll` is a normal or a delay-load import, and plan 02 Task 2 writes the result into D-028. No machine without a Vulkan loader is available, so loading there stays unverified in Phase 2. If it fails on such a machine, the loader returns `LoadError::LoadFailed`, the renderer exits with code 2 (plan 07) and the tray tooltip shows the D-20 reason with a repair hint (plan 09); nothing panics.
6. **`StayAwake` (legacy):** the maintainer's `Engine.json` has `StayAwake: true` and the legacy mpv config generator maps it to `stop-screensaver=always`, but D-09's key list omits it. M1 as written uses `stop-screensaver=no`. Decide: leave out and note in FEATURES.md, or add the key to the read list (it also changes whether the PC can sleep while the wallpaper plays). Recommendation: leave out in M1 and record the difference.
   - **Resolution (RESOLVED: decided by CONTEXT.md D-09):** the locked D-09 key list does not include `StayAwake`, so M1 does not read it and runs mpv with `stop-screensaver=no` (plans 07 and 11). Plan 15 Task 3 records the difference in `docs/FEATURES.md`; the setting returns with the settings work of Phase 3/4.
7. **GPL compatibility and source offer for the bundled libmpv:** record `GPL` build + `mpv-winbuild-cmake` tag/commit in D-028 and ship license texts with the package (Phase 5). Needs maintainer sign-off (A11). The `lgpl` zhongfly builds (x86_64, aarch64 only) are the fallback.
   - **Resolution (RESOLVED: maintainer checkpoint):** plan 02 Task 1 asks for the GPL sign-off ("GPL: yes" or "GPL: no"; "no" stops the phase because ENG-01 then needs a new delivery decision). Plan 02 Task 2 records the license, the recipe tag and the commit (source-offer coordinates) in D-028; license texts in the package are Phase 5 packaging. The LGPL builds are not offered because they have no i686 build (D-027).
8. **When Pause is verified (criterion 3), who edits `Backgroundog.json`?** CLAUDE.md treats it as read-only; D-03 requires `FullScreenPerformance=Pause`. Make it an explicit maintainer action in the plan, and the reverse edit afterward.
   - **Resolution (RESOLVED: maintainer action in a named check):** in the plan 13 Task 2 human-check the maintainer backs up `Backgroundog.json`, adds `"FullScreenPerformance": "Pause"`, runs the check and restores the backup. Sucrose and the executor never edit legacy files (D-08).
9. **Stop cleanup and Quit semantics:** does destroying the surface restore the system wallpaper without a refresh call on the raised desktop? Check in the spike; add `RefreshDesktop` only if the last frame lingers.
   - **Resolution (RESOLVED: settled at execution):** the plan 06 Task 1 and plan 07 Task 1 human-checks note whether the surface or the last video frame stays visible after stop. Plan 09 Task 2 adds `refresh_desktop()` only if one of those SUMMARYs records it, and otherwise says so in its SUMMARY.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (Windows, via wrapper) | everything | ✓ | cargo 1.99.0, rustc 1.99.0 | — |
| Windows 11 target machine | surface, mpv, measure | ✓ | 10.0.26300.9974, one display 1920x1080 | second/virtual display for multi-monitor checks |
| libmpv-2.dll (x64/x86/ARM64) | ENG-01 | ✗ (to be fetched by xtask) | pinned 20261111 (mpv v0.42.0-38) | none; D-20 path when absent |
| libclang / LLVM | `bindgen-mpv` task only | ✗ (no `clang` in WSL or on the Windows PATH) | — | commit generated `sys.rs`; install LLVM only for regeneration |
| GitHub release download (network) | `fetch-engines` | ✓ | — | `--from <archive>` |
| `7z` (Linux), `gh`, `curl` | research only (not used by xtask) | ✓ | — | — |
| `typeperf` / PDH GPU counters | MEAS-01 GPU % | ✓ | `\GPU Engine(*)\Utilization Percentage`, `\GPU Process Memory(*)\Dedicated Usage` present | report CPU/RAM only |
| C# Sucrose install (legacy settings, selected theme) | FMT-08, D-08, MEAS-01 C# side | ✓ | settings files present (Library 5 keys, General 7, Engine 30, Backgroundog 30) | fixtures only for CI |
| Windows ARM64 / x86 hardware | architecture verification | ✗ | CI builds/tests only (`windows-11-arm`, i686 on `windows-latest`) | mark "builds, manually unverified" |
| Docker (Linux builds) | not needed this phase | not probed | — | — |

**Missing dependencies with no fallback:** none blocking planning; libmpv arrives through slice 1.
**Missing dependencies with fallback:** libclang (commit generated bindings); second display (unit-test topology logic, human check later).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` (no external framework); integration tests under each crate's `tests/`; fixtures under `tests/fixtures/` |
| Config file | none beyond workspace `Cargo.toml` / `[workspace.lints]` |
| Quick run command | `cargo test -p <crate>` (for example `cargo test -p sucrose-config`) |
| Full suite command | `cargo xtask ci` (fmt check, clippy `-D warnings`, `cargo test --workspace`) |
| Engine-dependent tests | `cargo xtask fetch-engines` then `cargo test -p sucrose-engine-mpv -- --ignored` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FMT-02 | `SucroseInfo.json` parse + round-trip incl. null fields, int/name `Type`, string/object `Version`, CRLF/indent, unknown-field preservation | unit/fixture | `cargo test -p sucrose-core theme` | ❌ Wave 0 |
| FMT-02 | `LegacyVersion` compare incl. missing = -1, `26.11.22+1` vs `26.11.22.0` | unit | `cargo test -p sucrose-core version` | ❌ Wave 0 |
| FMT-08 | Legacy reader: wrapper, BOM, CRLF, enum names, clamps, defaults on bad/missing keys, truncated file retry | unit/fixture | `cargo test -p sucrose-config` | ❌ Wave 0 |
| TYPE-01 | Gate: missing theme, non-Video type, newer `AppVersion`, `Source` escaping folder (`..`, absolute, UNC, URL) each give a distinct reason | unit | `cargo test -p sucrose-core gate` | ❌ Wave 0 |
| ENG-01 | Loader error mapping (not found, bad file, wrong arch if a fixture DLL exists), search order, accepted names | unit | `cargo test -p sucrose-engine-mpv loader` | ❌ Wave 0 |
| ENG-01 | Generated option set applies cleanly to real libmpv; open/pause/volume/loop on a real file | integration, ignored by default | `cargo test -p sucrose-engine-mpv -- --ignored` | ❌ Wave 0 |
| ENG-01 | Stretch and hwdec mapping, config generation, user-config override (D-10) | unit | `cargo test -p sucrose-engine-mpv config` | ❌ Wave 0 |
| DESK-01 | Layout classifier on snapshot fixtures (classic, raised, missing Progman) | unit | `cargo test -p sucrose-platform layout` | ❌ Wave 0 |
| DESK-01 | Video visible behind icons on build 26300 | **manual (human-verify)**: capture via `PrintWindow` + eyes; cannot be automated headlessly | — | — |
| DESK-06 | Rect math from monitor snapshot to parent client coordinates at 100 %/150 %/200 %; PMv2 runtime assertion logged | unit + manual sharpness check | `cargo test -p sucrose-platform rects` | ❌ Wave 0 |
| DESK-07 | Topology diff (add/remove/move/DPI), debounce, restart budget (3/60 s), shell-restart exemption | unit | `cargo test -p sucrose supervisor` | ❌ Wave 0 |
| DESK-07 | Explorer restart and hot-plug on the real machine | **manual**: `taskkill /f /im explorer.exe` then check recovery; second display needed for hot-plug | — | — |
| PERF-05 | `is_fullscreen` predicate and 5/3 debounce, desktop-focus skip | unit | `cargo test -p sucrose fullscreen` | ❌ Wave 0 |
| PERF-05 | Pause on a fullscreen game and resume (key set to `Pause`) | **manual (human-action to set the key)** | — | — |
| TRAY-01 | Quit stops service and renderers | manual; unit for the quit path (channel → shutdown order) | `cargo test -p sucrose shutdown` | ❌ Wave 0 |
| D-21/D-23 | Frame encode/decode round-trip, oversize frame rejected, single instance (second listener fails) | unit/integration | `cargo test -p sucrose-ipc` | ❌ Wave 0 |
| D-02 | Sync controller actions for drift cases incl. loop wrap | unit | `cargo test -p sucrose-engine-mpv sync` | ❌ Wave 0 |
| MEAS-01 | `cargo xtask measure` prints a row; release build; both sides measured | manual run, tool smoke test | `cargo xtask measure --help` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p <touched crate>` plus `cargo clippy -p <touched crate> --all-targets -- -D warnings` (each under 30 s for pure crates).
- **Per wave merge:** `cargo xtask ci`.
- **Phase gate:** `cargo xtask ci` green; ignored engine tests green on the maintainer's machine; human checkpoints for criteria 1–3 and 5 recorded; MEASUREMENTS rows added.

### Wave 0 Gaps
- [ ] `crates/sucrose-core/tests/fixtures/` (SucroseInfo variants from the real theme with personal data removed) — FMT-02
- [ ] `crates/sucrose-config/tests/fixtures/` (Library/General/Engine/Backgroundog from real files, sanitized; plus a BOM file and a truncated file) — FMT-08
- [ ] `crates/sucrose-platform/tests/fixtures/` (desktop-tree snapshots, including the 26300 tree above) — DESK-01
- [ ] `xtask/engines.toml` + `fetch-engines` — ENG-01 integration tests depend on it
- [ ] New crates scaffolded: `sucrose-config`, `sucrose-ipc`, `sucrose-engine`, `sucrose-engine-mpv`
- [ ] Framework install: none (cargo built-in)

## Security Domain

Security enforcement is enabled (`security_enforcement: true`, ASVS level 1, block on high).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — (no accounts) |
| V3 Session Management | no | — |
| V4 Access Control | yes | Named pipe restricted to the current user via a security descriptor; per-user pipe name |
| V5 Input Validation | yes | Typed lenient parsing (`serde_json` + getters with clamps), `Source` path containment, bounded IPC frames, mpv option values built from validated enums, never string-concatenated user text |
| V6 Cryptography | yes (hash only) | `sha2` SHA-256 for engine archive pins; no custom crypto |
| V8 Data Protection | partial | Logs avoid theme titles/personal paths beyond what is needed; no telemetry in M1 |
| V10 Malicious Code | yes | Engine archive hash verified before extraction; DLL loaded by absolute path; mirror source recorded |
| V12 Files and Resources | yes | Refuse `Source` outside the theme folder; read-only access to legacy settings; write only under `SUCROSE_DATA_DIR`/data dir |
| V14 Configuration | yes | DLL search hardening (no CWD, filtered PATH); mpv hardened options (`load-scripts=no`, `ytdl=no`, `config=no`) |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| DLL planting (`libmpv-2.dll` in CWD or writable PATH entry) | Elevation / Tampering | Absolute-path load; bundled copy first; skip relative/CWD PATH entries; log the loaded path |
| Malicious store theme: `Source` = URL, `..\..` path, UNC or device path | Tampering / Info disclosure | Resolve against the theme folder, canonicalize, require existing regular file under it; reject URLs for Video type; set `ytdl=no` and `load-scripts=no` so mpv never shells out |
| Named-pipe squatting or hijack by another local process/user | Spoofing / Tampering | First-instance flag, per-user name, current-user-only ACL, `Hello` with protocol version and the renderer's PID verified against the spawned child |
| IPC resource exhaustion (oversized or malformed frames) | DoS | `MAX_FRAME` before allocation, drop on decode error, bounded channels |
| User mpv config (`uMpvPlayer.config`) enabling scripts/IPC servers | Elevation (same user) | Same trust boundary as the user's profile; log when a user config is used (D-10); do not load it when it is not owned by the current user [ASSUMED] |
| Tampered engine download | Tampering | SHA-256 pin checked before extraction; mirror under maintainer control; no unpinned fallback |
| Orphan renderers after service crash | DoS (visual) | Job object `KILL_ON_JOB_CLOSE` |
| Renderer crash loops | DoS | Restart budget (3 per 60 s) and logged reason |

## Sources

### Primary (HIGH confidence)
- mpv v0.42.0 source and docs, fetched this session: `video/out/w32_common.c` (`get_workerw_hwnd`, `set_raised_desktop`, `gui_thread`, `resize_child_win`, `install_parent_hook`, WM_DESTROY handling), `DOCS/man/options.rst` (`--wid`, `--video-sync`, `--hwdec`), `DOCS/man/input.rst` (`seek`, `time-pos`), `include/mpv/render_d3d11.h`.
- Shipped `client.h` / `render.h` from the pinned archive (API 2.6; defaults at L438-461; `mpv_load_config_file` L585-600; locale note L147-149).
- Local verification this session: SHA256 of three dev archives and the x64 full archive against GitHub digests; PE types of the three DLLs; `strings` configuration; `mpv.com --list-options`, `--hwdec=help` and option-error probes on v0.42.0-38; `typeperf -q` GPU counters; PMv2 desktop-window enumeration on Windows 10.0.26300.9974; shape of the maintainer's legacy settings and selected theme; mpv probe of the theme video.
- GitHub API: `shinchiro/mpv-winbuild-cmake` releases (30, newest 20261111), `zhongfly/mpv-winbuild` releases (52), `mpv-player/mpv` releases (v0.42.0 on 2026-11-05), `Taiizor/Sucrose` latest release `v26.11.22.0`.
- crates.io API (versions, licenses, MSRV) and `gsd_run query package-legitimacy check`.
- tray-icon v0.26.1 `Cargo.toml` and `src/platform_impl/windows/mod.rs`; interprocess main `src/os/windows/local_socket.rs` and `named_pipe/listener/create_instance.rs`; docs.rs `ListenerOptions` page.
- Repo files read this session: `CLAUDE.md` (root and `src/`), `.claude/rules/*`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/FEATURES.md`, `docs/MEASUREMENTS.md`, `docs/legacy/engines.md`, `services.md`, `formats.md`, `Cargo.toml`, `deny.toml`, crates and `xtask` sources, `.github/workflows/rust.yml`, legacy `Sucrose.Shared.Engine/Manage/Internal.cs` (`MpvConfig` L100-149), `MpvPlayer/Helper/Config.cs`, `Helper/Video.cs`, `View/Video.xaml.cs`, `Directory.Build.props`.

### Secondary (MEDIUM confidence)
- github.com/lively-community/lively/issues/2074 (via fetch summary; quotes Microsoft's explanation of the 24H2 change and the layered-child guidance).
- github.com/77worldd/Intonaco/pull/5 (via fetch summary; strategy B on 26H2 build 26300.9711).
- WebSearch result set on 24H2 `WorkerW` behavior (AutoHotkey forum, blog posts; no official Microsoft documentation found).

### Tertiary (LOW confidence)
- Everything tagged [ASSUMED] in the Assumptions Log (Win32 semantics not re-verified this session, sync tuning values, license compatibility reading).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH, versions and hashes read from registries and verified locally; `windows` crate has had no release for 13 months (low risk).
- Architecture: MEDIUM, surface strategy needs the spike; process choreography is standard.
- Pitfalls: MEDIUM-HIGH, the mpv-side ones are verified in source and by running the pinned build; Win32 ones partly [ASSUMED].
- Sync tolerances: LOW until measured.

**Research date:** 2026-11-22
**Valid until:** 2026-12-06 for the libmpv pin source (upstream daily releases and retention change fast; mirror early) and the Windows 11 layout notes; crate versions 30 days.
