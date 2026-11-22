# Phase 2: M1 - Video wallpaper on the desktop (Windows) - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 30 (new or modified)
**Analogs found:** 14 / 30 (the repo has only the M0 skeleton, so most analogs are structural conventions, not feature code)

All analogs below are git-tracked (verified with `git ls-files`). No gitignored mirror paths.

**Reconciled with the plans (revision 1, 2026-11-22):** the file paths below are the ones plans 02-01 … 02-15 create or modify. Where RESEARCH proposed other names (a separate legacy-paths module, an IPC endpoint module, a `windows/` module folder) the table uses the plans' files (`paths.rs`, `sucrose-ipc/src/lib.rs`, `os_windows/`). The Engine trait crate proposed by RESEARCH is not created in Phase 2: M1 has one engine, and the crate stays planned for the second engine in Phase 4 (plan 15 lists it as planned in `docs/ARCHITECTURE.md`).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `Cargo.toml` (workspace deps, member paths; version form chosen at the plan 08 checkpoint, RESEARCH proposes `26.11.22+1`) | config | n/a | `Cargo.toml` itself (L15-26) | exact |
| `crates/sucrose-config/{Cargo.toml,src/lib.rs,src/legacy.rs,tests/legacy.rs,tests/fixtures/*.json}` (plans 03–04) | service (reader) | file-I/O, transform | `crates/sucrose-core/src/paths.rs` + `crates/sucrose-log/Cargo.toml` | role-match |
| `crates/sucrose-core/src/{theme.rs,settings.rs,version.rs}` (SucroseInfo and WallpaperType; setting enums; LegacyVersion) + `tests/theme.rs`, `tests/fixtures/SucroseInfo.*.json` (plans 03, 04, 08) | model | transform | `crates/sucrose-core/src/lib.rs`, `paths.rs` | partial |
| `crates/sucrose-core/src/paths.rs` (new `legacy_data_dir`, `legacy_setting_dir`, `legacy_mpv_cache_dir` next to the existing functions, plan 03) | utility | request-response | `crates/sucrose-core/src/paths.rs` | exact |
| `crates/sucrose-ipc/{Cargo.toml,src/lib.rs (endpoint name, Listener, connect),src/frame.rs,src/protocol.rs,tests/socket.rs}` (plan 05) | service | pub-sub / request-response | none (log crate for error enum style) | no analog |
| `crates/sucrose-engine-mpv/{Cargo.toml,src/lib.rs,src/sys.rs,src/loader.rs,src/player.rs,src/config.rs,src/sync.rs,tests/load.rs,tests/options.rs}` (plans 01, 07, 11, 14) | service (FFI) | event-driven | none; `sucrose-platform` for the unsafe opt-out rule | no analog |
| `crates/sucrose-platform/src/lib.rs` (extend trait, Capabilities) | service | request-response | itself L22-45 | exact |
| `crates/sucrose-platform/src/os_windows.rs` (monitors, DPI awareness, desktop refresh, monotonic clock) and `src/os_windows/{surface,event_loop,tray,job,foreground}.rs`; portable `src/{geometry,layout,icon}.rs`; `tests/layout.rs`, `tests/fixtures/desktop-*.json` (plans 05, 06, 09, 10, 12, 13, 14) | service | event-driven | `crates/sucrose-platform/src/os_windows.rs` | exact (placement) |
| `crates/sucrose-platform/src/os_linux.rs`, `os_macos.rs` | service stub | n/a | `os_windows.rs` M0 stub | exact |
| `crates/sucrose-platform/Cargo.toml` (+ `windows` crate, cfg target deps) | config | n/a | itself | exact |
| `apps/sucrose/{Cargo.toml,src/main.rs,src/service.rs,src/render.rs,src/supervisor.rs,src/topology.rs,src/fullscreen.rs,assets/TrayIcon.ico}` (single files, no `service/` or `render/` folders; the tray itself is `crates/sucrose-platform/src/os_windows/tray.rs`) | controller (binary) | event-driven | `apps/sucrose-portal/{Cargo.toml,src/main.rs}` | role-match |
| `xtask/src/{main,fetch_engines,bindgen_mpv,measure}.rs`, `xtask/engines.toml` | utility | batch, file-I/O | `xtask/src/main.rs` | exact |
| `crates/sucrose-log` (unchanged, called with "Service" / "Renderer-N") | utility | n/a | `crates/sucrose-log/src/lib.rs` | exact |
| tests and fixtures `crates/*/tests/fixtures/` | test | transform | in-file `#[cfg(test)]` modules (paths.rs L37-50, platform lib.rs L47-53) | role-match |
| docs (DECISIONS D-028, FEATURES, MEASUREMENTS, ARCHITECTURE, CLAUDE.md, rules) | docs | n/a | existing docs | n/a |

## Pattern Assignments

### New crate manifests (sucrose-config, -ipc, -engine-mpv, apps/sucrose)

**Analog:** `crates/sucrose-log/Cargo.toml` (lib) and `apps/sucrose-portal/Cargo.toml` (app)
```toml
[package]
name = "sucrose-log"
description = "Logging setup shared by Sucrose binaries"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
sucrose-core = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[lints]
workspace = true
```
Rules: members use `dep = { workspace = true }`; add new crates to root `[workspace.dependencies]` (Cargo.toml L16-18 path style: `sucrose-x = { path = "crates/sucrose-x" }`) and versions there (L20-26). `members = ["crates/*", "apps/*", "xtask"]` already globs new crates. Forbidden crates per `.claude/rules/cargo.md`: vlc-rs, libmpv-sys, single-instance, libmpv2.

### FFI crates (sucrose-platform os_windows/, sucrose-engine-mpv)

**Analog:** `crates/sucrose-platform/src/lib.rs` L1-3 doc and workspace lint (Cargo.toml L28-30 `unsafe_code = "deny"`).
Opt out per crate with `#![allow(unsafe_code)]` at the crate root, each block with `// SAFETY:`. Do not opt out in sucrose-ipc, -config, -core, -engine (the ACL helper goes in sucrose-platform if it needs unsafe). Clippy lints (L32-37) deny-ish: `unwrap_used`, `print_*`, `todo`, `dbg_macro`; use `tracing`, no `unwrap` on I/O/FFI.

### `crates/sucrose-platform` extensions

**Analog:** `crates/sucrose-platform/src/lib.rs` L5-45 and `os_windows.rs`
```rust
#[cfg(target_os = "windows")]
mod os_windows;
#[cfg(target_os = "windows")]
use os_windows as imp;

pub struct Capabilities { pub desktop_wallpaper: bool, pub embed_application: bool, pub fullscreen_detection: bool }

pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
}
pub fn current() -> Box<dyn Platform> { Box::new(imp::Current) }
```
`os_windows.rs` L5-15 is the stub to fill: `pub(crate) struct Current;` + `impl Platform for Current` returning `Capabilities::default()`; flip `desktop_wallpaper` and `fullscreen_detection` to true. Put new Windows modules under `os_windows/` (`os_windows.rs` plus `os_windows/*.rs`, as the plans do; RESEARCH's `windows/` name is not used), keeping the `#[cfg(target_os = "windows")] mod` + `imp` alias scheme. Portable pure code (layout classifier, rect math, ICO parsing) sits at the crate root so it is tested on every OS. Linux and macOS stubs mirror the Windows stub (return default capabilities) so CI keeps compiling on all three OSes. Test style (L47-53): `current_matches_the_build_target`. Pure parts (layout classifier) must be testable with fixtures and no desktop.
Caveat: workspace dep name `windows` must be added under `[target.'cfg(windows)'.dependencies]` to avoid Windows-only deps on Linux/macOS CI (RESEARCH Pitfall 10).

### Engine error enums (`sucrose-engine-mpv` in M1; the Engine trait crate comes in Phase 4)

No plan of this phase creates an Engine trait crate; it arrives with the second engine in Phase 4. In M1 this error-enum pattern applies to `sucrose-engine-mpv` (`LoadError`, `PlayerError`).


**Analog:** `crates/sucrose-platform/src/lib.rs` (trait + capability struct + doc-comment style) and `crates/sucrose-log/src/lib.rs` L19-30 (error enum)
```rust
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("the OS reports no data directory for log files")]
    NoDataDir,
    #[error("cannot create the log folder: {0}")]
    CreateDir(#[from] std::io::Error),
}
```
Use for `LoadError` in M1 (and EngineError in Phase 4): "engine not installed" is a normal variant, not a panic (D-20).

### `crates/sucrose-core/src/paths.rs` (new legacy_* functions) and `theme.rs`

**Analog:** `crates/sucrose-core/src/paths.rs`
```rust
use std::path::PathBuf;
use crate::APP_NAME;
pub const DATA_DIR_ENV: &str = "SUCROSE_DATA_DIR";
pub fn data_dir() -> Option<PathBuf> {
    match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => dirs::config_dir().map(|dir| dir.join(APP_NAME)),
    }
}
pub fn setting_dir() -> Option<PathBuf> { data_dir().map(|dir| dir.join("Setting")) }
```
The new `legacy_data_dir` / `legacy_setting_dir` / `legacy_mpv_cache_dir` in `paths.rs` must differ: always `dirs::config_dir()/Sucrose`, ignoring `SUCROSE_DATA_DIR` (D-08), returning `Option<PathBuf>`. Register modules in `lib.rs` (L5 `pub mod paths;`). Keep core OS-free. Test shape (paths.rs L37-50): in-file `#[cfg(test)] mod tests`, early-return when `data_dir()` is None. `lib.rs` L14 `VERSION = env!("CARGO_PKG_VERSION")` is what `LegacyVersion::from_cargo` in `version.rs` parses (plan 08); its test confirms how the chosen version form (plan 08 checkpoint) survives in `CARGO_PKG_VERSION`.

### `crates/sucrose-config/src/legacy.rs`

**Analog:** `paths.rs` for path helpers (Option-returning, doc on every pub item). No JSON reader exists yet; RESEARCH Pattern 7 and `docs/legacy/formats.md` §1.1/§1.3 define the Properties wrapper. Missing or invalid key returns the legacy default (lenient typed getters). Fixtures under `crates/sucrose-config/tests/fixtures/`.

### `apps/sucrose/src/main.rs` (+ service.rs, render.rs, supervisor.rs, topology.rs, fullscreen.rs)

**Analog:** `apps/sucrose-portal/src/main.rs`
```rust
fn main() -> anyhow::Result<()> {
    let _log = sucrose_log::init("Portal")?;
    let no_focus = std::env::args().any(|arg| arg == "--no-focus");
    ...
    if let Err(error) = gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Portal)) {
        tracing::error!(%error, "cannot open the Portal window");
        cx.quit();
        return;
    }
```
Copy: `anyhow::Result<()>` main, hold the `LogGuard` for the process lifetime, `sucrose_log::init("Service")` vs `init("Renderer-<n>")` per mode (`--render`), structured `tracing::error!(%error, "...")` and no `println!`. Line 5 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` applies to the new binary too. Cargo.toml deps style as portal (anyhow, sucrose-core, sucrose-log, tracing workspace deps). No GPUI in this app (tray + Win32 only; threads + `crossbeam-channel` per `.claude/rules/rust.md`). Note `print_stderr` lint is warn: do not eprintln in the app.

### `xtask/src/main.rs` (+ fetch_engines, bindgen_mpv, measure) and `xtask/engines.toml`

**Analog:** `xtask/src/main.rs`
```rust
#![allow(clippy::print_stdout, clippy::print_stderr)]
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        _ => { eprint!("{USAGE}"); ExitCode::FAILURE }
    }
}
fn cargo() -> String { std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()) }
```
Add arms `Some("fetch-engines")`, `Some("bindgen-mpv")`, `Some("measure")` and lines in `USAGE` (L7-11). Split each task into its own module (`mod fetch_engines;`) returning `ExitCode`. Failure style: print `failed: ...` to stderr and `ExitCode::FAILURE`. Cross-platform only (D-014): no shell or PowerShell scripts. xtask deps (sha256, HTTP, TOML) go in `xtask/Cargo.toml` (currently empty `[dependencies]`) via workspace deps.

## Shared Patterns

### Logging
**Source:** `crates/sucrose-log/src/lib.rs` L33-55. `init(app)` writes `<data>/Log/<app>.<date>.log`, filter env `SUCROSE_LOG`. **Apply to:** service and each renderer; user-visible failure reasons (D-13, D-20, D-22) also go to the tray tooltip.

### Errors
**Source:** `crates/sucrose-log/src/lib.rs` L19-30 (thiserror, `#[from]`). **Apply to:** all library crates (config, ipc, engine, engine-mpv, platform); anyhow only in `apps/sucrose` and xtask.

### Data folders
**Source:** `crates/sucrose-core/src/paths.rs`. Writes (logs, generated mpv config, caches) use `paths::*` honoring `SUCROSE_DATA_DIR`; legacy reads use the new `paths::legacy_*` functions (D-08).

### Lints, edition, metadata
**Source:** root `Cargo.toml` L7-13, L28-37. Every crate has `[lints] workspace = true`.

### Tests
In-file `#[cfg(test)] mod tests` for unit tests; compatibility tests with fixtures in `tests/fixtures/` from real files minus personal data. Tests must not need the desktop (RESEARCH Pitfall 11).

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `crates/sucrose-ipc/*` | service | pub-sub | No IPC code exists; use RESEARCH Pattern 8 and Code Examples (length-prefixed postcard, single-instance listener) |
| `crates/sucrose-engine-mpv/*` | service | event-driven FFI | No libloading/bindgen code; use RESEARCH Pattern 2/3 and "Loading libmpv safely" |
| `apps/sucrose/src/{supervisor,topology,fullscreen}.rs` and `crates/sucrose-platform/src/os_windows/tray.rs` | controller | event-driven | No supervisors or tray; RESEARCH Patterns 5-6, debounce example |
| `crates/sucrose-platform/src/{layout,geometry,icon}.rs`, `os_windows.rs`, `os_windows/{surface,event_loop,tray,job,foreground}.rs` | service | event-driven | Only a stub exists; RESEARCH Patterns 1, 4, 6 |
| `xtask/src/measure.rs`, `fetch_engines.rs` | utility | batch | Only the `ci` task exists to copy dispatch style from |

## Metadata

**Analog search scope:** crates/, apps/, xtask/, root Cargo.toml (the only 15 tracked Rust/Cargo files)
**Files scanned:** 15
**Pattern extraction date:** 2026-11-22
