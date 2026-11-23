# Phase 3: M2 - Service, IPC and Portal connected - Research

**Researched:** 2026-11-24
**Domain:** Rust desktop app: local-socket IPC service plus a GPUI UI, versioned settings with legacy migration, OS autostart, library scan and bounded image caches (Windows first, macOS and Linux compiling)
**Confidence:** HIGH for the in-repo contracts and the real-data findings, MEDIUM for GPUI rendering behavior and Windows shell behavior that this session could only read, not run

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Settings format and migration
- **D-01:** The new settings are **one versioned file inside the legacy `Setting/` folder** (for example `Setting/Sucrose.json`; the name must not collide with the 14 legacy file names) with a top-level version field. The data folder layout stays as it is (FMT-06): the C# `ControlFile` only touches its own 14 files, and its "Create Log File" zip picks the new file up with the folder. Sucrose never writes, renames or deletes a legacy `Setting/*.json` file (criterion 2). — **Reversibility:** one-way — once released, every later Rust version, the Phase 5 installer and the updater read the file at this path; moving it needs another migration.
- **D-02:** The format is **JSON** (`serde_json`, already a workspace dependency): human-readable, versioned, written atomically (temporary file in the same folder, flushed, then renamed over the old one) so that killing Sucrose during a write never leaves a corrupt file. — **Reversibility:** costly — the reader, the writer, the fixtures and any later migration step depend on it.
- **D-03:** Migration runs **once, when the new file does not exist**: read the legacy files (always from the real `%APPDATA%\Sucrose\Setting`, ignoring `SUCROSE_DATA_DIR`, Phase 2 D-08), write the new file, and never read the legacy files again. Changes made in the C# app afterwards are not picked up. Without a C# install the legacy defaults apply (D-024). A missing or invalid key falls back to its legacy default with the Phase 2 rules (clamp out-of-range numbers, wrong type or unknown enum name gives the default, enums by name, retry a locked or empty file 3 × 50 ms).
- **D-04:** **Every migrated key is a typed field now**, not only the keys Phase 3 uses: each has its legacy default, range and enum names from `docs/legacy/formats.md` §1.3 and §6. Fixture tests from real files prove each key migrates. Later phases use the ready fields instead of converting raw values.
- **D-05:** Documented exclusions, **not migrated**: `Donate.json` (D-023); `Objectionable.json` `PersonalAccessToken` (GitHub integrations removed by D-020, and a plaintext secret); `Portal.json` `StoreServerType` (Soferity only, D-020); `Aurora.json` `AppProcessName` (runtime state, ARCH-04); `Kernel.json` `Information` and the `System.json` interface lists (write-only system information, regenerated at runtime). In the same change, update success criterion 2 in `.planning/ROADMAP.md`, FMT-01 in `.planning/REQUIREMENTS.md` and the matching `docs/FEATURES.md` row to "every value except `Donate.json` and the documented exclusions" and list them.
- **D-06:** `Warehouse.json` values (`Showcase` list, `HintTrayIcon`, the CefSharp/WebView time and continue flags) migrate into the new file like every other key. The showcase copy (LIB-01) is tracked in the new settings from then on; reword LIB-01 ("tracked in `Warehouse.json`") accordingly. On the maintainer's machine all 20 showcase names are already listed, so nothing is copied again.

### Side by side with the C# app
- **D-07:** The library is the migrated `Library.json` `Location`, used as is (the maintainer's is `H:\Sucrose`, shared with the C# app). LIB-09 moves it **like the legacy app** (the destination must be empty, themes are copied and the old folder removed when `Move` is on, the wallpaper restarts). When a C# install is detected (its `Setting` folder exists), the confirmation says that the C# app keeps pointing at the old location and will see an empty library. Sucrose never writes the C# `Library.json`.
- **D-08:** "Start with the OS" (TRAY-03, normal startup): migration copies `RunStartup` but **creates no startup entry by itself**. Sucrose writes **its own entry** (Windows: an HKCU Run value with a name different from the legacy one; macOS LaunchAgent; Linux XDG autostart) only when the user turns it on in the Rust Portal. It **never modifies or removes** a C# entry (the legacy Run value, likely named `Sucrose` per `formats.md` `✔Startup✖Sucrose✖…`, or the scheduled task `\Sucrose\Autorun for Sucrose`). When a C# entry exists, the Portal warns that both apps will start at sign-in. Values 2 (Priority) and 3 (Scheduler) are kept in the settings, shown as available later (TRAY-04, Phase 5) and create no entry; the maintainer's value is 3. — **Reversibility:** costly — the entry name becomes something the Phase 5 updater and uninstaller must find and remove.
- **D-09:** Corrupt themes (LIB-02, only when `DeleteCorrupt` is on; legacy default and the maintainer's value are off) are **moved to the OS recycle bin / trash instead of deleted permanently**, so a theme the Rust parser rejects but Newtonsoft accepted can be restored. The theme's `Cache\Properties` files are deleted as in the legacy app; locked or inaccessible folders are skipped as in the legacy app. Validity rules are exactly the legacy ones (`SucroseInfo.json` parses, Preview and Thumbnail exist, Source is a URL or an existing file), and fixture tests from real themes prove the parser is as lenient as Newtonsoft where real themes need it. Record the recycle bin as an improvement in `docs/FEATURES.md`.
- **D-10:** Showcase themes come from **`data_dir()/Showcase`**, as in the legacy app (the C# installer put them there; the Rust installer will in Phase 5). No packaging work in this phase; the copy is proven by fixture tests. Under `SUCROSE_DATA_DIR` the scratch `Showcase` folder is empty unless a test puts themes there.

### Service and Portal lifecycle
- **D-11:** **The service is the only reader and writer of the settings file.** The Portal receives a snapshot over IPC when it connects and sends changes; the service validates them, writes atomically (D-02) and broadcasts the change to connected clients. No process polls the file (ARCH-04). — **Reversibility:** costly — every settings page and the Portal side of the IPC protocol are built on it.
- **D-12:** Startup follows the **legacy Launcher rule**: `sucrose` starts the wallpaper when a selected theme exists and is not running (and also opens the Portal when the tray is hidden); with nothing selected it opens the Portal. A second `sucrose` asks the running service to open or focus the Portal and exits (extends Phase 2 D-23). A second Portal brings the existing window to the front. A Portal started on its own starts the service when none runs and connects to it; a service started that way does not open another Portal. Both single-instance checks go through the IPC endpoint (D-009), not named mutexes.
- **D-13:** **Service lifetime:** with the tray visible (`AppVisible` true) the service stays until Exit, like the legacy Launcher. With the tray hidden, no wallpaper running and no Portal connected, it exits; the next Portal start or `sucrose` launch starts it again.
- **D-14:** **Tray menu in Phase 3:** "Open Sucrose" (also on left double-click; disabled while the Portal is open, as in the legacy app) and the exit item labelled by `AppExit`. `AppExit` true → "Close Applications": stop the renderers, tell the Portal over IPC to close, exit the service (TRAY-05). `AppExit` false → "Exit": only the tray module stops, the wallpaper keeps playing and D-13 applies. It never touches C# processes (Phase 2 D-14). The rest of the menu is TRAY-02 in Phase 5.

### Portal scope in Phase 3
- **D-15:** The header shows **only actions that work in Phase 3**: search (MaxLength 20, 500 ms debounce, lowercased; no telemetry POST until telemetry exists), the light/dark toggle, the Premium link (`https://sucrose.dev/premium`) and ⋮ with Help and About. Create Wallpaper, Wallpaper Cycling and Display Preferences appear in the phase that brings their feature; no disabled placeholders.
- **D-16:** Settings use the **real settings mode** (gear → Back plus pages, as in the legacy app). Phase 3 builds the General and Personal pages showing **only the keys this phase needs, in their legacy places**: startup (D-08), notification area icon and `AppExit`, theme mode (D-21), window backdrop with background image, stretch and opacity, library location and move; corrupt themes, library preview and hide details, adaptive margin and count, library sort mode and kind, library pagination. Phase 5 (UI-04) fills the same pages and adds the others.
- **D-17:** Cards of **non-Video themes** are shown normally, but Use is disabled with a reason (for example "This wallpaper type is not supported yet"), without the hand cursor; the selection and the running wallpaper stay unchanged. Note this as temporary until Phase 4 in `docs/FEATURES.md`.
- **D-18:** Theme cards in Phase 3 have **no "…" button and no context menu**; left click is Use, as in the legacy app (disabled while that theme is selected and running). A theme for a newer Sucrose shows the red badge with the legacy tooltip "Update Sucrose to use this theme" and does nothing until the updater exists (Phase 5). The context menu arrives whole with LIB-05.
- **D-19:** The status bar memory value is the **total of all Sucrose processes** (service, every renderer, Portal), with a per-process breakdown in its tooltip. The service reports it over IPC about once a second, and only while a Portal is connected (no polling otherwise, D-008). Record as a behavior change (legacy: Portal process only).
- **D-20:** Every user-visible string goes through **rust-i18n from the start**, with key names matching the legacy XAML keys (for example `Portal.Quoting0` … `Portal.Quoting85`). Phase 3 ships English only; the migrated `Culture` (the maintainer's is `TR`) is stored and applied from Phase 5 (UI-06).

### Theme mode and backdrop
- **D-21:** **Two independent settings:** theme mode {Light, Dark, System} and backdrop {None, Mica, Acrylic, Tabbed}. Migration: `ThemeType` Dark/Light → Light/Dark; `BackdropType` `Auto` → theme mode System plus backdrop Mica; the other `BackdropType` values map one to one (Tabbed is GPUI `MicaAltBackdrop`). A fresh install defaults to System + None (closest to the legacy "OS theme on first read"). The header toggle sets Light or Dark explicitly. Record the split in `docs/FEATURES.md`. The maintainer migrates to Dark + None.
- **D-22:** Backdrops an OS cannot show (macOS, Linux, Windows 10) stay **visible but disabled with the reason in a tooltip** (for example "Requires Windows 11"), driven by platform capabilities. A stored value the OS cannot show is applied as None without changing the setting. The background image with stretch and opacity works on every OS (UI-01).

### Fullscreen rule and Use
- **D-23:** While the fullscreen rule has paused or closed the wallpaper, Use is **disabled with the legacy suffix " (Paused)" or " (Closed)"** and enabled again when the rule recovers. The service publishes the rule state over IPC (runtime state, never stored in settings). The selection does not change.

### Hover preview and memory
- **D-24:** With `LibraryPreview` on (legacy default off), the Preview GIF is **decoded at card size on a background thread, only the hovered card animates, and its frames are released and dropped from GPUI's image atlas when the hover ends**; the static thumbnail stays. `LibraryPreviewHide` hides the title/description strip while previewing.
- **D-25:** UI-07 is measured in a release build and recorded in `docs/MEASUREMENTS.md` with two scenarios: (1) the maintainer's real library (108 themes, 30 per page, 4 pages): first page, every page forward and back for 3 rounds, hover previews; Portal private bytes peak and final value. (2) A synthetic library of about 500 themes under `SUCROSE_DATA_DIR`, with a prepared new settings file whose `Location` points at it (no test-only app flag), same scenario. Pass: the final value stays within the first-page value plus the cache cap and does not grow across rounds. Reuse `cargo xtask measure`.

### Use transition
- **D-26:** Switching from one Video theme to another **loads the new source into the running renderers** through a new IPC message (appended, never inserted): no restart, no black or static frame in between, and the master clock realigns per D-030. When the new theme needs a different engine (Phase 4 onward), the renderers restart. A failure falls back to the Phase 2 D-13 behavior.

### Claude's Discretion
- The exact settings file name, its internal structure (sections, field names), the version field and the atomic-write details (flush, rename or `ReplaceFile` on Windows, keeping a last good copy).
- The Portal ↔ service messages (append-only, `PROTOCOL_VERSION`), whether the Portal uses the renderers' endpoint or its own, and how a Portal connection is authenticated (same user, as the owner-SID check of plan 02-05).
- How the Portal finds and starts the service (both executables in the same folder) and how "open the Portal" is carried out by the service.
- The Run value name and the detection of legacy startup entries (Run value, scheduled task).
- Where the recycle-bin move lives (`sucrose-platform` or a maintained, GPL-compatible crate such as `trash`, per `.claude/rules/cargo.md`).
- Library scan on a background thread; search index and ordering as in `portal.md` §2; whether cards appear progressively (legacy: one per 50 ms); empty and search-empty pages with the legacy texts.
- Thumbnail decode size (legacy: 360 px), cache cap and eviction, `drop_image` usage; animated GIF for the background image within the same memory rules.
- Status bar texts adapted to Rust (release label, framework, architecture, version, quote with copy).
- Help and About contents, the tray hint dialog on first close (`HintTrayIcon`, UI-05), plain links (wiki, report an issue).
- Command-line page arguments (UI-03): `GeneralSetting` opens General; `SystemSetting` (no System page yet) opens General as well until Phase 5.
- System theme mode follows OS theme changes live through a platform event.
- The fullscreen scope setting from Phase 2 D-05 gets its field in the new file (default: all monitors, no UI until PERF-01 in Phase 4).
- Which optional modules exist in Phase 3 (tray at least) and how "does not start when off" is shown in a test (criterion 4).

### Deferred Ideas (OUT OF SCOPE)
- Re-importing the legacy settings on demand (considered for D-03, not chosen).
- Taking over or converting the C# startup entry (Run value or `Autorun for Sucrose` task) when the Rust installer replaces the C# app: decide with the Phase 5 installer migration (D-019).
- Search telemetry POST: with telemetry in Phase 5.
- Card context menu (Review, Customize, Find, Share, Delete, Edit) and the incompatible-theme Update action: LIB-05 and the updater in Phase 5.
- Header actions Create Wallpaper, Wallpaper Cycling, Display Preferences: in their phases.
- Applying the migrated `Culture`: UI-06 in Phase 5.

</deferred>
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARCH-01 | The service runs one renderer process per running wallpaper (per monitor or span); a crashed renderer is restarted with a bounded retry, never in a loop (D-008) | Supervisor, RestartBudget (3 crashes / 60 s) and job-object cleanup exist from Phase 2. Phase 3 only adds a runtime-changeable job (Use), LoadSource (D-26) and keeps the budgets across a Use. See Pattern 4. |
| ARCH-02 | All Sucrose processes talk over one local-socket IPC, and the same endpoint keeps the service and the Portal single-instance (D-009) | One endpoint, first-message routing (renderer Hello vs PortalHello vs Launch), Portal registry in the service gives both single-instance rules. See Patterns 2 and 3. |
| ARCH-03 | An optional service module the user turns off does not start at all: no thread, no polling, no allocations (D-008) | Module gate pattern (closure-counted start, like start_tray); modules in Phase 3: tray, fullscreen poll, memory ticker, Portal broadcaster. See Pattern 9. |
| ARCH-04 | Settings use a new versioned format with atomic writes; runtime state (pause/close flags, ports) is never stored in settings, and processes share state over IPC instead of polled files (D-007) | Versioned Setting/Sucrose.json, atomic write, .bak, service is the only writer; runtime keys excluded (Open Question 2). See Pattern 1. |
| ARCH-05 | "Use" in the Portal applies the wallpaper through the service, and settings changed in the Portal persist across restarts | PortalRequest::Use and SetSetting messages, service-side validation, broadcast. See Patterns 2 and 4. |
| FMT-01 | Legacy `Setting/*.json` files (14 files, `Properties` wrapper, enums by name) migrate to the new settings format on first run, `Donate.json` excluded (D-023), proven with fixtures | Typed model for every key of formats.md section 1.3 minus the exclusions, migration with the Phase 2 getters, fixtures from the real files (findings F1, F2). |
| FMT-06 | The data folder layout (`Setting`, `Log`, `Cache`, `Library`, `Showcase`) and 25-character theme folder names stay the same (`formats.md` §2) | Setting/Sucrose.json beside the 14 legacy files; the C# Reset/Import/Export globs touch it (Pitfall 3, .bak). |
| TRAY-03 | Sucrose can start with the OS (normal startup) (Win ✅ · mac ✅ LaunchAgent · Linux ✅ XDG autostart) | Own Run value on Windows (windows crate registry, precedent gpu.rs), LaunchAgent plist, XDG .desktop, StartupApproved reset (Pattern 8, Pitfall 9). |
| TRAY-05 | Exiting closes all Sucrose apps (Win ✅ · mac ✅ · Linux ✅) | Close Applications: stop renderers (start_shutdown exists), Close to the Portal over IPC, then exit; bounded wait, then kill (Pattern 3). |
| LIB-01 | Showcase themes are copied on first run and tracked in `Warehouse.json` | Showcase copy as a service-side library operation tracked in the new settings (D-06, Pattern 5). |
| LIB-02 | The library is scanned with the legacy validity rules and corrupt themes are deleted | Shared pure scan in sucrose-core, exact legacy rules; corrupt folders to the OS bin through a platform wrapper over the trash crate (Pattern 5, finding F1). |
| LIB-03 | User can sort the library by name, creation or modification date (ascending/descending) and search it | Pure sort and search index in sucrose-core (portal.md section 2); debounce lives in the Portal. |
| LIB-04 | The library is paginated with an adaptive layout (margin, max per row) | Layout math as a pure function; gpui-component Pagination exists with an ellipsis menu but no jump field (Pattern 7). |
| LIB-06 | Theme cards show hover previews, optionally hiding details while previewing | Background GIF decode at card size with a frame and byte cap, drop_image on hover end (Pattern 6, Pitfall 6). |
| LIB-08 | Themes made for an incompatible version are handled as in the legacy app | Newer AppVersion keeps the card visible with the red badge; reuse AppVersion comparison from sucrose-core (D-029). |
| LIB-09 | User can change the library location and move the existing themes | Service-side verified move with same-volume rename fast path, nested-destination refusal, wallpaper restart (Pattern 5, Pitfall 5). |
| UI-01 | The Portal has a window backdrop (Mica/Acrylic on Windows) and a background image with stretch and opacity on every OS | WindowBackgroundAppearance via window_background and set_background_appearance; Mica and Tabbed need build 22621 (Pattern 10); background image is a decoded RenderImage layer. |
| UI-02 | User can pick light or dark, or follow the OS theme | ThemeMode plus embedded ThemeConfig JSON, observe_window_appearance for System (Pattern 10). |
| UI-03 | The Portal has header actions, search, a status bar (memory, quotes with copy) and command-line page arguments | Header, search debounce, status bar and memory report; argument parsing for GeneralSetting and SystemSetting (Patterns 3 and 7). |
| UI-05 | Help and About dialogs and the tray hint dialog exist; plain links (wiki, report an issue) stay (D-020) | Dialog and AlertDialog components, cx.open_url, tray hint with on_window_should_close (Pattern 7). |
| UI-07 | Browsing the library and store keeps memory bounded: thumbnails are decoded at card size, the cache is bounded, and images no longer shown are released from GPUI (`docs/GPUI.md`, prototype lesson). Measured as part of MEAS-02 | Bounded decoded-image cache (64 images / 48 MiB), drop_image on eviction, measured with an extended xtask measure (Pattern 6, Validation Architecture). |
</phase_requirements>

## Summary

Phase 2 left a service that already owns the single endpoint, supervises one renderer per monitor with a bounded restart budget, and reads four legacy files directly. Phase 3 turns that into the product shape: the service becomes the only owner of a new versioned settings file (D-11), the Portal becomes a second kind of IPC peer on the same endpoint (first-message routing, no second endpoint), and the Portal itself is a new GPUI app that scans the library read-only, decodes thumbnails at card size into a bounded cache, and asks the service for every mutation (Use, settings, startup, showcase copy, corrupt-theme move, library move). Almost every decision is already locked in CONTEXT.md; this research fills in the how, and, more importantly, corrects four things the locked text got wrong or left open because nobody had looked at the maintainer's real files.

The four material findings (details in "Real-data findings"): (F1) the maintainer's library has 90 valid themes, not 108, so the D-25 scenario is 3 pages, and 16 non-theme folders sit in it that the legacy scan would delete with Corrupt Themes on; (F2) the maintainer's `Portal.json` says `BackdropType = Acrylic`, not None, so D-21 migrates them to Dark + Acrylic; (F3) `Backgroundog.json` holds live runtime state (`TransmissionPort = 63920`, the `*Required` and `*Performance` flags), which D-04's "every key is typed" would carry into the new file against ARCH-04, and it also holds four stale key names that formats.md does not list; (F4) `MAX_FRAME` is 64 KiB, so neither the library list nor the whole settings file can travel as one IPC frame, which fixes the division of labor (Portal scans, service mutates; the snapshot is a Portal view, not the file).

Three pieces of new engineering carry most of the risk and deserve their own plans and checkpoints: the service refactor (a 350-line `run()` with a job fixed at startup must accept Use, settings changes, Portal connections and tray changes at runtime), the load-new-source path for D-26 (`sucrose-engine-mpv` has no load-file API and the "no black frame" claim needs a spike on real hardware), and the bounded image pipeline (real previews are GIFs up to 5 MB, so a naive full decode blows the memory budget the phase is measured against).

**Primary recommendation:** Build bottom-up in four waves: (1) pure crates (`sucrose-core` library scan and enums, `sucrose-config` model, migration and atomic store, `sucrose-ipc` Portal protocol, `sucrose-platform` autostart, trash, capabilities, memory), (2) service integration (settings host, Portal registry and lifecycle, Use and LoadSource, library operations, tray menu, module gates), (3) the Portal app (shell, library, settings pages, dialogs, thumbnail cache), (4) measurement tooling, docs and the human-verify checkpoints. Keep every layout, sort, search, pagination, cache-eviction and migration rule a pure function so it is unit-tested without GPUI, and use the real settings and theme files as sanitized fixtures.

## Project Constraints (from CLAUDE.md)

- Parity: every C# feature is tracked in `docs/FEATURES.md`; record every behavior change there (D-09 bin, D-17, D-19, D-21 in this phase). Compatibility (D-007): settings migrate automatically, proven with fixtures.
- No web runtime in the core (D-004): UI is GPUI via `gpui-kit =0.10.0`. Cross-platform from day one (D-002): OS-specific code only in `sucrose-platform` and engine crates; anything an OS cannot do is a capability, never faked; CI builds Windows, macOS and Linux.
- `unsafe` only in `sucrose-platform` and engine/FFI crates, each block with `// SAFETY:` (D-011). Errors: `thiserror` in crates, `anyhow` in apps; `tracing` only; no `unwrap`/`expect` on I/O, parsing, IPC or FFI; no async runtime in the service (D-012); never block the GPUI thread.
- Low resource use is a feature: modules that are off start nothing (D-008); measure RAM/CPU of user-visible changes in release builds into `docs/MEASUREMENTS.md`.
- Dependencies only through root `[workspace.dependencies]`; check crates.io maintenance and GPL-3.0 compatibility; do not add `vlc-rs`, `libmpv-sys`, `single-instance`, `libmpv2`; prefer one crate per job.
- No WSL-only scripts, no `/mnt/...` paths, no symlinks in committed files; automation goes into `xtask`. Do not edit `src/` (C# reference). No test-only app flags (memory: tests run on the maintainer machine with the C# app installed). Real settings, library and store are read-only for automated work; run apps with `SUCROSE_DATA_DIR` (+ `WSLENV=SUCROSE_DATA_DIR` from WSL); GUI runs use `--no-focus` and `PrintWindow`, never synthesized input; stop started apps before rebuilding.
- Keep line endings: `.planning/*` and most `docs/*` are LF in the working copy, `docs/FEATURES.md` is CRLF (checked with `git ls-files --eol`); GSD tools rewrite planning files as LF (memory), re-check after verify steps.
- Decisions go to `docs/DECISIONS.md` (append-only; the next free number is D-032), features to `docs/FEATURES.md`, numbers to `docs/MEASUREMENTS.md`, structure changes to `docs/ARCHITECTURE.md`, GPUI lessons to `docs/GPUI.md`, in the same change as the code.
- Start edits through GSD commands; commit/push only as the workflow allows (workflows may commit, pushing needs the maintainer).

## Real-data findings (read-only scan of the maintainer's machine, 2026-11-24)

These come from reading `H:\Sucrose` and `%APPDATA%\Sucrose\Setting` through WSL with no writes. The planner must treat them as corrections to CONTEXT.md and the fixtures as the ground truth.

- **F1. Library content.** [VERIFIED: scan of /mnt/h/Sucrose] The folder has 108 entries: 90 directories with a valid `SucroseInfo.json` whose Preview, Thumbnail and Source files all exist (80 Video, 8 Web, 2 Gif; every Source is a file, no URL themes), 16 directories without `SucroseInfo.json` (15 unpacked Wallpaper-Engine style projects with `project.json`, some with a `.git` or `.idea` folder or a Unity player, and one "Dynamic" folder of image sets), and 2 loose files (`Url.txt`, `ten-print-wallpaper-main.zip`). The legacy scan lists directories only and shows 90 themes. D-25 scenario 1 therefore has 90 themes, 3 pages at 30 per page (not 108 and 4). With `DeleteCorrupt` on, the legacy app would delete those 16 folders, which is exactly what D-09's recycle bin protects against. All 90 thumbnails are JPEG (median 187 KB, max 640 KB) and all 90 previews are GIF (median 1.4 MB, p90 4.2 MB, max 4.9 MB). All 90 `Type` values are JSON numbers, all `AppVersion` and `Version` values are strings, no theme has a key outside the 14 known ones.
- **F2. Settings content.** [VERIFIED: read of Setting/*.json, key names and non-path values only] `Portal.json`: `BackdropType: "Acrylic"`, `BackgroundOpacity: 55`, `LibraryPreviewHide: true`, `LibraryPreview: false`, `AdaptiveLayout: 6`, `StorePreview: true`, `StoreDuration: 1`. D-21 says the maintainer migrates to "Dark + None"; the mapping rule in D-21 (Acrylic maps one to one) gives Dark + Acrylic. `General.json`: `Culture: "TR"`, `RunStartup: 3`, `ThemeType: "Dark"`. `Cycling.json`: `Active: true`, `TransitionTime: 99999`, `PassingTime: 70180` (a counter the C# app writes at runtime). `Update.json` `Time` is `"2026-11-23T22:02:12.9475265+03:00"` (7 fractional digits and an offset: store it as the raw string, do not parse). `Warehouse.json` has only `Showcase` (20 names) and `HintTrayIcon: false`. The 14 files all exist (Aurora, Backgroundog, Cycling, Donate, Engine, General, Hook, Kernel, Library, Objectionable, Portal, System, Update, Warehouse).
- **F3. Runtime state and stale keys in Backgroundog.json.** [VERIFIED: same read] The file contains `TransmissionPort: 63920`, `PipeRequired`, `SignalRequired`, `TransmissionRequired`, `AudioRequired`, `PausePerformance`, `ClosePerformance` (formats.md marks all of these as runtime state written by engines or Backgroundog) and `CommunicationType: "Pipe"` (the legacy IPC choice D-009 replaced). It also contains four keys that formats.md section 1.3 does not list and the current Rust reader ignores: `GpuUsage`, `CpuUsage`, `CpuPerformance: "Pause"`, `GpuPerformance` (older names of `GraphicUsage`, `ProcessorUsage`, `ProcessorPerformance`, `GraphicPerformance`). D-05's exclusion list names only `Aurora.json AppProcessName` as runtime state; ARCH-04 (a requirement) says runtime state is never stored in settings. See Open Question 2.
- **F4. Frame limit.** [VERIFIED: crates/sucrose-ipc/src/frame.rs:13] `pub const MAX_FRAME: u32 = 64 * 1024;` A library list of 500 themes with localized text, or `Cycling.Exclusion` (folder names, 25 characters each, unbounded), does not fit in one frame. The IPC must carry small typed messages and a Portal-specific settings view, not the file and not the library.
- **F5. Legacy validity is stricter than the Rust parser on missing keys.** [VERIFIED: src/Shared/Sucrose.Shared.Theme/Helper/Info.cs:11-50] `[JsonProperty("AppVersion", Required = Required.Always)]` (same for Version, Title, Thumbnail, Preview, Description, Author, Contact, Type, Source), `Required.AllowNull` for License, Tags, Arguments (key must exist, value may be null) and `Required.Default` for Localization. `sucrose_core::theme::SucroseInfo` gives missing keys defaults (`#[serde(default)]`) and requires only `Type` and `Source`, so it accepts themes the legacy scan calls corrupt. That direction is safe for the bin (it trashes fewer themes than legacy). The reverse direction (Rust rejects, legacy accepts) is the dangerous one: `deserialize_type` compares enum names case-sensitively while Newtonsoft's built-in enum parsing is case-insensitive [ASSUMED: A6]. Add a case-insensitive fallback for scan purposes and a fixture for it.
- **F6. The legacy library move copies everything and then deletes the source.** [VERIFIED: src/Portal/Sucrose.Portal/ViewModels/Pages/Setting/GeneralSettingViewModel.cs:718-742 and src/Shared/Sucrose.Shared.Space/Helper/Copy.cs:8-45] `SSSHC.Folder(SMML.Location, Destination)` copies every file and every subfolder of the library folder (not only theme folders), recursion with `Delete = true` then runs `Directory.Delete(Source, true)` per folder with no check that the copy succeeded, and nothing stops a destination inside the source (the copy would recurse into itself). On the maintainer's library that moves the 16 non-theme folders, the zip and `Url.txt` too. See Pattern 5 and Pitfall 5.
- **F7. C# tools glob the Setting folder.** [VERIFIED: docs/legacy/services.md section 2, Commandog Reset/Import/Export rows] Reset deletes `%AppData%\Sucrose\Setting\*.json`, Import copies `*.json` into it, Export copies them out. A new `Setting/Sucrose.json` (D-01) is matched by all three. See Pitfall 3.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Settings file (read, validate, atomic write, migrate) | Service (`sucrose` process, logic in `sucrose-config`) | Portal (view and requests only) | D-11: the service is the only reader and writer; no polling (ARCH-04) |
| Settings change broadcast, runtime state (rule paused/closed, selected and running, memory) | Service | Portal (renders) | Runtime state travels over IPC and is never stored (ARCH-04, D-23) |
| Library scan, sort, search, pagination, layout math | Portal (pure functions in `sucrose-core`) | Service (re-validates before mutating) | Read-only and UI-shaped; the 64 KiB frame limit (F4) rules out shipping the list over IPC |
| Library mutations: showcase copy, corrupt-theme move, library move | Service | Portal (requests, shows progress) | They change settings (Warehouse.Showcase, Library.Location) and must restart the wallpaper |
| Use (apply a theme) | Service | Portal (request, spinner, result) | The service owns renderers and the gate (`check_selected`) |
| Renderer fleet, restarts, fullscreen rule | Service (existing) | Renderer process | Phase 2 code; Phase 3 makes the job changeable at runtime |
| Single instance of service and Portal | Service (Portal registry on the one endpoint) | Launcher process (second `sucrose`) | D-12, D-009: no named mutexes |
| Start with the OS entry | Platform layer (`sucrose-platform`) | Service (executes the Portal request) | OS-specific (Run value, LaunchAgent, XDG); capability-reported |
| Recycle bin move | Platform layer (wrapper over the `trash` crate) | Service worker thread | OS-specific; COM must not run on a UI thread |
| Window backdrop and system theme | Portal (GPUI window) | Platform layer (capability and build probe) | GPUI applies the backdrop; the platform reports what the OS can show (D-22) |
| Thumbnail decode and cache, hover preview, background image | Portal (background executor, bounded cache) | — | UI-07; must never run on the UI thread |
| Status bar memory total | Service (samples all Sucrose pids, only while a Portal is connected) | Portal (renders) | D-19, D-008 |
| Tray (Open Sucrose, Exit or Close Applications) | Service (module, only when AppVisible) | Platform layer (tray-icon wrapper) | D-14, ARCH-03 |

## Standard Stack

### Core (already in the workspace, reused)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| gpui-kit | =0.10.0 (re-exports gpui-component 0.10.0, gpui-pre 0.3.12) | Portal UI | D-004; pinned exactly [VERIFIED: Cargo.toml `gpui-kit = "=0.10.0"`] |
| interprocess | 2.4.9 | Local sockets, one endpoint | D-009 [VERIFIED: Cargo.toml] |
| postcard + serde | 1.2.0 / 1.0.229 | Wire format; settings JSON via serde_json 1.0.154 | D-009, D-02 [VERIFIED: Cargo.toml] |
| crossbeam-channel | 0.5.17 | Service threads | D-012 [VERIFIED: Cargo.toml] |
| tray-icon | 0.26.1 (default features off) | Tray | Phase 2 [VERIFIED: Cargo.toml] |
| windows | 0.62.2 | Registry, RtlGetVersion, AllowSetForegroundWindow, TerminateProcess | Phase 2 precedent (`gpu.rs` uses `RegCreateKeyExW`) [VERIFIED: Cargo.toml, gpu.rs] |
| sysinfo | 0.40.4, default features off, `system` only | Per-process memory for the status bar | Already a workspace dependency (xtask); `memory()` is the working set and `virtual_memory()` the private bytes on Windows (documented in `xtask/src/measure.rs`) [VERIFIED: crates.io 0.40.4, 2026-11-16] |

### New dependencies (add to root `[workspace.dependencies]`, members use `workspace = true`)
| Library | Version | Used by | Purpose | Why |
|---------|---------|---------|---------|-----|
| rust-i18n | 4.2.4 | sucrose-portal | Localization (D-015, D-20) | Already in the tree through gpui-component with the same version; MIT [VERIFIED: crates.io 4.2.4, Cargo.lock] |
| image | 0.25 (resolves 0.25.10), `default-features = false`, features `jpeg png gif webp tiff` | sucrose-portal | PNG/WebP/TIFF/GIF decode, resize, frames; GPUI's `RenderImage` holds `image::Frame`, so the Portal must use the same semver-compatible copy | GPUI already depends on `image ^0.25.1`; the legacy background picker accepts png jpg jpeg tiff webp gif [VERIFIED: gpui-pre Cargo.toml, portal.md section 4] |
| jpeg-decoder | 0.3.2, `default-features = false` (no rayon) | sucrose-portal | JPEG DCT scaling to 1/2, 1/4, 1/8 so a 1080p cover is never fully decoded | The prototype measured about 230 MB to 142 MB with decode at card size; all 90 real thumbnails are JPEG (F1) [VERIFIED: exp/GPUI/Sucrose.Portal/src/thumbs.rs, docs/GPUI.md, crates.io 0.3.2] |
| trash | 5.4.0 | sucrose-platform (behind a wrapper) | Recycle bin / trash on all three OSes (D-09) | MIT, released 2026-11-17; its Windows dependencies are `windows ^0.62`, the version already in the workspace, so no second copy [VERIFIED: crates.io API dependencies] |
| async-channel | 2.5.0 | sucrose-portal | Bridge from the IPC reader thread into a GPUI `cx.spawn` loop (`send_blocking` on the thread, `recv().await` in the task) | Already in the tree via GPUI/smol. A documented exception to "one channel crate" because crossbeam receivers cannot be awaited [VERIFIED: crates.io 2.5.0, Cargo.lock; bridging pattern A11] |
| tempfile | 3.27.0 | dev-dependency of the crates and apps whose tests need scratch folders | Fixtures and kill tests | Already in the tree [VERIFIED: Cargo.lock] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| JSON settings via `serde_json` (D-02, locked) | TOML | Locked, not reopened |
| `trash` crate | `IFileOperation` by hand in `sucrose-platform` | More unsafe code, no macOS or Linux; the crate sits behind a platform wrapper so it can be swapped |
| `sysinfo` for status bar memory | `GetProcessMemoryInfo` in the platform crate | Less code in the always-on service, but three implementations; the status bar must work on every OS |
| `plist` crate for the LaunchAgent | A tested text template with XML escaping | A 12-line template is testable in Windows CI; the crate would compile only on macOS |
| `tempfile::NamedTempFile::persist` for the atomic write | `std::fs` recipe (create_new temp in the same folder, write, `sync_all`, `rename`) | Same result, one less dependency in the service; keep the std recipe and retry on sharing violations |
| `fs_extra` / `walkdir` for the library move | A small recursive copy | The move needs symlink skipping, a nested-destination guard and size/count verification that the crates do not give |

**Installation (root Cargo.toml additions):**
```toml
rust-i18n = "4.2.4"
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "gif", "webp", "tiff"] }
jpeg-decoder = { version = "0.3.2", default-features = false }
trash = "5.4.0"
async-channel = "2.5"
tempfile = "3.27"
```
Add `"Wdk_System_SystemServices"` to the `windows` features of `sucrose-platform` for `RtlGetVersion` (the same call GPUI makes, see Pattern 10). `Win32_System_Registry`, `Win32_UI_WindowsAndMessaging` and `Win32_System_Threading` are already enabled [VERIFIED: crates/sucrose-platform/Cargo.toml]. Check `deny.toml` after adding the crates: its licenses job is already red on `bzip2-1.0.6` reached only through `xtask` (UI-SPEC Registry Safety), so a new red line from this phase must be told apart from that one.

**Version verification:** [VERIFIED: crates.io API, 2026-11-24] trash 5.4.0 (MIT, 2026-11-17), rust-i18n 4.2.4 (MIT, 2026-09-29), image 0.25.10 (MIT OR Apache-2.0, 2026-03-10), jpeg-decoder 0.3.2 (MIT OR Apache-2.0, 2025-06-21), async-channel 2.5.0 (Apache-2.0 OR MIT, 2025-07-06), tempfile 3.27.0 (2026-03-11), sysinfo 0.40.4 (MIT, 2026-11-16). jpeg-decoder's last release is more than a year old (maintenance mode); it is the only crate with DCT scaling in the tree (the prototype used it), and the fallback is `image`'s own JPEG decode at full size under the same concurrency cap.

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| rust-i18n | crates.io | 5 yrs (2021-12) | 166K/wk | github.com/longbridge/rust-i18n | [OK] | Approved (already in tree) |
| image | crates.io | 12 yrs | 6.9M/wk | github.com/image-rs/image | [OK] | Approved (already in tree) |
| trash | crates.io | 7 yrs | 164K/wk | github.com/ArturKovacs/trash | [OK] | Approved |
| jpeg-decoder | crates.io | 10 yrs | 1.2M/wk | github.com/image-rs/jpeg-decoder | [OK] | Approved (prototype precedent) |
| async-channel | crates.io | 7 yrs | 7.2M/wk | github.com/smol-rs/async-channel | [OK] | Approved (already in tree) |
| tempfile | crates.io | 11 yrs | 21M/wk | github.com/Stebalien/tempfile | [OK] | Approved (already in tree, dev only) |
| sysinfo | crates.io | 11 yrs | 5.1M/wk | github.com/GuillaumeGomez/sysinfo | [OK] | Approved (already a workspace dependency) |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none
All verdicts come from `gsd-tools query package-legitimacy check --ecosystem crates`, run this session. Crates have no install-time scripts apart from `build.rs`; the build scripts of the new crates were not audited [ASSUMED: A13].

## Architecture Patterns

### System Architecture Diagram

```text
 user clicks "Use"                                     Windows sign-in (Run value)
        |                                                          |
        v                                                          v
 +--------------+   PortalHello / PortalRequest    +----------------------------------------------+
 | sucrose-     |  ------------------------------> | sucrose (service)  one process, one endpoint |
 | portal (GPUI)|  <------------------------------ |  main thread: OS message loop + event queue  |
 |              |   PortalEvent (snapshot, runtime |                                              |
 |  scan (bg)   |   state, memory, Focus, Close)   |  ipc-accept thread -> reader thread per peer |
 |  decode (bg) |                                  |    first message routes the peer:            |
 |  cache 64/48M|                                  |    Hello       -> renderer (pid must be ours)|
 +--------------+                                  |    PortalHello -> Portal registry (1 allowed)|
        ^                                          |    Launch      -> second "sucrose" (reply)   |
        | read-only scan of Library.Location       |                                              |
        | (theme files, preview, thumbnail)        |  SettingsHost (only writer)                  |
        |                                          |    validate -> apply -> atomic write -> bcast|
   Library folder                                  |  Fleet / Supervisor (Phase 2, job now mutable)|
   Setting/Sucrose.json   <-- atomic write ------- |  Library ops worker: showcase, bin, move     |
   Setting/Sucrose.json.bak                        |  Modules (each off = no thread, no timer):   |
   Setting/*.json (legacy: read once to migrate)   |    tray | fullscreen poll | memory ticker     |
                                                   +----------------------+-----------------------+
                                                                          | ToRenderer: Assign, LoadSource, Pause, ...
                                                                          v
                                                   +----------------------------------------------+
                                                   | sucrose --render <slot>  (one per monitor)   |
                                                   | libmpv on the desktop surface (Phase 2)      |
                                                   +----------------------------------------------+
```

### Recommended Project Structure
```text
crates/sucrose-core/src/
  settings.rs      # + every legacy enum the migrated keys use (formats.md section 6), via legacy_enum!
  paths.rs         # + showcase_dir(), default_library_dir(), startup value name constant
  library.rs       # NEW pure: scan(location) -> Scan{themes, corrupt, ignored}, validity (legacy rules), sort, search index, folder-name check
crates/sucrose-config/src/
  legacy.rs        # LegacyFile grows from 4 to 9 variants (no Donate, Objectionable, System, Kernel, Aurora)
  model.rs         # NEW typed, versioned model (sections), PortalSettings view, SettingChange
  migrate.rs       # NEW legacy -> model, exclusions, D-21 mapping, DateTime strings kept raw
  store.rs         # NEW atomic write, .bak, load order (file, .bak, migrate), retry on sharing violations
crates/sucrose-ipc/src/
  protocol.rs      # + ToService::PortalHello, Launch (appended); ToRenderer::LoadSource (appended)
  portal.rs        # NEW PortalRequest, PortalEvent, LaunchReply, snapshot types, wire-index tests
  lib.rs           # endpoint_name gets the data-dir suffix
crates/sucrose-platform/src/
  autostart.rs     # NEW per-OS impls (Run value / LaunchAgent / XDG) and legacy entry detection
  trash.rs         # NEW wrapper over the trash crate
  # Capabilities gains: autostart, recycle_bin, window_backdrop; plus allow_set_foreground, terminate by pid
apps/sucrose/src/
  service.rs       # run() shrinks; state moves into the modules below
  settings_host.rs # NEW owner of the store, applies SettingChange, broadcasts
  portal_link.rs   # NEW Portal registry, per-peer writer (reuse Link), Focus/Close, launch routing
  lifecycle.rs     # NEW D-12/D-13/D-14 rules as a pure state machine
  library_ops.rs   # NEW showcase copy, bin, verified move (worker thread)
  modules.rs       # NEW gate helper + memory ticker
apps/sucrose-portal/src/
  main.rs ipc.rs state.rs theme.rs i18n.rs header.rs status_bar.rs banner.rs
  library/{mod,layout,grid,card,pagination,thumbs,preview}.rs
  settings/{mod,general,personal,row}.rs   dialogs/{help,about,tray_hint,move_library}.rs
apps/sucrose-portal/locales/en.yml         # generated by xtask localize from the legacy EN XAML
apps/sucrose-portal/themes/sucrose.json    # embedded ThemeConfig (light and dark)
xtask/src/localize.rs  synth_library.rs  measure.rs (+ Portal mode)
```

### Suggested plan slicing (granularity fine, mode mvp)
- **Wave 1 (independent, parallel by crate):** core library scan + enums; config model, migration and store with fixtures; ipc Portal protocol and endpoint naming; platform autostart, trash, capabilities, memory; xtask localize (English only) and the synthetic library generator.
- **Wave 2 (service, sequential where plans touch `service.rs`):** shrink `run()` and switch the plan from `LegacySettings` to the settings host; Portal registry, single instance, Focus, Close Applications; Use and LoadSource including the engine-mpv change (spike checkpoint first); library operations; tray menu and module gates; autostart wiring.
- **Wave 3 (Portal):** shell (window, theme, backdrop, i18n, header, banner, status bar); library page (grid, card, thumbnail cache, pagination, search, sort); hover preview; settings General and Personal; dialogs; background image.
- **Wave 4:** xtask measure Portal mode, release measurements, docs and requirement wording updates (D-05, D-06, the D-21 and D-25 corrections), DECISIONS entries, human-verify checkpoints (screenshots, real-machine Use, reboot with start with the OS, side by side with C#).

### Pattern 1: Versioned settings file, migration, atomic store
**What:** One `Setting/Sucrose.json` (the name is not among the 14 legacy names [VERIFIED: formats.md section 1.3 file list and the 14 files found in F2]) with a top-level `"Version": 1` and one object per legacy file (`General`, `Portal`, `Engine`, `Backgroundog`, `Cycling`, `Hook`, `Library`, `Update`, `Warehouse`), legacy key names kept so Phase 5 settings pages and support map them one to one. Struct-level `#[serde(default)]` so a missing key gives its legacy default; unknown keys survive a round trip in a `#[serde(flatten)]` map; a file with a newer `Version` is never downgraded on write. D-21 adds `Portal.ThemeMode` (Light, Dark, System) and `Portal.Backdrop` (None, Mica, Acrylic, Tabbed). DateTime keys (`Update.Time`, `Warehouse.CefSharpTime`, `WebViewTime`) stay raw strings (F2). Skylark enums whose value sets formats.md cannot enumerate (`StorageType`) stay validated strings, not guessed enums (formats.md porting note 17).
**Load order:** (1) `Sucrose.json` parses, use it; (2) it is missing or corrupt and `Sucrose.json.bak` parses, restore from it (this also survives the C# Reset glob, F7); (3) neither exists, migrate from the real legacy folder (ignoring `SUCROSE_DATA_DIR`, Phase 2 D-08) with the Phase 2 getters and retry rules, then write. The migrator opens only the nine files it needs, so `Donate.json`, `Objectionable.json` (the plaintext token) and the write-only files are never read.
**Atomic write:** [VERIFIED: doc.rust-lang.org/std/fs/fn.rename.html] `std::fs::rename` is documented as `MoveFileExW` with a fallback to `SetFileInformationByHandle` on Windows, with Unix-like replace semantics on Windows 10 1607+ when the filesystem supports `FileRenameInfoEx`. The page makes no atomicity statement (A1), so the kill test below is part of the contract.
```rust
// Illustrative: the write path of sucrose-config/store.rs
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path.parent().ok_or_else(|| io::Error::other("settings path has no folder"))?;
    let tmp = dir.join(format!("Sucrose.json.{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;                                   // data on disk before the name flips
        drop(file);
        retry_sharing_violation(|| fs::rename(&tmp, path))  // AV and indexers hold files for a few ms
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);                      // never leave a stale temp behind
    }
    result
}
// Before replacing, copy the current good file to Sucrose.json.bak with the same recipe (once per service start
// and when the change is not trivial); the .bak name stays outside the *.json glob (F7).
```
**Service side:** `SettingsHost` holds the only `Store`. A request arrives as a typed `SettingChange` (one variant per Phase 3 key, append-only), is validated (clamp ranges from formats.md section 1.3, enum membership, path checks), applied in memory, written atomically, then broadcast. A write failure reverts the in-memory value, answers `Rejected` (the Portal shows "Setting not saved") and logs. The Portal commits slider drags on release or after 150 ms of silence; the service writes every accepted change (a file of a few KB, no coalescing needed). The fullscreen scope (Phase 2 D-05) gets its field here, default all monitors, no UI.
**Kill test (criterion 2):** a helper process rewrites the file in a loop, the test kills it at random moments (50 or more iterations) and after each kill asserts that `Sucrose.json` either parses completely or is absent next to a good `.bak`; the harness lives in `sucrose-config` tests and starts an example binary with `std::process::Command`, no app flag.

### Pattern 2: One endpoint, first-message routing, append-only Portal protocol
**What:** Keep the single `Listener`. `read_connection` today does `connection.recv::<ToService>()` and requires a `Hello` from a pid the supervisor spawned [VERIFIED: apps/sucrose/src/service.rs, `read_connection` and `accept_hello`]. Append variants and route on the first message:

| Enum | Existing variants (wire index) | Append |
|------|-------------------------------|--------|
| `ToService` [VERIFIED: protocol.rs:16; the tests assert Hello 0, Failed 1, Playing 2, Clock 3] | Hello 0, Failed 1, Playing 2, Clock 3 | `PortalHello { portal_protocol_version: u16, pid: u32 }` 4, `Launch(Launch)` 5 |
| `ToRenderer` [VERIFIED: protocol.rs:43; the tests assert Close 0 through Clock 6] | Close 0, Assign 1, Pause 2, SetVolume 3, SetRect 4, SetRole 5, Clock 6 | `LoadSource { source: String, settings: PlaybackSettings }` 7 (D-26) |

After `PortalHello` the connection speaks a second pair of enums defined in `sucrose-ipc/src/portal.rs` (`PortalRequest` from the Portal, `PortalEvent` from the service), each with its own wire-index tests like the existing ones. A `Launch` connection gets one `LaunchReply` and closes. `PROTOCOL_VERSION` stays 3 ([VERIFIED: protocol.rs:12] `pub const PROTOCOL_VERSION: u16 = 3;`) because appending needs no bump; the Portal protocol has its own constant, bumped only for changes appending cannot express.
```rust
// Illustrative message sets (names are discretion; keep them append-only and small)
pub enum PortalRequest {
    Use { folder: String },                 // the service re-validates with check_selected
    Change(SettingChange),                  // typed, one variant per Phase 3 key
    SetStartup(StartupMode),                // Normal now; Priority and Scheduler refused with a reason
    MaintainLibrary,                        // showcase copy + bin (when DeleteCorrupt)
    MoveLibrary { destination: String },
    CloseApplications,
    ReportClosing,                          // the tray hint was shown and the window is really closing
}
pub enum PortalEvent {
    Welcome { snapshot: PortalSettings, runtime: RuntimeState, startup: StartupInfo },
    AlreadyOpen { existing_pid: u32 },      // single instance (D-12)
    Changed(SettingChange),                 // broadcast after the service wrote it
    Rejected { change: SettingChange, reason: String },
    Runtime(RuntimeState),                  // selected+running, rule Paused/Closed, library moving, tray visible
    UseResult { folder: String, outcome: UseOutcome },
    LibraryReady(MaintainReport),
    MoveResult(Result<(), String>),
    Memory(MemoryReport),                   // droppable like the clock: dropped when the queue is full
    Focus,
    Close,
}
```
**Snapshot is a view, not the file (F4):** `PortalSettings` holds only the keys the Portal pages and the library use in Phase 3 (about 30 values, a few KB); per-key `Changed` events keep it current. A unit test encodes a worst-case `Welcome` (260-character paths, every string at its cap) and asserts it is far below `MAX_FRAME`.
**Writers:** reuse the bounded `Link` queue (`OUTGOING_CAPACITY = 64` [VERIFIED: service.rs:103]) for each Portal: one writer thread per Portal, `Memory` events dropped when the queue is full, any other event that cannot be queued ends that connection (the Portal reconnects), so a stuck Portal never blocks the main thread.
**Trust and validation:** the pipe DACL already limits the endpoint to the owner SID (plan 02-05), so a Portal is as trusted as any process of the same user; the service still treats every field as untrusted (single-segment folder names, range clamps, enum membership, destination path rules, string length caps) so a Portal bug cannot corrupt the settings file. Renderers stay restricted to pids the service started; a Portal cannot be pid-checked (the user starts it), so a second `PortalHello` is answered `AlreadyOpen`.
**Endpoint name and tests:** make the endpoint depend on the data folder when `SUCROSE_DATA_DIR` is set (append a short hash of the folder to `Sucrose.Service.<user_key>`). Without it a scratch-folder service started by a test or by the synthetic-library measurement collides with the maintainer's running service ("already running") or attaches to it. This is not a test-only flag: the variable already means "everything the Rust app writes goes here" (CLAUDE.md).

### Pattern 3: Service and Portal lifecycle (D-12, D-13, D-14, TRAY-05)
| Event | Action |
|-------|--------|
| `sucrose` starts, endpoint free | Bind, load settings (migrate if needed), apply the legacy Launcher rule: selected theme exists and not running, start the wallpaper (and open the Portal when the tray is hidden); nothing selected, open the Portal |
| `sucrose` starts, endpoint taken | Connect as a Launcher, send `Launch(OpenPortal { page })`; the service answers `Opened` or `Focused { portal_pid }`; on `Focused` the launcher calls `AllowSetForegroundWindow(portal_pid)` (it was just started by the user, so it can hand out the foreground right [CITED: learn.microsoft.com SetForegroundWindow page for the foreground rules; the exact conditions are A4]) and exits. This replaces the Phase 2 "log and exit" (D-23 extension) |
| `sucrose-portal` starts | Connect; if the endpoint is not there, spawn the service as `sucrose --no-portal` (a new, strictly parsed argument; `parse_mode` must keep refusing anything unknown [VERIFIED: apps/sucrose/src/main.rs:29, `const EXIT_USAGE: u8 = 64;`]) from its own folder (`current_exe().parent()`), retry the connect with backoff 50 ms doubling to 500 ms for about 10 s while showing the "Service connecting" banner; send `PortalHello`; `Welcome` continues, `AlreadyOpen` calls `AllowSetForegroundWindow(existing_pid)` and exits 0 (the service already told the first Portal `Focus`) |
| Portal registers | One Portal at a time: the registry decides under its lock, so two Portals started together get one `Welcome` and one `AlreadyOpen`. Start the memory ticker (1 s) |
| Portal disconnects | Remove it from the registry, stop the memory ticker, evaluate the D-13 exit rule |
| D-13 exit rule | Exit when the tray is not visible AND no wallpaper is wanted AND no Portal is connected; never while the tray is visible. Evaluate on Portal disconnect, wallpaper close and an `AppVisible` change |
| Tray "Open Sucrose" or double click | Disabled while a Portal is registered (legacy); else spawn the sibling `sucrose-portal`. On spawn failure log and set the tray tooltip to a short failure text (UI-SPEC E14 is unresolved, see Open Question 7) |
| Exit with `AppExit` true ("Close Applications", TRAY-05) | Start the existing shutdown worker (renderers close, 2 s grace, survivors killed), send `Close` to the Portal, wait about 2 s for its disconnect, terminate it by the pid from its Hello if it is still there, then quit. Only pids this build started or that said Hello are touched; C# processes never are (Phase 2 D-14) |
| Exit with `AppExit` false | Remove only the tray module; renderers keep playing; apply D-13 afterwards |

Opening the Portal with a page: `GeneralSetting` opens General, `SystemSetting` also opens General until Phase 5 (CONTEXT.md, Claude's discretion). The Portal's argument parser must be as strict as `parse_mode` (unknown arguments refused with exit 64 and a log line) and keep `--no-focus` [VERIFIED: apps/sucrose-portal/src/main.rs].

### Pattern 4: Use, the changeable job, and LoadSource (ARCH-05, D-23, D-26)
**Flow:** `Use { folder }` is refused when the rule has paused or closed the wallpaper (D-23, authoritative on the service side), when the type is not Video (D-17; the gate already says `NotVideo` [VERIFIED: theme.rs `ThemeRejection::NotVideo`]) or when the version is too new. Otherwise: `check_selected(location, folder, &app_version())`, write `Library.Selected` through the settings host, then apply:
1. **A fleet is running and the new theme is Video too:** build the new `Job`, update the shared assignment map (so a restarted or late renderer gets the new source), send `ToRenderer::LoadSource` to every connected renderer, no restart. The renderer calls a new `Player::load` that issues mpv's `loadfile <path> replace` [ASSUMED: A5, standard mpv command, not run this session], then publishes a fresh clock so the master election and followers realign (D-030). Reset the slot restart budgets on an explicit Use, otherwise a user who fixes a bad theme stays locked out by the old crashes.
2. **No fleet (nothing was selected, or it was closed):** set the job and run the existing `Fleet::apply(diff(&[], &monitors))` path.
3. **A failure at any step:** fall back to the Phase 2 behavior (close and respawn); a renderer that exits keeps going through `RestartBudget` (3 crashes per 60 s [VERIFIED: supervisor.rs:30-32, `const WINDOW: Duration = Duration::from_secs(60);` and `const MAX_CRASHES: usize = 3;`]), so criterion 3 holds.

**Needed code outside the service:** `sucrose-engine-mpv/src/player.rs` has `open`, `set_paused`, `seek_exact`, `loop_gen` and no load call [VERIFIED: its pub fn list]; `render.rs`'s `wait_for_close` loop handles Close, Pause, SetVolume, SetRect, SetRole and Clock and logs a second Assign as unexpected, so `LoadSource` is a new arm (and the match list in `judge_hello_reply` must name it). **Spike first:** whether the old frame stays up until the new first frame, and whether a size change flashes black, can only be seen on the maintainer's machine; plan one checkpoint with a `PrintWindow` capture before building the rest on D-26.
**Runtime state the Portal needs:** `RuntimeState { selected: Option<String>, running: bool, rule: RuleState { None, Paused, Closed }, library_moving: bool, tray_visible: bool }`, published on change only. The fullscreen `Rule` and `RuleClosed` already know the state; add a derived accessor, not a second source of truth.

### Pattern 5: Library scan, showcase, bin and move
**Scan (pure, in `sucrose-core/src/library.rs`, used by both processes):** list directories only (files such as `Url.txt` are ignored, F1). A directory is a theme when `SucroseInfo.json` parses, `Preview` and `Thumbnail` are non-empty and exist, and `Source` is a URL or an existing file [VERIFIED: src/Portal/Sucrose.Portal/Views/Pages/LibraryPage.xaml.cs:156-161, quoted: `if (File.Exists(PreviewPath) && File.Exists(ThumbnailPath) && (SSTHV.IsUrl(Info.Source) || File.Exists(Path.Combine(Folder, Info.Source))))`]. The URL test is the legacy regex `^(http|https)://[\w-]+(\.[\w-]+)+([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?(\.)?$`, case-insensitive [VERIFIED: src/Shared/Sucrose.Shared.Theme/Helper/Various.cs:21-24]; port it as code, not as a regex dependency. Add what legacy lacks: `Preview`, `Thumbnail` and `Source` come from untrusted JSON, so resolve each against the theme folder and refuse `..`, absolute and drive-prefixed values and links that leave the folder (the Phase 2 gate does this for `Source` only [VERIFIED: theme.rs `source_text_problem`]); a preview pointing at `..\..\x` would otherwise make the Portal decode an arbitrary file. Result classes: `Valid`, `Incompatible { required }` (AppVersion newer, still shown), `Corrupt`; non-directories are ignored.
**Sort and search:** `LibrarySortMode` Name (localized title), Creation (directory creation time) or Modification (last write of `SucroseInfo.json`), `LibrarySortKind` Ascending or Descending; the index per theme is localized title + description + tags, lowercased, punctuation removed, de-duplicated; a query is split on spaces, a theme matches when at least one query word is a substring of an index token, ordered by the number of matched words descending [VERIFIED: portal.md section 2]. Build the index once per scan on the scan thread.
**Maintain (service worker thread, on `MaintainLibrary`):** (a) create the library folder when it is missing; (b) showcase: every directory of `data_dir()/Showcase` whose name is not in `Warehouse.Showcase` is copied into the library with the non-deleting copy and its name recorded in the new settings only after the copy succeeded [VERIFIED: LibraryPage.xaml.cs:91-124]; under `SUCROSE_DATA_DIR` the folder is empty unless a test fills it (D-10); (c) when `DeleteCorrupt` is on, for each directory the shared scan classifies Corrupt that is accessible and not locked, move it to the bin and delete its `Cache/Properties` files as legacy does (D-09); never trust a list from the Portal, re-scan. Reply with counts (the Portal toasts singular or plural).
**Move (LIB-09, D-07):** refuse when the destination is not empty ("Destination not empty") or not writable, when the destination is inside the library or the library inside the destination (F6), or when a move is already running. Then: close the renderers and publish `library_moving`; for each top-level entry try `fs::rename` (instant and atomic on one volume) and on `ErrorKind::CrossesDevices` [ASSUMED: A2, stabilized in Rust 1.85] copy recursively, skipping symlinks, counting files and bytes; verify counts and sizes; only then write `Library.Location`, then delete the source entries that were copied (not those that were renamed), then restart the wallpaper with the new path. Any failure before the settings write: remove what this move created in the destination, keep the old location, restart the wallpaper, answer an error. Parity note for FEATURES.md: legacy moved everything in the folder and deleted the source unverified; this port moves everything too (non-theme files included), verifies first and refuses nested destinations. Do not send the old library to the bin: a large library exceeds the bin quota and Windows then deletes it permanently after a prompt [ASSUMED: A9].

### Pattern 6: Bounded image pipeline (UI-07, LIB-06)
- **Decode pool:** at most 3 concurrent decodes on `cx.background_executor()` (the prototype's `DECODE_SLOTS = 3` [VERIFIED: exp/GPUI/Sucrose.Portal/src/thumbs.rs]); requests are ordered by visibility (the current page first) and dropped when the page changes.
- **Thumbnails:** JPEG goes through `jpeg_decoder::Decoder::scale(w, h)` (returns the DCT-scaled size [VERIFIED: jpeg-decoder-0.3.2 src/decoder.rs, `pub fn scale(&mut self, requested_width: u16, requested_height: u16) -> Result<(u16, u16)>`]) then `image::DynamicImage::thumbnail` to the bucket size; other formats through `image::load_from_memory`. Buckets from the UI-SPEC cache contract: 360 px wide at scale factor 1.0 or less, 720 px above, so a resize never re-decodes. Convert RGBA to BGRA (swap bytes 0 and 2) before `RenderImage::new` [VERIFIED: gpui-pre assets.rs, `/// A cached and processed image, in BGRA format`; the prototype swaps the same way].
- **Cache:** an LRU keyed by (path, mtime, bucket) with both UI-SPEC caps (64 images or 48 MiB, whichever first, least recently shown evicted). On eviction call `cx.drop_image(image, Some(window))` (or `None` from a context without the window), which removes every frame from the sprite atlas on all windows [VERIFIED: gpui-pre app.rs:3217, `pub fn drop_image(&mut self, image: Arc<RenderImage>, current_window: Option<&mut Window>)`; window.rs:5425]. Count bytes as width x height x 4 x frames. The cache is a plain struct behind a trait for "release", so eviction is unit-tested without GPUI, and the Phase 5 store reuses it unchanged.
- **Hover preview (D-24):** decode the Preview GIF only after the hover lasted about 150 ms (a mouse sweep decodes nothing), on the background executor, with `image::codecs::gif::GifDecoder` and `into_frames()` [VERIFIED: image-0.25.10 codecs/gif.rs:436], each frame shrunk with `imageops::thumbnail` to card size and rebuilt with `Frame::from_parts(buf, 0, 0, frame.delay())` [VERIFIED: image-0.25.10 animation.rs:99,110]. **Hard caps are mandatory (F1: previews reach 4.9 MB):** at card size one 360 x 203 frame is about 292 KB, so 100 frames are about 29 MB; stop at 24 MiB or 120 frames (default pending the D-25 numbers, A8) and loop what was decoded. A multi-frame `RenderImage` animates by itself: `img()` advances `frame_index` with each frame's delay [VERIFIED: gpui-pre elements/img.rs:294-341], so only the hovered card renders it. When the hover ends drop the element, call `drop_image` for the preview and drop the Arc; the static thumbnail stays in the cache. A failed decode never starts the preview and only logs.
- **Background image:** decode once at no more than the window's pixel size (cap the long edge at 2560 px) on the same pool; static unless it is a GIF, in which case the frame and byte caps apply and a GIF over the caps shows its first frame only (UI-SPEC E9 asks for the same memory rules; this is the planner-facing reading, A8). The picker has no extension filter (`PathPromptOptions { files, directories, multiple, prompt }` [VERIFIED: gpui-pre platform.rs:2935-2944]), so validate the extension and that the file opens after selection, and show "Background image errors" inline.

### Pattern 7: Portal app structure
- **State:** one root entity (`PortalApp`) holding the mode (Library or a Settings page), search text, the scanned list, current page, the `PortalSettings` view, `RuntimeState`, connection state and the toast queue. Panels are small `Render` entities that read it. Everything that decides something (columns and card width, page slicing, the ellipsis list, sort, search, cache eviction, the memory text) is a pure function in its own file with tests.
- **IPC bridge:** one std thread runs the blocking client (`connect`, `recv` loop; a clone of the connection writes) and forwards events with `async_channel::Sender::send_blocking`; a `cx.spawn` task awaits the receiver and applies each event with `this.update(cx, ...)` [A11]. Requests go through a second bounded channel to a writer thread so the UI thread never touches the pipe. After a loss the bridge shows "Service lost" and retries 3 times, then "Service failed" with Try Again (UI-SPEC E6).
- **Pagination:** gpui-component `Pagination` has `current_page`, `total_pages`, `visible_pages`, `compact` and an ellipsis menu [VERIFIED: gpui-component-0.10.0 src/pagination.rs:42-120]; it has no jump field and the legacy scheme (9 buttons, ellipsis above 6 pages) is Sucrose's own, so decide in a short spike whether to drive the widget or compose Buttons (UI-SPEC allows both); the page list is a pure function either way.
- **Dialogs and links:** `cx.open_url(url)` for Premium, Help and About [VERIFIED: gpui-pre app.rs:1945], `cx.write_to_clipboard` for the quote [VERIFIED: app.rs:1868], `cx.prompt_for_paths` for the library folder and the image [VERIFIED: app.rs:2032]. The first close shows the tray hint (UI-05) by registering `window.on_window_should_close` [VERIFIED: gpui-pre window.rs:7162] and returning false until the dialog's countdown ends; then send `ReportClosing` so the service writes `Warehouse.HintTrayIcon = false` (the service owns the file).
- **i18n:** `rust_i18n::i18n!("locales", fallback = "en")` in the Portal crate. gpui-component has its own `rust_i18n::i18n!("locales", fallback = "en")` [VERIFIED: gpui-component-0.10.0 src/lib.rs:126] and `rust_i18n::set_locale` is process-global, so the Portal's locale and the component labels (for example `Pagination.previous`) follow one setting; Phase 3 stays English. Keys are the legacy dotted names; rust-i18n flattens nested files to dotted keys [CITED: rust-i18n-support-4.2.4 src/lib.rs, `flatten_keys`], and flat dotted keys in YAML are expected to behave the same (A7; add a test with `Portal.Quoting0`).
- **Strings:** the English text is spread over many legacy XAML files (86 quote entries alone in `src/Library/Sucrose.Resources/Locales/Portal/Portal.xaml` [VERIFIED: grep count 86]). Hand-copying is error-prone and Phase 5 needs the converter anyway (D-015), so add `cargo xtask localize --lang EN` now (reads the `x:Key` entries of the neutral and EN XAML for the keys of the UI-SPEC inventory and writes `locales/en.yml`); the UI-SPEC's new keys are added by hand.

### Pattern 8: Start with the OS (TRAY-03, D-08)
- **Windows:** HKCU `Software\Microsoft\Windows\CurrentVersion\Run`, value name from one constant in `sucrose-core` (recommend `Sucrose.Service`; the legacy value is likely `Sucrose` per the formats.md Commandog notes, so never write or delete that name), data `"<current_exe>"` quoted, no arguments, so the legacy Launcher rule resumes the wallpaper at sign-in (criterion 4). Write it with the `windows` registry API following the `gpu.rs` pattern (a safe wrapper, `// SAFETY:` on each call, the handle closed once). On enable also delete our own value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`: Task Manager's disable flag lives there and rewriting the Run value does not clear it [CITED: flamingo.run/blog/windows-startup-programs and winhelponline.com via web search; the binary format is undocumented, A3]. Detect a legacy entry read-only: the Run value `Sucrose`, and the scheduled task `\Sucrose\Autorun for Sucrose` through `schtasks /Query /TN` run hidden, only when the General page opens (never polled); the task folder is probably not readable by a standard user (A10), so do not read it directly.
- **macOS:** `~/Library/LaunchAgents/<label>.plist` with `Label`, `ProgramArguments` (an array, so no quoting problems) and `RunAtLoad`; **Linux:** `~/.config/autostart/<name>.desktop` with `Type=Application`, `Name` and `Exec` (quoted per the Desktop Entry rules) [ASSUMED: A12, training knowledge; the maintainer cannot verify either]. Build the text with pure functions (XML-escape the path, quote and escape the `Exec` value) and unit-test them on every OS; the file location is a function of an injected home directory so tests write under a temp folder.
- **Capabilities:** add `autostart: bool`; the Portal shows Priority and Scheduler disabled with "Available in a later version" and writes nothing for them; the service refuses `SetStartup(Priority|Scheduler)` with a reason. Keep the migrated `RunStartup` value (the maintainer's is 3) and never create an entry from it (D-08).
- **One-way door:** the value name is what the Phase 5 updater and uninstaller must find; record it in `docs/DECISIONS.md` (D-032 or later) in the same change.

### Pattern 9: Optional modules that really start nothing (ARCH-03)
Modules in Phase 3: tray (`AppVisible`), fullscreen poll (`FullScreenPerformance != Resume`), memory ticker and Portal broadcaster (a Portal is connected). Use the shape `start_tray` already has, `fn start_tray<T>(app_visible: bool, create: impl FnOnce() -> Result<T, PlatformError>) -> Option<T>` [VERIFIED: service.rs:328], promote it to `modules.rs` and test it with a closure that records calls. Make the live toggles real: an `AppVisible` change from the Portal creates or drops the `Tray` on the main thread (the `Option<Tray>` is mutable); the memory ticker is a `LoopTimers` timer set when the first Portal registers and killed when the last one leaves, so "off" means no timer and no sampling, not a timer that returns early. Criterion 4 evidence: unit tests for each gate (the closure is never called when off, the timer id is absent from `LoopTimers`), plus one process-level check on the real binary with a scratch data folder whose prepared settings say tray off and fullscreen Resume (the log's module lines show none started).

### Pattern 10: Theme mode, backdrop, background image (UI-01, UI-02, D-21, D-22)
- **Embedded theme:** ship the Sucrose light and dark `ThemeConfig` as one JSON string and load it with `ThemeRegistry::global_mut(cx).load_themes_from_str(include_str!(...))`, then `Theme::global_mut(cx).apply_config(&config)` [VERIFIED: gpui-component-0.10.0 theme/registry.rs:151 and theme/schema.rs:1082]; no theme folder watcher (D-008). Never edit `Theme::global_mut` colors directly (`docs/GPUI.md`). Switch mode with `Theme::change(mode, None, cx)` [VERIFIED: theme/mod.rs:408]; System follows the OS through `cx.observe_window_appearance` and `Window::appearance()` [VERIFIED: gpui-pre window.rs:2537, 3154] with `Theme::sync_system_appearance` [VERIFIED: theme/mod.rs:374].
- **Backdrop mapping:** None to `WindowBackgroundAppearance::Opaque`, Mica to `MicaBackdrop`, Acrylic to `Blurred`, Tabbed to `MicaAltBackdrop` [VERIFIED: gpui-pre platform.rs:2899-2919, variants Opaque, Transparent, Blurred, MicaBackdrop, MicaAltBackdrop]. Set it at window creation (`window_background` in `WindowOptions`) and live with `Window::set_background_appearance` [VERIFIED: window.rs:3295]. Page backgrounds need alpha 0.78 for the backdrop to show (UI-SPEC Color, `docs/GPUI.md`).
- **What the OS can show (D-22):** GPUI silently does nothing for Mica and Tabbed below Windows build 22621 (`if !status.is_ok() || version.dwBuildNumber < 22621 { return; }` with the comment `DWMWA_SYSTEMBACKDROP_TYPE is available only on version 22621 or later` [VERIFIED: gpui-pre-windows-0.3.12 src/window.rs:1610-1612]) and applies Acrylic through an undocumented composition attribute from build 17763 (`version.dwBuildNumber < 17763` returns, line 1634). The capability is therefore computed in `sucrose-platform` with the same `RtlGetVersion` read (feature `Wdk_System_SystemServices`) and exposed as `window_backdrop: bool`, true from build 22621; D-22 and the legacy rule keep Acrylic disabled on Windows 10 even though it would render. The tooltip "Requires Windows 11" is slightly off for Windows 11 21H2 (build 22000), which also lacks it; flag it to the copy owner, the UI-SPEC is not changed here. A stored value the OS cannot show applies as None without rewriting the setting.
- **Migration and defaults:** `ThemeType` Dark or Light maps to the mode; `BackdropType = Auto` maps to System + Mica; the other names map one to one (Tabbed is `MicaAltBackdrop`); a fresh install is System + None (D-21). The maintainer lands on Dark + Acrylic (F2), not Dark + None as CONTEXT.md says; the screenshot checkpoint should expect Acrylic.
- **Not decided here (UI-SPEC E9 unresolved):** the background image on macOS and Linux is an assumption until CI screenshots exist; the capability-driven disabled options are unit-testable, the rendering is not.

### Anti-Patterns to Avoid
- **Polling the settings file or a runtime-state file from any process** (ARCH-04): the Portal never reads `Sucrose.json`; it gets a snapshot and change events.
- **Putting the library list or the settings file into one IPC message** (F4): frames are 64 KiB.
- **Doing the scan, decode, trash or copy on the GPUI thread or the service main thread** (D-012). The trash call needs COM on Windows: run it on a worker thread, never on the GPUI main thread whose apartment is already set [ASSUMED: A9].
- **Trusting a folder list or a path from the Portal:** re-scan and re-validate in the service before trashing, moving or selecting.
- **Writing or deleting any legacy `Setting/*.json` or legacy startup entry** (criterion 2, D-08), including "tidying" the stale keys of F3.
- **Rendering the whole preview GIF at original size** or keeping decoded frames after the hover ends (F1, Pattern 6).
- **Answering a second launch with `Window::activate_window` from the background Portal** and expecting it to work: Windows may only flash the taskbar button; hand the foreground right from the new process (Pattern 3, A4).
- **Adding a test-only app flag** (memory: no test-only flags): tests use `SUCROSE_DATA_DIR`, prepared settings files and the data-folder-dependent endpoint.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recycle bin / trash | `IFileOperation` or per-OS code | `trash` 5.4.0 behind `sucrose-platform` | Three OSes, COM and the freedesktop trash spec |
| JPEG downscale | A resize of a full decode | `jpeg-decoder` `scale` (DCT) + `image::thumbnail` | Memory spike of a full 1080p to 4K decode per card |
| GIF frame decode and composition | A GIF parser | `image::codecs::gif::GifDecoder` | Disposal modes, transparency, delays |
| Localization lookup and fallback | A string table | `rust-i18n` (D-015) | Already used by gpui-component |
| Per-process memory | Win32 and procfs code per OS | `sysinfo` (`system` feature only) | One implementation on three OSes |
| Opening URLs, clipboard, file pickers, system theme event | Win32 calls | `cx.open_url`, `cx.write_to_clipboard`, `cx.prompt_for_paths`, `observe_window_appearance` | GPUI already abstracts them |
| Windows registry and version probes | Raw extern declarations | the `windows` crate (`RtlGetVersion`, `Reg*`) | Platform rule in `.claude/rules/platform-ffi.md` |
| Local socket, framing, owner-only DACL | Anything new | existing `sucrose-ipc` | Phase 2 work, tested |
| Restart bound for renderers | A new counter | existing `RestartBudget` | Criterion 3 is already met by it |
| Deliberate small exceptions | — | Folder copy with symlink skip and verification; the XML and `.desktop` text builders; the legacy URL check | The needed semantics are not in a crate; each is under about 60 lines and gets tests on every OS |

**Key insight:** the risky parts of this phase are the seams (settings ownership, IPC routing, process lifecycle, image memory), not any single algorithm. Every algorithm that is not a seam is small and should be a pure function, so the seams can be tested against it.

## Runtime State Inventory

This phase migrates data (legacy settings into a new file) and adds startup entries, so the inventory is part of the contract.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | 14 legacy `Setting/*.json` files (9 read, 5 never opened), `Library.json Location` (shared with the C# app), the `Warehouse.Showcase` list, theme folders in the library, `Cache/Properties` files per theme | Data migration into `Setting/Sucrose.json` once; no write to legacy files; `Cache/Properties` files deleted only for a theme moved to the bin |
| Live service config | The C# Launcher may be running with its own tray, mutexes and signal files; after a Rust library move the C# app still points at the old location (D-07) | None to change; the move confirmation warns when a C# `Setting` folder exists |
| OS-registered state | A legacy HKCU Run value (likely `Sucrose`) and the scheduled task `\Sucrose\Autorun for Sucrose` (the maintainer's `RunStartup` is 3, so the task likely exists); the GPU preference value per exe (Phase 2) | Detect and warn only; Rust writes its own Run value (recommended name `Sucrose.Service`) and never touches the legacy ones; the Phase 5 uninstaller must remove ours |
| Secrets and env vars | `Objectionable.json PersonalAccessToken` (plaintext) must not be copied; `SUCROSE_DATA_DIR`, `SUCROSE_LOG` | Never open `Objectionable.json`; no new secret is stored |
| Build artifacts | `target/` holds `libmpv-2.dll` next to the exes; the new `sucrose-portal.exe` must sit in the same folder as `sucrose.exe` for the sibling lookup | Keep both in one output folder; Phase 5 packaging preserves it |

## Common Pitfalls

### Pitfall 1: Migrating runtime keys and stale keys as typed settings
**What goes wrong:** `TransmissionPort = 63920`, `ClosePerformance` and `PausePerformance` enter the new file; a stale `ClosePerformance: true` (a C# crash artifact) later disables Use or closes the wallpaper; the four stale names (`GpuUsage`, `CpuUsage`, `CpuPerformance`, `GpuPerformance`) are dropped without anyone deciding.
**Why it happens:** D-04 says "every key is typed" and D-05's exclusions miss the runtime keys (F3).
**How to avoid:** extend the documented exclusions (Open Question 2) and test the migrator against the real `Backgroundog.json` as a fixture. Unknown legacy keys are ignored with a debug log; the new names have defaults and the maintainer's file also carries the new names for the other flags.
**Warning signs:** a migrated field named after a flag the Portal never sets; a fixture test that asserts "every key of the file" without the exclusion list.

### Pitfall 2: Strict versus lenient theme parsing
**What goes wrong:** a theme legacy shows disappears from the Rust library, or is moved to the bin.
**Why it happens:** the parsers differ in both directions (F5): Rust defaults missing keys and compares `Type` names case-sensitively; legacy requires keys and parses enum names case-insensitively [ASSUMED: A6].
**How to avoid:** classify Corrupt only for files the legacy rules also reject; add a case-insensitive type-name fallback for the scan; fixtures from the 90 real `SucroseInfo.json` files (sanitized: titles and descriptions replaced) plus synthetic edge files (the repo already has an object-version fixture [VERIFIED: crates/sucrose-core/tests/fixtures/SucroseInfo.object-version.json is tracked]). Never move a theme on a read error that may be transient (a locked file): skip it and retry on the next scan.

### Pitfall 3: The C# Reset, Import and Export tools and the new settings file
**What goes wrong:** the C# Reset deletes `Sucrose.json`; the next Rust start re-migrates from legacy files that are now defaults and the user loses Rust-only choices (the startup entry state, theme mode).
**How to avoid:** the `.bak` copy (not matched by `*.json`) and the restore-before-migrate order (Pattern 1). Document in FEATURES.md that a C# Import can overwrite the Rust file.

### Pitfall 4: A 64 KiB frame meets a big library or a long list
**What goes wrong:** `FrameTooLarge` ends the Portal connection for users with a long `Cycling.Exclusion` or long paths.
**How to avoid:** the Portal view of settings (Pattern 2) and a test with the largest realistic `Welcome`; chunking is not needed in Phase 3.

### Pitfall 5: Library move data loss
**What goes wrong:** the copy fails halfway and the source is deleted anyway (legacy), a destination inside the source recurses into itself, or the wallpaper keeps files open during the move.
**How to avoid:** the Pattern 5 order (close renderers, rename or copy, verify, write the setting, delete), nested-destination refusal, delete only after verification, restart the wallpaper on every exit path including errors. Test on a scratch tree with a read-only file and a symlink.

### Pitfall 6: Preview GIF memory
**What goes wrong:** a 5 MB GIF decodes to tens of MB of frames and each frame is also uploaded to the GPU atlas, so hovering through a page grows memory and the UI-07 criterion fails.
**How to avoid:** the caps in Pattern 6, one preview alive at a time, `drop_image` on hover end, and a measurement run that hovers over a library of GIFs for 3 rounds (scenario 1 does).

### Pitfall 7: No console in release builds
**What goes wrong:** both binaries are GUI subsystem in release, so a bad argument prints nothing; a child spawned with a console flashes a window.
**How to avoid:** log the reason (as `parse_mode` users already do); spawn the service and the Portal with `std::os::windows::process::CommandExt::creation_flags` (`CREATE_NO_WINDOW` and `DETACHED_PROCESS`) where a console could appear; use argument vectors, never a shell string.

### Pitfall 8: Two Portals started together, or a Portal started while the service shuts down
**What goes wrong:** both show a window, or the Portal connects to a service that is exiting and hangs.
**How to avoid:** the registry decides under one lock (Pattern 3); the Portal retries the whole start once when the connection closes during Hello; the service answers `Close` to a Hello that arrives after shutdown began (the renderer path already does this with `closing`).

### Pitfall 9: Task Manager's disable flag
**What goes wrong:** the user disabled Sucrose in Task Manager once, later turns "start with the OS" on in the Portal, the Run value is written and nothing starts.
**How to avoid:** clear our own `StartupApproved\Run` value on enable (Pattern 8) and read it when showing the state so the Portal can say it is disabled in Task Manager.

### Pitfall 10: The memory total counts what it should not
**What goes wrong:** the total includes C# processes (also named Sucrose.*) or counts a process twice.
**How to avoid:** the service sums only its own pid, the renderer pids from the supervisor (`Supervisor::pids`) and the registered Portal's pid; the per-process values go in the tooltip (D-19). Use the working set for the status bar and record private bytes in measurements, the `docs/MEASUREMENTS.md` convention.

### Pitfall 11: Native dialogs, focus and automation
**What goes wrong:** `prompt_for_paths` and the Windows foreground lock steal focus or fail in automated runs.
**How to avoid:** automated runs never open dialogs (CLAUDE.md); a failed picker is a `Result` from the receiver and must not crash; verify dialogs manually at the checkpoint.

### Pitfall 12: The measurement tool cannot yet measure the Portal
**What goes wrong:** D-25 asks for the Portal's private-bytes peak and final value across paging rounds, but `cargo xtask measure` samples only `sucrose.exe` processes (service and renderers), averages them over a fixed window and prints an average row [VERIFIED: xtask/src/measure.rs, usage text: `--side rust|csharp`, "rust: every sucrose.exe (service and --render renderers)"]; and the synthetic library of about 500 themes does not exist.
**How to avoid:** a Wave 1 plan adds a Portal mode (`--side portal`: the `sucrose-portal.exe` pid, time series output with peak, first-page value and final value, and a marker per round) and an `xtask synth-library` command that writes about 500 valid themes (real-size JPEG thumbnails, GIF previews between 0.2 and 5 MB, a tiny MP4 source, localized text) under a scratch folder together with a prepared `Setting/Sucrose.json` whose `Location` points at it, so no app flag is needed. Paging between rounds is manual or by the maintainer's script (no synthesized input, CLAUDE.md), so the xtask only samples and marks time.

## Code Examples

### Strict command line with the Portal-start flag
```rust
// apps/sucrose/src/main.rs: extend, do not loosen, parse_mode (VERIFIED shape: Mode::Service / Mode::Render(u8))
enum Mode { Service { open_portal: bool }, Render(u8) }
fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Service { open_portal: true }),
        [flag] if flag == "--no-portal" => Ok(Mode::Service { open_portal: false }), // started by the Portal
        [flag, slot] if flag == "--render" => slot.parse().map(Mode::Render)
            .map_err(|_| "the renderer slot must be a number from 0 to 255".to_owned()),
        _ => Err(format!("unknown arguments ({} given)", args.len())),
    }
}
```

### First-message routing in the accept path
```rust
// read_connection(): one match decides what the peer is; each branch owns its own loop and its own enum
match connection.recv::<ToService>()? {
    ToService::Hello { .. } => serve_renderer(connection, first),          // unchanged Phase 2 path
    ToService::PortalHello { portal_protocol_version, pid } => serve_portal(connection, portal_protocol_version, pid),
    ToService::Launch(launch) => answer_launch(connection, launch),         // reply once, then close
    _ => { tracing::warn!("a connection must start with a Hello"); return; }
}
```

### Capped GIF preview decode (card size)
```rust
// Portal background executor. Caps are defaults pending the D-25 measurement (A8).
const MAX_FRAMES: usize = 120;
const MAX_BYTES: usize = 24 * 1024 * 1024;
fn decode_preview(bytes: &[u8], width: u32, height: u32) -> Option<Vec<image::Frame>> {
    use image::{AnimationDecoder, codecs::gif::GifDecoder};
    let decoder = GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
    let (mut frames, mut used) = (Vec::new(), 0_usize);
    for frame in decoder.into_frames() {
        let frame = frame.ok()?;
        let delay = frame.delay();
        let mut small = image::imageops::thumbnail(frame.buffer(), width, height);
        for px in small.chunks_exact_mut(4) { px.swap(0, 2); }               // RGBA -> BGRA for GPUI
        used += small.as_raw().len();
        if frames.len() >= MAX_FRAMES || used > MAX_BYTES { break; }          // loop what we have
        frames.push(image::Frame::from_parts(small, 0, 0, delay));
    }
    (!frames.is_empty()).then_some(frames)
}
// gpui_kit::RenderImage::new(frames) accepts anything Into<SmallVec<[Frame; 1]>>, which includes Vec<Frame>.
```

### Backdrop and appearance
```rust
// sucrose-platform decides capability; the Portal only maps names (variants VERIFIED in gpui-pre platform.rs:2899-2919)
fn appearance(backdrop: Backdrop, supported: bool) -> WindowBackgroundAppearance {
    if !supported { return WindowBackgroundAppearance::Opaque; }            // stored value kept, shown as None
    match backdrop {
        Backdrop::None => WindowBackgroundAppearance::Opaque,
        Backdrop::Mica => WindowBackgroundAppearance::MicaBackdrop,
        Backdrop::Acrylic => WindowBackgroundAppearance::Blurred,
        Backdrop::Tabbed => WindowBackgroundAppearance::MicaAltBackdrop,
    }
}
```

### Start-with-OS text builders (pure, tested on every OS)
```rust
fn xml_escape(text: &str) -> String { text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;") }
fn launch_agent(label: &str, exe: &str) -> String {
    format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\
<key>Label</key><string>{}</string><key>ProgramArguments</key><array><string>{}</string></array>\
<key>RunAtLoad</key><true/></dict></plist>\n", xml_escape(label), xml_escape(exe))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Legacy settings: truncate-then-write, named mutex per file, swallowed errors | One versioned file, temp + flush + rename, service as the only writer | This phase (D-02, D-11) | Kill during a write cannot corrupt; no cross-process lost update |
| Named mutexes for single instance, signal files and TCP for commands | One local-socket endpoint, first-message routing | Phase 2 (D-009), extended here | No mutex names, no polling |
| Legacy Reset/Import with process kills | Settings change over IPC, validated by the service | This phase | Live update, no restart |
| `img(path)` full-resolution decode that stays cached | Own decode at card size, bounded LRU, `drop_image` on eviction | Prototype lesson (`docs/GPUI.md`) | UI-07 |
| Corrupt themes deleted permanently | Moved to the OS bin | D-09 | Restorable when the parser is stricter than Newtonsoft |

**Deprecated/outdated:** the `Setting/Library.json` and `Warehouse.json` values stop being written (the legacy files are frozen after migration, D-03); LIB-01 wording "tracked in Warehouse.json" and FMT-01 "except Donate.json" change with D-05 and D-06.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `fs::rename` replaces the destination and is crash-safe on NTFS in practice (std documents no atomicity) | Pattern 1 | A kill during rename could still lose the main file; the `.bak` and the kill test bound the damage |
| A2 | `io::ErrorKind::CrossesDevices` is stable in Rust 1.99 and returned for a cross-volume rename | Pattern 5 | The move falls back to copy only on a different error kind; add a test on a second volume or treat any rename error as "copy" |
| A3 | A Task Manager "disabled" flag under `StartupApproved\Run` survives rewriting the Run value, and deleting our value there resets it | Pattern 8, Pitfall 9 | Start with the OS appears enabled but does not start; checkpoint on a real reboot |
| A4 | A process just started by the user may call `AllowSetForegroundWindow(pid)` so the running Portal can take the foreground | Pattern 3 | The existing Portal only flashes its taskbar button; fallback is acceptable, state it in the checkpoint |
| A5 | mpv `loadfile <path> replace` keeps the last frame up until the new first frame for a same-size video and the master clock realigns cleanly | Pattern 4 | D-26 "no black or static frame" fails; fall back to restart (the D-26 failure path) and re-ask |
| A6 | Newtonsoft's built-in enum reader accepts any casing of a name, so a legacy theme with `"video"` is valid | Pitfall 2 | Rust hides a theme legacy shows; low frequency, tested by a fixture |
| A7 | rust-i18n resolves a flat YAML key containing dots (`Portal.Quoting0`) the same way as a nested key | Pattern 7 | Strings show as keys; a one-line test finds it immediately |
| A8 | The preview caps (120 frames, 24 MiB), the cache caps (64 images, 48 MiB) and the 150 ms hover delay are right | Pattern 6 | UI-07 measured numbers decide; the UI-SPEC marks the cache numbers as defaults |
| A9 | The `trash` crate needs a non-UI thread on Windows (COM apartment) and a large folder to the bin triggers a permanent-delete prompt | Pattern 5, anti-patterns | Hang or a surprise dialog; both are avoided by the chosen design (worker thread, no bin for the move) |
| A10 | The scheduled-task folder under `System32\Tasks` is not readable by a standard user, so `schtasks /Query` is the way to detect the legacy task | Pattern 8 | Detection fails and the "both apps will start" warning is missing |
| A11 | Awaiting an `async-channel` receiver inside `cx.spawn` wakes the GPUI executor correctly when `send_blocking` is called from another thread | Pattern 7 | Events arrive late; verified in the first Portal IPC plan with a smoke test |
| A12 | LaunchAgent plist keys (`Label`, `ProgramArguments`, `RunAtLoad`) and the XDG `.desktop` fields are as described | Pattern 8 | macOS and Linux autostart fails; the maintainer cannot check, CI only checks the text |
| A13 | The build scripts of the new crates do nothing unexpected | Package audit | Supply-chain risk in a dependency |

## Open Questions (RESOLVED)

Every question below carries its resolution: answered by the maintainer on 2026-11-24 and recorded in 03-CONTEXT.md (questions 1, 2, 4 and 5), or the recommendation adopted by a named plan (questions 3, 6, 7, 8 and 9). Plan numbers refer to the revised set 03-01 … 03-26 (plan 26 was split from plan 04).

1. **Correct two facts in CONTEXT.md (D-21 and D-25)?**
   - What we know: the maintainer's real `BackdropType` is `Acrylic`, so D-21 migrates them to Dark + Acrylic; the real library has 90 valid themes (3 pages), not 108 (4 pages), because 18 entries are not themes (F1, F2).
   - What's unclear: whether the maintainer wants D-25 to measure the 90-theme library as is (yes by the data) or also a 108-entry variant.
   - Recommendation: plans use the real numbers and say so; ask the maintainer to confirm at the first checkpoint, no re-discussion needed.
   - **Resolution (RESOLVED: maintainer, 2026-11-24, CONTEXT.md D-21 and D-25):** both facts are corrected in CONTEXT.md: D-21 says the maintainer migrates to Dark + Acrylic, and D-25 measures the real library as it is (90 valid themes among 108 entries, 30 per page, 3 pages); no 108-entry variant is measured. Plan 04 Task 1 checks Dark + Acrylic on the real files (ignored test `real_install_migrates`); plan 24 measures the 90-theme library and the about-500-theme synthetic library of plan 05, and plan 25 records the result.
2. **Runtime-state keys in the migration (ARCH-04 versus D-04, D-05)**
   - What we know: `Backgroundog.json` carries `TransmissionPort`, `PipeRequired`, `SignalRequired`, `TransmissionRequired`, `AudioRequired`, `PausePerformance`, `ClosePerformance`, `Cycling.PassingTime` and `CommunicationType` (F2, F3); formats.md calls the first seven runtime state; ARCH-04 forbids runtime state in settings.
   - What's unclear: the maintainer's wording for the exclusion list.
   - Recommendation: add these nine to the D-05 exclusion list (same change that edits criterion 2, FMT-01 and FEATURES.md) and map none of the four stale old names.
   - **Resolution (RESOLVED: maintainer, 2026-11-24, CONTEXT.md D-05):** the eight Backgroundog runtime keys and `Cycling.json` `PassingTime` are on the D-05 exclusion list, and `GpuUsage`, `CpuUsage`, `CpuPerformance` and `GpuPerformance` map to no field. Plan 26 Task 1 implements and tests the exclusions and the stale names; plan 26 Task 2 rewrites ROADMAP criterion 2, FMT-01 and the FEATURES.md Formats row in the same plan.
3. **Legacy strictness (Required.Always) for the scan**
   - What we know: Rust is more lenient on missing keys (safe for the bin) and less lenient on enum name case (unsafe).
   - Recommendation: keep Rust lenient on missing keys, add case-insensitive type names, document both in FEATURES.md.
   - **Resolution (RESOLVED: recommendation adopted by plan 02):** the scan keeps missing optional `SucroseInfo.json` keys lenient and matches `Type` names in any case like Newtonsoft (finding F5), while ordinals stay as they are; plan 02 implements and tests both and records them in the FEATURES.md Library rows.
4. **Library move: everything or themes only**
   - What we know: legacy moves the whole folder content including non-theme files and deletes the source (F6); D-07 says "themes are copied and the old folder removed".
   - Recommendation: move everything with verification (parity, nothing lost); confirm wording with the maintainer when they review the move dialog copy.
   - **Resolution (RESOLVED: maintainer, 2026-11-24, CONTEXT.md D-07):** everything in the library folder moves (theme folders and every other file): rename first, otherwise copy and verify, write the setting, and only then delete the source; a destination inside the source is refused, and the confirmation mentions the C# app when its `Setting` folder exists. Plan 22 implements the move, the refusals, the rollback and the confirmation copy.
5. **Should the D-26 spike gate the dependent plans?**
   - What we know: `Player::load` does not exist and the no-black-frame claim is untested (A5).
   - Recommendation: a checkpoint plan before the dependent Portal plans; the fallback (restart) is already specified in D-26.
   - **Resolution (RESOLVED: maintainer, 2026-11-24, CONTEXT.md D-26):** yes. Plan 09 builds the LoadSource transition with the restart path behind `USE_TRANSITION`; plan 11 is the blocking checkpoint on the maintainer's machine before plan 12 (Use from a card), and it switches to the restart path and asks the maintainer again if a black or static frame appears.
6. **Endpoint naming per data folder and the `--no-portal` flag**
   - Both are small additions CONTEXT.md leaves to discretion (Portal starts the service; both single-instance checks use the endpoint). Recommendation: adopt both as written in Patterns 2 and 3 and record them in DECISIONS.md.
   - **Resolution (RESOLVED: recommendation adopted by plans 06, 08 and 15):** plan 06 names the endpoint per data folder (a hash of `SUCROSE_DATA_DIR` when it is set, the unchanged name otherwise) and records it, with the Portal-started service flag, in D-033 of `docs/DECISIONS.md`; plan 08 adds `sucrose --no-portal` and starts the service with it from the Portal; plan 15 makes the service honor it.
7. **UI-SPEC E14 (unresolved): what the user sees when "Open Sucrose" cannot start the Portal**
   - Recommendation: log an error and set the tray tooltip to a short failure text; no dialog, because the service has no window.
   - **Resolution (RESOLVED: recommendation adopted by plan 16):** when Open Sucrose cannot start the Portal, the service logs the error and sets the tray tooltip to "Sucrose couldn't open its window"; no dialog.
8. **UI-SPEC E9 on macOS and Linux (unresolved):** the background image there stays an assumption until CI screenshots exist; the plans test only the capability-driven option states and the setting plumbing.
   - **Resolution (RESOLVED: recommendation adopted by plan 17, accepted limitation):** plan 17 unit-tests the capability-driven backdrop option states and the background image plumbing and states that the image rendering on macOS and Linux is not checked by hand until CI screenshots exist (D-002); no hardware for those systems is available this phase (Environment Availability).
9. **The CI `deny` job is red** on `bzip2-1.0.6` through `xtask` (UI-SPEC); decide in the first dependency plan whether to allow it in `deny.toml` so this phase's new crates can be judged by a green job.
   - **Resolution (RESOLVED: recommendation adopted by plan 03):** plan 03 allows the `bzip2-1.0.6` license id in `deny.toml` with a comment naming the crate that needs it (reached only through `xtask`), so the CI deny job judges this phase's new crates; the advisories, bans and sources checks stay unchanged (T-03-08).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (Windows wrapper in WSL) | Everything | ✓ | cargo 1.99.0, rustc 1.99.0 (pinned in `rust-toolchain.toml`) | — |
| Docker | Linux build check | ✓ | 29.9.3 | CI |
| bun | GSD tooling | ✓ | present at ~/.bun/bin | — |
| Windows 11 desktop with the C# app installed | UAT, backdrop, startup, side by side | ✓ (maintainer machine) | build 26300 (Phase 2 measurements) | — |
| Maintainer library `H:\Sucrose` and `%APPDATA%\Sucrose` | Fixtures, D-25 scenario 1 | ✓ | 90 valid themes, 14 setting files | Synthetic library (xtask) |
| libmpv next to the exes | Use on the desktop | ✓ | `target/release/libmpv-2.dll` present, pinned in `xtask/engines.toml` | `cargo xtask fetch-engines` |
| macOS and Linux desktops | Autostart, backdrop disabled states | ✗ | — | CI builds and unit tests only; text builders tested on Windows |
| Second monitor, 4K video | Not needed this phase | — | — | — |

**Missing dependencies with no fallback:** none for the phase; macOS and Linux behavior stays unverified by hand (known, D-002 and the UI-SPEC E9 note).
**Missing dependencies with fallback:** macOS and Linux desktops (CI only).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in harness, Rust 1.99.0); GPUI `TestAppContext` is available behind `test-support` but is not needed: logic is pure |
| Config file | none beyond `Cargo.toml` per crate; fixtures under each crate's `tests/fixtures/` |
| Quick run command | `cargo test -p sucrose-config -p sucrose-core -p sucrose-ipc` (and `-p sucrose`, `-p sucrose-portal`, `-p sucrose-platform` for the touched crate) |
| Full suite command | `cargo xtask ci` (fmt, clippy `-D warnings`, test, the same as CI) |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FMT-01 | Every key of the 9 files migrates (real-file fixtures incl. stale and runtime keys), exclusions honored, D-21 mapping, DateTime kept raw, enums by name, clamps | unit + fixtures | `cargo test -p sucrose-config migrate` | ❌ Wave 0 |
| FMT-06, ARCH-04 | Setting layout kept, `Sucrose.json` beside 14 files, atomic write, `.bak` restore, kill-during-write loop never leaves a corrupt file, legacy files byte-identical after migration | integration | `cargo test -p sucrose-config store` | ❌ Wave 0 |
| ARCH-02 | First-message routing, `AlreadyOpen`, Launch reply, wire indices of the appended variants, largest Welcome below MAX_FRAME, endpoint depends on the data folder | unit + local-socket tests (Windows named pipe in CI) | `cargo test -p sucrose-ipc` | ✅ crate, ❌ new tests |
| ARCH-05, TRAY-05 | A Portal client against the real service binary on a scratch data folder: Welcome, Change persists across restart, Use refused for non-Video and for rule paused, CloseApplications ends service and renderers | integration (`CARGO_BIN_EXE_sucrose`) | `cargo test -p sucrose --test portal` | ❌ Wave 0 |
| ARCH-01 | Restart budget survives a Use; a renderer that keeps crashing stops after 3 in 60 s | unit (existing supervisor tests plus Use cases) | `cargo test -p sucrose supervisor` | ✅ partial |
| ARCH-03 | Gate closures never called when off; memory timer absent without a Portal; tray not built when AppVisible is false | unit | `cargo test -p sucrose modules` | ✅ `start_tray` tests, ❌ rest |
| TRAY-03 | Run value text and path quoting, StartupApproved reset, legacy entry never touched (registry in a test hive path), plist and `.desktop` text, injected home dir | unit | `cargo test -p sucrose-platform autostart` | ❌ Wave 0 |
| LIB-01, LIB-02, LIB-08 | Showcase copy once and recorded; legacy validity rules on real and synthetic themes; non-directories ignored; incompatible theme classified; corrupt to the bin only when on (bin call behind a trait, real bin tested once manually) | unit + fixtures | `cargo test -p sucrose-core library` | ❌ Wave 0 |
| LIB-03, LIB-04 | Sort modes and kinds, search index and ranking, columns, card width, page slicing, ellipsis list | unit | `cargo test -p sucrose-portal library` | ❌ Wave 0 |
| LIB-09 | Move: empty-destination rule, nested refusal, rename and copy paths, verification, failure rollback, setting written only after success | unit on temp trees | `cargo test -p sucrose library_ops` | ❌ Wave 0 |
| LIB-06, UI-07 | Cache eviction by count and bytes, release callback per eviction, GIF caps, one preview alive | unit | `cargo test -p sucrose-portal thumbs` | ❌ Wave 0 |
| UI-01, UI-02 | Backdrop mapping, capability false gives Opaque and a kept setting, theme mode mapping (Auto to System + Mica) | unit | `cargo test -p sucrose-portal theme` | ❌ Wave 0 |
| UI-03, UI-05 | Search debounce rule and lowercasing, header actions present (no placeholders), argument parsing strictness, Help and About link set, tray hint countdown | unit + screenshot | `cargo test -p sucrose-portal`; screenshot by hand | ❌ Wave 0 |
| UI-07 (measurement) | Portal private-bytes peak and final stay within first page + cache cap across 3 rounds, on 90 real and about 500 synthetic themes | manual, release build | `cargo xtask measure --side portal` (to be added) | ❌ Wave 1 |

### Sampling Rate
- **Per task commit:** the quick command of the touched crate.
- **Per wave merge:** `cargo xtask ci`.
- **Phase gate:** full suite green, then the human-verify checkpoints (`human_verify_mode: end-of-phase`) before `/gsd-verify-work`: screenshots of the library, settings and dialogs in light, dark and Acrylic; Use of a Video theme; Exit closes every Sucrose process; reboot with start with the OS on; the C# app still works side by side with unchanged `Setting/*.json`.

### Wave 0 Gaps
- [ ] `crates/sucrose-config/tests/fixtures/`: sanitized copies of the nine real settings files (Backgroundog with the runtime and stale keys, Portal with Acrylic, General with TR and RunStartup 3), a fake `Objectionable.json` and `Donate.json` that must never be read — covers FMT-01
- [ ] `crates/sucrose-core/tests/fixtures/library/`: a handful of real `SucroseInfo.json` files (Video, Gif, Web, sanitized), synthetic corrupt and incompatible themes, a folder with only `project.json`, a loose file — covers LIB-01, LIB-02, LIB-08
- [ ] `crates/sucrose-config/examples/write_loop.rs` (or a test helper binary) for the kill test — covers ARCH-04
- [ ] `apps/sucrose/tests/portal.rs`: a Portal test client helper against the real binary — covers ARCH-02, ARCH-05, TRAY-05
- [ ] `xtask measure --side portal` and `xtask synth-library` — covers UI-07
- [ ] `xtask localize` and `apps/sucrose-portal/locales/en.yml` — covers D-20, UI-03, UI-05
- [ ] Framework install: none (the harness is built in)

## Security Domain

`security_enforcement` is enabled and `security_asvs_level` is 1 in `.planning/config.json` [VERIFIED: file read].

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No accounts; the peer is the same OS user |
| V3 Session Management | no | Connections are local sockets without sessions |
| V4 Access Control | yes | Pipe DACL limited to the owner SID (plan 02-05); renderers only from pids the service started; one Portal; the service validates every request regardless of the peer |
| V5 Input Validation | yes | Typed postcard messages with a 64 KiB frame limit; typed `SettingChange` with clamps and enum membership; single-segment folder names; destination and image path checks; string length caps; untrusted `SucroseInfo.json` paths resolved inside the theme folder |
| V6 Cryptography | no | None added; no secret stored; `Objectionable.json` token never read |
| V8 Data Protection | yes | Settings contain no secrets (PAT excluded); logs carry values and not paths or theme names where Phase 2 already avoids them (T-02-40) |
| V12 Files and Resources | yes | Symlinks skipped in copy; paths from JSON resolved and contained; temp file in the same folder removed on failure; the bin instead of delete |
| V14 Configuration | yes | HKCU Run value written from `current_exe` only, quoted; `CREATE_NO_WINDOW` spawns with argument vectors; no shell strings |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| A theme's `Preview`, `Thumbnail` or `Source` points outside its folder (`..`, absolute, drive, link) | Information disclosure, Tampering | Resolve against the theme folder, refuse and log; the Phase 2 gate for `Source` stays; the scan applies the same rule to all three |
| Hostile or oversized GIF or JPEG (decompression bomb) | Denial of service | Decode pool of 3, byte and frame caps, dimension limits before decode, never on the UI thread |
| A Portal bug or another same-user process sends crafted requests | Tampering | Service-side validation and clamping, typed enums, rejected requests change nothing |
| Endpoint squatting before the service starts | Spoofing | Owner-only DACL (same user only); residual risk is the same-user trust boundary; the Portal sends only paths and setting values |
| Library move or showcase copy follows a symlink or recurses into itself | Tampering, Denial of service | Skip symlinks, nested-destination refusal, verification before delete |
| Autostart value injection through a path with quotes | Elevation of privilege | Value written from `current_exe` only and quoted; plist and `.desktop` text escaped and tested |
| Corrupt-theme removal destroys user data | Tampering | Bin instead of delete, only on re-scan in the service, never on a transient read error |
| Settings file corrupted by a kill | Denial of service | Temp + flush + rename, `.bak` restore |

## Sources

### Primary (HIGH confidence)
- In-repo files read this session: `03-CONTEXT.md`, `03-UI-SPEC.md`, `REQUIREMENTS.md`, `STATE.md`, `ROADMAP.md`, `docs/{ARCHITECTURE,DECISIONS,GPUI,MEASUREMENTS}.md`, `docs/legacy/{formats,portal,services}.md`, `.claude/rules/*`, `crates/sucrose-{ipc,core,config,platform}`, `apps/sucrose/src/{main,service,supervisor,render}.rs`, `apps/sucrose-portal`, `xtask/src/measure.rs`, `exp/GPUI/Sucrose.Portal/src/thumbs.rs`, C# sources `LibraryPage.xaml.cs`, `GeneralSettingViewModel.cs`, `Copy.cs`, `Info.cs`, `Various.cs`.
- Registry sources of the pinned crates (read in `%USERPROFILE%/.cargo/registry/src`): gpui-pre 0.3.12, gpui-pre-windows 0.3.12, gpui-component 0.10.0, gpui-kit 0.10.0, image 0.25.10, jpeg-decoder 0.3.2, rust-i18n-support 4.2.4.
- crates.io API (versions, licenses, dependencies) and `gsd-tools query package-legitimacy check`, 2026-11-24.
- Read-only scan of `H:\Sucrose` and `%APPDATA%\Sucrose\Setting` on the maintainer's machine, 2026-11-24 (names, types and sizes only; the token file was not opened).

### Secondary (MEDIUM confidence)
- doc.rust-lang.org/std/fs/fn.rename.html (fetched): Windows implementation notes.
- Web search summary on `StartupApproved` (flamingo.run, winhelponline.com, thewindowsclub.com): where Task Manager keeps the disabled state.

### Tertiary (LOW confidence)
- Training knowledge, tagged [ASSUMED] in the Assumptions Log: foreground rules, mpv `loadfile` behavior, macOS and Linux autostart formats, trash crate COM behavior.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH, all crates verified on crates.io and most already in the lock file.
- Architecture: HIGH for the seams (read from the code), MEDIUM for the Use transition and GPUI rendering behavior (A5, A11 need a spike).
- Pitfalls: HIGH for the real-data ones (F1 to F7), MEDIUM for the Windows shell ones (A3, A4, A10).

**Research date:** 2026-11-24
**Valid until:** 2026-12-24 (stable toolchain and pinned UI crates; re-check the crates.io versions if planning slips past a month)
