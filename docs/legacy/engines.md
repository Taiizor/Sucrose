# Legacy reference: engines and the wallpaper API

> Inventory of the **C# app** (not the Rust rewrite), produced on 2026-11-21 by reading the sources in `src/`. Use it as the behavior spec when porting. Items marked "unclear" were not verified; when you find a mistake, fix it here.

All paths below are relative to the repository root. Short prefixes:
`SE/` = `src/Shared/Engine/Sucrose.Shared.Engine/`, `SE.<X>/` = `src/Shared/Engine/Sucrose.Shared.Engine.<X>/`, `Live.<X>/` = `src/Live/Sucrose.Live.<X>/`, `SSp/` = `src/Shared/Sucrose.Shared.Space/`, `SDep/` = `src/Shared/Sucrose.Shared.Dependency/`, `STh/` = `src/Shared/Sucrose.Shared.Theme/`, `SLive/` = `src/Shared/Sucrose.Shared.Live/`.

**Skylark.Wing** is a binary NuGet dependency (`Directory.Packages.props`, `Skylark.Wing 3.2.1.7`). I read its source from the maintainer's separate Skylark repository (`Lib/Skylark/src/Skylark.Wing/`). That checkout's header says **3.2.1.8**, so it may differ slightly from what the app ships with. The 3.2.1.7 DLL in the NuGet cache does contain the same window class strings and P/Invoke names.

---

## 1. Wallpaper type × engine matrix

The engines allowed per type come from `SDep/Enum/EngineType.cs`. Defaults come from `SSp/Manage/Internal.cs` (`GifEngine=MpvPlayer`, `UrlEngine=CefSharp`, `WebEngine=CefSharp`, `VideoEngine=MpvPlayer`, `YouTubeEngine=CefSharp`, `ApplicationEngine=Aurora`). The user's choice is stored in Engine.json under the keys `Gif`/`Url`/`Web`/`Video`/`YouTube`/`Application` (`SDep/Manage/Manager/Engine.cs`).

| Type | Aurora | CefSharp | WebView | MpvPlayer | VlcPlayer | Nebula | Vexana | Xavier |
|---|---|---|---|---|---|---|---|---|
| Gif | – | ✔ generated `GifContent.html` | ✔ generated `GifContent.html` | ✔ (default) libmpv | ✔ libVLC | – | ✔ | ✔ |
| Url | – | ✔ (default) navigate to URL | ✔ | – | – | – | – | – |
| Web | – | ✔ (default) local page over HTTP | ✔ local page via `file:///` | – | – | – | – | – |
| Video | – | ✔ generated `VideoContent.html` | ✔ | ✔ (default) | ✔ | ✔ | – | – |
| YouTube | – | ✔ (default) generated `YouTubeContent.html` | ✔ | – | – | – | – | – |
| Application | ✔ (only) | – | – | – | – | – | – | – |

How each engine renders:
- **CefSharp**: `CefSharp.Wpf.HwndHost.ChromiumWebBrowser` (aliased `CefEngine`, `SE.CefSharp/Manage/Internal.cs`).
  - CefSettings use `WindowlessRenderingEnabled=true` plus the arg `off-screen-rendering-enabled=1`.
  - BrowserSettings: WebGL on, `WindowlessFrameRate=60`, black background.
  - Setup is in `Live.CefSharp/App.xaml.cs`; views are in `SE.CefSharp/View/*.xaml.cs`.
- **WebView**: `Microsoft.Web.WebView2.Wpf.WebView2` (`SE.WebView/Manage/Internal.cs`), `DefaultBackgroundColor=Black`, set up in `Live.WebView/App.xaml.cs`.
- **MpvPlayer**: `Sucrose.Mpv.NET.Player.MpvPlayer` (src/Library) renders into a WinForms `Panel` inside a `WindowsFormsHost` (`SE.MpvPlayer/View/Video.xaml`).
  - The native DLL is `Requirements/libmpv-{x86|x64|ARM64}.dll` (`SE.MpvPlayer/Manage/Internal.cs`).
- **VlcPlayer**: LibVLCSharp.WPF `VideoView` + `MediaPlayer` (`SE.VlcPlayer/View/*.xaml.cs`); the package is `VideoLAN.LibVLC.Windows 3.0.24`.
- **Nebula**: WPF `MediaElement`, `LoadedBehavior=Manual` (`SE.Nebula/Manage/Internal.cs`).
- **Vexana**: no animation library.
  - It decodes the GIF with System.Drawing (`Image.SelectActiveFrame`; frame delay from PropertyItem `0x5100` ×10 ms, default 100 ms).
  - Each frame is written as `Cache/Gif/frame_{i}_{delayMs}.png`; the folder is cleared at every start.
  - Frames are shown in a WPF `Image`, swapped by an async loop driven by a 1 s `DispatcherTimer` (`SE.Vexana/Helper/Parse.cs`, `Event/Gif.cs`).
- **Xavier**: `Sucrose.XamlAnimatedGif.AnimationBehavior` on a WPF `Image`, `CacheFramesInMemory=false` (`SE.Xavier/View/Gif.xaml.cs`).
- **Aurora**: launches an external exe and re-parents its window into the desktop (§8).

How the two browser engines handle each type (`SE/Helper/Source.cs`):
- **Gif**: writes `GifContent.html` to `<AppData>\Sucrose\Cache\Content\`.
  - The page draws on a `<canvas>` using an embedded GIF parser bundle (parseGIF/decompressFrames, LZW); the GIF is fetched with XHR.
  - Engine files: `SE.WebView/Event/Gif.cs`, `SE.CefSharp/Event/Gif.cs`.
- **Video**: writes `VideoContent.html` (`<video id="video" autoplay>` with the webkit media controls hidden).
  - After load it injects `style=position:fixed…100%`, `controls=false`, `loop=true` (`SE.*/Helper/Video.cs`).
- **YouTube**: writes `YouTubeContent.html` (§8).
- **Url**: navigates directly to `Info.Source`.
- **Web**: navigates to `Host + Info.Source`, or to the URL itself if `Info.Source` is a URL.

`Host` differs per engine:
- WebView: `file:///<Library>/<Selected>/`
- CefSharp: `http://127.0.0.1:<port>/` (the Watson server, §9)
- All other engines: `<Library>/<Selected>/`

`GetSource(src, host)` returns URLs unchanged. The one-argument `GetSource(string)` downloads a URL to `Cache/Content/<SHA256(url)><ext>`, but engines only ever call it with local paths, so the download branch looks unused.

---

## 2. Desktop attachment

Every engine window is created off-screen and borderless: `WindowStyle=None`, `ResizeMode=NoResize`, `Left=-99999`, `AllowsTransparency=False` (`SE.*/View/*.xaml`).

**Window setup** (`SE/Event/Handler.cs`):
- `WindowLoaded` [WIN]: calls `RemoveWindowFromTaskbar`, then toggles `ShowInTaskbar` true→false. The code comment says this is a workaround for a Win10/11 Task View crash.
- `ContentRendered` calls `Skylark.Wing.Engine.WallpaperWindow(window, …)`, choosing the overload by `DisplayScreenType` (§3).
- Separate overloads exist for raw handles (`WallpaperHandle`) and processes (`WallpaperProcess`, uses `MainWindowHandle`).

**Attach sequence** (`Skylark.Wing/Helper/DesktopCore.cs` `FixHandle`) [WIN]:
1. `FindWindow("Progman","Program Manager")`. If not found: `Process.Start("explorer.exe")`, sleep 1 s, retry; return false if still missing.
2. Up to 8 attempts. On even attempts send `SendMessageTimeout(Progman, 0x052C, wParam=0xD, lParam=0x1, SMTO_NORMAL, 10000)` (Windows 7 uses `0,0`).
3. Run `EnumWindows`: for a top-level window that has a `SHELLDLL_DefView` child, take `WorkerW = FindWindowEx(NULL, thatTop, "WorkerW", NULL)` (the next WorkerW sibling). Sleep 250 ms if nothing was found.
4. Fallback for the "some Windows 11 builds" layout (WorkerW is a child of Progman, i.e. the 24H2 shell): `FindWindowEx(Progman, 0, "WorkerW")`.
5. `SetParent(handle, WorkerW)`. On Windows 7 it instead hides WorkerW (`ShowWindow 0`) and calls `SetParent(handle, Progman)`.
   - The "WorkerW not found" check is commented out, so the code can call `SetParent(h, NULL)`.

**Sizing** (`Skylark.Wing/Utility/Screene.cs`):
- `FillScreenWindow`/`FillScreenHandle` choose `rcWork` (ScreenType `WorkingArea`) or `rcMonitor` (`DisplayBound`).
- They `MoveWindow` to that rectangle, offset from the combined virtual-desktop origin (`CombinedRectangles.Left/Top`).
- DPI fix (WPF windows only): `GetDpiForWindow` [WIN]; if it is not 96, set `Content.LayoutTransform = ScaleTransform(96/DPI)`. The code comment explains that a window parented into WorkerW stops receiving `WM_DPICHANGED`.

**24H2-specific code in Sucrose itself:**
- CefSharp removes the `disable-gpu-compositing` switch when `IsGermanium()` (build ≥ 26000) (`Live.CefSharp/App.xaml.cs`, `SSp/Helper/Windows.cs`).
- On those builds, turning hardware acceleration off does not add `disable-gpu`/`disable-gpu-vsync`.
- I found no other 24H2 branches.

**Child HWND fit-up:** `SE.{WebView,CefSharp,MpvPlayer,VlcPlayer}/Helper/Surface.cs` `Correct()` runs on SizeChanged. If the inner browser or player HWND is smaller than the wallpaper's client rect, it calls `MoveWindow(child, 0,0, client)` [WIN `GetWindowRect`/`GetClientRect`/`MoveWindow`].

**Aurora window preparation** (`Handler.ApplicationLoaded`/`ApplicationRendered`/`EngineLoaded`/`EngineRendered`) [WIN]:
- `RemoveWindowFromTaskbar`, `ShowWindow(SW_HIDE)`.
- `SetWindowLong(GWL_STYLE, style & ~(WS_CAPTION|WS_THICKFRAME|WS_MINIMIZE|WS_MAXIMIZE|WS_SYSMENU|WS_DLGFRAME|WS_BORDER|WS_EX_CLIENTEDGE))`, `BorderlessWinStyle`, then attach, then `SW_SHOW`.
  - `WS_EX_CLIENTEDGE` is an extended-style flag mixed into the `GWL_STYLE` mask.

**Display changes:** `SystemEvents.DisplaySettingsChanged` → hide the window, wait 2000 ms (cancellable debounce), re-enumerate screens with `Screene.Initialize()`, wait 500 ms, re-attach, show (`Handler.DisplaySettingsChanged`).

**Explorer restart recovery** (`SE/Helper/Crashing.cs`, 5 s timer, only if Engine.json `CrashExplorer`):
- Progman missing → start `explorer.exe`, wait 3 s, send Commandog `RestartLive`.
- Progman HWND changed → `RestartLive`.

**Stop:** `RefreshDesktop` repaints the desktop so the last frame does not stay on screen (`SLive/Helper/Kill.cs`, `SLive/Helper/Run.cs`).

---

## 3. Display modes, stretch, transitions, multi-monitor, DPI

Enums and Engine.json keys (`Skylark/Enum/ScreenType.cs`, `src/Library/Sucrose.Manager/Manage/Engine.cs`):

| Key | Values | Default |
|---|---|---|
| `DisplayScreenType` | `PerDisplay`, `SpanAcross`, `SameDuplicate` | `PerDisplay` |
| `ExpandScreenType` | `Default` (an `Advanced` value is commented out) | `Default` |
| `DuplicateScreenType` | `Default` | `Default` |
| `ScreenType` | `WorkingArea`, `DisplayBound` | `DisplayBound` |
| `ScreenDevice` | monitor device name (string) | `""` |

Modes (`SE/Event/Handler.cs`, `Skylark.Wing/Skylark.Wing.cs`, `Helper/ScreenManage.cs`):
- **PerDisplay**: one window on the monitor matched by `ScreenDevice`.
- **SpanAcross**: `OwnerScreen(ExpandScreenType)` returns the union rectangle of all `Screen.AllScreens`; one window covers everything.
- **SameDuplicate**: `WallpaperWindow(window, DuplicateScreenType)`, which is marked `[Obsolete("This method is currently unavailable.")]` but is still called.
  - It attaches the real window to screen 0. For every other screen it creates a new WPF `Window` whose `Grid.Background = VisualBrush(original)` and attaches it.
  - The `WallpaperHandle`/`WallpaperProcess` overloads for this mode just `return false`.
  - Likely blank for HWND-hosted content (WebView2, CefSharp HwndHost, mpv WinForms panel, VLC VideoView), because a VisualBrush cannot capture child HWNDs. This is my inference; I did not test it.
- **Aurora** always starts one process per screen (`SE.Aurora/View/Application.xaml.cs`).
  - SameDuplicate attaches process *i* to screen *i*.
  - PerDisplay and SpanAcross attach every instance to the same target (looks like a bug; §Porting notes).
- **YouTube page size**: `{Width}`/`{Height}` are replaced with the monitor or work-area size of the target monitor (span union, screen 0, or `ScreenDevice`) (`SE/Helper/Source.cs` `GetYouTubeContent`).

**StretchType** (`SDep/Enum/StretchType.cs`): `None`, `Fill`, `Uniform`, `UniformToFill`.
- Engine.json `StretchType`, default `UniformToFill`. Out-of-range values fall back to `SSDMI.DefaultStretchType` (`SE/Helper/Data.cs`).
- Applied every 1 s tick:

| Engine | How stretch is applied |
|---|---|
| Web pages (Video/Gif) | `object-fit` = `none` / `fill` / `contain` / `cover`. Gif uses `SucroseStretchMode('<Name>')`. |
| mpv | `panscan` = `1.0` only for UniformToFill; `keepaspect` = `no` only for Fill; `video-unscaled` = `yes` only for None (`SE.MpvPlayer/Helper/Video.cs`) |
| VLC | Fill → `AspectRatio = "<viewW>:<viewH>"`; Uniform → defaults; UniformToFill → `CropGeometry = "<viewW>:<viewH>"`; None → `Scale = 1` (`SE.VlcPlayer/Helper/Video.cs`, `Aspect.cs`) |
| Nebula / Xavier / Vexana | WPF `Stretch` (enum cast) |
| YouTube / Url / Web | not applied; Web only receives it through the compatibility `StretchMode` (§6) |

**Transitions:** `SDep/Enum/TransitionType.cs` only has `TransitionCycleType { Random, Sequential }`. This is wallpaper *cycling*; there are no visual transition effects.
- Engine side: a 10 s timer adds 10 to Cycling.json `PassingTime` and calls `Cycyling.Check()/Change()` (`SE/Helper/Cycyling.cs`).
- `SSp/Helper/Cycyling.cs` picks the next valid theme (respects `Exclusion`, `AppVersion` ≤ current), resets `PassingTime`, sets Library `Selected`, and sends Commandog `Cycyling`. Cycling is skipped while Backgroundog is pausing or closing.

**DPI:**
- All 8 engines' `App.manifest` declare `dpiAware true/PM` and `dpiAwareness PerMonitorV2, PerMonitor` (`Live.*/App.manifest`).
- Plus the `FillScreenWindow` LayoutTransform fix above.
- The WebView wheel event converts to CSS pixels by dividing by `DpiScaleX × ZoomFactor` (`SE.WebView/Extension/Interaction.cs`).

---

## 4. Playback controls

**Common loop:** each view adds a handler to the shared static `SSEMI.GeneralTimer` (WPF DispatcherTimer, 1 s). Each tick (`SE.*/View/*.xaml.cs`):
1. `Dispose()` → `GC.Collect()` (every second, in every engine).
2. `Run.Control()`: restarts Backgroundog via Commandog if `PerformanceCounter` is on and it is not running.
3. Applies loop, volume and stretch.
4. If Backgroundog.json `PausePerformance` is set → Pause; when it clears → Play.
   - The web Gif/Video/YouTube views only do this after `Initialized`.

**Pause/Resume and volume per engine:**

| Engine / type | Pause → Resume | Volume |
|---|---|---|
| WebView Web/Url | [WIN] `DebugActiveProcess(pid)` / `DebugActiveProcessStop` on the GPU process (PID of child window class `"Intermediate D3D Window"`, `SE.WebView/Helper/Handle.cs`, `Web.cs`, `Url.cs`) | per-process audio session volume on every `msedgewebview2` process whose command line contains `Sucrose`; discovery retried 3 times |
| CefSharp Web/Url | same debugger freeze, applied to **all** `CefSharp.BrowserSubprocess` PIDs (`SE.CefSharp/Helper/Web.cs`) | same, on `CefSharp.BrowserSubprocess` processes |
| Web Video page | `video.pause()` / `play()` | `video.volume = v/100` |
| YouTube page | `pauseVideo()` / `playVideo()` | `setVolume(n)` |
| Web Gif page | `SucrosePause()` / `SucrosePlay()` | none |
| mpv | `Pause()` / `Resume()` | `Volume` (Video only) |
| VLC | `SetPause(true)` / `Play()` | `Volume` |
| Nebula | `Pause()` / `Play()` | `Volume = v/100` |
| Xavier | animator `Pause()` / `Resume()` | none |
| Vexana | `ImageState` flag | none |
| Aurora | no in-engine pause | first instance gets the volume, others 0 (`SE.Aurora/Helper/Application.cs`) |

Audio-session volume uses Skylark.Wing `AudioController`/`VideoPlayerController.AudioManager` [WIN Core Audio sessions; exact interfaces unclear].

**Heavy vs Light pause** (Backgroundog `Helper/Performance.cs` `Lifecycle()`):
- `Close` → `Kill.Stop()`.
- Otherwise sets `PausePerformance`. If `PausePerformanceType == Heavy`, it also calls [WIN] `NtSuspendProcess` on the engine process, all `msedgewebview2`/`CefSharp.BrowserSubprocess` processes with `Sucrose` in their command line, and the Aurora app processes (`SSp/Extension/Lifecycle.cs`).
- `Light` = flag only; the engines pause themselves.
- Trigger categories (`SDep/Enum/PerformanceType.cs` `CategoryPerformanceType`): Lock, Focus, Sleep, Memory, Remote, Battery, Console, Graphic, Network, Session, Virtual, Processor, FullScreen, ScreenSaver, BatterySaver. The rules live in Backgroundog (outside this scope).

**Volume logic** (`SE/Helper/Data.cs` `GetVolume`):
- `WallpaperVolume` 0..100, default 100.
- `VolumeSilent` (default false): returns 0 while `PauseVolume` is true.
- `VolumeDesktop` (default false): returns 0 unless the foreground window is the desktop.
  - `IsDesktopBasic` = foreground class is `WorkerW` or `SHELLDLL_DefView`.
  - `IsDesktopAdvanced` = foreground is the WorkerW that hosts DefView, or Progman.
- `SE/Helper/Volume.cs` sets `PauseVolume`: a timer every `VolumeSilentSensitivity` seconds (1..10, default 5) scans all processes except Sucrose-named ones, own browser children and Aurora apps.
  - If any has an active, unmuted audio session with volume > 0 → `PauseVolume=true` [WIN].

**Loop** (`WallpaperLoop`, default true):
- web Video: `video.loop`, plus `play()` if ended.
- YouTube: `setLoop(b)`.
- Gif page: `SucroseLoopMode(b)`.
- mpv: `Loop`; reload if `Remaining ≤ 0`.
- VLC: rebuilds the media with `:input-repeat=65535` (detailed comment in `SE.VlcPlayer/Helper/Video.cs`: avoids a blank HWND on a cold restart), plus an `EndReached` fallback.
- Nebula: seek to 0 on `MediaEnded`.
- Xavier: `RepeatBehavior.Forever` vs `1` (reloads the source).
- Vexana: `ImageLoop`.

**Shuffle** (`WallpaperShuffle`, default true): YouTube `setShuffle(b)` and the compatibility `ShuffleMode` only.

**Speed and mute:** only through properties (`playbackRate`/`muted` on the web pages, `speed`/`mute` in mpv/VLC; §7).

**Stay awake** (`StayAwake`, default false): `SE/Helper/Awakening.cs`, 30 s timer.
- On: [WIN] `SetThreadExecutionState(ES_CONTINUOUS|ES_SYSTEM_REQUIRED|ES_DISPLAY_REQUIRED)`; otherwise `ES_CONTINUOUS`.
- mpv config `stop-screensaver=always|no`; VLC option `disable-screensaver|no-disable-screensaver`.

---

## 5. Input forwarding

Files: `SE.WebView/Extension/Interaction.cs`, `SE.CefSharp/Extension/Interaction.cs`, `SE.Aurora/Extension/Interaction.cs`.

Settings:
- `InputType` (Skylark): `Close`, `OnlyMouse`, `OnlyKeyboard`, `MouseKeyboard` (default).
- `InputModuleType` (`SDep/Enum/InputType.cs`): only `RawInput` (a `Native` value is commented out).
- `InputDesktop` (default false): when on, forwarding only happens while the desktop is foreground (re-checked every 1 s).

**Which engines forward input:** WebView and CefSharp for **Web and Url only** (registered on DOMContentLoaded / FrameLoadEnd / InitializedChanged), and Aurora. Gif, Video, YouTube and the media engines get no input.

**Capture** [WIN]: Linearstar.Windows.RawInput `RegisterDevice(Mouse|Keyboard, RIDEV_INPUTSINK)` on the wallpaper WPF HWND (Aurora also uses `NoLegacy`), plus `HwndSource.AddHook` handling `WM_INPUT`.

**Mouse:**
- Position: `GetCursorPos` → `Calculate.MousePosition` (SpanAcross: minus the virtual-screen origin; otherwise: minus the bounds of the monitor under the cursor).
- Sent with `PostMessageW(target, msg, wParam, lParam=(Y<<16)|X)`:
  - `WM_MOUSEMOVE` with wParam `0x20`
  - `WM_LBUTTONDOWN/UP` with `0x1`
  - `WM_RBUTTONDOWN/UP` with `0x2`
  - `WM_MBUTTONDOWN/UP` with `0x10`

**Wheel:**
- WebView: `CallDevToolsProtocolMethodAsync("Input.dispatchMouseEvent", {"type":"mouseWheel","x":X,"y":Y,"deltaX":0,"deltaY":-ButtonData,"modifiers":0,"pointerType":"mouse"})`, coordinates divided by DPI×zoom, invariant culture.
- CefSharp: `IBrowserHost.SendMouseWheelEvent(MouseEvent(x,y), 0, ButtonData)`.
- Aurora: `PostMessageW(App.NativeHandle, WM_MOUSEWHEEL, 0, ButtonData)`.

**Keyboard:**
- Dead keys are skipped (`MapVirtualKeyW(vk, MAPVK_VK_TO_CHAR)` high bit).
- In `OnlyKeyboard` mode the first key first sends a fake click at (99999,99999) to give the page focus.
- Then `PostMessageW(raw.WindowMessage, wParam=VK, lParam = 1 | scan<<16 | 1<<24 | (keyUp ? 3<<30 : 0))`. The code comment notes Alt combos do not work.

**Target windows:**
- WebView: WebView2 HWND → child `Chrome_WidgetWin_0` → its child `Chrome_WidgetWin_1`.
- CefSharp: `GetBrowserHost().GetWindowHandle()` → child `Chrome_WidgetWin_0`, else `_1` (`Helper/Handle.cs`).
- Aurora posts to four targets: the first child of the process's visible top-level window, `NativeHandle`, `MainWindowHandle`, and the global class `"Engine"` window.
  - It only sends to the instance whose index+1 equals the cursor's monitor index, or to all instances in SpanAcross.
  - Its hook always sets `Handled=true`.

**Browser key blocking** (CefSharp only, `SE.CefSharp/Handler/CustomKeyboard.cs`): Ctrl+F/F3, Ctrl+P/PrintScreen, Ctrl+R/F5, Ctrl+±, Ctrl+Shift+C/F12, BrowserBack/Forward/Search. WebView instead disables accelerator keys and context menus unless `DeveloperMode` is on.

---

## 6. Web wallpaper JavaScript API

The engine never exposes host objects or message channels. A grep for `AddHostObjectToScript`, `JavascriptObjectRepository`, `WebMessageReceived`, `AddScriptToExecuteOnDocumentCreated` and `chrome.webview` found nothing. All communication is one-way: the engine builds script strings and runs them with `ExecuteScriptAsync` (WebView) or `ExecuteScriptAsync` (CefSharp).

### 6a. Properties (any author-defined function)

`SE/Helper/Properties.cs` `ExecuteNormal`:
- Script = `string.Format(Properties.PropertyListener, key, JsonConvert.SerializeObject(control, Formatting.Indented))`.
- Every bundled showcase theme uses the listener `"SucrosePropertyListener('{0}', {1});"`, so the call is `SucrosePropertyListener('<key>', {"type":"slider","value":…,"min":…,…})`.
- For `filedropdown` controls, the engine fills `files` first by listing `<wallpaper>/<folder>` with the `filter` (`|`-separated `*.ext`). `*` or `*.*` means all files; paths outside the wallpaper folder return `[]`.

When it is sent:
- All keys on WebView `DOMContentLoaded` or CefSharp `FrameLoadEnd`.
- Afterwards, on change: a FileSystemWatcher (Created) on `Cache/Properties/*.<Selected><PropertiesType>`.
  - The Property app writes `<Guid>.<Selected><Type>` containing only the changed key (`src/Project/Sucrose.Property/Helper/Properties.cs`).
  - The engine reads it after 50 ms, executes it, and deletes it after 3 s (`SE/Helper/Properties.cs`).
- Applies to Web (and the generated Gif/Video/YouTube pages). **Url gets no properties** in either engine (`SE.WebView/Event/Url.cs`, `SE.CefSharp/Event/Url.cs`).

### 6b. Compatible API (Web type only)

`STh/Helper/Compatible.cs`, `SE/Helper/Compatible.cs`; enabled by `StartCompatible()` in `SE.{WebView,CefSharp}/Helper/Web.cs`.

`SucroseCompatible.json` keys, verbatim. Each value is a JS format string with `{0}`; the engine substitutes:

| Key | `{0}` is |
|---|---|
| `LoopMode` | C# bool → **`True`/`False`** (capitalised) |
| `ThemeType` | `SSDMMG.ThemeType` enum name |
| `SystemBios` | JSON |
| `SystemDate` | JSON |
| `ShuffleMode` | C# bool |
| `StretchMode` | `None`/`Fill`/`Uniform`/`UniformToFill` |
| `SystemAudio` | JSON |
| `SystemTheme` | same `ThemeType` value |
| `VolumeLevel` | int 0..100 (after silent/desktop rules) |
| `SystemMemory`, `SystemBattery`, `SystemGraphic`, `SystemNetwork`, `SystemStorage`, `SystemProcessor`, `SystemMotherboard` | JSON |

Names used by the showcase themes (`src/Bundle/Sucrose.Bundle/Showcase/*/SucroseCompatible.json`): `SucroseAudioData({0});`, `SucroseStretchMode('{0}');`, `SucroseLoopMode('{0}');`, `SucroseShuffleMode('{0}');`, `SucroseMemoryData({0});`, `SucroseGraphicData({0});`, `SucroseNetworkData({0});`, `SucroseProcessorData({0});`.

**Trigger:** each time a Backgroundog message arrives while the engine is not paused (`State`), all non-empty templates run in the table order above. So even `LoopMode`/`StretchMode` only update when Backgroundog sends data.

**Transport** (Backgroundog.json `CommunicationType`: `Pipe`/`Signal`/`Transmission`, `SDep/Enum/CommunicationType.cs`):
- **Pipe**: named pipe `Sucrose.Wallpaper.Engine.Backgroundog.Pipe`; the engine is the server (`src/Library/Sucrose.Pipe/Manage/Internal.cs`).
- **Signal**: file `Backgroundog.sgnl`, read through a watcher (`src/Library/Sucrose.Signal/Manage/Internal.cs`).
- **Transmission**: TCP on loopback; the engine picks a free port and writes `TransmissionPort` to Backgroundog.json.
- The engine also sets `PipeRequired`/`SignalRequired`/`TransmissionRequired` and `AudioRequired = SystemAudio != ""`.
- Handlers: `src/Shared/Sucrose.Shared.{Pipe,Signal,Transmission}/Services/Backgroundog*Service.cs`, compiled only under `LIVE_WEBVIEW || LIVE_CEFSHARP`.

**Cadence:**
- Backgroundog `Initialize` timer: 3000 ms due, **100 ms period** (`src/Project/Sucrose.Backgroundog/Manage/Internal.cs`, `Helper/Initialize.cs`).
- Each tick, `Specification.Start` sends one full snapshot if the previous send has finished, so up to about 10 pushes per second (`Helper/Specification.cs` ~L947–1060).
- How often each sensor refreshes its own value is not fully traced (unclear).

**Payload** (`src/Library/Sucrose.Pipe/Interface/Backgroundog.cs`): `{Bios, Date, Audio, Memory, Battery, Graphic, Network, Storage, Processor, Motherboard}`, each a JObject. Keys from `src/Project/Sucrose.Backgroundog/Extension/Data.cs` (commented-out keys excluded):

| Object | Keys |
|---|---|
| Bios | Name, State, Caption, Version, Description, ReleaseDate, Manufacturer, SerialNumber, CurrentLanguage |
| Date | Day, Hour, Year, Month, State, Minute, Second, Kind, DayOfYear, Millisecond, DayOfWeek |
| **Audio** | State, Title, Artist, Subtitle, AlbumTitle, **Data**, AlbumArtist, SourceAppId, TrackNumber, MediaType, PlaybackRate, PropsValid, RepeatMode, ShuffleEnabled, **DataLeft**, PlaybackMode, AlbumTrackCount, **DataRight**, ThumbnailString, PlaybackState, EndTime, Position, StartTime (ms), LastPlayingFileTime, PositionSetFileTime, MaxSeekTime, MinSeekTime (ms) |
| Memory | Name, State, MemoryLoad, MemoryUsed, VirtualName, MemoryAvailable, VirtualMemoryLoad, VirtualMemoryUsed, VirtualMemoryAvailable |
| Battery | Name, State, Status, Chemistry, SavingMode, BatteryFlag, Description, LifePercent, SaverStatus, FullLifetime, ACPowerStatus, DesignVoltage, LifeRemaining, ChargeStatus, BatteryLifeTime, PowerPlanType, EstimatedRunTime, EnergySaverType, PowerLineStatus, BatteryLifePercent, BatteryFullLifeTime, EstimatedChargeRemaining |
| Graphic | Max, Min, Now, Luid, Name, State, Manufacturer, Instances |
| Network | Host, Name, Ping, State, Total, Online, Upload, Download, PingData, TotalData, UploadData, PingAddress, DownloadData, FormatTotalData, FormatUploadData, FormatDownloadData |
| Storage | State, Drivers, LogicalDrivers, PhysicalDrivers |
| Processor | Max, Min, Now, Core, Name, Type, Cores, State, Thread, CoresMax, CoresMin, CoresNow, ProcessorCount |
| Motherboard | Name, State, Product, Version, Manufacturer |

- `Data`/`DataLeft`/`DataRight` are **128 doubles in [0,1]** on log-spaced bins (FFT 4096, Hann window, per-band envelope) (`src/Project/Sucrose.Backgroundog/Extension/AudioVisualizer.cs`).
- The "now playing" fields look like Windows media session (GSMTC) data [WIN; source not traced, unclear]. Whether `ThumbnailString` is base64 is unclear.

### 6c. Functions built into the generated pages

Exact names, from `SE/Helper/Source.cs`:
- **GifContent.html**: `SucrosePropertyListener`, `SucroseStretchMode`, `SucroseLoopMode`, `SucroseSpeedRate`, `SucrosePlay`, `SucrosePause`, `SucrosePlayPause`. On `DOMContentLoaded` it runs `setTimeout(SucrosePlay,1500)`.
- **VideoContent.html**: `SucrosePropertyListener`.
- **YouTubeContent.html**: `onYouTubeIframeAPIReady`, `onPlayerReady`, `onPlayerStateChange`, `onPlayerError`, `SucrosePropertyListener`, `playFirst`, `playVideo`, `pauseVideo`, `setVolume`, `setShuffle`, `forceReload`, `setLoop`, `checkVideoEnded`, `checkPlayingStatus`, `toggleCaptions`, `toggleAudioTrack`, `hideIframeElement`, `toggleFullScreen`.
- Property keys these pages handle:
  - `scale` → `body scale(1+v/10)`
  - `saturate`, `hue-rotate`, `brightness`, `contrast`, `blur`, `grayscale`, `sepia`, `invert` → CSS filters
  - `mirror` (0..3: Flat/Vertical/Horizontal/VerticalHorizontal)
  - `animation` (0..5: Flat/Flip/Pulse/Shake/Bounce/Wiggle keyframes)
  - `playbackRate`, `muted`; `caption` is handled by YouTube only.
- `GetScreenshot()` returns an embedded html2canvas bundle; I found no caller (unclear).

---

## 7. Property system (`SucroseProperties.json`)

Schema (`STh/Helper/Properties.cs`): `PropertyListener` (string), `PropertyList` (`key → control`), `PropertyLocalization` (`culture → {key → text}`).

Every control has `name`, `text`, `type`, `help` (`STh/Model/ControlModel.cs`); the subtype is chosen by `type` (`STh/Converter/ModelConverter.cs`):

| type | extra fields |
|---|---|
| `label` | value: string |
| `button` | value: string |
| `slider` | min, max, step, value (double) |
| `textbox` | value: string |
| `checkbox` | value: bool |
| `dropdown` | value: int, items: string[], valuetext: string |
| `numberbox` | min, max: double, places: int, value: double |
| `colorpicker` | value: string (showcase uses `#AARRGGBB`, e.g. `#FF00FF00`) |
| `passwordbox` | value: string |
| `filedropdown` | desc, value, title, folder, filter: string; files: string[] (filled at runtime) |

**Storage:**
- Working copy: `<AppData>\Sucrose\Cache\Properties\<Selected><PropertiesType>`, copied from the source file on first run.
- `PropertiesType` ∈ `Base`, `WebView`, `CefSharp`, `MpvPlayer`, `VlcPlayer` (`SDep/Enum/PropertiesType.cs`).
- A file that fails to parse is deleted and the error re-thrown (`Live.*/App.xaml.cs`).

**Which property file each engine uses:**
- Web: the theme's own file, type `Base`.
- WebView/CefSharp Gif/Video/YouTube: always the engine's built-in set, written to `Cache/WebView2/SucroseProperties.json` or `Cache/CefSharp/...`. **A theme's own file is ignored** for these types (`SE.{WebView,CefSharp}/Helper/Properties.cs`).
- mpv and VLC: the theme's file if it exists, otherwise the built-in set (`SE.{MpvPlayer,VlcPlayer}/Helper/Properties.cs`).
- The built-in sets are base64 JSON in `SE/Manage/Internal.cs`:
  - **Cef/Web**: `onlyCef`|`onlyWeb` (label), `scale` [-5..5, step 0.5, default 0], `saturate`/`hue-rotate`/`brightness`/`contrast` [-100..100], `blur`/`grayscale`/`sepia`/`invert` [0..100], `mirror` (dropdown, 4 items), `animation` (dropdown, 6 items), `playbackRate` [0.25..2.5, step 0.1, default 1], `muted`, `caption`.
  - **Mpv**: `onlyMpv`, `video-zoom` [-5..5], `saturation`, `hue`, `sharpen` [0..100], `brightness`, `contrast`, `scale-blur` [0..100], `gamma`, `speed` [0.25..2.5], `mute`, `sid`.
  - **Vlc**: `onlyVlc`, `saturation`, `hue`, `brightness`, `contrast`, `gamma`, `speed`, `mute`.

**How values are applied:**
- **Web**: the JS listener (§6a).
- **mpv** (`SE.MpvPlayer/Helper/Properties.cs`): parses the generated listener script; the key is an **mpv property name**.
  - `slider`/`numberbox` → `SetPropertyDouble` (`scale-blur` also sets `cscale-blur`/`dscale-blur`/`tscale-blur`).
  - `textbox`/`colorpicker`/`passwordbox` → `SetPropertyString`.
  - `checkbox` → `"yes"`/`"no"` (`sid` → `"auto"`/`"no"`).
  - `dropdown` → `SetPropertyLong(value)` if `valuetext` is numeric, else `SetPropertyString(valuetext.lower)`.
  - `label`/`button`/`filedropdown` are ignored.
- **VLC** (`SE.VlcPlayer/Helper/Properties.cs`): fixed keys only.
  - `saturation` → 0..3, `hue` → -180..180, `brightness` → 0..2, `contrast` → 0..2, `gamma` → 0.01..10. Each maps -100..100 with a pivot at 0 → 1 (hue 0) via `SetAdjustFloat`; the code comments that gamma "is not working?".
  - `speed` → `SetRate`, `mute` → `Mute`.
- **Nebula / Xavier / Vexana / Aurora / Url**: no properties.

---

## 8. YouTube, URL and Application details

**YouTube** (`SE/Helper/Source.cs` `GetYouTubeContent`, `SE.WebView/Event/YouTube.cs`, `SE.CefSharp/Event/YouTube.cs`):
- IDs are the `v` and `list` query parameters of `Info.Source` (`STh/Helper/Various.cs`). **No API key**: it loads the public `https://www.youtube.com/iframe_api`.
- playerVars: `autoplay:1, loop:1, controls:0, disablekb:1, modestbranding:1, fs:0, rel:0, iv_load_policy:3, playsinline:0, cc_load_policy:0, version:3, showinfo:0, suggestedQuality:"highres", origin:"https://sucrose.wallpaper.engine/", widget_referrer:"strict-origin-when-cross-origin"`.
- `onReady`: if a playlist → `loadPlaylist({list, index:0})`; a single video gets `setLoop(false)`; quality highres; `setVolume(0)`; `hideIframeElement()`; `toggleFullScreen()`. The engine then sends `setVolume(<vol>)` and `toggleFullScreen()`.
- Playlist end: shuffle → `playVideoAt(random)`, otherwise wrap-around logic. The first state −1 with shuffle → random start. Single video end → `seekTo(0); playVideo()`.
- Errors: in a playlist skip to the next video; single video error `5` → `location.reload`.
- `setLoop(true)` is forced to false for a single video.
- `hideIframeElement` uses a MutationObserver on the iframe's `contentDocument` to hide every YouTube UI element except the video and captions, and injects the animation keyframes. This only works because web security is disabled.
- WebView: `SetVirtualHostNameToFolderMapping("sucrose.wallpaper.engine", Cache/Content, Allow)`; the page loads from `https://sucrose.wallpaper.engine/YouTubeContent.html`. A `WebResourceRequested` filter on `*youtube.com/*` adds `Referer: https://sucrose.wallpaper.engine/` when it is missing.
- CefSharp: loads `http://127.0.0.1:<port>/YouTubeContent.html`; the `origin` playerVar still says `sucrose.wallpaper.engine`.

**Url:** WebView `Source = new Uri(Info.Source)`; CefSharp `Address = Info.Source`. Input is forwarded, volume uses audio sessions, pause uses the debugger freeze; no properties and no compatibility API.

**Application (Aurora)** (`Live.Aurora/App.xaml.cs`, `SE.Aurora/*`):
- `Info.Source` must be a local exe inside the theme folder (a URL makes it close).
- `SucroseInfo.json` `Arguments` is passed through `Processor.Parse`, which **wraps the whole string in quotes** (one argument) (`SSp/Helper/Processor.cs`).
- Started `Screens.Count` times with `UseShellExecute`, `ProcessWindowStyle.Normal`, `CreateNoWindow=true`, then `WaitForInputIdle`, then a 250 ms busy-wait on `Ready.Check`.
- The exe name is saved as Aurora.json `AppProcessName` (used by Kill/Suspend).
- 3 s later (`Event/Application.cs`): finds a top-level class `"Engine"`. If absent, it looks in the process's visible top-level window for `Button "Play!"` and `BM_CLICK`s it [WIN `EnumWindows`, `FindWindowEx`, `SendMessage`]. Probably a launcher's resolution dialog, Unity-style (inferred). A found `"Engine"` window is attached as a wallpaper.
- The 1 s tick closes the engine if the app is no longer running or any instance exited. Closing kills the app.
- GPU preference (`SDep/Helper/Graphic.cs` → `SSp/Extension/GraphicPreference.cs`): [WIN registry] `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` `GpuPreference` set to high performance. Applied to the engine exe and, for WebView, to each discovered `msedgewebview2` exe (`SE.WebView/Helper/Management.cs`). Engine.json `GraphicPreference` default `HighPerformance`.

---

## 9. CefSharp local server and engine settings

**Watson.Lite server** (`SE.CefSharp/Extension/WatsonWebServer.cs`):
- `WebserverLite` on `127.0.0.1:<free port>` (`SSp/Helper/Port.cs`).
- Every request looks for the file in the **theme folder first, then `Cache/Content`** (the generated Gif/Video/YouTube pages).
- Adds CORS `Access-Control-Allow-Origin: *`, a fixed MIME table, `Content-Disposition: inline`, and 404/500 text responses.
- **Why it exists** (inferred; the code has no comment): CefSharp has no WebView2-style virtual-host mapping. Serving over `http://127.0.0.1` gives pages a real origin for XHR/fetch/modules, the iframe API and the generated pages' XHR of media.
  - A commented-out `FolderSchemeHandlerFactory` custom scheme in `Live.CefSharp/App.xaml.cs` was the earlier approach.
- `SE/Extension/LocalHttpServer.cs` is a near-duplicate built on `HttpListener` and is **unused**.
- The path is not normalised, so `..` requests can reach files outside the theme folder (loopback only).

**CefSettings** (`Live.CefSharp/App.xaml.cs`):
- `UserAgent = General.UserAgent`, `CachePath = Cache/CefSharp`, `PersistSessionCookies`, `IgnoreCertificateErrors`, `LocalesDirPath`, `Locale` = the `.pak` matching the culture, `WindowlessRenderingEnabled`, `LogFile = Log/CefSharpNative-<date>-<guid>.log`.
- `Cef.InitializeAsync(performDependencyCheck:true)`; `Cef.Shutdown()` on exit.
- Downloads cancelled (`Handler/CustomDownload.cs`), context menu cleared (`Handler/CustomContextMenu.cs`).

**WebView2 environment** (`Live.WebView/App.xaml.cs`):
- `Language = culture`, `AreBrowserExtensionsEnabled=false`, `IsCustomCrashReportingEnabled=false`, `ScrollBarStyle=FluentOverlay`, user data folder `Cache/WebView2`.
- `AdditionalBrowserArguments = join(WebArguments)`.
- Certificate errors → `AlwaysAllow`; downloads cancelled; UserAgent overridden.

**Engine.json keys** (`src/Library/Sucrose.Memory/Manage/Constant/Engine.cs`, defaults from `src/Library/Sucrose.Manager/Manage/Engine.cs` and `SDep/Manage/Manager/Engine.cs`):

| Group | Keys (defaults) |
|---|---|
| Engine per type | `Gif`, `Url`, `Web`, `Video`, `YouTube`, `Application` (§1) |
| Browser | `CefArguments` (dict; empty → the built-in default is written back), `WebArguments` (list; same) |
| Hardware | `HardwareAcceleration` (true) |
| Developer | `DeveloperMode` (false), `DeveloperPort` (0, range 0..65535) |
| Input | `InputType` (MouseKeyboard), `InputModuleType` (RawInput), `InputDesktop` (false) |
| Display | `StretchType` (UniformToFill), `ScreenType` (DisplayBound), `ScreenDevice` (""), `DisplayScreenType` (PerDisplay), `ExpandScreenType`, `DuplicateScreenType` |
| Volume | `WallpaperVolume` (100), `VolumeSilent`, `VolumeDesktop`, `VolumeSilentSensitivity` (5) |
| Playback | `WallpaperLoop` (true), `WallpaperShuffle` (true) |
| System | `StayAwake`, `CrashExplorer`, `GraphicPreference` |
| Windows wallpaper | `BackgroundImage` (false), `BackgroundImagePath`, `DisableSlideshow`, `DisableSlideshowLive` |
| Not used by engines | `LibraryStart` (true), `StoreStart` (true) |

Effect of the switches:
- `HardwareAcceleration` on: add `--enable-gpu --enable-gpu-vsync` (Cef: `enable-gpu=1`, `enable-gpu-vsync=1`). Off: `--disable-gpu --disable-gpu-vsync` (Cef only when build < 26000).
- `DeveloperPort > 0`: WebView gets `--remote-debugging-port=N --remote-allow-origins=* --enable-features=msEdgeDevToolsWdpRemoteDebugging`; Cef gets `RemoteDebuggingPort = N` and `remote-allow-origins=*`.
- `DeveloperMode`: WebView opens a DevTools window and enables context menus and accelerator keys; Cef calls `ShowDevTools()` and skips the menu/keyboard handlers.

Default Chromium switches (`SE/Manage/Internal.cs` `WebArguments`/`CefArguments`; Cef additionally has `off-screen-rendering-enabled`):
`disable-gpu-compositing`, `disable-direct-write`, `enable-begin-frame-scheduling`, `disable-breakpad`, `disable-extensions`, `disable-third-party-extensions`, `multi-threaded-message-loop`, `no-sandbox`, `disable-back-forward-cache`, `disable-web-security`, `disable-geolocation`, `disable-oor-cors`, `disable-surfaces`, `autoplay-policy=no-user-gesture-required`, `enable-media-stream`, `enable-accelerated-video-decode`, `allow-running-insecure-content`, `use-fake-ui-for-media-stream`, `enable-usermedia-screen-capture(-capturing)`, `debug-plugin-loading`, `allow-outdated-plugins`, `always-authorize-plugins`, `enable-npapi`, `disable-speech-input`, `allow-file-access(-from-files|-from-file-urls)`, `allow-universal-access-from-files`, `unsafely-disable-devtools-self-xss-warnings`, `disable-site-isolation-trials`, `disable-blink-features=BlockCredentialedSubresources`, `enable-features=ThirdPartyStoragePartitioning,PartitionedCookies`, `disable-features=MediaEngagementBypassAutoplayPolicies,CrossSiteDocumentBlockingIfIsolating,BlockInsecurePrivateNetworkRequests,CrossSiteDocumentBlockingAlways,PreloadMediaEngagementData,HardwareMediaKeyHandling,OutOfBlinkCors,IsolateOrigins`.

**mpv config** (`SE/Manage/Internal.cs` `MpvConfig`, `SE.MpvPlayer/Helper/Config.cs`):
- Written to `Cache/MpvPlayer/MpvPlayer.config` unless a user file `uMpvPlayer.config` exists.
- Contents: `vo=gpu`, `gpu-context=d3d11`, `hwdec=no|auto-safe` (from HardwareAcceleration), `profile=low-latency`, `framedrop=decoder`, `video-sync=audio`, `keep-open=no`, `cache=yes`, `demuxer-max-bytes=64M`, `audio-channels=stereo`, `audio-samplerate=48000`, `media-controls=no`, `stop-screensaver=no|always`, `fullscreen=yes`, and others.

**VLC options** (`SE.VlcPlayer/View/*.xaml.cs`): `no-lua`, `no-osd`, `no-spu`, `no-stats`, `no-drop-late-frames`, `no-snapshot-preview`, `video-filter=adjust`, `no-sub-autodetect-file`, `no-metadata-network-access`, `[no-]disable-screensaver`. Player: `Fullscreen`, key and mouse input off, `EnableHardwareDecoding = HardwareAcceleration`.

**Prerequisite checks:**

| Engine | Check | On failure |
|---|---|---|
| WebView | `CoreWebView2Environment.GetAvailableBrowserVersionString()` must be newer than `131.0.2903.70` | dialog Continue / Download / Remember (snooze 1 day, Warehouse.json `WebViewTime`) / Close. Download fetches `https://go.microsoft.com/fwlink/p/?LinkId=2124703` to Temp and runs it (Warehouse `WebViewContinue`) |
| CefSharp | [WIN registry] `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\{x86,x64,ARM64}` `Installed==1` and `Version ≥ v14.40.33816.0` | similar dialog; download URL `https://aka.ms/vc14/vc_redist.{0}.exe` (`src/Library/Sucrose.Memory/Manage/Readonly/Url.cs`) |
| MpvPlayer | Windows build ≥ 14393 | warning dialog, then close |
| Others | none | – |

---

## 10. Lifecycle

**Launch** (`SLive/Helper/Run.cs` `Run.Start`):
1. Skipped if Backgroundog is in close/pause state (unless Backgroundog is not running).
2. Reads `<Library>/<Selected>/SucroseInfo.json`. It needs `AppVersion ≤ current`. Fields (`STh/Helper/Info.cs`): AppVersion, Version, Title, Thumbnail, Preview, Description, Author, License, Contact, Type, Source, Tags, Arguments, Localization.
3. `RefreshDesktop`.
4. If `BackgroundImage`: sets the Windows wallpaper to the theme thumbnail (`SSp/Helper/Background.cs`) [WIN `SystemParametersInfo`/registry style; details not traced].
5. If `DisableSlideshowLive`: disables the Windows slideshow [WIN `IDesktopWallpaper`, Skylark].
6. If `PerformanceCounter`: starts Backgroundog.
7. Runs `Commandog.exe "✔Live✖<engine exe path>"`. The separator characters are `✔` and `✖` (`src/Library/Sucrose.Memory/Manage/Readonly/General.cs`). Commandog just starts the exe (`src/Project/Sucrose.Commandog/Helper/Arguments.cs`).

**Engine startup** (`Live.*/App.xaml.cs`):
1. `OnStartup` → shutdown mode explicit.
2. Mutex `{Sucrose-Wallpaper-Engine-Live}` plus own process count ≤ 1 (`SSp/Helper/Instance.cs`), plus `Run.Check()`: total running engine processes ≤ 1 (`SE/Helper/Run.cs`).
3. `Security.Apply()`: accept every TLS certificate and enable all TLS versions.
4. If cycling is due → `Cycyling.Change()` instead of starting.
5. Otherwise `Checker()` → `Configure()`:
   - read the Library path; `Kill.StopSubprocess()` resumes and then kills leftover `msedgewebview2`/`CefSharp.BrowserSubprocess` processes that have `Sucrose` in their command line;
   - reset the Backgroundog.json flags;
   - start the Awakening (30 s), Crashing (5 s) and Cycling (10 s) timers;
   - load properties and compatibility, create the view window.

**Stop** (`SLive/Helper/Kill.cs` `Stop`): kill all 8 engine exes and the Aurora `AppProcessName`, `Background.ResetDefault()` (restores the Windows wallpaper), `RefreshDesktop`, clear `AppProcessName`.

**Commandog commands** (`src/Project/Sucrose.Commandog/Helper/Arguments.cs`):
- `Kill <name>`
- `Live <path>`
- `RestartLive`: Stop + StopSubprocess + kill Backgroundog + `Run.Start`; after 1.5 s, start again if nothing is running.
- `Cycyling`: same, without killing Backgroundog.
- Switching wallpapers from the Portal presumably goes through these commands (Portal side not read; unclear).

**Crash handling:**
- Every handler `catch` → `Watch_CatchException` plus Commandog `RestartLive` (`SE/Event/Handler.cs`).
- App-level exception hooks start Watchdog with the engine log file (`SSp/Helper/Watchdog.cs`) and exit on fatal errors.
- WebView `ProcessFailed` is only logged (no restart). Explorer restart detection: §2. Aurora exits when its app exits.

**Communication:** there is no direct Portal → engine IPC. Engines read the shared JSON settings files (Engine.json, Backgroundog.json, Library.json, Cycling.json, Aurora.json) on each access/tick. Backgroundog → engine goes through Pipe/Signal/Transmission (Web compatibility data only) plus the `PausePerformance`/`ClosePerformance` flags and process suspension. Property app → engine goes through the watcher files in `Cache/Properties`. Process control goes through Commandog.

---

## Porting notes

- **Desktop attach depends on undocumented shell behaviour** (`0x052C`, WorkerW sibling search, Progman-child WorkerW fallback for 24H2) and lives in the external Skylark.Wing package. The local source (3.2.1.8) may not exactly match the pinned NuGet 3.2.1.7.
- **SameDuplicate** relies on an `[Obsolete]` VisualBrush mirroring path that probably shows nothing for HWND-hosted content. Rust needs a real per-monitor renderer (or texture sharing) to match the intent.
- **Aurora oddities:**
  - It launches one process per screen in every mode, and in PerDisplay/SpanAcross attaches them all to the same target.
  - `Handle.NativeHandle` is `Process.Handle` (a process handle), yet it is passed to `ShowWindow`, `PostMessageW` and `RemoveWindowFromTaskbar` (no-ops or bugs).
  - The `"Play!"` button auto-click is a launcher-specific hack.
  - `Arguments` gets wrapped in one pair of quotes.
- **Pausing web wallpapers attaches a debugger** (`DebugActiveProcess`) to Chromium child processes, and Heavy mode calls `NtSuspendProcess`. Both are fragile; consider page visibility, CDP `Emulation.setCPUThrottlingRate`, or `TrySuspend` (WebView2) in Rust.
- **Compatibility values are formatted with C# `string.Format`**, so booleans arrive as `True`/`False` and JSON is indented. Wallpapers in the wild may depend on this exact text.
- **Compatibility pushes happen only when Backgroundog sends** (≈100 ms cadence when connected). Loop, stretch and theme values therefore never reach a Web wallpaper if Backgroundog is not running.
- **Inconsistent property handling:** the browser engines ignore a theme's own `SucroseProperties.json` for Gif/Video/YouTube; mpv uses theme keys as raw mpv property names; VLC accepts only fixed keys; Url gets no properties at all.
- **Security posture:** web security is disabled, every certificate is accepted (process-wide `ServicePointManager` too), `--no-sandbox` is on, and the Watson server has no path traversal check and serves `CORS *`.
- **Dead or duplicate code:** `SE/Extension/LocalHttpServer.cs` is unused; the html2canvas `GetScreenshot()` payload and `Screenshot.Capture()` are only referenced in commented-out code; `GetVideoStyle()` is commented out.
- **Polling everywhere:** a 1 s DispatcherTimer with `GC.Collect()` each tick, a full process enumeration every 1–10 s for silence detection, and `Process.GetProcessesByName` for every state check.
- **Every engine handler error triggers RestartLive**, which can cause restart loops. WebView `ProcessFailed` does not restart at all.
- **The WebView Web/Url ticks skip the `Initialized` guard** that the Gif/Video/YouTube ticks have.
- **YouTube on CefSharp** declares origin `https://sucrose.wallpaper.engine/` while actually being served from `http://127.0.0.1:<port>`. WebView injects a Referer header for `youtube.com` that CefSharp does not, so YouTube embed errors may differ between the two engines (unclear).
- **The VLC `gamma` adjust is flagged as possibly not working** in the code comments.
- **Vexana re-extracts every frame to PNG on each start** and waits 1 s between loops, so loop timing is not exact.
