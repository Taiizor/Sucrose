//! Renderer mode (`sucrose --render <slot>`): connects to the service and says who it is.

use std::{thread, time::Duration};

use sucrose_ipc::{
    Connection, IpcError, connect,
    protocol::{PROTOCOL_VERSION, ToRenderer, ToService},
};

/// How often and how long the renderer retries while the service is still opening its endpoint.
const CONNECT_ATTEMPTS: u32 = 20;
const CONNECT_PAUSE: Duration = Duration::from_millis(100);

/// Runs the renderer for `slot` until the service closes the connection or asks it to stop.
pub fn run(slot: u8) -> anyhow::Result<()> {
    let endpoint = crate::service::endpoint();
    let mut connection = connect_with_retry(&endpoint)?;
    connection.send(&ToService::Hello {
        protocol_version: PROTOCOL_VERSION,
        pid: std::process::id(),
        slot,
    })?;
    tracing::info!(slot, protocol_version = PROTOCOL_VERSION, "Hello sent to the service");

    // Close is the only message today, so the loop returns on its first pass; later plans add messages that continue it.
    #[allow(clippy::never_loop)]
    loop {
        match connection.recv::<ToRenderer>() {
            Ok(ToRenderer::Close) => {
                tracing::info!("the service asked this renderer to close");
                return Ok(());
            }
            Err(IpcError::Closed) => {
                tracing::info!("the service closed the connection");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
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
