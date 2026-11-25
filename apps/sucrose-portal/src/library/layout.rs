//! The grid arithmetic of the library page (LIB-04): columns, card width and page slices. Pure numbers, no GPUI.

use std::ops::Range;

/// Narrowest card; the column count follows from it (legacy parity).
pub const MIN_CARD_WIDTH: f32 = 260.0;
/// Widest card; a row of capped cards is centered.
pub const MAX_CARD_WIDTH: f32 = 400.0;
/// Height of every card.
pub const CARD_HEIGHT: f32 = 160.0;

/// A width this close below a column boundary still reaches it, so float noise from a scale factor does not drop a column.
const BOUNDARY_TOLERANCE: f32 = 0.01;
/// Card widths are rounded down to this step (a power of two, so sums of them are exact in `f32`).
const WIDTH_STEP: f32 = 1.0 / 64.0;

/// The number of columns for a content `width` and a `gap`: as many cards of at least [`MIN_CARD_WIDTH`] as fit, at least
/// one, and at most `max_per_row` when that is above zero (the `AdaptiveLayout` setting). A width that is not a number
/// gives one column.
pub fn columns(width: f32, gap: f32, max_per_row: usize) -> usize {
    let fit = ((width + gap + BOUNDARY_TOLERANCE) / (MIN_CARD_WIDTH + gap))
        .floor()
        .max(1.0) as usize;
    if max_per_row > 0 { fit.min(max_per_row) } else { fit }
}

/// The width of one card when `columns` of them share `width` with `gap` between them: capped at [`MAX_CARD_WIDTH`], never
/// negative, and small enough that [`row_width`] never exceeds `width`.
pub fn card_width(width: f32, gap: f32, columns: usize) -> f32 {
    let columns = columns.max(1) as f32;
    let gaps = gap * (columns - 1.0);
    let card = ((width - gaps) / columns).clamp(0.0, MAX_CARD_WIDTH);
    if !card.is_finite() {
        return 0.0;
    }
    let mut card = (card / WIDTH_STEP).floor() * WIDTH_STEP;
    // Rounding down makes this a safeguard only; it keeps the promise whatever the inputs were.
    while card > 0.0 && card * columns + gaps > width {
        card = (card - WIDTH_STEP).max(0.0);
    }
    card
}

/// The width a row of `columns` cards of `card_width` takes, gaps included.
pub fn row_width(card_width: f32, gap: f32, columns: usize) -> f32 {
    let columns = columns.max(1) as f32;
    card_width * columns + gap * (columns - 1.0)
}

/// An arrow key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Left,
    Right,
    Up,
    Down,
}

impl Arrow {
    /// The arrow a GPUI key name stands for.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            _ => None,
        }
    }
}

/// The card that arrow key `arrow` moves focus to from card `position` of `count` cards laid out row by row in `columns`
/// columns, or `None` at an edge. Left and Right walk the row-major order (Left on the first card of a row goes to the
/// last of the row above); Up and Down keep the column, and Down from a column the last row lacks goes to the last card.
pub fn neighbor(position: usize, count: usize, columns: usize, arrow: Arrow) -> Option<usize> {
    let columns = columns.max(1);
    if position >= count {
        return None;
    }
    match arrow {
        Arrow::Left => position.checked_sub(1),
        Arrow::Right => (position + 1 < count).then_some(position + 1),
        Arrow::Up => position.checked_sub(columns),
        Arrow::Down => {
            let below = position + columns;
            if below < count {
                Some(below)
            } else if position / columns < (count - 1) / columns {
                Some(count - 1)
            } else {
                None
            }
        }
    }
}

/// The rows of a grid that meet the scroll viewport: rows are `rows` in number, row `i` starts at `padding + i * (CARD_HEIGHT
/// + gap)` in content coordinates and is [`CARD_HEIGHT`] tall; the viewport spans `scroll_top` to `scroll_top +
/// viewport_height`. A viewport that is zero, negative or not finite shows no row.
pub fn rows_in_view(_scroll_top: f32, _viewport_height: f32, _padding: f32, _gap: f32, _rows: usize) -> Range<usize> {
    0..0
}

/// The page positions to request thumbnails for, nearest first.
pub fn cards_to_request(
    _visible_rows: Range<usize>,
    _rows: usize,
    _columns: usize,
    _count: usize,
    _lookahead: usize,
) -> Vec<usize> {
    Vec::new()
}

/// How many pages `count` themes make with `size` themes on a page.
pub fn page_count(count: usize, size: usize) -> usize {
    if size == 0 { 0 } else { count.div_ceil(size) }
}

/// The themes of 1-based `page`, as a range of indexes into the sorted list. Page 0 is the first page and a page past the
/// last is empty.
pub fn page_slice(count: usize, page: usize, size: usize) -> Range<usize> {
    let start = (page.max(1) - 1).saturating_mul(size).min(count);
    start..start.saturating_add(size).min(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAPS: [f32; 4] = [0.0, 5.0, 10.0, 25.0];

    const PAD: f32 = 16.0;

    #[test]
    fn layout_rows_in_view_at_the_top() {
        assert_eq!(rows_in_view(0.0, 440.0, PAD, 5.0, 34), 0..3);
    }

    #[test]
    fn layout_rows_in_view_scrolled_to_the_screenshot() {
        // Cards 61-69 counted from 1 are the rows 20, 21 and 22 of three columns.
        let top = PAD + 20.0 * (CARD_HEIGHT + 5.0);
        assert_eq!(top, 3316.0);
        assert_eq!(rows_in_view(top, 440.0, PAD, 5.0, 34), 20..23);
    }

    #[test]
    fn layout_rows_in_view_counts_a_row_that_is_only_partly_in_view() {
        // Row 0 spans 16..176: one pixel of it is still in view at a scroll top of 175, none at 176.
        assert_eq!(rows_in_view(175.0, 100.0, PAD, 5.0, 34), 0..2);
        assert_eq!(rows_in_view(176.0, 100.0, PAD, 5.0, 34), 1..2);
        // Row 3 starts at 511: the viewport ending exactly there does not reach it.
        assert_eq!(rows_in_view(71.0, 440.0, PAD, 5.0, 34), 0..3);
        assert_eq!(rows_in_view(72.0, 440.0, PAD, 5.0, 34), 0..4);
    }

    #[test]
    fn layout_rows_in_view_follows_the_gap() {
        // Rows 25 + 160 apart: row 2 starts at 16 + 370 = 386.
        assert_eq!(rows_in_view(0.0, 440.0, PAD, 25.0, 34), 0..3);
        assert_eq!(rows_in_view(0.0, 370.0, PAD, 25.0, 34), 0..2);
    }

    #[test]
    fn layout_rows_in_view_clamps() {
        assert_eq!(
            rows_in_view(100_000.0, 440.0, PAD, 5.0, 34),
            34..34,
            "past the content ends at rows"
        );
        assert_eq!(rows_in_view(0.0, 440.0, PAD, 5.0, 0), 0..0);
        assert_eq!(
            rows_in_view(0.0, 10_000.0, PAD, 5.0, 34),
            0..34,
            "a tall viewport shows every row"
        );
        assert_eq!(
            rows_in_view(-50.0, 440.0, PAD, 5.0, 34),
            0..3,
            "a negative scroll top is the top"
        );
        for height in [0.0, -10.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                rows_in_view(0.0, height, PAD, 5.0, 34).is_empty(),
                "viewport {height} shows no row"
            );
        }
        assert!(rows_in_view(f32::NAN, 440.0, PAD, 5.0, 34).is_empty());
    }

    #[test]
    fn layout_cards_to_request_puts_the_visible_rows_first() {
        let positions = cards_to_request(20..23, 34, 3, 100, 1);
        let expected: Vec<usize> = (60..69).chain(69..72).chain(57..60).collect();
        assert_eq!(positions, expected);
        assert!(
            positions.contains(&64) && positions.contains(&71),
            "positions past the 64th are included"
        );
    }

    #[test]
    fn layout_cards_to_request_stops_at_the_last_card() {
        // 100 cards in 3 columns: the last row (33) holds the single card 99.
        let positions = cards_to_request(32..34, 34, 3, 100, 1);
        let expected: Vec<usize> = (96..100).chain(93..96).collect();
        assert_eq!(positions, expected);
        assert_eq!(positions.iter().max(), Some(&99));
    }

    #[test]
    fn layout_cards_to_request_at_the_top_has_no_row_above() {
        let positions = cards_to_request(0..3, 34, 3, 100, 1);
        let expected: Vec<usize> = (0..9).chain(9..12).collect();
        assert_eq!(positions, expected);
    }

    #[test]
    fn layout_cards_to_request_looks_ahead_nearest_row_first() {
        let positions = cards_to_request(10..11, 34, 3, 100, 2);
        let expected: Vec<usize> = (30..33)
            .chain(33..36)
            .chain(36..39)
            .chain(27..30)
            .chain(24..27)
            .collect();
        assert_eq!(positions, expected);
        assert_eq!(
            cards_to_request(10..11, 34, 3, 100, 0),
            vec![30, 31, 32],
            "no look-ahead, no extra rows"
        );
    }

    #[test]
    fn layout_cards_to_request_wants_nothing_without_rows_in_view() {
        assert!(cards_to_request(0..0, 34, 3, 100, 1).is_empty());
        assert!(cards_to_request(0..3, 0, 3, 0, 1).is_empty());
        assert_eq!(cards_to_request(0..1, 1, 0, 2, 1), vec![0], "zero columns count as one");
    }

    #[test]
    fn layout_columns_follow_the_legacy_formula() {
        assert_eq!(columns(900.0 - 32.0, 5.0, 0), 3, "3 columns at the 900 px minimum");
        assert_eq!(columns(2000.0, 5.0, 0), 7);
        assert_eq!(columns(100.0, 5.0, 0), 1, "never fewer than one");
        assert_eq!(columns(0.0, 5.0, 0), 1);
        assert_eq!(columns(-50.0, 5.0, 0), 1);
        assert_eq!(columns(f32::NAN, 5.0, 0), 1);
    }

    #[test]
    fn layout_columns_change_exactly_at_the_card_boundary() {
        for gap in GAPS {
            for n in 1..=8_usize {
                let exact = MIN_CARD_WIDTH * n as f32 + gap * (n - 1) as f32;
                assert_eq!(columns(exact, gap, 0), n, "n={n} gap={gap}");
                if n > 1 {
                    assert_eq!(columns(exact - 1.0, gap, 0), n - 1, "one pixel less, n={n} gap={gap}");
                }
            }
        }
    }

    #[test]
    fn layout_columns_absorb_float_noise_at_the_boundary() {
        // A width that a scale factor left a hair below the boundary still reaches it.
        assert_eq!(columns(1055.0 - 0.0005, 5.0, 0), 4);
        assert_eq!(columns(1055.0 - 0.5, 5.0, 0), 3);
    }

    #[test]
    fn layout_columns_are_capped_by_the_adaptive_layout_setting() {
        assert_eq!(columns(2000.0, 5.0, 2), 2);
        assert_eq!(columns(2000.0, 5.0, 100), 7, "a cap above the fit changes nothing");
        assert_eq!(columns(300.0, 5.0, 4), 1);
    }

    #[test]
    fn layout_card_width_is_capped_and_a_capped_row_is_narrower_than_the_content() {
        assert_eq!(card_width(1000.0, 5.0, 1), MAX_CARD_WIDTH);
        assert!(row_width(card_width(1000.0, 5.0, 1), 5.0, 1) < 1000.0);
        assert_eq!(card_width(868.0, 5.0, 3), (868.0 - 10.0) / 3.0);
    }

    #[test]
    fn layout_card_width_is_never_negative() {
        assert_eq!(card_width(-20.0, 5.0, 1), 0.0);
        assert_eq!(card_width(10.0, 25.0, 3), 0.0);
    }

    #[test]
    fn layout_a_row_never_exceeds_the_content_width() {
        for gap in GAPS {
            let mut width = 100.0_f32;
            while width < 3000.0 {
                for cap in [0, 1, 2, 3, 5] {
                    let n = columns(width, gap, cap);
                    let card = card_width(width, gap, n);
                    assert!(card <= MAX_CARD_WIDTH);
                    assert!(
                        row_width(card, gap, n) <= width.max(0.0) || card == 0.0,
                        "width={width} gap={gap} cap={cap} n={n} card={card}"
                    );
                }
                width += 37.3;
            }
        }
    }

    #[test]
    fn layout_page_count_rounds_up() {
        assert_eq!(page_count(0, 30), 0);
        assert_eq!(page_count(30, 30), 1);
        assert_eq!(page_count(31, 30), 2);
        assert_eq!(page_count(5, 0), 0);
    }

    #[test]
    fn layout_page_slice_is_the_themes_of_a_page() {
        assert_eq!(page_slice(31, 1, 30), 0..30);
        assert_eq!(page_slice(31, 2, 30), 30..31);
        assert_eq!(page_slice(0, 1, 30), 0..0);
    }

    #[test]
    fn layout_page_slice_survives_pages_out_of_range() {
        assert_eq!(page_slice(31, 0, 30), 0..30, "page 0 is the first page");
        assert_eq!(page_slice(31, 3, 30), 31..31, "past the last page is empty");
        assert_eq!(page_slice(31, usize::MAX, 30), 31..31);
        assert_eq!(page_slice(31, 1, 0), 0..0);
    }

    #[test]
    fn arrow_keys_name_four_directions() {
        assert_eq!(Arrow::from_key("left"), Some(Arrow::Left));
        assert_eq!(Arrow::from_key("down"), Some(Arrow::Down));
        assert_eq!(Arrow::from_key("enter"), None);
        assert_eq!(Arrow::from_key("Left"), None);
    }

    #[test]
    fn arrows_walk_the_cards_in_row_major_order() {
        // 7 cards in 3 columns: rows [0 1 2] [3 4 5] [6].
        assert_eq!(neighbor(1, 7, 3, Arrow::Right), Some(2));
        assert_eq!(
            neighbor(2, 7, 3, Arrow::Right),
            Some(3),
            "the next row continues the order"
        );
        assert_eq!(neighbor(6, 7, 3, Arrow::Right), None, "the last card has no next one");
        assert_eq!(neighbor(3, 7, 3, Arrow::Left), Some(2));
        assert_eq!(neighbor(0, 7, 3, Arrow::Left), None);
    }

    #[test]
    fn arrows_up_and_down_keep_the_column() {
        assert_eq!(neighbor(4, 7, 3, Arrow::Up), Some(1));
        assert_eq!(neighbor(1, 7, 3, Arrow::Up), None, "the first row has nothing above");
        assert_eq!(neighbor(0, 7, 3, Arrow::Down), Some(3));
        assert_eq!(neighbor(3, 7, 3, Arrow::Down), Some(6));
    }

    #[test]
    fn arrow_down_from_a_short_last_row_goes_to_the_last_card() {
        assert_eq!(neighbor(5, 7, 3, Arrow::Down), Some(6));
        assert_eq!(neighbor(6, 7, 3, Arrow::Down), None, "the last row has nothing below");
        assert_eq!(neighbor(2, 7, 3, Arrow::Down), Some(5));
    }

    #[test]
    fn arrows_cope_with_one_column_and_no_cards() {
        assert_eq!(neighbor(0, 3, 1, Arrow::Down), Some(1));
        assert_eq!(neighbor(2, 3, 1, Arrow::Up), Some(1));
        assert_eq!(neighbor(0, 0, 3, Arrow::Right), None);
        assert_eq!(neighbor(0, 1, 0, Arrow::Down), None, "zero columns count as one");
    }
}
