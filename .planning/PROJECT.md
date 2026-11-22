# Sucrose (Rust rewrite)

## What This Is

Sucrose is an open-source live wallpaper engine: Video, GIF, Web, URL, YouTube and Application wallpapers, with audio-reactive and system-status data for web wallpapers. The `develop-rust` branch rewrites the Windows-only C# (WPF) app from scratch in Rust for Windows, macOS and Linux, keeping every existing feature and improving them where possible. It serves today's Sucrose users on Windows first, then macOS and Linux users.

## Core Value

An existing user upgrades and loses nothing (themes, store, web wallpapers and their JavaScript API, settings migrated automatically) while the new app uses less memory and CPU than the C# one. Feature parity serves these two goals.

## Requirements

### Validated

- ✓ Cargo workspace at the repo root (`crates/`, `apps/`, `xtask/`) with `[workspace.dependencies]` and `[workspace.lints]`; `src/` and `exp/` excluded (D-011) — M0
- ✓ `rust-toolchain.toml` (1.99.0), `rustfmt.toml`, `clippy.toml`, `deny.toml` (all dependencies pass the license list) — M0
- ✓ CI workflow `.github/workflows/rust.yml`: fmt; clippy `-D warnings` and tests on Windows, Ubuntu and macOS; cargo-deny (checks pass locally on Windows via `cargo xtask ci`; CI has not run yet) — M0
- ✓ `sucrose-log`: daily rolling log files under `Log/`; `SUCROSE_DATA_DIR` and `SUCROSE_LOG` overrides — M0
- ✓ `sucrose-core` (paths) and `sucrose-platform` (trait + stubs for all three OSes) — M0
- ✓ `apps/sucrose` (service stub) and `apps/sucrose-portal` (empty GPUI window, built from scratch) — M0
- ✓ `cargo xtask ci` runs the CI checks locally — M0

### Active

- [ ] CI green on Windows, macOS and Linux with stub platform implementations (needs a push) — M0
- [ ] A video theme plays behind the desktop icons on every monitor on Windows (libmpv), survives explorer restarts and monitor changes, pauses for fullscreen apps; RAM/CPU measured against the C# MpvPlayer + Backgroundog — M1
- [ ] Service with one IPC and single instance, optional modules that do not start when disabled, new versioned settings with atomic writes and automatic migration of legacy `Setting/*.json`; the Portal applies wallpapers and persists settings through the service; autostart — M2
- [ ] Every engine and wallpaper type: WebView2 and CEF with the compatible JavaScript API, built-in GIF/APNG/WebP and OS video, libVLC; URL, YouTube, Application; system data, audio spectrum, media session, input forwarding; display modes including a different wallpaper per monitor; stretch modes; property editing; engine arguments editor; all performance rules — M3
- [ ] Product features: store (Soferity), theme create/edit/import/export/share/report/review, wallpaper cycling, lock screen wallpaper (D-026), display preferences, 25 languages with RTL, Discord Rich Presence, crash reporting and telemetry with the existing opt-outs, own signed installer/updater/uninstaller with migration from the C# install — M4
- [ ] macOS: every `docs/FEATURES.md` row macOS allows; everything else reported through platform capabilities, never faked — M5
- [ ] Linux, X11 first, then Wayland compositors, under the same rule — M6
- [ ] Parity sign-off: every `docs/FEATURES.md` row ported or covered by an accepted decision; C# code removed; docs and `CLAUDE.md` rewritten — M7
- [ ] Legacy problems are fixed, not carried over: TLS validation, verified updates, CEF path traversal, crash reports despite opt-out, non-atomic settings, restart loops, known bugs (`docs/FEATURES.md` → "Legacy problems not to carry over")

### Out of Scope

- GitHub store server, GitHub update source, personal access token, theme report GitHub issue template — Soferity API only (D-020)
- Donate page, advertising settings and `Donate.json` migration — removed (D-023)
- Commandog trampoline and the Pipe/Signal/Transmission channels as contracts — replaced by one IPC (D-008, D-009)
- Bundling or installing WebView2, CEF, libmpv or libVLC — optional and user-installed; Sucrose works with built-in engines alone (D-005, D-006)
- A web runtime in the core or the UI — GPUI via gpui-kit (D-004)
- Building on `exp/GPUI/Sucrose.Portal` — throwaway experiment, not the base of `apps/sucrose-portal`
- Faking what an OS cannot do (for example Application wallpapers on macOS or Wayland) — reported through platform capabilities (D-002)
- Changing `src/` (legacy C#) — read-only reference until it is removed at M7 (D-016)

## Context

- **Legacy reference:** the C# app in `src/` (read-only). Exact behavior, settings keys, file formats and the JavaScript API are in `docs/legacy/` (`portal.md`, `services.md`, `engines.md`, `formats.md`, `lock-screen.md`). The lock screen exists only on branch `lock-screen-native` (112de37f).
- **Living docs:** `docs/DECISIONS.md` (decision log, append-only), `docs/FEATURES.md` (parity checklist with per-OS support), `docs/ARCHITECTURE.md` (target processes, crates, traits), `docs/PORTING.md` (porting procedure, NuGet → crate mapping), `docs/GPUI.md` (UI notes). `.planning/` holds the roadmap and phase work; `.planning/intel/` is the synthesized snapshot these were derived from.
- **Process model (D-008, D-009):** `sucrose` service (always on while a wallpaper is active), `sucrose --render` (one renderer per wallpaper surface, crash-isolated), `sucrose-portal` (only while its window is open); all talk over one local-socket IPC.
- **UI:** GPUI via `gpui-kit 0.10` (pins the Zed snapshot `gpui-pre 0.3.12`). Prototype numbers: window in ~0.3 s, ~90 MB on the settings page, ~142 MB with 30 thumbnails; the real Portal needs a bounded thumbnail cache.
- **Environment:** the maintainer works on Windows 11 from WSL2 and PowerShell. `cargo` in WSL is the Windows toolchain; Linux builds run in Docker or CI; macOS builds run in CI and need a real Mac for manual testing.
- **Open questions:** store and update protocol deferred to the API rewrite, v8 endpoints until then (D-022); update signing key scheme (Ed25519 proposed, D-019/D-025); Linux package formats (M6); Apple Developer ID; lock screen mechanism before the M4 item (D-026).

## Constraints

- **Compatibility**: themes (`SucroseInfo.json`, `SucroseProperties.json`, `SucroseCompatible.json`, archives) and the web JavaScript API stay compatible; legacy settings are migrated automatically; proven with fixture tests — D-007
- **Platforms**: Windows, macOS and Linux from the first commit, parity order Windows → macOS → Linux; OS-specific code only in `sucrose-platform` and engine crates — D-002
- **Tech stack**: Rust edition 2024 on the pinned stable toolchain; `rustfmt` and `clippy -D warnings`; C/C++ only through FFI — D-001
- **UI**: GPUI via an exactly pinned `gpui-kit`; one design language on every OS — D-004
- **Engines**: optional, user-installed, detected and loaded at runtime with install hints; own `bindgen` bindings loaded with `libloading` — D-005, D-013
- **Safety**: `unsafe` only in `sucrose-platform` and engine/FFI crates, each block with a `// SAFETY:` comment — D-011
- **Performance**: optional modules never start when disabled (D-008); user-visible changes are measured (RAM/CPU, release build) and notable numbers recorded
- **Dependencies**: port the capabilities Sucrose uses, not whole packages (D-010); licenses must pass `cargo-deny` (GPL-3.0 compatible)
- **Security**: TLS validation on, installers and updates verified before they run, crash reports respect the opt-out — D-017, D-019
- **Signing**: Windows signed with signtool, certificate location from environment variables; macOS unsigned until a Developer ID exists — D-025
- **Defaults**: legacy defaults are kept (`StoreAdult`, `TelemetryData`, `ExceptionData` = true) — D-024
- **Repository**: everything committed works from WSL and PowerShell; automation lives in `xtask` (D-014); no symlinks, no `/mnt/...` paths

## Key Decisions

The authoritative, append-only log is `docs/DECISIONS.md` (D-001 … D-026). The ones that shape the roadmap most:

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rewrite from scratch in Rust (D-001) | Cross-platform, lower RAM/CPU for an always-on app, single native binaries | — Pending |
| Windows, macOS, Linux; parity order Windows → macOS → Linux (D-002) | Existing users and the reference implementation are on Windows | — Pending |
| Keep every feature, improve where possible, record behavior changes (D-003) | Nobody loses a feature on upgrade | — Pending |
| UI is GPUI through gpui-kit (D-004) | No web runtime in the core | — Pending |
| Engines optional and user-installed; built-in GIF and OS video (D-005, D-006) | Sucrose works with no engine installed | — Pending |
| Keep theme/API formats, migrate settings automatically (D-007) | Existing themes, store and web wallpapers keep working | — Pending |
| Modular service, one renderer per wallpaper, Portal only while open (D-008) | Crash isolation, near-idle cost | — Pending |
| One IPC: local sockets with postcard (D-009) | Speed; same endpoint gives single instance | — Pending |
| Own installer, updater and uninstaller (D-019) | Full control, verified updates | — Pending |
| GitHub integrations removed (D-020) | Soferity API only | — Pending |
| Lock screen wallpaper in scope (D-026) | Maintainer decision; mechanism decided before it is built | — Pending |
| Roadmap and phase work move to `.planning/` (GSD); Phase 1–8 = M0–M7, each phase title carries its M label (e.g. "Phase 2 — M1: Video wallpaper on the desktop (Windows)") | One planning system; GSD phases may commit | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-11-22 after initialization*
