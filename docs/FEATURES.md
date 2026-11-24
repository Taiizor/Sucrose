# Features and parity

Every feature of the C# app, where it goes in the rewrite, and what each OS allows. This is the parity checklist for M7. Exact legacy behavior (keys, defaults, timings, APIs) is in `docs/legacy/`: `portal.md`, `services.md`, `engines.md`, `formats.md`, `lock-screen.md`.

- **Status:** ⬜ not started · 🟨 in progress · ✅ ported · ➖ dropped (needs a `DECISIONS.md` entry)
- **Platform columns** (planned support): ✅ full · ⚠️ limited · ❌ not possible · ? not investigated yet
- When you port a row: set the status, correct the platform cells with what you learned, and note behavior changes and improvements.

## Wallpaper types

| Feature | Win | mac | Linux | Notes / improvements | Status |
|---|---|---|---|---|---|
| Video | ✅ | ✅ | ✅ | Engines: mpv, VLC, built-in OS video (D-006), web engines. Legacy default mpv. Rust (M1, Windows): bundled libmpv (D-028), hardware decoding, loop, volume and stretch from the legacy Engine.json; mpv options re-derived for mpv v0.42 (obsolete legacy keys dropped); the user's uMpvPlayer.config is honored read-only (D-10; unit-tested, not exercised against a real file on the maintainer's machine). Plays looping behind the icons on every monitor, one renderer each, embedded with `wid` in the renderer process only (D-017); hardened (`load-scripts=no`, `ytdl=no`, `config=no`, no input or OSC; after the user's config, so it cannot undo them: no `scripts`, `input-ipc-server`, `input-conf`, `log-file`, `use-filedir-conf` or `save-position-on-quit`, checked against the real libmpv) so a theme or a user config cannot make mpv run scripts, open a control socket, write a log or fetch URLs; libmpv is searched next to `sucrose.exe`, then on absolute `PATH` entries (D-028; the current folder and network folders are skipped, and a load from `PATH` is logged as a warning with the full path); a missing libmpv ends the renderer with exit code 2 and a repair hint; a file mpv cannot open or decode ends the renderer as a failed start (exit code 4, counted in the restart budget) with mpv's reason in the tray tooltip, which stays through the restarts and is named again when the budget is spent (a `Failed` that arrives after its renderer's exit still counts; a renderer's report is ignored when another renderer holds or runs the monitor's slot, when its monitor was removed or the fullscreen rule closed it, and a late `Failed` does not replace the status once the restart budget is spent; a late `Playing` of an ended renderer is always ignored). Measured on the same GPU (RTX 3050, 4K30 HEVC theme, `MEASUREMENTS.md`): RAM and CPU at or below the C# MpvPlayer engine, GPU 16.4 % vs 16.1 % (R = 1.02), so the legacy mpv options (`deband=yes`, `profile=low-latency`) are kept; the renderer reaches the RTX 3050 through the GpuPreference value the service writes (DESK-12). Other engines in Phase 4. Use between Video themes: LoadSource without a restart, checked on the maintainer's machine on 2026-11-24 (D-26): ten switches on two monitors (release build) showed no flicker or resize jump, icons stayed visible, and no black frame was noticed (at most one too short to notice); each renderer logged ten source loads and none was respawned. Renderer memory stays level across 30 Uses (`MEASUREMENTS.md`). | 🟨 |
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
| Monitor changes, shell restart recovery (CrashExplorer) | ✅ | ? | ? | Behavior change (D-11): always on and event-driven (TaskbarCreated), no 5 s polling; legacy CrashExplorer is ignored. Monitor changes are debounced 2000 ms + 500 ms; after each applied change every surface is re-placed at its monitor's rectangle, changed or not, like the C# app, which re-places every engine window 2000 + 500 ms after each display change; only monitors that appeared or went away get a renderer started or closed. Killing explorer.exe does not send WM_DESTROY to the surfaces (observed in UAT; the cause is not established), so recovery relies on TaskbarCreated alone. After a shell restart a surface waits up to 10 s for the shell's wallpaper WorkerW and an event-driven order guard (WinEvent hook on explorer's Progman children) keeps it between the icons and the WorkerW; past 10 successful restores in a minute the guard holds back and arms a one-shot timer that looks at the order again when the oldest restore leaves the minute (so the end of a burst cannot leave the video hidden), it does not remove itself; waiting for the WorkerW keeps dispatching the renderer's messages. Display and shell messages are sent (not posted) messages; the service loop handles them at once. Until the new shell broadcasts TaskbarCreated, the old renderers keep playing into a surface that is not shown. A surface lives in its parent window's client coordinates (Progman on the raised desktop), so where it shows depends on where explorer keeps that window: every move is read back and judged against its target and the parent's client area (the renderer logs the parent rectangle, the local rectangle and a verdict, and warns with the virtual screen when the target lies partly or wholly outside the parent), and an event-driven location watch (WinEvent hook on the parent's location, limited to the thread that owns it; no polling, D-11) re-places the surface at its target whenever explorer moves or resizes that window. After a display change explorer moves its desktop window by itself; until it has, a monitor, or the part of it, that lies outside that window shows the static Windows wallpaper and no icons. In a run on the maintainer's machine (two primary switches and two origin-only changes) the location watch caught 8 of 8 moves of that window and every such episode ended within 1.73-2.11 s; an episode of about 90 s seen in UAT, after Windows applied two layouts in a row, did not recur, so its cause is not established. The surface logs each episode (a warning at its start, an info line at its end). Accepted limitation (G-02-7, D-031): Sucrose waits for explorer and shows no fallback window meanwhile, so the length of an episode is explorer's, not bounded by the app. | 🟨 |
| Hide desktop icons toggle | ✅ | ? | ? | Not stored as a setting in legacy. | ⬜ |
| Set OS wallpaper from thumbnail, disable slideshow | ✅ | ⚠️ | ⚠️ | | ⬜ |
| Lock screen wallpaper (Video and GIF themes, live on the lock and sign-in screen) | ✅ | ? | ? | Only on branch `lock-screen-native` (112de37f), not in `develop` (D-026). Legacy runs a credential-provider DLL inside LogonUI; behavior and its security problems in `lock-screen.md`. | ⬜ |
| Stay awake | ✅ | ✅ | ✅ | Legacy also runs elevated `powercfg`; avoid UAC if possible. M1 does not read StayAwake (not in the D-09 key list); mpv runs with stop-screensaver=no; the legacy mapping to stop-screensaver=always returns with the setting in Phase 3/4. | ⬜ |
| GPU preference (high performance) | ✅ | ❌ | ❌ | Registry `UserGpuPreferences`. Rust (Windows, pulled into Phase 2 by the UAT): before the first renderer starts, the service writes `GpuPreference=<n>;` for its own `sucrose.exe` (the renderers run the same exe) from legacy Engine.json `GraphicPreference` (Default 0, PowerSaving 1, HighPerformance 2; default HighPerformance), keeping the value's other pairs and skipping an unchanged value like the C# app. Like the C# app it also replaces a different choice the user made for `sucrose.exe` in Windows Settings > Display > Graphics at each service start, and with no `GraphicPreference` in Engine.json that means HighPerformance (parity behavior; honoring the user's own Settings choice is a maintainer decision, not made here); with `SUCROSE_LOG=debug` each renderer logs the adapter libmpv's Direct3D 11 device gets (read back from a device created the same way, with no adapter named; not done at the default level because it costs a Direct3D device). Capability `gpu_preference` (false on macOS and Linux). The value stays after the app ends; the uninstaller removes it (SVC-04). | ✅ |

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
| Showcase themes copied on first run | tracked in the new settings (migrated from Warehouse.json). The service copies every folder of `data_dir()/Showcase` whose name is not listed into the library on a worker thread when the Portal loads it; it never overwrites an existing folder, skips links and records a name only after the copy succeeded (a failed copy removes what it created). | 🟨 |
| Scan, validity rules, delete corrupt themes | Rust scan (`sucrose_core::library`) keeps the legacy rules and only reads. It reads missing optional SucroseInfo keys leniently, matches Type names in any case like Newtonsoft, and treats a Preview, Thumbnail or Source outside the theme folder as corrupt (new safety rule). Improvement (D-09): with Corrupt Themes on, corrupt themes go to the recycle bin / trash, not a permanent delete; their Cache/Properties file is deleted as before; locked or unreadable folders are skipped; only the service decides, after its own scan. | 🟨 |
| Sorting (name, creation, modification; asc/desc), search index | Ties keep folder-name order. Search lowercases with Unicode lowercase only (no Turkish folding) and orders results by the number of matched words. The Portal's search box takes at most 20 characters, filters the whole library 500 ms after typing stops (at once when emptied with the clear button or Escape), lowercased, and returns to page 1; a blank query shows everything; Ctrl+F focuses it; the query stays in the process (no telemetry, D-15). A changed sort setting re-sorts the open library at once. | ⬜ |
| Pagination, adaptive layout (margin, max per row) | Columns = floor((width + gap) / (260 + gap)), at least one, capped by the max per row; cards are at most 400 px wide and a capped row is centered. One page shows at a time (page size from `LibraryPagination`, a change returns to page 1). The pagination bar keeps the legacy scheme: Previous, up to nine page buttons (above six pages the first two and last two plus the pages around the current one, with "..." slots), Next, a numeric field that takes at most as many digits as the last page number has, and Jump (an empty field does nothing, a larger number goes to the last page); the bar hides at one page; a page change scrolls to the top. Empty library, no search match and an unreadable library folder have their own pages (Try Again rescans); Change Library Location joins them with the settings pages. | 🟨 |
| Card: use, review, customize, add/remove from cycle, find, share, delete (confirm), edit | Phase 3: left click, or Enter and Space on the focused card, is Use through the service (the Selected pill follows the service's broadcast, never the click; the last click wins); arrow keys move focus between cards. There is no "…" button or context menu until LIB-05 (D-18). Non-Video themes are shown with Use disabled until Phase 4 (D-17, temporary; the tooltip says "This wallpaper type is not supported yet."). A disabled card stays focusable and gives its reason as tooltip and accessible description, in this priority: service not connected, newer theme, non-Video, selected and running, fullscreen rule paused or closed. | 🟨 |
| Hover previews, hide details while previewing | Phase 3: with Library Preview on, a card the pointer rests on for 200 ms decodes its Preview GIF on a background thread at card size (at most 120 frames and 24 MiB, a zero frame delay plays at 100 ms) and plays it above the thumbnail; Hide Details hides the strip meanwhile. One preview lives at a time and its frames are dropped from the GPUI atlas when the pointer leaves, the page or search changes, the setting turns off or the window hides. A GIF that cannot be decoded never starts and only logs. Memory effect to be measured in the phase check. | 🟨 |
| Drag & drop: files, `.url`, archives, URLs | | ⬜ |
| Incompatible theme version handling | The card stays visible with a red PackageX pill and the legacy tooltip; Use is disabled and does nothing until the updater (Phase 5, D-18). | 🟨 |
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
| Window backdrop (Mica/Acrylic/…), background image with stretch and opacity | Windows backdrops; image works everywhere. Theme mode (Light/Dark/System) and backdrop (None/Mica/Acrylic/Tabbed) are two settings (D-21): the legacy ThemeType Dark/Light migrates to the mode, BackdropType Auto migrates to System + Mica, the other BackdropType names map one to one, and a fresh install is System + None. Rust: backdrops need Windows 11 22H2 (build 22621) and show disabled with a reason elsewhere (D-22); a stored backdrop the OS cannot show is applied as None and the setting is not rewritten; the background image with stretch and opacity is drawn by the Portal on every OS (macOS and Linux unverified by hand). | 🟨 |
| Light/dark, follow OS theme | | ⬜ |
| Header actions, search, status bar (memory, quotes with copy), command-line page arguments | Rust (Phase 3): search, light/dark, Premium, ⋮ Help/About (D-15); status bar memory = total of all Sucrose processes with a per-process tooltip, reported by the service once a second only while the Portal is connected (behavior change D-19; legacy: Portal process only); quote copy; command-line GeneralSetting and SystemSetting open General. | 🟨 |
| Settings pages: General, Personal, Performance, Wallpaper, System, Other (every key in `portal.md` §4) | | ⬜ |
| Donate page and advertising settings | Removed (D-023). | ➖ |
| Help and About dialogs, tray hint dialog | Rust (Phase 3): Help has five link rows; About has the author blurb, product name, version and release notes, contributors, terms, privacy and code of conduct links, without update status, partners or "Last Checked" until the updater (Phase 5); all links are plain https addresses fixed in the build and open in the OS browser (D-020, D-023). The tray hint shows on the first window close while `HintTrayIcon` is on and the tray icon is visible, with a static tray illustration instead of the legacy GIF and a 5 s countdown on Close; Close turns the hint off through the service and closes the window. Escape and a click outside do not dismiss the tray hint at all (legacy: blocked during the countdown only; the dialog layer fixes this setting when the dialog opens, so the Rust dialog cannot follow the countdown). Maintainer check of the dialogs is due at the end of Phase 3. | 🟨 |
| 25 languages, RTL for Arabic and Hebrew, live language switch | | ⬜ |
| Single instance | | ⬜ |

## Tray and app lifecycle (`services.md` §3)

| Feature | Win | mac | Linux | Notes | Status |
|---|---|---|---|---|---|
| Tray icon and menu (open, close/reload/change/customize wallpaper, settings, report, updates, exit) | ✅ | ✅ | ⚠️ GNOME needs AppIndicator | Grayscale icon while paused. M1: Quit only (TRAY-01); Phase 3 menu: Open Sucrose (also left double-click; disabled while the Portal is open), Close Applications or Exit by AppExit (D-14); the tray is an optional module: AppVisible off starts none and the Portal switch adds or removes it live (ARCH-03); labels are English until UI-06; a failed Open sets the tooltip to "Sucrose couldn't open its window" (E14); Exit with AppExit off removes only the tray (the wallpaper keeps playing and the D-13 exit rule decides about the service) and a later `sucrose` launch shows it again while AppVisible is on; before Phase 3: AppVisible false starts no tray (D-15); full menu is TRAY-02 (Phase 5); Quit stops only the Rust service and its renderers (D-14). The tooltip says what plays or why nothing does (theme rejected by the D-13 gate, libmpv missing). With AppVisible false only Task Manager stops the service in M1. After the renderers exit, Quit re-applies the current wallpaper (SPI_SETDESKWALLPAPER, no registry write) so no frame of the video is left. | 🟨 |
| Startup: normal, priority, scheduled task | ✅ | ✅ LaunchAgent | ✅ XDG autostart | | ⬜ |
| Close all Sucrose apps on exit | ✅ | ✅ | ✅ | Windows: renderers, Portal and service; only processes this build started or registered. Close Applications (AppExit on) stops the renderers, tells the Portal to close, waits about 2 s, ends a Portal that is still there only after the OS confirms its executable is the sibling `sucrose-portal`, then the service exits; it never touches a C# process (D-14). Unit-tested (the closing state and the executable check); the real tray click is checked by hand at the end of the phase. | 🟨 |

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
| Legacy `Setting/*.json` (14 files, `Properties` wrapper, enums by name) are migrated to the new versioned settings format on first run (D-007). Nine files migrate key by key into typed fields of `Setting/Sucrose.json` (versioned JSON, atomic writes, `.bak`, D-032): General, Portal, Engine, Backgroundog, Cycling, Hook, Library, Update, Warehouse; the legacy files are only read, never changed. Not migrated and never written (D-05): `Donate.json` (removed, D-023); `Objectionable.json` (a plaintext GitHub token, GitHub store server removed in D-020, the file is never opened); Portal `StoreServerType` (D-020); Aurora `AppProcessName` (runtime state); Kernel `Information` and the System interface lists `NetworkInterfaces`, `GraphicInterfaces`, `ProcessorInterfaces` (write-only system information); Backgroundog `TransmissionPort`, `PipeRequired`, `SignalRequired`, `TransmissionRequired`, `AudioRequired`, `PausePerformance`, `ClosePerformance` (runtime state written by the engines and Backgroundog) and `CommunicationType` (one IPC, D-009); Cycling `PassingTime` (runtime state of the cycling timer). The stale Backgroundog names `GpuUsage`, `CpuUsage`, `CpuPerformance` and `GpuPerformance` map to no field; the current names keep their own values. Date-time values (Warehouse `CefSharpTime`, `WebViewTime`, Update `Time`) stay the raw legacy text; storage types (`DownloadType`, `UploadType`, `LimitType`) stay non-empty strings; an engine outside the allowed set of its wallpaper type is the key's default. New key: `Backgroundog.FullScreenScope` (default `AllMonitors`). | `formats.md` §1 | 🟨 |
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