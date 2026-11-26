---
paths:
  - "crates/sucrose-platform/**"
  - "crates/sucrose-sysinfo/**"
  - "crates/sucrose-engine*/**"
---

# Platform and FFI crates

- These are the only crates where `unsafe` is allowed (D-011): `sucrose-platform`, the `sucrose-engine*` crates and, from D-039, the system data provider crate `sucrose-sysinfo`. Every `unsafe` block gets a `// SAFETY:` comment stating the invariant the caller upholds. Wrap raw APIs in small safe functions and keep the unsafe surface minimal.
- Use the `windows` crate for Win32/COM/WinRT; do not hand-write declarations it already provides.
- Engine libraries are bundled from `cargo xtask fetch-engines` (D-028). libmpv and libVLC are loaded at runtime with `libloading` and never linked at build time (D-013). CEF's `libcef.dll` is a delay-loaded import, so only a process that calls CEF maps it (D-036). The WebView2 loader is linked statically (D-013). A missing or unloadable library is a normal state: report it with a repair hint, don't panic.
- Every OS feature is exposed through the `Platform` trait and reported in `Capabilities`. Implement Windows first, but keep macOS and Linux compiling (stubs returning "unsupported" are fine).
- Desktop attachment on Windows must handle the Windows 11 24H2 Progman/WorkerW hierarchy, explorer.exe restarts (`TaskbarCreated`), monitor and DPI changes.
- When porting from Skylark.Wing, port only the behavior Sucrose uses (`docs/PORTING.md`) and record OS quirks in `docs/FEATURES.md`.
