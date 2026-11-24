//! The tray hint dialog (UI-05, D-11): on the first close of the window, while the hint is on and the tray icon is
//! visible, the Portal says it keeps running in the system tray. Close is disabled for a 5 second countdown; it turns the
//! hint off through the service and then closes the window (see `PortalApp::finish_tray_hint`).
//!
//! The dialog keeps Escape and a press on the backdrop off: the countdown is the point of it, and the Close button is the
//! one way out. (The UI-SPEC lets Escape close the dialog after the countdown; the dialog layer fixes that setting when
//! the dialog opens, so it cannot follow the countdown.)

use std::{rc::Rc, time::Duration};

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogFooter,
    h_flex, v_flex,
};
use gpui_kit::*;

use super::{DIALOG_WIDTH, body};
use sucrose_ipc::portal::{PortalRequest, SettingChange};

use crate::{
    i18n::{tr, tr_with},
    ipc::ServiceLink,
};

/// What runs when Close is pressed.
type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Seconds the Close button stays disabled.
pub const COUNTDOWN_START: u32 = 5;
/// How often the countdown moves.
const TICK: Duration = Duration::from_secs(1);

/// Whether closing the window shows the hint: the setting is on, the tray icon is visible and the hint was not shown
/// during this window's life yet.
pub fn should_hint(hint_tray_icon: bool, tray_visible: bool, already_shown: bool) -> bool {
    hint_tray_icon && tray_visible && !already_shown
}

/// The Close label with the seconds that remain; plain "Close" once the countdown is over.
pub fn countdown_label(remaining: u32) -> String {
    if remaining == 0 {
        tr("Portal.TrayIcon.Close").to_string()
    } else {
        let seconds = remaining.to_string();
        tr_with("Portal.TrayIcon.CloseCountdown", &[("seconds", &seconds)]).to_string()
    }
}

/// Whether Close can be pressed.
pub fn close_enabled(remaining: u32) -> bool {
    remaining == 0
}

/// The seconds that remain after one more second.
pub fn next_remaining(remaining: u32) -> u32 {
    remaining.saturating_sub(1)
}

/// The Close action of the dialog and its countdown: the footer, which stays fixed while the body scrolls.
pub struct TrayHint {
    remaining: u32,
    /// Close was pressed: the setting is on its way to the service and the window follows.
    closed: bool,
    /// Sends the setting change to the service.
    link: ServiceLink,
    /// Runs once when Close is pressed, before the change is sent.
    on_close: CloseHandler,
    /// The countdown; dropped with the dialog.
    _tick: Task<()>,
}

impl TrayHint {
    /// The footer with a countdown that starts now. Close turns the hint off through `link` and runs `on_close`.
    pub fn new(link: ServiceLink, on_close: impl Fn(&mut Window, &mut App) + 'static, cx: &mut Context<Self>) -> Self {
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let Ok(finished) = this.update(cx, |this, cx| {
                    this.remaining = next_remaining(this.remaining);
                    cx.notify();
                    close_enabled(this.remaining)
                }) else {
                    break;
                };
                if finished {
                    break;
                }
            }
        });
        Self {
            remaining: COUNTDOWN_START,
            closed: false,
            link,
            on_close: Rc::new(on_close),
            _tick: tick,
        }
    }
}

impl Render for TrayHint {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let enabled = close_enabled(self.remaining) && !self.closed;
        DialogFooter::new().child(
            Button::new("tray-hint-close")
                .primary()
                .label(countdown_label(self.remaining))
                .disabled(!enabled)
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                    if this.closed {
                        return;
                    }
                    this.closed = true;
                    cx.notify();
                    let on_close = this.on_close.clone();
                    on_close(window, cx);
                    // The service writes the setting (D-11); the window closes when it answers.
                    this.link
                        .send(PortalRequest::Change(SettingChange::HintTrayIcon(false)));
                })),
        )
    }
}

/// A static picture of a notification area with the app's icon in it (the legacy dialog plays a GIF; this one costs no
/// memory for frames).
fn illustration(cx: &App) -> AnyElement {
    let theme = cx.theme();
    let dot = |alpha: f32| div().size_3().rounded_full().bg(theme.muted_foreground.opacity(alpha));
    h_flex()
        .w_full()
        .h_12()
        .px_3()
        .gap_3()
        .items_center()
        .justify_end()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(dot(0.35))
        .child(dot(0.35))
        .child(
            div()
                .p_1()
                .rounded(theme.radius)
                .bg(theme.accent)
                .text_color(theme.accent_foreground)
                .child(Icon::new(IconName::LayoutGrid)),
        )
        .into_any_element()
}

/// Opens the dialog with `hint` as its footer.
pub fn open(hint: Entity<TrayHint>, window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, move |dialog, window, cx| {
        let message = div().text_sm().child(tr("Portal.TrayIcon.Message")).into_any_element();
        let content = v_flex()
            .gap_4()
            .child(illustration(cx))
            .child(message)
            .into_any_element();
        dialog
            .title(tr("Portal.TrayIcon.Title"))
            .w(px(DIALOG_WIDTH))
            .keyboard(false)
            .overlay_closable(false)
            .close_button(false)
            .child(body(window, [content]))
            .footer(hint.clone())
    });
}

#[cfg(test)]
mod tests {
    use super::{COUNTDOWN_START, close_enabled, countdown_label, next_remaining, should_hint};

    #[test]
    fn the_hint_shows_only_when_everything_allows_it() {
        assert!(should_hint(true, true, false));
        assert!(!should_hint(false, true, false), "the user turned the hint off");
        assert!(
            !should_hint(true, false, false),
            "without a tray icon there is nothing to point at"
        );
        assert!(!should_hint(true, true, true), "once per window");
        assert!(!should_hint(false, false, true));
    }

    #[test]
    fn the_countdown_counts_from_five_to_a_plain_close() {
        let labels: Vec<String> = (0..=COUNTDOWN_START).rev().map(countdown_label).collect();
        assert_eq!(
            labels,
            ["Close (5)", "Close (4)", "Close (3)", "Close (2)", "Close (1)", "Close"]
        );
    }

    #[test]
    fn close_works_only_after_the_countdown() {
        for remaining in 1..=COUNTDOWN_START {
            assert!(!close_enabled(remaining), "{remaining}");
        }
        assert!(close_enabled(0));
    }

    #[test]
    fn the_countdown_stops_at_zero() {
        assert_eq!(next_remaining(5), 4);
        assert_eq!(next_remaining(1), 0);
        assert_eq!(next_remaining(0), 0);
    }
}
