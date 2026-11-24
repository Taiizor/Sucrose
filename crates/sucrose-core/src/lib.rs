//! Platform-independent building blocks shared by every Sucrose crate.
//!
//! This crate must not contain OS-specific code (see `docs/ARCHITECTURE.md`).

pub mod library;
pub mod paths;
pub mod settings;
pub mod theme;
pub mod version;

/// Product name used for folders, log files and identifiers.
pub const APP_NAME: &str = "Sucrose";

/// Product name shown to users.
pub const APP_LONG_NAME: &str = "Sucrose Wallpaper Engine";

/// Version of the running build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
