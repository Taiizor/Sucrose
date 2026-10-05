<#
.SYNOPSIS
    Production-ready build and publish script for Sucrose solution.

.DESCRIPTION
    Publishes multiple .NET projects with automatic detection of build parameters.
    Features:
    - Auto-detection of Configuration, PlatformTarget, RuntimeIdentifier, TargetFramework
    - Clean build with retry mechanism
    - Comprehensive error handling and logging
    - Optional .NET runtime installation
    - Optional package compression with 7zip
    - Runtime cleanup for minimal distribution

.PARAMETER Configuration
    Build configuration (Release/Debug). Default: Release (auto-detected from environment)

.PARAMETER PlatformTarget
    Target platform (x64, x86, ARM64). Default: x64 (auto-detected from environment)

.PARAMETER SelfContained
    Whether the publish should be self-contained. Default: false

.PARAMETER RuntimeIdentifier
    Runtime identifier for publish. Default: auto-detected based on PlatformTarget

.PARAMETER TargetFramework
    Target framework. Default: auto-detected from first project

.PARAMETER PublishDir
    Base publish folder relative to solution root. Default: ..\src\Sucrose\Package

.PARAMETER PublishBaseDir
    Base directory of the solution. Default: script location

.PARAMETER MaxAttempts
    Number of retry attempts per project. Default: 3

.PARAMETER RetryDelay
    Delay between retries in seconds. Default: 3

.PARAMETER InstallRuntime
    If true, installs .NET runtimes. Default: true

.PARAMETER CompressPackage
    If true, compresses the published package using 7zip. Default: true

.PARAMETER DotNetVersion
    Version of .NET to install. Default: 10.0.112

.EXAMPLE
    .\Sucrose.ps1
    Builds with default settings (Release, x64, no self-contained)

.EXAMPLE
    .\Sucrose.ps1 -Configuration Debug -PlatformTarget x86
    Builds with specific configuration and platform

.EXAMPLE
    .\Sucrose.ps1 -SelfContained "true" -CompressPackage "false"
    Builds self-contained without compression
#>

[CmdletBinding()]
param (
    [Parameter(HelpMessage = "Build configuration (Release/Debug)")]
    [ValidateSet("Release", "Debug", "", IgnoreCase = $false)]
    [AllowEmptyString()]
    [string]$Configuration = "",

    [Parameter(HelpMessage = "Target platform (x64, x86, ARM64)")]
    [ValidateSet("x64", "x86", "ARM64", "", IgnoreCase = $false)]
    [AllowEmptyString()]
    [string]$PlatformTarget = "",

    [Parameter(HelpMessage = "Self-contained publish")]
    [ValidateSet("true", "false")]
    [string]$SelfContained = "false",

    [Parameter(HelpMessage = "Runtime identifier")]
    [ValidateSet("win-x64", "win-x86", "win-arm64", "", IgnoreCase = $false)]
    [AllowEmptyString()]
    [string]$RuntimeIdentifier = "",

    [Parameter(HelpMessage = "Target framework")]
    [string]$TargetFramework,

    [Parameter(HelpMessage = "Publish directory relative to solution")]
    [string]$PublishDir = "..\src\Sucrose\Package",

    [Parameter(HelpMessage = "Base directory of the solution")]
    [string]$PublishBaseDir = (Split-Path -Parent $PSCommandPath),

    [Parameter(HelpMessage = "Maximum retry attempts")]
    [ValidateRange(1, 10)]
    [int]$MaxAttempts = 3,

    [Parameter(HelpMessage = "Retry delay in seconds")]
    [ValidateRange(1, 60)]
    [int]$RetryDelay = 2,

    [Parameter(HelpMessage = "Install .NET runtime after publish")]
    [ValidateSet("true", "false")]
    [string]$InstallRuntime = "true",

    [Parameter(HelpMessage = "Compress package after publish")]
    [ValidateSet("true", "false")]
    [string]$CompressPackage = "true",

    [Parameter(HelpMessage = ".NET version to install")]
    [string]$DotNetVersion = "10.0.112"
)

#region Initialization

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Set console encoding to UTF-8 for proper character display
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Configure execution policy for current process
try {
    $currentPolicy = Get-ExecutionPolicy -Scope Process
    if ($currentPolicy -in @("Restricted", "AllSigned")) {
        Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] Current execution policy: $currentPolicy. Setting to Bypass..." -ForegroundColor Yellow
        Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force
        Write-Host "$(Get-Date -Format 'HH:mm:ss') [SUCCESS] Execution policy updated to Bypass" -ForegroundColor Green
    } else {
        Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] Current execution policy: $currentPolicy" -ForegroundColor Gray
    }
} catch {
    Write-Warning "$(Get-Date -Format 'HH:mm:ss') [WARNING] Could not check/set execution policy: $($_.Exception.Message)"
}

#endregion

#region Configuration Detection

# Auto-detect Configuration
if (-not $Configuration) {
    $Configuration = $env:Configuration
    if (-not $Configuration) { 
        $Configuration = "Release" 
    }
}
Write-Host "$(Get-Date -Format 'HH:mm:ss') [CONFIG] Configuration: $Configuration" -ForegroundColor Cyan

# Auto-detect PlatformTarget
if (-not $PlatformTarget) {
    $PlatformTarget = $env:Platform
    if (-not $PlatformTarget) { 
        $PlatformTarget = "x64" 
    }
}
Write-Host "$(Get-Date -Format 'HH:mm:ss') [CONFIG] PlatformTarget: $PlatformTarget" -ForegroundColor Cyan

# Auto-detect RuntimeIdentifier from PlatformTarget
if (-not $RuntimeIdentifier) {
    $RuntimeIdentifier = switch ($PlatformTarget) {
        "x86"   { "win-x86" }
        "x64"   { "win-x64" }
        "ARM64" { "win-arm64" }
        default { throw "Unsupported PlatformTarget: $PlatformTarget" }
    }
}
Write-Host "$(Get-Date -Format 'HH:mm:ss') [CONFIG] RuntimeIdentifier: $RuntimeIdentifier" -ForegroundColor Cyan

#endregion

#region Project Configuration

# Define all projects to publish
$script:Projects = @(
    "..\src\Launcher\Sucrose.Launcher\Sucrose.Launcher.csproj",
    "..\src\Live\Sucrose.Live.Aurora\Sucrose.Live.Aurora.csproj",
    "..\src\Live\Sucrose.Live.CefSharp\Sucrose.Live.CefSharp.csproj",
    "..\src\Live\Sucrose.Live.MpvPlayer\Sucrose.Live.MpvPlayer.csproj",
    "..\src\Live\Sucrose.Live.Nebula\Sucrose.Live.Nebula.csproj",
    "..\src\Live\Sucrose.Live.Vexana\Sucrose.Live.Vexana.csproj",
    "..\src\Live\Sucrose.Live.VlcPlayer\Sucrose.Live.VlcPlayer.csproj",
    "..\src\Live\Sucrose.Live.WebView\Sucrose.Live.WebView.csproj",
    "..\src\Live\Sucrose.Live.Xavier\Sucrose.Live.Xavier.csproj",
    "..\src\Localizer\Sucrose.Localizer\Sucrose.Localizer.csproj",
    "..\src\Portal\Sucrose.Portal\Sucrose.Portal.csproj",
    "..\src\Project\Sucrose.Backgroundog\Sucrose.Backgroundog.csproj",
    "..\src\Project\Sucrose.Commandog\Sucrose.Commandog.csproj",
    "..\src\Project\Sucrose.Property\Sucrose.Property.csproj",
    "..\src\Project\Sucrose.Reportdog\Sucrose.Reportdog.csproj",
    "..\src\Project\Sucrose.Undo\Sucrose.Undo.csproj",
    "..\src\Project\Sucrose.Watchdog\Sucrose.Watchdog.csproj",
    "..\src\Update\Sucrose.Update\Sucrose.Update.csproj"
)

# Auto-detect TargetFramework from first project if not specified
if (-not $TargetFramework) {
    $firstProjectPath = Join-Path $PublishBaseDir $script:Projects[0]
    
    if (-not (Test-Path $firstProjectPath)) {
        throw "First project not found: $firstProjectPath"
    }

    try {
        [xml]$projXml = Get-Content -Path $firstProjectPath -ErrorAction Stop
        
        $detectedFramework = $null
        foreach ($pg in $projXml.Project.PropertyGroup) {
            # Check for TargetFrameworks (plural) - safely
            $targetFrameworksNode = $pg.SelectSingleNode("TargetFrameworks")
            if ($null -ne $targetFrameworksNode -and -not [string]::IsNullOrWhiteSpace($targetFrameworksNode.InnerText)) {
                $tfValue = $targetFrameworksNode.InnerText.Trim()
                if ($tfValue -like "*;*") {
                    $frameworks = $tfValue -split ';' | ForEach-Object { $_.Trim() } | Where-Object { $_ }
                    $detectedFramework = $frameworks[0]
                } else {
                    $detectedFramework = $tfValue
                }
                break
            }

            # Check for TargetFramework (singular) - safely
            $targetFrameworkNode = $pg.SelectSingleNode("TargetFramework")
            if ($null -ne $targetFrameworkNode -and -not [string]::IsNullOrWhiteSpace($targetFrameworkNode.InnerText)) {
                $detectedFramework = $targetFrameworkNode.InnerText.Trim()
                break
            }
        }

        if ([string]::IsNullOrWhiteSpace($detectedFramework)) {
            throw "No TargetFramework found in project file"
        }

        $TargetFramework = $detectedFramework
    } catch {
        Write-Warning "$(Get-Date -Format 'HH:mm:ss') [WARNING] Could not auto-detect TargetFramework: $($_.Exception.Message)"
        $TargetFramework = "net10.0-windows"
    }
}
Write-Host "$(Get-Date -Format 'HH:mm:ss') [CONFIG] TargetFramework: $TargetFramework" -ForegroundColor Cyan

#endregion

#region Helper Functions

function Write-StatusMessage {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Message,
        
        [Parameter(Mandatory = $false)]
        [ValidateSet("Info", "Success", "Warning", "Error")]
        [string]$Type = "Info"
    )

    $timestamp = Get-Date -Format 'HH:mm:ss'
    $color = switch ($Type) {
        "Info"    { "Cyan" }
        "Success" { "Green" }
        "Warning" { "Yellow" }
        "Error"   { "Red" }
    }

    $prefix = switch ($Type) {
        "Info"    { "[INFO]" }
        "Success" { "[SUCCESS]" }
        "Warning" { "[WARNING]" }
        "Error"   { "[ERROR]" }
    }

    Write-Host "$timestamp $prefix $Message" -ForegroundColor $color
}

function Publish-SucroseProject {
    <#
    .SYNOPSIS
        Publishes a single .NET project with retry logic
    #>
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string]$ProjectPath,
        
        [Parameter(Mandatory = $true)]
        [string]$ProjectName
    )

    # Determine output directory
    $destination = Join-Path $PublishBaseDir $PublishDir
    $destination = Join-Path $destination "$TargetFramework\$PlatformTarget\$ProjectName"

    # Clean and create destination directory
    if (Test-Path $destination) {
        Write-StatusMessage "Cleaning destination: $destination" -Type "Warning"
        Remove-Item -Path $destination -Recurse -Force -ErrorAction Stop
    }
    New-Item -ItemType Directory -Path $destination -Force | Out-Null

    $logFile = Join-Path $destination "Publish.log"
    Write-StatusMessage "Publishing $ProjectName..." -Type "Info"

    # Retry loop
    $attempt = 0
    $success = $false

    do {
        $attempt++
        Write-Host "$(Get-Date -Format 'HH:mm:ss') [ATTEMPT] $attempt of $MaxAttempts" -ForegroundColor Gray

        try {
            $publishArgs = @(
                "publish",
                $ProjectPath,
                "-c", $Configuration,
                "/p:PlatformTarget=$PlatformTarget",
                "-r", $RuntimeIdentifier,
                "-f", $TargetFramework,
                "--self-contained", $SelfContained,
                "-o", $destination,
                "--nologo",
                "--use-current-runtime",
                "--verbosity", "minimal"
            )

            dotnet @publishArgs *>&1 | Tee-Object -FilePath $logFile

            if ($LASTEXITCODE -eq 0) {
                $success = $true
                Write-StatusMessage "Successfully published $ProjectName" -Type "Success"

                # Special processing for CefSharp project
                if ($ProjectName -eq "Sucrose.Live.CefSharp") {
                    Configure-CefSharpSubprocess -OutputPath $destination
                }
            } else {
                Write-StatusMessage "Publish failed for $ProjectName (exit code: $LASTEXITCODE)" -Type "Error"
                
                if ($attempt -lt $MaxAttempts) {
                    Write-StatusMessage "Retrying in $RetryDelay seconds..." -Type "Warning"
                    Start-Sleep -Seconds $RetryDelay
                }
            }
        } catch {
            Write-StatusMessage "Exception during publish: $($_.Exception.Message)" -Type "Error"
            
            if ($attempt -lt $MaxAttempts) {
                Write-StatusMessage "Retrying in $RetryDelay seconds..." -Type "Warning"
                Start-Sleep -Seconds $RetryDelay
            }
        }

    } while (-not $success -and $attempt -lt $MaxAttempts)

    if (-not $success) {
        throw "Failed to publish $ProjectName after $MaxAttempts attempts. See log: $logFile"
    }
}

function Install-DotNetRuntime {
    <#
    .SYNOPSIS
        Installs .NET runtime for distribution
    #>
    [CmdletBinding()]
    param()

    $dotnetInstallScript = Join-Path $PublishBaseDir "dotnet-install.ps1"
    
    if (-not (Test-Path $dotnetInstallScript)) {
        throw "dotnet-install.ps1 not found at: $dotnetInstallScript"
    }

    # Prepare runtime installation directory
    $runtimeInstallDir = Join-Path $PublishBaseDir $PublishDir
    $runtimeInstallDir = Join-Path $runtimeInstallDir "$TargetFramework\$PlatformTarget\Sucrose.Runtime"

    if (Test-Path $runtimeInstallDir) {
        Write-StatusMessage "Cleaning runtime directory: $runtimeInstallDir" -Type "Warning"
        Remove-Item -Path $runtimeInstallDir -Recurse -Force -ErrorAction Stop
    }
    New-Item -ItemType Directory -Path $runtimeInstallDir -Force | Out-Null

    Write-StatusMessage "Installing .NET $DotNetVersion runtime ($PlatformTarget)..." -Type "Info"

    try {
        & $dotnetInstallScript -Version $DotNetVersion -NoPath -Architecture $PlatformTarget -InstallDir $runtimeInstallDir

        if ($LASTEXITCODE -ne 0) {
            throw "Installation failed with exit code: $LASTEXITCODE"
        }

        Write-StatusMessage ".NET runtime installed successfully" -Type "Success"

        # Clean unnecessary runtime files
        Remove-UnnecessaryRuntimeFiles -RuntimeDir $runtimeInstallDir

    } catch {
        throw "Failed to install .NET runtime: $($_.Exception.Message)"
    }
}

function Configure-CefSharpSubprocess {
    <#
    .SYNOPSIS
        Configures CefSharp.BrowserSubprocess.exe to use custom .NET runtime
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    Write-StatusMessage "Configuring CefSharp.BrowserSubprocess for custom runtime..." -Type "Info"

    $cefSharpExe = Join-Path $OutputPath "CefSharp.BrowserSubprocess.exe"
    $runtimeConfigPath = Join-Path $OutputPath "CefSharp.BrowserSubprocess.runtimeconfig.json"

    if (Test-Path $cefSharpExe) {
        # Extract framework version from TargetFramework (e.g., net10.0 or net10.0-windows)
        $tfmValue = $TargetFramework -replace '-windows$', ''
        
        # Extract version number (e.g., 10.0 from net10.0)
        if ($tfmValue -match '^net(\d+)\.(\d+)$') {
            $majorVersion = [int]$matches[1]
            $minorVersion = [int]$matches[2]
        } elseif ($tfmValue -match '^net(\d+)$') {
            $majorVersion = [int]$matches[1]
            $minorVersion = 0
        } else {
            Write-StatusMessage "Could not parse TargetFramework '$TargetFramework', defaulting to net10.0" -Type "Warning"
            $majorVersion = 10
            $minorVersion = 0
        }
        
        # Clamp to valid range: minimum net6.0, maximum net10.0
        if ($majorVersion -lt 6) {
            Write-StatusMessage "Framework version too low ($majorVersion.$minorVersion), clamping to net6.0" -Type "Warning"
            $majorVersion = 6
            $minorVersion = 0
        } elseif ($majorVersion -gt 10) {
            Write-StatusMessage "Framework version too high ($majorVersion.$minorVersion), clamping to net10.0" -Type "Warning"
            $majorVersion = 10
            $minorVersion = 0
        }
        
        # Build tfm and version strings
        $runtimeTfm = "net$majorVersion.$minorVersion"
        $runtimeVersion = "$majorVersion.$minorVersion.0"
        
        # Create custom runtime configuration
        $runtimeConfig = @{
            runtimeOptions = @{
                tfm = $runtimeTfm
                rollForward = "LatestMajor"
                framework = @{
                    name = "Microsoft.NETCore.App"
                    version = $runtimeVersion
                }
                additionalProbingPaths = @(
                    "../Sucrose.Runtime/shared/Microsoft.NETCore.App/$runtimeVersion"
                )
                configProperties = @{
                    "DOTNET_ROOT" = "../Sucrose.Runtime"
                    "DOTNET_MULTILEVEL_LOOKUP" = "0"
                }
            }
        }

        # Convert to JSON and write
        $jsonContent = $runtimeConfig | ConvertTo-Json -Depth 10
        Set-Content -Path $runtimeConfigPath -Value $jsonContent -Encoding UTF8

        Write-StatusMessage "CefSharp.BrowserSubprocess runtime configuration applied" -Type "Success"
    } else {
        Write-StatusMessage "CefSharp.BrowserSubprocess.exe not found in output" -Type "Warning"
    }
}

function Remove-UnnecessaryRuntimeFiles {
    <#
    .SYNOPSIS
        Removes unnecessary files from runtime installation to minimize size
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$RuntimeDir
    )

    Write-StatusMessage "Cleaning unnecessary runtime files..." -Type "Info"

    # Files to remove
    $filesToRemove = @(
        "dnx.cmd",
        "dnx.ps1",
        "dotnet.exe",
        "LICENSE.txt",
        "ThirdPartyNotices.txt"
    )

    # Directories to remove
    $dirsToRemove = @(
        "templates",
        "sdk-manifests",
        "sdk",
        "packs",
        "shared\Microsoft.AspNetCore.App"
    )

    # Remove files
    foreach ($file in $filesToRemove) {
        $path = Join-Path $RuntimeDir $file
        if (Test-Path $path) {
            Write-Host "$(Get-Date -Format 'HH:mm:ss') [CLEANUP] Removing file: $file" -ForegroundColor Gray
            Remove-Item -Path $path -Force -ErrorAction SilentlyContinue
        }
    }

    # Remove directories
    foreach ($dir in $dirsToRemove) {
        $path = Join-Path $RuntimeDir $dir
        if (Test-Path $path) {
            Write-Host "$(Get-Date -Format 'HH:mm:ss') [CLEANUP] Removing directory: $dir" -ForegroundColor Gray
            Remove-Item -Path $path -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Write-StatusMessage "Runtime cleanup completed" -Type "Success"
}

function Compress-SucrosePackage {
    <#
    .SYNOPSIS
        Compresses the published package using 7zip
    #>
    [CmdletBinding()]
    param()

    Write-StatusMessage "Starting package compression..." -Type "Info"

    # Detect system architecture for 7zip selection
    $systemArch = if ([Environment]::Is64BitOperatingSystem) {
        $procArch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
        switch ($procArch.ToLower()) {
            "amd64" { "x64" }
            "arm64" { "ARM64" }
            default { "x64" }
        }
    } else {
        "x86"
    }

    Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] System architecture: $systemArch" -ForegroundColor Gray

    # Locate 7zip executable
    $sevenZipExe = Join-Path $PublishBaseDir "..\src\Bundle\Sucrose.Bundle\SevenZip\7z-$systemArch\7z.exe"
    
    if (-not (Test-Path $sevenZipExe)) {
        throw "7zip executable not found: $sevenZipExe"
    }

    # Prepare output directory
    $outputDir = Join-Path $PublishBaseDir "$PublishDir\Compressed\$TargetFramework"
    if (Test-Path $outputDir) {
        Write-StatusMessage "Cleaning compression output directory" -Type "Warning"
        Remove-Item -Path $outputDir -Recurse -Force -ErrorAction Stop
    }
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

    # Define paths
    $sourceDir = Join-Path $PublishBaseDir "$PublishDir\$TargetFramework\$PlatformTarget"
    $archiveFile = Join-Path $outputDir "Sucrose-$PlatformTarget.7z"

    Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] Source: $sourceDir" -ForegroundColor Gray
    Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] Archive: $archiveFile" -ForegroundColor Gray

    # Build 7zip arguments
    $excludeFolders = @("Sucrose.Bundle", "Sucrose.Localizer")
    $excludeArgs = $excludeFolders | ForEach-Object { "-xr!$_" }

    # Resolve paths to full absolute paths
    $resolvedSourceDir = (Resolve-Path $sourceDir -ErrorAction Stop).Path
    $resolvedArchiveFile = [System.IO.Path]::GetFullPath($archiveFile)

    Write-Host "$(Get-Date -Format 'HH:mm:ss') [DEBUG] Resolved Source: $resolvedSourceDir" -ForegroundColor Gray
    Write-Host "$(Get-Date -Format 'HH:mm:ss') [DEBUG] Resolved Archive: $resolvedArchiveFile" -ForegroundColor Gray

    $arguments = @(
        "a",                           # Add to archive
        "-t7z",                        # 7z format
        "-m0=lzma2",                   # Compression method
        "-mx=9",                       # Maximum compression
        "-mfb=64",                     # Fast bytes
        "-ms=64m",                     # Solid block size
        $resolvedArchiveFile,          # Output archive (absolute path)
        "$resolvedSourceDir\*"         # Source files (absolute path with wildcard)
    ) + $excludeArgs

    Write-StatusMessage "Compressing package..." -Type "Info"

    try {
        & $sevenZipExe @arguments

        if ($LASTEXITCODE -ne 0) {
            throw "Compression failed with exit code: $LASTEXITCODE"
        }

        $archiveSize = (Get-Item $archiveFile).Length / 1MB
        Write-StatusMessage "Package compressed successfully ($([math]::Round($archiveSize, 2)) MB)" -Type "Success"
        Write-Host "$(Get-Date -Format 'HH:mm:ss') [INFO] Archive location: $archiveFile" -ForegroundColor Cyan

    } catch {
        throw "Failed to compress package: $($_.Exception.Message)"
    }
}

#endregion

#region Main Execution

try {
    $scriptStartTime = Get-Date
    
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  Sucrose Production Build Script" -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan

    # Publish all projects
    Write-StatusMessage "Starting project publishing..." -Type "Info"
    
    $projectCount = $script:Projects.Count
    $currentProject = 0

    foreach ($proj in $script:Projects) {
        $currentProject++
        $projName = [System.IO.Path]::GetFileNameWithoutExtension($proj)
        $fullPath = Join-Path $PublishBaseDir $proj

        Write-Host "`n--- Project $currentProject of $projectCount ---" -ForegroundColor Magenta
        
        if (-not (Test-Path $fullPath)) {
            Write-StatusMessage "Project not found: $fullPath" -Type "Error"
            throw "Project file not found: $fullPath"
        }

        Publish-SucroseProject -ProjectPath $fullPath -ProjectName $projName
    }

    Write-Host "`n" -NoNewline
    Write-StatusMessage "All projects published successfully" -Type "Success"

    # Install .NET runtime if requested
    if ($InstallRuntime -eq "true") {
        Write-Host "`n========================================" -ForegroundColor Cyan
        Install-DotNetRuntime
    }

    # Compress package if requested
    if ($CompressPackage -eq "true") {
        Write-Host "`n========================================" -ForegroundColor Cyan
        Compress-SucrosePackage
    }

    # Calculate and display execution time
    $scriptEndTime = Get-Date
    $executionTime = $scriptEndTime - $scriptStartTime
    
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  Build Completed Successfully" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Execution time: $($executionTime.ToString('hh\:mm\:ss'))" -ForegroundColor Cyan
    Write-Host "Configuration: $Configuration | Platform: $PlatformTarget | Framework: $TargetFramework" -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan

} catch {
    Write-Host "`n========================================" -ForegroundColor Red
    Write-Host "  Build Failed" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Location: $($_.InvocationInfo.ScriptName):$($_.InvocationInfo.ScriptLineNumber)" -ForegroundColor Red
    Write-Host "========================================`n" -ForegroundColor Red
    exit 1
}

#endregion
