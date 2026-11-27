//! Built-in GIF renderer (D-10: `VexanaLive` and `XavierLive`).
//!
//! The animation is decoded lazily on a thread of its own, one frame at a time and never the whole animation in memory
//! ([`decode`], pure and the same on every OS). The frames are shown with Direct2D on a child window of the desktop
//! surface ([`present`]), driven from the thread that owns the surface ([`engine`]).

// Engine/FFI crate: allowed to opt out of the workspace `unsafe_code` lint (D-011); only the presenter uses it.
#![allow(unsafe_code)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod decode;
pub mod engine;
pub mod present;

/// Whether this build contains the built-in GIF renderer: only Windows has the Direct2D presenter (D-002).
pub const BUILT: bool = cfg!(windows);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_windows_contains_the_built_in_renderer() {
        assert_eq!(BUILT, cfg!(windows));
    }
}
