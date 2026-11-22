//! Repainting the desktop after the wallpaper surface is gone.

use std::ffi::c_void;

use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

/// Re-applies the current wallpaper so the shell repaints the desktop without a stale frame, without writing the
/// registry or broadcasting a settings change.
pub(crate) fn refresh_desktop() {
    // MAX_PATH wide characters, the size SPI_GETDESKWALLPAPER documents.
    let mut path = [0_u16; 260];
    // SAFETY: `path` is writable for the 260 wide characters passed as uiParam.
    let read = unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            path.len() as u32,
            Some(path.as_mut_ptr().cast::<c_void>()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    if let Err(error) = read {
        tracing::warn!(%error, "cannot read the current wallpaper, the desktop is not refreshed");
        return;
    }
    if path[0] == 0 {
        // A solid colour or no picture: the shell repaints that by itself.
        tracing::debug!("the desktop has no wallpaper picture, nothing to re-apply");
        return;
    }
    // SAFETY: `path` still holds the NUL-terminated string the call above wrote; the call only reads it. Flags 0 mean
    // no registry write and no WM_SETTINGCHANGE broadcast.
    let applied = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(path.as_mut_ptr().cast::<c_void>()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    if let Err(error) = applied {
        tracing::warn!(%error, "cannot re-apply the wallpaper, a stale frame may stay until the next repaint");
    }
}
