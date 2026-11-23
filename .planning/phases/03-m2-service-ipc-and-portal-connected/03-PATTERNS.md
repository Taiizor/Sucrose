# Phase 3: M2 - Service, IPC and Portal connected - Pattern Map

**Mapped:** 2026-11-24
**Files analyzed:** 47 new or modified (from the RESEARCH "Recommended Project Structure")
**Analogs found:** 41 / 47 (all analog paths verified git-tracked via `git ls-files`)

Line numbers refer to the tree at commit 94dae916. `exp/GPUI/Sucrose.Portal` is cited only as a GPUI usage analog (throwaway, not the base).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| `crates/sucrose-core/src/settings.rs` (more enums) | model | transform | itself (`legacy_enum!`, lines 1-100) | exact |
| `crates/sucrose-core/src/paths.rs` (+showcase_dir etc.) | utility | transform | itself, lines 14-66 | exact |
| `crates/sucrose-core/src/library.rs` (scan, validity, sort, search) | service (pure) | batch | `crates/sucrose-core/src/theme.rs` (`read_info`, `check_selected`) | role-match |
| `crates/sucrose-config/src/legacy.rs` (4 to 9 files) | model | file-I/O | itself, lines 1-140 | exact |
| `crates/sucrose-config/src/model.rs` | model | CRUD | `legacy.rs` `Properties` / `LegacySettings` | role-match |
| `crates/sucrose-config/src/migrate.rs` | service | transform | `legacy.rs` `LegacySettings::load` | role-match |
| `crates/sucrose-config/src/store.rs` | service | file-I/O | `theme.rs::write_info` (line 221) plus `legacy.rs::read` retry | partial |
| `crates/sucrose-config/tests/migrate.rs` + fixtures | test | file-I/O | `crates/sucrose-config/tests/legacy.rs` | exact |
| `crates/sucrose-ipc/src/protocol.rs` (append) | model | request-response | itself | exact |
| `crates/sucrose-ipc/src/portal.rs` | model | pub-sub | `protocol.rs` | exact |
| `crates/sucrose-ipc/src/lib.rs` (endpoint suffix) | utility | request-response | itself, lines 58-62 | exact |
| `crates/sucrose-ipc/tests/portal.rs` | test | request-response | `crates/sucrose-ipc/tests/duplex.rs` | exact |
| `crates/sucrose-platform/src/autostart.rs` | provider | file-I/O | `os_windows.rs` + `unsupported.rs` split | role-match |
| `crates/sucrose-platform/src/trash.rs` | utility | file-I/O | `unsupported.rs` capability pattern | partial |
| `crates/sucrose-platform/src/lib.rs` (Capabilities) | config | transform | itself, lines 99-112 | exact |
| `crates/sucrose-platform/src/os_*.rs` | provider | request-response | `os_linux.rs` (stub), `os_windows.rs` | exact |
| `apps/sucrose/src/main.rs` (new flags) | config | request-response | itself `parse_mode` | exact |
| `apps/sucrose/src/service.rs` (shrink run) | controller | event-driven | itself | exact |
| `apps/sucrose/src/portal_link.rs` | service | pub-sub | `service.rs` `Link`, `accept_loop`, `read_connection` | exact |
| `apps/sucrose/src/settings_host.rs` | service | CRUD | `service.rs` `Notifier`/`Event` plus config store | role-match |
| `apps/sucrose/src/lifecycle.rs` | utility | event-driven | `supervisor.rs` (pure `decide`) | role-match |
| `apps/sucrose/src/library_ops.rs` | service | batch / file-I/O | `service.rs` worker threads | partial |
| `apps/sucrose/src/modules.rs` | utility | event-driven | `service.rs::start_tray` (line 328) | role-match |
| `apps/sucrose/src/render.rs` + `crates/sucrose-engine-mpv` (LoadSource) | controller | event-driven | `render.rs` handling of `ToRenderer` | exact |
| `crates/sucrose-platform/src/os_windows/tray.rs` (menu) | component | event-driven | itself | exact |
| `apps/sucrose-portal/src/main.rs` | component | event-driven | itself, lines 1-74 | exact |
| `apps/sucrose-portal/src/ipc.rs` | service | pub-sub | `ipc/tests/duplex.rs` + `connect` | role-match |
| `apps/sucrose-portal/src/library/thumbs.rs, preview.rs` | service | streaming/transform | `exp/GPUI/Sucrose.Portal/src/thumbs.rs` | exact (GPUI use only) |
| `apps/sucrose-portal/src/theme.rs` (backdrop) | utility | transform | `exp/.../src/app.rs` lines 107-135 | exact (GPUI use only) |
| `apps/sucrose-portal/src/{header,status_bar,banner,library/*,settings/*,dialogs/*,state,i18n}.rs` | component | event-driven | `apps/sucrose-portal/src/main.rs` render, plus exp `app.rs`, `i18n.rs` | partial |
| `xtask/src/measure.rs` (Portal mode) | utility | batch | itself | exact |
| `xtask/src/localize.rs`, `synth_library.rs` | utility | batch | `xtask/src/measure.rs` arg parsing (lines 80-160) | partial |

## Pattern Assignments

### `crates/sucrose-core/src/settings.rs` (model, transform)
**Analog:** itself. Add every enum of `formats.md` section 6 with the macro (lines 6-45 define it; usage lines 47-100):
```rust
legacy_enum! {
    /// How a wallpaper is fitted ... legacy default is `UniformToFill`.
    Stretch, default UniformToFill {
        None, Fill, Uniform,
        #[default] UniformToFill,
    }
}
```
The macro yields `ALL`, `legacy_name`, `from_legacy_name` (exact, case-sensitive) and serde derives. Use `legacy_index!` where ordinals are documented. Declaration order is the legacy order; never reorder (serde name based, but tests assert ordinals).

### `crates/sucrose-core/src/paths.rs` (utility)
**Analog:** itself. Pattern (lines 14-66): one tiny fn per folder returning `Option<PathBuf>` built from `data_dir()`; legacy ones use `legacy_data_dir()` which ignores `SUCROSE_DATA_DIR`.
```rust
pub fn setting_dir() -> Option<PathBuf> { data_dir().map(|dir| dir.join("Setting")) }
```
Add `showcase_dir()` (`data_dir()/Showcase`, D-10), `default_library_dir()`, the Run value name constant. Extend the existing `#[cfg(test)]` test (lines 68-79) the same way.

### `crates/sucrose-core/src/library.rs` (service-pure, batch)
**Analog:** `crates/sucrose-core/src/theme.rs`: `read_info(theme_dir)` (line 184), `parse_info(text)` (205), `check_selected(location, selected, app: &LegacyVersion)` (362), `INFO_FILE`. Imports (lines 5-12): `std::{fs, io, path::{Path, PathBuf}}`, `serde`, `crate::version::LegacyVersion`. Reuse `read_info` and the `AppVersion` comparison for validity and LIB-08; no OS code (rust.md); errors are `thiserror` (see `ThemeError`, `ThemeRejection`). Pitfall 2: keep the parser as lenient as Newtonsoft; prove with fixtures from real themes.

### `crates/sucrose-config/src/legacy.rs` (extend to 9 files)
**Analog:** itself. `LegacyFile` enum + `file_name()` (lines 20-44) gains variants (no Donate, Objectionable, System, Kernel, Aurora); typed getters (lines 70-140) are the only way to read values:
```rust
pub fn get_clamped(&self, key: &str, default: i64, min: i64, max: i64) -> i64 { ... }
pub fn get_enum<T>(&self, key, default: T, from_name, from_index) -> T
```
Constants `ATTEMPTS = 3`, `RETRY_DELAY = 50 ms` implement the locked-file retry (D-03). Read-only: open for reading only.

### `crates/sucrose-config/src/model.rs`, `migrate.rs`, `store.rs`
**Analog:** `legacy.rs` for field-by-field defaulting (one `get_*` call per key, legacy default in the call). `migrate.rs` is `LegacySettings::load` generalised: read `legacy_setting_dir()` (never `setting_dir()`), build the model, apply D-05 exclusions and the D-21 mapping (`BackdropType Auto` to System + Mica). Doc-comment first line per public item; module header names the decision (`//! ... (D-008).` style).
**store.rs:** no atomic-write analog exists. Nearest: `theme.rs::write_info(path, &info) -> io::Result<()>` (line 221) for the signature style; follow RESEARCH Pattern 1 (temp file in same folder, flush, rename; `.bak`; load order file, .bak, migrate). Errors: `thiserror` enum.

### Tests: `crates/sucrose-config/tests/migrate.rs` (test, file-I/O)
**Analog:** `crates/sucrose-config/tests/legacy.rs` lines 1-50:
```rust
fn fixtures() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures") }
fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("legacy-settings").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the scratch folder under target can be created");
    ...
}
```
Fixtures live in `tests/fixtures/*.json` (existing: Library, Library.bom, General, Engine, Engine.gpu, Engine.invalid, Backgroundog, Backgroundog.pause); copy real files with personal data removed. Add Portal, Warehouse, Cycling, etc. Scratch folders only under `CARGO_TARGET_TMPDIR`, never a real legacy folder; assert legacy files are byte-identical afterwards.

### `crates/sucrose-ipc/src/protocol.rs` and `portal.rs` (model)
**Analog:** `protocol.rs` lines 1-110. Header rule and shape:
```rust
//! ... variants are therefore **only ever appended** at the end of these enums ...
pub const PROTOCOL_VERSION: u16 = 3;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ToService { Hello { protocol_version: u16, pid: u32, slot: u8 }, Failed {..}, Playing, Clock(ClockTick) }
pub enum ToRenderer { Close, Assign(Assignment), Pause(bool), SetVolume(u8), SetRect([i32;4]), SetRole(Role), Clock(ClockTick) }
```
Append `ToService::PortalHello`, `Launch`, `ToRenderer::LoadSource` at the END; `portal.rs` types derive the same set and carry wire-index tests (see the existing wire-index tests lower in `protocol.rs`). Frames are capped by `frame::MAX_FRAME = 64 KiB` (frame.rs line 13): library lists must be paged or chunked (Pitfall 4).

### `crates/sucrose-ipc/src/lib.rs` (endpoint naming)
**Analog:** itself. `endpoint_name(user_key)` (line 58) returns `Sucrose.Service.{user_key}`; add the data-dir suffix here so scratch runs do not meet the real service. Bind = single-instance check:
```rust
match options.create_sync() {
    Ok(inner) => Ok(Self { inner }),
    Err(error) if is_endpoint_taken(&error) => Err(IpcError::AlreadyRunning),
    Err(error) => Err(IpcError::Io(error)),
}
```
`connect(name)` and `Connection::{send, recv, try_clone}` (lines 130-158) are what the Portal's `ipc.rs` uses; one thread reads, a clone writes.

### `crates/sucrose-ipc/tests/portal.rs` (test)
**Analog:** `tests/duplex.rs`: unique endpoint `format!("Sucrose.Test.Duplex.{}", std::process::id())`, `Listener::bind(&name, None).expect("bind")`, client in a thread using `connect`, server `accept` then `try_clone` for a writer, `mpsc` channels and `sleep(300ms)` to prove a blocked read does not block a write.

### `apps/sucrose/src/portal_link.rs` (service, pub-sub)
**Analog:** `apps/sucrose/src/service.rs`.
- Per-peer writer `Link` (lines 103-172): bounded queue (`OUTGOING_CAPACITY = 64`), writer thread per connection, `offer()` never waits, only skippable messages are dropped when full:
```rust
let (queue, receiver) = crossbeam_channel::bounded::<ToRenderer>(OUTGOING_CAPACITY);
thread::Builder::new().name(name).spawn(move || { while let Ok(m) = receiver.recv() { if !write(m) { break; } } })?;
```
Generalise over the message type (Portal events: memory ticks and rule state are skippable; settings changes are not).
- Accept path (lines 795-830): `accept_loop` spawns `ipc-reader` thread per connection, `MAX_FAILURES = 10`, 200 ms sleep. `read_connection` (833+) reads the FIRST message and routes: `Hello` goes to the renderer path (`accept_hello`, lines 1185-1215, pure and unit-tested), `PortalHello`/`Launch` to the Portal path. Add Portal peer validation analogous to `accept_hello` (pure fn returning `Result<_, Refusal>`, tests like lines 1812-2151 and 2599).
- Tracing: `tracing::warn!(%error, slot, "...")` structured fields.

### `apps/sucrose/src/settings_host.rs` (service, CRUD)
**Analog:** `service.rs` `Event` enum (line 73) and `Notifier` (lines 85-100): threads tell the main thread through `crossbeam_channel::Sender<Event>`; no async (D-012). The host owns the `sucrose_config` store (only writer, D-11), validates a `SettingChange`, writes atomically, broadcasts via the Portal registry. Replace the direct `LegacySettings::load` use (service.rs import line 12, `plan_wallpapers` line 1687, `plan_for` 1763, `playback_settings` 1787).

### `apps/sucrose/src/lifecycle.rs` (utility, event-driven)
**Analog:** `apps/sucrose/src/supervisor.rs` pure-decision style:
```rust
pub fn decide(budget: &mut RestartBudget, class: ExitClass, now: Instant) -> Decision { match class {...} }
```
Pure enum-in, enum-out, `Instant` passed in, exhaustive unit tests. Model D-12/D-13/D-14 the same way. Restart bounds reuse `RestartBudget` (3 crashes per 60 s per monitor, line 140+); do not write a second budget.

### `apps/sucrose/src/main.rs` (strict args)
**Analog:** itself lines 17-78. Keep `parse_mode(&[String]) -> Result<Mode, String>`; add variants (e.g. `--start-portal`, Portal page flags) as explicit arms, everything else stays an error (`EXIT_USAGE = 64`) and tests listing bad inputs (lines 80-93). Log init per mode: `sucrose_log::init("Service")`.

### `apps/sucrose/src/modules.rs` (utility)
**Analog:** `service.rs::start_tray<T>(app_visible: bool, create: impl FnOnce() -> Result<T, PlatformError>) -> Option<T>` (line 328): the gate takes the setting plus a constructor closure so a test proves the constructor is never called when off (ARCH-03, D-008).

### `apps/sucrose/src/render.rs` and engine-mpv (LoadSource)
**Analog:** `render.rs` message loop for `ToRenderer::Pause/SetVolume/SetRect` (append an arm for `LoadSource`); player calls live in `crates/sucrose-engine-mpv/src/player.rs`. Unsafe stays in the engine crate with `// SAFETY:` comments. Fall back to the Phase 2 D-13 behavior on failure.

### `crates/sucrose-platform/*` (autostart, trash, capabilities, tray menu)
**Capabilities analog:** `lib.rs` lines 99-112:
```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities { pub desktop_wallpaper: bool, pub embed_application: bool, pub fullscreen_detection: bool, pub gpu_preference: bool }
```
Add `autostart`, `recycle_bin`, `window_backdrop` as documented bools; `Default` = false.
**Per-OS split:** `os_linux.rs`/`os_macos.rs` re-export `crate::unsupported::*` and return `Capabilities::default()`; Windows implementation in `os_windows.rs` + `os_windows/*.rs`. Anything an OS cannot do returns `PlatformError::Unsupported`, never faked. Pure text builders (Run value, LaunchAgent plist, XDG desktop file) go in a non-cfg module and are tested on every OS (RESEARCH "Start-with-OS text builders"). `trash` crate wrapper: check licence per `.claude/rules/cargo.md`.
**Tray menu:** `os_windows/tray.rs` `Tray::new(ico, tooltip, proxy)` builds `Menu` with one `MenuItem` and `MenuEvent::set_event_handler` capturing ids; Drop clears the handler. Extend to Open Sucrose + exit item (label from `AppExit`); event delivery goes through `LoopProxy` into the service `Event` channel, with a matching stub in `unsupported.rs`. Every `unsafe` keeps its `// SAFETY:` comment (lines 30-45).

### `apps/sucrose-portal/src/main.rs` (component)
**Analog:** itself (whole file). Entry shape to keep: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, `sucrose_log::init("Portal")`, `--no-focus`, `gpui_kit::application().with_assets(...).run(...)`, quit on last window closed, window options (900x600 min, `TitleBar::window_options()`), `open_window` error logged with `tracing::error!(%error, ...)`. Render root: `v_flex().size_full().bg(cx.theme().background).text_color(cx.theme().foreground).child(TitleBar::new()...)`. Add `WindowBackgroundAppearance` to `WindowOptions` from settings.

### `apps/sucrose-portal/src/library/thumbs.rs`, `preview.rs` (GPUI usage analog only)
**Analog:** `exp/GPUI/Sucrose.Portal/src/thumbs.rs` lines 1-68: decode at card size on a background thread, global decode-slot semaphore (`DECODE_SLOTS = 3`), JPEG DCT scaling, BGRA swap, `RenderImage::new(SmallVec::from_elem(image::Frame::new(pixels), 1))`. Adapt: legacy size 360 px (D-24 RESEARCH), add a bounded LRU cache and `drop_image`; replace `unwrap()` on the mutex (rust.md: no unwrap on non-invariants) and the HTTP fetch path (store is out of scope). Preview GIF: decode capped frames only for the hovered card, release on hover end.

### `apps/sucrose-portal/src/theme.rs` (backdrop)
**Analog:** `exp/GPUI/Sucrose.Portal/src/app.rs` lines 107-135: enum `Backdrop` with `appearance()` mapping to `WindowBackgroundAppearance::{Blurred, MicaBackdrop, MicaAltBackdrop, Opaque}`. Phase 3 maps None/Mica/Acrylic/Tabbed (Tabbed = MicaAltBackdrop), gated by `Capabilities.window_backdrop` (D-22: disabled with tooltip, unsupported stored value applied as None without changing setting).

### Other Portal UI files (header, status_bar, banner, library/*, settings/*, dialogs/*, i18n)
**Analog:** exp `src/app.rs` (sections, settings rows) and `src/i18n.rs` for string tables are usage hints only; follow `03-UI-SPEC.md` and `docs/GPUI.md`. The `rust-i18n` keys must match legacy XAML keys (D-20); never block the GPUI thread with I/O (rust.md).

### `xtask/src/measure.rs` (Portal mode)
**Analog:** itself: `classify(side, name, cmd)` (line 178), `scan` (225), `parse_args` (112) with `value()`/`seconds()` helpers, `sample` (616), row builders `csharp_cell`/`rust_cell` (917-935). Add a Portal role to `classify` (Pitfall 12), keep parse errors as `Result<_, String>` and the pure parsing functions unit-tested.

## Shared Patterns

### Thread + channel IPC (no async)
**Source:** `apps/sucrose/src/service.rs` lines 103-172, 795-830. Apply to portal_link, settings_host, Portal `ipc.rs`. Bounded per-peer queue, writer thread, never write under a lock, skippable messages dropped when full.

### Errors and logging
**Source:** `crates/sucrose-ipc/src/lib.rs` lines 20-50 (`thiserror` enum with `#[error("...")]`, `From<io::Error>` mapping), `apps/sucrose/src/main.rs` (`anyhow::Context`). Apply: thiserror in crates, anyhow in apps, `tracing` with structured fields, no `println!`.

### Read-only legacy access
**Source:** `crates/sucrose-core/src/paths.rs` (`legacy_setting_dir`, ignores `SUCROSE_DATA_DIR`) and `legacy.rs` header. Apply: migration and every test; assert untouched afterwards.

### Pure decision functions with unit tests
**Source:** `supervisor.rs::decide`, `service.rs::accept_hello`, `main.rs::parse_mode`. Apply: lifecycle, Portal peer acceptance, command line, autostart text builders, module gates.

### Fixture tests
**Source:** `crates/sucrose-config/tests/legacy.rs` and `crates/sucrose-core/tests/theme.rs` with `tests/fixtures`. Apply: migration, library scan (real themes), protocol wire-index.

### Platform gaps
**Source:** `os_linux.rs` + `unsupported.rs`. Apply: every new platform function gets a stub returning `Unsupported` and a capability flag.

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `crates/sucrose-config/src/store.rs` (atomic write, .bak) | service | file-I/O | No atomic write exists; use RESEARCH Pattern 1 |
| `crates/sucrose-platform/src/autostart.rs` Run value / LaunchAgent / XDG | provider | file-I/O | No registry-write or autostart code exists |
| `crates/sucrose-platform/src/trash.rs` | utility | file-I/O | New dependency |
| `apps/sucrose/src/library_ops.rs` verified move | service | batch | No file move/copy-verify code exists; RESEARCH Pattern 5 |
| `apps/sucrose-portal/src/library/{grid,layout,pagination}.rs` and settings/dialog views | component | event-driven | Portal is from scratch; only the empty window and exp usage hints exist |
| `xtask/src/localize.rs`, `synth_library.rs` | utility | batch | No generator exists; copy xtask arg style only |

## Metadata

**Analog search scope:** crates/, apps/, xtask/, exp/GPUI/Sucrose.Portal (usage only)
**Files scanned:** about 20 read in part, 90 tracked sources listed
**Pattern extraction date:** 2026-11-24
