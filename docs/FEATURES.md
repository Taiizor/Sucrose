# Features and parity

Every feature of the C# app, where it goes in the rewrite, and what each OS allows. This is the parity checklist for M7. Exact legacy behavior (keys, defaults, timings, APIs) is in `docs/legacy/`: `portal.md`, `services.md`, `engines.md`, `formats.md`, `lock-screen.md`.

- **Status:** ⬜ not started · 🟨 in progress · ✅ ported · ➖ dropped (needs a `DECISIONS.md` entry)
- **Platform columns** (planned support): ✅ full · ⚠️ limited · ❌ not possible · ? not investigated yet
- When you port a row: set the status, correct the platform cells with what you learned, and note behavior changes and improvements.

## Wallpaper types

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Video | ✅ | ✅ | ✅ | Engines: mpv, VLC, built-in OS video (D-006), web engines. Legacy default mpv. Rust (M1, Windows): the selected video plays looping behind the icons on the primary monitor through the bundled libmpv, embedded with `wid` in the renderer process only (D-017); hardened (`load-scripts=no`, `ytdl=no`, `config=no`, no input or OSC) so a theme cannot make mpv run scripts or fetch URLs; a missing libmpv ends the renderer with exit code 2 and a repair hint. Multi-monitor, pause, volume and Engine.json options follow in later M1 plans. | 🟨 |
| GIF | ✅ | ✅ | ✅ | Built-in renderer replaces Vexana/Xavier; add APNG/WebP. Legacy default mpv. | ⬜ |
| Web (local HTML) | ✅ | ✅ | ✅ | WebView2/CEF (Win), WKWebView/CEF (mac), WebKitGTK/CEF (Linux). JS API must stay compatible. | ⬜ |
| URL | ✅ | ✅ | ✅ | Web engines; no properties, input forwarded. | ⬜ |
| YouTube | ✅ | ✅ | ✅ | IFrame API page without an API key (playlist, shuffle, loop, UI hiding). Possible improvement: mpv + yt-dlp option. | ⬜ |
| Application | ✅ | ❌ | ⚠️ X11 only | Aurora embeds another app's window. Fix legacy bugs (one process per screen in every mode, process handle used as window handle). | ⬜ |

## Desktop and display

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Surface behind the desktop icons | ✅ | ✅ | ⚠️ | Win: Progman/WorkerW incl. 24H2. Linux: X11 desktop window; Wayland layer-shell (wlroots), KDE/GNOME may need a plugin/extension. Rust (M1, Windows 10.0.26300): the Raised/LayeredChild strategy works with a libmpv video child (icons stay visible and clickable above it, no taskbar or Alt-Tab entry); after stopping the renderer the normal wallpaper came back with no leftover frame in the run with video (a black surface lingered once after a force-kill in plan 06, see the SUMMARY). Primary monitor only so far. | 🟨 |
| Per display / span across / duplicate | ✅ | ✅ | ⚠️ | Legacy "duplicate" mirrors with a VisualBrush and is probably blank for HWND engines; implement it properly. | ⬜ |
| **New:** a different wallpaper on each monitor | ✅ | ✅ | ⚠️ | Not in the C# app; requested by the maintainer. One renderer per monitor. | ⬜ |
| Working area vs. display bounds | ✅ | ✅ | ⚠️ | | ⬜ |
| Stretch modes (None, Fill, Uniform, UniformToFill) | ✅ | ✅ | ✅ | Per-engine mapping in `engines.md` §3. | ⬜ |
| Per-monitor DPI | ✅ | ✅ | ✅ | | ⬜ |
| Monitor changes, shell restart recovery (CrashExplorer) | ✅ | ? | ? | | ⬜ |
| Hide desktop icons toggle | ✅ | ? | ? | Not stored as a setting in legacy. | ⬜ |
| Set OS wallpaper from thumbnail, disable slideshow | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Lock screen wallpaper (Video and GIF themes, live on the lock and sign-in screen) | ✅ | ? | ? | Only on branch `lock-screen-native` (112de37f), not in `develop` (D-026). Legacy runs a credential-provider DLL inside LogonUI; behavior and its security problems in `lock-screen.md`. | ⬜ |
| Stay awake | ✅ | ✅ | ✅ | Legacy also runs elevated `powercfg`; avoid UAC if possible. | ⬜ |
| GPU preference (high performance) | ✅ | ❌ | ❌ | Registry `UserGpuPreferences`. | ⬜ |

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
| Fullscreen app | ✅ | ✅ | ⚠️ Wayland | | ⬜ |
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
| Tray icon and menu (open, close/reload/change/customize wallpaper, settings, report, updates, exit) | ✅ | ✅ | ⚠️ GNOME needs AppIndicator | Grayscale icon while paused. | ⬜ |
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
| `SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json` | `formats.md` §3 | ⬜ |
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
- **Reliability:** settings writes are not atomic and a crash resets a whole file; cross-process state is shared through settings files polled by mtime; every engine error triggers a restart (possible loops); `GC.Collect()` runs every second.
- **Bugs:** the Update limit type reads the Backgroundog download type; quiet uninstall does nothing; Aurora attaches all instances to one target; the network rule checks ping first even at threshold 0.