//! Renderer mode (`sucrose --render <slot>`): connects to the service, receives its assignment and keeps a surface
//! behind the desktop icons until it is told to stop.

use std::{path::Path, process::ExitCode, sync::Arc, thread, time::Duration};

use anyhow::{Context as _, bail};
use sucrose_engine_mpv::{
    loader::{self, LoadError},
    player::{Player, PlayerError, embedded_options},
};
use sucrose_ipc::{
    Connection, IpcError, connect,
    protocol::{Assignment, PROTOCOL_VERSION, ToRenderer, ToService},
};
use sucrose_platform::{DesktopSurface, DpiAwareness, EventLoop, Flow, LoopEvent, RectPx};

/// How often and how long the renderer retries while the service is still opening its endpoint.
const CONNECT_ATTEMPTS: u32 = 20;
const CONNECT_PAUSE: Duration = Duration::from_millis(100);

/// Exit code for "the desktop surface could not be attached", reported to the service like any other start failure.
const EXIT_START_FAILURE: u8 = 4;

/// Exit code for "libmpv is missing or cannot be loaded" (D-20).
const EXIT_ENGINE_MISSING: u8 = 2;

/// Runs the renderer for `slot` until the service closes the connection or asks it to stop.
pub fn run(slot: u8) -> anyhow::Result<ExitCode> {
    // Before any window or thread exists: engine threads inherit the process default (A15).
    let awareness = sucrose_platform::init_dpi_awareness();
    if awareness == DpiAwareness::PerMonitorV2 {
        tracing::info!(?awareness, "DPI awareness");
    } else {
        tracing::error!(
            ?awareness,
            "the renderer is not Per-Monitor-V2 aware, wallpaper rectangles may be scaled"
        );
    }

    let endpoint = crate::service::endpoint()?;
    let mut connection = connect_with_retry(&endpoint.name)?;
    connection.send(&ToService::Hello {
        protocol_version: PROTOCOL_VERSION,
        pid: std::process::id(),
        slot,
    })?;
    tracing::info!(slot, protocol_version = PROTOCOL_VERSION, "Hello sent to the service");

    let Some(assignment) = wait_for_assignment(&mut connection)? else {
        return Ok(ExitCode::SUCCESS);
    };
    let [left, top, right, bottom] = assignment.rect;
    let rect = RectPx {
        left,
        top,
        right,
        bottom,
    };
    tracing::info!(slot = assignment.slot, monitor = %assignment.monitor, ?rect, "assignment received");

    let event_loop = EventLoop::new().context("cannot prepare the message loop")?;
    let surface = match DesktopSurface::attach(rect) {
        Ok(surface) => surface,
        Err(error) => {
            tracing::error!(%error, "cannot attach the desktop surface");
            return Ok(ExitCode::from(EXIT_START_FAILURE));
        }
    };

    // Opened here, on the thread that pumps the surface's messages: mpv hooks that thread to resize its child.
    let player = match start_engine(surface.native_handle(), Path::new(&assignment.source)) {
        Ok(player) => player,
        Err(error) => {
            error.log();
            return Ok(ExitCode::from(error.exit_code()));
        }
    };

    // The reader thread only waits for the end of the session; the loop below must stay on this thread because the
    // surface was created here.
    let proxy = event_loop.proxy();
    thread::Builder::new()
        .name("ipc-reader".to_owned())
        .spawn(move || {
            wait_for_close(&mut connection);
            proxy.quit();
        })
        .context("cannot start the reader thread")?;

    // The renderer has nothing to drain on a wake; it stops when the reader thread asks it to.
    event_loop.run(|event| match event {
        LoopEvent::Wake => Flow::Continue,
        LoopEvent::Quit => Flow::Exit,
    });
    // The player first: its child window lives inside the surface.
    drop(player);
    drop(surface);
    tracing::info!("the renderer stopped");
    Ok(ExitCode::SUCCESS)
}

/// Why the engine did not start. Every variant is a value with one log line and one exit code, never a panic (D-20).
#[derive(Debug)]
enum EngineError {
    /// libmpv is missing or cannot be loaded.
    Load(LoadError),
    /// libmpv loaded but the player could not be created or could not start the file.
    Player(PlayerError),
}

impl EngineError {
    /// The exit code the service reads as the reason the renderer ended.
    fn exit_code(&self) -> u8 {
        match self {
            Self::Load(_) => EXIT_ENGINE_MISSING,
            Self::Player(_) => EXIT_START_FAILURE,
        }
    }

    /// Logs the failure once, at error level, with the repair hint when there is one.
    fn log(&self) {
        match self {
            Self::Load(error) => tracing::error!(%error, hint = error.hint(), "libmpv cannot be loaded"),
            Self::Player(error) => tracing::error!(%error, "cannot start the player"),
        }
    }
}

/// Loads the bundled libmpv (only the renderer does, D-17) and starts the player embedded in the window `wid`.
fn start_engine(wid: u64, source: &Path) -> Result<Player, EngineError> {
    let mpv = Arc::new(loader::load_default().map_err(EngineError::Load)?);
    Player::open(mpv, wid, source, &embedded_options()).map_err(EngineError::Player)
}

/// Waits for the service's answer to the Hello: `Some` for a job, `None` when the service asks to stop.
fn wait_for_assignment(connection: &mut Connection) -> anyhow::Result<Option<Assignment>> {
    match connection.recv::<ToRenderer>() {
        Ok(ToRenderer::Assign(assignment)) => Ok(Some(assignment)),
        Ok(ToRenderer::Close) => {
            tracing::info!("the service asked this renderer to close");
            Ok(None)
        }
        Err(IpcError::Closed) => {
            tracing::info!("the service closed the connection");
            Ok(None)
        }
        Err(error) => bail!("the service sent no valid assignment: {error}"),
    }
}

/// Blocks until the service asks the renderer to stop or the connection ends.
fn wait_for_close(connection: &mut Connection) {
    loop {
        match connection.recv::<ToRenderer>() {
            Ok(ToRenderer::Close) => {
                tracing::info!("the service asked this renderer to close");
                return;
            }
            // A second assignment is not part of this plan; later plans act on it.
            Ok(ToRenderer::Assign(_)) => tracing::warn!("ignoring an unexpected second assignment"),
            Err(IpcError::Closed) => {
                tracing::info!("the service closed the connection");
                return;
            }
            Err(error) => {
                tracing::warn!(%error, "the service channel failed");
                return;
            }
        }
    }
}

/// Connects to the service, retrying while it starts.
fn connect_with_retry(endpoint: &str) -> Result<Connection, IpcError> {
    let mut attempt = 1;
    loop {
        match connect(endpoint) {
            Ok(connection) => return Ok(connection),
            Err(error) if attempt >= CONNECT_ATTEMPTS => return Err(error),
            Err(error) => {
                tracing::debug!(attempt, %error, "the service is not reachable yet");
                attempt += 1;
                thread::sleep(CONNECT_PAUSE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_engine_exits_with_code_2() {
        let error = EngineError::Load(LoadError::NotFound { searched: Vec::new() });
        assert_eq!(error.exit_code(), EXIT_ENGINE_MISSING);
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn a_player_failure_exits_with_code_4() {
        let error = EngineError::Player(PlayerError::CreateFailed);
        assert_eq!(error.exit_code(), EXIT_START_FAILURE);
        assert_eq!(error.exit_code(), 4);
    }

    #[test]
    fn logging_a_failure_does_not_panic() {
        EngineError::Load(LoadError::NotFound { searched: Vec::new() }).log();
        EngineError::Player(PlayerError::CreateFailed).log();
    }
}
