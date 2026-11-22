# Phase 2: M1 — Video wallpaper on the desktop (Windows) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-22
**Phase:** 02-m1-video-wallpaper-on-the-desktop-windows
**Areas discussed:** Monitors and audio, Fullscreen behavior, Coexistence with the C# app, Tray (Quit), Finding libmpv, Process model in M1, Measurement method

The discussion was held in Turkish; options and answers are translated here.

---

## Monitors and audio

**What "plays on every monitor" means in M1**

| Option | Description | Selected |
|--------|-------------|----------|
| One renderer per monitor | Same video on each monitor, own mpv and surface; matches D-008 and DESK-03; N decodes | ✓ |
| Follow the legacy setting | DisplayScreenType/ScreenDevice; the maintainer's PerDisplay/DISPLAY1 → one monitor only |  |
| One window spanning all | Single decode, video stretched over the union rectangle |  |

**Audio with several renderers**

| Option | Description | Selected |
|--------|-------------|----------|
| Primary monitor only | Primary renderer at WallpaperVolume, others muted |  |
| Silent in M1 | No audio until PLAY-02 |  |
| All play audio | Echo risk |  |

**User's choice (free text):** "If the same wallpaper runs on several screens, video and audio should be synced between them so there is no drift or echo. If different wallpapers run on several screens, each can output its own audio. Audio usually comes from Video or YouTube wallpapers; both can be synced."
**Follow-up confirmation:** "Synced, audio only from the master renderer" (✓) vs "Synced, all renderers play audio" (needs < ~10 ms offset across processes).

---

## Fullscreen behavior

**Where the action comes from**

| Option | Description | Selected |
|--------|-------------|----------|
| Follow the legacy setting | FullScreenPerformance, default Resume (rule off); D-024, PERF-05 | ✓ |
| New default Pause | Exception to D-024, needs a decision entry |  |
| Always Pause in M1 | Setting ignored until PERF-01 |  |

**What stops when one monitor is covered**

| Option | Description | Selected |
|--------|-------------|----------|
| All monitors | Legacy behavior, simplest for sync |  |
| Only the covered monitor | Others keep playing, re-align on resume |  |

**User's choice (free text):** "This should be selectable in the settings, so either the affected screen or all of them can be paused/closed."
**Follow-up confirmation:** both scopes implemented in M1, default "all" (legacy), no switch in M1; persisted in Phase 3, UI in Phase 4, "New" in FEATURES.md (✓). Alternatives offered: a switch via environment variable or flag in M1; more fullscreen questions.

**What counts as fullscreen**

| Option | Description | Selected |
|--------|-------------|----------|
| Foreground window covering a monitor | Legacy rule; includes borderless games, browser F11, video players | ✓ |
| Only what Windows reports | SHQueryUserNotificationState; misses borderless games |  |

---

## Coexistence with the C# app

**C# app running at the same time**

| Option | Description | Selected |
|--------|-------------|----------|
| Detect and do not start | Mutex/process detection, log and tray notice |  |
| Stop the C# engine and take over | Kills Sucrose.Live.*; Backgroundog may restart it |  |
| Ignore | Both attach to WorkerW; layering undefined | ✓ |

**Legacy settings honored beyond Library.json, General.json and the Backgroundog fullscreen keys** (multi-select)

| Option | Description | Selected |
|--------|-------------|----------|
| Playback (Engine.json) | WallpaperVolume, WallpaperLoop, StretchType, HardwareAcceleration | ✓ |
| ScreenType | DisplayBound or WorkingArea | ✓ |
| uMpvPlayer.config | User mpv config replaces the generated one | ✓ |

**Explorer restart recovery (legacy CrashExplorer, default false)**

| Option | Description | Selected |
|--------|-------------|----------|
| Always on | Event-driven (TaskbarCreated); behavior change recorded | ✓ |
| Follow CrashExplorer | Off by default |  |

**AppVersion comparison (themes need AppVersion ≤ app version; Rust is 0.1.0, the selected theme 26.10.19.0)**

| Option | Description | Selected |
|--------|-------------|----------|
| Continue the date-based scheme | YY.MM.DD, semver-valid; new decision entry | ✓ |
| Separate compatibility constant | App stays 0.x, two version numbers |  |
| Skip the check in M1 | Defer |  |

**Selected theme missing or not a Video theme**

| Option | Description | Selected |
|--------|-------------|----------|
| Wait in the tray | Reason in log and tray tooltip; Quit to exit | ✓ |
| Log and exit | Closes silently |  |

**Theme selection on machines without the C# app or in tests** (multi-select: `--theme <folder>` flag, `SUCROSE_DATA_DIR` fixture)
**User's choice (free text):** "Is that needed? Testing happens on my computer and the old Sucrose is installed." → No test-only theme selection.

**Follow-up (raised because CLAUDE.md asks for `SUCROSE_DATA_DIR` during runs):** legacy settings always read read-only from the real `%APPDATA%\Sucrose\Setting`; `SUCROSE_DATA_DIR` redirects only Rust's own writes (✓). Alternative: an extra `SUCROSE_LEGACY_DIR` variable.

---

## Tray (Quit)

**Menu in M1**

| Option | Description | Selected |
|--------|-------------|----------|
| Quit only | TRAY-01; stops only Rust processes | ✓ |
| Quit + Pause/Resume | Handy for manual tests |  |
| Quit + Pause/Resume + Reload | Reload restarts renderers |  |

**AppVisible = false**

| Option | Description | Selected |
|--------|-------------|----------|
| Tray always visible in M1 | No Portal yet to quit otherwise |  |
| Honor AppVisible | No tray when false (D-008) | ✓ |

---

## Finding libmpv

**Search order**

| Option | Description | Selected |
|--------|-------------|----------|
| Legacy install → next to exe → PATH | Reuses the C# Requirements DLL |  |
| Next to exe → PATH (legacy install excluded) | Stricter | ✓ |
| Legacy install only | Works only with the C# app installed |  |

**libmpv missing or unloadable**

| Option | Description | Selected |
|--------|-------------|----------|
| Wait in the tray with an install hint | Same as a missing theme; ENG-06 in Phase 4 | ✓ |
| Log and exit |  |  |

**D-005 says engines were never bundled "as today", but the C# app ships libmpv**

| Option | Description | Selected |
|--------|-------------|----------|
| Clarify with a new decision entry | Rust does not bundle; upgraders use the legacy DLL |  |
| Rust bundles libmpv too | Changes D-005 for libmpv | ✓ |

**Follow-up — which engines does bundling cover?** Options: libmpv only; libmpv and libVLC.
**User's choice (free text):** "I think 'optional' was misunderstood. mpv and VLC were not optional in Sucrose either; they were in the bundle, no extra download was needed. Extra downloads applied only to WebView2 and the VC++ Redist." Claude checked the legacy install: libmpv (123 MB), libVLC (115 MB) and CEF (419 MB, `libcef.dll`) are all bundled.

**Follow-up — confirm D-028 (libmpv, libVLC and CEF bundled; WebView2 Runtime and VC++ Redist prerequisites)**
**User's choice (free text):** "In C# the mpv, VLC and CEF DLLs were either included in the project or came from NuGet packages, so no extra download was needed. How will this work in Rust? Let mpv be optionally downloadable, but I don't know how it would work for CEF and VLC." Claude explained the Rust equivalent of NuGet (xtask fetches pinned, SHA256-verified binaries at packaging; libVLC from the same `.nupkg`, CEF through the `cef` crate's CDN download).

**Engine delivery model**

| Option | Description | Selected |
|--------|-------------|----------|
| In the package, fetched by xtask | NuGet equivalent; nothing extra for the user | ✓ |
| libmpv in the package, CEF and libVLC on demand | Smaller package; in-app downloader in Phase 4 |  |
| Everything on demand | Downloader needed already in M1 |  |

**Linking libmpv**

First answer: "Link at build time". Claude then explained the consequences: the service and renderer are the same binary (D-008), so the always-on service would load libmpv; a missing DLL stops the exe from starting; macOS and Linux CI would need libmpv development files.

| Option | Description | Selected |
|--------|-------------|----------|
| Load at runtime | D-013 unchanged; only renderers load libmpv | ✓ |
| Build-time link + delay-load | Missing DLL raises an exception on first call |  |
| Build-time link + separate renderer exe | Changes D-008 and D-013 |  |

**libmpv binary source**

| Option | Description | Selected |
|--------|-------------|----------|
| xtask downloads a pinned version | SHA256-verified, nothing committed | ✓ |
| Reuse the DLLs in src/ | src/ is removed at M7 |  |
| Build ourselves | Large CI effort |  |

---

## Process model in M1

| Option | Description | Selected |
|--------|-------------|----------|
| D-008 + IPC core in M1 | Service spawns one renderer per monitor; sucrose-ipc for sync, pause, close | ✓ |
| D-008 + temporary channel | stdin/stdout, replaced in Phase 3 |  |
| Single process in M1 | Threads per monitor, split in Phase 3 |  |

**Renderer crash**

| Option | Description | Selected |
|--------|-------------|----------|
| Bounded restart with backoff | For example at most 3 in 60 s, then leave the monitor | ✓ |
| No restart | Log only |  |

**Second instance**

| Option | Description | Selected |
|--------|-------------|----------|
| Second instance exits in M1 | Same IPC endpoint (D-009) | ✓ |
| Leave it to Phase 3 |  |  |

---

## Measurement method

| Option | Description | Selected |
|--------|-------------|----------|
| Video: the selected theme's video | Resolution, codec, bitrate recorded | ✓ |
| Video: fixed reference set | 1080p H.264 + 4K HEVC |  |
| Video: both |  |  |

| Option | Description | Selected |
|--------|-------------|----------|
| Single monitor like-for-like + all monitors separately | Rust limited to one monitor for the main row | ✓ |
| Each with its own defaults | Not comparable |  |

| Option | Description | Selected |
|--------|-------------|----------|
| cargo xtask measure | Working set, private bytes, avg/peak CPU, GPU %; 60 s warm-up + 5 min | ✓ |
| Manual | Process Explorer / Task Manager |  |

| Option | Description | Selected |
|--------|-------------|----------|
| Launcher + MpvPlayer + Backgroundog | Rust service includes the tray; criterion-4 subtotal shown too | ✓ |
| MpvPlayer + Backgroundog only | Literal criterion 4 |  |

---

## Claude's Discretion

- How Rust is limited to one monitor for the measurement (keep any switch measurement-only)
- Master selection and promotion, sync mechanism and tolerance
- Fullscreen detection mechanism (event-driven or polling) with the legacy 5/3 debounce
- Heavy pause handled as engine-level pause
- 24H2 attach technique, hot-plug and DPI handling, debounce timing
- Generated mpv config contents, stretch mapping, loop handling
- Accepted libmpv DLL names and the minimum client API version
- Desktop repaint on stop
- Tray icon, tooltip text, English-only strings in M1
- Crate placement

## Deferred Ideas

- Per-monitor different wallpapers with their own audio (DESK-03, Phase 4)
- Persisting and exposing the action-scope setting (Phase 3/4)
- libVLC and CEF bundled through xtask (Phase 4); WebView2 Runtime and VC++ Redist hints (ENG-06)
- Installer packaging of the bundled engines (Phase 5); macOS and Linux libmpv packaging (Phases 6–7)
- Quit closing all Sucrose apps, AppVisible with the Portal, second instance opening the Portal (Phase 3)
- Decode once and share frames across monitors (possible later optimization)
