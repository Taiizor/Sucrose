//! System data for web wallpapers: one provider per payload group of the legacy app, each returning the group as a JSON
//! object with the legacy keys and value types (docs/legacy/services.md section 1.2). The service's data hub calls a
//! provider only while a running page needs its group (D-24, D-008). Per-OS code lives inside the providers behind
//! `cfg`; what an OS cannot give is [`SysInfoError::Unsupported`], never a faked value (D-002).

// D-039: this crate is a platform-provider crate like sucrose-platform (D-011): the providers call OS APIs, and each
// unsafe block carries a SAFETY comment, which clippy enforces. `sucrose-core` stays free of OS code.
#![allow(unsafe_code)]
#![warn(clippy::undocumented_unsafe_blocks)]

mod date;
mod memory;
mod processor;

pub use date::date_snapshot;
pub use memory::{memory_snapshot, memory_unavailable};
pub use processor::{ProcessorSampler, processor_unavailable};

/// Why a group of system data could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SysInfoError {
    /// This OS (or this build) has no provider for the data.
    #[error("{0} is not available on this OS")]
    Unsupported(&'static str),
    /// The OS refused the call.
    #[error("{call} failed: {message}")]
    Os {
        /// The OS call that failed.
        call: &'static str,
        /// What the OS said.
        message: String,
    },
}
