//! The notification-area icon of the service: a tooltip and a menu with Open and an exit item (TRAY-01, D-14).

use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError};

use tray_icon::{
    Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconId,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};
use windows::Win32::UI::{
    HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi},
    WindowsAndMessaging::{CreateIconFromResourceEx, LR_DEFAULTCOLOR, SM_CXSMICON},
};

use super::os_error;
use crate::{PlatformError, TrayAction, TrayLabels, icon};

/// The icon resource format version `CreateIconFromResourceEx` expects.
const ICON_RESOURCE_VERSION: u32 = 0x0003_0000;

/// What the service wants to hear when the user acts on the icon.
type Callback = Arc<dyn Fn(TrayAction) + Send + Sync>;

/// Which menu items and icon the process-wide event handlers answer for, and whom they tell.
struct Wiring {
    open: MenuId,
    exit: MenuId,
    icon: TrayIconId,
    on_action: Callback,
}

/// The wiring of the tray that exists now. The event handlers of tray-icon and muda can be set only once per process,
/// so they are installed once and read the current wiring here: a tray that is dropped and built again (the
/// Notification Area Icon switch, a relaunch after Exit) gets its own ids and callback without touching them.
static WIRING: Mutex<Option<Wiring>> = Mutex::new(None);
static HANDLERS: Once = Once::new();

fn wiring() -> MutexGuard<'static, Option<Wiring>> {
    WIRING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Installs the menu and double-click handlers, the first time only. Both run on the thread that pumps the loop.
fn install_handlers() {
    HANDLERS.call_once(|| {
        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            let routed = wiring().as_ref().and_then(|wiring| {
                let action = if event.id == wiring.open {
                    TrayAction::Open
                } else if event.id == wiring.exit {
                    TrayAction::Exit
                } else {
                    return None;
                };
                Some((action, Arc::clone(&wiring.on_action)))
            });
            if let Some((action, on_action)) = routed {
                on_action(action);
            }
        }));
        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            // The legacy tray opens the app on a left double-click.
            if let TrayIconEvent::DoubleClick {
                id,
                button: MouseButton::Left,
                ..
            } = event
            {
                let on_action = wiring()
                    .as_ref()
                    .filter(|wiring| wiring.icon == id)
                    .map(|wiring| Arc::clone(&wiring.on_action));
                if let Some(on_action) = on_action {
                    on_action(TrayAction::Open);
                }
            }
        }));
    });
}

/// The service's tray icon. Dropping it removes the icon.
pub struct Tray {
    icon: TrayIcon,
    open: MenuItem,
    exit: MenuItem,
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

fn menu_error(error: &tray_icon::menu::Error) -> PlatformError {
    PlatformError::Failed {
        context: "tray menu",
        message: error.to_string(),
    }
}

impl Tray {
    /// Shows the tray icon with the tooltip and a menu of two items, Open and the exit item, labelled by `labels`.
    /// Choosing an item, or a left double-click on the icon (Open), calls `on_action` on the thread that pumps the
    /// message loop, so it must only hand the action over (for example to a channel). Must run on that thread.
    pub fn new(
        ico: &[u8],
        tooltip: &str,
        labels: &TrayLabels,
        on_action: impl Fn(TrayAction) + Send + Sync + 'static,
    ) -> Result<Self, PlatformError> {
        let icon = load_icon(ico)?;
        let open = MenuItem::new(&labels.open, true, None);
        let exit = MenuItem::new(&labels.exit, true, None);
        let menu = Menu::new();
        menu.append(&open).map_err(|e| menu_error(&e))?;
        menu.append(&exit).map_err(|e| menu_error(&e))?;
        install_handlers();
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            // A left click does nothing and a double-click opens; the menu is on the right button.
            .with_menu_on_left_click(false)
            .with_icon(icon)
            .with_tooltip(icon::truncate_tooltip(tooltip))
            .build()
            .map_err(|e| PlatformError::Failed {
                context: "tray icon",
                message: e.to_string(),
            })?;
        *wiring() = Some(Wiring {
            open: open.id().clone(),
            exit: exit.id().clone(),
            icon: tray.id().clone(),
            on_action: Arc::new(on_action),
        });
        Ok(Self { icon: tray, open, exit })
    }

    /// Replaces the tooltip, cut to what the notification area keeps (127 UTF-16 units).
    pub fn set_tooltip(&self, tooltip: &str) {
        if let Err(error) = self.icon.set_tooltip(Some(icon::truncate_tooltip(tooltip))) {
            tracing::warn!(%error, "cannot update the tray tooltip");
        }
    }

    /// Enables or greys out the Open item (it is greyed out while the Portal is open).
    pub fn set_open_enabled(&self, enabled: bool) {
        self.open.set_enabled(enabled);
    }

    /// Replaces the text of the exit item (it follows AppExit).
    pub fn set_exit_label(&self, label: &str) {
        self.exit.set_text(label);
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        // The callback holds what the service needs for this run; later events of this icon are not wanted. A tray
        // built after this one owns the wiring by then and is left alone.
        let mut current = wiring();
        if current.as_ref().is_some_and(|wiring| wiring.open == *self.open.id()) {
            *current = None;
        }
    }
}
