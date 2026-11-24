//! The header row of the library mode (UI-SPEC E3, E4): the Library tab and the search box. Later plans add the theme
//! toggle, Premium, the more menu and the settings gear to the row.

use std::time::Duration;

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    Icon, Sizable as _, h_flex,
    input::{Input, InputEvent, InputState},
    tab::{Tab, TabBar},
    tooltip::Tooltip,
};
use gpui_kit::*;

use crate::{i18n::tr, library::LibraryView};

/// Height of the row (UI-SPEC spacing: 2xl).
const HEADER_HEIGHT: f32 = 48.0;
/// The search box width, and the least it shrinks to before anything else collapses.
const SEARCH_WIDTH: f32 = 320.0;
const SEARCH_MIN_WIDTH: f32 = 200.0;
/// The longest search text, in characters (legacy parity).
pub const SEARCH_MAX_CHARS: usize = 20;
/// How long typing must pause before the library is filtered.
const DEBOUNCE: Duration = Duration::from_millis(500);

/// The search text as the library filters by it: trimmed, lowercased and cut to [`SEARCH_MAX_CHARS`] characters.
pub fn normalize_query(text: &str) -> String {
    text.trim().to_lowercase().chars().take(SEARCH_MAX_CHARS).collect()
}

/// The header row. The search text lives here, so it survives a visit to the settings.
pub struct Header {
    library: Entity<LibraryView>,
    search: Entity<InputState>,
    /// The pending debounce; replacing it cancels the one before.
    pending: Option<Task<()>>,
    _subscription: Subscription,
}

impl Header {
    /// The header that filters `library`.
    pub fn new(library: Entity<LibraryView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr("Portal.MainWindow.SearchBoxHint"))
                // The box takes at most the legacy 20 characters: an edit that would pass that is refused.
                .validate(|text, _| text.chars().count() <= SEARCH_MAX_CHARS)
                // Escape clears the box while it is focused (UI-SPEC E3).
                .clean_on_escape()
        });
        let subscription = cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.on_search_changed(cx);
            }
        });
        Self {
            library,
            search,
            pending: None,
            _subscription: subscription,
        }
    }

    /// Puts the cursor in the search box (Ctrl+F).
    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |state, cx| state.focus(window, cx));
    }

    /// The box text changed: filter after the debounce, or at once when the box was emptied (clear button, Escape).
    fn on_search_changed(&mut self, cx: &mut Context<Self>) {
        let query = normalize_query(&self.search.read(cx).value());
        if query.is_empty() {
            self.pending = None;
            self.apply(&query, cx);
            return;
        }
        let timer = cx.background_executor().timer(DEBOUNCE);
        self.pending = Some(cx.spawn(async move |this, cx| {
            timer.await;
            this.update(cx, |this, cx| this.apply(&query, cx)).ok();
        }));
    }

    /// Hands the query to the library. It stays in this process: nothing is sent anywhere and it is not logged.
    fn apply(&mut self, query: &str, cx: &mut Context<Self>) {
        self.library.update(cx, |library, cx| library.set_query(query, cx));
    }
}

impl Render for Header {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let tip = tr("Portal.MainWindow.SearchBoxTip");
        h_flex()
            .w_full()
            .h(px(HEADER_HEIGHT))
            .flex_shrink_0()
            .px_4()
            .gap_3()
            .items_center()
            .child(
                div().flex_none().child(
                    TabBar::new("header-tabs").underline().selected_index(0).child(
                        Tab::new()
                            .prefix(Icon::new(IconName::Library).small())
                            .label(tr("Portal.MainWindow.MenuLibrary")),
                    ),
                ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("search-box")
                    .w(px(SEARCH_WIDTH))
                    .min_w(px(SEARCH_MIN_WIDTH))
                    .flex_shrink(1.)
                    .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
                    .child(
                        Input::new(&self.search)
                            .prefix(Icon::new(IconName::Search).small())
                            .cleanable(true)
                            .aria_label(tr("Portal.MainWindow.SearchBoxHint")),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{SEARCH_MAX_CHARS, normalize_query};

    #[test]
    fn search_text_is_trimmed_and_lowercased() {
        assert_eq!(normalize_query("  Sea NIGHT "), "sea night");
        assert_eq!(normalize_query("   "), "");
        assert_eq!(normalize_query(""), "");
    }

    #[test]
    fn search_text_is_cut_to_twenty_characters_not_bytes() {
        assert_eq!(normalize_query(&"a".repeat(30)), "a".repeat(SEARCH_MAX_CHARS));
        // Each of these is two bytes: the cut counts characters.
        assert_eq!(normalize_query(&"é".repeat(25)), "é".repeat(SEARCH_MAX_CHARS));
        assert_eq!(normalize_query(&"é".repeat(25)).chars().count(), SEARCH_MAX_CHARS);
    }
}
