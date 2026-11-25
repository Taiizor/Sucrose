# Phase 4: M3 - Engines and the wallpaper API - Pattern Map

**Mapped:** 2026-11-26
**Files analyzed:** 58 new or modified (grouped by D-31 slice)
**Analogs found:** 52 / 58 (6 have only a partial or no in-repo analog, listed at the end)

All analog paths below were checked against `git ls-files` (tracked source only). Line numbers are from the working copy on 2026-11-26. `src/` (legacy C#) is a behavior reference only and is never named as an analog. Paths in the "Closest Analog" column that are modified (not new) are marked (mod).

## File Classification

### Slice 1: Engine abstraction, CEF, virtual origin, JS API, first system data

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-engine/src/lib.rs` (trait, `EngineId`, `Availability`, `choose_engine`, `PauseLevel`) | trait crate (pure) | request-response | `crates/sucrose-engine-mpv/src/lib.rs` + `apps/sucrose/src/supervisor.rs` (pure decision fns) | role-match |
| `crates/sucrose-engine/src/sync.rs` (clock follower moved) | utility | transform | `crates/sucrose-engine-mpv/src/sync.rs` (mod, move) | exact |
| `crates/sucrose-engine-web/src/{origin,mime,js,args,pages}.rs` | utility (pure strings/resolvers) | transform | `crates/sucrose-core/src/theme.rs` + `library.rs` (pure parsing, fixture tests) | role-match |
| `crates/sucrose-engine-cef/{Cargo.toml,build.rs,src/*}` | engine/FFI crate | event-driven (message pump) | `crates/sucrose-engine-mpv/src/{lib,loader,player}.rs` | role-match |
| `crates/sucrose-core/src/{properties,compatible}.rs` | model | transform | `crates/sucrose-core/src/theme.rs` | exact |
| `crates/sucrose-ipc/src/protocol.rs` (mod, v4, `Blob`) | model | request-response | itself | exact |
| `crates/sucrose-ipc/src/portal.rs` (mod, new events/changes) | model | request-response | itself | exact |
| `crates/sucrose-config/src/model.rs` (mod, new keys) | model | CRUD | itself (`Engine`, lines 223-335) | exact |
| `apps/sucrose/src/render.rs` (mod, engine-agnostic host) | service (renderer process) | event-driven | itself | exact |
| `apps/sucrose/src/install.rs` (new, prerequisite check/installer) | service | request-response | `apps/sucrose/src/startup.rs` + `library_ops.rs` | role-match |
| `apps/sucrose/src/data_hub.rs` (new, on-demand data push) | service module | pub-sub, timer | `apps/sucrose/src/modules.rs` (`MemoryModule`) | exact |
| `crates/sucrose-sysinfo/src/*` (new, unsafe-allowed provider crate) | provider | batch / polling | `crates/sucrose-platform/src/os_windows/memory.rs` + `lib.rs` | role-match |
| `apps/sucrose-portal/src/settings/wallpaper.rs` (new page), `settings/mod.rs` (mod) | component | CRUD | `settings/general.rs`, `settings/personal.rs` | exact |
| `apps/sucrose-portal/src/settings/row.rs` (mod, warning alert) | component | request-response | itself | exact |
| `apps/sucrose-portal/src/dialogs/install.rs` (new) | component (dialog) | request-response | `dialogs/move_library.rs` | exact |
| `apps/sucrose-portal/src/toast.rs` (mod, warning kind) | component | event-driven | itself | exact |
| `apps/sucrose-portal/src/theme.rs`, `themes/sucrose.json` (mod, warning pair) | config | transform | itself | exact |
| `xtask/src/fetch_engines.rs`, `xtask/engines.toml` (mod, CEF + libVLC pins, tar.bz2) | utility (xtask) | file-I/O | itself | exact |
| `xtask/src/localize.rs` (mod, `PREFIXES`) | utility (xtask) | transform | itself | exact |
| `apps/sucrose/tests/*.rs` (mod/new) | test | request-response | `apps/sucrose/tests/portal.rs`, `tests/common/mod.rs` | exact |

### Slice 2: WebView2

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-engine-webview2/src/*` | engine/FFI crate | event-driven | `crates/sucrose-engine-mpv/src/player.rs` | role-match |
| `crates/sucrose-sysinfo/src/audio.rs` (WASAPI + `realfft`), `nowplaying.rs` (GSMTC) | provider | streaming | `crates/sucrose-platform/src/os_windows/*.rs` | partial |
| `apps/sucrose/src/install.rs` (WebView2 bootstrapper, Authenticode) | service | file-I/O | `xtask/src/fetch_engines.rs` (`verify_sha256`, `download`) | partial |

### Slice 3: GIF, generated pages, URL, YouTube

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-engine-builtin/src/gif/*` | engine/FFI crate | streaming (decode thread -> surface thread) | `crates/sucrose-engine-mpv/src/player.rs` | role-match |
| `crates/sucrose-engine-web/src/pages/{Gif,Video,YouTube}Content.html` + `tests/fixtures/pages/` | utility / fixture | transform | `crates/sucrose-core/tests/fixtures/library/web/index.html` | partial |
| `apps/sucrose-portal/src/library/card.rs` (mod, lift Use-disabled per type) | component | request-response | itself (`DisabledReason`, `CardUse`) | exact |

### Slice 4: Display modes, per-monitor wallpapers, Display Preferences

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-platform/src/os_windows/display.rs`, `geometry.rs`, `lib.rs` (mod, stable monitor id) | platform | request-response | itself | exact |
| `crates/sucrose-config/src/model.rs`, `migrate.rs` (mod, `DisplayAssignments`) | model/migration | CRUD | itself + `tests/migrate.rs` | exact |
| `apps/sucrose/src/topology.rs` (mod, per-target fleet, Span union) | service | event-driven | itself (`diff`, `SlotMap`, `Debounce`) | exact |
| `apps/sucrose/src/service.rs` (mod, fleet per target, placement restart) | service | event-driven | itself | exact |
| `apps/sucrose-portal/src/dialogs/display_preferences.rs` (new) | component (dialog) | CRUD | `dialogs/move_library.rs` + `dialogs/tray_hint.rs` | role-match |
| `apps/sucrose-portal/src/header.rs` (mod, header action) | component | request-response | itself | exact |
| `apps/sucrose-portal/src/library/card.rs` (mod, monitor badge) | component | request-response | itself | exact |

### Slice 5: Properties editor

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `apps/sucrose/src/properties.rs` (new, working-copy store, single writer, D-20 copy) | service | CRUD / file-I/O | `apps/sucrose/src/settings_host.rs` | exact |
| `apps/sucrose-portal/src/editor/*.rs` (new, second window) | component | CRUD | `settings/general.rs` + `settings/row.rs` controls; window open: `main.rs` | role-match |
| `apps/sucrose-portal/src/ipc.rs` (mod) | provider | pub-sub | itself | exact |

### Slice 6: Performance and sound rules

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `apps/sucrose/src/governor.rs` (new, performance governor) | service module | event-driven + timer | `apps/sucrose/src/fullscreen.rs` | exact |
| `crates/sucrose-platform/src/os_windows/{session,power,events}.rs` (new event sources) | platform | event-driven | `os_windows/event_loop.rs`, `foreground.rs` | role-match |
| `crates/sucrose-sysinfo/src/counters.rs` (PDH, ICMP, battery) | provider | polling | `os_windows/memory.rs` | role-match |
| `apps/sucrose-portal/src/settings/performance.rs` (new page), `banner.rs` (mod) | component | CRUD | `settings/general.rs`, `banner.rs` | exact |
| tray tooltip/icon state (`apps/sucrose/src/service.rs` `Status`, `os_windows/tray.rs`) | service/platform | event-driven | `Status::text/tooltip` in `service.rs` | exact |

### Slice 7: libVLC and Media Foundation

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-engine-vlc/src/{lib,loader,player,sys}.rs` | engine/FFI crate | event-driven | `crates/sucrose-engine-mpv/src/*` | exact |
| `xtask/src/bindgen_vlc.rs` (+ `main.rs` wiring) | utility (xtask) | transform | `xtask/src/bindgen_mpv.rs` | exact |
| `crates/sucrose-engine-builtin/src/mf/*` | engine/FFI crate | streaming | `crates/sucrose-engine-mpv/src/player.rs` | role-match |

### Slice 8: Application wallpapers

| File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-engine-app/src/*` or `apps/sucrose/src/embed.rs` (process spawn + window embed) | engine/FFI | event-driven | `os_windows/surface.rs` + `job.rs` + `apps/sucrose/src/supervisor.rs` | role-match |

## Pattern Assignments

### Engine crates: `sucrose-engine-vlc`, `-cef`, `-webview2`, `-builtin` (engine/FFI, event-driven)

**Analog:** `crates/sucrose-engine-mpv/src/{lib.rs,loader.rs,player.rs,config.rs}`

**Crate header pattern** (`lib.rs` 1-16): doc comment saying "loaded at runtime, never linked (D-013, D-017)", `#![allow(unsafe_code)]` with the D-011 comment, then `pub mod config; pub mod loader; pub mod player; pub mod sync;` and the generated `sys` module:
```rust
// Engine/FFI crate: allowed to opt out of the workspace `unsafe_code` lint (D-011).
#![allow(unsafe_code)]
...
#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case, dead_code, clippy::all)]
pub mod sys;
```

**Loader pattern** (`loader.rs` 16-26, 193-276). `LIBRARY_NAMES` per `cfg`, `candidates`/`locate` (service may call; only renderers load), `resolve` -> `LoadError::NotFound { searched }`, `load_default` warns when loaded from a PATH folder, `load` requires an absolute path, checks the API major version, then `sys::Mpv::from_library`. Every `unsafe` block has `// SAFETY:`:
```rust
pub fn locate(exe_dir: &Path, path_var: Option<&OsStr>) -> Option<PathBuf> {
    candidates(exe_dir, path_var).into_iter().find(|file| file.is_file())
}
...
// SAFETY: the name and the VersionFn signature come from the pinned client.h ...
let reported = match unsafe { library.get::<VersionFn>(b"mpv_client_api_version\0") } { ... }
```
Copy for libVLC: `LIBRARY_NAMES = ["libvlc.dll"]` plus the plugins folder next to it (`VLC_PLUGIN_PATH`), same `folders()` order (next to exe, then absolute PATH entries, D-028; never the C# install, D-018). For CEF/WebView2 only `locate`-style detection (no `load`) is needed in the service, because the service must never load `libcef.dll` (D-032).

**Clock following** (`sync.rs` 9-100): `correct(tick, now_ns, local_pos_s, duration_s, frame_s) -> SyncAction`, `Follower`, constants `SEEK_THRESHOLD_S`, `MIN_SEEK_GAP_NS`. Slice 1 moves this to `sucrose-engine` unchanged (pure, no unsafe); each engine implements only "read local position / seek / set speed". `render.rs` `follow_clock` (604-661) and `publish_clock` (686-749) are the host side to generalise behind the trait.

**Tests** (`crates/sucrose-engine-mpv/tests/load.rs`, `options.rs`): integration tests that skip when the engine is not present; copy that shape.

---

### `crates/sucrose-engine/src/lib.rs` (trait, `choose_engine`, `Availability`)

**Analog:** `apps/sucrose/src/supervisor.rs` (pure decision functions with exit-code constants) and `fullscreen.rs` `plan_action` (pure planner + table tests).

**Pattern:** keep the choice a pure function (`choose_engine(type, chosen, allowed, availability) -> Choice { engine, fell_back_from }`), no I/O, table-driven unit tests inside `#[cfg(test)] mod tests` named in the sentence style used across the repo (`fn a_missing_engine_exits_with_code_2()`, `render.rs` 770). Exit codes for the renderer stay in `supervisor.rs` 19-27 (`CLEAN`, `ENGINE_MISSING = 2`, `SURFACE_LOST = 3`, `START_FAILED = 4`, `LOOP_FAILED = 5`); the ENG-07 fallback should reuse `ENGINE_MISSING` classification (`classify_exit`, 56) and the `Failed { reason, hint }` message.

---

### `crates/sucrose-core/src/{properties,compatible}.rs` (model, transform)

**Analog:** `crates/sucrose-core/src/theme.rs` (legacy-ordinal enums, tolerant `serde` parse, newer-version handling) and `tests/theme.rs`.
**Fixtures:** follow `crates/sucrose-core/tests/fixtures/` (real files copied from the maintainer's library under `library-real/`, hand-made edge cases beside them). Add `SucroseProperties.*.json` and `SucroseCompatible.*.json` the same way (FMT-03, FMT-07). Test file names mirror `tests/theme.rs` / `tests/library.rs`.

---

### `crates/sucrose-ipc/src/protocol.rs` and `portal.rs` (append-only protocol)

**Analog:** the files themselves.

**Append-only rule** (`protocol.rs` 1-12): "Variants are only ever appended at the end of these enums, never inserted, reordered or removed ... `PROTOCOL_VERSION` is bumped only for a change that appending cannot express." Current `PROTOCOL_VERSION: u16 = 3`, `PORTAL_PROTOCOL_VERSION: u16 = 1`.
```rust
pub enum ToRenderer {
    Close, Assign(Assignment), Pause(bool), SetVolume(u8), SetRect([i32; 4]),
    SetRole(Role), Clock(ClockTick), LoadSource { source: String, settings: PlaybackSettings },
    // new variants (SetPauseLevel, SetProperty, Blob chunk, Data) go AFTER LoadSource
}
```
`PlaybackSettings` (118-136) and `Assignment` (139+) are plain structs, not enums: adding fields to them is not append-safe under postcard, so either bump `PROTOCOL_VERSION` to 4 (research Pattern 7) or add a new variant that carries the extra data. Frames are capped at 64 KiB (`frame.rs`, Pitfall 7), so large payloads (property files, compatible data) use the `Blob` chunk message.

**Portal side** (`portal.rs` 144-182): each new setting is an appended `SettingChange` variant with a doc comment stating its range; the service validates in `settings_host::validate` (clamp), writes in `write`, and exposes it in `PortalSettings`/`view_of`.

---

### `crates/sucrose-config/src/model.rs` (+ `migrate.rs`, `legacy.rs`)

**Analog:** `Engine` struct (223-335): one field per legacy key with `#[serde(rename = "LegacyKey")]` and a doc comment naming the key, `extra: Map<String, Value>` flatten for unknown keys, typed enums with legacy ordinals. New per-monitor assignments (`DisplayAssignments`) and the certificate switch are new keys in the legacy-section precedent. Tests: `crates/sucrose-config/tests/{migrate,store,legacy}.rs` with fixtures in `tests/fixtures/*.real.json`; add a fixture for the migration step (D-01, D-08 are costly, so a round-trip and an "old file without the key" test are mandatory).

---

### `apps/sucrose/src/governor.rs` (performance governor, event-driven + timer)

**Analog:** `apps/sucrose/src/fullscreen.rs`

**Constants and debounce** (9-18, 48-92):
```rust
pub const FULLSCREEN_TIMER: u32 = 3;
pub const FULLSCREEN_POLL_MS: u32 = 1000;
pub const TRIGGER_SAMPLES: u8 = 5;
pub const RECOVER_SAMPLES: u8 = 3;
pub struct Debounce { trigger: u8, recover: u8, on: bool, run: u8 }
pub fn feed(&mut self, active: bool) -> Option<bool> { ... }   // Some(new) only when it flipped
```
Generalise to one `Debounce` per condition (D-25: 5/3 on 1 s polls, 0/0 for immediate ones; first match in legacy order wins; while active only its recovery is evaluated). Reuse `ActionScope`, `RuleAction`, `plan_action` (96-140) and `Rule::tick` (162) returning `Vec<(String, RuleAction)>` per monitor; `covered_monitors` (33) stays the FullScreen predicate. Tests are pure (`fn five_positive_samples_in_a_row_switch_the_rule_on()` 302 through `a_rule_acts_on_the_fifth_positive_sample...` 450): keep predicates, debounce and planning free of OS calls so they run on every CI OS.

**Optional-module lifecycle (D-008, D-24):** `apps/sucrose/src/modules.rs` 21-47 `start_module(name, enabled, create)` (never calls `create` when off; logs "module off") and `MemoryModule::follow` (176-194), which returns `Option<TimerCommand>` (`Arm { id, ms }` / `Kill(id)`) so the main loop owns timers. The governor starts only when `PerformanceCounter` is on and a rule is not `Resume`, and arms its timer through the same `TimerCommand`. Timer ids are consecutive constants (`MEMORY_TIMER = 4`, `FULLSCREEN_TIMER = 3`, `CLOSE_TIMER = 5`, `MOVE_TIMER = 6` in `service.rs`): pick the next free ids and document them.

**Tray/Use state:** `Status` (`service.rs` 796-904) has `text()`/`tooltip()`; extend it with a rule variant ("Paused: Processor usage", D-27) and mirror in `RuleState` (`portal.rs` 266).

---

### `apps/sucrose/src/data_hub.rs` (on-demand system data to web wallpapers)

**Analog:** `modules.rs` `MemoryModule` + `MemorySampler` (122-163) for "exists only while wanted, emits a report on a timer". The hub exists only while a renderer with `SucroseCompatible.json` runs and samples only the groups that file defines (D-24). Provider code lives in `sucrose-sysinfo` (unsafe allowed, D-34); the hub is safe code.

---

### `crates/sucrose-sysinfo/src/*` (platform provider crate, new)

**Analog:** `crates/sucrose-platform/src/{lib.rs,os_windows/memory.rs,unsupported.rs}`: public safe API in `lib.rs`, `cfg(windows)` module with `// SAFETY:` per `unsafe` block, `unsupported.rs` returning a capability error so unsupported OSes report rather than fake (D-002). `PlatformError` (see `lib.rs`) is the error type used by `start_module`. Payload models and the audio visualizer algorithm are pure and need fixture tests on every OS. Remember D-34: a new `docs/DECISIONS.md` entry plus updates to CLAUDE.md non-negotiable 5, `.claude/rules/platform-ffi.md`, `docs/ARCHITECTURE.md` in the same change.

---

### `apps/sucrose/src/properties.rs` (property working-copy store, single writer)

**Analog:** `apps/sucrose/src/settings_host.rs`

**Core pattern** (27-100): host struct owns the store, `open(...) -> (Self, LoadOutcome)`, `unsaved(...)` for tests, `apply(change) -> Result<SettingChange, String>` = `validate` then `update`, and a user-facing constant `NOT_SAVED = "Your setting couldn't be saved. Try again."`; a failed write keeps the previous value:
```rust
pub fn apply(&mut self, change: SettingChange) -> Result<SettingChange, String> {
    let stored = validate(change)?;
    self.update(|settings| write(settings, &stored))?;
    Ok(stored)
}
```
Copy: validate/clamp then write, return the stored value so the Portal shows exactly what is persisted, "the same change twice leaves the same file bytes" test (settings_host tests 287), scratch-dir test helper `Scratch` (211-226). D-20 one-time copy from the real `%APPDATA%\Sucrose\Cache\Properties` into `<data_dir>/Properties` follows the legacy migration style in `crates/sucrose-config/src/legacy.rs`/`migrate.rs` (read-only source, never write the C# files). Portal-side library maintenance with temp-file + rename is in `apps/sucrose/src/library_ops.rs`.

---

### `apps/sucrose/src/render.rs` (engine-agnostic renderer host)

**Analog:** itself. Key seams to keep: connect-with-retry (34-35, 750), `run(slot)` (83), `EngineError` -> `exit_code()`/`report()`/`hint()` (274-310), `report_failure` (319) which sends `ToService::Failed { reason, hint }`, `start_engine(wid, &Assignment)` (351, becomes the trait factory), `wait_for_close` (432) message loop, `switch_source` (504, `LoadSource`), `apply_role` (528), `follow_clock`/`publish_clock`. Change must be "no behavior change" for mpv; `apps/sucrose/tests/*` is the regression net.

---

### `apps/sucrose/src/topology.rs` and `service.rs` (per-target fleet, Span/Duplicate)

**Analog:** itself.

- `SlotMap` (110-157: `assign`, `slot_of`, `monitor_of`, `retire`, `release`) maps monitor id -> renderer slot; key it by the stable monitor id (D-08) rather than `\\.\DISPLAYn`.
- `diff`/`replace_all`/`actions_between` (52-91) produce `TopologyAction` (`Close`, `SetRect`, `Spawn`) with the ordering "closes, then rect changes, then spawns" (tests 342, 433). A Span target is one synthetic monitor whose rect is the union; reuse the same actions.
- `Debounce` with `TOPOLOGY_DEBOUNCE`, `DEBOUNCE_MS = 2000`, `SETTLE_MS = 500` (158-260) is the hot-plug debounce for DISPLAY9 connect/disconnect.
- `service.rs` is 5,493 lines: use `grep -n` for `Fleet`, `Assignment`, `selected_job` (655), `close_for_move`/`resume_wallpaper` (583-635) as the "close all renderers then restart" template for placement changes (D-02), and `start_fullscreen` (920) for per-monitor pause. Do not read the file whole.

---

### Portal settings pages: `settings/wallpaper.rs`, `settings/performance.rs`, `settings/mod.rs`

**Analog:** `apps/sucrose-portal/src/settings/general.rs` (1,252 lines, use grep for the row builders) and `personal.rs` (423) with `row.rs`.

**Row pattern** (`row.rs` 54-130): rows are `SettingRow::new(title, description).control(control, width).footer(footer_line(label, control)).state(control_state(has_settings, connected)).build(cx)`. `ControlState` is `Loading | Disabled | Enabled`; controls are disabled while the service is disconnected. Helpers: `choice_select`/`show_choice`/`on_choice` (243-283) for enum selects, `NumberField` (293-333) with `commit_number` clamp, `slider_percent` (41), `label_text(key)` (49) for rust-i18n labels, widths `SELECT_WIDTH = 200`, `NUMBER_WIDTH = 100`.
- Engine choice rows: `Choice::new(value, title)` implements `SelectItem` with `disabled()` (220), so a missing engine is a disabled item plus the persistent warning alert (D-11, UI-SPEC warning colors, add the light pair to `themes/sucrose.json` and read through one `theme.rs` helper).
- New settings: append a `SettingChange` variant, validate in `settings_host::validate`, write in `settings_host::write`, show through `PortalSettings`.
- Strings: add keys to `locales/sucrose.en.yml`; legacy-derived keys come from `xtask/src/localize.rs` (extend `PREFIXES`, `REPLACED_KEYS`, never both files).

---

### `apps/sucrose-portal/src/dialogs/{install,display_preferences}.rs`

**Analog:** `dialogs/move_library.rs` (45-82) and `dialogs/mod.rs`.

```rust
window.open_dialog(cx, move |dialog, window, _cx| {
    dialog
        .title(tr("Portal.GeneralSettingPage.PrivateLibrary.ConfirmTitle"))
        .w(px(DIALOG_WIDTH))
        .child(body(window, [v_flex().gap_3().children(rows).into_any_element()]))
        .footer(DialogFooter::new()
            .child(Button::new("move-library-keep").label(...).on_click(|_, window, cx| window.close_dialog(cx)))
            .child(Button::new("move-library-confirm").danger().label(...).on_click(move |_, window, cx| {
                window.close_dialog(cx); on_confirm(window, cx); })))
});
```
Reuse `dialogs::{body, close_footer, link_row, open_link, DIALOG_WIDTH = 480, body_max_height}`; keep the pure text builder (`paragraphs(...)`) separate from `open` so it is unit-testable (tests 87-100). Display Preferences needs a wider custom canvas: keep the scaled-rectangle math pure (like `crates/sucrose-platform/src/geometry.rs` / `layout.rs`, tests in `crates/sucrose-platform/tests/layout.rs`). Register every new Lucide icon in `assets.rs` (`icon_assets!`) before use.

---

### `apps/sucrose-portal/src/toast.rs` (fallback and hint toasts)

**Analog:** itself. `show_info`/`show_error` (75-82) wrap `show(Kind, text, window, cx)`; `ToastStack` bounds to `MAX_TOASTS = 3` with eviction; errors self-time to 8 s via `window.spawn`. A warning toast is one more `Kind` variant with `duration()` and `Notification::warning`; keep the stack logic untouched and add the matching pure test (`errors_last_eight_seconds_and_the_rest_five`, 116).

---

### `apps/sucrose-portal/src/library/card.rs`

**Analog:** itself (`DisabledReason`, `CardUse`, hover). Lift "Use disabled for non-Video" per type as each slice lands by removing that type from the disabled reason; add the Customize icon and monitor badge in the hover/overlay area using the existing hover pattern; add pure tests beside the existing card tests.

---

### `xtask/src/fetch_engines.rs`, `xtask/engines.toml`, `xtask/src/bindgen_vlc.rs`

**Analog:** the same files plus `xtask/src/bindgen_mpv.rs`.

- **Pin shape** (`engines.toml` 7-20, `fetch_engines.rs` 24-40): `[[engine]]` with `name, arch (x86_64|i686|aarch64), url, sha256, size, file, header, upstream_repo, upstream_tag, upstream_commit, <version>, client_api, license`. `Pin` requires every field (`mpv_version`, `client_api` are mpv-named): generalise those into optional fields or `version` before adding CEF/libVLC rows, and extend `parse_manifest` (67) validation and the test `the_committed_manifest_pins_three_architectures` (435).
- **Verify before open** (`verify_sha256` 111; test `a_mismatching_archive_is_deleted_and_never_extracted` 501); `extract_pinned` (336) writes only named entries to fixed paths (test 553): `.tar.bz2` extraction must keep the "fixed paths, no traversal" property.
- **bindgen** (`bindgen_mpv.rs` 20-116): `FUNCTIONS`/`TYPES` allowlists and a `cfg`-gated `generate`; copy as `bindgen_vlc.rs` over `sdk/include/vlc`, output `crates/sucrose-engine-vlc/src/sys.rs`, wire into `xtask/src/main.rs`.
- `xtask ci` must export `CEF_PATH` (D-32) so no build script downloads.
- Measurement rows: `xtask/src/measure.rs` (3,262 lines; grep for the per-process row builders) for per-engine RAM/CPU (D-10).

---

### Tests for the service and Portal

**Analog:** `apps/sucrose/tests/portal.rs` with `tests/common/mod.rs`; `crates/sucrose-ipc/tests/{duplex,socket}.rs` for IPC round trips (add a postcard round-trip per appended variant and a "v3 renderer vs v4 service" case).

## Shared Patterns

### Doc comments and naming
**Source:** every analog above. Each public item has a `///` comment that states the decision id (D-0xx) behind it; tests are sentence-named (`fn a_refused_value_changes_nothing()`). Code, comments and docs in English.

### unsafe isolation (D-011)
**Source:** `crates/sucrose-engine-mpv/src/lib.rs` 6-7 and `loader.rs` 249-266.
**Apply to:** `sucrose-engine-{vlc,cef,webview2,builtin}`, `sucrose-sysinfo`, new `sucrose-platform` modules. `#![allow(unsafe_code)]` only in these crates; each `unsafe` block carries `// SAFETY:`. `sucrose-engine`, `sucrose-engine-web`, `sucrose-core`, `apps/*` stay `unsafe`-free.

### Optional modules do not start when off (D-008)
**Source:** `apps/sucrose/src/modules.rs` 21-47, 176-194.
**Apply to:** governor, data hub, audio capture, now-playing, process-event watcher.

### Pure decision functions plus thin OS adapters
**Source:** `fullscreen.rs` (`covered_monitors`, `Debounce`, `plan_action`), `topology.rs` (`diff`), `supervisor.rs` (`decide`), `settings_host.rs` (`validate`).
**Apply to:** `choose_engine`, origin resolver (SEC-02 traversal vectors), condition predicates, display assignment resolution, property validation. These run on all three CI OSes.

### Renderer failure reporting
**Source:** `render.rs` 274-331 and `supervisor.rs` 19-27, 56-66.
**Apply to:** every engine start path: map failures to an exit code class, send `ToService::Failed { reason, hint }` with one sentence each, log details, never panic.

### Settings write path (single writer)
**Source:** `apps/sucrose/src/settings_host.rs` 84-120 and `crates/sucrose-config/src/store.rs`.
**Apply to:** assignments, certificate switch, engine arguments, property working copies. The Portal never writes files (Phase 3 D-11); it sends a change and shows the stored value returned.

### Cross-platform build rule
**Source:** `crates/sucrose-platform/src/{lib.rs,unsupported.rs,os_windows.rs}`.
**Apply to:** new engine crates compile on every OS (Windows-only code behind `cfg(windows)`, other OSes report unavailable through capabilities, never faked, D-002). CEF and WebView2 crates need `cfg` guards so `cargo clippy --workspace` passes on Linux and macOS CI.

### Portal i18n
**Source:** `apps/sucrose-portal/src/i18n.rs` (`tr`) and `locales/sucrose.en.yml`.
**Apply to:** every new visible string uses a `tr("Portal....")` key; none are hard-coded in UI code.

## No Analog Found

| File | Role | Data Flow | Reason / fallback source |
|---|---|---|---|
| `crates/sucrose-engine-cef/build.rs` (delay-load of `libcef.dll`) | build script | build | No build script exists in the workspace; use RESEARCH.md "Delay-load build script" (assumption A1, spike gate in slice 1) |
| CEF message pump / windowed child browser | engine glue | event-driven | No windowed-browser code yet; use RESEARCH.md "Process entry order" and cef-rs external-pump example; `crates/sucrose-platform/src/os_windows/{surface,event_loop}.rs` for the parent HWND and thread |
| `crates/sucrose-engine-builtin/src/gif/*` Direct2D presenter | presenter | streaming | No Direct2D code in the repo; use RESEARCH.md Findings 4 and `os_windows/surface.rs` for the surface thread |
| `crates/sucrose-engine-builtin/src/mf/*` (`IMFMediaEngine`) | engine | streaming | No Media Foundation code; RESEARCH.md Findings 5 |
| Second Portal window (properties editor) | component | event-driven | Portal opens exactly one window (`apps/sucrose-portal/src/main.rs`, `gpui_kit::open_window`); spike first (RESEARCH slice 5) |
| Process start/stop events and Application window embed (`Win32_ProcessStartTrace`, WinEvent hook) | platform | event-driven | No WMI/WinEvent code; `os_windows/job.rs` (process containment) and `foreground.rs` (window inspection) are partial; RESEARCH.md Findings 9, 10, D-36 |

## Metadata

**Analog search scope:** `crates/`, `apps/`, `xtask/` (tracked files via `git ls-files`); `target/`, `exp/`, `src/` (legacy) not searched.
**Files scanned:** about 30 read or grepped for structure (large files `service.rs`, `general.rs`, `measure.rs`, `player.rs` by grep only).
**Pattern extraction date:** 2026-11-26
