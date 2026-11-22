# Porting guide

How to move a capability from the C# app (`src/`, read-only) to the Rust workspace. The principle is D-010: **port what Sucrose uses, not whole packages.**

## Procedure for one feature

1. **Find it in C#.** `src/CLAUDE.md` explains the layout and the using-alias convention (`SMMG` = `Sucrose.Manager.Manage.General`, …). Resolve aliases at the top of each file before reading the code.
2. **Write down the behavior first.** Update the feature's entry in `FEATURES.md`: what it does, settings keys and defaults, file formats, edge cases, Windows APIs used. This is the spec the Rust code is checked against.
3. **Place it.** Decide the crate (see `ARCHITECTURE.md`) and split OS-specific parts into `sucrose-platform` behind a capability.
4. **Implement with tests.** Formats and rules get unit tests; compatibility uses fixtures copied from real files (strip personal data).
5. **Record it.** Mark the row in `FEATURES.md`, note any behavior change or improvement, add a `DECISIONS.md` entry if a choice was made, and record notable RAM/CPU numbers in `MEASUREMENTS.md`. Requirement and phase status in `.planning/` is updated by the GSD workflow.

## NuGet packages

| Package | Used by (C#) | Rust replacement |
|---|---|---|
| Newtonsoft.Json | Manager, Pipe, Signal, Transmission | `serde` + `serde_json` |
| CsvHelper | Localizer | `csv` (if still needed by `xtask`) |
| SharpCompress | Bundle, Update | `zip`, `sevenz-rust2` |
| Downloader | Update | own updater (`sucrose-update`, D-019) on `ureq`, resumable, bandwidth limit |
| DiscordRichPresence | Launcher | `discord-rich-presence` |
| NAudio | Backgroundog (audio capture) | `wasapi` (Windows loopback) or `cpal`; PipeWire on Linux |
| MathNet.Numerics | Backgroundog (FFT) | `realfft` |
| Vortice.DXGI | Backgroundog (GPU info) | `windows` (DXGI), `nvml-wrapper` |
| RawInput.Sharp | Aurora, CefSharp, WebView engines | `windows` Raw Input, inside `sucrose-platform` |
| TaskScheduler | Commandog (startup via scheduler) | `windows` Task Scheduler COM; `auto-launch` for the simple modes |
| Watson.Lite | CefSharp engine (local web server) | not needed if CEF serves files through a custom scheme; else `tiny_http` |
| CefSharp.Wpf.HwndHost | CefSharp engine | `cef` (same CEF 156 line) |
| Microsoft.Web.WebView2 | WebView engine | `webview2-com` |
| LibVLCSharp.WPF, VideoLAN.LibVLC.Windows | VlcPlayer engine | own `bindgen` bindings + `libloading` (D-013) |
| WPF-UI, WPF-UI.DependencyInjection, HandyControls, XamlFlair.WPF, CommunityToolkit.Mvvm, Microsoft.Extensions.Hosting | Portal, Property, Update | gone: GPUI + gpui-kit |
| Costura.Fody | Bundle | gone: Rust produces single binaries |
| WindowsAPICodePackShell | Portal (file/folder dialogs) | `rfd` |
| Skylark.Clipboard (own) | Portal | `arboard` |
| Playhead (own) | Backgroundog (now-playing media session) | Windows: WinRT media sessions via `windows`; Linux: MPRIS (`zbus`/`mpris`); macOS: no public API |
| Skylark, Skylark.Standard, Skylark.Wing (own) | almost everything | only the used parts, see below |

## In-repo libraries

| C# | Rust |
|---|---|
| Sucrose.Manager (settings) | `sucrose-config` |
| Sucrose.Memory (constants, paths, URLs) | `sucrose-core` (paths/constants modules) |
| Sucrose.Pipe, Sucrose.Signal, Sucrose.Transmission | `sucrose-ipc` (D-009) |
| Sucrose.Resources (locales, images) | `locales/` (rust-i18n) + assets, converted by `xtask` |
| Sucrose.Mpv.NET | `sucrose-engine-mpv` |
| Sucrose.XamlAnimatedGif | built-in GIF renderer |
| Shared projects (.shproj) + preprocessor symbols (`PORTAL`, `LIVE_*`, …) | normal crates; no conditional compilation per app |

## Skylark types Sucrose uses

Counted from the using-aliases in `src/` (number of files that alias the type). Port these, nothing else.

| Type | Files | Likely Rust home |
|---|---|---|
| Skylark.Wing.Native.Methods | 26 | gone: `windows` crate provides the Win32 declarations |
| Skylark.Helper.Culture | 24 | `sys-locale` + `sucrose-core` |
| Skylark.Helper.Versionly | 20 | `env!("CARGO_PKG_VERSION")`, OS version in `sucrose-platform` |
| Skylark.Enum.WindowsThemeType | 14 | `dark-light` |
| Skylark.Wing.Utility.Desktop | 11 | `sucrose-platform/windows` (desktop surface) |
| Skylark.Standard…CryptologyExtension | 11 | only `TextToBase`/`BaseToText` are used → `base64` |
| Skylark.Enum.StorageType, Struct.Storage, StorageExtension | 11/7/7 | only `StorageExtension.Convert` (size formatting) → small helper |
| Skylark.Enum.LevelLogType | 11 | `tracing` levels |
| Skylark.Enum.InputType | 10 | `sucrose-core` enum |
| Skylark.Helper.Skymath, Numeric, ClearNumericType | 9/7/7 | small helpers in `sucrose-core` |
| Skylark.Wing.Helper.WindowInterop | 8 | `sucrose-platform/windows` |
| Skylark.Enum.AssemblyType, Helper.Assemblies | 8/5 | gone (no .NET assemblies) |
| Skylark.Enum.ModeStorageType | 7 | `sucrose-core` |
| Skylark.Wing…AudioManager (two variants) | 6+6 | per-engine volume in engine crates; Windows audio session API in `sucrose-platform` |
| Skylark.Enum.DisplayScreenType, ScreenType, ExpandScreenType, DuplicateScreenType | 6/4/3/3 | display modes in `sucrose-core` |
| Skylark.Wing.Utility.Screene, Helper.ScreenManage, Struct.Monitor | 4/1/2 | monitors in `sucrose-platform` |
| Skylark.Wing.Helper.WindowsTheme, BackgroundImage | 4/3 | `sucrose-platform` |
| Skylark.Wing.Helper.SystemInfo, Extension.OperatingSystem, Provider.MajorVersion10Provider | 4/3/1 | `sucrose-platform` / `sysinfo` |
| Skylark.Enum.EnergySaverType, PowerPlanType, Wing.Utility.Power/Plan | 4/2/1/1 | power state in `sucrose-platform` |
| Skylark.Wing.Helper.Calculate, Helper.Generator, Guidly, Encode, EncodeType | 3 each | small helpers / `uuid` |
| Skylark.Wing.Helper.WindowsTaskbar, ScreenSaver, WindowOperations, WinAPI | 2/2/1/1 | `sucrose-platform` |
| Skylark.Wing.Helper.WindowsStartup, StartupPriority, StartupMachine | 1 each | autostart in `sucrose-platform` |
| Skylark.Wing.Helper.ShortcutDefault, ShortcutBasic | 1 each | installer (`apps/sucrose-setup`, D-019) |
| Skylark.Wing.Utility.SingleInstance | 1 | `sucrose-ipc` (D-009) |
| Skylark.Standard.Helper.GitHub, Interface.IReleases | 2 | gone: GitHub integrations removed (D-020) |
| Skylark.Wing.Interface.IEventListener, IDisplayManager | 1 each | `Platform` trait events |

The Skylark sources are in the maintainer's separate Skylark repository; read the implementation there when porting a type.