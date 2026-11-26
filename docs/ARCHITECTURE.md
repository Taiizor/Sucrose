# Architecture (target)

> Status: **planned**, except what M1 and M2 ship (Windows only for the platform code that has a Windows implementation): `sucrose-core` (paths, legacy paths, theme, version, setting enums, and the library scan: validity rules, sort, search index, theme-folder containment), `sucrose-log`, `sucrose-config` (the versioned `Setting/Sucrose.json` with lenient reading, atomic store, `.bak` restore and the key-by-key migration from the legacy `Setting/*.json` files, plus the legacy reader), `sucrose-ipc` (service and renderer messages, the Portal protocol, one local endpoint per data folder routed by the first message, single instance), `sucrose-platform` (Windows desktop surface, monitors, event loop, tray, job object, foreground window and foreground handoff, GPU preference and adapters, autostart entries for Windows, macOS and Linux, move to the recycle bin or trash, `window_backdrop` and `recycle_bin` capabilities, the window backdrop type (DWM system backdrop, `Platform::set_window_backdrop`); macOS and Linux stubs), `sucrose-engine` (the engine identities, the availability report, the pure per-type engine choice with its fallback order, and the clock follower shared by the time-based engines), `sucrose-engine-mpv` (bundled libmpv bindings, loader, player), `sucrose-engine-web` (pure: the virtual origin `https://sucrose.wallpaper.engine/` with its request-path resolver, `Range` math and MIME table; the JS API builders follow), `sucrose-sysinfo` (the system data providers; so far the Date group, sampled only on demand, and the first unsafe-allowed provider crate besides `sucrose-platform`, D-039), `sucrose-engine-webview2` (Windows only: the user-installed WebView2 Runtime behind the statically linked loader, D-013; a COM host on the surface thread whose pages come from the virtual origin through WebResourceRequested; stub elsewhere), `sucrose-engine-cef` (the pinned CEF built through `CEF_PATH` and `libcef.dll` delay-loaded, so only CEF hosts map it, D-32; it routes CEF subprocess command lines and hosts the windowed browser, whose pages come from the virtual origin through a scheme handler), `apps/sucrose` (service and renderer; the service adds the settings host, the Portal registry, the lifecycle rules, the library operations worker, the on-demand data hub and the optional modules behind one gate), `apps/sucrose-portal` (the connected Portal: library, settings, status bar, dialogs) and `xtask` (ci, fetch-engines for libmpv and CEF, bindgen-mpv, measure with a Portal mode, localize, synth-library). The `Engine` trait and its first two engines (mpv and CEF) have landed; the other engines and the rest are planned. Update this file as code lands; when code and this file disagree, fix one of them in the same change.

## Goals

- Every feature of the C# app, on Windows, macOS and Linux (see `FEATURES.md` for what each OS allows).
- Lower memory and CPU than the C# app. The always-on part must be close to idle cost.
- No web runtime in the core. Engine binaries ship inside the package (D-028).
- Existing themes, the store and web wallpapers keep working; settings are migrated automatically (D-007).

## Targets

Sucrose builds for these targets (D-027):

- Windows: `i686-pc-windows-msvc`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`.
- macOS: `aarch64-apple-darwin` and `x86_64-apple-darwin`, combined into one universal binary at packaging.
- Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`.
- CI (`.github/workflows/rust.yml`): clippy and the tests run natively on `windows-latest`, `windows-11-arm`, `ubuntu-latest`, `ubuntu-24.04-arm` and `macos-latest`; the `windows-latest` job also runs clippy and the tests for `i686-pc-windows-msvc`, and the `macos-latest` job runs `cargo check` for `x86_64-apple-darwin`.
- Which optional engines have a build for each architecture: `FEATURES.md` → Architectures.

## Processes (D-008)

| Process | Lifetime | Responsibilities |
|---|---|---|
| `sucrose` (service) | Always on while a wallpaper is active | Core: settings, single instance, IPC server, starting and supervising renderers. Optional modules, each started only when the user enables it (D-008): tray, performance rules + system data, audio capture, media session, cycling, Discord, crash/telemetry upload, update checks. |
| `sucrose --render …` (renderer) | One per running wallpaper (per monitor or per span) | Hosts one engine on one desktop surface. Crashes are contained; the service restarts it. |
| CEF subprocesses (`sucrose.exe --type=…`) | While a Web wallpaper plays | Started by CEF from a renderer (`browser_subprocess_path` is the same executable, or `sucrose-cef.exe` had the delay-load gate failed, D-036); `--type=` routes them to CEF before any Sucrose code. The renderer hosts CEF on its surface thread with an external message pump; the service never loads `libcef.dll`. The renderer also closes the browser itself: its `DoClose` answers 1 and the browser window is destroyed, because CEF's default close posts `WM_CLOSE` to the browser's top-level ancestor, which under the desktop surface is the shell's Progman (the Shut Down Windows dialog). |
| `sucrose-portal` (UI) | Only while the window is open | Library, store, settings, theme creation/editing. Talks only to the service over IPC, never to engines directly. |

- The Portal is a peer on the one local endpoint of the service (D-033): its first message, `PortalHello`, routes it to the Portal registry instead of the renderer path. A second Portal gets `AlreadyOpen` (and a foreground grant) and exits before it opens a window. The endpoint name carries a hash of `SUCROSE_DATA_DIR` when that variable is set, so a scratch-folder service never meets the real one.
- When no service runs, the Portal starts it with `sucrose --no-portal` (the service then never starts a Portal itself) and reconnects with backoff if the link drops.
- Service lifetime (D-13): the service starts by the legacy Launcher rule (Portal, tray and wallpaper from the stored settings) and exits by itself when the tray is hidden and nothing needs it: no Portal registered, no renderer running, and no library operation under way. A second `sucrose` launch asks the running service to open the Portal and exits with code 0.
- Optional modules (tray, the 1 s memory report to a connected Portal, the fullscreen rule, the data hub) go through one start gate and do not exist while their setting or the Portal's presence does not call for them (D-008). The data hub (`apps/sucrose/src/data_hub.rs`, D-24) exists only while at least one running web page needs system data: a renderer sends `DataWanted` with the groups its `SucroseCompatible.json` names once the page loaded, the hub ticks every 100 ms, reads each wanted group once (at the legacy rate of that group) through `sucrose-sysinfo`, and offers `Data` snapshots to each renderer through its bounded writer queue, which drops a snapshot instead of blocking; it ends with the last renderer that wanted data and never reads the `PerformanceCounter` setting.

## Workspace layout

```
Cargo.toml              # [workspace], [workspace.dependencies] (central versions), [workspace.lints]
rust-toolchain.toml
deny.toml               # cargo-deny: licenses (GPL-3.0 compatible), advisories
crates/
  sucrose-core/         # domain types and formats: themes (SucroseInfo/Properties/Compatible), wallpaper types, paths; no OS code
  sucrose-log/          # logging setup shared by the binaries
  sucrose-config/       # settings (new versioned format) + migration from legacy Setting/*.json
  sucrose-ipc/          # messages + local-socket transport + single instance
  sucrose-platform/     # Platform trait + windows/, macos/, linux-x11/, linux-wayland/
  sucrose-engine/       # Engine identities, availability report, per-type engine choice (choose_engine), shared clock follower, the Engine/EngineClock/SurfacePump traits
  sucrose-engine-web/   # Pure web core: virtual origin resolver (SEC-02), Range math, MIME table, profile rule, Chromium switches model (D-17), the legacy-exact JavaScript API strings (`js`: property and compatible calls, filedropdown lists, in-page pause); no OS code; sucrose-ipc carries its Switch type
  sucrose-engine-*/     # mpv, vlc, webview2, cef, wkwebview, webkitgtk, builtin (gif + OS video)
  sucrose-sysinfo/      # CPU/RAM/GPU/network/battery/audio spectrum/media session providers (unsafe allowed, per-OS cfg inside, D-039); exists: the Date group
  sucrose-store/        # store protocol, cache, downloads
  sucrose-update/       # update check, download, signature + hash verification (D-019)
apps/
  sucrose/              # service + renderer binary
  sucrose-portal/       # GPUI UI, built from scratch (exp/GPUI/Sucrose.Portal is not its base)
  sucrose-setup/        # installer and uninstaller (D-019)
xtask/                  # build, package, localize, release
docs/
src/                    # LEGACY C# reference (read-only, see src/CLAUDE.md)
exp/                    # experiments, not part of the product
```

### Dependency rules

- `sucrose-core` depends on nothing Sucrose-specific and contains no OS-specific code.
- Only `sucrose-platform`, `sucrose-sysinfo` (D-039) and `sucrose-engine-*` may contain `unsafe` and OS-specific code (D-011). Other crates use `cfg` only to pick an implementation, never to implement one.
- Apps depend on crates; crates never depend on apps.
- The UI never links engine crates.

## Platform layer

One trait, one implementation per OS (sketch, not final):

```rust
pub trait Platform {
    /// What this OS/session can do; the UI hides or explains anything unsupported.
    fn capabilities(&self) -> Capabilities;
    fn monitors(&self) -> Vec<Monitor>;
    /// A native surface behind the desktop icons on one monitor (or spanning several).
    fn create_desktop_surface(&self, target: SurfaceTarget) -> Result<DesktopSurface>;
    /// Fullscreen app, maximized window, desktop focused, locked, remote session, ...
    fn foreground_state(&self) -> ForegroundState;
    fn power_state(&self) -> PowerState;
    fn events(&self) -> Receiver<PlatformEvent>; // monitors changed, shell restarted, theme changed, ...
}
```

Known per-OS mechanisms:

| | Windows | macOS | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Desktop surface | Progman/WorkerW (handle the Windows 11 24H2 hierarchy) | window at desktop level | `_NET_WM_WINDOW_TYPE_DESKTOP` | wlr-layer-shell background layer; KDE/GNOME may need a DE plugin/extension |
| Fullscreen detection | shell notification state + foreground window | CGWindowList | EWMH | compositor-specific |
| Audio loopback | WASAPI loopback | ScreenCaptureKit / Core Audio taps (permission) | PipeWire/PulseAudio monitor | same |
| Media session | WinRT GlobalSystemMediaTransportControls | no public API (limited) | MPRIS (D-Bus) | MPRIS |
| Autostart | registry Run / Task Scheduler | LaunchAgent | XDG autostart | XDG autostart |
| GPU preference | registry UserGpuPreferences per exe | not available (capability false) | not available | not available |

## Engines

```rust
pub trait Engine {
    fn id(&self) -> EngineId;
    fn supports(&self, kind: WallpaperType) -> bool;
    fn open(&mut self, surface: &DesktopSurface, theme: &Theme, settings: &EngineSettings) -> Result<()>;
    fn set_paused(&mut self, paused: bool);
    fn set_volume(&mut self, volume: f32);
    fn apply_property(&mut self, key: &str, value: &PropertyValue);
    fn push_data(&mut self, data: &WallpaperData); // audio, system, media for web wallpapers
}
```

- Engine binaries (libmpv now; libVLC and CEF later) ship inside the package and are loaded at runtime with `libloading` from next to the executable (D-028, D-013); a missing or unloadable engine is reported to the UI with a repair hint. The WebView2 Runtime and VC++ Redist are user-side prerequisites detected with download hints (ENG-06).
- `sucrose-engine` decides which engine plays a wallpaper and never touches an OS: `EngineId` names the seven engines (mpv, libVLC, CEF, WebView2, the built-in GIF renderer, the built-in video player, the application embedder), `Availability` is the report of what the machine can run (installed, missing with a repair hint, not loadable, unsupported), and `choose_engine(type, configured, availability)` is a pure function that returns a `Choice`. The configured engine wins when it is installed. Otherwise the fallback order is: Web, URL and YouTube try the other web engine; GIF tries the built-in GIF renderer, mpv, libVLC, then a web engine playing a generated page; Video tries mpv, the built-in video player, libVLC, then a web engine; Application tries the embedder only and is never faked by another engine (D-11). The `Choice` names the engine it fell back from and why, has no engine when none is available, and never changes the setting: a fallback applies to the current run only.
- The renderer plays through one seam, the `Engine` trait in `sucrose-engine` (ENG-03). The service chooses the engine with `choose_engine` and puts its legacy name into the `Assignment`; the renderer maps it with `EngineId::from_legacy` and starts it, or refuses an engine its build does not contain. An `Engine` is a thread-safe handle called from the IPC reader and the clock thread (pause with the migrated `PausePerformanceType` as its level, volume, loop, stretch, audio, `load_source`, and an optional `EngineClock` for engines that play on a timeline). An engine bound to the surface thread also returns a `SurfacePump`, which the renderer's message loop drives on `LoopEvent::Wake` and its timers: CEF queues commands from the IPC thread through a bounded channel and the pump drains them on the surface thread, so no CEF function runs on another thread. mpv follows its parent window by itself and has no pump. No pause path suspends a process or attaches a debugger (PLAY-05).
- Web pages are served from one virtual origin, `https://sucrose.wallpaper.engine/` (D-14): every theme shares it, no local port is opened and nothing is served from `file://`. `sucrose-engine-web::origin::resolve` is the one function that maps a request path to a file: query and fragment dropped, the path decoded once, NUL, backslash, colon, leading `//`, dot segments and double encoding refused, then the theme folder first and `<cache>/Content` second with canonical containment (a junction or link that leads out is refused). It holds no state and opens no file, so concurrent requests resolve independently. `sucrose-engine-web::serve::plan` turns a request (URL and `Range`) into the one reply both web engines stream: status, headers and a file range, with the refusals logged (a single `Range` is a 206, a range past the end a 416). CEF puts a scheme handler for that host only on it (`sucrose-engine-cef::scheme`); every other https address keeps CEF's own networking. WebView2 (`sucrose-engine-webview2`, the user-installed Runtime behind the static loader) puts a WebResourceRequested handler with a filter for that host on the same planner and decides nothing itself: both web adapters stream `serve::plan`. A WebView2 range is read into a memory stream of at most `serve::MAX_RANGE_BODY` (4 MiB) with the `Content-Range` of the bytes it carries (`serve::range_body`; CEF streams a range from the file and needs no cap), a whole file streams from disk. Both engines take the same `SurfaceHooks` from `sucrose-engine` and run on the surface thread behind `SurfacePump`.
- The settings keep the legacy engine names (D-10): `VexanaLive` and `XavierLive` both mean the built-in GIF renderer, `NebulaLive` the built-in video player, `AuroraLive` the application embedder, `WebViewLive` WebView2, `CefSharpLive` CEF, `MpvPlayerLive` mpv and `VlcPlayerLive` libVLC. An engine a type does not allow (a hand-edited file) counts as the type's default before the choice. The `Engine` trait above lands with its first two engines in plan 05 of Phase 4.
- All web engines expose the same JavaScript API (compatible with the C# one, see `FEATURES.md`).

## Data and paths

- Settings, cache, logs and showcase live in the same places as the C# app on Windows (`%APPDATA%\Sucrose\…`) and in the platform config directory elsewhere (`dirs::config_dir()/Sucrose`).
- Settings use a new versioned format with atomic writes (D-007): `Setting/Sucrose.json` with a `Setting/Sucrose.json.bak` restore copy (a corrupt file the store is about to replace is first kept as `Setting/Sucrose.json.corrupt`), written only by the service (the Portal sends `SettingChange` messages and shows the value the service broadcasts back, D-032). Runtime state (pause/close flags, ports) is never stored in settings. The nine legacy `Setting/*.json` files that carry settings are migrated key by key on first run, read-only (the C# files are never changed).
- The library location comes from the settings (legacy: `Setting/Library.json` → `Location`).
- File formats: see `FEATURES.md` → "Formats and contracts".

## Threading, errors, logging (D-012)

- Service: OS event loop on the main thread; IPC, network and decoding on worker threads; communication with `crossbeam-channel`. Each renderer connection has a bounded outgoing queue drained by a writer thread of its own, so a renderer that stops reading never blocks the main thread or the other renderers (a clock reading is dropped when the queue is full, any other message is a failure).
- UI: GPUI executor (`cx.spawn`, `background_executor`). Never block the UI thread with I/O or decoding.
- Errors: `thiserror` in crates, `anyhow` in apps. No `unwrap()` on I/O, parsing or FFI results.
- Logging: `tracing`; daily rolling files under `Log/`; no `println!` outside `xtask`.