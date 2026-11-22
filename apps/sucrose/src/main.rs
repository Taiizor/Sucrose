//! The Sucrose service (D-008). For now it only starts, logs what it found and exits.

fn main() -> anyhow::Result<()> {
    let _log = sucrose_log::init("Service")?;
    let platform = sucrose_platform::current();
    tracing::info!(
        version = sucrose_core::VERSION,
        os = platform.name(),
        capabilities = ?platform.capabilities(),
        "service started"
    );
    tracing::info!("nothing to do yet (M0), exiting");
    Ok(())
}
