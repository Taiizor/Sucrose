# Architecture (target)

> Status: **planned**, except the M0 skeleton: `sucrose-core` (paths), `sucrose-log`, `sucrose-platform` (trait and stubs), `apps/sucrose`, `apps/sucrose-portal` (empty window) and `xtask`. Update this file as code lands; when code and this file disagree, fix one of them in the same change.

## Goals

- Every feature of the C# app, on Windows, macOS and Linux (see `FEATURES.md` for what each OS allows).
- Lower memory and CPU than the C# app. The always-on part must be close to idle cost.
- No web runtime in the core. Engines are optional and user-installed.
- Existing themes, the store and web wallpapers keep working; settings are migrated automatically (D-007).

## Processes (D-008)

| Process | Lifetime | Responsibilities |
|---|---|---|
| `sucrose` (service) | Always on while a wallpaper is active | Core: settings, single instance, IPC server, starting and supervising renderers. Optional modules, each started only when the user enables it (D-008): tray, performance rules + system data, audio capture, media session, cycling, Discord, crash/telemetry upload, update checks. |
| `sucrose --render …` (renderer) | One per running wallpaper (per monitor or per span) | Hosts one engine on one desktop surface. Crashes are contained; the service restarts it. |
| `sucrose-portal` (UI) | Only while the window is open | Library, store, settings, theme creation/editing. Talks only to the service over IPC, never to engines directly. |

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
  sucrose-engine/       # Engine trait, engine registry, runtime detection of installed engines
  sucrose-engine-*/     # mpv, vlc, webview2, cef, wkwebview, webkitgtk, builtin (gif + OS video)
  sucrose-sysinfo/      # CPU/RAM/GPU/network/battery/audio spectrum/media session providers
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
- Only `sucrose-platform` and `sucrose-engine-*` may contain `unsafe` and OS-specific code (D-011). Other crates use `cfg` only to pick an implementation, never to implement one.
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

- Engines are discovered at runtime: installed libraries are detected and loaded with `libloading`; missing engines are reported to the UI with an install hint (D-005).
- All web engines expose the same JavaScript API (compatible with the C# one, see `FEATURES.md`).

## Data and paths

- Settings, cache, logs and showcase live in the same places as the C# app on Windows (`%APPDATA%\Sucrose\…`) and in the platform config directory elsewhere (`dirs::config_dir()/Sucrose`).
- Settings use a new versioned format with atomic writes (D-007); runtime state (pause/close flags, ports) is never stored in settings. Legacy `Setting/*.json` files are migrated on first run.
- The library location comes from the settings (legacy: `Setting/Library.json` → `Location`).
- File formats: see `FEATURES.md` → "Formats and contracts".

## Threading, errors, logging (D-012)

- Service: OS event loop on the main thread; IPC, network and decoding on worker threads; communication with `crossbeam-channel`.
- UI: GPUI executor (`cx.spawn`, `background_executor`). Never block the UI thread with I/O or decoding.
- Errors: `thiserror` in crates, `anyhow` in apps. No `unwrap()` on I/O, parsing or FFI results.
- Logging: `tracing`; daily rolling files under `Log/`; no `println!` outside `xtask`.