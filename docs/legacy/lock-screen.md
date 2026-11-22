# Legacy reference: Lock screen wallpaper

> Inventory of the **C# app** (not the Rust rewrite) on branch `lock-screen-native` (commit `112de37f`, 2026-11-21), which was **not merged into `develop`** (nor `develop-rust`). Produced on 2026-11-22 by reading that commit with `git show`; read the sources with `git show lock-screen-native:<path>`. Use it as the behavior spec when porting. Items marked "unclear" were not verified; "inferred" means read from the code's shape, not stated by it. When you find a mistake, fix it here.

**Path roots used below (relative to the repository root, on that branch):**
- `N` = `src/Native/Sucrose.LockScreen.Core`
- `P` = `src/Portal/Sucrose.Portal`
- `S` = `src/Shared`
- `L` = `src/Library`
- `U` = `src/Project/Sucrose.Undo`
- `LOC` = `src/Library/Sucrose.Resources/Locales/Portal/Controls`

**Aliases.** `SWML` = `Skylark.Wing.Manage.LockScreen`, `SWMLC` = `Skylark.Wing.Model.LockScreenConfigModel`, `SELFT` = `Skylark.Enum.LockScreenFitType`, `SSSMI` = `Sucrose.Shared.Space.Manage.Internal`, `SSDHR` = `Sucrose.Shared.Dependency.Helper.Runtime`.

**Not in this repository.** On the C# side, reading/writing the config and registering/unregistering the provider are calls into `SWML` (Skylark.Wing 3.2.1.8, restored from a local package source, §6). That code is not in the commit, so the JSON the C# side writes, how `Muted` maps to native keys, and how registration gets admin rights are unclear.

---

## 1. What the user gets

- A live background on the Windows lock and sign-in screen (Win + L) for **Video and GIF themes**. The Portal's "Lock Screen" menu item is visible only for `WallpaperType.Video` and `Gif`; it is collapsed for Web, Url, YouTube and Application themes (`P/Views/Controls/LibraryCard.xaml.cs` `ContextMenu_Opened`).
- Exception: "Sync with Desktop" (§3.3) also writes the Preview file (or the Thumbnail) of non-video themes as the lock-screen media. Whether a still image (JPG/PNG) plays is unclear: the native side opens every non-GIF file with Media Foundation (§4.4).
- Native media formats: folder mode accepts `mp4, m4v, mov, wmv, avi, mkv, gif` (`N/Config.cpp` `IsMediaExtension`). A file path is used as-is: `.gif` (case-insensitive) goes to the WIC GIF decoder, anything else to Media Foundation (`N/VideoDecoder.cpp` `Start`). Codec support is whatever Media Foundation has installed.
- One medium, drawn into one bitmap shared by all lock-screen images; the canvas aspect follows the primary monitor only (`GetSystemMetrics(SM_CXSCREEN/SM_CYSCREEN)`, `N/dllmain.cpp`).
- Windows version: no version check anywhere in the commit. The hook relies on the undocumented `Windows.UI.Xaml` tree of `LogonUI.exe` (§4.3), i.e. the Windows 10/11 LogonUI; which builds work is unclear. SDK target `10.0` (`N/Sucrose.LockScreen.Core.vcxproj`).
- Built for x86 (Win32), x64 and ARM64 (§6). Inferred: an x86 build cannot work on 64-bit Windows (64-bit LogonUI cannot load a 32-bit DLL, and a 32-bit registrar writes to WOW6432Node).

## 2. Portal UI

**Library card** (`P/Views/Controls/LibraryCard.xaml(.cs)`)
- New context-menu item `MenuLockScreen` (icon `LockClosed24`) after "Use" and before "Preview"; header `Portal.LibraryCard.MenuLockScreen`.
- `ContextMenu_Opened`: for Video/GIF themes, `SWML.LoadConfig()`; when `Enabled` and `MediaPath` equals `<theme folder>\<Info.Source>` (ordinal, case-insensitive), the header gets the suffix ` (Selected)` (existing key `Portal.LibraryCard.Selected`).
- `MenuLockScreen_Click`: if the theme folder exists, show `ThemeLockScreen` with `Info` and `Theme` (folder), await it, dispose it.

**ThemeLockScreen dialog** (`P/Views/Controls/ThemeLockScreen.xaml(.cs)`): WPF-UI `ContentDialog`, max width 620, content min width 500.
- Identity card: thumbnail 110×70 (`<theme>\<Info.Thumbnail>`, `ImageLoader.LoadOptimal(path, true, 300)`, exceptions go to the watchdog), localized title and description (`LocalizationConverter`), type icon `Image24` for GIF, else `Video24`.
- Settings card (values loaded in `ContentDialog_Loaded` from `SWML.LoadConfig()`):

| Row (icon) | Control | Initial value |
|---|---|---|
| Enable (`LockClosed24`) | `EnableLockScreenSwitch` | `Enabled` and `MediaPath` == `<theme>\<Info.Source>` (case-insensitive) |
| Fit (`ArrowExpand24`) | `FitComboBox`, 170 px: Cover, Fill, Fit, Center (Tag = `SELFT`) | `FitMode`, else the first item (Cover) |
| Mute (`SpeakerMute24`) | `MuteAudioSwitch` | `Muted` |
| Sync with Desktop (`DesktopSync24`) | `MatchDesktopSwitch` | `MatchDesktopWallpaper` |

- While Enable is off, the Fit, Mute and Sync rows are disabled at opacity 0.6; on, enabled at 1.0 (`UpdateCardsState`).
- Buttons: "Save" (primary) and "Close" (no change).
- Save (`OnButtonClick`, primary only): reload the config; copy Fit, Mute and Sync into `FitMode`, `Muted`, `MatchDesktopWallpaper`; then:
  - Enable on and the media file exists: `MediaPath = <theme>\<Info.Source>`, `Enabled = true`, `SWML.SaveConfig(cfg, true)`; then, if `!SWML.IsProviderRegistered()` and the file `SSSMI.LockScreenCore` exists, `SWML.RegisterProvider(corePath)`.
  - Enable on but the media file is missing: nothing is saved (Fit/Mute/Sync changes are lost too).
  - Enable off: `Enabled = false` only if this theme is the current media; `SWML.SaveConfig(cfg, false)` always. The provider stays registered.
  - The meaning of the second `SaveConfig` argument is unclear.
- No success or error message exists anywhere in this flow.

**Localization.** New `LOC/ThemeLockScreen/Portal.ThemeLockScreen.<LANG>.xaml` (15 keys, all 25 languages), merged in `LOC/Portal.Controls.<LANG>.xaml`; one key added to `LOC/LibraryCard/Portal.LibraryCard.<LANG>.xaml`. Side effect: the commit drops the UTF-8 BOM from the 25 `Portal.Controls.*.xaml` files. Fit items are read with `SRER.GetValue("Portal", "ThemeLockScreen", "Fit", "<Mode>")`, falling back to the English mode name.

| Key | EN text |
|---|---|
| `Portal.LibraryCard.MenuLockScreen` | Lock Screen |
| `Portal.ThemeLockScreen.Title` / `.Close` / `.Save` | Lock Screen / Close / Save |
| `Portal.ThemeLockScreen.EnableTitle` | Set as Lock Screen Wallpaper |
| `Portal.ThemeLockScreen.EnableDesc` | Plays this theme live on the Windows lock and sign-in screen (Win + L). |
| `Portal.ThemeLockScreen.FitTitle` | Fit Mode |
| `Portal.ThemeLockScreen.FitDesc` | Determines how the video scales to match the display. |
| `Portal.ThemeLockScreen.Fit.Cover` / `.Fit.Fill` | Cover (Crop and Fill) / Fill (Stretch to Screen) |
| `Portal.ThemeLockScreen.Fit.Fit` / `.Fit.Center` | Fit (Letterbox to Screen) / Center (1:1 Centered) |
| `Portal.ThemeLockScreen.MuteTitle` | Mute Audio |
| `Portal.ThemeLockScreen.MuteDesc` | Mutes video audio playback on the lock screen. |
| `Portal.ThemeLockScreen.MatchDesktopTitle` | Sync with Desktop |
| `Portal.ThemeLockScreen.MatchDesktopDesc` | Automatically update the lock screen when a Video or GIF theme is selected on the desktop. |

## 3. Settings and config

### 3.1 Locations and constants
- Root `%ProgramData%\Sucrose\LockScreen` (machine-wide, not per user):
  - C#: `SWML.RootDirectory` is set in `SSDHR.Configure()` in every app, DEBUG and RELEASE (`S/Sucrose.Shared.Dependency/Helper/Runtime.cs`). `SSSMI.LockScreenRoot` holds the same path and is unused (`S/Sucrose.Shared.Space/Manage/Internal.cs`).
  - Native: `CoreState::InitializePaths` (`N/Common.cpp`) uses `SHGetKnownFolderPath(FOLDERID_ProgramData)` + `\Sucrose\LockScreen` (fallback `C:\ProgramData\Sucrose\LockScreen`), creates it plus `Cache` and `Logs`, and reads `config.json` there.
- New constants: `App.LockScreenCore = "Sucrose.LockScreen.Core.dll"` (`L/Sucrose.Memory/Manage/Readonly/App.cs`); `Folder.Native = "Native"`, `Folder.LockScreen = "LockScreen"` (`.../Folder.cs`).
- DLL path `SSSMI.LockScreenCore`: `<install>\Native\Sucrose.LockScreen.Core.dll` if that file exists, else `<install>\Native\<x86|x64|ARM64>\Release\Sucrose.LockScreen.Core.dll` (`SSSMI.Architecture` from the `X86`/`X64`/`ARM64` compile constants). `<install>` = `SSSMI.Folder`, i.e. `%LocalAppData%\Sucrose` (services.md §0).
- No `Setting/*.json` key is added. All lock-screen state is in `config.json`.

### 3.2 config.json as the native side reads it
`ConfigManager::Load` (`N/Config.cpp`): UTF-8, at most 1 MiB, parsed with `Windows.Data.Json.JsonObject::TryParse`. A missing, empty, oversized or unparsable file gives all defaults; missing keys keep their default; an exception mid-parse keeps what was read so far. Numbers are truncated to int, then clamped. Whether the C# writer adds a BOM (which `TryParse` may reject) is unclear.

| Key | Type | Default | Range / values | Effect |
|---|---|---|---|---|
| `General.Enabled` | bool | true | | false: core exits at once |
| `General.MediaPath` | string | "" | file or folder | the medium |
| `General.FitMode` | string | cover | `cover`, `fill`, `fit`, `center` (case-insensitive; anything else = cover) | §4.4 |
| `General.Speed` | int % | 100 | 25–400 | playback rate |
| `General.AudioVolume` | int | 0 | 0–100 | 0 = no audio thread |
| `General.FolderOrder` | int | 0 | 0 random, 1 "sequential", 2 fixed | folder mode only |
| `Performance.RenderWidth` | int px | 960 | 320–1920 | canvas width |
| `Performance.HwDecode` | bool | true | | D3D11 decode |
| `Performance.PlayOnPasswordScreen` | bool | true | | false: hold on the password screen |
| `Performance.IdlePauseSeconds` | number | 300 | not clamped; 0 = never | idle hold |
| `Performance.LowBatteryThreshold` | int % | 20 | 0–100 | parsed, no effect |
| `Performance.LowBatteryLevel` | int | 2 | 0 off, 1 light, 2 balanced, 3 maximum | parsed, no effect |
| `ColorAdjustments.Hue` / `Saturation` | int | 0 / 100 | −180–180 / 0–300 | color matrix |
| `ColorAdjustments.Brightness` / `Contrast` | int | 0 / 100 | −100–100 / 25–300 | LUT |
| `ColorAdjustments.Gamma` / `Dim` | int | 100 / 0 | 20–300 / 0–90 | LUT |
| `Screens.ShowOnShutdown` | bool | false | | parsed, no effect |
| `Screens.BootStillSeconds` | number | 180 | | parsed, no effect |
| `Screens.BootStillFrame` | bool | true | | load `last.bgra` when no first-frame cache (§4.7) |

- `lowBatteryOnAC` (default false) exists in `LockScreenConfig` (`N/Config.h`) but is never read.
- The C# code only touches `Enabled`, `MediaPath`, `FitMode` (`SELFT`), `Muted` and `MatchDesktopWallpaper`. The native side reads neither `Muted` nor `MatchDesktopWallpaper`; their JSON names, the `Muted` → `AudioVolume` mapping, the `SELFT` → string mapping and the values written for the other keys are in Skylark.Wing: unclear.
- Folder mode (`ResolveMediaFile`): files sorted case-insensitively; 2 = first file; 1 = `files[(GetTickCount64() / 1000) % n]` (time-based, nothing persisted, so not really sequential); 0 = `files[GetTickCount() % n]`. The Portal never writes a folder.
- Color (`CalculateColorMatrices`, `ApplyColor`): skipped when all six values are default. Otherwise a hue-rotation × saturation 3×3 matrix (luma weights 0.213/0.715/0.072, 8.8 fixed point; skipped when hue 0 and saturation 100), then a 256-entry LUT: `x = (x − 0.5) × contrast + 0.5 + brightness`, clamp to 0–1, `x^(1/gamma)`, `x × (1 − dim)`. In fit and center modes, pure black pixels (the bars) stay black.

### 3.3 Sync with the desktop wallpaper
`Run.Start()` (`S/Sucrose.Shared.Live/Helper/Run.cs`) calls `SyncLockScreen(Info)` each time a live wallpaper starts, after the Backgroundog launch and before the engine launch:
- Only when `Enabled` and `MatchDesktopWallpaper`.
- Video/GIF theme: media = `<Library.Location>\<Library.Selected>\<Info.Source>`. Other types: `<…>\<Info.Preview>`, or `<Info.Thumbnail>` when Preview is empty (contradicts the EN text, which says Video or GIF only).
- Writes (`SaveConfig(cfg, true)`) only when the file exists and differs (case-insensitive) from the current `MediaPath`. All exceptions are swallowed.
- Nothing notifies the lock screen: the native side reads `config.json` once per load into LogonUI (§4.2), so a change applies at the next lock (inferred).

## 4. Native component Sucrose.LockScreen.Core.dll

### 4.1 Kind and registration
- C++20 DLL using C++/WinRT; exports (`N/Sucrose.LockScreen.Core.def`, all PRIVATE): `DllGetClassObject`, `DllCanUnloadNow`, `DllRegisterServer`, `DllUnregisterServer`.
- It is a **dummy credential provider** (`N/CredentialProvider.cpp`): `ICredentialProvider` that accepts every usage scenario and reports 0 fields and 0 credentials (`CREDENTIAL_PROVIDER_NO_DEFAULT`, no auto-logon), so no tile appears. It exists only to make `LogonUI.exe` load the DLL; the real work starts in `DllMain`.
- CLSID `{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}`, name `Sucrose Lock Screen Live Wallpaper Provider` (`N/CredentialProvider.h`).
- `DllRegisterServer` writes REG_SZ values under HKLM, so it needs admin rights:
  - `HKLM\SOFTWARE\Classes\CLSID\{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}` (default) = name
  - `…\InprocServer32` (default) = full DLL path (`GetModuleFileNameW`); `ThreadingModel` = `Apartment`
  - `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\Credential Providers\{7C4B13B8-439D-4F18-A7F0-18DCEE32A001}` (default) = name
- `DllUnregisterServer` deletes the three keys (`RegDeleteKeyW`, errors ignored, always `S_OK`). `DllCanUnloadNow` always returns `S_FALSE`, so the DLL stays resident in LogonUI.
- Callers: `SWML.RegisterProvider(path)` (Portal, §2) and `SWML.UnregisterProvider()` (Undo, §5). Portal and Undo run `asInvoker` (`P/App.manifest`, `U/App.manifest`); how Skylark.Wing elevates (regsvr32 with UAC, a helper, or nothing) is unclear.
- Host: `LogonUI.exe`, which runs as SYSTEM on the Winlogon desktop (Windows behavior, not in the code). `DllMain` acts only when the host exe name is `LogonUI.exe` (case-insensitive); in other hosts (CredUI prompts also enumerate providers: inferred) the DLL stays inert.

### 4.2 Startup and shutdown (`N/dllmain.cpp`)
- `DLL_PROCESS_ATTACH` in LogonUI: `DisableThreadLibraryCalls`, then a thread runs `MainInitThreadProc`:
  1. Paths (§3.1) and config; exit if `!Enabled` or no media file resolves.
  2. Canvas: width = clamp(RenderWidth, 320, 1920) rounded down to even; height = width × screenH / screenW, even, clamped 180–4096; screen fallback 1920×1080 when below 320×200. One BGRA buffer (`g_shared`) behind a critical section.
  3. First-frame cache (§4.7): hit → apply color and show at once; miss and `BootStillFrame` → try `last.bgra`.
  4. `VideoDecoder::Start`; `AudioEngine::Start` only if not GIF, `AudioVolume > 0` and `Speed == 100`; `XamlHook::Start`.
- `DLL_PROCESS_DETACH` in LogonUI: stop flag; `XamlHook::Stop` asks the UI thread to restore the original images (posts `WM_NULL` up to 60 × 50 ms), waits 3 s for the worker, unhooks; decoder and audio threads are waited 3 s each, the init thread 1 s. All inside `DllMain`.

### 4.3 XAML hook (`N/XamlHook.cpp`)
- A worker thread looks for the window of class `LogonUI Logon Window` owned by its own PID (`EnumWindows` every 10 ms, at most 3000 tries ≈ 30 s), then installs a thread-local `SetWindowsHookExW(WH_GETMESSAGE, GetMsgProc, nullptr, <UI thread id>)`. Nothing is injected into other processes.
- The worker posts `WM_NULL` to that window every 20 ms (every 1000 ms while held or after the decoder ended); decoders also post one per new frame. So `Step()` runs on the XAML UI thread inside the hook. Input for idle detection: `WM_KEYDOWN`, `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `0x0246` (WM_POINTERDOWN).
- `Step()`:
  - Creates once a `WriteableBitmap` of canvas size (opaque black) and takes its pixel pointer via `IBufferByteAccess` `{905A0FEF-BC53-11DF-8C49-001E4FC686DA}`. A new frame is copied in when the sequence changed and `TryEnterCriticalSection` succeeds, then `Invalidate()`.
  - Walks `Window::Current().Content()` with `VisualTreeHelper` (depth ≤ 60) by `x:Name`: targets `LockLogonImage` and `SwapLockLogonImage`; `LockScreenUserControl` and `RequestCredentialUserControl` for password-screen detection; `DisplayMessageUserControl` is collected and unused. Scan interval 30 ms for the first 5 s, then 100 ms, 1000 ms once both images are found.
  - Attach: saves the image's original `Source` and `Stretch`, sets `Source` = bitmap, `Stretch = UniformToFill`; an attach that does not stick counts as an error.
  - Password screen = credential control visible with width > 0 while the lock control is not visible.
  - Hold = idle (`IdlePauseSeconds > 0` and no input that long) or (password screen and `!PlayOnPasswordScreen`) or battery hold (`g_batHold`, never set). While held, decoders sleep in 100 ms steps and audio fades out.
- Fail-safe (`TripFailSafe`): restore the original `Source`/`Stretch` on images still showing the bitmap, release the XAML objects, stop everything. Triggers: decoder died before the first frame; no first frame 30 s after the worker started; target images never found (≥ 20 misses spanning ≥ 30 s); 3 XAML errors within 30 s (at most one counted per 500 ms; none counted for 10 s after the password screen closes).

### 4.4 Video decoding (`N/VideoDecoder.cpp` `DecodeRun`)
- Media Foundation Source Reader (`MFCreateSourceReaderFromURL`), first video stream only, `MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING`.
- Hardware (`HwDecode`): D3D11 hardware device (`VIDEO_SUPPORT | BGRA_SUPPORT`, multithread-protected) with `IMFDXGIDeviceManager` and `MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS`, output `ARGB32`; each frame texture is copied to a staging texture and mapped. Software: output `RGB32`, contiguous buffer; negative stride = bottom-up.
- Fallback: D3D init failure → software in the same run; 10 samples without a usable frame before the first one → `DecoderThreadProc` reruns in software.
- Pre-downscale: when the scale to the canvas (min for fit, max otherwise) is below 0.98, MF is asked for a smaller even frame size; if refused, the native size is used.
- Scaling (`RenderFrame`, nearest neighbor, centered): cover = max scale (crop), fill = per-axis stretch, fit = min scale (black bars), center = canvas width / screen width (1:1 with the physical screen). Alpha forced opaque.
- Timing: a frame is due at `t0 + pts_ms × 100 / Speed`; sleeps of ≤ 4 ms with `timeBeginPeriod(1)`; when more than 250 ms late the clock shifts (no catch-up). End of stream → seek to 0, reset the clock, bump the loop counter (endless loop).
- Delivery: copy into `g_shared` under the lock, bump the sequence, post `WM_NULL`. The first frame is written to the first-frame cache (raw, before color adjustment).

### 4.5 GIF decoding (`GifRun`)
- WIC (`CLSID_WICImagingFactory` `{CACAF262-9370-4615-A13B-9F5539DA4C0A}`); frames converted to PBGRA and composited on a logical-screen canvas using `/imgdesc/Left`, `/imgdesc/Top`, `/grctlext/Delay` (1/100 s; ≤ 1 → 10 = 100 ms) and `/grctlext/Disposal` (2 = clear to transparent, 3 = restore previous). Transparent pixels show black. Loops forever (loop count ignored); canvas cleared each loop. Speed divides the delay. No audio for GIF.

### 4.6 Audio (`N/AudioEngine.cpp`)
- Second MF Source Reader on the same file, first audio stream, decoded to float32 PCM at the native rate, 1–2 channels.
- WASAPI shared mode on the default render endpoint (`eRender`, `eConsole`), `AUTOCONVERTPCM | SRC_DEFAULT_QUALITY`, 200 ms buffer, fed every 10 ms.
- Gain = (AudioVolume / 100)², with a 0.6 s linear fade on hold and resume. When the video loops, audio seeks to 0 (`Stop` + `Reset`); that is the only A/V sync. Audio ends when the video decoder ends.

### 4.7 Cache (`N/CacheManager.cpp`)
- First-frame cache `Cache\frame_<16 hex>.bgra`; name = FNV-1a 64 of `<mediaPath>|<w>|<h>|<fitMode>|raw`. Format: 16-byte header of uint32 `{magic 0x4C565734 "LVW4", w, h, pad}` + w × h × 4 bytes BGRA. Used only when magic and size match the current canvas.
- Written after the first decoded frame, then pruned: `frame_*.bgra` older than 30 days (mtime) deleted, only the 5 newest kept.
- `last.bgra` (same format, written via `last.tmp` + `MoveFileExW`) is read at startup (§4.2), but `SaveLastFrame` is never called, so nothing writes it. `kOverlayMagic` `0x4C4F5631` "LOV1" is defined and unused (`N/Common.h`).

### 4.8 Logging (`N/Logger.cpp`)
- Every message goes to `OutputDebugStringW` (prefix `[Sucrose.LockScreen.Core]`) and to `Logs\core.log` (UTF-8, CRLF): `HH:MM:SS.mmm up=<uptime>s pid=<pid> <msg>`, message cut at 1024 chars. File logging is always on (`SetDebugEnabled` is never called). The file is deleted before a write once it exceeds 2 MiB (no rotation).

## 5. Install and uninstall
- Install: no install-time step and no Bundle change. The DLL ships in the `Native` folder of the install (§6) and is registered the first time the user saves "Set as Lock Screen Wallpaper" (§2). Disabling only sets `Enabled = false`: the provider stays registered and the DLL still loads into LogonUI, then exits early.
- Undo (`U/App.xaml.cs` `OnStartup`), for Yes (remove data) and No (keep data):
  1. After killing Sucrose processes and refreshing the desktop: `SWML.UnregisterProvider()` (exceptions swallowed), then the install folder (`SUMI.UninstallPath`) is deleted.
  2. Yes only: delete `%APPDATA%\Sucrose` and `%ProgramData%\Sucrose\LockScreen` (`LockScreenDataPath`, `U/Manage/Internal.cs`); `CleanEmptyParentDirectory` then removes `%ProgramData%\Sucrose` if empty (never `%ProgramData%` itself).
  3. `DeleteSelf(CleanAll: Yes)`: the self-delete batch also retries `rd /s /q` on both data folders after its 3 s wait.
- Left behind: with No, all of `%ProgramData%\Sucrose\LockScreen` (config, cache, logs). If unregistration fails (silently), the HKLM keys stay and point at the deleted DLL path. Whether the unelevated Undo may delete files LogonUI wrote as SYSTEM depends on folder ACLs that no code here sets: unclear.

## 6. Build and packaging
- `N/Sucrose.LockScreen.Core.vcxproj`: `DynamicLibrary`, toolset `v145`, `VCProjectVersion` 17.0, SDK `10.0`, Unicode, `stdcpp20`, SDL checks, PCH `pch.h`, `EnableUAC=false`, Release with `MultiThreadedDLL` (dynamic CRT) and whole-program optimization. ProjectGuid `{4A17B672-005D-4D32-9D9B-F4E7502B0F33}`. Platforms Win32, x64, ARM64 × Debug, Release. Links `ole32 oleaut32 runtimeobject mfplat mfreadwrite mfuuid winmm uuid d3d11 gdi32 user32 shell32 uxtheme shlwapi advapi32 windowsapp`.
- Output `src/Sucrose/Native/<x86|x64|ARM64>/<Configuration>/Sucrose.LockScreen.Core.dll` (Win32 → `x86`); intermediates `src/Sucrose/Object/Sucrose.LockScreen.Core/<plat>/<Configuration>/`. Added to `src/Sucrose.slnx` under `/Native/` (solution x86 → project Win32). The dynamic CRT must already be installed for LogonUI to load the DLL; nothing installs it (inferred).
- `Directory.Build.targets`: the `ApplicationIcon`/`ApplicationManifest`, thread-pool and `App.<tfm>.config` blocks now skip `.vcxproj` projects.
- `.build/Sucrose.ps1` `Build-NativeBinaries`, run after all .NET projects are published and before the runtime step:
  - MSBuild from `vswhere -latest -prerelease` (`MSBuild\Current\Bin\MSBuild.exe`), else `MSBuild.exe` on PATH; if none, a warning and only the copy step.
  - `/p:Configuration=$Configuration /p:Platform=<Win32|x64|ARM64> /m /v:minimal /nologo` (x86 → Win32, unknown → x64). A failed build logs an error and the script continues.
  - `Copy-NativeBinaries`: `Sucrose.LockScreen.Core.*` files with extension `.dll` or `.pdb` from `src/Sucrose/Native/<PlatformTarget>/<Configuration>` to `<PublishDir>/<TargetFramework>/<PlatformTarget>/Native/` (the PDB ships).
- **Build concern: local Skylark packages.** `NuGet.Config` adds the sources `local-skylark`, `local-skylark-wing`, `local-skylark-standard`, `local-skylark-clipboard` = absolute paths `E:/MyProject/Skylark/Lib/Skylark/src/<Package>/bin/Release`, and `Directory.Packages.props` adds the same four as `RestoreAdditionalProjectSources`. Versions: Skylark 3.1.6.6 → 3.1.6.7, Skylark.Wing 3.2.1.7 → 3.2.1.8, Skylark.Standard 3.1.8.6 → 3.1.8.7, Skylark.Clipboard 3.1.6.1 → 3.1.6.2; unrelated bumps CsvHelper 34.1.0, CefSharp.Wpf.HwndHost 156.0.300, Microsoft.Web.WebView2 1.0.4392-prerelease. Whether these Skylark versions are on nuget.org is unclear; if not, the branch restores only on a machine with that folder layout.

## 7. Failure modes, limitations and security

**Behavior**
- Every native failure falls back to the normal Windows lock screen (fail-safe §4.3, early exits §4.2). Portal-side failures (missing media, missing DLL, failed registration, failed sync) are silent.
- Config is read once per LogonUI load, so changes apply at the next lock (inferred).
- Audio only at Speed 100, no A/V sync beyond loop restarts, and it plays on the default device before anyone signs in.
- Primary-monitor aspect, one bitmap for every image; canvas at most 1920 px wide (default 960), upscaled by XAML.
- Dead options: battery hold (`LowBatteryThreshold`, `LowBatteryLevel`, `lowBatteryOnAC`, `g_lowBat`, `g_batHold`), `ShowOnShutdown`, `BootStillSeconds`, the `last.bgra` writer, the "LOV1" overlay. FolderOrder 1 is not sequential.
- Depends on undocumented LogonUI internals (window class, `x:Name`s); a Windows update renaming them disables the feature after the 30 s fail-safe.
- Thread creation and multi-second waits inside `DllMain` (loader lock).

**Security-sensitive**
- Sucrose code runs inside `LogonUI.exe` (SYSTEM, before authentication) whenever the provider is registered, enabled or not.
- The registered DLL lives in `%LocalAppData%\Sucrose\Native`, writable by the user and by any process running as the user. Replacing it gives code execution as SYSTEM at the next lock (inferred; the code has no signature or ACL check).
- `config.json` must be writable by the unelevated Portal, so `MediaPath` is user-controlled: a SYSTEM process opens any path with Media Foundation/WIC parsers and shows the result on the lock screen to anyone at the machine.
- As SYSTEM, the DLL creates, appends, overwrites, renames and deletes files in `%ProgramData%\Sucrose\LockScreen\{Logs,Cache}`. If that folder is user-writable, links or junctions could redirect those writes and deletes (inferred; who sets its ACLs, if anyone, is unclear).
- A failed unregistration leaves an HKLM `InprocServer32` pointing into the user profile.

## 8. Porting notes for the Rust rewrite
- Never released: the feature exists only on an unmerged branch, so no installed `config.json` needs migrating (unless a build from this branch was shipped: unclear). D-007 compatibility applies to themes, not to this file.
- Keep (user-visible): the per-theme "Lock Screen" action for Video/GIF themes with the "(Selected)" marker; Enable, Fit (Cover/Fill/Fit/Center), Mute and Sync with Desktop; the 16 strings above in 25 languages as conversion input (D-015).
- Keep (behavior): no credential tile; restore the original Windows image on any failure (triggers and timings §4.3); idle hold (300 s default) and the optional password-screen hold; HW → SW decode fallback; first-frame cache for instant display; audio fade; color-adjustment ranges if they get a UI.
- Windows mechanism: a COM in-proc credential provider that LogonUI loads, plus a `WH_GETMESSAGE` hook on LogonUI's UI thread that swaps `Image.Source` of `LockLogonImage`/`SwapLockLogonImage`. In Rust this is a separate Windows-only `cdylib` with COM exports and `unsafe` FFI, so it belongs in `sucrose-platform` or an engine/FFI crate (D-011). Which crate, and whether to ship code into LogonUI at all, need a `DECISIONS.md` entry first. Whether the chosen Windows bindings expose `Windows.UI.Xaml` is unclear: check before committing to the design.
- Decoding inside LogonUI can reuse the built-in engines of D-006 (Media Foundation for video, Sucrose's own GIF decoder), with the same codec caveats.
- D-008: when the user turns the feature off, unregister the provider (or otherwise make sure nothing loads) instead of the legacy "registered but inert" state. Registration writes HKLM, so it needs an explicit elevated step, and the D-019 uninstaller needs a matching one.
- Do not carry over: a DLL registered from a user-writable folder (install it to an admin-only location and verify it); SYSTEM writes into user-writable folders; an unvalidated `MediaPath` (restrict it to library files or copy the medium into a protected folder); silent failures; dead settings; the fake "sequential" order; syncing thumbnails of non-video themes against the UI text; waits in `DllMain`; always-on file logging; absolute local NuGet sources (the rewrite builds from crates.io, D-010).
- macOS (FEATURES.md: ?): nothing in this code carries over, and the code gives no hint of a macOS mechanism. Feasibility is unclear; report it through a platform capability, never fake it (CLAUDE.md, non-negotiable 4).
- Linux (FEATURES.md: ?): nothing carries over. The lock screen belongs to each desktop's screen locker (GNOME, KDE, wlroots lockers and so on), so any support would be per desktop: unclear.
- No RAM/CPU numbers exist for this branch; measure a release build before shipping (CLAUDE.md, non-negotiable 6).
