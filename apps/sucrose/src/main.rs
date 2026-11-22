//! The Sucrose service (D-008). For now it starts, logs what it found and exits.

use sucrose_config::legacy::{LegacyFile, LibrarySettings, read};
use sucrose_core::{
    paths::{legacy_data_dir, legacy_setting_dir},
    theme::read_info,
};

fn main() -> anyhow::Result<()> {
    let _log = sucrose_log::init("Service")?;
    let platform = sucrose_platform::current();
    tracing::info!(
        version = sucrose_core::VERSION,
        os = platform.name(),
        capabilities = ?platform.capabilities(),
        "service started"
    );
    log_selected_theme();
    tracing::info!("nothing to do yet (M0), exiting");
    Ok(())
}

/// Logs the theme selected in the C# install, read-only from its real folder (D-008).
fn log_selected_theme() {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return;
    };
    let properties = read(&setting_dir, LegacyFile::Library);
    let library = LibrarySettings::from_properties(&properties, &root);
    if library.selected.is_empty() {
        tracing::info!("no theme selected in the C# install");
        return;
    }
    match read_info(&library.location.join(&library.selected)) {
        Ok(info) => tracing::info!(
            theme = %library.selected,
            kind = ?info.kind,
            source = %info.source,
            "selected theme"
        ),
        Err(error) => tracing::warn!(%error, "selected theme cannot be read"),
    }
}
