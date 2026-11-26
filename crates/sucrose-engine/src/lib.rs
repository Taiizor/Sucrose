//! Which engine plays a wallpaper: engine identities (D-10), what the machine can run (the availability report) and
//! the pure choice with its fallback order (D-11), and the engine traits every engine implements (ENG-03). No OS code
//! and no unsafe: it builds and tests on every OS.

pub mod choice;
pub mod engine;
pub mod sync;

pub use choice::{Availability, Choice, EngineId, Hint, Unsupported, choose_engine, fallback_order};
pub use engine::{Engine, EngineClock, EngineError, Started, SurfacePump};

/// How far a paused wallpaper is put to rest (D-22). Light resumes instantly; Heavy also frees resources (hidden
/// surface, frozen page, released decoder) and resumes more slowly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseLevel {
    /// Stop advancing but keep everything ready, so resuming is instant.
    Light,
    /// Stop advancing and also free what the engine can free, so resuming takes a moment.
    Heavy,
}

impl From<sucrose_core::settings::PauseKind> for PauseLevel {
    /// The migrated `PausePerformanceType` is the level every engine's pause receives.
    fn from(kind: sucrose_core::settings::PauseKind) -> Self {
        match kind {
            sucrose_core::settings::PauseKind::Light => Self::Light,
            sucrose_core::settings::PauseKind::Heavy => Self::Heavy,
        }
    }
}
