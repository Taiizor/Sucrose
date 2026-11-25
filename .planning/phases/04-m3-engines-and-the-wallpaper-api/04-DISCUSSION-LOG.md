# Phase 4: M3 — Engines and the wallpaper API - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-25
**Phase:** 04-m3-engines-and-the-wallpaper-api
**Areas discussed:** Monitor placement, Web engine defaults, Property editor, Performance / pausing, Application wallpapers, Install hints, Argument editor, MVP slice order

---

## Monitor placement

### How does per-monitor wallpaper (DESK-03) fit the mode model?

| Option | Selected |
|--------|----------|
| Extend PerDisplay (recommended) | ✓ |
| Separate 4th mode |  |

**User's choice:** Extend PerDisplay: three modes stay (Per display, Span, Duplicate); Per display now means each monitor holds its own wallpaper

### In Per display mode with several monitors, which monitor does Use target?

| Option | Selected |
|--------|----------|
| Target monitor (recommended) | ✓ |
| Picker on every Use |  |
| Both |  |

**User's choice:** Target monitor chosen in Display Preferences (legacy ScreenDevice); cards show which monitor(s) a theme runs on

### How is the maintainer's migrated PerDisplay + DISPLAY1 interpreted?

| Option | Selected |
|--------|----------|
| Legacy behavior (recommended) | ✓ |
| Spread to all monitors |  |
| Convert to Duplicate |  |

**User's choice:** Legacy behavior: selected theme only on DISPLAY1, other monitors show the Windows wallpaper; legacy defaults kept

### How does Duplicate work on web engines?

| Option | Selected |
|--------|----------|
| One instance mirrored (recommended) |  |
| Instance per monitor |  |
| User chooses |  |
| Other (free text) | ✓ |

**User's choice:** Free text: like video engines, one instance per monitor. Time-based content (Video, YouTube, GIF) syncs across instances; web pages can be seeked from JS. Follow-up: Web and URL sync too where possible (start together after load, align requestAnimationFrame time)

### Two different wallpapers with sound on two monitors?

| Option | Selected |
|--------|----------|
| All play (recommended) | ✓ |
| Only one plays |  |
| Per-monitor volume |  |

**User's choice:** All play at the global volume. Also add per-wallpaper volume through the property editor

### Per-wallpaper volume: scope?

| Option | Selected |
|--------|----------|
| In Phase 4 (recommended) | ✓ |
| Built-in sets only |  |
| Defer |  |

**User's choice:** In Phase 4. Web themes with their own SucroseProperties.json keep using it unchanged; a Web theme without one and URL get a Sucrose mute + volume set; built-in Video/GIF/YouTube sets gain volume; the theme's own file still overrides for mpv/VLC as in legacy

### How is a monitor's wallpaper removed in Per display mode?

| Option | Selected |
|--------|----------|
| Remove on the map (recommended) | ✓ |
| Second click on the card |  |
| Both |  |

**User's choice:** A Remove button per monitor on the Display Preferences map

### A monitor with an assignment is unplugged?

| Option | Selected |
|--------|----------|
| Remember and restore (recommended) | ✓ |
| Forget |  |
| Move to primary |  |

**User's choice:** Remember by a stable monitor id and restore when it returns; shown as not connected on the map, removable

### Span with mixed resolutions/offsets?

| Option | Selected |
|--------|----------|
| Legacy union (recommended) | ✓ |
| Skip gaps |  |

**User's choice:** Legacy union rectangle

---

## Web engine defaults

### Default engine for Web, URL, YouTube on a fresh install?

| Option | Selected |
|--------|----------|
| CEF stays (recommended) | ✓ |
| WebView2 |  |
| Measure then decide |  |

**User's choice:** CEF stays (legacy default, D-024); migrated choices kept

### Default GIF engine on a fresh install?

| Option | Selected |
|--------|----------|
| mpv stays (recommended) | ✓ |
| Built-in |  |

**User's choice:** mpv stays (D-024); migrated Vexana/Xavier map to built-in GIF, Nebula to built-in OS video; built-in GIF vs mpv GIF RAM measured

### How is an engine fallback shown?

| Option | Selected |
|--------|----------|
| Notify, keep setting (recommended) | ✓ |
| Change the setting |  |
| Silent |  |

**User's choice:** Notify, keep the setting: fallback for this run only; Portal toast with install hint, tray tooltip, log, persistent warning on the engine settings row; back to the chosen engine on the next start once installed

### Invalid TLS certificates in web wallpapers?

| Option | Selected |
|--------|----------|
| Validate, opt-in setting (recommended) | ✓ |
| Accept like legacy |  |
| Validate, no exception |  |

**User's choice:** Validate by default; a web engine setting 'Allow invalid certificates' (default off); behavior change recorded in FEATURES.md

### DevTools when DeveloperMode is on?

| Option | Selected |
|--------|----------|
| On demand (recommended) | ✓ |
| Auto like legacy |  |
| Auto on primary only |  |

**User's choice:** On demand: context menu Inspect, shortcuts, remote debugging port when DeveloperPort > 0; Portal button 'Open DevTools' per monitor; no auto-open; behavior change recorded

### Origin for local pages?

| Option | Selected |
|--------|----------|
| Shared virtual HTTPS (recommended) | ✓ |
| Keep legacy per engine |  |
| Loopback HTTP for both |  |

**User's choice:** Shared virtual HTTPS origin https://sucrose.wallpaper.engine/ on both engines (WebView2 virtual host, CEF scheme handler, no port); theme folder first then generated content; path normalized, outside rejected (SEC-02); per-theme storage isolation at Claude's discretion

---

## Property editor

### Where does the property editor live?

| Option | Selected |
|--------|----------|
| Separate small window (recommended) | ✓ |
| Panel inside the Portal |  |
| Separate exe |  |

**User's choice:** Separate small window opened by the Portal process (second GPUI window, no separate exe), placed next to the taskbar like legacy, DPI/RTL aware, stays open when the Portal is minimized, releases resources on close

### Entry point in Phase 4?

| Option | Selected |
|--------|----------|
| Customize icon on the card (recommended) | ✓ |
| Context menu early |  |
| Running wallpapers only |  |

**User's choice:** Customize icon on hover of cards that have properties (legacy Customize enable rule) plus Customize next to each running monitor on the Display Preferences map; LIB-05 context menu later calls the same action

### The 34 C# property copies in Cache\Properties?

| Option | Selected |
|--------|----------|
| Copy once (recommended) | ✓ |
| Share the same files |  |
| Start fresh |  |

**User's choice:** Copy once into a Rust-owned folder when it does not exist; never write the C# files

### Language of theme property texts in Phase 4?

| Option | Selected |
|--------|----------|
| Migrated Culture (recommended) | ✓ |
| OS language |  |
| Leave to Phase 5 |  |

**User's choice:** Migrated Culture (TR) selects the PropertyLocalization texts, legacy fallback to the first culture; Sucrose's own editor strings stay English until Phase 5

---

## Performance / pausing

### What do Light and Heavy pause mean at engine level?

| Option | Selected |
|--------|----------|
| Two real levels (recommended) | ✓ |
| Single level |  |
| Heavy = unload |  |

**User's choice:** Two real levels: Light pauses playback (mpv/VLC pause, page visibility hidden, video/YouTube pause), instant resume; Heavy also releases resources (WebView2 TrySuspend, CEF WasHidden and stopped painting, throttled timers); migrated PausePerformanceType kept

### Remote desktop and VM detection?

| Option | Selected |
|--------|----------|
| API + lists (recommended) | ✓ |
| OS API only |  |
| Lists like legacy |  |

**User's choice:** OS API + lists: WTS session API and events for RDP/console; third-party remote tools and VM/emulator apps matched against the legacy lists through process start/stop events (no 2 s scan), only while the rule is on; lists kept as data with tests

### System data collection?

| Option | Selected |
|--------|----------|
| Decoupled, on demand (recommended) | ✓ |
| Legacy coupling |  |
| Always collect |  |

**User's choice:** Decoupled and on demand: rules run only when PerformanceCounter is on and a rule is not Resume, reading only the signals they need; web data only while a page with SucroseCompatible.json runs and only for the groups it defines; audio spectrum only with SystemAudio; legacy push rates; PerformanceCounter only disables rules (recorded)

### Several conditions hold at once?

| Option | Selected |
|--------|----------|
| Strongest wins (recommended) |  |
| First match like legacy | ✓ |

**User's choice:** Legacy first match: legacy evaluation order kept; while a condition is active, others are not evaluated

### Settings layout for this phase's keys?

| Option | Selected |
|--------|----------|
| Legacy places (recommended) | ✓ |
| Separate Engines page |  |

**User's choice:** Legacy places: Wallpaper and Performance pages with only Phase 4 keys in legacy order; engine rows show installed/missing with hint; new items under their related rows; CommunicationType not shown

### Tray indication when a rule pauses or closes?

| Option | Selected |
|--------|----------|
| Grey icon + tooltip (recommended) | ✓ |
| Tooltip only |  |

**User's choice:** Grey icon + tooltip naming the condition and action; Use ' (Paused)'/' (Closed)' extended to all conditions

---

## Application wallpapers

### How many processes per mode?

| Option | Selected |
|--------|----------|
| By mode (recommended) | ✓ |
| One per screen in every mode |  |

**User's choice:** By mode: Per display one per assigned monitor, Duplicate one per monitor, Span one process over the union; each embedded on its own monitor with real window handles; TYPE-06 wording corrected

### Which window and the Unity 'Play!' hack?

| Option | Selected |
|--------|----------|
| Generic + compatibility (recommended) | ✓ |
| Generic only |  |
| Exact legacy |  |

**User's choice:** Generic + compatibility: first visible main window of the process tree, waited for by window-creation events with a timeout instead of a fixed 3 s; 'Engine' class preferred; 'Play!' click kept for compatibility; no window → process closed and reason in the tray

### Arguments quoting?

| Option | Selected |
|--------|----------|
| Legacy single argument (recommended) | ✓ |
| Parse like a command line |  |

**User's choice:** Legacy single argument kept on purpose; recorded as a kept legacy quirk in FEATURES.md

---

## Install hints

### What does Install do for WebView2 Runtime / VC++ Redist?

| Option | Selected |
|--------|----------|
| Download, verify, run (recommended) | ✓ |
| Open the page only |  |

**User's choice:** Download from the official Microsoft link per architecture, verify the Authenticode signature is Microsoft's before running (D-017/D-019), ask the user, UAC by Windows; re-detect afterwards and switch to the chosen engine

### Snooze ('Remember' one day)?

| Option | Selected |
|--------|----------|
| Snooze on the toast (recommended) | ✓ |
| No snooze |  |

**User's choice:** Snooze button on the fallback toast using the migrated WebViewTime/CefSharpTime fields; the settings-row warning is never snoozed

---

## Argument editor

### One editor or per engine?

| Option | Selected |
|--------|----------|
| Per engine (recommended) | ✓ |
| One shared list |  |

**User's choice:** Per engine: an Arguments editor under each web engine row on the Wallpaper page; key + optional value rows, add/remove, Reset to legacy defaults; Sucrose converts dict/list; dynamically added args shown read-only

### When are changes applied?

| Option | Selected |
|--------|----------|
| Restart with confirmation (recommended) | ✓ |
| Restart immediately |  |
| Next start |  |

**User's choice:** Restart with confirmation: 'Restart now / Later' when running web wallpapers use that engine; Later applies on next start; unknown switches are not blocked, only logged

---

## MVP slice order

### Which vertical slice first?

| Option | Selected |
|--------|----------|
| Web + JS API first (recommended) | ✓ |
| Display modes first |  |
| Planner decides |  |

**User's choice:** Web + JS API first: 1) engine abstraction, detection and choice + CEF + virtual origin + JS API with the 8 Web themes and fixture pages; 2) WebView2; 3) built-in GIF and generated pages, URL, YouTube; 4) display modes and per-monitor wallpapers; 5) property editor; 6) performance and sound rules; 7) libVLC and Media Foundation; 8) Application wallpapers; each slice closes with a maintainer checkpoint

---

## One-way confirmations (2026-11-25)

- Local page origin: one shared origin `https://sucrose.wallpaper.engine/` for all themes (per-theme subdomains declined).
- Rust property copies: `%APPDATA%\Sucrose\Properties`, file names `<Theme><Type>` (`Cache\Sucrose\Properties` declined).

## Claude's Discretion

- Crate layout, IPC messages, CEF/libVLC pins and bindings, built-in GIF and Media Foundation technique.
- Web/URL best-effort sync technique and per-engine sync tolerance.
- Stable monitor id source and settings shape of per-monitor assignments.
- Fallback order per type; editor live-apply cadence, colour picker, filedropdown rules.
- Input forwarding details, "others silent" detection mechanism, generated pages, test themes, measurements.

## Deferred Ideas

- Decode once and share frames across monitors (from Phase 2) stays deferred.
- Not chosen: mirroring one web instance, per-monitor volume, per-theme origins, "strongest action wins", a separate Engines page.
