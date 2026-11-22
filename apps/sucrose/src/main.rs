//! The Sucrose service (D-008). For now it starts, logs what it found and exits.

use sucrose_config::legacy::{LegacySettings, LibrarySettings};
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
    log_legacy_settings();
    tracing::info!("nothing to do yet (M0), exiting");
    Ok(())
}

/// Reads the settings of the C# install, read-only from its real folder (D-008), and logs what the service will use.
fn log_legacy_settings() {
    let (Some(root), Some(setting_dir)) = (legacy_data_dir(), legacy_setting_dir()) else {
        tracing::warn!("the OS reports no config folder, so the C# install cannot be read");
        return;
    };
    let settings = LegacySettings::load(&setting_dir, &root);
    log_selected_theme(&settings.library);
    // Values only: no paths and no theme names (D-08, D-09).
    tracing::info!(
        volume = settings.engine.volume,
        looping = settings.engine.looping,
        stretch = ?settings.engine.stretch,
        hardware_acceleration = settings.engine.hardware_acceleration,
        screen = ?settings.engine.screen,
        fullscreen = ?settings.backgroundog.fullscreen,
        pause = ?settings.backgroundog.pause,
        app_visible = settings.general.app_visible,
        "legacy settings"
    );
}

/// Logs the theme selected in the C# install.
fn log_selected_theme(library: &LibrarySettings) {
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
