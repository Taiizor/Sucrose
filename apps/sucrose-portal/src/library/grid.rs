//! The library grid (UI-SPEC E1): rows of cards by the layout functions, inside a vertical scroll area.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use super::{
    card::{self, CardData},
    layout,
};

/// Padding around the grid.
pub const CONTENT_PADDING: f32 = 16.0;

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
