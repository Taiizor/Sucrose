//! Which engine plays a wallpaper: engine identities (D-10), what the machine can run (the availability report) and
//! the pure choice with its fallback order (D-11). No OS code and no unsafe: it builds and tests on every OS. The
//! engine trait itself lands with the first two engines that implement it.

pub mod choice;

pub use choice::{Availability, Choice, EngineId, Hint, Unsupported, choose_engine, fallback_order};

/// How far a paused wallpaper is put to rest (D-22). Light resumes instantly; Heavy also frees resources (hidden
/// surface, frozen page, released decoder) and resumes more slowly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseLevel {
    /// Stop advancing but keep everything ready, so resuming is instant.
    Light,
    /// Stop advancing and also free what the engine can free, so resuming takes a moment.
    Heavy,
}
