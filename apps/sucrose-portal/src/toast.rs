//! Toasts (UI-SPEC E12) over gpui-component's notifications: bottom right above the status bar, at most 3 stacked (a 4th
//! closes the oldest at once), 5 s, errors 8 s with a close button, warnings (a fallback engine, UI-SPEC E3) 12 s with a
//! close button and a snooze action; the text wraps.

use std::{collections::VecDeque, time::Duration};

use gpui_kit::component::{
    Sizable as _, Theme, WindowExt as _,
    button::{Button, ButtonVariants as _},
    notification::Notification,
};
use gpui_kit::*;
use sucrose_core::settings::EngineType;
use sucrose_ipc::portal::FallbackView;

/// The most toasts on screen.
pub const MAX_TOASTS: usize = 3;
/// How far the toasts sit above the window's bottom edge: the 32 px status bar and 16 px of air.
const BOTTOM_MARGIN: f32 = 48.0;

/// What a toast reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A plain message.
    Info,
    /// A failure; it stays longer.
    Error,
    /// A warning the user may act on (a fallback engine); it stays longest.
    Warning,
}

impl Kind {
    /// How long the toast stays before it closes by itself.
    pub fn duration(self) -> Duration {
        match self {
            Self::Info => Duration::from_secs(5),
            Self::Error => Duration::from_secs(8),
            Self::Warning => Duration::from_secs(12),
        }
    }
}

/// The toasts on screen, oldest first. The numbers are ours: they name a toast to the notification layer.
#[derive(Debug, Default)]
struct ToastStack {
    shown: VecDeque<usize>,
    next: usize,
}

impl ToastStack {
    /// Adds a toast; returns its number and, when the stack was full, the oldest toast, which must close now.
    fn push(&mut self) -> (usize, Option<usize>) {
        let id = self.next;
        self.next += 1;
        let evicted = if self.shown.len() >= MAX_TOASTS {
            self.shown.pop_front()
        } else {
            None
        };
        self.shown.push_back(id);
        (id, evicted)
    }

    /// Forgets a toast that closed.
    fn remove(&mut self, id: usize) {
        self.shown.retain(|shown| *shown != id);
    }
}

impl Global for ToastStack {}

/// Marks our notifications, so they are closed by number and never clash with another kind.
struct ToastKey;

/// Puts the toasts where the UI-SPEC wants them. Call once after the theme is installed.
pub fn configure(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.notification.placement = Anchor::BottomRight;
        theme.notification.margins.bottom = px(BOTTOM_MARGIN);
        theme.notification.max_items = MAX_TOASTS;
    });
}

/// Shows a plain toast.
pub fn show_info(text: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
    show(Kind::Info, Notification::info(text.into()), window, cx);
}

/// Shows an error toast: 8 s, with the close button.
pub fn show_error(text: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
    show(Kind::Error, Notification::error(text.into()), window, cx);
}

/// Shows a warning toast (UI-SPEC E3): 12 s, with the close button and a ghost action that runs `on_action` and closes the
/// toast.
pub fn show_warning(
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    action_label: impl Into<SharedString>,
    on_action: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) {
    let label: SharedString = action_label.into();
    let on_action = std::rc::Rc::new(on_action);
    let note = Notification::warning(body.into())
        .title(title.into())
        .content(move |_, _, cx| {
            let on_action = on_action.clone();
            div()
                .mt_2()
                .child(
                    Button::new("toast-action")
                        .ghost()
                        .small()
                        .label(label.clone())
                        .on_click(cx.listener(move |this, _, window, cx| {
                            on_action(window, cx);
                            this.dismiss(window, cx);
                        })),
                )
                .into_any_element()
        });
    show(Kind::Warning, note, window, cx);
}

/// The fallbacks the Portal raises a warning toast for now: the ones the service wants announced (`notify`), one per
/// configured engine and run. `shown` remembers the engines announced in this run (UI-SPEC E3, repeat).
pub fn fresh_fallbacks(fallbacks: &[FallbackView], shown: &mut Vec<EngineType>) -> Vec<FallbackView> {
    let mut fresh = Vec::new();
    for fallback in fallbacks {
        if fallback.notify && !shown.contains(&fallback.configured) {
            shown.push(fallback.configured);
            fresh.push(*fallback);
        }
    }
    fresh
}

fn show(kind: Kind, note: Notification, window: &mut Window, cx: &mut App) {
    let (id, evicted) = cx.default_global::<ToastStack>().push();
    if let Some(oldest) = evicted {
        window.remove_notification1::<ToastKey>(oldest, cx);
    }
    let note = match kind {
        Kind::Info => note,
        Kind::Error | Kind::Warning => note.autohide(false),
    }
    .id1::<ToastKey>(id)
    .on_close(move |_, cx| cx.default_global::<ToastStack>().remove(id));
    window.push_notification(note, cx);
    if kind != Kind::Info {
        // The notification layer hides after 5 s at most; an error stays 8 s and a warning 12 s, so they time themselves.
        let wait = kind.duration();
        window
            .spawn(cx, async move |cx| {
                cx.background_executor().timer(wait).await;
                cx.update(|window, cx| window.remove_notification1::<ToastKey>(id, cx))
                    .ok();
            })
            .detach();
    }
}

#[cfg(test)]
mod tests {
    // Not `use super::*`: the crate root glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{Kind, MAX_TOASTS, ToastStack, fresh_fallbacks};
    use std::time::Duration;
    use sucrose_core::{settings::EngineType, theme::WallpaperType};
    use sucrose_ipc::portal::FallbackView;

    #[test]
    fn errors_last_eight_seconds_warnings_twelve_and_the_rest_five() {
        assert_eq!(Kind::Info.duration(), Duration::from_secs(5));
        assert_eq!(Kind::Error.duration(), Duration::from_secs(8));
        assert_eq!(Kind::Warning.duration(), Duration::from_secs(12));
    }

    fn fallback(kind: WallpaperType, configured: EngineType, notify: bool) -> FallbackView {
        FallbackView {
            kind,
            configured,
            playing: EngineType::CefSharpLive,
            notify,
        }
    }

    #[test]
    fn a_fallback_is_announced_once_per_engine_and_run() {
        let mut shown = Vec::new();
        let web = fallback(WallpaperType::Web, EngineType::WebViewLive, true);
        // Two types fall back from the same engine: one toast.
        let url = fallback(WallpaperType::Url, EngineType::WebViewLive, true);

        assert_eq!(fresh_fallbacks(&[web, url], &mut shown), [web]);
        assert!(
            fresh_fallbacks(&[web], &mut shown).is_empty(),
            "the next report is quiet"
        );
        assert!(fresh_fallbacks(&[], &mut shown).is_empty());
    }

    #[test]
    fn two_unavailable_engines_raise_two_toasts_and_a_snoozed_one_none() {
        let mut shown = Vec::new();
        let web_view = fallback(WallpaperType::Web, EngineType::WebViewLive, true);
        let cef = fallback(WallpaperType::Video, EngineType::CefSharpLive, true);
        let snoozed = fallback(WallpaperType::Gif, EngineType::MpvPlayerLive, false);

        assert_eq!(fresh_fallbacks(&[web_view, cef, snoozed], &mut shown), [web_view, cef]);
        assert!(
            !shown.contains(&EngineType::MpvPlayerLive),
            "a snoozed engine is not marked as shown"
        );
    }

    #[test]
    fn up_to_three_toasts_stack_without_closing_one() {
        let mut stack = ToastStack::default();
        for _ in 0..MAX_TOASTS {
            assert_eq!(stack.push().1, None);
        }
    }

    #[test]
    fn a_fourth_toast_closes_the_oldest_at_once() {
        let mut stack = ToastStack::default();
        let first = stack.push().0;
        let second = stack.push().0;
        let third = stack.push().0;
        let (fourth, evicted) = stack.push();
        assert_eq!(evicted, Some(first));
        let (_, evicted) = stack.push();
        assert_eq!(evicted, Some(second));
        assert_ne!(third, fourth);
        assert_eq!(stack.shown.len(), MAX_TOASTS);
    }

    #[test]
    fn a_closed_toast_leaves_room_and_numbers_never_repeat() {
        let mut stack = ToastStack::default();
        let ids: Vec<usize> = (0..MAX_TOASTS).map(|_| stack.push().0).collect();
        stack.remove(ids[1]);
        let (new, evicted) = stack.push();
        assert_eq!(evicted, None, "a closed toast made room");
        assert!(!ids.contains(&new));
        stack.remove(999);
        assert_eq!(stack.shown.len(), MAX_TOASTS, "an unknown number changes nothing");
    }
}
