//! Renderer mode (`sucrose --render <slot>`): connects to the service, receives its assignment and keeps a surface
//! behind the desktop icons until it is told to stop.

use std::{process::ExitCode, thread, time::Duration};

use anyhow::{Context as _, bail};
use sucrose_ipc::{
    Connection, IpcError, connect,
    protocol::{Assignment, PROTOCOL_VERSION, ToRenderer, ToService},
};
use sucrose_platform::{DesktopSurface, DpiAwareness, EventLoop, RectPx};

/// How often and how long the renderer retries while the service is still opening its endpoint.
const CONNECT_ATTEMPTS: u32 = 20;
const CONNECT_PAUSE: Duration = Duration::from_millis(100);

/// Exit code for "the desktop surface could not be attached", reported to the service like any other start failure.
const EXIT_START_FAILURE: u8 = 4;

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

    event_loop.run();
    drop(surface);
    tracing::info!("the renderer stopped");
    Ok(ExitCode::SUCCESS)
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
