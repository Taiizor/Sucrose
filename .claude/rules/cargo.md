---
paths:
  - "**/Cargo.toml"
  - "deny.toml"
  - "rust-toolchain.toml"
---

# Dependencies

- Versions live in the root `[workspace.dependencies]`; member crates use `dep = { workspace = true }`.
- Before adding a crate, check on crates.io that it is maintained (recent release), widely used, and GPL-3.0 compatible; `cargo-deny` enforces licenses and advisories.
- `gpui-kit` is pinned to an exact version (`=x.y.z`); upgrade it deliberately and re-check the UI.
- Do not add: `vlc-rs`, `libmpv-sys`, `single-instance`, `libmpv2` (see D-013), or any crate that links an engine runtime at build time.
- Prefer one crate per job across the workspace (one HTTP client, one image stack, one channel crate).
