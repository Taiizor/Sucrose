# Legacy reference: libraries, settings, file formats, store protocol and IPC

> Inventory of the **C# app** (not the Rust rewrite), produced on 2026-11-21 by reading the sources in `src/`. Use it as the behavior spec when porting. Items marked "unclear" were not verified; when you find a mistake, fix it here.

Paths are relative to `src/`. `L/` = `Library/`, `S/` = `Shared/`. Nothing in the repo was modified.


## 1. Settings storage (Sucrose.Manager)

### 1.1 Mechanism (`L/Sucrose.Manager/SettingManager.cs`, `Helper/{Reader,Writer,Cleaner,Unique,Validator}.cs`, `Converter/*`)
- **Path:** `%APPDATA%\Sucrose\Setting\<File>.json`. This is Roaming AppData (`Environment.SpecialFolder.ApplicationData`), built from `SMMRP.ApplicationData` + `SMMRG.AppName` + `SMMRF.Setting`.
- **Format:** Newtonsoft.Json 14.0.1-beta2, `Formatting.Indented` (2 spaces), `TypeNameHandling.None`. Line endings are CRLF via `TextWriter.NewLine`; the bundled samples are CRLF.
- **Encoding:** UTF-8 **without BOM** (default `StreamWriter`). Reading uses `StreamReader`, which tolerates a BOM.
- **Wrapper:** every file is `{"Properties": { "<Key>": <value>, ... }}`, from the private class `Settings { Dictionary<string,object> Properties }`.
- **Value encoding:**
  - bool → `true`/`false`; int → number; string → string.
  - enums → **name strings**, written by the custom `EnumConverter`, e.g. `"Resume"`, `"MpvPlayerLive"`.
  - `List<string>`/`string[]` → array; `Dictionary<string,string>` → object.
  - `DateTime` → Newtonsoft ISO round-trip, e.g. `"2026-11-22T01:02:03.1234567+03:00"` for `DateTime.Now`.
  - `IPAddress` → string (`IPAddressConverter`; no setting uses it).
- **Read (`GetSetting<T>`):**
  - On every call, if the file's mtime is later than `_lastWrite` (the `DateTime.Now` of the last load/write), the whole file is re-read.
  - `ConvertToType` handles IPAddress, Uri, Guid, enum (`Enum.Parse(type, value.ToString())`: case-sensitive, numeric strings accepted), `KeyValuePair<string,string>` (`"a:b"`), `string[]`/`List<string>` (from `JArray`), and `Dictionary<string,string>` (re-parse). Anything else is a plain cast `(T)value`.
  - **Any exception returns the default silently.**
- **`GetSettingStable<T>`:** `JsonConvert.DeserializeObject<T>(value.ToString())`. Used for **all ints**, because Newtonsoft boxes integers as `Int64` and a direct cast would fail.
- **Write (`SetSetting`):** re-read and deserialize the whole file, set the key(s), serialize the whole file, then `Writer.WriteStream`.
  - `WriteStream` uses `FileMode.Truncate` (or `CreateNew`) with `FileShare.None`.
  - **Not atomic:** no temp file and no rename. Exceptions are swallowed, so a failed write is lost silently.
- **Locking:** a named `Mutex` per file. The name is `Skylark.Helper.Guidly.GuidToText(Guid(first 16 bytes of SHA-256(UTF-8 full path)))`; the exact text format is **unclear** (Skylark is an external NuGet).
  - [WIN] Named mutex (CreateMutex). There is no `Global\` prefix, so it is session-local.
  - The mutex covers each read and each write separately, **not** the read-modify-write cycle.
  - Reads also use `FileShare.None`.
- **Corruption handling (`ControlFile`, run in the constructor):**
  - A missing, empty, invalid-JSON (`Validator.Json`), or `Properties == null` file is **reset** to `{"Properties":{}}`.
  - `Cleaner.Clean` recursively strips a duplicated leading `{{` or trailing `}}` on every read and write.
- **Static initialization:** `L/Sucrose.Manager/Manage/Internal.cs` holds `static readonly` managers. The first access to `SMMI` in any process constructs **all 14** managers, which creates any missing setting files.
- **Dead code:** `SettingManager2.cs` (lock-based) and `SettingManager3.cs` (FileSystemWatcher-based) are referenced nowhere.

### 1.2 Logs (`L/Sucrose.Manager/LogManager.cs`, `L/Sucrose.Memory/Manage/Valuable/Log.cs`)
- **File:** `%APPDATA%\Sucrose\Log\<Name>-yy.MM.dd.log`.
  - Names: Undo, Portal, Update, Launcher, Property, Watchdog, Commandog, Reportdog, AuroraLive, NebulaLive, VexanaLive, XavierLive, WebViewLive, Backgroundog, CefSharpLive, MpvPlayerLive, VlcPlayerLive.
  - CefSharp also writes `CefSharpNative-{date}-{guid}.log`.
- **Line format:** `[HH:mm:ss] ~ [SucroseWatchdog Thread-<random 1000-9999 per manager>/<Skylark LevelLogType>] ~ [<msg>]`, appended with the same mutex scheme.
- **Exception logging:** `S/Sucrose.Shared.Watchdog/Extension/Watch.cs` writes `<TYPE> EXCEPTION START`, then message, inner exception and stack trace, then `... FINISH`. TYPE is one of CATCH, THREAD, UNOBSERVED TASK, GLOBAL UNHANDLED, DISPATCHER UNHANDLED. FIRST CHANCE is disabled. The target log is picked by the project's preprocessor symbol.
- **Log bundle** (`S/Sucrose.Shared.Launcher/Helper/Create.cs`, Portal `SystemSettingViewModel.cs:405-432`):
  - The zip is named `sucrose_log_yyyyMMdd_HHmmss.zip`.
  - It contains the top-level files of `Log\` and `Setting\` as entries `"<FolderName>\<file>"` (built with `Path.Combine`, so backslashes).
  - **`Objectionable.json` is excluded.**
- **Settings import/export/reset** (`S/Sucrose.Shared.Space/Helper/{Export,Import,Reset}.cs`):
  1. Kill live engines plus Undo, Portal, Update, Launcher, Property, Watchdog, Reportdog and Backgroundog; wait 3 s.
  2. Copy (or delete, for Reset) the top-level `*.json` files between `Setting\` and the chosen folder; wait 1 s.
  3. Relaunch Portal with the argument `SystemSetting`, plus the given application.

### 1.3 Files and keys (the compatibility contract)
- Key-name constants: `L/Sucrose.Memory/Manage/Constant/*.cs`.
- Getters: `L/Sucrose.Manager/Manage/*.cs` (M), `S/Sucrose.Shared.Dependency/Manage/Manager/*.cs` (D), `S/Sucrose.Shared.Core/Manage/Manager/Update.cs` (C), `Portal/Sucrose.Portal/Manage/Manager/Portal.cs` (P).
- "Clamp" means the value is clamped by `Skymath.Clamp` on read.

**General.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| Culture | string | `Skylark Culture.CurrentUITwoLetterISOLanguageName` (OS UI language) | M; uppercased when applied |
| RunStartup | int | 0 | clamp 0..10. 0 = off, 1 = `Startup` command, 2 = `StartupP` (elevated, runas), 3 = Scheduler task (Portal `GeneralSettingViewModel.cs:784-854`) |
| UserAgent | string | `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit` | M |
| TelemetryData | bool | true | M |
| ExceptionData | bool | true | M |
| AppVisible | bool | true | M |
| AppExit | bool | true | M |
| ThemeType | Skylark `WindowsThemeType` {Dark, Light} | `Skylark.Wing WindowsTheme.GetTheme()` [WIN, OS theme; API unclear] | D |

**Portal.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| BackgroundOpacity | int | 100 | 0..100 |
| LibraryPagination | int | 30 | 1..100 |
| StorePagination | int | 30 | 1..100 |
| AdaptiveLayout | int | 0 | 0..100 |
| AdaptiveMargin | int | 5 | 5..25 |
| StoreDuration | int | 3 | 1..24 (hours; store cache TTL) |
| BackgroundImage | string | "" | |
| LibraryPreviewHide / StorePreviewHide | bool | false | |
| LibraryPreview / StorePreview | bool | false | |
| StoreAdult | bool | **true** | show adult items |
| BackgroundStretch | WPF `Stretch` {None, Fill, Uniform, UniformToFill} | UniformToFill | D |
| LibrarySortKind | SortKindType | Descending | D |
| LibrarySortMode | SortModeType | Creation | D |
| StoreServerType | StoreServerType | Soferity | D |
| BackdropType | WPF-UI `WindowBackdropType` {None, Auto, Mica, Acrylic, Tabbed} | None | P (`Portal/Sucrose.Portal/Manage/Internal.cs:42`), also Property |

**Engine.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| CefArguments | object<string,string> | {} | |
| WebArguments | string[] | [] | |
| VolumeSilentSensitivity | int | 5 | 1..10 |
| WallpaperVolume | int | 100 | 0..100 |
| DeveloperPort | int | 0 | 0..65535 |
| DuplicateScreenType | Skylark {Default, Advanced} | Default | |
| ExpandScreenType | Skylark {Default, Advanced} | Default | |
| DisplayScreenType | Skylark {PerDisplay, SameDuplicate, SpanAcross} | PerDisplay | |
| ScreenType | Skylark {DisplayBound, WorkingArea} | DisplayBound | |
| ScreenDevice | string | "" | |
| InputType | Skylark {Close, OnlyMouse, OnlyKeyboard, MouseKeyboard} | MouseKeyboard | |
| InputModuleType | InputModuleType | RawInput | D |
| StretchType | Dependency StretchType | UniformToFill | D |
| GraphicPreference | HighPerformanceType | HighPerformance | D |
| Gif | EngineType name | MpvPlayerLive | D; allowed: Vexana/Xavier/WebView/CefSharp/MpvPlayer/VlcPlayer Live |
| Url | EngineType | CefSharpLive | WebView/CefSharp |
| Web | EngineType | CefSharpLive | WebView/CefSharp |
| Video | EngineType | MpvPlayerLive | Nebula/WebView/CefSharp/MpvPlayer/VlcPlayer |
| YouTube | EngineType | CefSharpLive | WebView/CefSharp |
| Application | EngineType | AuroraLive | Aurora only |
| BackgroundImagePath | string | "" | |
| BackgroundImage | bool | false | |
| DisableSlideshow | bool | false | |
| DisableSlideshowLive | bool | false | |
| HardwareAcceleration | bool | true | |
| WallpaperShuffle | bool | true | |
| WallpaperLoop | bool | true | |
| VolumeDesktop | bool | false | |
| VolumeSilent | bool | false | |
| InputDesktop | bool | false | |
| DeveloperMode | bool | false | |
| CrashExplorer | bool | false | |
| LibraryStart | bool | true | |
| StoreStart | bool | true | |
| StayAwake | bool | false | |

- Engine defaults come from `S/Sucrose.Shared.Space/Manage/Internal.cs`. Values are stored as `(EngineType)` names.
- Skylark enum names come from code usage and `Portal.Enum.EN.xaml`; their ordinals are unclear. The values are stored as names.

**Backgroundog.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| TransmissionPort | int | 0 | 0..65535; **written at runtime by web engines** |
| DownloadValue | int | 10 | 0..99999999 |
| DownloadType | Skylark StorageType | Megabyte | |
| UploadValue | int | 800 | 0..99999999 |
| UploadType | StorageType | Kilobyte | |
| ProcessorUsage | int | 70 | 0..100 |
| GraphicUsage | int | 70 | 0..100 |
| BatteryUsage | int | 50 | 0..100 |
| MemoryUsage | int | 80 | 0..100 |
| PingValue | int | 100 | 0..1000 |
| PingType | string | "Bing" | host keys in `S/Sucrose.Shared.Space/Helper/Network.cs` (Bing, Baidu, Yahoo, Google, Yandex, Microsoft, ...) |
| NetworkAdapter / GraphicAdapter | string | "" | |
| PerformanceCounter | bool | true | start Backgroundog |
| PipeRequired / SignalRequired / TransmissionRequired / AudioRequired | bool | false | **runtime flags** set by engines |
| PausePerformance / ClosePerformance | bool | false | **runtime state** written by Backgroundog |
| CommunicationType | CommunicationType | Transmission | D |
| PausePerformanceType | PausePerformanceType | Light | D |
| BatterySaverPerformance | PerformanceType | Resume | D |
| ScreenSaverPerformance | PerformanceType | **Pause** | D |
| FullScreenPerformance | PerformanceType | Resume | D |
| ProcessorPerformance | PerformanceType | Resume | D |
| VirtualPerformance | PerformanceType | Resume | D |
| NetworkPerformance | PerformanceType | Resume | D |
| GraphicPerformance | PerformanceType | Resume | D |
| BatteryPerformance | PerformanceType | Resume | D |
| RemotePerformance | PerformanceType | Resume | D |
| MemoryPerformance | PerformanceType | Resume | D |
| FocusPerformance | PerformanceType | Resume | D |
| SessionPerformance | PerformanceType | **Close** | D |
| ConsolePerformance | PerformanceType | **Close** | D |
| SleepPerformance | PerformanceType | **Close** | D |
| LockPerformance | PerformanceType | **Close** | D |

**Cycling.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| TransitionTime | int | 1800 | 10..99999 (unit unclear, likely seconds) |
| PassingTime | int | 0 | 0..99999 |
| Exclusion | string[] | [] | |
| Active | bool | false | |
| TransitionType | TransitionCycleType | Random | D |

**Donate.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| AdvertisingDelay | int | 30 | 30..720 (unit unclear) |
| AdvertisingActive | bool | true | |
| MenuVisible | bool | true | |

**Hook.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| DiscordRefreshDelay | int | 60 | 60..3600, seconds |
| DiscordRefreshConnect | bool | true | |
| DiscordConnect | bool | true | |

**Library.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| Location | string | `%APPDATA%\Sucrose\Library` | |
| Selected | string | "" | theme folder name (25 random chars) |
| DeleteCorrupt | bool | false | |
| DeleteConfirm | bool | true | |
| Move | bool | true | |

**Update.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| LimitValue | int | 500 | 0..99999999 |
| LimitType | StorageType | Megabyte | |
| Time | DateTime | `new DateTime()` (0001-01-01) | set to `DateTime.Now` |
| State | bool | false | |
| Auto | bool | true | |
| ModuleType | UpdateModuleType | Downloader | D |
| ServerType | UpdateServerType | Soferity | D |
| AutoType | UpdateAutoType | SemiSilent | D |
| ExtensionType | UpdateExtensionType | Executable | C |
| ChannelType | UpdateChannelType | Release | C |

**Warehouse.json**
| Key | Type | Default | Notes |
|---|---|---|---|
| Showcase | string[] | [] | showcase folders already copied |
| CefSharpTime / WebViewTime | DateTime | min | WebViewTime is set to Now+1 day |
| CefsharpContinue | bool | false | note the lowercase "s" |
| WebViewContinue | bool | false | |
| HintTrayIcon | bool | true | |

**Other files**
| File | Key | Type | Default | Notes |
|---|---|---|---|---|
| System.json | NetworkInterfaces | string[] | [] | |
| System.json | GraphicInterfaces | string[] | [] | |
| System.json | ProcessorInterfaces | string[] (from `User.GetProcessor()`) | – | write-only, `Project/Sucrose.Backgroundog/Helper/Specification.cs:643` |
| Aurora.json | AppProcessName | string | "" | process name of the launched app, killed on stop |
| Kernel.json | Information | object<string,string> | – | write-only. Keys: Version, Framework, Architecture, OperatingSystem, OperatingSystemBuild, ProcessArchitecture, ProcessorArchitecture, OperatingSystemArchitecture (`Portal/.../MainWindowViewModel.cs:124-137`) |
| Objectionable.json | PersonalAccessToken | string | "" | GitHub PAT, **plaintext** |

## 2. Paths and constants (Sucrose.Memory, `L/Sucrose.Memory/Manage/Readonly/*`, `Valuable/*`)

**Data layout under `%APPDATA%\Sucrose\`**
- `Setting\` – the 14 JSON files above.
- `Log\` – log files.
- `Library\` – default theme library.
- `Showcase\` – bundled themes, copied once into the library and tracked in `Warehouse.Showcase` (`Portal/.../LibraryPage.xaml.cs:91-122`).
- `Cache\`:
  - `Store\` – `Store.json`, `Pattern.json`, `<Category>\<sanitized WallpaperKey>\` (Info and cover), `Temporary\` (downloads plus the XamlAnimatedGif URI cache).
  - `SignalT\` – `*.sgnl` signal files.
  - `Properties\` – per-theme user property values; named `<SelectedFolder><PropertiesType name>` with **no extension**, plus watcher files `<guid>.<Selected><Type>` (`Project/Sucrose.Property/App.xaml.cs:265-267`).
  - `WebView2\`, `CefSharp\`, `MpvPlayer\`, `VlcPlayer\` – engine profiles; each may hold an engine-level `SucroseProperties.json`.
  - `Content\` – local web content such as `GifContent.html`.
  - `Gif\` – Vexana cache.
  - `Report\<Id>.json` – exception reports.
  - `Bundle\` – updater downloads.

**Other locations**
- `%LOCALAPPDATA%\Sucrose` is the install root (Undo `UninstallPath`, the `Versioning` command).
- Install layout (`S/Sucrose.Shared.Space/Manage/Internal.cs`): `<Root>\<ExeNameWithoutExt>\<Exe>`. Each app directory has `Assets\`, `locales\` and `Requirements\` (libmpv); the shared runtime is `<Root>\Sucrose.Runtime`.
- `%TEMP%` holds VC++/WebView2 installers, `del_<guid>.bat`, and `<Title>.zip` for theme publishing.

**Executables (`App.cs`)**
- `Sucrose.{Undo,Portal,Update,Launcher,Property,Watchdog,Commandog,Reportdog,Backgroundog}.exe`
- `Sucrose.Live.{Aurora,Nebula,Vexana,Xavier,WebView,CefSharp,MpvPlayer,VlcPlayer}.exe`

**Content file names (`Content.cs`)**
- `SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json`
- `SucroseStore.json` – empty marker meaning "downloaded from the store"
- `SucrosePremium.json` – marker
- `Store.json`, `Pattern.json`
- `GifContent.html`, `ImageContent.html`, `VideoContent.html`, `YouTubeContent.html`
- `MpvPlayer.config`, `uMpvPlayer.config`

**Folder names (`Folder.cs`):** Gif, Log, Cache, Store, Assets, Bundle, Report, Content, Library, `locales` (lowercase), Runtime, Setting, SignalT, CefSharp, Showcase, WebView2, MpvPlayer, Temporary, VlcPlayer, Properties, Requirements.

**General constants (`General.cs`)**
- AppName `Sucrose`; AppLongName `Sucrose Wallpaper Engine`; Language `EN`.
- PipeServerName `.`; Loopback `127.0.0.1`.
- Local host `sucrose.wallpaper.engine` (`Url.Local` = `https://sucrose.wallpaper.engine/`).
- `StartCommandChar` `✔` (U+2714); `ValueSeparatorChar` `✖` (U+2716); `ExceptionSplit` `" -> "`.
- Default and Unknown strings are `"Default"` and `"Unknown"`.

**Single-instance mutexes (`Mutex.cs`)** [WIN named mutex]: `{Sucrose-Wallpaper-Engine-Live}`, `-Portal`, `-Update`, `-Property`, `-Launcher`, `-Reportdog`, `-Backgroundog` (all in braces).

**Other readonly constants**
- Scheduler task: `Autorun for Sucrose` (`Scheduler.cs`).
- Discord app id: `1126294965950103612` (`Hook.cs`).
- Process names: `msedgewebview2`, `CefSharp.BrowserSubprocess` (`Process.cs`).
- Watch keys: `WatchText`, `WatchSource`.
- `Valuable/App.Chars` = `A-Z a-z 0-9`, used for 25-character random theme folder names.

**URLs (`Url.cs`, `GitHub.cs`, `Soferity.cs`)**
- Site and store: `https://sucrose.dev` (`/premium`), `https://sucrose.soferity.com`, SoferityStore = `https://sucrose.soferity.com/Store`.
- GitHub: `https://github.com/Taiizor/{Sucrose|Store}` with `/wiki`, `/discussions`, `/releases`, `/releases/tag`, `/releases/latest`, `/issues/new/choose`, and a wallpaper report template URL.
- Raw GitHub: `https://raw.githubusercontent.com/Taiizor/{Store|Sucrose}/develop`.
- Third-party: VCRedist `https://aka.ms/vc14/vc_redist.{0}.exe`; WebView2 `https://go.microsoft.com/fwlink/p/?LinkId=2124703`; Google search and translate templates; Microsoft Store listing `https://apps.microsoft.com/detail/XP8JGPBHTJGLCQ?...`; a YouTube PAT tutorial video.
- Soferity API version `v8`. Segments: File, Check, Theme, Throw, Kernel, Online, Report, Search, Update, Upload, Pattern, Release, Analytic, Download, Feedback, Optional, Exception, Telemetry.

## 3. Theme package formats (`S/Sucrose.Shared.Theme/*`)

### Serializer (`Helper/Converter.cs`)
- Newtonsoft with `MetadataPropertyHandling.Ignore`, `DateParseHandling.None`, Indented.
- Converters: `ModelConverter`, `VersionConverter`, `IsoDateTimeConverter` (AssumeUniversal).
- **No** `StringEnumConverter`: enums are written as ints, and reading accepts an int or a name.
- Files are read and written through `S/Sucrose.Shared.Space/Helper/Filing.cs`: mutex name from MD5 (SHA-256 fallback) via `Space/Helper/Unique.cs`, `FileShare.None`, truncate-write.

### SucroseInfo.json (`Helper/Info.cs`; sample `Bundle/Sucrose.Bundle/Showcase/Bing Wallpaper-1/SucroseInfo.json`)
"Always" means the key must be present and not null. "AllowNull" means the key **must be present** but may be null.

| Field | Type | Required | Default |
|---|---|---|---|
| AppVersion | Version | Always | 0.0.0.0 |
| Version | Version | Always | 1.0.0.0 |
| Title | string | Always | |
| Thumbnail | string (relative file) | Always | |
| Preview | string (relative file) | Always | |
| Description | string | Always | |
| Author | string | Always | |
| License | string | **AllowNull** | |
| Contact | string (URL or e-mail) | Always | |
| Type | WallpaperType int (0 Gif, 1 Url, 2 Web, 3 Video, 4 YouTube, 5 Application) | Always | |
| Source | string (relative file or URL) | Always | |
| Tags | string[] | **AllowNull** | |
| Arguments | string | **AllowNull** | |
| Localization | `{ "<LANG>": {Title, Description} }` | optional | null |

- `Version` (`Converter/VersionConverter.cs`) reads a string such as `"1.0.1.1"` or an object `{Major, Minor, Build, Revision}`, and writes a string. It uses .NET `System.Version` semantics: missing components become -1.
- Localization (`Helper/Localization.cs`): both inner fields are required. Keys are uppercase two-letter codes in the samples and are matched case-insensitively against `General.Culture`; when nothing matches, the base Title and Description are used (`Converter/LocalizationConverter.cs`).

### SucroseProperties.json (`Helper/Properties.cs`, `Model/*.cs`, `Converter/ModelConverter.cs`; sample `.../Living Room-1/SucroseProperties.json`)
- Top level:
  - `PropertyListener` (Always): JS format string with `{0}` = name and `{1}` = value, e.g. `"SucrosePropertyListener('{0}', {1});"`.
  - `PropertyList` (Always): `{ "<name>": control }`.
  - `PropertyLocalization` (optional): `{ "<LANG>": { "<locKey>": "text" } }`.
- Common control fields: `type` (Always), `text` (Always; empty allowed for label), `help` (optional). `name` is ignored in JSON and comes from the dictionary key.
- Control types, dispatched on lower-cased `type` (any other type throws `NotSupportedException`):

| type | Fields |
|---|---|
| label | value: string |
| button | value: string |
| textbox | value: string |
| passwordbox | value: string |
| colorpicker | value: string |
| checkbox | value: bool |
| slider | min, max, step, value: double |
| numberbox | min, max, value: double; places: int |
| dropdown | value: int index, items: string[], `valuetext` (computed `items[value]`, written on serialize) |
| filedropdown | desc, value, title, folder, filter (e.g. `*.jpeg|*.jpg|*.png`): string; files: string[] (optional, omitted when null) |

- Per-user values live in a copy of the file under `Cache\Properties\` (see §2), managed by the Property app.

### SucroseCompatible.json (`Helper/Compatible.cs`)
- All fields are optional strings. Each is a JS template containing `{0}`, which the web engines execute with live data (e.g. `"SucroseAudioData({0});"`, `"SucroseStretchMode('{0}');"`).
- Fields: LoopMode, ThemeType, SystemBios, SystemDate, ShuffleMode, StretchMode, SystemAudio, SystemTheme, VolumeLevel, SystemMemory, SystemBattery, SystemGraphic, SystemNetwork, SystemStorage, SystemProcessor, SystemMotherboard.

### Archive format and validation (`S/Sucrose.Shared.Zip/Helper/Archive.cs`, `Helper/Zip.cs`, `Extension/Zip.cs`)
**Export (`Zip.Compress`)**
- `ZipFile.CreateFromDirectory(theme, dest, SmallestSize, includeBaseDirectory:false)`, so files sit at the zip root.
- Portal `ThemeShare.xaml.cs:106` uses it for Export.
- Publish additionally requires a size of at most **90 MiB** and a passing `Check` (`ThemeShare.xaml.cs:269-313`).

**Import** (`Portal/.../LibraryPage.xaml.cs:342-358`): `Archive.Check`, then extract into `<Library.Location>\<25 random alnum>` with overwrite.

**`Check` order** (the first failure returns that CompatibilityType):
1. NotFound.
2. Access, Locked [WIN file-lock check; Skylark FileLocksmith is used in Space].
3. Extension is not `.zip` (case-sensitive).
4. ZipType: the magic is not `50 4B 03 04`. The Encrypt check is commented out.
5. Folder: the first entry ends with `/`.
6. InfoFile missing. The match is on entry Name **or** FullName.
7. EmptyInfo, InvalidInfo. `ReadFile` uses `GetEntry(FullName)`.
8. Title (1..50), Description (1..500), Author (1..50), Contact (1..250), InvalidContact (must pass the URL or mail regex in `Helper/Various.cs`).
9. Tags: 1..5 tags, each 1..20 characters and not whitespace (Tags, InvalidTags).
10. Arguments: at most 250.
11. Localization: if present it must be non-empty, and each entry must have Title 1..50 and Description 1..500.
12. Thumbnail and Preview must exist in the archive.
13. AppVersion: rejected if greater than the running app version (`Skylark Versionly.Entry()`, `yy.MM.dd.0`).
14. Type: the int must be smaller than the WallpaperType count.
15. Source rules by Type:
    - Gif: URL or file in the archive, and the extension must be `.gif`.
    - Url: valid URL.
    - Web: file in the archive with a Skylark `WebExtensionType` extension (list unclear).
    - Video: URL or file, with a Skylark `VideoExtensionType` extension (unclear).
    - YouTube: youtube.com or music.youtube.com watch or playlist URL.
    - Application: file with a Skylark `AppExtensionType` extension (unclear).
16. If `SucroseProperties.json` exists: EmptyProperties, InvalidProperties; PropertyListener must contain both `{0}` and `{1}`; PropertyList non-empty; PropertyLocalization non-empty if present.
17. If `SucroseCompatible.json` exists: EmptyCompatible, InvalidCompatible; every non-empty field must contain `{0}`.
18. Pass. Any exception returns UnforeseenConsequences.

**Other archive code**
- `S/Sucrose.Shared.SevenZip`: SharpCompress 0.51.0-beta. `Extract` takes any archive type; `Compress` writes a zip with LZMA; `CheckArchive` tests the 7z magic `37 7A BC AF 27 1C 00`. It is used **only by Sucrose.Update**, not for themes.

**Library validity** (`LibraryPage.xaml.cs:135-170`): a theme is valid when Info parses, Preview and Thumbnail files exist, and Source is a URL or an existing file. Otherwise, if `DeleteCorrupt` is on and the folder is accessible and unlocked, the folder is deleted.

## 4. Store protocol (`S/Sucrose.Shared.Store/*`, Portal `Views/Pages/StorePage.xaml.cs`, `Store/FullStorePage.xaml.cs`, `Views/Controls/StoreCard.xaml.cs`)

### Store.json and Pattern.json (`Interface/{Store,Category,Wallpaper}.cs`)
```
{ "Categories": { "<Category>": { "Wallpapers": { "<WallpaperKey>": {
   "Size": long (bytes, optional, 0), "Adult": bool (optional, false),
   "Live": string (required, animated preview GIF), "Cover": string (required, cover image),
   "Source": string (required, store sub-path; the theme lives at <Source>/<WallpaperKey>),
   "Pattern": string (optional, search keywords) } } } } }
```
- `Categories` is Required.Always; the serializer is Newtonsoft Indented (`Helper/Converter.cs`).
- `Pattern.json` is parsed and validated with the **same schema**.

### Servers
| Step | Soferity (default) | GitHub |
|---|---|---|
| Store index | `GET https://sucrose.soferity.com/Store/src/Store.json` | Skylark.Standard `GitHub.ContentsList("Taiizor","Store","src","develop",UA,PAT)`; the entry named `Store.json` is fetched via its `download_url`. Exact API URL unclear (external lib) |
| Pattern | `GET https://sucrose.soferity.com/v8/Kernel/Pattern`; if valid, **replaces** Store.json for display (`StorePage.xaml.cs:63`) | none |
| Info and cover | `{SoferityStore}/{Source}/{Key}/SucroseInfo.json` and `/{Cover}` | `{RawGitHubStoreBranch}/{Source}/{Key}/...` |
| Hover preview GIF | `{server root}/{Source}/{Key}/{Live}`, through XamlAnimatedGif | same |
| Folder listing | `GET https://sucrose.soferity.com/v8/Kernel/File/Store/<path>` (leading `Store/` stripped) returns a JSON array of `Contents{size, sha, url, name, path, type:"file"|"dir", download_url, direct_download_url}` (`Interface/Contents.cs`) | GitHub contents API via Skylark |
| Auth | User-Agent header | User-Agent plus `Authorization: Bearer <PAT>` when set (`Helper/GitHub/Download.cs:InitializeClient`) |

- Spaces in URLs are replaced with `%20` only.
- Both download helpers recurse into `dir` entries.
- Soferity reports byte progress when `Wallpaper.Size` > 0; otherwise progress is per file. Progress is kept in `StoreService.Info[Keys] = Data{ProgressPercentage, DownloadedFileCount, TotalFileCount, "xx.xx%", "n/m", Guid = Path.Combine(Source, Key)}`.

### Caching
- `Cache\Store\Store.json` and `Pattern.json` are re-downloaded when their mtime is older than `Portal.StoreDuration` hours, or when invalid.
- Per-card cache: `Cache\Store\<CategoryKey>\<FileName-sanitized Key>\` holds SucroseInfo.json and the cover. Expiry checks the **directory** mtime.
- The GIF preview cache is `Cache\Store\Temporary` with the same TTL (Portal `MainWindow.xaml.cs:124-127`).
- If no host is reachable (`Network.GetHostEntry`), the stage is Broken; a failed fetch gives Unknown; success gives Full.

### Install flow (`StoreCard.xaml.cs:157-250`)
1. POST download telemetry to `{Soferity}/v8/Telemetry/Download/{userGuid}` with `DownloadTelemetryData` JSON.
2. Pick a 25-character random `Keys` that is unused in the library.
3. Download recursively into `Cache\Store\Temporary\<Keys>`.
4. `Copy.Folder` into `<Library>\<Keys>` and delete the source.
5. Write an empty `SucroseStore.json` marker.
6. If `Engine.StoreStart` is on and performance is not paused or closed (or Backgroundog is not running): set `Library.Selected = Keys` and restart the live engine.
7. The card is downloadable only if `Info.AppVersion` is at most the app version.

### Adult flag
- A category is shown if `StoreAdult` is on or not all of its wallpapers are adult.
- A wallpaper is shown if `!Adult || StoreAdult` (`FullStorePage.xaml.cs:58,169,204`).

### Search
- Score = number of query words contained in `Pattern ?? Key.ToLowerInvariant()`, split on spaces (`FullStorePage.xaml.cs:242-299`).

### Other Soferity endpoints (`{userGuid}` = `User.GetGuid()`, an MD5 GUID of `"{UserName}-{Model}-{Identifier}"` from WMI)
- `v8/Telemetry/{Analytic|Online|Search|Update|Download}/{guid}`
- `v8/Exception/Throw/{guid}` (Reportdog)
- `v8/Optional/Report[/Check]/{guid}` (ThemeReport)
- `v8/Optional/Upload/Theme/Check/{guid}` and `v8/Optional/Upload/Theme/{guid}/{categoryTag}/{appVersionText}`, multipart field `file`, `application/zip` (ThemeShare)
- `v8/Kernel/Release` (Update)
- Payload models are in `S/Sucrose.Shared.Space/Model/*.cs`:
  - `AnalyticTelemetryData`: about 100 fields mirroring the settings.
  - `ThrowExceptionData`: Id, Sid, AppId, AppName, UserName, AppVersion, Exception (JObject), culture, OS, arch, ...
  - The others: Download, Feedback, Online, Report, Search, Update.

### HTTP client (`S/Sucrose.Shared.Dependency/Manage/Internal.cs`)
- `SocketsHttpHandler` with redirects, 100 connections per server, all decompression methods.
- **Accepts any TLS certificate**; SSL2/SSL3/TLS1.0-1.3 enabled.
- Infinite timeout; User-Agent from settings.

## 5. IPC

| Channel | Transport | Name / port | Server (receiver) | Client (sender) | Payload |
|---|---|---|---|---|---|
| Backgroundog data | Pipe | `Sucrose.Wallpaper.Engine.Backgroundog.Pipe` | WebView / CefSharp engine | Backgroundog (`Specification.cs:942-1010`) | Backgroundog JSON |
| Backgroundog data | Signal (files) | `Backgroundog.sgnl` | WebView / CefSharp engine | Backgroundog (`Specification.cs:1028`) | Backgroundog JSON |
| Backgroundog data | Transmission (TCP) | 127.0.0.1, ephemeral port | WebView / CefSharp engine | Backgroundog (`Specification.cs:1064-1132`) | Backgroundog JSON |
| Launcher control | Signal | `Launcher.sgnl` | Launcher (`Launcher/Sucrose.Launcher/App.xaml.cs:145`) | Portal (`GeneralSettingViewModel.cs:515,519`), Backgroundog (`Performance.cs:183`, `Condition.cs:193`) | Launcher JSON |
| Launcher control | Pipe / Transmission | `Sucrose.Wallpaper.Engine.Launcher.Pipe` | services exist (`S/Sucrose.Shared.{Pipe,Transmission}/Services/Launcher*Service.cs`, `#if LAUNCHER`) | **none wired; unused** | – |

### Payloads (identical classes in `L/Sucrose.{Pipe,Signal,Transmission}/Interface/*.cs`)
- **Backgroundog:** `{"Bios","Date","Audio","Memory","Battery","Graphic","Network","Storage","Processor","Motherboard"}`, each a free-form JObject from Backgroundog's `SBED.Get*Info()` (schema is in Backgroundog, outside this scope). Serialized with `Formatting.None`.
- **Receiver side** (`S/Sucrose.Shared.{Pipe,Signal,Transmission}/Services/Backgroundog*Service.cs`, `#if LIVE_WEBVIEW || LIVE_CEFSHARP`): each non-null section is stored as indented JSON in engine statics; the engine then runs the compatible JS templates.
- **Launcher:** `{"Hide":bool,"Icon":bool,"Show":bool,"Release":bool}`. The handler calls Hide, Show and Release on the tray icon when set, then **always** calls `TrayIconManager.Icon(Data.Icon)`: true shows the grayscale tray icon (paused), false the normal icon.

### Handshake (engine side, e.g. `S/Engine/Sucrose.Shared.Engine.WebView/Helper/Web.cs:75-180`)
1. At startup the engine resets PipeRequired, AudioRequired, SignalRequired, PausePerformance and TransmissionRequired to false (`Live/*/App.xaml.cs`).
2. It sets `AudioRequired` when `Compatible.SystemAudio` is non-empty.
3. Depending on `Backgroundog.CommunicationType` it sets the matching `*Required=true` and starts its server.
   - For Transmission it first picks a free port (`S/Sucrose.Shared.Space/Helper/Port.cs`: listen on port 0, fallback 65432) and writes it to `Backgroundog.json:TransmissionPort`.
4. Backgroundog sends every cycle while `!Condition && *Required`.

### Pipe (`L/Sucrose.Pipe/*`)
- Server: `NamedPipeServerStream(name, In, maxInstances 10, Byte, Async)`. Accept-wait timeout 5 s (loops); a read timeout of 15 s drops the client; the server is recreated after errors.
- Client: `NamedPipeClientStream(".", name, Out)`, connect timeout 5 s, retries 3 with backoff 1/2/4 s, plus one reconnect on a send failure.
- Framing: UTF-8 text, **one JSON per line** (`WriteLineAsync` / `ReadLineAsync`).
- [WIN] Named pipe `\\.\pipe\<name>`.

### Transmission (`L/Sucrose.Transmission/*`)
- TCP with `TcpListener(127.0.0.1, port)`; line-delimited JSON.
- KeepAlive on, TcpKeepAliveTime 10, interval 2.
- Client send timeout 2.5 s, receive 5 s; server send 5 s, receive 15 s; accept timeout 5 s.
- Same retry scheme as the pipe.
- `Manage/Internal.cs` holds nullable statics that are created at runtime.

### Signal (`L/Sucrose.Signal/*`)
- Directory: `%APPDATA%\Sucrose\Cache\SignalT`.
- **Writer:** creates a new file `<Base>-<Guid>.sgnl` (compact JSON) under mutex `SignalWriter_<filename>`, `FileShare.None`, 3 retries at 100 ms.
- **Receiver:** a `FileSystemWatcher` (filter `*.*`, buffer 64 KB) fires on Created for names containing `<Base>`.
  - It reads after a random 5–50 ms delay under mutex `SignalReader_<filename>` with `FileShare.Read`, then deletes the file 3 s later (System.Timers; delete retries 3 at 50–200 ms).
  - `StartChannel` first deletes stale matching files.
  - Corrupt JSON returns the default and the file is deleted.

### Other cross-process channels
- **Commandog command line:** `Sucrose.Commandog.exe "✔<CommandType>✖<arg>✖<arg>..."`. bools render as `True`/`False`. Examples from scope:
  - `✔Live✖<engine exe>`, `✔Backgroundog✖<exe>` (`S/Sucrose.Shared.Live/Helper/Run.cs`)
  - `✔Interface✖<Portal>[✖GeneralSetting]`, `✔Property✖<exe>`, `✔Reportdog✖<exe>`
  - `✔RestartLive✖Unknown`, `✔Update✖<exe>[✖<bool>]`
  - `✔Startup✖Sucrose✖<Launcher>✖<bool>`, `✔StartupP✖<Launcher>✖<bool>` (runas)
  - `✔Scheduler✖Create✖<Launcher>` / `✔Scheduler✖Delete`
  - `✔Report✖<url>`, `✔Versioning✖%LOCALAPPDATA%\Sucrose✖<ver>✖False`
- **Portal first argument:** an `ArgumentCommandType` name.
- **Shared setting files:** `Backgroundog.json` runtime flags and `Aurora.json:AppProcessName` act as shared state; there is no notification, only mtime polling.

## 6. Enums (`S/Sucrose.Shared.Dependency/Enum/*.cs`; ordinals are 0-based in declaration order)
- **ArchitectureType:** x64, x86, ARM64, Unknown.
- **CommandType:** Log, Kill, Live, Test, Temp, Wiki, Reset, Bundle, Update, Import, Export, Report, Premium, Publish, Startup, Cycyling (sic), Property, StartupM, StartupP, Official, Watchdog, Interface, PropertyA, Reportdog, Scheduler, Repository, Versioning, Discussions, RestartLive, Backgroundog, QuotingGoogle, QuotingTranslate.
- **ArgumentCommandType:** OtherSetting, DonateSetting, SystemSetting, GeneralSetting, PersonalSetting, WallpaperSetting, PerformanceSetting.
- **SchedulerCommandType:** Create, Enable, Delete, Disable.
- **CommunicationType:** Pipe, Signal, Transmission.
- **CompatibilityType** (public): Pass, Tags, Type, Title, Access, Author, Folder, Locked, Source, Contact, Encrypt, Preview, ZipType, InfoFile, LoopMode, NotFound, Arguments, EmptyInfo, Extension, ThemeType, Thumbnail, AppVersion, InvalidUrl, SystemBios, SystemDate, Description, InvalidFile, InvalidInfo, InvalidTags, ShuffleMode, StretchMode, SystemAudio, SystemTheme, TriggerTime, VolumeLevel, Localization, PropertyList, SystemMemory, SystemBattery, SystemGraphic, SystemNetwork, SystemStorage, InvalidContact, EmptyCompatible, EmptyProperties, SystemProcessor, InvalidExtension, PropertyListener, InvalidCompatible, InvalidProperties, SystemMotherboard, PropertyLocalization, UnforeseenConsequences.
- **DialogType:** None, Close, Continue, Download, Remember.
- **EngineType:** AuroraLive, NebulaLive, VexanaLive, XavierLive, WebViewLive, CefSharpLive, MpvPlayerLive, VlcPlayerLive.
  - These sub-enums alias EngineType values: GifEngineType (Vexana, Xavier, WebView, CefSharp, MpvPlayer, VlcPlayer), UrlEngineType and WebEngineType and YouTubeEngineType (WebView, CefSharp), VideoEngineType (Nebula, WebView, CefSharp, MpvPlayer, VlcPlayer), ApplicationEngineType (Aurora).
- **InputModuleType:** RawInput (`Native` is commented out).
- **PerformanceType:** Close, Pause, Resume.
- **HighPerformanceType:** Default, PowerSaving, HighPerformance.
- **PausePerformanceType:** Heavy, Light.
- **NetworkPerformanceType:** Not, Ping, Upload, Download.
- **CategoryPerformanceType:** Not, Lock, Focus, Sleep, Memory, Remote, Battery, Console, Graphic, Network, Session, Virtual, Processor, FullScreen, ScreenSaver, BatterySaver.
- **PropertiesType:** Base, WebView, CefSharp, MpvPlayer, VlcPlayer.
- **ReportThemeType:** Spam, Other, Nudity, Harmful, Violence, Malicious, CopyrightViolation, MisleadingInformation.
- **SortKindType:** Ascending, Descending.
- **SortModeType:** Name, Creation, Modification.
- **StoreServerType:** GitHub, Soferity.
- **StoreStageType:** Full, Broken, Unknown.
- **StretchType:** None, Fill, Uniform, UniformToFill.
- **TransitionCycleType:** Random, Sequential.
- **UpdateAutoType:** Visible, SemiSilent, UpdateSilent, CompleteSilent.
- **UpdateModuleType:** Native, Downloader.
- **UpdateServerType:** GitHub, Soferity.
- **WallpaperType:** Gif, Url, Web, Video, YouTube, Application.
- **Core enums** (`S/Sucrose.Shared.Core/Enum/*.cs`, with `[Display]` names):
  - FrameworkType: Unknown, NET_6_0 … NET_15_0, NET_Framework_4_8, NET_Framework_4_8_1.
  - UpdateChannelType: Release, PreRelease.
  - UpdateExtensionType: Compressed (`.7z`), Executable (`.exe`).

## 7. Resources (`L/Sucrose.Resources/*`)
- **Format:** WPF `ResourceDictionary` XAML holding `<system:String x:Key="Area.Prefix.Key[.Suffix[.Last]]">`, compiled into `Sucrose.Resources.dll`. The pack URI is `/Sucrose.Resources;component/Locales/Locale.{LANG}.xaml`.
- **`Locales/Locale.{LANG}.xaml` merges:**
  - Language-neutral dictionaries: `Locale.xaml` (display names with versions, e.g. `English (v1.5)`), `Portal/Portal.xaml` (Release, Copyright, Quoting0..N), `Discord/Discord.xaml` (image keys), `Launcher/Launcher.xaml` (icon file names).
  - Per-language dictionaries: `Live`, `Undo`, `Update`, `Portal`, `Discord`, `Launcher`, `Property`, `Watchdog` `.{LANG}.xaml`.
  - `Portal.{LANG}.xaml` merges Area, Enum, Category, Controls (DisplayPreferences, ExpanderCard, LibraryCard, OtherAbout, OtherHelp, Pagination, StoreCard, ThemeCreate/Delete/Edit/Import/Report/Review/Share, TrayIcon, WallpaperCycling), StorePage (Broken/Full/Search/Unknown), MainWindow, LibraryPage (Empty/Full/Search) and SettingPage (Donate/General/Other/Performance/Personal/System/Wallpaper).
  - EN: 47 `*.EN.xaml` files, about 987 `x:Key` lines (grep count).
- **Languages (25, `Helper/Resources.cs:ListLanguage`):** AR, CS, DA, DE, EL, EN, ES, FR, HE, HI, ID, IT, JA, KO, MS, NB, NL, PL, PT, RO, RU, SV, TR, UK, ZH. `ListLanguageManipulated` prepends `""` (auto).
- **Loading (`SetLanguage`):**
  1. Uppercase the code; fall back to `EN` if the dictionary does not load.
  2. Remove merged dictionaries whose source contains `Locales/` and add the new one.
  3. Set `Skylark Culture.All`.
  4. Apply RTL `FlowDirection` for RTL cultures (AR, HE) to all windows except engine render windows (namespace `Sucrose.Shared.Engine.<X>.View`). Images, ImageIcon, AsyncImage and `Tag="LeftToRight"|"NonFlowable"` stay LTR. TextBoxes auto-flip when text contains U+0590–U+08FF.
- **Lookup (`Extension/Resources.cs`):** `GetValue(parts...)` joins the parts with `.` and calls `Application.Current.TryFindResource`. A missing key yields `"[key]"`; `CheckBack` detects that fallback.
- **Assets:**
  - Launcher `.ico` files: Exit, Sent, TrayIcon, Reload, TestTube, Fire, LightOn, Change, LightOff, Engineering, DashboardLayout, ColorPalette, TrayIcon.Grayscale. They are resolved to `<app>\Assets\<file>` by `S/Sucrose.Shared.Space/Helper/Assets.cs`.
  - Discord asset keys: `logo-3840x3840`, `robot-1..`.
  - `Styles/Button.xaml` (SucroseDark/LightButton) and `Styles/ScrollBar.xaml` (SucroseDarkScrollBar and brushes).
  - `Properties/` is empty.
- **Enum display strings:** `Portal/Enum/Portal.Enum.{LANG}.xaml` has keys `Portal.Enum.<EnumName>.<Value>`.

## 8. Mpv.NET and XamlAnimatedGif

### Mpv.NET
Location: `L/Sucrose.Mpv.NET`, a fork with namespace `Sucrose.Mpv.NET`. Usage is in `S/Engine/Sucrose.Shared.Engine.MpvPlayer/**`.
- **Native library:** `Requirements\libmpv-{x86|x64|ARM64}.dll`, loaded with [WIN] `kernel32 LoadLibrary`/`GetProcAddress`/`FreeLibrary` (`API/Interop/Utils/WindowsDllLoadUtils.cs`).
- **Constructor:** `MpvPlayer(hwnd, libPath)` sets mpv `wid` to the host HWND, so the video is embedded in a window. It observes `pause`, `eof-reached` and `paused-for-cache`; the library sets a default Volume of 50.
- **Properties used:** AutoPlay, Loop, Volume, Speed, Remaining, IsMediaLoaded; event MediaLoaded.
- **Methods used:**
  - `Load(path[, force])`, which runs the `loadfile` command with replace or append.
  - `LoadConfig(MpvPlayer.config)`, Pause, Resume, Stop, Dispose.
  - `API.SetPropertyString` for `keepaspect`, `panscan` and `video-unscaled` (stretch modes).
  - `API.SetPropertyString`/`Double`/`Long` for arbitrary mpv properties taken from the engine-level SucroseProperties (`Helper/Properties.cs`), e.g. `cscale-blur`, `dscale-blur`, `tscale-blur`.
- **Unused:** playlists, seeking, yt-dlp, KeepOpen, the render-context API and structs, the Linux loader.
- `Player/WinFunctions.cs` holds a [WIN] user32 `DestroyWindow` import.

### XamlAnimatedGif
Location: `L/Sucrose.XamlAnimatedGif`, a fork.
- **Attached properties** on WPF `Image` (Portal `LibraryCard`, `StoreCard`, `TrayIcon.xaml`, `MainWindow`; Xavier engine): `SourceUri`, `RepeatBehavior`, `AutoStart`, `CacheFramesInMemory`, `AnimateInDesignMode`, plus `AddLoadedHandler`.
- **Globals:** `SetHttpClient(shared client)`, `SetClientUserAgent`, `SetDownloadCacheLocation(Cache\Store\Temporary)`, `SetDownloadCacheExpiration(StoreDuration h)`. The cache file name is the SHA-256 of the URI (`UriLoader.cs`).
- **Animator:** Play, Pause, Resume, Rewind (Xavier `Helper/Gif.cs`).
- **In-house pieces to replace:** the GIF decoder (`Decoding/*`, LZW) and the frame timing.

## 9. Other non-engine Shared projects (brief)
- **Shared.Live:**
  - `Run.Start` validates the selected Info and checks AppVersion against the app version, then:
    - refreshes the desktop;
    - optionally sets the thumbnail as the OS wallpaper (`Space/Helper/Background.cs`, Skylark.Wing [WIN]);
    - optionally disables the slideshow (`Skylark.Wing BackgroundImage.DisableSlideshow` [WIN]);
    - starts Backgroundog and the engine for the theme's Type through Commandog.
  - `Kill.Stop` kills the live engine and the Aurora app process, resets the wallpaper and refreshes the desktop.
  - `StopSubprocess` kills `msedgewebview2` and `CefSharp.BrowserSubprocess` processes whose command line contains "Sucrose": [WIN] WMI `Win32_Process.CommandLine` with a PowerShell `Get-CimInstance` fallback, resumed first via `ntdll NtResumeProcess`.
- **Shared.Space** (Windows-specific helpers):
  - `Extension/Lifecycle.cs` [WIN ntdll NtSuspendProcess/NtResumeProcess, kernel32 Suspend/ResumeThread/OpenThread].
  - `Extension/GraphicPreference.cs` [WIN registry `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, value `GpuPreference=N;` keyed by exe path].
  - `Helper/User.cs` [WIN WMI Win32_ComputerSystemProduct.UUID, Win32_OperatingSystem.OSLanguage, Win32_ComputerSystem, Win32_VideoController, Win32_NetworkAdapter, Win32_Processor; WindowsIdentity].
  - `Helper/Power.cs` [WIN `powercfg /requestsoverride PROCESS "<exe>" DISPLAY SYSTEM` via elevated cmd].
  - `Helper/Processor.cs` (`Process.Start` with optional `runas`; kill by name).
  - `Helper/Remote.cs` and `Helper/Virtual.cs` (process-name lists for remote desktop and Android emulators).
  - `Helper/Windows.cs` (build thresholds 14393, 19041, 22000, 22621, 26000).
  - Not inspected: `Dark.cs`, `Security.cs`, `Instance.cs`.
- **Shared.Core:** framework, architecture and OS info; total memory via [WIN] WMI `Win32_OperatingSystem.TotalVisibleMemorySize`.
- **Shared.Dependency:** `Helper/Runtime.cs` sets `DOTNET_ROOT*`, `DOTNET_ROLL_FORWARD=LatestMajor` and prepends `Sucrose.Runtime` to PATH (.NET-only, can be dropped); `Helper/Graphic.cs` configures the GPU preference.
- **Shared.Discord:** DiscordRPC client.
  - Every 5 s, if DiscordConnect is on and `Discord.exe` or `DiscordPTB.exe` is running, it initializes and refreshes every DiscordRefreshDelay seconds.
  - Presence: Details; random StatementText0..48; buttons Browse (GitHub) and Download (Microsoft Store); large image `logo-3840x3840`; random LargestText and SmallestText (0..23); random SmallestImage0..36.
- **Shared.Launcher:** WinForms `NotifyIcon` + `ContextMenuStrip` with Dark/Light renderers and rounded corners [WIN Shell_NotifyIcon via WinForms]. Commands: Interface, Property, Feedback, Setting, Update, Report, Reportdog, Engine (open/close/change/reload/customize), Reload, Close, Version; plus the log-bundle `Create`.
- **Shared.Watchdog:** exception-to-log extension and an in-memory `Hashtable` dataset.

## Porting notes
1. **Two different mutex hashes for the same path.** Manager uses the first 16 bytes of SHA-256; Space `Filing` uses MD5. So `SettingManager` and `Filing` (settings import/export/reset, theme files) never exclude each other. The text form is Skylark `Guidly.GuidToText` (unclear).
2. **Settings writes are not atomic.** Truncate-then-write means a crash leaves an empty or partial file, and the next startup **resets the whole file** to `{"Properties":{}}`. All errors are swallowed, so getters return defaults and setters drop writes silently.
3. **Cross-process lost updates.** The read-modify-write cycle is not under one lock.
4. **Value typing quirks to replicate:**
   - Newtonsoft deserializes any ISO-date-looking string inside `object` as a `DateTime`, so a string setting whose value looks like a date reads back as the default.
   - Integers must be read leniently.
   - Enums are written as names and parsed case-sensitively; numeric strings are also accepted.
5. **Required-but-nullable fields.** `SucroseInfo.json` `License`, `Tags` and `Arguments` must be present as keys (may be null). `Type` is an int on write, and an int or name on read.
6. **Archive check quirks.**
   - `CheckFile` matches by Name or FullName, but `ReadFile` uses FullName only.
   - `CheckFolder` inspects only the first entry.
   - The extension check is case-sensitive `.zip`.
   - For Gif and Video with a URL source, the extension check still runs on the URL.
   - The `Tags.Count()<1` branch is unreachable.
7. **Properties cache naming is inconsistent.** The Property app and engines use `<folder><PropertiesType>` with no extension; `LibraryPage` builds `<folder>.json`; `LibraryCard` deletes any cache file whose path contains the folder name.
8. **Backslashes in non-Windows contexts.** `Path.Combine(Source, Key)` is used as a store URL and API path, and log-zip entry names are `Log\file`. A Rust port should use `/`.
9. **HTTP client accepts any TLS certificate** and enables SSL2/3. The GitHub PAT is stored in plaintext (`Objectionable.json`), excluded only from the log bundle.
10. **Signal locking is ineffective.** The reader and writer use different mutex names, so there is no real exclusion. Deletion relies on a 3 s timer; FileSystemWatcher can overflow.
11. **Launcher Pipe and Transmission managers and services are dead code.** Only Signal is wired, and every Launcher message also resets the tray icon state (`Icon(Data.Icon)` is called unconditionally).
12. **Engine identity strings.** Engine settings store full `EngineType` names (`"CefSharpLive"`), not the sub-enum names. Server-side identity (`{guid}` in Soferity URLs) is the MD5 of WMI-derived values; replicate it to keep user identity stable.
13. **Version semantics.** Version compatibility uses .NET `System.Version` comparison (-1 for missing components) against the app version `yy.MM.dd.0`.
14. **Pattern.json and cache expiry.** Pattern.json must satisfy the Store schema (Categories required). Per-card cache expiry uses the directory mtime.
15. **Every process creates all 14 setting files** (static initializer), so a Rust port that lazily creates files changes observable behaviour slightly.
16. **Names and enums out of sync:**
    - The alias `SEEST` means ExpandScreenType in Manager but EnergySaverType in Backgroundog.
    - The localized enum lists include values the code lacks: `SortModeType.None`, `InputModuleType.Native`.
    - `CompatibilityType.TriggerTime`, `InvalidFile` and `Encrypt` are unused.
    - The key `CefsharpContinue` is spelled with a lowercase "s".
17. **External enums are known only by name.** Skylark enum value sets (StorageType, DisplayScreenType, InputType, ScreenType, WindowsThemeType) are known from usage and localization only; their ordinals are unclear. Since they are stored as names, compatibility holds as long as the names match.
18. **Dead code:** `SettingManager2` and `SettingManager3`.
