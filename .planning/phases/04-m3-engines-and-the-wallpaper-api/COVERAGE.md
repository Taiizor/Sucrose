# API Coverage — Phase 4 Round A (CEF, Windows system APIs, the legacy web wallpaper JavaScript API)

> Full coverage by default. Opt-outs are explicit, reasoned decisions. Round A covers D-31 slice 1; Rounds B–E add the rows of their engines and APIs (WebView2, libVLC, Media Foundation, WASAPI, GSMTC, …) and turn the "not needed yet" rows below into decisions of their own.

| capability | decision | reason |
|---|---|---|
| cef: execute_process for subprocesses (04-01) | INTEGRATE | |
| cef: initialize and shutdown with CefSettings (04-02, 04-07, 04-08) | INTEGRATE | settings used: no_sandbox, external_message_pump, browser_subprocess_path, cache paths, log_file, locale, persist_session_cookies, remote_debugging_port, ignore_certificate_errors |
| cef: windowed child browser, WindowInfo::set_as_child (04-02) | INTEGRATE | |
| cef: external message pump (04-02) | INTEGRATE | |
| cef: windowless (off-screen) rendering | OPT-OUT | explicitly out of scope: the windowed child was chosen (no frame copy, no paint callbacks; research Alternatives Considered) |
| cef: multi_threaded_message_loop | OPT-OUT | explicitly out of scope: the external pump on the surface thread is used and the two are not mixed (research Findings 1) |
| cef: scheme handler and resource handler with Range (04-06) | INTEGRATE | |
| cef: ExecuteJavaScript on the main frame, load handler OnLoadEnd (04-09) | INTEGRATE | |
| cef: life span, download, context menu, keyboard handlers | INTEGRATE | |
| cef: ShowDevTools and the remote debugging port (04-08, 04-17) | INTEGRATE | |
| cef: command-line processing for Chromium switches (04-08) | INTEGRATE | |
| cef: SendMouseWheelEvent for forwarded input (04-19) | INTEGRATE | |
| cef: SetAudioMuted and per-browser volume | OPT-OUT | not needed yet: the per-wallpaper volume (D-06) arrives in Round B/C |
| cef: WasHidden / WasResized | OPT-OUT | not applicable: documented for windowless rendering only (research Pitfall 4) |
| cef: DevTools protocol methods (lifecycle, evaluate, scripts) | OPT-OUT | not needed yet (Page.setWebLifecycleState, Runtime.evaluate, Page.addScriptToEvaluateOnNewDocument): Heavy pause is Round D, web Duplicate sync is Round C |
| cef: OnBeforeResourceLoad Referer fill for youtube.com | OPT-OUT | not needed yet: YouTube arrives in Round B (research Findings 12) |
| cef: OnCertificateError override | OPT-OUT | explicitly out of scope: the default reject is the D-12 behaviour; only ignore_certificate_errors is used for the opt-in switch |
| cef: sandbox | OPT-OUT | explicitly out of scope: the legacy no-sandbox default is kept (D-021) and the crate's sandbox feature is off |
| cef: JS bindings, message router, host objects | OPT-OUT | explicitly out of scope: the legacy API is one-way script strings; a page-to-host channel is a research anti-pattern |
| Win32: WinVerifyTrust with the Microsoft root chain policy (04-18) | INTEGRATE | |
| Win32: ShellExecuteEx for the Microsoft installer (UAC by Windows) (04-18) | INTEGRATE | |
| Win32: RegisterRawInputDevices and GetRawInputData (04-19) | INTEGRATE | |
| Win32: registry read of the VC++ Runtimes keys (04-12) | INTEGRATE | |
| Win32: GetLocalTime and GlobalMemoryStatusEx (04-10, 04-11) | INTEGRATE | |
| PDH: Processor % Processor Time, _Total and per core (04-11) | INTEGRATE | |
| PDH: GPU Engine, Network Interface, LogicalDisk, PhysicalDisk | OPT-OUT | not needed yet: Graphic, Network and Storage groups arrive in Round D (WAPI-03) |
| WMI (wmi crate): Win32_BIOS, Win32_BaseBoard, Win32_Processor (04-11) | INTEGRATE | |
| WMI: Win32_Battery, Win32_LogicalDisk, Win32_ProcessStartTrace | OPT-OUT | not needed yet: Battery and Storage in Round D, process events in Round D (D-36 spike) |
| WASAPI loopback, GlobalSystemMediaTransportControls | OPT-OUT | not needed yet: audio spectrum and now playing arrive in Round B (WAPI-05, WAPI-06) |
| WebView2 (webview2-com) | OPT-OUT | not needed yet: slice 2, Round B |
| Legacy JS API: PropertyListener calls and filedropdown lists (04-09) | INTEGRATE | |
| Legacy JS API: the six non-data SucroseCompatible keys (04-09) | INTEGRATE | LoopMode, ThemeType, SystemTheme, ShuffleMode, StretchMode, VolumeLevel |
| Legacy JS API: Round A data keys (04-10, 04-11) | INTEGRATE | SystemDate, SystemProcessor, SystemMemory, SystemBios, SystemMotherboard |
| Legacy JS API: SucroseCompatible SystemAudio | OPT-OUT | not needed yet: Round B (WAPI-05, WAPI-06) |
| Legacy JS API: Round D data keys | OPT-OUT | SystemGraphic, SystemNetwork, SystemStorage, SystemBattery; not needed yet: Round D (WAPI-03) |
| Legacy JS API: generated-page functions | OPT-OUT | GifContent, VideoContent, YouTubeContent pages; not needed yet: generated pages arrive in Round B (WAPI-08); SucrosePause and SucrosePlay are already called when a page defines them (04-09) |

## Round B (D-31 slices 2 and 3: WebView2, Windows audio and media session APIs, Direct2D, the YouTube IFrame API)

> Same rule: full coverage by default, every opt-out reasoned. The Round A rows above stay as written; where a Round B row decides a capability that Round A marked "not needed yet", the Round B row is the decision.

| capability | decision | reason |
|---|---|---|
| webview2: runtime version for detection (04-22) | INTEGRATE | GetAvailableCoreWebView2BrowserVersionString, no WebView created |
| webview2: environment with options (04-22, 04-25) | INTEGRATE | user data folder, language, extensions and crash reporting off, AdditionalBrowserArguments |
| webview2: controller as a child of the surface (04-22) | INTEGRATE | bounds, RasterizationScale, raw-pixel BoundsMode, black background, Close |
| webview2: WebResourceRequested responses for the origin (04-22, 04-23) | INTEGRATE | AddWebResourceRequestedFilter, CreateWebResourceResponse, Range |
| webview2: request header edit for youtube hosts (04-35) | INTEGRATE | Referer filled only when missing |
| webview2: SetVirtualHostNameToFolderMapping | OPT-OUT | explicitly out of scope: one folder per host would split the D-14 single origin; media over it is documented as slow |
| webview2: Navigate and NavigationCompleted (04-22) | INTEGRATE | HttpStatusCode decides the page calls |
| webview2: ExecuteScript for the page calls (04-24) | INTEGRATE | |
| webview2: AddScriptToExecuteOnDocumentCreated | OPT-OUT | not needed yet: the web Duplicate time-base shim is Round C (D-04) |
| webview2: DevTools, context menu and accelerator settings (04-25) | INTEGRATE | AreDevToolsEnabled, AreDefaultContextMenusEnabled, AcceleratorKeyPressed |
| webview2: OpenDevToolsWindow (04-25) | INTEGRATE | only on the Portal's Open DevTools (D-13) |
| webview2: ServerCertificateErrorDetected AlwaysAllow (04-25) | INTEGRATE | only with the opt-in switch (D-12) |
| webview2: DownloadStarting cancel (04-22) | INTEGRATE | legacy |
| webview2: Input.dispatchMouseEvent for the wheel (04-25) | INTEGRATE | CallDevToolsProtocolMethod |
| webview2: TrySuspend and Resume | OPT-OUT | not needed yet: Heavy pause is Round D (D-22); Light pause runs inside the page |
| webview2: IsMuted and audio state | OPT-OUT | not needed yet: Web and URL page volume and the sound rules are Round D; generated pages set their media volume in the page |
| webview2: host objects and web messages | OPT-OUT | explicitly out of scope: the legacy API is one-way script strings; a page-to-host channel is a research anti-pattern |
| webview2: ProcessFailed handling | OPT-OUT | not needed yet: a failed browser process ends the renderer, which the Phase 2 supervisor restarts |
| cef: Referer fill in OnBeforeResourceLoad (04-35) | INTEGRATE | decides the Round A "not needed yet" row; only a missing Referer, only youtube hosts |
| cef: SetAudioMuted / browser volume, Round B decision | OPT-OUT | not needed yet: generated pages set their media volume in the page (04-33); Web and URL page volume is Round D |
| wasapi: loopback capture of the default render device (04-28) | INTEGRATE | IAudioClient loopback, IAudioCaptureClient, IMMNotificationClient device change; decides the Round A row |
| gsmtc: current media session and its change events (04-29) | INTEGRATE | media properties, playback info, timeline, thumbnail stream; decides the Round A row |
| gsmtc: playback control commands | OPT-OUT | explicitly out of scope: the legacy API only reports now-playing data; Sucrose never controls another app's playback |
| direct2d: HwndRenderTarget presenter (04-30) | INTEGRATE | D2D1CreateFactory, CreateBitmap, CopyFromMemory, DrawBitmap |
| gdi: StretchDIBits fallback presenter (04-31) | INTEGRATE | |
| image: GIF, APNG and animated WebP decoding (04-30, 04-32) | INTEGRATE | also GIF and lossless WebP encoding for the generated fixtures |
| youtube iframe api: the legacy player calls (04-35) | INTEGRATE | YT.Player with the legacy playerVars, loadPlaylist, playVideoAt, setLoop, setShuffle, setVolume, play/pause, state and error events |
| youtube iframe api: getCurrentTime and seekTo for clock sync | OPT-OUT | not needed yet: Duplicate sync is Round C (D-04) |
| youtube data api | OPT-OUT | explicitly out of scope: no API key, as in the legacy app (TYPE-05) |
| legacy js api: SucroseCompatible SystemAudio (04-28, 04-29) | INTEGRATE | decides the Round A row |
| legacy js api: GifContent, VideoContent, YouTubeContent pages (04-33, 04-35) | INTEGRATE | decides the Round A row |
| microsoft: WebView2 Evergreen Bootstrapper install (04-27) | INTEGRATE | download over HTTPS, Authenticode-verified, run with /silent /install |

## Round C (D-31 slices 4 and 5: Windows display configuration and core audio sessions, document-start scripts, the YouTube clock calls, GPUI windows, the legacy property system)

> Same rule: full coverage by default, every opt-out reasoned. The rows above stay as written; where a Round C row decides a capability an earlier round marked "not needed yet", the Round C row is the decision.

| capability | decision | reason |
|---|---|---|
| display config: QueryDisplayConfig, DisplayConfigGetDeviceInfo (04-40) | INTEGRATE | GetDisplayConfigBufferSizes and the source and target names: monitorDevicePath, the EDID ids and the friendly name for the stable monitor id (D-08, D-042) |
| windows display config: SetDisplayConfig | OPT-OUT | explicitly out of scope: Sucrose reads the monitor layout and never changes it |
| shell: SHAppBarMessage ABM_GETTASKBARPOS | OPT-OUT | not needed: the editor finds the taskbar edge where the work area is smaller than the monitor rectangle (04-48) |
| webview2: AddScriptToExecuteOnDocumentCreated (04-44) | INTEGRATE | the time-base shim of Web and URL pages in Duplicate (D-04, D-043); decides the Round B row |
| cef: CDP Page.addScriptToEvaluateOnNewDocument (04-44) | INTEGRATE | through the browser host in process; the same shim; decides the Round C part of the Round A row; Page.setWebLifecycleState and Runtime.evaluate stay with Heavy pause in Round D |
| youtube iframe api: getCurrentTime and seekTo for clock sync (04-44) | INTEGRATE | decides the Round B row |
| core audio: session enumeration and per-session volume (04-51) | INTEGRATE | IAudioSessionManager2, IAudioSessionControl2 process id, ISimpleAudioVolume volume and mute: the Sucrose set's volume and the followers' silence of Web and URL pages (D-06, D-04) |
| core audio: IAudioSessionNotification session-created events | OPT-OUT | not needed yet: a bounded 2 s re-apply runs only while a level is set; Round D may use the events for PLAY-04 |
| toolhelp: CreateToolhelp32Snapshot parent ids (04-51) | INTEGRATE | the renderer's process tree for its audio sessions |
| cef: SetAudioMuted and per-browser volume, Round C decision | OPT-OUT | the audio sessions of the renderer tree serve both web engines the same way (04-51); generated pages keep their in-page volume |
| webview2: IsMuted and audio state, Round C decision | OPT-OUT | the same reason as the CEF row |
| gpui: a second window, its bounds, the native path prompt (04-48, 04-53) | INTEGRATE | opened from the running app, bounds read back: the properties editor window (D-18) and filedropdown's Select a file |
| gpui-component: radio, stepper, popover, color_picker (04-46, 04-52) | INTEGRATE | Placement Mode, numberbox, colorpicker |
| platform: screen color picker (Eye Dropper) | OPT-OUT | not needed: no OS capability is implemented, so the button is not drawn (UI-SPEC, D-002: never faked) |
| mpv: property writes from property sets (04-50) | INTEGRATE | the legacy raw property names, except REFUSED_PROPERTIES (file writes, scripts, shaders, IPC) |
| legacy js api: live PropertyListener calls (04-48, 04-49) | INTEGRATE | over IPC instead of the legacy delta watcher files (PROP-02) |
| legacy property files: one-time import of the C# copies (04-48) | INTEGRATE | from Cache\Properties; delta files skipped, the C# folder never written (D-20) |
| legacy vlc property set | OPT-OUT | not needed yet: libVLC arrives in Round E (PROP-05) |
