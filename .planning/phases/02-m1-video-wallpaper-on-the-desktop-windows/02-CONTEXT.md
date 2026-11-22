# Phase 2: M1 — Video wallpaper on the desktop (Windows) - Context

**Gathered:** 2026-11-22
**Status:** Ready for planning

> Numbering: `D-NN` (two digits) are decisions of this phase context. `D-0NN` (three digits) are entries in `docs/DECISIONS.md`.

<domain>
## Phase Boundary

On Windows, the video theme selected in an existing C# install (legacy `Library.json`, read-only) plays through libmpv behind the desktop icons on every monitor, sharp at each monitor's DPI. It survives explorer.exe restarts and monitors being plugged or unplugged, applies the configured fullscreen action (legacy `FullScreenPerformance`), and stops from a tray icon with Quit. RAM and CPU are measured against the C# app and recorded in `docs/MEASUREMENTS.md`.

Requirements: TYPE-01, ENG-01, DESK-01, DESK-06, DESK-07, PERF-05, TRAY-01, FMT-02, FMT-08, MEAS-01.

Pulled forward into this phase by this discussion (smallest parts only): the `sucrose-ipc` core between the service and its renderers plus single instance (D-21, D-23; the rest of IPC stays in Phase 3), the decision that engine binaries ship inside the Sucrose package (D-16, new `docs/DECISIONS.md` entry D-028 superseding D-005), and the date-based version scheme (D-12).

Not in this phase: display modes and a different wallpaper per monitor (DESK-02/03, Phase 4), stretch and volume settings UI (Phase 4), other engines (Phase 4), settings migration and the Portal (Phase 3), full tray menu (Phase 5), installer (Phase 5), macOS and Linux (Phases 6–7).

</domain>

<decisions>
## Implementation Decisions

### Monitors and audio
- **D-01:** Every monitor gets its own renderer process (`sucrose --render`) that plays the same video on its own desktop surface. The legacy `DisplayScreenType` and `ScreenDevice` keys are not read in M1. The maintainer's own setting is `PerDisplay` on one display; honoring it would show the video on one monitor only, against success criterion 1. Display modes arrive with DESK-02 in Phase 4. Record this M1-only behavior in the `docs/FEATURES.md` notes.
- **D-02:** Renderers that show the same wallpaper stay in sync. One renderer is the master clock and the others align to it, so every monitor shows the same frame with no drift. Audio comes only from the master, at the legacy `WallpaperVolume`; the other renderers are muted, so there is no echo even with a few ms offset between processes. The maintainer's words: "if the same wallpaper runs on several screens, video and audio should be synced between them so there is no drift or echo."

### Fullscreen
- **D-03:** The action comes from legacy `Backgroundog.json` `FullScreenPerformance` (`Close` / `Pause` / `Resume`). When the key is absent, the legacy default `Resume` applies, which disables the rule (D-024, PERF-05 "configured action"). Success criterion 3 is verified with the key set to `Pause`; the maintainer's file currently has no such key. Planning adjusts the wording of criterion 3 in `.planning/ROADMAP.md` to "applies the configured action (Pause: pauses and resumes)".
- **D-04:** "Fullscreen" means the legacy rule: the foreground window covers an entire monitor rectangle, and the check is skipped while the desktop is focused. This includes borderless games, browser F11 and fullscreen video players.
- **D-05:** The scope of the action is a **new setting**: apply Pause or Close to the affected monitor only, or to all monitors. M1 implements both scopes with the legacy behavior "all monitors" as the fixed default and no switch. The setting is persisted with the new settings format (Phase 3) and gets UI with PERF-01 (Phase 4). Add a "**New:**" row or note in `docs/FEATURES.md`. When only the affected monitor pauses, it re-aligns to the master on resume.
- **D-06:** `Close` stops the renderers and starts them again when the condition recovers. `Pause` pauses playback in the engine (mpv `pause`).

### Legacy install and settings
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

### Tray
- **D-14:** The tray menu has only **Quit** (TRAY-01). Quit stops the Rust service and its renderers and never touches C# processes; closing all Sucrose apps is TRAY-05 in Phase 3.
- **D-15:** `General.json` `AppVisible` is honored: when it is `false`, the tray module does not start (D-008). Consequence: in M1 the wallpaper then has no UI to stop it, only Task Manager. The maintainer's value is `true`.

### Engine delivery (libmpv)
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

### Process model
- **D-21:** The D-008 model applies from M1 on. The `sucrose` service spawns one `sucrose --render` per monitor and talks to the renderers through a new `sucrose-ipc` crate (`interprocess` local sockets + `postcard`, D-009): sync clock, pause/resume, close, mute/volume. Phase 3 adds the Portal, settings and the rest of the protocol on top, so no throwaway channel is built. Measurements therefore reflect the final architecture. — **Reversibility:** costly — every later phase builds on the service ↔ renderer protocol.
- **D-22:** When a renderer crashes, the service restarts it with backoff and a bound (for example at most 3 restarts in 60 s). Past the bound, that monitor stays empty and the reason goes to the log and the tray tooltip. This avoids the legacy restart loops.
- **D-23:** Single instance works in M1 through the same IPC endpoint (D-009): a second `sucrose` logs and exits immediately. Opening the Portal from a second instance comes in Phase 3.

### Measurement (MEAS-01)
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` → Phase 2: goal, success criteria 1–5, notes (24H2 hierarchy, fixtures from real themes, release-build measurement)
- `.planning/REQUIREMENTS.md` → TYPE-01, ENG-01, DESK-01, DESK-06, DESK-07, PERF-05, TRAY-01, FMT-02, FMT-08, MEAS-01 (and neighbors DESK-02/03, PERF-01, ENG-06/07, TRAY-02/05, FMT-01 for what stays out)
- `.planning/PROJECT.md`: constraints and out-of-scope list (to be updated per D-16)

### Decisions and architecture
- `docs/DECISIONS.md`: D-002, D-005 (superseded by D-028 in this phase), D-006, D-007, D-008, D-009, D-011, D-012, D-013, D-014, D-024, D-027
- `docs/ARCHITECTURE.md`: Processes, Workspace layout and dependency rules, Platform layer (trait sketch, per-OS mechanisms), Engines, Data and paths, Threading
- `docs/FEATURES.md`: Wallpaper types → Video; Desktop and display; Performance rules → Fullscreen; Tray and app lifecycle; Formats and contracts; Architectures (libmpv row); Legacy problems not to carry over
- `docs/PORTING.md`: Skylark.Wing mapping (`Utility.Desktop`, `Utility.Screene`, `Helper.WindowInterop`, `Struct.Monitor`) and `Sucrose.Mpv.NET` → `sucrose-engine-mpv`
- `docs/MEASUREMENTS.md`: table format for MEAS-01

### Legacy behavior
- `docs/legacy/engines.md` §1 (MpvPlayer), §2 (desktop attachment: Progman/WorkerW sequence, 24H2 fallback, sizing, DPI, display changes, explorer-restart recovery, `RefreshDesktop`), §3 (mpv stretch mapping, DPI), §4 (pause, volume, loop), §9 (mpv config), §10 (lifecycle)
- `docs/legacy/services.md` §1.1 (FullScreen rule, debounce, Close/Pause/Resume, Light/Heavy), §3 (Launcher tray)
- `docs/legacy/formats.md` §1.1 (settings mechanism, `Properties` wrapper), §1.3 (Library/General/Engine/Backgroundog keys and defaults), §2 (paths, mutex names, `Cache\MpvPlayer`), §3 (`SucroseInfo.json` fields, `Version` converter, Localization), §8 (Mpv.NET usage)
- `src/Library/Sucrose.Mpv.NET/` and `src/Shared/Engine/Sucrose.Shared.Engine.MpvPlayer/`: legacy mpv usage (read-only reference)
- `src/Live/Sucrose.Live.MpvPlayer/Requirements/`: the legacy bundled `libmpv-{x86,x64,ARM64}.dll` (evidence for D-016; not to be reused from `src/`)

### Project rules
- `.claude/rules/platform-ffi.md`, `.claude/rules/rust.md`, `.claude/rules/cargo.md`, `.claude/rules/docs.md`
- `CLAUDE.md`: non-negotiables, environment (`SUCROSE_DATA_DIR`, `WSLENV`, no-focus GUI runs, stop the app before rebuilding)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/sucrose-platform`: `Platform` trait (`name`, `capabilities`), `Capabilities { desktop_wallpaper, embed_application, fullscreen_detection }`, per-OS modules with a Windows stub. Extend it along the `docs/ARCHITECTURE.md` sketch (monitors, desktop surface, foreground state, events) and add `#![allow(unsafe_code)]` when FFI arrives (D-011). macOS and Linux stay compiling stubs.
- `crates/sucrose-core/src/paths.rs`: `data_dir()` honors `SUCROSE_DATA_DIR`, plus `log_dir`, `setting_dir` and `cache_dir`. D-08 needs a separate legacy location that ignores the override.
- `crates/sucrose-log`: `init(name)` with daily rolling files. The service uses `"Service"`; renderers need their own name.
- `apps/sucrose/src/main.rs`: stub service that logs and exits. It becomes the service plus `--render` mode (same binary, D-008).
- `xtask/src/main.rs`: only `ci`. Add the pinned libmpv fetch and placement (D-16, D-19) and `measure` (D-26).

### Established Patterns
- Central versions in the root `[workspace.dependencies]`, `[workspace.lints]` with `unsafe_code = "deny"` and `unwrap_used = "warn"`. New crates such as `windows`, `libloading`, `interprocess`, `postcard`, `serde`, `serde_json`, `crossbeam-channel` and a tray crate must be maintained and GPL-3.0 compatible (`deny.toml`).
- Errors: `thiserror` in crates, `anyhow` in apps. `tracing` only, no `println!`. No async runtime in the service (D-012).
- CI builds and tests Windows x64, x86 (i686) and ARM64, Linux x86_64 and aarch64, and macOS (D-027). Windows-only code must keep the other targets compiling.
- `.planning/` files use LF; `docs/DECISIONS.md` uses CRLF. Preserve each file's line endings.

### Integration Points
- `Cargo.toml` `[workspace.package] version = "0.1.0"` changes with D-12.
- `docs/ARCHITECTURE.md` status line and crate list must follow the crates that land (`sucrose-config`, `sucrose-ipc`, `sucrose-engine`, `sucrose-engine-mpv`).
- `docs/FEATURES.md` rows for Video, desktop surface, DPI, monitor and shell recovery, fullscreen, tray and `SucroseInfo.json` change status, and get the behavior notes from D-01, D-05, D-11 and the Heavy-pause note.

</code_context>

<specifics>
## Specific Ideas

- On the maintainer's machine:
  - The C# app is installed. The selected theme is a Video theme (an `.mp4`) with `AppVersion` 26.10.19.0.
  - `Engine.json` has `PerDisplay` on one display, `UniformToFill`, volume 100, loop on and hardware acceleration on.
  - `Backgroundog.json` has no `FullScreenPerformance` key, so the rule is off by default.
  - `General.json` `AppVisible` is `true`.
- The legacy install bundles roughly 123 MB for mpv, 115 MB for VLC and 419 MB for CEF. The libmpv DLLs alone are about 75–101 MB per architecture.
- Tests run on the maintainer's real machine with the C# app installed. Do not add test-only theme selection.

</specifics>

<deferred>
## Deferred Ideas

- Phase 4 (DESK-03): when different wallpapers run on different monitors, each may output its own audio. Same-content sync (D-02) also applies to other video engines and YouTube.
- Phase 3/4 (PERF-01): persist and expose the action-scope setting from D-05 (affected monitor or all, for Pause and Close).
- Phase 4: libVLC and CEF delivered as pinned, xtask-fetched bundled binaries (D-028). ENG-06 then covers WebView2 Runtime and VC++ Redist detection and hints.
- Phase 5: per-architecture packaging of the bundled engines in the installer, and migration from the C# install (SVC-04).
- Phases 6–7: how libmpv ships on macOS and Linux.
- Phase 3: Quit closes all Sucrose apps (TRAY-05), `AppVisible` together with the Portal, and a second instance opening the Portal.
- Possible later improvement (raised by Claude, not requested): decode once and share frames across monitors, instead of one decode per renderer, if the measurements show the N× decode cost matters.

</deferred>

---

*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Context gathered: 2026-11-22*
