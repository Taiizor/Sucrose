//! The profile allocator of web wallpapers (D-33, D-35): which browser profile folder a renderer's engine may use.
//!
//! The service chooses which instance of an engine keeps its web data (localStorage, cookies) in
//! `<data_dir>/Web/<engine>`: the renderer of the primary monitor (D-33 as amended by 04-37 D9). Only that instance asks
//! for the persistent profile, and the OS file lock `.sucrose-profile.lock` in that folder guards that no two instances
//! ever share it: an owner that finds the lock held (a renderer that is still ending) waits a moment and then runs
//! ephemeral. Every other instance goes straight to an ephemeral profile under the cache folder that is emptied when it
//! is acquired and never kept across runs. A crashed renderer's lock goes with its process.

use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

/// The name of the lock file inside the persistent profile folder.
pub const LOCK_FILE: &str = ".sucrose-profile.lock";

/// How long an instance waits at its start for a profile folder another instance is still letting go of.
pub const START_WAIT: Duration = Duration::from_secs(1);

/// How often a wait for a lock is tried again.
const RETRY_EVERY: Duration = Duration::from_millis(50);

/// Which kind of profile a lease holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Gives an instance the profile it `wanted`: the persistent profile of `engine` when it asked for it and no other
/// running instance holds it (waiting up to `wait` for a holder that is ending), otherwise an emptied ephemeral profile
/// for `slot`. An instance that asks for [`ProfileKind::Ephemeral`] never touches the persistent folder or its lock.
///
/// # Errors
/// Any I/O error of creating, locking or emptying the folders, and `InvalidInput` for an engine name that is empty or
/// contains a path separator.
pub fn acquire(
    web_dir: &Path,
    engine: &str,
    ephemeral_root: &Path,
    slot: u8,
    wanted: ProfileKind,
    wait: Duration,
) -> io::Result<ProfileLease> {
    if engine.is_empty() || engine == ".." || engine == "." || engine.contains(['/', '\\']) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the engine name of a profile is a single folder name",
        ));
    }
    if wanted == ProfileKind::Persistent
        && let Some(lease) = try_persistent(web_dir, engine, wait)?
    {
        return Ok(lease);
    }
    let path = ephemeral_root.join(engine).join(slot.to_string());
    match fs::remove_dir_all(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs::create_dir_all(&path)?;
    Ok(ProfileLease {
        kind: ProfileKind::Ephemeral,
        path,
        _lock: None,
    })
}

/// Locks the persistent folder of `engine`, trying again every [`RETRY_EVERY`] until `wait` has passed; `None` when
/// another instance still holds it.
fn try_persistent(web_dir: &Path, engine: &str, wait: Duration) -> io::Result<Option<ProfileLease>> {
    let persistent = web_dir.join(engine);
    fs::create_dir_all(&persistent)?;
    // The OS releases the lock when the handle closes, which includes the end of a crashed process, so no stale lock
    // can keep the persistent profile away from the next start.
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(persistent.join(LOCK_FILE))?;
    let deadline = Instant::now() + wait;
    loop {
        match lock.try_lock() {
            Ok(()) => {
                return Ok(Some(ProfileLease {
                    kind: ProfileKind::Persistent,
                    path: persistent,
                    _lock: Some(lock),
                }));
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(error)) => return Err(error),
        }
        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }
        thread::sleep(RETRY_EVERY.min(deadline - now));
    }
}
