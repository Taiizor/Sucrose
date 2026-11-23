# Phase 3: M2 — Service, IPC and Portal connected - Context

**Gathered:** 2026-11-23
**Status:** Ready for planning

<domain>
## Phase Boundary

The service and the Portal work as one app. The Portal (built from scratch in `apps/sucrose-portal`) shows the library (showcase copy, legacy validity scan, corrupt and incompatible themes, sort, search, pagination, adaptive layout, hover previews, moving the library), and "Use" applies a Video theme through the service. Legacy `Setting/*.json` values migrate once into a new versioned settings file with atomic writes; the service owns and persists it. Single instance for the service and the Portal over the one IPC (D-009), Exit closes every Sucrose process, renderer restarts stay bounded, optional modules do not start when off, and Sucrose can start with the OS (normal startup) and resume the wallpaper. The Portal shell has the backdrop and background image, light/dark/OS theme, header actions, search, status bar, Help/About and the tray hint dialog, and a bounded thumbnail cache measured in a release build.

Not in this phase: other engines and wallpaper types (Phase 4), the full settings pages (UI-04), the card context menu (LIB-05), drag and drop (LIB-07), the full tray menu (TRAY-02), Priority/Scheduler startup (TRAY-04), 25 languages (UI-06), telemetry, the updater and the installer (Phase 5).

</domain>

<decisions>
## Implementation Decisions

### Settings format and migration
- **D-01:** The new settings are **one versioned file inside the legacy `Setting/` folder** (for example `Setting/Sucrose.json`; the name must not collide with the 14 legacy file names) with a top-level version field. The data folder layout stays as it is (FMT-06): the C# `ControlFile` only touches its own 14 files, and its "Create Log File" zip picks the new file up with the folder. Sucrose never writes, renames or deletes a legacy `Setting/*.json` file (criterion 2). — **Reversibility:** one-way — once released, every later Rust version, the Phase 5 installer and the updater read the file at this path; moving it needs another migration. **Confirmed by the maintainer on 2026-11-24 (reversibility gate, plan-check iteration 2): the path is exactly `Setting/Sucrose.json`.**
- **D-02:** The format is **JSON** (`serde_json`, already a workspace dependency): human-readable, versioned, written atomically (temporary file in the same folder, flushed, then renamed over the old one) so that killing Sucrose during a write never leaves a corrupt file. — **Reversibility:** costly — the reader, the writer, the fixtures and any later migration step depend on it.
- **D-03:** Migration runs **once, when the new file does not exist**: read the legacy files (always from the real `%APPDATA%\Sucrose\Setting`, ignoring `SUCROSE_DATA_DIR`, Phase 2 D-08), write the new file, and never read the legacy files again. Changes made in the C# app afterwards are not picked up. Without a C# install the legacy defaults apply (D-024). A missing or invalid key falls back to its legacy default with the Phase 2 rules (clamp out-of-range numbers, wrong type or unknown enum name gives the default, enums by name, retry a locked or empty file 3 × 50 ms).
- **D-04:** **Every migrated key is a typed field now**, not only the keys Phase 3 uses: each has its legacy default, range and enum names from `docs/legacy/formats.md` §1.3 and §6. Fixture tests from real files prove each key migrates. Later phases use the ready fields instead of converting raw values.
- **D-05:** Documented exclusions, **not migrated**: `Donate.json` (D-023); `Objectionable.json` `PersonalAccessToken` (GitHub integrations removed by D-020, and a plaintext secret); `Portal.json` `StoreServerType` (Soferity only, D-020); `Aurora.json` `AppProcessName` (runtime state, ARCH-04); `Kernel.json` `Information` and the `System.json` interface lists (write-only system information, regenerated at runtime); `Backgroundog.json` `TransmissionPort`, `PipeRequired`, `SignalRequired`, `TransmissionRequired`, `AudioRequired`, `PausePerformance`, `ClosePerformance`, `CommunicationType` and `Cycling.json` `PassingTime` (runtime state, ARCH-04; added after research finding F3, 2026-11-24). The stale `Backgroundog.json` names `GpuUsage`, `CpuUsage`, `CpuPerformance` and `GpuPerformance` are not mapped to any field. In the same change, update success criterion 2 in `.planning/ROADMAP.md`, FMT-01 in `.planning/REQUIREMENTS.md` and the matching `docs/FEATURES.md` row to "every value except `Donate.json` and the documented exclusions" and list them.
- **D-06:** `Warehouse.json` values (`Showcase` list, `HintTrayIcon`, the CefSharp/WebView time and continue flags) migrate into the new file like every other key. The showcase copy (LIB-01) is tracked in the new settings from then on; reword LIB-01 ("tracked in `Warehouse.json`") accordingly. On the maintainer's machine all 20 showcase names are already listed, so nothing is copied again.

### Side by side with the C# app
- **D-07:** The library is the migrated `Library.json` `Location`, used as is (the maintainer's is `H:\Sucrose`, shared with the C# app). LIB-09 moves it **like the legacy app** (the destination must be empty, the whole folder content is moved and the old folder removed when `Move` is on, the wallpaper restarts). Like legacy, everything moves (theme folders and every other file); unlike legacy, nothing is lost: rename first, otherwise copy and verify, write the setting, and only then delete the source; a destination inside the source is refused (maintainer, 2026-11-24, after research finding F6). When a C# install is detected (its `Setting` folder exists), the confirmation says that the C# app keeps pointing at the old location and will see an empty library. Sucrose never writes the C# `Library.json`.
- **D-08:** "Start with the OS" (TRAY-03, normal startup): migration copies `RunStartup` but **creates no startup entry by itself**. Sucrose writes **its own entry** (Windows: an HKCU Run value with a name different from the legacy one; macOS LaunchAgent; Linux XDG autostart) only when the user turns it on in the Rust Portal. It **never modifies or removes** a C# entry (the legacy Run value, likely named `Sucrose` per `formats.md` `✔Startup✖Sucrose✖…`, or the scheduled task `\Sucrose\Autorun for Sucrose`). When a C# entry exists, the Portal warns that both apps will start at sign-in. Values 2 (Priority) and 3 (Scheduler) are kept in the settings, shown as available later (TRAY-04, Phase 5) and create no entry; the maintainer's value is 3. — **Reversibility:** costly — the entry name becomes something the Phase 5 updater and uninstaller must find and remove. The maintainer confirmed the names on 2026-11-24: HKCU Run value `Sucrose.Service`, LaunchAgent label `dev.sucrose.Sucrose`, XDG file `sucrose.desktop`.
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
- **D-21:** **Two independent settings:** theme mode {Light, Dark, System} and backdrop {None, Mica, Acrylic, Tabbed}. Migration: `ThemeType` Dark/Light → Light/Dark; `BackdropType` `Auto` → theme mode System plus backdrop Mica; the other `BackdropType` values map one to one (Tabbed is GPUI `MicaAltBackdrop`). A fresh install defaults to System + None (closest to the legacy "OS theme on first read"). The header toggle sets Light or Dark explicitly. Record the split in `docs/FEATURES.md`. The maintainer migrates to Dark + Acrylic (their `Portal.json` `BackdropType` is `Acrylic`; corrected after research finding F2, 2026-11-24).
- **D-22:** Backdrops an OS cannot show (macOS, Linux, Windows 10) stay **visible but disabled with the reason in a tooltip** (for example "Requires Windows 11"), driven by platform capabilities. A stored value the OS cannot show is applied as None without changing the setting. The background image with stretch and opacity works on every OS (UI-01).

### Fullscreen rule and Use
- **D-23:** While the fullscreen rule has paused or closed the wallpaper, Use is **disabled with the legacy suffix " (Paused)" or " (Closed)"** and enabled again when the rule recovers. The service publishes the rule state over IPC (runtime state, never stored in settings). The selection does not change.

### Hover preview and memory
- **D-24:** With `LibraryPreview` on (legacy default off), the Preview GIF is **decoded at card size on a background thread, only the hovered card animates, and its frames are released and dropped from GPUI's image atlas when the hover ends**; the static thumbnail stays. `LibraryPreviewHide` hides the title/description strip while previewing.
- **D-25:** UI-07 is measured in a release build and recorded in `docs/MEASUREMENTS.md` with two scenarios: (1) the maintainer's real library (90 valid themes among 108 entries, 30 per page, 3 pages; corrected after research finding F1, 2026-11-24): first page, every page forward and back for 3 rounds, hover previews; Portal private bytes peak and final value. (2) A synthetic library of about 500 themes under `SUCROSE_DATA_DIR`, with a prepared new settings file whose `Location` points at it (no test-only app flag), same scenario. Pass: the final value stays within the first-page value plus the cache cap and does not grow across rounds. Reuse `cargo xtask measure`.

### Use transition
- **D-26:** Switching from one Video theme to another **loads the new source into the running renderers** through a new IPC message (appended, never inserted): no restart, no black or static frame in between, and the master clock realigns per D-030. When the new theme needs a different engine (Phase 4 onward), the renderers restart. A failure falls back to the Phase 2 D-13 behavior. A checkpoint plan verifies the no-black-frame load on the maintainer's machine before the dependent Use plans; if it fails, the restart path applies and the maintainer is asked again (maintainer, 2026-11-24, research assumption A5).

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` → Phase 3: goal, success criteria 1–5, notes (Portal from scratch, runtime state over IPC, Video only until Phase 4, UI-07 cache reused by the store, normal startup only)
- `.planning/REQUIREMENTS.md` → ARCH-01…05, FMT-01, FMT-06, TRAY-03, TRAY-05, LIB-01…04, LIB-06, LIB-08, LIB-09, UI-01…03, UI-05, UI-07 (and neighbours LIB-05, LIB-07, UI-04, UI-06, TRAY-02, TRAY-04, MEAS-02 for what stays out)
- `.planning/PROJECT.md`: constraints and out-of-scope list
- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-CONTEXT.md`: Phase 2 decisions carried forward (D-08 legacy read location, D-13, D-14, D-15, D-21, D-22, D-23)

### Decisions and architecture
- `docs/DECISIONS.md`: D-002, D-004, D-007, D-008, D-009, D-011, D-012, D-015, D-017, D-020, D-023, D-024, D-029, D-030
- `docs/ARCHITECTURE.md`: Processes, Workspace layout and dependency rules, Platform layer (autostart row), Data and paths, Threading
- `docs/FEATURES.md`: Library, Portal, Settings, Tray and app lifecycle, Formats and contracts, Legacy problems not to carry over (non-atomic settings, restart loops)
- `docs/GPUI.md`: versions, thumbnail decoding, `drop_image`, backdrops, `--no-focus` and `PrintWindow` capture
- `docs/MEASUREMENTS.md`: table format for the UI-07 rows

### Legacy behavior
- `docs/legacy/portal.md` §1 (window, navigation, command-line arguments, header actions, status bar, backdrop and background image, tray hint, single instance), §2 (library load, showcase, scan, corrupt themes, sort, search, pagination, layout, card, hover), §4 General and Personal (the Phase 3 keys and their controls), §5 (Help, About, tray hint dialogs), "Porting notes"
- `docs/legacy/formats.md` §1.1 (settings mechanism, `Properties` wrapper, non-atomic writes), §1.3 (every file and key with defaults and ranges), §2 (data layout, Showcase, mutex names, startup command, 25-character folder names), §3 (`SucroseInfo.json`), §6 (enums), "Porting notes"
- `docs/legacy/services.md` §2 (Commandog `Startup`, `StartupP`, `Scheduler`), §3 (Launcher startup rule, tray menu, Close Applications/Exit), §7 (Undo: what uninstall removes)

### Project rules
- `.claude/rules/rust.md`, `.claude/rules/ui.md`, `.claude/rules/cargo.md`, `.claude/rules/platform-ffi.md`, `.claude/rules/docs.md`
- `CLAUDE.md`: non-negotiables, environment (`SUCROSE_DATA_DIR`, `WSLENV`, `--no-focus` GUI runs, stop running apps before rebuilding, real user data is read-only)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/sucrose-config/src/legacy.rs`: `LegacyFile`, `Properties` getters (`get_str`, `get_bool`, `get_i64`, `get_clamped`, `get_enum`), `read` with the locked-file retry, `LegacySettings::load`. Phase 2 built it for reuse by the FMT-01 migration; it covers only the M1 keys so far.
- `crates/sucrose-core/src/settings.rs`: the `legacy_enum!` / `legacy_index!` macros (names, ordinals, defaults). Extend them to every enum the migrated keys use (`formats.md` §6).
- `crates/sucrose-core/src/paths.rs`: `data_dir` (honours `SUCROSE_DATA_DIR`), `setting_dir`, `cache_dir`, and `legacy_data_dir` / `legacy_setting_dir` (ignore the override). Library and Showcase helpers are missing.
- `crates/sucrose-core/src/theme.rs` and `version.rs`: `SucroseInfo.json` types and the `AppVersion` comparison (D-029) for scan and LIB-08.
- `crates/sucrose-ipc`: `frame.rs`, `protocol.rs` (append-only `ToService` / `ToRenderer`, `PROTOCOL_VERSION = 3`, wire-index tests), `lib.rs` (local-socket bind, `AlreadyRunning`, owner-SID validation). The Portal messages extend this crate.
- `apps/sucrose/src/service.rs` (event loop, tray, renderer fleet, fullscreen rule), `supervisor.rs` (`RestartBudget`: 3 crashes per 60 s per monitor), `main.rs` (`parse_mode` refuses unknown arguments; keep that strictness when adding anything).
- `crates/sucrose-platform/src/os_windows/tray.rs` (tray menu), `event_loop.rs`, `job.rs` (kills renderers with the service).
- `apps/sucrose-portal/src/main.rs`: empty GPUI window with `--no-focus`, 900×600 minimum, `TitleBar`.
- `xtask/src/measure.rs`: per-process sampling and `MEASUREMENTS.md` rows (D-25).

### Established Patterns
- postcard enums are append-only; `PROTOCOL_VERSION` is bumped only for changes appending cannot express.
- `thiserror` in crates, `anyhow` in apps, `tracing` only; no async runtime in the service (D-012); GPUI executor in the Portal, never I/O or decoding on the UI thread.
- Dependencies through `[workspace.dependencies]`, `gpui-kit` pinned exactly; licenses checked by `cargo-deny`.
- CI builds Windows x86/x64/ARM64, Linux x86_64/aarch64 and macOS; Windows-only code keeps the other targets compiling, and OS gaps are reported through capabilities.
- `.planning/` files are LF; check each `docs/` file's line endings before editing and keep them.

### Integration Points
- The service reads `LegacySettings` directly today (Phase 2); it switches to the new settings (selected theme, volume, loop, stretch, hardware decoding, screen area, fullscreen action, `AppVisible`).
- `.planning/ROADMAP.md` criterion 2, `.planning/REQUIREMENTS.md` FMT-01 and LIB-01, and `docs/FEATURES.md` rows change with D-05 and D-06; behavior notes for D-09, D-17, D-19 and D-21 go to `docs/FEATURES.md`.
- New `docs/DECISIONS.md` entries are likely for the settings file location and format (D-01, D-02) and the service as the only settings writer (D-11).
- `docs/ARCHITECTURE.md` status line: `sucrose-config` gains the new format and the migration; `apps/sucrose-portal` stops being an empty window.

</code_context>

<specifics>
## Specific Ideas

- The maintainer's legacy settings (read-only):
  - `Library.json`: `Location` `H:\Sucrose` (108 themes), `DeleteCorrupt` false, `DeleteConfirm` true, `Move` true, `Selected` set.
  - `General.json`: `Culture` TR, `RunStartup` 3 (scheduled task), `AppExit` true, `AppVisible` true, `ThemeType` Dark.
  - `Warehouse.json`: all 20 showcase themes already listed; `HintTrayIcon` false.
  - `%APPDATA%\Sucrose` holds `Cache`, `Log`, `Setting` and `Showcase`; there is no default `Library` folder because the location is custom.
- With the real data, a first run therefore copies no showcase theme, deletes nothing, creates no startup entry (value 3 is not available yet) and opens on a dark theme. The behaviors those values skip are proven with fixture tests and the scratch `SUCROSE_DATA_DIR`.

</specifics>

<deferred>
## Deferred Ideas

- Re-importing the legacy settings on demand (considered for D-03, not chosen).
- Taking over or converting the C# startup entry (Run value or `Autorun for Sucrose` task) when the Rust installer replaces the C# app: decide with the Phase 5 installer migration (D-019).
- Search telemetry POST: with telemetry in Phase 5.
- Card context menu (Review, Customize, Find, Share, Delete, Edit) and the incompatible-theme Update action: LIB-05 and the updater in Phase 5.
- Header actions Create Wallpaper, Wallpaper Cycling, Display Preferences: in their phases.
- Applying the migrated `Culture`: UI-06 in Phase 5.

</deferred>

---

*Phase: 03-m2-service-ipc-and-portal-connected*
*Context gathered: 2026-11-23*
