//! The library grid (UI-SPEC E1): rows of cards by the layout functions, inside a vertical scroll area.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use super::{
    card::{self, CardData},
    layout,
};

/// Padding around the grid.
pub const CONTENT_PADDING: f32 = 16.0;

/// What the library area shows once the scan is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code, reason = "the states are rendered in the next commit")]
pub enum LibraryState {
    /// The library folder cannot be read.
    LoadError,
    /// There are no themes at all.
    Empty,
    /// The search matches none of the themes.
    SearchEmpty,
    /// The cards of the page.
    Grid,
}

/// Which state the library area is in: `failed` is a scan error, `themes` the number of themes found, `query` the
/// normalized search text and `matches` the number of themes it lets through.
#[allow(dead_code, reason = "the states are rendered in the next commit")]
pub fn library_state(_failed: bool, _themes: usize, _query: &str, _matches: usize) -> LibraryState {
    LibraryState::Grid
}

#[cfg(test)]
mod tests {
    use super::{LibraryState, library_state};

    #[test]
    fn a_scan_error_is_a_load_error_whatever_else_is_known() {
        assert_eq!(library_state(true, 0, "", 0), LibraryState::LoadError);
        assert_eq!(library_state(true, 5, "sea", 2), LibraryState::LoadError);
    }

    #[test]
    fn no_themes_is_the_empty_state_even_with_a_query() {
        assert_eq!(library_state(false, 0, "", 0), LibraryState::Empty);
        assert_eq!(library_state(false, 0, "sea", 0), LibraryState::Empty);
    }

    #[test]
    fn a_query_without_a_match_is_the_search_empty_state() {
        assert_eq!(library_state(false, 12, "zzz", 0), LibraryState::SearchEmpty);
    }

    #[test]
    fn a_blank_query_shows_the_whole_library() {
        assert_eq!(library_state(false, 12, "", 12), LibraryState::Grid);
        assert_eq!(library_state(false, 12, "   ", 12), LibraryState::Grid);
    }

    #[test]
    fn one_theme_is_one_card() {
        assert_eq!(library_state(false, 1, "", 1), LibraryState::Grid);
        assert_eq!(library_state(false, 1, "sea", 1), LibraryState::Grid);
    }
}

/// How the cards of a page are placed.
struct Plan {
    columns: usize,
    card_width: f32,
    block_width: f32,
}

fn plan(content_width: f32, gap: f32, max_per_row: usize) -> Plan {
    let columns = layout::columns(content_width, gap, max_per_row);
    let card_width = layout::card_width(content_width, gap, columns);
    Plan {
        columns,
        card_width,
        block_width: layout::row_width(card_width, gap, columns),
    }
}

/// The scroll area around `block`: centered, with the content padding.
fn scroll_area(scroll: &ScrollHandle, block: impl IntoElement) -> AnyElement {
    div()
        .id("library-scroll")
        .size_full()
        .overflow_y_scroll()
        .track_scroll(scroll)
        .child(v_flex().items_center().p(px(CONTENT_PADDING)).child(block))
        .into_any_element()
}

/// The page of `cards`.
pub fn page(
    cards: Vec<CardData>,
    content_width: f32,
    gap: f32,
    max_per_row: usize,
    scroll: &ScrollHandle,
    cx: &App,
) -> AnyElement {
    let plan = plan(content_width, gap, max_per_row);
    let mut block = v_flex().gap(px(gap)).w(px(plan.block_width));
    let mut cards = cards.into_iter();
    loop {
        let row: Vec<CardData> = cards.by_ref().take(plan.columns).collect();
        if row.is_empty() {
            break;
        }
        block = block.child(
            h_flex().gap(px(gap)).children(
                row.into_iter()
                    .map(|data| card::render_card(data, px(plan.card_width), cx)),
            ),
        );
    }
    scroll_area(scroll, block)
}

/// The skeleton grid (columns × 3) that shows when the scan takes long.
pub fn skeleton(content_width: f32, gap: f32, max_per_row: usize, scroll: &ScrollHandle) -> AnyElement {
    let plan = plan(content_width, gap, max_per_row);
    let mut block = v_flex().gap(px(gap)).w(px(plan.block_width));
    for _ in 0..3 {
        block = block.child(
            h_flex()
                .gap(px(gap))
                .children((0..plan.columns).map(|_| card::render_skeleton_card(px(plan.card_width)))),
        );
    }
    scroll_area(scroll, block)
}
