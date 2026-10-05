# Sucrose Publish Build Script

## Overview
`Sucrose.ps1` orchestrates a production-ready publish pipeline for the Sucrose solution. It detects build metadata automatically, rebuilds and publishes every supported project, optionally installs redistributable .NET runtimes, and produces compressed delivery packages ready for distribution.

## Prerequisites
- Windows with PowerShell 7.0 or newer.
- .NET SDK installed and available on `PATH` (the script can bootstrap the runtime when `InstallRuntime` is enabled).
- SevenZip binaries that ship with the repository under `src/Bundle/Sucrose.Bundle/SevenZip/`.
- Sufficient permissions to adjust the process execution policy and create/remove directories in the solution tree.

## Outputs
- Published binaries for each project under `../src/Sucrose/Package/<TargetFramework>/<PlatformTarget>/<ProjectName>`.
- Optional runtime bundle in `../src/Sucrose/Package/<TargetFramework>/<PlatformTarget>/Sucrose.Runtime`.
- Optional compressed archive at `../src/Sucrose/Package/Compressed/<TargetFramework>/Sucrose-<PlatformTarget>.7z`.
- Per-project publish logs stored in `Publish.log` within each project's output directory.

## Key Features
- Automatic detection of `Configuration`, `PlatformTarget`, `RuntimeIdentifier`, and `TargetFramework`.
- Clean rebuild of destination directories with retry logic per project.
- Centralized, timestamped status output for every operation.
- Optional runtime installation via the bundled `dotnet-install.ps1`.
- Optional package compression leveraging the repository's SevenZip binaries.
- Post-install cleanup to minimize runtime footprint.

## Parameters
| Parameter | Type / Values | Default | Description |
| --- | --- | --- | --- |
| `Configuration` | `Release`, `Debug`, empty | Environment `Configuration` or `Release` | Build configuration. |
| `PlatformTarget` | `x64`, `x86`, `ARM64`, empty | Environment `Platform` or `x64` | .NET target platform. |
| `SelfContained` | `true`, `false` | `false` | Controls self-contained publishes. |
| `RuntimeIdentifier` | `win-x64`, `win-x86`, `win-arm64`, empty | Derived from `PlatformTarget` | Target runtime identifier. |
| `TargetFramework` | string | Auto-detected from the first project | Target framework moniker. |
| `PublishDir` | string | `../src/Sucrose/Package` | Base publish directory relative to the solution root. |
| `PublishBaseDir` | string | Script directory | Root directory for path resolution. |
| `MaxAttempts` | integer `1-10` | `3` | Retry attempts per project publish. |
| `RetryDelay` | integer `1-60` | `2` | Seconds between retries. |
| `InstallRuntime` | `true`, `false` | `true` | Installs the .NET runtime bundle when `true`. |
| `CompressPackage` | `true`, `false` | `true` | Compresses the published output via SevenZip. |
| `DotNetVersion` | string | `10.0.112` | .NET runtime version passed to `dotnet-install.ps1`. |

## Usage
1. Open a PowerShell session at the repository root.
2. Run the script with optional parameters:
   ```powershell
   .\.build\Sucrose.ps1 -Configuration Release -PlatformTarget x64
   ```
3. Monitor the timestamped console output for progress. Logs for each project appear in the respective publish directories.

### Examples
- Publish using defaults:
  ```powershell
  .\.build\Sucrose.ps1
  ```
- Publish Debug build without runtime compression:
  ```powershell
  .\.build\Sucrose.ps1 -Configuration Debug -CompressPackage false
  ```
- Produce a self-contained ARM64 build:
  ```powershell
  .\.build\Sucrose.ps1 -PlatformTarget ARM64 -SelfContained true
  ```

## Runtime Installation
When `InstallRuntime` is `true`, the script executes `dotnet-install.ps1` to create a trimmed runtime bundle. After installation it removes unused SDK, templates, and ancillary files to reduce size.

## Compression Workflow
When `CompressPackage` is `true`, the script selects the appropriate `7z.exe` binary for the host architecture, recreates the `Compressed/<TargetFramework>` directory, and generates a solid LZMA2 archive of the published payload while excluding non-distribution folders.

## Error Handling
- Each publish attempt logs to both the console and `Publish.log`.
- Failures trigger retry cycles up to `MaxAttempts`. The script surfaces the last error with context if all retries fail.
- Runtime and compression stages throw descriptive errors when required assets are missing.

## Troubleshooting
- Verify the solution projects listed in `Sucrose.ps1` exist relative to the script directory.
- Ensure SevenZip binaries are present under `src/Bundle/Sucrose.Bundle/SevenZip/` for the required architecture.
- Confirm network access if `dotnet-install.ps1` must download runtimes.
- Review the per-project `Publish.log` files for MSBuild diagnostics when a publish fails.