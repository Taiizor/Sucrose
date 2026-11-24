//! The atomic store of `Setting/Sucrose.json` (D-01, D-02, ARCH-04).
//!
//! A write goes to a temporary file in the same folder, is flushed to disk and then renamed over the target, so a kill
//! at any moment leaves the old file or the new one, never half of one. The service is the only writer.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crate::{
    migrate::migrate,
    model::{SETTINGS_VERSION, Settings},
};

/// Name of the settings file inside the `Setting` folder.
pub const SETTINGS_FILE: &str = "Sucrose.json";
/// Name of the copy of the previous settings file; it does not end in `.json`, so the C# Reset, Import and Export
/// tools that glob `*.json` leave it alone.
pub const BACKUP_FILE: &str = "Sucrose.json.bak";
/// Name a corrupt `Sucrose.json` is moved to before the store writes over it, so a typo in a hand-edited file or a
/// file a build cannot parse is never lost; like the backup it does not end in `.json`.
pub const CORRUPT_FILE: &str = "Sucrose.json.corrupt";

/// How often a rename or a read is attempted while another program holds the target for a moment.
const ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(50);

/// Where the settings of a run came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOutcome {
    /// `Sucrose.json` was read.
    Loaded,
    /// `Sucrose.json` was missing or corrupt and `Sucrose.json.bak` took its place. A file that merely cannot be read
    /// (locked by another program, access denied) is never replaced: the store refuses to open instead.
    RestoredFromBackup,
    /// Neither file existed and the legacy files were migrated.
    Migrated,
    /// Neither file existed and there was nothing to migrate: the legacy defaults.
    Defaults,
}

/// Why the store could not read or write the settings.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The folder or a file could not be read or written.
    #[error("the settings file cannot be read or written: {0}")]
    Io(#[from] io::Error),
    /// The file was written by a newer build and is kept as it is.
    #[error("the settings file has format version {found}, newer than this build; it is kept unchanged")]
    NewerVersion {
        /// The version found in the file.
        found: u32,
    },
}

/// The settings of a run and the file they are saved in.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    backup: PathBuf,
    settings: Settings,
    /// The format version of a file written by a newer build; the store never writes over such a file.
    newer: Option<u32>,
}

/// What reading one of the two files gave.
enum Read {
    Missing,
    Corrupt,
    /// The file exists but cannot be read (another program holds it, access denied), even after a few attempts. It
    /// says nothing about the content, so it must never lead to a restore or a migration over the file.
    Unreadable(io::Error),
    /// The parsed settings and the exact bytes they came from.
    Good(Box<Settings>, Vec<u8>),
}

impl Store {
    /// Opens the store in `setting_dir` (created when missing). The load order is the settings file, then its `.bak`,
    /// then the migration of the legacy files (`legacy`: the legacy `Setting` folder and the C# data folder); the legacy
    /// files are read, never written, and only when neither settings file exists in a good state. A file that exists but
    /// cannot be read (locked, access denied) is never replaced by the backup, the migration or the defaults: the store
    /// refuses with [`StoreError::Io`] and writes nothing. Temporary files a killed writer left behind are removed.
    pub fn open(setting_dir: &Path, legacy: Option<(&Path, &Path)>) -> Result<(Self, LoadOutcome), StoreError> {
        fs::create_dir_all(setting_dir)?;
        remove_stale_temporaries(setting_dir);
        let path = setting_dir.join(SETTINGS_FILE);
        let backup = setting_dir.join(BACKUP_FILE);

        let main = read_file(&path);
        let main_corrupt = matches!(main, Read::Corrupt);
        match main {
            Read::Good(settings, _) => {
                let newer = newer_version(&settings);
                let settings = *settings;
                return Ok((
                    Self {
                        path,
                        backup,
                        settings,
                        newer,
                    },
                    LoadOutcome::Loaded,
                ));
            }
            Read::Unreadable(error) => return Err(StoreError::Io(error)),
            Read::Missing | Read::Corrupt => {}
        }
        match read_file(&backup) {
            Read::Good(settings, bytes) => {
                let newer = newer_version(&settings);
                let settings = *settings;
                if newer.is_none() {
                    if main_corrupt {
                        keep_corrupt_aside(&path);
                    }
                    if let Err(error) = write_atomic(&path, &bytes) {
                        tracing::warn!(%error, "the settings file cannot be restored from its backup, using the backup in memory");
                    }
                }
                tracing::info!("the settings file was missing or corrupt and is restored from its backup");
                return Ok((
                    Self {
                        path,
                        backup,
                        settings,
                        newer,
                    },
                    LoadOutcome::RestoredFromBackup,
                ));
            }
            Read::Unreadable(error) => return Err(StoreError::Io(error)),
            Read::Missing | Read::Corrupt => {}
        }
        let (settings, outcome) = match legacy {
            Some((legacy_setting_dir, legacy_root)) if legacy_setting_dir.is_dir() => {
                (migrate(legacy_setting_dir, legacy_root), LoadOutcome::Migrated)
            }
            _ => (Settings::default(), LoadOutcome::Defaults),
        };
        let store = Self {
            path,
            backup,
            settings,
            newer: None,
        };
        store.persist()?;
        Ok((store, outcome))
    }

    /// The settings of this run.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Applies `change` and saves the result; the file it replaces becomes the `.bak`. When the save fails the previous
    /// settings stay in memory, and a file written by a newer build is refused with [`StoreError::NewerVersion`].
    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) -> Result<(), StoreError> {
        if let Some(found) = self.newer {
            return Err(StoreError::NewerVersion { found });
        }
        let previous = self.settings.clone();
        change(&mut self.settings);
        self.settings.version = SETTINGS_VERSION;
        if let Err(error) = self.persist() {
            self.settings = previous;
            return Err(error);
        }
        Ok(())
    }

    fn persist(&self) -> Result<(), StoreError> {
        let bytes = self.settings.to_bytes().map_err(io::Error::other)?;
        // The file about to be replaced is kept as the backup, but only when it is good: a corrupt file must not
        // push out a good backup, and is moved aside so the write does not destroy it. A failed backup does not stop
        // the save.
        match read_file(&self.path) {
            Read::Good(_, previous) => {
                if let Err(error) = write_atomic(&self.backup, &previous) {
                    tracing::warn!(%error, "the settings backup cannot be written");
                }
            }
            Read::Corrupt => keep_corrupt_aside(&self.path),
            Read::Missing | Read::Unreadable(_) => {}
        }
        write_atomic(&self.path, &bytes)?;
        Ok(())
    }
}

/// Moves a corrupt settings file to [`CORRUPT_FILE`] (replacing an older one) before the store writes over it. Best
/// effort: when the move fails the save goes on, as the file is unusable for the store anyway.
fn keep_corrupt_aside(path: &Path) {
    let aside = path.with_file_name(CORRUPT_FILE);
    match retry_busy(|| fs::rename(path, &aside)) {
        Ok(()) => tracing::warn!("the corrupt settings file is kept aside before it is replaced"),
        Err(error) => tracing::warn!(%error, "the corrupt settings file cannot be kept aside"),
    }
}

/// The version of a file written by a newer build, logged once when the store opens it.
fn newer_version(settings: &Settings) -> Option<u32> {
    let found = settings.version;
    (found > SETTINGS_VERSION).then(|| {
        tracing::warn!(
            found,
            supported = SETTINGS_VERSION,
            "the settings file is from a newer build and stays unchanged"
        );
        found
    })
}

/// Reads `path`, trying again a few times when the read fails for any reason but "not found" (a sharing violation while a
/// virus scanner or the C# app holds the file, as `legacy::read` does).
fn read_retrying(mut read: impl FnMut() -> io::Result<Vec<u8>>) -> io::Result<Vec<u8>> {
    let mut attempt = 1;
    loop {
        match read() {
            Err(error) if error.kind() != io::ErrorKind::NotFound && attempt < ATTEMPTS => {
                attempt += 1;
                thread::sleep(RETRY_DELAY);
            }
            result => return result,
        }
    }
}

fn read_file(path: &Path) -> Read {
    match read_retrying(|| fs::read(path)) {
        Ok(bytes) => match Settings::from_slice(&bytes) {
            Ok(settings) => Read::Good(Box::new(settings), bytes),
            Err(error) => {
                tracing::warn!(%error, "a settings file is corrupt");
                Read::Corrupt
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => Read::Missing,
        Err(error) => {
            // The error kind only: the message of an OS error can carry the path.
            tracing::warn!(kind = ?error.kind(), "a settings file cannot be read and is left as it is");
            Read::Unreadable(error)
        }
    }
}

/// Removes the `Sucrose.json.<pid>.tmp` files a killed writer left in `dir`. Best effort: the service is the only
/// writer, so none of them belongs to a write in progress.
fn remove_stale_temporaries(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let prefix = format!("{SETTINGS_FILE}.");
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(&prefix) && name.ends_with(".tmp") {
            match fs::remove_file(entry.path()) {
                Ok(()) => tracing::debug!("a stale temporary settings file was removed"),
                Err(error) => tracing::debug!(%error, "a stale temporary settings file cannot be removed"),
            }
        }
    }
}

/// Writes `bytes` to `path` so that a reader, or a kill at any moment, sees the old content or the new, never a mix:
/// a temporary file in the same folder, flushed to disk, renamed over the target; the temporary file is removed on
/// every error path.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        Some(_) => Path::new("."),
        None => return Err(io::Error::other("the settings path has no folder")),
    };
    let temporary = dir.join(format!("{SETTINGS_FILE}.{}.tmp", std::process::id()));
    let result = write_temporary(&temporary, bytes).and_then(|()| retry_busy(|| fs::rename(&temporary, path)));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Creates the temporary file and flushes its content to disk. A leftover with the same name (a killed writer whose
/// process id came back) is replaced.
fn write_temporary(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)?;
            OpenOptions::new().write(true).create_new(true).open(path)?
        }
        Err(error) => return Err(error),
    };
    file.write_all(bytes)?;
    file.sync_all()
}

/// Runs `action` again a few times while a virus scanner or an indexer holds the file for a few milliseconds.
fn retry_busy(mut action: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        match action() {
            Err(error) if is_busy(&error) && attempt < ATTEMPTS => {
                attempt += 1;
                thread::sleep(RETRY_DELAY);
            }
            result => return result,
        }
    }
}

fn is_busy(error: &io::Error) -> bool {
    // 32 is ERROR_SHARING_VIOLATION.
    error.kind() == io::ErrorKind::PermissionDenied || error.raw_os_error() == Some(32)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn a_read_that_fails_once_with_a_sharing_violation_is_retried() {
        let calls = Cell::new(0);

        let result = read_retrying(|| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                Err(io::Error::from_raw_os_error(32))
            } else {
                Ok(b"bytes".to_vec())
            }
        });

        assert_eq!(result.expect("the second attempt reads"), b"bytes");
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn a_read_that_keeps_failing_is_tried_attempts_times() {
        let calls = Cell::new(0);

        let result = read_retrying(|| {
            calls.set(calls.get() + 1);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });

        assert!(result.is_err());
        assert_eq!(calls.get(), ATTEMPTS);
    }

    #[test]
    fn a_missing_file_is_not_retried() {
        let calls = Cell::new(0);

        let result = read_retrying(|| {
            calls.set(calls.get() + 1);
            Err(io::Error::from(io::ErrorKind::NotFound))
        });

        assert_eq!(
            result.expect_err("not found stays an error").kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(calls.get(), 1);
    }
}
