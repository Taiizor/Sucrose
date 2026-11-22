# Legacy reference: services, Launcher, Update, Bundle, Localizer

> Inventory of the **C# app** (not the Rust rewrite), produced on 2026-11-21 by reading the sources in `src/`. Use it as the behavior spec when porting. Items marked "unclear" were not verified; when you find a mistake, fix it here.

**Path legend.** Every path below is relative to the repository root once expanded:
- `$R` = `.` (repository root)
- `$SH` = `$R/src/Shared`
- `$LIB` = `$R/src/Library`
- `$MEM` = `$LIB/Sucrose.Memory/Manage/Readonly`

**Alias legend.**
- `SSSHP` = `Shared.Space.Helper.Processor`
- `SMMB` = `Manager.Manage.Backgroundog`
- `SSDMMB` = `Shared.Dependency.Manage.Manager.Backgroundog`
- `SBMI` = `Backgroundog.Manage.Internal`

---

## 0. Cross-cutting infrastructure (all in-scope executables use it)

| Topic | Facts | Source |
|---|---|---|
| Install layout | Install root is `%LocalAppData%\Sucrose\`. Each exe sits in its own folder: `%LocalAppData%\Sucrose\Sucrose.<X>\Sucrose.<X>.exe`. The private .NET runtime is `%LocalAppData%\Sucrose\Sucrose.Runtime`. `SSSMI.Folder` = parent of the running exe's folder. | `$SH/Sucrose.Shared.Space/Manage/Internal.cs` |
| Data root | `%AppData%\Sucrose\` holds `Setting\*.json`, `Log\`, `Cache\{SignalT,Report,Bundle,Properties,CefSharp,WebView2,MpvPlayer,VlcPlayer}`, `Library\` (default library) and `Showcase\` | `$MEM/Folder.cs`, `$LIB/Sucrose.Manager/Manage/Library.cs` |
| Settings | One JSON file per area: `Hook, Aurora, Donate, Engine, Kernel, Portal, System, Update, Cycling, General, Library, Warehouse, Backgroundog, Objectionable` (all `.json`). A file is re-read whenever its mtime is newer than the last read. There is no cross-process locking. Several keys double as **cross-process state flags** (see Backgroundog, Update). | `$LIB/Sucrose.Manager/Manage/Internal.cs`, `$LIB/Sucrose.Manager/SettingManager.cs` |
| Logs | `%AppData%\Sucrose\Log\<App>-<yy.MM.dd>.log`. Line format: `[HH:mm:ss] ~ [SucroseWatchdog Thread-<rand1000-9999>/<Level>] ~ [msg]` | `$LIB/Sucrose.Manager/LogManager.cs`, `$LIB/Sucrose.Memory/Manage/Valuable/Log.cs` |
| Single instance | Named mutex plus a check that the process-name count is ≤ 1. Mutex names: `{Sucrose-Wallpaper-Engine-Launcher|Portal|Update|Property|Reportdog|Backgroundog|Live}`. Watchdog, Commandog and Undo have no mutex. | `$SH/Sucrose.Shared.Space/Helper/Instance.cs`, `$MEM/Mutex.cs` |
| Process launch | `SSSHP.Run(app, args)` uses ShellExecute with a hidden window, wraps the whole argument string in `"…"`, then calls `WaitForInputIdle`. `Runas` adds the `runas` verb (UAC). | `$SH/Sucrose.Shared.Space/Helper/Processor.cs` |
| Command protocol | Almost every cross-app launch goes through Commandog as a single argument `✔<CommandType>✖<v1>✖<v2>…`. `StartCommandChar` = U+2714, `ValueSeparatorChar` = U+2716. | `$MEM/General.cs` |
| Exception pipeline | Each app hooks WinForms ThreadException, AppDomain Unhandled, UnobservedTask and Dispatcher exceptions; first-chance logging is disabled. Each exception is logged (`<TYPE> EXCEPTION START/FINISH`, message, inner, stack), `"Exception Type"` is added to a static Hashtable, and `SSSHW.Start(app, ex, show, logPath)` is called. Only the first "show" exception escalates (`HasError` flag). | `$SH/Sucrose.Shared.Watchdog/Extension/Watch.cs`, `…/Helper/Dataset.cs`, `$SH/Sucrose.Shared.Space/Helper/Watchdog.cs`, e.g. `$R/src/Launcher/Sucrose.Launcher/App.xaml.cs` |
| HTTP | Shared `HttpClient` with infinite timeout and `User-Agent` = General.json `UserAgent` (default `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit`). **Certificate validation is disabled** and SSL2/SSL3/TLS1.0–1.3 are allowed. `ClientGitHub` adds `Authorization: Bearer <Objectionable.PersonalAccessToken>`. `Security.Apply()` also sets a global `ServerCertificateValidationCallback` that always returns true. | `$SH/Sucrose.Shared.Dependency/Manage/Internal.cs`, `$SH/Sucrose.Shared.Space/Helper/Security.cs` |
| Runtime bootstrap (`SSDHR.Configure`, RELEASE builds only) | Sets `DOTNET_ROOT[(x86)|(arm64)]` to `Sucrose.Runtime`, `DOTNET_MULTILEVEL_LOOKUP=0`, `DOTNET_ROLL_FORWARD=LatestMajor`, and prepends Runtime and the exe dir to PATH. | `$SH/Sucrose.Shared.Dependency/Helper/Runtime.cs` |
| GPU preference (`SSDHG.Configure`) | [WIN] Registry `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, value name = exe path, data `GpuPreference=<n>;`. `n` comes from Engine setting `GraphicPreference` (Default/PowerSaving/HighPerformance). | `$SH/Sucrose.Shared.Dependency/Helper/Graphic.cs`, `$SH/Sucrose.Shared.Space/Extension/GraphicPreference.cs` |
| URLs / IDs | Soferity API `https://sucrose.soferity.com/v8/...`; GitHub `Taiizor/Sucrose` (branch `develop`) and `Taiizor/Store`; Discord app id `1126294965950103612` | `$MEM/Url.cs`, `$MEM/Soferity.cs`, `$MEM/GitHub.cs`, `$MEM/Hook.cs` |

**Shared imports per executable** (from each `.csproj`, `<Import Label="Shared">`):

| Exe (OutputType) | Shared projects | Libraries | Notable NuGet |
|---|---|---|---|
| Launcher (WinExe) | Zip, Live, Space, Theme, Signal, Discord, Launcher, Watchdog, Dependency | Signal, Manager, Resources | DiscordRichPresence, Skylark.Wing |
| Backgroundog (**Exe**, console) | Live, Space, Theme, Watchdog, Dependency | Pipe, Signal, Manager, Transmission | NAudio, Playhead, Vortice.DXGI, MathNet.Numerics |
| Commandog (Exe) | Live, Theme, Space, Watchdog, Dependency | Manager | TaskScheduler |
| Reportdog (Exe) | Core, Live, Space, Theme, Watchdog, Dependency | Pipe, Signal, Manager | — |
| Watchdog / Undo (WinExe) | Core, Live, Space, Theme, Watchdog, Dependency | Manager, Resources | — |
| Property (WinExe) | Live, Theme, Space, Watchdog, Dependency | Manager, Resources | WPF-UI, HandyControls |
| Update (WinExe) | Core, Live, Theme, Space, SevenZip, Watchdog, Dependency | Manager, Resources | WPF-UI, Downloader, SharpCompress |
| Bundle (WinExe, **net48**) | none | none | Skylark(.Wing/.Standard), SharpCompress, Costura.Fody |
| Localizer (Exe) | none | none | CsvHelper, Skylark |

---

## 1. Sucrose.Backgroundog: performance governor and system-data provider

**Purpose.**
- Watches system conditions and pauses, closes or resumes the live wallpaper engine.
- Collects system telemetry (CPU, GPU, RAM, disks, network, battery, audio, BIOS and so on) and streams it to web engines.

**How it is started.** Always via Commandog `✔Backgroundog✖<Backgroundog.exe>`, which calls `Run(exe, "")`. Callers:
1. `Shared.Live.Helper.Run.Start()` when a wallpaper starts and Backgroundog.json `PerformanceCounter` is true (default true) (`$SH/Sucrose.Shared.Live/Helper/Run.cs`).
2. The engine general timer, `SSEHR.Control()`, restarts it if it is not running and `PerformanceCounter` is true (`$SH/Engine/Sucrose.Shared.Engine/Helper/Run.cs`).
3. Portal Performance settings (`…/PerformanceSettingViewModel.cs:1294`).

It is killed by RestartLive, Temp/Reset/Import/Export, and Launcher Close (when AppExit is on).

**Lifetime and loop** (`$R/src/Project/Sucrose.Backgroundog/App.cs`, `Helper/Initialize.cs`, `Manage/Internal.cs`):
- `Main`: configure the GPU preference and runtime, set culture. If single instance, call `Security.Apply()` then `Initialize.Start()`. Then loop: GC every 1 s while `Exit`.
- `Initialize.Start` (`Extension/Windows.cs`):
  - Subscribes to [WIN] `Microsoft.Win32.SystemEvents.SessionSwitch` and `PowerModeChanged`.
  - Subscribes to [WIN] a Skylark.Wing `EventSubscriptionManager` with a `WindowsListener` exposing OnLogon, OnLogoff, OnDisplayLock, OnDisplayUnlock, OnStart/StopScreenSaver. This is likely the COM SENS `ISensLogon` interface; inferred from the method set, the implementation is inside the closed NuGet.
  - Sets the initial lock state by checking whether the foreground window's process is `LockApp` ([WIN] `GetForegroundWindow` / `GetWindowThreadProcessId`).
  - Starts a `System.Threading.Timer`: due 3000 ms, period **100 ms**.
  - Resets Backgroundog.json `ClosePerformance=false` and `PausePerformance=false`.
- On every 100 ms tick:
  - `Specification.Start()` fires guarded background tasks (see data collection below).
  - If `Processing` is set: when a live engine runs and no condition is active, run `Performance.Start()` (trigger checks). Else if a condition is active, run `Condition.Start()` (recovery checks). Else run `Attempt.Start()`.
  - Then wait 3000 ms before the next evaluation.
- `Attempt.Start()` (`Helper/Attempt.cs`): checks once per second for 5 s whether any engine process is running. If none appears it sets `Exit=false`, disposes the media session, resets both flags, and the process exits.

### 1.1 Performance conditions (`Helper/Performance.cs` triggers, `Helper/Condition.cs` recovery)

**Actions** (`SSDEPT` = `PerformanceType`: `Close`, `Pause`, `Resume`; `Resume` disables the rule). Evaluation order (first match wins): Lock, Focus, Sleep, Memory, Remote, Battery, Console, Graphic, Network, Session, Virtual, Processor, FullScreen, ScreenSaver, BatterySaver.

"Debounce" is the number of consecutive 1 s polls the predicate must hold, written trigger / recover.

| Category | Action key (Backgroundog.json) and default | Trigger predicate | Debounce (trigger/recover) | Threshold keys (default, clamp) | Source of signal |
|---|---|---|---|---|---|
| Lock | `LockPerformance` = **Close** | `WindowsLock` | 0/0 | — | [WIN] SessionSwitch SessionLock/Unlock, SENS OnDisplayLock/Unlock, LockApp foreground |
| Focus | `FocusPerformance` = Resume | desktop **not** focused | 5/3 | — | Skylark `Desktop.IsDesktopBasic()`/`IsDesktopAdvanced()` (foreground is Progman/WorkerW, inferred), every 1 s |
| Sleep | `SleepPerformance` = **Close** | `WindowsSleep` | 0/0 | — | [WIN] PowerModeChanged Suspend/Resume |
| Memory | `MemoryPerformance` = Resume | `MemoryLoad >= MemoryUsage` (and >0) | 5/3 | `MemoryUsage` 80 % (0–100) | [WIN] `GlobalMemoryStatusEx` |
| Remote | `RemotePerformance` = Resume | `WindowsRemote` (SessionSwitch RemoteConnect/Disconnect) **or** a process whose `<name>.exe` is in a 52-entry remote-tool list (mstsc, anydesk, teamviewer, rustdesk, vnc*, …) | 5/3 | — | `$SH/Sucrose.Shared.Space/Helper/Remote.cs`, `Extension/Remote.cs`; polled every 2 s only when the rule is active |
| Battery | `BatteryPerformance` = Resume | battery present && (`PowerLineStatus != Online` **or** `ACPowerStatus != "Online"`) && `EstimatedChargeRemaining <= BatteryUsage` | 5/3 | `BatteryUsage` 50 % (0–100) | WMI and power APIs (see 1.2) |
| Console | `ConsolePerformance` = **Close** | `!WindowsConsole` | 0/0 | — | SessionSwitch ConsoleConnect/Disconnect |
| Graphic | `GraphicPerformance` = Resume | `GPU.Now >= GraphicUsage` | 5/3 | `GraphicUsage` 70 % | PDH GPU Engine counters |
| Network | `NetworkPerformance` = Resume | `Ping >= PingValue`, else `Upload >= UploadValue`, else `Download >= DownloadValue` (bytes/s after unit conversion). The sub-type is remembered (`NetworkPerformanceType` Ping/Upload/Download) for recovery. | 5/3 | `PingValue` 100 ms (0–1000); `UploadValue` 800 + `UploadType` Kilobyte; `DownloadValue` 10 + `DownloadType` Megabyte (0–99999999) | ping and PDH (see 1.2) |
| Session | `SessionPerformance` = **Close** | `!WindowsSession` | 0/0 | — | SessionSwitch Logon/Logoff and SENS OnLogon/OnLogoff |
| Virtual | `VirtualPerformance` = Resume | a process from a 40-entry VM/emulator list (vmware, virtualbox, qemu, vmwp, bluestacks, nox, WindowsSandbox, sandboxie, generic `player.exe`, …) | 5/3 | — | `$SH/Sucrose.Shared.Space/Helper/Virtual.cs`; polled every 2 s only when active |
| Processor | `ProcessorPerformance` = Resume | `CPU.Now >= ProcessorUsage` | 5/3 | `ProcessorUsage` 70 % | PDH `Processor\% Processor Time\_Total` |
| FullScreen | `FullScreenPerformance` = Resume | foreground window covers any monitor rect (skipped while the desktop is focused) | 5/3 | — | [WIN] `GetForegroundWindow`, Skylark `Screene.Screens` / `FullScreen.IsFullScreen(hwnd, rcMonitor)`, every 1 s |
| ScreenSaver | `ScreenSaverPerformance` = **Pause** | `ScreenSaver.IsRunning()` **or** SENS OnStartScreenSaver | 0/0 (recovery requires `!IsRunning || !SENS flag`) | — | Skylark `ScreenSaver.IsRunning` (probably `SPI_GETSCREENSAVERRUNNING`, unclear) |
| BatterySaver | `BatterySaverPerformance` = Resume | battery present && `EnergySaverType == On` | 5/3 | — | Skylark `Power.GetEnergySaverState()` [WIN], exact API unclear |

**Other keys.**
- `PausePerformanceType` = **Light** (or Heavy).
- `CommunicationType` = **Transmission** (Pipe/Signal/Transmission).
- `NetworkAdapter` / `GraphicAdapter`: Backgroundog auto-fills these with the first enumerated adapter if empty or invalid.
- `PingType` = "Bing" (hosts: Bing, Baidu, Yahoo, Google, Yandex, Microsoft, Cloudflare, DuckDuckGo; `$SH/Sucrose.Shared.Space/Helper/Network.cs`).
- Defaults: `$SH/Sucrose.Shared.Dependency/Manage/Manager/Backgroundog.cs`, `$LIB/Sucrose.Manager/Manage/Backgroundog.cs`. Key names: `$LIB/Sucrose.Memory/Manage/Constant/Backgroundog.cs`.

**On trigger** (`Performance.Lifecycle`):
- **Close**:
  - Sets `ClosePerformance=true`.
  - `Shared.Live.Helper.Kill.Stop()` kills all engine exes and the Aurora app process (`Aurora.json AppProcessName`), resets the wallpaper to default, refreshes the desktop, and clears `AppProcessName` (`$SH/Sucrose.Shared.Live/Helper/Kill.cs`).
- **Pause**:
  - Sets `PausePerformance=true`.
  - **Light**: nothing else. Engines poll `SMMB.PausePerformance` in their own timer and pause themselves. For example the WebView engine calls [WIN] `DebugActiveProcess` on its D3D intermediate-window process (`$SH/Engine/Sucrose.Shared.Engine.WebView/Helper/Web.cs`, `View/*.xaml.cs`).
  - **Heavy**: also [WIN] `ntdll!NtSuspendProcess` on the live engine, on all `msedgewebview2` / `CefSharp.BrowserSubprocess` processes whose command line contains "Sucrose" (WMI command-line lookup), and on Aurora app processes (`$SH/Sucrose.Shared.Space/Extension/Lifecycle.cs`).
- Writes the signal `Launcher.sgnl {Icon=true}`, which makes the tray icon grayscale.

**On recovery** (`Condition.Lifecycle`):
- Close: clear the flag, then `Live.Run.Start()` restarts the wallpaper.
- Pause: clear the flag, then `NtResumeProcess` for Heavy.
- Writes `Launcher.sgnl {Icon=false}`.
- If the paused engine or app has exited, the condition is reset.

`Live.Run.Start()` refuses to start a wallpaper while `ClosePerformance` or `PausePerformance` is set and Backgroundog is running. The Portal, the tray, Cycling and the Library cards all check the same flags.

### 1.2 Data collected (`Helper/Specification.cs`, serialized by `Extension/Data.cs`)

Each item runs as a fire-and-forget Task guarded by a bool flag.

| Group | JSON fields sent | Source / API | Rate |
|---|---|---|---|
| Date | Day, Hour, Year, Month, State, Minute, Second, Kind, DayOfYear, Millisecond, DayOfWeek | `DateTime.Now` | every 100 ms tick |
| Bios | Name, State, Caption, Version, Description, ReleaseDate, Manufacturer, SerialNumber, CurrentLanguage | [WIN] WMI `Win32_BIOS` | once |
| Motherboard | Name (= Manufacturer + Product), State, Product, Version, Manufacturer | [WIN] WMI `Win32_BaseBoard` | once |
| Processor | Max, Min, Now (_Total %, running min/max), Core, Thread, Name, Type (counter type), ProcessorCount, Cores[] {Index, Name `Core #n`, Now, IsMax, IsMin, Type, TypeText}, CoresNow (max across cores), CoresMax, CoresMin | [WIN] WMI `Win32_Processor` (once; also writes System.json `ProcessorInterfaces`); [WIN] PDH `Processor\% Processor Time` for `_Total` and per core | 1 s |
| Memory | Name "Total Memory", State, MemoryLoad (%), MemoryUsed, MemoryAvailable (GiB), VirtualName, VirtualMemoryLoad, VirtualMemoryUsed, VirtualMemoryAvailable (page file) | [WIN] `GlobalMemoryStatusEx` | 1 s |
| Graphic | Max, Min, Now (%), Name, State, Manufacturer (AMD / Intel / NVIDIA / Microsoft by VendorId) | Adapter list: [WIN] DXGI `CreateDXGIFactory1`/`EnumAdapters1` via Vortice, once, written to System.json `GraphicInterfaces`. Usage: [WIN] PDH `GPU Engine\Utilization Percentage` summed over instances ending in `engtype_3D` that contain the adapter LUID `0x{High:X8}_0x{Low:X8}`, clamped 0–100, instances re-synced each poll (`Enumerators/VorticeAdapterEnumerator.cs`, `Extension/Graphic.cs`) | 1 s |
| Network | Host, Name, Ping (ms), State, Online, Upload, Download, Total (bytes/s), PingData (ping struct JSON), Upload/Download/TotalData (auto-unit struct), Format*Data strings, PingAddress `"<ip> (<host>)"` | Interfaces: [WIN] PDH category `Network Interface` instance names, once, written to System.json `NetworkInterfaces`. Throughput: PDH `Bytes Sent/sec`, `Bytes Received/sec`. Online: DNS resolve of the host list. Ping: Skylark ICMP send, 1000 ms timeout, address cached until PingType changes | throughput 1 s; ping ~1 s + RTT |
| Storage | State; Drivers[] {Size, Name, Caption, DriveType, MediaType, FreeSpace, UsedSpace, FileSystem, VolumeName, Description, Compressed, VolumeSerialNumber, SupportsDiskQuotas, SupportsFileBasedCompression, MaximumComponentLength, Size/Free/UsedSpaceData, Format* strings}; LogicalDrivers[] and PhysicalDrivers[] {Name, Read, Write (B/s), ReadData, WriteData, FormatRead/WriteData} | [WIN] WMI `Win32_LogicalDisk` (10 s); [WIN] PDH `LogicalDisk` / `PhysicalDisk` `Disk Read/Write Bytes/sec` (1 s) | 10 s / 1 s |
| Battery | Name, State, Status, Chemistry, SavingMode, BatteryFlag, Description, LifePercent, SaverStatus, FullLifetime, ACPowerStatus, DesignVoltage, LifeRemaining, ChargeStatus, BatteryLifeTime, PowerPlanType, EstimatedRunTime, EnergySaverType, PowerLineStatus, BatteryLifePercent, BatteryFullLifeTime, EstimatedChargeRemaining | [WIN] WMI `Win32_Battery`; Skylark `Utility.Power` (likely `GetSystemPowerStatus` / energy-saver API, unclear); WinForms `SystemInformation.PowerStatus`; Skylark `Plan.GetActivePowerSchemeGuid` (likely `PowerGetActiveScheme`) | 2 s |
| Audio (only when `AudioRequired`) | State, Title, Artist, Subtitle, AlbumTitle, AlbumArtist, TrackNumber, AlbumTrackCount, MediaType, SourceAppId, RepeatMode, PropsValid, PlaybackRate, PlaybackMode, PlaybackState, ShuffleEnabled, LastPlayingFileTime, PositionSetFileTime, EndTime/Position/StartTime/MinSeekTime/MaxSeekTime (ms), ThumbnailString (base64), Data / DataLeft / DataRight (128 doubles each) | Media session: [WIN] Playhead NuGet (`NowPlayingSessionManager`, `MediaPlaybackDataSource`), event-driven on session or data change; the underlying Windows API is unclear. Thumbnail: base64 of the stream, cropped to PNG 233×233 at (34,1) when pixel (0,0) is transparent and `!IsCobalt()` (`Extension/Thumbnail.cs`). Spectrum: [WIN] WASAPI loopback (NAudio), see below | session init re-checked every 2 s; spectrum up to ~60 Hz |

**Audio visualizer algorithm** (`Extension/AudioVisualizer.cs`):
- Default render device; capture is re-created on `OnDefaultDeviceChanged` via `IMMNotificationClient`.
- Accepts float32, PCM16 or PCM32 input.
- Ring buffer of 4096 samples per channel; Hann window; MathNet FFT with NoScaling.
- 128 log-spaced bands from 20 Hz to min(16 kHz, Nyquist), peak magnitude per band. Mono = |(L+R)/2|.
- Each band is scaled by a tilt of +2 dB/oct, multiplied by 2.5 / reference (reference = Σwindow/2), minus a 0.01 noise floor, then clamped to 0–1.
- Envelope: attack 0.35 / release 0.06; ±1-bin neighbour average; emits at most every 16 ms.

### 1.3 How the data reaches engines

**Producer side** (`Specification.cs`, tail): each tick, when no condition is active and the matching `*Required` flag is set, one payload `{Bios, Date, Audio, Memory, Battery, Graphic, Network, Storage, Processor, Motherboard}` (JObjects, `Formatting.None`) is sent per enabled channel. A new send starts once the previous one finishes, giving up to ~10/s.

**Channels:**
- **Transmission** (default): newline-delimited JSON over TCP to `127.0.0.1:<TransmissionPort>`.
  - The engine is the server. It picks a free port (`SSSHP.Available`) and writes `TransmissionRequired=true` and `TransmissionPort` to Backgroundog.json.
  - Client: 5 s connect timeout, keepalive 10/2 s, send timeout 2.5 s, 3 retries with exponential backoff.
  - Files: `$LIB/Sucrose.Transmission/*`.
- **Pipe**: [WIN] named pipe `\\.\pipe\Sucrose.Wallpaper.Engine.Backgroundog.Pipe`, newline-delimited JSON.
  - Backgroundog is the client (Out); the engine is the server (In, max 10 instances, 5 s accept loop, 15 s read timeout).
  - Files: `$LIB/Sucrose.Pipe/*`.
- **Signal**: file drop. Each send writes `%AppData%\Sucrose\Cache\SignalT\Backgroundog-<guid>.sgnl`.
  - The consumer uses a FileSystemWatcher (Created), reads the file, and deletes it 3 s later.
  - Files: `$LIB/Sucrose.Signal/SignalT.cs`.

**Consumer side:** only the WebView and CefSharp engines consume this data (`Helper/Web.cs StartCompatible`; the Signal service is compiled `#if LIVE_WEBVIEW || LIVE_CEFSHARP`). They set `AudioRequired` when SucroseCompatible.json defines `SystemAudio`. They store each group as JSON text and call the JS format strings from `SucroseCompatible.json`: `SystemBios/Date/Audio/Memory/Battery/Graphic/Network/Storage/Processor/Motherboard`, plus `LoopMode/ThemeType/SystemTheme/ShuffleMode/StretchMode/VolumeLevel` (`$SH/Engine/Sucrose.Shared.Engine/Helper/Compatible.cs`, `$SH/Sucrose.Shared.Theme/Helper/Compatible.cs`). Every engine resets all four `*Required` flags to false on exit.

---

## 2. Sucrose.Commandog: command trampoline

**Purpose and start.** A short-lived console exe started (hidden) by nearly every app. It parses the single argument and exits.

**Argument format** (`$R/src/Project/Sucrose.Commandog/Helper/Arguments.cs`):
- Args are joined with spaces and must start with `✔` and contain `✖`.
- `Name✖v1✖v2…` is split on `✖`; at least 2 parts are required.
- `Name` is matched case-insensitively against `CommandType` (`$SH/Sucrose.Shared.Dependency/Enum/CommandType.cs`).
- Values are strings; bool/int/enum are parsed as needed (`Helper/Parse.cs`).

| CommandType | Values | Action | Known callers |
|---|---|---|---|
| Log | path | ShellExecute path | only a commented-out call in Watchdog dialog |
| Kill | exe | kill all processes by name | no caller found |
| Live | engine exe | run | `Shared.Live/Helper/Run.cs` |
| Test | any | prints typed values to console | none |
| Temp | dir, app | kill Undo/Portal/Update/Launcher/Property/Watchdog/Reportdog/Backgroundog and the engine; wait 3 s; delete dir (or create it if missing); wait 1 s; run `Portal SystemSetting`, then `app` (`Space/Helper/Temporary.cs`) | Portal System settings (log, cache, store-temp, library folder) |
| Wiki, Report, Official, Repository, Discussions, Premium, Publish, QuotingGoogle, QuotingTranslate, Bundle | url | ShellExecute (opens browser) | Portal help/about, ThemeReport, Watchdog Help, Launcher ReportBox, Update (download URL fallback) |
| Reset | app | kill all; delete `%AppData%\Sucrose\Setting\*.json`; relaunch Portal(SystemSetting) and app (`Space/Helper/Reset.cs`) | Portal System |
| Update | UpdateExe [, "False"] | run Update; a second value means auto/silent mode | Launcher tray and auto-update, Portal cards |
| Import / Export | dir, app | kill all; copy `*.json` dir→Setting or Setting→dir; relaunch (`Space/Helper/Import.cs`, `Export.cs`) | Portal System |
| Startup | name, exe, bool | [WIN] Skylark `WindowsStartup.SetStartup`. The DLL contains `Software\Microsoft\Windows\CurrentVersion\Run`, so HKCU Run is likely. | Portal General, RunStartup=1 |
| StartupM | name, exe, bool | [WIN] `WindowsStartupMachine` (HKLM Run? unclear) | no caller found |
| StartupP | exe, bool | [WIN] `WindowsStartupPriority`; mechanism unclear; launched elevated (`Runas`) | Portal General, RunStartup=2 |
| Scheduler | Create✖exe / Enable / Disable / Delete | [WIN] Task Scheduler (TaskScheduler NuGet): folder `\Sucrose\`, task "Autorun for Sucrose", description "Sucrose Wallpaper Engine", LogonTrigger for the current user, no battery stop, no time limit (`Helper/Scheduler.cs`) | Portal General, RunStartup=3 (0 = none) |
| Cycyling | Unknown | stop engine, kill WebView/Cef subprocesses, `Run.Start()`; after 1.5 s start again if not running | `Space/Helper/Cycyling.cs Change()` |
| RestartLive | Unknown | same as Cycyling, plus kills Backgroundog | tray Reload, engine event handler and crash recovery (`$SH/Engine/Sucrose.Shared.Engine/Event/Handler.cs`, `Helper/Crashing.cs`) |
| Property | PropertyExe | run | tray "Customize Wallpaper" |
| PropertyA | PropertyExe, themeFolder | run with the theme folder name as argument | Portal LibraryCard |
| Interface | PortalExe [, ArgumentCommandType] | run Portal (optional page: OtherSetting, DonateSetting, SystemSetting, GeneralSetting, PersonalSetting, WallpaperSetting, PerformanceSetting) | Launcher open/settings |
| Watchdog | WatchdogExe, base64 | run (only with exactly 2 values) | `Space/Helper/Watchdog.cs` |
| Reportdog / Backgroundog | exe | `Run(exe, "")` | Launcher, Portal, Live.Run, engines |
| Versioning | dir, exclude, bool | recursive delete except a subfolder named `exclude` (`Space/Helper/Delete.cs`) | only `Shared.Launcher/Command/Version.cs`, which has no caller |

---

## 3. Sucrose.Launcher: entry point and tray (`$R/src/Launcher/Sucrose.Launcher/App.xaml.cs`, `$SH/Sucrose.Shared.Launcher/*`)

**Started by:** Bundle after install, Desktop and Start-menu shortcuts, the startup entry (Run key / scheduled task / priority), and the Temp/Reset/Import/Export relaunch.

**Startup flow:**
1. Set language; `ShutdownMode=OnExplicitShutdown`.
2. If single instance (mutex + count): `TrayIconManager.Start()`, then the Signal channel `Launcher.sgnl`, then `Discord.Initialize()`.
3. If another instance is running: open Portal through Commandog (`Interface`) and exit.

**`TrayIconManager.Start`** (`Manager/TrayIconManager.cs`):
- Creates a [WIN] WinForms NotifyIcon. Text is "Sucrose Wallpaper Engine"; visibility comes from General.json `AppVisible` (default true).
- `Engine.Command(false)`:
  - Wallpaper not running and `Library.Selected` folder exists: start the wallpaper, and also open Portal if the tray is hidden.
  - Nothing selected: open Portal.
  - Wallpaper already running: open Portal.
- Starts Reportdog via Commandog.
- If Update.json `Auto` (default true): runs `✔Update✖<exe>✖False` (auto mode).
- Left double-click opens Portal. Right-click rebuilds the menu each time, with a dark/light renderer, RTL support and rounded corners.

**Tray menu items** (English strings from `$LIB/Sucrose.Resources/Locales/Launcher/Launcher.EN.xaml`):
1. **Open Sucrose**: Portal. Disabled while Portal runs.
2. *(separator)* If an engine is running:
   - **Close Wallpaper**: stop the engine.
   - **Reload Wallpaper**: RestartLive.
   - **Change Wallpaper**: only if `Cycyling.Check(false)` (cycling active and another compatible theme exists). Picks the next theme by `TransitionType` Random/Sequential, skipping the Exclusion list, then runs `✔Cycyling`.
   - **Customize Wallpaper**: only when properties exist. Web needs a theme `SucroseProperties.json`. Gif/Video need the theme file or `%AppData%\Sucrose\Cache\<VlcPlayer|MpvPlayer|CefSharp|WebView2>\SucroseProperties.json` for the selected engine. YouTube needs the Cef/WebView cache file. Disabled while Property runs.
   - While paused (`PausePerformance` set and Backgroundog running), all four are disabled with a " (Paused)" suffix.

   Else, if the library is configured and the selected theme's `AppVersion <=` the app version: **Open Wallpaper**, disabled " (Closed)" when `ClosePerformance` is set.
3. *(separator)*
   - **Settings**: Portal `GeneralSetting`. Disabled while Portal runs.
   - **Report Issue**: Report box with **Create Log File** and **Report an Issue** (opens GitHub issues/new/choose). Create Log File shows a SaveFileDialog (default name `sucrose_log_yyyyMMdd_HHmmss.zip`, on the Desktop) and zips `%AppData%\Sucrose\Log` and `\Setting`, excluding `Objectionable.json` (`Helper/Create.cs`).
   - **Send Feedback**: always disabled (`FeedbackBox=false`, dialog commented out).
   - **Check for Updates**: disabled while Update runs, labelled "Checking for Updates", or "Preparing Update" when Update.json `State=true`.
4. *(separator)* **Close Applications** if General.json `AppExit` is true (default), else **Exit**.
   - With AppExit: stop the engine (or its subprocesses) and kill Undo, Portal, Update, Property, Watchdog, Commandog, Reportdog and Backgroundog.
   - Then release the tray and kill self (`Command/Close.cs`).

**Signal handling** (`$SH/Sucrose.Shared.Signal/Services/LauncherSignalService.cs`): `{Hide, Show, Release, Icon}`. Hide/Show toggle the tray, Release disposes it, and `Icon=true` switches to the grayscale tray icon.

**Discord Rich Presence** (`$SH/Sucrose.Shared.Discord/Extension/Hook.cs`, `Manage/Internal.cs`; settings `$LIB/Sucrose.Manager/Manage/Hook.cs`):
- A 5 s timer checks `DiscordConnect` (default true) and whether `Discord.exe` or `DiscordPTB.exe` is running. If so it initializes the client and sets presence once; otherwise it clears the presence.
- If `DiscordRefreshConnect` (default true), a refresh timer re-sets the presence every `DiscordRefreshDelay` seconds (60–3600, default 60).
- Presence fields:
  - `Details` = "Wallpaper Engine"; `StatusDisplay=Details`.
  - `State` = random `StatementText0..48`.
  - `Timestamps` Start/End = `Timestamps.Now` captured at startup.
  - Buttons: "Browse" → `https://github.com/Taiizor/Sucrose`; "Download" → `https://apps.microsoft.com/detail/XP8JGPBHTJGLCQ?launch=true&mode=full`.
  - `LargeImageKey` = `logo-3840x3840` (`Locales/Discord/Discord.xaml`); `LargeImageText` = random `LargestText0..23`; `SmallImageKey` = random `SmallestImage0..36`; `SmallImageText` = random `SmallestText0..23`.

---

## 4. Sucrose.Watchdog: crash dialog and report writer (`$R/src/Project/Sucrose.Watchdog/App.xaml.cs`, `View/*ErrorMessageBox.xaml.cs`)

**Started by:**
1. Any app's exception handler calls `SSSHW.Start` (only if Commandog.exe and Watchdog.exe exist).
2. That runs Commandog `✔Watchdog✖<Watchdog.exe>✖<base64>`.
3. The base64 payload is `App✖<SerializableException JSON>✖<Show bool>✖<log path>`. Exception.Data carries the accumulated Dataset, e.g. "Exception Type", and `WatchText` / `WatchSource` help text and link. Update sets "Solve Problem" plus the installer URL.

**Behaviour:**
- Software rendering only.
- Builds `ThrowExceptionData` with these fields:
  - Ids: Id (new GUID), Sid (MD5-GUID of `User-Model-Manufacturer`), AppId (MD5-GUID of app name).
  - App: AppName, AppVersion, AppFramework, AppArchitecture, Exception (JObject).
  - User and device: UserName, DeviceModel, ManufacturerBrand.
  - Culture: CultureCode, CultureName, CultureDisplay (from [WIN] `GetUserDefaultUILanguage`).
  - OS: OperatingSystem, OperatingSystemBuild (with UBR), IsServer, IsWorkstation.
  - Architecture: ProcessArchitecture, ProcessorArchitecture, OperatingSystemArchitecture.
- Writes `%AppData%\Sucrose\Cache\Report\<Id>.json`. This happens **regardless of the opt-out setting**.
- Kills the crashed app (unless it is Watchdog itself).
- If Show: shows the dark or light dialog (by ThemeType), with a system "Hand" sound.
  - Buttons: Show/Hide details (raw exception vs. message), Help (WatchSource or the GitHub wiki, via Commandog `Wiki`), Close.
  - If WPF fails: [WIN] `MessageBox(MB_OK|MB_ICONERROR|MB_SYSTEMMODAL)` with a VC++ / .NET hint.
- Strings: `Locales/Watchdog/Watchdog.EN.xaml`.

---

## 5. Sucrose.Reportdog: telemetry and exception uploader (`$R/src/Project/Sucrose.Reportdog/*`)

**Start and lifetime.**
- Started by Launcher at tray start and by Portal Other settings, via Commandog `Reportdog`.
- Runs only if it is the single instance **and** (General.json `ExceptionData` or `TelemetryData`; both default true).
- Exits when no other process whose name contains "Sucrose" has been seen for 5 s.

**Behaviour** (`Helper/Initialize.cs`):
- **Exception upload**: scans `%AppData%\Sucrose\Cache\Report` (1 s apart per file) and watches it (Created, +1 s). Each file goes to `POST https://sucrose.soferity.com/v8/Exception/Throw/<guid>` with the ThrowExceptionData JSON. The file is deleted after success and also when it cannot be parsed. Gated by `ExceptionData` and DNS reachability.
- **Online ping**: `POST …/v8/Telemetry/Online/<guid>` `{AppVersion, Time}`. Sent at start with Time=1, then every random 120–150 s with Time = interval seconds. Gated by `TelemetryData`.
- **Analytic**: `POST …/v8/Telemetry/Analytic/<guid>` at start and every 10 min until the first success, then the timer is disposed. The payload holds about 90 fields:
  - App and preferences: AppExit, AppVisible, RunStartup, AppVersion, framework, architecture, update Auto/AutoType/Module/Server/Channel/Extension, engine per wallpaper type, input, screen and display settings, stretch, volume settings, loop and shuffle, developer mode/port, store and library settings, **LibraryLocation path**, cycling, donate/advertising, Discord, PerformanceCounter, every *Performance action plus PausePerformanceType and CommunicationType.
  - Device and identity: selected Graphic/Network adapter names, WMI lists of GPUs, NICs and CPUs, cores, processors, TotalMemory, OS build, culture, **UserName**, **UserIdentifier (Windows SID)**, **UserIdentifying (Win32_ComputerSystemProduct.IdentifyingNumber)**, DeviceModel, Manufacturer.
  - Model file: `$SH/Sucrose.Shared.Space/Model/AnalyticTemeletryData.cs`.
- `<guid>` = MD5-GUID of `UserName-Model-SID` (`Space/Helper/User.cs`, `Unique.cs`).

**Related telemetry outside Reportdog.**
- Update posts `…/v8/Telemetry/Update/<guid>` `{AppVersion, UpdateAutoType}`.
- Download, Search, Feedback and Report models also exist (`Space/Model/*`, Portal-side, out of scope).

**Opt-out**: General.json `TelemetryData` and `ExceptionData` (`$LIB/Sucrose.Manager/Manage/General.cs`).

---

## 6. Sucrose.Property: wallpaper property editor (`$R/src/Project/Sucrose.Property/*`)

**Started by:**
- Commandog `Property` (tray; no argument: uses Library.json `Selected`).
- Commandog `PropertyA` (Portal LibraryCard; argument = theme folder name).
- Single instance.

**Resolve** (`App.xaml.cs`):
- Picks `PropertiesType`:
  - Base: Web theme with `SucroseProperties.json`.
  - Otherwise by the engine chosen for the type (Gif/Video: Vlc/Mpv/Cef/WebView; YouTube: Cef/WebView). Source is the theme's file or the engine default `%AppData%\Sucrose\Cache\<Engine>\SucroseProperties.json`.
- The working copy is `%AppData%\Sucrose\Cache\Properties\<Theme><Type>`, created by copying the source. If parsing fails the copy is deleted and an error is thrown.

**UI** (`View/MainWindow.xaml.cs`):
- WPF-UI FluentWindow. Backdrop comes from Portal.json `BackdropType`; theme is auto or ThemeType.
- Positioned bottom-right, or next to the taskbar per [WIN] its anchor and coordinates, DPI-aware and RTL-aware.
- One control per `PropertyList` entry:

| type | fields |
|---|---|
| label | value |
| button | value (click sends the key) |
| slider | min, max, step, value |
| textbox | value |
| checkbox | value |
| dropdown | value index, items, valuetext |
| numberbox | min, max, places, value |
| passwordbox | value |
| filedropdown | desc, value, title, folder, filter (`a|b`, `*` = all files), files. Open copies the chosen file into `<theme>\<folder>` after access/lock checks; Delete removes it. |
| colorpicker | value hex (HandyControl popup) |

  Common fields: name, text, type, help tooltip. Models: `$SH/Sucrose.Shared.Theme/Model/*.cs`.
- Text is localized via the JSON `PropertyLocalization` map (culture → key → text, falling back to the first culture) or built-in `Property.Localization.*` keys.
- Window buttons:
  - **Restore Default**: re-copy the source; if live, also push it as a delta.
  - **Refresh Interface**: reload the controls.
  - **Delete Copy**: stop the engine, delete and re-copy, restart the engine.

**Live apply** (`Helper/Properties.cs`):
- Every change rewrites the full working copy.
- If the edited theme is the selected one and an engine runs, a **delta file** `Cache\Properties\<guid>.<Theme><Type>` is also written with `{PropertyListener, PropertyList:{key: control}}`.
- The engine's FileSystemWatcher (filter `*.<Theme><Type>`) applies the delta, e.g. by calling the JS `PropertyListener` format, and deletes it after 3 s (`$SH/Engine/Sucrose.Shared.Engine/Helper/Properties.cs`).

---

## 7. Sucrose.Undo: the uninstaller, not an undo/rollback (`$R/src/Project/Sucrose.Undo/App.xaml.cs`, `Manage/Internal.cs`)

**Started by:** Windows "Apps & features" via the `UninstallString` `%LocalAppData%\Sucrose\Sucrose.Undo\Sucrose.Undo.exe`. `QuietUninstallString` is `"<exe>" -s`.

**Flow:**
1. Without arguments, show a Yes/No/Cancel MessageBox:
   - Yes = remove the app and all settings.
   - No = remove the app, keep settings.
   - Cancel = abort.
2. Kill every process whose name contains "Sucrose" (twice).
3. [WIN] Re-show desktop icons if hidden; refresh the desktop.
4. Delete `%LocalAppData%\Sucrose`, except the `Sucrose.Undo` and `Sucrose.Runtime` folders.
5. If Yes: also delete `%AppData%\Sucrose` (settings, logs, cache, default Library, Showcase).
6. Delete `Desktop\Sucrose Wallpaper Engine.lnk` and `Start Menu\Programs\Sucrose Wallpaper Engine.lnk`.
7. [WIN] Delete `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Sucrose`.
8. Write `%TEMP%\del_<guid>.bat`: taskkill self, wait 3 s, `rd /s /q` the Undo folder, Runtime folder and install dir, then delete itself. Run it via `cmd /c start /B`.

Strings: `Locales/Undo/Undo.EN.xaml`.

---

## 8. Sucrose.Update (`$R/src/Update/Sucrose.Update/*`)

**Start and modes.**
- Launcher auto-update passes argument `False`, which selects the mode from Update.json `AutoType` (default **SemiSilent**).
- Tray and Portal start it with no argument: **Visible**.
- Modes:
  - `UpdateSilent`: hidden.
  - `CompleteSilent`: hidden, and the installer gets `/s`.
  - `SemiSilent`: hidden until an update is found, then visible.
- Single instance. Update.json `State` is true while downloading and is reset on start and exit.

**Steps** (`View/MainWindow.xaml.cs`):
1. Clear `%AppData%\Sucrose\Cache\Bundle`.
2. Network: DNS check over the 8 hosts, `Security.Apply`, save Update.json `Time`, send the telemetry POST.
3. List releases by `ServerType`:
   - **Soferity** (default): `GET https://sucrose.soferity.com/v8/Kernel/Release`, cached 5 min, GitHub-release-shaped JSON (`Helper/Update.cs`).
   - **GitHub**: Skylark `GitHub.ReleasesList(Taiizor, Sucrose, UA, PAT)`.
4. Filter by `ChannelType`: Release takes the first non-prerelease (default); PreRelease takes the first entry.
5. Compare the entry-assembly version with the cleaned `TagName` via Skylark `Versionly.Compare`; continue only when it returns `VersionType.Latest`.
6. Find an asset whose name contains `Sucrose_Bundle_<.NET_11.0>_<x64|x86|ARM64>_<version><.exe|.7z>`, falling back to `.NET_Framework_4.8`. The extension comes from `ExtensionType` (default Executable).
7. Download per `ModuleType`:
   - **Downloader** (default): single chunk, `.sucrose` temp extension, rate limit `LimitValue` 500 × `LimitType` Megabyte converted to bytes/s and refreshed every 1 s.
   - **Native**: HttpClient stream with a manual throttle.
   - A stall watchdog cancels after more than 5 s with no progress.
8. Install:
   - `.exe`: run the installer (`/s` when CompleteSilent).
   - `.7z`: check the 7z magic header `37 7A BC AF 27 1C`, extract with SharpCompress, run the extracted `.exe`.

**Verification:** none beyond the 7z header check. There is no signature or hash check of the downloaded installer.

**Failure handling:** after a download error, "TRY AGAIN" opens the asset URL in the browser via Commandog `Bundle`.

Settings: `$SH/Sucrose.Shared.Core/Manage/Manager/Update.cs`, `$SH/Sucrose.Shared.Dependency/Manage/Manager/Update.cs`, `$LIB/Sucrose.Manager/Manage/Update.cs`. Strings: `Locales/Update/Update.EN.xaml`.

---

## 9. Sucrose.Bundle: installer (`$R/src/Bundle/Sucrose.Bundle/App.xaml.cs`, `Main.xaml.cs`, `Sucrose.Bundle.csproj`)

**Build and form:**
- A net48 WPF exe with Costura (single file). The target machine needs **.NET Framework 4.8**.
- Embedded resources:
  - `Packages\Sucrose.7z` (from `src/Sucrose/Package/Compressed/net11.0-windows/Sucrose-<Platform>.7z`).
  - `SevenZip\7z.exe` and dlls (per platform).
  - `Showcase\**` (20 sample wallpapers).
  - `Checksum\Template.html` and `Checksum\<Platform>\Hashes.json`. Where Hashes.json is generated is unclear; it is not in the csproj and not in the `Sucrose.ps1` grep.
- UI in 60+ languages (`Properties/Resources_XX.xaml`, falls back to EN), RTL support, random `Background1..36.jpg`.
- Silent flags: `-s`, `/s`, `-silent`, `/silent` (the window is hidden).

**Steps:**
1. [WIN] DWM round corners.
2. Kill every process whose name contains "Sucrose" (twice); refresh the desktop.
3. Recreate `%TEMP%\Packages`, `%TEMP%\SevenZip` and `%AppData%\Sucrose\Showcase`; empty `%LocalAppData%\Sucrose`.
4. Extract the embedded resources (Showcase goes straight to `%AppData%\Sucrose\Showcase`).
5. Extract the package: first `7z.exe x "<pkg>" -o"<install>" -aoa`; fallback SharpCompress from the temp file; fallback SharpCompress from the embedded stream.
6. Verify the SHA-256 of every file listed in Hashes.json (85 in parallel).
   - **On success:** create `Sucrose Wallpaper Engine.lnk` on the Desktop and in Start Menu\Programs, pointing at `%LocalAppData%\Sucrose\Sucrose.Launcher\Sucrose.Launcher.exe` ([WIN] Skylark ShortcutBasic, fallback ShortcutDefault; presumably IShellLink / WSH, unclear). Then write [WIN] `HKCU\…\Uninstall\Sucrose` with `NoModify=1`, `NoRepair=1`, `Contact=taiizor@vegalya.com`, `DisplayName`, `URLInfoAbout`, `URLUpdateInfo`, `EstimatedSize` (KB of the installer exe), `Publisher` and `PublisherName=Taiizor`, `Comments`, `DisplayIcon` (Launcher), `BundleVersion` and `DisplayVersion`, `UninstallString`, `InstallLocation`, `QuietUninstallString`. Then start the Launcher.
   - **On failure:** open `%TEMP%\Template.html` (error page with `{Version}` and `{Architecture}` filled in) and wipe the install dir.

**Not done by the installer:**
- No file associations.
- No .NET, WebView2 or VC++ installers. The .NET runtime ships inside the package as `Sucrose.Runtime` (`.build/Sucrose.ps1` `Install-DotNetRuntime`). The WebView2 bootstrapper and VC++ redist URLs are used by the engines, out of scope.
- No startup entry; the user sets that in Portal.

---

## 10. Sucrose.Localizer: developer tool (`$R/src/Localizer/Sucrose.Localizer/*`)

An interactive console tool, not shipped to users. Its menu (`Helper/Program.cs`):
1. XAML → CSV
2. CSV → XAML
3. CSV → POEditor CSV
4. POEditor → CSV
5. Cross-check CSVs
6. Cross-check POEditor files
7. Reindex CSVs against a reference language
8. Alphabetic key sort
9. Create a new language CSV from a source language; also adds a `Locale.<code>` entry "`<Name> (v0.1)`" to `Locale.csv`

Format details:
- CSV columns `File, Key, Value`.
- `MergedDictionaries` blocks are stored as `Key=Base64`.
- POEditor rows use `⦙` placeholders marked "Don't Touch".

---

## Porting notes (risks and oddities)

1. **Commandog is a mandatory trampoline** for nearly every process launch, using ✔/✖ separators inside one quoted argument. Several commands have no caller: Kill, Test, Publish, StartupM, QuotingGoogle, Versioning.
2. **Settings JSON files double as IPC state** (`ClosePerformance`, `PausePerformance`, `*Required`, `TransmissionPort`, Update `State`). They are re-read by mtime with no locking, so cross-process races are possible.
3. **Light pause depends on each engine polling Backgroundog.json.** Heavy pause uses the undocumented `NtSuspendProcess`/`NtResumeProcess`. The WebView "pause" uses a `DebugActiveProcess` hack.
4. **Backgroundog behaviour quirks:**
   - The 100 ms timer spawns Tasks every tick, and Date is refreshed every tick.
   - In Signal mode it creates a new file per send (up to ~10/s).
   - BIOS, motherboard, CPU name and the adapter lists are collected once and never refreshed, so hot-plugged adapters are missed.
   - Only the WebView and CefSharp engines consume this data.
5. **Logic quirks to decide on during the port:**
   - Network trigger: `Ping >= PingValue` is checked first, so `PingValue=0` with Upload or Download thresholds set makes every poll count as a Ping trigger.
   - Battery trigger uses `PowerLineStatus != Online` **or** `ACPowerStatus != "Online"`.
   - ScreenSaver recovery uses an OR/AND asymmetry.
   - Remote/VM detection is by process-name lists with generic names (`player.exe`, `helpdesk.exe`), which invites false positives.
6. **Security:**
   - TLS certificate validation is disabled globally, and SSL2/SSL3 are enabled.
   - Update has no signature or hash check of the downloaded installer.
   - The GitHub path sends a PAT from `Objectionable.json`.
   - Do not carry any of this over.
7. **Privacy:** telemetry sends UserName, Windows SID, hardware serial (IdentifyingNumber), model, manufacturer and the library path. Watchdog writes report files even when `ExceptionData` is off; they are uploaded later if the setting is turned back on.
8. **Undo bugs:**
   - `QuietUninstallString -s` is a **no-op**: any argument skips the dialog while the result stays Cancel.
   - Uninstall does not remove the Run-key or scheduled-task startup entries, nor the GPU-preference registry values.
9. **Installer and updater:**
   - Bundle and Undo kill every process whose name *contains* "Sucrose".
   - Bundle requires .NET Framework 4.8.
   - The app relies on a private `Sucrose.Runtime` via the `DOTNET_ROOT` env var; a Rust port drops this.
   - The update asset naming encodes the framework name (`.NET_11.0` / `.NET_Framework_4.8`); a Rust release pipeline needs a new naming scheme or a compatibility shim.
10. **Launcher:**
    - The tray is a WinForms NotifyIcon inside a WPF app.
    - "Send Feedback" is permanently disabled.
    - The Discord presence uses hard-coded random ranges (49 statements, 24 texts, 37 images) and detects only `Discord.exe` and `DiscordPTB.exe`, not Canary.
11. **Behaviour that comes from closed Skylark.Wing NuGet code** and must be re-derived: desktop-focus detection, fullscreen test, screen-saver check, energy-saver state, power-plan lookup, startup registration (Run key seen in DLL strings; the "Priority" and "Machine" variants are unclear), shortcut creation, SENS listener, and `IsCobalt` (likely a Windows 11 check, unclear).
12. **Thumbnail crop:** a hard-coded crop (34,1,233,233) is applied when pixel (0,0) is transparent and the system is not "Cobalt". This looks like a workaround for one player's padded artwork; verify before porting.
13. **Exception escalation:** `HasError` lets only the first fatal exception reach Watchdog, and the app then exits. First-chance logging is disabled.
