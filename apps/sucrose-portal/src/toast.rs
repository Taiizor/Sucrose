//! Toasts (UI-SPEC E12) over gpui-component's notifications: bottom right above the status bar, at most 3 stacked (a 4th
//! closes the oldest at once), 5 s, errors 8 s with a close button; the text wraps.

use std::{collections::VecDeque, time::Duration};

use gpui_kit::component::{Theme, WindowExt as _, notification::Notification};
use gpui_kit::*;

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
}

impl Kind {
    /// How long the toast stays before it closes by itself.
    pub fn duration(self) -> Duration {
        match self {
            Self::Info => Duration::from_secs(5),
            Self::Error => Duration::from_secs(8),
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
#[allow(
    dead_code,
    reason = "the settings and library plans show info toasts (corrupt themes moved, copied)"
)]
pub fn show_info(text: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
    show(Kind::Info, text.into(), window, cx);
}

/// Shows an error toast: 8 s, with the close button.
pub fn show_error(text: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
    show(Kind::Error, text.into(), window, cx);
}

fn show(kind: Kind, text: SharedString, window: &mut Window, cx: &mut App) {
    let (id, evicted) = cx.default_global::<ToastStack>().push();
    if let Some(oldest) = evicted {
        window.remove_notification1::<ToastKey>(oldest, cx);
    }
    let note = match kind {
        Kind::Info => Notification::info(text),
        Kind::Error => Notification::error(text).autohide(false),
    }
    .id1::<ToastKey>(id)
    .on_close(move |_, cx| cx.default_global::<ToastStack>().remove(id));
    window.push_notification(note, cx);
    if kind == Kind::Error {
        // The notification layer hides after 5 s at most; an error stays 8 s, so it times itself.
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
    use super::{Kind, MAX_TOASTS, ToastStack};
    use std::time::Duration;

    #[test]
    fn errors_last_eight_seconds_and_the_rest_five() {
        assert_eq!(Kind::Info.duration(), Duration::from_secs(5));
        assert_eq!(Kind::Error.duration(), Duration::from_secs(8));
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
