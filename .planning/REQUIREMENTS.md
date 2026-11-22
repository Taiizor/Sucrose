# Requirements: Sucrose (Rust rewrite)

**Defined:** 2026-11-22
**Core Value:** An existing user upgrades and loses nothing (themes, store, web wallpapers and their JavaScript API, settings migrated automatically) while the new app uses less memory and CPU than the C# one.

Derived from `docs/FEATURES.md` (parity checklist, D-003: every feature is kept), the M0–M7 milestone items (formerly `docs/ROADMAP.md`) and `docs/DECISIONS.md`. Exact legacy behavior for each row is in `docs/legacy/`. The source rows define no acceptance criteria of their own; phase success criteria and the legacy references define them.

- Categories TYPE … FMT are delivered on **Windows first** (D-002). MAC-* and LNX-* bring them to macOS and Linux as far as each OS allows.
- Platform tags come from `docs/FEATURES.md`: ✅ full · ⚠️ limited · ❌ not possible · ? not investigated yet.

## v1 Requirements

### Foundation (M0)

- [x] **FND-01**: CI is green on Windows, macOS and Linux (fmt, clippy `-D warnings`, tests, cargo-deny) with stub platform implementations for macOS and Linux
- [x] **FND-02**: CI builds and tests Windows ARM64 and Linux ARM64 natively, builds and tests the Windows x86 (i686) target on the Windows x64 runner, and checks the macOS x86_64 target (D-027)

### Architecture

- [ ] **ARCH-01**: The service runs one renderer process per running wallpaper (per monitor or span); a crashed renderer is restarted with a bounded retry, never in a loop (D-008)
- [ ] **ARCH-02**: All Sucrose processes talk over one local-socket IPC, and the same endpoint keeps the service and the Portal single-instance (D-009)
- [ ] **ARCH-03**: An optional service module the user turns off does not start at all: no thread, no polling, no allocations (D-008)
- [ ] **ARCH-04**: Settings use a new versioned format with atomic writes; runtime state (pause/close flags, ports) is never stored in settings, and processes share state over IPC instead of polled files (D-007)
- [ ] **ARCH-05**: "Use" in the Portal applies the wallpaper through the service, and settings changed in the Portal persist across restarts

### Wallpaper types

- [x] **TYPE-01**: User can set a Video theme as the desktop wallpaper (Win ✅ · mac ✅ · Linux ✅)
- [ ] **TYPE-02**: User can set a GIF theme, drawn by Sucrose's built-in renderer, which also plays APNG and WebP (Win ✅ · mac ✅ · Linux ✅)
- [ ] **TYPE-03**: User can set a Web theme (local HTML) on a web engine: WebView2/CEF on Windows, WKWebView/CEF on macOS, WebKitGTK/CEF on Linux (Win ✅ · mac ✅ · Linux ✅)
- [ ] **TYPE-04**: User can set a URL wallpaper; it has no properties and receives forwarded input (Win ✅ · mac ✅ · Linux ✅)
- [ ] **TYPE-05**: User can set a YouTube wallpaper (IFrame API page, no API key) with playlist, shuffle, loop and hidden player UI (Win ✅ · mac ✅ · Linux ✅)
- [ ] **TYPE-06**: User can set an Application wallpaper that embeds another app's window, with one process per screen in every mode and real window handles (legacy bugs fixed) (Win ✅ · mac ❌ · Linux ⚠️ X11 only)

### Engines

- [x] **ENG-01**: The libmpv engine ships inside the Sucrose package, is loaded at runtime, embeds into the desktop surface, and supports hardware decoding, loop, pause and volume (D-013, D-028)
- [ ] **ENG-02**: The WebView2 engine hosts Web, URL and YouTube wallpapers with the C#-compatible JavaScript API
- [ ] **ENG-03**: The CEF engine hosts the same wallpapers with the same JavaScript API
- [ ] **ENG-04**: The libVLC engine plays video when libVLC is installed
- [ ] **ENG-05**: The built-in OS video engine (Media Foundation on Windows) plays video with no engine installed (D-006)
- [ ] **ENG-06**: Sucrose detects the engines and prerequisites it needs (bundled engine binaries present and loadable, WebView2 Runtime version, VC++ Redist for CEF) and shows install or repair hints; the WebView2 Runtime and VC++ Redist are user-side downloads (D-028)
- [ ] **ENG-07**: User can choose, per wallpaper type, which installed engine plays it (legacy `Engine.json` keys Gif/Url/Web/Video/YouTube/Application). If the chosen engine is not installed, Sucrose falls back to the built-in engine or another installed one and tells the user (D-006, D-028)

### Desktop and display

- [ ] **DESK-01**: The wallpaper renders on a surface behind the desktop icons; on Windows through Progman/WorkerW including the 24H2 hierarchy (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **DESK-02**: User can show a wallpaper per display, spanned across displays or duplicated; duplicate works for every engine (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **DESK-03**: **New:** user can put a different wallpaper on each monitor, one renderer per monitor (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **DESK-04**: User can choose the working area or the full display bounds (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **DESK-05**: User can choose a stretch mode (None, Fill, Uniform, UniformToFill), mapped per engine as in `engines.md` §3 (Win ✅ · mac ✅ · Linux ✅)
- [ ] **DESK-06**: Wallpapers are sharp on every monitor with per-monitor DPI (Win ✅ · mac ✅ · Linux ✅)
- [ ] **DESK-07**: Wallpapers recover from monitor changes and shell restarts (legacy CrashExplorer) (Win ✅ · mac ? · Linux ?)
- [ ] **DESK-08**: User can toggle hiding the desktop icons (Win ✅ · mac ? · Linux ?)
- [ ] **DESK-09**: User can set the OS wallpaper from a theme thumbnail and disable the OS slideshow (Win ✅ · mac ⚠️ · Linux ⚠️)
- [ ] **DESK-10**: User can set a Video or GIF theme as the live lock and sign-in screen wallpaper with fit mode, mute and sync with the desktop (`lock-screen.md`; mechanism decided first, D-026) (Win ✅ · mac ? · Linux ?)
- [ ] **DESK-11**: User can keep the computer awake while a wallpaper runs, avoiding UAC where possible (Win ✅ · mac ✅ · Linux ✅)
- [ ] **DESK-12**: Sucrose can request the high-performance GPU (registry `UserGpuPreferences`) (Win ✅ · mac ❌ · Linux ❌)

### Playback and sound

- [ ] **PLAY-01**: User can loop and shuffle playback (Win ✅ · mac ✅ · Linux ✅)
- [ ] **PLAY-02**: User can set a global wallpaper volume (Win ✅ · mac ✅ · Linux ✅)
- [ ] **PLAY-03**: User can limit sound to when the desktop is focused (Win ✅ · mac ⚠️ · Linux ⚠️)
- [ ] **PLAY-04**: User can limit sound to when other apps are silent (legacy scans every process's audio session every N seconds) (Win ✅ · mac ? · Linux ⚠️ PipeWire)
- [ ] **PLAY-05**: User can pause and resume wallpapers through engine-level pause (page visibility, CDP, WebView2 TrySuspend) instead of the legacy debugger-attach and `NtSuspendProcess` hacks (Win ✅ · mac ✅ · Linux ✅)

### Performance rules (legacy Backgroundog, `services.md` §1.1)

- [ ] **PERF-01**: User can set Close, Pause or Resume per condition; a condition triggers after 5 consecutive 1 s polls and recovers after 3 (some immediately), with Light and Heavy pause
- [ ] **PERF-02**: Session lock, sleep, user switch and log out trigger the configured action (default Close) (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **PERF-03**: The screen saver triggers the configured action (default Pause) (Win ✅ · mac ✅ · Linux ⚠️)
- [ ] **PERF-04**: The desktop losing focus (Focus) triggers the configured action (Win ✅ · mac ⚠️ · Linux ⚠️)
- [ ] **PERF-05**: A fullscreen app triggers the configured action (Win ✅ · mac ✅ · Linux ⚠️ Wayland)
- [ ] **PERF-06**: CPU, memory and per-adapter GPU usage thresholds trigger the configured action (Win ✅ · mac ✅ · Linux ✅)
- [ ] **PERF-07**: Network ping, upload and download thresholds trigger the configured action, without the legacy bug that checks ping first even at threshold 0 (Win ✅ · mac ✅ · Linux ✅)
- [ ] **PERF-08**: Battery level and battery saver trigger the configured action (Win ✅ · mac ✅ · Linux ✅)
- [ ] **PERF-09**: Remote desktop and virtual machine/emulator sessions trigger the configured action, detected through OS session APIs where possible instead of process-name lists (Win ✅ · mac ? · Linux ?)

### Web wallpaper API (`engines.md` §6–7)

- [ ] **WAPI-01**: Web wallpapers receive `PropertyListener` property calls, including `filedropdown` file lists, formatted by the strings in `SucroseProperties.json` (Win ✅ · mac ✅ · Linux ✅)
- [ ] **WAPI-02**: `SucroseCompatible.json` templates (16 keys) work, keeping legacy `True`/`False` booleans and indented JSON (Win ✅ · mac ✅ · Linux ✅)
- [ ] **WAPI-03**: Web wallpapers receive date, CPU, memory, GPU, network, storage and battery data with the legacy payload keys (`formats.md` §5, `engines.md` §6b) (Win ✅ · mac ✅ · Linux ✅)
- [ ] **WAPI-04**: Web wallpapers receive BIOS and motherboard data (Win ✅ · mac ⚠️ · Linux ⚠️)
- [ ] **WAPI-05**: Web wallpapers receive the audio spectrum: 128 logarithmic bands, left/right/mono (`services.md` §1.2) (Win ✅ · mac ⚠️ permission · Linux ✅)
- [ ] **WAPI-06**: Web wallpapers receive now-playing data (media session with thumbnail) (Win ✅ · mac ❌/⚠️ · Linux ✅ MPRIS)
- [ ] **WAPI-07**: Mouse and keyboard input is forwarded to Web, URL and Application wallpapers (Win ✅ · mac ⚠️ · Linux ⚠️ X11 / ❌ most Wayland)
- [ ] **WAPI-08**: GIF, Video and YouTube themes can run as generated pages in web engines, with their property keys (Win ✅ · mac ✅ · Linux ✅)
- [ ] **WAPI-09**: User can turn on developer mode with a remote debugging port (Win ✅ · mac ✅ · Linux ✅)
- [ ] **WAPI-10**: **New:** user can add, remove and reset web engine arguments (Chromium switches) in the UI; legacy switches stay the defaults (D-021) (Win ✅ · mac ✅ · Linux ✅)

### Properties editor (legacy Sucrose.Property, `services.md` §6, `formats.md` §3)

- [ ] **PROP-01**: The properties editor offers label, button, slider, textbox, checkbox, dropdown, numberbox, passwordbox, filedropdown and colorpicker controls
- [ ] **PROP-02**: Property changes apply live to the running wallpaper over IPC (replacing legacy watcher files)
- [ ] **PROP-03**: Property texts appear in the user's language (`PropertyLocalization`)
- [ ] **PROP-04**: User can restore defaults, refresh, and delete the property copy
- [ ] **PROP-05**: Built-in property sets exist for web pages, mpv (raw mpv property names) and VLC

### Library (`portal.md` §2)

- [ ] **LIB-01**: Showcase themes are copied on first run and tracked in `Warehouse.json`
- [ ] **LIB-02**: The library is scanned with the legacy validity rules and corrupt themes are deleted
- [ ] **LIB-03**: User can sort the library by name, creation or modification date (ascending/descending) and search it
- [ ] **LIB-04**: The library is paginated with an adaptive layout (margin, max per row)
- [ ] **LIB-05**: A theme card offers use, review, customize, add/remove from cycling, find, share, delete (with confirmation) and edit
- [ ] **LIB-06**: Theme cards show hover previews, optionally hiding details while previewing
- [ ] **LIB-07**: User can drag and drop files, `.url` files, archives and URLs onto the library
- [ ] **LIB-08**: Themes made for an incompatible version are handled as in the legacy app
- [ ] **LIB-09**: User can change the library location and move the existing themes

### Store (`portal.md` §3, `formats.md` §4)

- [ ] **STOR-01**: The store talks to the Soferity server (protocol per D-022; the existing v8 endpoints until then)
- [ ] **STOR-02**: Store data uses `Store.json`/`Pattern.json` with the cache TTL and shows offline, broken and unknown states
- [ ] **STOR-03**: User can browse categories, filter adult content, search, page and preview store themes
- [ ] **STOR-04**: User can install a store theme (temp download, copy, `SucroseStore.json` marker, optional auto-start)
- [ ] **STOR-05**: User can report a theme through the Soferity API (GitHub issue template removed, D-020)

### Themes (`portal.md` §5)

- [ ] **THEM-01**: User can create themes of all six types with thumbnail, preview, metadata, tags and arguments (shell thumbnail on Windows, ffmpeg-free elsewhere) without a whole parent folder being copied by accident
- [ ] **THEM-02**: User can edit a theme, including per-language title and description
- [ ] **THEM-03**: User can review and delete a theme
- [ ] **THEM-04**: User can import a `.zip` theme, validated with the `CompatibilityType` codes
- [ ] **THEM-05**: User can export a theme and publish it to the store within the legacy limits

### Portal UI (`portal.md` §1, §4, §6)

- [ ] **UI-01**: The Portal has a window backdrop (Mica/Acrylic on Windows) and a background image with stretch and opacity on every OS
- [ ] **UI-02**: User can pick light or dark, or follow the OS theme
- [ ] **UI-03**: The Portal has header actions, search, a status bar (memory, quotes with copy) and command-line page arguments
- [ ] **UI-04**: The settings pages General, Personal, Performance, Wallpaper, System and Other cover every key in `portal.md` §4
- [ ] **UI-05**: Help and About dialogs and the tray hint dialog exist; plain links (wiki, report an issue) stay (D-020)
- [ ] **UI-06**: The UI is available in the 25 existing languages, converted by `xtask`, with RTL for Arabic and Hebrew and a live language switch (D-015)
- [ ] **UI-07**: Browsing the library and store keeps memory bounded: thumbnails are decoded at card size, the cache is bounded, and images no longer shown are released from GPUI (`docs/GPUI.md`, prototype lesson). Measured as part of MEAS-02

### Tray and app lifecycle (`services.md` §3)

- [x] **TRAY-01**: A tray icon with Quit is available while a wallpaper runs
- [ ] **TRAY-02**: The full tray menu offers open, close/reload/change/customize wallpaper, settings, report, updates and exit, with a grayscale icon while paused (Win ✅ · mac ✅ · Linux ⚠️ GNOME needs AppIndicator)
- [ ] **TRAY-03**: Sucrose can start with the OS (normal startup) (Win ✅ · mac ✅ LaunchAgent · Linux ✅ XDG autostart)
- [ ] **TRAY-04**: User can choose the priority and scheduled-task startup modes (Win ✅ · mac ? · Linux ?)
- [ ] **TRAY-05**: Exiting closes all Sucrose apps (Win ✅ · mac ✅ · Linux ✅)

### Services and integration

- [ ] **SVC-01**: Wallpaper cycling with interval, random or sequential order and exclusions
- [ ] **SVC-02**: Discord Rich Presence, also detecting Discord Canary
- [ ] **SVC-03**: Updates with Release/PreRelease channels, auto modes and the bandwidth limit; packages are signed and verified before install; Soferity only; the legacy limit-type bug is fixed (D-019, D-020)
- [ ] **SVC-04**: Own installer and uninstaller signed with signtool; quiet uninstall works and removes startup entries and GPU-preference registry values; an existing C# install is migrated (D-019, D-025); packages for Windows x86, x64 and ARM64 (D-027)
- [ ] **SVC-05**: The crash dialog and crash reports respect the opt-out before anything is written (D-017)
- [ ] **SVC-06**: Telemetry (online, analytic, search, download, update) keeps the legacy payload structure with per-OS identity inputs and the existing opt-outs (D-018, D-024)
- [ ] **SVC-07**: Logs and a log bundle that excludes `Objectionable.json`
- [ ] **SVC-08**: User can back up, restore and reset settings, and clear the cache, store, library and logs

### Formats and contracts (D-007)

- [ ] **FMT-01**: Legacy `Setting/*.json` files (14 files, `Properties` wrapper, enums by name) migrate to the new settings format on first run, `Donate.json` excluded (D-023), proven with fixtures
- [x] **FMT-02**: `SucroseInfo.json` is read and written compatibly, proven with fixtures from real themes
- [ ] **FMT-03**: `SucroseProperties.json` and `SucroseCompatible.json` are read and written compatibly, proven with fixtures
- [ ] **FMT-04**: Theme archives are validated with the legacy rules (`formats.md` §3)
- [ ] **FMT-05**: The store protocol and cache layout stay compatible (D-022, `formats.md` §4)
- [ ] **FMT-06**: The data folder layout (`Setting`, `Log`, `Cache`, `Library`, `Showcase`) and 25-character theme folder names stay the same (`formats.md` §2)
- [ ] **FMT-07**: The web wallpaper JavaScript API stays compatible (`engines.md` §6), proven with fixture pages
- [x] **FMT-08**: The selected theme and library location of an existing C# install are read from the legacy `Library.json` and `General.json` (read-only until FMT-01)

### Security

- [ ] **SEC-01**: TLS certificate validation stays on for every connection, with SSL2/SSL3 disabled (D-017)
- [ ] **SEC-02**: Local file serving for web wallpapers rejects paths outside the theme folder (D-017)

### Measurements

- [ ] **MEAS-01**: RAM and CPU of the Windows video wallpaper (release build) are measured against the C# MpvPlayer engine + Backgroundog and recorded
- [ ] **MEAS-02**: At sign-off the release build uses less RAM and CPU than the C# app in every recorded scenario, and the always-on service is close to idle cost

### macOS (M5)

- [ ] **MAC-01**: macOS platform layer: desktop-level window surface, monitors and DPI, fullscreen detection (CGWindowList), power state and events
- [ ] **MAC-02**: Engines on macOS: WKWebView, CEF, libmpv, libVLC and the AVFoundation built-in video engine
- [ ] **MAC-03**: System data, audio capture (ScreenCaptureKit / Core Audio taps, with permission) and the limited media session work on macOS as `docs/FEATURES.md` allows
- [ ] **MAC-04**: Tray, LaunchAgent autostart and data under the platform config directory on macOS
- [ ] **MAC-05**: macOS packages and updates as one universal binary (arm64 + x86_64), unsigned until an Apple Developer ID exists (D-025, D-027)
- [ ] **MAC-06**: Every `?` in the macOS column of `docs/FEATURES.md` (including the lock screen) is investigated and set; what macOS cannot do is reported through platform capabilities and hidden or explained in the UI

### Linux (M6)

- [ ] **LNX-01**: Linux X11 platform layer: `_NET_WM_WINDOW_TYPE_DESKTOP` surface, monitors and DPI, EWMH fullscreen detection, power state and events
- [ ] **LNX-02**: Wayland: wlr-layer-shell background surface, with a KDE/GNOME plugin or extension where needed; unsupported features reported per compositor
- [ ] **LNX-03**: Engines on Linux: WebKitGTK, CEF, libmpv, libVLC and the GStreamer built-in video engine
- [ ] **LNX-04**: System data, PipeWire/PulseAudio audio capture and the MPRIS media session work on Linux
- [ ] **LNX-05**: Tray (AppIndicator on GNOME), XDG autostart and data under the platform config directory on Linux
- [ ] **LNX-06**: Linux packages for x86_64 and aarch64 (formats decided at M6) and updates (D-027)
- [ ] **LNX-07**: Every `?` in the Linux column of `docs/FEATURES.md` (including the lock screen) is investigated and set; what Linux cannot do is reported through platform capabilities

### Parity sign-off (M7)

- [ ] **SIGN-01**: Every row in `docs/FEATURES.md` is ported (✅) or covered by an accepted decision (➖), with correct platform cells
- [ ] **SIGN-02**: The C# code in `src/` is removed and `CLAUDE.md` and `docs/` are rewritten for the finished project (D-016)

## v2 Requirements

Deferred; tracked but not in the current roadmap.

- **YT-01**: YouTube wallpapers can play through mpv + yt-dlp as an alternative to the IFrame page (`docs/FEATURES.md`: possible improvement)

## Out of Scope

| Feature | Reason |
|---------|--------|
| GitHub store server and personal access token (`Objectionable.json`) | Removed; Soferity API only (D-020) |
| GitHub update source | Removed; Soferity API only (D-020) |
| Theme report GitHub issue template ("Report from GitHub") | Removed; reports go through the Soferity API (D-020) |
| Donate page, advertising settings, `Donate.json` migration | Removed (D-023) |
| Commandog command trampoline as a contract | Replaced by IPC (D-008, D-009) |
| Pipe / Signal / Transmission channels | Replaced by one IPC (D-009) |
| Bundling the WebView2 Runtime or the VC++ Redist | User-side prerequisites, detected with download hints (D-028) |
| A web runtime in the core or the UI | GPUI via gpui-kit (D-004) |
| Building the Portal on `exp/GPUI/Sucrose.Portal` | Throwaway experiment, not the base of `apps/sucrose-portal` |
| Faking features an OS cannot do | Reported through platform capabilities (D-002) |

## Traceability

Which phases cover which requirements. Filled during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| ARCH-01 | Phase 3 | Pending |
| ARCH-02 | Phase 3 | Pending |
| ARCH-03 | Phase 3 | Pending |
| ARCH-04 | Phase 3 | Pending |
| ARCH-05 | Phase 3 | Pending |
| TYPE-01 | Phase 2 | Complete |
| TYPE-02 | Phase 4 | Pending |
| TYPE-03 | Phase 4 | Pending |
| TYPE-04 | Phase 4 | Pending |
| TYPE-05 | Phase 4 | Pending |
| TYPE-06 | Phase 4 | Pending |
| ENG-01 | Phase 2 | Complete |
| ENG-02 | Phase 4 | Pending |
| ENG-03 | Phase 4 | Pending |
| ENG-04 | Phase 4 | Pending |
| ENG-05 | Phase 4 | Pending |
| ENG-06 | Phase 4 | Pending |
| ENG-07 | Phase 4 | Pending |
| DESK-01 | Phase 2 | Pending |
| DESK-02 | Phase 4 | Pending |
| DESK-03 | Phase 4 | Pending |
| DESK-04 | Phase 4 | Pending |
| DESK-05 | Phase 4 | Pending |
| DESK-06 | Phase 2 | Pending |
| DESK-07 | Phase 2 | Pending |
| DESK-08 | Phase 5 | Pending |
| DESK-09 | Phase 5 | Pending |
| DESK-10 | Phase 5 | Pending |
| DESK-11 | Phase 5 | Pending |
| DESK-12 | Phase 5 | Pending |
| PLAY-01 | Phase 4 | Pending |
| PLAY-02 | Phase 4 | Pending |
| PLAY-03 | Phase 4 | Pending |
| PLAY-04 | Phase 4 | Pending |
| PLAY-05 | Phase 4 | Pending |
| PERF-01 | Phase 4 | Pending |
| PERF-02 | Phase 4 | Pending |
| PERF-03 | Phase 4 | Pending |
| PERF-04 | Phase 4 | Pending |
| PERF-05 | Phase 2 | Pending |
| PERF-06 | Phase 4 | Pending |
| PERF-07 | Phase 4 | Pending |
| PERF-08 | Phase 4 | Pending |
| PERF-09 | Phase 4 | Pending |
| WAPI-01 | Phase 4 | Pending |
| WAPI-02 | Phase 4 | Pending |
| WAPI-03 | Phase 4 | Pending |
| WAPI-04 | Phase 4 | Pending |
| WAPI-05 | Phase 4 | Pending |
| WAPI-06 | Phase 4 | Pending |
| WAPI-07 | Phase 4 | Pending |
| WAPI-08 | Phase 4 | Pending |
| WAPI-09 | Phase 4 | Pending |
| WAPI-10 | Phase 4 | Pending |
| PROP-01 | Phase 4 | Pending |
| PROP-02 | Phase 4 | Pending |
| PROP-03 | Phase 4 | Pending |
| PROP-04 | Phase 4 | Pending |
| PROP-05 | Phase 4 | Pending |
| LIB-01 | Phase 3 | Pending |
| LIB-02 | Phase 3 | Pending |
| LIB-03 | Phase 3 | Pending |
| LIB-04 | Phase 3 | Pending |
| LIB-05 | Phase 5 | Pending |
| LIB-06 | Phase 3 | Pending |
| LIB-07 | Phase 5 | Pending |
| LIB-08 | Phase 3 | Pending |
| LIB-09 | Phase 3 | Pending |
| STOR-01 | Phase 5 | Pending |
| STOR-02 | Phase 5 | Pending |
| STOR-03 | Phase 5 | Pending |
| STOR-04 | Phase 5 | Pending |
| STOR-05 | Phase 5 | Pending |
| THEM-01 | Phase 5 | Pending |
| THEM-02 | Phase 5 | Pending |
| THEM-03 | Phase 5 | Pending |
| THEM-04 | Phase 5 | Pending |
| THEM-05 | Phase 5 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 3 | Pending |
| UI-03 | Phase 3 | Pending |
| UI-04 | Phase 5 | Pending |
| UI-05 | Phase 3 | Pending |
| UI-06 | Phase 5 | Pending |
| UI-07 | Phase 3 | Pending |
| TRAY-01 | Phase 2 | Complete |
| TRAY-02 | Phase 5 | Pending |
| TRAY-03 | Phase 3 | Pending |
| TRAY-04 | Phase 5 | Pending |
| TRAY-05 | Phase 3 | Pending |
| SVC-01 | Phase 5 | Pending |
| SVC-02 | Phase 5 | Pending |
| SVC-03 | Phase 5 | Pending |
| SVC-04 | Phase 5 | Pending |
| SVC-05 | Phase 5 | Pending |
| SVC-06 | Phase 5 | Pending |
| SVC-07 | Phase 5 | Pending |
| SVC-08 | Phase 5 | Pending |
| FMT-01 | Phase 3 | Pending |
| FMT-02 | Phase 2 | Complete |
| FMT-03 | Phase 4 | Pending |
| FMT-04 | Phase 5 | Pending |
| FMT-05 | Phase 5 | Pending |
| FMT-06 | Phase 3 | Pending |
| FMT-07 | Phase 4 | Pending |
| FMT-08 | Phase 2 | Complete |
| SEC-01 | Phase 5 | Pending |
| SEC-02 | Phase 4 | Pending |
| MEAS-01 | Phase 2 | Pending |
| MEAS-02 | Phase 8 | Pending |
| MAC-01 | Phase 6 | Pending |
| MAC-02 | Phase 6 | Pending |
| MAC-03 | Phase 6 | Pending |
| MAC-04 | Phase 6 | Pending |
| MAC-05 | Phase 6 | Pending |
| MAC-06 | Phase 6 | Pending |
| LNX-01 | Phase 7 | Pending |
| LNX-02 | Phase 7 | Pending |
| LNX-03 | Phase 7 | Pending |
| LNX-04 | Phase 7 | Pending |
| LNX-05 | Phase 7 | Pending |
| LNX-06 | Phase 7 | Pending |
| LNX-07 | Phase 7 | Pending |
| SIGN-01 | Phase 8 | Pending |
| SIGN-02 | Phase 8 | Pending |

**Coverage:**
- v1 requirements: 127 total
- Mapped to phases: 127 (Phase 1: 2 · Phase 2: 10 · Phase 3: 21 · Phase 4: 46 · Phase 5: 32 · Phase 6: 6 · Phase 7: 7 · Phase 8: 3)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-11-22*
*Last updated: 2026-11-22 after Phase 1 completion (FND-01, FND-02)*
