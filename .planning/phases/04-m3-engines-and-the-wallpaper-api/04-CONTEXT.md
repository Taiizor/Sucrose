# Phase 4: M3 — Engines and the wallpaper API - Context

**Gathered:** 2026-11-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Every wallpaper type plays on Windows on every engine that supports it, existing web wallpapers work unchanged with the C#-compatible JavaScript API, and display modes, property editing and the performance rules match or improve on the C# app. This phase covers 46 requirements: TYPE-02..06, ENG-02..07, DESK-02..05, PLAY-01..05, PERF-01..04 and PERF-06..09, WAPI-01..10, PROP-01..05, FMT-03, FMT-07 and SEC-02. Concretely:
- GIF (with APNG and WebP), Web, URL, YouTube and Application themes besides Video.
- Engines: WebView2, CEF, libVLC, a built-in GIF renderer and built-in OS video (Media Foundation), with detection, install hints and a per-type engine choice with fallback.
- The full web wallpaper JavaScript API: properties, `SucroseCompatible.json`, system data, audio spectrum, now playing and input forwarding.
- Display modes, including a different wallpaper per monitor, plus stretch modes.
- The properties editor and the engine arguments editor.
- All performance rules, and volume, loop, shuffle and the two sound rules.

Windows only; macOS and Linux follow in Phases 6–7. Store, theme creation, cycling, the card context menu (LIB-05), the full tray menu (TRAY-02), 25 languages (UI-06) and packaging stay in Phase 5.

</domain>

<decisions>
## Implementation Decisions

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
- **D-09:** **Span keeps the legacy union rectangle**: one renderer covers the union of all monitors. Stretch (DESK-05, mapped per engine as in `engines.md` §3) and the working-area / display-bounds choice (DESK-04) apply to that rectangle.

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
  - The YouTube `origin` playerVar then matches the real origin on both engines; research checks whether the `youtube.com` Referer injection (`engines.md` §8) is still needed on both.
  - Each engine gets its own Rust-owned user data folder, not the C# `Cache/WebView2` / `Cache/CefSharp`.
  - **Reversibility:** one-way — after release, the localStorage, IndexedDB and cookies of every web wallpaper are bound to this origin; changing it loses that data. **Confirmed by the maintainer on 2026-11-25: one shared origin `https://sucrose.wallpaper.engine/` for all themes (per-theme subdomains were offered and declined; `--disable-web-security` is on by default, so origin isolation would add no security).**
- **D-15:** **Prerequisite detection and install (ENG-06):** the WebView2 Runtime must be newer than 131.0.2903.70 and the VC++ Redist at least 14.40.33816.0 for the right architecture (legacy thresholds and URLs, `engines.md` §9). The bundled engine binaries (libmpv, libVLC, CEF) are checked as present and loadable, with a repair hint.
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
  - Push cadences follow the legacy app (`services.md` §1.2–1.3).
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
  - **Planning rounds (maintainer decision on 2026-11-26).** The first planner run returned PHASE SPLIT RECOMMENDED (about 55–65 plans, and slices 2–8 depend on what slice 1 builds and on the D-32 spike). Decimal phases are ruled out (ROADMAP/STATE), so Phase 4 is planned in five rounds that keep the D-31 order. Each round is a separate `/gsd-plan-phase 4` run:

    | Round | D-31 slices |
    |---|---|
    | A | 1 |
    | B | 2 and 3 |
    | C | 4 and 5 |
    | D | 6 |
    | E | 7 and 8 |

    - Round A is planned now. Each later round is planned after the previous round has executed.
    - Plan numbers and `T-04-NN` threat IDs continue across rounds.
    - A requirement that spans rounds is marked Complete only in the round that lands its last part.
    - Requirements and decisions that belong to later rounds are not gaps of an earlier round. The requirement and decision coverage gates of each round report them as deferred to their round.
    - Phase verification (`human_verify_mode` end-of-phase) runs only after Round E has executed.
    - **Round B scope (maintainer decision on 2026-11-26, at the Round B /gsd-plan-phase run).** Round B adds plans from 04-22 on and leaves the Round A plans unchanged. Its final maintainer checkpoint also reruns the 04-20 slice-1 steps that failed or were skipped (1, 3, 5, 6 and 8) for the 8 Web themes and the fixture pages on both CEF and WebView2. When they pass, it marks the Round A requirements that 04-20 left Pending (TYPE-03, ENG-07, WAPI-01, WAPI-02, WAPI-04, FMT-07). The open defects from 04-20 (Web themes that fail inside the page, for example the CORS-like error of "Test Sucrose API" and "SucrosePropertyListener is not defined" on Critter Crossing, and the CEF GPU process crash) are fixed in separate /gsd-debug sessions, not in Round B plans. The checkpoint plan names those fixes as its precondition; if they have not landed, its Complete rule applies as written.

### Research resolutions (confirmed by the maintainer on 2026-11-26)
The research (`04-RESEARCH.md`, Open Questions for the Maintainer) raised five questions. The maintainer chose the recommended option for each:
- **D-32:** **CEF keeps the `cef` crate (D-013) and delay-loads `libcef.dll`.**
  - `.claude/rules/cargo.md` gains an explicit exception for `cef` and `webview2-com`.
  - Every build sets `CEF_PATH` to the `cargo xtask fetch-engines` result, so the crate never downloads CEF itself (D-028 stays true).
  - `libcef.dll` is delay-loaded, so the service and the mpv renderers never load it.
  - If the slice-1 spike shows that delay-load does not work, the fallback is a second executable, `sucrose-cef.exe`, as the CEF renderer and subprocess; record that outcome in `docs/DECISIONS.md`.
  - If the `cef` crate has no working Windows x86 build, CEF is reported as missing on x86 (D-027, D-002).
  - **Reversibility:** costly — the build setup and the renderer process model depend on it.
- **D-33:** **Concurrent web wallpapers: the first running web instance per engine uses the persistent profile**, guarded by a lock file. Every other concurrent instance on that engine gets its own ephemeral profile.
  - localStorage, IndexedDB and cookies therefore persist only for the first instance; volumes stay independent (D-06).
  - Record it in `docs/FEATURES.md`.
  - **Reversibility:** costly — which instance keeps its web data is user-visible.
  - **Amended 2026-11-27 (04-37 D9, maintainer):** with several monitors the persistent profile goes to the primary monitor's instance, not to the first one that locks; see `docs/DECISIONS.md` D-040.
- **D-34:** **`sucrose-sysinfo` becomes an unsafe-allowed platform-provider crate.**
  - A new `docs/DECISIONS.md` entry adds it to the D-011 list, with per-OS `cfg` inside it (PDH, WASAPI, GSMTC, ICMP) and a `// SAFETY:` comment on each block.
  - `sucrose-core` stays free of OS code.
  - Update the places that list the unsafe-allowed crates (CLAUDE.md non-negotiable 5, `.claude/rules/platform-ffi.md`, `docs/ARCHITECTURE.md`) in the same change.
- **D-35:** **Persistent web data lives outside `Cache`**: `<data_dir>\Web\WebView2` and `<data_dir>\Web\Cef` (the maintainer's: `%APPDATA%\Sucrose\Web\...`), so the Phase 5 cache cleaning never deletes a wallpaper's localStorage.
  - The D-33 ephemeral profiles are not kept across runs; where they live is the planner's choice.
  - **Reversibility:** one-way — every later version reads this folder; moving it needs a migration. **Confirmed by the maintainer on 2026-11-26.**
- **D-36:** **Remote and Virtual rules (D-23): a spike in slice 6 tries `Win32_ProcessStartTrace` on the maintainer's non-elevated session.**
  - If the subscription is denied, the fallback is a bounded ToolHelp snapshot every 2 s, taken only while the Remote or Virtual rule is not `Resume` (the legacy cadence at native cost).
  - Record the outcome in `docs/FEATURES.md`.
  - This relaxes D-23's "not the legacy 2 s full process scan" only for that fallback.

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Legacy behavior (read-only reference for parity)
- `docs/legacy/engines.md`
  - §1: wallpaper type × engine matrix, generated pages, `Host` per engine.
  - §3: display modes, stretch per engine, DPI.
  - §4: pause, volume, loop, shuffle, sound rules.
  - §5: input forwarding.
  - §6: JavaScript API, §6a properties, §6b compatible API, §6c generated-page functions.
  - §7: property system and built-in sets.
  - §8: YouTube, URL, Application.
  - §9: CEF server, engine settings, Chromium switches, prerequisite checks.
  - §10: lifecycle.
  - Porting notes.
- `docs/legacy/services.md`
  - §1.1: performance conditions, order, debounce, defaults.
  - §1.2: collected data, payload keys, audio visualizer algorithm.
  - §1.3: how data reaches engines.
  - §6: `Sucrose.Property` editor behavior.
- `docs/legacy/formats.md`
  - §1.3: Engine.json, Backgroundog.json and Warehouse.json keys.
  - §3: `SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json`.
  - §5: legacy IPC payloads.
  - §6: enums.
- `docs/legacy/portal.md`: Settings → Performance and Wallpaper pages, DisplayPreferences dialog, LibraryCard Customize enable rule.

### Decisions and parity
- `docs/DECISIONS.md`
  - D-002 (capabilities, never faked), D-006 (built-in engines), D-007 (compatibility).
  - D-008 (process model, optional modules), D-009 (one IPC), D-011 (unsafe isolation), D-012 (threading).
  - D-013 (bindgen + libloading, `webview2-com`, `cef` crate).
  - D-017 (secure by default), D-019 (verified installers), D-021 (user-editable browser switches).
  - D-024 (defaults stay), D-027 (architectures), D-028 (bundled engines, `fetch-engines`).
  - D-030 (master clock), D-032 (settings file, service as single writer), D-033 (Portal IPC).
- `docs/FEATURES.md`: parity rows for every type, engine, display mode, JS API, property and performance item; behavior changes from this phase are recorded here.
- `docs/ARCHITECTURE.md` (Engines, Processes, Workspace layout): the `Engine` trait sketch and crate rules.
- `docs/PORTING.md`: porting procedure and NuGet → crate mapping.
- `docs/GPUI.md`: UI lessons for the Portal, the editor window and Display Preferences.
- `docs/MEASUREMENTS.md`: where per-engine RAM/CPU numbers go.

### Planning
- `.planning/ROADMAP.md` Phase 4: goal, five success criteria, notes (legacy bugs fixed, not ported).
- `.planning/REQUIREMENTS.md`: TYPE-02..06, ENG-02..07, DESK-02..05, PLAY-01..05, PERF-01..04 and PERF-06..09, WAPI-01..10, PROP-01..05, FMT-03, FMT-07, SEC-02.
- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-CONTEXT.md`: renderer per monitor, sync, fullscreen rule and scope, restart budget.
- `.planning/phases/03-m2-service-ipc-and-portal-connected/03-CONTEXT.md`: settings file, service as single writer, Portal scope rules (D-15..D-18), Use transition (D-26).

### Rules
- `CLAUDE.md`, `.claude/rules/platform-ffi.md`, `.claude/rules/cargo.md`, `.claude/rules/rust.md`, `.claude/rules/docs.md`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/sucrose-engine-mpv`: `loader.rs` (search next to the exe, then absolute `PATH` entries, D-028), `sync.rs` (follower correction toward the master clock), `player.rs`, `config.rs`. The model for the libVLC loader and for clock-following in other engines.
- `apps/sucrose/src`
  - `service.rs`: renderer orchestration, master election (D-030).
  - `supervisor.rs`: restart with backoff and bound (Phase 2 D-22).
  - `topology.rs`: monitors and hot-plug.
  - `fullscreen.rs`: the 5/3 debounced rule with scope; the starting point of the performance governor.
  - `render.rs`: the renderer process.
  - `modules.rs`: optional modules that do not start when off.
  - `settings_host.rs`: the single settings writer.
  - `portal_link.rs`: Portal protocol with `RuleState` / `RuntimeState`.
- `crates/sucrose-ipc` (`protocol.rs`: `ToService`, `ToRenderer`, `Assignment`, `PlaybackSettings`, `ClockTick`; `portal.rs`): extend append-only.
- `crates/sucrose-config/src/model.rs`: typed `Engine` and `Backgroundog` sections already migrated (engine per type with allowed lists, `cef_arguments` / `web_arguments`, `DisplayScreenType`, `ScreenDevice`, `FullScreenScope`, `PauseKind`, `InputType`, volume rules, developer keys, thresholds) and `Warehouse` time flags.
- `crates/sucrose-core/src/theme.rs` (`WallpaperType` with legacy ordinals, `SucroseInfo.json` parsing) and `library.rs`.
- `crates/sucrose-platform`: `layout.rs` / `geometry.rs` (desktop attach, monitor rectangles), `os_windows`.
- `xtask/src`: `fetch_engines.rs` + `xtask/engines.toml` (pinned, SHA256-verified engines), `bindgen_mpv.rs` (model for libVLC bindings), `measure.rs` (per-process RAM/CPU/GPU rows), `synth_library.rs`.
- `apps/sucrose-portal/src`
  - `settings/row.rs` with `general.rs` / `personal.rs`: the settings-page pattern for the Wallpaper and Performance pages.
  - `dialogs/`: the dialog pattern for Display Preferences.
  - `toast.rs`: fallback and hint toasts.
  - `library/card.rs`: Use-disabled reasons and hover, where the Customize icon goes.
  - `i18n.rs`: rust-i18n keys named after the legacy keys.

### Established Patterns
- IPC messages are appended, never inserted; the protocol has version constants.
- The service is the only settings writer. The Portal gets snapshots and sends changes (Phase 3 D-11).
- Legacy enums are stored by name; a missing or invalid key falls back to the legacy default.
- Format compatibility is proven with fixtures from real files under each crate's `tests/fixtures/`.
- One renderer process per monitor; crash isolation with a restart budget; reasons go to the log and the tray tooltip.
- `unsafe` only in `sucrose-platform` and engine/FFI crates, each block with `// SAFETY:`.
- Maintainer checkpoints run release builds on their machine with `SUCROSE_DATA_DIR` scratch folders; GUI checks never synthesize input.

### Integration Points
- The Use flow: the LoadSource path without a restart for the same engine; a restart when the engine changes (Phase 3 D-26).
- The card Use-disabled rule for non-Video types (Phase 3 D-17), lifted per slice.
- The header action slot for Display Preferences (Phase 3 D-15).
- The settings pages for the Wallpaper and Performance keys.
- The tray tooltip and icon state for rule status.

</code_context>

<specifics>
## Specific Ideas

- The maintainer's legacy `Engine.json` (read-only):
  - engine choices: `Web` / `Url` / `YouTube` = `CefSharpLive`, `Gif` / `Video` = `MpvPlayerLive`;
  - placement: `DisplayScreenType` `PerDisplay`, `ScreenDevice` `\\.\DISPLAY1`, `ScreenType` `DisplayBound`, `StretchType` `UniformToFill`;
  - developer: `DeveloperMode` true, `DeveloperPort` 0;
  - sound and input: `VolumeSilentSensitivity` 10, `InputDesktop` true, `WallpaperVolume` 100;
  - other: `StayAwake` true, `HardwareAcceleration` true;
  - `CefArguments` / `WebArguments` equal to the legacy defaults.
- The maintainer's `Backgroundog.json`:
  - `PausePerformanceType` `Light`;
  - Lock / Sleep / Console / Session = `Close`, ScreenSaver = `Pause`, every other rule = `Resume`;
  - `GraphicAdapter` "NVIDIA GeForce RTX 3050 Laptop GPU", `NetworkAdapter` "Intel[R] Wi-Fi 6 AX201 160MHz".
- With the real data, the first Phase 4 run therefore:
  - plays the selected theme on DISPLAY1 only;
  - opens no DevTools and no debugging port;
  - starts the performance governor, because Lock, Sleep, Console, Session and ScreenSaver are not `Resume`, and reads only their signals; the CPU, GPU, memory and network rules are all `Resume`, so no counters are sampled for them.
- Library `H:\Sucrose`: 80 Video, 8 Web, 2 GIF themes. 7 have `SucroseProperties.json`, only 1 has `SucroseCompatible.json`. There are no URL, YouTube or Application themes in the library. Showcase: 17 Web, 3 YouTube.
- 34 C# property copies in `%APPDATA%\Sucrose\Cache\Properties` (for example `<theme>MpvPlayer`, `<theme>VlcPlayer`) to carry over once (D-20).
- The maintainer's words on Duplicate: "like the video engines, a separate instance on every screen; if YouTube, video or GIF run as wallpapers, syncing them is not hard; for YouTube, video and GIF the time can be set forward or back from JS", and Web/URL "too, if we can".
- The maintainer's words on sound: every wallpaper may play; there is already a global volume; add a volume setting to the customization options of video and YouTube wallpapers, and give Web/URL wallpapers a customization option where sound can be turned down or off. Web themes with their own `SucroseProperties.json` keep it.

</specifics>

<deferred>
## Deferred Ideas

- Decoding once and sharing frames across monitors instead of one decode per renderer (raised in Phase 2): still deferred. The maintainer chose one instance per monitor for Duplicate on every engine (D-04).
- Considered and not chosen, kept for the record:
  - mirroring one web instance across monitors;
  - per-monitor volume levels;
  - per-theme web origins;
  - "strongest action wins" for overlapping performance conditions;
  - a separate Engines settings page.

</deferred>

---

*Phase: 04-m3-engines-and-the-wallpaper-api*
*Context gathered: 2026-11-25*
