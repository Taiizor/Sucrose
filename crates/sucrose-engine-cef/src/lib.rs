//! CEF engine for web wallpapers (D-013, D-32).
//!
//! The pinned CEF dist is linked through `CEF_PATH` and `libcef.dll` is delay-loaded by the executables that host
//! CEF: only renderers and CEF subprocesses ever load it, so the service and the mpv renderers stay without it.

// Engine/FFI crate: allowed to opt out of the workspace `unsafe_code` lint (D-011).
#![allow(unsafe_code)]

/// The CEF version of the pinned dist (`xtask/engines.toml`), without the commit and Chromium suffix.
pub const CEF_VERSION: &str = "156.0.31";
