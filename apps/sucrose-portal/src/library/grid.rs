//! The library grid (UI-SPEC E1): rows of cards by the layout functions, inside a vertical scroll area.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _,
    button::Button,
    empty::{Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle},
    h_flex,
    scroll::{Scrollbar, ScrollbarAxis},
    v_flex,
};
use gpui_kit::*;
use std::ops::Range;

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

/// The scroll area around `block`: centered, with the content padding. The scrollbar is a sibling of the scroll area inside a
/// non-scrolling relative wrapper, not a child of it: GPUI offsets every child of a scroll container, absolute ones included,
/// so a bar appended to the scrolled element moves away with the cards (WR-15). Both use the `scroll` handle the view resets
/// on a page change and reads for the rows in view (`overflow_y_scrollbar` would scroll a handle of its own). The bar follows
/// `Theme.scrollbar_mode` as gpui-component syncs it from the OS at init (Windows: shown on hover while the OS auto-hides
/// scrollbars, always with "Always show scrollbars" on).
fn scroll_area(scroll: &ScrollHandle, block: impl IntoElement) -> AnyElement {
    div()
        .relative()
        .size_full()
        .child(
            div()
                .id("library-scroll")
                .size_full()
                .overflow_y_scroll()
                .track_scroll(scroll)
                .debug_selector(|| "library-scroll".to_string())
                .child(v_flex().items_center().p(px(CONTENT_PADDING)).child(block)),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .debug_selector(|| "library-scrollbar".to_string())
                .child(
                    Scrollbar::new(scroll)
                        .id("library-scrollbar")
                        .axis(ScrollbarAxis::Vertical)
                        .viewport_from_layout(),
                ),
        )
        .into_any_element()
}

/// What `visible_rows` read from the scroll handle.
#[derive(Clone, Copy, PartialEq, Debug)]
struct ScrollRead {
    height: Pixels,
    offset_y: Pixels,
}

fn read(scroll: &ScrollHandle) -> ScrollRead {
    ScrollRead {
        height: scroll.bounds().size.height,
        offset_y: scroll.offset().y,
    }
}

/// The rows of the grid in view, for the thumbnail requests: the rows of the page's `rows` rows (`gap` apart) that meet the
/// scroll viewport, from the handle's bounds and offset as the previous frame left them. The window's height stands in for
/// the viewport before the first paint (its bounds are zero then; asking for no row would leave the cards static and never
/// trigger another frame).
///
/// Those reads lag (WR-16): GPUI clamps the offset to the content only in the scroll container's prepaint, after `render`,
/// and neither that clamp nor a resize notifies the view, while a card left on the static `Idle` fill asks for no frame, so
/// nothing else would render the view again. The offset is therefore clamped to the content of the same rows
/// ([`layout::clamped_scroll_top`]), and once the frame is drawn `Window::on_next_frame` notifies the view when the
/// handle's bounds or offset differ from what this call read. The comparison is an exact one of the raw reads, not of the
/// clamped value and with no tolerance: the render it causes reads exactly the painted values, so the next check is equal
/// and a still window draws one extra frame at most.
pub fn visible_rows<V: 'static>(
    scroll: &ScrollHandle,
    gap: f32,
    rows: usize,
    window: &Window,
    cx: &mut Context<V>,
) -> Range<usize> {
    let used = read(scroll);
    let measured = f32::from(used.height);
    let viewport = if measured > 0.0 {
        measured
    } else {
        f32::from(window.viewport_size().height)
    };
    let scroll_top = (-f32::from(used.offset_y)).max(0.0);
    let scroll_top = layout::clamped_scroll_top(scroll_top, viewport, CONTENT_PADDING, gap, rows);
    let view = cx.entity().downgrade();
    let handle = scroll.clone();
    window.on_next_frame(move |_, cx| {
        if read(&handle) != used {
            // A view that is gone has nothing to render.
            view.update(cx, |_, cx| cx.notify()).ok();
        }
    });
    layout::rows_in_view(scroll_top, viewport, CONTENT_PADDING, gap, rows)
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

    // Not `use super::*`: the crate root glob-imports GPUI, whose `test` macro would shadow the built-in attribute.
    use super::{ScrollRead, layout, read, scroll_area, visible_rows};
    use gpui_kit::component::v_flex;
    use gpui_kit::{
        Bounds, Context, Entity, InteractiveElement as _, IntoElement, Modifiers, ParentElement as _, Pixels, Render,
        ScrollDelta, ScrollEvent, ScrollHandle, StatefulInteractiveElement as _, Styled as _, TestAppContext,
        VisualTestContext, Window, div, point, px, size,
    };
    use std::{cell::Cell, ops::Range, rc::Rc};

    /// A window root that shows `rows` rows of 100 px (the first one clickable, the last one tagged `last-row`) in
    /// `scroll_area`.
    struct ScrollHost {
        scroll: ScrollHandle,
        rows: usize,
        clicked: Rc<Cell<bool>>,
    }

    impl Render for ScrollHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let last = self.rows.saturating_sub(1);
            let clicked = self.clicked.clone();
            let block = v_flex().w(px(400.)).children((0..self.rows).map(|ix| {
                let row = div().id(("row", ix)).h(px(100.)).w_full().flex_shrink_0();
                let row = if ix == 0 {
                    let clicked = clicked.clone();
                    row.on_click(move |_, _, _| clicked.set(true))
                } else {
                    row
                };
                if ix == last {
                    row.debug_selector(|| "last-row".to_string())
                } else {
                    row
                }
            }));
            div().size_full().child(scroll_area(&self.scroll, block))
        }
    }

    fn draw(window: &mut VisualTestContext) {
        window.run_until_parked();
        window.update(|window, cx| {
            _ = window.draw(cx);
        });
    }

    /// One wheel step of `dy` pixels at (`x`, `y`), then a draw.
    fn wheel(window: &mut VisualTestContext, x: f32, y: f32, dy: f32) {
        window.simulate_event(ScrollEvent {
            position: point(px(x), px(y)),
            delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
            ..Default::default()
        });
        draw(window);
    }

    /// A 600 x 400 window with `rows` rows, drawn once.
    fn window_with_rows(cx: &mut TestAppContext, rows: usize, clicked: Rc<Cell<bool>>) -> &mut VisualTestContext {
        cx.update(gpui_kit::init);
        let (_, window) = cx.add_window_view(|_, _| ScrollHost {
            scroll: ScrollHandle::new(),
            rows,
            clicked,
        });
        window.simulate_resize(size(px(600.), px(400.)));
        draw(window);
        window
    }

    fn bounds(window: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
        window
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("no element tagged {selector}"))
    }

    #[gpui_kit::test]
    fn grid_scrollbar_stays_on_the_right_edge_while_the_cards_scroll(cx: &mut TestAppContext) {
        let window = window_with_rows(cx, 30, Rc::default());
        let overlay = bounds(window, "library-scrollbar");
        let area = bounds(window, "library-scroll");
        assert_eq!(overlay, area, "the overlay covers the scroll area");
        let before = bounds(window, "last-row").origin.y;

        wheel(window, 300., 200., -200.);

        assert_eq!(
            bounds(window, "library-scrollbar"),
            overlay,
            "the overlay keeps its bounds while the cards scroll"
        );
        assert_eq!(
            overlay.right(),
            area.right(),
            "the overlay ends at the scroll area's right edge"
        );
        assert_eq!(
            before - bounds(window, "last-row").origin.y,
            px(200.),
            "the last row moved up by the wheel step"
        );
    }

    #[gpui_kit::test]
    fn grid_scrollbar_overlay_lets_clicks_reach_the_cards(cx: &mut TestAppContext) {
        let clicked = Rc::new(Cell::new(false));
        let window = window_with_rows(cx, 30, clicked.clone());
        let area = bounds(window, "library-scroll");
        // Row 0 spans y 16..116 and the 400 px block is centered, so (300, 50) is inside it, far left of the track.
        let at = point(px(300.), px(50.));
        assert!(
            area.right() - at.x >= px(40.),
            "the click is at least 40 px left of the right edge"
        );

        window.simulate_click(at, Modifiers::none());
        draw(window);

        assert!(clicked.get(), "the click went through the overlay to the row's handler");
    }

    #[gpui_kit::test]
    fn grid_scrollbar_page_end_and_short_pages_do_not_move(cx: &mut TestAppContext) {
        {
            let window = window_with_rows(cx, 30, Rc::default());
            let area = bounds(window, "library-scroll");
            // The content is 30 * 100 + 2 * 16 = 3032 px in a 400 px viewport: 2632 px of travel, passed after three steps.
            for _ in 0..3 {
                wheel(window, 300., 200., -1000.);
            }
            let last = bounds(window, "last-row");
            assert!(
                last.bottom() <= area.bottom(),
                "the last row ends inside the scroll area"
            );
            assert!(last.bottom() > area.top(), "the last row is in view");
            let overlay = bounds(window, "library-scrollbar");

            wheel(window, 300., 200., -200.);

            assert_eq!(
                bounds(window, "last-row"),
                last,
                "scrolled past the end the content stays on its last row"
            );
            assert_eq!(bounds(window, "library-scrollbar"), overlay, "the overlay stays too");
            assert_eq!(bounds(window, "library-scroll"), area);
        }
        let window = window_with_rows(cx, 2, Rc::default());
        let last = bounds(window, "last-row");

        wheel(window, 300., 200., -200.);

        assert_eq!(
            bounds(window, "last-row"),
            last,
            "a page shorter than the viewport does not move"
        );
    }

    /// A window root that takes its rows in view from `visible_rows` in every render, the way the library view does.
    struct SettleHost {
        rows: usize,
        scroll: ScrollHandle,
        range: Range<usize>,
        used: Option<ScrollRead>,
    }

    impl Render for SettleHost {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.used = Some(read(&self.scroll));
            self.range = visible_rows(&self.scroll, 5.0, self.rows, window, cx);
            let block = v_flex()
                .gap(px(5.))
                .w(px(400.))
                .children((0..self.rows).map(|_| div().h(px(layout::CARD_HEIGHT)).w(px(400.)).flex_shrink_0()));
            div().size_full().child(scroll_area(&self.scroll, block))
        }
    }

    fn settle_window(cx: &mut TestAppContext, rows: usize) -> (Entity<SettleHost>, &mut VisualTestContext) {
        cx.update(gpui_kit::init);
        let (view, window) = cx.add_window_view(|_, _| SettleHost {
            rows,
            scroll: ScrollHandle::new(),
            range: 0..0,
            used: None,
        });
        window.simulate_resize(size(px(600.), px(400.)));
        window.run_until_parked();
        (view, window)
    }

    /// The range of the last render, the read it used and the handle's current read.
    fn settle_state(
        window: &mut VisualTestContext,
        view: &Entity<SettleHost>,
    ) -> (Range<usize>, Option<ScrollRead>, ScrollRead) {
        window.update(|_, cx| {
            let host = view.read(cx);
            (host.range.clone(), host.used, read(&host.scroll))
        })
    }

    /// Delivers the next frame (the only thing that can render the view: nothing draws the window by hand) until the read
    /// the last render used equals the handle's current read, at most five rounds.
    fn settle(window: &mut VisualTestContext, view: &Entity<SettleHost>) {
        for _ in 0..5 {
            window.update(|window, cx| {
                window.simulate_next_frame(cx);
            });
            window.run_until_parked();
            let (_, used, now) = settle_state(window, view);
            if used == Some(now) {
                return;
            }
        }
    }

    fn assert_settled(window: &mut VisualTestContext, view: &Entity<SettleHost>) {
        let (_, used, now) = settle_state(window, view);
        assert_eq!(
            used,
            Some(now),
            "the last render read what is painted, so the next frame notifies nothing"
        );
    }

    #[gpui_kit::test]
    fn grid_request_settles_after_a_content_shrink(cx: &mut TestAppContext) {
        let (view, window) = settle_window(cx, 30);
        settle(window, &view);
        // Scroll to the end of 30 rows (GPUI clamps the offset in prepaint).
        let scroll = view.read_with(window, |host, _| host.scroll.clone());
        scroll.set_offset(point(px(0.), px(-100_000.)));
        view.update(window, |_, cx| cx.notify());
        settle(window, &view);
        assert_eq!(settle_state(window, &view).0, 27..30, "the end of 30 rows");

        view.update(window, |host, cx| {
            host.rows = 12;
            cx.notify();
        });
        settle(window, &view);

        assert_eq!(settle_state(window, &view).0, 9..12, "the last row of 12 is in view");
        assert_settled(window, &view);
    }

    #[gpui_kit::test]
    fn grid_request_settles_after_the_window_grows(cx: &mut TestAppContext) {
        let (view, window) = settle_window(cx, 30);
        settle(window, &view);
        assert_eq!(settle_state(window, &view).0, 0..3, "three rows in 400 px");

        window.simulate_resize(size(px(600.), px(800.)));
        window.run_until_parked();
        settle(window, &view);

        assert_eq!(
            settle_state(window, &view).0,
            0..5,
            "five rows in 800 px, no input of any kind"
        );
        assert_settled(window, &view);
    }
}
