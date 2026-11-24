//! The library grid (UI-SPEC E1): rows of cards by the layout functions, inside a vertical scroll area.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _,
    button::Button,
    empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex, v_flex,
};
use gpui_kit::*;

use crate::i18n::tr;

use super::{
    card::{self, CardData},
    layout,
};

/// Padding around the grid.
pub const CONTENT_PADDING: f32 = 16.0;

/// What the library area shows once the scan is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub fn library_state(failed: bool, themes: usize, query: &str, matches: usize) -> LibraryState {
    if failed {
        LibraryState::LoadError
    } else if themes == 0 {
        LibraryState::Empty
    } else if !query.trim().is_empty() && matches == 0 {
        LibraryState::SearchEmpty
    } else {
        LibraryState::Grid
    }
}

/// Distance of an empty-state block from the top of the content area (UI-SPEC spacing: 3xl).
const STATE_OFFSET: f32 = 64.0;

/// An empty-state block: an icon, a Display-size heading and a body, then `action`.
fn state(icon: IconName, title: SharedString, body: SharedString, action: Option<AnyElement>) -> AnyElement {
    let mut block = Empty::new().flex_none().max_w(px(560.)).header(
        EmptyHeader::new()
            .max_w(px(520.))
            .media(
                EmptyMedia::new()
                    .with_variant(EmptyMediaVariant::Icon)
                    .child(Icon::new(icon).with_size(px(24.))),
            )
            .title(EmptyTitle::new().text_size(px(24.)).child(title))
            .description(EmptyDescription::new().child(body)),
    );
    if let Some(action) = action {
        block = block.content(EmptyContent::new().child(action));
    }
    v_flex()
        .size_full()
        .items_center()
        .pt(px(STATE_OFFSET))
        .child(block)
        .into_any_element()
}

/// The Change Library Location button: it opens Settings > General.
fn change_location(on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> AnyElement {
    Button::new("library-change-location")
        .label(tr("Portal.LibraryPage.ChangeLocation"))
        .on_click(on_click)
        .into_any_element()
}

/// The library holds no theme at all: Change Library Location opens the settings.
pub fn empty(on_change_location: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> AnyElement {
    state(
        IconName::LibraryBig,
        tr("Portal.EmptyLibraryPage.TitleText"),
        tr("Portal.EmptyLibraryPage.DescriptionText"),
        Some(change_location(on_change_location)),
    )
}

/// The search matches no theme; clearing the box brings the library back, so there is no button.
pub fn search_empty() -> AnyElement {
    state(
        IconName::SearchX,
        tr("Portal.SearchLibraryPage.TitleText"),
        tr("Portal.SearchLibraryPage.DescriptionText"),
        None,
    )
}

/// The library folder cannot be read: Try Again scans it again, Change Library Location opens the settings.
pub fn load_error(
    on_retry: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_change_location: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    state(
        IconName::FolderX,
        tr("Portal.LibraryPage.LoadError.Title"),
        tr("Portal.LibraryPage.LoadError.Body"),
        Some(
            h_flex()
                .gap_2()
                .child(
                    Button::new("library-retry")
                        .label(tr("Portal.LibraryPage.TryAgain"))
                        .on_click(on_retry),
                )
                .child(change_location(on_change_location))
                .into_any_element(),
        ),
    )
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

/// The first-run line under the skeleton grid while the service prepares the library (UI-SPEC E1 loading row).
pub fn preparing(cx: &App) -> AnyElement {
    div()
        .w_full()
        .flex_none()
        .py(px(CONTENT_PADDING))
        .text_center()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(tr("Portal.LibraryPage.Preparing"))
        .into_any_element()
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
