---
paths:
  - "crates/sucrose-platform/**"
  - "crates/sucrose-engine*/**"
---

# Platform and FFI crates

- These are the only crates where `unsafe` is allowed (D-011). Every `unsafe` block gets a `// SAFETY:` comment stating the invariant the caller upholds. Wrap raw APIs in small safe functions and keep the unsafe surface minimal.
- Use the `windows` crate for Win32/COM/WinRT; do not hand-write declarations it already provides.
- Engine libraries (libmpv, libVLC, CEF, WebView2 loader) are loaded at runtime with `libloading` and never linked at build time (D-013, D-028). Bundled binaries come from `cargo xtask fetch-engines`. A missing or unloadable library is a normal state: report it with a repair hint, don't panic.
- Every OS feature is exposed through the `Platform` trait and reported in `Capabilities`. Implement Windows first, but keep macOS and Linux compiling (stubs returning "unsupported" are fine).
- Desktop attachment on Windows must handle the Windows 11 24H2 Progman/WorkerW hierarchy, explorer.exe restarts (`TaskbarCreated`), monitor and DPI changes.
- When porting from Skylark.Wing, port only the behavior Sucrose uses (`docs/PORTING.md`) and record OS quirks in `docs/FEATURES.md`.
