# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sucrose is an open-source wallpaper engine for Windows. It supports GIF, URL, Web, Video, YouTube, and Application wallpaper types with audio-reactive and system-status APIs. The application is built as a multi-process architecture where separate executables handle rendering, UI, background services, and system monitoring.

## Build Commands

```powershell
# Restore packages
dotnet restore src/Sucrose.slnx

# Build entire solution (x64, Release)
dotnet build src/Sucrose.slnx -c Release -p:PlatformTarget=x64

# Build a single project
dotnet build src/Portal/Sucrose.Portal/Sucrose.Portal.csproj -c Release -p:PlatformTarget=x64

# Full publish pipeline (uses .build/Sucrose.ps1)
powershell -File .build/Sucrose.ps1
powershell -File .build/Sucrose.ps1 -Configuration Debug -PlatformTarget x86
powershell -File .build/Sucrose.ps1 -SelfContained "true" -CompressPackage "false"
```

There are three solutions: `src/Sucrose.slnx` (main app), `src/Sucrose.Bundle.slnx` (`Sucrose.Bundle` installer/packager), and `src/Sucrose.Localizer.slnx` (`Sucrose.Localizer` translation tool). Note that **no CI job builds the full solution** — full builds happen only locally or via the publish script (see Git Workflow for what CI actually compiles).

Build output goes to `src/Sucrose/` (defined via `BaseOutputPath` in `Directory.Build.props`); the publish script writes packages to `src/Sucrose/Package`. `EnforceCodeStyleInBuild=true`, so `.editorconfig` style violations fail the build. CI builds with `-p:UseSharedCompilation=false`.

**There is no automated test suite** — no test projects exist in the solution, so there are no test commands to run.

### Suppressed Warnings

The CI pipeline suppresses these warnings: `CS0067, CS0108, CS0109, CS0114, CS0169, CS0414, CS0618, CS0649, CS8632, CA1416, NU5104, NETSDK1138, SYSLIB0003`. Additionally `CS8632, WFO0003, SYSLIB0014` are suppressed in `Directory.Build.props`.

## Framework & Tooling

- **.NET 10.0-windows** target framework (`TargetFrameworks` in `Directory.Build.props`), built with the **.NET 11 preview SDK** — `global.json` pins `11.0.0` with `rollForward: latestMajor` and `allowPrerelease: true`; CI uses `dotnet-version: 11.0.x` / `dotnet-quality: preview`. The publish script installs runtime `10.0.112` by default (`-DotNetVersion`)
- **C# preview** language version with implicit usings enabled, nullable disabled
- **WPF** for all UI (no WinUI/MAUI — `exp/` holds experimental Avalonia/Uno/WinUI ports that are not part of the main solution)
- **Platforms**: x86, x64, ARM64 (conditional compilation symbols: `X86`, `X64`, `ARM64`)
- **Centralized package management** via `Directory.Packages.props` with transitive pinning
- **Solution format**: `.slnx` (new XML-based solution format)

## Architecture

### Repository Layout

The physical `src/` folder names do **not** match the logical layer names below — map them when navigating:

| Folder | Contains |
|--------|----------|
| `src/Launcher/` | `Sucrose.Launcher` |
| `src/Portal/` | `Sucrose.Portal` |
| `src/Live/` | Live rendering engines |
| `src/Project/` | The **Services** (`Backgroundog`, `Commandog`, `Watchdog`, `Reportdog`, `Property`, `Undo`) |
| `src/Library/` | Shared **class libraries** (`Manager`, `Memory`, `Mpv.NET`, `Pipe`, `Resources`, `Signal`, `Transmission`, `XamlAnimatedGif`) |
| `src/Shared/` | Shared Item Projects (`.shproj`), with engine-specific ones under `src/Shared/Engine/` |
| `src/Update/` | `Sucrose.Update` |
| `src/Bundle/` | `Sucrose.Bundle` (installer/packager) |
| `src/Localizer/` | `Sucrose.Localizer` (translation tool) |
| `src/Sucrose/` | Build output (generated; `BaseOutputPath`) |
| `exp/` | Experimental ports (Avalonia, Uno, WinUI, Services) — **not** part of the main solution |

### Multi-Process Design

Each component runs as a separate Windows process, communicating via named pipes (`Sucrose.Pipe`) and signals (`Sucrose.Signal`):

| Layer | Projects | Purpose |
|-------|----------|---------|
| **Launcher** | `Sucrose.Launcher` | Entry point, instance checking, Discord Rich Presence |
| **Portal** | `Sucrose.Portal` | Main settings/management UI (WPF-UI + MVVM + DI) |
| **Live Engines** | `Sucrose.Live.{WebView,CefSharp,MpvPlayer,VlcPlayer,Aurora,Nebula,Vexana,Xavier}` | Wallpaper rendering backends |
| **Services** | `Sucrose.Backgroundog` | Background wallpaper service with audio visualization |
| | `Sucrose.Commandog` | Command-line handler |
| | `Sucrose.Watchdog` | System monitoring |
| | `Sucrose.Reportdog` | Error reporting |
| | `Sucrose.Property` | Wallpaper property management |
| | `Sucrose.Undo` | Rollback functionality |
| **Update** | `Sucrose.Update` | Auto-update service |
| **Libraries** | `Sucrose.Manager, .Memory, .Pipe, .Signal, .Transmission, .Resources, .Mpv.NET, .XamlAnimatedGif` | Shared libraries |

### Shared Item Projects (.shproj)

Code reuse is achieved through **Shared Item Projects** (not NuGet packages or class libraries). These are compiled into each consuming project:

- **Core shared**: `Sucrose.Shared.{Core,Dependency,Discord,Launcher,Live,Pipe,Signal,Space,Store,Theme,Transmission,Watchdog,Zip,SevenZip}`
- **Engine-specific shared**: `Sucrose.Shared.Engine.{Aurora,CefSharp,MpvPlayer,VlcPlayer,Nebula,Vexana,WebView,Xavier}` + base `Sucrose.Shared.Engine`

Each executable project imports shared items via `<Import Project="..." Label="Shared" />` in its `.csproj`.

### Preprocessor Symbols

Every executable defines a project-specific symbol used in shared code for conditional compilation:
- Apps: `LAUNCHER`, `PORTAL`, `BACKGROUNDOG`, `COMMANDOG`, `WATCHDOG`, `REPORTDOG`, `PROPERTY`, `UNDO`, `UPDATE`
- Engines: `ENGINE` + `LIVE_WEBVIEW`, `LIVE_CEFSHARP`, `LIVE_MPVPLAYER`, `LIVE_VLCPLAYER`, `LIVE_AURORA`, `LIVE_NEBULA`, `LIVE_VEXANA`, `LIVE_XAVIER`
- Platforms: `X86`, `X64`, `ARM64`

### Portal (UI) Architecture

The Portal uses **WPF-UI** framework with:
- **MVVM** via `CommunityToolkit.Mvvm`
- **Dependency Injection** via `Microsoft.Extensions.Hosting`
- Pages: Library, Store, Settings (General, Personal, System, Wallpaper, Performance, Other, Donate)
- Reflection-based service registration by namespace

## Code Conventions

### Namespace Alias Pattern (Critical)

The codebase uses **aggressive `using` aliases** as a core convention. Every namespace import is aliased to a short acronym:

```csharp
using SMMG = Sucrose.Manager.Manage.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SPVPLP = Sucrose.Portal.Views.Pages.LibraryPage;
```

The alias is formed from the **first letter of each namespace segment**. When writing new code, follow this pattern — never use fully qualified names or bare `using` imports for Sucrose namespaces.

### Shared Project Organization

Files in shared projects follow a consistent directory structure:
- `Enum/` — Enum types
- `Helper/` — Static helper/utility classes
- `Manage/` — Configuration and state management
- `Manage/Manager/` — Settings manager classes
- `Manage/Readonly/` — Read-only constants
- `Struct/` — Struct types
- `Extension/` — Extension methods

### Style Rules

- 4-space indentation, CRLF line endings
- Block-scoped namespaces (`namespace Foo { }`)
- PascalCase for types/methods/properties, camelCase for fields
- Interfaces prefixed with `I`
- `using` directives outside namespace
- Prefer pattern matching, switch expressions, null coalescing
- Expression-bodied members for accessors/properties/indexers/lambdas (not for methods/constructors)
- XML indent: 2 spaces

## Key Configuration Files

| File | Purpose |
|------|---------|
| `Directory.Build.props` | Central build properties, version (auto-generated from date: `yy.MM.dd`), framework, platforms |
| `Directory.Build.targets` | Runtime config (thread pool, GC, platform symbols), app manifest |
| `Directory.Packages.props` | All NuGet package versions (centralized) |
| `global.json` | SDK version (`11.0.0` preview, `rollForward: latestMajor`) |
| `.editorconfig` | Code style enforcement |
| `NuGet.Config` | Package source (nuget.org only) |

## Git Workflow

- **Main branch**: `develop`
- Version is auto-derived from build date (`yy.MM.dd.0` format)
- CI runs CodeQL analysis on `develop` (push/PR/nightly schedule). The CodeQL job builds a matrix of only **seven `src/Library/` projects** (`Pipe, Memory, Signal, Manager, Resources, Transmission, XamlAnimatedGif`; `Mpv.NET` is excluded) with `-p:UseSharedCompilation=false` — it does **not** build the executables, engines, or full solution.
- `congratulations.yml` posts a welcome comment on first-time issues/PRs. This is the only other workflow — there is no automated app-build or release CI.