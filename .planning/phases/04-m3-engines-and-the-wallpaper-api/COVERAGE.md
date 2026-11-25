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
