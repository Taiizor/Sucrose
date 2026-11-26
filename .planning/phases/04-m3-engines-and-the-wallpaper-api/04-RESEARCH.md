# Phase 4: M3 — Engines and the wallpaper API - Research

**Researched:** 2026-11-26
**Domain:** Windows desktop wallpaper engine: Chromium engines (CEF, WebView2), libVLC, built-in GIF and Media Foundation players, the C#-compatible web wallpaper JavaScript API, display modes, properties editor, performance governor, application embedding
**Confidence:** MEDIUM overall (HIGH for pins, crate versions, documented Win32/WebView2/CEF API facts; MEDIUM for integration behavior that needs a spike on the maintainer's machine, see the Assumptions Log)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Monitor placement and display modes (DESK-02..05)
- **D-01:** **Three modes stay, with the legacy enum names** `PerDisplay`, `SpanAcross` and `SameDuplicate`. **Per display now means each monitor holds its own wallpaper** (DESK-03). The legacy "Selected Screen Only" is the case where one monitor is filled and the others are empty; it is not a separate fourth mode. — **Reversibility:** costly — the per-monitor assignments become part of the versioned `Setting/Sucrose.json` schema, and the service orchestration and Display Preferences are built on them. Changing the model later needs a settings migration step.
- **D-02:** **Use applies to the target monitor** the user picks on the Display Preferences monitor map (legacy `ScreenDevice` semantics; an empty value auto-chooses the first screen, as in the legacy app).
  - Use asks no question per click.
  - Theme cards show which monitor(s) a theme runs on, for example a monitor number badge.
  - Display Preferences arrives now as the header action (Phase 3 D-15). It uses the legacy layout: a monitor canvas scaled from the monitor rectangles plus Placement Mode radios (`portal.md` DisplayPreferences).
  - A placement change restarts the affected renderers unless the wallpaper is closed or paused, as in the legacy app.
- **D-03:** **The migrated `PerDisplay` + `ScreenDevice` keep their legacy meaning.** The selected theme plays only on that monitor (the maintainer's is `\\.\DISPLAY1`), and the other monitors show the Windows wallpaper. Legacy defaults are kept (D-024).
  - The M1-only "same video on every monitor" behavior (Phase 2 D-01) ends here; record that in `docs/FEATURES.md`.
- **D-04:** **Duplicate runs one instance per monitor on every engine**, web engines included. Mirroring one instance was offered and declined.
  - **Time-based content follows the D-030 master clock**: Video, YouTube and GIF, also when they run as generated pages inside a web engine. Each engine follows the clock its own way:
    - generated web pages through JS (`currentTime` / `playbackRate` correction, seeks);
    - YouTube through `seekTo` / `getCurrentTime`;
    - the built-in GIF renderer by frame index;
    - mpv as in M1;
    - libVLC through its time API.
  - Only the master plays audio (Phase 2 D-02).
  - **Web and URL pages are synced on a best-effort basis where it is feasible**, for example by starting them together after load or aligning the `requestAnimationFrame` / time base. Research decides the technique. If it is not feasible, they run independently and `docs/FEATURES.md` records it.
  - The legacy Duplicate radio was disabled in the XAML, so record Duplicate as effectively new.
  - The same theme on two monitors in Per display mode follows the same sync and master-audio rule.
- **D-05:** **Different wallpapers on different monitors each play their own sound** at the global `WallpaperVolume`. The sound rules (PLAY-03 desktop focused, PLAY-04 others silent) apply to every one of them.
- **D-06:** **New, in this phase: a per-wallpaper volume through the properties editor.** Effective volume = global `WallpaperVolume` × per-wallpaper volume. Record it as "New" in `docs/FEATURES.md` and update the TYPE-04 and PROP-05 wording in `.planning/REQUIREMENTS.md`.
  - The built-in property sets (mpv, VLC and the web engines' generated Gif/Video/YouTube pages) gain a `volume` slider (0–100) next to their existing `mute`/`muted`.
  - A **Web theme with its own `SucroseProperties.json` keeps using that file unchanged** (no Sucrose volume is added).
  - A **Web theme without one, and every URL wallpaper, get a small Sucrose set (mute + volume)**. The editor shows it, and it is never written into the theme folder. URL had no properties in the legacy app.
  - As in the legacy app, a theme's own file still overrides the built-in set for mpv and VLC.
  - The volume is applied through the engine API or the renderer process's own audio session.
- **D-07:** **A monitor's wallpaper is removed with a Remove button for that monitor on the Display Preferences map**; the monitor then shows the Windows wallpaper again. Assignments of monitors that are not connected are listed there and can be removed.
- **D-08:** **Assignments are remembered by a stable monitor id**, not by `\\.\DISPLAYn`, which can change order. The id is the device interface path or an EDID-based identity, falling back to the device name when none is available.
  - A wallpaper comes back automatically when its monitor returns; until then the map shows the monitor as "not connected".
  - The maintainer tests with DISPLAY1 (physical) plus an intentional DISPLAY9 virtual display (VDD).
  - **Reversibility:** costly — the id format is stored in settings; changing it needs a migration step.
- **D-09:** **Span keeps the legacy union rectangle**: one renderer covers the union of all monitors. Stretch (DESK-05, mapped per engine as in `engines.md` `3) and the working-area / display-bounds choice (DESK-04) apply to that rectangle.

### Engines, defaults and web engine behavior (ENG-02..07, WAPI-07..10, SEC-02)
- **D-10:** **Fresh-install engine defaults stay as in the legacy app (D-024)**: Web, URL and YouTube on CEF, Gif and Video on mpv, Application on the native embedder. Migrated choices are kept (the maintainer's: Web/URL/YouTube CEF, Gif/Video mpv).
  - Legacy engines that no longer exist map to their replacements (D-006): `VexanaLive` and `XavierLive` to the built-in GIF renderer, `NebulaLive` to built-in OS video. Record the mapping in `docs/FEATURES.md`.
  - Measure the built-in GIF renderer against mpv playing the same GIF (RAM/CPU, release build) and record the numbers in `docs/MEASUREMENTS.md`.
- **D-11:** **Fallback (ENG-07) notifies the user and keeps the setting.** The fallback applies to the current run only.
  - It is reported through a Portal toast with an install or repair hint (when the Portal is open), the tray tooltip and the log.
  - The engine's settings row shows a persistent warning.
  - Once the missing piece is installed, the next start goes back to the chosen engine.
  - The fallback order is Claude's discretion: the other web engine for web types; for Gif/Video, the built-in engine or another installed one.
- **D-12:** **Web engines validate TLS certificates by default.** This is a behavior change: the legacy CEF `IgnoreCertificateErrors` and WebView2 `AlwaysAllow` accepted everything.
  - The web engine settings get a new **"Allow invalid certificates"** switch (default off) that restores the legacy behavior.
  - Record it in `docs/FEATURES.md` and in a `docs/DECISIONS.md` entry that relates it to D-017 and D-021 (the Chromium switches themselves stay user-controlled).
- **D-13:** **Developer mode (WAPI-09) never opens DevTools by itself.** With per-monitor instances, the legacy auto-open would open one DevTools window per monitor.
  - Developer mode enables the context menu (Inspect) and the keyboard shortcuts.
  - With `DeveloperPort > 0`, it also enables the remote debugging port with the legacy switches.
  - The Portal's developer settings offer **"Open DevTools"** for a running web wallpaper (pick the monitor) and an `edge://inspect` / `chrome://inspect` hint.
  - The maintainer has `DeveloperMode` on and `DeveloperPort` 0. Record the change in `docs/FEATURES.md`.
- **D-14:** **Both web engines serve local pages from one fixed virtual HTTPS origin, `https://sucrose.wallpaper.engine/`**, shared by all themes. No local port is opened: WebView2 uses virtual host mapping or a resource-request handler, CEF a scheme handler.
  - Requests look in the theme folder first, then in the generated-content folder (the legacy CEF lookup order).
  - **The path is normalised and anything outside these folders is rejected (SEC-02)**, proven with fixture tests that try traversal (`..`, percent-encoded forms, absolute and UNC paths).
  - The YouTube `origin` playerVar then matches the real origin on both engines; research checks whether the `youtube.com` Referer injection (`engines.md` `8) is still needed on both.
  - Each engine gets its own Rust-owned user data folder, not the C# `Cache/WebView2` / `Cache/CefSharp`.
  - **Reversibility:** one-way — after release, the localStorage, IndexedDB and cookies of every web wallpaper are bound to this origin; changing it loses that data. **Confirmed by the maintainer on 2026-11-25: one shared origin `https://sucrose.wallpaper.engine/` for all themes (per-theme subdomains were offered and declined; `--disable-web-security` is on by default, so origin isolation would add no security).**
- **D-15:** **Prerequisite detection and install (ENG-06):** the WebView2 Runtime must be newer than 131.0.2903.70 and the VC++ Redist at least 14.40.33816.0 for the right architecture (legacy thresholds and URLs, `engines.md` `9). The bundled engine binaries (libmpv, libVLC, CEF) are checked as present and loadable, with a repair hint.
  - **Install** downloads the official Microsoft bootstrapper and **verifies that its Authenticode signature chains to Microsoft before running it** (D-017, D-019).
  - It asks the user to confirm, leaves UAC to Windows, re-detects afterwards and switches to the chosen engine.
- **D-16:** **The fallback toast has a "Don't remind me for a day" snooze** that uses the migrated Warehouse `WebViewTime` / `CefSharpTime` fields (Phase 3 D-06). The persistent warning on the settings row is never snoozed.
- **D-17:** **Engine arguments (WAPI-10, D-021) have one editor per web engine**, under that engine's row on the Wallpaper page.
  - Each row is a switch with an optional value. Rows can be added and removed, and **Reset** restores the legacy defaults.
  - Sucrose converts between the CEF dictionary (`CefArguments`) and the WebView2 list (`WebArguments`).
  - Switches Sucrose adds itself (hardware acceleration, the debugging port, developer mode) are shown read-only and are not stored.
  - Unknown switches are not blocked, only logged.
  - **When saved while web wallpapers on that engine are running, the editor asks "Restart now / Later"**; Later applies the change on the next start.

### Properties editor (PROP-01..05, FMT-03)
- **D-18:** **The editor is a separate small window opened by the Portal process**: a second GPUI window, no new executable (D-008).
  - Like the legacy `Sucrose.Property`, it sits next to the taskbar and respects the taskbar edge, DPI and RTL, so the wallpaper stays visible while editing.
  - It stays open while the Portal window is minimized and releases its resources when it closes.
- **D-19:** **Entry points in Phase 4:**
  - A **Customize icon appears on hover on cards of themes that have properties.** It follows the legacy Customize enable rule (`portal.md`), extended by D-06: every Web and URL theme now has at least the Sucrose volume set.
  - The Display Preferences map has **Customize** next to each running monitor.
  - There is still no "…" button and no context menu (Phase 3 D-18); LIB-05 later calls the same action.
- **D-20:** **The C# property copies are copied once into a Rust-owned folder and the C# files are never written.**
  - When `<data_dir>/Properties` does not exist, every working copy `Cache\Properties\<Theme><Type>` is copied there with the same file name. The source is always the real `%APPDATA%\Sucrose\Cache\Properties`, whatever `SUCROSE_DATA_DIR` says (Phase 2 D-08 / Phase 3 D-03 rule).
  - Delta files (`<guid>.<Theme><Type>`) are skipped.
  - `PropertiesType` names stay (`Base`, `WebView`, `CefSharp`, `MpvPlayer`, `VlcPlayer`) so the copies keep working.
  - The maintainer has 34 copies.
  - **Reversibility:** one-way — every later Rust version reads this folder; moving it needs another migration. **Confirmed by the maintainer on 2026-11-25: `%APPDATA%\Sucrose\Properties` (outside `Cache`, so the Phase 5 cache cleaning never deletes customizations), file names `<Theme><Type>` as in the legacy app.**
- **D-21:** **Theme property texts use the migrated `Culture`** (the maintainer's is `TR`) to pick `PropertyLocalization` entries, falling back to the first culture as in the legacy app. PROP-03 is proven in this phase. Sucrose's own editor strings (Restore Default, Refresh, Delete Copy and others) stay English until UI-06 in Phase 5, through rust-i18n keys named after the legacy `Property.*` keys.

### Performance rules and pausing (PERF-01..04, PERF-06..09, PLAY-05)
- **D-22:** **Light and Heavy are two real engine-level levels.** Neither uses `NtSuspendProcess` or a debugger attach (PLAY-05).
  - **Light** pauses playback with an instant resume: mpv/VLC pause, `visibilitychange`/hidden plus `video.pause()`, YouTube `pauseVideo()` and `SucrosePause()` on web pages, and pause in the built-in GIF and OS video engines.
  - **Heavy** also releases resources: WebView2 `TrySuspend`, CEF `WasHidden` plus stopped painting and throttled timers (CDP where needed). Research finds the equivalent for mpv, VLC and GIF that keeps the position.
  - The migrated `PausePerformanceType` is kept (the maintainer's is Light).
  - This supersedes the Phase 2 discretion "Heavy is handled like Light"; update `docs/FEATURES.md`.
- **D-23:** **Remote desktop and VM detection (PERF-09) uses OS APIs plus the legacy lists.**
  - RDP and console switches come from the WTS session API and session-change notifications.
  - Third-party remote tools (the legacy 52-entry list) and VM/emulator apps (the legacy 40-entry list) are matched through **process start and stop events, not the legacy 2 s full process scan**, and only while the rule is not `Resume`.
  - The lists are kept as data with tests.
- **D-24:** **Data collection is decoupled and on demand (D-008).**
  - The performance rules run only when `PerformanceCounter` is on and at least one rule is not `Resume`, and they read only the signals the active rules need.
  - Web wallpaper data is collected only while a page with `SucroseCompatible.json` runs, and only for the groups it defines (`SystemBios` … `SystemMotherboard`).
  - The audio spectrum is captured only when `SystemAudio` is defined.
  - Push cadences follow the legacy app (`services.md` `1.2–1.3).
  - **`PerformanceCounter` off now disables only the rules, not the web data.** This is a behavior change; record it.
  - There is no `CommunicationType`: one IPC (D-009, Phase 3 D-05).
- **D-25:** **Condition evaluation keeps the legacy semantics.**
  - Conditions are evaluated in the legacy order (Lock, Focus, Sleep, Memory, Remote, Battery, Console, Graphic, Network, Session, Virtual, Processor, FullScreen, ScreenSaver, BatterySaver), and **the first match wins**.
  - **While a condition is active, only its recovery is evaluated.**
  - The legacy debounce applies: 5 polls to trigger and 3 to recover on 1 s polls, 0/0 for the immediate conditions. Events may drive the predicates, but the timings stay legacy.
  - PERF-07 fix: a network sub-check counts only when its threshold is above 0 (ping is no longer checked first at 0).
  - The maintainer considered "the strongest action wins" and chose the legacy rule.
  - The FullScreen action scope (affected monitor or all, Phase 2 D-05) gets its UI here and applies to FullScreen only.
- **D-26:** **The settings for this phase go in their legacy places (Phase 3 D-16 pattern), with only this phase's keys**: the Wallpaper and Performance pages in the legacy order (`portal.md` Settings). Volume and developer keys go wherever the legacy Portal had them.
  - Engine choice rows show installed/missing with the hint (D-11).
  - New items sit under the row they belong to: the argument editors (D-17), the certificate switch (D-12), "Open DevTools" (D-13) and the fullscreen scope (D-25).
  - `CommunicationType` is not shown.
- **D-27:** **The tray shows rule state**: the icon turns grey while a rule pauses or closes the wallpaper, and the tooltip names the condition and the action (for example "Paused: Processor usage"). The Portal's Use suffix " (Paused)" / " (Closed)" (Phase 3 D-23) extends to every condition.

### Application wallpapers (TYPE-06)
- **D-28:** **The number of processes follows the mode**: one per assigned monitor in Per display, one per monitor in Duplicate, one over the union rectangle in Span.
  - Each is embedded on its own target with real window handles, an `HWND`, never the process handle (legacy bug).
  - Update the TYPE-06 wording ("one process per screen in every mode") in `.planning/REQUIREMENTS.md` and `docs/FEATURES.md`.
- **D-29:** **Window selection is generic, with legacy compatibility.**
  - The embedded window is the first visible top-level main window of the app's process tree. It is waited for through window-creation events with a timeout instead of the fixed 3 s, and the class `Engine` is preferred when present.
  - **The legacy "Play!" `BM_CLICK` is kept for compatibility** when that launcher dialog appears.
  - If no window appears before the timeout, the process is closed and the reason goes to the log and the tray.
  - As in the legacy app, `Info.Source` must be a local exe inside the theme folder.
- **D-30:** **`Arguments` stays one quoted argument, as in the legacy app**, on purpose for existing themes. Record it as a kept legacy quirk in `docs/FEATURES.md`.

### Delivery order (Mode: mvp)
- **D-31:** **The phase is delivered in vertical slices.** Each slice ends with a working wallpaper and a maintainer checkpoint on their machine:
  1. Engine abstraction (`sucrose-engine` trait), detection and per-type choice with fallback (ENG-06/07), CEF bundled through `cargo xtask fetch-engines`, the virtual origin (D-14) and the JS API (`PropertyListener`, `SucroseCompatible.json`, system data). Proven with the maintainer's 8 Web themes and fixture pages.
  2. WebView2.
  3. Built-in GIF/APNG/WebP and the generated pages (WAPI-08), URL, YouTube.
  4. Display modes and per-monitor wallpapers (DESK-02..05) with Display Preferences.
  5. Properties editor (PROP-*).
  6. Performance rules and sound rules (PERF-*, PLAY-*).
  7. libVLC and Media Foundation.
  8. Application wallpapers.

  The Phase 3 D-17 rule ("Use disabled for non-Video themes") is lifted per type as each slice lands.

### Claude's Discretion

- Crate layout under `docs/ARCHITECTURE.md`: the `sucrose-engine` trait and one crate per engine (for example `sucrose-engine-web` with WebView2 and CEF backends, `-vlc`, `-gif`, `-mf`), `sucrose-sysinfo` for data collection. Where the performance governor lives (a service module that does not start when off, D-008).
- IPC messages for engine choice, placement, properties, compatible data and rule state (append-only, `PROTOCOL_VERSION` / `PORTAL_PROTOCOL_VERSION`). Which process writes the property working copy: the service as single writer (Phase 3 D-11 spirit) or the Portal.
- CEF version pin and `xtask/engines.toml` entries for CEF and libVLC per architecture (x86, x64, ARM64; D-027), `bindgen` for libVLC (as `bindgen_mpv.rs`), the `cef` crate and `webview2-com` (D-013). License texts and source offers follow D-028.
- Built-in GIF/APNG/WebP renderer technique (GPU or CPU presentation, frame decoding and memory bound) and the Media Foundation player, within the low-resource rule.
- The best-effort web/URL sync technique (D-04) and the sync tolerance for each time-based engine.
- The stable monitor id source (D-08) and the settings shape of per-monitor assignments.
- Fallback order for each type (D-11) and how "chosen engine is back" is detected.
- Live-apply cadence in the editor (for example throttled updates while a slider drags, debounced file write), the colour picker control, `filedropdown` copy and delete rules (legacy access and lock checks), Restore Default / Refresh / Delete Copy semantics as in the legacy app.
- Input forwarding (WAPI-07) through the legacy `InputType`, `InputModuleType` (RawInput) and `InputDesktop` semantics.
- PLAY-04 "others silent" detection: audio session notifications or the legacy poll at `VolumeSilentSensitivity` seconds (the maintainer's is 10).
- Generated pages (`GifContent.html`, `VideoContent.html`, `YouTubeContent.html`) reproduced from the legacy ones for FMT-07 compatibility; fixture pages for the JS API.
- Test themes for URL, YouTube (3 in Showcase) and Application (none locally; a small test app or Unity build). They are prepared under `SUCROSE_DATA_DIR`, never in the maintainer's library.
- Per-engine RAM/CPU measurements from release builds against the C# engines (`cargo xtask measure`), recorded in `docs/MEASUREMENTS.md`.

### Deferred Ideas (OUT OF SCOPE)

- Decoding once and sharing frames across monitors instead of one decode per renderer (raised in Phase 2): still deferred. The maintainer chose one instance per monitor for Duplicate on every engine (D-04).
- Considered and not chosen, kept for the record:
  - mirroring one web instance across monitors;
  - per-monitor volume levels;
  - per-theme web origins;
  - "strongest action wins" for overlapping performance conditions;
  - a separate Engines settings page.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TYPE-02 | User can set a GIF theme, drawn by Sucrose's built-in renderer, which also plays APNG and WebP (Win ✅ · mac ✅ · Linux ✅) | Built-in GIF/APNG/WebP renderer (Findings 4, slice 3): lazy frame decode via the workspace `image` crate, Direct2D presentation, frame-index clock following |
| TYPE-03 | User can set a Web theme (local HTML) on a web engine: WebView2/CEF on Windows, WKWebView/CEF on macOS, WebKitGTK/CEF on Linux (Win ✅ · mac ✅ · Linux ✅) | CEF (slice 1) and WebView2 (slice 2) host Web themes on the virtual origin (Findings 1, 2, 6) |
| TYPE-04 | User can set a URL wallpaper; it has no properties and receives forwarded input (Win ✅ · mac ✅ · Linux ✅) | URL page on both web engines, Sucrose mute+volume set (D-06), input forwarding (Findings 6, 8; slice 3) |
| TYPE-05 | User can set a YouTube wallpaper (IFrame API page, no API key) with playlist, shuffle, loop and hidden player UI (Win ✅ · mac ✅ · Linux ✅) | Generated YouTubeContent.html, Referer check (Findings 12; slice 3) |
| TYPE-06 | User can set an Application wallpaper that embeds another app's window, with one process per screen in every mode and real window handles (legacy bugs fixed) (Win ✅ · mac ❌ · Linux ⚠️ X11 only) | Application embedder: window-creation events, SetParent of a real HWND, process per target (Findings 10; slice 8) |
| ENG-02 | The WebView2 engine hosts Web, URL and YouTube wallpapers with the C#-compatible JavaScript API | webview2-com 0.39.1 backend (Findings 2; slice 2) |
| ENG-03 | The CEF engine hosts the same wallpapers with the same JavaScript API | cef 156.7.0+156.0.31 backend (Findings 1; slice 1) |
| ENG-04 | The libVLC engine plays video when libVLC is installed | libVLC 3.0.24 pinned archives, own bindgen + libloading (Findings 3; slice 7) |
| ENG-05 | The built-in OS video engine (Media Foundation on Windows) plays video with no engine installed (D-006) | IMFMediaEngine in HWND mode (Findings 5; slice 7) |
| ENG-06 | Sucrose detects the engines and prerequisites it needs (bundled engine binaries present and loadable, WebView2 Runtime version, VC++ Redist for CEF) and shows install or repair hints; the WebView2 Runtime and VC++ Redist are user-side downloads (D-028) | Detection table and install path with Authenticode check (Findings 2, Security Domain; slice 1, 2) |
| ENG-07 | User can choose, per wallpaper type, which installed engine plays it (legacy `Engine.json` keys Gif/Url/Web/Video/YouTube/Application). If the chosen engine is not installed, Sucrose falls back to the built-in engine or another installed one and tells the user (D-006, D-028) | Pure `choose_engine` function plus fallback toast/row alert/snooze (Pattern 1; slice 1) |
| DESK-02 | User can show a wallpaper per display, spanned across displays or duplicated; duplicate works for every engine (Win ✅ · mac ✅ · Linux ⚠️) | Per-monitor instances for every mode, span union rectangle, Duplicate sync (Findings 7, 11; slice 4) |
| DESK-03 | **New:** user can put a different wallpaper on each monitor, one renderer per monitor (Win ✅ · mac ✅ · Linux ⚠️) | `Engine.DisplayAssignments` keyed by stable monitor id (Findings 7; slice 4) |
| DESK-04 | User can choose the working area or the full display bounds (Win ✅ · mac ✅ · Linux ⚠️) | Existing `ScreenArea` plus per-engine rect handling (Findings 7; slice 4) |
| DESK-05 | User can choose a stretch mode (None, Fill, Uniform, UniformToFill), mapped per engine as in `engines.md` `3 (Win ✅ · mac ✅ · Linux ✅) | Stretch mapping table per engine (Findings 7; slices 1-4, 7) |
| PLAY-01 | User can loop and shuffle playback (Win ✅ · mac ✅ · Linux ✅) | Loop/shuffle per engine (engines.md section 4; every slice) |
| PLAY-02 | User can set a global wallpaper volume (Win ✅ · mac ✅ · Linux ✅) | Global volume x per-wallpaper volume (D-06) (Findings 9; slice 6) |
| PLAY-03 | User can limit sound to when the desktop is focused (Win ✅ · mac ⚠️ · Linux ⚠️) | Desktop-focused rule through the foreground window (Findings 9; slice 6) |
| PLAY-04 | User can limit sound to when other apps are silent (legacy scans every process's audio session every N seconds) (Win ✅ · mac ? · Linux ⚠️ PipeWire) | IAudioSessionManager2 enumeration of the default render endpoint instead of a process scan (Findings 9; slice 6) |
| PLAY-05 | User can pause and resume wallpapers through engine-level pause (page visibility, CDP, WebView2 TrySuspend) instead of the legacy debugger-attach and `NtSuspendProcess` hacks (Win ✅ · mac ✅ · Linux ✅) | Light/Heavy as engine calls, no NtSuspendProcess (Pattern 4; slice 6, engines earlier) |
| PERF-01 | User can set Close, Pause or Resume per condition; a condition triggers after 5 consecutive 1 s polls and recovers after 3 (some immediately), with Light and Heavy pause | Governor generalising fullscreen.rs: legacy order, first match wins, 5/3 debounce (Findings 9; slice 6) |
| PERF-02 | Session lock, sleep, user switch and log out trigger the configured action (default Close) (Win ✅ · mac ✅ · Linux ⚠️) | WM_WTSSESSION_CHANGE and power broadcasts (Findings 9; slice 6) |
| PERF-03 | The screen saver triggers the configured action (default Pause) (Win ✅ · mac ✅ · Linux ⚠️) | Screen saver poll at 1 s (Findings 9; slice 6) |
| PERF-04 | The desktop losing focus (Focus) triggers the configured action (Win ✅ · mac ⚠️ · Linux ⚠️) | Focus rule from the foreground window (Findings 9; slice 6) |
| PERF-06 | CPU, memory and per-adapter GPU usage thresholds trigger the configured action (Win ✅ · mac ✅ · Linux ✅) | PDH counters, GlobalMemoryStatusEx, only for non-Resume rules (Findings 9; slice 6) |
| PERF-07 | Network ping, upload and download thresholds trigger the configured action, without the legacy bug that checks ping first even at threshold 0 (Win ✅ · mac ✅ · Linux ✅) | Network sub-check only above 0 (Findings 9; slice 6) |
| PERF-08 | Battery level and battery saver trigger the configured action (Win ✅ · mac ✅ · Linux ✅) | GetSystemPowerStatus incl. SystemStatusFlag, power-setting notification (Findings 9; slice 6) |
| PERF-09 | Remote desktop and virtual machine/emulator sessions trigger the configured action, detected through OS session APIs where possible instead of process-name lists (Win ✅ · mac ? · Linux ?) | WTS remote events plus 52/40-entry lists with a process-event fallback decision (Findings 9, Open Question 4; slice 6) |
| WAPI-01 | Web wallpapers receive `PropertyListener` property calls, including `filedropdown` file lists, formatted by the strings in `SucroseProperties.json` (Win ✅ · mac ✅ · Linux ✅) | PropertyListener builder in sucrose-engine-web (Findings 6; slice 1) |
| WAPI-02 | `SucroseCompatible.json` templates (16 keys) work, keeping legacy `True`/`False` booleans and indented JSON (Win ✅ · mac ✅ · Linux ✅) | Compatible template formatter with True/False and indented JSON (Findings 6; slice 1) |
| WAPI-03 | Web wallpapers receive date, CPU, memory, GPU, network, storage and battery data with the legacy payload keys (`formats.md` `5, `engines.md` `6b) (Win ✅ · mac ✅ · Linux ✅) | Service-side collectors and ordered push (Findings 6; slices 1, 6) |
| WAPI-04 | Web wallpapers receive BIOS and motherboard data (Win ✅ · mac ⚠️ · Linux ⚠️) | WMI Win32_BIOS and Win32_BaseBoard once (Findings 6; slice 1) |
| WAPI-05 | Web wallpapers receive the audio spectrum: 128 logarithmic bands, left/right/mono (`services.md` `1.2) (Win ✅ · mac ⚠️ permission · Linux ✅) | WASAPI loopback + realfft visualizer (Findings 6; slice 2) |
| WAPI-06 | Web wallpapers receive now-playing data (media session with thumbnail) (Win ✅ · mac ❌/⚠️ · Linux ✅ MPRIS) | GlobalSystemMediaTransportControls + chunked thumbnail (Findings 6; slice 2) |
| WAPI-07 | Mouse and keyboard input is forwarded to Web, URL and Application wallpapers (Win ✅ · mac ⚠️ · Linux ⚠️ X11 / ❌ most Wayland) | RawInput on the renderer message window, legacy PostMessage targets (Findings 8; slice 1, 2) |
| WAPI-08 | GIF, Video and YouTube themes can run as generated pages in web engines, with their property keys (Win ✅ · mac ✅ · Linux ✅) | Generated pages reproduced from the legacy ones with sync functions added (Findings 6, 11; slice 3) |
| WAPI-09 | User can turn on developer mode with a remote debugging port (Win ✅ · mac ✅ · Linux ✅) | Developer mode switches, per-slot debugging port, Open DevTools message (Findings 1, 2; slices 1, 2) |
| WAPI-10 | **New:** user can add, remove and reset web engine arguments (Chromium switches) in the UI; legacy switches stay the defaults (D-021) (Win ✅ · mac ✅ · Linux ✅) | Argument editor model converting CefArguments dictionary and WebArguments list (Findings 1; slices 1, 2) |
| PROP-01 | The properties editor offers label, button, slider, textbox, checkbox, dropdown, numberbox, passwordbox, filedropdown and colorpicker controls | Ten control types in the editor window (Findings 13; slice 5) |
| PROP-02 | Property changes apply live to the running wallpaper over IPC (replacing legacy watcher files) | Delta over IPC, service as single writer (Pattern 6; slice 5) |
| PROP-03 | Property texts appear in the user's language (`PropertyLocalization`) | Culture from migrated settings, fall back to first culture (slice 5) |
| PROP-04 | User can restore defaults, refresh, and delete the property copy | Restore Default / Refresh / Delete Copy semantics from services.md section 6 (slice 5) |
| PROP-05 | Built-in property sets exist for web pages, mpv (raw mpv property names) and VLC | Built-in sets for web pages, mpv, VLC, plus the Sucrose mute+volume set (D-06; slice 1, 3, 5, 7) |
| FMT-03 | `SucroseProperties.json` and `SucroseCompatible.json` are read and written compatibly, proven with fixtures | New `properties` and `compatible` modules in sucrose-core with real-file fixtures (slice 1) |
| FMT-07 | The web wallpaper JavaScript API stays compatible (`engines.md` `6), proven with fixture pages | Fixture pages under crates/sucrose-engine-web/tests/fixtures (Validation Architecture; slice 1) |
| SEC-02 | Local file serving for web wallpapers rejects paths outside the theme folder (D-017) | One pure origin resolver shared by CEF and WebView2 with traversal vectors (Pattern 2; slice 1) |
</phase_requirements>

## Summary

Phase 4 turns a one-engine (libmpv) Windows renderer into a multi-engine one. The research found that every locked decision (D-01..D-31) is implementable on the pinned stack, with five hidden costs the planner must schedule explicitly. (1) The `cef` crate links `libcef` through an import library at build time and its build script downloads and compiles CEF with CMake and Ninja; that collides with the always-on cost of the service (D-008) and with `.claude/rules/cargo.md`, so slice 1 starts with a link-model spike (delay-load of `libcef.dll`, fallback: a separate helper exe). (2) CEF takes a process-singleton lock on its cache root (CEF 120+), so one CEF browser process per monitor (D-04) cannot share the one persistent profile that D-14 implies; WebView2 shares one browser process (and one audio service) per user data folder, which makes the per-wallpaper volume (D-06) unsettable per monitor. Both need one profile policy (Open Question 2). (3) WebView2 virtual host mapping maps one folder per host, is documented as slow for media and limited to MAX_PATH, so D-14 is best served on both engines by one Rust request resolver (SEC-02 lives in one tested function) behind CEF's scheme handler and WebView2's WebResourceRequested. (4) `WasHidden` in CEF only applies to windowless rendering; Heavy pause for a windowed CEF browser must hide the HWND and freeze the page through the DevTools protocol. (5) IPC frames are capped at 64 KiB, so now-playing thumbnails and property documents need a chunked blob message.

The delivery order of D-31 is sound. Slice 1 is by far the largest (engine trait, IPC v4, CEF, virtual origin, JS API, system data, prerequisite detection); the guidance below splits it into sub-steps so each ends green. The pure parts (engine choice, origin resolver, JS formatting, Compatible/Properties models, sync controller, governor state machine, property sets) are cross-platform Rust with no unsafe and carry most of the test weight; the OS parts are thin and are proven on the maintainer's machine per slice.

**Primary recommendation:** Build the pure cores first (`sucrose-engine`, `sucrose-engine-web` with the origin resolver and JS builders, `sucrose-core` properties/compatible models) and prove CEF's link model in the first plan; use the pinned CEF 156.0.31 and libVLC 3.0.24 archives through `cargo xtask fetch-engines` and set `CEF_PATH` so no build script ever downloads an engine.

## Architectural Responsibility Map

The tiers are the processes of D-008 plus the platform layer (this is a desktop app, not a browser/API/CDN app).

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Engine detection, choice per type, fallback (ENG-06/07) | Service (pure `choose_engine` over availability reports) | Portal (rows, toast, install dialog) | The service owns settings (D-032) and starts renderers; the Portal only displays. |
| Hosting a page / video / GIF / app window | Renderer process (engine backend on the surface thread) | Platform (desktop surface HWND) | One renderer per monitor (D-04); crash isolation (D-008). |
| Virtual origin and path resolution (D-14, SEC-02) | `sucrose-engine-web` pure resolver | CEF scheme handler / WebView2 WebResourceRequested glue in the renderer | One tested function, two thin adapters. |
| JS API formatting (WAPI-01/02) | `sucrose-engine-web` (pure) | Renderer executes the script | Output must be byte-identical to the legacy format strings. |
| System / BIOS / audio / now-playing data (WAPI-03..06) | Service (one collector per group, only while a page needs it, D-24) | Renderers (receive over IPC, call the page) | Collect once for N monitors; renderers stay light. |
| Performance rules (PERF-*) | Service governor module (does not exist when all rules are Resume) | Renderer applies Pause/Close | Same place as the fullscreen rule today. |
| Pause, volume, loop, stretch, clock following | Engine backend in the renderer | Service decides policy and master (D-030) | Only the engine can pause itself without NtSuspendProcess (PLAY-05). |
| Properties working copy and live delta (PROP-*) | Service (single writer) | Portal editor window (UI), renderer (apply) | D-032 / Phase 3 D-11 pattern. |
| Display assignments and modes (DESK-*) | Service (topology + settings) | Portal Display Preferences | Assignments are settings (D-01). |
| Input forwarding (WAPI-07) | Renderer (RawInput on its own message window) | Platform | Only the renderer knows its surface and engine HWNDs. |
| Application embedding (TYPE-06) | Renderer (one per target) | Platform (SetParent, WinEvent hook, job object) | Real HWNDs, process per target (D-28). |
| Prerequisite install (ENG-06) | Service worker thread (download, Authenticode check, run) | Portal (confirm dialog) | Service already runs the library worker pattern; UAC stays with Windows. |

## Project Constraints (from CLAUDE.md)

- Parity: every C# feature is tracked in `docs/FEATURES.md`; record every behavior change there (D-03, D-04, D-06, D-12, D-13, D-22, D-24, D-28 and the engine-name mapping all need rows).
- Compatibility (D-007): themes, `SucroseProperties.json`, `SucroseCompatible.json` and the JS API stay compatible; prove with fixtures from real files.
- No web runtime in the core (D-004); engine binaries ship in the package, pinned and hash-verified by `cargo xtask fetch-engines`, loaded at runtime (D-028, D-013).
- OS-specific code only in `sucrose-platform` and engine/FFI crates; `unsafe` only there, each block with `// SAFETY:` (D-011).
- Optional modules do not start when disabled (D-008); measure RAM/CPU in release builds and record notable numbers in `docs/MEASUREMENTS.md`.
- Do not reopen Accepted decisions; confirm Proposed ones before expensive or hard-to-undo work.
- Rust 2024, `rustfmt` and `clippy -D warnings`; English code, comments, docs, commits. Commands: `cargo build/test/clippy/fmt --workspace`, `cargo xtask ci`.
- Environment: `cargo` in WSL is the Windows toolchain (Windows binaries only); no WSL-only scripts, no `/mnt/...` paths or symlinks in committed files; automation goes in `xtask`; GUI runs use `SUCROSE_DATA_DIR` scratch (+ `WSLENV=SUCROSE_DATA_DIR`), never synthesize input, close what you start; the maintainer's settings/library/store cache are read-only unless the task writes them.
- GSD: start edits through a GSD command; workflows may commit planning files; pushing needs the maintainer.
- `.claude/rules/cargo.md`: versions only in root `[workspace.dependencies]`; crate must be maintained, widely used, GPL-3.0 compatible (cargo-deny); do not add "any crate that links an engine runtime at build time"; engine binaries come from `xtask/engines.toml` pins "never from a crate". **Tension:** D-013 (Accepted) names the `cef` and `webview2-com` crates, and `cef-dll-sys` links `libcef` at build time and downloads CEF by itself unless `CEF_PATH` is set. See Open Question 1.
- `.claude/rules/platform-ffi.md`: use the `windows` crate, never hand-write declarations it provides; missing engine is a normal state with a repair hint; every OS feature behind `Platform`/`Capabilities` with macOS/Linux stubs compiling.
- `.claude/rules/rust.md`: `thiserror` in libs, `anyhow` in bins, no `unwrap()` on I/O/IPC/FFI, `tracing`, doc comments on public items, no OS code in `sucrose-core`, tests next to code, fixtures from real files under `tests/fixtures/`.
- Memory notes that bind this phase: no test-only app flags (test apps are separate crates/xtask); maintainer checkpoints are prepared in Turkish and the clock starts only after "start"; UI-gate and reversibility-gate behaviors of the GSD checker (confirm one-way decisions before the planner; D-14 and D-20 are already confirmed).

## Standard Stack

### Core (new for this phase)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cef` | 156.7.0+156.0.31, `default-features = false` | CEF bindings and wrapper (ENG-03) | D-013 names it. Latest on crates.io (2026-11-19), license Apache-2.0 OR MIT; the version after `+` is the CEF build it matches. [VERIFIED: crates.io API this session] |
| `webview2-com` | 0.39.1 | WebView2 COM bindings (ENG-02) | D-013 names it. Depends on `windows ^0.62`, the workspace's 0.62.2, so no second `windows` copy; MIT; ships `WebView2LoaderStatic.lib` for x86, x64 and arm64. [VERIFIED: crates.io API + wravery/webview2-rs tree] |
| libVLC (binary) | 3.0.24 (`vlc-3.0.24-win32.7z`, `-win64.7z`, `-winarm64.7z`) | libVLC engine (ENG-04) | Same 3.0.24 line the legacy app used; official VideoLAN builds exist for all three Windows architectures. [VERIFIED: download.videolan.org index, fetched] |
| `realfft` | 3.5.1 | 4096-point real FFT for the audio visualizer (WAPI-05) | Thin layer over `rustfft` 6.4.1; legacy used MathNet FFT with a Hann window. [VERIFIED: crates.io API] |
| `wmi` | 0.18.4 | BIOS, motherboard, processor, battery, logical-disk groups (WAPI-03/04) | Safe API (no unsafe in the caller), `windows >=0.59,<0.63` matches the workspace. MIT OR Apache-2.0. [VERIFIED: crates.io API] |
| `tar` + `bzip2` | 0.4.46 + 0.6.1 | xtask only: unpack the CEF `minimal` `.tar.bz2` | `bzip2` 0.6 uses the pure-Rust `libbz2-rs-sys` already in the tree through `sevenz-rust2` (`bzip2-1.0.6` is already allowed in `deny.toml`). [VERIFIED: crates.io API; deny.toml read] |
| `percent-encoding` | 2.3.2 | Decode request paths in the resolver and in traversal tests | Same crate family `url` uses. [VERIFIED: crates.io API] |

### Already in the workspace and reused

| Library | Version | Purpose |
|---------|---------|---------|
| `windows` | 0.62.2 | Win32/COM/WinRT: Media Foundation, Direct2D, WASAPI, GSMTC (`Media_Control`), WTS, RawInput, DisplayConfig, WinTrust. New cargo features per crate (names to be confirmed at compile time, see A9). [VERIFIED: Cargo.toml:60] |
| `image` | 0.25 (gif, png, webp on) | Lazy animated decode for the built-in renderer (A6). [VERIFIED: Cargo.toml:33] |
| `libloading` 0.9.1, `bindgen` 0.73.2 | | libVLC loader and bindings, as for libmpv (D-013). [VERIFIED: root Cargo.toml] |
| `ureq` 3.5.2, `sha2` 0.11.1, `sevenz-rust2` 0.25.0 | | Bootstrapper download, pins, 7z unpack for libVLC. [VERIFIED: root Cargo.toml] |
| `postcard`, `serde`, `serde_json`, `crossbeam-channel`, `interprocess` | | IPC and threading as in Phases 2-3. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| WebView2 virtual host mapping | `WebResourceRequested` with a Rust response | Mapping is simplest and gives Range for free, but is one folder per host, limited to MAX_PATH and documented as slow for media (quoted below). Recommended: the resolver on both engines; keep mapping as a measured fallback. |
| CEF windowed child | CEF windowless (OSR) | Legacy used OSR at 60 fps. Windowed has no paint callback cost and no copy; but `WasHidden` is OSR-only (quoted below). Recommended: windowed child via `WindowInfo::set_as_child`; Heavy by HWND hide + CDP freeze. |
| `windows` WASAPI | `wasapi` 0.27.0 crate | Safer API but a second wrapper over the same COM; the loopback code is ~150 lines. Recommended: `windows` directly, in the crate that may hold unsafe (Open Question 3). |
| `Win32_ProcessStartTrace` | ToolHelp snapshot at 2 s while the rule is active | Events may need elevation (A12, spike). Fallback keeps the legacy cadence at a fraction of the .NET cost. |
| Direct2D HwndRenderTarget | D3D11 flip swap chain with a textured quad, or GDI `StretchDIBits` | D3D11 needs shaders; GDI costs CPU on 4K. Recommended: Direct2D (GPU, no shaders, dest/source rects give the four stretch modes), GDI as the software fallback. Measure against mpv (D-10). |
| `mime_guess` | Fixed MIME table | The legacy server has a fixed table; parity plus no dependency. (The legitimacy seam also returned SUS for `mime_guess` because the registry lookup failed, so it is not recommended.) |

**Installation (workspace root `Cargo.toml`, members use `workspace = true`):**
```toml
cef = { version = "=156.7.0+156.0.31", default-features = false }
webview2-com = "0.39.1"
realfft = "3.5.1"
wmi = "0.18.4"
percent-encoding = "2.3.2"
# xtask only
tar = "0.4.46"
bzip2 = "0.6.1"
```

**Version verification (this session, crates.io API and vendor indexes):**

| Item | Verified value |
|------|----------------|
| `cef` / `cef-dll-sys` / `download-cef` | `156.7.0+156.0.31` (updated 2026-11-19) / same / `3.0.1` (2026-11-15) |
| `webview2-com` / `webview2-com-sys` | `0.39.1` (2026-03-11) |
| `windows` | `0.62.2` (2025-10-06); `windows-core` 0.100.0 exists but `webview2-com` and `wmi` ask for `^0.62` and `>=0.59,<0.63` |
| `image` 0.25.10, `gif` 0.14.2, `png` 0.18.1, `image-webp` 0.2.4, `rustfft` 6.4.1, `realfft` 3.5.1, `wmi` 0.18.4, `tar` 0.4.46, `bzip2` 0.6.1, `percent-encoding` 2.3.2, `libloading` 0.9.1, `bindgen` 0.73.2 | as listed |
| CEF stable on spotifycdn | `158.0.6` is the newest stable; `156.0.32+g3d1f4ab` and `156.0.31+ge603d97` exist for windows32, windows64, windowsarm64. The `cef` crate pins `156.0.31`, which is also the CEF line of the legacy CefSharp 156.0.150 (FEATURES.md). |
| WebView2 Runtime on the maintainer's machine | `158.0.4370.33` (registry `pv` under `WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`), above the legacy floor 131.0.2903.70. [VERIFIED: reg query this session] |

### Engine pins to add to `xtask/engines.toml` (D-028; SHA256 computed by the fetch task, upstream hashes below)

| Engine | Arch | Archive | Size (bytes) | Upstream hash |
|--------|------|---------|--------------|---------------|
| libVLC 3.0.24 | i686 | `https://download.videolan.org/vlc/3.0.24/win32/vlc-3.0.24-win32.7z` | 38188211 | sha256 `80b61c03a5c7c4d4b44a79a2db34eda244ad6b09f1e0f44fcf6183f4c10e9966` |
| libVLC 3.0.24 | x86_64 | `.../win64/vlc-3.0.24-win64.7z` | 39384682 | sha256 `1ed59c09152e78aff84663fa76efab26057716576db74332858810e5ef54ae1f` |
| libVLC 3.0.24 | aarch64 | `.../winarm64/vlc-3.0.24-winarm64.7z` | 35567426 | sha256 `4c448ca4995bafc4954a4c929bc7dad756eb0a234afc9d58946ad05b8fe51389` (also re-computed here with `sha256sum` on the downloaded file: identical) |
| CEF 156.0.31+ge603d97+chromium-156.0.8078.100 | i686 | `https://cef-builds.spotifycdn.com/cef_binary_156.0.31%2Bge603d97%2Bchromium-156.0.8078.100_windows32_minimal.tar.bz2` | 153578723 | sha1 `cba6c57226f4d72dddbdf8fab18440da69573214` |
| CEF (same) | x86_64 | `..._windows64_minimal.tar.bz2` | 175156160 | sha1 `d179fb251141d6c30992a1775db80a6a879852ba` |
| CEF (same) | aarch64 | `..._windowsarm64_minimal.tar.bz2` | 170159472 | sha1 `0d1a346ac91fef15ad32e81db68a19a0c547e2b0` |

[VERIFIED: https://cef-builds.spotifycdn.com/index.json fetched this session (names, sizes, sha1); VLC hashes and sizes from the .sha256 files and HEAD requests; the aarch64 archive was downloaded and hashed]. The CEF index publishes SHA1 only; the fetch task downloads each archive once, records SHA256 in `engines.toml` like the libmpv entries, and keeps the SHA1 as a second check. The VLC 7z unpacks to 175,222,089 bytes (ARM64), contains `libvlc.dll` (151,976 B), `libvlccore.dll`, `plugins/` (464 listing lines), `sdk/include/vlc/libvlc*.h` and `COPYING.txt` [VERIFIED: `7z l` on the ARM64 archive]; the headers define `LIBVLC_VERSION_MAJOR (3)`, `MINOR (0)`, `REVISION (24)` [VERIFIED: libvlc_version.h]. The CEF `minimal` distribution is the one `download-cef` uses (`minimal()` in `download-cef/src/lib.rs` [VERIFIED: source]); it carries the headers, the CMake files for `libcef_dll_wrapper` and the runtime files.

## Package Legitimacy Audit

Run with `gsd_run query package-legitimacy check --ecosystem crates` this session.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| cef | crates.io | 3.5 yrs (2023-05) | 20.6K/wk | github.com/tauri-apps/cef-rs | [OK] | Approved (named by D-013) |
| cef-dll-sys | crates.io | 1.9 yrs | 18.6K/wk | github.com/tauri-apps/cef-rs | [OK] | Approved (transitive of cef) |
| download-cef | crates.io | 1.9 yrs | 19.1K/wk | github.com/tauri-apps/cef-rs | [OK] | Approved (build dependency of cef-dll-sys; never called when CEF_PATH is valid) |
| webview2-com / -sys | crates.io | 5 yrs | 1.10M/wk | github.com/wravery/webview2-rs | [OK] | Approved (named by D-013) |
| realfft | crates.io | 6 yrs | 471K/wk | github.com/HEnquist/realfft | [OK] | Approved |
| rustfft | crates.io | 11 yrs | 965K/wk | github.com/ejmahler/RustFFT | [OK] | Approved (transitive) |
| tar | crates.io | 12 yrs | 6.7M/wk | github.com/composefs/tar-rs | [OK] | Approved (xtask) |
| bzip2 | crates.io | 12 yrs | 3.7M/wk | github.com/trifectatechfoundation/bzip2-rs | [OK] | Approved (xtask) |
| percent-encoding | crates.io | 9 yrs | 23.9M/wk | github.com/servo/rust-url | [OK] | Approved |
| wmi | crates.io | 7 yrs | 205K/wk | github.com/ohadravid/wmi-rs | [OK] | Approved |
| mime_guess | crates.io | | | | [SUS] (registry lookup failed: unknown-age, no-repository) | Not recommended; fixed MIME table instead |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** mime_guess (not used). Build scripts: `cef-dll-sys` `build.rs` downloads and runs CMake; it was read in full (it only fetches from the CDN in `download-cef`, honours `CEF_PATH`, writes under `OUT_DIR` and the target dir); `webview2-com-sys` `build.rs` only copies the bundled loader libs. No registry `postinstall` concept exists for crates. Crate names: `cef` and `webview2-com` come from D-013 and are confirmed by the owners' repositories and docs (tauri-apps, wravery). `wmi`, `realfft`, `tar`, `bzip2` and `percent-encoding` were found by search but returned [OK] from the seam this session; they are tagged [ASSUMED] as choices only until the planner confirms the API in the crates' docs (A13), because registry existence alone is not provenance.


## Architecture Patterns

### System Architecture Diagram

```
 Portal (sucrose-portal.exe)                         Service (sucrose.exe, always on while a wallpaper is active)
 ┌──────────────────────────┐  PortalRequest        ┌───────────────────────────────────────────────────────────────┐
 │ Wallpaper / Performance  │ ───────────────────▶  │ settings_host: single writer (Engine.DisplayAssignments,      │
 │ pages, Display Prefs,    │                       │   <data_dir>/Properties/<Theme><Type> working copies)         │
 │ properties editor window │ ◀───────────────────  │ engine registry: detect() -> Availability; choose_engine()    │
 │ install dialog, toasts   │  PortalEvent + Blob   │ topology/fleet: one slot per target (monitor or union rect)   │
 └──────────────────────────┘                       │ governor (exists only if PerformanceCounter and a rule != Resume)
                                                    │   signals -> legacy order -> 5/3 debounce -> Pause{level}|Close │
                                                    │ data hub (exists only while a running page defines a group)   │
                                                    │   WMI / PDH / WASAPI loopback / GSMTC -> PushData              │
                                                    │ prerequisite installer worker (download, Authenticode, run)   │
                                                    └───────┬─────────────────────────────────▲─────────────────────┘
                              ToRenderer: Assign(v4), Pause{level}, SetVolume, SetRect, SetRole, Clock, LoadSource,
                              ApplyProperty, PushData, Blob, OpenDevTools, Barrier       ToService: Hello, Playing, Failed, Clock, Ready
                                                            ▼                                     │
                    Renderer (sucrose.exe --render <slot>), one per target; surface thread = Win32 loop (EventLoop)
                    ┌───────────────────────────────────────────────────────────────────────────────────────────┐
                    │ DesktopSurface HWND (layered child of Progman/WorkerW, Phase 2)                           │
                    │   └─ Engine (trait object, !Send, driven by commands drained on LoopEvent::Wake)          │
                    │        mpv │ libVLC │ Media Foundation │ built-in GIF (decode thread + Direct2D)          │
                    │        web: CEF (external message pump) │ WebView2 (STA, same thread) │ app embedder    │
                    │ web engines: origin resolver <- CEF scheme handler / WebView2 WebResourceRequested        │
                    │              JS builders (PropertyListener, Compatible) -> execute script                 │
                    │ RawInput hidden window -> PostMessage to Chrome_WidgetWin_* (only if cursor in my rect)   │
                    └───────────────────────────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
crates/
  sucrose-engine/            # NEW. Engine trait, EngineId, Availability, choose_engine (pure), PauseLevel, PropertyValue,
                             #   the clock follower moved from sucrose-engine-mpv (pure Rust, no unsafe), stretch helpers
  sucrose-engine-web/        # NEW, pure, no unsafe, builds on every OS: origin.rs (resolver), mime.rs, js.rs (PropertyListener,
                             #   Compatible formatter), pages/ (GifContent, VideoContent, YouTubeContent), args.rs (CefArguments <-> WebArguments),
                             #   sync_shim.js; tests/fixtures/pages/ (FMT-07) and traversal vectors (SEC-02)
  sucrose-engine-cef/        # NEW, Windows only (cfg), FFI: cef crate glue, delay-load build.rs, scheme handler, CDP helpers
  sucrose-engine-webview2/   # NEW, Windows only, FFI: webview2-com glue
  sucrose-engine-vlc/        # NEW: loader.rs, sys.rs (bindgen_vlc), player.rs (model: sucrose-engine-mpv)
  sucrose-engine-builtin/    # NEW: gif/ (decode + Direct2D) and mf/ (IMFMediaEngine); split into -gif and -mf if CI time matters
  sucrose-sysinfo/           # NEW: payload models, audio visualizer, WMI providers; OS providers per Open Question 3
  sucrose-core/              # + properties.rs, compatible.rs (FMT-03), display id and assignment types
  sucrose-config/            # + Engine.DisplayAssignments, web certificate switch (new keys, legacy-section precedent)
  sucrose-ipc/               # protocol v4, Blob chunks
  sucrose-platform/          # + monitor id (DisplayConfig), session/power/event sources, taskbar edge, WinEvent hook, audio sessions, embed helpers
apps/sucrose/src/            # render.rs becomes an engine-agnostic host; governor.rs; data_hub.rs; install.rs; assignments in service/topology
apps/sucrose-portal/src/     # settings pages, dialogs/display_preferences.rs, dialogs/install.rs, editor/ (second window)
xtask/src/                   # fetch_engines (+ tar.bz2, CEF_PATH layout), bindgen_vlc, measure (per engine rows)
```

The UI never links engine crates (ARCHITECTURE dependency rules): the Portal learns availability through the service.

### Pattern 1: Engine trait, availability and the pure choice (ENG-06, ENG-07)

**What:** Extend the sketch in `docs/ARCHITECTURE.md` to what the renderer really calls: `open`, `load_source` (Use without restart), `set_rect` (the surface moved; WebView2 `put_Bounds` and CEF `SetWindowPos` need it, mpv follows its parent by itself), `pause(level)`, `set_volume`, `set_stretch`, `set_loop`, `apply_property`, `push_data`, `position()` and `seek()` for D-030, `open_devtools()`, `shutdown()`. Engines are `!Send` and live on the surface thread; the IPC thread posts commands and calls `LoopProxy::wake` (the `PendingRect` pattern in `render.rs` today).
**Choice:** `choose_engine(kind, configured, availability) -> Choice { engine, fallback: Option<Fallback> }` is a pure function with a table test per type. Legacy names stay in settings: the enum already has `VexanaLive`, `XavierLive`, `NebulaLive`, `WebViewLive`, `CefSharpLive`, `MpvPlayerLive`, `VlcPlayerLive` and `AuroraLive` [VERIFIED: crates/sucrose-core/src/settings.rs:267-283] and the per-type allowed lists `GIF = [VexanaLive, XavierLive, WebViewLive, CefSharpLive, MpvPlayerLive, VlcPlayerLive]`, `VIDEO = [NebulaLive, WebViewLive, CefSharpLive, MpvPlayerLive, VlcPlayerLive]`, `URL/WEB/YOUTUBE = [WebViewLive, CefSharpLive]`, `APPLICATION = [AuroraLive]` [VERIFIED: settings.rs:288-317]. So D-10's mapping needs **no enum change and no migration**: `VexanaLive` and `XavierLive` both run the built-in GIF renderer, `NebulaLive` the Media Foundation player, `AuroraLive` the application embedder; the Portal shows one "Built-in" entry per type and writes one canonical legacy name (recommend `XavierLive` for GIF, `NebulaLive` for video).
**Fallback order (discretion, D-11):** Web/Url/YouTube: chosen web engine, then the other web engine, else "no web engine" with the install hint. Gif: chosen, then built-in GIF, then mpv, then a web engine (generated page). Video: chosen, then mpv, then Media Foundation, then libVLC, then a web engine. Application: none (embedder only; a failure is reported, not faked, D-002). "Chosen engine is back" is detected by re-running `detect()` at service start and when the Portal opens the Wallpaper page; the setting is never changed (D-11), so the next start picks it again.
**Availability report:** `Installed(version)`, `Missing{hint}`, `NotLoadable{reason, hint}` (libmpv today: `LoadError` with `hint()` in `loader.rs`), `Unsupported{arch}` for an engine with no build for the architecture (D-027). WebView2: registry `pv` (see Findings 2). CEF: VC++ Redist registry check per D-15 plus `libcef.dll` present and loadable. libVLC and CEF "loadable" is checked by a short-lived probe in the renderer path, never in the service (D-028: only renderers load engines).
**When to use:** slice 1 (registry, choice, mpv behind the trait with no behavior change, regression tests of Phases 2-3 stay green).

### Pattern 2: One origin resolver, two adapters (D-14, SEC-02)

**What:** `sucrose-engine-web::origin::resolve(url_path, theme_dir, content_dir) -> Result<ResolvedFile, Reject>`. Steps: take the URL path only (drop query and fragment); percent-decode **once**; reject if the decoded text contains NUL, a backslash, a drive letter or colon, or starts with `//` (UNC); split on `/` and reject any `..` segment (also after a second decode pass, so `%252e%252e` is rejected, not decoded); map empty to the theme's `Source` page only where the legacy did; join under the theme folder, then the generated-content folder (the legacy lookup order, engines.md `9); `canonicalize` the candidate and require `starts_with` of the canonical root (defeats junctions and symlinks); reject directories; return the file with the MIME from a fixed table (legacy parity). It never opens files. CEF adapter: a `CefSchemeHandlerFactory` registered for scheme `https` and domain `sucrose.wallpaper.engine` (the header says an empty domain matches all domains, a non-empty one only that domain) [CITED: cef_request_context.h RegisterSchemeHandlerFactory] returns a resource handler that streams the resolved file and implements `Range` (206 with `Content-Range`). WebView2 adapter: `AddWebResourceRequestedFilter("https://sucrose.wallpaper.engine/*", ALL)` and a `WebResourceRequested` handler that answers from the same resolver with an `IStream` over the file and honours `Range`. 
**Why not host mapping:** the documentation of `SetVirtualHostNameToFolderMapping` says "Due to a current implementation limitation, media files accessed using virtual host name can be very slow to load", "the folderPath length must not exceed the Windows MAX_PATH limit", and "If there are multiple folders being used, define one unique virtual host per folder" [CITED: learn.microsoft.com ICoreWebView2_3, fetched]. D-14 needs two folders (theme, then generated content) under one origin, and web wallpapers often play video. Keep the mapping as a measured fallback only if the custom path misbehaves on a real theme.
**Proof:** a table of traversal vectors run against the resolver (`..`, `%2e%2e`, `%252e%252e`, `..%5c`, `/`-prefixed, `\\server\share`, `C:\`, `%00`, long paths, case variants, a junction pointing outside) plus an adapter test per engine on the maintainer's machine that requests them from a real page and expects 404. The generated pages themselves are served from the content folder by name only.

### Pattern 3: JS API builders are pure strings (WAPI-01, WAPI-02, FMT-07)

**What:** The engine never exposes host objects (legacy grep result in engines.md `6); all calls are script strings run with `ExecuteScript` (WebView2) / `ExecuteJavaScript` on the main frame (CEF). Build them in `sucrose-engine-web::js`: `property_call(listener_format, key, control_json)` is `string.Format(listener, key, JsonConvert.SerializeObject(control, Formatting.Indented))`; the compatible formatter substitutes `{0}` with C# text: `True`/`False` for booleans, enum names for `ThemeType`/`StretchMode`, 0..100 for `VolumeLevel`, JSON text for the `System*` groups, and runs the non-empty templates in the table order of engines.md `6b. `filedropdown` lists `<theme>/<folder>` with the `|`-separated `*.ext` filter, returns `[]` outside the theme folder. When: all property keys after the main-frame load finished (WebView2 `DOMContentLoaded`, CEF `OnLoadEnd` for the main frame), then on every change. Fixtures: the 7 themes with `SucroseProperties.json` and the 1 with `SucroseCompatible.json` of the maintainer's library, sanitised (FMT-03), plus hand-written fixture pages that record every call into `window.__calls` for the engine tests.
**Reading values from a page** (clock for D-04, DevTools state): use `Runtime.evaluate` through the DevTools protocol on CEF (`ExecuteDevToolsMethod` plus a message observer) and `ExecuteScript`'s result callback on WebView2, so the page still gets no host object. A page-to-host message channel is never added.

### Pattern 4: Light and Heavy are engine calls (D-22, PLAY-05)

| Engine | Light | Heavy (keeps position) |
|--------|-------|------------------------|
| mpv | `pause=yes` (existing `Player::set_paused`) | `pause=yes` then `vid=no` to drop the decoder and video output; resume: `vid=auto` then exact seek to the saved `time-pos`. Fallback if RAM does not fall: `stop` and `loadfile ... start=<pos>` on resume. [ASSUMED: measure Light vs Heavy RAM] |
| libVLC | `libvlc_media_player_set_pause(1)` | save `get_time`, `stop`; resume: `play` then `set_time` |
| Media Foundation | `Pause()` | `Pause()`, drop the swap chain/window content, keep source and position; resume re-creates the output |
| built-in GIF | stop the timer; keep the current frame | free decoded buffers and Direct2D bitmaps; keep the frame index and phase; resume re-decodes to the index |
| CEF | page `visibilitychange` hidden plus `video.pause()` / YouTube `pauseVideo()` / `SucrosePause()` | Light plus hide the browser HWND (`ShowWindow(SW_HIDE)`) and `Page.setWebLifecycleState {state: "frozen"}` over CDP; resume: `"active"` then show. CEF `WasHidden` **cannot** be used: "This method is only used when window rendering is disabled" [VERIFIED: include/cef_browser.h, CEF master] |
| WebView2 | same page-level calls | `put_IsVisible(false)` then `TrySuspend`; "The CoreWebView2Controller's IsVisible property must be false when the API is called" and "The WebView will be automatically resumed when it becomes visible"; best effort, completes with `isSuccessful` false when Chromium refuses [CITED: ICoreWebView2_3, fetched] |

`Page.setWebLifecycleState` is an experimental DevTools method [ASSUMED: A2, devtools protocol docs not fetched]; the fallback if it is missing on a CEF build is CPU throttling with `Emulation.setCPUThrottlingRate` plus the hidden HWND. The legacy `DebugActiveProcess` and `NtSuspendProcess` code is not ported.

### Pattern 5: Clock following for every time-based engine (D-04, D-30)

Move the pure controller (`correct`, `SyncAction`, `Follower` in `crates/sucrose-engine-mpv/src/sync.rs`, which depends on `ClockTick` only) into `sucrose-engine` and re-export from `sucrose-engine-mpv`. Each engine implements `position()`/`seek()`/`set_rate()`: mpv as in Phase 2; libVLC `libvlc_media_player_get_time/set_time/set_rate` (all present in the 3.0.24 headers [VERIFIED: libvlc_media_player.h grep]); web video pages through `video.currentTime` and `playbackRate` (±δ rate correction, seeks beyond the threshold); YouTube `getCurrentTime/seekTo`; built-in GIF by frame index (the follower waits or skips *presentation*, because GIF frames depend on earlier frames, so it keeps decoding continuously and picks the frame for the master phase; a seek re-decodes from the start and is only used beyond the drift threshold or at a master loop wrap, `loop_gen`). The master's clock reading of a web page is a once-per-second pull (Pattern 3). Constants stay where Phase 2 put them (`CLOCK_PERIOD` 1 s, `CLOCK_POLL` 100 ms [VERIFIED: apps/sucrose/src/render.rs:57-58]).

### Pattern 6: Properties: service writes, Portal edits, renderers apply (PROP-*, D-18..D-21)

The service is the single writer (D-032 spirit): the Portal sends `PropertyChange{theme, key, value}` (and `Restore`, `Refresh`, `DeleteCopy`); the service updates the working copy under `<data_dir>/Properties/<Theme><Type>` with the same atomic write used for `Sucrose.json` (debounced, e.g. 150 ms after the last change, but the live delta is sent immediately and throttled to ~30 Hz while a slider drags), and forwards `ApplyProperty` to the renderers of that theme. The editor gets the control document through the generic `Blob` chunk message (Pattern 7) because a property file can exceed one 64 KiB frame. The one-time copy (D-20): when `<data_dir>/Properties` does not exist, copy every file of the real `%APPDATA%\Sucrose\Cache\Properties` (never `SUCROSE_DATA_DIR`'s) whose name is not `<guid>.<Theme><Type>`; never write the C# folder. Culture for `PropertyLocalization` comes from the migrated `General` culture (the maintainer's is TR), falling back to the first culture. The ten control types and value rules are in engines.md `7 and services.md `6; unknown types render a muted "not supported" line (UI-SPEC). mpv applies keys as mpv property names, VLC accepts only the fixed keys, web engines call the listener; the built-in sets (D-06) gain `volume` next to `mute`/`muted`.

### Pattern 7: IPC v4, append-only where possible, blobs for big payloads

`Assignment` and `PlaybackSettings` are structs inside enum variants; postcard has no field tags, so adding fields is **not** append-compatible. Because the service and its renderers are the same executable (Hello already refuses another `PROTOCOL_VERSION`, currently `pub const PROTOCOL_VERSION: u16 = 3;` [VERIFIED: crates/sucrose-ipc/src/protocol.rs:14]), bump it to 4 once, in slice 1, and add the new fields (`kind`, `engine`, `web` settings, `properties` source) to a new `Assignment` shape; every later message is a new variant appended at the end of `ToRenderer`/`ToService` (the wire-index tests in `protocol.rs` pin that). The Portal protocol is likewise appended (`PORTAL_PROTOCOL_VERSION` is `1` [VERIFIED: crates/sucrose-ipc/src/portal.rs:15]; bump only for a struct change). Frames are capped: "`pub const MAX_FRAME: u32 = 64 * 1024;`" [VERIFIED: crates/sucrose-ipc/src/frame.rs:13]. A now-playing thumbnail (233x233 PNG as base64 in the legacy) and a property document can exceed it, so add one reusable `Blob{id, seq, total, bytes}` message pair (chunks of ≤ 48 KiB, reassembled with a cap) rather than raising the cap. `PushData` carries one group per message as JSON text; the 128-band audio arrays go as `f32` binary and the renderer renders them to JSON.

### Pattern 8: The performance governor generalises the fullscreen rule (D-23..D-25)

Start from `apps/sucrose/src/fullscreen.rs` (5/3 debounce, scope, action) and `modules.rs` (the start gate). One `Governor` object holds, per condition, `{action, debounce_state, signal_reader}` for the conditions whose action is not `Resume`; it exists only while `PerformanceCounter` is on and one rule is not `Resume` (the maintainer's data: Lock, Sleep, Console, Session = Close, ScreenSaver = Pause, so it starts but reads only those signals). Evaluate on the legacy order (Lock, Focus, Sleep, Memory, Remote, Battery, Console, Graphic, Network, Session, Virtual, Processor, FullScreen, ScreenSaver, BatterySaver), first match wins, only recovery while active, 5 polls to trigger and 3 to recover on 1 s polls and 0/0 for Lock, Sleep, Console, Session, ScreenSaver [CITED: docs/legacy/services.md `1.1]. Keep the state machine pure (inputs: a snapshot struct of booleans/percentages; outputs: `Action` and the active condition) so the whole table is unit-tested without Windows. Events (WTS, power) set booleans; the 1 s timer samples only the polled signals; no timer runs when only event conditions are armed. Output: `Pause{level}`/`Close`/`Resume` to the fleet through the existing `RuleClosed` bookkeeping, and `RuleState`/tray text for D-27.

### Pattern 9: Per-monitor assignments and instances (D-01..D-09)

Settings: a new key `Engine.DisplayAssignments` (list of `{MonitorId, DeviceName, Theme}`) next to the legacy keys, following the precedent of `Backgroundog.FullScreenScope` (a new key inside a legacy-named section, "new, FullScreenScope" in `model.rs`). Migration of the maintainer's data: PerDisplay with `ScreenDevice \\.\DISPLAY1` becomes one assignment of `Library.Selected` to that monitor's id; Span and Duplicate keep `Selected` as the single theme. The service's topology and `Fleet` already create one slot per monitor; Phase 4 changes the *job* from "one job for all slots" to a map target -> job (a job per assignment in PerDisplay, the same job on every monitor in Duplicate, one slot over the union rectangle in Span). Master election (D-030) runs per *theme*: among renderers playing the same source in Duplicate, or the same theme in PerDisplay, the existing `choose_master` applies; different themes are separate clocks and each plays its own sound (D-05).

### Anti-Patterns to Avoid

- **Linking CEF into the service's startup path** (service and mpv renderers would load `libcef.dll`, hundreds of MB of address space, and the service stops being near idle). Delay-load or a separate exe (Open Question 1).
- **One shared persistent CEF profile for every renderer** (the process-singleton lock makes the second renderer fail or relaunch into the first; see Pitfall 2).
- **Host objects or a message channel into pages** (legacy has none; a channel is a new attack surface for untrusted theme pages).
- **Polling the process list for remote/VM tools every 2 s with a full enumeration** when no rule needs it (legacy cost; D-23/D-24).
- **Decoding every GIF frame into memory** (a 1080p GIF with 200 frames is 1.6 GB of RGBA).
- **Writing property files from the Portal or the renderer** (breaks the single-writer rule; races with the live delta).
- **Using `NtSuspendProcess`, `DebugActiveProcess` or process-wide `SuspendThread`** for pausing (PLAY-05).
- **Opening DevTools automatically** (D-13) or a fixed debugging port for every CEF process.


## Delivery Slices (D-31) — Guidance for the Planner

Every slice ends with a working wallpaper and a maintainer checkpoint on a release build with a `SUCROSE_DATA_DIR` scratch folder (`human_verify_mode: end-of-phase` in config means the checkpoints are collected at the phase gate, but each slice still needs a prepared run). The Phase 3 rule "Use disabled for non-Video themes" is lifted per type in the slice that makes the type playable.

### Slice 1: Engine abstraction, CEF, virtual origin, JS API, first system data (ENG-03, ENG-06, ENG-07, TYPE-03, WAPI-01/02/03/04, WAPI-09/10 (CEF part), FMT-03, FMT-07, SEC-02, PROP-05 (web set), WAPI-07 (input))

Split into plans that each end green:
1. **Pure cores (no CEF yet):** `sucrose-engine` (trait, `Availability`, `choose_engine`, clock follower moved), `sucrose-core` `properties`/`compatible` models with fixtures, `sucrose-engine-web` (origin resolver + vectors, MIME table, JS builders, argument converters, page templates), IPC v4 + `Blob`, `Engine.DisplayAssignments` left for slice 4. mpv moves behind the trait with **no behavior change**; the Phase 2/3 integration tests (`apps/sucrose/tests`) are the regression net.
2. **CEF link-model spike (gate):** build `sucrose-engine-cef` with `CEF_PATH` pointing at an extracted pinned dist; prove `libcef.dll` is *not* loaded by `sucrose.exe` in service mode and mpv-renderer mode (delay-load, `Process Explorer`/`tasklist /m libcef.dll`), and that `execute_process` + a windowed browser run in a renderer. If the delay-load gate fails, take the fallback (separate helper exe) *before* writing more CEF code. Also settle: `crt-static` (A4) and Ninja/CMake on the build machines (see Environment Availability).
3. **xtask:** `fetch-engines` learns `.tar.bz2` (tar + bzip2), writes `target/engines/<arch>/cef/<version>/<os_arch>/` plus `archive.json` (`{type:"minimal", name, sha1}` as `download-cef` expects [VERIFIED: download-cef/src/lib.rs]) and exports `CEF_PATH` for cargo; a pin row per arch; the libVLC rows can land here too. `xtask ci` sets `CEF_PATH` so no build script downloads.
4. **CEF backend:** windowed child (`WindowInfo::set_as_child` [VERIFIED: cef-rs window_info.rs]) of the surface HWND, external message pump on the surface thread (cef-rs ships a Windows external-pump example, `examples/tests_shared/src/browser/main_message_loop_external_pump/win.rs` [VERIFIED: repo tree]), scheme handler, JS calls, argument/Chromium switch plumbing, per-instance cache path (Pitfall 2), no sandbox (`no_sandbox`, legacy default `no-sandbox`), `remote_debugging_port` per slot.
5. **Service side:** detection report, engine rows in the Portal Wallpaper page (UI-SPEC slice 1), fallback reporting (toast, tray, log, row alert), snooze through the migrated `WebViewTime`/`CefSharpTime`, prerequisite check for VC++ Redist (registry value from engines.md `9) and the install dialog (WebView2 install can wait for slice 2). First system data: the data hub framework with the cheap groups (Date, Bios, Motherboard, Processor, Memory) and the non-data keys (`LoopMode`, `ThemeType`, `StretchMode`, `ShuffleMode`, `VolumeLevel`).
6. **Input forwarding:** RawInput on the renderer's message window (WAPI-07), only forwarded while the cursor is inside this renderer's rectangle.

**Depends on:** Phases 2-3. **Riskiest unknown:** the CEF link model and a windowed CEF browser inside the layered Progman child on 24H2+ (the surface strategies of Phase 2 `LayeredChild`/`WorkerWChild`/`ClassicWorkerW` each need one render proof). **Proof on the maintainer's machine:** his 8 Web themes (the one with `SucroseCompatible.json` shows live data) and the fixture pages play on DISPLAY1 under CEF; the traversal test page gets 404s; `tasklist` shows no `libcef.dll` in the service.

### Slice 2: WebView2 (ENG-02, WAPI-05, WAPI-06, ENG-06 install, D-12 meaning on both engines)

Crates: `sucrose-engine-webview2`, the same resolver and JS builders. Environment: `CreateCoreWebView2EnvironmentWithOptions` with the user data folder, `AdditionalBrowserArguments` from `WebArguments`, language; controller bounds in physical pixels and `RasterizationScale` from the monitor DPI (a parent in WorkerW gets no `WM_DPICHANGED`, the legacy needed a LayoutTransform for the same reason, engines.md `2); `ServerCertificateErrorDetected` left unhandled by default and set to always allow only with the new switch (A8). Also lands here: audio spectrum (WASAPI loopback + `realfft`) and now playing (GSMTC), because they need a proof on both engines and slice 2 is otherwise small; the Bootstrapper install with Authenticode verification (the maintainer already has Runtime 158.0.4370.33 so the install path is proven with a mocked prerequisite check and a signature test on a real Microsoft-signed file; the real install needs a machine without the Runtime). **Riskiest unknown:** WebResourceRequested streaming with Range for large videos. **Proof:** the same 8 themes on WebView2; a theme with a large local video seeks; DevTools through `edge://inspect` with a port.

### Slice 3: Built-in GIF/APNG/WebP, generated pages, URL, YouTube (TYPE-02, TYPE-04, TYPE-05, WAPI-08, PROP-05 sets, D-06 volume)

Crates: `sucrose-engine-builtin` (gif), `sucrose-engine-web` pages. Decode with the workspace `image` crate's animation iterators (lazy; one frame in flight plus the previous canvas) in a decode thread, deliver RGBA frames to the surface thread, present with Direct2D (Findings 4). Generated pages: reproduce `GifContent.html`, `VideoContent.html`, `YouTubeContent.html` from `src/Shared/Engine/Sucrose.Shared.Engine/Helper/Source.cs` (read-only reference, embedded GIF parser bundle included; check its license when copying) and add the new sync functions (`SucroseSeek`, `SucroseGetTime`). YouTube: Referer decision (Findings 12). Test themes for URL (any HTTPS page), YouTube (the 3 in the Showcase) are prepared under `SUCROSE_DATA_DIR`. **Riskiest unknown:** Direct2D HwndRenderTarget inside the layered surface and the RAM/CPU against mpv playing the same GIF (D-10 measurement). **Proof:** the maintainer's 2 GIF themes (built-in vs mpv numbers), a YouTube showcase theme with playlist/shuffle/loop, a URL page with forwarded input.

### Slice 4: Display modes and per-monitor wallpapers (DESK-02..05)

Crates: `sucrose-platform` (stable monitor id, `Win32_Devices_Display`), `sucrose-config` (`Engine.DisplayAssignments` + migration), `apps/sucrose` (fleet per target, Duplicate and Span), `apps/sucrose-portal` (header action, Display Preferences dialog, card badges, Remove, Customize placeholder). Duplicate sync for web/URL (Findings 11). **Riskiest unknown:** monitor id stability and the `DISPLAY9` virtual display; a Chromium child window spanning the union rectangle (three 4K monitors are 11520 px wide; texture limits per engine are untested). **Proof:** with DISPLAY1 + DISPLAY9 (VDD): different wallpapers on each, Span, Duplicate (Video, GIF, a web theme), disconnect/reconnect DISPLAY9 restores its wallpaper (D-08).

### Slice 5: Properties editor (PROP-01..05, FMT-03)

Crates: `apps/sucrose-portal` (editor window), `apps/sucrose` (property store, deltas), `sucrose-core` (models from slice 1). The second window: `gpui_kit::open_window` is what the Portal uses once (`apps/sucrose-portal/src/main.rs:159`); opening a second window from the running app, positioning it by the taskbar edge (`SHAppBarMessage(ABM_GETTASKBARPOS)` plus the monitor work area from the platform layer, [ASSUMED: API names]) and the "close on Escape unless a field has focus" rule are the unknowns; spike first. **Proof:** the 7 themes with `SucroseProperties.json` and 34 copied property files: change a slider and see the wallpaper change; Restore Default, Refresh, Delete Copy; Turkish texts.

### Slice 6: Performance and sound rules (PERF-01..04, PERF-06..09, PLAY-01..05)

Crates: `apps/sucrose` (governor, tray rule state), `sucrose-platform` (event sources), `sucrose-sysinfo` (counters, shared with the data hub: Graphic, Network, Battery, Storage groups complete WAPI-03 here), Portal Performance page. **Riskiest unknown:** event delivery on the service's hidden window (`WTSRegisterSessionNotification` needs a window handle; check the platform `EventLoop` window), and the non-elevated process-event question (Open Question 4). **Proof:** the maintainer's rule set (Lock/Sleep/Console/Session Close, ScreenSaver Pause): lock and unlock, user switch, screen saver, plus a prepared run that sets CPU = Pause at a low threshold to see Light then Heavy.

### Slice 7: libVLC and Media Foundation (ENG-04, ENG-05)

Crates: `sucrose-engine-vlc` (loader copies `sucrose-engine-mpv/src/loader.rs`: next to the exe then absolute `PATH` entries; `sys.rs` from `xtask bindgen-vlc` over `sdk/include/vlc`), `sucrose-engine-builtin` (mf). VLC needs `VLC_PLUGIN_PATH` / the plugins folder next to `libvlc.dll` and `libvlc_new` options from engines.md `9. Media Foundation: `IMFMediaEngineClassFactory` with `MF_MEDIA_ENGINE_PLAYBACK_HWND` ("Using this attribute will put MediaEngine into a legacy HWND mode", "Consider using the MF_MEDIA_ENGINE_OPM_HWND attribute instead" [CITED: learn.microsoft.com MF_MEDIA_ENGINE_PLAYBACK_HWND, fetched]); the frame-server (DXGI) mode is the alternative if HWND mode misbehaves in the layered surface; loop via `SetLoop`, volume `SetVolume`, rate `SetPlaybackRate`, seek `SetCurrentTime`. Stretch None/Fill/Uniform/UniformToFill through `UpdateVideoStream` source/destination rectangles [ASSUMED]. Codec limits (HEVC/VP9/AV1 need Store extensions) are reported with a hint (D-006). **Proof:** the 80 Video themes: H.264 plays on Media Foundation with no mpv; a VP9/WebM theme reports the limitation and suggests mpv; VLC plays and follows the clock on two monitors.

### Slice 8: Application wallpapers (TYPE-06)

Crates: `sucrose-platform` (WinEvent hook, embed helpers, job), `apps/sucrose` (renderer app engine). Findings 10. No Application theme exists locally: build a tiny Win32 test app as a separate crate/xtask output (`Memory: no test-only app flags`), place it under `SUCROSE_DATA_DIR`. **Riskiest unknown:** foreign windows that fight reparenting (Unity-style launchers, DPI-unaware apps, topmost). **Proof:** the test app per monitor in PerDisplay, one over the union in Span, kill-on-close.


## Findings by Topic (the 13 questions)

### 1. CEF

- **Crate and build:** `cef 156.7.0+156.0.31` (CEF 156.0.31). Windows targets in the README table: x86_64 and ARM64; the repository also contains `i686_pc_windows_msvc` bindings and `download-cef` lists `i686-pc-windows-msvc` among its targets, but x86 is not in the README table (A3: treat Windows x86 CEF as unproven; if it fails, D-027 says report it like a missing engine). Default features are `sandbox`, `build-util`, `resources`; use `default-features = false` (no sandbox bootstrap exe, no bundler dependencies) [VERIFIED: cef/Cargo.toml]. Pre-generated bindings ship in the crate, so no `bindgen` for CEF.
- **Build script (cef-dll-sys):** honours `CEF_PATH` (a dir that either contains `archive.json` or has a `<cef_version>/<os_arch>/` subdir), otherwise downloads the `minimal` archive into `OUT_DIR`; then builds `libcef_dll_wrapper` with CMake, generator **Ninja**, profile RelWithDebInfo, `CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded` (static CRT), links `libcef` dylib (the import library) and `delayimp`, `shell32`, ... and copies the CEF runtime files next to the target dir [VERIFIED: sys/build.rs read in full]. `check_archive_json` requires `archive.json` whose `name` is `cef_binary_<ver>...` with a version ≤ the crate's CEF version. Consequence: build machines need MSVC, CMake **and Ninja** (the maintainer's machine has CMake at `D:\Program Files\CMake\bin`, no `ninja` or `cl` on `PATH`, and Visual Studio 2026 Insiders with Ninja at `Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe` [VERIFIED: where/find this session]); CI runners need the same (A5).
- **Link model (the hidden cost):** the `cef` crate links `libcef.dll` at build time through the import library. A plain build makes every `sucrose.exe` mode (service, mpv renderer) refuse to start without `libcef.dll` and map it. Recommended gate: add `cargo::rustc-link-arg-bins=/DELAYLOAD:libcef.dll` in `apps/sucrose/build.rs` (`delayimp` is already on the link line) so the DLL loads at the first CEF call only in renderers that host a web page; CEF calls this exe as subprocess (`--type=...`) and those processes load it too. [ASSUMED: A1 — delay-load of a CEF-wrapper link is not documented by cef-rs; the first plan of slice 1 proves or refutes it.] Fallback: a second bin target (`sucrose-cef.exe`) that is the web renderer and the CEF subprocess; the service spawns it instead of `sucrose --render` for CEF wallpapers.
- **Process model:** multi-process; by default the same executable is the subprocess (cefsimple has a separate helper bin mainly for macOS: `cefsimple_helper` [VERIFIED: repo tree]). The first thing `main()` does must be `cef::execute_process` (and `api_hash` with `CEF_API_VERSION_LAST` first [VERIFIED: cef/src/lib.rs test init]); the strict command-line parser of Phase 3 must accept `--type=` subprocess arguments and the subprocess must not start service logic or single-instance checks (Pitfall 1). Expect GPU, network, audio/utility and renderer processes per CEF browser process, so N monitors cost N times that (measure; Open Question 2).
- **Message loop:** use `external_message_pump` with `OnScheduleMessagePumpWork` waking the surface thread's `EventLoop` and `do_message_loop_work` from `LoopEvent::Wake`/`Timer`, so every CEF UI-thread callback runs on the thread that owns the surface HWND (the same discipline as mpv). `multi_threaded_message_loop` (the legacy switch) is the alternative; do not mix.
- **Scheme handler (D-14):** `RegisterSchemeHandlerFactory("https", "sucrose.wallpaper.engine", factory)`; the header text "An empty domain_name value for a standard scheme will cause the factory to match all domain names... If scheme_name is a built-in scheme and no handler is returned by factory then the built-in scheme handler factory will be called" [CITED: include/cef_request_context.h]. Return a handler for the exact host and nothing for anything else, so real https traffic still works. The handler streams the resolved file, adds `Access-Control-Allow-Origin` only if the legacy table did (it did, `*`; keep parity or drop it as the page's own origin makes it unnecessary: record in FEATURES), supports `Range`. Do not forget: the origin is HTTPS, so the page may not mix in `http://` subresources without `allow-running-insecure-content` (a legacy default switch; keep).
- **Pause/Heavy:** see Pattern 4. **DevTools (D-13):** `remote_debugging_port` is a `CefSettings` field of the browser process, so every CEF renderer needs its own port; use `DeveloperPort + slot` and log it; the legacy extra switch `remote-allow-origins=*` is added with it. `ShowDevTools` [VERIFIED: cef_browser.h signature] is called only on an `OpenDevTools` message. **Certificates (D-12):** the settings struct has an ignore flag ("If cef_settings_t.ignore_certificate_errors is set all invalid certificates will be accepted without calling this method" [VERIFIED: cef_request_handler.h]); default off; the new switch sets it, and `OnCertificateError` stays unhandled (default reject).
- **User data:** "Multiple application instances writing to the same root_cache_path directory could result in data corruption. A process singleton lock based on the root_cache_path value is therefore used... This singleton behavior applies to all CEF-based applications using version 120 or newer" and "If this value is empty then browsers will be created in 'incognito mode' where in-memory caches are used... HTML5 databases such as localStorage will only persist across sessions if a cache path is specified" [VERIFIED: include/internal/cef_types.h]. See Pitfall 2 and Open Question 2.
- **Licenses and package:** CEF is BSD-3-Clause; Chromium's third-party notices are served by `chrome://credits` in the browser; include the CEF `LICENSE.txt` and the credits in the Phase 5 package notices next to the libmpv/libVLC texts and the source offer (D-028). The dist is large: record the measured unpacked and installed size in FEATURES (D-028 consequences). [ASSUMED: credits page and sizes; measure.]

### 2. WebView2

- `webview2-com 0.39.1` (MIT) over `windows 0.62`; `webview2-com-sys` bundles `WebView2LoaderStatic.lib` and `WebView2Loader.dll` for `x86`, `x64` and `arm64` and adds the matching link search path [VERIFIED: crates/bindings tree and build.rs], so all three Windows architectures are covered and no loader DLL has to be shipped when the static library is used (A7: confirm which one the bindings link). A missing Runtime shows up as an error from environment creation, which maps to `Missing{hint}`.
- **Environment:** one `CoreWebView2Environment` per renderer process with the user data folder, `AdditionalBrowserArguments` (the WebArguments list; every process sharing a folder must use identical options, A10), language, extensions off, crash reporting off (legacy settings, engines.md `9). Everything runs on the surface thread (STA); `webview2_com::wait_with_pump` exists for startup but the renderer already owns a message loop, so use the callbacks.
- **Virtual origin:** see Pattern 2. The legacy `SetVirtualHostNameToFolderMapping("sucrose.wallpaper.engine", Cache/Content, Allow)` plus a Referer filter on `*youtube.com/*` was the old YouTube-only path.
- **Heavy:** `put_IsVisible(false)` then `TrySuspend` (Pattern 4); `Resume` may be called while invisible and the view resumes by itself when shown [CITED: ICoreWebView2_3].
- **TLS (D-12):** by default leave Chromium's behavior; with "Allow invalid certificates" handle `ServerCertificateErrorDetected` and set `AlwaysAllow` (the legacy behavior); [ASSUMED: A8 — event and enum names are from memory of ICoreWebView2_14; the `webview2-com` bindings expose whatever the winmd has, verify at compile time].
- **Runtime detection (D-15):** "Inspect the pv (REG_SZ) regkey ... HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5} ... HKEY_CURRENT_USER\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" (on 32-bit Windows the HKLM path has no WOW6432Node); "at least one of these regkeys must be present and defined with a version greater than 0.0.0.0" [CITED: learn.microsoft.com distribution, fetched]. Compare against 131.0.2903.70 (legacy floor, engines.md `9) numerically per component. Equivalent API: `GetAvailableCoreWebView2BrowserVersionString`.
- **Install (D-15, D-019):** the official Bootstrapper `MicrosoftEdgeWebview2Setup.exe /silent /install` "installs the matching WebView2 Runtime" for the device architecture [CITED: distribution doc]; URL from legacy `https://go.microsoft.com/fwlink/p/?LinkId=2124703` (engines.md `9). Verify the Authenticode signature before running: `WinVerifyTrust` with `WINTRUST_ACTION_GENERIC_VERIFY_V2` and revocation check, then read the signer certificate subject and require the Microsoft Corporation publisher and a Microsoft root in the chain [ASSUMED: A11 — API names from memory; verify on a real Microsoft-signed installer and on an unsigned/modified copy as the negative test]. Download with `ureq` (rustls), to a per-run temp folder, delete after. UAC is Windows' (the installer self-elevates); re-detect afterwards.
- **User data folder:** all WebView2 processes of one UDF are tied to one browser process, and a UDF "can be shared by multiple applications" [CITED: process-model doc, fetched]. So several Sucrose renderers on one UDF share one browser process and **one audio service** (Pitfall 3).

### 3. libVLC

- Pin libVLC 3.0.24 for i686, x86_64, aarch64 (table above). The 7z contains `libvlc.dll`, `libvlccore.dll`, `plugins/`, `sdk/include/vlc/*.h`, `COPYING.txt`, `NEWS.txt` [VERIFIED: `7z l`]. Extract the whole folder next to the exe (e.g. `vlc/`) because libVLC discovers plugins relative to `libvlc.dll` (A14), load `libvlc.dll` with `libloading` from there with `LOAD_WITH_ALTERED_SEARCH_PATH`-style path handling like the libmpv loader (search next to the exe, then absolute `PATH` entries only, D-028).
- Bindings: `xtask bindgen-vlc` modeled on `bindgen_mpv.rs` over `libvlc.h`, `libvlc_media.h`, `libvlc_media_player.h`, `libvlc_events.h` with a dynamic-loading config; i686 needs the same ABI care as libmpv (Phase 2 Pitfall 9). API names present in the 3.0.24 header: `libvlc_media_player_set_hwnd`, `get_time`, `set_time`, `get_position`, `set_position`, `set_pause`, `set_rate`, `get_rate`, `get_length`, `play`, `stop`, `libvlc_audio_set_volume`, `libvlc_audio_set_mute`, `libvlc_video_set_aspect_ratio`, `libvlc_video_set_crop_geometry`, `libvlc_video_set_scale`, `libvlc_video_set_adjust_float`, `libvlc_video_set_mouse_input`, `libvlc_video_set_key_input` [VERIFIED: grep of libvlc_media_player.h in the ARM64 archive].
- Stretch per engines.md `3 table: Fill `set_aspect_ratio("<w>:<h>")`, UniformToFill `set_crop_geometry("<w>:<h>")`, Uniform defaults, None `set_scale(1.0)`. Loop: legacy rebuilds the media with `:input-repeat=65535` to avoid a blank HWND on cold restart (SE.VlcPlayer comment in engines.md `4); keep that. Volume 0..100 per instance through `libvlc_audio_set_volume` (a per-player volume, so D-06 works for VLC); the legacy "gamma not working" flag stays a known limit.
- License: libVLC/libvlccore are LGPL-2.1+, some plugins GPL; the package is GPL-3.0-only so this is compatible; Phase 5 adds the texts and the source offer (D-028). [ASSUMED: license split per VideoLAN's usual statement; confirm from COPYING.txt in the archive.]

### 4. Built-in GIF / APNG / WebP renderer

- **Decode:** the workspace `image` 0.25 with `gif`, `png`, `webp` features already decodes the three formats; its animation interface (`AnimationDecoder::into_frames`, `PngDecoder::apng`, `WebPDecoder`) yields frames as an iterator, so only the current canvas and one frame are alive [ASSUMED: A6 — lazy behavior of `Frames` is from memory; Wave 0 test: decode a large synthetic GIF and assert the working set stays bounded]. Pre-scan once for per-frame delays and total duration (needed for the master clock) without keeping pixels; decode runs on its own thread with a 2-frame channel, loops by reopening the file, and respects the frame delay clamp the legacy used (delay property 0x5100 × 10 ms, default 100 ms, engines.md `1).
- **Present:** Direct2D `ID2D1HwndRenderTarget` (or a device context on the surface HWND) `DrawBitmap` with dest/source rects and the interpolation mode, which gives the four stretch modes without shaders: None = 1:1 centered, Fill = dest rect = surface, Uniform = letterbox, UniformToFill = crop via the source rect. One bitmap re-used per frame via `CopyFromMemory`. Fall back to GDI `StretchDIBits` if Direct2D cannot create a target (software mode). Draw only when a new frame is due; no timer when paused. [ASSUMED: A15 — works inside the layered Progman child; Slice 3 spike.]
- **Memory bound:** cap decoded dimension (downscale on decode to the surface size × 1, never larger), 2 frame buffers + 1 canvas; for very large animations (> a configurable pixel budget, say 64 MP total frame cost) decode in streaming mode only (no frame cache). Heavy pause frees both and keeps `frame_index` + phase.
- **Clock following:** Pattern 5. **Measurement:** D-10 requires the built-in renderer vs mpv on the same GIF (RAM/CPU, release build) in `MEASUREMENTS.md`.

### 5. Built-in OS video (Media Foundation)

- `IMFMediaEngineClassFactory::CreateInstance` with `MF_MEDIA_ENGINE_PLAYBACK_HWND` set to a child window of the surface (legacy HWND mode; the docs recommend `OPM_HWND` for protected content only) [CITED: learn.microsoft.com, fetched]. The engine methods exist: `SetLoop`, `SetCurrentTime`, `SetPlaybackRate`, `SetMuted`, `Play`/`Pause` [CITED: IMFMediaEngine member list, fetched]; `SetVolume` completes the set. `MFStartup` on the thread; notifications arrive through `IMFMediaEngineNotify` (error, loaded, ended). Codec reality (D-006): H.264 everywhere; HEVC/VP9/AV1 may need Store extensions: map the engine's `MF_MEDIA_ENGINE_ERR_SRC_NOT_SUPPORTED`-style error to the "suggest mpv" hint. [ASSUMED: error constant name.]
- If HWND mode shows black inside the layered surface, use frame-server mode (`MF_MEDIA_ENGINE_DXGI_MANAGER`, `TransferVideoFrame` into the Direct2D presenter of Findings 4), which also reuses the GIF presenter and the stretch code.
- Audio goes through the process audio session, so volume is per renderer (D-06 satisfied with `SetVolume`).

### 6. Web wallpaper JavaScript API (WAPI-01..10)

- **Injection:** Pattern 3. Call order on load: property keys, then the compatible templates on the first data tick (legacy pushed only on Backgroundog ticks, so `LoopMode`/`StretchMode`/`VolumeLevel` only changed with data; D-24 decouples collection from `PerformanceCounter` but cadence stays: the service pushes the non-data keys with every data tick and **also once after load**, which is a compatible superset).
- **Data hub (service, D-24):** a group runs only while a running page defines it. Sources (the legacy source in parentheses): Date (`DateTime.Now` at the tick) from `std::time` local time; Bios `Win32_BIOS` and Motherboard `Win32_BaseBoard` once via `wmi`; Processor `Win32_Processor` once + PDH `Processor\% Processor Time` total and per core at 1 s; Memory `GlobalMemoryStatusEx` at 1 s; Graphic PDH `GPU Engine\Utilization Percentage` summed over `engtype_3D` instances with the adapter LUID, clamped 0-100 (the platform crate already names adapters by LUID in `GpuAdapter`); Network PDH `Network Interface` bytes/s, DNS-based online check, ICMP ping (`IcmpSendEcho`, 1000 ms timeout), first adapter chosen if the setting is empty; Storage `Win32_LogicalDisk` every 10 s + PDH disk bytes/s; Battery `Win32_Battery` + `GetSystemPowerStatus` every 2 s [CITED: services.md `1.2]. Payload keys are exactly the tables in engines.md `6b (`{Bios, Date, Audio, Memory, Battery, Graphic, Network, Storage, Processor, Motherboard}`); the C# JSON uses indented formatting inside the page call and PascalCase keys; keep that byte-for-byte, with fixtures.
- **Audio spectrum:** WASAPI loopback of the default render device, recreated on default-device change (`IMMNotificationClient`), 4096-sample ring per channel, Hann window, `realfft`, 128 log-spaced bands 20 Hz to min(16 kHz, Nyquist), peak magnitude per band, +2 dB/oct tilt, ×2.5/reference, minus 0.01 floor, clamp 0-1, envelope attack 0.35 / release 0.06, ±1-bin neighbour average, emit at most every 16 ms [CITED: services.md `1.2]. Delivered to pages with the legacy snapshot cadence (D-24: up to ~10 pushes per second); the capture thread exists only while `SystemAudio` is defined and a page is running.
- **Now playing:** WinRT `GlobalSystemMediaTransportControlsSessionManager` (the `windows` crate's `Media_Control` module) event-driven on session/properties change; fields as in the `Audio` table; thumbnail: legacy crops a 233×233 PNG from (34,1) when pixel (0,0) is transparent and not Cobalt; send via `Blob` chunks only on track change.
- **Input (WAPI-07):** Findings 8. **Generated pages (WAPI-08):** Slice 3. **Developer mode (WAPI-09):** Developer mode enables the context menu (Inspect) and shortcuts; with a port it adds the legacy remote-debugging switches (WebView2: `--remote-debugging-port=N --remote-allow-origins=* --enable-features=msEdgeDevToolsWdpRemoteDebugging`; CEF: `remote_debugging_port` + `remote-allow-origins=*`, engines.md `9); never opens DevTools by itself (D-13). **Arguments (WAPI-10):** `sucrose-engine-web::args` converts the CEF dictionary and the WebView2 list in both directions, marks Sucrose-added switches read-only (hardware acceleration: `--enable-gpu --enable-gpu-vsync` on, `--disable-gpu --disable-gpu-vsync` off, and for CEF off only below Windows build 26000; developer; debugging port), logs unknown switches, and `Reset` restores the legacy default list recorded in engines.md `9.

### 7. Display modes (DESK-02..05)

- **Stable monitor id (D-08):** `QueryDisplayConfig(QDC_ONLY_ACTIVE_PATHS)`, then for each path `DisplayConfigGetDeviceInfo` with the source name (GDI device name such as `\\.\DISPLAY1`) and the target name; `DISPLAYCONFIG_TARGET_DEVICE_NAME` carries `monitorFriendlyDeviceName[64]` and `monitorDevicePath[128]` [VERIFIED: wingdi.h doc, fetched]. Recommend `MonitorId = monitorDevicePath` (hardware id + instance), plus the GDI name as fallback and for display; if the OS returns no path (virtual or remote displays) fall back to the GDI name. EDID-hash identity is the improvement when one physical monitor changes ports (the path embeds the connection instance); keep it out of Phase 4 unless the spike shows paths change when the same monitor moves ports, then add an EDID-derived second key. The `DISPLAY9` VDD must be in the spike. [ASSUMED: A16 — stability across reboot and replug is not documented; spike.]
- **Settings shape:** Pattern 9 (`Engine.DisplayAssignments`: `MonitorId`, `DeviceName`, `Theme`). The current `Engine` already holds `DisplayScreenType`, `ScreenDevice`, `ScreenType` (`ScreenArea`), `Stretch` [VERIFIED: crates/sucrose-config/src/model.rs:223-330, `pub display_screen_type: DisplayScreenType`, `pub screen_device: String`].
- **Span:** one renderer over the union of `MonitorInfo.rect` (or `.work` for WorkingArea), origin possibly negative; the engines get the union size; stretch applies to the union. **Working area / bounds (DESK-04):** `MonitorInfo` already carries both rectangles (`rect`, `work`) [VERIFIED: crates/sucrose-platform/src/geometry.rs MonitorInfo]. **Duplicate:** one renderer per monitor with the same job (D-04), master clock per Pattern 9.
- **Stretch per engine (DESK-05, engines.md `3):**

| Engine | None | Fill | Uniform | UniformToFill |
|--------|------|------|---------|---------------|
| web generated Video/Gif page | CSS `object-fit: none` | `fill` | `contain` | `cover` (Gif page: `SucroseStretchMode('<Name>')`) |
| mpv | `video-unscaled=yes` | `keepaspect=no` | default | `panscan=1.0` |
| libVLC | `set_scale(1)` | `set_aspect_ratio(view W:H)` | default | `set_crop_geometry(view W:H)` |
| built-in GIF / Media Foundation | 1:1 centered | dest = surface | letterbox | crop by source rect |
| YouTube / URL / Web page | not applied (Web gets `StretchMode` through the compatible API) | | | |

### 8. Input forwarding (WAPI-07)

Keep the legacy mechanism (it is proven on both Chromium shells): `RegisterRawInputDevices` for mouse and keyboard with `RIDEV_INPUTSINK` on a hidden window of the renderer, `GetRawInputData` on `WM_INPUT`, mouse position from `GetCursorPos` relative to the monitor under the cursor (span: the virtual-screen origin), `PostMessageW` of `WM_MOUSEMOVE` (wParam `0x20`), `WM_LBUTTONDOWN/UP` (`0x1`), `WM_RBUTTONDOWN/UP` (`0x2`), `WM_MBUTTONDOWN/UP` (`0x10`), lParam `(Y<<16)|X` to the Chromium widget window; wheel via CDP `Input.dispatchMouseEvent` (WebView2, divided by DPI × zoom) and `SendMouseWheelEvent` (CEF); keys `WM_KEYDOWN/UP` with the legacy lParam, dead keys skipped, the fake click at (99999, 99999) in OnlyKeyboard; target window: CEF host HWND → `Chrome_WidgetWin_0` else `_1`; WebView2 HWND → `Chrome_WidgetWin_0` → `Chrome_WidgetWin_1` [CITED: engines.md `5]. `InputType` (`Close`, `OnlyMouse`, `OnlyKeyboard`, `MouseKeyboard`), `InputDesktop` (forward only while the desktop is foreground) and `InputModuleType` (only `RawInput`) are already in settings. New with per-monitor instances: a renderer forwards only when the cursor is inside its own rectangle (span: always). CEF blocked keys (`Ctrl+F`, `Ctrl+R`, `F5`, `Ctrl+±`, `F12`...) become a key handler. Alt combinations did not work in the legacy and are not promised.

### 9. Performance governor, sound rules, signals

| Condition | Windows signal (Rust) | Notes |
|-----------|----------------------|-------|
| Lock, Unlock, Session logon/logoff, Console connect/disconnect, Remote connect/disconnect | `WTSRegisterSessionNotification` and `WM_WTSSESSION_CHANGE` with `WTS_CONSOLE_CONNECT 0x1`, `CONSOLE_DISCONNECT 0x2`, `REMOTE_CONNECT 0x3`, `REMOTE_DISCONNECT 0x4`, `SESSION_LOGON 0x5`, `LOGOFF 0x6`, `LOCK 0x7`, `UNLOCK 0x8` [VERIFIED: learn.microsoft.com WM_WTSSESSION_CHANGE, fetched] | Event driven, 0/0 debounce. Replaces legacy SENS and SystemEvents. Needs a window handle: the platform event loop's hidden window. |
| Sleep | `WM_POWERBROADCAST` `PBT_APMSUSPEND`/`PBT_APMRESUMEAUTOMATIC` (or `RegisterSuspendResumeNotification`) [ASSUMED: A17] | 0/0. |
| Focus | foreground window class is `Progman`/`WorkerW`/`SHELLDLL_DefView` (existing `foreground_window()` in the platform) | 5/3, 1 s poll while armed. |
| FullScreen | existing rule (`fullscreen.rs`) with scope | already built in Phase 2. |
| ScreenSaver | `SystemParametersInfo(SPI_GETSCREENSAVERRUNNING)` [ASSUMED: A18, the legacy likely did the same] | 0/0, 1 s poll while armed. |
| Memory | `GlobalMemoryStatusEx` | 5/3; threshold `MemoryUsage` 80. |
| Processor | PDH `Processor\% Processor Time\_Total` | 5/3, `ProcessorUsage` 70; counters opened only when armed. |
| Graphic | PDH `GPU Engine` by adapter LUID, `engtype_3D` | 5/3, `GraphicUsage` 70; the instance list must be re-synced each poll (legacy), the costly part: measure. |
| Network | ICMP ping, PDH bytes/s; **PERF-07 fix: a sub-check counts only when its threshold is above 0** | order Ping, Upload, Download as legacy, remembered sub-type for recovery. |
| Battery / BatterySaver | `GetSystemPowerStatus`: `ACLineStatus`, `BatteryLifePercent`, `SystemStatusFlag` ("The status of battery saver... To be notified when this value changes, call RegisterPowerSettingNotification with GUID_POWER_SAVING_STATUS") [VERIFIED: learn.microsoft.com SYSTEM_POWER_STATUS, fetched] | One cheap call covers both rules; WMI only for the web Battery payload. |
| Remote (tools), Virtual | session events above for RDP; for the 52-entry and 40-entry process lists see Open Question 4 | polled/evented only while the rule is not Resume. |

Sound rules: global volume × per-wallpaper volume (D-06); `VolumeDesktop` = silence unless the desktop is foreground; `VolumeSilent` = silence while another app plays. "Others silent" (PLAY-04): enumerate the default render endpoint's sessions with `IAudioSessionManager2::GetSessionEnumerator` (documented method [CITED: audiopolicy docs, fetched]) every `VolumeSilentSensitivity` seconds (the maintainer's is 10; the C# app scanned every *process*): a session counts if it is active, unmuted, volume > 0 and its process is not in the Sucrose tree (service pid, renderer pids and descendants via a ToolHelp parent map built per poll). Event-driven (`IAudioSessionNotification` + per-session events) is an optimisation, not required. Per-process volume of web engines: enumerate the same sessions and set `ISimpleAudioVolume` on those inside the renderer's tree (CEF: the audio utility process is a descendant of the renderer; WebView2: Pitfall 3). Heavy/Light and sound rules never touch process state beyond audio sessions.

### 10. Application wallpapers (TYPE-06)

- **Start:** `Info.Source` must be a local exe inside the theme folder (legacy). Build the command with `CreateProcessW` and an argument vector: `Arguments` stays **one quoted argument** exactly as the legacy wrapped it (D-30), launched into the renderer's existing job object (kill on close, no orphan trees; `job.rs` exists in the platform) and with the GPU preference registry value for that exe (`apply_gpu_preference` pattern).
- **Window discovery (D-29):** `SetWinEventHook` with `EVENT_OBJECT_SHOW` (and `EVENT_OBJECT_CREATE`) over all processes (the hook takes an `idProcess` filter for one process only [VERIFIED: SetWinEventHook parameters, fetched], so use 0 and filter by ancestry against the started pid with a ToolHelp parent map), top-level windows only (`idObject == OBJID_WINDOW`, no owner), first visible main window of the tree, prefer class `Engine`; timeout (say 15 s) then close the process and report to log and tray. If a visible top-level window has a button named `Play!` and no `Engine` window exists, `BM_CLICK` it (legacy compatibility, D-29). [ASSUMED: A19 — hook flags and filtering details; spike.]
- **Embed:** strip `WS_CAPTION`, `WS_THICKFRAME`, `WS_MINIMIZE`, `WS_MAXIMIZE`, `WS_SYSMENU`, `WS_DLGFRAME`, `WS_BORDER` and `WS_EX_CLIENTEDGE` (legacy masks; note the legacy mixed an ex-style flag into the style mask, port the intent), add `WS_CHILD`, `SetParent(real_hwnd, surface)`, `MoveWindow` to the surface rect, remove from taskbar; never use the process handle as a window handle (legacy bug). Cross-process `SetParent` attaches the two input queues; foreign DPI-unaware apps are scaled by their own manifest.
- **Count (D-28):** one process per assigned monitor in PerDisplay and Duplicate, one over the union in Span; volume only on the master/primary instance as the legacy did for the first instance (D-05 per-wallpaper volume does not apply: an app has no property set).
- **Input:** WAPI-07 for Application posts to four targets in the legacy (first child of the visible top-level window, the main window, the process main window, the global class `Engine` window) only for the instance whose monitor holds the cursor; keep that.

### 11. Duplicate sync for web and URL pages (D-04)

Feasible, best effort: (1) **barrier start:** each renderer loads its page and reports `Ready`; the service sends `Barrier{t0_ns}` after all are ready, and each renderer starts the page's time-dependent activity at `t0` on the shared `monotonic_ns` clock (generated pages: `video.play()` at `t0`; arbitrary web pages cannot be held before load, so for them the barrier is "reload at t0 + load budget" only when the user enables sync, otherwise skip); (2) **shared time base shim:** inject at document start a script that offsets `performance.now()`, `Date.now()` and the `requestAnimationFrame` timestamp by (shared clock − page start), which aligns pages that animate from elapsed time; (3) **animations:** after load, set `currentTime` on `document.getAnimations()` to the shared time so CSS animations line up. Pages that use randomness, network data or user input diverge by nature. Mechanism: WebView2 `AddScriptToExecuteOnDocumentCreated`; CEF has no direct equivalent without render-process code, so inject through CDP `Page.addScriptToEvaluateOnNewDocument` [ASSUMED: A20]. Time-based generated pages (Video, Gif, YouTube) use the clock follower (Pattern 5). Record "web/URL pages: best effort, random/stateful pages may differ" in FEATURES. A deterministic fixture page (canvas drawn from `performance.now()`) proves < 1 frame skew on two monitors.

### 12. YouTube Referer under the D-14 origin

The YouTube required-minimum-functionality summary says "Embedded players must identify the client through the HTTP Referer header" [CITED: developers.google.com/youtube/terms/required-minimum-functionality, via fetch summary]. Under D-14 the page is a real `https://sucrose.wallpaper.engine/` document, so the browser sends that origin as Referer for the embed request on both engines (default `strict-origin-when-cross-origin`; the legacy page also sets `widget_referrer`). The WebView2-only injection of `Referer: https://sucrose.wallpaper.engine/` for `*youtube.com/*` was needed because the old page did not always carry one; with D-14 it is a safety net. Recommendation: **no injection by default, keep an equivalent hook in both adapters (WebView2 WebResourceRequested filter, CEF `OnBeforeResourceLoad`) behind a small function that only fills a missing Referer**, and make the slice-3 acceptance test play the 3 Showcase YouTube themes and check for the embed error codes (150/152/153) on both engines [ASSUMED: A21 — whether the player accepts the origin-only Referer must be confirmed by that run]. Note for the record: `hideIframeElement` reaches into the cross-origin iframe's document and only works because `disable-web-security` is a default switch; it also changes the player UI beyond what the API documents (YouTube terms); both are legacy behavior kept under D-021.

### 13. Per-engine RAM/CPU with `cargo xtask measure`

`measure` samples process statistics per process (RAM working set, private bytes, CPU as percent of the machine, GPU by `typeperf`) and never starts or stops a process [VERIFIED: xtask/src/measure.rs header]. For Phase 4 add engine rows: the web engines spawn process trees, so the sampled set must be "the renderer plus all descendants" (CEF: browser, GPU, network, audio, renderer processes; WebView2: the shared `msedgewebview2.exe` tree, which belongs to the first renderer and is shared when several renderers share a UDF); compare against the C# CefSharp/WebView engines on the same theme and the same adapter (`--row` refuses mixed adapters, G-02-6). Rows to record: built-in GIF vs mpv on the same GIF (D-10), CEF vs WebView2 vs C# on a Web theme, Light vs Heavy paused RAM per engine, N monitors (1, 2) per engine, libVLC and Media Foundation vs mpv on one video. Release builds only.


## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP/1 local server for web themes | A loopback server (legacy Watson) | CEF scheme handler / WebView2 WebResourceRequested over one resolver | No port, no CORS surface, one traversal-tested function (SEC-02). |
| Path containment | String prefix checks on user paths | Decode once, reject `..`/backslash/drive/UNC, `canonicalize`, `starts_with` canonical root | Junctions, symlinks, encoded forms and case are classic bypasses. |
| Chromium process suspension | `DebugActiveProcess`, `NtSuspendProcess` | `TrySuspend`, CDP `Page.setWebLifecycleState`, hidden HWND, page-level pause | PLAY-05; the legacy approach attaches a debugger. |
| Authenticode verification | Parsing PE signatures | `WinVerifyTrust` (+ signer subject check) | Chain, revocation and timestamp rules are the OS's. |
| FFT, window function | A custom FFT | `realfft` with a Hann table | Parity with the legacy bands is about the post-processing, not the transform. |
| WMI access | Hand-written `IWbemServices` COM | `wmi` crate | Safe API, no unsafe in the caller. |
| Animated image decoding | GIF/APNG/WebP parsers | `image` animation decoders | Already in the tree; frames are correct under disposal rules. |
| 7z / tar.bz2 handling in xtask | Shelling out to `7z` or `tar` | `sevenz-rust2` (exists), `tar` + `bzip2` | xtask must run the same from PowerShell and WSL (D-014). |
| Single-writer atomic file write for property copies | New write code | The existing atomic writer of `sucrose-config::store` | Phase 3 proved it with a kill test. |
| Debounce 5/3 | A second implementation | The fullscreen rule's debounce, extracted | One tested state machine for all conditions. |
| Clock following | Engine-specific correction code | The shared `Follower` | Phase 2 tuned the thresholds on real hardware. |

**Key insight:** every engine integration here is thin glue around three shared pure pieces: the origin resolver, the JS builders and the clock follower. Keep the risk in the glue small and put every rule that can be wrong in the pure code where `cargo test` on any OS covers it.

## Runtime State Inventory

This is not a rename phase, but it migrates and introduces state; each category answered:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | 34 C# property copies in `%APPDATA%\Sucrose\Cache\Properties` (e.g. `<theme>MpvPlayer`); delta files `<guid>.<Theme><Type>` are noise. New stores: `<data_dir>/Properties/`, web profiles (Open Questions 2, 5), `Engine.DisplayAssignments`. | One-time **data migration** (copy, D-20) plus code edits for the new writers. The settings migration gains the PerDisplay+ScreenDevice -> assignment step (code + a migration step bumping nothing in `Version` if additive; **if the struct shape needs a `Version` bump, add the migration step in `sucrose-config`**). |
| Live service config | None outside the repo: no external service holds engine names. The Warehouse `WebViewTime`/`CefSharpTime` snooze fields are already migrated (Phase 3 D-06). | None. |
| OS-registered state | GPU preference registry values per exe (existing `apply_gpu_preference` for `sucrose.exe`); new: the application exe of an Application theme and `msedgewebview2.exe` (legacy did both). The C# autostart entry is untouched (D-034). | Code edit: extend the GPU preference targets; no re-registration of existing entries. |
| Secrets/env vars | `CEF_PATH` (build-time only, set by xtask). `SUCROSE_DATA_DIR` unchanged. | Code edit in xtask; none for runtime. |
| Build artifacts | `target/engines/<arch>/` gains CEF and libVLC trees; the CEF runtime files (`libcef.dll`, `*.pak`, `icudtl.dat`, `v8_context_snapshot.bin`, `locales/`, `chrome_elf.dll`...) must sit next to `sucrose.exe` for dev runs; a stale `libmpv` copy is the existing precedent. | `fetch-engines` copies them next to the dev exe; stale-copy check by hash. |

## Common Pitfalls

### Pitfall 1: CEF subprocesses run the service's startup code
**What goes wrong:** `sucrose.exe --type=gpu-process ...` is rejected by the strict command-line parser or starts a second service/tray/single-instance check. **Why:** CEF re-launches the same executable. **How to avoid:** `execute_process` is the first call of `main` and returns before anything else when it is a subprocess; the parser treats `--type=` as CEF's. **Warning signs:** a flood of renderer restarts, "AlreadyOpen" in the log, extra tray icons.

### Pitfall 2: One persistent CEF profile cannot be shared by per-monitor browser processes
**What goes wrong:** the second renderer's `cef::initialize` fails or relaunches into the first (process singleton on `root_cache_path`, CEF 120+, quoted above) and the wallpaper on monitor 2 is blank. **How to avoid:** a profile allocator: the first live CEF instance takes `<web data>/Cef/Profile` through a lock file; any other concurrent instance gets its own ephemeral `root_cache_path` under `<web data>/Cef/Slot-<n>` (wiped at start), or an empty `cache_path` (incognito; localStorage in memory). Never derive the persistent profile from the slot number. This keeps D-14's promise for the first instance and degrades predictably for the rest. **Open Question 2.**

### Pitfall 3: WebView2 renderers on one user data folder share one audio service
**What goes wrong:** per-wallpaper volume (D-06) on two monitors changes both, because the audio sessions belong to the one shared browser process tree; WebView2 offers `IsMuted` per view but no volume. **How to avoid:** use the same profile allocator for WebView2 (the second concurrent instance gets its own UDF, hence its own browser process), accept the RAM cost, and document it; for generated pages also set `video.volume` in the page. **Warning signs:** the volume slider on monitor 2 moves monitor 1.

### Pitfall 4: `WasHidden` does nothing for a windowed CEF browser
Documented as OSR-only (quoted in Pattern 4). Heavy = hide HWND + CDP freeze; verify with the RAM/CPU rows that Heavy really lowers renderer CPU to ~0 and keeps the position.

### Pitfall 5: Chromium child windows inside the layered Progman surface
**What goes wrong:** a black or invisible browser on some Windows 11 24H2+/26H2 layouts, wrong DPI scale (a parent in WorkerW gets no `WM_DPICHANGED`), or a child that does not follow `set_rect`. **How to avoid:** render proof per surface strategy in slice 1/2; always size in physical pixels; WebView2 `RasterizationScale` from `MonitorInfo.dpi` and `ShouldDetectMonitorScaleChanges` off; CEF `SetWindowPos` plus `NotifyMoveOrResizeStarted` on `SetRect`. Phase 2's Pitfall 1 (surface attached but not visible) applies unchanged.

### Pitfall 6: Media over custom resource handlers needs Range
**What goes wrong:** videos in web themes cannot seek or stall at the end. **How to avoid:** both adapters honour `Range` (206, `Content-Range`, `Accept-Ranges: bytes`) and the resolver exposes file length; a fixture with a large local video seeks on both engines. If WebView2's custom path is unusable, fall back to host mapping for that engine (accepting its documented media slowness) and record it.

### Pitfall 7: 64 KiB frames
Thumbnails and property documents exceed `MAX_FRAME`. Use `Blob` chunks (Pattern 7); test the reassembly cap and an out-of-order/oversized chunk.

### Pitfall 8: Legacy text parity of the compatible API
C# `string.Format` makes booleans `True`/`False` and Newtonsoft `Formatting.Indented` makes multi-line JSON; pages written against that text break on `true` or on minified JSON. Pin the exact output in fixtures (the Newtonsoft default is two-space indent and `Environment.NewLine`, CRLF on Windows: A22 — capture one real call string from the maintainer's C# app, read-only, as the fixture instead of trusting memory).

### Pitfall 9: Switch merging
`CefArguments` is stored as a sorted map (`BTreeMap<String, String>` in `model.rs`) and `WebArguments` as a list. Chromium honours the last occurrence of a repeated switch, so Sucrose's own `--enable-features` (WebView2 developer port adds `msEdgeDevToolsWdpRemoteDebugging`) must be **merged** into the user's `enable-features`/`disable-features` values, not appended as a second switch (the legacy default list already has an `enable-features`). [ASSUMED: A23 last-occurrence behavior.]

### Pitfall 10: CEF in CI and on the maintainer's machine
The wrapper is compiled by CMake/Ninja on every Windows build job (windows-latest, windows-11-arm, and the i686 job), and the dist is ~170 MB per architecture. Cache the extracted dist keyed by the pin, export `CEF_PATH` from xtask, and keep Linux/macOS jobs free of CEF by gating the engine crates with `cfg(windows)` in the manifest. Do not let `cargo clippy --workspace` on Linux see `cef`.

### Pitfall 11: Per-monitor DevTools ports collide
Each CEF browser process binds its own port; a shared `DeveloperPort` makes the second fail. Use `port + slot` for CEF; WebView2 shares one browser process per UDF so one port serves all; the Portal's "Open DevTools" picks the monitor and the service routes `OpenDevTools` to that renderer.

### Pitfall 12: GIF memory and disposal
Frames depend on earlier frames (disposal); never cache all canvases; downscale on decode to the surface size; reopen the file at loop end; free everything on Heavy.

### Pitfall 13: Foreign windows fight the embed (Application)
Some apps re-add styles, move themselves or recreate their main window. Re-apply style and rect on `EVENT_OBJECT_LOCATIONCHANGE` for the embedded HWND with a cap, report persistent failure, and never loop forever (restart budget as for renderers).

### Pitfall 14: Pinned URLs disappear
Phase 2 Pitfall 3 stands: VideoLAN keeps releases for years but not forever, and the CEF CDN has a long history; mirroring remains the Open Question in DECISIONS.md. The hash never changes.

## Code Examples

Shapes only; every API name marked [ASSUMED] must be confirmed against the crate docs at compile time.

### Origin constants and a resolver skeleton (pure, any OS)
```rust
// Source: docs/legacy/formats.md (Url.Local) and the D-14 decision text.
pub const ORIGIN: &str = "https://sucrose.wallpaper.engine/";
pub const HOST: &str = "sucrose.wallpaper.engine";

pub enum Reject { Empty, Nul, Backslash, DriveOrColon, Unc, Dots, Outside, NotAFile, Io }

pub fn resolve(url_path: &str, roots: &[&std::path::Path]) -> Result<ResolvedFile, Reject> {
    let path = url_path.split(['?', '#']).next().unwrap_or("");
    let decoded = percent_encoding::percent_decode_str(path).decode_utf8().map_err(|_| Reject::Nul)?;
    // a second decode that changes the text means double encoding: reject, never decode twice
    if percent_encoding::percent_decode_str(&decoded).decode_utf8_lossy() != decoded { return Err(Reject::Dots); }
    if decoded.contains('\0') { return Err(Reject::Nul); }
    if decoded.contains('\\') { return Err(Reject::Backslash); }
    if decoded.starts_with("//") { return Err(Reject::Unc); }
    let rel = decoded.trim_start_matches('/');
    if rel.is_empty() || rel.contains(':') { return Err(Reject::DriveOrColon); }
    if rel.split('/').any(|s| s == ".." ) { return Err(Reject::Dots); }
    for root in roots {                       // theme folder first, then generated content (legacy order)
        let candidate = root.join(rel);
        if let (Ok(c), Ok(r)) = (candidate.canonicalize(), root.canonicalize()) {
            if c.starts_with(&r) && c.is_file() { return Ok(ResolvedFile::new(c)); }
        }
    }
    Err(Reject::NotAFile)
}
```

### Legacy property call text
```text
// Source: docs/legacy/engines.md section 6a (every bundled showcase theme uses this listener)
SucrosePropertyListener('<key>', <control as indented JSON>);
```

### Delay-load build script (spike target, [ASSUMED] A1)
```rust
// apps/sucrose/build.rs
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo::rustc-link-arg-bins=/DELAYLOAD:libcef.dll");
        println!("cargo::rustc-link-lib=delayimp"); // cef-dll-sys already links delayimp
    }
}
```

### Process entry order ([ASSUMED] shape of the `cef` API)
```rust
fn main() {
    // 1. CEF subprocesses must exit here before any Sucrose logic runs.
    #[cfg(windows)]
    if let Some(code) = sucrose_engine_cef::try_run_subprocess() { std::process::exit(code); }
    // 2. normal strict command line: service, --render <slot>, ...
}
```

### Session notifications on the platform event loop
```rust
// Source: WM_WTSSESSION_CHANGE values quoted in Findings 9. The window is the EventLoop's hidden window.
// WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION); on WM_WTSSESSION_CHANGE map wParam to LoopEvent::Session(code).
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Debugger attach / NtSuspendProcess to pause Chromium | WebView2 TrySuspend, CDP lifecycle freeze, page visibility | PLAY-05 decision | No fragile process surgery; position kept. |
| Loopback HTTP server for CEF | Scheme handler on a virtual HTTPS origin | D-14 | No open port, same origin as YouTube's expectation. |
| SENS/SystemEvents, 2 s process scans | WTS session notifications, power broadcasts, bounded scans | D-23 | No always-on polling. |
| `PerformanceCounter` off = no web data | Decoupled: rules vs web data | D-24 | Behavior change to record. |
| Accept every certificate | Validate by default, opt-in switch | D-12 | Behavior change to record. |
| CefSharp 156.0.150 | `cef` 156.7.0+156.0.31 (same CEF 156 line) | this phase | Web behavior should match the legacy Chromium generation. |
| libVLC 3.0.24 via NuGet | Same version from the VideoLAN archive, own bindings | this phase | Same engine generation. |

**Deprecated/outdated:** `MF_MEDIA_ENGINE_PLAYBACK_HWND` is flagged by Microsoft as a legacy HWND mode (use OPM_HWND or frame-server mode if it misbehaves); the legacy `WMI Win32_ProcessStartTrace` route may need elevation (A12); VLC 4.0 is not used (3.0.24 is the stable line).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Delay-loading `libcef.dll` with `/DELAYLOAD` works with the `cef`/wrapper link and CEF subprocesses | Findings 1, Slice 1 | CEF would load in the service; fallback is a separate helper exe (more code, deviates from "same binary"). |
| A2 | CDP `Page.setWebLifecycleState` with `frozen` exists in CEF 156 | Pattern 4 | Heavy for CEF falls back to CPU throttling + hidden HWND. |
| A3 | A working `cef` build for `i686-pc-windows-msvc` exists | Findings 1 | Win x86 has no CEF; reported like a missing engine (D-027). |
| A4 | The static-CRT wrapper links cleanly into the workspace's default-CRT Rust build (or needs `crt-static`) | Slice 1 plan 2 | Link errors; global `crt-static` changes every binary. |
| A5 | CI runners have CMake/Ninja/MSVC for the wrapper build | Pitfall 10 | CI setup work. |
| A6 | `image`'s animation iterators decode lazily | Findings 4 | Memory spikes; switch to the `gif`/`png`/`image-webp` decoders directly. |
| A7 | `webview2-com-sys` links the static loader | Findings 2 | A loader DLL has to ship and be located. |
| A8 | WebView2 `ServerCertificateErrorDetected` + `AlwaysAllow` names | Findings 2 | Compile-time fix. |
| A9 | The `windows` crate feature names for MF, D2D, WASAPI, GSMTC, WTS, RawInput, WinTrust, DisplayConfig | Standard Stack | Compile-time fix. |
| A10 | Shared UDF across processes needs identical environment options | Findings 2 | Second renderer fails to create its environment. |
| A11 | `WinVerifyTrust` + signer-subject check API names | Findings 2 | Compile-time fix; the negative test catches a wrong check. |
| A12 | Process start/stop WMI events need elevation for a standard user | Open Question 4 | If no elevation needed, D-23 is met by events as written. |
| A13 | API shapes of `wmi`, `realfft`, `tar`, `bzip2` (seam OK, not read) | Standard Stack | Small. |
| A14 | libVLC finds `plugins/` next to `libvlc.dll` | Findings 3 | Needs `VLC_PLUGIN_PATH` or an option. |
| A15 | Direct2D/MF HWND modes render in the layered Progman child | Findings 4, 5 | Use frame-server / GDI fallback. |
| A16 | Monitor device path is stable across reboot/replug; VDD included | Findings 7 | Add EDID-based key. |
| A17 | Sleep via `WM_POWERBROADCAST` (`PBT_APMSUSPEND`) | Findings 9 | Small. |
| A18 | Screen saver via `SPI_GETSCREENSAVERRUNNING` | Findings 9 | Another signal (desktop switch) needed. |
| A19 | WinEvent hook flags/filtering for Application windows | Findings 10 | Fall back to timed `EnumWindows`. |
| A20 | CDP `Page.addScriptToEvaluateOnNewDocument` available for CEF | Findings 11 | Best-effort sync reduced to barrier + animations. |
| A21 | YouTube accepts the origin-only Referer of the virtual origin | Findings 12 | Keep the Referer injection (hook exists). |
| A22 | Newtonsoft `Indented` = two spaces and CRLF on Windows | Pitfall 8 | Parity break in compatible calls; fixture capture decides. |
| A23 | Chromium honours the last occurrence of a repeated switch | Pitfall 9 | Merge logic still harmless. |
| A24 | CEF dist size, `chrome://credits` availability | Findings 1 | Package-size and notice work in Phase 5. |

## Open Questions for the Maintainer (RESOLVED)

**Resolved on 2026-11-26:** the maintainer confirmed the recommended option for all five questions. They are recorded in `04-CONTEXT.md` as D-32 (question 1), D-33 (2), D-34 (3), D-36 (4) and D-35 (5).

These need a maintainer decision (not covered by a locked decision, or in tension with one). Recommended option first; research does not stop for them.

1. **CEF link model and the cargo rule (resolved: recommended option, D-32).** D-013 names the `cef` crate, but `.claude/rules/cargo.md` forbids "any crate that links an engine runtime at build time" and says engine binaries come "never from a crate"; `cef-dll-sys` links `libcef` through an import library and downloads CEF unless `CEF_PATH` is set. **Recommended:** keep D-013, add an explicit exception line to `cargo.md` for `cef` and `webview2-com`, always build with `CEF_PATH` pointing at the `fetch-engines` result (D-028 stays true), and delay-load `libcef.dll` so the service and mpv renderers never load it; if the slice-1 spike shows delay-load does not work, ship a second executable (`sucrose-cef.exe`) as the CEF renderer and subprocess. Alternatives: hard link (rejected: service stops being near idle), separate exe from the start (simpler, but breaks "same binary in renderer mode", D-008). Also accept in advance: if `cef` has no working Windows x86 build, CEF is reported as missing on x86 (D-027).
2. **Persistent web profile with several concurrent web wallpapers (resolved: recommended option, D-33).** CEF's process-singleton lock and WebView2's shared audio service mean N per-monitor web instances (D-04) cannot all use the one persistent profile (D-14) *and* have separate volumes (D-06). **Recommended:** the first running web instance per engine uses the persistent profile (lock file); every other concurrent instance gets its own ephemeral profile. Effect: localStorage/IndexedDB/cookies persist only for the first monitor's wallpaper; volumes are independent. Alternatives: (a) per-slot persistent profiles (data differs by monitor), (b) one browser process hosting all monitors (breaks one-renderer-per-monitor isolation), (c) share the profile and document that WebView2 volume is per profile.
3. **Where OS data providers live (D-011).** `ARCHITECTURE.md` puts CPU/RAM/GPU/network/battery/audio/media providers in `sucrose-sysinfo`, but D-011 allows `unsafe` only in `sucrose-platform` and engine/FFI crates (PDH, WASAPI, GSMTC, ICMP need it; WMI via `wmi` does not). **Recommended:** a new DECISIONS entry that adds `sucrose-sysinfo` to the unsafe-allowed list (it is the platform-provider crate, per-OS `cfg` inside it), keeping `sucrose-core` free of OS code. Alternative: put every OS call in `sucrose-platform` and keep `sucrose-sysinfo` pure.
4. **Process start/stop events for Remote and Virtual (D-23).** The documented `Win32_ProcessStartTrace` class does not say whether a standard user may subscribe; in practice it often needs elevation (A12). **Recommended:** spike on the maintainer's non-elevated session in slice 6; if it is denied, accept a bounded ToolHelp snapshot every 2 s only while the rule is not Resume (the legacy cadence at native cost), and say so in FEATURES. Alternative: a window-event heuristic (misses windowless tools).
5. **Where persistent web data lives.** D-14 gives each engine a Rust-owned folder but not its location; the C# ones were under `Cache` (cleaned by Phase 5 cache cleaning), and D-20 moved properties out of `Cache` for exactly that reason. **Recommended:** `%APPDATA%\Sucrose\Web\WebView2` and `...\Web\Cef\...` (outside `Cache`), so cache cleaning never deletes a wallpaper's localStorage. Alternative: under `Cache` and accept loss on cleaning.

## Open Questions (technical, resolved by the planned spikes)

1. **Does delay-load work for CEF?** (A1) — first plan of slice 1; outcome selects Open Question 1's branch.
2. **Do WebView2 Range streams and CEF handlers seek large videos?** — slice 1/2 fixture.
3. **Is the monitor device path stable for the VDD and across ports?** (A16) — slice 4 spike.
4. **Does Direct2D / MF HWND mode draw in the layered surface on all three strategies?** (A15) — slice 3 and 7.
5. **Does the second GPUI window open and position from the running Portal?** — slice 5 spike (UI-SPEC placement rules).
6. **Heavy for mpv: does `vid=no` lower RAM enough, or is stop-and-reload needed?** — slice 6 measurement.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (Windows wrapper in WSL) | everything | ✓ | cargo 1.99.0, rustc 1.99.0, `rust-toolchain.toml` channel 1.99.0 | — |
| CMake | cef-dll-sys wrapper build | ✓ | at `D:\Program Files\CMake\bin\cmake.exe` (on PATH) | — |
| Ninja | cef-dll-sys (generator "Ninja") | ✗ on PATH | VS 2026 Insiders ships `ninja.exe` under `Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja` | add that folder to PATH for the build in xtask, or install Ninja |
| MSVC (`cl.exe`, linker) | Rust MSVC target, CEF wrapper | installed (Visual Studio 2026 Insiders at `D:\Program Files\Microsoft Visual Studio\2026\Insiders`); not on the bare PATH | — | build from a developer prompt / xtask sets the environment |
| libclang | bindgen for libVLC | unknown (`LIBCLANG_PATH` unset); Phase 2 already generated libmpv bindings | — | Phase 2's setup |
| 7-Zip | developer convenience | ✓ | `D:\Program Files\7-Zip\7z.exe` (xtask does not need it) | `sevenz-rust2` |
| WebView2 Runtime | WebView2 engine | ✓ | 158.0.4370.33 (> 131.0.2903.70) | — |
| VC++ Redist ≥ 14.40.33816.0 | CEF prerequisite check (legacy rule) | not probed | — | probe in slice 1; note a static-CRT build may not need it (A4) |
| CEF and libVLC archives | engines | not downloaded yet | CDN reachable (index fetched) | `fetch-engines` |
| Network | pin downloads | ✓ | — | `--from` a local archive (existing option) |
| Second display | slice 4 | ✓ per maintainer notes (DISPLAY1 physical, DISPLAY9 VDD) | — | — |
| Application theme, URL theme | slice 3, 8 | ✗ none in the library | — | prepared under `SUCROSE_DATA_DIR` (tiny Win32 test app as its own crate/xtask output) |
| Docker | Linux build check | ✓ (Phase 3) | — | CI |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** Ninja on PATH (use the VS-bundled one), VC++ Redist probe.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in harness, Rust 1.99.0); GPUI `TestAppContext` available behind `test-support` if needed |
| Config file | per-crate `Cargo.toml`; fixtures under each crate's `tests/fixtures/` |
| Quick run command | `cargo test -p <touched crate>` (e.g. `-p sucrose-engine-web -p sucrose-engine -p sucrose-core`) |
| Full suite command | `cargo xtask ci` (fmt, clippy `-D warnings`, test) — as Phase 3 |

Engine and OS behavior cannot run in CI sandboxes: those requirements get an automated pure part (decision logic, formats, resolver, state machines) and a **prepared maintainer run** on a release build with `SUCROSE_DATA_DIR` scratch (screenshots through `PrintWindow` per the port-feature skill; no synthesized input).

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SEC-02 | Resolver rejects `..`, `%2e%2e`, `%252e%252e`, backslash, drive, UNC, NUL, junction; theme-then-content order | unit | `cargo test -p sucrose-engine-web origin` | ❌ Wave 0 |
| FMT-03 | `SucroseProperties.json`/`SucroseCompatible.json` parse and round-trip from real files; unknown control kept | unit + fixtures | `cargo test -p sucrose-core properties compatible` | ❌ Wave 0 |
| FMT-07, WAPI-01, WAPI-02 | JS builders produce legacy text (`True`/`False`, indented JSON, template order, `filedropdown` scope) | unit + fixtures | `cargo test -p sucrose-engine-web js` | ❌ Wave 0 |
| WAPI-01..04, WAPI-07, WAPI-09 | A fixture page receives calls on CEF/WebView2; input events arrive | maintainer run + engine integration test (Windows) | `cargo test -p sucrose-engine-cef --test fixture_pages` (ignored by default, run locally) | ❌ Wave 0 |
| WAPI-05 | Visualizer: band edges, tilt, envelope on a synthetic sine/chirp; 128×3 arrays | unit | `cargo test -p sucrose-sysinfo visualizer` | ❌ Wave 0 |
| WAPI-06 | Thumbnail chunking/reassembly bounds | unit | `cargo test -p sucrose-ipc blob` | ❌ Wave 0 |
| WAPI-10 | `CefArguments` <-> `WebArguments` conversion, Reset defaults, read-only switches, merge of `enable-features` | unit | `cargo test -p sucrose-engine-web args` | ❌ Wave 0 |
| ENG-06, ENG-07 | `choose_engine` table per type incl. missing/unsupported-arch/not-loadable; version comparison of WebView2 and VC++ | unit | `cargo test -p sucrose-engine choose` | ❌ Wave 0 |
| ENG-02, ENG-03, ENG-04, ENG-05 | Each engine plays its type; pause/volume/loop | maintainer run (slice checkpoints) + smoke tests | `cargo test -p sucrose --test engines -- --ignored` | ❌ Wave 0 |
| TYPE-02 | GIF/APNG/WebP decode, delays, loop, bounded memory | unit + fixtures | `cargo test -p sucrose-engine-builtin gif` | ❌ Wave 0 |
| TYPE-03, TYPE-04, TYPE-05 | Use enabled, pages load, YouTube playlist/shuffle/loop | maintainer run | slice checkpoints | n/a |
| TYPE-06 | Window discovery filter, style mask, process count per mode | unit (pure parts) + maintainer run with the test app | `cargo test -p sucrose-platform embed` | ❌ Wave 0 |
| DESK-02..05 | Assignments, migration of PerDisplay+ScreenDevice, span union, stretch mapping, monitor id fallback | unit | `cargo test -p sucrose-config -p sucrose-core -p sucrose display` | ❌ Wave 0 |
| PLAY-01..04 | Effective volume = global × per-wallpaper; desktop-focus and others-silent predicates; own-tree exclusion | unit | `cargo test -p sucrose sound` | ❌ Wave 0 |
| PLAY-05, PERF-01..04, PERF-06..09 | Governor: legacy order, first match, only recovery while active, 5/3 and 0/0, PERF-07, scope, lists | unit (pure state machine) | `cargo test -p sucrose governor` | ❌ Wave 0 |
| PERF-02, PERF-03, PERF-09 | Real lock/user switch/screen saver/RDP signals | maintainer run | slice 6 checkpoint | n/a |
| PROP-01..05 | Ten controls, delta apply per engine (mpv names, VLC fixed keys, JS), restore/refresh/delete, culture fallback, one-time copy skips deltas | unit + fixtures | `cargo test -p sucrose properties` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** the quick command of the touched crate.
- **Per wave merge:** `cargo xtask ci`.
- **Phase gate:** full suite green, then the end-of-phase maintainer checkpoints per slice (list in the Delivery Slices section) before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] `crates/sucrose-engine-web/tests/fixtures/pages/` — fixture pages (call recorder, traversal probe, deterministic canvas for sync, video seek page) — FMT-07, SEC-02, WAPI-*
- [ ] `crates/sucrose-core/tests/fixtures/` — sanitised copies of the 7 `SucroseProperties.json` and 1 `SucroseCompatible.json` — FMT-03
- [ ] `crates/sucrose-engine-builtin/tests/fixtures/` — small GIF, APNG, animated WebP and one large synthetic GIF — TYPE-02
- [ ] `crates/sucrose-config/tests/fixtures/` — the maintainer's `Engine.json` already used by migrate tests; add a PerDisplay case — DESK-03
- [ ] A tiny Win32 test application as its own crate (not a flag of `sucrose.exe`) — TYPE-06
- [ ] `xtask fetch-engines` tests for the tar.bz2 path and CEF layout — ENG-03
- [ ] Framework install: none.

## Security Domain

`security_enforcement` is enabled and `security_asvs_level` is 1 in `.planning/config.json` [VERIFIED: file read this session].

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — (local single-user; IPC peers are validated by pid as in Phase 3) |
| V3 Session Management | no | — |
| V4 Access Control | yes | Pages can read only the theme folder and generated content (resolver); IPC messages from renderers/Portal are validated by the service; Application themes may only launch an exe inside their own folder |
| V5 Input Validation | yes | Resolver (SEC-02), argument converter, Blob reassembly caps, `SucroseInfo`/`Properties` parsing lenient-but-bounded, Chromium switch strings never passed through a shell |
| V6 Cryptography | yes | `WinVerifyTrust` for installers; SHA256 pins for engine archives; no custom crypto |
| V9 Communications | yes | TLS validation on by default in both web engines (D-12); installers over HTTPS |
| V10/V12/V14 Files, malicious code, configuration | yes | Installer signature check; bundled engines hash-verified; no DLL search in cwd/relative PATH; documented insecure defaults kept for legacy parity (`no-sandbox`, `disable-web-security`) under D-021 |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal through the virtual origin (`..`, encoded, UNC, junction) | Tampering / Information disclosure | One resolver, canonical containment, vector tests on both engines (SEC-02) |
| Untrusted theme page reaching host capabilities | Elevation of privilege | No host objects or message channel; one-way script strings; pages share one origin by D-14 so themes can read each other's storage (accepted, D-14 note) |
| Another theme's page reading local files via `file://` or the debug port | Information disclosure | No `file://` for Web themes (legacy WebView used it); debug port only when `DeveloperPort > 0`; `remote-allow-origins=*` is a legacy switch and widens that port to any web origin, so show the port as developer-only in the UI |
| Substituted prerequisite installer | Tampering / Spoofing | Authenticode chain to Microsoft before execution, download to a private temp, delete after |
| Application theme runs arbitrary code | Elevation of privilege | Exe must live inside the theme folder (legacy rule), argument vector not a shell string, job object, user confirmation is a product decision outside this phase (record the risk in FEATURES) |
| DLL planting for libvlc/CEF | Elevation of privilege | Search next to the exe, then absolute PATH entries only (D-028); plugin path pinned to the engine folder |
| Oversized or malformed IPC/Blob frames | Denial of service | Frame cap, chunk cap, reassembly timeout, drop on mismatch |
| Certificate errors ignored | Spoofing | Default validation (D-12); the switch is explicit and recorded |
| Command-line injection via `Arguments` | Tampering | `CreateProcessW` with a built command line where `Arguments` is one quoted argument (D-30) and quotes inside are escaped |

## Sources

### Primary (HIGH confidence)
- crates.io API (fetched this session): versions, dependency lists, licenses for `cef`, `cef-dll-sys`, `download-cef`, `webview2-com`, `webview2-com-sys`, `windows`, `wmi`, `realfft`, `rustfft`, `image`, `tar`, `bzip2`, `percent-encoding`.
- https://github.com/tauri-apps/cef-rs (dev): `sys/build.rs`, `sys/Cargo.toml`, `cef/Cargo.toml`, `cef/src/lib.rs`, `cef/src/window_info.rs`, `download-cef/src/lib.rs`, README, tree listing.
- https://github.com/wravery/webview2-rs (main): `crates/bindings/build.rs`, tree listing (loader libs for x86, x64, arm64).
- https://cef-builds.spotifycdn.com/index.json: 156.0.31 / 156.0.32 / 158.0.6 entries for windows32, windows64, windowsarm64.
- CEF headers on GitHub (chromiumembedded/cef master): `cef_types.h` (`root_cache_path`, `cache_path`), `cef_browser.h` (`WasHidden`, `ExecuteDevToolsMethod`, `SetAudioMuted`, `ShowDevTools`), `cef_request_context.h` and `cef_scheme.h` (`RegisterSchemeHandlerFactory`), `cef_request_handler.h` (`OnCertificateError`).
- https://download.videolan.org/vlc/3.0.24/ (win32, win64, winarm64 7z + sha256), and the ARM64 archive listing and headers.
- learn.microsoft.com: ICoreWebView2_3 (`TrySuspend`, `Resume`, `SetVirtualHostNameToFolderMapping`), WebView2 process model and distribution (registry `pv`, `/silent /install`), `MF_MEDIA_ENGINE_PLAYBACK_HWND`, `IMFMediaEngine`, `WM_WTSSESSION_CHANGE`, `SYSTEM_POWER_STATUS`, `SetWinEventHook`, `DISPLAYCONFIG_TARGET_DEVICE_NAME`, `IAudioSessionManager2`.
- Repository files read this session: `CLAUDE.md`, `.claude/rules/*`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/legacy/engines.md`, `docs/legacy/services.md` (`1`, `6`), `docs/GPUI.md` (headings, window notes), `docs/FEATURES.md` (architectures), `xtask/engines.toml`, `Cargo.toml`, `deny.toml`, `crates/sucrose-ipc/src/{protocol,frame}.rs`, `crates/sucrose-core/src/{settings,theme}.rs`, `crates/sucrose-config/src/model.rs`, `crates/sucrose-platform/src/{lib,geometry}.rs`, `crates/sucrose-platform/src/os_windows/surface.rs` (API), `apps/sucrose/src/render.rs` (head), `xtask/src/*` (headers), Phase 2 and 3 RESEARCH (headings, validation, legitimacy).

### Secondary (MEDIUM confidence)
- developers.google.com/youtube/terms/required-minimum-functionality (via the fetch tool's summary of the page).
- Phase 2/3 research patterns (reused, not re-derived).

### Tertiary (LOW confidence)
- Everything tagged [ASSUMED] (A1-A24): Windows API details from training knowledge, DevTools protocol methods, CEF/WebView2 integration behavior inside the layered surface.

## Metadata

**Confidence breakdown:**
- Standard stack and pins: HIGH — versions, hashes and sizes fetched this session; dependency compatibility read from the registry.
- Architecture: MEDIUM — the pure cores are certain; CEF link model, windowed browsers inside the layered surface, Direct2D/MF in that surface and the second GPUI window are spikes.
- Pitfalls: MEDIUM-HIGH — several are quoted from vendor headers/docs (singleton lock, WasHidden, TrySuspend, virtual host mapping); others are inferred from the process models.

**Research date:** 2026-11-26
**Valid until:** 2026-12-26 for pins and versions (CEF stable moves roughly monthly; the `cef` crate follows within days), 30 days for the rest.
