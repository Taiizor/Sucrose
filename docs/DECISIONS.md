# Decisions

Architecture decision log for the Rust rewrite. Newest last. Every entry has a status:

- **Accepted**: decided by the maintainer. Do not re-open without a new entry that supersedes it.
- **Proposed**: recommended and being followed, but not explicitly confirmed yet. Confirm or replace it before building something that would be expensive to undo.
- **Superseded by D-xxx**: kept for history.

When you make or change a decision, add an entry here in the same change.

---

## D-001: Rewrite Sucrose from scratch in Rust
- Status: Accepted (2026-11-21)
- Decision: Leave C#/.NET entirely. The new implementation is Rust. C/C++ only through FFI where a native library has no Rust equivalent (libmpv, libVLC, CEF, WebView2 COM).
- Why: cross-platform goal, lower memory/CPU for an always-on app, single native binaries without a runtime install.
- Consequences: the C# app in `src/` becomes a read-only reference until feature parity (see D-016).

## D-002: Windows, macOS and Linux
- Status: Accepted (2026-11-21)
- Decision: Ship on all three. Feature parity order: Windows, then macOS, then Linux. The architecture is cross-platform from the first commit and CI builds all three from day one.
- Why: maintainer requirement. Windows first because the existing users and the reference implementation are there.
- Consequences: features the OS cannot support (for example embedding another app's window on Wayland or macOS) are reported through platform capabilities and hidden or explained in the UI, never faked. See `FEATURES.md`.

## D-003: Keep every feature, improve where possible
- Status: Accepted (2026-11-21)
- Decision: Every feature of the C# app is ported (tracked in `FEATURES.md`). Where a feature can be done better, do it better, but record the behavior change.

## D-004: UI is GPUI through gpui-kit
- Status: Accepted (2026-11-21)
- Decision: Portal and every other window use GPUI via `gpui-kit` (Longbridge), pinned to an exact version. Sucrose gets its own design language that looks the same on every platform. Windows uses Mica/Acrylic backdrops where available.
- Rejected: Tauri or any webview as the application core (the UI must work with no web runtime installed); Slint (maintainer did not like it); WinUI/Fluent-only look (Windows-only).
- Context: Discord announced a Rust + GPUI desktop client on 2026-11-04. A working prototype exists in `exp/GPUI/Sucrose.Portal` (see `GPUI.md`).

## D-005: Engines are optional and installed by the user
- Status: Superseded by D-028 (2026-11-22)
- Decision: WebView2, CEF, libmpv and libVLC are never installed or bundled by Sucrose, as today. Sucrose detects them and lets the user install them.
- Consequences: engine libraries are loaded at runtime (`libloading`), never linked at build time (D-013).

## D-006: Built-in engines
- Status: Accepted (2026-11-22)
- Decision: Sucrose works with no engine installed. GIF/APNG/WebP are rendered by Sucrose itself; video uses the OS media stack (Media Foundation on Windows, AVFoundation on macOS, GStreamer on Linux). These replace Nebula, Vexana and Xavier.
- Caveat: codec support depends on the OS. Windows plays H.264 everywhere, HEVC/VP9/AV1 need Store extensions; macOS plays H.264/HEVC/ProRes but usually not WebM/VP9; Linux depends on the installed GStreamer plugins. When the built-in engine cannot play a file, Sucrose says so and suggests the bundled mpv engine (D-028).

## D-007: Compatibility: keep what is good, migrate what is improved
- Status: Accepted (2026-11-22), changed from the proposal
- Principle: keep a legacy format unless a new one is clearly better; when it is, migrate automatically.
- Themes (`SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json`, archives) and the web wallpaper JavaScript API stay compatible (the store has 3,230 themes). New fields are optional only. The legacy JS API is kept; an improved v2 API may be added next to it.
- Settings move to a new, versioned format with atomic writes and with runtime state kept out of settings. The legacy `Setting/*.json` files are migrated automatically on first run.
- The store protocol follows D-022.
- Consequences: compatibility is proven with fixture tests from real files; any other break needs its own entry and a migration.

## D-008: Process model: modular service
- Status: Accepted (2026-11-22), changed from the proposal
- Decision: `sucrose` service + one renderer process per running wallpaper (crash isolation; same binary in renderer mode) + `sucrose-portal` UI only while open. Every optional subsystem inside the service is a module that **does not start at all when the user turns it off** (no thread, no polling, no allocations): tray, performance rules and system data, audio capture, media session, Discord, crash and telemetry upload, wallpaper cycling, update checks.
- Why: the legacy split into ~17 executables let users save resources by not running parts (tray, Reportdog, Backgroundog). Modules keep that benefit without a process per feature.

## D-009: One IPC mechanism
- Status: Accepted (2026-11-22)
- Decision: Local sockets via `interprocess` (named pipes on Windows, Unix domain sockets elsewhere) with binary messages serialized by `serde` + `postcard`, chosen for speed. The same endpoint provides single-instance behavior.
- Replaces: Sucrose.Pipe, Sucrose.Signal (file watcher), Sucrose.Transmission (TCP), and the Commandog command trampoline.

## D-010: Port capabilities, not packages
- Status: Accepted (2026-11-22)
- Decision: Third-party NuGet packages are replaced by crates or disappear. Of the maintainer's own packages (Skylark family, Playhead) only the parts Sucrose uses move into Sucrose crates, rewritten for all three OSes. Mapping in `PORTING.md`.

## D-011: unsafe is isolated
- Status: Accepted (2026-11-22)
- Decision: `unsafe_code = "deny"` workspace-wide, allowed only in `sucrose-platform` and the engine/FFI crates. Every `unsafe` block has a `// SAFETY:` comment.

## D-012: Threading, errors and logging
- Status: Accepted (2026-11-22)
- Decision: No global async runtime in the service: the OS event loop runs on the main thread, IPC/network/decoding run on worker threads talking through `crossbeam-channel`. The UI uses GPUI's executor. Errors: `thiserror` in libraries, `anyhow` in binaries. Logging: `tracing` with daily rolling files.

## D-013: Native library bindings
- Status: Accepted (2026-11-22)
- Decision: Generate our own bindings with `bindgen` in dynamic-loading mode and load with `libloading` (libmpv, libVLC). Use `webview2-com` for WebView2 and the `cef` crate for CEF.
- Avoid: `vlc-rs` (last release 2018), `libmpv-sys` (2020), `single-instance` (2021), `libmpv2` (links libmpv at build time, conflicts with D-005).

## D-014: Build automation is `cargo xtask`
- Status: Accepted (2026-11-22)
- Decision: Packaging, signing, localization conversion and other automation live in an `xtask` crate, so they run the same from PowerShell, WSL, macOS and Linux. No new PowerShell or shell scripts.

## D-015: Localization is rust-i18n
- Status: Accepted (2026-11-22)
- Decision: `rust-i18n` (already used by gpui-kit) with YAML/JSON files. The 25 existing languages are converted from the XAML dictionaries by `xtask`. The POEditor workflow continues through YAML/JSON import/export.

## D-016: The C# code is reference only
- Status: Accepted (2026-11-22)
- Decision: On `develop-rust`, `src/` (C#) is not modified unless the maintainer asks. It is removed only after parity is signed off (M7), and then this documentation and `CLAUDE.md` are rewritten for the finished project.

## D-017: Secure by default
- Status: Accepted (2026-11-22), narrowed by D-021
- Decision: TLS certificate validation stays on (no SSL2/SSL3); installers and updates are verified before they run (D-019); local servers (if any) reject paths outside the theme folder; crash reports are not written when reporting is off.
- Out of scope: browser engine switches, which stay user-controlled (D-021).

## D-018: Telemetry and crash reporting keep the legacy structure
- Status: Accepted (2026-11-22)
- Decision: Keep the existing identity derivation and payload structure so server-side statistics continue. Define the per-OS equivalents of the Windows identity inputs (Windows: user name, model, SID; macOS: user name, hardware model, IOPlatformUUID; Linux: user name, DMI product name, `/etc/machine-id`). Opt-out settings and defaults stay as they are (D-024).

## D-019: Installer, updater and uninstaller are our own Rust tools
- Status: Accepted (2026-11-22)
- Decision: Bundle, Update and Undo are rewritten in Rust (Velopack was considered and rejected: the maintainer prefers full control, and its packaging CLI needs the .NET SDK).
- Requirements: per-OS packages (Windows first); update packages signed and verified (signature + hash) before anything is installed; resumable downloads with the bandwidth limit; Release/PreRelease channels; the existing silent modes; uninstall that really works quietly and removes startup entries and GPU-preference registry values (legacy bugs); migration from an existing C# installation.

## D-020: GitHub integrations are removed
- Status: Accepted (2026-11-22), extended the same day to the theme report template
- Decision: The GitHub store server, the GitHub update source, the personal access token (`Objectionable.json`) and the theme report GitHub issue template ("Report from GitHub") are dropped. Store, updates and theme reports use the Soferity API only. Plain links, such as the wiki and issue links in the help menu, stay.
- Context: the API server is currently .NET and will itself be rewritten in Rust or Go (separate project). Its GitHub-compatible release output is no longer needed.

## D-021: Browser engine arguments are user-editable
- Status: Accepted (2026-11-22)
- Decision: Keep all legacy Chromium switches as defaults for the web engines. The UI lets the user add, remove and reset them (the legacy `CefArguments`/`WebArguments` settings existed but had no UI).

## D-022: Store and update protocol
- Status: Deferred (2026-11-22)
- Decision: Decide together with the API rewrite. Until then the Rust client talks to the existing v8 endpoints.

## D-023: Donate and advertising are removed
- Status: Accepted (2026-11-22)
- Decision: The hidden Donate page and its settings (`Donate.json`) are not ported or migrated.

## D-024: Defaults stay as they are
- Status: Accepted (2026-11-22)
- Decision: Legacy defaults are kept, including `StoreAdult = true`, `TelemetryData = true` and `ExceptionData = true`.

## D-025: Code signing
- Status: Accepted (2026-11-22) for Windows; macOS pending
- Decision: Windows binaries and installers are signed with signtool using the maintainer's existing certificate, as the C# app is today. The certificate and its password stay outside the repository; `xtask` reads their location from environment variables and never logs them. macOS has no Apple Developer ID yet, so macOS builds are unsigned until one is available.
- Update packages additionally get a signature from our own key (D-019); the key scheme (proposed: Ed25519) is decided when the updater is built.

## D-026: Lock screen wallpaper is in scope
- Status: Accepted (2026-11-22)
- Decision: The lock screen wallpaper for Video and GIF themes is part of parity although it exists only on branch `lock-screen-native` (commit 112de37f) and was never merged into `develop`. Its behavior is recorded in `docs/legacy/lock-screen.md`.
- Consequences: how the Rust version reaches the lock screen is decided separately before it is built; the legacy one runs a DLL inside LogonUI as SYSTEM, registered from a user-writable folder.

## D-027: Target architectures
- Status: Accepted (2026-11-22)
- Decision: Windows x86 (i686), x64 and ARM64; macOS arm64 and x86_64, shipped as one universal binary; Linux x86_64 and aarch64.
- Why: keep every Windows architecture the legacy app shipped (x86, x64 and ARM64; D-003) and add ARM64 Linux and both macOS architectures.
- Consequences: CI builds and tests Windows ARM64 and Linux ARM64 natively, builds and tests the Windows x86 (i686) target on the Windows x64 runner, and checks the macOS x86_64 target (FND-02); packages are built per architecture (SVC-04, MAC-05, LNX-06); per-architecture engine availability is tracked in `FEATURES.md` → Architectures; an engine with no build for the running architecture is reported like a missing engine (D-005).

## D-028: Engine binaries ship inside the Sucrose package
- Status: Accepted (2026-11-22); supersedes D-005
- Decision: engine binaries (libmpv now; libVLC and CEF in Phase 4) ship inside the Sucrose package, as they did in the C# app (which bundled `libmpv-x86/x64/ARM64.dll`, `VideoLAN.LibVLC.Windows` and CefSharp's `libcef.dll`). `cargo xtask fetch-engines` downloads the versions pinned in `xtask/engines.toml`, verifies the SHA256 before extraction, places them next to the built exe for development runs and, from Phase 5, into the packages. Binaries are never fetched at runtime and never committed. The WebView2 Runtime and the VC++ Redist stay user-side prerequisites that are detected and get download hints (ENG-06). libmpv is still loaded at runtime with `libloading` through our own `bindgen` bindings (D-013); only renderer processes load it; it is searched next to `sucrose.exe`, then on absolute `PATH` entries, never in the C# install. The `PATH` step narrows D-18: only absolute `PATH` entries are searched; relative and empty entries and the current folder are skipped against DLL planting.
- Pin: libmpv comes from shinchiro/mpv-winbuild-cmake release 20261111 (mpv v0.42.0-38-gbcde6b93c, client API 2.6, built with `gpl`), one `mpv-dev-<arch>-20261111-git-bcde6b93cc.7z` per architecture (x86_64, i686, aarch64) pinned by SHA256 and size in `xtask/engines.toml`. `libmpv-2.dll` is 99,536,384 B (x86), 121,789,952 B (x64) and 84,073,984 B (ARM64). The build is GPL-2.0-or-later (mpv `-Dgpl`, FFmpeg `--enable-gpl`); the maintainer signed off on shipping it inside the GPL-3.0-only package (2026-11-22). The archives contain no license file, so Phase 5 packaging adds the license texts and a source offer pointing at the recipe tag 20261111 and commit `bcde6b93cc`. The LGPL builds (zhongfly) have no i686 build and would drop Windows x86 (D-027), so they are not used. The pinned URLs stay on the upstream release (maintainer decision 2026-11-22: no mirror yet); upstream keeps about 30 releases, so mirroring is an open question below. `libmpv-2.dll` imports `vulkan-1.dll` as a normal (not delay-load) import (objdump: normal import table, empty Delay Import Directory), so a PC without the Vulkan loader gets `LoadError::LoadFailed`, the D-20 "cannot be loaded" path, instead of a working libmpv.
- Why: parity with the C# package and wallpapers that work right after install.
- Consequences: package size grows by about 100-120 MB per architecture for libmpv; ENG-01 and ENG-06 wording changes (they no longer say engines are never bundled); Phase 5 adds license texts and the source offer; macOS and Linux packaging of engines is decided in Phases 6-7.

## D-029: Version scheme continues the legacy date
- Status: Accepted (2026-11-22)
- Decision: the workspace version (`[workspace.package] version`) is the build date as `YY.M.D+R`, with month and day written without zero padding (Cargo semver forbids leading zeros) and the build metadata `+R` as the same-day build number, starting at 1. The first value is `26.11.22+1`. It maps one-to-one onto the four-part legacy version `YY.M.D.R` (`26.11.22.1`): `LegacyVersion::from_cargo` reads `+R` as the fourth component and uses 0 when it is absent. Same-day rebuilds that are published bump R (+2, +3); a new day resets R to 1. `26.11.22.1` is greater than the latest C# release v26.11.22.0, which the maintainer chose on 2026-11-22 over the plain date form `26.11.23` (one release per day, and today's date would have been unusable).
- Why: the `AppVersion` <= app version check on themes, store filtering and the Phase 5 update from the C# app all compare against the legacy date version (D-012); with the M0 version 0.1.0 every theme would be refused.
- Consequences: one-way, because published versions cannot go back down (the updater and the store filter compare against them); release tooling bumps it in Phase 5. Cargo ignores build metadata for precedence, which does not matter for `publish = false` crates. Assumption A16 was tested: `CARGO_PKG_VERSION` keeps the `+R` suffix (the `cargo_version_text_keeps_the_build_metadata` test pins it), so `sucrose_core::VERSION` and `version::app_version()` read it directly and no Cargo.toml parsing at build time is needed. A theme whose `AppVersion` is newer than `app_version()` is refused with a "needs a newer app" reason (D-13); an equal version is accepted.

## D-030: Master clock election for multi-monitor sync
- Status: Accepted (2026-11-23), maintainer choice at the plan 02-19 checkpoint
- Decision: a renderer counts only once its first frame shows (it reports Playing, publishes a clock and follows a clock only from then on); the first playing renderer, primary monitor first at start, is the master clock and keeps the role while its monitor exists and it plays; when its monitor goes away or it stops playing, the primary monitor's playing renderer, else the lowest device name, takes over.
- Why: a plug or a primary-monitor change must not restart or jump playback on the monitors that already play (UAT gap G-02-2c): the re-enabled virtual display came back as primary, its brand-new renderer was elected master and published position 0.0002 as a running clock, and the renderer that had played for 8.5 min seeked back to the start.
- Deviation: departs from the Phase 2 context rule (02-CONTEXT, Claude's discretion: "the primary monitor, with another renderer promoted when the master's monitor goes away"): after a primary-monitor change the master stays on the renderer that already plays, which can be on a non-primary monitor, until its monitor goes away or it stops playing. The other half of the rule stays: another renderer is promoted when the master's monitor goes away.
- Consequences: the audio is played by the master's process; it goes to the same default output device whichever renderer plays it, and the legacy app has no master clock, so no legacy behavior changes. No protocol change; the policy is the pure `choose_master` in `apps/sucrose/src/service.rs`.

## D-031: Wait for explorer when a display change leaves a monitor outside its desktop window
- Status: Accepted (2026-11-23), maintainer choice at the plan 02-26 checkpoint (verbatim reply "wait")
- Decision: after a display change each surface stays in explorer's desktop window (Progman on the raised desktop) and is re-placed when explorer moves or resizes that window (location watch) and after every applied display change (re-place-all); while explorer leaves a monitor, or part of it, outside that window, that part shows the static Windows wallpaper and no icons; Sucrose does not switch to a fallback window there.
- Why: a spike on the maintainer's machine with the release build of 02-23..02-25 made 4 display changes (two primary switches and two origin-only changes); the location watch caught 8 of 8 moves of the window, every outside episode lasted 1.73-2.11 s and ended with the surface shown, so the run was classified "explorer keeps up". The episode of about 90 s seen in UAT test 7(c), after Windows applied two layouts in a row, did not recur (Windows applied one layout per change in the spike), so its cause is neither confirmed nor refuted.
- Rejected: the fallback window (while explorer's window does not reach a monitor, the surface leaves Progman for a bottom-most, non-activating tool window on that monitor, and its live video child is reparented twice per episode): it is a new surface mode for an episode that lasted about 2 s in the spike, and the legacy app parents its engine windows into the same shell windows (docs/legacy/engines.md, section 2), so a fallback would be new behavior, not parity.
- Consequences: UAT gap G-02-7 is an accepted limitation and stays open (not closed); the length of an episode is explorer's, not bounded by the app; the surface logs each episode (a warning when it starts, an info line when it ends) so a long one can be diagnosed from the logs; if a long episode recurs, a new entry that supersedes this one reopens the choice; recorded in docs/FEATURES.md, row "Monitor changes, shell restart recovery (CrashExplorer)".

## D-032: Settings live in Setting/Sucrose.json, written atomically, with the service as the only writer
- Status: Accepted (2026-11-24)
- Decision: the settings are one versioned JSON file, `Setting/Sucrose.json`, in the legacy `Setting` folder next to the 14 legacy files (phase 3 D-01). It is a one-way door: the Phase 5 installer and the updater read it at that path, and the exact path was confirmed by the maintainer on 2026-11-24. The top-level `Version` (1 now) is followed by one section per legacy file (`Library`, `General`, `Engine`, `Backgroundog`, more as plans add keys), with the legacy key names, enums by their legacy names and the legacy defaults, so support and the settings pages map them one to one. Unknown keys and sections are kept on write, an invalid value reads as its legacy default, and a file with a newer `Version` is read but never overwritten. When neither `Sucrose.json` nor `Sucrose.json.bak` can be read, the settings are migrated once from the real legacy `Setting` folder (read only, ignoring `SUCROSE_DATA_DIR`; the legacy files are never written, D-03); afterwards the legacy files are not opened. A write goes to `Sucrose.json.<pid>.tmp` in the same folder, is flushed (`sync_all`) and renamed over the target (D-02); the file it replaces is first copied to `Sucrose.json.bak`, a name outside the C# Commandog Reset/Import/Export `*.json` glob, so a C# Reset that deletes `Sucrose.json` restores from the `.bak` instead of migrating again. Stale temporary files of a killed writer are removed at the next open. The service is the only reader and writer; the Portal gets a view of the values over IPC (D-11, implemented by plans 06 and 07). Runtime state (pause and close flags, ports, memory figures) is never stored in it (ARCH-04).
- Why: one atomic file instead of 14 files that are truncated in place and polled (the legacy design), and a first start that keeps the user's settings without touching the C# install.
- Consequences: a kill during a write leaves the old or the new file (kill test: 60 kills of a writer in a tight loop, `crates/sucrose-config/tests/store.rs`); `std::fs::rename` is not documented as atomic, so that test and the `.bak` are the contract (research assumption A1). A C# Import can still overwrite `Sucrose.json`; the `.bak` limits the damage. The maintainer's own runs create `%APPDATA%\Sucrose\Setting\Sucrose.json` by design; automated runs use a scratch `SUCROSE_DATA_DIR`.

## D-033: The Portal talks to the service over the one endpoint, routed by its first message
- Status: Accepted (2026-11-24)
- Decision: the service, its renderers and the Portal share the one local socket per user (D-009); there is no second endpoint and no named mutex. The first message of a connection decides the peer: `Hello` is a renderer, `PortalHello` is a Portal, `Launch` is a launcher (answered by plan 15); anything else, and anything that is not a valid frame, ends that connection only. One Portal is registered at a time, on the service's main thread: a second `PortalHello` gets `AlreadyOpen` with the first Portal's pid and the first is told `Focus` (the single-instance check of the Portal, D-12), and a Portal that connects after shutdown began gets `Close`. The Portal protocol (`sucrose_ipc::portal`) has its own `PORTAL_PROTOCOL_VERSION` (1), and its enums are append-only like the renderer protocol; `PortalHello` and `Launch` are appended to `ToService` (indexes 4 and 5) without a bump of `PROTOCOL_VERSION`. The Portal gets a view of the settings and the change events, never the file (D-11, finding F4); the largest possible `Welcome` (every text at `MAX_PATH_TEXT`) is far below `MAX_FRAME`. Each Portal has one bounded writer: a `Memory` event is dropped when its queue is full, any other overflow gives up that connection, so a stuck Portal never blocks the main thread and reconnects. With `SUCROSE_DATA_DIR` set, the endpoint name carries the FNV-1a hash of that folder (`Sucrose.Service.<user>.<16 hex digits>`), so a scratch-folder service never meets the user's own service (research open question 6); without it the name is unchanged. A Portal that finds no service starts the sibling `sucrose` with `--no-portal` (plan 08).
- Why: one endpoint is also the single-instance check, needs no extra OS object and works the same on every OS; deciding the registration on one thread makes two simultaneous Portals impossible by construction; a hash in the name keeps scratch runs and tests apart from the maintainer's service without putting a path into an OS-wide name.
- Consequences: every later settings page, library action and lifecycle plan builds on these message sets, so a variant's meaning cannot change after they land (append only). A Portal that claims another Portal's pid gains nothing: the connection, not the pid, identifies it, and every request is validated by the service (plans 07, 09, 21, 22, 23). The integration tests start the real `sucrose` binary on a scratch folder (`apps/sucrose/tests/common`).

## D-034: Sucrose's own start entry is "Sucrose.Service", separate from the C# entry
- Status: Accepted (2026-11-24)
- Decision: Normal startup writes Sucrose's own entry, named differently from the C# one so both can exist and neither is touched. Windows: the HKCU `Software\Microsoft\Windows\CurrentVersion\Run` value `Sucrose.Service`, whose data is the quoted path of `sucrose.exe` and no arguments (the service starts with no argument and the legacy Launcher rule resumes the selected wallpaper, D-12); enabling also deletes Sucrose's own value under `...\Explorer\StartupApproved\Run`, which resets Task Manager's off flag. macOS: the LaunchAgent `~/Library/LaunchAgents/dev.sucrose.Sucrose.plist` (label `dev.sucrose.Sucrose`). Linux: `$XDG_CONFIG_HOME/autostart/sucrose.desktop` (default `~/.config/autostart`). The entry is written only when the user turns Normal on in the Portal and removed when they choose None; migration copies RunStartup but writes nothing, and a stored Priority or Scheduler (the legacy Scheduler task is the maintainer's own value) creates no entry and is shown as not available yet (TRAY-04, Phase 5). The C# Run value `Sucrose` and the scheduled task `\Sucrose\Autorun for Sucrose` are never written, renamed or removed: they are only looked for (a registry read and a `schtasks /Query`, run when the General page opens and never polled) so the row can say that both apps will start together. The path in the entry is always the service's own `current_exe`, never text from IPC; the plist and the desktop file escape it (XML escaping; Desktop Entry `Exec` quoting with the string rule on top) and are tested on every OS.
- Why: a separate name lets the user keep the C# app's startup while trying Sucrose, and keeps an uninstall of either from removing the other's entry (D-08).
- Consequences: the Phase 5 updater and uninstaller must find and remove exactly these names; renaming them later means cleaning up the old names too, which is why they were confirmed by the maintainer on 2026-11-24 (03-CONTEXT.md D-08). The LaunchAgent and XDG formats are checked only as text and file locations in CI; they need a real Mac and a Linux desktop for a sign-in test.

---

## Open questions

- Store and update protocol: see D-022.
- Linux package formats for the installer (AppImage, deb, rpm, Flatpak): decide at M6.
- Apple Developer ID for macOS signing and notarization: when available.
- Lock screen mechanism (D-026): before the M4 lock screen item.
- Mirror the pinned libmpv archives (`xtask/engines.toml`) on a Sucrose-controlled release before Phase 5 packaging, because upstream prunes old releases (D-028).
