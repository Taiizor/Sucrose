---
paths:
  - "crates/**/*.rs"
  - "apps/**/*.rs"
  - "xtask/**/*.rs"
  - "exp/GPUI/**/*.rs"
---

# Rust code

- Errors: `thiserror` enums in library crates, `anyhow` in binaries. No `unwrap()`/`expect()` on I/O, parsing, IPC or FFI results; `expect` is fine for true invariants with a message that says why.
- Logging: `tracing` macros with structured fields. No `println!`/`eprintln!` outside `xtask` and tests.
- Public items of library crates get a short doc comment saying what they are for, not how.
- Keep `sucrose-core` free of OS-specific code. If you need `#[cfg(target_os)]` outside `sucrose-platform` or an engine crate, the code is in the wrong crate.
- Prefer plain threads + `crossbeam-channel` in the service; never block the GPUI thread with I/O or decoding (D-012).
- Tests: unit tests next to the code; format compatibility tests use fixtures under the crate's `tests/fixtures/`, taken from real files with personal data removed.
- Before finishing: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
