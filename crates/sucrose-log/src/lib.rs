//! Logging setup shared by Sucrose binaries: daily rolling files in the log folder, plus stderr in debug builds.

use sucrose_core::paths;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Environment variable with the log filter (same syntax as `RUST_LOG`); defaults to `info`.
pub const LOG_FILTER_ENV: &str = "SUCROSE_LOG";

/// Log files older than this many days are deleted.
const KEEP_DAYS: usize = 14;

/// Keeps the background log writer alive; pending lines are flushed when it is dropped.
#[must_use = "logging stops when the guard is dropped"]
pub struct LogGuard {
    _worker: WorkerGuard,
}

/// Why logging could not be started.
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("the OS reports no data directory for log files")]
    NoDataDir,
    #[error("cannot create the log folder: {0}")]
    CreateDir(#[from] std::io::Error),
    #[error("cannot create the log file: {0}")]
    Appender(#[from] rolling::InitError),
    #[error("a global logger is already installed: {0}")]
    AlreadyInitialized(#[from] tracing_subscriber::util::TryInitError),
}

/// Starts logging for `app` (for example "Portal") into `<data>/Log/<app>.<date>.log`.
pub fn init(app: &str) -> Result<LogGuard, LogError> {
    let dir = paths::log_dir().ok_or(LogError::NoDataDir)?;
    // Created up front: the appender prunes old files before it creates the folder itself.
    std::fs::create_dir_all(&dir)?;
    let appender = rolling::Builder::new()
        .rotation(rolling::Rotation::DAILY)
        .filename_prefix(app)
        .filename_suffix("log")
        .max_log_files(KEEP_DAYS)
        .build(dir)?;
    let (writer, worker) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_env(LOG_FILTER_ENV).unwrap_or_else(|_| EnvFilter::new("info"));
    let file = fmt::layer().with_writer(writer).with_ansi(false);
    let stderr = cfg!(debug_assertions).then(|| fmt::layer().with_writer(std::io::stderr));
    tracing_subscriber::registry()
        .with(filter)
        .with(file)
        .with(stderr)
        .try_init()?;

    Ok(LogGuard { _worker: worker })
}
