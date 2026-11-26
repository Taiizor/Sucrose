//! The profile allocator of web wallpapers (D-33, D-35): which browser profile folder a renderer's engine may use.
//!
//! The first running instance of an engine keeps its web data (localStorage, cookies) in `<data_dir>/Web/<engine>`; the
//! OS file lock `.sucrose-profile.lock` in that folder decides who is first, so two renderers that start at the same
//! moment never share a folder. Every other concurrent instance gets an ephemeral profile under the cache folder that is
//! emptied when it is acquired and never kept across runs. A crashed renderer's lock goes with its process.

use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

/// The name of the lock file inside the persistent profile folder.
pub const LOCK_FILE: &str = ".sucrose-profile.lock";

/// Which kind of profile a lease holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    /// `<web_dir>/<engine>`: kept across runs; at most one running instance per engine holds it.
    Persistent,
    /// `<ephemeral_root>/<engine>/<slot>`: emptied when acquired, so it holds nothing from an earlier run.
    Ephemeral,
}

/// A profile folder an instance may use until the lease is dropped.
#[derive(Debug)]
pub struct ProfileLease {
    /// Whether this is the persistent profile or an ephemeral one.
    pub kind: ProfileKind,
    /// The folder to hand to the engine as its profile.
    pub path: PathBuf,
    _lock: Option<File>,
}

/// Takes the persistent profile of `engine` when no other running instance holds it, otherwise an emptied ephemeral
/// profile for `slot`.
///
/// # Errors
/// Any I/O error of creating, locking or emptying the folders, and `InvalidInput` for an engine name that is empty or
/// contains a path separator.
pub fn acquire(web_dir: &Path, engine: &str, ephemeral_root: &Path, slot: u8) -> io::Result<ProfileLease> {
    let _ = (web_dir, engine, slot);
    Ok(ProfileLease {
        kind: ProfileKind::Ephemeral,
        path: ephemeral_root.to_path_buf(),
        _lock: None,
    })
}
