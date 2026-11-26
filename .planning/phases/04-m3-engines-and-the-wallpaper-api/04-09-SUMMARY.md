---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 09
subsystem: engines
tags: [cef, javascript-api, property-listener, sucrose-compatible, filedropdown, pause, newtonsoft, csharp-format]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-03 read_properties, Control::to_json_value, read_compatible, CompatibleKey; 04-05/04-08 CefEngine state and WebOptions"
provides:
  - "sucrose_engine_web::js: csharp_format/try_csharp_format, indented_json, property_call, property_calls, control_json, filedropdown_files, CompatibleValue, compatible_call, CompatibleState, non_data_calls, compatible_calls, LEGACY_RUN_ORDER, pause_script, pause_call, PageCall"
  - "CEF runs the property calls and the non-data compatible calls on every main-frame load end (status 200), in order, logged as 'page call' (kind, key) at debug; Pause commands run the in-page pause script"
  - "protocol v8: WebOptions { ..., shuffle, theme_type }, filled by the service from Engine.WallpaperShuffle and Portal.ThemeMode"
  - "fixture theme 'calls' (ten control types, media folder, SucroseCompatible.json, a page that reports every call as an origin request)"
affects: [04-10, 04-11, 04-20, round-b-webview2, round-d-sound]
plan_head_before: 64dca7e5595dfef96aa55284ce0d4453239786c3
plan_head_after: ca0526e2c88b3b1b146a51c353e4ef35cfa28c13
actuals:
  tokens: 17550
  tasks: 2
  commits: 2
tech-stack:
  added: []
  patterns:
    - "pure builders in sucrose-engine-web (js), a thin CEF callback that only executes the returned PageCall list"
    - "a call that .NET could not format is an empty script and is never run (the legacy engine catches the FormatException and skips the call)"
key-files:
  created:
    - crates/sucrose-engine-web/src/js.rs
    - crates/sucrose-engine-web/tests/js.rs
    - apps/sucrose/tests/fixtures/web-themes/calls/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/calls/SucroseProperties.json
    - apps/sucrose/tests/fixtures/web-themes/calls/SucroseCompatible.json
    - apps/sucrose/tests/fixtures/web-themes/calls/index.html
    - apps/sucrose/tests/fixtures/web-themes/calls/media/one.txt
    - apps/sucrose/tests/fixtures/web-themes/calls/media/two.png
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-engine-web/Cargo.toml
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
key-decisions:
  - "serde_json gets the preserve_order feature workspace-wide so control objects keep the legacy key order (type, text, help, fields); indexmap was already in the lock file and the whole workspace test suite passes"
  - "The non-data compatible calls run in the order of the legacy code (LoopMode, ThemeType, SystemTheme, ShuffleMode, StretchMode, VolumeLevel), not in the order of the table in engines.md and CompatibleKey::ALL; templates() still decides which keys run"
  - "Portal.ThemeMode System is sent as the legacy name Dark until the OS app theme is read (the legacy default came from a package outside this repository)"
  - "A page that finishes loading while the renderer is paused is paused again after its calls, so a pause that arrives before the load end is not lost"
requirements-completed: []
coverage:
  - id: D1
    description: "Legacy-exact script strings: .NET composite format, Newtonsoft indented JSON (two spaces, CRLF, key order), property calls for all ten control types, compatible values (True/False, enum names, ints, JSON)"
    requirement: WAPI-01
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine-web --test js (28 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The calls fixture page receives one property call per control in file order and the six non-data compatible calls after them on CEF, each reported as an origin request in the renderer log"
    requirement: FMT-07
    verification:
      - kind: integration
        ref: "cargo xtask synth-library --out target/web-lib --count 2 --web-fixtures --select calls --force, then target/debug/sucrose.exe --no-portal: renderer log has 10 __calls/property and 6 __calls/compatible reports"
        status: pass
    human_judgment: false
  - id: D3
    description: "filedropdown listing with the filter rules and canonical containment (outside, absolute, link give an empty list; blank folder or filter gives null)"
    requirement: WAPI-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/js.rs#filedropdown_*"
        status: pass
    human_judgment: false
  - id: D4
    description: "Light pause inside the CEF page: media pause, hidden visibility event, SucrosePause and SucrosePlay, resume restores"
    requirement: PLAY-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/js.rs#the_pause_script_* and a scripted-DOM run of both scripts in bun"
        status: pass
    human_judgment: true
    rationale: "The real pause (a fullscreen app in front of the wallpaper, Phase 2 rule with the Light level) cannot be raised without synthesized input; the maintainer sees it at 04-20"
  - id: D5
    description: "The maintainer's 8 Web themes receive their calls unchanged"
    requirement: WAPI-01
    verification: []
    human_judgment: true
    rationale: "Checked on the maintainer's machine at 04-20; WAPI-01, WAPI-02 and FMT-07 are marked Complete only there"
duration: 22 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 09: Property and compatible calls on CEF Summary

**Legacy-exact SucrosePropertyListener and non-data SucroseCompatible calls (C# string.Format text, Newtonsoft-indented JSON, filedropdown lists with canonical containment) run on every CEF main-frame load, plus a Light pause inside the page, proven with a call-recorder fixture page**

## Performance

- **Duration:** 22 min
- **Started:** 2026-11-26T03:48:00Z
- **Completed:** 2026-11-26T04:10:00Z
- **Tasks:** 2
- **Files modified:** 18 (8 created)

## Accomplishments

- `sucrose_engine_web::js` builds the scripts the way C# does: `{0}`/`{1}` composite format with `{{` and `}}`, alignment, and an empty (skipped) script where .NET would throw; `indented_json` writes two spaces, CRLF, `"key": value`, empty `[]`/`{}` and keeps the key order of the input.
- Ten control types give stored expected strings; filedropdown lists the theme folder with the `|`-separated filter (`*` and `*.*` mean all, sorted ignoring case, files only) and gives `[]` for `..`, an absolute folder, a missing folder or a link out of the theme, `null` for a blank folder or filter.
- On every main-frame load end with status 200 CEF runs the property calls in file order, then the non-data compatible calls, then (if paused) the pause, before the first frame is reported; each is logged as `page call` with kind and key at debug. A reload sends everything again; a theme with no file or an empty list gets nothing; an unparsable file is logged once without its path and the page still plays.
- `set_paused` runs the in-page pause (Light and Heavy alike): playing media pause and are remembered, `document.visibilityState` reads hidden with a `visibilitychange` event, `SucrosePause` runs when defined; resume undoes exactly that and calls `SucrosePlay`.
- Protocol 8: `WebOptions.shuffle` and `theme_type`, filled by the service.

## Fixture run (renderer 0 log, quoted)

Property reports in the fixture's file order, then the compatible reports in the legacy run order (the six data keys of the fixture file are not sent: they belong to plans 10 and 11):

```
page loaded url="https://sucrose.wallpaper.engine/index.html" status=200
origin request rejected path=/__calls/property/title
origin request rejected path=/__calls/property/press
origin request rejected path=/__calls/property/amount
origin request rejected path=/__calls/property/name
origin request rejected path=/__calls/property/enabled
origin request rejected path=/__calls/property/mode
origin request rejected path=/__calls/property/count
origin request rejected path=/__calls/property/secret
origin request rejected path=/__calls/property/picture
origin request rejected path=/__calls/property/tint
origin request rejected path=/__calls/compatible/LoopMode
origin request rejected path=/__calls/compatible/ThemeType
origin request rejected path=/__calls/compatible/SystemTheme
origin request rejected path=/__calls/compatible/ShuffleMode
origin request rejected path=/__calls/compatible/StretchMode
origin request rejected path=/__calls/compatible/VolumeLevel
```

The "rejected" wording is the origin's normal log for a 404; the page used it as a one-way report channel (no host object, no message channel).

## Task Commits

1. **Task 1: property calls with legacy text on CEF** - `832fe9b1` (feat)
2. **Task 2: non-data compatible calls and the in-page pause** - `ca0526e2` (feat)

**Plan metadata:** added by the docs commit below (docs: complete plan)

## Files Created/Modified

- `crates/sucrose-engine-web/src/js.rs` - the script builders, listing, compatible state and pause scripts
- `crates/sucrose-engine-web/tests/js.rs` - 28 tests with stored expected strings
- `crates/sucrose-engine-cef/src/engine.rs` - `page_calls` (properties, compatible, pause-on-load), `State` with shuffle and theme type, pause command on the surface thread
- `crates/sucrose-engine-cef/src/browser.rs` - `CefStart.page_calls`, `execute_script`, calls executed in the load handler
- `crates/sucrose-ipc/src/protocol.rs` - protocol 8, `WebOptions.shuffle` and `theme_type`
- `apps/sucrose/src/service.rs` - fills the new options; `legacy_theme_type`
- `apps/sucrose/tests/fixtures/web-themes/calls/*` - the call-recorder fixture theme
- `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - rows and module note

## Decisions Made

See `key-decisions`. The ThemeType rule: the legacy default (`SSDMMG.ThemeType`) is `SWHWT.GetTheme()`, the OS app theme, from the external Skylark package; the Portal only compares it with `Light`. Nothing in this repository reads the OS app theme yet, so `System` is sent as `Dark` (also the migrated legacy default in `LegacyThemeType`); a stored Light or Dark is exact.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] serde_json preserve_order enabled workspace-wide**
- **Found during:** Task 1 (indented_json "keeps the key order of the input")
- **Issue:** serde_json without `preserve_order` stores objects sorted, so `Control::to_json_value()` and any `json!` value lost the legacy order (04-03 deliberately avoided the feature by using its own ordered reader).
- **Fix:** `serde_json = { version = "1.0.154", features = ["preserve_order"] }` in the root manifest (a feature of an existing dependency; `indexmap` was already in `Cargo.lock`, no new crate). `cargo test --workspace` and `cargo xtask ci` pass.
- **Files modified:** Cargo.toml, Cargo.lock
- **Commit:** 832fe9b1

**2. [Rule 1 - Bug in the reference] Compatible run order**
- **Found during:** Task 2 (comparing `CompatibleKey::ALL` with the legacy `Compatible.cs`)
- **Issue:** the plan text lists the non-data order as LoopMode, ThemeType, SystemTheme, ShuffleMode, StretchMode, VolumeLevel (the legacy code), but `SucroseCompatible::templates()` from 04-03 returns the table order of `engines.md` §6b (… ShuffleMode, StretchMode, SystemAudio, SystemTheme, VolumeLevel …), which is not what the C# code does.
- **Fix:** `js::LEGACY_RUN_ORDER` follows the code; `templates()` still decides which keys have a template. `CompatibleKey::ALL` and the `engines.md` table are left as they are (not in this plan's files); plans 10 and 11 should push data keys in `LEGACY_RUN_ORDER` too.
- **Files modified:** crates/sucrose-engine-web/src/js.rs
- **Commit:** ca0526e2

**3. [Rule 2 - Missing critical] A page that loads while paused**
- **Found during:** Task 2
- **Issue:** the pause command can reach a renderer before its page finished loading, so the in-page script would run in an empty document and the page would then play unpaused.
- **Fix:** `page_calls` appends the pause when the engine state says paused.
- **Commit:** ca0526e2

**4. [Contract detail] Signatures**
- `csharp_format` returns `String` as in the contract (empty where .NET throws, so the call is skipped); `try_csharp_format` returns the `FormatError` for callers and tests that need the reason. `property_calls`, `compatible_calls`, `control_json`, `non_data_calls`, `CompatibleState`, `LEGACY_RUN_ORDER`, `pause_call` and `PageCall` are additions to the contract; `WebOptions` now has a manual `Default` (shuffle on, `Dark`) because the legacy defaults are not the zero values.

### Process notes

- **TDD ordering:** both tasks are marked `tdd="true"`, `workflow.tdd_mode` is off, and tests and code were written together; each task is one `feat` commit with its tests, no RED evidence record. The first test run passed.
- `cargo fmt` rewrote the line endings of the Rust files it touched to LF; they were put back to CRLF before each commit (`git ls-files --eol` shows w/crlf). `Cargo.lock` is written by cargo as LF.
- `crates/sucrose-engine-web/src/args.rs` and `tests/args.rs` were already shown as modified when the plan started (line-ending artifacts, same content as the index); they are not part of this plan and were not staged.

**Total deviations:** 3 auto-fixed (1 blocking, 1 reference discrepancy, 1 missing critical) plus 1 contract detail. **Impact:** none on the plan's goal; the order decision is recorded for plans 10 and 11.

## Issues Encountered

None.

## Known Stubs

None. System-theme resolution (System sent as Dark) is a documented limitation, not a stub; see Deferred.

## Deferred Items

- Read the OS app theme so `ThemeType`/`SystemTheme` match the legacy value for a user with no stored theme and a light OS (needs a platform capability; not in this plan).
- `CompatibleKey::ALL`, `templates()` and the `engines.md` §6b table order differ from the legacy code's run order (see deviation 2); correct them or keep `LEGACY_RUN_ORDER` as the one authority.
- The real Light pause and the maintainer's 8 Web themes are checked at 04-20.

## Next Phase Readiness

Plans 10 and 11 can push the data keys with `compatible_call(template, &CompatibleValue::Json(..))` in `LEGACY_RUN_ORDER` and reuse `CompatibleState`; the page-call path (`PageCall` list executed in the load handler, `CefHost::execute_script` for later pushes) is in place.

## Self-Check: PASSED

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
