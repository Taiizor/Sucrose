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

/// How often a rename is attempted while another program holds the target for a moment.
const ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_millis(50);

/// Where the settings of a run came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOutcome {
    /// `Sucrose.json` was read.
    Loaded,
    /// `Sucrose.json` was missing or corrupt and `Sucrose.json.bak` took its place.
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
    settings: Settings,
}

impl Store {
    /// Opens the store in `setting_dir` (created when missing). `legacy` is the legacy `Setting` folder and the C#
    /// data folder; the legacy files are read, never written, and only when no settings file exists.
    pub fn open(setting_dir: &Path, legacy: Option<(&Path, &Path)>) -> Result<(Self, LoadOutcome), StoreError> {
        fs::create_dir_all(setting_dir)?;
        let path = setting_dir.join(SETTINGS_FILE);
        if let Some(settings) = read_good(&path) {
            return Ok((Self { path, settings }, LoadOutcome::Loaded));
        }
        let (settings, outcome) = match legacy {
            Some((legacy_setting_dir, legacy_root)) if legacy_setting_dir.is_dir() => {
                (migrate(legacy_setting_dir, legacy_root), LoadOutcome::Migrated)
            }
            _ => (Settings::default(), LoadOutcome::Defaults),
        };
        let store = Self { path, settings };
        store.persist()?;
        Ok((store, outcome))
    }

    /// The settings of this run.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Applies `change` and saves the result. When the save fails the previous settings stay in memory.
    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) -> Result<(), StoreError> {
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
        write_atomic(&self.path, &bytes)?;
        Ok(())
    }
}

/// The settings in `path` when the file exists and parses.
fn read_good(path: &Path) -> Option<Settings> {
    match fs::read(path) {
        Ok(bytes) => match Settings::from_slice(&bytes) {
            Ok(settings) => Some(settings),
            Err(error) => {
                tracing::warn!(%error, "the settings file is corrupt");
                None
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            tracing::warn!(%error, "the settings file cannot be read");
            None
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
