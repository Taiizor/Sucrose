# Features and parity

Every feature of the C# app, where it goes in the rewrite, and what each OS allows. This is the parity checklist for M7. Exact legacy behavior (keys, defaults, timings, APIs) is in `docs/legacy/`: `portal.md`, `services.md`, `engines.md`, `formats.md`, `lock-screen.md`.

- **Status:** ⬜ not started · 🟨 in progress · ✅ ported · ➖ dropped (needs a `DECISIONS.md` entry)
- **Platform columns** (planned support): ✅ full · ⚠️ limited · ❌ not possible · ? not investigated yet
- When you port a row: set the status, correct the platform cells with what you learned, and note behavior changes and improvements.

## Wallpaper types

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Video | ✅ | ✅ | ✅ | Engines: mpv, VLC, built-in OS video (D-006), web engines. Legacy default mpv. Rust (M1, Windows): bundled libmpv (D-028), hardware decoding, loop, volume and stretch from the legacy Engine.json; mpv options re-derived for mpv v0.42 (obsolete legacy keys dropped); the user's uMpvPlayer.config is honored read-only (D-10; unit-tested, not exercised against a real file on the maintainer's machine). Plays looping behind the icons on every monitor, one renderer each, embedded with `wid` in the renderer process only (D-017); hardened (`load-scripts=no`, `ytdl=no`, `config=no`, no input or OSC; after the user's config, so it cannot undo them: no `scripts`, `input-ipc-server`, `input-conf`, `log-file`, `use-filedir-conf` or `save-position-on-quit`, checked against the real libmpv) so a theme or a user config cannot make mpv run scripts, open a control socket, write a log or fetch URLs; libmpv is searched next to `sucrose.exe`, then on absolute `PATH` entries (D-028; the current folder and network folders are skipped, and a load from `PATH` is logged as a warning with the full path); a missing libmpv ends the renderer with exit code 2 and a repair hint. Measured on the same GPU (RTX 3050, 4K30 HEVC theme, `MEASUREMENTS.md`): RAM and CPU at or below the C# MpvPlayer engine, GPU 16.4 % vs 16.1 % (R = 1.02), so the legacy mpv options (`deband=yes`, `profile=low-latency`) are kept; the renderer reaches the RTX 3050 through the GpuPreference value the service writes (DESK-12). Other engines in Phase 4. | 🟨 |
| GIF | ✅ | ✅ | ✅ | Built-in renderer replaces Vexana/Xavier; add APNG/WebP. Legacy default mpv. | ⬜ |
| Web (local HTML) | ✅ | ✅ | ✅ | WebView2/CEF (Win), WKWebView/CEF (mac), WebKitGTK/CEF (Linux). JS API must stay compatible. | ⬜ |
| URL | ✅ | ✅ | ✅ | Web engines; no properties, input forwarded. | ⬜ |
| YouTube | ✅ | ✅ | ✅ | IFrame API page without an API key (playlist, shuffle, loop, UI hiding). Possible improvement: mpv + yt-dlp option. | ⬜ |
| Application | ✅ | ❌ | ⚠️ X11 only | Aurora embeds another app's window. Fix legacy bugs (one process per screen in every mode, process handle used as window handle). | ⬜ |

## Desktop and display

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Surface behind the desktop icons | ✅ | ✅ | ⚠️ | Win: window-tree classification; raised desktop (24H2+) via a layered child of Progman ordered under SHELLDLL_DefView (strategy B, 02-07), classic Progman/WorkerW sequence otherwise. Linux: X11 desktop window; Wayland layer-shell (wlroots), KDE/GNOME may need a plugin/extension. Rust (M1, Windows 10.0.26300): the Raised/LayeredChild strategy works with a libmpv video child (icons stay visible and clickable above it, no taskbar or Alt-Tab entry); after stopping the renderer the normal wallpaper came back with no leftover frame in the run with video (a black surface lingered once after a force-kill in plan 06, see the SUMMARY). One surface per monitor (D-01). | 🟨 |
| Per display / span across / duplicate | ✅ | ✅ | ⚠️ | Legacy "duplicate" mirrors with a VisualBrush and is probably blank for HWND engines; implement it properly. M1 (D-01): the same video on every monitor, one renderer each; legacy DisplayScreenType and ScreenDevice are not read until DESK-02 (Phase 4). M1 sync (D-02): monitors playing the same wallpaper follow one master clock. Follower correction (02-20): drift up to one frame of the actual video is left alone; above it the follower's speed is corrected proportionally (at most ±5 %, inaudible because followers play no audio) until it is within a quarter frame, an integral trim absorbs a master or follower that runs slower than real time, and beyond 0.5 s it seeks exactly; each follower logs a drift summary every 30 s. Master election (02-19): a renderer counts once its first frame shows; the first playing renderer, primary monitor first at start, is the master clock and keeps the role while its monitor exists and it plays; when its monitor goes away or it stops, the primary monitor's playing renderer, else the lowest device name, takes over; this departs from the Phase 2 rule that the primary monitor's renderer is the master (D-030); only the master plays audio; with different refresh rates a residual offset below one frame remains. | 🟨 |
| **New:** a different wallpaper on each monitor | ✅ | ✅ | ⚠️ | Not in the C# app; requested by the maintainer. One renderer per monitor. | ⬜ |
| Working area vs. display bounds | ✅ | ✅ | ⚠️ | | ⬜ |
| Stretch modes (None, Fill, Uniform, UniformToFill) | ✅ | ✅ | ✅ | Per-engine mapping in `engines.md` §3. | ⬜ |
| Per-monitor DPI | ✅ | ✅ | ✅ | Rust (M1): renderers and service are PerMonitorV2 aware; rects in physical pixels. | 🟨 |
| Monitor changes, shell restart recovery (CrashExplorer) | ✅ | ? | ? | Behavior change (D-11): always on and event-driven (TaskbarCreated), no 5 s polling; legacy CrashExplorer is ignored. Monitor changes are debounced 2000 ms + 500 ms and applied as per-monitor diffs. Killing explorer.exe does not send WM_DESTROY to the surfaces (observed in UAT; the cause is not established), so recovery relies on TaskbarCreated alone. After a shell restart a surface waits up to 10 s for the shell's wallpaper WorkerW and an event-driven order guard (WinEvent hook on explorer's Progman children) keeps it between the icons and the WorkerW; past 10 successful restores in a minute the guard holds back and arms a one-shot timer that looks at the order again when the oldest restore leaves the minute (so the end of a burst cannot leave the video hidden), it does not remove itself; waiting for the WorkerW keeps dispatching the renderer's messages. Display and shell messages are sent (not posted) messages; the service loop handles them at once. Until the new shell broadcasts TaskbarCreated, the old renderers keep playing into a surface that is not shown. | 🟨 |
| Hide desktop icons toggle | ✅ | ? | ? | Not stored as a setting in legacy. | ⬜ |
| Set OS wallpaper from thumbnail, disable slideshow | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Lock screen wallpaper (Video and GIF themes, live on the lock and sign-in screen) | ✅ | ? | ? | Only on branch `lock-screen-native` (112de37f), not in `develop` (D-026). Legacy runs a credential-provider DLL inside LogonUI; behavior and its security problems in `lock-screen.md`. | ⬜ |
| Stay awake | ✅ | ✅ | ✅ | Legacy also runs elevated `powercfg`; avoid UAC if possible. M1 does not read StayAwake (not in the D-09 key list); mpv runs with stop-screensaver=no; the legacy mapping to stop-screensaver=always returns with the setting in Phase 3/4. | ⬜ |
| GPU preference (high performance) | ✅ | ❌ | ❌ | Registry `UserGpuPreferences`. Rust (Windows, pulled into Phase 2 by the UAT): before the first renderer starts, the service writes `GpuPreference=<n>;` for its own `sucrose.exe` (the renderers run the same exe) from legacy Engine.json `GraphicPreference` (Default 0, PowerSaving 1, HighPerformance 2; default HighPerformance), keeping the value's other pairs and skipping an unchanged value like the C# app; each renderer logs the adapter libmpv's Direct3D 11 device gets (read back from a device created the same way, with no adapter named). Capability `gpu_preference` (false on macOS and Linux). The value stays after the app ends; the uninstaller removes it (SVC-04). | ✅ |

## Playback and sound

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Loop, shuffle | ✅ | ✅ | ✅ | | ⬜ |
| Global volume | ✅ | ✅ | ✅ | | ⬜ |
| Sound only when the desktop is focused | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Sound only when other apps are silent | ✅ | ? | ⚠️ PipeWire | Legacy scans every process's audio session every N seconds. | ⬜ |
| Pause / resume | ✅ | ✅ | ✅ | Replace legacy debugger-attach and `NtSuspendProcess` hacks with engine-level pause (page visibility, CDP, WebView2 TrySuspend). | ⬜ |

## Performance rules (legacy Backgroundog)

Actions Close / Pause / Resume per condition; trigger after 5 consecutive 1 s polls, recover after 3 (some are immediate). Light vs. Heavy pause. Details: `services.md` §1.1.

| Condition | Win | mac | Linux | Notes | Status |
|---|---|---|---|---|---|
| Session lock, sleep, user switch, log out | ✅ | ✅ | ⚠️ | Default Close. | ⬜ |
| Screen saver | ✅ | ✅ | ⚠️ | Default Pause. | ⬜ |
| Desktop not focused (Focus) | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Fullscreen app | ✅ | ✅ | ⚠️ Wayland | Legacy rule: foreground covers a monitor rect, desktop-focused skip, 5 polls to trigger, 3 to recover; 1 s timer only while the action is not Resume (D-008); Heavy pause behaves like Light in M1 (engine pause; PLAY-05 replaces NtSuspendProcess). | 🟨 |
| **New:** fullscreen action scope — affected monitor only or all monitors | ✅ | ✅ | ⚠️ | Both implemented in M1, fixed to all monitors (legacy) with no switch (D-05); persisted with the new settings in Phase 3, UI with PERF-01; a paused monitor re-aligns to the master on resume (plan 14) | 🟨 |
| CPU, memory, GPU usage thresholds | ✅ | ✅ | ✅ | GPU per adapter; NVML/DXGI/IOKit. | ⬜ |
| Network: ping, upload, download thresholds | ✅ | ✅ | ✅ | Fix legacy ordering bug (ping checked first even when its threshold is 0). | ⬜ |
| Battery level, battery saver | ✅ | ✅ | ✅ | | ⬜ |
| Remote desktop, virtual machine / emulator | ✅ | ? | ? | Legacy uses process-name lists with false positives; use OS session APIs where possible. | ⬜ |

## Web wallpaper API (`engines.md` §6–7)

| Feature | Win | mac | Linux | Notes | Status |
|---|---|---|---|---|---|
| `PropertyListener` property calls (incl. `filedropdown` file lists) | ✅ | ✅ | ✅ | Format strings from `SucroseProperties.json`. | ⬜ |
| `SucroseCompatible.json` templates (16 keys) | ✅ | ✅ | ✅ | Legacy formats booleans as `True`/`False` and indented JSON; keep it. | ⬜ |
| System data: Date, CPU, memory, GPU, network, storage, battery | ✅ | ✅ | ✅ | Payload keys in `formats.md` §5 / `engines.md` §6b. | ⬜ |
| System data: BIOS, motherboard | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Audio spectrum (128 log bands, L/R/mono) | ✅ | ⚠️ permission | ✅ | Algorithm in `services.md` §1.2. | ⬜ |
| Now playing (media session, thumbnail) | ✅ | ❌/⚠️ | ✅ MPRIS | | ⬜ |
| Mouse/keyboard forwarding (Web, URL, Application) | ✅ | ⚠️ | ⚠️ X11 / ❌ most Wayland | | ⬜ |
| Generated pages for GIF/Video/YouTube in web engines and their property keys | ✅ | ✅ | ✅ | | ⬜ |
| Developer mode, remote debugging port | ✅ | ✅ | ✅ | | ⬜ |
| **New:** edit engine arguments (Chromium switches) in the UI | ✅ | ✅ | ✅ | Legacy defaults kept; add, remove, reset (D-021). | ⬜ |

## Properties editor (legacy Sucrose.Property)

| Feature | Notes | Status |
|---|---|---|
| Controls: label, button, slider, textbox, checkbox, dropdown, numberbox, passwordbox, filedropdown, colorpicker | `services.md` §6, `formats.md` §3 | ⬜ |
| Live apply to the running wallpaper | Legacy uses watcher files; replace with IPC. | ⬜ |
| Per-language texts (`PropertyLocalization`) | | ⬜ |
| Restore default, refresh, delete copy | | ⬜ |
| Built-in property sets for web pages, mpv, VLC | mpv keys are raw mpv property names. | ⬜ |

## Library (`portal.md` §2)

| Feature | Notes | Status |
|---|---|---|
| Showcase themes copied on first run | tracked in `Warehouse.json` | ⬜ |
| Scan, validity rules, delete corrupt themes | | ⬜ |
| Sorting (name, creation, modification; asc/desc), search index | | ⬜ |
| Pagination, adaptive layout (margin, max per row) | | ⬜ |
| Card: use, review, customize, add/remove from cycle, find, share, delete (confirm), edit | | ⬜ |
| Hover previews, hide details while previewing | | ⬜ |
| Drag & drop: files, `.url`, archives, URLs | | ⬜ |
| Incompatible theme version handling | | ⬜ |
| Library location change and move | | ⬜ |

## Store (`portal.md` §3, `formats.md` §4)

| Feature | Notes | Status |
|---|---|---|
| Soferity server | Protocol per D-022. | ⬜ |
| GitHub server and personal access token | Removed (D-020). | ➖ |
| `Store.json` / `Pattern.json`, cache TTL, offline/broken/unknown states | | ⬜ |
| Categories, adult filter, search, pagination, previews | | ⬜ |
| Install flow (temp download, copy, `SucroseStore.json` marker, auto-start) | | ⬜ |
| Report a theme (Soferity API) | GitHub issue template removed (D-020). | ⬜ |

## Themes (`portal.md` §5)

| Feature | Notes | Status |
|---|---|---|
| Create for all six types (thumbnail, preview, metadata, tags, arguments) | Shell thumbnail on Windows; ffmpeg-free alternative elsewhere. Don't copy a whole parent folder by accident (legacy does for Web/App). | ⬜ |
| Edit, including per-language title/description | | ⬜ |
| Review, delete | | ⬜ |
| Import `.zip` with validation (`CompatibilityType` codes) | `formats.md` §3 | ⬜ |
| Export, publish to store (limits, upload) | | ⬜ |

## Portal UI (`portal.md` §1, §4, §6)

| Feature | Notes | Status |
|---|---|---|
| Window backdrop (Mica/Acrylic/…), background image with stretch and opacity | Windows backdrops; image works everywhere. | ⬜ |
| Light/dark, follow OS theme | | ⬜ |
| Header actions, search, status bar (memory, quotes with copy), command-line page arguments | | ⬜ |
| Settings pages: General, Personal, Performance, Wallpaper, System, Other (every key in `portal.md` §4) | | ⬜ |
| Donate page and advertising settings | Removed (D-023). | ➖ |
| Help and About dialogs, tray hint dialog | | ⬜ |
| 25 languages, RTL for Arabic and Hebrew, live language switch | | ⬜ |
| Single instance | | ⬜ |

## Tray and app lifecycle (`services.md` §3)

| Feature | Win | mac | Linux | Notes | Status |
|---|---|---|---|---|---|
| Tray icon and menu (open, close/reload/change/customize wallpaper, settings, report, updates, exit) | ✅ | ✅ | ⚠️ GNOME needs AppIndicator | Grayscale icon while paused. M1: Quit only (TRAY-01); AppVisible false starts no tray (D-15); full menu is TRAY-02 (Phase 5); Quit stops only the Rust service and its renderers (D-14). The tooltip says what plays or why nothing does (theme rejected by the D-13 gate, libmpv missing). With AppVisible false only Task Manager stops the service in M1. After the renderers exit, Quit re-applies the current wallpaper (SPI_SETDESKWALLPAPER, no registry write) so no frame of the video is left. | 🟨 |
| Startup: normal, priority, scheduled task | ✅ | ✅ LaunchAgent | ✅ XDG autostart | | ⬜ |
| Close all Sucrose apps on exit | ✅ | ✅ | ✅ | | ⬜ |

## Services and integration

| Feature | Notes | Status |
|---|---|---|
| Wallpaper cycling (interval, random/sequential, exclusions) | | ⬜ |
| Discord Rich Presence | Also detect Discord Canary. | ⬜ |
| Updates: channels, auto modes, bandwidth limit | Own updater (D-019): packages signed and verified before install (legacy has no verification); Soferity only (D-020). | ⬜ |
| Installer and uninstaller | Own Rust tools (D-019), signed with signtool (D-025). Fix legacy: quiet uninstall is a no-op; startup entries and GPU registry values are left behind. | ⬜ |
| Crash dialog and crash reports | Respect the opt-out before writing reports. | ⬜ |
| Telemetry (online, analytic, search, download, update) | Legacy structure kept, with per-OS identity inputs (D-018). | ⬜ |
| Logs, log bundle (excluding `Objectionable.json`) | | ⬜ |
| Settings backup, restore, reset; clear cache/store/library/logs | | ⬜ |
| Engine prerequisite detection with install hints (WebView2 version, VC++ for CEF, …) | | ⬜ |
| Commandog command trampoline | Replaced by IPC (D-008, D-009); not a contract. | ➖ |
| Pipe / Signal / Transmission channels | Replaced by one IPC (D-009). | ➖ |

## Formats and contracts (D-007)

| Contract | Spec | Status |
|---|---|---|
| Legacy `Setting/*.json` (14 files, `Properties` wrapper, enums by name) are migrated to the new versioned settings format on first run (D-007) | `formats.md` §1 | ⬜ |
| `SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json` (SucroseInfo.json is read and written compatibly, proven with fixtures, FMT-02; the D-13 gate and the AppVersion rule use it; Properties and Compatible follow in Phase 4) | `formats.md` §3 | 🟨 |
| Theme archives and validation rules | `formats.md` §3 | ⬜ |
| Store protocol and cache layout (D-022) | `formats.md` §4 | ⬜ |
| Data folder layout (`Setting`, `Log`, `Cache`, `Library`, `Showcase`), 25-character theme folder names | `formats.md` §2 | ⬜ |
| Web wallpaper JavaScript API | `engines.md` §6 | ⬜ |

## Architectures (D-027)

Target CPU architectures. Target triples and CI coverage: `ARCHITECTURE.md` → Targets.

| Target | Supported | Notes | Status |
|---|---|---|---|
| Windows x86 (32-bit) | ✅ | Legacy built it: `.build/Sucrose.ps1 -PlatformTarget x86` (`win-x86`); `src/` ships `libmpv-x86.dll`. | ⬜ |
| Windows x64 | ✅ | Legacy built it: `.build/Sucrose.ps1 -PlatformTarget x64` (`win-x64`, the default); `src/` ships `libmpv-x64.dll`. | ⬜ |
| Windows ARM64 | ✅ | Legacy built it: `.build/Sucrose.ps1 -PlatformTarget ARM64` (`win-arm64`); `src/` ships `libmpv-ARM64.dll`. | ⬜ |
| macOS arm64 + x86_64, one universal binary | ✅ | New; the legacy app is Windows-only. | ⬜ |
| Linux x86_64 | ✅ | New; the legacy app is Windows-only. | ⬜ |
| Linux aarch64 | ✅ | New; the legacy app is Windows-only. | ⬜ |

Optional engines per architecture. A cell says whether a build of that engine exists for that architecture; ✅ needs evidence in this repository, cited in Notes. That `.build/Sucrose.ps1` publishes the legacy engine hosts for each `win-*` runtime does not count: it does not show which architectures the engine's own binaries cover. An engine with no build for the running architecture is reported like a missing engine (D-028).

| Engine | Win x86 | Win x64 | Win ARM64 | mac arm64 | mac x86_64 | Linux x86_64 | Linux aarch64 | Notes |
|---|---|---|---|---|---|---|---|---|
| libmpv | ✅ | ✅ | ✅ | ? | ? | ? | ? | Bundled (D-028): pinned shinchiro/mpv-winbuild-cmake 20261111 dev builds in `xtask/engines.toml` (mpv v0.42.0-38, client API 2.6, GPL-2.0-or-later); libmpv-2.dll 99.5 MB (x86), 121.8 MB (x64), 84.1 MB (ARM64). Legacy ships `src/Live/Sucrose.Live.MpvPlayer/Requirements/libmpv-x86.dll`, `libmpv-x64.dll` and `libmpv-ARM64.dll`. |
| libVLC | ? | ? | ? | ? | ? | ? | ? | Legacy uses one package, `VideoLAN.LibVLC.Windows` 3.0.24 (`Directory.Packages.props`); no per-architecture package in `Directory.Packages.props` or `.build/Sucrose.ps1`. |
| CEF | ? | ? | ? | ? | ? | ? | ? | Legacy uses `CefSharp.Wpf.HwndHost` 156.0.150 (`Directory.Packages.props`); no per-architecture package in `Directory.Packages.props` or `.build/Sucrose.ps1`. |
| WebView2 | ? | ? | ? | ❌ | ❌ | ❌ | ❌ | Windows-only. Legacy uses one package, `Microsoft.Web.WebView2` 1.0.4365-prerelease (`Directory.Packages.props`); no per-architecture package in `Directory.Packages.props` or `.build/Sucrose.ps1`. |

## Legacy problems not to carry over

- **Security:** TLS certificate validation is disabled everywhere (SSL2/3 enabled); updates are not verified; the CEF local web server has no path-traversal check. Fix these (D-017, D-019). The legacy Chromium switches stay as defaults but become user-editable (D-021).
- **Privacy:** crash reports are written even when reporting is off; fix it (D-017). The analytic payload keeps its legacy structure (D-018).
- **Reliability:** settings writes are not atomic and a crash resets a whole file; cross-process state is shared through settings files polled by mtime; every engine error triggers a restart (possible loops); `GC.Collect()` runs every second. M1: renderer restarts are bounded (3 crashes per 60 s per monitor, D-22); an engine that is missing is reported, not restarted.
- **Bugs:** the Update limit type reads the Backgroundog download type; quiet uninstall does nothing; Aurora attaches all instances to one target; the network rule checks ping first even at threshold 0.