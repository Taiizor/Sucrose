//! The notification-area icon of the service: a tooltip and a menu with one item, Quit (TRAY-01).

use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use windows::Win32::UI::{
    HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi},
    WindowsAndMessaging::{CreateIconFromResourceEx, LR_DEFAULTCOLOR, SM_CXSMICON},
};

use super::{event_loop::LoopProxy, os_error};
use crate::{PlatformError, icon};

/// The icon resource format version `CreateIconFromResourceEx` expects.
const ICON_RESOURCE_VERSION: u32 = 0x0003_0000;

/// The service's tray icon. Dropping it removes the icon.
pub struct Tray {
    icon: TrayIcon,
}

impl std::fmt::Debug for Tray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tray").finish_non_exhaustive()
    }
}

/// An HICON for the small-icon size of this session, built from the best entry of the ICO file `ico`.
fn load_icon(ico: &[u8]) -> Result<Icon, PlatformError> {
    // SAFETY: no arguments; reads the system DPI.
    let dpi = unsafe { GetDpiForSystem() };
    // SAFETY: plain integer arguments.
    let size = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    let size = if size > 0 { size } else { 16 };
    let entry = icon::pick_ico_entry(ico, size.unsigned_abs()).ok_or(PlatformError::Failed {
        context: "tray icon",
        message: "the icon file has no usable image".to_owned(),
    })?;
    // SAFETY: `entry` is a valid slice that outlives the call; the function copies what it needs.
    let handle = unsafe { CreateIconFromResourceEx(entry, true, ICON_RESOURCE_VERSION, size, size, LR_DEFAULTCOLOR) }
        .map_err(|e| os_error("CreateIconFromResourceEx", &e))?;
    // tray-icon owns the handle from here on and destroys it when the icon is dropped, so it is not destroyed here.
    Ok(Icon::from_handle(handle.0 as isize))
}

impl Tray {
    /// Shows the tray icon with the tooltip and a menu whose only item, Quit, calls `proxy.quit()`. Must run on the
    /// thread that pumps the loop of `proxy`.
    pub fn new(ico: &[u8], tooltip: &str, proxy: LoopProxy) -> Result<Self, PlatformError> {
        let icon = load_icon(ico)?;
        let quit = MenuItem::new("Quit", true, None);
        let quit_id = quit.id().clone();
        let menu = Menu::new();
        menu.append(&quit).map_err(|e| PlatformError::Failed {
            context: "tray menu",
            message: e.to_string(),
        })?;
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if event.id == quit_id {
                proxy.quit();
            }
        }));
        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip(icon::truncate_tooltip(tooltip))
            .build()
            .map_err(|e| PlatformError::Failed {
                context: "tray icon",
                message: e.to_string(),
            })?;
        Ok(Self { icon })
    }

    /// Replaces the tooltip, cut to what the notification area keeps (127 UTF-16 units).
    pub fn set_tooltip(&self, tooltip: &str) {
        if let Err(error) = self.icon.set_tooltip(Some(icon::truncate_tooltip(tooltip))) {
            tracing::warn!(%error, "cannot update the tray tooltip");
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        // The handler holds a LoopProxy for a loop that is about to end; later menu events are not wanted.
        MenuEvent::set_event_handler::<fn(MenuEvent)>(None);
    }
}
