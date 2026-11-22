# CLAUDE.md

Sucrose is an open-source live wallpaper engine (GIF, URL, Web, Video, YouTube and Application wallpapers, with audio-reactive and system-status APIs for web wallpapers). The `develop-rust` branch is a **from-scratch rewrite in Rust** of the existing C# app, for Windows, macOS and Linux, keeping every existing feature and improving them where possible.

## Status

- M0 (workspace skeleton and CI) is in place; no product feature exists yet. The roadmap, current phase and requirements are in `.planning/` (see Planning below).
- `exp/GPUI/Sucrose.Portal` is a throwaway UI experiment (GPUI) that reads the real library, settings and store read-only. It is not the base of `apps/sucrose-portal`.
- `src/` is the legacy C# app: a **read-only reference** for porting. `src/CLAUDE.md` (loaded when you open files there) explains its layout and the using-alias convention. Do not edit, build or reformat it unless the maintainer asks.

## Read before you work

| Task | Read first |
|---|---|
| New crate, process, trait, IPC or data flow | `docs/ARCHITECTURE.md` |
| Porting a feature from C# | `docs/PORTING.md` and the feature's row in `docs/FEATURES.md` (or the `port-feature` skill) |
| Exact legacy behavior, settings keys, file formats, JS API | `docs/legacy/` |
| UI work | `docs/GPUI.md` |
| Choosing an approach or a dependency | `docs/DECISIONS.md` |
| What to do next | `.planning/STATE.md`, then `.planning/ROADMAP.md` |

## Non-negotiables

1. **Parity.** Every C# feature is tracked in `docs/FEATURES.md`. Never drop or change one silently; record behavior changes there.
2. **Compatibility (D-007).** Themes (`SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json`, archives) and the web wallpaper JavaScript API stay compatible; legacy `Setting/*.json` files are migrated automatically to the new settings format. Prove it with fixture tests.
3. **No web runtime in the core (D-004).** The UI is GPUI via `gpui-kit`. WebView2, CEF, libmpv and libVLC are optional engines the user installs; Sucrose never bundles or installs them and loads them at runtime (D-005, D-013).
4. **Cross-platform from day one (D-002).** OS-specific code lives only in `sucrose-platform` and engine crates. Anything an OS cannot do is reported through platform capabilities, never faked. CI builds Windows, macOS and Linux.
5. **unsafe is isolated (D-011).** Only in `sucrose-platform` and engine/FFI crates, each block with a `// SAFETY:` comment.
6. **Low resource use is a feature.** Optional service modules must not start when the user disables them (D-008). For user-visible changes, measure RAM/CPU (release build) and record notable numbers in `docs/MEASUREMENTS.md`.
7. **Decisions are written down.** Follow `docs/DECISIONS.md`. Do not reopen Accepted entries; confirm Proposed ones with the maintainer before expensive or hard-to-undo work.

## Workspace (see `docs/ARCHITECTURE.md`)

`crates/` (exists: sucrose-core, sucrose-log, sucrose-platform; planned: -config, -ipc, -engine, -engine-*, -sysinfo, -store, -update), `apps/` (sucrose service/renderer, sucrose-portal UI; planned: sucrose-setup), `xtask/` (automation), `docs/`, `src/` (legacy, excluded from the workspace), `exp/` (experiments, excluded).

## Commands

From the repository root:

```
cargo build --workspace                 # add --release for measurements
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo xtask ci                          # the same checks as CI: fmt, clippy, test
```

The GPUI experiment is a separate Cargo project: `cd exp/GPUI/Sucrose.Portal && cargo run --release -- --no-focus`.

## Environment

- The maintainer works on Windows 11 from both WSL2 and native PowerShell. Everything committed must work from both: no WSL-only scripts, no `/mnt/...` paths, no symlinks. Automation goes into `xtask`, not shell or PowerShell scripts.
- In WSL, `cargo` is the Windows toolchain behind a wrapper and produces Windows binaries only. Never install a Linux Rust toolchain in WSL. Linux builds run in Docker (official `rust` image) or CI; macOS builds run in CI and need a real Mac for manual testing.
- A first full GPUI build takes 3–4 minutes; incremental builds take seconds. Windows locks running executables: stop the app you started before rebuilding.
- Running a GUI build opens a real window on the maintainer's desktop. For automated checks start it without focus (`--no-focus` in the prototype), capture with `PrintWindow`, never synthesize mouse or keyboard input, and close what you started.
- When you run the apps, set `SUCROSE_DATA_DIR` to a scratch folder so the real `%APPDATA%\Sucrose` is not touched (from WSL also set `WSLENV=SUCROSE_DATA_DIR`); `SUCROSE_LOG` sets the log level.
- The settings, library and store cache the apps read are the maintainer's real data. Treat them as read-only unless the task is about writing them; never delete user files.

## Conventions

- Rust edition 2024, stable toolchain pinned in `rust-toolchain.toml`; code must pass `rustfmt` and `clippy -D warnings`.
- Code, comments, docs and commit messages in English.
- Language-specific rules load from `.claude/rules/` when you open matching files (Rust, platform/FFI, UI, Cargo manifests, docs).

## Keeping the docs true

- Decision made or changed → `docs/DECISIONS.md`. Feature ported → its row in `docs/FEATURES.md`. Measurement → `docs/MEASUREMENTS.md`. Requirement or phase done → `.planning/` (the GSD workflow updates it). Structure changed → `docs/ARCHITECTURE.md`. New GPUI lesson → `docs/GPUI.md`. Do this in the same change as the code.
- Keep this file short (under ~150 lines) and free of anything the code already shows; details belong in `docs/` or `.claude/rules/`.
- When parity is signed off (M7), the C# code is removed and this file and `docs/` are rewritten for the finished project.

## Planning (GSD)

- The roadmap lives in `.planning/` (GSD): `PROJECT.md`, `REQUIREMENTS.md`, `ROADMAP.md`, `STATE.md`. Phases 1–8 are milestones M0–M7 ("Phase 2: M1 — …"). `docs/ROADMAP.md` only points there.
- Start edits through a GSD command unless the user explicitly asks for a direct change: `/gsd-execute-phase` for planned phase work, `/gsd-quick` for small fixes and doc updates, `/gsd-debug` for bugs, `/gsd-fast` for trivial edits.
- GSD workflows may commit their own work (planning files and atomic per-plan commits). Pushing still needs the maintainer.
- `docs/` stays authoritative for decisions, parity, architecture and legacy behavior; `.planning/` holds requirements, phases and progress. When they disagree, fix one of them in the same change.

## Git

- `develop-rust` is the rewrite's main branch; larger work goes on branches off it.
- Commit or push only when asked; GSD workflows are the exception for commits (see Planning). Never commit `target/`, `.claude/settings.local.json`, certificates (`*.pfx`) or signing keys; signing reads them from environment variables (D-025).