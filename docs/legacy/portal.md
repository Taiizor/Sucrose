# Legacy reference: Portal UI

> Inventory of the **C# app** (not the Rust rewrite), produced on 2026-11-21 by reading the sources in `src/`. Use it as the behavior spec when porting. Items marked "unclear" were not verified; when you find a mistake, fix it here.

**Path roots used below (relative to the repository root):**
- `P` = `src/Portal/Sucrose.Portal`
- `S` = `src/Shared`
- `L` = `src/Library`
- `LOC` = `src/Library/Sucrose.Resources/Locales`

**Settings storage.** Each `SMMI.<X>SettingManager` is one JSON file at `%APPDATA%\Sucrose\Setting\<X>.json` (`L/Sucrose.Manager/Manage/Internal.cs`, `L/Sucrose.Manager/SettingManager.cs`). `%APPDATA%` here is `SpecialFolder.ApplicationData`, i.e. Roaming.
- Key strings are identical to the constant names (`L/Sucrose.Memory/Manage/Constant/*.cs`), e.g. `Engine.json:"WallpaperVolume"`.
- Enums are written **by name** through the custom `EnumConverter` (`L/Sucrose.Manager/Converter/EnumConverter.cs`).
- Defaults and clamps:
  - `L/Sucrose.Manager/Manage/*.cs` (bool, int, string values)
  - `S/Sucrose.Shared.Dependency/Manage/Manager/*.cs` (enum values)
  - `S/Sucrose.Shared.Core/Manage/Manager/Update.cs` (Update.json enum values)
  - `P/Manage/Manager/Portal.cs` (BackdropType)
- Other data folders: Library defaults to `%APPDATA%\Sucrose\Library`; cache is `%APPDATA%\Sucrose\Cache`; logs are `%APPDATA%\Sucrose\Log` (`L/Sucrose.Memory/Manage/Readonly/Folder.cs`, `L/Sucrose.Memory/Manage/Readonly/Path.cs`).

**Commandog.** "Commandog X" below means the Portal launches the `Sucrose.Commandog` process with the arg string `✔X✖v1✖v2…` (`SSSHP.Run(SSSMI.Commandog, …)`; separators are defined in `L/Sucrose.Memory/Manage/Readonly/General.cs`).

---

## 1. Window chrome and navigation

Files: `P/Views/Windows/MainWindow.xaml(.cs)`, `P/ViewModels/Windows/MainWindowViewModel.cs`, `LOC/Portal/MainWindow/Portal.MainWindow.EN.xaml`

**Window**
- WPF-UI `FluentWindow`, 900×600 (minimum 900×600), centered on screen, `ExtendsContentIntoTitleBar`.
- Starts `Topmost=True` and `ShowInTaskbar=False`. On `ContentRendered` it switches to Topmost=false and ShowInTaskbar=true.
- F5 is swallowed.
- Title "Sucrose Wallpaper Engine". WPF-UI `TitleBar` shows the `Icon256.png` icon; double-clicking the icon closes the window.
- The window holds a `ContentDialogHost` and a `SnackbarPresenter`.

**Navigation**
- `NavigationView` with `PaneDisplayMode=Top`.
- Library mode shows the items named `General*`: "Library" (LibraryPage) and "Store" (StorePage).
- The footer gear item ("Settings") switches to settings mode:
  - It shows a Back arrow item ("Go Back"), plus General, Personal, Performance, Wallpaper, System and Other.
  - Donate is commented out in the XAML.
  - It hides the whole header action dock and navigates to General.
- Back returns to Library mode.

**Command-line arguments** (`MainWindow.xaml.cs`)
- `argv[1] == "SystemSetting"` opens settings mode on the System page.
- `argv[1] == "GeneralSetting"` opens settings mode on the General page.
- Anything else opens Library.
- The other `ArgumentCommandType` values (`OtherSetting`, `DonateSetting`, `PersonalSetting`, `WallpaperSetting`, `PerformanceSetting`) exist in `S/Sucrose.Shared.Dependency/Enum/CommandType.cs` but the Portal does not handle them.

**Header actions** (in `PaneFooter`, left to right)
1. **SearchBox**
   - Placeholder "Search for a Wallpaper", MaxLength 20, debounced 500 ms. It sets the shared `SearchService.SearchText`, which is lowercased.
   - If `General.json:TelemetryData` is on, it POSTs `{ActivePage, AppVersion, QueryText}` to `https://sucrose.soferity.com/v8/Telemetry/Search/{userGuid}`.
   - Its left margin is recomputed whenever the window is resized.
2. **Light/Dark toggle** (two buttons, only one visible at a time).
   - Flips `General.json:ThemeType` (Skylark `WindowsThemeType` Dark/Light) and calls `ApplicationThemeManager.Apply`.
   - At startup, if `Portal.json:BackdropType == Auto`, the Windows theme is followed and written back to `ThemeType`. **[WIN]** via `Skylark.Wing.Helper.WindowsTheme.GetTheme`; the underlying API is unclear (probably the `AppsUseLightTheme` registry value).
3. **Premium** ("Premium Theme Store"): Commandog `Premium` opens `https://sucrose.dev/premium`.
4. **Create Wallpaper** (+): opens `ThemeCreate`. On Primary:
   - If Backgroundog has not closed or paused the wallpaper (or is not running) and `Engine.json:LibraryStart` is true, it sets `Library.json:Selected` to the new theme, stops any running live process and starts the live engine again.
   - It then fires `LibraryService.CreatedWallpaper`, which makes the Library reload.
5. **Wallpaper Cycling**: opens the `WallpaperCycling` dialog.
6. **Display Preferences**: opens the `DisplayPreferences` dialog.
7. **Donate options**: disabled and always collapsed (`GetDonater()` hard-returns Collapsed).
8. **⋮ Other Options**: context menu with "Help" (`OtherHelp`) and "About" (`OtherAbout`).

**Status bar** (left to right)
- Apps icon.
- `Portal.Release` = "Pre-Release" (from the language-neutral `LOC/Portal/Portal.xaml`).
- Framework name.
- Architecture.
- Version.
- Process memory, refreshed by a 1 s `DispatcherTimer`.
- A random quote `Portal.Quoting{0..85}`: 86 entries in `Portal.xaml`, picked with `Randomise.Next(86)`.
  - Clicking it copies the text to the clipboard. **[WIN]** `Skylark.Clipboard`.
  - If the copy throws, it opens Google Translate through Commandog `QuotingTranslate`.
- `Portal.Copyright` text.

At startup the ViewModel also writes a system-info dictionary to `Kernel.json:"Information"`.

**Backdrop and background image**
- `WindowBackdropType` comes from `Portal.json:BackdropType` (default `None`, set in `P/Manage/Internal.cs`). If `WindowBackdrop.IsSupported` fails, it falls back to None. **[WIN]** DWM system backdrop through WPF-UI (Mica, Acrylic, Tabbed).
- A full-window `VisualBrush` displays `Portal.json:BackgroundImage`. Animated GIFs work (XamlAnimatedGif). Stretch comes from `BackgroundStretch`; opacity is `BackgroundOpacity / 100`.
- Changes apply live through `BackdropService` events (`P/Services/BackdropService.cs`).

**Tray interaction from the Portal**
- `MainWindow_Closing`: if `Warehouse.json:HintTrayIcon` (default true), the first close is cancelled and the `TrayIcon` hint dialog is shown. That dialog sets `HintTrayIcon=false`, then the window closes. The tray icon itself belongs to the Launcher process.
- The "Notification Area Icon" setting (see §4) sends the signal `Sucrose.Signal.Interface.Launcher{Show|Hide}` via `SSMI.LauncherManager.FileSave`, but only if the Launcher is running.

**Other startup behavior** (`P/App.xaml.cs`)
- Single instance via the named mutex `{Sucrose-Wallpaper-Engine-Portal}` plus a process-count check (`S/Sucrose.Shared.Space/Helper/Instance.cs`). **[WIN]** named Mutex. A second instance just exits.
- Language: `SRHR.SetLanguage(General.Culture)`.
- Global exception hooks hand off to Watchdog.
- XamlAnimatedGif's HTTP cache lives in `Cache\Store\Temporary` and expires after `StoreDuration` hours.

## 2. Library page

Files: `P/Views/Pages/LibraryPage.xaml(.cs)`, `P/Views/Pages/Library/{Full,Empty,Search}LibraryPage.xaml(.cs)`, `P/Views/Controls/LibraryCard.xaml(.cs)`, `P/Controls/LibraryStackPanel.cs`, `S/Sucrose.Shared.Space/Helper/Sort.cs`

**Load**
- **Showcase:** on first sight, each folder in `%APPDATA%\Sucrose\Showcase\*` is copied into the library. Names are tracked in `Warehouse.json:Showcase` so each is copied only once.
- **Scan:** a folder of `Library.json:Location` is a valid theme when all of these hold:
  - `SucroseInfo.json` parses;
  - the Preview and Thumbnail files exist;
  - Source is either a URL or an existing file.
- **Corrupt themes:** if `Library.json:DeleteCorrupt` is on, invalid folders are deleted (only if they are accessible and not locked), together with `Cache\Properties\<name>.json`.
- **Display:** a progress ring shows, then after 500 ms `FullLibraryPage` (themes exist) or `EmptyLibraryPage` (no themes) appears.

**Sorting** (`Sort.cs`)
- `Portal.json:LibrarySortMode`:
  - `Name`: the localized title.
  - `Creation` (default): directory creation time.
  - `Modification`: last-write time of `SucroseInfo.json`.
- `Portal.json:LibrarySortKind`: `Ascending` or `Descending` (default Descending).
- The same pass builds a search index per theme: localized title + description + tags, lowercased, punctuation removed, de-duplicated.

**Search** (shared `SearchService`)
- The query is split on spaces after punctuation is removed.
- A theme matches when at least one query word is a substring of an index token. Results are ordered by number of matched words, descending.
- If no card is displayed, `SearchLibraryPage` is shown.

**Pagination**
- `Portal.json:LibraryPagination` items per page (1–100, default 30).
- Cards are added one every 50 ms.
- `MaxPage = ceil(count / pageSize)`.
- When a card is deleted and the page becomes empty, the page is reloaded or the previous page is selected.

**Layout** (`LibraryStackPanel`)
- Wrap layout. Item margin is `Portal.json:AdaptiveMargin`.
- At most `AdaptiveLayout` cards per row (0 = unlimited).
- Spare row width is spread over the cards (each card is 260–400 px wide, 160 px high).

**Card** (`LibraryCard`)
- Shows the thumbnail (downscaled decode at 360 px, `P/Extension/ImageLoader.cs`) with the localized title and description. RTL text is auto-detected.
- **Hover:**
  - Hand cursor, unless the theme is currently selected and running, or its version is incompatible.
  - If `LibraryPreview` is on, the Preview GIF animates.
  - If `LibraryPreviewHide` is also on, the title/description strip is hidden during the preview.
- **Left-click = Use:**
  - Requires `Info.AppVersion <= app version`.
  - If Backgroundog has not closed or paused the wallpaper (or is not running) and the theme is not already selected and running: set `Library.json:Selected`, stop the live engine if it is running, start it again.
- **Incompatible version:** a red "BoxDismiss" button ("Update Sucrose to use this theme") replaces "…"; clicking it runs Commandog `Update`.
- **Context menu** (opened by right-click or the "…" button):
  - **Review**: `ThemeReview` dialog.
  - **Use**: disabled with " (Selected)" if this theme is selected and running, " (Incompatible)" if the version is too new, " (Closed)" or " (Paused)" if Backgroundog has closed or paused the wallpaper.
  - **Preview**: always disabled; the handler does nothing.
  - **Customize**: Commandog `PropertyA` launches `Sucrose.Property` with the theme folder name. It is enabled when:
    - Web: the theme has `SucroseProperties.json`;
    - Gif/Video: depends on the selected engine. Vlc/Mpv need the theme file or an engine cache `Cache\{VlcPlayer|MpvPlayer}\SucroseProperties.json`; CefSharp/WebView need `Cache\{CefSharp|WebView2}\SucroseProperties.json`;
    - YouTube: CefSharp or WebView cache file;
    - otherwise disabled.
  - **Add to Cycle / Remove from Cycle**: only visible when `Cycling.json:Active`. Edits the `Cycling.json:Exclusion` list.
  - **Find**: opens the folder via the shell. **[WIN]** Explorer.
  - **Share**: `ThemeShare` dialog.
  - **Delete**:
    - Asks for confirmation with `ThemeDelete` if `Library.json:DeleteConfirm`.
    - Then deletes every `Cache\Properties` file whose name contains the theme name, then the theme folder.
    - Disabled while the theme is selected and running.
  - **Edit**: `ThemeEdit` dialog; the card refreshes after a save.

**Drag and drop onto the library page** (a dashed DodgerBlue rectangle shows while dragging)
- One non-zip file:
  - `.url`: parse its `URL=` line and open ThemeCreate prefilled with that URL;
  - gif, video, web or app extension: open ThemeCreate prefilled with the file.
- Files that are not handled above: each archive is validated (`SSZHA.Check`, producing `CompatibilityType`), then extracted into `Library\<25 random [A-Za-z0-9]>`. The `ThemeImport` dialog then lists per-file results.
- Dropped text that is a URL: ThemeCreate with that URL (YouTube URL → YouTube type, other URL → Url type).

**Empty and search-empty texts**
- No themes: "Your Library is Woven by Spider Webs"
- No search matches: "Your Library Enchanted with Spider Webs"

## 3. Store page

Files: `P/Views/Pages/StorePage.xaml.cs`, `P/Views/Pages/Store/*.xaml(.cs)`, `P/Views/Controls/StoreCard.xaml.cs`, `P/Services/StoreService.cs`, `S/Sucrose.Shared.Store/Helper/{Store,Soferity/Download,GitHub/Download}.cs`

**Stage** (`StoreStageType`)
- **Broken**: none of the hosts resolves via DNS. Hosts: bing, baidu, yahoo, google, yandex, microsoft, cloudflare, duckduckgo (`S/Sucrose.Shared.Space/Helper/Network.cs`). Shows "A Moment When Internet Lights Dimmed" with a WifiOff icon.
- **Unknown**: the store index could not be downloaded or parsed ("Uncover the Mysterious State…").
- **Full**: otherwise.

**Server** (`Portal.json:StoreServerType`)
- **Soferity** (default):
  - Index: `https://sucrose.soferity.com/Store/src/Store.json`.
  - Pattern file: `https://sucrose.soferity.com/v8/Kernel/Pattern`. If it downloads successfully, the store is read from `Pattern.json` instead of `Store.json`.
  - File listing: `/v8/Kernel/File/Store/<path>`.
- **GitHub**:
  - Contents API of `Taiizor/Store`, folder `src`, branch `develop`.
  - Raw files from `https://raw.githubusercontent.com/Taiizor/Store/develop`.
  - Requests use `Authorization: Bearer <Objectionable.json:PersonalAccessToken>`.
- Cache location: `%APPDATA%\Sucrose\Cache\Store\Store.json` and `Pattern.json`. Both are refreshed when older than `Portal.json:StoreDuration` hours or when invalid.
- Schema (`S/Sucrose.Shared.Store/Interface/*.cs`): `Categories{name → Wallpapers{key → {Size, Adult, Live, Cover, Source, Pattern}}}`.

**Categories**
- Left `NavigationView`. "All" (Home icon) is first; the rest are sorted by localized name.
- Lookup key is `Portal.Category.<name without spaces>`; icons come from the map in `P/Manage/Internal.cs` (default Wallpaper icon).
- A category is hidden if it has no wallpapers, or if every wallpaper in it is `Adult` while `Portal.json:StoreAdult == false`.

**Adult filter**
- Wallpapers with `Adult=true` are skipped unless `StoreAdult` (default **true**).

**Search**
- Each word is substring-matched against `Wallpaper.Pattern` (falls back to the lowercased key). Results are ordered by match count and respect the selected category.
- No results: `SearchStorePage` ("The Answer to What You're Looking For May Be Hidden in the Sky").

**Pagination**
- `Portal.json:StorePagination` items per page (default 30). Same control and 50 ms per-card insertion as the Library.
- A search change resets to page 1.

**Card** (`StoreCard`)
- **On load:** fetches `SucroseInfo.json` and the cover into `Cache\Store\<Category>\<sanitized key>\`.
  - These are also refreshed after `StoreDuration` hours.
  - Duplicate fetches are de-duplicated through `SPMI.StoreDownloader` / `StoreDownloading`.
  - If the fetch fails, a "Warn" overlay is shown (a full crimson ring).
- **Preview:** on hover, if `StorePreview` is on, the remote GIF `{server root}/{Source}/{Key}/{Live}` animates. `StorePreviewHide` hides the details strip.
- **Install** (cloud button, card click, or menu "Install"):
  1. Checks DNS. If offline, a red CloudDismiss shows for 3 s.
  2. If `TelemetryData`, POSTs `/v8/Telemetry/Download/{guid}`.
  3. Downloads recursively into `Cache\Store\Temporary\<key25>`. Progress ring with a "xx.xx%" tooltip is driven by `SSSTMI.StoreService.Info[key]`; a re-created card resumes the progress display by Guid = `Source/Key`.
  4. Copies the result into `Library\<key25>` and writes the marker `SucroseStore.json`.
  5. If `Engine.json:StoreStart` (default true) and the wallpaper is not closed or paused: set Selected and restart the live engine.
  6. Success: green CloudCheckmark for 3 s. Error: red CloudOff for 3 s.
- **Incompatible** (AppVersion is newer than the app): red BoxDismiss icon, and Commandog `Update` is launched.
- **Menu:**
  - "Check for Updates": only when incompatible; shows "Checking for Updates" while the Update process runs.
  - "Install": disabled with " (Incompatible)" when incompatible.
  - "Report": enabled once Info is loaded; opens `ThemeReport`.

## 4. Settings

All settings pages are built in code (`ExpanderCard` list) inside `P/ViewModels/Pages/Setting/*ViewModel.cs`. English strings are in `LOC/Portal/SettingPage/<Page>/Portal.<Page>.EN.xaml`. Page shells are `P/Views/Pages/Setting/*.xaml.cs`.

### General (`GeneralSettingViewModel.cs`)

| Section / label | Control | Options / range | Default | Key / effect |
|---|---|---|---|---|
| **Appearance & Behavior**: Application Language | ComboBox | AR CS DA DE EL EN ES FR HE HI ID IT JA KO MS NB NL PL PT RO RU SV TR UK ZH (display names from `Locale.<XX>`) | current UI two-letter ISO code | General.json `Culture` (see §6) |
| Startup at Application Launch | ComboBox | None / Normal / Priority / Scheduler (index 0–3) | 0 | General.json `RunStartup`, plus Commandog `Startup` (Normal), `StartupP` run elevated (Priority; on failure shows "Administrator Rights Warning"), `Scheduler Create/Delete` (Scheduler). The previous mode is removed first. **[WIN]** Run-key / Task Scheduler via Skylark.Wing (exact API unclear) |
| Notification Area Icon | ComboBox | Visible / Invisible | Visible | General.json `AppVisible`, plus Launcher signal Show/Hide |
| ↳ "Close all Sucrose applications when the system tray is closed" | CheckBox | | true | General.json `AppExit` |
| Window Backdrop | ComboBox | WPF-UI `WindowBackdropType` (None, Auto, Mica, Acrylic, Tabbed); every item except None is disabled unless running on Windows 11 | None | Portal.json `BackdropType`, applied live **[WIN]** |
| ↳ Background image | Button (file dialog; png jpg jpeg tiff webp gif) + Remove button | rejects files without access ("…dont have permission…") or locked ("…used by another process") | "" | Portal.json `BackgroundImage` |
| ↳ "Background Stretch:" | ComboBox | WPF `Stretch`: None / Fill / Uniform / Uniform to Fill | UniformToFill | Portal.json `BackgroundStretch` |
| ↳ "Background Opacity (%):" | NumberBox | 0–100 | 100 | Portal.json `BackgroundOpacity` |
| **Sound**: Engine Volume | Slider (icon changes with level) | 0–100 | 100 | Engine.json `WallpaperVolume` |
| ↳ "Play sound only when focused on the desktop" | CheckBox | | false | Engine.json `VolumeDesktop` |
| ↳ "Play sound only when all applications are silent" | CheckBox | | false | Engine.json `VolumeSilent` |
| ↳ "Sensitivity (Seconds):" | NumberBox | 1–10 | 5 | Engine.json `VolumeSilentSensitivity` |
| **Library**: Library Location | Button (FolderBrowserDialog **[WIN]** WinForms) + Open-folder button | with Move on, the destination must be empty ("Please select an empty folder"). Shows "Please wait…" while copying; stops and restarts the live engine | `%APPDATA%\Sucrose\Library` | Library.json `Location` |
| ↳ "Move your current library to the new location" | CheckBox | | true | Library.json `Move` |

### Personal (`PersonalSettingViewModel.cs`)

| Label | Control | Options / range | Default | Key / effect |
|---|---|---|---|---|
| **Store**: Store Server | ComboBox (+ hint about the PAT) | GitHub / Soferity | Soferity | Portal.json `StoreServerType`; on change deletes `Store.json` and `Pattern.json` and sets `Store.Internal.State=true` |
| Store Refresh | NumberBox | 1–24 (hours) | 3 | Portal.json `StoreDuration` |
| Auto Start (store) | ToggleSwitch | | true | Engine.json `StoreStart` |
| Unsafe Content | ToggleSwitch | | **true** | Portal.json `StoreAdult` |
| **Library**: Deletion Confirmation | ToggleSwitch | | true | Library.json `DeleteConfirm` |
| Corrupt Themes (+ data-loss hint) | ToggleSwitch | | false | Library.json `DeleteCorrupt` |
| Auto Start (created theme) | ToggleSwitch | | true | Engine.json `LibraryStart` |
| **Appearance**: Theme Preview → Store Preview / Hide Details / Library Preview / Hide Details | 4 ToggleSwitches | | all false | Portal.json `StorePreview`, `StorePreviewHide`, `LibraryPreview`, `LibraryPreviewHide` |
| Adaptive Layout → "Spacing Distance (Pixels):" | NumberBox | 5–25 | 5 | Portal.json `AdaptiveMargin` |
| ↳ "Layout Order (Count):" | NumberBox | 0–100 (0 = unlimited) | 0 | Portal.json `AdaptiveLayout` |
| Library Sorting → Sort Mode / Sort Type | 2 ComboBoxes | Name / Creation / Modification; Ascending [A-Z] / Descending [Z-A] | Creation, Descending | Portal.json `LibrarySortMode`, `LibrarySortKind` |
| Store Pagination | NumberBox | 1–100 | 30 | Portal.json `StorePagination` |
| Library Pagination | NumberBox | 1–100 | 30 | Portal.json `LibraryPagination` |

### Performance (`PerformanceSettingViewModel.cs`)

All keys are in **Backgroundog.json**. "Action" means `PerformanceType` Close / Pause / Resume.

| Label | Control | Range / options | Default | Key |
|---|---|---|---|---|
| Performance Counters (+ hint) | ToggleSwitch | on: Commandog `Backgroundog` (start it); off: kill the Backgroundog process | true | `PerformanceCounter` |
| Pause Rule | ComboBox | Heavy / Light | Light | `PausePerformanceType` |
| Communication Type ("…messages to themes of type Web") | ComboBox | Pipe / Signal / Transmission | Transmission | `CommunicationType` |
| Processor Usage + "Processor Usage (%):" | Action + NumberBox 0–100 | | Resume, 70 | `ProcessorPerformance`, `ProcessorUsage` |
| Graphics Usage + "Graphics Processing Unit:" + "Graphics Usage (%):" | Action + ComboBox (from System.json `GraphicInterfaces`; empty-list text if none) + NumberBox 0–100 | | Resume, first adapter, 70 | `GraphicPerformance`, `GraphicAdapter`, `GraphicUsage` |
| Memory Usage + "Memory Usage (%):" | Action + NumberBox 0–100 | | Resume, 80 | `MemoryPerformance`, `MemoryUsage` |
| Network Usage + "Network Adapter:" | Action + ComboBox (System.json `NetworkInterfaces`) | | Resume | `NetworkPerformance`, `NetworkAdapter` |
| ↳ "Upload Size:" | NumberBox 0–99999999 + `StorageType` ComboBox (Bit…Yottabyte) | | 800 Kilobyte | `UploadValue`, `UploadType` |
| ↳ "Download Size:" | same controls | | 10 Megabyte | `DownloadValue`, `DownloadType` |
| ↳ "Ping Value (MS):" | NumberBox 0–1000 + host ComboBox (Bing, Baidu, Yahoo, Google, Yandex, Microsoft, Cloudflare, DuckDuckGo) | | 100, "Bing" | `PingValue`, `PingType` |
| **Operating System**: Session Lock / Sleep Mode / User Switch / Log Out / Screen Saver | Action each | | Close / Close / Close / Close / **Pause** | `LockPerformance`, `SleepPerformance`, `ConsolePerformance`, `SessionPerformance`, `ScreenSaverPerformance` |
| **Laptop**: Battery Power + "Battery Status (%):" | Action + NumberBox 0–100 (icon changes with value) | | Resume, 50 | `BatteryPerformance`, `BatteryUsage` |
| Battery Saver | Action | | Resume | `BatterySaverPerformance` |
| **System**: Virtualization / Remote Desktop / Full Screen Mode / Focus Mode | Action each | | Resume ×4 | `VirtualPerformance`, `RemotePerformance`, `FullScreenPerformance`, `FocusPerformance` |

### Wallpaper (`WallpaperSettingViewModel.cs`)

All keys are in **Engine.json** unless noted otherwise.

| Label | Control | Options | Default | Key / effect |
|---|---|---|---|---|
| Interaction Method | ComboBox | Skylark `InputType`: Closed / Mouse Only / Keyboard Only / Mouse & Keyboard | MouseKeyboard | `InputType` |
| ↳ "Interaction Module:" | ComboBox | RawInput (Native is commented out) | RawInput | `InputModuleType` |
| ↳ "Process interaction only when desktop is focused" | CheckBox | | false | `InputDesktop` |
| ↳ "Hide to avoid accidentally selecting desktop icons" | CheckBox | | live OS state | **Not stored.** Toggles Windows desktop icon visibility. **[WIN]** `Skylark.Wing.Utility.Desktop.Get/SetDesktopIconVisibility` (API unclear) |
| Screen Layout | ComboBox | Working Area / Display Bound | DisplayBound | `ScreenType` |
| Stretch Mode | ComboBox | None / Fill / Uniform / Uniform to Fill | UniformToFill | `StretchType` |
| Stay Awake | ToggleSwitch | also runs elevated `powercfg /requestsoverride PROCESS <WebView2/CefSharp exe> [DISPLAY SYSTEM]` (`S/Sucrose.Shared.Space/Helper/Power.cs`) **[WIN]** UAC + powercfg | false | `StayAwake` |
| Crash Mode | ToggleSwitch | | false | `CrashExplorer` |
| Loop Mode | ToggleSwitch | | true | `WallpaperLoop` |
| Shuffle Mode | ToggleSwitch | | true | `WallpaperShuffle` |
| Windows Slideshow | ToggleSwitch | on: `Skylark.Wing.Helper.BackgroundImage.DisableSlideshow()` **[WIN]** | false | `DisableSlideshowLive` |
| Background Image | ToggleSwitch | on: sets the Windows desktop wallpaper from the live wallpaper (`Shared.Space.Helper.Background.SetWallpaper`); off: `ResetDefault` **[WIN]** (API unclear) | false | `BackgroundImage` |
| ↳ "Disable Windows slideshow" | CheckBox | | false | `DisableSlideshow` |
| Hardware Acceleration | ToggleSwitch | | true | `HardwareAcceleration` |
| **Extensions**: Gif Player | ComboBox (raw enum names, not localized) | Vexana, Xavier, WebView, CefSharp, MpvPlayer, VlcPlayer | MpvPlayer | `Gif` (stored as `EngineType`, e.g. "MpvPlayerLive") |
| Video Player | ComboBox | Nebula, WebView, CefSharp, MpvPlayer, VlcPlayer | MpvPlayer | `Video` |
| **Engines**: Url Engine / Web Engine / YouTube Engine | ComboBox each | WebView, CefSharp | CefSharp | `Url`, `Web`, `YouTube` |
| Application Engine | ComboBox | Aurora | Aurora | `Application` |

Engine defaults come from `S/Sucrose.Shared.Space/Manage/Internal.cs`.

### System (`SystemSettingViewModel.cs`): action buttons only, no stored settings

Each hint shows a folder size, refreshed every 3 s.
- **Log Files**
  - "Create": SaveFileDialog to a zip of the `Log` and `Setting` folders, excluding `Objectionable.json`.
  - "Clear": Commandog `Temp <LogDir> <Launcher>`.
- **Clear Cache**: Commandog `Temp <Cache>`.
- **Clear Store**: Commandog `Temp <Cache\Store>`.
- **Clear Library**: Commandog `Temp <Library.Location>`.
- **Backup Settings**: "Backup" and "Restore" each use FolderBrowserDialog, then Commandog `Export` / `Import <dir>`. If the folder is not accessible, an "Access Warning" dialog is shown.
- **Reset Settings**: Commandog `Reset`.

### Other (`OtherSettingViewModel.cs`)

| Label | Control | Range | Default | Key / effect |
|---|---|---|---|---|
| **Data**: Automatic Error Reporting | ToggleSwitch | starts Reportdog through Commandog when either data switch is on; kills it when both are off | true | General.json `ExceptionData` |
| Share Usage Statistics | ToggleSwitch | same Reportdog logic | true | General.json `TelemetryData` |
| **Hook**: Discord | ToggleSwitch | | true | Hook.json `DiscordConnect` |
| ↳ "Connection Refresh:" | ToggleSwitch | | true | Hook.json `DiscordRefreshConnect` |
| ↳ "Refresh Interval (Seconds):" | NumberBox | 60–3600 | 60 | Hook.json `DiscordRefreshDelay` |
| **Priority**: Internet Agent | TextBox, **read-only**, max 100 | | `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit` | General.json `UserAgent` |
| Personal Access Token | Hyperlink (YouTube tutorial) + TextBox max 93 | saved only when exactly 93 characters, otherwise cleared and placeholder "valid…" shown; also sets Store `State=true` | "" | Objectionable.json `PersonalAccessToken` |
| **Update**: Update Server (+ hint) | ComboBox | GitHub / Soferity | Soferity | Update.json `ServerType` |
| Update Module | ComboBox | Native / Downloader | Downloader | Update.json `ModuleType` |
| Update Channel | ComboBox | Default (=Release) / Developer (=PreRelease) | Release | Update.json `ChannelType` |
| Update Method | ComboBox | Compressed (.7z) / Executable (.exe) | Executable | Update.json `ExtensionType` |
| ↳ "Download Bandwidth Limit:" | NumberBox 0–99999999 + `StorageType` ComboBox (only Megabyte and above enabled) | | 500 Megabyte | Update.json `LimitValue`, `LimitType` |
| Automatic Update | ToggleSwitch | | true | Update.json `Auto` |
| ↳ "How the Check Will be Performed:" | ComboBox | Visible / Sometimes Visible / Update Invisible / Completely Invisible | SemiSilent | Update.json `AutoType` |
| **Developer**: "Default Developer Tools:" | ToggleSwitch | | false | Engine.json `DeveloperMode` |
| ↳ "Debugging Port:" (+ hint `http://localhost:{port}`, `chrome://inspect`…) | NumberBox | 0–65535 | 0 | Engine.json `DeveloperPort` |

### Donate (`DonateSettingViewModel.cs`)
The page exists and is registered in DI, but is **not reachable**: its nav item is commented out in `MainWindow.xaml` and the `DonateSetting` argument is not handled.

| Label | Control | Range | Default | Key |
|---|---|---|---|---|
| Donate Menu | ComboBox Visible / Invisible | sets `DonateService.DonateVisibility`, but the window ignores it | true | Donate.json `MenuVisible` |
| Advertising Support | ToggleSwitch | | true | Donate.json `AdvertisingActive` |
| ↳ "Ad Delay (Minutes):" | NumberBox | 30–720 | 30 | Donate.json `AdvertisingDelay` |

## 5. Dialogs and controls (`P/Views/Controls/*`, strings in `LOC/Portal/Controls/<Name>/*.EN.xaml`)

**ThemeCreate** (`ThemeCreate.xaml(.cs)`, `P/Extension/ThemeCreate.cs`, `P/Extension/ThumbnailLoader.cs`)
- First a type picker with six tiles. After picking, the type's form shows with a Back button.
- **Fields common to every type:**
  - Thumbnail picker (png, jpg, jpeg, tiff, webp).
  - Preview picker (gif).
  - Title (≤50), Description (≤500), Author (≤50; prefilled with the Windows user name), Contact (≤250; prefilled with a random `<user>@example.com` or `https://<user>.example.com`; must be a URL or an email).
  - Tags (≤104 characters, comma-separated, 1–5 tags of 1–20 characters each).
- **Per type:**
  - **Gif**: drop zone or click to pick `.gif`.
  - **Video**: drop zone or click; accepts mp4, avi, mov, mkv, ogv, flv, wmv, hevc, webm, mpeg, mpeg1, mpeg2, mpeg4.
  - Gif and Video thumbnails come from the Windows shell thumbnail and are JPEG-encoded if the user picks none. **[WIN]** `ShellObject.Thumbnail` (WindowsAPICodePack).
  - **Url**: URL field (`IsUrl`).
  - **YouTube**: URL field (`IsYouTubeAll`).
  - **Web**: HTML picker (accepts htm, html, shtml, xhtml, xml). The **whole parent folder** is copied into the theme, after a "Copy" notice.
  - **Application**: exe picker (accepts EXE, MSI). The whole folder is copied. Extra Arguments field (≤250).
- **Output:** folder `Library\<25 random alphanumeric>` containing:
  - `s_<source>` for Gif and Video;
  - `t_<thumb>`, or the embedded default `Thumbnail.jpg` from `Assets/Theme`;
  - `p_<preview>`, or the embedded default `Preview.gif`;
  - `SucroseInfo.json` with Version 1.0.0.0 and AppVersion = current app version.
- **Behavior:**
  - A validation failure focuses the offending field or turns the drop zone red.
  - Files that are locked or not accessible produce warning dialogs.
  - While creating, a secondary "Creating" button is shown; Enter and Escape are blocked.
  - `InitialFilePath` / `InitialUrl` (from Library drag & drop) preselect the type and prefill the form.

**ThemeEdit**
- Localization ComboBox: "Default" plus the 25 languages. Each item has a status icon: Checkmark (complete), Edit (changed), Prohibited (only one of title/description), Dismiss (empty).
- Title and Description are edited per language; Author, Contact, Tags; Arguments only for Application themes.
- Save validates (same rules as ThemeCreate), increments `Info.Version` and rewrites `SucroseInfo.json`.

**ThemeDelete**
- Shows the thumbnail, title, description, type, author, computed size, "Version (AppVersion)", tags, creation date (directory) and modification date (`SucroseInfo.json`).
- Primary button "Delete" does nothing while the theme is selected and running.

**ThemeReview**: same fields as ThemeDelete, plus a "Theme Contact Address" hyperlink (`mailto:` for emails).

**ThemeImport**: result list of messages per file (green success / red failure), plus "Reason Descriptions" (`CompatibilityType: localized text`). There are about 50 codes; English texts are in `Portal.Enum.EN.xaml`.

**ThemeShare**
- **Export**: SaveFileDialog, then zip the theme. Blocked for premium themes (`SucrosePremium.json`).
- **Publish in Store**:
  - Blocked if the theme came from the store (`SucroseStore.json`) or the premium store.
  - Requires network; fetches the store index to fill the category ComboBox.
  - Steps:
    1. Check the daily limit: GET `/v8/Optional/Upload/Theme/Check/{guid}`.
    2. Compress the theme to `%TEMP%`.
    3. Check size ≤ 90 MB.
    4. Check integrity with `Archive.Check`.
    5. Upload multipart with progress: POST `/v8/Optional/Upload/Theme/{guid}/{category}/{appVersion}`.
  - The dialog cannot be closed during upload; Enter and Escape are blocked.
- **Copy Link**: hidden and disabled.

**ThemeReport** (store cards)
- Reason ComboBox (`ReportThemeType`; default Other), Contact email (required, must be an email), Notes (required, ≤2500).
- "Report": GET `/v8/Optional/Report/Check/{guid}`, then POST `/v8/Optional/Report/{guid}` with JSON.
- "Report from GitHub": opens a new-issue URL on `Taiizor/Store` with the `wallpaper_report.yaml` template and query parameters.

**ThemeLockScreen**: not in `src/` on `develop`. It exists only on branch `lock-screen-native` (commit 112de37f); see `lock-screen.md`.

**WallpaperCycling**
- "Automatic Wallpaper Cycling": toggle, Cycling.json `Active` (default false).
- "Time Interval": NumberBox 10–99999 seconds, `TransitionTime` (default 1800).
- "Transition Mode": Random / Sequential, `TransitionType` (default Random).
- "Cycle List → Reset": sets `Exclusion = []`.
- `PassingTime` has no UI.

**DisplayPreferences** (and `Display/Screen.xaml`, `Display/Duplicate.xaml`)
- A 680×230 canvas preview of monitors, scaled from `rcMonitor`. **[WIN]** `Skylark.Wing.Utility.Screene` (MONITORINFOEX-like `szDevice`, `rcMonitor`).
- "Placement Mode" radio buttons:
  - **Selected Screen Only**: `DisplayScreenType=PerDisplay`. Clicking a monitor sets Engine.json `ScreenDevice = szDevice`; if empty, the first screen is auto-chosen.
  - **Expand Across Screens**: `SpanAcross`, with sub-radio `ExpandScreenType` Default / Advanced.
  - **Duplicate Same Wallpaper**: `SameDuplicate`. The radio is `IsEnabled=False` in the XAML. Sub-radio `DuplicateScreenType` Default / Advanced.
- Defaults: PerDisplay, Default, Default.
- Every change restarts the live engine unless it is closed or paused.
- The "Expand.Monitor" string is not referenced in the code-behind.

**TrayIcon**: "System Tray" hint dialog with an animated GIF. The close button counts down 5→0 and is unusable until the countdown ends (Enter and Escape are blocked). Dispose sets `Warehouse.json:HintTrayIcon=false`.

**OtherAbout**
- Author card with social links.
- "Version: v{ver}" and "Last Checked: {Update.json Time}".
- Release Notes link `https://github.com/Taiizor/Sucrose/releases/tag/v{ver}`.
- Partners (disabled), Contributors, and links to the ToS, Privacy Policy and Code of Conduct (GitHub `.github/*.md`).

**OtherHelp**: five cards, each opened through Commandog:
- Web Page: `https://sucrose.dev`
- Documentation: `…/Taiizor/Sucrose/wiki`
- Source Code: the repository
- Discussions
- Report Issue: `…/issues/new/choose`

**Pagination** (`Pagination.xaml.cs`)
- Prev/Next buttons, 9 toggle buttons with an ellipsis scheme when MaxPage > 6, a NumberBox and a "Jump" button.
- The whole control hides when MaxPage ≤ MinPage. It raises `SelectPageChanged`.

**ExpanderCard** (`ExpanderCard.xaml.cs`)
- Left icon, Title, Description, `HeaderFrame` (right-hand control), `FooterCard` (expanded content).
- `Expandable` (default true) and `IsExpand` (default false). Clicking the card or the chevron toggles it.

**Not requested but present:** `StoreCard`, `LibraryCard`, `AsyncImage`, `CustomGroupBox`, `SizeConstrainingContainer`.

## 6. Localization at runtime

Files: `L/Sucrose.Resources/Helper/Resources.cs`, `L/Sucrose.Resources/Extension/Resources.cs`, `LOC/Locale.EN.xaml`

**Resource layout**
- `Locale.<XX>.xaml` merges the language-neutral `Locale.xaml`, `Portal/Portal.xaml` (release string, copyright, quotes), Discord and Launcher files, plus every `<Area>.<XX>.xaml`.
- `App.xaml` hard-merges `Locale.EN.xaml` at startup.

**Switching language** (`SetLanguage(code)`)
1. Uppercase the code. If `Locale.<XX>.xaml` cannot be loaded, fall back to "EN".
2. Remove every merged dictionary whose Source contains `Locales/`.
3. Add the new `Locale.<XX>.xaml`.
4. Set `CultureInfo` through `Skylark.Helper.Culture.All`.
5. Set FlowDirection: RTL if `CultureInfo.TextInfo.IsRightToLeft` (in practice AR, HE). This is applied to all open windows, and to every later `FrameworkElement.Loaded` through a class handler. Images, `Tag="LeftToRight"` elements and engine render windows are exempt. TextBoxes switch direction per character content (U+0590–U+08FF).

**How strings update**
- XAML strings use `{DynamicResource}` and update immediately.
- Strings set in code (`SRER.GetValue("Portal", …)`, which falls back to `[Key]`) are read once when the control is built:
  - The General page subscribes to `CultureService.CultureCodeChanged` and rebuilds its whole content.
  - Other pages and ViewModels are Transient, so they pick up the new language on the next navigation.
  - Already-built cards and context menus keep their old texts, except LibraryCard and StoreCard menu headers, which are re-read in `ContextMenu_Opened`.
- The language is persisted to General.json `Culture`; the default is the OS UI two-letter ISO code.

---

## Porting notes (risks and oddities)

- **Bug, Update Limit Type.** `UpdateLimitType.SelectedIndex` is initialised from `Backgroundog.DownloadType` instead of `Update.LimitType` (`OtherSettingViewModel.cs`).
- **Typo, ThemeReport.** The initial state key `Reporter.Reporting.Preparing` does not exist (the English key is `Reporter.Preparing`). The text is hidden and overwritten before it is shown.
- **Defaults to keep or revisit.** `StoreAdult` defaults to **true** (NSFW shown). `BackdropType` defaults to None.
- **Dead or hidden UI.** "Preview" menu item; "Copy Link"; Donate page and button; Duplicate placement radio (disabled); `InputModuleType.Native`; unused `CommandType.StartupM`; `Cycling.PassingTime`.
- **No Skylark source in the repo.** `InputType`, `ScreenType`, `StorageType` and `Display/Expand/DuplicateScreenType` come from the Skylark NuGet package. Member order was read from the package XML docs; for Expand and Duplicate, "Advanced" is only in the locale files, so its position is unclear. Since settings store enum **names**, a port must keep identical member names for JSON compatibility.
- **Combo indices equal enum ordinals.** Most combos map SelectedIndex straight to the enum ordinal, so reordering an enum breaks the UI mapping.
- **Static library list.** `LibraryPage.Themes` is a static list that is only ever added to (deletions are removed only from the page-local copy). Stale entries are filtered later by `Directory.Exists`.
- **Web and Application themes copy the entire source folder.** Picking a file in Downloads copies all of Downloads.
- **Blocking I/O on the UI path.** Store fetch, Pattern fetch and cache downloads use `.Result` (blocking HTTP) inside `Task.Run`. Cards insert with a 50 ms delay each. `Dispose()` calls `GC.Collect()` on nearly every navigation, resize and search.
- **TLS validation is disabled.** The shared HttpClient accepts every certificate and enables SSL2/SSL3 (`S/Sucrose.Shared.Dependency/Manage/Internal.cs`). Do not replicate this.
- **Telemetry is on by default.** Search and download telemetry and the report/upload endpoints identify the user by `SSSHU.GetGuid()` (source unclear); ExceptionData and TelemetryData both default to true.
- **Heavy Windows coupling** (each needs a Windows-specific Rust replacement):
  - DWM backdrops
  - Windows theme detection
  - Shell thumbnails
  - Win32 and WinForms file/folder dialogs
  - Desktop icon toggle
  - Desktop wallpaper and slideshow control
  - `powercfg` elevation
  - Startup via Run key / Task Scheduler (through Commandog)
  - Monitor enumeration
  - Clipboard
  - Named mutex
  - Elevated `Runas`
- **Process boundaries.** Many actions are delegated to other processes (Commandog, Launcher signal files, Backgroundog, Reportdog, Property, Update). The Rust port has to keep or replace these process and IPC boundaries.
- **Backgroundog flags are read, never written here.** `ClosePerformance` and `PausePerformance` are runtime state flags the Portal reads. They are presumably written by Backgroundog (unclear).
- **PAT length is hard-coded.** The token is accepted only at exactly 93 characters, which ties it to one GitHub fine-grained token format.
